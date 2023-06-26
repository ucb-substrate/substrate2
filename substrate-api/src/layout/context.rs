//! Layout-specific context data.
//!
//! Stores generated layout cells as well as state used for assigning unique cell IDs.

use std::collections::HashMap;

use crate::{
    generator::Generator,
    pdk::layers::{GdsLayerSpec, LayerId, LayerInfo},
};

use super::element::CellId;

/// A wrapper around layout-specific context data.
#[derive(Debug, Default, Clone)]
pub struct LayoutContext {
    next_id: CellId,
    pub(crate) gen: Generator,
}

impl LayoutContext {
    #[allow(dead_code)]
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn get_id(&mut self) -> CellId {
        self.next_id.increment();
        self.next_id
    }
}
