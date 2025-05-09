use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::{Parse, ParseBuffer, ParseStream};
use syn::token::Brace;
use syn::{braced, Result, ReturnType, Token};

use super::section::Section;

pub enum SectionTail {
    Dot(Token![.]),
    Content {
        output: ReturnType,
        brace: Brace,
        body: TokenStream,
    },
}

pub struct ChainedSection {
    pub section: Section,
    pub tail: SectionTail,
}

impl Parse for ChainedSection {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut section: Section = input.parse()?;

        let tail = if input.peek(Token![.]) {
            let dot_token: Token![.] = input.parse()?;
            SectionTail::Dot(dot_token)
        } else {
            let output: ReturnType = input.parse()?;
            section.generics.where_clause = input.parse()?;

            let content: ParseBuffer<'_>;
            let brace: Brace = braced!(content in input);

            let body: TokenStream = content.parse()?;

            SectionTail::Content {
                output,
                brace,
                body,
            }
        };
        Ok(ChainedSection { section, tail })
    }
}

impl ToTokens for SectionTail {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Dot(dot_token) => dot_token.to_tokens(tokens),
            Self::Content {
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

impl ToTokens for ChainedSection {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.section.to_tokens(tokens);
        self.tail.to_tokens(tokens);
    }
}

#[cfg(test)]
mod tests {
    use super::ChainedSection;
    use quote::quote;

    #[test]
    fn parse_chained_section() {
        let tokens = quote!(naked().);

        asserts::tokens_are_matching!(ChainedSection, tokens, "naked () .");

        let tokens = quote!(get_first<T: Debug>(vec: Vec<T>) -> &T { 5 });

        asserts::tokens_are_matching!(
            ChainedSection,
            tokens,
            "get_first < T : Debug > (vec : Vec < T >) -> & T { 5 }"
        );

        let tokens = quote!(find<'a>(collection: &'a Collection) { explode() });

        asserts::tokens_are_matching!(
            ChainedSection,
            tokens,
            "find < 'a > (collection : & 'a Collection) { explode () }"
        );

        let tokens = quote!(find<'a, T>(vec: &'a Vec::<T>) where T: Sync { explode() });

        asserts::tokens_are_matching!(
            ChainedSection,
            tokens,
            "find < 'a , T > (vec : & 'a Vec :: < T >) where T : Sync { explode () }"
        );

        let tokens = quote!(naked()!);

        asserts::tokens_are_not_matching!(ChainedSection,tokens, "expected curly braces");
    }
}
