use convert_case::{Case, Casing};
use darling::{FromDeriveInput, FromMeta};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::substrate_ident;

pub mod layout;
pub mod schematic;

#[derive(Debug, FromDeriveInput)]
#[darling(forward_attrs, supports(struct_any))]
pub struct BlockInputReceiver {
    ident: syn::Ident,
    generics: syn::Generics,
    attrs: Vec<syn::Attribute>,
}

#[derive(Debug, FromMeta)]
struct BlockData {
    io: syn::Type,
}

impl ToTokens for BlockInputReceiver {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let substrate = substrate_ident();
        let BlockInputReceiver {
            ref ident,
            ref generics,
            ref attrs,
        } = *self;

        let (imp, ty, wher) = generics.split_for_impl();

        let name = ident.to_string().to_case(Case::Snake);

        let block_attr = attrs
            .iter()
            .find(|attr| attr.path().is_ident("block"))
            .map(|attr| {
                BlockData::from_meta(&attr.meta).expect("could not parse provided block arguments")
            })
            .expect("must provide the #[block] attribute");
        let io_type = block_attr.io;
        tokens.extend(quote! {
            impl #imp #substrate::block::Block for #ident #ty #wher {
                type Io = #io_type;
                fn id() -> #substrate::arcstr::ArcStr {
                    #substrate::arcstr::literal!(::std::concat!(::std::module_path!(), "::", ::std::stringify!(#ident)))
                }
                fn name(&self) -> #substrate::arcstr::ArcStr {
                    #substrate::arcstr::literal!(#name)
                }
                fn io(&self) -> <Self as #substrate::block::Block>::Io {
                    <<Self as #substrate::block::Block>::Io as ::std::default::Default>::default()
                }
            }
        });
    }
}
