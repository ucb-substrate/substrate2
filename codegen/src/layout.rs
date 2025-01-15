use macrotools::{DeriveInputHelper, ImplTrait, MapField};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_quote, DeriveInput};

use crate::substrate_ident;

pub(crate) fn impl_translate_ref(view_helper: &DeriveInputHelper) -> TokenStream {
    let substrate = substrate_ident();
    let mut translate_ref_helper = view_helper.clone();
    translate_ref_helper.push_where_predicate_per_field(
        |ty, _| parse_quote! { #ty: #substrate::geometry::transform::TranslateRef },
    );

    let body = translate_ref_helper.map_data(
        &translate_ref_helper.get_type(),
            |MapField { ty, refer, .. }| {
                    quote! { <#ty as #substrate::geometry::transform::TranslateRef>::translate_ref(&#refer, __substrate_point) }
            });
    translate_ref_helper.impl_trait(&ImplTrait {
        trait_name: quote! { #substrate::geometry::transform::TranslateRef },
        trait_body: quote! {
            fn translate_ref(&self, __substrate_point: #substrate::geometry::point::Point) -> Self {
                #body
            }
        },
        extra_generics: vec![],
        extra_where_predicates: vec![],
    })
}

pub(crate) fn impl_transform_ref(view_helper: &DeriveInputHelper) -> TokenStream {
    let substrate = substrate_ident();
    let mut transform_ref_helper = view_helper.clone();
    transform_ref_helper.push_where_predicate_per_field(
        |ty, _| parse_quote! { #ty: #substrate::geometry::transform::TransformRef },
    );

    let body = transform_ref_helper.map_data(
        &transform_ref_helper.get_type(),
            |MapField { ty, refer, .. }| {
                    quote! { <#ty as #substrate::geometry::transform::TransformRef>::transform_ref(&#refer, __substrate_transformation) }
            });
    transform_ref_helper.impl_trait(&ImplTrait {
        trait_name: quote! { #substrate::geometry::transform::TransformRef },
        trait_body: quote! {
            fn transform_ref(&self, __substrate_transformation: #substrate::geometry::transform::Transformation) -> Self {
                #body
            }
        },
        extra_generics: vec![],
        extra_where_predicates: vec![],
    })
}

pub(crate) fn layout_bundle_kind(view_helper: &DeriveInputHelper) -> TokenStream {
    let mut all_decls_impls = Vec::new();

    all_decls_impls.push(impl_translate_ref(view_helper));
    all_decls_impls.push(impl_transform_ref(view_helper));

    quote! {
        #( #all_decls_impls )*
    }
}
