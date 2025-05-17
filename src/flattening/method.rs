use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::token::Brace;

use crate::model::{attribute::DocumentationBlockView, branch::BranchTail, prelude::Prelude};

use super::{
    chain::BrowsingChain,
    output::{
        produce_output_deconstruction, produce_output_instance, produce_output_name_with_namespace,
    },
    prelude::produce_method_prelude,
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
// pub ⟨asyncness⟩ fn ⟨name⟩⟨generics⟩(self, ⟨args⟩) -> ⟨return_type⟩ {
//   let ⟨field1⟩ = self.⟨field1⟩;
//   …
//   let ⟨fieldN⟩ = self.⟨fieldN⟩;
//   ⟨body⟩
// }
pub fn produce_method(
    prelude: &Prelude,
    view: &DocumentationBlockView,
    chain: &BrowsingChain,
    tail: &BranchTail,
    tokens: &mut TokenStream,
) {
    let output_section = chain.section();
    let span = Span::call_site();
    let spans = [span];

    let depth = chain.depth();
    let is_deepest = matches!(tail, BranchTail::Leaf { .. });

    view.section_at(depth).to_tokens(tokens);
    produce_method_prelude(prelude, tokens, depth, is_deepest);

    syn::token::Fn { span }.to_tokens(tokens);
    output_section.ident.to_tokens(tokens);
    chain.generics().produce_last_contrained_generics(tokens);
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
            ..
        } => {
            output.to_tokens(tokens);
            chain.generics().produce_where_clause(tokens);
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
    use crate::flattening::trunk::{flatten_trunk, FlatteningResult};
    use crate::model::attribute::DocumentationBlockView;
    use crate::model::branch::BranchTail;
    use crate::model::prelude::Prelude;
    use crate::model::trunk::Trunk;
    use proc_macro2::TokenStream;
    use quote::quote;

    use super::produce_method;

    fn collect_method_data(
        stream: &mut TokenStream,
        method_data: &mut Vec<TokenStream>,
        prelude: &Prelude,
        chain: &BrowsingChain,
        tail: &BranchTail,
    ) -> FlatteningResult {
        let mut method = TokenStream::new();
        let view = DocumentationBlockView::new();
        produce_method(prelude, &view, chain, tail, &mut method);
        method_data.push(method);

        if let BranchTail::Alternative { rest, .. } = tail {
            let next_chain = chain.concat(&rest.0.branch.section)?;
            let next_tail = &rest.0.branch.tail;
            collect_method_data(stream, method_data, prelude, &next_chain, next_tail)?
        }
        Ok(())
    }

    #[test]
    fn test_methods() {
        let tokens = quote!(pub(crate) fn first<'a>(text: &'a str, uuid: Uuid).second<T>(n: &'a mut T).third(l: usize) -> i64 { compose(l, uuid, combine(text, n)) });

        let trunk = asserts::tokens_are_parsable_as::<Trunk>(tokens);

        let mut stream = TokenStream::new();
        let mut method_data = Vec::<TokenStream>::new();

        flatten_trunk(&mut stream, &trunk, |stream, prelude, _, chain, tail| {
            collect_method_data(stream, &mut method_data, prelude, chain, tail)
        })
        .expect("Should not have failed");

        assert_eq!(3, method_data.len());
        asserts::equivalent!(
            method_data[0].to_string().as_str(),
            "pub (crate) fn first < 'a > (text : & 'a str , uuid : Uuid) -> first :: Output :: < 'a > {
                first :: Output :: < 'a > { text , uuid , }
            }"
        );
        asserts::equivalent!(
            method_data[1].to_string().as_str(),
            "pub fn second < T > (self , n : & 'a mut T) -> second :: Output :: < 'a , T > {
                let text = self . text ;
                let uuid = self . uuid ;
                second :: Output :: < 'a , T > { n , text , uuid , }
            }"
        );
        asserts::equivalent!(
            method_data[2].to_string().as_str(),
            "pub fn third (self , l : usize) -> i64 {
                let n = self . n ;
                let text = self . text ;
                let uuid = self . uuid ;
                compose (l , uuid , combine (text , n))
            }"
        );
    }
}
