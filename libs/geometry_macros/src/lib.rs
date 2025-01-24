//! Macros for the `geometry` crate.
#![warn(missing_docs)]

use macrotools::{
    derive_trait, handle_syn_error, DeriveInputHelper, DeriveTrait, ImplTrait, MapField, Receiver,
};
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;
use syn::{parse_macro_input, parse_quote, DeriveInput, Ident};

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

/// Derives `geometry::transform::TranslateMut`.
#[proc_macro_derive(TranslateMut)]
pub fn derive_translate_mut(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as DeriveInput);
    let geometry = geometry_ident();
    let config = DeriveTrait {
        trait_: quote!(#geometry::transform::TranslateMut),
        method: quote!(translate_mut),
        receiver: Receiver::MutRef,
        extra_arg_idents: vec![quote!(__geometry_derive_point)],
        extra_arg_tys: vec![quote!(#geometry::point::Point)],
    };

    let expanded = derive_trait(&config, &parsed);
    proc_macro::TokenStream::from(expanded)
}

pub(crate) fn impl_translate_ref(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let geometry = geometry_ident();
    let helper = DeriveInputHelper::new(input.clone())?;
    let body = helper.map_data(&parse_quote! { Self }, |MapField { ty, refer, .. }| {
        quote! { <#ty as #geometry::transform::TranslateRef>::translate_ref(#refer, __geometry_derive_point) }
    });
    Ok(helper.impl_trait(&ImplTrait {
        trait_name: quote! { #geometry::transform::TranslateRef },
        trait_body: quote! {
            fn translate_ref(&self, __geometry_derive_point: #geometry::point::Point) -> Self {
                #body
            }
        },
        extra_generics: vec![],
        extra_where_predicates: vec![],
    }))
}

/// Derives `geometry::transform::TranslateRef`.
#[proc_macro_derive(TranslateRef)]
pub fn derive_translate_ref(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as DeriveInput);
    let output = handle_syn_error!(impl_translate_ref(&parsed));
    quote!(
        #output
    )
    .into()
}

/// Derives `geometry::transform::TransformMut`.
#[proc_macro_derive(TransformMut)]
pub fn derive_transform_mut(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as DeriveInput);
    let geometry = geometry_ident();
    let config = DeriveTrait {
        trait_: quote!(#geometry::transform::TransformMut),
        method: quote!(transform_mut),
        receiver: Receiver::MutRef,
        extra_arg_idents: vec![quote!(__geometry_derive_transformation)],
        extra_arg_tys: vec![quote!(#geometry::transform::Transformation)],
    };

    let expanded = derive_trait(&config, &parsed);
    proc_macro::TokenStream::from(expanded)
}

pub(crate) fn impl_transform_ref(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let geometry = geometry_ident();
    let helper = DeriveInputHelper::new(input.clone())?;
    let body = helper.map_data(&parse_quote! { Self }, |MapField { ty, refer, .. }| {
        quote! { <#ty as #geometry::transform::TransformRef>::transform_ref(#refer, __geometry_derive_transformation) }
    });
    Ok(helper.impl_trait(&ImplTrait {
        trait_name: quote! { #geometry::transform::TransformRef },
        trait_body: quote! {
            fn transform_ref(&self, __geometry_derive_transformation: #geometry::transform::Transformation) -> Self {
                #body
            }
        },
        extra_generics: vec![],
        extra_where_predicates: vec![],
    }))
}

/// Derives `geometry::transform::TransformRef`.
#[proc_macro_derive(TransformRef)]
pub fn derive_transform_ref(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as DeriveInput);
    let output = handle_syn_error!(impl_transform_ref(&parsed));
    quote!(
        #output
    )
    .into()
}
