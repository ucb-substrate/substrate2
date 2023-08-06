//! Macros for dispatching based on generic types.

#![warn(missing_docs)]

use crate::types::{dispatch_const_impl, dispatch_fn_impl, dispatch_type_impl};
use proc_macro::{TokenStream, TokenTree};
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;

use crate::impls::impl_dispatch_impl;

mod impls;
mod types;

/// Dispatches a trait implementation to a specified set of generic types.
///
/// # Syntax
///
/// Sets are delimited with curly braces (`{}`) and have semicolon-separated elements. Sets
/// are used to enumerate types to be assigned to a generic argument or set of generic arguments.
///
/// Comma-separated elements are combined using a cartesian product, making it easier to
/// implement traits on any combination of the provided types.
///
/// # Semantics
///
/// The supplied dispatch types are dispatched starting from the first generic type argument
/// of the trait implementation.
///
/// # Examples
///
/// ```
/// # use type_dispatch_macros::impl_dispatch;
/// struct GenericStruct<A, B>(A, B);
///
/// // Creates 4 trait implementations.
/// #[impl_dispatch({u64; u16}, {u32, usize; u8, u64})]
/// impl<A, B, C> Into<C> for GenericStruct<A, B> {
///     fn into(self) -> C {
///        self.0 as C + self.1 as C
///    }
/// }
///
/// let x: usize = GenericStruct(1u64, 3u32).into();
/// assert_eq!(x, 4);
/// let x: u64 = GenericStruct(1u64, 3u8).into();
/// assert_eq!(x, 4);
/// let x: usize = GenericStruct(1u16, 3u32).into();
/// assert_eq!(x, 4);
/// let x: u64 = GenericStruct(1u16, 3u8).into();
/// assert_eq!(x, 4);
///
/// // The following two lines will not compile as `GenericStruct` does not implement
/// // these particular type combinations:
/// // ```
/// // let x: u64 = GenericStruct(1u64, 3u32).into();
/// // let x: usize = GenericStruct(1u64, 3u8).into();
/// // ```
/// ```
#[proc_macro_attribute]
pub fn impl_dispatch(args: TokenStream, input: TokenStream) -> TokenStream {
    impl_dispatch_impl(args, input)
}

/// A function-like variant of [`macro@impl_dispatch`].
///
/// # Syntax
///
/// The syntax is the same as [`macro@impl_dispatch`], but the contents of the attribute must now
/// be contained by brackets (`[]`) or braces (`{}`).
///
/// # Examples
///
/// ```
/// # use type_dispatch_macros::dispatch_impl;
/// struct GenericStruct<A, B>(A, B);
///
/// // Creates 4 trait implementations.
/// dispatch_impl!{
///     [{u64; u16}, {u32, usize; u8, u64}]
///     impl<A, B, C> Into<C> for GenericStruct<A, B> {
///         fn into(self) -> C {
///            self.0 as C + self.1 as C
///        }
///     }
/// }
///
/// let x: usize = GenericStruct(1u64, 3u32).into();
/// assert_eq!(x, 4);
/// let x: u64 = GenericStruct(1u64, 3u8).into();
/// assert_eq!(x, 4);
/// let x: usize = GenericStruct(1u16, 3u32).into();
/// assert_eq!(x, 4);
/// let x: u64 = GenericStruct(1u16, 3u8).into();
/// assert_eq!(x, 4);
/// ```
#[proc_macro]
pub fn dispatch_impl(input: TokenStream) -> TokenStream {
    let mut iter = input.into_iter();
    impl_dispatch_impl(
        if let TokenTree::Group(g) = iter.next().unwrap() {
            g.stream()
        } else {
            panic!()
        },
        iter.collect(),
    )
}

/// Dispatches a constant based on a given generic type.
///
/// # Syntax
///
/// The syntax is effectively the same as a match statement, but the patterns are instead
/// simply comma-separated lists of types.
///
/// # Semantics
///
/// Unlike normal match statements, duplicate arms are not allowed. Only the constant
/// corresponding to the unique matching arm will be dispatched.
///
/// [`dispatch_const!`] internally uses the `DispatchConst` trait to dispatch constant values,
/// meaning that types do not have to match exactly (i.e. [`macro@impl_dispatch`] might have
/// `std::vec::Vec` while the [`dispatch_const!`] arm has `Vec`).
///
/// # Examples
///
/// ```
/// # use type_dispatch_macros::{dispatch_const, impl_dispatch};
/// struct GenericStruct<A, B>(A, B);
///
/// // Creates 4 trait implementations.
/// #[impl_dispatch({u64; u16}, {u32, usize; u8, u64})]
/// impl<A, B, C> Into<C> for GenericStruct<A, B> {
///     fn into(self) -> C {
///        self.0 as C + self.1 as C + dispatch_const!(
///             match A, B {
///                 u64, u32 => 1: C,
///                 u64, u8 => 2: C,
///                 u16, u32 => 3: C,
///                 u16, u8 => 4: C,
///             }           
///         )
///    }
/// }
///
/// let x: usize = GenericStruct(1u64, 3u32).into();
/// assert_eq!(x, 5);
/// let x: u64 = GenericStruct(1u64, 3u8).into();
/// assert_eq!(x, 6);
/// let x: usize = GenericStruct(1u16, 3u32).into();
/// assert_eq!(x, 7);
/// let x: u64 = GenericStruct(1u16, 3u8).into();
/// assert_eq!(x, 8);
/// ```
#[proc_macro]
pub fn dispatch_const(input: TokenStream) -> TokenStream {
    dispatch_const_impl(input)
}

