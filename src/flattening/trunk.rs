use super::{chain::BrowsingChain, method::produce_method, root_impl::produce_root_impl};
use crate::model::{
    attribute::DocumentationBlockView,
    branch::BranchTail,
    prelude::Prelude,
    trunk::{Trunk, TrunkAlternative},
};
use proc_macro2::TokenStream;
use quote::ToTokens;

pub type FlatteningResult = std::result::Result<(), TokenStream>;

pub fn flatten_trunk(
    tokens: &mut TokenStream,
    trunk: &Trunk,
    mut yield_module: impl FnMut(
        &mut TokenStream,
        &Prelude,
        &DocumentationBlockView,
        &BrowsingChain,
        &BranchTail,
    ) -> FlatteningResult,
) -> FlatteningResult {
    match &trunk.alternative {
        TrunkAlternative::Fn { documented, .. } => {
            let branch = &documented.branch;
            let view = documented.doc_block.create_view_starting_at(0);
            let chain = BrowsingChain::new(&branch.section)?;
            produce_method(&trunk.prelude, &view, &chain, &branch.tail, tokens);
            yield_module(tokens, &trunk.prelude, &view, &chain, &branch.tail)
        }
        TrunkAlternative::Impl { header, fn_trunks } => {
            let mut impl_body_tokens = TokenStream::new();
            for fn_trunk in fn_trunks {
                let branch = &fn_trunk.documented.branch;
                let view = fn_trunk.documented.doc_block.create_view_starting_at(0);
                let chain = BrowsingChain::new_root_impl(&header.self_ty, &branch.section)?;
                produce_method(
                    &fn_trunk.prelude,
                    &view,
                    &chain,
                    &branch.tail,
                    &mut impl_body_tokens,
                );
                yield_module(tokens, &fn_trunk.prelude, &view, &chain, &branch.tail)?;
            }
            trunk.prelude.to_tokens(tokens);
            produce_root_impl(header, &impl_body_tokens, tokens);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::flattening::chain::BrowsingChain;
    use crate::flattening::trunk::{flatten_trunk, FlatteningResult};
    use crate::model::branch::BranchTail;
    use crate::model::prelude::Prelude;
    use crate::model::trunk::Trunk;

    use proc_macro2::TokenStream;
    use quote::quote;

    fn analyse_branch(
        stream: &mut TokenStream,
        calls: &mut usize,
        prelude: &Prelude,
        chain: &BrowsingChain,
        tail: &BranchTail,
    ) -> FlatteningResult {
        *calls += 1;
        assert!(prelude.asyncness.is_none());
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
            let next_chain = chain.concat(&rest.0.branch.section)?;
            let next_tail = &rest.0.branch.tail;
            analyse_branch(stream, calls, prelude, &next_chain, next_tail)?;
        }
        Ok(())
    }

    #[test]
    fn test_trunk() {
        let tokens = quote!(fn first<'a, T>(text: &'a str).second<U>(n: &'a mut i32, ok: bool,) {});

        let trunk = asserts::tokens_are_parsable_as::<Trunk>(tokens);

        let mut calls = 0;
        let mut stream = TokenStream::new();

        flatten_trunk(&mut stream, &trunk, |stream, prelude, _, chain, tail| {
            analyse_branch(stream, &mut calls, prelude, chain, tail)
        })
        .expect("Should not have failed");

        assert_eq!(2, calls);
    }
}
