//! The global context.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use once_cell::sync::OnceCell;

use tracing::{span, Level};

use crate::block::Block;
use crate::error::Result;
use crate::io::{
    FlatLen, Flatten, HasNameTree, LayoutDataBuilder, LayoutType, NodeContext, NodePriority, Port,
    SchematicType,
};
use crate::layout::element::RawCell;
use crate::layout::error::{GdsExportError, LayoutError};
use crate::layout::gds::GdsExporter;
use crate::layout::Cell as LayoutCell;
use crate::layout::CellBuilder as LayoutCellBuilder;
use crate::layout::HasLayoutImpl;
use crate::layout::LayoutContext;
use crate::pdk::layers::GdsLayerSpec;
use crate::pdk::layers::LayerContext;
use crate::pdk::layers::LayerId;
use crate::pdk::layers::Layers;
use crate::pdk::Pdk;
use crate::schematic::{
    Cell as SchematicCell, CellBuilder as SchematicCellBuilder, HasSchematicImpl, SchematicContext,
};
use crate::simulation::{
    HasTestbenchSchematicImpl, SimController, SimulationConfig, Simulator, Testbench,
};

/// The global context.
///
/// Stores configuration such as the PDK and tool plugins to use during generation.
///
/// # Examples
///
/// ```
#[doc = include_str!("../../docs/api/code/prelude.rs.hidden")]
#[doc = include_str!("../../docs/api/code/pdk/layers.rs.hidden")]
#[doc = include_str!("../../docs/api/code/pdk/pdk.rs.hidden")]
#[doc = include_str!("../../docs/api/code/block/inverter.rs.hidden")]
#[doc = include_str!("../../docs/api/code/layout/inverter.rs.hidden")]
#[doc = include_str!("../../docs/api/code/block/buffer.rs.hidden")]
#[doc = include_str!("../../docs/api/code/layout/buffer.rs.hidden")]
#[doc = include_str!("../../docs/api/code/layout/generate.rs")]
/// ```
pub struct Context<PDK: Pdk> {
    /// PDK-specific data.
    pub pdk: PdkData<PDK>,
    inner: Arc<RwLock<ContextInner>>,
    simulators: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

/// Builder for creating a Substrate [`Context`].
pub struct ContextBuilder<PDK: Pdk> {
    pdk: Option<PDK>,
    simulators: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
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
        }
    }

    /// Set the PDK.
    #[inline]
    pub fn pdk(mut self, pdk: PDK) -> Self {
        self.pdk = Some(pdk);
        self
    }

    /// Install the given simulator.
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

    /// Build the context based on the configuration in this builder.
    pub fn build(self) -> Context<PDK> {
        // Instantiate PDK layers.
        let mut layer_ctx = LayerContext::new();
        let layers = layer_ctx.install_layers::<PDK::Layers>();

        Context {
            pdk: PdkData {
                pdk: Arc::new(self.pdk.unwrap()),
                layers,
            },
            inner: Arc::new(RwLock::new(ContextInner::new(layer_ctx))),
            simulators: self.simulators.clone(),
        }
    }
}

impl<PDK: Pdk> Clone for Context<PDK> {
    fn clone(&self) -> Self {
        Self {
            pdk: self.pdk.clone(),
            inner: self.inner.clone(),
            simulators: self.simulators.clone(),
        }
    }
}

/// PDK data stored in the global context.
pub struct PdkData<PDK: Pdk> {
    /// PDK configuration and general data.
    pub pdk: Arc<PDK>,
    /// The PDK layer set.
    pub layers: Arc<PDK::Layers>,
}

