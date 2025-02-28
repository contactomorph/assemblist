use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::{Parse, ParseBuffer, ParseStream};
use syn::token::Brace;
use syn::{braced, Result, ReturnType, Token};

use super::section::Section;

pub struct ContinuingSection {
    pub section: Section,
    pub dot_token: Token![.],
}

pub struct FinalSection {
    pub section: Section,
    pub output: ReturnType,
    pub brace: Brace,
    pub body: TokenStream,
}

pub enum ChainedSection {
    Continuing(ContinuingSection),
    Final(FinalSection),
}

impl Parse for ChainedSection {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut section: Section = input.parse()?;

        let maybe_dot_token: Result<Token![.]> = input.parse();

        match maybe_dot_token {
            Ok(dot_token) => {
                let inner = ContinuingSection { section, dot_token };
                Ok(ChainedSection::Continuing(inner))
            }
            Err(_) => {
                let output: ReturnType = input.parse()?;
                section.generics.where_clause = input.parse()?;

                let content: ParseBuffer<'_>;
                let brace: Brace = braced!(content in input);

                let body: TokenStream = content.parse()?;
                let inner = FinalSection {
                    section,
                    output,
                    brace,
                    body,
                };
                Ok(ChainedSection::Final(inner))
            }
        }
    }
}

impl ToTokens for ContinuingSection {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.section.to_tokens(tokens);
        self.dot_token.to_tokens(tokens);
    }
}

impl ToTokens for FinalSection {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.section.to_tokens(tokens);
        self.output.to_tokens(tokens);
        self
            .brace
            .surround(tokens, |tokens| self.body.to_tokens(tokens));
    }
}

impl ToTokens for ChainedSection {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            ChainedSection::Continuing(inner) => inner.to_tokens(tokens),
            ChainedSection::Final(inner) => inner.to_tokens(tokens),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ChainedSection;
    use crate::tools::asserts::{assert_tokens_are_matching, assert_tokens_are_not_matching};
    use quote::quote;

    #[test]
    fn parse_chained_section() {
        let tokens = quote!(naked().);

        assert_tokens_are_matching::<ChainedSection>(tokens, r##"naked () ."##);

        let tokens = quote!(get_first<T: Debug>(vec: Vec<T>) -> &T { 5 });

        assert_tokens_are_matching::<ChainedSection>(
            tokens,
            r##"get_first < T : Debug > (vec : Vec < T >) -> & T { 5 }"##,
        );

        let tokens = quote!(find<'a>(collection: &'a Collection) { explode() });

        assert_tokens_are_matching::<ChainedSection>(
            tokens,
            r##"find < 'a > (collection : & 'a Collection) { explode () }"##,
        );

        let tokens = quote!(find<'a, T>(vec: &'a Vec::<T>) where T: Sync { explode() });

        assert_tokens_are_matching::<ChainedSection>(
            tokens,
            r##"find < 'a , T > (vec : & 'a Vec :: < T >) where T : Sync { explode () }"##,
        );

        let tokens = quote!(naked()!);

        assert_tokens_are_not_matching::<ChainedSection>(tokens, "expected curly braces");
    }
}
