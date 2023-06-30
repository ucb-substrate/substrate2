//! The Substrate analog circuit generator framework.
#![warn(missing_docs)]

pub mod block;
pub mod context;
pub mod error;
pub(crate) mod generator;
pub mod io;
pub mod layout;
pub mod pdk;
pub mod schematic;
pub mod simulation;
pub use scir;

// Re-exported for procedural macros.
#[doc(hidden)]
pub use arcstr;
