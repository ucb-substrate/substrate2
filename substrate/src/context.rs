//! The global context.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use config::Config;
use indexmap::IndexMap;
use substrate::schematic::{CellBuilder, ConvCacheKey, RawCellContentsBuilder};
use tracing::{span, Level};

use crate::block::Block;
use crate::cache::Cache;
use crate::diagnostics::SourceInfo;
use crate::error::Result;
use crate::execute::{Executor, LocalExecutor};
use crate::layout::conv::export_multi_top_layir_lib;
use crate::layout::element::{NamedPorts, RawCell};
use crate::layout::error::LayoutError;
use crate::layout::CellBuilder as LayoutCellBuilder;
use crate::layout::{Cell as LayoutCell, CellHandle as LayoutCellHandle};
use crate::layout::{Layout, LayoutContext};
use crate::schematic::conv::{export_multi_top_scir_lib, ConvError, RawLib};
use crate::schematic::schema::{FromSchema, Schema};
use crate::schematic::{
    Cell as SchematicCell, CellCacheKey, CellHandle as SchematicCellHandle, CellId, CellMetadata,
    RawCellInnerBuilder, SchemaCellCacheValue, SchemaCellHandle, Schematic, SchematicContext,
};
use crate::simulation::{SimController, SimulationContext, Simulator, Testbench};
use crate::types::layout::PortGeometryBuilder;
use crate::types::schematic::{IoNodeBundle, NodeContext, NodePriority, Port};
use crate::types::{FlatLen, Flatten, Flipped, HasBundleKind, HasNameTree, NameBuf};

