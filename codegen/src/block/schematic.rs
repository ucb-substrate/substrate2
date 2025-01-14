use darling::ast::{Fields, Style};
use darling::{ast, FromDeriveInput, FromField, FromVariant};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::token::Where;
use syn::{parse_quote, GenericParam, WhereClause};

use crate::substrate_ident;
use macrotools::{add_trait_bounds, struct_body};

#[derive(Debug, FromDeriveInput)]
#[darling(supports(any), forward_attrs(allow, doc, cfg))]
pub struct DataInputReceiver {
    ident: syn::Ident,
    generics: syn::Generics,
    data: ast::Data<DataVariant, DataField>,
    vis: syn::Visibility,
    attrs: Vec<syn::Attribute>,
}

#[derive(Debug, Clone, FromVariant)]
#[darling(forward_attrs(allow, doc, cfg))]
#[allow(dead_code)]
pub struct DataVariant {
    ident: syn::Ident,
    fields: Fields<DataField>,
    attrs: Vec<syn::Attribute>,
}

#[derive(Debug, Clone, FromField)]
#[darling(forward_attrs(allow, doc, cfg))]
pub struct DataField {
    ident: Option<syn::Ident>,
    vis: syn::Visibility,
    ty: syn::Type,
    attrs: Vec<syn::Attribute>,
}

fn tuple_ident(idx: usize) -> syn::Ident {
    format_ident!("__substrate_derive_field{idx}")
}

fn field_decl(_idx: usize, field: &DataField) -> TokenStream {
    let DataField {
        ref ident,
        ref vis,
        ref ty,
        ref attrs,
    } = field;

    match ident {
        Some(ident) => {
            quote! {
                #(#attrs)*
                #vis #ident: #ty,
            }
        }
        None => {
            quote! {
                #(#attrs)*
                #vis #ty,
            }
        }
    }
}

fn field_assign(
    prefix: Option<&TokenStream>,
    idx: usize,
    field: &DataField,
    val: impl FnOnce(&syn::Type, &TokenStream) -> TokenStream,
) -> TokenStream {
    let DataField {
        ref ident, ref ty, ..
    } = field;
    let tuple_ident = tuple_ident(idx);
    let idx = syn::Index::from(idx);

    let refer = match (prefix, ident) {
        (Some(prefix), Some(ident)) => quote!(&#prefix.#ident),
        (Some(prefix), None) => quote!(&#prefix.#idx),
        (None, Some(ident)) => quote!(&#ident),
        (None, None) => quote!(&#tuple_ident),
    };

    let value = val(ty, &refer);

    match ident {
        Some(ident) => quote! { #ident: #value, },
        None => quote! { #value, },
    }
}
