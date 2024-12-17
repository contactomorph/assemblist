use crate::signature::AssemblistSignature;
use proc_macro2::{Group, Ident, Span, TokenStream};
use quote::quote_spanned;
use std::fmt::Debug;

pub struct AssemblistTree {
    signature: AssemblistSignature,
    content: AssemblistTreeContent,
}

pub struct AssemblistDefinition {
    pub result_data: TokenStream,
    pub body: Group,
}

enum AssemblistTreeContent {
    Definition(AssemblistDefinition),
    SubTrees(Vec<AssemblistTree>),
}

impl Debug for AssemblistTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.signature.fmt(f)?;
        match &self.content {
            AssemblistTreeContent::SubTrees(subtrees) => {
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

impl AssemblistTree {
    pub fn from_sub_tree(
        name: Ident,
        cumulated_arguments: (&Vec<Group>, Group),
        sub_tree: AssemblistTree,
    ) -> Self {
        let mut sub_trees = Vec::new();
        sub_trees.push(sub_tree);
        Self::from_sub_trees(name, cumulated_arguments, sub_trees)
    }

    pub fn from_sub_trees(
        name: Ident,
        cumulated_arguments: (&Vec<Group>, Group),
        sub_trees: Vec<AssemblistTree>,
    ) -> Self {
        Self {
            signature: AssemblistSignature::new(name, cumulated_arguments),
            content: AssemblistTreeContent::SubTrees(sub_trees),
        }
    }

    pub fn from_function(
        name: Ident,
        cumulated_arguments: (&Vec<Group>, Group),
        definition: AssemblistDefinition,
    ) -> Self {
        Self {
            signature: AssemblistSignature::new(name, cumulated_arguments),
            content: AssemblistTreeContent::Definition(definition),
        }
    }

    pub fn visit<T>(
        self,
        f_leaf: &mut impl FnMut(usize, AssemblistSignature, AssemblistDefinition) -> T,
        f_branch: &mut impl FnMut(usize, AssemblistSignature, Vec<T>) -> T,
    ) -> T {
        self.visit_with_depth(0, f_leaf, f_branch)
    }

    fn visit_with_depth<T>(
        self,
        depth: usize,
        f_leaf: &mut impl FnMut(usize, AssemblistSignature, AssemblistDefinition) -> T,
        f_branch: &mut impl FnMut(usize, AssemblistSignature, Vec<T>) -> T,
    ) -> T {
        match self.content {
            AssemblistTreeContent::Definition(definition) => {
                f_leaf(depth, self.signature, definition)
            }
            AssemblistTreeContent::SubTrees(sub_trees) => {
                let values = sub_trees
                    .into_iter()
                    .map(|tree| tree.visit_with_depth(depth + 1, f_leaf, f_branch))
                    .collect::<Vec<_>>();
                f_branch(depth, self.signature, values)
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
