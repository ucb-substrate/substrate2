//! Procedural macros for the Substrate analog circuit generator framework.
#![warn(missing_docs)]

mod block;
mod io;

use darling::FromDeriveInput;
use io::bundle_kind;
use macrotools::{handle_darling_error, handle_syn_error};
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;
use snippets::include_snippet;
use syn::Ident;
use syn::{parse_macro_input, DeriveInput};

/// Derives `Io` for a struct.
///
/// # Examples
///
/// By default, deriving `Io` for a struct creates two new structs, one corresponding to the IO's `BundleKind`
/// and the other to relevant views of the IO (called a `Bundle`). Relevant schematic and layout
/// traits are automatically implemented using these two additional structs.
///
#[doc = include_snippet!("substrate", "buffer_io_simple")]
#[proc_macro_derive(Io, attributes(substrate))]
pub fn derive_io(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as DeriveInput);
    let bundle_impl = handle_syn_error!(bundle_kind(&parsed, true));
    // let layout = layout_io(&input);
    quote!(
        #bundle_impl
    )
    .into()
}

/// Derives `substrate::schematic::NestedData` for a struct.
#[proc_macro_derive(NestedData, attributes(substrate))]
pub fn derive_nested_data(input: TokenStream) -> TokenStream {
    let receiver = block::schematic::DataInputReceiver::from_derive_input(&parse_macro_input!(
        input as DeriveInput
    ));
    let receiver = handle_darling_error!(receiver);
    quote!(
        #receiver
    )
    .into()
}

/// Derives `substrate::block::Block` for a struct or enum.
///
/// You must specify the block's IO by adding a `#[substrate(io = "IoType")]` attribute:
/// ```
/// use substrate::block::Block;
///
/// #[derive(Block, Copy, Clone, Eq, PartialEq, Hash, Debug)]
/// #[substrate(io = "substrate::types::TestbenchIo")]
/// pub struct MyBlock {
///   // ...
/// }
/// ```
///
/// This derive macro only works if you want to use the default value of the IO.
/// If the IO type does not implement [`Default`], or you want to use a non-default
/// value, you must implement `Block` manually.
#[proc_macro_derive(Block, attributes(substrate))]
pub fn derive_block(input: TokenStream) -> TokenStream {
    let receiver =
        block::BlockInputReceiver::from_derive_input(&parse_macro_input!(input as DeriveInput));
    let receiver = handle_darling_error!(receiver);
    quote!(
        #receiver
    )
    .into()
}

pub(crate) fn substrate_ident() -> TokenStream2 {
    match crate_name("substrate").expect("substrate is present in `Cargo.toml`") {
        FoundCrate::Itself => quote!(::substrate),
        FoundCrate::Name(name) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!(::#ident)
        }
    }
}
