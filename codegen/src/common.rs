use macrotools::{DeriveInputHelper, ImplTrait, MapField};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_quote, DeriveInput};

use crate::substrate_ident;

pub(crate) fn impl_clone(helper: &DeriveInputHelper) -> TokenStream {
    let mut helper = helper.clone();
    helper.push_where_predicate_per_field(|ty, _| parse_quote! { #ty: ::std::clone::Clone });
    let clone_body = helper.map_data(&parse_quote! { Self }, |MapField { ty, refer, .. }| {
        quote! { <#ty as ::std::clone::Clone>::clone(#refer) }
    });
    helper.impl_trait(&ImplTrait {
        trait_name: quote! { ::std::clone::Clone },
        trait_body: quote! {
            fn clone(&self) -> Self {
                #clone_body
            }
        },
        extra_generics: vec![],
        extra_where_predicates: vec![],
    })
}

pub(crate) fn impl_debug(helper: &DeriveInputHelper) -> TokenStream {
    let mut helper = helper.clone();
    helper.push_where_predicate_per_field(|ty, _| parse_quote! { #ty: ::std::fmt::Debug });
    let ident = helper.get_ident();
    let debug_body = helper.map(|fields| {
        let mapped_fields = fields.iter().map(
            |MapField {
                 refer,
                 pretty_ident,
                 ..
             }| quote! { .field(stringify!(#pretty_ident), #refer) },
        );
        quote! { __substrate_f.debug_struct(stringify!(#ident))#(#mapped_fields)*.finish() }
    });
    helper.impl_trait(&ImplTrait {
        trait_name: quote! { ::std::fmt::Debug },
        trait_body: quote! {
            fn fmt(&self, __substrate_f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                #debug_body
            }
        },
        extra_generics: vec![],
        extra_where_predicates: vec![],
    })
}

pub(crate) fn impl_partial_eq(helper: &DeriveInputHelper) -> TokenStream {
    let mut helper = helper.clone();
    helper.push_where_predicate_per_field(|ty, _| parse_quote! { #ty: ::std::fmt::Debug });
    let partial_eq_body = helper.double_map(
        (&quote! { self }, &quote! { __substrate_other }),
        |fields: &[(&MapField, &MapField)]| {
            if fields.is_empty() {
                quote! { true }
            } else {
                let mapped_fields = fields.iter().map(
                    |(
                        MapField {
                            ty, refer: refer0, ..
                        },
                        MapField { refer: refer1, .. },
                    )| quote! { <#ty as ::std::cmp::PartialEq>::eq(#refer0, #refer1) },
                );
                quote! { #(#mapped_fields)&&* }
            }
        },
        quote! { false },
    );
    helper.impl_trait(&ImplTrait {
        trait_name: quote! { ::std::cmp::PartialEq },
        trait_body: quote! {
            fn eq(&self, __substrate_other: &Self) -> bool {
                #partial_eq_body
            }
        },
        extra_generics: vec![],
        extra_where_predicates: vec![],
    })
}

pub(crate) fn impl_eq(helper: &DeriveInputHelper) -> TokenStream {
    let mut helper = helper.clone();
    helper.push_where_predicate_per_field(|ty, _| parse_quote! { #ty: ::std::cmp::Eq });
    helper.impl_trait(&ImplTrait {
        trait_name: quote! { ::std::cmp::Eq },
        trait_body: quote! {},
        extra_generics: vec![],
        extra_where_predicates: vec![],
    })
}

pub(crate) fn impl_flatlen(helper: &DeriveInputHelper) -> TokenStream {
    let substrate = substrate_ident();
    let mut helper = helper.clone();
    helper.push_where_predicate_per_field(|ty, _| parse_quote! { #ty: #substrate::types::FlatLen });
    let flatlen_body = helper.map(|fields| {
        if fields.is_empty() {
            quote! { 0 }
        } else {
            let mapped_fields = fields
                .iter()
                .map(|MapField{ty, refer, ..}| quote! { <#ty as #substrate::types::FlatLen>::len(#refer) });
            quote! { #(#mapped_fields)+* }
        }
    });
    helper.impl_trait(&ImplTrait {
        trait_name: quote! { #substrate::types::FlatLen },
        trait_body: quote! {
            fn len(&self) -> usize {
                #flatlen_body
            }
        },
        extra_generics: vec![],
        extra_where_predicates: vec![],
    })
}

pub(crate) fn impl_flatten_direction(helper: &DeriveInputHelper) -> TokenStream {
    let substrate = substrate_ident();
    let mut helper = helper.clone();
    helper.push_where_predicate_per_field(
        |ty, _| parse_quote! { #ty: #substrate::types::Flatten<#substrate::types::Direction> },
    );
    let flatten_direction_body = helper.map(
            |fields| {
                let mapped_fields = fields.iter().map(|MapField { ty, refer, .. }| quote! { <#ty as #substrate::types::Flatten<#substrate::types::Direction>>::flatten(#refer, __substrate_output_sink); });
                quote! { #(#mapped_fields)* }
            },
        );
    helper.impl_trait(&ImplTrait {
        trait_name: quote! { #substrate::types::Flatten<#substrate::types::Direction> },
        trait_body: quote! {
            fn flatten<E>(&self, __substrate_output_sink: &mut E)
            where
                E: ::std::iter::Extend<#substrate::types::Direction> {
                #flatten_direction_body
            }
        },
        extra_generics: vec![],
        extra_where_predicates: vec![],
    })
}

pub(crate) fn impl_has_name_tree(helper: &DeriveInputHelper) -> TokenStream {
    let substrate = substrate_ident();
    let mut helper = helper.clone();
    helper.push_where_predicate_per_field(
        |ty, _| parse_quote! { #ty: #substrate::types::HasBundleKind },
    );
    let name_fields = helper.map(|fields| {
        let mapped_fields = fields.iter().map(|MapField{ ty, pretty_ident, refer, .. }|
            quote! {
                (#substrate::arcstr::literal!(::std::stringify!(#pretty_ident)), <#ty as #substrate::types::HasNameTree>::names(&#refer))
            }
        );
        quote! { vec![ #(#mapped_fields),* ] }
    });
    helper.impl_trait(&ImplTrait {
        trait_name: quote! { #substrate::types::HasNameTree },
        trait_body: quote! {
            fn names(&self) -> ::std::option::Option<::std::vec::Vec<#substrate::types::NameTree>> {
                let v: ::std::vec::Vec<#substrate::types::NameTree> =  #name_fields
                     .into_iter()
                     .filter_map(|(frag, children)| children.map(|c| #substrate::types::NameTree::new(frag, c)))
                     .collect();
                if v.len() == 0 { None } else { Some(v) }
            }
        },
        extra_generics: vec![],
        extra_where_predicates: vec![],
    })
}

pub(crate) fn impl_flatten_generic(helper: &DeriveInputHelper) -> TokenStream {
    let substrate = substrate_ident();
    let flatten_generic = parse_quote! { SubstrateF };
    let mut helper = helper.clone();
    helper.push_where_predicate_per_field(
        |ty, _| parse_quote! { #ty: #substrate::types::Flatten<#flatten_generic> },
    );
    let flatten_body = helper.map(
            |fields| {
                let mapped_fields = fields.iter().map(|MapField { ty, refer, .. }| quote! { <#ty as #substrate::types::Flatten<#flatten_generic>>::flatten(#refer, __substrate_output_sink); });
                quote! { #(#mapped_fields)* }
            },
        );
    helper.impl_trait(&ImplTrait {
        trait_name: quote! { #substrate::types::Flatten<#flatten_generic> },
        trait_body: quote! {
            fn flatten<E>(&self, __substrate_output_sink: &mut E)
            where
                E: ::std::iter::Extend<#flatten_generic> {
                #flatten_body
            }
        },
        extra_generics: vec![flatten_generic],
        extra_where_predicates: vec![],
    })
}

pub(crate) fn impl_unflatten(
    kind_helper: &DeriveInputHelper,
    view_helper: &DeriveInputHelper,
    bundle_kind: &syn::Type,
) -> TokenStream {
    let substrate = substrate_ident();
    let unflatten_generic = parse_quote! { SubstrateS };
    let mut kind_helper = kind_helper.clone();
    let mut view_helper = view_helper.clone();
    view_helper.push_where_predicate_per_field(|_ty, prev_tys| {
        let prev_ty = &prev_tys[0];
        parse_quote! { #prev_ty: #substrate::types::HasBundleKind }
    });
    view_helper.push_where_predicate_per_field(
        |ty, prev_tys| {
            let prev_ty = &prev_tys[0];
            parse_quote! { #ty: #substrate::types::Unflatten<<#prev_ty as #substrate::types::HasBundleKind>::BundleKind, #unflatten_generic> }
        },
    );
    let unflatten_referent = quote! { __substrate_data };
    kind_helper.set_referent(unflatten_referent.clone());
    let unflatten_body = kind_helper.map_data(
        &view_helper.get_type(),
            |MapField { ty, refer, prev_tys, .. }| {
                    let root_ty = if let Some(prev_ty) = prev_tys.first() {
                        prev_ty
                    } else { ty };
                    quote! { <<#root_ty as #substrate::types::codegen::HasView<SubstrateV>>::View as #substrate::types::Unflatten<#ty, #unflatten_generic>>::unflatten(&#refer, __substrate_source)? }
            });
    view_helper.impl_trait(&ImplTrait {
        trait_name: quote! { #substrate::types::Unflatten<#bundle_kind, #unflatten_generic> },
        trait_body: quote! {
            fn unflatten<SubstrateI>(#unflatten_referent: &#bundle_kind, __substrate_source: &mut SubstrateI) -> Option<Self>
            where
                SubstrateI: Iterator<Item = #unflatten_generic> {
                ::std::option::Option::Some(#unflatten_body)
            }
        },
        extra_generics: vec![unflatten_generic],
        extra_where_predicates: vec![],
    })
}
