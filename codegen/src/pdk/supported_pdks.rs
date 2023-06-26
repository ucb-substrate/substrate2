use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    token, Error, ItemImpl, Result, Type,
};

pub struct SupportedPdks(Vec<Ident>);

impl Parse for SupportedPdks {
    fn parse(input: ParseStream) -> Result<Self> {
        let args = Punctuated::<Ident, token::Comma>::parse_terminated(input)?;

        if args.is_empty() {
            return Err(Error::new(
                Span::call_site(),
                "expected at least one supported PDK",
            ));
        }

        Ok(SupportedPdks(args.into_iter().collect()))
    }
}

pub struct PdkImpl {
    placeholder: Ident,
}

impl Parse for PdkImpl {
    fn parse(input: ParseStream) -> Result<Self> {
        let item_impl = ItemImpl::parse(input)?;
        let error_fn = || {
            Error::new(
                Span::call_site(),
                "expected an implementation of HasSchematic or HasLayout",
            )
        };
        let placeholder = match &item_impl
            .trait_
            .ok_or_else(error_fn)?
            .1
            .segments
            .last()
            .ok_or_else(error_fn)?
            .arguments
        {
            syn::PathArguments::None | syn::PathArguments::Parenthesized(_) => Err(error_fn()),
            syn::PathArguments::AngleBracketed(args) => {
                if args.args.len() != 1 {
                    Err(error_fn())
                } else {
                    match args.args.first().unwrap() {
                        syn::GenericArgument::Type(Type::Path(type_path)) => {
                            type_path.path.get_ident().ok_or_else(error_fn).cloned()
                        }
                        _ => Err(error_fn()),
                    }
                }
            }
        }?;
        Ok(PdkImpl { placeholder })
    }
}

pub(crate) fn supported_pdks_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let supported_pdks = parse_macro_input!(args as SupportedPdks).0;
    let input2: proc_macro2::TokenStream = input.clone().into();
    let pdk_impl = parse_macro_input!(input as PdkImpl);
    let placeholder = pdk_impl.placeholder;

    quote!(
        #[::substrate::duplicate::duplicate_item(#placeholder; #( [ #supported_pdks ]; )*)]

        #input2
    )
    .into()
}
