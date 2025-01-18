use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse::Parse, parse_quote, Generics, Index, LitInt, Token, Type};

use crate::substrate_ident;

pub struct SaveTuplesInput {
    num_tuples: usize,
    ty: Type,
    generics: Generics,
}

impl Parse for SaveTuplesInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let num_tuples = LitInt::parse(input)?.base10_parse()?;
        let _comma = <Token![,]>::parse(input)?;
        let ty = Type::parse(input)?;
        let generics = Generics::parse(input)?;
        Ok(SaveTuplesInput {
            num_tuples,
            ty,
            generics,
        })
    }
}

pub(crate) fn save_tuples(input: SaveTuplesInput) -> syn::Result<TokenStream> {
    let SaveTuplesInput {
        num_tuples,
        ty,
        generics,
    } = input;
    let substrate = substrate_ident();
    let sim_generic = "S";
    let sim_generic_ident = format_ident!("{sim_generic}");
    let analysis_generic = "A";
    let impls = (1..num_tuples).map(|i| {
        let mut impl_generics = generics.clone();
        let tuple_idents = (0..i)
            .map(|j| format_ident!("{analysis_generic}{j}"))
            .collect::<Vec<_>>();
        let idxs = (0..i).map(Index::from).collect::<Vec<_>>();
        impl_generics
            .params
            .push(parse_quote! { #sim_generic_ident: #substrate::simulation::Simulator });
        for ident in &tuple_idents {
            impl_generics
                .params
                .push(parse_quote! { #ident: #substrate::simulation::Analysis });
        }
        impl_generics.make_where_clause().predicates.push(parse_quote! {
            #ty: #( #substrate::simulation::data::Save<#sim_generic_ident, #tuple_idents> )+*
        });

        let (_, gen_ty, _) = generics.split_for_impl();
        let (imp, _, wher) = impl_generics.split_for_impl();

        quote! {
            impl #imp #substrate::simulation::data::Save<#sim_generic_ident, (#(#tuple_idents,)*)> for #ty #gen_ty #wher {
                type SaveKey = ( #(<#ty as #substrate::simulation::data::Save<#sim_generic_ident, #tuple_idents>>::SaveKey,)* );
                type Saved = ( #(<#ty as #substrate::simulation::data::Save<#sim_generic_ident, #tuple_idents>>::Saved,)* );

                fn save(
                    &self,
                    ctx: &#substrate::simulation::SimulationContext<S>,
                    opts: &mut <S as #substrate::simulation::Simulator>::Options,
                ) -> <Self as #substrate::simulation::data::Save<#sim_generic_ident, (#(#tuple_idents,)*)>>::SaveKey {
                    (
                        #(<#ty as #substrate::simulation::data::Save<#sim_generic_ident, #tuple_idents>>::save(self, ctx, opts),)*
                    )
                }

                fn from_saved(
                    output: &<(#(#tuple_idents,)*) as Analysis>::Output,
                    key: &<Self as #substrate::simulation::data::Save<#sim_generic_ident, (#(#tuple_idents,)*)>>::SaveKey,
                ) -> <Self as #substrate::simulation::data::Save<#sim_generic_ident, (#(#tuple_idents,)*)>>::Saved {
                    (
                        #(<#ty as #substrate::simulation::data::Save<#sim_generic_ident, #tuple_idents>>::from_saved(&output.#idxs, &key.#idxs),)*
                    )
                }
            }
        }
    });

    Ok(quote! {
        #(#impls)*
    })
}
