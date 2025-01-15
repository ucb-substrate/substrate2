use macrotools::{DeriveInputHelper, ImplTrait, MapField};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_quote, DeriveInput};

use crate::common::*;
use crate::layout::*;
use crate::schematic::*;

use crate::substrate_ident;

/// Derives `BundleKind` for the provided input and creates a struct representing views of this
/// `BundleKind`.
///
/// Implements schematic traits for the `BundleKind` by associating it with the appropriate node
/// and terminal bundle views, and implements `LayoutBundle` on the associated `PortGeometryBundle<S>` view.
///
/// If `io` is `true`, treats the input as an IO struct and creates a separate struct for the
/// `BundleKind`. In either case, implements the appropriate `Io` and `BundleKind` traits.
pub(crate) fn bundle_kind(input: &DeriveInput, io: bool) -> syn::Result<TokenStream> {
    let substrate = substrate_ident();
    let helper = DeriveInputHelper::new(input.clone())?;
    let view_ident = format_ident!("{}View", &input.ident);
    let mut all_decls_impls = Vec::new();

    // Create `BundleKind` struct and implement traits for IO struct if `io` is `true`.
    let kind_helper = if io {
        let kind_ident = format_ident!("{}Kind", &input.ident);
        let mut kind_helper = helper.clone();
        kind_helper.set_ident(kind_ident.clone());
        kind_helper.push_where_predicate_per_field(
            |ty, _| parse_quote! { #ty: #substrate::types::HasBundleKind },
        );
        kind_helper
            .map_types(|ty| parse_quote! { <#ty as #substrate::types::HasBundleKind>::BundleKind });
        let kind_type = kind_helper.get_full_type();

        all_decls_impls.push(kind_helper.decl_data());
        all_decls_impls.push(impl_flatlen(&helper));
        all_decls_impls.push(impl_flatten_direction(&helper));
        all_decls_impls.push(impl_has_bundle_kind(&helper, &kind_helper));
        // all_decls_impls.push(impl_view_source(&helper, Some(&kind_type)));

        kind_helper
    } else {
        helper.clone()
    };

    // Implement traits for `BundleKind`.
    let kind_type = kind_helper.get_full_type();
    all_decls_impls.push(impl_clone(&kind_helper));
    all_decls_impls.push(impl_debug(&kind_helper));
    all_decls_impls.push(impl_partial_eq(&kind_helper));
    all_decls_impls.push(impl_eq(&kind_helper));
    all_decls_impls.push(impl_flatlen(&kind_helper));
    all_decls_impls.push(impl_has_bundle_kind(&kind_helper, &kind_helper));
    // all_decls_impls.push(impl_view_source(&kind_helper, None));
    all_decls_impls.push(impl_has_name_tree(&kind_helper));

    // Create `View` struct
    // - Needs to add a generic along with a where clause per field that uses that generic
    // - Potentially be able to add separate where clauses to new generic
    let mut view_helper = helper.clone();
    view_helper.set_ident(view_ident);
    let view_generic_ty = quote! { SubstrateV };
    view_helper.push_generic_param(parse_quote! { #view_generic_ty });
    view_helper.push_where_predicate_per_field(
        |ty, _| parse_quote! { #ty: #substrate::types::codegen::HasView<#view_generic_ty> },
    );
    view_helper.map_types(
        |ty| parse_quote! { <#ty as #substrate::types::codegen::HasView<#view_generic_ty>>::View },
    );
    all_decls_impls.push(view_helper.decl_data());
    // all_decls_impls.push(impl_view_source(&view_helper, None));
    let mut has_bundle_kind_helper = view_helper.clone();
    has_bundle_kind_helper.push_where_predicate_per_field(|ty, prev_tys| {
        let prev_ty = &prev_tys[0];
        parse_quote!{
            #ty: #substrate::types::HasBundleKind<BundleKind = <#prev_ty as #substrate::types::HasBundleKind>::BundleKind>
        }
    });
    has_bundle_kind_helper.push_where_predicate_per_field(|_ty, prev_tys| {
        let prev_ty = &prev_tys[0];
        parse_quote! {
            #prev_ty: #substrate::types::HasBundleKind
        }
    });
    all_decls_impls.push(impl_has_bundle_kind(&has_bundle_kind_helper, &kind_helper));
    all_decls_impls.push(impl_flatlen(&view_helper));
    all_decls_impls.push(impl_flatten_generic(&view_helper));
    all_decls_impls.push(impl_unflatten(&kind_helper, &view_helper, &kind_type));

    // Implement schematic traits
    all_decls_impls.push(schematic_bundle_kind(&helper, &kind_helper, &view_helper));
    // Implement layout traits
    all_decls_impls.push(layout_bundle_kind(&view_helper));
    Ok(quote! {
        #( #all_decls_impls )*
    })
}
