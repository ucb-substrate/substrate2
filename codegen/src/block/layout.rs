use darling::ast::{Fields, Style};
use darling::{ast, FromDeriveInput, FromField, FromMeta, FromVariant};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use crate::substrate_ident;
use type_dispatch::derive::{add_trait_bounds, struct_body};

#[derive(Debug, FromDeriveInput)]
#[darling(supports(struct_any, enum_any), forward_attrs(allow, doc, cfg))]
pub struct DataInputReceiver {
    ident: syn::Ident,
    generics: syn::Generics,
    data: ast::Data<DataVariant, DataField>,
    vis: syn::Visibility,
    attrs: Vec<syn::Attribute>,
}

#[derive(Debug, FromVariant)]
#[darling(forward_attrs(allow, doc, cfg))]
#[allow(dead_code)]
pub struct DataVariant {
    ident: syn::Ident,
    fields: Fields<DataField>,
    attrs: Vec<syn::Attribute>,
}

#[derive(Debug, FromField)]
#[darling(attributes(substrate), forward_attrs(allow, doc, cfg))]
pub struct DataField {
    ident: Option<syn::Ident>,
    vis: syn::Visibility,
    ty: syn::Type,
    attrs: Vec<syn::Attribute>,
}

fn transform_variant_decl(variant: &DataVariant) -> TokenStream {
    let DataVariant {
        ref ident,
        ref fields,
        ..
    } = variant;
    let decls = fields
        .iter()
        .enumerate()
        .map(|(i, f)| transform_field_decl(i, f));
    match fields.style {
        Style::Unit => quote!(#ident,),
        Style::Tuple => quote!(#ident( #(#decls)* ),),
        Style::Struct => quote!(#ident { #(#decls)* },),
    }
}

fn tuple_ident(idx: usize) -> syn::Ident {
    format_ident!("__substrate_derive_field{idx}")
}

fn transform_variant_match_arm(
    transformed_ident: syn::Ident,
    variant: &DataVariant,
) -> TokenStream {
    let DataVariant {
        ref ident,
        ref fields,
        ..
    } = variant;
    let destructure = fields
        .iter()
        .enumerate()
        .map(|(i, f)| f.ident.clone().unwrap_or_else(|| tuple_ident(i)))
        .map(|i| quote!(ref #i));
    let assign = fields
        .iter()
        .enumerate()
        .map(|(i, f)| transform_field_assign(false, i, f));
    match fields.style {
        Style::Unit => quote!(Self::#ident => #transformed_ident::#ident,),
        Style::Tuple => {
            quote!(Self::#ident( #(#destructure),* ) => #transformed_ident::#ident( #(#assign)* ),)
        }
        Style::Struct => {
            quote!(Self::#ident { #(#destructure),* } => #transformed_ident::#ident { #(#assign)* },)
        }
    }
}

fn transform_field_decl(_idx: usize, field: &DataField) -> TokenStream {
    let DataField {
        ref ident,
        ref vis,
        ref ty,
        ref attrs,
    } = field;
    let substrate = substrate_ident();
    let field_ty = quote!(#substrate::geometry::transform::Transformed<#ty>);

    match ident {
        Some(ident) => {
            quote! {
                #(#attrs)*
                #vis #ident: #field_ty,
            }
        }
        None => {
            quote! {
                #(#attrs)*
                #vis #field_ty,
            }
        }
    }
}

fn transform_field_assign(use_self: bool, idx: usize, field: &DataField) -> TokenStream {
    let DataField {
        ref ident, ref ty, ..
    } = field;
    let substrate = substrate_ident();
    let tuple_ident = tuple_ident(idx);
    let idx = syn::Index::from(idx);

    let val = match (use_self, ident) {
        (true, Some(ident)) => quote!(&self.#ident),
        (true, None) => quote!(&self.#idx),
        (false, Some(ident)) => quote!(&#ident),
        (false, None) => quote!(&#tuple_ident),
    };

    let value = quote!(<#ty as #substrate::geometry::transform::HasTransformedView>::transformed_view(#val, __substrate_derive_transformation));

    match ident {
        Some(ident) => quote! { #ident: #value, },
        None => quote! { #value, },
    }
}

impl ToTokens for DataInputReceiver {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let substrate = substrate_ident();
        let DataInputReceiver {
            ref ident,
            ref generics,
            ref data,
            ref vis,
            ref attrs,
        } = *self;
        let mut generics = generics.clone();
        add_trait_bounds(
            &mut generics,
            quote!(#substrate::geometry::transform::HasTransformedView),
        );

        let (imp, ty, wher) = generics.split_for_impl();
        let transformed_ident = format_ident!("{}TransformedView", ident);

        let expanded = match data {
            ast::Data::Struct(ref fields) => {
                let decls = fields
                    .iter()
                    .enumerate()
                    .map(|(i, f)| transform_field_decl(i, f));
                let assignments = fields
                    .iter()
                    .enumerate()
                    .map(|(i, f)| transform_field_assign(true, i, f));
                let retval = match fields.style {
                    Style::Unit => quote!(#transformed_ident),
                    Style::Tuple => quote!(#transformed_ident( #(#assignments)* )),
                    Style::Struct => quote!(#transformed_ident { #(#assignments)* }),
                };

                let body = struct_body(fields.style, true, quote! {#( #decls )*});

                quote! {
                    #(#attrs)*
                    #vis struct #transformed_ident #generics #body

                    impl #imp #substrate::geometry::transform::HasTransformedView for #ident #ty #wher {
                        type TransformedView = #transformed_ident #ty;

                        fn transformed_view(
                            &self,
                            __substrate_derive_transformation: #substrate::geometry::transform::Transformation,
                        ) -> Self::TransformedView {
                            #retval
                        }
                    }
                }
            }
            ast::Data::Enum(ref variants) => {
                let decls = variants.iter().map(transform_variant_decl);
                let arms = variants
                    .iter()
                    .map(|v| transform_variant_match_arm(transformed_ident.clone(), v));
                quote! {
                    #(#attrs)*
                    #vis enum #transformed_ident #generics {
                        #( #decls )*
                    }
                    impl #imp #substrate::geometry::transform::HasTransformedView for #ident #ty #wher {
                        type TransformedView = #transformed_ident #ty;

                        fn transformed_view(
                            &self,
                            __substrate_derive_transformation: #substrate::geometry::transform::Transformation,
                        ) -> Self::TransformedView {
                            match self {
                                #(#arms)*
                            }
                        }
                    }
                }
            }
        };

        tokens.extend(quote! {
            #expanded
        });
    }
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(substrate), supports(any), allow_unknown_fields)]
pub struct HasLayoutInputReceiver {
    ident: syn::Ident,
    generics: syn::Generics,
    #[darling(multiple)]
    layout: Vec<LayoutHardMacro>,
}

#[derive(Debug, FromMeta)]
pub struct LayoutHardMacro {
    source: syn::Expr,
    fmt: darling::util::SpannedValue<String>,
    pdk: syn::Type,
    name: String,
}

impl ToTokens for HasLayoutInputReceiver {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let substrate = substrate_ident();
        let HasLayoutInputReceiver {
            ref ident,
            ref generics,
            ref layout,
            ..
        } = *self;

        let (imp, ty, wher) = generics.split_for_impl();

        let has_layout = quote! {
            impl #imp #substrate::layout::ExportsLayoutData for #ident #ty #wher {
                type LayoutData = ();
            }
        };

        let has_layout_impls = layout.iter().map(|layout| {
            let LayoutHardMacro { source, fmt, pdk, name } = layout;

            // The raw_cell token stream must create an Arc<RawCell>.
            // The token stream has access to source.
            let raw_cell = match fmt.as_str() {
                "gds" => quote! {
                    cell.ctx.read_gds_cell(source, #name)?
                },
                fmtstr => proc_macro_error::abort!(fmt.span(), "unsupported layout hard macro format: `{}`", fmtstr),
            };

            quote! {
                impl #imp #substrate::layout::Layout<#pdk> for #ident #ty #wher {
                    fn layout(
                        &self,
                        io: &mut <<Self as #substrate::block::Block>::Io as #substrate::io::layout::HardwareType>::Builder,
                        cell: &mut #substrate::layout::CellBuilder<#pdk>,
                    ) -> #substrate::error::Result<Self::LayoutData> {

                        let source = { #source };

                        let raw_cell = { #raw_cell };

                        #substrate::io::layout::HierarchicalBuildFrom::<#substrate::layout::element::NamedPorts>::build_from_top(io, raw_cell.port_map());
                        let inst = #substrate::layout::element::RawInstance::new(raw_cell, #substrate::geometry::transform::Transformation::default());
                        cell.draw(inst)?;

                        Ok(())
                    }
                }
            }
        });

        let expanded = quote! {
            #has_layout

            #(#has_layout_impls)*
        };

        tokens.extend(quote! {
            #expanded
        });
    }
}