/// The global context.
///
/// Stores configuration such as the PDK and tool plugins to use during generation.
///
/// Cheaply clonable.
#[derive(Clone)]
pub struct Context {
    pub(crate) inner: Arc<RwLock<ContextInner>>,
    installations: Arc<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
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

/// A private item that can be installed in a context after it is built.
pub trait PrivateInstallation: Any + Send + Sync {}

/// Builder for creating a Substrate [`Context`].
pub struct ContextBuilder {
    installations: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
    executor: Arc<dyn Executor>,
    cache: Option<Cache>,
}

impl Default for ContextBuilder {
    fn default() -> Self {
        Self {
            installations: Default::default(),
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

    /// Gets an installation from the context installation map.
    pub fn get_installation<I: Installation>(&self) -> Option<Arc<I>> {
        retrieve_installation(&self.installations)
    }
}

#[derive(Debug, Default)]
pub(crate) struct ContextInner {
    pub(crate) schematic: SchematicContext,
    layout: LayoutContext,
    private_installations: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
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

    /// Allocates a new [`CellId`].
    fn alloc_cell_id(&self) -> CellId {
        let mut inner = self.inner.write().unwrap();
        let SchematicContext { next_id, .. } = &mut inner.schematic;
        next_id.increment();
        *next_id
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
    pub(crate) fn generate_schematic_inner<B: Schematic>(
        &self,
        block: Arc<B>,
    ) -> SchemaCellHandle<B::Schema, B> {
        let key = CellCacheKey {
            block: block.clone(),
            phantom: PhantomData::<B::Schema>,
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
                    prepare_cell_builder(Some(*next_id), context, key.block.as_ref());
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
                let fatal = cell_builder.fatal_error;
                let raw = Arc::new(cell_builder.finish());
                (!fatal)
                    .then_some(())
                    .ok_or(crate::error::Error::CellBuildFatal)
                    .and(res.map(|data| SchemaCellCacheValue {
                        raw: raw.clone(),
                        cell: Arc::new(SchematicCell::new(
                            id,
                            io_data,
                            block_clone,
                            raw,
                            Arc::new(data),
                        )),
                    }))
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

    fn generate_cross_schematic_inner<B: Schematic, S2: FromSchema<B::Schema> + ?Sized>(
        &self,
        block: Arc<B>,
    ) -> SchemaCellHandle<S2, B> {
        let handle = self.generate_schematic_inner(block);
        let mut inner = self.inner.write().unwrap();
        SchemaCellHandle {
            handle: inner.schematic.cell_cache.generate(
                ConvCacheKey::<B, S2, B::Schema> {
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
    pub fn generate_cross_schematic<B: Schematic, S2: FromSchema<B::Schema> + ?Sized>(
        &self,
        block: B,
    ) -> SchemaCellHandle<S2, B> {
        self.generate_cross_schematic_inner(Arc::new(block))
    }

    /// Generates a schematic for `block` in the background.
    ///
    /// Returns a handle to the cell being generated.
    pub fn generate_schematic<T: Schematic>(&self, block: T) -> SchemaCellHandle<T::Schema, T> {
        let block = Arc::new(block);
        self.generate_schematic_inner(block)
    }

    /// Export the given block and all sub-blocks as a SCIR library.
    ///
    /// Returns a SCIR library and metadata for converting between SCIR and Substrate formats.
    pub fn export_scir<T: Schematic>(&self, block: T) -> Result<RawLib<T::Schema>, ConvError> {
        let cell = self.generate_schematic(block);
        // TODO: Handle errors.
        let SchemaCellCacheValue { raw, .. } = cell.handle.unwrap_inner();
        raw.to_scir_lib()
    }

    /// Export the given cells and all their subcells as a SCIR library.
    ///
    /// Returns a SCIR library and metadata for converting between SCIR and Substrate formats.
    pub fn export_scir_all<S: Schema + ?Sized>(
        &self,
        cells: &[&crate::schematic::RawCell<S>],
    ) -> Result<RawLib<S>, ConvError> {
        export_multi_top_scir_lib(cells)
    }

    /// Returns a simulation controller for the given testbench and simulator.
    pub fn get_sim_controller<S, T>(
        &self,
        block: T,
        work_dir: impl Into<PathBuf>,
    ) -> Result<SimController<S, T>>
    where
        S: Simulator,
        T: Testbench<S>,
    {
        let simulator = self
            .get_installation::<S>()
            .expect("Simulator must be installed");
        let block = Arc::new(block);
        let cell = self.generate_schematic_inner(block.clone());
        // TODO: Handle errors.
        let SchemaCellCacheValue { raw, cell } = cell.handle.unwrap_inner();
        let lib = raw.to_scir_lib()?;
        let ctx = SimulationContext {
            lib: Arc::new(lib),
            work_dir: work_dir.into(),
            ctx: self.clone(),
        };
        Ok(SimController {
            tb: cell.clone(),
            simulator,
            ctx,
        })
    }

    /// Installs the given [`PrivateInstallation`].
    ///
    /// Only one installation of any given type can exist. Overwrites
    /// conflicting installations of the same type.
    #[inline]
    pub fn install<I>(&mut self, installation: I) -> Arc<I>
    where
        I: PrivateInstallation,
    {
        let installation = Arc::new(installation);
        self.inner
            .write()
            .unwrap()
            .private_installations
            .insert(TypeId::of::<I>(), installation.clone());
        installation
    }

    /// Installs the given [`PrivateInstallation`].
    ///
    /// Returns the existing installation if one is present.
    #[inline]
    pub fn get_or_install<I>(&self, installation: I) -> Arc<I>
    where
        I: PrivateInstallation,
    {
        let installation = Arc::new(installation);
        self.inner
            .write()
            .unwrap()
            .private_installations
            .entry(TypeId::of::<I>())
            .or_insert(installation.clone())
            .clone()
            .downcast()
            .unwrap()
    }

    /// Gets a private installation from the context installation map.
    pub fn get_private_installation<I: PrivateInstallation>(&self) -> Option<Arc<I>> {
        retrieve_installation(&self.inner.read().unwrap().private_installations)
    }

    /// Gets an installation from the context installation map.
    pub fn get_installation<I: Installation>(&self) -> Option<Arc<I>> {
        retrieve_installation(&self.installations)
    }

    /// Generates a layout for `block` in the background.
    ///
    /// Returns a handle to the cell being generated.
    pub fn generate_layout<T: Layout>(&self, block: T) -> LayoutCellHandle<T> {
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
            block: block.clone(),
            cell: inner_mut.layout.cell_cache.generate(block, move |block| {
                let block_io = block.io();
                let mut cell_builder = LayoutCellBuilder::new(context_clone);
                let _guard = span.enter();
                let (io, data) = block.layout(&mut cell_builder)?;
                if block_io.kind() != io.kind()
                    || block_io.kind().flat_names(None).len() != io.len()
                {
                    tracing::event!(
                        Level::ERROR,
                        "layout IO and block IO have different bundle kinds or flattened lengths"
                    );
                    return Err(LayoutError::IoDefinition.into());
                }
                let ports = IndexMap::from_iter(
                    block
                        .io()
                        .kind()
                        .flat_names(None)
                        .into_iter()
                        .zip(io.flatten_vec()),
                );
                Ok(LayoutCell::new(
                    block.clone(),
                    data,
                    io,
                    Arc::new(cell_builder.finish(id, block.name()).with_ports(ports)),
                ))
            }),
        }
    }

    /// Exports the layout of a block to a LayIR library.
    pub fn export_layir<T: Layout>(
        &self,
        block: T,
    ) -> Result<crate::layout::conv::RawLib<<T::Schema as crate::layout::schema::Schema>::Layer>>
    {
        let handle = self.generate_layout(block);
        let cell = handle.try_cell()?;
        let lib = cell.raw().to_layir_lib()?;
        Ok(lib)
    }

    /// Writes a set of layout cells to a LayIR library.
    pub fn export_layir_all<'a, L: Clone + 'a>(
        &self,
        cells: impl IntoIterator<Item = &'a RawCell<L>>,
    ) -> Result<crate::layout::conv::RawLib<L>> {
        let cells = cells.into_iter().collect::<Vec<_>>();
        let lib = export_multi_top_layir_lib(&cells)?;
        Ok(lib)
    }

    /// Imports a LayIR library into the context.
    pub fn import_layir<S: crate::layout::schema::Schema>(
        &self,
        lib: layir::Library<S::Layer>,
        top: layir::CellId,
    ) -> Result<Arc<crate::layout::element::RawCell<S::Layer>>> {
        use crate::layout::element::{RawCell, RawInstance};
        let mut inner = self.inner.write().unwrap();
        let mut cells: HashMap<layir::CellId, Arc<RawCell<S::Layer>>> = HashMap::new();
        for id in lib.topological_order() {
            let cell = lib.cell(id);
            let sid = inner.layout.get_id();
            let mut raw = RawCell::new(sid, cell.name());
            for elt in cell.elements() {
                raw.add_element(elt.clone());
            }
            for (_, inst) in cell.instances() {
                let rinst = RawInstance::new(
                    cells.get(&inst.child()).unwrap().clone(),
                    inst.transformation(),
                );
                raw.add_element(rinst);
            }
            let mut ports = NamedPorts::new();
            for (name, port) in cell.ports() {
                let mut pg = PortGeometryBuilder::default();
                for elt in port.elements() {
                    if let layir::Element::Shape(s) = elt {
                        pg.push(s.clone());
                    }
                }
                let pg = pg.build()?;
                ports.insert(NameBuf::from(name), pg);
            }
            raw = raw.with_ports(ports);
            let cell = Arc::new(raw);
            cells.insert(id, cell);
        }
        Ok(cells.get(&top).unwrap().clone())
    }
}

fn retrieve_installation<I: Any + Send + Sync>(
    map: &HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
) -> Option<Arc<I>> {
    map.get(&TypeId::of::<I>())
        .map(|arc| arc.clone().downcast().unwrap())
}

/// Only public for use in ATOLL. Do NOT use externally.
///
/// If the `id` argument is Some, the cell will use the given ID.
/// Otherwise, a new [`CellId`] will be allocated by calling [`Context::alloc_cell_id`].
#[doc(hidden)]
pub fn prepare_cell_builder<T: Schematic>(
    id: Option<CellId>,
    context: Context,
    block: &T,
) -> (CellBuilder<T::Schema>, IoNodeBundle<T>) {
    let id = id.unwrap_or_else(|| context.alloc_cell_id());
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

    let names = <<T as Block>::Io as HasBundleKind>::kind(&io_outward).flat_names(None);
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
            cell_name,
            ctx: context,
            node_ctx,
            node_names,
            fatal_error: false,
            ports,
            flatten: false,
            contents: RawCellContentsBuilder::Cell(RawCellInnerBuilder::default()),
        },
        io_data,
    )
}
