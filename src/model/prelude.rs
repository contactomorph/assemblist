use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    Attribute, Result, Token, Visibility,
};

pub struct Prelude {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub asyncness: Option<Token![async]>,
}

impl Parse for Prelude {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis: Visibility = input.parse()?;
        let asyncness: Option<Token![async]> = input.parse()?;
        Ok(Prelude {
            attrs,
            vis,
            asyncness,
        })
    }
}

impl ToTokens for Prelude {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for attr in &self.attrs {
            attr.to_tokens(tokens);
        }
        self.vis.to_tokens(tokens);
        self.asyncness.to_tokens(tokens);
    }
}
