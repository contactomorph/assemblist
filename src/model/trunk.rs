use super::branch::Branch;
use super::prelude::Prelude;
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::{braced, Error, Generics, Result, Token};

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

#[cfg(test)]
mod tests {
    use super::Trunk;
    use quote::quote;

    #[test]
    fn parse_trunks() {
        let tokens = quote!(fn first().second() {});

        asserts::tokens_are_matching!(Trunk, tokens, "fn first () . second () { }");

        let tokens = quote!(fn first.second() {});

        asserts::tokens_are_not_matching!(Trunk, tokens, "expected parentheses");
    }

    #[test]
    fn parse_tree_with_trivial_branch_alternative() {
        let tokens = quote!(fn first().{ fn second() { } });

        asserts::tokens_are_matching!(Trunk, tokens, "fn first () . second () { }");
    }

    #[test]
    fn parse_trunk_with_real_branch_alternative() {
        let tokens = quote!(fn first().{ fn second() { } fn second_prime() { } });

        asserts::tokens_are_matching!(
            Trunk,
            tokens,
            "fn first () . { fn second () { } fn second_prime () { } }"
        );
    }

    #[test]
    fn parse_trunk_with_impl() {
        let tokens = quote!(
            impl Zobi
            {
                fn first().second() { }
                fn third().fourth() { }
            }
        );

        asserts::tokens_are_matching!(
            Trunk,
            tokens,
            "impl Zobi { fn first () . second () { } fn third () . fourth () { } }"
        );
    }
}
