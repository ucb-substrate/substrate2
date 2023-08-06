use darling::ast::{Fields, Style};
use darling::{ast, FromDeriveInput, FromField, FromMeta, FromVariant};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::parse_quote;

use crate::substrate_ident;
use type_dispatch::derive::{add_trait_bounds, struct_body};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(substrate), supports(any), forward_attrs(allow, doc, cfg))]
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
    #[darling(default)]
    nested: bool,
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
        nested,
    } = field;
    let substrate = substrate_ident();
    let field_ty = if *nested {
        quote!(#substrate::schematic::NestedView<'__substrate_derive_lifetime, #ty>)
    } else {
        quote!(&'__substrate_derive_lifetime #ty)
    };

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
        ref ident,
        ref ty,
        nested,
        ..
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

    let value = if *nested {
        quote!(<#ty as #substrate::schematic::HasNestedView>::nested_view(#val, __substrate_derive_parent))
    } else {
        quote!(#val)
    };

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

        let generics = add_trait_bounds(
            quote!(#substrate::schematic::HasNestedView),
            generics.clone(),
        );
        let lifetime: syn::GenericParam = parse_quote!('__substrate_derive_lifetime);
        let mut ref_generics = generics.clone();
        ref_generics.params.push(lifetime.clone());

        let (imp, ty, wher) = generics.split_for_impl();
        let (_ref_imp, ref_ty, _ref_wher) = ref_generics.split_for_impl();
        let transformed_ident = format_ident!("{}NestedView", ident);

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
                    #vis struct #transformed_ident #ref_generics #body

                    impl #imp #substrate::schematic::HasNestedView for #ident #ty #wher {
                        type NestedView<#lifetime> = #transformed_ident #ref_ty;

                        fn nested_view<#lifetime>(
                            &#lifetime self,
                            __substrate_derive_parent: &#substrate::schematic::InstancePath,
                        ) -> Self::NestedView<#lifetime> {
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
                    #vis enum #transformed_ident #ref_generics {
                        #( #decls )*
                    }
                    impl #imp #substrate::schematic::HasNestedView for #ident #ty #wher {
                        type NestedView<#lifetime> = #transformed_ident #ref_ty;

                        fn nested_view<#lifetime>(
                            &#lifetime self,
                            __substrate_derive_parent: &#substrate::schematic::InstancePath,
                        ) -> Self::NestedView<#lifetime> {
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
#[darling(attributes(substrate), supports(any))]
pub struct HasSchematicInputReceiver {
    ident: syn::Ident,
    generics: syn::Generics,
    #[allow(unused)]
    io: darling::util::Ignored,
    #[darling(multiple)]
    #[allow(unused)]
    layout: Vec<darling::util::Ignored>,
    #[darling(multiple)]
    schematic: Vec<SchematicHardMacro>,
    #[darling(default)]
    #[allow(unused)]
    flatten: darling::util::Ignored,
}

#[derive(Debug, FromMeta)]
pub struct SchematicHardMacro {
    source: syn::Expr,
    fmt: darling::util::SpannedValue<String>,
    pdk: syn::Type,
    name: String,
}

impl ToTokens for HasSchematicInputReceiver {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let substrate = substrate_ident();
        let HasSchematicInputReceiver {
            ref ident,
            ref generics,
            ref schematic,
            ..
        } = *self;

        let (imp, ty, wher) = generics.split_for_impl();

        let has_schematic = quote! {
            impl #imp #substrate::schematic::HasSchematicData for #ident #ty #wher {
                type Data = ();
            }
        };

        let has_schematic_impls = schematic.iter().map(|schematic| {
            let SchematicHardMacro { source, fmt, pdk, name } = schematic;

            let parsed_to_scir = quote! {
                let mut conv = #substrate::spice::parser::conv::ScirConverter::new(::std::stringify!(#ident), &parsed.ast);

                for prim in cell.ctx.pdk.schematic_primitives() {
                    conv.blackbox(#substrate::arcstr::Substr::full(prim));
                }

                let lib = ::std::sync::Arc::new(conv.convert().unwrap());
                let cell_id = lib.cell_id_named(#name);

                (lib, cell_id)
            };

            // The SCIR token stream must create two variables:
            // * lib, of type Arc<scir::Library>
            // * cell_id, of type scir::CellId
            // The token stream has access to source.
            let scir = match fmt.as_str() {
                "spice" => quote! {
                    let parsed = #substrate::spice::parser::Parser::parse_file(source).unwrap();
                    #parsed_to_scir
                },
                "inline-spice" | "inline_spice" => quote! {
                    let parsed = #substrate::spice::parser::Parser::parse(source).unwrap();
                    #parsed_to_scir
                },
                fmtstr => proc_macro_error::abort!(fmt.span(), "unsupported schematic hard macro format: `{}`", fmtstr),
            };

            quote! {
                impl #imp #substrate::schematic::HasSchematic<#pdk> for #ident #ty #wher {
                    fn schematic(
                        &self,
                        io: &<<Self as #substrate::block::Block>::Io as #substrate::io::SchematicType>::Bundle,
                        cell: &mut #substrate::schematic::CellBuilder<#pdk, Self>,
                    ) -> #substrate::error::Result<Self::Data> {
                        use #substrate::pdk::Pdk;

                        let source = {
                            #source
                        };

                        let (lib, cell_id) = { #scir };

                        use #substrate::io::StructData;
                        let connections: ::std::collections::HashMap<#substrate::arcstr::ArcStr, ::std::vec::Vec<#substrate::io::Node>> =
                            ::std::collections::HashMap::from_iter(io.fields().into_iter().map(|f| {
                                let nodes = io.field_nodes(&f).unwrap();
                                (f, nodes)
                            }));

                        cell.add_primitive(#substrate::schematic::PrimitiveDevice::new(
                            #substrate::schematic::PrimitiveDeviceKind::ScirInstance {
                                lib,
                                cell: cell_id,
                                name: #substrate::arcstr::literal!(#name),
                                connections,
                            }
                        ));
                        Ok(())
                    }
                }
            }
        });

        let expanded = quote! {
            #has_schematic

            #(#has_schematic_impls)*
        };

        tokens.extend(quote! {
            #expanded
        });
    }
}
