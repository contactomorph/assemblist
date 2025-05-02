use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::{
    token::{Brace, Paren},
    Ident,
};

use super::chain::BrowsingChain;

// pub struct Output ⟨generics⟩ {
//      pub (super) ⟨field1⟩: ⟨ty1⟩,
//      …
//      pub (super) ⟨fieldN⟩: ⟨tyN⟩,
// }
pub fn produce_output_definition(chain: &BrowsingChain, tokens: &mut TokenStream) {
    let span = Span::call_site();

    syn::token::Pub { span }.to_tokens(tokens);
    syn::token::Struct { span }.to_tokens(tokens);
    Ident::new("Output", span).to_tokens(tokens);
    chain
        .generics()
        .produce_complete_constrained_generics(tokens);
    Brace::default().surround(tokens, |tokens| {
        for current in chain {
            for arg in current.args() {
                let span = arg.colon_token.span;
                syn::token::Pub { span }.to_tokens(tokens);
                Paren::default().surround(tokens, |tokens| {
                    syn::token::Super { span }.to_tokens(tokens);
                });
                arg.ident.to_tokens(tokens);
                arg.colon_token.to_tokens(tokens);
                arg.ty.to_tokens(tokens);
                syn::token::Comma { spans: [span] }.to_tokens(tokens);
            }
        }
    });
}

// ⟨path⟩ :: Output :: ⟨generics⟩
pub fn produce_output_name_with_namespace(chain: &BrowsingChain, tokens: &mut TokenStream) {
    let span = Span::call_site();
    chain.section().ident.to_tokens(tokens);
    syn::token::PathSep {
        spans: [span, span],
    }
    .to_tokens(tokens);
    Ident::new("Output", span).to_tokens(tokens);
    chain
        .generics()
        .produce_complete_generic_names(true, tokens);
}

// impl ⟨generics⟩ Output ⟨generics⟩
pub fn produce_inherent_impl_header_for_output(chain: &BrowsingChain, tokens: &mut TokenStream) {
    let span = Span::call_site();
    syn::token::Impl { span }.to_tokens(tokens);
    chain
        .generics()
        .produce_complete_constrained_generics(tokens);
    Ident::new("Output", span).to_tokens(tokens);
    chain
        .generics()
        .produce_complete_generic_names(false, tokens);
}

// ⟨path⟩::Output ⟨generics⟩ { ⟨field1⟩, …, ⟨fieldN⟩, }
pub fn produce_output_instance(chain: &BrowsingChain, tokens: &mut TokenStream) {
    let span = Span::call_site();
    let spans = [span];

    produce_output_name_with_namespace(chain, tokens);

    Brace::default().surround(tokens, |tokens| {
        for current in chain {
            for arg in current.args() {
                arg.ident.to_tokens(tokens);
                syn::token::Comma { spans }.to_tokens(tokens)
            }
        }
    })
}

// let ⟨field1⟩ = self.⟨field1⟩;
// …
// let ⟨fieldN⟩ = self.⟨fieldN⟩;
pub fn produce_output_deconstruction(chain: &BrowsingChain, tokens: &mut TokenStream) {
    let span = Span::call_site();
    let spans = [span];

    for current in chain.into_iter().skip(1) {
        for arg in current.args() {
            syn::token::Let { span }.to_tokens(tokens);
            arg.ident.to_tokens(tokens);
            syn::token::Eq { spans }.to_tokens(tokens);
            syn::token::SelfValue { span }.to_tokens(tokens);
            syn::token::Dot { spans }.to_tokens(tokens);
            arg.ident.to_tokens(tokens);
            syn::token::Semi { spans }.to_tokens(tokens);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::flattening::chain::BrowsingChain;
    use crate::flattening::trunk::{flatten_trunk, FlatteningResult};
    use crate::model::prelude::Prelude;
    use crate::model::tree::BranchTail;
    use crate::model::tree::Trunk;
    use proc_macro2::TokenStream;
    use quote::quote;

    use super::{
        produce_inherent_impl_header_for_output, produce_output_definition, produce_output_instance,
    };

    fn collect_output_data(
        stream: &mut TokenStream,
        output_data: &mut Vec<TokenStream>,
        prelude: &Prelude,
        chain: &BrowsingChain,
        tail: &BranchTail,
    ) -> FlatteningResult {
        let mut output_definition = TokenStream::new();
        produce_output_definition(chain, &mut output_definition);
        output_data.push(output_definition);

        let mut output_instance = TokenStream::new();
        produce_output_instance(chain, &mut output_instance);
        output_data.push(output_instance);

        let mut output_instance = TokenStream::new();
        produce_inherent_impl_header_for_output(chain, &mut output_instance);
        output_data.push(output_instance);

        if let BranchTail::Alternative { rest, .. } = tail {
            let next_chain = chain.concat(&rest.0.section)?;
            let next_tail = &rest.0.tail;
            collect_output_data(stream, output_data, prelude, &next_chain, next_tail)?
        }
        Ok(())
    }

    #[test]
    fn test_output_definition() {
        let tokens = quote!(pub(crate) fn first<'a>(text: &'a str).second<T>(n: &'a mut T) {});

        let trunk = asserts::tokens_are_parsable_as::<Trunk>(tokens);

        let mut stream = TokenStream::new();
        let mut output_data = Vec::<TokenStream>::new();

        flatten_trunk(&mut stream, &trunk, |stream, prelude, chain, tail| {
            collect_output_data(stream, &mut output_data, prelude, chain, tail)
        })
        .expect("Should not have failed");

        assert_eq!(6, output_data.len());
        assert_eq!(
            output_data[0].to_string().as_str(),
            "pub struct Output < 'a > { pub (super) text : & 'a str , }"
        );
        assert_eq!(
            output_data[1].to_string().as_str(),
            "first :: Output :: < 'a > { text , }"
        );
        assert_eq!(
            output_data[2].to_string().as_str(),
            "impl < 'a > Output < 'a >"
        );
        assert_eq!(
            output_data[3].to_string().as_str(),
            "pub struct Output < 'a , T > { pub (super) n : & 'a mut T , pub (super) text : & 'a str , }"
        );
        assert_eq!(
            output_data[4].to_string().as_str(),
            "second :: Output :: < 'a , T > { n , text , }"
        );
        assert_eq!(
            output_data[5].to_string().as_str(),
            "impl < 'a , T > Output < 'a , T >"
        );
    }
}
