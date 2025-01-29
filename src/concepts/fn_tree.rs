use proc_macro2::{Group, Span, TokenTree};
use std::fmt::Debug;

use super::{prelude::AssemblistPrelude, signature::AssemblistFnSignature};
use crate::tools::joining_spans::join_spans;

pub struct AssemblistFnTree {
    prelude: AssemblistPrelude,
    signature: AssemblistFnSignature,
    content: AssemblistFnTreeContent,
    span: Span,
}

pub struct AssemblistFnDefinition {
    pub result_data: Vec<TokenTree>,
    pub body: Group,
}

enum AssemblistFnTreeContent {
    Definition(AssemblistFnDefinition),
    SubTrees(Vec<AssemblistFnTree>),
}

impl Debug for AssemblistFnTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.signature.fmt(f)?;
        match &self.content {
            AssemblistFnTreeContent::SubTrees(subtrees) => {
                write!(f, " {{ ")?;
                let mut first = true;
                for subtree in subtrees {
                    if first {
                        first = false;
                    } else {
                        write!(f, " | ")?
                    }
                    write!(f, "{:?}", subtree)?;
                }
                write!(f, "}}")?;
            }
            _ => {}
        }
        Ok(())
    }
}

impl AssemblistFnTree {
    pub fn from_sub_tree(
        prelude: AssemblistPrelude,
        signature: AssemblistFnSignature,
        sub_tree: AssemblistFnTree,
    ) -> Self {
        let mut sub_trees = Vec::new();
        sub_trees.push(sub_tree);
        Self::from_sub_trees(prelude, signature, sub_trees)
    }

    pub fn from_sub_trees(
        prelude: AssemblistPrelude,
        signature: AssemblistFnSignature,
        sub_trees: Vec<AssemblistFnTree>,
    ) -> Self {
        let first_span = prelude.span().unwrap_or(signature.span());
        let last_span = sub_trees.last().map(|t| t.span).unwrap_or(signature.span());
        Self {
            prelude,
            signature,
            content: AssemblistFnTreeContent::SubTrees(sub_trees),
            span: join_spans(first_span, last_span),
        }
    }

    pub fn from_function(
        prelude: AssemblistPrelude,
        signature: AssemblistFnSignature,
        definition: AssemblistFnDefinition,
    ) -> Self {
        let first_span = prelude.span().unwrap_or(signature.span());
        let last_span = definition.body.span();
        Self {
            prelude,
            signature,
            content: AssemblistFnTreeContent::Definition(definition),
            span: join_spans(first_span, last_span),
        }
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn visit<T>(
        self,
        f_leaf: &mut impl FnMut(
            usize,
            AssemblistPrelude,
            AssemblistFnSignature,
            AssemblistFnDefinition,
        ) -> T,
        f_branch: &mut impl FnMut(usize, AssemblistPrelude, AssemblistFnSignature, Vec<T>) -> T,
    ) -> T {
        self.visit_with_depth(0, f_leaf, f_branch)
    }

    fn visit_with_depth<T>(
        self,
        depth: usize,
        f_leaf: &mut impl FnMut(
            usize,
            AssemblistPrelude,
            AssemblistFnSignature,
            AssemblistFnDefinition,
        ) -> T,
        f_branch: &mut impl FnMut(usize, AssemblistPrelude, AssemblistFnSignature, Vec<T>) -> T,
    ) -> T {
        match self.content {
            AssemblistFnTreeContent::Definition(definition) => {
                f_leaf(depth, self.prelude, self.signature, definition)
            }
            AssemblistFnTreeContent::SubTrees(sub_trees) => {
                let values = sub_trees
                    .into_iter()
                    .map(|tree| tree.visit_with_depth(depth + 1, f_leaf, f_branch))
                    .collect::<Vec<_>>();
                f_branch(depth, self.prelude, self.signature, values)
            }
        }
    }
}
