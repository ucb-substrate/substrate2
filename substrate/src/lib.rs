//! The Substrate analog circuit generator framework.
#![warn(missing_docs)]

extern crate self as substrate;

pub mod block;
pub mod cache;
pub mod context;
pub mod error;
pub mod execute;
pub mod io;
pub mod layout;
pub mod lut;
pub mod pdk;
pub mod schematic;
pub mod simulation;

mod diagnostics;

// Re-exported for procedural macros.
#[doc(hidden)]
pub use arcstr;
#[doc(hidden)]
pub use duplicate;
#[doc(inline)]
pub use geometry;
#[doc(inline)]
pub use scir;
#[doc(hidden)]
pub use serde;
#[doc(inline)]
pub use type_dispatch;
