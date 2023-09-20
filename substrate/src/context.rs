//! The global context.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use arcstr::ArcStr;
use config::Config;
use examples::get_snippets;
use indexmap::IndexMap;
use scir::schema::Schema;
use substrate::schematic::{CellBuilder, CellBuilderInner, PdkCellBuilder};
use tracing::{span, Level};

use crate::block::Block;
use crate::cache::Cache;
use crate::diagnostics::SourceInfo;
use crate::error::Result;
use crate::execute::{Executor, LocalExecutor};
use crate::io::{
    Flatten, Flipped, HasNameTree, LayoutBundleBuilder, LayoutType, NodeContext, NodePriority,
    Port, SchematicType, TestbenchIo,
};
use crate::layout::element::RawCell;
use crate::layout::error::{GdsExportError, LayoutError};
use crate::layout::gds::{GdsExporter, GdsImporter, ImportedGds};
use crate::layout::CellBuilder as LayoutCellBuilder;
use crate::layout::{Cell as LayoutCell, CellHandle as LayoutCellHandle};
use crate::layout::{LayoutContext, LayoutImplemented};
use crate::pdk::layers::GdsLayerSpec;
use crate::pdk::layers::LayerContext;
use crate::pdk::layers::LayerId;
use crate::pdk::layers::Layers;
use crate::pdk::{Pdk, PdkSchematic};
use crate::schematic::conv::RawLib;
use crate::schematic::{
    Cell as SchematicCell, CellBuilder as SchematicCellBuilder, CellBuilderContents, CellCacheKey,
    CellHandle as SchematicCellHandle, CellId, CellInner, InstanceId, InstancePath,
    PdkCellCacheKey, PreGenerateCellData, Primitive, RawCell as SchematicRawCell, Schematic,
    SchematicContext,
};
use crate::sealed::Token;
use crate::simulation::{SimController, SimulationContext, Simulator, Testbench};

/// The global context.
///
/// Stores configuration such as the PDK and tool plugins to use during generation.
///
/// Cheaply clonable.
///
/// # Examples
///
#[doc = get_snippets!("core", "generate")]
pub struct Context<PDK: Pdk> {
    /// PDK configuration and general data.
    pub pdk: Arc<PDK>,
    /// The PDK layer set.
    pub layers: Arc<PDK::Layers>,
    inner: Arc<RwLock<ContextInner>>,
    simulators: Arc<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
    executor: Arc<dyn Executor>,
    /// A cache for storing the results of expensive computations.
    pub cache: Cache,
}

/// Builder for creating a Substrate [`Context`].
pub struct ContextBuilder<PDK: Pdk> {
    pdk: Option<PDK>,
    simulators: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
    executor: Arc<dyn Executor>,
    cache: Option<Cache>,
}

impl<PDK: Pdk> Default for ContextBuilder<PDK> {
    fn default() -> Self {
        Self::new()
    }
}
impl<PDK: Pdk> ContextBuilder<PDK> {
    /// Creates a new, uninitialized builder.
    #[inline]
    pub fn new() -> Self {
        Self {
            pdk: None,
            simulators: Default::default(),
            executor: Arc::new(LocalExecutor),
            cache: None,
        }
    }

    /// Sets the PDK.
    #[inline]
    pub fn pdk(mut self, pdk: PDK) -> Self {
        self.pdk = Some(pdk);
        self
    }

    /// Sets the executor.
    pub fn executor<E: Executor>(mut self, executor: E) -> Self {
        self.executor = Arc::new(executor);
        self
    }

    /// Installs the given simulator.
    ///
    /// Only one simulator of any given type can exist.
    #[inline]
    pub fn with_simulator<S>(mut self, simulator: S) -> Self
    where
        S: Simulator,
    {
        self.simulators
            .insert(TypeId::of::<S>(), Arc::new(simulator));
        self
    }

    /// Sets the desired cache configuration.
    pub fn cache(mut self, cache: Cache) -> Self {
        self.cache = Some(cache);
        self
    }

    /// Builds the context based on the configuration in this builder.
    pub fn build(self) -> Context<PDK> {
        // Instantiate PDK layers.
        let mut layer_ctx = LayerContext::new();
        let layers = layer_ctx.install_layers::<PDK::Layers>();

        let cfg = Config::default().expect("requires valid Substrate configuration");

        Context {
            pdk: Arc::new(self.pdk.unwrap()),
            layers,
            inner: Arc::new(RwLock::new(ContextInner::new(layer_ctx))),
            simulators: Arc::new(self.simulators),
            executor: self.executor,
            cache: self.cache.unwrap_or_else(|| {
                Cache::new(
                    cfg.cache
                        .into_cache()
                        .expect("requires valid Substrate cache configuration"),
                )
            }),
        }
    }
}

