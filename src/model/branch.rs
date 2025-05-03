use super::chained_section::{ChainedSection, SectionTail};
use super::section::Section;
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::{Parse, ParseBuffer, ParseStream};
use syn::spanned::Spanned;
use syn::token::Brace;
use syn::{braced, Result, ReturnType, Token};

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
