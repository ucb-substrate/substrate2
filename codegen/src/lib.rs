//! Procedural macros for the Substrate analog circuit generator framework.
#![warn(missing_docs)]

mod block;
mod io;
mod pdk;

use block::DataInputReceiver;
use darling::FromDeriveInput;
use io::{IoInputReceiver, LayoutIoInputReceiver, SchematicIoInputReceiver};
use pdk::layers::{LayerInputReceiver, LayersInputReceiver};
use pdk::supported_pdks::supported_pdks_impl;
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;
use syn::Ident;
use syn::{parse_macro_input, DeriveInput};

/// Enumerates PDKs supported by a certain layout implementation of a block.
///
/// Automatically implements the appropriate trait for all specified PDKs given a process-portable
/// implementation in a single PDK.
///
/// # Examples
///
/// ```
#[doc = include_str!("../../docs/api/code/prelude.md.hidden")]
#[doc = include_str!("../../docs/api/code/pdk/layers.md.hidden")]
#[doc = include_str!("../../docs/api/code/pdk/several_pdks.md.hidden")]
#[doc = include_str!("../../docs/api/code/block/inverter.md.hidden")]
#[doc = include_str!("../../docs/api/code/block/buffer.md.hidden")]
#[doc = include_str!("../../docs/api/code/layout/inverter_multiprocess.md")]
#[doc = include_str!("../../docs/api/code/layout/buffer_multiprocess.md")]
/// ```
#[proc_macro_attribute]
pub fn supported_pdks(args: TokenStream, input: TokenStream) -> TokenStream {
    supported_pdks_impl(args, input)
}

/// Derives a layer implementation on a tuple struct containing only an ID.
///
/// # Examples
///
/// ```
/// # use substrate::Layer;
/// # use substrate::pdk::layers::LayerId;
/// #[derive(Layer, Clone, Copy)]
/// #[layer(name = "poly", gds = "66/20")]
/// pub struct Poly(LayerId);
/// ```
#[proc_macro_derive(Layer, attributes(layer))]
pub fn derive_layer(input: TokenStream) -> TokenStream {
    let receiver =
        LayerInputReceiver::from_derive_input(&parse_macro_input!(input as DeriveInput)).unwrap();
    quote!(
        #receiver
    )
    .into()
}

/// Derives a layer set implementation on a struct.
///
/// # Examples
///
/// ```
#[doc = include_str!("../../docs/api/code/prelude.md.hidden")]
#[doc = include_str!("../../docs/api/code/pdk/layers.md")]
/// ```
#[proc_macro_derive(Layers, attributes(layer, pin, alias))]
pub fn derive_layers(input: TokenStream) -> TokenStream {
    let receiver =
        LayersInputReceiver::from_derive_input(&parse_macro_input!(input as DeriveInput)).unwrap();
    quote!(
        #receiver
    )
    .into()
}

/// Derives `Io` for a struct.
#[proc_macro_derive(Io, attributes(io))]
pub fn derive_io(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as DeriveInput);
    let receiver_io = IoInputReceiver::from_derive_input(&parsed).unwrap();
    let receiver_schematic = SchematicIoInputReceiver::from_derive_input(&parsed).unwrap();
    let receiver_layout = LayoutIoInputReceiver::from_derive_input(&parsed).unwrap();
    let ident = parsed.ident;
    let (imp, ty, wher) = parsed.generics.split_for_impl();
    let substrate = substrate_ident();
    quote!(
        impl #imp #substrate::io::Io for #ident #ty #wher {}
        #receiver_io
        #receiver_schematic
        #receiver_layout
    )
    .into()
}

/// Derives `LayoutType` for a struct.
#[proc_macro_derive(LayoutType)]
pub fn derive_layout_io(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as DeriveInput);
    let receiver_io = IoInputReceiver::from_derive_input(&parsed).unwrap();
    let receiver_layout = LayoutIoInputReceiver::from_derive_input(&parsed).unwrap();
    quote!(
        #receiver_io
        #receiver_layout
    )
    .into()
}

/// Derives `substrate::layout::Data` for a struct.
#[proc_macro_derive(LayoutData, attributes(transform))]
pub fn derive_layout_data(input: TokenStream) -> TokenStream {
    let receiver =
        DataInputReceiver::from_derive_input(&parse_macro_input!(input as DeriveInput)).unwrap();
    quote!(
        #receiver
    )
    .into()
}

pub(crate) fn substrate_ident() -> TokenStream2 {
    match crate_name("substrate")
        .or_else(|_| crate_name("substrate_api"))
        .expect("substrate is present in `Cargo.toml`")
    {
        FoundCrate::Itself => quote!(crate),
        FoundCrate::Name(name) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!(::#ident)
        }
    }
}
