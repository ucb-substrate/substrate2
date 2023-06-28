//! The global context.

use std::collections::HashMap;
use std::marker::PhantomData;
use std::path::Path;
use std::sync::{Arc, RwLock};

use once_cell::sync::OnceCell;

use crate::error::Result;
use crate::io::{FlatLen, Flatten, LayoutType, NodeContext, Port, SchematicType};
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

/// The global context.
///
/// Stores configuration such as the PDK and tool plugins to use during generation.
///
/// # Examples
///
/// ```
#[doc = include_str!("../../docs/api/code/prelude.md.hidden")]
#[doc = include_str!("../../docs/api/code/pdk/layers.md.hidden")]
#[doc = include_str!("../../docs/api/code/pdk/pdk.md.hidden")]
#[doc = include_str!("../../docs/api/code/block/inverter.md.hidden")]
#[doc = include_str!("../../docs/api/code/layout/inverter.md.hidden")]
#[doc = include_str!("../../docs/api/code/block/buffer.md.hidden")]
#[doc = include_str!("../../docs/api/code/layout/buffer.md.hidden")]
#[doc = include_str!("../../docs/api/code/layout/generate.md")]
/// ```
pub struct Context<PDK: Pdk> {
    /// PDK-specific data.
    pub pdk: PdkData<PDK>,
    inner: Arc<RwLock<ContextInner>>,
}

impl<PDK: Pdk> Clone for Context<PDK> {
    fn clone(&self) -> Self {
        Self {
            pdk: self.pdk.clone(),
            inner: self.inner.clone(),
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

#[derive(Debug, Clone)]
pub(crate) struct ContextInner {
    layers: LayerContext,
    schematic: SchematicContext,
    layout: LayoutContext,
}

impl<PDK: Pdk> Context<PDK> {
    /// Creates a new global context.
    pub fn new(pdk: PDK) -> Self {
        // Instantiate PDK layers.
        let mut layer_ctx = LayerContext::new();
        let layers = layer_ctx.install_layers::<PDK::Layers>();

        Self {
            pdk: PdkData {
                pdk: Arc::new(pdk),
                layers,
            },
            inner: Arc::new(RwLock::new(ContextInner::new(layer_ctx))),
        }
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
        inner_mut.layout.gen.generate(block.clone(), move || {
            let mut io_builder = block.io().builder();
            let mut cell_builder = LayoutCellBuilder::new(id, block.name(), context_clone);
            let data = block.layout(&mut io_builder, &mut cell_builder);
            data.map(|data| LayoutCell::new(block, data, Arc::new(cell_builder.into())))
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
        let context_clone = self.clone();
        let mut inner_mut = self.inner.write().unwrap();
        let id = inner_mut.schematic.get_id();
        inner_mut.schematic.gen.generate(block.clone(), move || {
            let mut node_ctx = NodeContext::new();
            let io = block.io();
            let nodes = node_ctx.nodes(io.len());
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
            let mut cell_builder = SchematicCellBuilder {
                id,
                cell_name,
                ctx: context_clone,
                node_ctx,
                instances: Vec::new(),
                primitives: Vec::new(),
                node_names,
                phantom: PhantomData,
                ports,
            };
            let data = block.schematic(&io_data, &mut cell_builder);
            data.map(|data| SchematicCell::new(block, data, Arc::new(cell_builder.finish())))
        })
    }

    /// Export the given block and all sub-blocks as a SCIR library.
    pub fn export_scir<T: HasSchematicImpl<PDK>>(&mut self, block: T) -> scir::Library {
        let cell = self.generate_schematic(block);
        let cell = cell.wait().as_ref().unwrap();
        cell.raw.to_scir_lib()
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
