//! PDK layer interface.

use arcstr::ArcStr;
use serde::{Deserialize, Serialize};

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
    pub gds: Option<(u16, u16)>,
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
pub trait Layers: Send + Sync {
    /// Instantiates the identifiers and data contained within the set of layers.
    fn new(ctx: &mut LayerContext) -> Self;
    /// Flattens `self` into a list of [`LayerInfo`] objects for Substrate's internal purposes.
    fn flatten(&self) -> Vec<LayerInfo>;
}
