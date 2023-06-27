//! Substrate's layout generator framework.
///
/// # Examples
///
/// ## Simple
/// ```
#[doc = include_str!("../../../docs/api/code/prelude.md.hidden")]
#[doc = include_str!("../../../docs/api/code/pdk/layers.md.hidden")]
#[doc = include_str!("../../../docs/api/code/pdk/pdk.md.hidden")]
#[doc = include_str!("../../../docs/api/code/block/inverter.md.hidden")]
#[doc = include_str!("../../../docs/api/code/layout/inverter.md")]
/// ```
///
/// ## With data
/// ```
#[doc = include_str!("../../../docs/api/code/prelude.md.hidden")]
#[doc = include_str!("../../../docs/api/code/pdk/layers.md.hidden")]
#[doc = include_str!("../../../docs/api/code/pdk/pdk.md.hidden")]
#[doc = include_str!("../../../docs/api/code/block/inverter.md.hidden")]
#[doc = include_str!("../../../docs/api/code/layout/inverter.md.hidden")]
#[doc = include_str!("../../../docs/api/code/block/buffer.md.hidden")]
#[doc = include_str!("../../../docs/api/code/layout/buffer.md")]
/// ```
use crate::block::Block;
use crate::error::Result;
use crate::io::LayoutType;
use crate::pdk::Pdk;

use self::builder::CellBuilder;

pub mod builder;
pub mod cell;
pub mod context;
pub mod draw;
pub mod element;
pub mod error;
pub mod gds;

/// A block that has a layout.
pub trait HasLayout: Block {
    /// Extra data to be stored with the block's generated cell.
    ///
    /// Common uses include storing important instances for access during simulation and any
    /// important computations that may impact blocks that instantiate this block.
    type Data: Send + Sync;
}

/// A block that has a layout for process design kit `PDK`.
pub trait HasLayoutImpl<PDK: Pdk>: HasLayout {
    /// Generates the block's layout.
    fn layout(
        &self,
        io: <<Self as Block>::Io as LayoutType>::Data,
        cell: &mut CellBuilder<PDK, Self>,
    ) -> Result<Self::Data>;
}
