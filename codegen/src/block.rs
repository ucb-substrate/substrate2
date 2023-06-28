use darling::{ast, FromDeriveInput};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::Field;

#[derive(Debug, FromDeriveInput)]
#[darling(supports(struct_any))]
pub struct DataInputReceiver {
    ident: syn::Ident,
    generics: syn::Generics,
    data: ast::Data<(), Field>,
}

impl ToTokens for DataInputReceiver {
    fn to_tokens(&self, tokens: &mut TokenStream) {
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

        let mut transformed_fields = Vec::new();
        let mut transformed_view_fields = Vec::new();

        let transformed_ident = format_ident!("Transformed{}", ident);

        for f in fields {
            let field_ident = f
                .ident
                .as_ref()
                .expect("could not find identifier for field");
            let field_ty = &f.ty;

            if f.attrs.iter().any(|attr| attr.path().is_ident("transform")) {
                transformed_fields.push(quote! { pub #field_ident: ::substrate::geometry::transform::Transformed<'a, #field_ty>, });
                transformed_view_fields.push(quote! { #field_ident: ::substrate::geometry::transform::HasTransformedView::transformed_view(&self.#field_ident, trans), });
            } else {
                // TODO: Might not work for pointers, but there shouldn't be any in data
                // (due to std::any::Any bound).
                transformed_fields.push(quote! { pub #field_ident: &'a #field_ty, });
                transformed_view_fields.push(quote! { #field_ident: &self.#field_ident, });
            }
        }

        tokens.extend(quote! {
            // TODO: How to do generics here?
            pub struct #transformed_ident<'a> {
                #( #transformed_fields )*
            }

            impl #imp ::substrate::geometry::transform::HasTransformedView for #ident #ty #wher {
                type TransformedView<'a> = #transformed_ident<'a>;

                fn transformed_view(
                    &self,
                    trans: ::substrate::geometry::transform::Transformation,
                ) -> Self::TransformedView<'_> {
                    Self::TransformedView {
                        #( #transformed_view_fields )*
                    }
                }
            }
        });
    }
}
