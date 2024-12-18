use crate::tree::{AssemblistDefinition, AssemblistTree, LocalizedFailure};
use proc_macro2::{token_stream::IntoIter, Delimiter, Group, Ident, Span, TokenStream, TokenTree};

const FN_IDENT: &'static str = "fn";

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
) -> Result<AssemblistDefinition, LocalizedFailure> {
    let mut last_span = first_token.span();
    let mut tokens = Vec::<TokenTree>::new();

    if let Some(body) = try_extract_body(first_token, &mut tokens) {
        let definition = AssemblistDefinition {
            body,
            result_data: TokenStream::from_iter(tokens),
        };
        return Ok(definition);
    }

    while let Some(token) = iter.next() {
        last_span = token.span();
        if let Some(body) = try_extract_body(token, &mut tokens) {
            let definition = AssemblistDefinition {
                body,
                result_data: TokenStream::from_iter(tokens),
            };
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
) -> Result<AssemblistTree, LocalizedFailure> {
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
                return Ok(AssemblistTree::from_function(
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
                let sub_trees =
                    parse_root(&mut body.stream().into_iter(), &new_cumulated_arguments)?;
                return Ok(AssemblistTree::from_sub_trees(
                    name,
                    (cumulated_arguments, arguments),
                    sub_trees,
                ));
            }
            (Step::ChainingFound { name, arguments }, TokenTree::Ident(new_name)) => {
                let mut new_cumulated_arguments = cumulated_arguments.clone();
                new_cumulated_arguments.push(arguments.clone());
                let sub_tree =
                    parse_assembly_tree(iter, Some(new_name), &new_cumulated_arguments, last_span)?;
                let assembly_tree =
                    AssemblistTree::from_sub_tree(name, (cumulated_arguments, arguments), sub_tree);
                return Ok(assembly_tree);
            }
            (_, token) => {
                return LocalizedFailure::new_err(token.span(), "Unexpected token");
            }
        }
    }
    LocalizedFailure::new_err(last_span, "Unexpected end of stream")
}

fn parse_root(
    iter: &mut IntoIter,
    cumulated_arguments: &Vec<Group>,
) -> Result<Vec<AssemblistTree>, LocalizedFailure> {
    let mut alternatives = Vec::<AssemblistTree>::new();
    while let Some(token) = iter.next() {
        match token {
            TokenTree::Ident(ref ident) => {
                if ident.to_string().as_str() == FN_IDENT {
                    let tree = parse_assembly_tree(iter, None, cumulated_arguments, ident.span())?;
                    alternatives.push(tree);
                } else {
                    return LocalizedFailure::new_err(
                        token.span(),
                        "Only function should be declared",
                    );
                }
            }
            _ => {
                return LocalizedFailure::new_err(token.span(), "Only function should be declared");
            }
        }
    }

    Ok(alternatives)
}

pub fn parse(input: proc_macro::TokenStream) -> Result<Vec<AssemblistTree>, LocalizedFailure> {
    let input: TokenStream = input.into();
    let no_arguments = Vec::<Group>::new();
    parse_root(&mut input.into_iter(), &no_arguments)
}
