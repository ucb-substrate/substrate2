//! Macros for the `geometry` crate.
#![warn(missing_docs)]

use darling::FromDeriveInput;
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident};
use type_dispatch::derive::{derive_trait, DeriveInputReceiver, DeriveTrait};

macro_rules! handle_error {
    ($expression:expr) => {
        match $expression {
            Ok(value) => value,
            Err(err) => {
                return err.write_errors().into();
            }
        }
    };
}

/// Derives `geometry::transform::TranslateMut`.
#[proc_macro_derive(TranslateMut)]
pub fn derive_translate_mut(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as DeriveInput);
    let receiver = handle_error!(DeriveInputReceiver::from_derive_input(&parsed));
    let geometry = geometry_ident();
    let config = DeriveTrait {
        trait_: quote!(#geometry::transform::TranslateMut),
        method: quote!(translate_mut),
        extra_arg_idents: vec![quote!(__geometry_derive_point)],
        extra_arg_tys: vec![quote!(#geometry::point::Point)],
    };

    let expanded = derive_trait(&config, receiver);
    proc_macro::TokenStream::from(expanded)
}

/// Derives `geometry::transform::TransformMut`.
#[proc_macro_derive(TransformMut)]
pub fn derive_transform_mut(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as DeriveInput);
    let receiver = handle_error!(DeriveInputReceiver::from_derive_input(&parsed));
    let geometry = geometry_ident();
    let config = DeriveTrait {
        trait_: quote!(#geometry::transform::TransformMut),
        method: quote!(transform_mut),
        extra_arg_idents: vec![quote!(__geometry_derive_transformation)],
        extra_arg_tys: vec![quote!(#geometry::transform::Transformation)],
    };

    let expanded = derive_trait(&config, receiver);
    proc_macro::TokenStream::from(expanded)
}

pub(crate) fn geometry_ident() -> TokenStream2 {
    match crate_name("geometry") {
        Ok(FoundCrate::Itself) => quote!(::geometry),
        Ok(FoundCrate::Name(name)) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!(::#ident)
        }
        Err(_) => match crate_name("substrate").expect("geometry not found in Cargo.toml") {
            FoundCrate::Itself => quote!(::substrate::geometry),
            FoundCrate::Name(name) => {
                let ident = Ident::new(&name, Span::call_site());
                quote!(::#ident::geometry)
            }
        },
    }
}
