use crate::model::section::Section;
use quote::quote;
use syn::parse_macro_input;

pub fn transform(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let signature = parse_macro_input!(input as Section);
    quote! { fn #signature { todo!() } }.into()
}
