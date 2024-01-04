use convert_case::{Case, Casing};
use darling::{ast, FromDeriveInput, FromMeta};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Attribute, Field, Generics, Visibility};

use crate::substrate_ident;

#[derive(Debug, FromDeriveInput)]
#[darling(supports(struct_any))]
pub struct LayersInputReceiver {
    ident: syn::Ident,
    generics: syn::Generics,
    data: ast::Data<(), Field>,
}

#[derive(Debug, FromDeriveInput)]
#[darling(supports(struct_any))]
pub struct LayerFamilyInputReceiver {
    ident: syn::Ident,
    generics: syn::Generics,
    data: ast::Data<(), Field>,
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(layer), supports(struct_any))]
pub struct LayerInputReceiver {
    ident: syn::Ident,
    generics: syn::Generics,
    #[darling(default)]
    name: Option<String>,
    #[darling(default)]
    gds: Option<String>,
}

#[derive(Debug, FromDeriveInput)]
#[darling(supports(struct_any))]
pub struct DerivedLayersInputReceiver {
    ident: syn::Ident,
    generics: syn::Generics,
    data: ast::Data<(), Field>,
}

#[derive(Debug, FromDeriveInput)]
#[darling(supports(struct_any))]
pub struct DerivedLayerFamilyInputReceiver {
    ident: syn::Ident,
    generics: syn::Generics,
    data: ast::Data<(), Field>,
}

#[derive(Debug, FromMeta)]
struct LayerData {
    name: Option<String>,
    gds: Option<String>,
    pin: Option<()>,
    label: Option<()>,
}

#[derive(Debug, FromMeta)]
struct LayerFamilyData {
    name: Option<String>,
    gds: Option<String>,
    primary: Option<()>,
    pin: Option<()>,
    label: Option<()>,
}

#[derive(Debug, FromMeta)]
struct DerivedLayerData {
    pin: Option<()>,
    label: Option<()>,
}

#[derive(Debug, FromMeta)]
struct DerivedLayerFamilyData {
    primary: Option<()>,
    pin: Option<()>,
    label: Option<()>,
}

