use darling::ast::{Fields, Style};
use darling::{ast, FromDeriveInput, FromField, FromVariant};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::parse_quote;
use type_dispatch::derive::{field_tokens_with_referent, tuple_ident, FieldTokens};

use crate::substrate_ident;

#[derive(Debug, FromDeriveInput)]
#[darling(
    attributes(substrate),
    supports(struct_named, struct_newtype, struct_tuple, enum_any),
    forward_attrs(allow, doc, cfg)
)]
pub struct FromSavedInputReceiver {
    ident: syn::Ident,
    generics: syn::Generics,
    data: ast::Data<SavedVariant, SavedField>,
    vis: syn::Visibility,
    attrs: Vec<syn::Attribute>,
}

#[derive(Debug, FromVariant)]
#[darling(forward_attrs(allow, doc, cfg))]
#[allow(dead_code)]
pub struct SavedVariant {
    ident: syn::Ident,
    fields: Fields<SavedField>,
    attrs: Vec<syn::Attribute>,
}

#[derive(Debug, FromField)]
#[darling(attributes(substrate), forward_attrs(allow, doc, cfg))]
pub struct SavedField {
    ident: Option<syn::Ident>,
    vis: syn::Visibility,
    ty: syn::Type,
    attrs: Vec<syn::Attribute>,
}

fn transform_field_decl(style: Style, idx: usize, field: &SavedField) -> TokenStream {
    let SavedField {
        ref ident,
        ref vis,
        ref ty,
        ref attrs,
    } = field;
    let FieldTokens { declare, .. } = field_tokens(style, vis, attrs, idx, ident);
    let substrate = substrate_ident();
    let field_ty = quote!(<#ty as #substrate::simulation::data::FromSaved<__SUBSTRATE_SIMULATOR, __SUBSTRATE_ANALYSIS>>::SavedKey);

    quote!(#declare #field_ty,)
}

fn transform_variant_decl(variant: &SavedVariant) -> TokenStream {
    let SavedVariant {
        ref ident,
        ref fields,
        ..
    } = variant;
    let decls = fields
        .iter()
        .enumerate()
        .map(|(i, f)| transform_field_decl(fields.style, i, f));
    match fields.style {
        Style::Unit => quote!(#ident,),
        Style::Tuple => quote!(#ident( #(#decls)* ),),
        Style::Struct => quote!(#ident { #(#decls)* },),
    }
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
    field: &SavedField,
) -> TokenStream {
    let SavedField {
        ref ident,
        ref ty,
        ref vis,
        ref attrs,
        ..
    } = field;
    let substrate = substrate_ident();
    let FieldTokens {
        refer,
        assign,
        temp,
        ..
    } = field_tokens(style, vis, attrs, idx, ident);
    let refer = if as_enum { temp } else { refer };
    quote!(#assign <#ty as #substrate::simulation::data::FromSaved<__SUBSTRATE_SIMULATOR, __SUBSTRATE_ANALYSIS>>::from_saved(__substrate_derive_output, &#refer),)
}

fn transform_variant_match_arm(key_ident: syn::Ident, variant: &SavedVariant) -> TokenStream {
    let SavedVariant {
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

impl ToTokens for FromSavedInputReceiver {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let substrate = substrate_ident();
        let FromSavedInputReceiver {
            ref ident,
            ref generics,
            ref data,
            ref vis,
            ref attrs,
            ..
        } = *self;

        let mut key_generics = generics.clone();
        let (_, ty, _) = generics.split_for_impl();
        let key_ident = format_ident!("{}SavedKey", ident);
        key_generics
            .params
            .push(parse_quote!(__SUBSTRATE_SIMULATOR: #substrate::simulation::Simulator));
        key_generics
            .params
            .push(parse_quote!(__SUBSTRATE_ANALYSIS: #substrate::simulation::Analysis));

        let expanded = match data {
            ast::Data::Struct(ref fields) => {
                let decls = fields
                    .iter()
                    .enumerate()
                    .map(|(i, f)| transform_field_decl(fields.style, i, f));
                let assignments = fields
                    .iter()
                    .enumerate()
                    .map(|(i, f)| transform_field_assign(false, fields.style, i, f))
                    .collect::<Vec<_>>();
                let body = struct_body(fields.style, quote! {#( #decls )*});
                for field in fields.iter() {
                    let ty = &field.ty;
                    key_generics
                        .make_where_clause()
                        .predicates
                        .push(parse_quote!(#ty: #substrate::simulation::data::FromSaved<__SUBSTRATE_SIMULATOR, __SUBSTRATE_ANALYSIS>));
                }
                let (key_imp, key_ty, key_wher) = key_generics.split_for_impl();
                let retval = match fields.style {
                    Style::Unit => quote!(#ident),
                    Style::Tuple => quote!(#ident( #(#assignments)* )),
                    Style::Struct => quote!(#ident { #(#assignments)* }),
                };
                let struct_def = match fields.style {
                    Style::Unit => panic!("unit structs not supported"),
                    Style::Tuple => quote!(#key_generics #body #key_wher;),
                    Style::Struct => quote!(#key_generics #key_wher #body),
                };

                quote! {
                    #(#attrs)*
                    #vis struct #key_ident #struct_def

                    impl #key_imp #substrate::simulation::data::FromSaved<__SUBSTRATE_SIMULATOR, __SUBSTRATE_ANALYSIS> for #ident #ty #key_wher {
                        type SavedKey = #key_ident #key_ty;
                        fn from_saved(__substrate_derive_output: &<__SUBSTRATE_ANALYSIS as #substrate::simulation::Analysis>::Output, __substrate_derive_key: &Self::SavedKey) -> Self {
                            #retval
                        }
                    }
                }
            }
            ast::Data::Enum(ref variants) => {
                let decls = variants.iter().map(transform_variant_decl);
                let arms = variants
                    .iter()
                    .map(|v| transform_variant_match_arm(key_ident.clone(), v));
                for v in variants.iter() {
                    for field in v.fields.iter() {
                        let ty = &field.ty;
                        key_generics
                            .make_where_clause()
                            .predicates
                            .push(parse_quote!(#ty: #substrate::simulation::data::FromSaved<__SUBSTRATE_SIMULATOR, __SUBSTRATE_ANALYSIS>));
                    }
                }
                let (key_imp, key_ty, key_wher) = key_generics.split_for_impl();
                quote! {
                    #(#attrs)*
                    #vis enum #key_ident #key_generics #key_wher {
                        #( #decls )*
                    }
                    impl #key_imp #substrate::simulation::data::FromSaved<__SUBSTRATE_SIMULATOR, __SUBSTRATE_ANALYSIS> for #ident #ty #key_wher {
                        type SavedKey = #key_ident #key_ty;
                        fn from_saved(__substrate_derive_output: &<__SUBSTRATE_ANALYSIS as #substrate::simulation::Analysis>::Output, __substrate_derive_key: &Self::SavedKey) -> Self {
                            match __substrate_derive_key {
                                #(#arms)*
                            }
                        }
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
