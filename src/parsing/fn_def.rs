use proc_macro2::TokenStream;
use proc_macro2::{token_stream::IntoIter, Delimiter, Group, TokenTree};

use crate::concepts::fn_tree::AssemblistFnDefinition;
use crate::tools::localized_failure::LocalizedFailure;

fn try_extract_body(token: TokenTree, tokens: &mut Vec<TokenTree>) -> Option<Group> {
    match token {
        TokenTree::Punct(punct) if punct.as_char() == ';' => {
            let mut body = Group::new(Delimiter::Brace, TokenStream::new());
            body.set_span(punct.span());
            Some(body)
        }
        TokenTree::Group(body) if body.delimiter() == Delimiter::Brace => Some(body),
        token => {
            tokens.push(token);
            None
        }
    }
}

pub fn parse_definition(
    iter: &mut IntoIter,
    first_token: TokenTree,
) -> Result<AssemblistFnDefinition, LocalizedFailure> {
    let mut last_span = first_token.span();
    let mut result_data = Vec::<TokenTree>::new();

    if let Some(body) = try_extract_body(first_token, &mut result_data) {
        let definition = AssemblistFnDefinition { body, result_data };
        return Ok(definition);
    }

    while let Some(token) = iter.next() {
        last_span = token.span();
        if let Some(body) = try_extract_body(token, &mut result_data) {
            let definition = AssemblistFnDefinition { body, result_data };
            return Ok(definition);
        }
    }
    LocalizedFailure::new_err(last_span, "Unexpected end of stream")
}
