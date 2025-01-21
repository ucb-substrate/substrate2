use macrotools::{DeriveInputHelper, ImplTrait, MapField};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::GenericParam;
use syn::{parse_quote, DeriveInput};

use crate::common::*;
use crate::substrate_ident;

pub(crate) fn impl_has_bundle_kind(
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

pub(crate) fn impl_schematic_bundle_kind(
    kind_helper: &DeriveInputHelper,
    terminal_bundle_helper: &DeriveInputHelper,
    node_bundle_helper: &DeriveInputHelper,
    io: bool,
) -> TokenStream {
    let substrate = substrate_ident();
    let mut schematic_bundle_kind_helper = kind_helper.clone();
    schematic_bundle_kind_helper.push_where_predicate_per_field(|ty, prev_tys| {
        let prev_ty = &prev_tys.first().unwrap_or(ty);
        if io {
            parse_quote! { #prev_ty: #substrate::types::codegen::HasSchematicBundleKindViews }
        } else {
            parse_quote! { #prev_ty: #substrate::types::schematic::SchematicBundleKind }
        }
    });

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

pub(crate) fn impl_has_node_bundle(
    kind_helper: &DeriveInputHelper,
    node_bundle_helper: &DeriveInputHelper,
    io: bool,
) -> TokenStream {
    let substrate = substrate_ident();
    let mut kind_helper = kind_helper.clone();
    kind_helper.push_where_predicate_per_field(|ty, _prev_tys| {
        parse_quote! { #ty: #substrate::types::schematic::HasNodeBundle }
    });
    kind_helper.push_where_predicate_per_field(|ty, prev_tys| {
        let prev_ty = prev_tys.first().unwrap_or(ty);
        if io {
            parse_quote! { #prev_ty: #substrate::types::codegen::HasSchematicBundleKindViews }
        } else {
            parse_quote! { #prev_ty: #substrate::types::schematic::SchematicBundleKind }
        }
    });

    let node_bundle_full_ty = node_bundle_helper.get_full_type();

    kind_helper.impl_trait(&ImplTrait {
        trait_name: quote! { #substrate::types::schematic::HasNodeBundle },
        trait_body: quote! {
            type NodeBundle = #node_bundle_full_ty;
        },
        extra_generics: vec![],
        extra_where_predicates: vec![],
    })
}

pub(crate) fn impl_has_terminal_bundle(
    kind_helper: &DeriveInputHelper,
    terminal_bundle_helper: &DeriveInputHelper,
    io: bool,
) -> TokenStream {
    let substrate = substrate_ident();
    let mut kind_helper = kind_helper.clone();
    kind_helper.push_where_predicate_per_field(|ty, _prev_tys| {
        parse_quote! { #ty: #substrate::types::schematic::HasTerminalBundle }
    });
    kind_helper.push_where_predicate_per_field(|ty, prev_tys| {
        let prev_ty = prev_tys.first().unwrap_or(ty);
        if io {
            parse_quote! { #prev_ty: #substrate::types::codegen::HasSchematicBundleKindViews }
        } else {
            parse_quote! { #prev_ty: #substrate::types::schematic::SchematicBundleKind }
        }
    });

    let terminal_bundle_full_ty = terminal_bundle_helper.get_full_type();

    kind_helper.impl_trait(&ImplTrait {
        trait_name: quote! { #substrate::types::schematic::HasTerminalBundle },
        trait_body: quote! {
            type TerminalBundle = #terminal_bundle_full_ty;
        },
        extra_generics: vec![],
        extra_where_predicates: vec![],
    })
}

pub(crate) fn impl_save_nested_bundle(view_helper: &DeriveInputHelper, nodes: bool) -> TokenStream {
    let substrate = substrate_ident();
    let mut view_helper = view_helper.clone();
    let simulator_ty = parse_quote! { SubstrateS };
    let analysis_ty = parse_quote! { SubstrateA };

    let (save_key_view, saved_view, bundle_view) = if nodes {
        (
            parse_quote! { #substrate::types::codegen::NestedNodeSaveKeyView<#simulator_ty, #analysis_ty> },
            parse_quote! { #substrate::types::codegen::NestedNodeSavedView<#simulator_ty, #analysis_ty> },
            parse_quote! { #substrate::types::codegen::NestedNodeBundle },
        )
    } else {
        (
            parse_quote! { #substrate::types::codegen::NestedTerminalSaveKeyView<#simulator_ty, #analysis_ty> },
            parse_quote! { #substrate::types::codegen::NestedTerminalSavedView<#simulator_ty, #analysis_ty> },
            parse_quote! { #substrate::types::codegen::NestedTerminalBundle },
        )
    };
    let mut save_key = view_helper.clone();
    save_key.add_generic_type_binding(parse_quote! { SubstrateV }, save_key_view);
    let mut saved = view_helper.clone();
    saved.add_generic_type_binding(parse_quote! { SubstrateV }, saved_view);

    view_helper.push_where_predicate_per_field(|ty, _prev_tys| {
        parse_quote! { #ty: #substrate::simulation::data::Save<#simulator_ty, #analysis_ty> }
    });

    view_helper.add_generic_type_binding(parse_quote! { SubstrateV }, bundle_view);

    let save_body = view_helper.map_data(
        &save_key.get_full_turbofish_type(),
            |MapField { ty, refer, .. }| {
                    quote! { <#ty as #substrate::simulation::data::Save<#simulator_ty, #analysis_ty>>::save(&#refer, __substrate_ctx, __substrate_opts) }
            });
    let mut from_saved_helper = view_helper.clone();
    from_saved_helper.set_referent(quote! { __substrate_key });
    let from_saved_body = from_saved_helper.map_data(
        &saved.get_full_turbofish_type(),
            |MapField { ty, refer, .. }| {
                    quote! { <#ty as #substrate::simulation::data::Save<#simulator_ty, #analysis_ty>>::from_saved(__substrate_output, #refer) }
            });

    let save_key_full_ty = save_key.get_full_type();
    let saved_full_ty = saved.get_full_type();

    view_helper.impl_trait(&ImplTrait {
        trait_name: quote! { #substrate::simulation::data::Save<#simulator_ty, #analysis_ty> },
        trait_body: quote! {
            type SaveKey = #save_key_full_ty;
            type Saved = #saved_full_ty;
            fn save(
                &self,
                __substrate_ctx: &#substrate::simulation::SimulationContext<#simulator_ty>,
                __substrate_opts: &mut <#simulator_ty as #substrate::simulation::Simulator>::Options,
            ) -> <Self as #substrate::simulation::data::Save<#simulator_ty, #analysis_ty>>::SaveKey {
                #save_body
            }

            fn from_saved(
                __substrate_output: &<#analysis_ty as #substrate::simulation::Analysis>::Output,
                __substrate_key: &<Self as #substrate::simulation::data::Save<#simulator_ty, #analysis_ty>>::SaveKey,
            ) -> <Self as #substrate::simulation::data::Save<#simulator_ty, #analysis_ty>>::Saved {
                #from_saved_body
            }
        },
        extra_where_predicates: vec![
            parse_quote! { #simulator_ty: #substrate::simulation::Simulator },
            parse_quote! { #analysis_ty: #substrate::simulation::Analysis },
        ],
        extra_generics: vec![simulator_ty, analysis_ty],
    })
}

pub(crate) fn impl_save_nested_native(view_helper: &DeriveInputHelper) -> TokenStream {
    let substrate = substrate_ident();
    let mut view_helper = view_helper.clone();
    let simulator_ty = parse_quote! { SubstrateS };
    let analysis_ty = parse_quote! { SubstrateA };

    let hnv_generic_ty: syn::Ident = parse_quote!(SubstrateT);
    let hnv_generic: syn::GenericParam = parse_quote!(#hnv_generic_ty);

    let save_key_view = parse_quote! { #substrate::types::codegen::NestedSaveKey<#hnv_generic_ty, #simulator_ty, #analysis_ty> };
    let saved_view = parse_quote! { #substrate::types::codegen::NestedSaved<#hnv_generic_ty, #simulator_ty, #analysis_ty> };

    let mut save_key = view_helper.clone();
    save_key.add_generic_type_binding(parse_quote! { SubstrateV }, save_key_view);
    let mut saved = view_helper.clone();
    saved.add_generic_type_binding(parse_quote! { SubstrateV }, saved_view);

    view_helper.push_where_predicate_per_field(|ty, prev_tys| {
        let ty = if prev_tys.is_empty() {
            ty
        } else {
            &prev_tys[0]
        };
        parse_quote! { #ty: #substrate::schematic::HasNestedView<#hnv_generic_ty> }
    });
    view_helper.push_where_predicate_per_field(|ty, prev_tys| {
        let ty = if prev_tys.is_empty() {
            ty
        } else {
            &prev_tys[0]
        };
        parse_quote! {<#ty as #substrate::schematic::HasNestedView<#hnv_generic_ty>>::NestedView: #substrate::simulation::data::Save<#simulator_ty, #analysis_ty> }
    });

    view_helper.add_generic_type_binding(
        parse_quote! { SubstrateV },
        parse_quote! { #substrate::types::codegen::Nested<#hnv_generic_ty> },
    );

    let save_body = view_helper.map_data(
        &save_key.get_full_turbofish_type(),
            |MapField { ty, refer, .. }| {
                    quote! { <#ty as #substrate::simulation::data::Save<#simulator_ty, #analysis_ty>>::save(&#refer, __substrate_ctx, __substrate_opts) }
            });
    let mut from_saved_helper = view_helper.clone();
    from_saved_helper.set_referent(quote! { __substrate_key });
    let from_saved_body = from_saved_helper.map_data(
        &saved.get_full_turbofish_type(),
            |MapField { ty, refer, .. }| {
                    quote! { <#ty as #substrate::simulation::data::Save<#simulator_ty, #analysis_ty>>::from_saved(__substrate_output, #refer) }
            });

    let save_key_full_ty = save_key.get_full_type();
    let saved_full_ty = saved.get_full_type();

    view_helper.impl_trait(&ImplTrait {
        trait_name: quote! { #substrate::simulation::data::Save<#simulator_ty, #analysis_ty> },
        trait_body: quote! {
            type SaveKey = #save_key_full_ty;
            type Saved = #saved_full_ty;
            fn save(
                &self,
                __substrate_ctx: &#substrate::simulation::SimulationContext<#simulator_ty>,
                __substrate_opts: &mut <#simulator_ty as #substrate::simulation::Simulator>::Options,
            ) -> <Self as #substrate::simulation::data::Save<#simulator_ty, #analysis_ty>>::SaveKey {
                #save_body
            }

            fn from_saved(
                __substrate_output: &<#analysis_ty as #substrate::simulation::Analysis>::Output,
                __substrate_key: &<Self as #substrate::simulation::data::Save<#simulator_ty, #analysis_ty>>::SaveKey,
            ) -> <Self as #substrate::simulation::data::Save<#simulator_ty, #analysis_ty>>::Saved {
                #from_saved_body
            }
        },
        extra_where_predicates: vec![
            parse_quote! { #simulator_ty: #substrate::simulation::Simulator },
            parse_quote! { #analysis_ty: #substrate::simulation::Analysis },
        ],
        extra_generics: vec![hnv_generic, simulator_ty, analysis_ty],
    })
}

/// If `generic` is Some(TY), generates an implementation of `HasNestedView<TY>`.
/// Otherwise, only generates an implementation of `HasNestedView` (i.e. only for
/// the default TY = InstancePath).
///
/// `nested_view_helper` should have its view generic type bound to `Nested<TY>`
/// or `Nested` if `generic` is None.
pub(crate) fn impl_has_nested_view(
    view_helper: &DeriveInputHelper,
    nested_view_helper: &DeriveInputHelper,
    generic: Option<GenericParam>,
) -> TokenStream {
    let substrate = substrate_ident();
    let mut has_nested_view_helper = view_helper.clone();
    if let Some(hnv_ty) = generic {
        has_nested_view_helper.push_where_predicate_per_field(
            |ty, _| parse_quote! { #ty: #substrate::schematic::HasNestedView<#hnv_ty> },
        );

        let nested_view_full_ty = nested_view_helper.get_full_type();

        let nested_view_body = has_nested_view_helper.map_data(
        &nested_view_helper.get_type(),
            |MapField { ty, refer, .. }| {
                    quote! { <#ty as #substrate::schematic::HasNestedView<#hnv_ty>>::nested_view(#refer, __substrate_parent) }
            });
        has_nested_view_helper.impl_trait(&ImplTrait {
            trait_name: quote! { #substrate::schematic::HasNestedView<#hnv_ty> },
            trait_body: quote! {
                type NestedView = #nested_view_full_ty;

                fn nested_view(&self, __substrate_parent: &#hnv_ty) -> #substrate::schematic::NestedView<Self, #hnv_ty> {
                    #nested_view_body
                }
            },
            extra_generics: vec![hnv_ty],
            extra_where_predicates: vec![],
        })
    } else {
        has_nested_view_helper.push_where_predicate_per_field(
            |ty, _| parse_quote! { #ty: #substrate::schematic::HasNestedView },
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
}

pub(crate) fn impl_view_as(view_helper: &DeriveInputHelper, nodes: bool) -> TokenStream {
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

    let node_bundle_fn = (!nodes).then(|| {
        quote!{
            /// Views this terminal bundle as a node bundle of the same kind.
            #vis fn node_bundle(&self) -> #substrate::types::schematic::NodeBundle<<Self as #substrate::types::HasBundleKind>::BundleKind> {
                let kind = <Self as #substrate::types::HasBundleKind>::kind(self);
                let mut flat_nodes = <Self as #substrate::types::Flatten<#substrate::types::schematic::Node>>::flatten_vec(self).into_iter();
                <#substrate::types::schematic::NodeBundle::<Self> as #substrate::types::Unflatten<<Self as #substrate::types::HasBundleKind>::BundleKind, #substrate::types::schematic::Node>>::unflatten(&kind, &mut flat_nodes).unwrap()
            }
        }
    });

    quote! {
        impl #imp #full_ty #wher {
            /// Views this bundle as a bundle of a different kind.
            #vis fn view_as<SubstrateT: #substrate::types::HasBundleKind<BundleKind: #substrate::types::schematic::SchematicBundleKind>>(&self) -> #substrate::types::schematic::#bundle_view_ident<<SubstrateT as #substrate::types::HasBundleKind>::BundleKind> where <Self as #substrate::types::HasBundleKind>::BundleKind: #substrate::types::schematic::DataView<<SubstrateT as #substrate::types::HasBundleKind>::BundleKind>{
                <<Self as #substrate::types::HasBundleKind>::BundleKind as #substrate::types::schematic::DataView<<SubstrateT as #substrate::types::HasBundleKind>::BundleKind>>::#view_as_fn(self)
            }

            #node_bundle_fn
        }
    }
}

/// Derives `HasNestedView` and `Save` for the input.
pub(crate) fn nested_data(input: &DeriveInput) -> syn::Result<TokenStream> {
    let substrate = substrate_ident();
    let helper = DeriveInputHelper::new(input.clone())?;
    let view_ident = format_ident!("{}View", &input.ident);
    let mut all_decls_impls = Vec::new();

    // Create `View` struct
    // - Needs to add a generic along with a where clause per field that uses that generic
    // - Potentially be able to add separate where clauses to new generic
    let mut view_helper = helper.clone();
    view_helper.set_ident(view_ident);
    let view_generic_ty = quote! { SubstrateV };
    view_helper.push_generic_param(parse_quote! { #view_generic_ty });
    let mut save_helper = view_helper.clone();
    // These where clauses should only be pushed to the view helper, not the save helper.
    // [`impl_save_nested_native`] will add sufficient where clauses, and adding the where
    // clauses below seems to confuse the trait solver.
    view_helper.push_where_predicate_per_field(
        |ty, _| parse_quote! { #ty: #substrate::types::codegen::HasView<#view_generic_ty> },
    );
    for helper in [&mut view_helper, &mut save_helper] {
        helper.map_types(
            |ty| parse_quote! { <#ty as #substrate::types::codegen::HasView<#view_generic_ty>>::View },
        );
    }
    all_decls_impls.push(view_helper.decl_data());
    all_decls_impls.push(impl_flatlen(&view_helper));
    all_decls_impls.push(impl_flatten_generic(&view_helper));

    let mut hnv_helper = view_helper.clone();
    let hnv_ty = parse_quote!(SubstrateParent);
    hnv_helper.add_generic_type_binding(
        parse_quote!(#view_generic_ty),
        parse_quote!(#substrate::types::codegen::Nested<#hnv_ty>),
    );

    all_decls_impls.push(impl_has_nested_view(&helper, &hnv_helper, Some(hnv_ty)));
    all_decls_impls.push(impl_save_nested_native(&save_helper));
    Ok(quote! {
        #( #all_decls_impls )*
    })
}

pub(crate) fn schematic_bundle_kind(
    io_helper: &DeriveInputHelper,
    kind_helper: &DeriveInputHelper,
    view_helper: &DeriveInputHelper,
    io: bool,
) -> TokenStream {
    let substrate = substrate_ident();
    let mut all_decls_impls = Vec::new();
    let view_generic_ty = quote! { SubstrateV };

    let original_helper = if io { io_helper } else { kind_helper };

    let mut node_bundle_helper = original_helper.clone();
    node_bundle_helper.set_ident(view_helper.get_ident().clone());
    node_bundle_helper.push_generic_param(parse_quote! { #view_generic_ty });
    node_bundle_helper.push_where_predicate_per_field(|ty, _| {
        if io {
            parse_quote! { #ty: #substrate::types::codegen::HasSchematicBundleKindViews }
        } else {
            parse_quote! { #ty: #substrate::types::schematic::SchematicBundleKind }
        }
    });

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

    if io {
        all_decls_impls.push(impl_has_node_bundle(io_helper, &node_bundle_helper, io));
        all_decls_impls.push(impl_has_terminal_bundle(
            io_helper,
            &terminal_bundle_helper,
            io,
        ));
    }
    all_decls_impls.push(impl_has_node_bundle(kind_helper, &node_bundle_helper, io));
    all_decls_impls.push(impl_has_terminal_bundle(
        kind_helper,
        &terminal_bundle_helper,
        io,
    ));
    all_decls_impls.push(impl_schematic_bundle_kind(
        kind_helper,
        &node_bundle_helper,
        &terminal_bundle_helper,
        io,
    ));
    all_decls_impls.push(impl_has_nested_view(
        &node_bundle_helper,
        &nested_node_bundle_helper,
        None,
    ));
    all_decls_impls.push(impl_has_nested_view(
        &terminal_bundle_helper,
        &nested_terminal_bundle_helper,
        None,
    ));
    all_decls_impls.push(impl_has_nested_view(
        &nested_node_bundle_helper,
        &nested_node_bundle_helper,
        None,
    ));
    all_decls_impls.push(impl_has_nested_view(
        &nested_terminal_bundle_helper,
        &nested_terminal_bundle_helper,
        None,
    ));
    all_decls_impls.push(impl_view_as(&node_bundle_helper, true));
    all_decls_impls.push(impl_view_as(&terminal_bundle_helper, false));
    all_decls_impls.push(impl_save_nested_bundle(view_helper, true));
    all_decls_impls.push(impl_save_nested_bundle(view_helper, false));

    quote! {
        #( #all_decls_impls )*
    }
}
