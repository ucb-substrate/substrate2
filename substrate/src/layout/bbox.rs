//! Bounding box utilities.

use crate::pdk::layers::LayerId;
use geometry::prelude::*;
use geometry::union::BoundingUnion;

/// A trait representing functions available for multi-layered objects with bounding boxes.
pub trait LayerBbox: Bbox {
    /// Compute the bounding box considering only objects occupying the given layer.
    fn layer_bbox(&self, layer: LayerId) -> Option<Rect>;
}

impl<T: LayerBbox> LayerBbox for Vec<T> {
    fn layer_bbox(&self, layer: LayerId) -> Option<Rect> {
        let mut bbox = None;
        for item in self {
            bbox = bbox.bounding_union(&item.layer_bbox(layer));
        }
        bbox
    }
}

impl<T: LayerBbox> LayerBbox for &T {
    fn layer_bbox(&self, layer: LayerId) -> Option<Rect> {
        (*self).layer_bbox(layer)
    }
}
