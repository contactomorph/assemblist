use proc_macro2::TokenStream;

use crate::model::prelude::Prelude;
use quote::ToTokens;

// ⟨attr⟩ ⟨visibility⟩
pub fn produce_short_prelude(prelude: &Prelude, tokens: &mut TokenStream) {
    for attr in &prelude.attrs {
        attr.to_tokens(tokens);
    }
    prelude.vis.to_tokens(tokens);
}

// ⟨attr⟩ ⟨visibility⟩ ⟨asyncness⟩
pub fn produce_complete_prelude(prelude: &Prelude, tokens: &mut TokenStream) {
    for attr in &prelude.attrs {
        attr.to_tokens(tokens);
    }
    prelude.vis.to_tokens(tokens);
    prelude.asyncness.to_tokens(tokens);
}
