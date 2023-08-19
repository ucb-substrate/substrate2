//! Macros for the `enumify` crate.
#![warn(missing_docs)]

use darling::ast::NestedMeta;
use darling::FromDeriveInput;

use proc_macro::TokenStream;

use proc_macro_error::proc_macro_error;

use syn::{parse_macro_input, DeriveInput};

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

/// Implement enum helper functions.
///
/// This adds implementations for `as_ref`, `as_mut`,
/// and other helpers.
#[proc_macro_error]
#[proc_macro_attribute]
pub fn enumify(args: TokenStream, input: TokenStream) -> TokenStream {
    let _args = handle_attr_error!(NestedMeta::parse_meta_list(args.into()));
    let mut enumval = input.clone().into();
    let receiver =
        impls::EnumifyInputReceiver::from_derive_input(&parse_macro_input!(input as DeriveInput));
    handle_error!(receiver).expand(&mut enumval);
    enumval.into()
}
