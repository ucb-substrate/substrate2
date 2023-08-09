//! Utilities for writing derive macros that dispatch a method call to fields in the struct.

use darling::ast::{Data, Style};
use darling::FromDeriveInput;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{GenericParam, Generics, Index, Visibility};

/// A receiver for parsing derive macro inputs.
#[derive(Debug, FromDeriveInput)]
#[darling(supports(struct_any, enum_any))]
pub struct DeriveInputReceiver {
    ident: syn::Ident,
    generics: syn::Generics,
    data: darling::ast::Data<syn::Variant, syn::Field>,
}

/// Add a bound `T: trait_` to every type parameter T.
pub fn add_trait_bounds(generics: &mut Generics, trait_: TokenStream) {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(syn::parse_quote!(#trait_));
        }
    }
}

/// Tokens used for generating struct fields in derived implementations.
pub struct FieldTokens {
    /// For named structs: "pub field:"
    /// For tuple structs: "pub"
    pub declare: TokenStream,
    /// For named structs: "self.field"
    /// For tuple structs: "self.2"
    pub refer: TokenStream,
    /// For named structs: "field:"
    /// For tuple structs: ""
    pub assign: TokenStream,
    /// For named structs: "field"
    /// For tuple structs: "__substrate_derive_field2"
    pub temp: TokenStream,
    /// For named structs: "field"
    /// For tuple structs: "elem2"
    pub pretty_ident: TokenStream,
}

/// Generates a [`struct@syn::Ident`] for a destructuring an element of a tuple.
pub fn tuple_ident(idx: usize) -> syn::Ident {
    format_ident!("__type_dispatch_derive_field{idx}")
}

/// Returns a [`FieldTokens`] object for a struct that can be referenced using
/// the tokens in `referent`.
pub fn field_tokens_with_referent(
    style: Style,
    vis: &Visibility,
    attrs: &Vec<syn::Attribute>,
    idx: usize,
    ident: &Option<syn::Ident>,
    referent: TokenStream,
) -> FieldTokens {
    let tuple_ident = tuple_ident(idx);
    let pretty_tuple_ident = format_ident!("elem{idx}");
    let idx = syn::Index::from(idx);

    let (declare, refer, assign, temp, pretty_ident) = match style {
        Style::Unit => (quote!(), quote!(), quote!(), quote!(), quote!()),
        Style::Struct => (
            quote!(#(#attrs)* #vis #ident:),
            quote!(#referent.#ident),
            quote!(#ident:),
            quote!(#ident),
            quote!(#ident),
        ),
        Style::Tuple => (
            quote!(#(#attrs)* #vis),
            quote!(#referent.#idx),
            quote!(),
            quote!(#tuple_ident),
            quote!(#pretty_tuple_ident),
        ),
    };

    FieldTokens {
        declare,
        refer,
        assign,
        temp,
        pretty_ident,
    }
}

/// Returns a [`FieldTokens`] object for a struct that can be referenced with `self`.
pub fn field_tokens(
    style: Style,
    vis: &Visibility,
    attrs: &Vec<syn::Attribute>,
    idx: usize,
    ident: &Option<syn::Ident>,
) -> FieldTokens {
    field_tokens_with_referent(style, vis, attrs, idx, ident, syn::parse_quote!(self))
}

/// Configuration for deriving a trait.
pub struct DeriveTrait {
    /// The trait to be implemented.
    pub trait_: TokenStream,
    /// The trait's associated method.
    pub method: TokenStream,
    /// Identifiers for extra arguments to the trait's associated methods.
    pub extra_arg_idents: Vec<TokenStream>,
    /// Types for extra arguments to the trait's associated methods.
    pub extra_arg_tys: Vec<TokenStream>,
}

/// Derives a trait using the given configuration and input.
pub fn derive_trait(
    config: &DeriveTrait,
    receiver: DeriveInputReceiver,
) -> proc_macro2::TokenStream {
    let DeriveTrait {
        ref trait_,
        ref method,
        ref extra_arg_idents,
        ref extra_arg_tys,
    } = *config;

    let mut generics = receiver.generics;
    add_trait_bounds(&mut generics, quote!(#trait_));
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

/// Formats the contents of a struct body in the appropriate style.
pub fn struct_body(style: Style, decl: bool, contents: TokenStream) -> TokenStream {
    if decl {
        match style {
            Style::Unit => quote!(;),
            Style::Tuple => quote!( ( #contents ); ),
            Style::Struct => quote!( { #contents } ),
        }
    } else {
        match style {
            Style::Unit => quote!(),
            Style::Tuple => quote!( ( #contents ) ),
            Style::Struct => quote!( { #contents } ),
        }
    }
}
