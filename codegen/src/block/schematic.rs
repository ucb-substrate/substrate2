use darling::ast::{Fields, Style};
use darling::{ast, FromDeriveInput, FromField, FromVariant};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::token::Where;
use syn::{parse_quote, GenericParam, WhereClause};

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

fn variant_decl(variant: &DataVariant) -> TokenStream {
    let DataVariant {
        ref ident,
        ref fields,
        ..
    } = variant;
    let decls = fields.iter().enumerate().map(|(i, f)| field_decl(i, f));
    match fields.style {
        Style::Unit => quote!(#ident,),
        Style::Tuple => quote!(#ident( #(#decls)* ),),
        Style::Struct => quote!(#ident { #(#decls)* },),
    }
}

fn tuple_ident(idx: usize) -> syn::Ident {
    format_ident!("__substrate_derive_field{idx}")
}

fn variant_match_arm(
    enum_ident: syn::Ident,
    variant: &DataVariant,
    val: impl Fn(&syn::Type, &TokenStream) -> TokenStream,
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
        .map(|(i, f)| field_assign(None, i, f, &val));
    match fields.style {
        Style::Unit => quote!(Self::#ident => #enum_ident::#ident,),
        Style::Tuple => {
            quote!(Self::#ident( #(#destructure),* ) => #enum_ident::#ident( #(#assign)* ),)
        }
        Style::Struct => {
            quote!(Self::#ident { #(#destructure),* } => #enum_ident::#ident { #(#assign)* },)
        }
    }
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

        let (generics_imp, generics_ty, generics_wher) = generics.split_for_impl();

        let hnv_generic_ty: syn::Ident = parse_quote!(__substrate_T);
        let hnv_generic: syn::GenericParam = parse_quote!(#hnv_generic_ty);
        let mut hnv_generics = generics.clone();
        add_trait_bounds(
            &mut hnv_generics,
            quote!(#substrate::schematic::HasNestedView<#hnv_generic>),
        );
        hnv_generics.params.push(hnv_generic.clone());

        let (hnv_imp, hnv_ty, hnv_wher) = hnv_generics.split_for_impl();

        let view_ident = format_ident!("{}View", ident);

        let generic_idents = generics
            .params
            .iter()
            .map(|generic| match generic {
                GenericParam::Lifetime(lt) => lt.lifetime.ident.clone(),
                GenericParam::Type(ty) => ty.ident.clone(),
                GenericParam::Const(c) => c.ident.clone(),
            })
            .collect::<Vec<_>>();

        let view_generic_ty: syn::Ident = parse_quote!(__substrate_V);
        let view_generic: syn::GenericParam = parse_quote!(#view_generic_ty);
        let mut view_generics = generics.clone();
        view_generics.params.push(view_generic);
        let (view_imp, view_ty, view_wher) = view_generics.split_for_impl();

        let mut save_generics = generics.clone();
        save_generics.params.push(hnv_generic);
        save_generics
            .params
            .push(parse_quote!(__substrate_S: #substrate::simulation::Simulator));
        save_generics
            .params
            .push(parse_quote!(__substrate_A: #substrate::simulation::Analysis));

        let expanded = match data {
            ast::Data::Struct(ref fields) => {
                let mut view_where_clause = view_wher.cloned().unwrap_or(WhereClause {
                    where_token: Where {
                        span: Span::call_site(),
                    },
                    predicates: Default::default(),
                });
                let mut hnv_where_clause =
                    hnv_generics.where_clause.clone().unwrap_or(WhereClause {
                        where_token: Where {
                            span: Span::call_site(),
                        },
                        predicates: Default::default(),
                    });
                let mut save_where_clause =
                    save_generics.where_clause.clone().unwrap_or(WhereClause {
                        where_token: Where {
                            span: Span::call_site(),
                        },
                        predicates: Default::default(),
                    });
                for f in fields.iter() {
                    let ty = &f.ty;
                    view_where_clause.predicates.push(
                        parse_quote!(#ty: #substrate::types::codegen::HasView<#view_generic_ty>),
                    );
                    hnv_where_clause.predicates.push(
                        parse_quote!(#ty: #substrate::schematic::HasNestedView<#hnv_generic_ty>),
                    );
                    save_where_clause.predicates.push(
                        parse_quote!(#ty: #substrate::schematic::HasNestedView<#hnv_generic_ty>),
                    );
                    save_where_clause.predicates.push(parse_quote!(<#ty as #substrate::schematic::HasNestedView<#hnv_generic_ty>>::NestedView: #substrate::simulation::data::Save<__substrate_S, __substrate_A>));
                }
                save_generics.where_clause = Some(save_where_clause.clone());
                let (save_imp, save_ty, save_wher) = save_generics.split_for_impl();

                let view_fields = fields.clone().map(|mut f| {
                    let ty = f.ty.clone();
                    f.ty = parse_quote!(<#ty as
                        #substrate::types::codegen::HasView<#view_generic_ty>>::View);
                    f
                });

                let nested_fields = fields.clone().map(|mut f| {
                    let ty = f.ty.clone();
                    f.ty = parse_quote!(<#ty as #substrate::schematic::HasNestedView<#hnv_generic_ty>>::NestedView);
                    f
                });

                let view_decls = view_fields
                    .iter()
                    .enumerate()
                    .map(|(i, f)| field_decl(i, f));
                let assignments = fields.iter().enumerate().map(|(i, f)| {
                    field_assign(
                        Some(&quote!{ self }),
                        i,
                        f,
                        |ty, val| quote! { <#ty as #substrate::schematic::HasNestedView<#hnv_generic_ty>>::nested_view(#val, __substrate_derive_parent) },
                    )
                });
                let retval = match fields.style {
                    Style::Unit => quote!(#view_ident),
                    Style::Tuple => quote!(#view_ident( #(#assignments)* )),
                    Style::Struct => quote!(#view_ident { #(#assignments)* }),
                };
                let view_body = struct_body(fields.style, true, quote! {#( #view_decls )*});

                let save_key_assignments = nested_fields.iter().enumerate().map(|(i, f)| {
                    field_assign(
                        Some(&quote!{ self }),
                        i,
                        f,
                        |ty, val| quote! { <#ty as #substrate::simulation::data::Save<__substrate_S, __substrate_A>>::save(#val, ctx, opts) },
                    )
                });
                let save_key_retval = match nested_fields.style {
                    Style::Unit => quote!(#view_ident),
                    Style::Tuple => quote!(#view_ident( #(#save_key_assignments)* )),
                    Style::Struct => quote!(#view_ident { #(#save_key_assignments)* }),
                };
                let save_assignments = nested_fields.iter().enumerate().map(|(i, f)| {
                    field_assign(
                        Some(&quote!{ key }),
                        i,
                        f,
                        |ty, val| quote! { <#ty as #substrate::simulation::data::Save<__substrate_S, __substrate_A>>::from_saved(output, #val) },
                    )
                });
                let save_retval = match nested_fields.style {
                    Style::Unit => quote!(#view_ident),
                    Style::Tuple => quote!(#view_ident( #(#save_assignments)* )),
                    Style::Struct => quote!(#view_ident { #(#save_assignments)* }),
                };

                quote! {
                    #(#attrs)*
                    #vis struct #view_ident #view_generics #view_where_clause #view_body

                    impl #hnv_imp #substrate::schematic::HasNestedView<#hnv_generic_ty> for #ident #generics_ty #hnv_where_clause {
                        type NestedView = #view_ident<#(#generic_idents,)*#substrate::types::codegen::Nested<#hnv_generic_ty>>;

                        fn nested_view(
                            &self,
                            __substrate_derive_parent: &#hnv_generic_ty,
                        ) -> Self::NestedView {
                            #retval
                        }
                    }
                    impl #save_imp #substrate::simulation::data::Save<__substrate_S, __substrate_A> for #view_ident<#(#generic_idents,)*#substrate::types::codegen::Nested<#hnv_generic_ty>> #save_wher {
                        type SaveKey = #view_ident<#(#generic_idents,)*#substrate::types::codegen::NestedSaveKey<#hnv_generic_ty, __substrate_S, __substrate_A>>;
                        type Saved = #view_ident<#(#generic_idents,)*#substrate::types::codegen::NestedSaved<#hnv_generic_ty, __substrate_S, __substrate_A>>;

                        fn save(
                            &self,
                            ctx: &#substrate::simulation::SimulationContext<__substrate_S>,
                            opts: &mut <__substrate_S as #substrate::simulation::Simulator>::Options,
                        ) -> <Self as #substrate::simulation::data::Save<__substrate_S, __substrate_A>>::SaveKey {
                            #save_key_retval
                        }

                        fn from_saved(
                            output: &<__substrate_A as #substrate::simulation::Analysis>::Output,
                            key: &<Self as #substrate::simulation::data::Save<__substrate_S, __substrate_A>>::SaveKey,
                        ) -> <Self as #substrate::simulation::data::Save<__substrate_S, __substrate_A>>::Saved {
                            #save_retval
                        }
                    }
                }
            }
            ast::Data::Enum(ref variants) => {
                unimplemented!()
            }
        };

        tokens.extend(quote! {
            #expanded
        });
    }
}
