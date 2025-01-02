use proc_macro2::{Span, TokenTree};
use std::fmt::Debug;

use crate::{fn_tree::AssemblistFnTree, prelude::AssemblistPrelude};

pub struct AssemblistImplTree {
    pub prelude: AssemblistPrelude,
    pub name: (TokenTree, Vec<TokenTree>),
    pub sub_trees: Vec<AssemblistFnTree>,
    span: Span,
}

pub enum AssemblistItemTree {
    Fn(AssemblistFnTree),
    Impl(AssemblistImplTree),
}

impl Into<AssemblistItemTree> for AssemblistFnTree {
    fn into(self) -> AssemblistItemTree {
        AssemblistItemTree::Fn(self)
    }
}

impl AssemblistImplTree {
    pub fn new(
        prelude: AssemblistPrelude,
        name: (TokenTree, Vec<TokenTree>),
        sub_trees: Vec<AssemblistFnTree>,
    ) -> AssemblistImplTree {
        let first_span = prelude.span().unwrap_or(name.0.span());
        let last_span = if let Some(last_token) = sub_trees.last() {
            last_token.span()
        } else if let Some(last_token) = name.1.last() {
            last_token.span()
        } else {
            name.0.span()
        };
        Self {
            prelude,
            name,
            sub_trees,
            span: first_span.join(last_span).unwrap(),
        }
    }

    pub fn span(&self) -> Span {
        self.span
    }
}

impl Debug for AssemblistImplTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "impl ")?;
        self.name.fmt(f)?;
        write!(f, " {{ ")?;
        let mut first = true;
        for subtree in &self.sub_trees {
            if first {
                first = false;
            } else {
                write!(f, " | ")?
            }
            write!(f, "{:?}", subtree)?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}
