use convert_case::{Case, Casing};
use darling::{ast, FromDeriveInput, FromMeta};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Block, Field, LitStr, Meta};

#[derive(Debug, FromDeriveInput)]
#[darling(supports(struct_any))]
pub struct LayersInputReceiver {
    ident: syn::Ident,
    generics: syn::Generics,
    data: ast::Data<(), Field>,
}

#[derive(Debug, FromMeta)]
pub struct LayerData {
    alias: Option<syn::Ident>,
    pin: Option<syn::Ident>,
    label: Option<syn::Ident>,
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
        let mut field_idents = Vec::new();
        let mut has_pin_impls = Vec::new();

        for f in fields {
            let field_ident = f
                .ident
                .as_ref()
                .expect("could not find identifier for field");
            let field_ty = &f.ty;

            field_idents.push(field_ident.clone());

            if let Some(attr) = f.attrs.iter().find(|attr| attr.path().is_ident("layer")) {
                layer_idents.push(field_ident.clone());
                match &attr.meta {
                    meta @ Meta::List(_) => {
                        let data = LayerData::from_meta(meta)
                            .expect("could not parse provided layer arguments");
                        field_init.push(quote!(let #field_ident = <#field_ty as ::substrate::pdk::layers::Layer>::new(ctx)));
                        if let Some(alias) = data.alias {
                            field_init.push(quote!(let #alias = #field_ident.clone()));
                        }

                        match (data.pin, data.label) {
                            (Some(pin), Some(label)) => {
                                has_pin_impls.push(quote!(
                                    impl ::substrate::pdk::layers::HasPin<#ident> for #field_ty {
                                        fn pin_id(&self, layers: &#ident) -> ::substrate::pdk::layers::LayerId {
                                            use ::substrate::pdk::layers::Layer;
                                            layers.#pin.info().id
                                        }
                                        fn label_id(&self, layers: &#ident) -> ::substrate::pdk::layers::LayerId {
                                            use ::substrate::pdk::layers::Layer;
                                            layers.#label.info().id
                                        }
                                    }
                                ));
                            }
                            (None, None) => {}
                            _ => panic!("pin and label fields must be specified together"),
                        }
                    }
                    Meta::Path(_) => {
                        field_init.push(quote!(let #field_ident = <#field_ty as ::substrate::pdk::layers::Layer>::new(ctx)));
                    }
                    _ => {
                        panic!("name value layer attributes are not supported")
                    }
                }
            } else if let Some(attr) = f.attrs.iter().find(|attr| attr.path().is_ident("value")) {
                let value =
                    LayerValue::from_meta(&attr.meta).expect("could not parse custom value");
                field_init.push(quote!(let #field_ident = #value));
            } else if f.attrs.iter().any(|attr| attr.path().is_ident("alias")) {
            } else {
                panic!("each field must be either a layer, alias, or value");
            }
        }

        tokens.extend(quote! {
            impl #imp ::substrate::pdk::layers::Layers for #ident #ty #wher {
                fn new(ctx: &mut ::substrate::pdk::layers::LayerContext) -> Self {
                    #( #field_init; )*
                    Self {
                        #( #field_idents ),*
                    }
                }

                fn flatten(&self) -> Vec<::substrate::pdk::layers::LayerInfo> {
                    use ::substrate::pdk::layers::Layer;
                    vec![
                        #( self.#layer_idents.info() ),*
                    ]
                }
            }

            #( #has_pin_impls )*
        });
    }
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(layer), supports(struct_any))]
pub struct LayerInputReceiver {
    /// The struct ident.
    ident: syn::Ident,
    generics: syn::Generics,
    data: ast::Data<(), Field>,

    #[darling(default)]
    name: Option<String>,

    #[darling(default)]
    gds: Option<String>,
}

pub struct LayerValue(Block);

impl LayerValue {
    pub fn is_empty(&self) -> bool {
        self.0.stmts.is_empty()
    }
}

impl ToTokens for LayerValue {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens)
    }
}

impl TryFrom<&'_ LitStr> for LayerValue {
    type Error = syn::Error;

    fn try_from(s: &LitStr) -> Result<Self, Self::Error> {
        let mut block_str = s.value();
        block_str.insert(0, '{');
        block_str.push('}');
        LitStr::new(&block_str, s.span()).parse().map(Self)
    }
}

impl darling::FromMeta for LayerValue {
    fn from_value(value: &syn::Lit) -> darling::Result<Self> {
        if let syn::Lit::Str(s) = value {
            let contents = LayerValue::try_from(s)?;
            if contents.is_empty() {
                Err(darling::Error::unknown_value("").with_span(s))
            } else {
                Ok(contents)
            }
        } else {
            Err(darling::Error::unexpected_lit_type(value))
        }
    }
}

impl ToTokens for LayerInputReceiver {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let LayerInputReceiver {
            ref ident,
            ref generics,
            ref data,
            name,
            gds,
        } = self;

        let (imp, ty, wher) = generics.split_for_impl();
        let fields = data
            .as_ref()
            .take_struct()
            .expect("Should never be enum")
            .fields;

        let field_list = fields
            .into_iter()
            .filter_map(|f| {
                let field_ident = &f.ident;

                if let Some(attr) = f.attrs.iter().find(|attr| attr.path().is_ident("value")) {
                    let value =
                        LayerValue::from_meta(&attr.meta).expect("could not parse custom value");
                    Some(quote!(#field_ident: #value))
                } else if field_ident
                    .as_ref()
                    .map(|field_ident| field_ident == "id")
                    .unwrap_or(false)
                {
                    None
                } else {
                    panic!("each field other than `id` must have a value");
                }
            })
            .collect::<Vec<_>>();

        let name = name
            .clone()
            .unwrap_or(ident.to_string().to_case(Case::Snake));
        let gds = if let Some((a, b)) = gds.as_ref().and_then(|gds| {
            gds.split_once('/').map(|(a, b)| {
                (
                    a.parse::<u16>().expect("failed to parse gds layer"),
                    b.parse::<u16>().expect("failed to parse gds data type"),
                )
            })
        }) {
            quote!(Some((#a, #b)))
        } else {
            quote!(None)
        };

        tokens.extend(quote! {
            impl #imp ::substrate::pdk::layers::Layer for #ident #ty #wher {
                fn new(ctx: &mut ::substrate::pdk::layers::LayerContext) -> Self {
                    Self {
                        id: ctx.new_layer(),
                        #( #field_list ),*
                    }
                }

                fn info(&self) -> ::substrate::pdk::layers::LayerInfo {
                    ::substrate::pdk::layers::LayerInfo {
                        id: self.id,
                        name: arcstr::literal!(#name),
                        gds: #gds,
                    }
                }
            }

            impl #imp AsRef<::substrate::pdk::layers::LayerId> for #ident #ty #wher {
                #[inline]
                fn as_ref(&self) -> &LayerId {
                    &self.id
                }
            }
        });
    }
}
