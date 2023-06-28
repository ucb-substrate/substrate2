use darling::{ast, FromDeriveInput};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::Field;

#[derive(Debug, FromDeriveInput)]
#[darling(supports(struct_any))]
pub struct IoInputReceiver {
    ident: syn::Ident,
    generics: syn::Generics,
    data: ast::Data<(), Field>,
}

impl ToTokens for IoInputReceiver {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let IoInputReceiver {
            ref ident,
            ref generics,
            ref data,
        } = *self;

        let (imp, ty, wher) = generics.split_for_impl();
        let fields = data
            .as_ref()
            .take_struct()
            .expect("Should never be enum")
            .fields;

        let mut ty_len = Vec::new();
        let mut data_len = Vec::new();
        let mut layout_data_len = Vec::new();
        let mut layout_builder_len = Vec::new();
        let mut flatten_dir_fields = Vec::new();
        let mut data_fields = Vec::new();
        let mut layout_data_fields = Vec::new();
        let mut layout_builder_fields = Vec::new();
        let mut transformed_layout_data_fields = Vec::new();
        let mut instantiate_fields = Vec::new();
        let mut construct_data_fields = Vec::new();
        let mut flatten_node_fields = Vec::new();
        let mut flatten_port_geometry_fields = Vec::new();
        let mut create_builder_fields = Vec::new();
        let mut transformed_view_fields = Vec::new();
        let mut build_data_fields = Vec::new();
        let mut name_fields = Vec::new();

        let data_ident = format_ident!("{}Data", ident);

        let layout_data_ident = format_ident!("{}Layout", ident);
        let layout_builder_ident = format_ident!("{}LayoutBuilder", ident);
        let transformed_layout_data_ident = format_ident!("Transformed{}Layout", ident);

        for f in fields {
            let field_ident = f
                .ident
                .as_ref()
                .expect("could not find identifier for field");
            let field_ty = &f.ty;

            ty_len.push(quote! {
                <#field_ty as ::substrate::io::FlatLen>::len(&self.#field_ident)
            });
            data_len.push(quote! {
                <<#field_ty as ::substrate::io::SchematicType>::Data as ::substrate::io::FlatLen>::len(&self.#field_ident)
            });
            layout_data_len.push(quote! {
                <<#field_ty as ::substrate::io::LayoutType>::Data as ::substrate::io::FlatLen>::len(&self.#field_ident)
            });
            layout_builder_len.push(quote! {
                <<#field_ty as ::substrate::io::LayoutType>::Builder as ::substrate::io::FlatLen>::len(&self.#field_ident)
            });
            flatten_dir_fields.push(quote! {
                <#field_ty as ::substrate::io::Flatten<::substrate::io::Direction>>::flatten(&self.#field_ident, __substrate_output_sink);
            });
            data_fields.push(quote! {
                pub #field_ident: <#field_ty as ::substrate::io::SchematicType>::Data,
            });
            layout_data_fields.push(quote! {
                pub #field_ident: <#field_ty as ::substrate::io::LayoutType>::Data,
            });
            layout_builder_fields.push(quote! {
                pub #field_ident: <#field_ty as ::substrate::io::LayoutType>::Builder,
            });
            transformed_layout_data_fields.push(quote! {
                pub #field_ident: ::substrate::geometry::transform::Transformed<'a, <#field_ty as ::substrate::io::LayoutType>::Data>,
            });
            instantiate_fields.push(quote! {
                let (#field_ident, __substrate_node_ids) = <#field_ty as ::substrate::io::SchematicType>::instantiate(&self.#field_ident, __substrate_node_ids);
            });
            construct_data_fields.push(quote! {
                #field_ident,
            });
            flatten_node_fields.push(quote! {
                <<#field_ty as ::substrate::io::SchematicType>::Data as ::substrate::io::Flatten<::substrate::io::Node>>::flatten(&self.#field_ident, __substrate_output_sink);
            });
            flatten_port_geometry_fields.push(quote! {
                <<#field_ty as ::substrate::io::LayoutType>::Data as ::substrate::io::Flatten<::substrate::io::PortGeometry>>::flatten(&self.#field_ident, __substrate_output_sink);
            });
            create_builder_fields.push(quote! {
                #field_ident: self.#field_ident.builder(),
            });
            transformed_view_fields.push(quote! {
                #field_ident: ::substrate::geometry::transform::HasTransformedView::transformed_view(&self.#field_ident, trans),
            });
            build_data_fields.push(quote! {
                #field_ident: self.#field_ident.build()?,
            });
            name_fields.push(quote! {
                (::substrate::arcstr::literal!(::std::stringify!(#field_ident)), <#field_ty as ::substrate::io::HasNameTree>::names(&self.#field_ident))
            });
        }

        // Return 0 from `FlatLen::len` if struct has no fields.
        if ty_len.is_empty() {
            ty_len.push(quote! { 0 });
        }

        if data_len.is_empty() {
            data_len.push(quote! { 0 });
        }

        if layout_data_len.is_empty() {
            layout_data_len.push(quote! { 0 });
        }

        if layout_builder_len.is_empty() {
            layout_builder_len.push(quote! { 0 });
        }

        tokens.extend(quote! {
            impl #imp ::substrate::io::Io for #ident #ty #wher { }
            impl #imp ::substrate::io::FlatLen for #ident #ty #wher {
                fn len(&self) -> usize {
                    #( #ty_len )+*
                }
            }
            impl #imp ::substrate::io::Flatten<::substrate::io::Direction> for #ident #ty #wher {
                fn flatten<E>(&self, __substrate_output_sink: &mut E)
                where
                    E: ::std::iter::Extend<::substrate::io::Direction> {
                    #( #flatten_dir_fields )*
                }
            }
            pub struct #data_ident #ty #wher {
                #( #data_fields )*
            }
            impl #imp ::substrate::io::FlatLen for #data_ident #ty #wher {
                fn len(&self) -> usize {
                    #( #data_len )+*
                }
            }
            impl #imp ::substrate::io::Flatten<::substrate::io::Node> for #data_ident #ty #wher {
                fn flatten<E>(&self, __substrate_output_sink: &mut E)
                where
                    E: ::std::iter::Extend<::substrate::io::Node> {
                    #( #flatten_node_fields )*
                }
            }
            impl #imp ::substrate::io::SchematicType for #ident #ty #wher {
                type Data = #data_ident;
                fn instantiate<'n>(&self, __substrate_node_ids: &'n [::substrate::io::Node]) -> (Self::Data, &'n [::substrate::io::Node]) {
                    #( #instantiate_fields )*
                    (#data_ident { #( #construct_data_fields )* }, __substrate_node_ids)
                }
            }

