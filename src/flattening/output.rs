use crate::flattening::trunk::flatten_trunk;
use crate::model::tree::{BranchTail, Trunk};
use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::{token::Brace, Ident};

use super::chain::BrowsingChain;
use super::generics::produce_complete_generics;

// pub struct Output ⟨generics⟩ {
//      pub ⟨field1⟩: ⟨ty1⟩;
//      …
//      pub ⟨fieldN⟩: ⟨tyN⟩;
// }
pub fn produce_output_definition(chain: &BrowsingChain, tokens: &mut TokenStream) {
    let span = Span::call_site();

    syn::token::Pub { span }.to_tokens(tokens);
    syn::token::Struct { span }.to_tokens(tokens);
    Ident::new("Output", span).to_tokens(tokens);
    produce_complete_generics(chain, tokens);
    Brace::default().surround(tokens, |tokens| {
        for current in chain {
            for arg in current.args() {
                let span = arg.colon_token.span;
                syn::token::Pub { span }.to_tokens(tokens);
                for attr in &arg.attrs {
                    attr.to_tokens(tokens)
                }
                arg.ident.to_tokens(tokens);
                arg.colon_token.to_tokens(tokens);
                arg.ty.to_tokens(tokens);
                syn::token::Semi { spans: [span] }.to_tokens(tokens);
            }
        }
    });
}

// ⟨path⟩::Output ⟨generics⟩
pub fn produce_output_name(chain: &BrowsingChain, tokens: &mut TokenStream) {
    let span = Span::call_site();
    let spans = [span];
    let section = chain.section();

    section.ident.to_tokens(tokens);
    syn::token::PathSep { spans: [span, span] }.to_tokens(tokens);
    Ident::new("Output", span).to_tokens(tokens);
    produce_complete_generics(chain, tokens);
}

// ⟨path⟩::Output ⟨generics⟩ { ⟨field1⟩, …, ⟨fieldN⟩, }
pub fn produce_output_instance(chain: &BrowsingChain, tokens: &mut TokenStream) {
    let span = Span::call_site();
    let spans = [span];
    
    produce_output_name(chain, tokens);

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

    for current in chain {
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
    use crate::flattening::trunk::{
        flatten_branch_rec, flatten_trunk, FlatteningResult,
    };
    use crate::model::tree::{BranchTail, Trunk};
    use crate::tools::asserts::assert_tokens_are_parsable_as;
    use proc_macro2::TokenStream;
    use quote::quote;

    use super::{produce_output_definition, produce_output_instance};

    fn collect_output_data(
        stream: &mut TokenStream,
        output_data: &mut Vec<TokenStream>,
        trunk: &Trunk,
        chain: &BrowsingChain,
        tail: &BranchTail,
    ) -> FlatteningResult {
        let mut output_definition = TokenStream::new();
        produce_output_definition(chain, &mut output_definition);
        output_data.push(output_definition);

        let mut output_instance = TokenStream::new();
        produce_output_instance(chain, &mut output_instance);
        output_data.push(output_instance);

        if let BranchTail::Alternative { rest, .. } = tail {
            flatten_branch_rec(
                stream,
                trunk,
                &rest.0,
                Some(&chain),
                |stream, trunk, chain, tail| {
                    collect_output_data(stream, output_data, trunk, chain, tail)
                },
            )?
        }
        Ok(())
    }

    #[test]
    fn test_output_definition() {
        let tokens = quote!(pub(crate) fn first<'a>(text: &'a str).second<T>(n: &'a mut T) {});

        let trunk = assert_tokens_are_parsable_as::<Trunk>(tokens);

        let mut stream = TokenStream::new();
        let mut output_data = Vec::<TokenStream>::new();

        flatten_trunk(&mut stream, &trunk, |stream, trunk, chain, tail| {
            collect_output_data(stream, &mut output_data, trunk, chain, tail)
        })
        .expect("Should not have failed");

        assert_eq!(4, output_data.len());
        assert_eq!(
            output_data[0].to_string().as_str(),
            "pub struct Output < 'a > { pub text : & 'a str ; }"
        );
        assert_eq!(
            output_data[1].to_string().as_str(),
            "first :: Output < 'a > { text , }"
        );
        assert_eq!(
            output_data[2].to_string().as_str(),
            "pub struct Output < T , 'a > { pub n : & 'a mut T ; pub text : & 'a str ; }"
        );
        assert_eq!(
            output_data[3].to_string().as_str(),
            "second :: Output < T , 'a > { n , text , }"
        );
    }
}
