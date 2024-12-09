use darling::ast::{Fields, Style};
use darling::{ast, FromDeriveInput, FromField, FromVariant};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::token::Where;
use syn::{parse_quote, Token, WhereClause};

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

        let mut hnv_generics = generics.clone();
        add_trait_bounds(
            &mut hnv_generics,
            quote!(#substrate::schematic::HasNestedView),
        );
        let (hnv_imp, hnv_ty, hnv_wher) = hnv_generics.split_for_impl();

        let mut save_generics = generics.clone();
        save_generics
            .params
            .push(parse_quote!(__substrate_S: #substrate::simulation::Simulator));
        save_generics
            .params
            .push(parse_quote!(__substrate_A: #substrate::simulation::Analysis));

        let has_nested_view_ident = format_ident!("{}NestedView", ident);
        let save_key_ident = format_ident!("{}SaveKey", ident);
        let save_ident = format_ident!("{}Save", ident);

        let expanded = match data {
            ast::Data::Struct(ref fields) => {
                let mut save_generics = save_generics.clone();
                let mut save_where_clause =
                    save_generics.where_clause.clone().unwrap_or(WhereClause {
                        where_token: Where {
                            span: Span::call_site(),
                        },
                        predicates: Default::default(),
                    });
                for f in fields.iter() {
                    let ty = &f.ty;
                    save_where_clause.predicates.push(parse_quote!(<#ty as #substrate::schematic::HasNestedView>::NestedView: #substrate::simulation::data::Save<__substrate_S, __substrate_A>));
                }
                save_generics.where_clause = Some(save_where_clause.clone());
                let (save_imp, save_ty, save_wher) = save_generics.split_for_impl();

                let nested_fields = fields.clone().map(|mut f| {
                    let ty = f.ty.clone();
                    f.ty = parse_quote!(<#ty as #substrate::schematic::HasNestedView>::NestedView);
                    f
                });

                let save_key_fields = nested_fields.clone().map(|mut f| {
                    let ty = f.ty.clone();
                    f.ty = parse_quote!(<#ty as #substrate::simulation::data::Save<__substrate_S, __substrate_A>>::SaveKey);
                    f
                });

                let save_fields = nested_fields.clone().map(|mut f| {
                    let ty = f.ty.clone();
                    f.ty = parse_quote!(<#ty as #substrate::simulation::data::Save<__substrate_S, __substrate_A>>::Save);
                    f
                });

                let nested_view_decls = nested_fields
                    .iter()
                    .enumerate()
                    .map(|(i, f)| field_decl(i, f));
                let save_key_decls = save_key_fields
                    .iter()
                    .enumerate()
                    .map(|(i, f)| field_decl(i, f));
                let save_decls = save_fields
                    .iter()
                    .enumerate()
                    .map(|(i, f)| field_decl(i, f));
                let assignments = fields.iter().enumerate().map(|(i, f)| {
                    field_assign(
                        Some(&quote!{ self }),
                        i,
                        f,
                        |ty, val| quote! { <#ty as #substrate::schematic::HasNestedView>::nested_view(#val, __substrate_derive_parent) },
                    )
                });
                let retval = match fields.style {
                    Style::Unit => quote!(#has_nested_view_ident),
                    Style::Tuple => quote!(#has_nested_view_ident( #(#assignments)* )),
                    Style::Struct => quote!(#has_nested_view_ident { #(#assignments)* }),
                };
                let nested_view_body =
                    struct_body(fields.style, true, quote! {#( #nested_view_decls )*});

                let save_key_body = struct_body(fields.style, true, quote! {#( #save_key_decls )*});
                let save_body = struct_body(fields.style, true, quote! {#( #save_decls )*});

                let nested_assignments = nested_fields.iter().enumerate().map(|(i, f)| {
                    field_assign(
                        Some(&quote!{ self }),
                        i,
                        f,
                        |ty, val| quote! { <#ty as #substrate::schematic::HasNestedView>::nested_view(#val, __substrate_derive_parent) },
                    )
                });
                let nested_retval = match nested_fields.style {
                    Style::Unit => quote!(#has_nested_view_ident),
                    Style::Tuple => quote!(#has_nested_view_ident( #(#nested_assignments)* )),
                    Style::Struct => quote!(#has_nested_view_ident { #(#nested_assignments)* }),
                };

                let save_key_assignments = nested_fields.iter().enumerate().map(|(i, f)| {
                    field_assign(
                        Some(&quote!{ self }),
                        i,
                        f,
                        |ty, val| quote! { <#ty as #substrate::simulation::data::Save<__substrate_S, __substrate_A>>::save(#val, ctx, opts) },
                    )
                });
                let save_key_retval = match nested_fields.style {
                    Style::Unit => quote!(#save_key_ident),
                    Style::Tuple => quote!(#save_key_ident( #(#save_key_assignments)* )),
                    Style::Struct => quote!(#save_key_ident { #(#save_key_assignments)* }),
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
                    Style::Unit => quote!(#save_ident),
                    Style::Tuple => quote!(#save_ident( #(#save_assignments)* )),
                    Style::Struct => quote!(#save_ident { #(#save_assignments)* }),
                };

                quote! {
                    #(#attrs)*
                    #vis struct #has_nested_view_ident #generics #nested_view_body
                    #(#attrs)*
                    #vis struct #save_key_ident #save_generics #save_where_clause #save_key_body
                    #(#attrs)*
                    #vis struct #save_ident #save_generics #save_where_clause #save_body

                    impl #hnv_imp #substrate::schematic::HasNestedView for #ident #hnv_ty #hnv_wher {
                        type NestedView = #has_nested_view_ident #hnv_ty;

                        fn nested_view(
                            &self,
                            __substrate_derive_parent: &#substrate::schematic::InstancePath,
                        ) -> Self::NestedView {
                            #retval
                        }
                    }
                    impl #hnv_imp #substrate::schematic::HasNestedView for #has_nested_view_ident #hnv_ty #hnv_wher {
                        type NestedView = #has_nested_view_ident #hnv_ty;

                        fn nested_view(
                            &self,
                            __substrate_derive_parent: &#substrate::schematic::InstancePath,
                        ) -> Self::NestedView {
                            #nested_retval
                        }
                    }
                    impl #save_imp #substrate::simulation::data::Save<__substrate_S, __substrate_A> for #has_nested_view_ident #hnv_ty #save_wher {
                        type SaveKey = #save_key_ident #save_ty;
                        type Save = #save_ident #save_ty;

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
                        ) -> <Self as #substrate::simulation::data::Save<__substrate_S, __substrate_A>>::Save {
                            #save_retval
                        }
                    }
                }
            }
            ast::Data::Enum(ref variants) => {
                let mut save_generics = save_generics.clone();
                let mut save_where_clause =
                    save_generics.where_clause.clone().unwrap_or(WhereClause {
                        where_token: Where {
                            span: Span::call_site(),
                        },
                        predicates: Default::default(),
                    });
                for v in variants.iter() {
                    for f in v.fields.iter() {
                        let ty = &f.ty;
                        save_where_clause.predicates.push(parse_quote!(<#ty as #substrate::schematic::HasNestedView>::NestedView: #substrate::simulation::data::Save<__substrate_S, __substrate_A>));
                    }
                }

                let mut nested_variants = (*variants).clone();
                nested_variants.iter_mut().for_each(|v| {
                    v.fields = v.fields.clone().map(|mut f| {
                        let ty = f.ty.clone();
                        f.ty =
                            parse_quote!(<#ty as #substrate::schematic::HasNestedView>::NestedView);
                        f
                    });
                });
                let mut save_key_variants = nested_variants.clone();
                save_key_variants.iter_mut().for_each(|v| {
                    v.fields = v.fields.clone().map(|mut f| {
                        let ty = f.ty.clone();
                        f.ty =
                            parse_quote!(<#ty as #substrate::simulation::data::Save<__substrate_S, __substrate_A>>::SaveKey);
                        f
                    });
                });
                let mut save_variants = nested_variants.clone();
                save_variants.iter_mut().for_each(|v| {
                    v.fields = v.fields.clone().map(|mut f| {
                        let ty = f.ty.clone();
                        f.ty =
                            parse_quote!(<#ty as #substrate::simulation::data::Save<__substrate_S, __substrate_A>>::Save);
                        f
                    });
                });

                let nested_view_decls = variants.iter().map(variant_decl);
                let save_key_decls = variants.iter().map(variant_decl);
                let save_decls = variants.iter().map(variant_decl);
                let arms = variants.iter().map(|v| {
                    variant_match_arm(
                        has_nested_view_ident.clone(),
                        v,
                        |ty, val| quote! { <#ty as #substrate::schematic::HasNestedView>::nested_view(#val, __substrate_derive_parent) },
                    )
                });
                let nested_arms = nested_variants.iter().map(|v| {
                    variant_match_arm(
                        has_nested_view_ident.clone(),
                        v,
                        |ty, val| quote! { <#ty as #substrate::schematic::HasNestedView>::nested_view(#val, __substrate_derive_parent) },
                    )
                });
                quote! {
                    #(#attrs)*
                    #vis enum #has_nested_view_ident #generics {
                        #( #nested_view_decls )*
                    }
                    #(#attrs)*
                    #vis enum #save_key_ident #save_generics #save_where_clause {
                        #( #save_key_decls )*
                    }
                    #(#attrs)*
                    #vis enum #save_key_ident #save_generics #save_where_clause {
                        #( #save_decls )*
                    }

                    impl #hnv_imp #substrate::schematic::HasNestedView for #ident #hnv_ty #hnv_wher {
                        type NestedView = #has_nested_view_ident #hnv_ty;

                        fn nested_view(
                            &self,
                            __substrate_derive_parent: &#substrate::schematic::InstancePath,
                        ) -> Self::NestedView {
                            match self {
                                #(#arms)*
                            }
                        }
                    }
                    impl #hnv_imp #substrate::schematic::HasNestedView for #has_nested_view_ident #hnv_ty #hnv_wher {
                        type NestedView = #has_nested_view_ident #hnv_ty;

                        fn nested_view(
                            &self,
                            __substrate_derive_parent: &#substrate::schematic::InstancePath,
                        ) -> Self::NestedView {
                            match self {
                                #(#nested_arms)*
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
