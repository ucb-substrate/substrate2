//! The global context.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use arcstr::ArcStr;
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
    Port, SchematicType,
};
use crate::layout::element::RawCell;
use crate::layout::error::{GdsExportError, LayoutError};
use crate::layout::gds::{GdsExporter, GdsImporter, ImportedGds};
use crate::layout::CellBuilder as LayoutCellBuilder;
use crate::layout::{Cell as LayoutCell, CellHandle as LayoutCellHandle};
use crate::layout::{Layout, LayoutContext};
use crate::pdk::layers::LayerContext;
use crate::pdk::layers::LayerId;
use crate::pdk::layers::Layers;
use crate::pdk::layers::{GdsLayerSpec, InstalledLayers};
use crate::pdk::Pdk;
use crate::schematic::conv::{ConvError, RawLib};
use crate::schematic::schema::{FromSchema, Schema};
use crate::schematic::{
    Cell as SchematicCell, CellCacheKey, CellHandle as SchematicCellHandle, CellId, CellMetadata,
    InstancePath, RawCellInnerBuilder, SchemaCellCacheValue, SchemaCellHandle, Schematic,
    SchematicContext,
};
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
#[derive(Clone)]
pub struct Context {
    pub(crate) inner: Arc<RwLock<ContextInner>>,
    installations: Arc<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
    /// Map from `PDK` to `InstalledLayers<PDK>`.
    layers: Arc<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
    /// The executor to which commands should be submitted.
    pub executor: Arc<dyn Executor>,
    /// A cache for storing the results of expensive computations.
    pub cache: Cache,
}

impl Default for Context {
    fn default() -> Self {
        let cfg = Config::default().expect("requires valid Substrate configuration");

        Self {
            inner: Default::default(),
            installations: Default::default(),
            layers: Default::default(),
            executor: Arc::new(LocalExecutor),
            cache: Cache::new(
                cfg.cache
                    .into_cache()
                    .expect("requires valid Substrate cache configuration"),
            ),
        }
    }
}

impl Context {
    /// Creates a new [`Context`].
    pub fn new() -> Self {
        Self::default()
    }
}

/// An item that can be installed in a context.
pub trait Installation: Any + Send + Sync {
    /// A post-installation hook for additional context modifications
    /// required by the installation.
    ///
    /// PDKs, for example, should use this hook to install their layer
    /// set and standard cell libraries.
    #[allow(unused_variables)]
    fn post_install(&self, ctx: &mut ContextBuilder) {}
}
/// A [`Context`] with an associated PDK `PDK`.
pub struct PdkContext<PDK: Pdk> {
    /// PDK configuration and general data.
    pub pdk: Arc<PDK>,
    /// The PDK layer set.
    pub layers: Arc<PDK::Layers>,
    layer_ctx: Arc<RwLock<LayerContext>>,
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
    installations: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
    /// Map from `PDK` to `InstalledLayers<PDK>`.
    layers: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
    executor: Arc<dyn Executor>,
    cache: Option<Cache>,
}

