//! The Substrate analog circuit generator framework.
#![warn(missing_docs)]

extern crate self as substrate;

#[doc(inline)]
pub use geometry;
#[doc(inline)]
pub use scir;
#[doc(inline)]
pub use spice;
#[doc(inline)]
pub use type_dispatch;

pub mod block;
pub mod cache;
pub mod context;
pub mod error;
pub mod execute;
pub mod io;
pub mod layout;
pub mod pdk;
pub mod schematic;
pub mod simulation;

mod diagnostics;

// Re-exported for procedural macros.
#[doc(hidden)]
pub use arcstr;
#[doc(hidden)]
pub use duplicate;
#[doc(hidden)]
pub use serde;
