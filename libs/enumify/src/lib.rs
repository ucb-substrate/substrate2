//! A crate for providing automatic enum utilities.
//!
//! # Example
//!
//! Consider an enum like this:
//!
//! ```
//! pub enum ArcNumber {
//!   Float(Arc<f64>),
//!   Int(Arc<u64>),
//! }
//! ```
//!
//! With enumify, you should write this:
//!
//! ```
//! #[enumify::enumify]
//! pub enum Number<F, I> {
//!   Float(F),
//!   Int(I),
//! }
//! pub type ArcNumber = Number<Arc<f64>, Arc<u64>>;
//!
//! let x = ArcNumber::Int(Arc::new(123));
//! let inner: &Arc<u64> = x.as_ref().unwrap_int();
//! assert_eq!(*inner, 123);
//! ```
//!
//! The enumify macro adds the following methods:
//! * `as_ref`, which converts `&MyEnum<T, U>` to `MyEnum<&T, &U>`
//! * `as_mut`, which converts `&mut MyEnum<T, U>` to `MyEnum<&mut T, &mut U>`
//! * `unwrap_{variant}`, which asserts that an enum value is a particular variant, and returns the
//!    inner value. Only generated for tuple enum variants with a single field.
//! * `is_{variant}`, which returns `true` if the enum value is the given variant.

pub use enumify_macros::*;