impl Default for ContextBuilder {
    fn default() -> Self {
        Self {
            installations: Default::default(),
            layers: Default::default(),
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
    pub fn executor<E: Executor>(&mut self, executor: E) -> &mut Self {
        self.executor = Arc::new(executor);
        self
    }

    /// Installs the given [`Installation`].
    ///
    /// Only one installation of any given type can exist. Overwrites
    /// conflicting installations of the same type.
    #[inline]
    pub fn install<I>(&mut self, installation: I) -> &mut Self
    where
        I: Installation,
    {
        let installation = Arc::new(installation);
        self.installations
            .insert(TypeId::of::<I>(), installation.clone());
        installation.post_install(self);
        self
    }

    /// Installs layers for a PDK.
    ///
    /// For use in [`Installation::post_install`] hooks for PDK types.
    pub fn install_pdk_layers<PDK: Pdk>(&mut self) -> &mut Self {
        let mut ctx = LayerContext::default();
        let layers = ctx.install_layers::<PDK::Layers>();
        self.layers.insert(
            TypeId::of::<PDK>(),
            Arc::new(InstalledLayers::<PDK> {
                layers,
                ctx: Arc::new(RwLock::new(ctx)),
            }),
        );
        self
    }

    /// Sets the desired cache configuration.
    pub fn cache(&mut self, cache: Cache) -> &mut Self {
        self.cache = Some(cache);
        self
    }

    /// Builds the context based on the configuration in this builder.
    pub fn build(&mut self) -> Context {
        let cfg = Config::default().expect("requires valid Substrate configuration");

        Context {
            inner: Arc::new(RwLock::new(ContextInner::new())),
            installations: Arc::new(self.installations.clone()),
            layers: Arc::new(self.layers.clone()),
            executor: self.executor.clone(),
            cache: self.cache.clone().unwrap_or_else(|| {
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
            layer_ctx: self.layer_ctx.clone(),
            ctx: self.ctx.clone(),
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct ContextInner {
    pub(crate) schematic: SchematicContext,
    layout: LayoutContext,
}

impl ContextInner {
    fn new() -> Self {
        Self::default()
    }
}

impl Context {
    /// Creates a builder for constructing a context.
    pub fn builder() -> ContextBuilder {
        Default::default()
    }

    /// Creates a [`PdkContext`] for the given installed PDK.
    ///
    /// The PDK must first be installed in the context.
    pub fn with_pdk<PDK: Pdk>(&self) -> PdkContext<PDK> {
        // Instantiate PDK layers.
        let pdk = self
            .installations
            .get(&TypeId::of::<PDK>())
            .expect("PDK must be installed")
            .clone()
            .downcast()
            .unwrap();

        let InstalledLayers { layers, ctx } = self
            .layers
            .get(&TypeId::of::<PDK>())
            .expect("PDK layer set must be installed")
            .downcast_ref::<InstalledLayers<PDK>>()
            .unwrap();

        PdkContext {
            pdk,
            layers: layers.clone(),
            layer_ctx: ctx.clone(),
            ctx: self.clone(),
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
        let block_clone = block.clone();
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
                    prepare_cell_builder(*next_id, context, key.block.as_ref());
                let io_data = Arc::new(io_data);
                (
                    CellMetadata::<B> {
                        id: *next_id,
                        io_data: io_data.clone(),
                    },
                    (*next_id, cell_builder, io_data),
                )
            },
            move |_key, (id, mut cell_builder, io_data)| {
                let res = B::schematic(block_clone.as_ref(), io_data.as_ref(), &mut cell_builder);
                res.map(|data| SchemaCellCacheValue {
                    raw: Arc::new(cell_builder.finish()),
                    cell: Arc::new(SchematicCell::new(id, io_data, block_clone, Arc::new(data))),
                })
            },
        );

        SchemaCellHandle {
            handle: handle.clone(),
            cell: SchematicCellHandle {
                id: metadata.id,
                block,
                io_data: metadata.io_data.clone(),
                cell: handle.map(|res| {
                    Ok(res?
                        .as_ref()
                        .map_err(|e| e.clone())
                        .map(|SchemaCellCacheValue { cell, .. }| cell.clone()))
                }),
            },
        }
    }

    fn generate_cross_schematic_inner<S1: Schema, S2: FromSchema<S1>, B: Schematic<S1>>(
        &self,
        block: Arc<B>,
    ) -> SchemaCellHandle<S2, B> {
        let handle = self.generate_schematic_inner(block);
        let mut inner = self.inner.write().unwrap();
        SchemaCellHandle {
            handle: inner.schematic.cell_cache.generate(
                ConvCacheKey::<B, S2, S1> {
                    block: handle.cell.block.clone(),
                    phantom: PhantomData,
                },
                move |_| {
                    let SchemaCellCacheValue { raw, cell } = handle
                        .handle
                        .try_get()
                        .unwrap()
                        .as_ref()
                        .map_err(|e| e.clone())?;
                    Ok(SchemaCellCacheValue {
                        raw: Arc::new((**raw).clone().convert_schema::<S2>()?),
                        cell: cell.clone(),
                    })
                },
            ),
            cell: handle.cell,
        }
    }

    /// Generates a schematic of a block in schema `S1` for use in schema `S2`.
    ///
    /// Can only generate a cross schematic with one layer of [`FromSchema`] indirection.
    pub fn generate_cross_schematic<S1: Schema, S2: FromSchema<S1>, B: Schematic<S1>>(
        &self,
        block: B,
    ) -> SchemaCellHandle<S2, B> {
        self.generate_cross_schematic_inner(Arc::new(block))
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
        let cell = self.generate_schematic(block);
        // TODO: Handle errors.
        let SchemaCellCacheValue { raw, .. } = cell.handle.unwrap_inner();
        raw.to_scir_lib()
    }

    /// Simulate the given testbench.
    ///
    /// The simulator must be installed in the context.
    pub fn simulate<S, T>(&self, block: T, work_dir: impl Into<PathBuf>) -> Result<T::Output>
    where
        S: Simulator,
        T: Testbench<S>,
    {
        let simulator = self.get_installation::<S>();
        let block = Arc::new(block);
        let cell = self.generate_schematic_inner::<<S as Simulator>::Schema, _>(block.clone());
        // TODO: Handle errors.
        let SchemaCellCacheValue { raw, cell } = cell.handle.unwrap_inner();
        let lib = raw.to_scir_lib()?;
        let ctx = SimulationContext {
            lib: Arc::new(lib),
            work_dir: work_dir.into(),
            ctx: self.clone(),
        };
        let controller = SimController {
            tb: cell.clone(),
            simulator,
            ctx,
        };

        // TODO caching
        Ok(block.run(controller))
    }

    /// Gets an installation from the context installation map.
    pub fn get_installation<I: Installation>(&self) -> Arc<I> {
        let arc = self.installations.get(&TypeId::of::<I>()).unwrap().clone();
        arc.downcast().unwrap()
    }
}

impl<PDK: Pdk> PdkContext<PDK> {
    /// Creates a new global context.
    #[inline]
    pub fn new(pdk: PDK) -> Self {
        ContextBuilder::new().install(pdk).build().with_pdk()
    }

    /// Generates a layout for `block` in the background.
    ///
    /// Returns a handle to the cell being generated.
    pub fn generate_layout<T: Layout<PDK>>(&self, block: T) -> LayoutCellHandle<T> {
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
                let data = block.layout(&mut io_builder, &mut cell_builder);

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
    pub fn write_layout<T: Layout<PDK>>(&self, block: T, path: impl AsRef<Path>) -> Result<()> {
        let handle = self.generate_layout(block);
        let cell = handle.try_cell()?;

        let layer_ctx = self.layer_ctx.read().unwrap();
        GdsExporter::new(cell.raw.clone(), &layer_ctx)
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
        let ContextInner { ref mut layout, .. } = *inner;
        let mut layer_ctx = self.layer_ctx.write().unwrap();
        let imported =
            GdsImporter::new(&lib, layout, &mut layer_ctx, PDK::LAYOUT_DB_UNITS).import()?;
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
        let ContextInner { ref mut layout, .. } = *inner;
        let mut layer_ctx = self.layer_ctx.write().unwrap();
        let imported = GdsImporter::new(&lib, layout, &mut layer_ctx, PDK::LAYOUT_DB_UNITS)
            .import_cell(cell)?;
        Ok(imported)
    }

    /// Installs a new layer set in the context.
    ///
    /// Allows for accessing GDS layers or other extra layers that are not present in the PDK.
    pub fn install_layers<L: Layers>(&self) -> Arc<L> {
        let mut layer_ctx = self.layer_ctx.write().unwrap();
        layer_ctx.install_layers::<L>()
    }

    /// Gets a layer by its GDS layer spec.
    ///
    /// Should generally not be used except for situations involving GDS import, where
    /// layers may be imported at runtime.
    pub fn get_gds_layer(&self, spec: GdsLayerSpec) -> Option<LayerId> {
        let layer_ctx = self.layer_ctx.read().unwrap();
        layer_ctx.get_gds_layer(spec)
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
