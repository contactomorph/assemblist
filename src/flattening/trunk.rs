use super::chain::BrowsingChain;
use crate::model::tree::{BranchTail, Trunk};
use proc_macro2::TokenStream;

pub type FlatteningResult = std::result::Result<(), TokenStream>;

pub fn flatten_trunk(
    stream: &mut TokenStream,
    trunk: &Trunk,
    mut f: impl FnMut(&mut TokenStream, &Trunk, &BrowsingChain, &BranchTail) -> FlatteningResult,
) -> FlatteningResult {
    let chain = BrowsingChain::new(&trunk.branch.section)?;
    f(stream, trunk, &chain, &trunk.branch.tail)
}

#[cfg(test)]
mod tests {
    use crate::flattening::chain::BrowsingChain;
    use crate::flattening::trunk::{flatten_trunk, FlatteningResult};
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
        match chain.depth() {
            0 => {
                assert!(chain.previous().is_none());
                assert_eq!(2, chain.section().generics.params.len());
                assert_eq!(1, chain.args().len());
                assert!(if let BranchTail::Leaf { .. } = tail {
                    false
                } else {
                    true
                });
            }
            1 => {
                assert!(chain.previous().is_some());
                assert_eq!(1, chain.section().generics.params.len());
                assert_eq!(2, chain.args().len());
                assert!(if let BranchTail::Leaf { .. } = tail {
                    true
                } else {
                    false
                });
            }
            _ => {}
        }
        if let BranchTail::Alternative { rest, .. } = tail {
            let next_chain = chain.concat(&rest.0.section)?;
            let next_tail = &rest.0.tail;
            analyse_branch(stream, calls, trunk, &next_chain, next_tail)?;
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
