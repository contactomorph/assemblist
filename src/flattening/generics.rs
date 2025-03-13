use proc_macro2::{Span, TokenStream};
use quote::ToTokens;

use super::chain::BrowsingChain;

fn count_generics(chain: &BrowsingChain) -> usize {
    let n = chain.section().generics.params.len();
    match chain.previous() {
        Some(previous) => n + count_generics(previous),
        None => n,
    }
}

// :: <⟨generic1⟩, …, ⟨genericN⟩>
pub fn produce_complete_generics(chain: &BrowsingChain, must_prefix: bool, tokens: &mut TokenStream) {
    let span = Span::call_site();
    let spans = [span];
    let count = count_generics(chain);

    let mut first = true;

    if 0 < count {
        if must_prefix {
            syn::token::PathSep {
                spans: [span, span],
            }
            .to_tokens(tokens);
        }
        syn::token::Lt { spans }.to_tokens(tokens);

        for current in chain {
            for param in current.section().generics.params.iter() {
                if first {
                    first = false;
                } else {
                    syn::token::Comma { spans }.to_tokens(tokens)
                }
                param.to_tokens(tokens);
            }
        }

        syn::token::Gt { spans }.to_tokens(tokens);
    }
}

// <⟨generic1⟩, …, ⟨genericN⟩>
pub fn produce_last_generics(chain: &BrowsingChain, tokens: &mut TokenStream) {
    let spans = [Span::call_site()];
    let count = chain.section().generics.params.len();

    let mut first = true;

    if 0 < count {
        syn::token::Lt { spans }.to_tokens(tokens);

        for param in chain.section().generics.params.iter() {
            if first {
                first = false;
            } else {
                syn::token::Comma { spans }.to_tokens(tokens)
            }
            param.to_tokens(tokens);
        }

        syn::token::Gt { spans }.to_tokens(tokens);
    }
}
