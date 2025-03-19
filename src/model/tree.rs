use super::chained_section::{ChainedSection, SectionTail};
use super::section::Section;
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::{Parse, ParseBuffer, ParseStream};
use syn::spanned::Spanned;
use syn::token::Brace;
use syn::{braced, Attribute, Result, ReturnType, Token, Visibility};

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

fn try_parse_brace(input: ParseStream) -> Result<ParseBuffer<'_>> {
    let content: ParseBuffer<'_>;
    let _: Brace = braced!(content in input);
    Ok(content)
}

fn try_parse_branches(input: ParseStream) -> Result<Box<(Branch, Vec<Branch>)>> {
    input.parse::<Token![fn]>()?;
    let first_branch: Branch = input.parse()?;
    let mut other_branches = Vec::<Branch>::new();
    while !input.is_empty() {
        input.parse::<Token![fn]>()?;
        let branch: Branch = input.parse()?;
        other_branches.push(branch);
    }
    Ok(Box::new((first_branch, other_branches)))
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
                if let Ok(inner) = try_parse_brace(input) {
                    let rest = try_parse_branches(&inner)?;
                    BranchTail::Alternative { dot, rest }
                } else {
                    let rest: Branch = input.parse()?;
                    let rest = Box::new((rest, Vec::new()));
                    BranchTail::Alternative { dot, rest }
                }
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
                if rest.1.is_empty() {
                    rest.0.to_tokens(tokens);
                } else {
                    let fn_token = syn::token::Fn { span: dot.span() };
                    syn::token::Brace::default().surround(tokens, |tokens| {
                        fn_token.to_tokens(tokens);
                        rest.0.to_tokens(tokens);
                        for branch in &rest.1 {
                            fn_token.to_tokens(tokens);
                            branch.to_tokens(tokens);
                        }
                    });
                }
            }
            BranchTail::Leaf {
                output,
                brace,
                body,
                ..
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

    #[test]
    fn parse_tree_with_trivial_branch_alternative() {
        let tokens = quote!(fn first().{ fn second() { } });

        assert_tokens_are_matching::<Trunk>(tokens, r##"fn first () . second () { }"##);
    }

    #[test]
    fn parse_tree_with_real_branch_alternative() {
        let tokens = quote!(fn first().{ fn second() { } fn second_prime() { } });

        assert_tokens_are_matching::<Trunk>(
            tokens,
            r##"fn first () . { fn second () { } fn second_prime () { } }"##,
        );
    }
}