fn layer_new(field_ty: &syn::Type) -> TokenStream {
    let substrate = substrate_ident();
    quote! { <#field_ty as #substrate::pdk::layers::Layer>::new(ctx) }
}

fn layer_info(field_ty: &impl ToTokens, info_ident: &impl ToTokens) -> TokenStream {
    let substrate = substrate_ident();
    quote! { <#field_ty as #substrate::pdk::layers::Layer>::info(&#info_ident) }
}

fn layer_family_info(field_ty: &syn::Type, field_ident: &syn::Ident) -> TokenStream {
    let substrate = substrate_ident();
    quote! { <#field_ty as #substrate::pdk::layers::LayerFamily>::info(&self.#field_ident) }
}

fn layer_family_new(field_ty: &syn::Type) -> TokenStream {
    let substrate = substrate_ident();
    quote! { <#field_ty as #substrate::pdk::layers::LayerFamily>::new(ctx) }
}

fn let_statement(assignee: &impl ToTokens, assignment: &impl ToTokens) -> TokenStream {
    quote! { let #assignee = #assignment }
}

fn impl_has_pin(
    generics: &Generics,
    ident: &syn::Ident,
    drawing: &impl ToTokens,
    pin: &impl ToTokens,
    label: &impl ToTokens,
) -> TokenStream {
    let substrate = substrate_ident();
    let (imp, ty, wher) = generics.split_for_impl();
    quote! {
        impl #imp #substrate::pdk::layers::HasPin for #ident #ty #wher {
            fn drawing(&self) -> #substrate::pdk::layers::LayerId {
                #drawing
            }
            fn pin(&self) -> #substrate::pdk::layers::LayerId {
                #pin
            }
            fn label(&self) -> #substrate::pdk::layers::LayerId {
                #label
            }
        }
    }
}

fn token_stream_option(contents: &Option<TokenStream>) -> TokenStream {
    contents
        .as_ref()
        .map(|contents| quote! { ::std::option::Option::Some(#contents) })
        .unwrap_or(quote! { ::std::option::Option::None })
}

fn derive_layer_with_attrs(
    field_ty: &syn::Type,
    name: &Option<String>,
    gds: &Option<String>,
    vis: &Visibility,
    attrs: &Vec<Attribute>,
) -> TokenStream {
    let substrate = substrate_ident();
    let mut layer_attrs = Vec::new();
    if let Some(name) = name {
        layer_attrs.push(quote!(name = #name));
    }
    if let Some(gds) = gds {
        layer_attrs.push(quote!(gds = #gds));
    }

    quote! {
        #[derive(#substrate::pdk::layers::Layer, ::std::clone::Clone, ::std::marker::Copy, ::std::fmt::Debug, ::std::cmp::Eq, ::std::cmp::PartialEq)]
        #[layer(#( #layer_attrs ),*)]
        #(#attrs)*
        #vis struct #field_ty(#substrate::pdk::layers::LayerId);
    }
}

fn impl_layer_family(
    generics: &syn::Generics,
    ident: &impl ToTokens,
    new_body: &impl ToTokens,
    layer_infos: &Vec<TokenStream>,
    primary: &impl ToTokens,
    pin: &Option<TokenStream>,
    label: &Option<TokenStream>,
) -> TokenStream {
    let substrate = substrate_ident();
    let (imp, ty, wher) = generics.split_for_impl();
    let pin = token_stream_option(pin);
    let label = token_stream_option(label);
    quote! {
        impl #imp #substrate::pdk::layers::LayerFamily for #ident #ty #wher {
            fn new(ctx: &mut #substrate::pdk::layers::LayerContext) -> Self {
                #new_body
            }

            fn info(&self) -> #substrate::pdk::layers::LayerFamilyInfo {
                #substrate::pdk::layers::LayerFamilyInfo {
                    layers: ::std::vec![ #( #layer_infos ),* ],
                    primary: #primary,
                    pin: #pin,
                    label: #label,
                }
            }
        }
    }
}

fn impl_as_ref_layer_id(
    generics: &syn::Generics,
    ident: &impl ToTokens,
    access: &impl ToTokens,
) -> TokenStream {
    let substrate = substrate_ident();
    let (imp, ty, wher) = generics.split_for_impl();
    quote! {
        impl #imp ::std::convert::AsRef<#substrate::pdk::layers::LayerId> for #ident #ty #wher {
            fn as_ref(&self) -> &#substrate::pdk::layers::LayerId {
                &#access
            }
        }
    }
}

fn impl_deref_layer_id(
    generics: &syn::Generics,
    ident: &impl ToTokens,
    access: &impl ToTokens,
) -> TokenStream {
    let substrate = substrate_ident();
    let (imp, ty, wher) = generics.split_for_impl();
    quote! {
        impl #imp ::std::ops::Deref for #ident #ty #wher {
            type Target = #substrate::pdk::layers::LayerId;
            fn deref(&self) -> &Self::Target {
                &#access
            }
        }
    }
}

fn impl_layer(
    generics: &syn::Generics,
    ident: &syn::Ident,
    name: &Option<String>,
    gds: &Option<String>,
) -> TokenStream {
    let substrate = substrate_ident();
    let (imp, ty, wher) = generics.split_for_impl();

    let name = name
        .clone()
        .unwrap_or(ident.to_string().to_case(Case::Snake));
    let gds = token_stream_option(
        &gds.as_ref()
            .and_then(|gds| {
                gds.split_once('/').map(|(a, b)| {
                    (
                        a.parse::<u16>().expect("failed to parse gds layer"),
                        b.parse::<u16>().expect("failed to parse gds data type"),
                    )
                })
            })
            .map(|(a, b)| quote! {#substrate::pdk::layers::GdsLayerSpec(#a, #b)}),
    );

    quote! {
        impl #imp #substrate::pdk::layers::Layer for #ident #ty #wher {
            fn new(ctx: &mut #substrate::pdk::layers::LayerContext) -> Self {
                Self(ctx.new_layer())
            }

            fn info(&self) -> #substrate::pdk::layers::LayerInfo {
                #substrate::pdk::layers::LayerInfo {
                    id: self.0,
                    name: #substrate::arcstr::literal!(#name),
                    gds: #gds,
                }
            }
        }
    }
}

fn impl_derived_layer(
    field_ty: &impl ToTokens,
    vis: &Visibility,
    attrs: &Vec<Attribute>,
) -> TokenStream {
    let substrate = substrate_ident();
    quote! {
        #[derive(::std::clone::Clone, ::std::marker::Copy)]
        #(#attrs)*
        #vis struct #field_ty(#substrate::pdk::layers::LayerId);

        impl #field_ty {
            fn new(id: impl ::std::convert::AsRef<#substrate::pdk::layers::LayerId>) -> Self {
                Self(*id.as_ref())
            }
        }
    }
}

impl ToTokens for LayersInputReceiver {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let substrate = substrate_ident();
        let LayersInputReceiver {
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

        let mut field_init = Vec::new();
        let mut field_idents = Vec::new();
        let mut info_init = Vec::new();

        for f in fields {
            let field_ident = f
                .ident
                .as_ref()
                .expect("could not find identifier for field");
            let field_ty = &f.ty;

            field_idents.push(field_ident.clone());
            info_init.push(layer_family_info(field_ty, field_ident));

            let layer_attr = f
                .attrs
                .iter()
                .find(|attr| attr.path().is_ident("layer"))
                .map(|attr| {
                    LayerData::from_meta(&attr.meta)
                        .expect("could not parse provided layer arguments")
                });

            let layer_family_attr = f
                .attrs
                .iter()
                .any(|attr| attr.path().is_ident("layer_family"));

            let filtered_attrs = f
                .attrs
                .clone()
                .into_iter()
                .filter(|attr| {
                    !attr.path().is_ident("layer") && !attr.path().is_ident("layer_family")
                })
                .collect();

            if let Some(data) = layer_attr {
                if data.pin.is_some() && data.label.is_some() {
                    let layer_id = quote! { self.0 };
                    tokens.extend(impl_has_pin(
                        generics, ident, &layer_id, &layer_id, &layer_id,
                    ));
                }

                let pin = data.pin.map(|_| quote! { self.0 });
                let label = data.label.map(|_| quote! { self.0 });

                tokens.extend(derive_layer_with_attrs(
                    field_ty,
                    &data.name,
                    &data.gds,
                    &f.vis,
                    &filtered_attrs,
                ));
                tokens.extend(impl_layer_family(
                    generics,
                    field_ty,
                    &layer_new(field_ty),
                    &vec![layer_info(&quote! { Self }, &quote! { self })],
                    &quote! { self.0 },
                    &pin,
                    &label,
                ));
                field_init.push(let_statement(field_ident, &layer_new(field_ty)));
            } else if layer_family_attr {
                field_init.push(let_statement(field_ident, &layer_family_new(field_ty)));
            } else {
                tokens.extend(derive_layer_with_attrs(
                    field_ty,
                    &None,
                    &None,
                    &f.vis,
                    &filtered_attrs,
                ));
                tokens.extend(impl_layer_family(
                    generics,
                    field_ty,
                    &layer_new(field_ty),
                    &vec![layer_info(&quote! { Self }, &quote! { self })],
                    &quote! { self.0 },
                    &None,
                    &None,
                ));
                field_init.push(let_statement(field_ident, &layer_new(field_ty)));
            }
        }

        tokens.extend(quote! {
            impl #imp #substrate::pdk::layers::Layers for #ident #ty #wher {
                fn new(ctx: &mut #substrate::pdk::layers::LayerContext) -> Self {
                    #( #field_init; )*
                    Self {
                        #( #field_idents ),*
                    }
                }

                fn flatten(&self) -> Vec<#substrate::pdk::layers::LayerFamilyInfo> {
                    ::std::vec![
                        #( #info_init ),*
                    ]
                }
            }
        });
    }
}

