use super::flattener::FlatteningResult;
use super::usual_args::UsualArg;
use crate::model::{
    section::Section,
    tree::{Branch, BranchTail, Trunk},
};

pub struct BrowsingChain<'a> {
    pub depth: usize,
    pub section: &'a Section,
    pub args: Vec<UsualArg>,
    pub previous: Option<&'a BrowsingChain<'a>>,
}

pub struct TrunkBrowser<'a> {
    trunk: &'a Trunk,
}

impl<'a> TrunkBrowser<'a> {
    pub fn new(trunk: &'a Trunk) -> Self {
        Self { trunk }
    }

    pub fn browse(&self, f: impl FnMut(&Trunk, &BrowsingChain, &BranchTail)) -> FlatteningResult {
        self.browse_rec(&self.trunk.branch, None, f)
    }

    fn browse_rec(
        &self,
        branch: &'a Branch,
        previous: Option<&'a BrowsingChain<'a>>,
        mut f: impl FnMut(&Trunk, &BrowsingChain, &BranchTail),
    ) -> FlatteningResult {
        let args = UsualArg::extract_usual_args(&branch.section.inputs)?;
        let depth = match previous {
            Some(previous) => previous.depth + 1,
            None => 0,
        };
        let chain = BrowsingChain::<'a> {
            previous,
            section: &branch.section,
            args,
            depth,
        };
        f(&self.trunk, &chain, &branch.tail);
        if let BranchTail::Alternative { rest, .. } = &branch.tail {
            self.browse_rec(&rest.0, Some(&chain), f)?
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        flattening::trunk_browser::TrunkBrowser, model::tree::Trunk,
        tools::asserts::assert_tokens_are_parsable_as,
    };
    use proc_macro2::TokenStream;
    use quote::quote;
    use syn::Visibility;

    #[test]
    fn browsing() {
        let tokens = quote!(fn first<'a, T>(text: &'a str).second<U>(n: &'a mut i32, ok: bool,) {});

        let trunk = assert_tokens_are_parsable_as::<Trunk>(tokens);

        let mut calls = 0;
        let browser = TrunkBrowser::new(&trunk);
        browser
            .browse(|trunk, chain, tail| {
                calls += 1;
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
            })
            .expect("Should not have failed");

        assert_eq!(2, calls);
    }
}
