extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Ident, Token,
};

struct SetStateFieldsInput {
    state: Ident,
    fields: Vec<Ident>,
}

impl Parse for SetStateFieldsInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let state: Ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let fields = input.parse_terminated(Ident::parse, Token![,])?.into_iter().collect();

        Ok(SetStateFieldsInput { state, fields })
    }
}

#[proc_macro]
pub fn set_state_fields(input: TokenStream) -> TokenStream {
    let SetStateFieldsInput { state, fields } = parse_macro_input!(input as SetStateFieldsInput);
    let assignments = fields.iter().map(|field| {
        quote! {
            #state.#field = #field;
        }
    });
    let expanded = quote! {
        #(#assignments)*
    };

    TokenStream::from(expanded)
}
