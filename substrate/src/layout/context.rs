//! Layout-specific context data.
//!
//! Stores generated layout cells as well as state used for assigning unique cell IDs.

use crate::generator::Generator;

use super::element::CellId;

/// A wrapper around layout-specific context data.
#[derive(Debug, Default, Clone)]
pub struct LayoutContext {
    next_id: CellId,
    gen: Generator,
}

impl LayoutContext {
    #[allow(dead_code)]
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn gen_mut(&mut self) -> &mut Generator {
        &mut self.gen
    }

    pub(crate) fn get_id(&mut self) -> CellId {
        let tmp = self.next_id;
        self.next_id += 1;
        tmp
    }
}
