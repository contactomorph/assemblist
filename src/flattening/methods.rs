use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::token::Brace;

use crate::model::tree::BranchTail;

use super::{
    chain::BrowsingChain,
    generics::produce_last_generics,
    output::{
        produce_output_deconstruction, produce_output_instance, produce_output_name_with_namespace,
    }
};

// pub fn ⟨name⟩⟨generics⟩(self, ⟨args⟩) -> ⟨name⟩::Output ⟨generics⟩ {
//   let ⟨field1⟩ = self.⟨field1⟩;
//   …
//   let ⟨fieldN⟩ = self.⟨fieldN⟩;
//   ⟨output_instance⟩
// }
//
// ∨
//
// pub fn ⟨name⟩⟨generics⟩(self, ⟨args⟩) -> ⟨return_type⟩ {
//   let ⟨field1⟩ = self.⟨field1⟩;
//   …
//   let ⟨fieldN⟩ = self.⟨fieldN⟩;
//   ⟨body⟩
// }
pub fn produce_method(chain: &BrowsingChain, tail: &BranchTail, tokens: &mut TokenStream) {
    let output_section = chain.section();
    let span = Span::call_site();
    let spans = [span];

    if !chain.is_last() {
        syn::token::Pub { span }.to_tokens(tokens);
    }
    syn::token::Fn { span }.to_tokens(tokens);
    output_section.ident.to_tokens(tokens);
    produce_last_generics(chain, tokens);
    output_section.paren_token.surround(tokens, |tokens| {
        if !chain.is_last() {
            syn::token::SelfValue { span }.to_tokens(tokens);
            syn::token::Comma { spans }.to_tokens(tokens);
        }
        output_section.inputs.to_tokens(tokens)
    });

    match tail {
        BranchTail::Alternative { .. } => {
            syn::token::RArrow {
                spans: [span, span],
            }
            .to_tokens(tokens);
            produce_output_name_with_namespace(chain, tokens);
            Brace::default().surround(tokens, |tokens| {
                produce_output_deconstruction(chain, tokens);
                produce_output_instance(chain, tokens)
            });
        }
        BranchTail::Leaf {
            output,
            brace,
            body,
        } => {
            output.to_tokens(tokens);
            brace.surround(tokens, |tokens| {
                produce_output_deconstruction(chain, tokens);
                body.to_tokens(tokens);
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::flattening::chain::BrowsingChain;
    use crate::flattening::trunk::{flatten_branch_rec, flatten_trunk, FlatteningResult};
    use crate::model::tree::{BranchTail, Trunk};
    use crate::tools::asserts::assert_tokens_are_parsable_as;
    use proc_macro2::TokenStream;
    use quote::quote;

    use super::produce_method;

    fn collect_method_data(
        stream: &mut TokenStream,
        method_data: &mut Vec<TokenStream>,
        trunk: &Trunk,
        chain: &BrowsingChain,
        tail: &BranchTail,
    ) -> FlatteningResult {
        let mut method = TokenStream::new();
        produce_method(chain, tail, &mut method);
        method_data.push(method);

        if let BranchTail::Alternative { rest, .. } = tail {
            flatten_branch_rec(
                stream,
                trunk,
                &rest.0,
                Some(&chain),
                |stream, trunk, chain, tail| {
                    collect_method_data(stream, method_data, trunk, chain, tail)
                },
            )?
        }
        Ok(())
    }

    #[test]
    fn test_methods() {
        let tokens = quote!(pub(crate) fn first<'a>(text: &'a str, uuid: Uuid).second<T>(n: &'a mut T).third(l: usize) -> i64 { compose(l, uuid, combine(text, n)) });

        let trunk = assert_tokens_are_parsable_as::<Trunk>(tokens);

        let mut stream = TokenStream::new();
        let mut method_data = Vec::<TokenStream>::new();

        flatten_trunk(&mut stream, &trunk, |stream, trunk, chain, tail| {
            collect_method_data(stream, &mut method_data, trunk, chain, tail)
        })
        .expect("Should not have failed");

        assert_eq!(3, method_data.len());
        assert_eq!(
            method_data[0].to_string().as_str(),
            "fn first < 'a > (text : & 'a str , uuid : Uuid) -> first :: Output :: < 'a > { \
                first :: Output :: < 'a > { text , uuid , } \
            }",
        );
        assert_eq!(
            method_data[1].to_string().as_str(),
            "pub fn second < T > (self , n : & 'a mut T) -> second :: Output :: < T , 'a > { \
                let text = self . text ; \
                let uuid = self . uuid ; \
                second :: Output :: < T , 'a > { n , text , uuid , } \
            }",
        );
        assert_eq!(
            method_data[2].to_string().as_str(),
            "pub fn third (self , l : usize) -> i64 { \
                let n = self . n ; \
                let text = self . text ; \
                let uuid = self . uuid ; \
                compose (l , uuid , combine (text , n)) \
            }",
        );
    }
}