impl ToTokens for LayerFamilyInputReceiver {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let LayerFamilyInputReceiver {
            ref ident,
            ref generics,
            ref data,
        } = *self;

        let fields = data
            .as_ref()
            .take_struct()
            .expect("Should never be enum")
            .fields;

        let mut field_init = Vec::new();
        let mut field_idents = Vec::new();
        let mut structs = Vec::new();
        let mut info_init = Vec::new();

        let mut primary = None;
        let mut pin = None;
        let mut label = None;

        for f in fields {
            let field_ident = f
                .ident
                .as_ref()
                .expect("could not find identifier for field");
            let field_ty = &f.ty;

            field_idents.push(field_ident.clone());
            info_init.push(layer_info(field_ty, &quote! { self.#field_ident }));

            let layer_attr = f.attrs.iter().find(|attr| attr.path().is_ident("layer"));

            let filtered_attrs = f
                .attrs
                .clone()
                .into_iter()
                .filter(|attr| !attr.path().is_ident("layer"))
                .collect();

            field_init.push(let_statement(field_ident, &layer_new(field_ty)));

            if let Some(attr) = layer_attr {
                let data = LayerFamilyData::from_meta(&attr.meta)
                    .expect("could not parse provided layer arguments");
                tokens.extend(derive_layer_with_attrs(
                    field_ty,
                    &data.name,
                    &data.gds,
                    &f.vis,
                    &filtered_attrs,
                ));

                for (current_value, new_value) in [
                    (&mut primary, data.primary),
                    (&mut pin, data.pin),
                    (&mut label, data.label),
                ] {
                    if new_value.is_some()
                        && current_value
                            .replace(quote! { self.#field_ident.0 })
                            .is_some()
                    {
                        panic!("cannot define the same type of layer twice in a layer family");
                    }
                }
            } else {
                structs.push(derive_layer_with_attrs(
                    field_ty,
                    &None,
                    &None,
                    &f.vis,
                    &filtered_attrs,
                ));
            }
        }

        let primary = primary.expect("no primary layer specified for layer family");

        if let (Some(pin), Some(label)) = (&pin, &label) {
            tokens.extend(impl_has_pin(generics, ident, &primary, pin, label));
        }

        tokens.extend(impl_layer_family(
            generics,
            ident,
            &quote! {
                #( #field_init; )*
                Self {
                    #( #field_idents ),*
                }
            },
            &info_init,
            &primary,
            &pin,
            &label,
        ));

        tokens.extend(impl_as_ref_layer_id(generics, ident, &primary));
    }
}

impl ToTokens for LayerInputReceiver {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let LayerInputReceiver {
            ref ident,
            ref generics,
            name,
            gds,
        } = self;

        tokens.extend(impl_layer(generics, ident, name, gds));
        tokens.extend(impl_as_ref_layer_id(generics, ident, &quote! { self.0 }));
        tokens.extend(impl_deref_layer_id(generics, ident, &quote! { self.0 }));
    }
}

impl ToTokens for DerivedLayersInputReceiver {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let DerivedLayersInputReceiver {
            ref ident,
            ref generics,
            ref data,
        } = *self;

        let fields = data
            .as_ref()
            .take_struct()
            .expect("Should never be enum")
            .fields;

        let mut field_idents = Vec::new();
        let mut info_init = Vec::new();

        for f in fields {
            let field_ident = f
                .ident
                .as_ref()
                .expect("could not find identifier for field");
            let field_ty = &f.ty;

            field_idents.push(field_ident.clone());
            info_init.push(layer_family_info(field_ty, field_ident));

            let layer_attr = f
                .attrs
                .iter()
                .find(|attr| attr.path().is_ident("layer"))
                .map(|attr| {
                    DerivedLayerData::from_meta(&attr.meta)
                        .expect("could not parse provided layer arguments")
                });

            let layer_family_attr = f
                .attrs
                .iter()
                .any(|attr| attr.path().is_ident("layer_family"));

            let filtered_attrs = f
                .attrs
                .clone()
                .into_iter()
                .filter(|attr| {
                    !attr.path().is_ident("layer") && !attr.path().is_ident("layer_family")
                })
                .collect();

            if let Some(data) = layer_attr {
                if data.pin.is_some() && data.label.is_some() {
                    let layer_id = quote! {self.0};
                    tokens.extend(impl_has_pin(
                        generics, ident, &layer_id, &layer_id, &layer_id,
                    ));
                }

                tokens.extend(impl_derived_layer(field_ty, &f.vis, &filtered_attrs));
                tokens.extend(impl_as_ref_layer_id(generics, field_ty, &quote! { self.0 }));
                tokens.extend(impl_deref_layer_id(generics, field_ty, &quote! { self.0 }));
            } else if !layer_family_attr {
                tokens.extend(impl_derived_layer(field_ty, &f.vis, &filtered_attrs));
                tokens.extend(impl_as_ref_layer_id(generics, field_ty, &quote! { self.0 }));
                tokens.extend(impl_deref_layer_id(generics, field_ty, &quote! { self.0 }));
            }
        }
    }
}

impl ToTokens for DerivedLayerFamilyInputReceiver {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let substrate = substrate_ident();
        let DerivedLayerFamilyInputReceiver {
            ref ident,
            ref generics,
            ref data,
        } = *self;

        let fields = data
            .as_ref()
            .take_struct()
            .expect("Should never be enum")
            .fields;

        let mut field_init = Vec::new();
        let mut field_idents = Vec::new();
        let mut info_init = Vec::new();

        let mut primary = None;
        let mut pin = None;
        let mut label = None;

        for f in fields {
            let field_ident = f
                .ident
                .as_ref()
                .expect("could not find identifier for field");
            let field_ty = &f.ty;

            field_idents.push(field_ident.clone());
            info_init.push(layer_info(field_ty, field_ident));

            let layer_attr = f.attrs.iter().find(|attr| attr.path().is_ident("layer"));

            field_init.push(
                quote!(let #field_ident = <#field_ty as #substrate::pdk::layers::Layer>::new(ctx)),
            );

            let filtered_attrs = f
                .attrs
                .clone()
                .into_iter()
                .filter(|attr| !attr.path().is_ident("layer"))
                .collect();
            tokens.extend(impl_derived_layer(field_ty, &f.vis, &filtered_attrs));
            tokens.extend(impl_as_ref_layer_id(generics, field_ty, &quote! { self.0 }));
            tokens.extend(impl_deref_layer_id(generics, field_ty, &quote! { self.0 }));
            if let Some(attr) = layer_attr {
                let data = DerivedLayerFamilyData::from_meta(&attr.meta)
                    .expect("could not parse provided layer arguments");

                for (current_value, new_value) in [
                    (&mut primary, data.primary),
                    (&mut pin, data.pin),
                    (&mut label, data.label),
                ] {
                    if new_value.is_some()
                        && current_value
                            .replace(quote! { self.#field_ident.0 })
                            .is_some()
                    {
                        panic!("cannot define the same type of layer twice in a layer family");
                    }
                }
            }
        }

        let primary = primary.expect("no primary layer specified for layer family");

        if let (Some(pin), Some(label)) = (&pin, &label) {
            tokens.extend(impl_has_pin(generics, ident, &primary, &pin, &label));
        }

        tokens.extend(impl_as_ref_layer_id(generics, ident, &primary));
    }
}
