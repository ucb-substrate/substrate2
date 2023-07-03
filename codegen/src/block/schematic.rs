use darling::{ast, FromDeriveInput};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::Field;

use crate::substrate_ident;

#[derive(Debug, FromDeriveInput)]
#[darling(supports(struct_any))]
pub struct DataInputReceiver {
    ident: syn::Ident,
    generics: syn::Generics,
    data: ast::Data<(), Field>,
}

impl ToTokens for DataInputReceiver {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let substrate = substrate_ident();
        let DataInputReceiver {
            ref ident,
            ref generics,
            ref data,
        } = *self;

        let (imp, ty, wher) = generics.split_for_impl();
        let fields = data
            .as_ref()
            .take_struct()
            .expect("Should never be enum")
            .fields;

        let mut nested_view_fields = Vec::new();
        let mut construct_nested_view_fields = Vec::new();

        let nested_view_ident = format_ident!("Nested{}View", ident);

        for f in fields {
            let field_ident = f
                .ident
                .as_ref()
                .expect("could not find identifier for field");
            let field_ty = &f.ty;

            if f.attrs.iter().any(|attr| attr.path().is_ident("nested")) {
                nested_view_fields.push(
                    quote! { pub #field_ident: #substrate::schematic::NestedView<'a, #field_ty>, },
                );
                construct_nested_view_fields.push(quote! { #field_ident: #substrate::schematic::HasNestedView::nested_view(&self.#field_ident, parent), });
            } else {
                nested_view_fields.push(quote! { pub #field_ident: &'a #field_ty, });
                construct_nested_view_fields.push(quote! { #field_ident: &self.#field_ident, });
            }
        }

        tokens.extend(quote! {
            // TODO: How to do generics here?
            pub struct #nested_view_ident<'a> {
                #( #nested_view_fields )*
            }

            impl #imp #substrate::schematic::HasNestedView for #ident #ty #wher {
                type NestedView<'a> = #nested_view_ident<'a>;

                fn nested_view<'a>(
                    &'a self,
                    parent: &#substrate::schematic::InstancePath,
                ) -> Self::NestedView<'a> {
                    Self::NestedView {
                        #( #construct_nested_view_fields )*
                    }
                }
            }
        });
    }
}
