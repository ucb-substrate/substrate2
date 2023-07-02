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

        let mut path_view_fields = Vec::new();
        let mut construct_path_view_fields = Vec::new();

        let path_view_ident = format_ident!("{}PathView", ident);

        for f in fields {
            let field_ident = f
                .ident
                .as_ref()
                .expect("could not find identifier for field");
            let field_ty = &f.ty;

            if f.attrs.iter().any(|attr| attr.path().is_ident("path_view")) {
                path_view_fields.push(
                    quote! { pub #field_ident: ::substrate::schematic::PathView<'a, #field_ty>, },
                );
                construct_path_view_fields.push(quote! { #field_ident: ::substrate::schematic::HasPathView::path_view(&self.#field_ident, parent.clone()), });
            } else {
                path_view_fields.push(quote! { pub #field_ident: &'a #field_ty, });
                construct_path_view_fields.push(quote! { #field_ident: &self.#field_ident, });
            }
        }

        tokens.extend(quote! {
            // TODO: How to do generics here?
            pub struct #path_view_ident<'a> {
                #( #path_view_fields )*
            }

            impl #imp ::substrate::schematic::HasPathView for #ident #ty #wher {
                type PathView<'a> = #path_view_ident<'a>;

                fn path_view<'a>(
                    &'a self,
                    parent: ::std::option::Option<::std::sync::Arc<::substrate::schematic::RetrogradeEntry>>,
                ) -> Self::PathView<'a> {
                    Self::PathView {
                        #( #construct_path_view_fields )*
                    }
                }
            }
        });
    }
}
