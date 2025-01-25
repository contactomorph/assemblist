use proc_macro2::{Delimiter, Group, Span, TokenStream, TokenTree};
use quote::quote_spanned;

use crate::tools::joining_spans::join_spans_of;

const PUB_IDENT: &'static str = "pub";

pub struct AssemblistPrelude {
    content: Option<Vec<TokenTree>>,
    visibility: AssemblistVisibility,
}

#[derive(Clone)]
enum AssemblistVisibility {
    Private,
    Public(Span),
    Rescricted(Group),
}

impl AssemblistPrelude {
    pub fn new(content: Vec<TokenTree>) -> AssemblistPrelude {
        AssemblistPrelude {
            visibility: Self::extract_visibility(&content),
            content: Some(content),
        }
    }

    pub fn span(&self) -> Option<Span> {
        match &self.content {
            None => None,
            Some(tokens) => {
                if let Some(first) = tokens.first() {
                    if let Some(last) = tokens.last() {
                        Some(join_spans_of(first, last))
                    } else {
                        Some(first.span())
                    }
                } else {
                    None
                }
            }
        }
    }

    pub fn reduce_to_visibility(&self) -> AssemblistPrelude {
        AssemblistPrelude {
            content: None,
            visibility: self.visibility.clone(),
        }
    }

    pub fn get_visibility_declaration(&self) -> TokenStream {
        match &self.visibility {
            AssemblistVisibility::Private => TokenStream::new(),
            AssemblistVisibility::Public(span) => {
                quote_spanned! { *span => pub }
            }
            AssemblistVisibility::Rescricted(group) => {
                quote_spanned! { group.span() => pub #group }
            }
        }
    }

    pub fn as_complete_declaration(self) -> TokenStream {
        match self.content {
            Some(token_trees) => TokenStream::from_iter(token_trees.into_iter()),
            None => Self::get_visibility_declaration(&self),
        }
    }

    fn extract_visibility(content: &Vec<TokenTree>) -> AssemblistVisibility {
        let mut i = content.len();
        if i == 0 {
            return AssemblistVisibility::Private;
        }
        i -= 1;
        loop {
            match &content[i] {
                TokenTree::Group(group) => {
                    if group.delimiter() == Delimiter::Bracket
                        || group.delimiter() == Delimiter::Brace
                    {
                        break AssemblistVisibility::Private;
                    }
                }
                TokenTree::Punct(punct) if punct.as_char() == ';' => {
                    break AssemblistVisibility::Private
                }
                TokenTree::Ident(ident) => {
                    if ident.to_string().as_str() == PUB_IDENT {
                        if i + 1 < content.len() {
                            let next = &content[i + 1];
                            if let TokenTree::Group(group) = next {
                                if group.delimiter() == Delimiter::Parenthesis {
                                    break AssemblistVisibility::Rescricted(group.clone());
                                }
                            }
                        }
                        break AssemblistVisibility::Public(ident.span());
                    }
                }
                _ => {}
            }
            if i == 0 {
                break AssemblistVisibility::Private;
            }
            i -= 1;
        }
    }
}
