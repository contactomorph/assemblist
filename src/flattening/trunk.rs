use super::{
    chain::BrowsingChain, methods::produce_method, prelude::produce_prelude,
    root_impl::produce_root_impl,
};
use crate::model::tree::{BranchTail, Trunk, TrunkAlternative};
use proc_macro2::TokenStream;

pub type FlatteningResult = std::result::Result<(), TokenStream>;

pub fn flatten_trunk(
    tokens: &mut TokenStream,
    trunk: &Trunk,
    mut yield_module: impl FnMut(
        &mut TokenStream,
        &Trunk,
        &BrowsingChain,
        &BranchTail,
    ) -> FlatteningResult,
) -> FlatteningResult {
    match &trunk.alternative {
        TrunkAlternative::Fn { branch, .. } => {
            let chain = BrowsingChain::new(&branch.section)?;
            produce_prelude(trunk, tokens);
            produce_method(&trunk.asyncness, &chain, &branch.tail, tokens);
            yield_module(tokens, trunk, &chain, &branch.tail)
        }
        TrunkAlternative::Impl { header, fn_trunks } => {
            let mut impl_body_tokens = TokenStream::new();
            for fn_trunk in fn_trunks {
                let branch = &fn_trunk.branch;
                let chain = BrowsingChain::new(&branch.section)?;
                //produce_prelude(trunk, &mut impl_body_tokens);
                produce_method(
                    &trunk.asyncness,
                    &chain,
                    &branch.tail,
                    &mut impl_body_tokens,
                );
                yield_module(tokens, trunk, &chain, &branch.tail)?;
            }
            produce_prelude(trunk, tokens);
            produce_root_impl(&trunk.asyncness, header, &impl_body_tokens, tokens);
            Ok(())
        }
    }
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
