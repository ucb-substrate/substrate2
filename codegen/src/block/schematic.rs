use darling::ast::{Fields, Style};
use darling::{ast, FromDeriveInput, FromField, FromVariant};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use crate::substrate_ident;
use type_dispatch::derive::{add_trait_bounds, struct_body};

#[derive(Debug, FromDeriveInput)]
#[darling(supports(any), forward_attrs(allow, doc, cfg))]
pub struct DataInputReceiver {
    ident: syn::Ident,
    generics: syn::Generics,
    data: ast::Data<DataVariant, DataField>,
    vis: syn::Visibility,
    attrs: Vec<syn::Attribute>,
}

#[derive(Debug, FromVariant)]
#[darling(forward_attrs(allow, doc, cfg))]
#[allow(dead_code)]
pub struct DataVariant {
    ident: syn::Ident,
    fields: Fields<DataField>,
    attrs: Vec<syn::Attribute>,
}

#[derive(Debug, FromField)]
#[darling(forward_attrs(allow, doc, cfg))]
pub struct DataField {
    ident: Option<syn::Ident>,
    vis: syn::Visibility,
    ty: syn::Type,
    attrs: Vec<syn::Attribute>,
}

fn transform_variant_decl(variant: &DataVariant) -> TokenStream {
    let DataVariant {
        ref ident,
        ref fields,
        ..
    } = variant;
    let decls = fields
        .iter()
        .enumerate()
        .map(|(i, f)| transform_field_decl(i, f));
    match fields.style {
        Style::Unit => quote!(#ident,),
        Style::Tuple => quote!(#ident( #(#decls)* ),),
        Style::Struct => quote!(#ident { #(#decls)* },),
    }
}

fn tuple_ident(idx: usize) -> syn::Ident {
    format_ident!("__substrate_derive_field{idx}")
}

fn transform_variant_match_arm(
    transformed_ident: syn::Ident,
    variant: &DataVariant,
) -> TokenStream {
    let DataVariant {
        ref ident,
        ref fields,
        ..
    } = variant;
    let destructure = fields
        .iter()
        .enumerate()
        .map(|(i, f)| f.ident.clone().unwrap_or_else(|| tuple_ident(i)))
        .map(|i| quote!(ref #i));
    let assign = fields
        .iter()
        .enumerate()
        .map(|(i, f)| transform_field_assign(false, i, f));
    match fields.style {
        Style::Unit => quote!(Self::#ident => #transformed_ident::#ident,),
        Style::Tuple => {
            quote!(Self::#ident( #(#destructure),* ) => #transformed_ident::#ident( #(#assign)* ),)
        }
        Style::Struct => {
            quote!(Self::#ident { #(#destructure),* } => #transformed_ident::#ident { #(#assign)* },)
        }
    }
}

fn transform_field_decl(_idx: usize, field: &DataField) -> TokenStream {
    let DataField {
        ref ident,
        ref vis,
        ref ty,
        ref attrs,
    } = field;
    let substrate = substrate_ident();
    let field_ty = quote!(#substrate::schematic::NestedView<#ty>);

    match ident {
        Some(ident) => {
            quote! {
                #(#attrs)*
                #vis #ident: #field_ty,
            }
        }
        None => {
            quote! {
                #(#attrs)*
                #vis #field_ty,
            }
        }
    }
}

fn transform_field_assign(use_self: bool, idx: usize, field: &DataField) -> TokenStream {
    let DataField {
        ref ident, ref ty, ..
    } = field;
    let substrate = substrate_ident();
    let tuple_ident = tuple_ident(idx);
    let idx = syn::Index::from(idx);

    let val = match (use_self, ident) {
        (true, Some(ident)) => quote!(&self.#ident),
        (true, None) => quote!(&self.#idx),
        (false, Some(ident)) => quote!(&#ident),
        (false, None) => quote!(&#tuple_ident),
    };

    let value = quote!(<#ty as #substrate::schematic::HasNestedView>::nested_view(#val, __substrate_derive_parent));

    match ident {
        Some(ident) => quote! { #ident: #value, },
        None => quote! { #value, },
    }
}

impl ToTokens for DataInputReceiver {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let substrate = substrate_ident();
        let DataInputReceiver {
            ref ident,
            ref generics,
            ref data,
            ref vis,
            ref attrs,
        } = *self;

        let mut generics = generics.clone();
        add_trait_bounds(&mut generics, quote!(#substrate::schematic::HasNestedView));

        let (imp, ty, wher) = generics.split_for_impl();
        let transformed_ident = format_ident!("{}NestedView", ident);

        let expanded = match data {
            ast::Data::Struct(ref fields) => {
                let decls = fields
                    .iter()
                    .enumerate()
                    .map(|(i, f)| transform_field_decl(i, f));
                let assignments = fields
                    .iter()
                    .enumerate()
                    .map(|(i, f)| transform_field_assign(true, i, f));
                let retval = match fields.style {
                    Style::Unit => quote!(#transformed_ident),
                    Style::Tuple => quote!(#transformed_ident( #(#assignments)* )),
                    Style::Struct => quote!(#transformed_ident { #(#assignments)* }),
                };
                let body = struct_body(fields.style, true, quote! {#( #decls )*});

                quote! {
                    #(#attrs)*
                    #vis struct #transformed_ident #generics #body

                    impl #imp #substrate::schematic::HasNestedView for #ident #ty #wher {
                        type NestedView = #transformed_ident #ty;

                        fn nested_view(
                            &self,
                            __substrate_derive_parent: &#substrate::schematic::InstancePath,
                        ) -> Self::NestedView {
                            #retval
                        }
                    }
                }
            }
            ast::Data::Enum(ref variants) => {
                let decls = variants.iter().map(transform_variant_decl);
                let arms = variants
                    .iter()
                    .map(|v| transform_variant_match_arm(transformed_ident.clone(), v));
                quote! {
                    #(#attrs)*
                    #vis enum #transformed_ident #generics {
                        #( #decls )*
                    }
                    impl #imp #substrate::schematic::HasNestedView for #ident #ty #wher {
                        type NestedView = #transformed_ident #ty;

                        fn nested_view(
                            &self,
                            __substrate_derive_parent: &#substrate::schematic::InstancePath,
                        ) -> Self::NestedView {
                            match self {
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
