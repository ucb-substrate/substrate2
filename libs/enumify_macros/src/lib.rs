//! Macros for the `enumify` crate.
#![warn(missing_docs)]

use darling::ast::{Fields, NestedMeta, Style};
use darling::FromDeriveInput;
use darling::{FromField, FromVariant};
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use proc_macro_crate::{crate_name, FoundCrate};
use proc_macro_error::proc_macro_error;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident};
use type_dispatch::derive::{derive_trait, DeriveInputReceiver, DeriveTrait};

pub(crate) mod impls;

macro_rules! handle_error {
    ($expression:expr) => {
        match $expression {
            Ok(value) => value,
            Err(err) => {
                return err.write_errors().into();
            }
        }
    };
}
macro_rules! handle_attr_error {
    ($expression:expr) => {
        match $expression {
            Ok(value) => value,
            Err(err) => {
                return TokenStream::from(darling::Error::from(err).write_errors());
            }
        }
    };
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn enumify(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = handle_attr_error!(NestedMeta::parse_meta_list(args.into()));
    let mut enumval = input.clone().into();
    let receiver =
        impls::EnumifyInputReceiver::from_derive_input(&parse_macro_input!(input as DeriveInput));
    let receiver = handle_error!(receiver).expand(&mut enumval);
    enumval.into()
}
