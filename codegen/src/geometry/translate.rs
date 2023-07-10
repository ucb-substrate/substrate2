use crate::substrate_ident;
use darling::{ast, FromDeriveInput};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::Field;

#[derive(Debug, FromDeriveInput)]
#[darling(supports(struct_any))]
pub struct TranslateMutInputReceiver {
    ident: syn::Ident,
    generics: syn::Generics,
    data: ast::Data<(), Field>,
}

impl ToTokens for TranslateMutInputReceiver {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let TranslateMutInputReceiver {
            ref ident,
            ref generics,
            ref data,
        } = *self;

        let (imp, ty, _) = generics.split_for_impl();
        let fields = data
            .as_ref()
            .take_struct()
            .expect("Should never be enum")
            .fields;

        let mut translate_fields = Vec::new();
        let mut bounds = Vec::new();

        let substrate = substrate_ident();

        for f in fields {
            let field_ident = f
                .ident
                .as_ref()
                .expect("could not find identifier for field");
            let field_ty = &f.ty;

            translate_fields.push(quote! {
                #substrate::geometry::transform::TranslateMut::translate_mut(&mut self.#field_ident, point);
            });
            bounds.push(quote! {
                #field_ty: #substrate::geometry::transform::TranslateMut,
            });
        }

        tokens.extend(quote! {
            impl #imp #substrate::geometry::transform::TranslateMut for #ident #ty
            where #(#bounds)*
            {
                fn translate_mut(&mut self, point: #substrate::geometry::point::Point) {
                    #(#translate_fields)*
                }
            }
        });
    }
}
