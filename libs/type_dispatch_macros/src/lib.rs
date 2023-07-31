//! Macros for dispatching based on generic types.

#![warn(missing_docs)]

use crate::const_dispatch::dispatch_const_impl;
use proc_macro::{TokenStream, TokenTree};
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;
use syn::{bracketed, parse_macro_input};

use crate::impl_dispatch::{impl_dispatch_impl, ImplDispatchesSetBracketed};

mod const_dispatch;
mod impl_dispatch;

#[proc_macro_attribute]
pub fn impl_dispatch(args: TokenStream, input: TokenStream) -> TokenStream {
    impl_dispatch_impl(args, input)
}

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

#[proc_macro]
pub fn dispatch_const(input: TokenStream) -> TokenStream {
    dispatch_const_impl(input)
}

pub(crate) fn type_dispatch_ident() -> TokenStream2 {
    match crate_name("type_dispatch").expect("type_dispatch is present in `Cargo.toml`") {
        FoundCrate::Itself => quote!(::type_dispatch),
        FoundCrate::Name(name) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!(::#ident)
        }
    }
}
