use super::chained_section::{ChainedSection, SectionTail};
use super::section::Section;
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::token::Brace;
use syn::{Attribute, Result, ReturnType, Token, Visibility};

pub enum BranchTail {
    Alternative {
        dot: Token![.],
        rest: Box<(Branch, Vec<Branch>)>,
    },
    Leaf {
        output: ReturnType,
        brace: Brace,
        body: TokenStream,
    },
}

pub struct Branch {
    pub section: Section,
    pub tail: BranchTail,
}

impl BranchTail {
    pub fn is_last(&self) -> bool {
        match self {
            Self::Alternative { .. } => false,
            Self::Leaf { .. } => true,
        }
    }
}

impl Branch {
    pub fn is_last(&self) -> bool {
        self.tail.is_last()
    }
}

pub struct Trunk {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub asyncness: Option<Token![async]>,
    pub fn_token: Token![fn],
    pub branch: Branch,
}

pub struct Tree {
    pub roots: Vec<Trunk>,
}

impl Parse for Branch {
    fn parse(input: ParseStream) -> Result<Self> {
        let section: ChainedSection = input.parse()?;

        let tail = match section.tail {
            SectionTail::Content {
                output,
                brace,
                body,
            } => BranchTail::Leaf {
                output,
                brace,
                body,
            },
            SectionTail::Dot(dot) => {
                let rest: Branch = input.parse()?;
                let rest = Box::new((rest, Vec::new()));
                BranchTail::Alternative { dot, rest }
            }
        };
        Ok(Branch {
            section: section.section,
            tail,
        })
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
        self.section.to_tokens(tokens);
        match &self.tail {
            BranchTail::Alternative { dot, rest } => {
                dot.to_tokens(tokens);
                rest.0.to_tokens(tokens);
            }
            BranchTail::Leaf {
                output,
                brace,
                body,
            } => {
                output.to_tokens(tokens);
                brace.surround(tokens, |tokens| body.to_tokens(tokens));
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
    use super::{Tree, Trunk};
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
            r##"fn first () . second () { } fn third () . fourth () { }"##,
        );
    }
}
