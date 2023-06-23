//! Substrate's layout generation framework.

use crate::block::Block;
use crate::error::Result;

use self::builder::CellBuilder;

pub mod builder;
pub mod cell;
pub mod context;
pub mod draw;
pub mod element;

/// A block that has a layout.
pub trait HasLayout: Block {
    /// Extra data to be stored with the block's generated cell.
    ///
    /// Common uses include storing important instances for access during simulation and any
    /// important computations that may impact blocks that instantiate this block.
    type Data: Send + Sync;

    /// Generates the block's layout.
    fn layout(&self, cell: &mut CellBuilder<Self>) -> Result<Self::Data>;
}
