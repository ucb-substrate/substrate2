//! The global context.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use arcstr::ArcStr;
use cache::{CacheHandle, MappedCacheHandle, SecondaryCacheHandle};
use config::Config;
use examples::get_snippets;
use indexmap::IndexMap;
use substrate::schematic::{CellBuilder, ConvCacheKey, RawCellContentsBuilder};
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
use crate::pdk::Pdk;
use crate::schematic::conv::{ConvError, RawLib, ScirLibExportContext};
use crate::schematic::schema::{Schema, ToSchema};
use crate::schematic::{
    Cell as SchematicCell, CellBuilder as SchematicCellBuilder, CellCacheKey,
    CellHandle as SchematicCellHandle, CellId, CellMetadata, ExportsNestedData, InstanceId,
    InstancePath, RawCell as SchematicRawCell, RawCellContents, RawCellInner, RawCellInnerBuilder,
    SchemaCellHandle, Schematic, SchematicContext,
};
use crate::sealed;
use crate::sealed::Token;
use crate::simulation::{SimController, SimulationContext, Simulator, Testbench};

#[derive(Clone)]
pub struct Context {
    pub(crate) inner: Arc<RwLock<ContextInner>>,
    simulators: Arc<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
    executor: Arc<dyn Executor>,
    /// A cache for storing the results of expensive computations.
    pub cache: Cache,
}

/// The global context.
///
/// Stores configuration such as the PDK and tool plugins to use during generation.
///
/// Cheaply clonable.
///
/// # Examples
///
#[doc = get_snippets!("core", "generate")]
pub struct PdkContext<PDK: Pdk> {
    /// PDK configuration and general data.
    pub pdk: Arc<PDK>,
    /// The PDK layer set.
    pub layers: Arc<PDK::Layers>,
    ctx: Context,
}

impl<PDK: Pdk> Deref for PdkContext<PDK> {
    type Target = Context;

    fn deref(&self) -> &Self::Target {
        &self.ctx
    }
}

/// Builder for creating a Substrate [`Context`].
pub struct ContextBuilder {
    simulators: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
    executor: Arc<dyn Executor>,
    cache: Option<Cache>,
}

