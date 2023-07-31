use crate::type_dispatch_ident;
use itertools::Itertools;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{braced, parse_macro_input, Expr, Token, Type};

struct Arm {
    types: Vec<Type>,
    body: Expr,
    output_type: Option<Type>,
}

impl Parse for Arm {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let types: Vec<Type> = Punctuated::<Type, Token![,]>::parse_separated_nonempty(input)?
            .into_iter()
            .collect();
        input.parse::<Token![=>]>()?;

        let body = input.parse()?;
        let output_type = if input.parse::<Token![:]>().is_ok() {
            Some(input.parse()?)
        } else {
            None
        };

        Ok(Self {
            types,
            body,
            output_type,
        })
    }
}

struct DispatchType {
    generics: Vec<Type>,
    arms: Vec<Arm>,
}

impl Parse for DispatchType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![match]>()?;
        let generics = Punctuated::<Type, Token![,]>::parse_separated_nonempty(input)?
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

    let DispatchType { generics, arms } = parse_macro_input!(input as DispatchType);

    let (types, bodies, output_types): (Vec<_>, Vec<_>, Vec<_>) = arms
        .into_iter()
        .map(|arm| {
            (
                arm.types,
                arm.body,
                arm.output_type.expect("output type required for all arms"),
            )
        })
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

pub(crate) fn dispatch_fn_impl(input: TokenStream) -> TokenStream {
    let type_dispatch = type_dispatch_ident();

    let DispatchType { generics, arms } = parse_macro_input!(input as DispatchType);

    let (types, bodies, output_types): (Vec<_>, Vec<_>, Vec<_>) = arms
        .into_iter()
        .map(|arm| {
            (
                arm.types,
                arm.body,
                arm.output_type.expect("output type required for all arms"),
            )
        })
        .multiunzip();

    assert!(!types.is_empty(), "must have at least one dispatch arm");
    let struct_generics = (0..types[0].len())
        .map(|i| format_ident!("__struct_generic_{i}"))
        .collect::<Vec<_>>();

    quote!(
        {
            struct __fn_dispatcher<#( #struct_generics ),*>(#( ::std::marker::PhantomData<#struct_generics>, )*);

            #(
            impl #type_dispatch::DispatchFn for __fn_dispatcher<#( #types ),*> {
                type Output = #output_types;

                fn dispatch_fn() -> Self::Output {
                    #bodies
                }
            }
            )*

            <__fn_dispatcher<#( #generics ),*> as #type_dispatch::DispatchFn>::dispatch_fn()
        }
    )
        .into()
}

pub(crate) fn dispatch_type_impl(input: TokenStream) -> TokenStream {
    let DispatchType { generics, arms } = parse_macro_input!(input as DispatchType);

    let matched_arm = arms
        .into_iter()
        .find(|arm| arm.types == generics)
        .expect("no matching arm found");

    let body = matched_arm.body;

    quote!(
        #body
    )
    .into()
}
