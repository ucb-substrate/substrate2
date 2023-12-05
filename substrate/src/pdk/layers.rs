//! PDK layer interface.

use std::sync::RwLock;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};

use crate::pdk::Pdk;
use arcstr::ArcStr;
pub use codegen::{DerivedLayerFamily, DerivedLayers, Layer, LayerFamily, Layers};
use serde::{Deserialize, Serialize};
use slotmap::{new_key_type, SlotMap};
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

new_key_type! {
    /// A key for layer families in a [`LayerContext`].
    pub(crate) struct LayerFamilyKey;
}

pub(crate) struct InstalledLayers<PDK: Pdk> {
    pub(crate) layers: Arc<PDK::Layers>,
    pub(crate) ctx: Arc<RwLock<LayerContext>>,
}

/// A context used for assigning identifiers to user-defined layers.
#[derive(Default, Debug, Clone)]
pub struct LayerContext {
    next_id: LayerId,
    installed_layers: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
    layers_gds_to_info: HashMap<GdsLayerSpec, LayerInfo>,
    layers_id_to_info: HashMap<LayerId, LayerInfo>,
    layer_id_to_family_key: HashMap<LayerId, LayerFamilyKey>,
    layer_families: SlotMap<LayerFamilyKey, LayerFamilyInfo>,
}

impl LayerContext {
    /// Generates a new layer ID.
    pub fn new_layer(&mut self) -> LayerId {
        self.next_id.increment();
        self.next_id
    }

    /// Installs a new layer, using a closure that produces a [`LayerInfo`]
    /// when given a new [`LayerId`].
    pub(crate) fn new_layer_with_id<F>(&mut self, f: F) -> LayerId
    where
        F: FnOnce(LayerId) -> LayerInfo,
    {
        let id = self.new_layer();
        let info = f(id);
        self.layers_id_to_info.insert(id, info.clone());
        if let Some(gds) = info.gds {
            self.layers_gds_to_info.insert(gds, info);
        }
        id
    }

    pub(crate) fn install_layers<L: Layers>(&mut self) -> Arc<L> {
        let layers = L::new(self);
        let id = TypeId::of::<L>();
        for layer_family in layers.flatten() {
            let family_key = self.layer_families.insert(layer_family);
            let layer_family = &self.layer_families[family_key];
            for layer in layer_family.layers.iter() {
                if let Some(gds) = layer.gds {
                    if self.layers_gds_to_info.insert(gds, layer.clone()).is_some() {
                        tracing::event!(
                            Level::WARN,
                            "installing previously installed GDS layer {:?} again",
                            gds
                        );
                    }
                }
                self.layers_id_to_info.insert(layer.id, layer.clone());
                self.layer_id_to_family_key.insert(layer.id, family_key);
            }
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

    pub(crate) fn layer_family_for_layer_id(&self, id: LayerId) -> Option<&LayerFamilyInfo> {
        let fkey = *self.layer_id_to_family_key.get(&id)?;
        self.layer_families.get(fkey)
    }
}

/// A GDS layer specification.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct GdsLayerSpec(pub u16, pub u16);

/// A struct containing general information for a PDK layer family.
#[derive(Debug, Clone)]
pub struct LayerFamilyInfo {
    /// A list of contained layers.
    pub layers: Vec<LayerInfo>,
    /// The primary drawing layer of this family.
    pub primary: LayerId,
    /// The layer where pin shapes should be drawn.
    pub pin: Option<LayerId>,
    /// The layer where pin labels should be drawn.
    pub label: Option<LayerId>,
}

impl AsRef<LayerId> for LayerFamilyInfo {
    #[inline]
    fn as_ref(&self) -> &LayerId {
        &self.primary
    }
}

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
    /// Gets the ID of `self`.
    fn id(&self) -> LayerId {
        self.info().id
    }
}

/// A PDK layer family.
pub trait LayerFamily: Copy {
    /// Instantiates the identifiers and data contained within this layer family.
    fn new(ctx: &mut LayerContext) -> Self;
    /// Converts a PDK layer family object to a general format that Substrate can use.
    fn info(&self) -> LayerFamilyInfo;
}

/// A set of layers that uniquely specifies how a pin should be drawn.
///
/// Shapes are drawn on the drawing and pin layers by default, with port name annotations added on
/// the label layer.
pub trait HasPin {
    /// The drawing layer corresponding to the pin.
    fn drawing(&self) -> LayerId;
    /// The layer where the pin is drawn.
    fn pin(&self) -> LayerId;
    /// The layer where the pin label is written.
    fn label(&self) -> LayerId;
}

/// A set of layers used by a PDK.
pub trait Layers: Any + Send + Sync {
    /// Instantiates the identifiers and data contained within the set of layers.
    fn new(ctx: &mut LayerContext) -> Self;
    /// Flattens `self` into a list of [`LayerInfo`] objects for Substrate's internal purposes.
    fn flatten(&self) -> Vec<LayerFamilyInfo>;
}

impl TryFrom<gds::GdsLayerSpec> for GdsLayerSpec {
    type Error = std::num::TryFromIntError;
    fn try_from(value: gds::GdsLayerSpec) -> Result<Self, Self::Error> {
        let layer = u16::try_from(value.layer)?;
        let xtype = u16::try_from(value.xtype)?;
        Ok(Self(layer, xtype))
    }
}
