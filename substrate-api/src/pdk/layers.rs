//! PDK layer interface.

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};

use arcstr::ArcStr;
use serde::{Deserialize, Serialize};
use tracing::Level;

/// A context-wide unique identifier for a layer.
#[derive(
    Default, Debug, Copy, Clone, Serialize, Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct LayerId(u64);

impl LayerId {
    fn increment(&mut self) {
        *self = LayerId(self.0 + 1);
    }
}

impl AsRef<LayerId> for LayerId {
    #[inline]
    fn as_ref(&self) -> &LayerId {
        self
    }
}

/// A context used for assigning identifiers to user-defined layers.
#[derive(Default, Debug, Clone)]
pub struct LayerContext {
    next_id: LayerId,
    installed_layers: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
    layers_gds_to_info: HashMap<GdsLayerSpec, LayerInfo>,
    layers_id_to_info: HashMap<LayerId, LayerInfo>,
}

impl LayerContext {
    /// Generates a new layer ID.
    pub fn new_layer(&mut self) -> LayerId {
        self.next_id.increment();
        self.next_id
    }

    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn install_layers<L: Layers>(&mut self) -> Arc<L> {
        let layers = L::new(self);
        let id = TypeId::of::<L>();
        for layer in layers.flatten() {
            if let Some(gds) = layer.gds {
                if self.layers_gds_to_info.insert(gds, layer.clone()).is_some() {
                    tracing::event!(
                        Level::WARN,
                        "installing previously installed GDS layer {:?} again",
                        gds
                    );
                }
            }
            self.layers_id_to_info.insert(layer.id, layer);
        }
        self.installed_layers
            .entry(id)
            .or_insert(Arc::new(layers))
            .clone()
            .downcast::<L>()
            .unwrap()
    }

    pub(crate) fn get_gds_layer(&self, spec: GdsLayerSpec) -> Option<LayerId> {
        self.layers_gds_to_info.get(&spec).map(|info| info.id)
    }

    pub(crate) fn get_gds_layer_from_id(&self, id: LayerId) -> Option<GdsLayerSpec> {
        self.layers_id_to_info.get(&id).unwrap().gds
    }
}

/// A GDS layer specification.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct GdsLayerSpec(pub u8, pub u8);

/// A struct containing general information for a PDK layer.
#[derive(Debug, Clone)]
pub struct LayerInfo {
    /// A context-wide unique identifier generated from the [`LayerContext`].
    pub id: LayerId,
    /// The layer name.
    pub name: ArcStr,
    /// The layer's corresponding GDS layer.
    ///
    /// Layers without a GDS layer will not be exported but can be used
    /// like normal within Substrate.
    pub gds: Option<GdsLayerSpec>,
}

impl AsRef<LayerId> for LayerInfo {
    #[inline]
    fn as_ref(&self) -> &LayerId {
        &self.id
    }
}

/// A PDK layer.
pub trait Layer: Copy {
    /// Instantiates the identifiers and data contained within this layer.
    fn new(ctx: &mut LayerContext) -> Self;
    /// Converts a PDK layer object to a general format that Substrate can use.
    fn info(&self) -> LayerInfo;
}

/// A PDK layer that has a corresponding pin layer in `L`.
pub trait HasPin<L: Layers>: Layer {
    /// Returns the ID of the corresponding pin layer.
    fn pin_id(&self, layers: &L) -> LayerId;
    /// Returns the ID of the corresponding text layer for port labels.
    fn label_id(&self, layers: &L) -> LayerId;
}

/// A set of layers used by a PDK.
pub trait Layers: Any + Send + Sync {
    /// Instantiates the identifiers and data contained within the set of layers.
    fn new(ctx: &mut LayerContext) -> Self;
    /// Flattens `self` into a list of [`LayerInfo`] objects for Substrate's internal purposes.
    fn flatten(&self) -> Vec<LayerInfo>;
}
