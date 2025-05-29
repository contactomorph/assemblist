use proc_macro2::{Span, TokenStream};

use crate::model::prelude::Prelude;
use quote::{quote, ToTokens};

// ⟨attr⟩ ⟨visibility⟩
//
// ∨
//
// pub
pub fn produce_module_prelude(prelude: &Prelude, tokens: &mut TokenStream, depth: usize) {
    if depth == 0 {
        prelude.attr_block.to_tokens(tokens);
        prelude.vis.to_tokens(tokens);
    } else {
        syn::token::Pub {
            span: Span::call_site(),
        }
        .to_tokens(tokens);
    }
}

// ⟨?#[inline]⟩ ⟨attr⟩ ⟨visibility⟩ ⟨?async⟩
//
// ∨
//
// ⟨?#[inline]⟩ pub ⟨?async⟩
pub fn produce_method_prelude(
    prelude: &Prelude,
    tokens: &mut TokenStream,
    depth: usize,
    is_deepest: bool,
) {
    if !is_deepest {
        quote! { #[inline] }.to_tokens(tokens);
    }

    if depth == 0 {
        prelude.attr_block.to_tokens(tokens);
        prelude.vis.to_tokens(tokens);
    } else {
        syn::token::Pub {
            span: Span::call_site(),
        }
        .to_tokens(tokens);
    }

    if is_deepest {
        prelude.asyncness.to_tokens(tokens);
    }
}
