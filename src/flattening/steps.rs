use proc_macro2::{Span, TokenStream};

use crate::model::section::Section;
use quote::ToTokens;
use super::{flattener::{Flattener, FlatteningResult}, usual_args::UsualArg};

pub struct Step<'a> {
    section: &'a Section,
    args: Vec<UsualArg>,
}

pub struct Chain<'a> {
    current: Step<'a>,
    previous: Option<&'a Chain<'a>>,
}

impl<'a> Chain<'a> {
    pub fn count_generics(&self) -> usize {
        let n = self.current.section.generics.params.len();
        match self.previous {
            Some(previous) => n + previous.count_generics(),
            None => n,
        }
    }

    pub fn feed_generics_to_tokens(&self, tokens: &mut TokenStream) {
        if let Some(previous) = self.previous {
            previous.feed_generics_to_tokens(tokens);
        }
        let spans = [Span::call_site()];
        for param in self.current.section.generics.params.iter() {
            param.to_tokens(tokens);
            syn::token::Comma { spans }.to_tokens(tokens)
        }
    }
}

pub struct ChainGenerics<'a> {
    chain: &'a Chain<'a>
}

impl<'a> Flattener for ChainGenerics<'a> {
    type Context = ();

    fn to_flat_representation(&self, _: &mut Self::Context, tokens: &mut TokenStream) -> FlatteningResult {
        let chain = self.chain;
        let generics = &chain.current.section.generics;

        if 0 < self.chain.count_generics() {
            generics.lt_token.to_tokens(tokens);
            chain.feed_generics_to_tokens(tokens);
            generics.gt_token.to_tokens(tokens);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{flattening::usual_args::UsualArg, model::tree::{Branch, BranchTail}, tools::asserts::assert_tokens_are_parsable_as};
    use proc_macro2::TokenStream;
    use quote::quote;

    use super::{Chain, Step};

    fn get_sub_chain<'a>(branch: &'a Branch, previous: &'a Chain) -> (Option<&'a Branch>, Chain<'a>) {
        let section = &branch.section;
        let args = UsualArg::extract_usual_args(&section.inputs).expect("Bouh");
        let current = Step::<'a> {
            section: &section,
            args,
        };
        let next = Chain { previous: Some(previous), current };
        let rest = match &branch.tail {
            BranchTail::Alternative { rest, .. } => Some(&rest.0),
            BranchTail::Leaf { .. } => None,
        };
        (rest, next)
    }

    fn get_last_chain<'a>(branch: &'a Branch, previous: &'a Chain) -> Chain<'a> {
        let mut current_branch = branch;
        let mut current_chain = previous;
        loop {
            let (rest, next) = get_sub_chain(current_branch, current_chain);
            match rest {
                None => { return &next },
                Some(rest) => {
                    current_branch = rest;
                    current_chain = &next;
                },
            }
        }
    }

    #[test]
    fn flatten_chain_generics() {
        let tokens = quote!(first<'a, T>(text: &'a str).second<U>(n: &'a mut i32) {});

        let branch = assert_tokens_are_parsable_as::<Branch>(tokens);

        // let mut otokens = TokenStream::new();
        // let res: std::result::Result::<_,_> = tree.to_flat_representation(&mut otokens);
        // assert!(res.is_ok());
        // assert_eq!(
        //     otokens.to_string().as_str(),
        //     "pub (crate) mod first { \
        //         use super :: * ; \
        //         pub struct Output < 'a > { pub text : & 'a str ; } \
        //     } \
        //     pub (crate) fn first < 'a > (text : & 'a str) -> first :: Output < 'a > { \
        //         first :: Output < 'a > { text , } \
        //     }"
        // );
    }
}