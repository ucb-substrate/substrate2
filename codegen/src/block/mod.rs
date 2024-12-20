use convert_case::{Case, Casing};
use darling::FromDeriveInput;
use macrotools::add_trait_bounds;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::substrate_ident;

pub mod schematic;

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(substrate), supports(struct_any, enum_any))]
pub struct BlockInputReceiver {
    ident: syn::Ident,
    generics: syn::Generics,
    io: syn::Type,
}

impl ToTokens for BlockInputReceiver {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let substrate = substrate_ident();
        let BlockInputReceiver {
            ref ident,
            ref generics,
            ref io,
            ..
        } = *self;

        let mut generics = generics.clone();
        add_trait_bounds(&mut generics, quote!(::std::hash::Hash));
        add_trait_bounds(&mut generics, quote!(::std::cmp::Eq));
        add_trait_bounds(&mut generics, quote!(::std::marker::Send));
        add_trait_bounds(&mut generics, quote!(::std::marker::Sync));
        add_trait_bounds(&mut generics, quote!(::std::any::Any));
        let (imp, ty, wher) = generics.split_for_impl();

        let name = ident.to_string().to_case(Case::Snake);

        tokens.extend(quote! {
            impl #imp #substrate::block::Block for #ident #ty #wher {
                type Io = #io;

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
