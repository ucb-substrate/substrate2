//! Macro implementations for the `enumify` crate.
#![warn(missing_docs)]

use convert_case::{Case, Casing};
use darling::ast::{self, Fields, NestedMeta, Style};
use darling::{FromDeriveInput, FromMeta};
use darling::{FromField, FromVariant};
use proc_macro2::TokenStream;

use quote::format_ident;
use quote::quote;

use syn::{DeriveInput, Generics};
use type_dispatch::derive::{field_tokens_with_referent, tuple_ident, FieldTokens};

macro_rules! handle_error {
    ($expression:expr) => {
        match $expression {
            Ok(value) => value,
            Err(err) => {
                return Err(err.write_errors().into());
            }
        }
    };
}
macro_rules! handle_attr_error {
    ($expression:expr) => {
        match $expression {
            Ok(value) => value,
            Err(err) => {
                return Err(TokenStream::from(darling::Error::from(err).write_errors()));
            }
        }
    };
}

pub(crate) struct Enumify {
    args: EnumifyArgs,
    input: EnumifyInputReceiver,
}

#[derive(Debug, FromMeta)]
pub(crate) struct EnumifyArgs {
    #[darling(default)]
    no_as_ref: bool,
    #[darling(default)]
    no_as_mut: bool,
    #[darling(default)]
    generics_only: bool,
}

#[derive(Debug, FromDeriveInput)]
#[darling(
    attributes(enumify),
    supports(enum_any),
    forward_attrs(allow, doc, cfg)
)]
pub(crate) struct EnumifyInputReceiver {
    ident: syn::Ident,
    vis: syn::Visibility,
    generics: syn::Generics,
    data: darling::ast::Data<EnumifyVariant, ()>,
    attrs: Vec<syn::Attribute>,
}

#[derive(Debug, FromVariant)]
#[darling(forward_attrs(allow, doc, cfg))]
#[allow(dead_code)]
pub(crate) struct EnumifyVariant {
    ident: syn::Ident,
    fields: Fields<EnumifyField>,
    attrs: Vec<syn::Attribute>,
}

#[derive(Debug, FromField)]
#[darling(forward_attrs(allow, doc, cfg))]
pub(crate) struct EnumifyField {
    ident: Option<syn::Ident>,
    vis: syn::Visibility,
    ty: syn::Type,
    attrs: Vec<syn::Attribute>,
}

pub(crate) fn field_tokens(
    style: Style,
    vis: &syn::Visibility,
    attrs: &Vec<syn::Attribute>,
    idx: usize,
    ident: &Option<syn::Ident>,
) -> FieldTokens {
    field_tokens_with_referent(
        style,
        vis,
        attrs,
        idx,
        ident,
        syn::parse_quote!(__enumify_derive_key),
    )
}

fn generate_single_tuple_variant_methods(
    variant: &EnumifyVariant,
    generic_overrides: Option<&[syn::Ident]>,
) -> Option<TokenStream> {
    if variant.fields.style != Style::Tuple || variant.fields.len() != 1 {
        return None;
    }

    let name = syn::Ident::new(
        &variant.ident.to_string().to_case(Case::Snake),
        variant.ident.span(),
    );

    let field = variant.fields.iter().next()?;
    let unwrap_name = format_ident!("unwrap_{}", name);
    let get_name = format_ident!("get_{}", name);
    let into_name = format_ident!("into_{}", name);

    let ident = &variant.ident;
    let ty = if let Some(generics) = generic_overrides {
        let generic = &generics[0];
        quote!(#generic)
    } else {
        let ty = &field.ty;
        quote!(#ty)
    };

    Some(quote! {
        /// Returns the value contained in this variant.
        ///
        /// # Panics
        ///
        /// Panics if the enum value is not of the expected variant.
        pub fn #unwrap_name(self) -> #ty {
            match self {
                Self::#ident(x) => x,
                _ => panic!("cannot unwrap"),
            }
        }

        /// Returns a reference to the value contained in this variant.
        ///
        /// Returns [`None`] if the enum value is not of the expected variant.
        pub fn #get_name(&self) -> Option<& #ty> {
            match self {
                Self::#ident(x) => Some(x),
                _ => None,
            }
        }

        /// Returns the value contained in this variant.
        ///
        /// Returns [`None`] if the enum value is not of the expected variant.
        pub fn #into_name(self) -> Option<#ty> {
            match self {
                Self::#ident(x) => Some(x),
                _ => None,
            }
        }
    })
}

