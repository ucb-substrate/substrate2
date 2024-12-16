use darling::{ast, FromDeriveInput, FromField};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{parse_quote, token::Where, GenericParam, WhereClause};
use type_dispatch::derive::{
    add_trait_bounds, field_tokens, field_tokens_with_referent, struct_body, FieldTokens,
};

use crate::substrate_ident;

#[derive(Debug, FromDeriveInput)]
#[darling(
    attributes(substrate),
    supports(struct_any),
    forward_attrs(allow, doc, cfg)
)]
pub struct IoInputReceiver {
    ident: syn::Ident,
    generics: syn::Generics,
    data: ast::Data<(), IoField>,
    vis: syn::Visibility,
    attrs: Vec<syn::Attribute>,
    #[darling(default)]
    layout_type: Option<syn::Type>,
}

#[derive(Debug, FromField)]
#[darling(attributes(substrate), forward_attrs(allow, doc, cfg))]
pub struct IoField {
    ident: Option<syn::Ident>,
    vis: syn::Visibility,
    ty: syn::Type,
    attrs: Vec<syn::Attribute>,
    #[darling(default)]
    layout_type: Option<syn::Type>,
}

pub(crate) fn schematic_io(input: &IoInputReceiver) -> TokenStream {
    let IoInputReceiver {
        ref ident,
        ref generics,
        ref data,
        ref vis,
        ..
    } = *input;

    let substrate = substrate_ident();

    let bundle_ident = format_ident!("{}Bundle", ident);
    let bundle_type_ident = format_ident!("{}BundleKind", ident);
    let bundle_primitive_ty: syn::Ident = parse_quote!(__substrate_T);
    let bundle_primitive_generic: syn::GenericParam = parse_quote!(#bundle_primitive_ty: #substrate::types::HasBundleKind<BundleKind = #substrate::types::Signal>);

    let mut hnt_generics = generics.clone();
    add_trait_bounds(&mut hnt_generics, quote!(#substrate::types::HasNameTree));

    let mut st_generics = generics.clone();
    add_trait_bounds(
        &mut st_generics,
        quote!(#substrate::types::schematic::SchematicBundleKind),
    );

    let mut st_any_generics = st_generics.clone();
    add_trait_bounds(&mut st_any_generics, quote!(::std::any::Any));
    let (st_any_imp, st_any_ty, st_any_where) = st_any_generics.split_for_impl();

    let mut hnv_generics = st_any_generics.clone();
    hnv_generics.params.push(bundle_primitive_generic.clone());
    let (hnv_imp, hnv_ty, hnv_where) = hnv_generics.split_for_impl();
    let generic_idents = generics
        .params
        .iter()
        .map(|generic| match generic {
            GenericParam::Lifetime(lt) => lt.lifetime.ident.clone(),
            GenericParam::Type(ty) => ty.ident.clone(),
            GenericParam::Const(c) => c.ident.clone(),
        })
        .collect::<Vec<_>>();

    let fields = data.as_ref().take_struct().unwrap();

    let mut construct_data_fields = Vec::new();
    let mut construct_nested_view_node_fields = Vec::new();
    let mut construct_nested_view_terminal_fields = Vec::new();
    let mut construct_nested_view_nested_node_fields = Vec::new();
    let mut construct_nested_view_nested_terminal_fields = Vec::new();
    let mut construct_terminal_view_fields = Vec::new();
    let mut instantiate_fields = Vec::new();
    let mut flatten_node_fields = Vec::new();

    for (i, &f) in fields.iter().enumerate() {
        let field_ty = &f.ty;
        let field_vis = &f.vis;
        let field_ident = &f.ident;
        let attrs = &f.attrs;

        let FieldTokens {
            refer,
            assign,
            temp,
            ..
        } = field_tokens(fields.style, field_vis, attrs, i, field_ident);

        let FieldTokens {
            refer: cell_io_refer,
            ..
        } = field_tokens_with_referent(
            fields.style,
            field_vis,
            attrs,
            i,
            field_ident,
            quote! { cell_io },
        );

        let FieldTokens {
            refer: instance_io_refer,
            ..
        } = field_tokens_with_referent(
            fields.style,
            field_vis,
            attrs,
            i,
            field_ident,
            quote! { instance_io },
        );

        construct_data_fields.push(quote! {
            #assign #temp,
        });
        construct_nested_view_node_fields.push(quote! {
                #assign <#substrate::types::schematic::NodeBundle<#field_ty> as #substrate::schematic::HasNestedView>::nested_view(&#refer, parent),
        });
        construct_nested_view_terminal_fields.push(quote! {
                #assign <#substrate::types::schematic::TerminalBundle<#field_ty> as #substrate::schematic::HasNestedView>::nested_view(&#refer, parent),
        });
        construct_nested_view_nested_node_fields.push(quote! {
                #assign <<<#field_ty as #substrate::types::HasBundleKind>::BundleKind as #substrate::types::codegen::HasView<#substrate::types::codegen::NestedNodeBundle>>::View as #substrate::schematic::HasNestedView>::nested_view(&#refer, parent),
        });
        construct_nested_view_nested_terminal_fields.push(quote! {
                #assign <<<#field_ty as #substrate::types::HasBundleKind>::BundleKind as #substrate::types::codegen::HasView<#substrate::types::codegen::NestedTerminalBundle>>::View as #substrate::schematic::HasNestedView>::nested_view(&#refer, parent),
        });
        construct_terminal_view_fields.push(quote! {
                #assign <<#field_ty as #substrate::types::HasBundleKind>::BundleKind as #substrate::types::schematic::SchematicBundleKind>::terminal_view(cell, &#cell_io_refer, instance, &#instance_io_refer),
        });
        instantiate_fields.push(quote! {
                let (#temp, __substrate_node_ids) = <<#field_ty as #substrate::types::HasBundleKind>::BundleKind as #substrate::types::schematic::SchematicBundleKind>::instantiate(&#refer, __substrate_node_ids);
        });
        flatten_node_fields.push(quote! {
                <<<#field_ty as #substrate::types::HasBundleKind>::BundleKind as #substrate::types::codegen::HasView<#substrate::types::schematic::Terminal>>::View as #substrate::types::Flatten<#substrate::types::schematic::Node>>::flatten(&#refer, __substrate_output_sink);
        });
    }

    let construct_data_body =
        struct_body(fields.style, false, quote!( #(#construct_data_fields)* ));
    let construct_nested_view_node_body = struct_body(
        fields.style,
        false,
        quote!( #(#construct_nested_view_node_fields)* ),
    );
    let construct_nested_view_terminal_body = struct_body(
        fields.style,
        false,
        quote!( #(#construct_nested_view_terminal_fields)* ),
    );
    let construct_nested_view_nested_node_body = struct_body(
        fields.style,
        false,
        quote!( #(#construct_nested_view_nested_node_fields)* ),
    );
    let construct_nested_view_nested_terminal_body = struct_body(
        fields.style,
        false,
        quote!( #(#construct_nested_view_nested_terminal_fields)* ),
    );
    let construct_terminal_view_body = struct_body(
        fields.style,
        false,
        quote!( #(#construct_terminal_view_fields)* ),
    );

    quote! {
        impl #st_any_imp #substrate::types::schematic::SchematicBundleKind for #bundle_type_ident #st_any_ty #st_any_where {
            type NodeBundle = #bundle_ident<#(#generic_idents,)*#substrate::types::codegen::NodeBundle>;
            type TerminalBundle = #bundle_ident<#(#generic_idents,)*#substrate::types::codegen::TerminalBundle>;
            fn terminal_view(
                cell: #substrate::schematic::CellId,
                cell_io: &#substrate::types::schematic::NodeBundle<Self>,
                instance: #substrate::schematic::InstanceId,
                instance_io: &#substrate::types::schematic::NodeBundle<Self>,
            ) -> #substrate::types::schematic::TerminalBundle<Self> {
                #bundle_ident #construct_terminal_view_body
            }
        }

        impl #st_any_imp #bundle_ident <#(#generic_idents,)*#substrate::types::codegen::NodeBundle> #st_any_where {
            /// Views this node bundle as a node bundle of a different kind.
            #vis fn view_as<__substrate_T: #substrate::types::HasBundleKind<BundleKind: #substrate::types::schematic::SchematicBundleKind>>(&self) -> #substrate::types::schematic::NodeBundle<<__substrate_T as #substrate::types::HasBundleKind>::BundleKind> where <Self as #substrate::types::HasBundleKind>::BundleKind: #substrate::types::schematic::DataView<<__substrate_T as #substrate::types::HasBundleKind>::BundleKind>{
                <<Self as #substrate::types::HasBundleKind>::BundleKind as #substrate::types::schematic::DataView<<__substrate_T as #substrate::types::HasBundleKind>::BundleKind>>::view_nodes_as(self)
            }
        }

        impl #st_any_imp #bundle_ident <#(#generic_idents,)*#substrate::types::codegen::TerminalBundle> #st_any_where {
            /// Views this terminal bundle as a terminal bundle of a different kind.
            #vis fn view_as<__substrate_T: #substrate::types::HasBundleKind<BundleKind: #substrate::types::schematic::SchematicBundleKind>>(&self) -> #substrate::types::schematic::TerminalBundle<<__substrate_T as #substrate::types::HasBundleKind>::BundleKind> where <Self as #substrate::types::HasBundleKind>::BundleKind: #substrate::types::schematic::DataView<<__substrate_T as #substrate::types::HasBundleKind>::BundleKind>{
                <<Self as #substrate::types::HasBundleKind>::BundleKind as #substrate::types::schematic::DataView<<__substrate_T as #substrate::types::HasBundleKind>::BundleKind>>::view_terminals_as(self)
            }
        }

        impl #st_any_imp #substrate::schematic::HasNestedView for #bundle_ident <#(#generic_idents,)*#substrate::types::codegen::NodeBundle> #hnv_where {
            type NestedView = #bundle_ident <#(#generic_idents,)*#substrate::types::codegen::NestedNodeBundle>;

            fn nested_view(&self, parent: &#substrate::schematic::InstancePath) -> #substrate::schematic::NestedView<Self> {
                #bundle_ident #construct_nested_view_node_body
            }
        }

        impl #st_any_imp #substrate::schematic::HasNestedView for #bundle_ident <#(#generic_idents,)*#substrate::types::codegen::TerminalBundle> #hnv_where {
            type NestedView = #bundle_ident <#(#generic_idents,)*#substrate::types::codegen::NestedTerminalBundle>;

            fn nested_view(&self, parent: &#substrate::schematic::InstancePath) -> #substrate::schematic::NestedView<Self> {
                #bundle_ident #construct_nested_view_terminal_body
            }
        }

        impl #st_any_imp #substrate::schematic::HasNestedView for #bundle_ident <#(#generic_idents,)*#substrate::types::codegen::NestedNodeBundle> #hnv_where {
            type NestedView = #bundle_ident <#(#generic_idents,)*#substrate::types::codegen::NestedNodeBundle>;

            fn nested_view(&self, parent: &#substrate::schematic::InstancePath) -> #substrate::schematic::NestedView<Self> {
                #bundle_ident #construct_nested_view_nested_node_body
            }
        }

        impl #st_any_imp #substrate::schematic::HasNestedView for #bundle_ident <#(#generic_idents,)*#substrate::types::codegen::NestedTerminalBundle> #hnv_where {
            type NestedView = #bundle_ident <#(#generic_idents,)*#substrate::types::codegen::NestedTerminalBundle>;

            fn nested_view(&self, parent: &#substrate::schematic::InstancePath) -> #substrate::schematic::NestedView<Self> {
                #bundle_ident #construct_nested_view_nested_terminal_body
            }
        }
    }
}

pub(crate) fn layout_io(input: &IoInputReceiver) -> TokenStream {
    let IoInputReceiver {
        ref ident,
        ref generics,
        ref attrs,
        ref data,
        ref vis,
        ref layout_type,
    } = *input;

    let substrate = substrate_ident();

    let bundle_type_ident = format_ident!("{}BundleKind", ident);

    let mut lt_generics = generics.clone();
    add_trait_bounds(
        &mut lt_generics,
        quote!(#substrate::types::layout::HardwareType),
    );
    let (lt_imp, lt_ty, lt_where) = lt_generics.split_for_impl();

    let mut lt_any_generics = lt_generics.clone();
    add_trait_bounds(&mut lt_any_generics, quote!(::std::any::Any));
    let (lt_any_imp, lt_any_ty, lt_any_where) = lt_any_generics.split_for_impl();

    let mut hbf_generics = lt_generics.clone();

    let mut idents = Vec::new();
    for param in &lt_generics.params {
        if let syn::GenericParam::Type(ref type_param) = *param {
            idents.push(type_param.ident.clone());
        }
    }
    for ident in idents {
        hbf_generics.make_where_clause().predicates.push(syn::parse_quote!(<#ident as #substrate::types::layout::HardwareType>::Builder: #substrate::types::layout::HierarchicalBuildFrom<#substrate::layout::element::NamedPorts>));
    }

    let (hbf_imp, hbf_ty, hbf_where) = hbf_generics.split_for_impl();

    let (_imp, ty, _wher) = generics.split_for_impl();
    let fields = data.as_ref().take_struct().unwrap();

    if let Some(layout_type) = layout_type {
        return quote! {
            impl #lt_any_imp #substrate::types::layout::HasHardwareType for #ident #lt_any_ty #lt_any_where{
                type HardwareType = #layout_type;

                fn builder(&self) -> <<Self as #substrate::types::layout::HasHardwareType>::HardwareType as #substrate::types::layout::HardwareType>::Builder {
                    <#layout_type as #substrate::types::layout::HasHardwareType>::builder(&<#layout_type as #substrate::types::layout::CustomHardwareType<#ident>>::from_layout_type(self))
                }
            }
        };
    }

    let mut ty_len = Vec::new();
    let mut layout_data_len = Vec::new();
    let mut layout_data_fields = Vec::new();
    let mut construct_data_ty_fields = Vec::new();
    let mut construct_builder_ty_fields = Vec::new();
    let mut layout_builder_fields = Vec::new();
    let mut flatten_port_geometry_fields = Vec::new();
    let mut create_builder_fields = Vec::new();
    let mut translated_fields = Vec::new();
    let mut transformed_fields = Vec::new();
    let mut build_data_fields = Vec::new();
    let mut hierarchical_build_from_fields = Vec::new();

    let layout_data_ident = format_ident!("{}Layout", ident);
    let layout_builder_ident = format_ident!("{}LayoutBuilder", ident);

    for (i, &f) in fields.iter().enumerate() {
        let (field_ty, switch_type) = match f.layout_type {
            Some(ref ty) => (ty.clone(), true),
            None => (f.ty.clone(), false),
        };
        let original_field_ty = &f.ty;

        let FieldTokens {
            declare,
            refer,
            assign,
            pretty_ident,
            ..
        } = field_tokens(fields.style, &f.vis, &f.attrs, i, &f.ident);

        ty_len.push(quote! {
            <#field_ty as #substrate::types::FlatLen>::len(&#refer)
        });
        layout_data_len.push(quote! {
                <<#field_ty as #substrate::types::layout::HardwareType>::Bundle as #substrate::types::FlatLen>::len(&#refer)
            });
        layout_data_fields.push(quote! {
            #declare <#field_ty as #substrate::types::layout::HardwareType>::Bundle,
        });
        construct_data_ty_fields.push(quote! {
            #assign <<#field_ty as #substrate::types::layout::HardwareType>::Bundle as #substrate::types::HasBundleKind>::kind(&#refer),
        });
        construct_builder_ty_fields.push(quote! {
            #assign <<#field_ty as #substrate::types::layout::HardwareType>::Builder as #substrate::types::HasBundleKind>::kind(&#refer),
        });
        layout_builder_fields.push(quote! {
            #declare <#field_ty as #substrate::types::layout::HardwareType>::Builder,
        });
        flatten_port_geometry_fields.push(quote! {
                <<#field_ty as #substrate::types::layout::HardwareType>::Bundle as #substrate::types::Flatten<#substrate::types::layout::PortGeometry>>::flatten(&#refer, __substrate_output_sink);
            });
        if switch_type {
            create_builder_fields.push(quote! {
                    #assign <#field_ty as #substrate::types::layout::HasHardwareType>::builder(&<#field_ty as #substrate::types::layout::CustomHardwareType<#original_field_ty>>::from_layout_type(&#refer)),
                });
        } else {
            create_builder_fields.push(quote! {
                #assign <#field_ty as #substrate::types::layout::HasHardwareType>::builder(&#refer),
            });
        }
        translated_fields.push(quote! {
                #assign #substrate::geometry::transform::TranslateRef::translate_ref(&#refer, p),
        });
        transformed_fields.push(quote! {
                #assign #substrate::geometry::transform::TransformRef::transform_ref(&#refer, trans),
        });
        build_data_fields.push(quote! {
                #assign #substrate::types::layout::BundleBuilder::<<#field_ty as #substrate::types::layout::HardwareType>::Bundle>::build(#refer)?,
        });
        hierarchical_build_from_fields.push(quote! {
            #substrate::types::NameBuf::push(path, #substrate::arcstr::literal!(::std::stringify!(#pretty_ident)));
            #substrate::types::layout::HierarchicalBuildFrom::<#substrate::layout::element::NamedPorts>::build_from(&mut #refer, path, source);
            #substrate::types::NameBuf::pop(path);
        });
    }

    // Return 0 from `FlatLen::len` if struct has no fields.
    if ty_len.is_empty() {
        ty_len.push(quote! { 0 });
    }

    if layout_data_len.is_empty() {
        layout_data_len.push(quote! { 0 });
    }

    let layout_data_body = struct_body(fields.style, true, quote! { #( #layout_data_fields )* });
    let construct_data_ty_body = struct_body(
        fields.style,
        true,
        quote! { #( #construct_data_ty_fields )* },
    );
    let construct_builder_ty_body = struct_body(
        fields.style,
        true,
        quote! { #( #construct_builder_ty_fields )* },
    );
    let layout_builder_body =
        struct_body(fields.style, true, quote! { #( #layout_builder_fields )* });
    let create_builder_body =
        struct_body(fields.style, false, quote! { #( #create_builder_fields )* });
    let translated_body = struct_body(fields.style, false, quote! { #( #translated_fields )* });
    let transformed_body = struct_body(fields.style, false, quote! { #( #transformed_fields )* });
    let build_layout_data_body =
        struct_body(fields.style, false, quote! { #( #build_data_fields )* });

    quote! {
        impl #lt_any_imp #substrate::types::layout::HasHardwareType for #ident #lt_any_ty #lt_any_where{
            type HardwareType = #ident;

            fn builder(&self) -> <<Self as #substrate::types::layout::HasHardwareType>::HardwareType as #substrate::types::layout::HardwareType>::Builder {
                #layout_builder_ident #create_builder_body
            }
        }

        impl #lt_any_imp #substrate::types::layout::HardwareType for #ident #lt_any_ty #lt_any_where {
            type Bundle = #layout_data_ident #ty;
            type Builder = #layout_builder_ident #ty;
        }

        #(#attrs)*
        #vis struct #layout_data_ident #lt_generics #layout_data_body

        #(#attrs)*
        #vis struct #layout_builder_ident #lt_generics #layout_builder_body

        impl #lt_imp #substrate::types::HasBundleKind for #layout_data_ident #lt_ty #lt_where {
            type BundleKind = #bundle_type_ident #lt_ty;

            fn kind(&self) ->  <Self as #substrate::types::HasBundleKind>::BundleKind {
                #bundle_type_ident #construct_data_ty_body
            }
        }

        impl #lt_imp #substrate::types::HasBundleKind for #layout_builder_ident #lt_ty #lt_where {
            type BundleKind = #bundle_type_ident #lt_ty;

            fn kind(&self) -> <Self as #substrate::types::HasBundleKind>::BundleKind {
                #bundle_type_ident #construct_builder_ty_body
            }
        }

        impl #lt_imp #substrate::types::FlatLen for #layout_data_ident #lt_ty #lt_where {
            fn len(&self) -> usize {
                #( #layout_data_len )+*
            }
        }

        impl #lt_imp #substrate::types::Flatten<#substrate::types::layout::PortGeometry> for #layout_data_ident #lt_ty #lt_where {
            fn flatten<E>(&self, __substrate_output_sink: &mut E)
            where
                E: ::std::iter::Extend<#substrate::types::layout::PortGeometry> {
                #( #flatten_port_geometry_fields )*
            }
        }

        impl #lt_any_imp #substrate::geometry::transform::TranslateRef for #layout_data_ident #lt_any_ty #lt_any_where {
            fn translate_ref(
                &self,
                p: #substrate::geometry::point::Point,
            ) -> Self {
                #layout_data_ident #translated_body
            }
        }

        impl #lt_any_imp #substrate::geometry::transform::TransformRef for #layout_data_ident #lt_any_ty #lt_any_where {
            fn transform_ref(
                &self,
                trans: #substrate::geometry::transform::Transformation,
            ) -> Self {
                #layout_data_ident #transformed_body
            }
        }


        impl #lt_any_imp #substrate::types::layout::BundleBuilder<#layout_data_ident #ty> for #layout_builder_ident #lt_any_ty #lt_any_where {
            fn build(self) -> #substrate::error::Result<#layout_data_ident #ty> {
                #substrate::error::Result::Ok(#layout_data_ident #build_layout_data_body)
            }
        }

        impl #hbf_imp #substrate::types::layout::HierarchicalBuildFrom<#substrate::layout::element::NamedPorts> for #layout_builder_ident #hbf_ty #hbf_where {
            fn build_from(&mut self, path: &mut #substrate::types::NameBuf, source: &#substrate::layout::element::NamedPorts) {
                #(#hierarchical_build_from_fields)*
            }
        }
    }
}

pub(crate) fn io_core_impl(input: &IoInputReceiver, flatten_dir: bool) -> TokenStream {
    let substrate = substrate_ident();
    let IoInputReceiver {
        ref ident,
        ref generics,
        ref data,
        ref vis,
        ref attrs,
        ..
    } = *input;

    let (generics_imp, generics_ty, generics_wher) = generics.split_for_impl();

    let bundle_ident = format_ident!("{}Bundle", ident);
    let bundle_type_ident = format_ident!("{}BundleKind", ident);
    let flatten_ty: syn::Ident = parse_quote!(__substrate_F);
    let flatten_generic: syn::GenericParam = parse_quote!(#flatten_ty);
    let bundle_primitive_ty: syn::Ident = parse_quote!(__substrate_T);
    let bundle_primitive_generic: syn::GenericParam = parse_quote!(#bundle_primitive_ty);
    let unflatten_source_ty: syn::Ident = parse_quote!(__substrate_S);
    let unflatten_source_generic: syn::GenericParam = parse_quote!(#unflatten_source_ty);

    let mut bundle_generics = generics.clone();
    bundle_generics.params.push(bundle_primitive_generic);
    let (bundle_imp, bundle_ty, bundle_wher) = bundle_generics.split_for_impl();

    let mut bundle_flatten_generics = bundle_generics.clone();
    bundle_flatten_generics.params.push(flatten_generic);
    let (bundle_flatten_imp, bundle_flatten_ty, bundle_flatten_wher) =
        bundle_flatten_generics.split_for_impl();

    let mut bundle_unflatten_generics = bundle_generics.clone();
    bundle_unflatten_generics
        .params
        .push(unflatten_source_generic);
    let (bundle_unflatten_imp, bundle_unflatten_ty, bundle_unflatten_wher) =
        bundle_unflatten_generics.split_for_impl();

    let mut hnt_generics = generics.clone();
    add_trait_bounds(&mut hnt_generics, quote!(#substrate::types::HasNameTree));

    let mut io_generics = generics.clone();
    add_trait_bounds(&mut io_generics, quote!(#substrate::types::BundleKind));
    add_trait_bounds(
        &mut io_generics,
        quote!(#substrate::types::layout::HardwareType),
    );
    add_trait_bounds(&mut io_generics, quote!(#substrate::types::Directed));

    let mut flatlen_generics = generics.clone();
    add_trait_bounds(&mut flatlen_generics, quote!(#substrate::types::FlatLen));

    let (hnt_imp, hnt_ty, hnt_wher) = hnt_generics.split_for_impl();
    let (flatlen_imp, flatlen_ty, flatlen_wher) = flatlen_generics.split_for_impl();

    let mut io_len = Vec::new();
    let mut name_fields = Vec::new();

    let mut st_generics = generics.clone();
    add_trait_bounds(
        &mut st_generics,
        quote!(#substrate::types::schematic::SchematicBundleKind),
    );

    let mut st_any_generics = st_generics.clone();
    add_trait_bounds(&mut st_any_generics, quote!(::std::any::Any));

    let mut fd_generics = generics.clone();
    add_trait_bounds(
        &mut fd_generics,
        quote!(#substrate::types::Flatten<#substrate::types::Direction>),
    );
    let (fd_imp, fd_ty, fd_where) = fd_generics.split_for_impl();

    let fields = data.as_ref().take_struct().unwrap();

    let mut bundle_where_clause = bundle_wher.cloned().unwrap_or(WhereClause {
        where_token: Where {
            span: Span::call_site(),
        },
        predicates: Default::default(),
    });
    for f in fields.iter() {
        let ty = &f.ty;
        bundle_where_clause.predicates.push(parse_quote!(
            #ty: #substrate::types::codegen::HasView<#bundle_primitive_ty>));
    }
    let mut has_bundle_kind_where_clause = bundle_where_clause.clone();
    let mut bundle_flat_len_where_clause = bundle_where_clause.clone();
    let mut bundle_flatten_where_clause = bundle_where_clause.clone();
    let mut bundle_unflatten_where_clause = bundle_where_clause.clone();
    for f in fields.iter() {
        let ty = &f.ty;
        has_bundle_kind_where_clause.predicates.push(parse_quote!(
            <#ty as #substrate::types::codegen::HasView<#bundle_primitive_ty>>::View: #substrate::types::HasBundleKind<BundleKind = <#ty as #substrate::types::HasBundleKind>::BundleKind>));
        bundle_flat_len_where_clause.predicates.push(parse_quote!(
            <#ty as #substrate::types::codegen::HasView<#bundle_primitive_ty>>::View: #substrate::types::FlatLen));
        bundle_flatten_where_clause.predicates.push(parse_quote!(
            <#ty as #substrate::types::codegen::HasView<#bundle_primitive_ty>>::View: #substrate::types::Flatten<#flatten_ty>));
        bundle_unflatten_where_clause.predicates.push(parse_quote!(
            <#ty as #substrate::types::codegen::HasView<#bundle_primitive_ty>>::View: #substrate::types::Unflatten<<#ty as #substrate::types::HasBundleKind>::BundleKind, #unflatten_source_ty>));
    }

    let mut data_len = Vec::new();
    let mut data_fields = Vec::new();
    let mut ty_fields = Vec::new();
    let mut construct_data_fields = Vec::new();
    let mut construct_io_ty_fields = Vec::new();
    let mut construct_ty_ty_fields = Vec::new();
    let mut construct_ty_fields = Vec::new();
    let mut flatten_dir_fields = Vec::new();
    let mut flatten_bundle_fields = Vec::new();
    let mut unflatten_bundle_fields = Vec::new();

    for (i, &f) in fields.iter().enumerate() {
        let field_ty = &f.ty;
        let field_vis = &f.vis;
        let field_ident = &f.ident;
        let attrs = &f.attrs;

        let FieldTokens {
            declare,
            refer,
            assign,
            temp,
            pretty_ident,
        } = field_tokens(fields.style, field_vis, attrs, i, field_ident);
        let FieldTokens {
            refer: refer_kind, ..
        } = field_tokens_with_referent(
            fields.style,
            field_vis,
            attrs,
            i,
            field_ident,
            parse_quote!(__substrate_data),
        );

        io_len.push(quote! {
            <#field_ty as #substrate::types::FlatLen>::len(&#refer)
        });
        name_fields.push(quote! {
                (#substrate::arcstr::literal!(::std::stringify!(#pretty_ident)), <<#field_ty as #substrate::types::HasBundleKind>::BundleKind as #substrate::types::HasNameTree>::names(&#refer))
            });
        flatten_dir_fields.push(quote! {
                <#field_ty as #substrate::types::Flatten<#substrate::types::Direction>>::flatten(&#refer, __substrate_output_sink);
        });
        data_len.push(quote! {
                <<#field_ty as #substrate::types::codegen::HasView<#bundle_primitive_ty>>::View as #substrate::types::FlatLen>::len(&#refer)
            });
        data_fields.push(quote! {
            #declare <#field_ty as #substrate::types::codegen::HasView<#bundle_primitive_ty>>::View,
        });
        ty_fields.push(quote! {
            #declare <#field_ty as #substrate::types::HasBundleKind>::BundleKind,
        });
        construct_io_ty_fields.push(quote! {
            #assign <#field_ty as #substrate::types::HasBundleKind>::kind(&#refer),
        });
        construct_ty_ty_fields.push(quote! {
            #assign <<#field_ty as #substrate::types::HasBundleKind>::BundleKind as #substrate::types::HasBundleKind>::kind(&#refer),
        });
        construct_data_fields.push(quote! {
            #assign #temp,
        });
        construct_ty_fields.push(quote! {
            #assign <<#field_ty as #substrate::types::codegen::HasView<#bundle_primitive_ty>>::View as #substrate::types::HasBundleKind>::kind(&#refer),
        });
        flatten_bundle_fields.push(quote! {
            <<#field_ty as #substrate::types::codegen::HasView<#bundle_primitive_ty>>::View as #substrate::types::Flatten<#flatten_ty>>::flatten(&#refer, __substrate_output_sink);
        });
        unflatten_bundle_fields.push(quote! {
            #assign <<#field_ty as #substrate::types::codegen::HasView<#bundle_primitive_ty>>::View as #substrate::types::Unflatten<<#field_ty as #substrate::types::HasBundleKind>::BundleKind, #unflatten_source_ty>>::unflatten(&#refer_kind, __substrate_source)?,
        });
    }

    // Return 0 from `FlatLen::len` if struct has no fields.
    if io_len.is_empty() {
        io_len.push(quote! { 0 });
    }

    // Return 0 from `FlatLen::len` if struct has no fields.
    if data_len.is_empty() {
        data_len.push(quote! { 0 });
    }

    let ty_body = struct_body(fields.style, true, quote!( #(#ty_fields)* ));
    let data_body = struct_body(fields.style, true, quote!( #(#data_fields)* ));
    let construct_io_ty_body =
        struct_body(fields.style, false, quote!( #(#construct_io_ty_fields)* ));
    let construct_ty_ty_body =
        struct_body(fields.style, false, quote!( #(#construct_ty_ty_fields)* ));
    let construct_ty_body = struct_body(fields.style, false, quote!( #(#construct_ty_fields)* ));
    let construct_unflatten_body =
        struct_body(fields.style, false, quote!( #(#unflatten_bundle_fields)* ));

    let flatten_dir_impl = flatten_dir.then(|| {
        quote! {
            impl #fd_imp #substrate::types::Flatten<#substrate::types::Direction> for #ident #fd_ty #fd_where {
                fn flatten<E>(&self, __substrate_output_sink: &mut E)
                where
                    E: ::std::iter::Extend<#substrate::types::Direction> {
                    #( #flatten_dir_fields )*
                }
            }
        }
    });

    quote! {
        #(#attrs)*
        #[derive(Clone, Debug, PartialEq, Eq)]
        #vis struct #bundle_type_ident #generics #ty_body
        #(#attrs)*
        #vis struct #bundle_ident #bundle_generics #bundle_where_clause #data_body

        impl #substrate::types::HasBundleKind for #ident {
            type BundleKind = #bundle_type_ident;

            fn kind(&self) -> Self::BundleKind {
                #bundle_type_ident #construct_io_ty_body
            }
        }

        impl #substrate::types::HasBundleKind for #bundle_type_ident {
            type BundleKind = #bundle_type_ident;

            fn kind(&self) -> Self::BundleKind {
                #bundle_type_ident #construct_ty_ty_body
            }
        }

        impl #flatlen_imp #substrate::types::FlatLen for #ident #flatlen_ty #flatlen_wher {
            fn len(&self) -> usize {
                #( #io_len )+*
            }
        }

        impl #hnt_imp #substrate::types::HasNameTree for #ident #hnt_ty #hnt_wher {
            fn names(&self) -> ::std::option::Option<::std::vec::Vec<#substrate::types::NameTree>> {
                <#bundle_type_ident as #substrate::types::HasNameTree>::names(&<#ident as #substrate::types::HasBundleKind>::kind(self))
            }
        }

        #flatten_dir_impl

        impl #hnt_imp #substrate::types::HasNameTree for #bundle_type_ident #hnt_ty #hnt_wher {
            fn names(&self) -> ::std::option::Option<::std::vec::Vec<#substrate::types::NameTree>> {
                let v: ::std::vec::Vec<#substrate::types::NameTree> = [ #( #name_fields ),* ]
                     .into_iter()
                     .filter_map(|(frag, children)| children.map(|c| #substrate::types::NameTree::new(frag, c)))
                     .collect();
                if v.len() == 0 { None } else { Some(v) }
            }
        }

        // TODO: Fix where clause
        impl #hnt_imp #substrate::types::codegen::ViewSource for #ident #hnt_ty {
            type Source = #substrate::types::codegen::DerivedView;
        }

        impl #hnt_imp #substrate::types::codegen::ViewSource for #bundle_type_ident #hnt_ty {
            type Source = #substrate::types::codegen::DirectView;
        }

        impl #bundle_imp #substrate::types::codegen::HasViewImpl<#bundle_primitive_ty, #substrate::types::codegen::DerivedView> for #ident #hnt_ty  where #bundle_type_ident: #substrate::types::codegen::HasView<#bundle_primitive_ty>{
            type View = <#bundle_type_ident as #substrate::types::codegen::HasView<#bundle_primitive_ty>>::View;
        }

        impl #bundle_imp #substrate::types::HasBundleKind for #bundle_ident #bundle_ty #has_bundle_kind_where_clause {
            type BundleKind = #bundle_type_ident;

            fn kind(&self) -> <Self as #substrate::types::HasBundleKind>::BundleKind {
                #bundle_type_ident #construct_ty_body
            }
        }

        impl #bundle_imp #substrate::types::FlatLen for #bundle_ident #bundle_ty #bundle_flat_len_where_clause {
            fn len(&self) -> usize {
                #( #data_len )+*
            }
        }

        impl #bundle_flatten_imp #substrate::types::Flatten<#flatten_ty> for #bundle_ident #bundle_ty #bundle_flatten_where_clause {
            fn flatten<E>(&self, __substrate_output_sink: &mut E)
            where
                E: Extend<#flatten_ty>,
            {
                #( #flatten_bundle_fields )*
            }
        }

        impl #bundle_unflatten_imp #substrate::types::Unflatten<#bundle_type_ident, #unflatten_source_ty> for #bundle_ident #bundle_ty #bundle_unflatten_where_clause {
            fn unflatten<__substrate_I>(__substrate_data: &#bundle_type_ident, __substrate_source: &mut __substrate_I) -> Option<Self>
            where
                __substrate_I: Iterator<Item = #unflatten_source_ty> {
                ::std::option::Option::Some(#bundle_ident #construct_unflatten_body)
            }
        }
    }
}
