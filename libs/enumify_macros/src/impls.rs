//! Macros for the `geometry` crate.
#![warn(missing_docs)]

use convert_case::{Case, Casing};
use darling::ast::{self, Fields, NestedMeta, Style};
use darling::FromDeriveInput;
use darling::{FromField, FromVariant};
use proc_macro2::{Span, TokenStream};
use proc_macro_crate::{crate_name, FoundCrate};
use proc_macro_error::proc_macro_error;
use quote::format_ident;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident};
use type_dispatch::derive::{
    derive_trait, field_tokens_with_referent, tuple_ident, DeriveInputReceiver, DeriveTrait,
    FieldTokens,
};

#[derive(Debug, FromDeriveInput)]
#[darling(
    attributes(substrate),
    supports(enum_any),
    forward_attrs(allow, doc, cfg)
)]
pub(crate) struct EnumifyInputReceiver {
    ident: syn::Ident,
    generics: syn::Generics,
    data: darling::ast::Data<EnumifyVariant, ()>,
    vis: syn::Visibility,
    attrs: Vec<syn::Attribute>,
}

#[derive(Debug, FromVariant)]
#[darling(forward_attrs(allow, doc, cfg))]
#[allow(dead_code)]
pub(crate) struct EnumifyVariant {
    ident: syn::Ident,
    fields: Fields<EnumifyField>,
    attrs: Vec<syn::Attribute>,
}

#[derive(Debug, FromField)]
#[darling(attributes(substrate), forward_attrs(allow, doc, cfg))]
pub(crate) struct EnumifyField {
    ident: Option<syn::Ident>,
    vis: syn::Visibility,
    ty: syn::Type,
    attrs: Vec<syn::Attribute>,
}

pub(crate) fn field_tokens(
    style: Style,
    vis: &syn::Visibility,
    attrs: &Vec<syn::Attribute>,
    idx: usize,
    ident: &Option<syn::Ident>,
) -> FieldTokens {
    field_tokens_with_referent(
        style,
        vis,
        attrs,
        idx,
        ident,
        syn::parse_quote!(__substrate_derive_key),
    )
}

fn transform_field_assign(
    as_enum: bool,
    style: Style,
    idx: usize,
    field: &EnumifyField,
) -> TokenStream {
    let EnumifyField {
        ref ident,
        ref ty,
        ref vis,
        ref attrs,
        ..
    } = field;
    let FieldTokens {
        refer,
        assign,
        temp,
        ..
    } = field_tokens(style, vis, attrs, idx, ident);
    let refer = if as_enum { temp } else { refer };
    quote!(#assign 1)
}

fn transform_variant_match_arm(key_ident: syn::Ident, variant: &EnumifyVariant) -> TokenStream {
    let EnumifyVariant {
        ref ident,
        ref fields,
        ..
    } = variant;
    let destructure = fields
        .iter()
        .enumerate()
        .map(|(i, f)| f.ident.clone().unwrap_or_else(|| tuple_ident(i)))
        .map(|i| quote!(#i));
    let assign = fields
        .iter()
        .enumerate()
        .map(|(i, f)| transform_field_assign(true, fields.style, i, f));
    match fields.style {
        Style::Unit => quote!(#key_ident::#ident => Self::#ident,),
        Style::Tuple => {
            quote!(#key_ident::#ident( #(#destructure),* ) => Self::#ident( #(#assign)* ),)
        }
        Style::Struct => {
            quote!(#key_ident::#ident { #(#destructure),* } => Self::#ident { #(#assign)* },)
        }
    }
}

fn unwrap_variant(variant: &EnumifyVariant) -> Option<TokenStream> {
    if variant.fields.style != Style::Tuple || variant.fields.fields.len() != 1 {
        return None;
    }

    let name = syn::Ident::new(
        &variant.ident.to_string().to_case(Case::Snake),
        variant.ident.span(),
    );
    let field = variant.fields.iter().next()?;
    let method_name = format_ident!("unwrap_{}", name);

    let ident = &variant.ident;
    let ty = &field.ty;

    Some(quote! {
        pub fn #method_name(self) -> #ty {
            match self {
                Self::#ident(x) => x,
                _ => panic!("cannot unwrap"),
            }
        }
    })
}

impl EnumifyInputReceiver {
    pub(crate) fn expand(&self, tokens: &mut TokenStream) {
        let EnumifyInputReceiver {
            ref ident,
            ref generics,
            ref data,
            ref vis,
            ref attrs,
            ..
        } = *self;

        let (imp, ty, wher) = generics.split_for_impl();
        let expanded = match data {
            ast::Data::Struct(ref fields) => {
                panic!("enumify does not support structs");
            }
            ast::Data::Enum(ref variants) => {
                let unwraps = variants.iter().filter_map(unwrap_variant);
                quote! {
                    impl #imp #ident #ty #wher {
                        #(#unwraps)*
                    }
                }
            }
        };

        tokens.extend(quote! {
            #expanded
        });
    }
}

fn struct_body(style: Style, contents: TokenStream) -> TokenStream {
    match style {
        Style::Unit => quote!(),
        Style::Tuple => quote!( ( #contents ) ),
        Style::Struct => quote!( { #contents } ),
    }
}
