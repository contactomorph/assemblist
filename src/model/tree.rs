use super::chained_section::{ChainedSection, SectionTail};
use super::prelude::Prelude;
use super::section::Section;
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::{Parse, ParseBuffer, ParseStream};
use syn::spanned::Spanned;
use syn::token::Brace;
use syn::{braced, Error, Generics, Result, ReturnType, Token};

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

pub struct FnTrunk {
    pub prelude: Prelude,
    pub fn_token: Token![fn],
    pub branch: Branch,
}

pub struct ImplHeader {
    pub impl_token: Token![impl],
    pub generics: Generics,
    pub self_ty: syn::Type,
    pub brace_token: syn::token::Brace,
}

pub enum TrunkAlternative {
    Fn {
        fn_token: Token![fn],
        branch: Branch,
    },
    Impl {
        header: ImplHeader,
        fn_trunks: Vec<FnTrunk>,
    },
}

pub struct Trunk {
    pub prelude: Prelude,
    pub alternative: TrunkAlternative,
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

impl Parse for FnTrunk {
    fn parse(input: ParseStream) -> Result<Self> {
        let prelude: Prelude = input.parse()?;
        let fn_token: Token![fn] = input.parse()?;
        let branch: Branch = input.parse()?;

        Ok(FnTrunk {
            prelude,
            fn_token,
            branch,
        })
    }
}

impl Parse for Trunk {
    fn parse(input: ParseStream) -> Result<Self> {
        // We let the compiler handle correct use of visibility and asyncness:
        // If such concepts are declared for impl, the compiler will complain.
        let prelude: Prelude = input.parse()?;

        if input.peek(Token![fn]) {
            let fn_token: Token![fn] = input.parse()?;
            let branch: Branch = input.parse()?;

            let alternative = TrunkAlternative::Fn { fn_token, branch };

            Ok(Trunk {
                prelude,
                alternative,
            })
        } else if input.peek(Token![impl]) {
            let impl_token: Token![impl] = input.parse()?;
            let has_generics = input.peek(Token![<])
                && (input.peek2(Token![>])
                    || input.peek2(Token![#])
                    || (input.peek2(syn::Ident) || input.peek2(syn::Lifetime))
                        && (input.peek3(Token![:])
                            || input.peek3(Token![,])
                            || input.peek3(Token![>])
                            || input.peek3(Token![=]))
                    || input.peek2(Token![const]));

            let mut generics: Generics = if has_generics {
                input.parse::<Generics>()?
            } else {
                Generics::default()
            };

            let self_ty: syn::Type = input.parse()?;
            generics.where_clause = input.parse()?;

            let content;
            let brace_token = braced!(content in input);

            let header = ImplHeader {
                impl_token,
                generics,
                self_ty,
                brace_token,
            };

            let mut fn_trunks = Vec::<FnTrunk>::new();

            while !content.is_empty() {
                let fn_trunk: FnTrunk = content.parse()?;
                fn_trunks.push(fn_trunk);
            }

            let alternative = TrunkAlternative::Impl { header, fn_trunks };

            Ok(Trunk {
                prelude,
                alternative,
            })
        } else {
            Err(Error::new(input.span(), "expected one of: `fn`, `impl`"))
        }
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

impl ToTokens for FnTrunk {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.prelude.to_tokens(tokens);
        self.fn_token.to_tokens(tokens);
        self.branch.to_tokens(tokens);
    }
}

impl ToTokens for Trunk {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.prelude.to_tokens(tokens);
        match &self.alternative {
            TrunkAlternative::Fn { fn_token, branch } => {
                fn_token.to_tokens(tokens);
                branch.to_tokens(tokens);
            }
            TrunkAlternative::Impl { header, fn_trunks } => {
                header.impl_token.to_tokens(tokens);
                header.generics.to_tokens(tokens);
                header.self_ty.to_tokens(tokens);
                header.brace_token.surround(tokens, |tokens| {
                    for fn_trunk in fn_trunks {
                        fn_trunk.to_tokens(tokens);
                    }
                });
            }
        }
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

    #[test]
    fn parse_tree_with_impl() {
        let tokens = quote!(
            impl Zobi
            {
                fn first().second() { }
                fn third().fourth() { }
            }
        );

        assert_tokens_are_matching::<Trunk>(
            tokens,
            r##"impl Zobi { fn first () . second () { } fn third () . fourth () { } }"##,
        );
    }
}
