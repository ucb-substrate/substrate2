use darling::{ast, FromDeriveInput, FromField};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::parse_quote;
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
        ref attrs,
        ..
    } = *input;

    let substrate = substrate_ident();

    let mut hnt_generics = generics.clone();
    add_trait_bounds(&mut hnt_generics, quote!(#substrate::io::HasNameTree));

    let mut st_generics = generics.clone();
    add_trait_bounds(
        &mut st_generics,
        quote!(#substrate::io::schematic::HardwareType),
    );
    let (st_imp, st_ty, st_where) = st_generics.split_for_impl();

    let mut st_any_generics = st_generics.clone();
    add_trait_bounds(&mut st_any_generics, quote!(::std::any::Any));
    let (st_any_imp, st_any_ty, st_any_where) = st_any_generics.split_for_impl();

    let mut fd_generics = generics.clone();
    add_trait_bounds(
        &mut fd_generics,
        quote!(#substrate::io::Flatten<#substrate::io::Direction>),
    );
    let (fd_imp, fd_ty, fd_where) = fd_generics.split_for_impl();

    let lifetime: syn::GenericParam = parse_quote!('__substrate_derive_lifetime);
    let mut ref_generics = st_generics.clone();
    add_trait_bounds(&mut ref_generics, quote!(::std::any::Any));
    ref_generics.params.push(lifetime.clone());

    let mut idents = Vec::new();
    for param in &ref_generics.params {
        if let syn::GenericParam::Type(ref type_param) = *param {
            idents.push(type_param.ident.clone());
        }
    }
    let ref_wher = ref_generics.make_where_clause();
    for ident in idents {
        ref_wher
            .predicates
            .push(syn::parse_quote!(<#ident as substrate::io::schematic::HardwareType>::Bundle: #lifetime));
    }

    let (_imp, ty, _wher) = generics.split_for_impl();
    let fields = data.as_ref().take_struct().unwrap();

    let mut data_len = Vec::new();
    let mut terminal_data_len = Vec::new();
    let mut data_fields = Vec::new();
    let mut nested_view_fields = Vec::new();
    let mut terminal_view_fields = Vec::new();
    let mut nested_terminal_view_fields = Vec::new();
    let mut construct_data_fields = Vec::new();
    let mut construct_nested_view_fields = Vec::new();
    let mut construct_terminal_view_fields = Vec::new();
    let mut construct_nested_terminal_view_fields = Vec::new();
    let mut instantiate_fields = Vec::new();
    let mut flatten_dir_fields = Vec::new();
    let mut flatten_node_fields = Vec::new();
    let mut terminal_view_flatten_node_fields = Vec::new();
    let mut field_list_elems = Vec::new();
    let mut field_match_arms = Vec::new();

    let data_ident = format_ident!("{}Schematic", ident);
    let nested_view_ident = format_ident!("{}NestedSchematicView", ident);
    let terminal_view_ident = format_ident!("{}TerminalView", ident);
    let nested_terminal_view_ident = format_ident!("{}NestedTerminalView", ident);

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

        data_len.push(quote! {
                <<#field_ty as #substrate::io::schematic::HardwareType>::Bundle as #substrate::io::FlatLen>::len(&#refer)
            });
        terminal_data_len.push(quote! {
                <<<#field_ty as #substrate::io::schematic::HardwareType>::Bundle as #substrate::io::schematic::HasTerminalView>::TerminalView as #substrate::io::FlatLen>::len(&#refer)
            });
        data_fields.push(quote! {
            #declare <#field_ty as #substrate::io::schematic::HardwareType>::Bundle,
        });
        nested_view_fields.push(quote! {
                #declare #substrate::schematic::NestedView<<#field_ty as #substrate::io::schematic::HardwareType>::Bundle>,
        });
        terminal_view_fields.push(quote! {
                #declare #substrate::io::schematic::TerminalView<<#field_ty as #substrate::io::schematic::HardwareType>::Bundle>,
        });
        nested_terminal_view_fields.push(quote! {
                #declare <#substrate::io::schematic::TerminalView<<#field_ty as #substrate::io::schematic::HardwareType>::Bundle> as #substrate::schematic::HasNestedView>::NestedView,
        });
        construct_data_fields.push(quote! {
            #assign #temp,
        });
        construct_nested_view_fields.push(quote! {
                #assign <<#field_ty as #substrate::io::schematic::HardwareType>::Bundle as #substrate::schematic::HasNestedView>::nested_view(&#refer, parent),
        });
        construct_terminal_view_fields.push(quote! {
                #assign <<#field_ty as #substrate::io::schematic::HardwareType>::Bundle as #substrate::io::schematic::HasTerminalView>::terminal_view(cell, &#cell_io_refer, instance, &#instance_io_refer),
        });
        construct_nested_terminal_view_fields.push(quote! {
                #assign <<<#field_ty as #substrate::io::schematic::HardwareType>::Bundle as #substrate::io::schematic::HasTerminalView>::TerminalView as #substrate::schematic::HasNestedView>::nested_view(&#refer, parent),
        });
        instantiate_fields.push(quote! {
                let (#temp, __substrate_node_ids) = <#field_ty as #substrate::io::schematic::HardwareType>::instantiate(&#refer, __substrate_node_ids);
        });
        flatten_dir_fields.push(quote! {
                <#field_ty as #substrate::io::Flatten<#substrate::io::Direction>>::flatten(&#refer, __substrate_output_sink);
        });
        flatten_node_fields.push(quote! {
                <<#field_ty as #substrate::io::schematic::HardwareType>::Bundle as #substrate::io::Flatten<#substrate::io::schematic::Node>>::flatten(&#refer, __substrate_output_sink);
        });
        terminal_view_flatten_node_fields.push(quote! {
                <<<#field_ty as #substrate::io::schematic::HardwareType>::Bundle as #substrate::io::schematic::HasTerminalView>::TerminalView as #substrate::io::Flatten<#substrate::io::schematic::Node>>::flatten(&#refer, __substrate_output_sink);
        });
        field_list_elems
            .push(quote! { #substrate::arcstr::literal!(::std::stringify!(#pretty_ident)) });
        field_match_arms.push(quote! {
            ::std::stringify!(#pretty_ident) => ::std::option::Option::Some(<<#field_ty as #substrate::io::schematic::HardwareType>::Bundle as #substrate::io::Flatten<#substrate::io::schematic::Node>>::flatten_vec(&#refer)),
        });
    }

    // Return 0 from `FlatLen::len` if struct has no fields.
    if data_len.is_empty() {
        data_len.push(quote! { 0 });
    }

    let data_body = struct_body(fields.style, true, quote!( #(#data_fields)* ));
    let nested_view_body = struct_body(fields.style, true, quote!( #(#nested_view_fields)* ));
    let terminal_view_body = struct_body(fields.style, true, quote!( #(#terminal_view_fields)* ));
    let nested_terminal_view_body = struct_body(
        fields.style,
        true,
        quote!( #(#nested_terminal_view_fields)* ),
    );
    let construct_nested_view_body = struct_body(
        fields.style,
        false,
        quote!( #(#construct_nested_view_fields)* ),
    );
    let construct_terminal_view_body = struct_body(
        fields.style,
        false,
        quote!( #(#construct_terminal_view_fields)* ),
    );
    let construct_nested_terminal_view_body = struct_body(
        fields.style,
        false,
        quote!( #(#construct_nested_terminal_view_fields)* ),
    );
    let construct_data_body =
        struct_body(fields.style, false, quote!( #(#construct_data_fields)* ));

    quote! {
        #[derive(Clone)]
        #(#attrs)*
        #vis struct #data_ident #st_generics #data_body
        #(#attrs)*
        #vis struct #nested_view_ident #st_generics #nested_view_body
        #(#attrs)*
        #vis struct #terminal_view_ident #st_generics #terminal_view_body
        #(#attrs)*
        #vis struct #nested_terminal_view_ident #st_generics #nested_terminal_view_body

        impl #st_imp #substrate::io::FlatLen for #data_ident #st_ty #st_where {
            fn len(&self) -> usize {
                #( #data_len )+*
            }
        }

        impl #fd_imp #substrate::io::Flatten<#substrate::io::Direction> for #ident #fd_ty #fd_where {
            fn flatten<E>(&self, __substrate_output_sink: &mut E)
            where
                E: ::std::iter::Extend<#substrate::io::Direction> {
                #( #flatten_dir_fields )*
            }
        }

        impl #st_imp #substrate::io::Flatten<#substrate::io::schematic::Node> for #data_ident #st_ty #st_where {
            fn flatten<E>(&self, __substrate_output_sink: &mut E)
            where
                E: ::std::iter::Extend<#substrate::io::schematic::Node> {
                #( #flatten_node_fields )*
            }
        }

        impl #st_any_imp #substrate::schematic::HasNestedView for #data_ident #st_any_ty #st_any_where {
            type NestedView = #nested_view_ident #st_any_ty;

            fn nested_view(&self, parent: &#substrate::schematic::InstancePath) -> Self::NestedView {
                #nested_view_ident #construct_nested_view_body
            }
        }

        impl #st_any_imp #substrate::io::schematic::HasTerminalView for #data_ident #st_any_ty #st_any_where {
            type TerminalView = #terminal_view_ident #st_any_ty;

            fn terminal_view(cell: #substrate::schematic::CellId, cell_io: &Self, instance: #substrate::schematic::InstanceId, instance_io: &Self) -> Self::TerminalView {
                #terminal_view_ident #construct_terminal_view_body
            }
        }

        impl #st_any_imp #substrate::schematic::HasNestedView for #terminal_view_ident #st_any_ty #st_any_where {
            type NestedView = #nested_terminal_view_ident #st_any_ty;

            fn nested_view(&self, parent: &#substrate::schematic::InstancePath) -> Self::NestedView {
                #nested_terminal_view_ident #construct_nested_terminal_view_body
            }
        }

        impl #st_imp #substrate::io::FlatLen for #terminal_view_ident #st_ty #st_where {
            fn len(&self) -> usize {
                #( #terminal_data_len )+*
            }
        }

        impl #st_imp #substrate::io::Flatten<#substrate::io::schematic::Node> for #terminal_view_ident #st_ty #st_where {
            fn flatten<E>(&self, __substrate_output_sink: &mut E)
            where
                E: ::std::iter::Extend<#substrate::io::schematic::Node> {
                #( #terminal_view_flatten_node_fields )*
            }
        }

        impl #st_imp #substrate::io::schematic::Connect<#terminal_view_ident #st_ty> for #data_ident #st_ty #st_where {}
        impl #st_imp #substrate::io::schematic::Connect<&#terminal_view_ident #st_ty> for #data_ident #st_ty #st_where {}
        impl #st_imp #substrate::io::schematic::Connect<#terminal_view_ident #st_ty> for &#data_ident #st_ty #st_where {}
        impl #st_imp #substrate::io::schematic::Connect<&#terminal_view_ident #st_ty> for &#data_ident #st_ty #st_where {}
        impl #st_imp #substrate::io::schematic::Connect<#data_ident #st_ty> for #terminal_view_ident #st_ty #st_where {}
        impl #st_imp #substrate::io::schematic::Connect<&#data_ident #st_ty> for #terminal_view_ident #st_ty #st_where {}
        impl #st_imp #substrate::io::schematic::Connect<#data_ident #st_ty> for &#terminal_view_ident #st_ty #st_where {}
        impl #st_imp #substrate::io::schematic::Connect<&#data_ident #st_ty> for &#terminal_view_ident #st_ty #st_where {}

        impl #st_any_imp #substrate::io::schematic::HardwareType for #ident #st_any_ty #st_any_where {
            type Bundle = #data_ident #ty;
            fn instantiate<'n>(&self, __substrate_node_ids: &'n [#substrate::io::schematic::Node]) -> (Self::Bundle, &'n [#substrate::io::schematic::Node]) {
                #( #instantiate_fields )*
                #[allow(redundant_field_names)]
                (#data_ident #construct_data_body, __substrate_node_ids)
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

    let mut lt_generics = generics.clone();
    add_trait_bounds(
        &mut lt_generics,
        quote!(#substrate::io::layout::HardwareType),
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
        hbf_generics.make_where_clause().predicates.push(syn::parse_quote!(<#ident as #substrate::io::layout::HardwareType>::Builder: #substrate::io::layout::HierarchicalBuildFrom<#substrate::layout::element::NamedPorts>));
    }

    let (hbf_imp, hbf_ty, hbf_where) = hbf_generics.split_for_impl();

    let (_imp, ty, _wher) = generics.split_for_impl();
    let fields = data.as_ref().take_struct().unwrap();

    if let Some(layout_type) = layout_type {
        return quote! {
            impl #lt_any_imp #substrate::io::layout::HardwareType for #ident #lt_any_ty #lt_any_where {
                type Bundle = <#layout_type as #substrate::io::layout::HardwareType>::Bundle;
                type Builder = <#layout_type as #substrate::io::layout::HardwareType>::Builder;

                fn builder(&self) -> Self::Builder {
                    <#layout_type as #substrate::io::layout::HardwareType>::builder(&<#layout_type as #substrate::io::layout::CustomHardwareType<#ident>>::from_layout_type(self))
                }
            }
        };
    }

    let mut ty_len = Vec::new();
    let mut layout_data_len = Vec::new();
    let mut layout_data_fields = Vec::new();
    let mut layout_builder_fields = Vec::new();
    let mut transformed_layout_data_fields = Vec::new();
    let mut flatten_port_geometry_fields = Vec::new();
    let mut create_builder_fields = Vec::new();
    let mut transformed_view_fields = Vec::new();
    let mut build_data_fields = Vec::new();
    let mut hierarchical_build_from_fields = Vec::new();

    let layout_data_ident = format_ident!("{}Layout", ident);
    let layout_builder_ident = format_ident!("{}LayoutBuilder", ident);
    let transformed_layout_data_ident = format_ident!("{}TransformedLayout", ident);

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
            <#field_ty as #substrate::io::FlatLen>::len(&#refer)
        });
        layout_data_len.push(quote! {
                <<#field_ty as #substrate::io::layout::HardwareType>::Bundle as #substrate::io::FlatLen>::len(&#refer)
            });
        layout_data_fields.push(quote! {
            #declare <#field_ty as #substrate::io::layout::HardwareType>::Bundle,
        });
        layout_builder_fields.push(quote! {
            #declare <#field_ty as #substrate::io::layout::HardwareType>::Builder,
        });
        transformed_layout_data_fields.push(quote! {
                #declare #substrate::geometry::transform::Transformed<<#field_ty as #substrate::io::layout::HardwareType>::Bundle>,
            });
        flatten_port_geometry_fields.push(quote! {
                <<#field_ty as #substrate::io::layout::HardwareType>::Bundle as #substrate::io::Flatten<#substrate::io::layout::PortGeometry>>::flatten(&#refer, __substrate_output_sink);
            });
        if switch_type {
            create_builder_fields.push(quote! {
                    #assign <#field_ty as #substrate::io::layout::HardwareType>::builder(&<#field_ty as #substrate::io::layout::CustomHardwareType<#original_field_ty>>::from_layout_type(&#refer)),
                });
        } else {
            create_builder_fields.push(quote! {
                #assign <#field_ty as #substrate::io::layout::HardwareType>::builder(&#refer),
            });
        }
        transformed_view_fields.push(quote! {
                #assign #substrate::geometry::transform::HasTransformedView::transformed_view(&#refer, trans),
        });
        build_data_fields.push(quote! {
                #assign #substrate::io::layout::BundleBuilder::<<#field_ty as #substrate::io::layout::HardwareType>::Bundle>::build(#refer)?,
        });
        hierarchical_build_from_fields.push(quote! {
            #substrate::io::NameBuf::push(path, #substrate::arcstr::literal!(::std::stringify!(#pretty_ident)));
            #substrate::io::layout::HierarchicalBuildFrom::<#substrate::layout::element::NamedPorts>::build_from(&mut #refer, path, source);
            #substrate::io::NameBuf::pop(path);
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
    let layout_builder_body =
        struct_body(fields.style, true, quote! { #( #layout_builder_fields )* });
    let create_builder_body =
        struct_body(fields.style, false, quote! { #( #create_builder_fields )* });
    let transformed_layout_data_body = struct_body(
        fields.style,
        true,
        quote! { #( #transformed_layout_data_fields )* },
    );
    let transformed_view_body = struct_body(
        fields.style,
        false,
        quote! { #( #transformed_view_fields )* },
    );
    let build_layout_data_body =
        struct_body(fields.style, false, quote! { #( #build_data_fields )* });

    quote! {
        impl #lt_any_imp #substrate::io::layout::HardwareType for #ident #lt_any_ty #lt_any_where {
            type Bundle = #layout_data_ident #ty;
            type Builder = #layout_builder_ident #ty;

            fn builder(&self) -> Self::Builder {
                #layout_builder_ident #create_builder_body
            }
        }

        #(#attrs)*
        #vis struct #layout_data_ident #lt_generics #layout_data_body

        #(#attrs)*
        #vis struct #layout_builder_ident #lt_generics #layout_builder_body

        impl #lt_imp #substrate::io::FlatLen for #layout_data_ident #lt_ty #lt_where {
            fn len(&self) -> usize {
                #( #layout_data_len )+*
            }
        }

        impl #lt_imp #substrate::io::Flatten<#substrate::io::layout::PortGeometry> for #layout_data_ident #lt_ty #lt_where {
            fn flatten<E>(&self, __substrate_output_sink: &mut E)
            where
                E: ::std::iter::Extend<#substrate::io::layout::PortGeometry> {
                #( #flatten_port_geometry_fields )*
            }
        }

        #(#attrs)*
        #vis struct #transformed_layout_data_ident #lt_generics #transformed_layout_data_body

        impl #lt_any_imp #substrate::geometry::transform::HasTransformedView for #layout_data_ident #lt_any_ty #lt_any_where {
            type TransformedView = #transformed_layout_data_ident #lt_ty;

            fn transformed_view(
                &self,
                trans: #substrate::geometry::transform::Transformation,
            ) -> Self::TransformedView {
                #transformed_layout_data_ident #transformed_view_body
            }
        }

        impl #lt_any_imp #substrate::io::layout::BundleBuilder<#layout_data_ident #ty> for #layout_builder_ident #lt_any_ty #lt_any_where {
            fn build(self) -> #substrate::error::Result<#layout_data_ident #ty> {
                #substrate::error::Result::Ok(#layout_data_ident #build_layout_data_body)
            }
        }

        impl #hbf_imp #substrate::io::layout::HierarchicalBuildFrom<#substrate::layout::element::NamedPorts> for #layout_builder_ident #hbf_ty #hbf_where {
            fn build_from(&mut self, path: &mut #substrate::io::NameBuf, source: &#substrate::layout::element::NamedPorts) {
                #(#hierarchical_build_from_fields)*
            }
        }
    }
}

pub(crate) fn io_core_impl(input: &IoInputReceiver) -> TokenStream {
    let substrate = substrate_ident();
    let IoInputReceiver {
        ref ident,
        ref generics,
        ref data,
        ..
    } = *input;

    let mut hnt_generics = generics.clone();
    add_trait_bounds(&mut hnt_generics, quote!(#substrate::io::HasNameTree));
    add_trait_bounds(&mut hnt_generics, quote!(#substrate::io::FlatLen));

    let mut io_generics = generics.clone();
    add_trait_bounds(
        &mut io_generics,
        quote!(#substrate::io::schematic::HardwareType),
    );
    add_trait_bounds(
        &mut io_generics,
        quote!(#substrate::io::layout::HardwareType),
    );
    add_trait_bounds(&mut io_generics, quote!(#substrate::io::Directed));

    let mut flatlen_generics = generics.clone();
    add_trait_bounds(&mut flatlen_generics, quote!(#substrate::io::FlatLen));

    let (hnt_imp, hnt_ty, hnt_wher) = hnt_generics.split_for_impl();
    let (flatlen_imp, flatlen_ty, flatlen_wher) = flatlen_generics.split_for_impl();
    let fields = data.as_ref().take_struct().unwrap();

    let mut ty_len = Vec::new();
    let mut name_fields = Vec::new();

    for (i, &f) in fields.iter().enumerate() {
        let FieldTokens {
            refer,
            pretty_ident,
            ..
        } = field_tokens(fields.style, &f.vis, &f.attrs, i, &f.ident);

        let field_ty = &f.ty;

        ty_len.push(quote! {
            <#field_ty as #substrate::io::FlatLen>::len(&#refer)
        });
        name_fields.push(quote! {
                (#substrate::arcstr::literal!(::std::stringify!(#pretty_ident)), <#field_ty as #substrate::io::HasNameTree>::names(&#refer))
            });
    }

    // Return 0 from `FlatLen::len` if struct has no fields.
    if ty_len.is_empty() {
        ty_len.push(quote! { 0 });
    }

    quote! {
        impl #flatlen_imp #substrate::io::FlatLen for #ident #flatlen_ty #flatlen_wher {
            fn len(&self) -> usize {
                #( #ty_len )+*
            }
        }

        impl #hnt_imp #substrate::io::HasNameTree for #ident #hnt_ty #hnt_wher {
            fn names(&self) -> ::std::option::Option<::std::vec::Vec<#substrate::io::NameTree>> {
                if <Self as #substrate::io::FlatLen>::len(&self) == 0 { return ::std::option::Option::None; }
                ::std::option::Option::Some([ #( #name_fields ),* ]
                     .into_iter()
                     .filter_map(|(frag, children)| children.map(|c| #substrate::io::NameTree::new(frag, c)))
                     .collect()
                )
            }
        }
    }
}
