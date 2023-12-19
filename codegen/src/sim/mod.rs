use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse::Parse;
use syn::parse_macro_input;

use crate::substrate_ident;

pub(crate) mod save;

pub(crate) struct TuplesImpl {
    max: usize,
}

impl Parse for TuplesImpl {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lit = syn::LitInt::parse(input)?;
        Ok(Self {
            max: lit.base10_parse::<usize>()?,
        })
    }
}

pub(crate) fn simulator_tuples_impl(input: TokenStream) -> TokenStream {
    let substrate = substrate_ident();

    let mut tokens = Vec::new();
    let max = parse_macro_input!(input as TuplesImpl).max;
    assert!(max >= 2, "Must implement on tuples of size at least 2");

    for n in 2..=max {
        let mut tys = Vec::new();
        let mut bounds = Vec::new();
        let mut into_inputs = Vec::new();
        let mut from_outputs = Vec::new();

        for i in 0..n {
            let ty = format_ident!("T{}", i);
            let idx = syn::Index::from(i);
            tys.push(ty.clone());
            bounds.push(quote! {
                #ty: #substrate::simulation::Analysis + #substrate::simulation::SupportedBy<S>
            });
            into_inputs.push(quote! {
                <#ty as #substrate::simulation::SupportedBy<S>>::into_input(self.#idx, inputs);
            });
            from_outputs.push(quote! {
                <#ty as #substrate::simulation::SupportedBy<S>>::from_output(outputs)
            });
        }

        tokens.push(quote! {
            impl <S, #( #tys ),*> #substrate::simulation::SupportedBy<S> for ( #( #tys ),* )
                where S: #substrate::simulation::Simulator, #(#bounds),*
            {
                fn into_input(self, inputs: &mut Vec<<S as #substrate::simulation::Simulator>::Input>) {
                    #(#into_inputs)*
                }

                fn from_output(outputs: &mut impl Iterator<Item = <S as #substrate::simulation::Simulator>::Output>) -> <Self as #substrate::simulation::Analysis>::Output {
                    (#(#from_outputs),*)
                }
            }
        });
    }

    quote!(
        #( #tokens )*
    )
    .into()
}
