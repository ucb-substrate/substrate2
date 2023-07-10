use crate::{substrate_ident, DeriveInputReceiver};
use darling::ast::{Data, Style};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{GenericParam, Generics, Index};

// Add a bound `T: HeapSize` to every type parameter T.
fn add_trait_bounds(trait_: TokenStream, mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(syn::parse_quote!(#trait_));
        }
    }
    generics
}

pub(crate) fn derive_translate_mut(receiver: DeriveInputReceiver) -> proc_macro2::TokenStream {
    let substrate = substrate_ident();

    let generics = add_trait_bounds(
        quote!(#substrate::geometry::transform::TranslateMut),
        receiver.generics,
    );
    let (imp, ty, wher) = generics.split_for_impl();

    let match_clause: TokenStream = match receiver.data {
        Data::Struct(ref fields) => match fields.style {
            Style::Tuple => {
                let recurse = fields.iter().enumerate().map(|(i, f)| {
                        let idx = Index::from(i);
                        quote_spanned! { f.span() =>
                            #substrate::geometry::transform::TranslateMut::translate_mut(&mut self.#idx, __substrate_derive_point);
                        }
                    });
                quote! { #(#recurse)* }
            }
            Style::Struct => {
                let recurse = fields.iter().map(|f| {
                        let name = f.ident.as_ref().unwrap();
                        quote_spanned! { f.span() =>
                            #substrate::geometry::transform::TranslateMut::translate_mut(&mut self.#name, __substrate_derive_point);
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
                                #substrate::geometry::transform::TranslateMut::translate_mut(#name, __substrate_derive_point);
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
                    },
                    syn::Fields::Unnamed(ref fields) => {
                        let recurse = fields.unnamed.iter().enumerate().map(|(i, f)| {
                            let ident = format_ident!("field{i}");
                            quote_spanned! { f.span() =>
                                #substrate::geometry::transform::TranslateMut::translate_mut(#ident, __substrate_derive_point);
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
                    },
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

    quote! {
        impl #imp #substrate::geometry::transform::TranslateMut for #ident #ty #wher {
            fn translate_mut(&mut self, __substrate_derive_point: #substrate::geometry::point::Point) {
                #match_clause
            }
        }
    }
}
