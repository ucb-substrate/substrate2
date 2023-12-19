//! Utilities for dispatching based on generic types.

#![warn(missing_docs)]

extern crate self as type_dispatch;

#[doc(hidden)]
pub use duplicate;
pub use type_dispatch_macros::*;

pub mod derive;
mod tests;

/// A dispatch of an object.
///
/// # Examples
///
/// ```
/// # use std::marker::PhantomData;
/// # use type_dispatch::Dispatch;
/// # use type_dispatch_macros::impl_dispatch;
/// #[derive(Debug, Default, PartialEq, Eq)]
/// struct Painting(Vec<usize>);
/// struct Stroke {
///     thickness: usize,
/// }
/// impl Painting {
///     fn draw(&mut self, stroke: Stroke) {
///         self.0.push(2 * stroke.thickness)
///     }
/// }
/// #[derive(Debug, Default, PartialEq, Eq)]
/// struct Photoshop(Vec<usize>);
/// struct Filter {
///     strength: usize,
/// }
/// impl Photoshop {
///     fn draw(&mut self, filter: Filter) {
///         self.0.push(5 * filter.strength)
///     }
/// }
///
/// struct Dispatcher<T>(usize, PhantomData<T>);
///
/// impl Dispatch for Dispatcher<Painting> {
///     type Output = Stroke;
///     fn dispatch(self) -> Self::Output {
///         Stroke { thickness: self.0 }
///     }
/// }
/// impl Dispatch for Dispatcher<Photoshop> {
///     type Output = Filter;
///     fn dispatch(self) -> Self::Output {
///         Filter { strength: self.0 }
///     }
/// }
///
/// #[impl_dispatch({Painting; Photoshop})]
/// impl<A> Into<A> for Vec<usize> {
///     fn into(self) -> A {
///         let mut drawing = A::default();
///         for num in self {
///             drawing.draw(Dispatcher(num, PhantomData::<A>).dispatch());
///         }
///         drawing
///     }
/// }
///
/// let painting: Painting = vec![1, 2, 3].into();
/// let photoshop: Photoshop = vec![1, 2, 3].into();
/// assert_eq!(painting, Painting(vec![2, 4, 6]));
/// assert_eq!(photoshop, Photoshop(vec![5, 10, 15]));
/// ```
pub trait Dispatch {
    /// The type of the output object.
    type Output;

    /// Dispatches the object to a new object.
    fn dispatch(self) -> Self::Output;
}

/// A dispatch of a static function.
///
/// Prefer using the [`dispatch_fn`] macro unless the dispatcher will
/// be used several times.
///
/// # Examples
///
/// ```
/// # use std::marker::PhantomData;
/// # use type_dispatch::DispatchFn;
/// # use type_dispatch_macros::impl_dispatch;
/// #[derive(Debug, Default, PartialEq, Eq)]
/// struct Painting(Vec<usize>);
/// struct Stroke {
///     thickness: usize,
/// }
/// impl Painting {
///     fn draw(&mut self, stroke: Stroke) {
///         self.0.push(2 * stroke.thickness)
///     }
/// }
/// #[derive(Debug, Default, PartialEq, Eq)]
/// struct Photoshop(Vec<usize>);
/// struct Filter {
///     strength: usize,
/// }
/// impl Photoshop {
///     fn draw(&mut self, filter: Filter) {
///         self.0.push(5 * filter.strength)
///     }
/// }
///
/// #[derive(Default)]
/// struct Dispatcher<T>(PhantomData<T>);
///
/// impl DispatchFn for Dispatcher<Painting> {
///     type Output = Stroke;
///     fn dispatch_fn() -> Self::Output {
///         Stroke { thickness: 5 }
///     }
/// }
/// impl DispatchFn for Dispatcher<Photoshop> {
///     type Output = Filter;
///     fn dispatch_fn() -> Self::Output {
///         Filter { strength: 3}
///     }
/// }
///
/// struct SingleStrokeMasterpiece;
///
/// #[impl_dispatch({Painting; Photoshop})]
/// impl<A> Into<A> for SingleStrokeMasterpiece {
///     fn into(self) -> A {
///         let mut drawing = A::default();
///         drawing.draw(Dispatcher::<A>::dispatch_fn());
///         drawing
///     }
/// }
///
/// let painting: Painting = SingleStrokeMasterpiece.into();
/// let photoshop: Photoshop = SingleStrokeMasterpiece.into();
/// assert_eq!(painting, Painting(vec![10]));
/// assert_eq!(photoshop, Photoshop(vec![15]));
/// ```
pub trait DispatchFn {
    /// The type of the dispatched function's output.
    type Output;

    /// Dispatches a static function.
    fn dispatch_fn() -> Self::Output;
}

/// A dispatch of a constant.
///
/// Prefer using the [`dispatch_const`] macro unless the dispatcher will
/// be used several times.
///
/// # Examples
///
///
/// ```
/// # use std::marker::PhantomData;
/// # use type_dispatch::{DispatchConst, DispatchFn};
/// # use type_dispatch_macros::impl_dispatch;
/// #[derive(Debug, Default, PartialEq, Eq)]
/// struct Painting(Vec<usize>);
/// impl Painting {
///     fn draw(&mut self, stroke: usize) {
///         self.0.push(2 * stroke)
///     }
/// }
/// #[derive(Debug, Default, PartialEq, Eq)]
/// struct Photoshop(Vec<usize>);
/// impl Photoshop {
///     fn draw(&mut self, filter: usize) {
///         self.0.push(5 * filter)
///     }
/// }
///
/// #[derive(Default)]
/// struct Dispatcher<T>(PhantomData<T>);
///
/// impl DispatchConst for Dispatcher<Painting> {
///     type Constant = usize;
///     const CONST: Self::Constant = 5;
/// }
/// impl DispatchConst for Dispatcher<Photoshop> {
///     type Constant = usize;
///     const CONST: Self::Constant = 3;
/// }
///
/// struct SingleStrokeMasterpiece;
///
/// #[impl_dispatch({Painting; Photoshop})]
/// impl<A> Into<A> for SingleStrokeMasterpiece {
///     fn into(self) -> A {
///         let mut drawing = A::default();
///         drawing.draw(Dispatcher::<A>::CONST);
///         drawing
///     }
/// }
///
/// let painting: Painting = SingleStrokeMasterpiece.into();
/// let photoshop: Photoshop = SingleStrokeMasterpiece.into();
/// assert_eq!(painting, Painting(vec![10]));
/// assert_eq!(photoshop, Photoshop(vec![15]));
/// ```
pub trait DispatchConst {
    /// The type of the constant.
    type Constant;

    /// The constant.
    const CONST: Self::Constant;
}
