use proc_macro2::{token_stream::IntoIter, Delimiter, Ident, Span, TokenTree};

use crate::concepts::deck::{AssemblistDeck, AssemblistGenericArg};
use crate::concepts::fn_tree::AssemblistFnTree;
use crate::concepts::prelude::AssemblistPrelude;
use crate::concepts::signature::AssemblistFnSignature;
use crate::tools::localized_failure::LocalizedFailure;

use super::common::{
    create_invalid_item_error, FN_IDENT, IMPL_IDENT, ONLY_FN_IMPL_MESSAGE, ONLY_FN_MESSAGE,
};
use super::fn_def::parse_definition;
use super::generic_arg::parse_generic_arguments;

enum Step {
    FnFound,
    NameFound {
        name: Ident,
    },
    GenericNameFound {
        name: Ident,
        generics: Vec<AssemblistGenericArg>,
    },
    ArgsFound {
        name: Ident,
        deck: AssemblistDeck,
    },
    ChainingFound {
        name: Ident,
        deck: AssemblistDeck,
    },
}

pub fn parse_assemblist_fn_tree(
    iter: &mut IntoIter,
    optional_function_name: Option<Ident>,
    decks: &Vec<AssemblistDeck>,
    mut last_span: Span,
    prelude: AssemblistPrelude,
) -> Result<AssemblistFnTree, LocalizedFailure> {
    let mut step = match optional_function_name {
        None => Step::FnFound,
        Some(name) => Step::NameFound { name },
    };
    while let Some(token) = iter.next() {
        last_span = token.span();
        match (step, token) {
            (Step::FnFound, TokenTree::Ident(name)) => {
                step = Step::NameFound { name };
            }
            (Step::NameFound { name }, TokenTree::Punct(punct)) if punct.as_char() == '<' => {
                let generics = parse_generic_arguments(iter, last_span)?;
                step = Step::GenericNameFound { name, generics }
            }
            (Step::NameFound { name }, TokenTree::Group(arguments))
                if arguments.delimiter() == Delimiter::Parenthesis =>
            {
                let deck = AssemblistDeck::new_non_generic(arguments);
                step = Step::ArgsFound { name, deck }
            }
            (Step::GenericNameFound { name, generics }, TokenTree::Group(arguments))
                if arguments.delimiter() == Delimiter::Parenthesis =>
            {
                let deck = AssemblistDeck::new_generic(generics, arguments);
                step = Step::ArgsFound { name, deck }
            }
            (Step::ArgsFound { name, deck }, TokenTree::Punct(punct)) if punct.as_char() == '.' => {
                step = Step::ChainingFound { name, deck }
            }
            (Step::ArgsFound { name, deck }, token) => {
                let definition = parse_definition(iter, token)?;
                let signature = AssemblistFnSignature::new(name, (decks, deck));
                return Ok(AssemblistFnTree::from_function(
                    prelude, signature, definition,
                ));
            }
            (Step::ChainingFound { name, deck }, TokenTree::Group(body))
                if body.delimiter() == Delimiter::Brace =>
            {
                let mut new_decks: Vec<_> = decks.clone();
                new_decks.push(deck.clone());
                let sub_trees =
                    parse_assemblist_fn_trees(&mut body.stream().into_iter(), &new_decks)?;
                let signature = AssemblistFnSignature::new(name, (decks, deck));
                return Ok(AssemblistFnTree::from_sub_trees(
                    prelude, signature, sub_trees,
                ));
            }
            (Step::ChainingFound { name, deck }, TokenTree::Ident(new_name)) => {
                let mut new_decks: Vec<_> = decks.clone();
                new_decks.push(deck.clone());
                let sub_tree = parse_assemblist_fn_tree(
                    iter,
                    Some(new_name),
                    &new_decks,
                    last_span,
                    prelude.reduce_to_visibility(),
                )?;
                let signature = AssemblistFnSignature::new(name, (decks, deck));
                let assembly_tree = AssemblistFnTree::from_sub_tree(prelude, signature, sub_tree);
                return Ok(assembly_tree);
            }
            (_, token) => {
                return LocalizedFailure::new_err(token.span(), "Unexpected token");
            }
        }
    }
    LocalizedFailure::new_err(last_span, "Unexpected end of stream")
}

pub fn parse_assemblist_fn_trees(
    iter: &mut IntoIter,
    decks: &Vec<AssemblistDeck>,
) -> Result<Vec<AssemblistFnTree>, LocalizedFailure> {
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
                    return create_invalid_item_error(&prelude, ONLY_FN_MESSAGE);
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
