use super::{flattener::FlatteningResult, usual_args::UsualArg};
use crate::model::{
    section::Section,
    tree::{Branch, BranchTail},
};

pub struct Chain<'a> {
    pub depth: usize,
    pub section: &'a Section,
    pub args: Vec<UsualArg>,
    pub previous: Option<&'a Chain<'a>>,
}

fn browse_branch_rec<'a>(
    branch: &'a Branch,
    previous: Option<&'a Chain<'a>>,
    mut f: impl FnMut(&Chain),
) -> FlatteningResult {
    let args = UsualArg::extract_usual_args(&branch.section.inputs)?;
    let depth = match previous {
        Some(previous) => previous.depth + 1,
        None => 0,
    };
    let chain = Chain::<'a> {
        previous,
        section: &branch.section,
        args,
        depth,
    };
    f(&chain);
    if let BranchTail::Alternative { rest, .. } = &branch.tail {
        browse_branch_rec(&rest.0, Some(&chain), f)?
    }
    Ok(())
}

pub fn browse_branch<'a>(branch: &'a Branch, mut f: impl FnMut(&Chain)) -> FlatteningResult {
    browse_branch_rec(branch, None, f)
}

#[cfg(test)]
mod tests {
    use crate::{
        flattening::usual_args::UsualArg,
        model::tree::{Branch, BranchTail},
        tools::asserts::assert_tokens_are_parsable_as,
    };
    use proc_macro2::TokenStream;
    use quote::quote;

    use super::browse_branch;

    #[test]
    fn checking_chains() {
        let tokens = quote!(first<'a, T>(text: &'a str).second<U>(n: &'a mut i32, ok: bool,) {});

        let branch = assert_tokens_are_parsable_as::<Branch>(tokens);

        let mut calls = 0;
        browse_branch(&branch, |chain| {
            calls += 1;
            match chain.depth {
                0 => {
                    assert!(chain.previous.is_none());
                    assert_eq!(2, chain.section.generics.params.len());
                    assert_eq!(1, chain.args.len());
                }
                1 => {
                    assert!(chain.previous.is_some());
                    assert_eq!(1, chain.section.generics.params.len());
                    assert_eq!(2, chain.args.len());
                }
                _ => {}
            }
        })
        .expect("Should not have failed");

        assert_eq!(2, calls);
    }
}
