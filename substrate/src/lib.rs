//! The Substrate analog circuit generator framework.
#![warn(missing_docs)]

extern crate self as substrate;

pub use codegen::*;
#[doc(inline)]
pub use geometry;
#[doc(inline)]
pub use scir;
#[doc(inline)]
pub use spice;

pub mod block;
pub mod context;
pub mod error;
pub mod execute;
pub mod io;
pub mod layout;
pub mod pdk;
pub mod schematic;
pub mod simulation;

// Re-exported for procedural macros.
#[doc(hidden)]
pub use arcstr;
#[doc(hidden)]
pub use duplicate;
