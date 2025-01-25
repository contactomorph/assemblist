use proc_macro2::{token_stream::IntoIter, Delimiter, Span, TokenStream, TokenTree};

use crate::concepts::deck::AssemblistDeck;
use crate::concepts::item_tree::{AssemblistImplTree, AssemblistItemTree};
use crate::concepts::prelude::AssemblistPrelude;
use crate::tools::localized_failure::LocalizedFailure;

use super::common::{
    create_invalid_item_error, FN_IDENT, FOR_IDENT, IMPL_IDENT, ONLY_FN_IMPL_MESSAGE,
};
use super::fn_tree::{parse_assemblist_fn_tree, parse_assemblist_fn_trees};

fn parse_assemblist_impl_tree(
    iter: &mut IntoIter,
    mut last_span: Span,
    prelude: AssemblistPrelude,
) -> Result<AssemblistImplTree, LocalizedFailure> {
    let mut tokens = Vec::<TokenTree>::new();
    while let Some(token) = iter.next() {
        last_span = token.span();
        match token {
            TokenTree::Group(body) if body.delimiter() == Delimiter::Brace => {
                if tokens.is_empty() {
                    return LocalizedFailure::new_err(
                        last_span,
                        "Implementation should have a name.",
                    );
                }
                let first_token = tokens.remove(0);
                let no_decks = Vec::<AssemblistDeck>::new();
                let mut body_iter = body.stream().into_iter();
                let sub_trees = parse_assemblist_fn_trees(&mut body_iter, &no_decks)?;
                let tree = AssemblistImplTree::new(prelude, (first_token, tokens), sub_trees);
                return Ok(tree);
            }
            TokenTree::Punct(punct) if punct.as_char() == ';' => {
                return LocalizedFailure::new_err(last_span, "Implementation should have a body.");
            }
            TokenTree::Ident(ident) if ident.to_string().as_str() == FOR_IDENT => {
                return LocalizedFailure::new_err(
                    last_span,
                    "Only inherent implementations are allowed.",
                );
            }
            _ => tokens.push(token),
        }
    }
    LocalizedFailure::new_err(last_span, "Bof")
}

fn parse_items(
    iter: &mut IntoIter,
    decks: &Vec<AssemblistDeck>,
) -> Result<Vec<AssemblistItemTree>, LocalizedFailure> {
    let mut alternatives = Vec::new();
    let mut prelude = Vec::<TokenTree>::new();
    while let Some(token) = iter.next() {
        match token {
            TokenTree::Ident(ref ident) => {
                let keyword = ident.to_string();
                let keyword = keyword.as_str();
                if keyword == FN_IDENT {
                    let tree = parse_assemblist_fn_tree(
                        iter,
                        None,
                        decks,
                        ident.span(),
                        AssemblistPrelude::new(prelude),
                    )?;
                    alternatives.push(tree.into());
                    prelude = Vec::<TokenTree>::new();
                } else if keyword == IMPL_IDENT {
                    let tree = parse_assemblist_impl_tree(
                        iter,
                        ident.span(),
                        AssemblistPrelude::new(prelude),
                    )?;
                    alternatives.push(AssemblistItemTree::Impl(tree));
                    prelude = Vec::<TokenTree>::new();
                } else {
                    prelude.push(token);
                }
            }
            TokenTree::Group(ref group) if group.delimiter() == Delimiter::Brace => {
                prelude.push(token);
                return create_invalid_item_error(&prelude, ONLY_FN_IMPL_MESSAGE);
            }
            TokenTree::Punct(ref punct) if punct.as_char() == ';' => {
                prelude.push(token);
                return create_invalid_item_error(&prelude, ONLY_FN_IMPL_MESSAGE);
            }
            _ => prelude.push(token),
        }
    }

    if prelude.len() != 0 {
        return create_invalid_item_error(&prelude, ONLY_FN_IMPL_MESSAGE);
    }

    Ok(alternatives)
}

pub fn parse(input: proc_macro::TokenStream) -> Result<Vec<AssemblistItemTree>, LocalizedFailure> {
    let input: TokenStream = input.into();
    let no_decks = Vec::<AssemblistDeck>::new();
    parse_items(&mut input.into_iter(), &no_decks)
}
