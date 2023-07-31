use itertools::Itertools;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{
    braced, bracketed,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    token, Error, GenericParam, ItemImpl, Result, Token, Type,
};

use crate::type_dispatch_ident;

struct ListElement {
    types: Vec<Type>,
}

impl Parse for ListElement {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        bracketed!(content in input);
        Ok(Self {
            types: Punctuated::<Type, token::Comma>::parse_terminated(&content)?
                .into_iter()
                .collect(),
        })
    }
}

enum ImplDispatchElement {
    Type(Type),
    List(ListElement),
}

impl Parse for ImplDispatchElement {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(token::Bracket) {
            input.parse().map(Self::List)
        } else {
            input.parse().map(Self::Type)
        }
    }
}

struct ImplDispatches {
    dispatches: Vec<Vec<Type>>,
}

impl Parse for ImplDispatches {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        braced!(content in input);
        let elements = Punctuated::<ImplDispatchElement, token::Comma>::parse_terminated(&content)?;

        let mut dispatches: Vec<Vec<Type>> = Vec::new();
        for element in elements {
            match element {
                ImplDispatchElement::Type(t) => {
                    dispatches.push(vec![t]);
                }
                ImplDispatchElement::List(l) => {
                    dispatches.push(l.types);
                }
            }
        }

        Ok(Self {
            dispatches: dispatches.into_iter().multi_cartesian_product().collect(),
        })
    }
}

struct ImplDispatchesSet {
    dispatches: Vec<Vec<Type>>,
}

impl Parse for ImplDispatchesSet {
    fn parse(input: ParseStream) -> Result<Self> {
        let elements = Punctuated::<ImplDispatches, Token![;]>::parse_terminated(input)?;

        Ok(Self {
            dispatches: elements
                .into_iter()
                .map(|dispatches| dispatches.dispatches)
                .flatten()
                .collect(),
        })
    }
}

pub(crate) struct ImplDispatchesSetBracketed {
    dispatches: Vec<Vec<Type>>,
}

impl Parse for ImplDispatchesSetBracketed {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        bracketed!(content in input);
        let dispatches: ImplDispatchesSet = content.parse()?;
        Ok(Self {
            dispatches: dispatches.dispatches,
        })
    }
}

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
    let dispatches = parse_macro_input!(args as ImplDispatchesSet).dispatches;
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
