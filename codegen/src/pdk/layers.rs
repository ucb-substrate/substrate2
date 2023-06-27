use convert_case::{Case, Casing};
use darling::{ast, FromDeriveInput, FromMeta};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::Field;

#[derive(Debug, FromDeriveInput)]
#[darling(supports(struct_any))]
pub struct LayersInputReceiver {
    ident: syn::Ident,
    generics: syn::Generics,
    data: ast::Data<(), Field>,
}

#[derive(Debug, FromMeta)]
pub struct LayerData {
    name: Option<String>,
    gds: Option<String>,
    alias: Option<syn::Ident>,
}

#[derive(Debug, FromMeta)]
pub struct HasPinData {
    pin: syn::Ident,
    label: syn::Ident,
}

impl ToTokens for LayersInputReceiver {
    fn to_tokens(&self, tokens: &mut TokenStream) {
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
        let mut layer_idents = Vec::new();
        let mut field_values = Vec::new();
        let mut has_pin_impls = Vec::new();
        let mut structs = Vec::new();

        for f in fields {
            let field_ident = f
                .ident
                .as_ref()
                .expect("could not find identifier for field");
            let field_ty = &f.ty;

            let layer_attr = f
                .attrs
                .iter()
                .find(|attr| attr.path().is_ident("layer"))
                .map(|attr| {
                    LayerData::from_meta(&attr.meta)
                        .expect("could not parse provided layer arguments")
                });

            if let Some(data) = layer_attr {
                if let Some(alias) = data.alias {
                    field_values.push(quote!(#field_ident: #alias));
                } else {
                    let mut attrs = Vec::new();
                    if let Some(name) = data.name {
                        attrs.push(quote!(name = #name));
                    }
                    if let Some(gds) = data.gds {
                        attrs.push(quote!(gds = #gds));
                    }
                    structs.push(quote!(
                            #[derive(::substrate::Layer, Clone, Copy)]
                            #[layer(#( #attrs ),*)]
                            pub struct #field_ty(::substrate::pdk::layers::LayerId);
                    ));
                    field_values.push(quote!( #field_ident ));
                    layer_idents.push(field_ident.clone());
                    field_init.push(quote!(let #field_ident = <#field_ty as ::substrate::pdk::layers::Layer>::new(ctx)));
                }
            } else {
                structs.push(quote!(
                        #[derive(::substrate::Layer, Clone, Copy)]
                        #[layer()]
                        pub struct #field_ty(::substrate::pdk::layers::LayerId);
                ));
                field_values.push(quote!( #field_ident ));
                layer_idents.push(field_ident.clone());
                field_init.push(quote!(let #field_ident = <#field_ty as ::substrate::pdk::layers::Layer>::new(ctx)));
            }
            if let Some(attr) = f.attrs.iter().find(|attr| attr.path().is_ident("pin")) {
                let HasPinData { pin, label } = HasPinData::from_meta(&attr.meta)
                    .expect("could not parse provided layer arguments");
                has_pin_impls.push(quote!(
                    impl ::substrate::pdk::layers::HasPin<#ident> for #field_ty {
                        fn pin_id(&self, layers: &#ident) -> ::substrate::pdk::layers::LayerId {
                            use ::substrate::pdk::layers::Layer;
                            layers.#pin.0
                        }
                        fn label_id(&self, layers: &#ident) -> ::substrate::pdk::layers::LayerId {
                            use ::substrate::pdk::layers::Layer;
                            layers.#label.0
                        }
                    }
                ));
            }
        }

        tokens.extend(quote! {
            impl #imp ::substrate::pdk::layers::Layers for #ident #ty #wher {
                fn new(ctx: &mut ::substrate::pdk::layers::LayerContext) -> Self {
                    #( #field_init; )*
                    Self {
                        #( #field_values ),*
                    }
                }

                fn flatten(&self) -> Vec<::substrate::pdk::layers::LayerInfo> {
                    use ::substrate::pdk::layers::Layer;
                    vec![
                        #( self.#layer_idents.info() ),*
                    ]
                }
            }

            #( #structs )*

            #( #has_pin_impls )*
        });
    }
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(layer), supports(struct_any))]
pub struct LayerInputReceiver {
    ident: syn::Ident,
    #[darling(default)]
    name: Option<String>,
    #[darling(default)]
    gds: Option<String>,
}

impl ToTokens for LayerInputReceiver {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let LayerInputReceiver {
            ref ident,
            name,
            gds,
        } = self;

        let name = name
            .clone()
            .unwrap_or(ident.to_string().to_case(Case::Snake));
        let gds = if let Some((a, b)) = gds.as_ref().and_then(|gds| {
            gds.split_once('/').map(|(a, b)| {
                (
                    a.parse::<u8>().expect("failed to parse gds layer"),
                    b.parse::<u8>().expect("failed to parse gds data type"),
                )
            })
        }) {
            quote!(Some(::substrate::pdk::layers::GdsLayerSpec(#a, #b)))
        } else {
            quote!(None)
        };

        tokens.extend(quote! {
            impl ::substrate::pdk::layers::Layer for #ident {
                fn new(ctx: &mut ::substrate::pdk::layers::LayerContext) -> Self {
                    Self(ctx.new_layer())
                }

                fn info(&self) -> ::substrate::pdk::layers::LayerInfo {
                    ::substrate::pdk::layers::LayerInfo {
                        id: self.0,
                        name: arcstr::literal!(#name),
                        gds: #gds,
                    }
                }
            }

            impl AsRef<::substrate::pdk::layers::LayerId> for #ident {
                #[inline]
                fn as_ref(&self) -> &::substrate::pdk::layers::LayerId {
                    &self.0
                }
            }
        });
    }
}