            impl #imp ::substrate::io::HasNameTree for #ident #ty #wher {
                fn names(&self) -> ::std::option::Option<::std::vec::Vec<::substrate::io::NameTree>> {
                    if <Self as ::substrate::io::FlatLen>::len(&self) == 0 { return ::std::option::Option::None; }
                    ::std::option::Option::Some([ #( #name_fields ),* ]
                         .into_iter()
                         .filter_map(|(frag, children)| children.map(|c| ::substrate::io::NameTree::new(frag, c)))
                         .collect()
                    )
                }
            }

            impl #imp ::substrate::io::LayoutType for #ident #ty #wher {
                type Data = #layout_data_ident;
                type Builder = #layout_builder_ident;

                fn builder(&self) -> Self::Builder {
                    Self::Builder {
                        #( #create_builder_fields )*
                    }
                }
            }

            pub struct #layout_data_ident #ty #wher {
                #( #layout_data_fields )*
            }

            pub struct #layout_builder_ident #ty #wher {
                #( #layout_builder_fields )*
            }

            impl #imp ::substrate::io::FlatLen for #layout_data_ident #ty #wher {
                fn len(&self) -> usize {
                    #( #layout_data_len )+*
                }
            }

            impl #imp ::substrate::io::Flatten<::substrate::io::PortGeometry> for #layout_data_ident #ty #wher {
                fn flatten<E>(&self, __substrate_output_sink: &mut E)
                where
                    E: ::std::iter::Extend<::substrate::io::PortGeometry> {
                    #( #flatten_port_geometry_fields )*
                }
            }

            impl #imp ::substrate::io::FlatLen for #layout_builder_ident #ty #wher {
                fn len(&self) -> usize {
                    #( #layout_builder_len )+*
                }
            }

            // TODO: How to correctly handle generics?
            pub struct #transformed_layout_data_ident<'a> {
                #( #transformed_layout_data_fields )*
            }

            impl #imp ::substrate::geometry::transform::HasTransformedView for #layout_data_ident #ty #wher {
                type TransformedView<'a> = #transformed_layout_data_ident<'a>;

                fn transformed_view(
                    &self,
                    trans: ::substrate::geometry::transform::Transformation,
                ) -> Self::TransformedView<'_> {
                    Self::TransformedView {
                        #( #transformed_view_fields )*
                    }
                }
            }

            impl #imp ::substrate::io::LayoutDataBuilder<#layout_data_ident #ty> for #layout_builder_ident #ty #wher {
                fn build(self) -> ::substrate::error::Result<#layout_data_ident #ty> {
                    ::substrate::error::Result::Ok(#layout_data_ident {
                        #( #build_data_fields )*
                    })
                }
            }
        });
    }
}
