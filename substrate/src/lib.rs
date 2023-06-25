//! The Substrate analog circuit generator framework.
#![warn(missing_docs)]
pub use codegen::*;
#[doc(inline)]
pub use geometry;
pub use substrate_api::*;

// Re-exports for macros.
#[doc(hidden)]
pub use duplicate;
