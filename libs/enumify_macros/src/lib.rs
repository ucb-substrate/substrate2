//! Macros for the `enumify` crate.
#![warn(missing_docs)]

use proc_macro::TokenStream;

use proc_macro_error::proc_macro_error;

use crate::impls::Enumify;
use syn::{parse_macro_input, DeriveInput};

pub(crate) mod impls;

/// Implement enum helper functions.
///
/// This adds implementations for `as_ref`, `as_mut`,
/// and other helpers.
#[proc_macro_error]
#[proc_macro_attribute]
pub fn enumify(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut enumval = input.clone().into();
    let data = match Enumify::new(args.into(), &parse_macro_input!(input as DeriveInput)) {
        Ok(data) => data,
        Err(tokens) => return tokens.into(),
    };
    data.expand(&mut enumval);
    enumval.into()
}
