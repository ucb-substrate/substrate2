use darling::ast::{Data, Style};
use darling::FromDeriveInput;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{GenericParam, Generics, Index};

#[derive(Debug, FromDeriveInput)]
#[darling(supports(struct_any, enum_any))]
pub(crate) struct DeriveInputReceiver {
    pub ident: syn::Ident,
    pub generics: syn::Generics,
    pub data: darling::ast::Data<syn::Variant, syn::Field>,
}

// Add a bound `T: HeapSize` to every type parameter T.
pub(crate) fn add_trait_bounds(trait_: TokenStream, mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(syn::parse_quote!(#trait_));
        }
    }
    generics
}

pub(crate) struct DeriveTrait {
    pub trait_: TokenStream,
    pub method: TokenStream,
    pub extra_arg_idents: Vec<TokenStream>,
    pub extra_arg_tys: Vec<TokenStream>,
}

pub(crate) fn derive_trait(
    config: &DeriveTrait,
    receiver: DeriveInputReceiver,
) -> proc_macro2::TokenStream {
    let DeriveTrait {
        ref trait_,
        ref method,
        ref extra_arg_idents,
        ref extra_arg_tys,
    } = *config;

    let generics = add_trait_bounds(quote!(#trait_), receiver.generics);
    let (imp, ty, wher) = generics.split_for_impl();

    let match_clause: TokenStream = match receiver.data {
        Data::Struct(ref fields) => match fields.style {
            Style::Tuple => {
                let recurse = fields.iter().enumerate().map(|(i, f)| {
                    let idx = Index::from(i);
                    quote_spanned! { f.span() =>
                        #trait_::#method(&mut self.#idx, #(#extra_arg_idents),*);
                    }
                });
                quote! { #(#recurse)* }
            }
            Style::Struct => {
                let recurse = fields.iter().map(|f| {
                    let name = f.ident.as_ref().unwrap();
                    quote_spanned! { f.span() =>
                        #trait_::#method(&mut self.#name, #(#extra_arg_idents),*);
                    }
                });
                quote! { #(#recurse)* }
            }
            Style::Unit => quote!(),
        },
        Data::Enum(ref data) => {
            let clauses = data.iter().map(|v| {
                let inner = match v.fields {
                    syn::Fields::Named(ref fields) => {
                        let recurse = fields.named.iter().map(|f| {
                            let name = f.ident.as_ref().unwrap();
                            quote_spanned! { f.span() =>
                                #trait_::#method(#name, #(#extra_arg_idents),*);
                            }
                        });
                        let declare = fields.named.iter().map(|f| {
                            let name = f.ident.as_ref().unwrap();
                            quote_spanned! { f.span() =>
                                ref mut #name,
                            }
                        });
                        quote! {
                            { #(#declare)* } => { #(#recurse)* },
                        }
                    }
                    syn::Fields::Unnamed(ref fields) => {
                        let recurse = fields.unnamed.iter().enumerate().map(|(i, f)| {
                            let ident = format_ident!("field{i}");
                            quote_spanned! { f.span() =>
                                #trait_::#method(#ident, #(#extra_arg_idents),*);
                            }
                        });
                        let declare = fields.unnamed.iter().enumerate().map(|(i, f)| {
                            let ident = format_ident!("field{i}");
                            quote_spanned! { f.span() =>
                                ref mut #ident,
                            }
                        });
                        quote! {
                            ( #(#declare)* ) => { #(#recurse)* },
                        }
                    }
                    syn::Fields::Unit => quote! { => (), },
                };

                let ident = &v.ident;
                quote! {
                    Self::#ident #inner
                }
            });
            quote! {
                match self {
                    #(#clauses)*
                }
            }
        }
    };

    let ident = &receiver.ident;

    let extra_args_sig = extra_arg_idents
        .iter()
        .zip(extra_arg_tys)
        .map(|(ident, ty)| {
            quote! {
                #ident: #ty
            }
        });

    quote! {
        impl #imp #trait_ for #ident #ty #wher {
            fn #method(&mut self, #(#extra_args_sig),*) {
                #match_clause
            }
        }
    }
}
