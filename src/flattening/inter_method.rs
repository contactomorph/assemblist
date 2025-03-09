use proc_macro2::{Span, TokenStream};
use quote::ToTokens;

use super::{chain::BrowsingChain, generics::produce_complete_generics, trunk::FlatteningResult};

// pub fn ⟨name⟩⟨generics⟩(⟨self⟩, ⟨args⟩) -> ⟨name⟩::Output ⟨generics⟩ {
//   ⟨output_instance⟩
// }
pub fn produce_inter_function(output_chain: &BrowsingChain, self_chain: &BrowsingChain, tokens: &mut TokenStream) -> FlatteningResult {
    let output_section = output_chain.section();
    let span = Span::call_site();

    syn::token::Pub { span }.to_tokens(tokens);
    //output_section.fn_token.to_tokens(tokens);
    output_section.ident.to_tokens(tokens);
    produce_complete_generics(output_chain, tokens);
    output_section.paren_token
        .surround(tokens, |tokens| 
            output_section.inputs.to_tokens(tokens)
        );

    // match &chain.conclusion {
    //     Some(conclusion) => to_conclusion(conclusion, tokens),
    //     None => to_output_creation(chain, tokens),
    // }
    Ok(())
}