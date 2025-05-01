use crate::flattening::trunk::{flatten_trunk, FlatteningResult};
use crate::model::prelude::Prelude;
use crate::model::tree::{Branch, BranchTail, Tree};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::token::Brace;

use super::chain::BrowsingChain;
use super::methods::produce_method;
use super::output::{produce_inherent_impl_header_for_output, produce_output_definition};
use super::prelude::produce_short_prelude;

// #![allow(unused_imports)]
// use super::*;
fn produce_common_imports(tokens: &mut TokenStream) {
    let use_stream = quote! { #![allow(unused_imports)] use super::*; };
    tokens.extend(use_stream);
}

// ⟨prelude⟩ mod ⟨name⟩
//
// ∨
//
// pub mod ⟨name⟩
fn produce_module_header(prelude: &Prelude, chain: &BrowsingChain, tokens: &mut TokenStream) {
    let span = Span::call_site();
    if chain.depth() == 0 {
        produce_short_prelude(prelude, tokens);
    } else {
        syn::token::Pub { span }.to_tokens(tokens);
    }
    syn::token::Mod { span }.to_tokens(tokens);
    chain.section().ident.to_tokens(tokens);
}

// ⟨common_imports⟩
// ⟨output_definition⟩
// ⟨impl_header⟩ {
//   ⟨method1⟩
//   …
//   ⟨methodN⟩
// }
// ⟨sub_module1⟩
// …
// ⟨sub_moduleN⟩
fn produce_module_body(
    prelude: &Prelude,
    rest: &(Branch, Vec<Branch>),
    chain: &BrowsingChain,
    tokens: &mut TokenStream,
) -> FlatteningResult {
    produce_common_imports(tokens);
    produce_output_definition(chain, tokens);

    let asyncness = &prelude.asyncness;
    let mut continuations = Vec::<(BrowsingChain, &BranchTail)>::new();

    let first_chain = chain.concat(&rest.0.section)?;
    let first_tail = &rest.0.tail;
    continuations.push((first_chain, first_tail));

    for branch in &rest.1 {
        let next_chain = chain.concat(&branch.section)?;
        let next_tail = &branch.tail;
        continuations.push((next_chain, next_tail));
    }

    produce_inherent_impl_header_for_output(chain, tokens);
    Brace::default().surround(tokens, |tokens| {
        for (next_chain, next_tail) in &continuations {
            produce_method(asyncness, next_chain, next_tail, tokens);
        }
    });

    for (next_chain, next_tail) in continuations {
        produce_module(tokens, prelude, &next_chain, next_tail)?
    }
    Ok(())
}

// ⟨module_header⟩ {
//     ⟨module_body⟩
// }
fn produce_module(
    tokens: &mut TokenStream,
    prelude: &Prelude,
    chain: &BrowsingChain,
    tail: &BranchTail,
) -> FlatteningResult {
    if let BranchTail::Alternative { rest, .. } = tail {
        produce_module_header(prelude, chain, tokens);
        let mut result: FlatteningResult = Ok(());
        Brace::default().surround(tokens, |tokens| {
            result = produce_module_body(prelude, rest, chain, tokens);
        });
        result
    } else {
        Ok(())
    }
}

pub fn flatten(tree: Tree) -> TokenStream {
    let mut tokens = TokenStream::new();
    for trunk in tree.roots {
        if let Err(error) = flatten_trunk(&mut tokens, &trunk, produce_module) {
            return error;
        }
    }
    tokens
}

#[cfg(test)]
mod tests {
    use quote::quote;

    use crate::{
        flattening::tree::flatten, model::tree::Tree, tools::asserts::assert_tokens_are_parsable_as,
    };

    #[test]
    fn test_flatten_all() {
        let tokens = quote!(pub(crate) fn first<'a>(text: &'a str, uuid: Uuid).second<T>(n: &'a mut T).third(l: usize) -> i64 { compose(l, uuid, combine(text, n)) });

        let tree = assert_tokens_are_parsable_as::<Tree>(tokens);

        let output = flatten(tree);

        assert_eq!(
            output.to_string().as_str(),
            "pub (crate) fn first < 'a > (text : & 'a str , uuid : Uuid) -> first :: Output :: < 'a > { \
                first :: Output :: < 'a > { text , uuid , } \
            } \
            pub (crate) mod first { \
                # ! [allow (unused_imports)] \
                use super :: * ; \
                pub struct Output < 'a > { \
                    pub (super) text : & 'a str , \
                    pub (super) uuid : Uuid , \
                } \
                impl < 'a > Output < 'a > { \
                    pub fn second < T > (self , n : & 'a mut T) -> second :: Output :: < 'a , T > { \
                        let text = self . text ; \
                        let uuid = self . uuid ; \
                        second :: Output :: < 'a , T > { n , text , uuid , } \
                    } \
                } \
                pub mod second { \
                    # ! [allow (unused_imports)] \
                    use super :: * ; \
                    pub struct Output < 'a , T > { \
                        pub (super) n : & 'a mut T , \
                        pub (super) text : & 'a str , \
                        pub (super) uuid : Uuid , \
                    } \
                    impl < 'a , T > Output < 'a , T > { \
                        pub fn third (self , l : usize) -> i64 { \
                            let n = self . n ; \
                            let text = self . text ; \
                            let uuid = self . uuid ; \
                            compose (l , uuid , combine (text , n)) \
                        } \
                    } \
                } \
            }",
        );
    }
}
