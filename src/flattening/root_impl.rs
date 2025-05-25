use proc_macro2::TokenStream;

use quote::ToTokens;

use crate::model::trunk::ImplHeader;

// impl⟨generics⟩ ⟨self_type⟩ {
//   ⟨impl_body⟩
// }
pub fn produce_root_impl(
    header: &ImplHeader,
    impl_body_tokens: &TokenStream,
    tokens: &mut TokenStream,
) {
    header.impl_token.to_tokens(tokens);
    header.generics.to_tokens(tokens);
    header.self_ty.to_tokens(tokens);
    header.generics.where_clause.to_tokens(tokens);
    header
        .brace_token
        .surround(tokens, |tokens| impl_body_tokens.to_tokens(tokens));
}
