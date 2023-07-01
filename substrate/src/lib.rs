//! The Substrate analog circuit generator framework.
#![warn(missing_docs)]
pub use codegen::*;
#[doc(inline)]
pub use geometry;
#[doc(inline)]
pub use scir;
pub use substrate_api::*;

pub mod ios;

// Re-exported for procedural macros.
#[doc(hidden)]
pub use arcstr;
#[doc(hidden)]
pub use duplicate;
