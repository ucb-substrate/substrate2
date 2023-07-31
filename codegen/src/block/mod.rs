use convert_case::{Case, Casing};
use darling::FromDeriveInput;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::substrate_ident;

pub mod layout;
pub mod schematic;

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(substrate), supports(struct_any, enum_any))]
pub struct BlockInputReceiver {
    ident: syn::Ident,
    generics: syn::Generics,
    io: syn::Type,
    #[darling(multiple)]
    #[allow(unused)]
    schematic: Vec<darling::util::Ignored>,
    #[darling(multiple)]
    #[allow(unused)]
    layout: Vec<darling::util::Ignored>,
    #[darling(default)]
    flatten: bool,
}

impl ToTokens for BlockInputReceiver {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let substrate = substrate_ident();
        let BlockInputReceiver {
            ref ident,
            ref generics,
            ref io,
            flatten,
            ..
        } = *self;

        let (imp, ty, wher) = generics.split_for_impl();

        let name = ident.to_string().to_case(Case::Snake);

        tokens.extend(quote! {
            impl #imp #substrate::block::Block for #ident #ty #wher {
                type Io = #io;
                const FLATTEN: bool = #flatten;

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
