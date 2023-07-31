use crate::type_dispatch_ident;
use itertools::Itertools;
use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{format_ident, quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{braced, parse, parse2, parse_macro_input, Expr, Token, Type};

struct Arm {
    types: Vec<Type>,
    body: Expr,
    output_type: Type,
}

impl Parse for Arm {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let types: Vec<Type> = Punctuated::<Type, Token![,]>::parse_separated_nonempty(input)?
            .into_iter()
            .collect();
        input.parse::<Token![=>]>()?;

        let body = input.parse()?;
        input.parse::<Token![:]>()?;
        let output_type = input.parse()?;

        Ok(Self {
            types,
            body,
            output_type,
        })
    }
}

struct DispatchConstant {
    generics: Vec<Type>,
    arms: Vec<Arm>,
}

impl Parse for DispatchConstant {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let generics = Punctuated::<Type, Token![,]>::parse_separated_nonempty(&input)?
            .into_iter()
            .collect();
        let contents;
        braced!(contents in input);
        let arms = Punctuated::<Arm, Token![,]>::parse_terminated(&contents)?
            .into_iter()
            .collect();

        Ok(Self { generics, arms })
    }
}

pub(crate) fn dispatch_const_impl(input: TokenStream) -> TokenStream {
    let type_dispatch = type_dispatch_ident();

    let DispatchConstant { generics, arms } = parse_macro_input!(input as DispatchConstant);

    let (types, bodies, output_types): (Vec<_>, Vec<_>, Vec<_>) = arms
        .into_iter()
        .map(|arm| (arm.types, arm.body, arm.output_type))
        .multiunzip();

    assert!(!types.is_empty(), "must have at least one dispatch arm");
    let struct_generics = (0..types[0].len())
        .map(|i| format_ident!("__struct_generic_{i}"))
        .collect::<Vec<_>>();

    quote!(
        {
            struct __const_dispatcher<#( #struct_generics ),*>(#( ::std::marker::PhantomData<#struct_generics>, )*);

            #(
            impl #type_dispatch::DispatchConst for __const_dispatcher<#( #types ),*> {
                type Constant = #output_types;

                const CONST: Self::Constant = #bodies;

            }
            )*

            <__const_dispatcher<#( #generics ),*> as #type_dispatch::DispatchConst>::CONST
        }
    )
    .into()
}