fn generate_is_method(variant: &EnumifyVariant) -> Option<TokenStream> {
    let name = syn::Ident::new(
        &variant.ident.to_string().to_case(Case::Snake),
        variant.ident.span(),
    );
    let method_name = format_ident!("is_{}", name);

    let ident = &variant.ident;

    let match_arm = match variant.fields.style {
        Style::Struct => Some(quote!({ .. })),
        Style::Tuple => Some(quote!((..))),
        Style::Unit => None,
    };

    Some(quote! {
        /// Returns true if this value is the expected variant.
        pub fn #method_name(&self) -> bool {
            match self {
                Self::#ident #match_arm => true,
                _ => false,
            }
        }
    })
}

pub(crate) fn generate_is_method_for_variants(
    variants: &[EnumifyVariant],
) -> impl Iterator<Item = TokenStream> + '_ {
    variants.iter().filter_map(generate_is_method)
}

fn ref_generics(generics: &Generics) -> TokenStream {
    let tys = generics.type_params().map(|t| {
        let ty = &t.ident;
        quote! { & #ty }
    });
    quote! {
        < #(#tys),* >
    }
}

fn mut_generics(generics: &Generics) -> TokenStream {
    let tys = generics.type_params().map(|t| {
        let ty = &t.ident;
        quote! { &mut #ty }
    });
    quote! {
        < #(#tys),* >
    }
}

fn variant_match_arm(xident: &syn::Ident, variant: &EnumifyVariant) -> TokenStream {
    let EnumifyVariant {
        ref ident,
        ref fields,
        ..
    } = variant;
    let destructure = fields
        .iter()
        .enumerate()
        .map(|(i, f)| f.ident.clone().unwrap_or_else(|| tuple_ident(i)));
    let assign = fields
        .iter()
        .enumerate()
        .map(|(i, f)| field_assign(true, fields.style, i, f));
    match fields.style {
        Style::Unit => quote!(Self::#ident => #xident::#ident,),
        Style::Tuple => {
            quote!(Self::#ident( #(#destructure),* ) => #xident::#ident( #(#assign)* ),)
        }
        Style::Struct => {
            quote!(Self::#ident { #(#destructure),* } => #xident::#ident { #(#assign)* },)
        }
    }
}

fn field_assign(as_enum: bool, style: Style, idx: usize, field: &EnumifyField) -> TokenStream {
    let EnumifyField {
        ref ident,
        ref vis,
        ref attrs,
        ..
    } = field;
    let FieldTokens {
        refer,
        assign,
        temp,
        ..
    } = field_tokens(style, vis, attrs, idx, ident);
    let refer = if as_enum { temp } else { refer };
    quote!(#assign #refer,)
}

impl Enumify {
    pub(crate) fn new(args: TokenStream, input: &DeriveInput) -> Result<Self, TokenStream> {
        Ok(Self {
            args: handle_attr_error!(EnumifyArgs::from_list(&handle_attr_error!(
                NestedMeta::parse_meta_list(args)
            ))),
            input: handle_error!(EnumifyInputReceiver::from_derive_input(input)),
        })
    }

    /// Returns `true` if the provided arguments indicate that an auxiliary reference enum is required.
    pub(crate) fn ref_enum_required(&self) -> bool {
        // Required to implement `as_ref` and `as_mut` if the target is not already fully generic.
        !self.args.generics_only && (!self.args.no_as_ref || !self.args.no_as_mut)
    }

    pub(crate) fn ref_enum_ident(&self) -> syn::Ident {
        format_ident!("{}Ref", self.input.ident)
    }

    pub(crate) fn expand_ref_enum(&self, variants: &[EnumifyVariant]) -> TokenStream {
        let all_fields = variants
            .iter()
            .flat_map(|variant| variant.fields.iter())
            .collect::<Vec<_>>();
        let generic_idents: Vec<syn::Ident> = (0..all_fields.len())
            .map(|i| format_ident!("V{}", i))
            .collect();
        let ref_generics = quote! {
            < #(& #generic_idents),* >
        };
        let mut_generics = quote! {
            < #(&mut #generic_idents),* >
        };
        let generic_fields: Vec<TokenStream> = all_fields
            .iter()
            .zip(generic_idents.iter())
            .map(
                |(
                    EnumifyField {
                        ident, vis, attrs, ..
                    },
                    generic,
                )| {
                    if let Some(ident) = ident {
                        quote! {
                        #(#attrs)*
                        #vis #ident: #generic
                        }
                    } else {
                        quote! {
                        #(#attrs)* #vis #generic
                        }
                    }
                },
            )
            .collect();

        let mut ctr = 0;
        let generic_variants = variants.iter().map(
            |EnumifyVariant {
                 fields,
                 ident,
                 attrs,
             }| {
                let generic_fields = &generic_fields[ctr..ctr + fields.len()];
                ctr += fields.len();
                match fields.style {
                    Style::Struct => {
                        quote! {
                        #(#attrs)*
                        #ident {
                        #(#generic_fields),*
                        }
                        }
                    }
                    Style::Tuple => {
                        quote! {
                        #(#attrs)*
                        #ident ( #(#generic_fields),* )
                        }
                    }
                    Style::Unit => {
                        quote! {
                        #(#attrs)*
                        #ident
                        }
                    }
                }
            },
        );

        let mut ctr = 0;
        let ref_ident = self.ref_enum_ident();
        let as_ref_arms = variants.iter().map(|v| variant_match_arm(&ref_ident, v));
        let as_mut_arms = as_ref_arms.clone();
        let unwraps = variants.iter().filter_map(|variant| {
            let idents = &generic_idents[ctr..ctr + variant.fields.len()];
            ctr += variant.fields.len();
            generate_single_tuple_variant_methods(variant, Some(idents))
        });

        let EnumifyInputReceiver { attrs, vis, .. } = &self.input;
        let is = generate_is_method_for_variants(variants);

        quote!(
            #(#attrs)*
            #vis enum #ref_ident < #(#generic_idents),* > {
                #(#generic_variants),*
            }

            impl < #(#generic_idents),* > #ref_ident < #(#generic_idents),* > {
                #(#unwraps)*
                #(#is)*

                /// Converts generic types to references.
                ///
                /// For example, transforms the type parameter `T` to `&T`.
                pub fn as_ref(&self) -> #ref_ident #ref_generics {
                    match self {
                        #(#as_ref_arms)*
                    }
                }

                /// Converts generic types to mutable references.
                ///
                /// For example, transforms the type parameter `T` to `&mut T`.
                pub fn as_mut(&mut self) -> #ref_ident #mut_generics {
                    match self {
                        #(#as_mut_arms)*
                    }
                }
            }
        )
    }

    pub(crate) fn expand_enum(&self, tokens: &mut TokenStream, variants: &[EnumifyVariant]) {
        let EnumifyInputReceiver {
            ref ident,
            ref generics,
            ..
        } = self.input;
        let (imp, ty, wher) = generics.split_for_impl();

        let ref_ident = self.ref_enum_ident();

        let ref_enum = self
            .ref_enum_required()
            .then(|| self.expand_ref_enum(variants));

        let unwraps = variants
            .iter()
            .filter_map(|variant| generate_single_tuple_variant_methods(variant, None));

        let (ref_ident, ref_generics, mut_generics) = if self.args.generics_only {
            (ident, ref_generics(generics), mut_generics(generics))
        } else {
            let field_types = variants
                .iter()
                .flat_map(|variant| variant.fields.iter().map(|field| &field.ty));
            let field_types_clone = field_types.clone();
            (
                &ref_ident,
                quote! { < #(& #field_types),* >},
                quote! { < #(&mut #field_types_clone),* >},
            )
        };

        let is = generate_is_method_for_variants(variants);

        let as_ref_arms = variants.iter().map(|v| variant_match_arm(ref_ident, v));
        let as_mut_arms = as_ref_arms.clone();

        let as_ref = (!self.args.no_as_ref).then(|| {
            quote! {
                /// Converts types to references.
                ///
                /// For example, transforms a variant field with type `T` to `&T`.
                pub fn as_ref(&self) -> #ref_ident #ref_generics {
                    match self {
                        #(#as_ref_arms)*
                    }
                }
            }
        });

        let as_mut = (!self.args.no_as_mut).then(|| {
            quote! {
                /// Converts types to mutable references.
                ///
                /// For example, transforms a variant field with type `T` to `&mut T`.
                pub fn as_mut(&mut self) -> #ref_ident #mut_generics {
                    match self {
                        #(#as_mut_arms)*
                    }
                }
            }
        });

        tokens.extend(quote! {
            #ref_enum

            impl #imp #ident #ty #wher {
                #(#unwraps)*
                #(#is)*
                #as_ref
                #as_mut
            }
        });
    }

    pub(crate) fn expand(&self, tokens: &mut TokenStream) {
        let EnumifyInputReceiver { ref data, .. } = self.input;
        match data {
            ast::Data::Struct(ref _fields) => {
                panic!("enumify does not support structs");
            }
            ast::Data::Enum(ref variants) => {
                self.expand_enum(tokens, variants);
            }
        };
    }
}
