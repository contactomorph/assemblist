use super::chained_section::{ChainedSection, ContinuingSection, FinalSection};
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::{Attribute, Result, Token, Visibility};

pub enum Branch {
    Alternative(ContinuingSection, Box<(Branch, Vec<Branch>)>),
    Final(FinalSection),
}

pub struct Trunk {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub asyncness: Option<Token![async]>,
    pub fn_token: Token![fn],
    pub branch: Branch,
}

pub struct Tree {
    pub roots: Vec<Trunk>
}

impl Parse for Branch {
    fn parse(input: ParseStream) -> Result<Self> {
        let section: ChainedSection = input.parse()?;

        match section {
            ChainedSection::Final(inner) => {
                Ok(Branch::Final(inner))
            },
            ChainedSection::Continuing(inner) => {
                let rest: Branch = input.parse()?;

                let rest = Box::new((rest, Vec::new()));
                Ok(Branch::Alternative(inner, rest))
            }
        }
    }
}

impl Parse for Trunk {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis: Visibility = input.parse()?;
        let asyncness: Option<Token![async]> = input.parse()?;
        let fn_token: Token![fn] = input.parse()?;
        let branch: Branch = input.parse()?;

        Ok(Trunk {
            attrs,
            vis,
            asyncness,
            fn_token,
            branch,
        })
    }
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

impl ToTokens for Branch {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Branch::Alternative(section, rest) => {
                section.to_tokens(tokens);
                rest.0.to_tokens(tokens);
            }
            Branch::Final(section) => {
                section.to_tokens(tokens);
            }
        }
    }
}

impl ToTokens for Trunk {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for attr in &self.attrs {
            attr.to_tokens(tokens);
        }
        self.vis.to_tokens(tokens);
        self.asyncness.to_tokens(tokens);
        self.fn_token.to_tokens(tokens);
        self.branch.to_tokens(tokens);
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
    use super::{Trunk, Tree};
    use crate::tools::asserts::{assert_tokens_are_matching, assert_tokens_are_not_matching};
    use quote::quote;

    #[test]
    fn parse_tree() {
        let tokens = quote!(fn first().second() {});

        assert_tokens_are_matching::<Trunk>(tokens, r##"fn first () . second () { }"##);

        let tokens = quote!(fn first.second() {});

        assert_tokens_are_not_matching::<Trunk>(tokens, "expected parentheses");
        
        let tokens = quote!(
            fn first().second() { }
            fn third().fourth() { }
        );

        assert_tokens_are_matching::<Tree>(
            tokens, 
            r##"fn first () . second () { } fn third () . fourth () { }"##
        );
    }
}
