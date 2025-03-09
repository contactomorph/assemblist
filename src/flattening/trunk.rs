use super::usual_args::UsualArg;
use crate::model::section::Section;
use crate::model::tree::{Branch, BranchTail, Trunk};
use proc_macro2::TokenStream;

pub type FlatteningResult = std::result::Result<(), TokenStream>;

pub struct BrowsingChain<'a> {
    pub depth: usize,
    pub section: &'a Section,
    pub args: Vec<UsualArg>,
    pub previous: Option<&'a BrowsingChain<'a>>,
}

impl<'a> IntoIterator for &'a BrowsingChain<'a> {
    type Item = &'a BrowsingChain<'a>;

    type IntoIter = BrowsingChainIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        BrowsingChainIterator { chain: Some(self) }
    }
}

pub struct BrowsingChainIterator<'a> {
    chain: Option<&'a BrowsingChain<'a>>,
}

impl<'a> Iterator for BrowsingChainIterator<'a> {
    type Item = &'a BrowsingChain<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.chain {
            Some(current) => {
                self.chain = current.previous;
                Some(current)
            }
            None => None
        }
    }
}

pub fn flatten_branch_rec(
    stream: &mut TokenStream,
    trunk: &Trunk,
    branch: &Branch,
    previous: Option<&BrowsingChain<'_>>,
    mut f: impl FnMut(&mut TokenStream, &Trunk, &BrowsingChain, &BranchTail) -> FlatteningResult,
) -> FlatteningResult {
    let args = UsualArg::extract_usual_args(&branch.section.inputs)?;
    let depth = match previous {
        Some(previous) => previous.depth + 1,
        None => 0,
    };
    let chain = BrowsingChain {
        previous,
        section: &branch.section,
        args,
        depth,
    };
    f(stream, &trunk, &chain, &branch.tail)
}

pub fn flatten_trunk(
    stream: &mut TokenStream,
    trunk: &Trunk,
    f: impl FnMut(&mut TokenStream, &Trunk, &BrowsingChain, &BranchTail) -> FlatteningResult,
) -> FlatteningResult {
    flatten_branch_rec(stream, trunk, &trunk.branch, None, f)
}

#[cfg(test)]
mod tests {
    use crate::flattening::trunk::{
        flatten_branch_rec, flatten_trunk, BrowsingChain, FlatteningResult,
    };
    use crate::model::tree::{BranchTail, Trunk};
    use crate::tools::asserts::assert_tokens_are_parsable_as;

    use proc_macro2::TokenStream;
    use quote::quote;

    fn analyse_branch(
        stream: &mut TokenStream,
        calls: &mut usize,
        trunk: &Trunk,
        chain: &BrowsingChain,
        tail: &BranchTail,
    ) -> FlatteningResult {
        *calls += 1;
        assert!(trunk.asyncness.is_none());
        match chain.depth {
            0 => {
                assert!(chain.previous.is_none());
                assert_eq!(2, chain.section.generics.params.len());
                assert_eq!(1, chain.args.len());
                assert!(!tail.is_last());
            }
            1 => {
                assert!(chain.previous.is_some());
                assert_eq!(1, chain.section.generics.params.len());
                assert_eq!(2, chain.args.len());
                assert!(tail.is_last());
            }
            _ => {}
        }
        if let BranchTail::Alternative { rest, .. } = tail {
            flatten_branch_rec(
                stream,
                trunk,
                &rest.0,
                Some(&chain),
                |stream, trunk, chain, tail| analyse_branch(stream, calls, trunk, chain, tail),
            )?
        }
        Ok(())
    }

    #[test]
    fn test_trunk() {
        let tokens = quote!(fn first<'a, T>(text: &'a str).second<U>(n: &'a mut i32, ok: bool,) {});

        let trunk = assert_tokens_are_parsable_as::<Trunk>(tokens);

        let mut calls = 0;
        let mut stream = TokenStream::new();

        flatten_trunk(&mut stream, &trunk, |stream, trunk, chain, tail| {
            analyse_branch(stream, &mut calls, trunk, chain, tail)
        })
        .expect("Should not have failed");

        assert_eq!(2, calls);
    }
}
