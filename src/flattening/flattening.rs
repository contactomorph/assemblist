use crate::flattening::trunk::{flatten_trunk, FlatteningResult};
use crate::model::tree::{BranchTail, Tree, Trunk};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::token::Brace;

use super::chain::BrowsingChain;
use super::methods::produce_method;
use super::output::{
    produce_inherent_impl_header_for_output, produce_output_definition
};
use super::trunk::flatten_branch_rec;

// ⟨attr⟩ ⟨visibility⟩ ⟨async⟩
fn produce_method_prelude(trunk: &Trunk, tokens: &mut TokenStream) {
    for attr in &trunk.attrs {
        attr.to_tokens(tokens)
    }
    trunk.vis.to_tokens(tokens);
    trunk.asyncness.to_tokens(tokens);
}

// #![allow(unused_imports)]
// use super::*;
fn produce_common_imports(tokens: &mut TokenStream) {
    let use_stream = quote! { #![allow(unused_imports)] use super::*; };
    tokens.extend(use_stream);
}

// mod ⟨name⟩ {
//     ⟨common_imports⟩
//     ⟨output_definition⟩
//     ⟨impl⟩
// }
fn flatten_section(
    tokens: &mut TokenStream,
    trunk: &Trunk,
    chain: &BrowsingChain,
    tail: &BranchTail,
) -> FlatteningResult {
    let at_root = chain.depth() == 0;

    if at_root {
        produce_method_prelude(trunk, tokens);
        produce_method(chain, tail, tokens);
    }
    else {
        Brace::default().surround(tokens, |tokens| {
            produce_method(chain, tail, tokens);
        });
    }
    let span = trunk.fn_token.span;

    if let BranchTail::Alternative { rest, .. } = tail {
        if at_root {
            trunk.vis.to_tokens(tokens);
        }
        syn::token::Mod { span }.to_tokens(tokens);
        chain.section().ident.to_tokens(tokens);
        let mut result: FlatteningResult = Ok(());
        Brace::default().surround(tokens, |tokens| {
            if result.is_ok() {
                produce_common_imports(tokens);
                produce_output_definition(chain, tokens);
                produce_inherent_impl_header_for_output(chain, tokens);
                result =
                    flatten_branch_rec(tokens, trunk, &rest.0, Some(&chain), flatten_section);
            }
        });
        result
    } else {
        Ok(())
    }
}

pub fn flatten(tree: Tree) -> TokenStream {
    let mut stream = TokenStream::new();
    for trunk in &tree.roots {
        let res = flatten_trunk(&mut stream, &trunk, flatten_section);
        if let Err(error) = res {
            return error;
        }
    }
    stream
}

#[cfg(test)]
mod tests {
    use quote::quote;

    use crate::{
        flattening::flattening::flatten, model::tree::Tree,
        tools::asserts::assert_tokens_are_parsable_as,
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
                    pub fn second < T > (self , n : & 'a mut T) -> second :: Output :: < T , 'a > { \
                        let text = self . text ; \
                        let uuid = self . uuid ; \
                        second :: Output :: < T , 'a > { n , text , uuid , } \
                    } \
                } \
                mod second { \
                    # ! [allow (unused_imports)] \
                    use super :: * ; \
                    pub struct Output < T , 'a > { \
                        pub (super) n : & 'a mut T , \
                        pub (super) text : & 'a str , \
                        pub (super) uuid : Uuid , \
                    } \
                    impl < T , 'a > Output < T , 'a > { \
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