impl<PDK: Pdk> Clone for Context<PDK> {
    fn clone(&self) -> Self {
        Self {
            pdk: self.pdk.clone(),
            layers: self.layers.clone(),
            inner: self.inner.clone(),
            simulators: self.simulators.clone(),
            executor: self.executor.clone(),
            cache: self.cache.clone(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct ContextInner {
    layers: LayerContext,
    schematic: SchematicContext,
    layout: LayoutContext,
}

impl<PDK: Pdk> Context<PDK> {
    /// Creates a builder for constructing a context.
    pub fn builder() -> ContextBuilder<PDK> {
        Default::default()
    }

    /// Creates a new global context.
    #[inline]
    pub fn new(pdk: PDK) -> Self {
        ContextBuilder::new().pdk(pdk).build()
    }

    /// Generates a layout for `block` in the background.
    ///
    /// Returns a handle to the cell being generated.
    pub fn generate_layout<T: LayoutImplemented<PDK>>(&self, block: T) -> LayoutCellHandle<T> {
        let context_clone = self.clone();
        let mut inner_mut = self.inner.write().unwrap();
        let id = inner_mut.layout.get_id();
        let block = Arc::new(block);

        let span = span!(
            Level::INFO,
            "generating layout",
            block = %block.name(),
        )
        .or_current();

        LayoutCellHandle {
            cell: inner_mut.layout.cell_cache.generate(block, move |block| {
                let mut io_builder = block.io().builder();
                let mut cell_builder = LayoutCellBuilder::new(context_clone);
                let _guard = span.enter();
                let data = block.layout_impl(&mut io_builder, &mut cell_builder, Token);

                let io = io_builder.build()?;
                let ports = IndexMap::from_iter(
                    block
                        .io()
                        .flat_names(None)
                        .into_iter()
                        .zip(io.flatten_vec().into_iter()),
                );
                data.map(|data| {
                    LayoutCell::new(
                        block.clone(),
                        data,
                        Arc::new(io),
                        Arc::new(cell_builder.finish(id, block.name()).with_ports(ports)),
                    )
                })
            }),
        }
    }

    /// Writes a layout to a GDS file.
    pub fn write_layout<T: LayoutImplemented<PDK>>(
        &self,
        block: T,
        path: impl AsRef<Path>,
    ) -> Result<()> {
        let handle = self.generate_layout(block);
        let cell = handle.try_cell()?;

        let inner = self.inner.read().unwrap();
        GdsExporter::new(cell.raw.clone(), &inner.layers)
            .export()
            .map_err(LayoutError::from)?
            .save(path)
            .map_err(GdsExportError::from)
            .map_err(LayoutError::from)?;
        Ok(())
    }

    /// Reads a layout from a GDS file.
    pub fn read_gds(&self, path: impl AsRef<Path>) -> Result<ImportedGds> {
        let lib = gds::GdsLibrary::load(path)?;
        let mut inner = self.inner.write().unwrap();
        let ContextInner {
            ref mut layers,
            ref mut layout,
            ..
        } = *inner;
        let imported = GdsImporter::new(&lib, layout, layers, PDK::LAYOUT_DB_UNITS).import()?;
        Ok(imported)
    }

    /// Reads the layout of a single cell from a GDS file.
    pub fn read_gds_cell(
        &self,
        path: impl AsRef<Path>,
        cell: impl Into<ArcStr>,
    ) -> Result<Arc<RawCell>> {
        let lib = gds::GdsLibrary::load(path)?;
        let mut inner = self.inner.write().unwrap();
        let ContextInner {
            ref mut layers,
            ref mut layout,
            ..
        } = *inner;
        let imported =
            GdsImporter::new(&lib, layout, layers, PDK::LAYOUT_DB_UNITS).import_cell(cell)?;
        Ok(imported)
    }

    fn generate_schematic_inner<S: Schema, T: Schematic<PDK, S>>(
        &self,
        block: Arc<T>,
    ) -> SchematicCellHandle<T> {
        let mut inner = self.inner.write().unwrap();
        let context = self.clone();
        let key = CellCacheKey {
            block: block.clone(),
            phantom: PhantomData::<(PDK, S)>,
        };
        let block_clone = block.clone();
        let id = inner.schematic.get_id();
        let PreGenerateCellData {
            id,
            cell_builder,
            io_data,
        } = inner
            .schematic
            .pre_generate_data
            .get_or_insert(key.clone(), move || {
                let (cell_builder, io_data) =
                    prepare_cell_builder::<_, S, _>(id, context.clone(), block_clone.as_ref());
                PreGenerateCellData::<T, PDK> {
                    id,
                    cell_builder: cell_builder.0.into(),
                    io_data: Arc::new(io_data),
                }
            });
        let context = self.clone();
        SchematicCellHandle {
            id,
            block: block.clone(),
            io_data: Clone::clone(&io_data),
            cell: inner.schematic.cell_cache.generate(
                key,
                move |CellCacheKey { block, .. }| {
                    let mut cell_builder = CellBuilder(cell_builder.into());
                    let data = block.schematic(&io_data, &mut cell_builder);
                    data.map(|data| {
                        context
                            .inner
                            .write()
                            .unwrap()
                            .schematic
                            .raw_cells
                            .insert(id, Arc::new(cell_builder.0.finish()));
                        SchematicCell::new(id, io_data, block.clone(), data)
                    })
                },
            ),
        };
        todo!();
    }

    /// Generates a schematic for `block` in the background.
    ///
    /// Returns a handle to the cell being generated.
    pub fn generate_schematic<S: Schema, T: Schematic<PDK, S>>(
        &self,
        block: T,
    ) -> SchematicCellHandle<T> {
        let block = Arc::new(block);
        self.generate_schematic_inner(block)
    }

    fn generate_pdk_schematic_inner<T: PdkSchematic<PDK>>(
        &self,
        block: Arc<T>,
    ) -> SchematicCellHandle<T> {
        let mut inner = self.inner.write().unwrap();
        let key = PdkCellCacheKey {
            block: block.clone(),
            phantom: PhantomData::<PDK>,
        };
        let context = self.clone();
        let block_clone = block.clone();
        let id = inner.schematic.get_id();
        let PreGenerateCellData {
            id,
            cell_builder,
            io_data,
        } = inner
            .schematic
            .pre_generate_data
            .get_or_insert(key.clone(), move || {
                let (cell_builder, io_data) =
                    prepare_pdk_cell_builder(id, context.clone(), block_clone.as_ref());
                PreGenerateCellData::<T, PDK> {
                    id,
                    cell_builder: cell_builder.0.into(),
                    io_data: Arc::new(io_data),
                }
            });
        let context = self.clone();
        SchematicCellHandle {
            id,
            block: block.clone(),
            io_data: io_data.clone(),
            cell: inner.schematic.cell_cache.generate(
                key,
                move |PdkCellCacheKey { block, .. }| {
                    let mut cell_builder = PdkCellBuilder(cell_builder.into());
                    let data = block.schematic(&io_data, &mut cell_builder);
                    data.map(|data| {
                        context
                            .inner
                            .write()
                            .unwrap()
                            .schematic
                            .raw_cells
                            .insert(id, Arc::new(cell_builder.0.finish()));
                        SchematicCell::new(id, io_data, block.clone(), data)
                    })
                },
            ),
        };
        todo!();
    }

    /// Generates a PDK schematic for `block` in the background.
    ///
    /// Returns a handle to the cell being generated.
    pub fn generate_pdk_schematic<T: PdkSchematic<PDK>>(&self, block: T) -> SchematicCellHandle<T> {
        let block = Arc::new(block);
        self.generate_pdk_schematic_inner(block)
    }

    /// Export the given block and all sub-blocks as a SCIR library.
    ///
    /// Returns a SCIR library and metadata for converting between SCIR and Substrate formats.
    pub fn export_pdk_scir<T: PdkSchematic<PDK>>(
        &self,
        block: T,
    ) -> Result<RawLib<PDK::Schema>, scir::Issues> {
        let cell = self.generate_pdk_schematic(block);
        let cell = cell.cell();
        self.get_raw_cell(cell.id).unwrap().to_scir_lib()
    }

    /// Export the given block and all sub-blocks as a SCIR library.
    ///
    /// Returns a SCIR library and metadata for converting between SCIR and Substrate formats.
    pub fn export_scir<S: Schema, T: Schematic<PDK, S>>(
        &self,
        block: T,
    ) -> Result<RawLib<S::Primitive>, scir::Issues> {
        let cell = self.generate_schematic(block);
        let cell = cell.cell();
        self.get_raw_cell(cell.id).unwrap().to_scir_lib()
    }

    /// Installs a new layer set in the context.
    ///
    /// Allows for accessing GDS layers or other extra layers that are not present in the PDK.
    pub fn install_layers<L: Layers>(&self) -> Arc<L> {
        let mut inner = self.inner.write().unwrap();
        inner.layers.install_layers::<L>()
    }

    /// Gets a layer by its GDS layer spec.
    ///
    /// Should generally not be used except for situations involving GDS import, where
    /// layers may be imported at runtime.
    pub fn get_gds_layer(&self, spec: GdsLayerSpec) -> Option<LayerId> {
        let inner = self.inner.read().unwrap();
        inner.layers.get_gds_layer(spec)
    }

    /// Simulate the given testbench.
    pub fn simulate<S, T>(&self, block: T, work_dir: impl Into<PathBuf>) -> Result<T::Output>
    where
        S: Simulator,
        T: Testbench<PDK, S>,
    {
        let simulator = self.get_simulator::<S>();
        let block = Arc::new(block);
        let cell = self.generate_schematic_inner(block.clone());
        // TODO: Handle errors.
        let cell = cell.cell();
        let lib = self.export_testbench_scir_for_cell(cell)?;
        let ctx = SimulationContext {
            lib: Arc::new(lib),
            work_dir: work_dir.into(),
            executor: self.executor.clone(),
            cache: self.cache.clone(),
        };
        let controller = SimController {
            pdk: self.pdk.clone(),
            tb: (*cell).clone(),
            simulator,
            ctx,
        };

        // TODO caching
        Ok(block.run(controller))
    }

    fn get_simulator<S: Simulator>(&self) -> Arc<S> {
        let arc = self.simulators.get(&TypeId::of::<S>()).unwrap().clone();
        arc.downcast().unwrap()
    }

    pub(crate) fn get_raw_cell<P: Primitive>(
        &self,
        id: CellId,
    ) -> Option<Arc<SchematicRawCell<P>>> {
        self.inner
            .read()
            .unwrap()
            .schematic
            .raw_cells
            .get(&id)
            .and_then(|cell| cell.clone().downcast().ok())
    }
}

impl ContextInner {
    #[allow(dead_code)]
    pub(crate) fn new(layers: LayerContext) -> Self {
        Self {
            layers,
            schematic: Default::default(),
            layout: Default::default(),
        }
    }
}

fn prepare_cell_builder<PDK: Pdk, S: Schema, T: Block>(
    id: crate::schematic::CellId,
    context: Context<PDK>,
    block: &T,
) -> (
    SchematicCellBuilder<PDK, S>,
    <<T as Block>::Io as SchematicType>::Bundle,
) {
    let mut node_ctx = NodeContext::new();
    // outward-facing IO (to other enclosing blocks)
    let io_outward = block.io();
    // inward-facing IO (this block's IO ports as viewed by the interior of the
    // block)
    let io_internal = Flipped(io_outward.clone());
    // FIXME: the cell's IO should not be attributed to this call site
    let (nodes, io_data) =
        node_ctx.instantiate_directed(&io_internal, NodePriority::Io, SourceInfo::from_caller());
    let cell_name = block.name();

    let names = io_outward.flat_names(None);
    let outward_dirs = io_outward.flatten_vec();
    assert_eq!(nodes.len(), names.len());
    assert_eq!(nodes.len(), outward_dirs.len());

    let ports = nodes
        .iter()
        .copied()
        .zip(outward_dirs)
        .map(|(node, direction)| Port::new(node, direction))
        .collect();

    let node_names = HashMap::from_iter(nodes.into_iter().zip(names));
    let cell_builder = SchematicCellBuilder::new(CellBuilderInner {
        id,
        root: InstancePath::new(id),
        cell_name,
        ctx: context,
        node_ctx,
        node_names,
        ports,
        contents: CellBuilderContents::Cell(CellInner {
            next_instance_id: InstanceId(0),
            instances: Vec::new(),
        }),
        flatten: false,
    });

    (cell_builder, io_data)
}

fn prepare_pdk_cell_builder<PDK: Pdk, T: Block>(
    id: crate::schematic::CellId,
    context: Context<PDK>,
    block: &T,
) -> (
    PdkCellBuilder<PDK>,
    <<T as Block>::Io as SchematicType>::Bundle,
) {
    let mut node_ctx = NodeContext::new();
    // outward-facing IO (to other enclosing blocks)
    let io_outward = block.io();
    // inward-facing IO (this block's IO ports as viewed by the interior of the
    // block)
    let io_internal = Flipped(io_outward.clone());
    // FIXME: the cell's IO should not be attributed to this call site
    let (nodes, io_data) =
        node_ctx.instantiate_directed(&io_internal, NodePriority::Io, SourceInfo::from_caller());
    let cell_name = block.name();

    let names = io_outward.flat_names(None);
    let outward_dirs = io_outward.flatten_vec();
    assert_eq!(nodes.len(), names.len());
    assert_eq!(nodes.len(), outward_dirs.len());

    let ports = nodes
        .iter()
        .copied()
        .zip(outward_dirs)
        .map(|(node, direction)| Port::new(node, direction))
        .collect();

    let node_names = HashMap::from_iter(nodes.into_iter().zip(names));
    let cell_builder = PdkCellBuilder(CellBuilderInner {
        id,
        root: InstancePath::new(id),
        cell_name,
        ctx: context,
        node_ctx,
        node_names,
        ports,
        contents: CellBuilderContents::Cell(CellInner {
            next_instance_id: InstanceId(0),
            instances: Vec::new(),
        }),
        flatten: false,
    });

    (cell_builder, io_data)
}
