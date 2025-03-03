use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

mod call_superset;

#[proc_macro_derive(CallSuperset, attributes(spalhad))]
pub fn derive_call_superset(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    match call_superset::derive(input) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.into_compile_error().into(),
    }
}
