//! The Substrate analog circuit generator framework.
#![warn(missing_docs)]

extern crate self as substrate;

use std::sync::Arc;

use arcstr::ArcStr;
pub use test_log::test;

pub mod block;
pub mod cache;
pub mod context;
mod diagnostics;
pub mod error;
pub mod execute;
pub mod layout;
pub mod lut;
pub mod schematic;
pub mod simulation;
#[cfg(test)]
pub(crate) mod tests;
pub mod types;

// Re-exported for procedural macros.
#[doc(hidden)]
pub use arcstr;
#[doc(hidden)]
pub use duplicate;
#[doc(inline)]
pub use geometry;
