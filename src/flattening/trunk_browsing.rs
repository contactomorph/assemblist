use proc_macro::TokenStream;
use super::usual_args::UsualArg;
use crate::model::{
    section::Section,
    tree::{Branch, BranchTail, Trunk},
};

pub type FlatteningResult = std::result::Result<(), TokenStream>;

pub struct BrowsingChain<'a> {
    pub depth: usize,
    pub section: &'a Section,
    pub args: Vec<UsualArg>,
    pub previous: Option<&'a BrowsingChain<'a>>,
}

pub fn browse_branch_rec(
    trunk: &Trunk,
    branch: &Branch,
    previous: Option<&BrowsingChain<'_>>,
    mut f: impl FnMut(&Trunk, &BrowsingChain, &BranchTail) -> FlatteningResult,
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
    f(&trunk, &chain, &branch.tail)
}

pub fn browse_trunk(
    trunk: &Trunk,
    f: impl FnMut(&Trunk, &BrowsingChain, &BranchTail) -> FlatteningResult,
) -> FlatteningResult {
    browse_branch_rec(trunk, &trunk.branch, None, f)
}

#[cfg(test)]
mod tests {
    use crate::{
        flattening::trunk_browsing::{browse_branch_rec, browse_trunk, BrowsingChain, FlatteningResult},
        model::tree::{BranchTail, Trunk},
        tools::asserts::assert_tokens_are_parsable_as,
    };
    
    use quote::quote;

    #[test]
    fn browsing() {
        let tokens = quote!(fn first<'a, T>(text: &'a str).second<U>(n: &'a mut i32, ok: bool,) {});

        let trunk = assert_tokens_are_parsable_as::<Trunk>(tokens);

        fn analyse_branch(
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
                browse_branch_rec(trunk, &rest.0, Some(&chain), |trunk, chain, tail| {
                    analyse_branch(calls, trunk, chain, tail)
                })?
            }
            Ok(())
        }

        let mut calls = 0;
        browse_trunk(&trunk, |trunk, chain, tail| {
            analyse_branch(&mut calls, trunk, chain, tail)
        })
        .expect("Should not have failed");

        assert_eq!(2, calls);
    }
}
