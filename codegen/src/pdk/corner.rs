use crate::substrate_ident;
use darling::FromDeriveInput;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

#[derive(Debug, FromDeriveInput)]
#[darling(supports(struct_any, enum_any))]
pub struct CornerReceiver {
    ident: syn::Ident,
    generics: syn::Generics,
}
impl ToTokens for CornerReceiver {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let substrate = substrate_ident();
        let CornerReceiver {
            ref ident,
            ref generics,
        } = *self;

        let (imp, ty, wher) = generics.split_for_impl();

        tokens.extend(quote! {
            impl #imp #substrate::pdk::corner::Corner for #ident #ty #wher {}

            impl #imp ::std::convert::AsRef<#ident #ty> for #ident #ty #wher {
                #[inline]
                fn as_ref(&self) -> &#ident { self }
            }
        });
    }
}