impl Default for ContextBuilder {
    fn default() -> Self {
        Self {
            simulators: Default::default(),
            executor: Arc::new(LocalExecutor),
            cache: None,
        }
    }
}
impl ContextBuilder {
    /// Creates a new, uninitialized builder.
    #[inline]
    pub fn new() -> Self {
        Self::default()
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
    pub fn build(self) -> Context {
        let cfg = Config::default().expect("requires valid Substrate configuration");

        Context {
            inner: Arc::new(RwLock::new(ContextInner::new(LayerContext::new()))),
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

impl<PDK: Pdk> Clone for PdkContext<PDK> {
    fn clone(&self) -> Self {
        Self {
            pdk: self.pdk.clone(),
            layers: self.layers.clone(),
            ctx: self.ctx.clone(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct ContextInner {
    pub(crate) schematic: SchematicContext,
    layout: LayoutContext,
    layers: LayerContext,
}

impl Context {
    /// Creates a builder for constructing a context.
    pub fn builder() -> ContextBuilder {
        Default::default()
    }

    pub fn with_pdk<PDK: Pdk>(self, pdk: PDK) -> PdkContext<PDK> {
        // Instantiate PDK layers.
        let mut inner = self.inner.write().unwrap();
        let layers = inner.layers.install_layers::<PDK::Layers>();

        drop(inner);

        PdkContext {
            pdk: Arc::new(pdk),
            layers,
            ctx: self,
        }
    }

    /// Steps to create schematic:
    /// - Check if block has already been generated
    /// - If not:
    ///     - Create io_data, returning it immediately
    ///     - Use io_data and the cell_builder associated with it to
    ///       generate cell in background
    /// - If yes:
    ///     - Retrieve created cell ID, io_data, and handle to cell
    ///     - being generated and return immediately
    pub(crate) fn generate_schematic_inner<S: Schema, B: Schematic<S>>(
        &self,
        block: Arc<B>,
    ) -> SchemaCellHandle<S, B> {
        let key = CellCacheKey {
            block: block.clone(),
            phantom: PhantomData::<S>,
        };
        let block_clone1 = block.clone();
        let block_clone2 = block.clone();
        let mut inner = self.inner.write().unwrap();
        let context = self.clone();
        let SchematicContext {
            next_id,
            cell_cache,
            ..
        } = &mut inner.schematic;
        let (metadata, handle) = cell_cache.generate_partial_blocking(
            key,
            |key| {
                next_id.increment();
                let (cell_builder, io_data) =
                    prepare_cell_builder(*next_id, context, block_clone1.as_ref());
                let io_data = Arc::new(io_data);
                (
                    CellMetadata::<B> {
                        id: *next_id,
                        io_data: io_data.clone(),
                    },
                    (*next_id, cell_builder, io_data),
                )
            },
            move |key, (id, mut cell_builder, io_data)| {
                let res = B::schematic(block_clone2.as_ref(), io_data.as_ref(), &mut cell_builder);
                res.map(|data| {
                    (
                        Arc::new(cell_builder.finish()),
                        Arc::new(SchematicCell::new(
                            id,
                            io_data,
                            block_clone2,
                            Arc::new(data),
                        )),
                    )
                })
            },
        );

        SchemaCellHandle {
            handle: handle.clone(),
            cell: SchematicCellHandle {
                id: metadata.id,
                block,
                io_data: metadata.io_data.clone(),
                cell: MappedCacheHandle::new(handle, |res| {
                    Ok(res?
                        .as_ref()
                        .map_err(|e| e.clone())
                        .map(|(_, cell)| cell.clone()))
                }),
            },
        }
    }

    // Can only generate with one layer of indirection (cannot arbitrarily convert).
    pub fn generate_cross_schematic<S1: ToSchema<S2>, S2: Schema, B: Schematic<S1>>(
        &self,
        block: B,
    ) -> SchemaCellHandle<S2, B> {
        let handle = self.generate_schematic(block);
        let mut inner = self.inner.write().unwrap();
        SchemaCellHandle {
            handle: inner.schematic.cell_cache.generate(
                ConvCacheKey::<B, S2, S1> {
                    block: handle.cell.block.clone(),
                    phantom: PhantomData,
                },
                move |_| {
                    let (raw, cell) = handle
                        .handle
                        .try_get()
                        .unwrap()
                        .as_ref()
                        .map_err(|e| e.clone())?;
                    Ok((
                        Arc::new((**raw).clone().convert_schema::<S2>()?),
                        cell.clone(),
                    ))
                },
            ),
            cell: handle.cell,
        }
    }

    /// Generates a schematic for `block` in the background.
    ///
    /// Returns a handle to the cell being generated.
    pub fn generate_schematic<S: Schema, T: Schematic<S>>(
        &self,
        block: T,
    ) -> SchemaCellHandle<S, T> {
        let block = Arc::new(block);
        self.generate_schematic_inner(block)
    }

    /// Export the given block and all sub-blocks as a SCIR library.
    ///
    /// Returns a SCIR library and metadata for converting between SCIR and Substrate formats.
    pub fn export_scir<S: Schema, T: Schematic<S>>(
        &self,
        block: T,
    ) -> Result<RawLib<S>, ConvError> {
        todo!()
    }
}

impl<PDK: Pdk> PdkContext<PDK> {
    /// Creates a new global context.
    #[inline]
    pub fn new(pdk: PDK) -> Self {
        ContextBuilder::new().build().with_pdk(pdk)
    }

    /// Generates a layout for `block` in the background.
    ///
    /// Returns a handle to the cell being generated.
    pub fn generate_layout<T: LayoutImplemented<PDK>>(&self, block: T) -> LayoutCellHandle<T> {
        let context_clone = self.clone();
        let mut inner_mut = self.ctx.inner.write().unwrap();
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

        let inner = self.ctx.inner.read().unwrap();
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
        let mut inner = self.ctx.inner.write().unwrap();
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
        let mut inner = self.ctx.inner.write().unwrap();
        let ContextInner {
            ref mut layers,
            ref mut layout,
            ..
        } = *inner;
        let imported =
            GdsImporter::new(&lib, layout, layers, PDK::LAYOUT_DB_UNITS).import_cell(cell)?;
        Ok(imported)
    }

    /// Installs a new layer set in the context.
    ///
    /// Allows for accessing GDS layers or other extra layers that are not present in the PDK.
    pub fn install_layers<L: Layers>(&self) -> Arc<L> {
        let mut inner = self.ctx.inner.write().unwrap();
        inner.layers.install_layers::<L>()
    }

    /// Gets a layer by its GDS layer spec.
    ///
    /// Should generally not be used except for situations involving GDS import, where
    /// layers may be imported at runtime.
    pub fn get_gds_layer(&self, spec: GdsLayerSpec) -> Option<LayerId> {
        let inner = self.ctx.inner.read().unwrap();
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
        let cell = self.ctx.generate_schematic_inner(block.clone());
        // TODO: Handle errors.
        let cell = cell.cell.cell();
        todo!();
        // let lib = self.export_testbench_scir_for_cell(cell)?;
        // let ctx = SimulationContext {
        //     lib: Arc::new(lib),
        //     work_dir: work_dir.into(),
        //     executor: self.executor.clone(),
        //     cache: self.cache.clone(),
        // };
        // let controller = SimController {
        //     pdk: self.pdk.clone(),
        //     tb: (*cell).clone(),
        //     simulator,
        //     ctx,
        // };

        // // TODO caching
        // Ok(block.run(controller))
    }

    fn get_simulator<S: Simulator>(&self) -> Arc<S> {
        let arc = self.ctx.simulators.get(&TypeId::of::<S>()).unwrap().clone();
        arc.downcast().unwrap()
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

fn prepare_cell_builder<S: Schema, T: Block>(
    id: CellId,
    context: Context,
    block: &T,
) -> (CellBuilder<S>, <<T as Block>::Io as SchematicType>::Bundle) {
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

    (
        CellBuilder {
            id,
            root: InstancePath::new(id),
            cell_name,
            ctx: context,
            node_ctx,
            node_names,
            ports,
            flatten: false,
            contents: RawCellContentsBuilder::Cell(RawCellInnerBuilder::default()),
        },
        io_data,
    )
}
