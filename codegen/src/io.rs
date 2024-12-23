use darling::{ast, FromDeriveInput, FromField};
use macrotools::{
    add_trait_bounds, field_tokens, field_tokens_with_referent, struct_body, DeriveInputHelper,
    FieldTokens, ImplTrait, MapField,
};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{parse_quote, token::Where, DeriveInput, GenericParam, Ident, WhereClause};

use crate::substrate_ident;

fn impl_clone(helper: &DeriveInputHelper) -> TokenStream {
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

fn impl_debug(helper: &DeriveInputHelper) -> TokenStream {
    let mut helper = helper.clone();
    helper.push_where_predicate_per_field(|ty, _| parse_quote! { #ty: ::std::fmt::Debug });
    let ident = helper.get_ident();
    let debug_body = helper.map(|fields| {
        let mapped_fields = fields.iter().map(
            |MapField {
                 ty,
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

fn impl_partial_eq(helper: &DeriveInputHelper) -> TokenStream {
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

fn impl_eq(helper: &DeriveInputHelper) -> TokenStream {
    let mut helper = helper.clone();
    helper.push_where_predicate_per_field(|ty, _| parse_quote! { #ty: ::std::cmp::Eq });
    helper.impl_trait(&ImplTrait {
        trait_name: quote! { ::std::cmp::Eq },
        trait_body: quote! {},
        extra_generics: vec![],
        extra_where_predicates: vec![],
    })
}

fn impl_flatlen(helper: &DeriveInputHelper) -> TokenStream {
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

fn impl_flatten_direction(helper: &DeriveInputHelper) -> TokenStream {
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

fn impl_has_bundle_kind(
    helper: &DeriveInputHelper,
    kind_helper: &DeriveInputHelper,
) -> TokenStream {
    let substrate = substrate_ident();
    let mut helper = helper.clone();
    helper.push_where_predicate_per_field(
        |ty, _| parse_quote! { #ty: #substrate::types::HasBundleKind },
    );
    let bundle_kind = kind_helper.get_type();
    let kind_body = helper.map_data(
        &parse_quote! { #bundle_kind },
        |MapField { ty, refer, .. }| {
            quote! { <#ty as #substrate::types::HasBundleKind>::kind(#refer) }
        },
    );
    let bundle_kind_full = kind_helper.get_full_type();
    helper.impl_trait(&ImplTrait {
        trait_name: quote! { #substrate::types::HasBundleKind },
        trait_body: quote! {
            type BundleKind = #bundle_kind_full;

            fn kind(&self) -> <Self as #substrate::types::HasBundleKind>::BundleKind {
                #kind_body
            }
        },
        extra_generics: vec![],
        extra_where_predicates: vec![],
    })
}

fn impl_view_source(helper: &DeriveInputHelper, other: Option<&syn::Type>) -> TokenStream {
    let substrate = substrate_ident();
    let mut helper = helper.clone();
    helper.push_where_predicate_per_field(
        |ty, _| parse_quote! { #ty: #substrate::types::HasBundleKind },
    );
    let view_source_body = if let Some(other) = other {
        quote! {
            type Kind = #substrate::types::codegen::FromOther;
            type Source = #other;
        }
    } else {
        quote! {
            type Kind = #substrate::types::codegen::FromSelf;
            type Source = Self;
        }
    };
    helper.impl_trait(&ImplTrait {
        trait_name: quote! { #substrate::types::codegen::ViewSource },
        trait_body: view_source_body,
        extra_generics: vec![],
        extra_where_predicates: vec![],
    })
}

fn impl_has_name_tree(helper: &DeriveInputHelper) -> TokenStream {
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

fn impl_flatten_generic(helper: &DeriveInputHelper) -> TokenStream {
    let substrate = substrate_ident();
    let flatten_generic = parse_quote! { __substrate_F };
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

fn impl_unflatten(
    kind_helper: &DeriveInputHelper,
    view_helper: &DeriveInputHelper,
    bundle_kind: &syn::Type,
) -> TokenStream {
    let substrate = substrate_ident();
    let unflatten_generic = parse_quote! { __substrate_S };
    let mut kind_helper = kind_helper.clone();
    let mut view_helper = view_helper.clone();
    view_helper.push_where_predicate_per_field(|ty, prev_tys| {
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
                    quote! { <<#root_ty as #substrate::types::codegen::HasView<__substrate_V>>::View as #substrate::types::Unflatten<#ty, #unflatten_generic>>::unflatten(&#refer, __substrate_source)? }
            });
    view_helper.impl_trait(&ImplTrait {
        trait_name: quote! { #substrate::types::Unflatten<#bundle_kind, #unflatten_generic> },
        trait_body: quote! {
            fn unflatten<__substrate_I>(#unflatten_referent: &#bundle_kind, __substrate_source: &mut __substrate_I) -> Option<Self>
            where
                __substrate_I: Iterator<Item = #unflatten_generic> {
                ::std::option::Option::Some(#unflatten_body)
            }
        },
        extra_generics: vec![unflatten_generic],
        extra_where_predicates: vec![],
    })
}

fn impl_schematic_bundle_kind(
    kind_helper: &DeriveInputHelper,
    node_bundle_helper: &DeriveInputHelper,
    terminal_bundle_helper: &DeriveInputHelper,
) -> TokenStream {
    let substrate = substrate_ident();
    let mut schematic_bundle_kind_helper = kind_helper.clone();
    schematic_bundle_kind_helper.push_where_predicate_per_field(|ty, prev_tys| {
        let prev_ty = &prev_tys[0];
        parse_quote! { #prev_ty: #substrate::types::codegen::HasSchematicBundleKindViews }
    });

    let node_bundle_full_ty = node_bundle_helper.get_full_type();
    let terminal_bundle_full_ty = terminal_bundle_helper.get_full_type();

    let terminal_view_body = node_bundle_helper.double_map_data(
        &terminal_bundle_helper.get_type(),
        (&quote!{ cell_io }, &quote!{ instance_io }),
            |MapField { refer: refer0, prev_tys, .. }, MapField { refer: refer1, .. }| {
                let prev_ty = &prev_tys[0];
                quote!{<<#prev_ty as #substrate::types::HasBundleKind>::BundleKind as #substrate::types::schematic::SchematicBundleKind>::terminal_view(cell, #refer0, instance, #refer1)}
            }, quote!{ panic!("cell and instance IOs are not the same kind") });

    schematic_bundle_kind_helper.impl_trait(&ImplTrait {
        trait_name: quote! { #substrate::types::schematic::SchematicBundleKind },
        trait_body: quote! {
            type NodeBundle = #node_bundle_full_ty;
            type TerminalBundle = #terminal_bundle_full_ty;
            fn terminal_view(
                cell: #substrate::schematic::CellId,
                cell_io: &#substrate::types::schematic::NodeBundle<Self>,
                instance: #substrate::schematic::InstanceId,
                instance_io: &#substrate::types::schematic::NodeBundle<Self>,
            ) -> #substrate::types::schematic::TerminalBundle<Self> {
                #terminal_view_body
            }
        },
        extra_generics: vec![],
        extra_where_predicates: vec![],
    })
}

fn impl_has_nested_view(
    view_helper: &DeriveInputHelper,
    nested_view_helper: &DeriveInputHelper,
) -> TokenStream {
    let substrate = substrate_ident();
    let mut has_nested_view_helper = view_helper.clone();
    has_nested_view_helper.push_where_predicate_per_field(
        |ty, _| parse_quote! { #ty: #substrate::schematic ::HasNestedView },
    );

    let nested_view_full_ty = nested_view_helper.get_full_type();

    let nested_view_body = has_nested_view_helper.map_data(
        &nested_view_helper.get_type(),
            |MapField { ty, refer, .. }| {
                    quote! { <#ty as #substrate::schematic::HasNestedView>::nested_view(&#refer, __substrate_parent) }
            });
    has_nested_view_helper.impl_trait(&ImplTrait {
        trait_name: quote! { #substrate::schematic::HasNestedView },
        trait_body: quote! {
            type NestedView = #nested_view_full_ty;

            fn nested_view(&self, __substrate_parent: &#substrate::schematic::InstancePath) -> #substrate::schematic::NestedView<Self> {
                #nested_view_body
            }
        },
        extra_generics: vec![],
        extra_where_predicates: vec![],
    })
}

fn impl_view_as(view_helper: &DeriveInputHelper, nodes: bool) -> TokenStream {
    let substrate = substrate_ident();
    let mut view_as_helper = view_helper.clone();

    let (bundle_view_ident, view_as_fn) = if nodes {
        (quote! { NodeBundle }, quote! { view_nodes_as })
    } else {
        (quote! { TerminalBundle }, quote! { view_terminals_as })
    };
    view_as_helper.push_where_predicate(parse_quote!{ Self: #substrate::types::HasBundleKind<BundleKind: #substrate::types::schematic::SchematicBundleKind<#bundle_view_ident = Self>> });

    let full_ty = view_as_helper.get_full_type();
    let (imp, _, wher) = view_as_helper.custom_split_for_impl();
    let vis = &view_as_helper.get_input().vis;

    quote! {
        impl #imp #full_ty #wher {
            /// Views this node bundle as a node bundle of a different kind.
            #vis fn view_as<__substrate_T: #substrate::types::HasBundleKind<BundleKind: #substrate::types::schematic::SchematicBundleKind>>(&self) -> #substrate::types::schematic::#bundle_view_ident<<__substrate_T as #substrate::types::HasBundleKind>::BundleKind> where <Self as #substrate::types::HasBundleKind>::BundleKind: #substrate::types::schematic::DataView<<__substrate_T as #substrate::types::HasBundleKind>::BundleKind>{
                <<Self as #substrate::types::HasBundleKind>::BundleKind as #substrate::types::schematic::DataView<<__substrate_T as #substrate::types::HasBundleKind>::BundleKind>>::#view_as_fn(self)
            }
        }
    }
}

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
        all_decls_impls.push(impl_view_source(&helper, Some(&kind_type)));

        kind_helper
    } else {
        helper.clone()
    };

    // Implement traits for `BundleKind`.
    let kind_ident = kind_helper.get_ident();
    let kind_type = kind_helper.get_full_type();
    all_decls_impls.push(impl_clone(&kind_helper));
    all_decls_impls.push(impl_debug(&kind_helper));
    all_decls_impls.push(impl_partial_eq(&kind_helper));
    all_decls_impls.push(impl_eq(&kind_helper));
    all_decls_impls.push(impl_flatlen(&kind_helper));
    all_decls_impls.push(impl_has_bundle_kind(&kind_helper, &kind_helper));
    all_decls_impls.push(impl_view_source(&kind_helper, None));
    all_decls_impls.push(impl_has_name_tree(&kind_helper));

    // Create `View` struct
    // - Needs to add a generic along with a where clause per field that uses that generic
    // - Potentially be able to add separate where clauses to new generic
    let mut view_helper = helper.clone();
    view_helper.set_ident(view_ident);
    let view_generic_ty = quote! { __substrate_V };
    view_helper.push_generic_param(parse_quote! { #view_generic_ty });
    view_helper.push_where_predicate_per_field(
        |ty, _| parse_quote! { #ty: #substrate::types::codegen::HasView<#view_generic_ty> },
    );
    view_helper.map_types(
        |ty| parse_quote! { <#ty as #substrate::types::codegen::HasView<#view_generic_ty>>::View },
    );
    all_decls_impls.push(view_helper.decl_data());
    all_decls_impls.push(impl_view_source(&view_helper, None));
    let mut has_bundle_kind_helper = view_helper.clone();
    has_bundle_kind_helper.push_where_predicate_per_field(|ty, prev_tys| {
        let prev_ty = &prev_tys[0];
        parse_quote!{
            #ty: #substrate::types::HasBundleKind<BundleKind = <#prev_ty as #substrate::types::HasBundleKind>::BundleKind>
        }
    });
    has_bundle_kind_helper.push_where_predicate_per_field(|ty, prev_tys| {
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
    //
    Ok(quote! {
        #( #all_decls_impls )*
    })
}

pub(crate) fn schematic_bundle_kind(
    original_helper: &DeriveInputHelper,
    kind_helper: &DeriveInputHelper,
    view_helper: &DeriveInputHelper,
) -> TokenStream {
    let substrate = substrate_ident();
    let mut all_decls_impls = Vec::new();
    let view_generic_ty = quote! { __substrate_V };

    let mut node_bundle_helper = original_helper.clone();
    node_bundle_helper.set_ident(view_helper.get_ident().clone());
    node_bundle_helper.push_generic_param(parse_quote! { #view_generic_ty });
    node_bundle_helper.push_where_predicate_per_field(
        |ty, _| parse_quote! { #ty: #substrate::types::codegen::HasSchematicBundleKindViews },
    );

    let mut terminal_bundle_helper = node_bundle_helper.clone();
    let mut nested_node_bundle_helper = node_bundle_helper.clone();
    let mut nested_terminal_bundle_helper = node_bundle_helper.clone();

    node_bundle_helper.add_generic_type_binding(
        parse_quote! { #view_generic_ty },
        parse_quote! { #substrate::types::codegen::NodeBundle },
    );
    node_bundle_helper.map_types(
        |ty| parse_quote! { <#ty as #substrate::types::codegen::HasView<#substrate::types::codegen::NodeBundle>>::View },
    );

    terminal_bundle_helper.add_generic_type_binding(
        parse_quote! { #view_generic_ty },
        parse_quote! { #substrate::types::codegen::TerminalBundle },
    );
    terminal_bundle_helper.map_types(
        |ty| parse_quote! { <#ty as #substrate::types::codegen::HasView<#substrate::types::codegen::TerminalBundle>>::View },
    );

    nested_node_bundle_helper.add_generic_type_binding(
        parse_quote! { #view_generic_ty },
        parse_quote! { #substrate::types::codegen::NestedNodeBundle },
    );
    nested_node_bundle_helper.map_types(
        |ty| parse_quote! { <#ty as #substrate::types::codegen::HasView<#substrate::types::codegen::NestedNodeBundle>>::View },
    );
    nested_node_bundle_helper.push_where_predicate_per_field(
        |ty, _| parse_quote! { #ty: #substrate::schematic::HasNestedView<NestedView = #ty> },
    );

    nested_terminal_bundle_helper.add_generic_type_binding(
        parse_quote! { #view_generic_ty },
        parse_quote! { #substrate::types::codegen::NestedTerminalBundle },
    );
    nested_terminal_bundle_helper.map_types(
        |ty| parse_quote! { <#ty as #substrate::types::codegen::HasView<#substrate::types::codegen::NestedTerminalBundle>>::View },
    );
    nested_terminal_bundle_helper.push_where_predicate_per_field(
        |ty, _| parse_quote! { #ty: #substrate::schematic::HasNestedView<NestedView = #ty>},
    );

    all_decls_impls.push(impl_schematic_bundle_kind(
        kind_helper,
        &node_bundle_helper,
        &terminal_bundle_helper,
    ));
    all_decls_impls.push(impl_has_nested_view(
        &node_bundle_helper,
        &nested_node_bundle_helper,
    ));
    all_decls_impls.push(impl_has_nested_view(
        &terminal_bundle_helper,
        &nested_terminal_bundle_helper,
    ));
    all_decls_impls.push(impl_has_nested_view(
        &nested_node_bundle_helper,
        &nested_node_bundle_helper,
    ));
    all_decls_impls.push(impl_has_nested_view(
        &nested_terminal_bundle_helper,
        &nested_terminal_bundle_helper,
    ));
    all_decls_impls.push(impl_view_as(&node_bundle_helper, true));
    all_decls_impls.push(impl_view_as(&terminal_bundle_helper, false));

    quote! {
        #( #all_decls_impls )*
    }
}
//
// // TODO: Signature might need to be modified to use macrotools.
// pub(crate) fn layout_bundle(input: &DeriveInput) -> TokenStream {}