/// Dispatches a function body based on a given generic type.
///
/// # Syntax
///
/// The syntax is effectively the same as a match statement, but the patterns are instead
/// simply comma-separated lists of types.
///
/// # Semantics
///
/// Unlike normal match statements, duplicate arms are not allowed. Only the constant
/// corresponding to the unique matching arm will be dispatched.
///
/// [`dispatch_fn!`] internally uses the `DispatchFn` trait to dispatch functions,
/// meaning that types do not have to match exactly (i.e. [`macro@impl_dispatch`] might have
/// `std::vec::Vec` while the [`dispatch_const!`] arm has `Vec`).
///
/// # Examples
///
/// ```
/// # use type_dispatch_macros::{dispatch_fn, impl_dispatch};
/// struct GenericStruct<A, B>(A, B);
///
/// // Creates 4 trait implementations.
/// #[impl_dispatch({u64; u16}, {u32, usize; u8, u64})]
/// impl<A, B, C> Into<C> for GenericStruct<A, B> {
///     fn into(self) -> C {
///        self.0 as C + self.1 as C + dispatch_fn!(
///             match A, B {
///                 u64, u32 => vec![()]: Vec<()>,
///                 u64, u8 => vec![1, 2]: Vec<u32>,
///                 u16, u32 => "ABC".to_string(): String,
///                 u16, u8 => "ABCD": &'static str,
///             }           
///         ).len() as C
///    }
/// }
///
/// let x: usize = GenericStruct(1u64, 3u32).into();
/// assert_eq!(x, 5);
/// let x: u64 = GenericStruct(1u64, 3u8).into();
/// assert_eq!(x, 6);
/// let x: usize = GenericStruct(1u16, 3u32).into();
/// assert_eq!(x, 7);
/// let x: u64 = GenericStruct(1u16, 3u8).into();
/// assert_eq!(x, 8);
/// ```
#[proc_macro]
pub fn dispatch_fn(input: TokenStream) -> TokenStream {
    dispatch_fn_impl(input)
}

/// Dispatches a constant based on a given generic type.
///
/// # Syntax
///
/// The syntax is effectively the same as a match statement, but the patterns are instead
/// simply comma-separated lists of types.
///
/// # Semantics
///
/// Unlike normal match statements, duplicate arms are not allowed. Only the constant
/// corresponding to the unique matching arm will be dispatched.
///
/// [`dispatch_type!`] matches based on the raw parsed type, meaning that types must match exactly
/// (i.e. [`macro@impl_dispatch`] cannot have `std::vec::Vec` while the [`dispatch_const!`] arm has `Vec`).
///
/// # Examples
///
/// ```
/// # use type_dispatch_macros::{dispatch_type, impl_dispatch};
/// struct GenericStruct<A, B>(A, B);
///
/// // Creates 4 trait implementations.
/// #[impl_dispatch({u64; u16}, {u32, usize; u8, u64})]
/// impl<A, B, C> Into<C> for GenericStruct<A, B> {
///     fn into(self) -> C {
///        self.0 as C + self.1 as C + dispatch_type!(
///             match A, B {
///                 u64, u32 => 0..self.1,
///                 u64, u8 => vec![self.0 + self.1 as u64],
///                 u16, u32 => "ABC".to_string(),
///                 u16, u8 => "ABCD",
///             }           
///         ).len() as C
///    }
/// }
///
/// let x: usize = GenericStruct(1u64, 3u32).into();
/// assert_eq!(x, 7);
/// let x: u64 = GenericStruct(1u64, 3u8).into();
/// assert_eq!(x, 5);
/// let x: usize = GenericStruct(1u16, 3u32).into();
/// assert_eq!(x, 7);
/// let x: u64 = GenericStruct(1u16, 3u8).into();
/// assert_eq!(x, 8);
/// ```
#[proc_macro]
pub fn dispatch_type(input: TokenStream) -> TokenStream {
    dispatch_type_impl(input)
}

pub(crate) fn type_dispatch_ident() -> TokenStream2 {
    match crate_name("type_dispatch") {
        Ok(FoundCrate::Itself) => quote!(::type_dispatch),
        Ok(FoundCrate::Name(name)) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!(::#ident)
        }
        Err(_) => match crate_name("substrate").expect("type_dispatch not found in Cargo.toml") {
            FoundCrate::Itself => quote!(::substrate::type_dispatch),
            FoundCrate::Name(name) => {
                let ident = Ident::new(&name, Span::call_site());
                quote!(::#ident::type_dispatch)
            }
        },
    }
}
