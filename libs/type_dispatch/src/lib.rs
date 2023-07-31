//! Utilities for dispatching based on generic types.

#![warn(missing_docs)]

extern crate self as type_dispatch;

#[doc(hidden)]
pub use duplicate;
pub use type_dispatch_macros::*;

mod tests;

pub trait Dispatch {
    type Output;

    fn dispatch(self) -> Self::Output;
}

pub trait DispatchConst {
    type Constant;
    const CONST: Self::Constant;
}
