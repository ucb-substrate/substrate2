use macrotools::{DeriveInputHelper, ImplTrait, MapField};
use proc_macro2::TokenStream;
use quote::quote;
use syn::parse_quote;

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

pub(crate) fn impl_has_default_layout_bundle(
    kind_helper: &DeriveInputHelper,
    view_helper: &DeriveInputHelper,
) -> TokenStream {
    let substrate = substrate_ident();
    let view_generic_ty: syn::Ident = parse_quote! { __substrate_V };
    let layer_ident: syn::Ident = parse_quote! { __substrate_L};
    let mut kind_helper = kind_helper.clone();
    kind_helper.push_where_predicate_per_field(|ty, prev_tys| {
        let prev_ty = prev_tys.first().unwrap_or(ty);
        parse_quote! { #prev_ty: #substrate::types::codegen::HasDefaultLayoutBundle }
    });
    let mut layout_bundle_helper = kind_helper.clone();
    layout_bundle_helper.set_ident(view_helper.get_ident().clone());
    layout_bundle_helper.push_generic_param(parse_quote! { #view_generic_ty });
    layout_bundle_helper.add_generic_type_binding(
        parse_quote! { #view_generic_ty },
        parse_quote! { #substrate::types::codegen::PortGeometryBundle<#layer_ident> },
    );
    layout_bundle_helper.map_types(
        |ty| parse_quote! { <#ty as #substrate::types::codegen::HasView<#substrate::types::codegen::PortGeometryBundle<#layer_ident>>>::View },
    );
    let layout_bundle_full_ty = layout_bundle_helper.get_full_type();

    kind_helper.impl_trait(&ImplTrait {
        trait_name: quote! { #substrate::types::codegen::HasDefaultLayoutBundle },
        trait_body: quote! {
            type Bundle<#layer_ident: #substrate::layout::schema::Schema> = #layout_bundle_full_ty;
        },
        extra_generics: vec![],
        extra_where_predicates: vec![],
    })
}

pub(crate) fn layout_bundle_kind(
    io_helper: &DeriveInputHelper,
    kind_helper: &DeriveInputHelper,
    view_helper: &DeriveInputHelper,
    io: bool,
) -> TokenStream {
    let mut all_decls_impls = Vec::new();

    if io {
        all_decls_impls.push(impl_has_default_layout_bundle(io_helper, view_helper));
    }
    all_decls_impls.push(impl_has_default_layout_bundle(kind_helper, view_helper));
    all_decls_impls.push(impl_translate_ref(view_helper));
    all_decls_impls.push(impl_transform_ref(view_helper));

    quote! {
        #( #all_decls_impls )*
    }
}
