use itertools::Itertools;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    token, GenericParam, ItemImpl, Result, Token, Type,
};

use crate::type_dispatch_ident;

#[derive(Debug, Clone)]
struct Product {
    elements: Vec<ProductElement>,
}

impl Parse for Product {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            elements: Punctuated::<ProductElement, token::Comma>::parse_separated_nonempty(input)?
                .into_iter()
                .collect(),
        })
    }
}

impl Product {
    fn into_dispatches(self) -> Vec<Vec<Type>> {
        let mut product: Vec<Vec<Vec<Type>>> = Vec::new();
        for element in self.elements {
            match element {
                ProductElement::Set(s) => {
                    product.push(s.into_dispatches());
                }
                ProductElement::Type(t) => {
                    product.push(vec![vec![t]]);
                }
            }
        }
        product
            .into_iter()
            .multi_cartesian_product()
            .map(|vecvec| vecvec.into_iter().flatten().collect::<Vec<_>>())
            .collect()
    }
}

#[derive(Debug, Clone)]
enum ProductElement {
    Type(Type),
    Set(Set),
}

impl Parse for ProductElement {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(token::Brace) {
            input.parse().map(Self::Set)
        } else {
            input.parse().map(Self::Type)
        }
    }
}

#[derive(Debug, Clone)]
struct Set {
    elements: Vec<Product>,
}

impl Parse for Set {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        braced!(content in input);
        Ok(Self {
            elements: Punctuated::<Product, Token![;]>::parse_separated_nonempty(&content)?
                .into_iter()
                .collect(),
        })
    }
}

impl Set {
    fn into_dispatches(self) -> Vec<Vec<Type>> {
        let mut dispatches: Vec<Vec<Type>> = Vec::new();
        for element in self.elements {
            dispatches.extend(element.into_dispatches());
        }
        dispatches
    }
}

#[derive(Debug, Clone)]
pub struct ImplGenericIdents {
    generics: Vec<Ident>,
}

impl Parse for ImplGenericIdents {
    fn parse(input: ParseStream) -> Result<Self> {
        let item_impl = ItemImpl::parse(input)?;
        let generics = item_impl
            .generics
            .params
            .into_iter()
            .filter_map(|x| {
                if let GenericParam::Type(t) = x {
                    Some(t.ident)
                } else {
                    None
                }
            })
            .collect();
        Ok(ImplGenericIdents { generics })
    }
}

pub(crate) fn impl_dispatch_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let type_dispatch = type_dispatch_ident();
    let set = parse_macro_input!(args as Product);
    let dispatches = set.into_dispatches();
    let input2 = input.clone();
    let mut idents = parse_macro_input!(input as ImplGenericIdents).generics;

    assert!(
        !dispatches.is_empty(),
        "should dispatch to at least one implementation"
    );
    let n_dispatch = dispatches[0].len();
    assert!(
        idents.len() >= n_dispatch,
        "cannot have more dispatch variables than type generics"
    );
    idents.truncate(n_dispatch);

    let dispatches = dispatches
        .into_iter()
        .map(|dispatch| quote!( #( [ #dispatch ] )* ;));

    let mut input = parse_macro_input!(input2 as ItemImpl);

    let mut count = 0;
    input.generics.params = input
        .generics
        .params
        .into_iter()
        .filter_map(|param| match param {
            param @ GenericParam::Type(_) => {
                if count < n_dispatch {
                    count += 1;
                    None
                } else {
                    Some(param)
                }
            }
            param => Some(param),
        })
        .collect();

    quote!(
        #[#type_dispatch::duplicate::duplicate_item(#( #idents )*; #( #dispatches )*)]

        #input
    )
    .into()
}
