use super::trunk::Trunk;
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::Result;

pub struct Tree {
    pub roots: Vec<Trunk>,
}

impl Parse for Tree {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut roots = Vec::new();
        while !input.is_empty() {
            let trunk: Trunk = input.parse()?;
            roots.push(trunk);
        }
        Ok(Tree { roots })
    }
}

impl ToTokens for Tree {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for trunk in &self.roots {
            trunk.to_tokens(tokens);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::Tree;
    use quote::quote;

    #[test]
    fn parse_tree() {
        let tokens = quote!(
            fn first().second() { }
            fn third().fourth() { }
        );

        asserts::tokens_are_matching::<Tree>(
            tokens,
            "fn first () . second () { } fn third () . fourth () { }",
        );
    }
}
