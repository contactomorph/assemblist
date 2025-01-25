use proc_macro2::{Span, TokenStream};
use quote::quote_spanned;

pub struct LocalizedFailure {
    span: Span,
    message: &'static str,
}

impl LocalizedFailure {
    pub fn new_err<T>(span: Span, message: &'static str) -> Result<T, Self> {
        Err(Self { span, message })
    }

    pub fn to_stream(self) -> TokenStream {
        let message = self.message;
        quote_spanned! { self.span => compile_error!(#message); }
    }
}
