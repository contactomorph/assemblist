use crate::fn_tree::{AssemblistFnDefinition, AssemblistFnTree, LocalizedFailure};
use crate::item_tree::{AssemblistImplTree, AssemblistItemTree};
use crate::joining_spans::join_spans_of;
use crate::prelude::AssemblistPrelude;
use proc_macro2::{token_stream::IntoIter, Delimiter, Group, Ident, Span, TokenStream, TokenTree};

const FN_IDENT: &'static str = "fn";
const IMPL_IDENT: &'static str = "impl";
const ONLY_FN_MESSAGE: &'static str = "Only functions should be declared in this scope.";
const ONLY_FN_IMPL_MESSAGE: &'static str =
    "Only functions and implementations should be declared in this scope.";

enum Step {
    FnFound,
    NameFound { name: Ident },
    ArgsFound { name: Ident, arguments: Group },
    ChainingFound { name: Ident, arguments: Group },
}

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

fn parse_definition(
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

fn parse_assembly_tree(
    iter: &mut IntoIter,
    optional_function_name: Option<Ident>,
    cumulated_arguments: &Vec<Group>,
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
            (Step::NameFound { name }, TokenTree::Group(arguments))
                if arguments.delimiter() == Delimiter::Parenthesis =>
            {
                step = Step::ArgsFound { name, arguments }
            }
            (Step::ArgsFound { name, arguments }, TokenTree::Punct(punct))
                if punct.as_char() == '.' =>
            {
                step = Step::ChainingFound { name, arguments }
            }
            (Step::ArgsFound { name, arguments }, token) => {
                let definition = parse_definition(iter, token)?;
                return Ok(AssemblistFnTree::from_function(
                    prelude,
                    name,
                    (cumulated_arguments, arguments),
                    definition,
                ));
            }
            (Step::ChainingFound { name, arguments }, TokenTree::Group(body))
                if body.delimiter() == Delimiter::Brace =>
            {
                let mut new_cumulated_arguments = cumulated_arguments.clone();
                new_cumulated_arguments.push(arguments.clone());
                let sub_trees = parse_items(
                    &mut body.stream().into_iter(),
                    &new_cumulated_arguments,
                    None,
                )?;
                return Ok(AssemblistFnTree::from_sub_trees(
                    prelude,
                    name,
                    (cumulated_arguments, arguments),
                    sub_trees,
                ));
            }
            (Step::ChainingFound { name, arguments }, TokenTree::Ident(new_name)) => {
                let mut new_cumulated_arguments = cumulated_arguments.clone();
                new_cumulated_arguments.push(arguments.clone());
                let sub_tree = parse_assembly_tree(
                    iter,
                    Some(new_name),
                    &new_cumulated_arguments,
                    last_span,
                    prelude.reduce_to_visibility(),
                )?;
                let assembly_tree = AssemblistFnTree::from_sub_tree(
                    prelude,
                    name,
                    (cumulated_arguments, arguments),
                    sub_tree,
                );
                return Ok(assembly_tree);
            }
            (_, token) => {
                return LocalizedFailure::new_err(token.span(), "Unexpected token");
            }
        }
    }
    LocalizedFailure::new_err(last_span, "Unexpected end of stream")
}

fn parse_items<I>(
    iter: &mut IntoIter,
    cumulated_arguments: &Vec<Group>,
    convert_impl_to_item: Option<&fn(AssemblistImplTree) -> I>,
) -> Result<Vec<I>, LocalizedFailure>
where
    AssemblistFnTree: Into<I>,
{
    let mut alternatives = Vec::<I>::new();
    let mut prelude = Vec::<TokenTree>::new();
    while let Some(token) = iter.next() {
        match token {
            TokenTree::Ident(ref ident) => {
                let keyword = ident.to_string();
                let keyword = keyword.as_str();
                if keyword == FN_IDENT {
                    let tree = parse_assembly_tree(
                        iter,
                        None,
                        cumulated_arguments,
                        ident.span(),
                        AssemblistPrelude::new(prelude),
                    )?;
                    alternatives.push(tree.into());
                    prelude = Vec::<TokenTree>::new();
                } else if keyword == IMPL_IDENT {
                    return match convert_impl_to_item {
                        None => create_invalid_item_error(&prelude, ONLY_FN_MESSAGE),
                        Some(_conv) => {
                            todo!()
                        }
                    };
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

fn create_invalid_item_error<T>(
    prelude: &Vec<TokenTree>,
    message: &'static str,
) -> Result<Vec<T>, LocalizedFailure> {
    let span = join_spans_of(&prelude[0], prelude.last().unwrap());
    LocalizedFailure::new_err(span, message)
}

pub fn parse(input: proc_macro::TokenStream) -> Result<Vec<AssemblistItemTree>, LocalizedFailure> {
    let input: TokenStream = input.into();
    let no_arguments = Vec::<Group>::new();
    let to_item = &(AssemblistItemTree::Impl as fn(i: AssemblistImplTree) -> AssemblistItemTree);
    parse_items(&mut input.into_iter(), &no_arguments, Some(&to_item))
}
