use crate::{prelude::AssemblistPrelude, signature::AssemblistFnSignature};
use proc_macro2::{Group, Ident, Span, TokenStream};
use quote::quote_spanned;
use std::fmt::Debug;

pub struct AssemblistFnTree {
    prelude: AssemblistPrelude,
    signature: AssemblistFnSignature,
    content: AssemblistFnTreeContent,
}

pub struct AssemblistFnDefinition {
    pub result_data: TokenStream,
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
        name: Ident,
        cumulated_arguments: (&Vec<Group>, Group),
        sub_tree: AssemblistFnTree,
    ) -> Self {
        let mut sub_trees = Vec::new();
        sub_trees.push(sub_tree);
        Self::from_sub_trees(prelude, name, cumulated_arguments, sub_trees)
    }

    pub fn from_sub_trees(
        prelude: AssemblistPrelude,
        name: Ident,
        cumulated_arguments: (&Vec<Group>, Group),
        sub_trees: Vec<AssemblistFnTree>,
    ) -> Self {
        Self {
            prelude,
            signature: AssemblistFnSignature::new(name, cumulated_arguments),
            content: AssemblistFnTreeContent::SubTrees(sub_trees),
        }
    }

    pub fn from_function(
        prelude: AssemblistPrelude,
        name: Ident,
        cumulated_arguments: (&Vec<Group>, Group),
        definition: AssemblistFnDefinition,
    ) -> Self {
        Self {
            prelude,
            signature: AssemblistFnSignature::new(name, cumulated_arguments),
            content: AssemblistFnTreeContent::Definition(definition),
        }
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

pub struct LocalizedFailure {
    span: Span,
    message: &'static str,
}

impl LocalizedFailure {
    pub fn new_err<T>(span: Span, message: &'static str) -> Result<T, Self> {
        Err(Self { span, message })
    }

    pub fn to_stream(self) -> TokenStream {
        let message = self.message;
        quote_spanned! { self.span => compile_error!(#message) }
    }
}
