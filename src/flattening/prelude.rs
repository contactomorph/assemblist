use proc_macro2::TokenStream;

use crate::model::tree::Trunk;
use quote::ToTokens;

// ⟨attr⟩ ⟨visibility⟩
pub fn produce_prelude(trunk: &Trunk, tokens: &mut TokenStream) {
    for attr in &trunk.attrs {
        attr.to_tokens(tokens)
    }
    trunk.vis.to_tokens(tokens);
}
