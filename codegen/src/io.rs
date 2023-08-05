use crate::derive::struct_body;
use crate::derive::FieldTokens;
use crate::substrate_ident;
use darling::{ast, FromDeriveInput, FromField};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::parse_quote;

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

    let lifetime: syn::GenericParam = parse_quote!('__substrate_derive_lifetime);
    let mut ref_generics = generics.clone();
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
            .push(syn::parse_quote!(<#ident as substrate::io::SchematicType>::Data: #lifetime));
    }

    let (imp, ty, wher) = generics.split_for_impl();
    let (_ref_imp, ref_ty, ref_wher) = ref_generics.split_for_impl();
    let fields = data.as_ref().take_struct().unwrap();

    let mut data_len = Vec::new();
    let mut data_fields = Vec::new();
    let mut nested_view_fields = Vec::new();
    let mut terminal_view_fields = Vec::new();
    let mut construct_data_fields = Vec::new();
    let mut construct_nested_view_fields = Vec::new();
    let mut construct_terminal_view_fields = Vec::new();
    let mut instantiate_fields = Vec::new();
    let mut flatten_dir_fields = Vec::new();
    let mut flatten_node_fields = Vec::new();
    let mut field_list_elems = Vec::new();
    let mut field_match_arms = Vec::new();

    let data_ident = format_ident!("{}Schematic", ident);
    let nested_view_ident = format_ident!("{}NestedSchematicView", ident);
    let terminal_view_ident = format_ident!("{}TerminalView", ident);

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
        } = crate::derive::field_tokens(fields.style, field_vis, attrs, i, field_ident);

        data_len.push(quote! {
                <<#field_ty as #substrate::io::SchematicType>::Data as #substrate::io::FlatLen>::len(&#refer)
            });
        data_fields.push(quote! {
            #declare <#field_ty as #substrate::io::SchematicType>::Data,
        });
        nested_view_fields.push(quote! {
                #declare #substrate::schematic::NestedView<#lifetime, <#field_ty as #substrate::io::SchematicType>::Data>,
        });
        terminal_view_fields.push(quote! {
                #declare #substrate::io::TerminalView<#lifetime, <#field_ty as #substrate::io::SchematicType>::Data>,
        });
        construct_data_fields.push(quote! {
            #assign #temp,
        });
        construct_nested_view_fields.push(quote! {
                #assign <<#field_ty as #substrate::io::SchematicType>::Data as #substrate::schematic::HasNestedView>::nested_view(&#refer, parent),
        });
        construct_terminal_view_fields.push(quote! {
                #assign <<#field_ty as #substrate::io::SchematicType>::Data as #substrate::io::HasTerminalView>::terminal_view(&#refer, parent),
        });
        instantiate_fields.push(quote! {
                let (#temp, __substrate_node_ids) = <#field_ty as #substrate::io::SchematicType>::instantiate(&#refer, __substrate_node_ids);
        });
        flatten_dir_fields.push(quote! {
                <#field_ty as #substrate::io::Flatten<#substrate::io::Direction>>::flatten(&#refer, __substrate_output_sink);
        });
        flatten_node_fields.push(quote! {
                <<#field_ty as #substrate::io::SchematicType>::Data as #substrate::io::Flatten<#substrate::io::Node>>::flatten(&#refer, __substrate_output_sink);
        });

        field_list_elems
            .push(quote! { #substrate::arcstr::literal!(::std::stringify!(#pretty_ident)) });
        field_match_arms.push(quote! {
            ::std::stringify!(#pretty_ident) => ::std::option::Option::Some(<<#field_ty as #substrate::io::SchematicType>::Data as #substrate::io::Flatten<#substrate::io::Node>>::flatten_vec(&#refer)),
        });
    }

    // Return 0 from `FlatLen::len` if struct has no fields.
    if data_len.is_empty() {
        data_len.push(quote! { 0 });
    }

    let data_body = struct_body(fields.style, true, quote!( #(#data_fields)* ));
    let nested_view_body = struct_body(fields.style, true, quote!( #(#nested_view_fields)* ));
    let terminal_view_body = struct_body(fields.style, true, quote!( #(#terminal_view_fields)* ));
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
    let construct_data_body =
        struct_body(fields.style, false, quote!( #(#construct_data_fields)* ));

    quote! {
        #[derive(Clone)]
        #(#attrs)*
        #vis struct #data_ident #ty #wher #data_body
        #(#attrs)*
        #vis struct #nested_view_ident #ref_ty #ref_wher #nested_view_body
        #(#attrs)*
        #vis struct #terminal_view_ident #ref_ty #ref_wher #terminal_view_body

        impl #imp #substrate::io::FlatLen for #data_ident #ty #wher {
            fn len(&self) -> usize {
                #( #data_len )+*
            }
        }

        impl #imp #substrate::io::Flatten<#substrate::io::Direction> for #ident #ty #wher {
            fn flatten<E>(&self, __substrate_output_sink: &mut E)
            where
                E: ::std::iter::Extend<#substrate::io::Direction> {
                #( #flatten_dir_fields )*
            }
        }

        impl #imp #substrate::io::Flatten<#substrate::io::Node> for #data_ident #ty #wher {
            fn flatten<E>(&self, __substrate_output_sink: &mut E)
            where
                E: ::std::iter::Extend<#substrate::io::Node> {
                #( #flatten_node_fields )*
            }
        }

        impl #imp #substrate::schematic::HasNestedView for #data_ident #ty #wher {
            type NestedView<#lifetime> = #nested_view_ident #ref_ty;

            fn nested_view<#lifetime>(&#lifetime self, parent: &#substrate::schematic::InstancePath) -> Self::NestedView<#lifetime> {
                #nested_view_ident #construct_nested_view_body
            }
        }

        impl #imp #substrate::io::HasTerminalView for #data_ident #ty #wher {
            type TerminalView<#lifetime> = #terminal_view_ident #ref_ty;

            fn terminal_view<#lifetime>(&#lifetime self, parent: &#substrate::schematic::InstancePath) -> Self::TerminalView<#lifetime> {
                #terminal_view_ident #construct_terminal_view_body
            }
        }

        impl #imp #substrate::io::StructData for #data_ident #ty #wher {
            fn fields(&self) -> ::std::vec::Vec<#substrate::arcstr::ArcStr> {
                std::vec![#(#field_list_elems),*]
            }

            fn field_nodes(&self, name: &str) -> ::std::option::Option<::std::vec::Vec<#substrate::io::Node>> {
                match name {
                    #(#field_match_arms)*
                    _ => None,
                }
            }
        }

        impl #imp #substrate::io::SchematicType for #ident #ty #wher {
            type Data = #data_ident #ty;
            fn instantiate<'n>(&self, __substrate_node_ids: &'n [#substrate::io::Node]) -> (Self::Data, &'n [#substrate::io::Node]) {
                #( #instantiate_fields )*
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

    let lifetime: syn::GenericParam = parse_quote!('__substrate_derive_lifetime);
    let mut ref_generics = generics.clone();
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
            .push(syn::parse_quote!(<#ident as substrate::io::SchematicType>::Data: #lifetime));
    }

    let (imp, ty, wher) = generics.split_for_impl();
    let (_ref_imp, ref_ty, ref_wher) = ref_generics.split_for_impl();
    let fields = data.as_ref().take_struct().unwrap();

    if let Some(layout_type) = layout_type {
        return quote! {
            impl #imp #substrate::io::LayoutType for #ident #ty #wher {
                type Data = <#layout_type as #substrate::io::LayoutType>::Data;
                type Builder = <#layout_type as #substrate::io::LayoutType>::Builder;

                fn builder(&self) -> Self::Builder {
                    <#layout_type as #substrate::io::LayoutType>::builder(&<#layout_type as #substrate::io::CustomLayoutType<#ident>>::from_layout_type(self))
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
        } = crate::derive::field_tokens(fields.style, &f.vis, &f.attrs, i, &f.ident);

        ty_len.push(quote! {
            <#field_ty as #substrate::io::FlatLen>::len(&#refer)
        });
        layout_data_len.push(quote! {
                <<#field_ty as #substrate::io::LayoutType>::Data as #substrate::io::FlatLen>::len(&#refer)
            });
        layout_data_fields.push(quote! {
            #declare <#field_ty as #substrate::io::LayoutType>::Data,
        });
        layout_builder_fields.push(quote! {
            #declare <#field_ty as #substrate::io::LayoutType>::Builder,
        });
        transformed_layout_data_fields.push(quote! {
                #declare #substrate::geometry::transform::Transformed<#lifetime, <#field_ty as #substrate::io::LayoutType>::Data>,
            });
        flatten_port_geometry_fields.push(quote! {
                <<#field_ty as #substrate::io::LayoutType>::Data as #substrate::io::Flatten<#substrate::io::PortGeometry>>::flatten(&#refer, __substrate_output_sink);
            });
        if switch_type {
            create_builder_fields.push(quote! {
                    #assign <#field_ty as #substrate::io::LayoutType>::builder(&<#field_ty as #substrate::io::CustomLayoutType<#original_field_ty>>::from_layout_type(&#refer)),
                });
        } else {
            create_builder_fields.push(quote! {
                #assign <#field_ty as #substrate::io::LayoutType>::builder(&#refer),
            });
        }
        transformed_view_fields.push(quote! {
                #assign #substrate::geometry::transform::HasTransformedView::transformed_view(&#refer, trans),
        });
        build_data_fields.push(quote! {
                #assign #substrate::io::LayoutDataBuilder::<<#field_ty as #substrate::io::LayoutType>::Data>::build(#refer)?,
        });
        hierarchical_build_from_fields.push(quote! {
            #substrate::io::NameBuf::push(path, #substrate::arcstr::literal!(::std::stringify!(#pretty_ident)));
            #substrate::io::HierarchicalBuildFrom::<#substrate::layout::element::NamedPorts>::build_from(&mut #refer, path, source);
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
        impl #imp #substrate::io::LayoutType for #ident #ty #wher {
            type Data = #layout_data_ident #ty;
            type Builder = #layout_builder_ident #ty;

            fn builder(&self) -> Self::Builder {
                #layout_builder_ident #create_builder_body
            }
        }

        #(#attrs)*
        #vis struct #layout_data_ident #ty #wher #layout_data_body

        #(#attrs)*
        #vis struct #layout_builder_ident #ty #wher #layout_builder_body

        impl #imp #substrate::io::FlatLen for #layout_data_ident #ty #wher {
            fn len(&self) -> usize {
                #( #layout_data_len )+*
            }
        }

        impl #imp #substrate::io::Flatten<#substrate::io::PortGeometry> for #layout_data_ident #ty #wher {
            fn flatten<E>(&self, __substrate_output_sink: &mut E)
            where
                E: ::std::iter::Extend<#substrate::io::PortGeometry> {
                #( #flatten_port_geometry_fields )*
            }
        }

        #(#attrs)*
        #vis struct #transformed_layout_data_ident #ref_ty #ref_wher #transformed_layout_data_body

        impl #imp #substrate::geometry::transform::HasTransformedView for #layout_data_ident #ty #wher {
            type TransformedView<#lifetime> = #transformed_layout_data_ident #ref_ty;

            fn transformed_view(
                &self,
                trans: #substrate::geometry::transform::Transformation,
            ) -> Self::TransformedView<'_> {
                #transformed_layout_data_ident #transformed_view_body
            }
        }

        impl #imp #substrate::io::LayoutDataBuilder<#layout_data_ident #ty> for #layout_builder_ident #ty #wher {
            fn build(self) -> #substrate::error::Result<#layout_data_ident #ty> {
                #substrate::error::Result::Ok(#layout_data_ident #build_layout_data_body)
            }
        }

        impl #imp #substrate::io::HierarchicalBuildFrom<#substrate::layout::element::NamedPorts> for #layout_builder_ident #ty #wher {
            fn build_from(&mut self, path: &mut #substrate::io::NameBuf, source: &#substrate::layout::element::NamedPorts) {
                #(#hierarchical_build_from_fields)*
            }
        }
    }
}

pub(crate) fn io_impl(input: &IoInputReceiver) -> TokenStream {
    let IoInputReceiver {
        ref ident,
        ref generics,
        ref data,
        ..
    } = *input;

    let (imp, ty, wher) = generics.split_for_impl();
    let fields = data.as_ref().take_struct().unwrap();

    let mut ty_len = Vec::new();
    let mut name_fields = Vec::new();

    let substrate = substrate_ident();

    for (i, &f) in fields.iter().enumerate() {
        let FieldTokens {
            refer,
            pretty_ident,
            ..
        } = crate::derive::field_tokens(fields.style, &f.vis, &f.attrs, i, &f.ident);

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
        impl #imp #substrate::io::FlatLen for #ident #ty #wher {
            fn len(&self) -> usize {
                #( #ty_len )+*
            }
        }

        impl #imp #substrate::io::HasNameTree for #ident #ty #wher {
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
