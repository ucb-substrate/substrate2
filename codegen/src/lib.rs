//! Procedural macros for the Substrate analog circuit generator framework.
#![warn(missing_docs)]

mod pdk;

use pdk::supported_pdks_impl;
use proc_macro::TokenStream;

/// Enumerates PDKs supported by a certain implementation of [`substrate_api::layout::HasLayout`].
///
/// Automatically implements the appropriate trait for all specified PDKs given a process-portable
/// implementation in a single PDK.
///
/// # Examples
///
/// ```
#[doc = include_str!("../../docs/api/code/prelude.md.hidden")]
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
