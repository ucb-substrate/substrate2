//! Substrate's layout generator framework.

use crate::block::Block;
use crate::error::Result;

use self::builder::CellBuilder;

pub mod builder;
pub mod cell;
pub mod context;
pub mod draw;
pub mod element;

/// A block that has a layout.
///
/// # Examples
///
/// ## Simple
/// ```
#[doc = include_str!("../../docs/layout/inverter.md")]
/// ```
///
/// ## With data
/// ```
#[doc = include_str!("../../docs/layout/buffer.md")]
/// ```
pub trait HasLayout: Block {
    /// Extra data to be stored with the block's generated cell.
    ///
    /// Common uses include storing important instances for access during simulation and any
    /// important computations that may impact blocks that instantiate this block.
    type Data: Send + Sync;

    /// Generates the block's layout.
    fn layout(&self, cell: &mut CellBuilder<Self>) -> Result<Self::Data>;
}