impl<PDK: Pdk> Clone for PdkData<PDK> {
    fn clone(&self) -> Self {
        Self {
            pdk: self.pdk.clone(),
            layers: self.layers.clone(),
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
    pub fn generate_layout<T: HasLayoutImpl<PDK>>(
        &mut self,
        block: T,
    ) -> Arc<OnceCell<Result<LayoutCell<T>>>> {
        let context_clone = self.clone();
        let mut inner_mut = self.inner.write().unwrap();
        let id = inner_mut.layout.get_id();
        let span = span!(
            Level::INFO,
            "generating layout",
            block = %block.name(),
        )
        .or_current();
        inner_mut.layout.gen.generate(block.clone(), move || {
            let mut io_builder = block.io().builder();
            let mut cell_builder = LayoutCellBuilder::new(id, block.name(), context_clone);
            let _guard = span.enter();
            let data = block.layout(&mut io_builder, &mut cell_builder);

            let io = io_builder.build()?;
            let ports = HashMap::from_iter(
                block
                    .io()
                    .flat_names(arcstr::literal!("io"))
                    .into_iter()
                    .zip(io.flatten_vec().into_iter()),
            );
            data.map(|data| {
                LayoutCell::new(
                    block,
                    data,
                    Arc::new(io),
                    Arc::new(RawCell::from_ports_and_builder(ports, cell_builder)),
                )
            })
        })
    }

    /// Writes a layout to a GDS files.
    pub fn write_layout<T: HasLayoutImpl<PDK>>(
        &mut self,
        block: T,
        path: impl AsRef<Path>,
    ) -> Result<()> {
        let handle = self.generate_layout(block);
        let cell = handle.wait().as_ref().map_err(|e| e.clone())?;

        let inner = self.inner.read().unwrap();
        GdsExporter::new(cell.raw.clone(), &inner.layers)
            .export()
            .map_err(LayoutError::from)?
            .save(path)
            .map_err(GdsExportError::from)
            .map_err(LayoutError::from)?;
        Ok(())
    }

    /// Generates a schematic for `block` in the background.
    ///
    /// Returns a handle to the cell being generated.
    pub fn generate_schematic<T: HasSchematicImpl<PDK>>(
        &mut self,
        block: T,
    ) -> Arc<OnceCell<Result<SchematicCell<T>>>> {
        let context = self.clone();
        let mut inner = self.inner.write().unwrap();
        let id = inner.schematic.get_id();
        inner.schematic.gen.generate(block.clone(), move || {
            let (mut cell_builder, io_data) = prepare_cell_builder(id, context, &block);
            let data = block.schematic(&io_data, &mut cell_builder);
            data.map(|data| SchematicCell::new(block, data, Arc::new(cell_builder.finish())))
        })
    }

    /// Generates a testbench schematic for `block` in the background.
    ///
    /// Returns a handle to the cell being generated.
    pub(crate) fn generate_testbench_schematic<T, S>(
        &mut self,
        block: T,
    ) -> Arc<OnceCell<Result<SchematicCell<T>>>>
    where
        T: HasTestbenchSchematicImpl<PDK, S>,
        S: Simulator,
    {
        let simulator = self.get_simulator::<S>();
        let context = self.clone();
        let mut inner = self.inner.write().unwrap();
        let id = inner.schematic.get_id();
        inner.schematic.gen.generate(block.clone(), move || {
            let (mut cell_builder, io_data) = prepare_cell_builder(id, context, &block);
            let data = block.schematic(&io_data, &simulator, &mut cell_builder);
            data.map(|data| SchematicCell::new(block, data, Arc::new(cell_builder.finish())))
        })
    }

    /// Export the given block and all sub-blocks as a SCIR library.
    pub fn export_scir<T: HasSchematicImpl<PDK>>(&mut self, block: T) -> scir::Library {
        let cell = self.generate_schematic(block);
        let cell = cell.wait().as_ref().unwrap();
        cell.raw
            .to_scir_lib(crate::schematic::conv::ExportAsTestbench::No)
    }

    /// Export the given block and all sub-blocks as a SCIR library.
    pub fn export_testbench_scir<T, S>(&mut self, block: T) -> scir::Library
    where
        T: HasTestbenchSchematicImpl<PDK, S>,
        S: Simulator,
    {
        let cell = self.generate_testbench_schematic(block);
        let cell = cell.wait().as_ref().unwrap();
        cell.raw
            .to_scir_lib(crate::schematic::conv::ExportAsTestbench::Yes)
    }

    /// Installs a new layer set in the context.
    ///
    /// Allows for accessing GDS layers or other extra layers that are not present in the PDK.
    pub fn install_layers<L: Layers>(&mut self) -> Arc<L> {
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
    pub fn simulate<S, T>(&mut self, block: T, work_dir: impl Into<PathBuf>) -> T::Output
    where
        S: Simulator,
        T: Testbench<PDK, S>,
    {
        let simulator = self.get_simulator::<S>();
        let lib = self.export_testbench_scir(block.clone());
        let config = SimulationConfig {
            lib,
            work_dir: work_dir.into(),
        };
        let controller = SimController { simulator, config };

        // TODO caching
        block.run(controller)
    }

    fn get_simulator<S: Simulator>(&self) -> Arc<S> {
        let arc = self.simulators.get(&TypeId::of::<S>()).unwrap().clone();
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

fn prepare_cell_builder<PDK: Pdk, T: Block>(
    id: crate::schematic::CellId,
    context: Context<PDK>,
    block: &T,
) -> (
    SchematicCellBuilder<PDK, T>,
    <<T as Block>::Io as SchematicType>::Data,
) {
    let mut node_ctx = NodeContext::new();
    let io = block.io();
    let nodes = node_ctx.nodes(io.len(), NodePriority::Io);
    let (io_data, nodes_rest) = io.instantiate(&nodes);
    assert!(nodes_rest.is_empty());
    let cell_name = block.name();

    let names = io.flat_names(arcstr::literal!("io"));
    let dirs = io.flatten_vec();
    assert_eq!(nodes.len(), names.len());
    assert_eq!(nodes.len(), dirs.len());

    let ports = nodes
        .iter()
        .copied()
        .zip(dirs)
        .map(|(node, direction)| Port::new(node, direction))
        .collect();

    let node_names = HashMap::from_iter(nodes.into_iter().zip(names));
    let cell_builder = SchematicCellBuilder {
        id,
        cell_name,
        ctx: context,
        node_ctx,
        instances: Vec::new(),
        primitives: Vec::new(),
        node_names,
        phantom: PhantomData,
        ports,
        blackbox: None,
    };

    (cell_builder, io_data)
}
