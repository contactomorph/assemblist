use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::token::Brace;

use super::{chain::BrowsingChain, generics::produce_complete_generics, output::{produce_output_deconstruction, produce_output_instance, produce_output_name}, trunk::FlatteningResult};

// pub fn ⟨name⟩⟨generics⟩(&self, ⟨args⟩) -> ⟨name⟩::Output ⟨generics⟩ {
//   ⟨output_instance⟩
// }
pub fn produce_inter_method(chain: &BrowsingChain, tokens: &mut TokenStream) {
    let output_section = chain.section();
    let span = Span::call_site();
    let spans = [span];

    if !chain.is_last() {
        syn::token::Pub { span }.to_tokens(tokens);
    }
    syn::token::Fn { span }.to_tokens(tokens);
    output_section.ident.to_tokens(tokens);
    produce_complete_generics(chain, tokens);
    output_section.paren_token
        .surround(tokens, |tokens| {
            if !chain.is_last() {
                syn::token::SelfValue { span }.to_tokens(tokens);
                syn::token::Comma { spans }.to_tokens(tokens);
            }
            output_section.inputs.to_tokens(tokens)
        });
    syn::token::RArrow { spans: [span, span] }.to_tokens(tokens);
    produce_output_name(chain, tokens);
    Brace::default().surround(tokens, |tokens| {
        for item in chain.into_iter().skip(1) {
            produce_output_deconstruction(item, tokens);
        }
        produce_output_instance(chain, tokens)
    });
}

#[cfg(test)]
mod tests {
    use crate::flattening::chain::BrowsingChain;
    use crate::flattening::trunk::{
        flatten_branch_rec, flatten_trunk, FlatteningResult,
    };
    use crate::model::tree::{BranchTail, Trunk};
    use crate::tools::asserts::assert_tokens_are_parsable_as;
    use proc_macro2::TokenStream;
    use quote::quote;

    use super::{produce_inter_method, produce_output_instance};

    fn collect_method_data(
        stream: &mut TokenStream,
        method_data: &mut Vec<TokenStream>,
        trunk: &Trunk,
        chain: &BrowsingChain,
        tail: &BranchTail,
    ) -> FlatteningResult {

        match tail {
            BranchTail::Alternative { .. } => {
                let mut inter_method = TokenStream::new();
                produce_inter_method(chain, &mut inter_method);
                method_data.push(inter_method);
            }
            _ => { }
        }

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
    fn test_inter_method() {
        let tokens = quote!(pub(crate) fn first<'a>(text: &'a str, uuid: Uuid).second<T>(n: &'a mut T).third(l: usize) {});

        let trunk = assert_tokens_are_parsable_as::<Trunk>(tokens);

        let mut stream = TokenStream::new();
        let mut method_data = Vec::<TokenStream>::new();

        flatten_trunk(&mut stream, &trunk, |stream, trunk, chain, tail| {
            collect_method_data(stream, &mut method_data, trunk, chain, tail)
        })
        .expect("Should not have failed");

        assert_eq!(2, method_data.len());
        assert_eq!(
            method_data[0].to_string().as_str(),
            "fn first < 'a > (text : & 'a str , uuid : Uuid) -> first :: Output < 'a > { \
                first :: Output < 'a > { text , uuid , } \
            }",
        );
        assert_eq!(
            method_data[1].to_string().as_str(),
            "pub fn second < T , 'a > (self , n : & 'a mut T) -> second :: Output < T , 'a > { \
                let text = self . text ; \
                let uuid = self . uuid ; \
                second :: Output < T , 'a > { n , text , uuid , } \
            }",
        );
    }
}