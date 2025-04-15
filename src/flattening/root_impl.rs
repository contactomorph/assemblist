use proc_macro2::TokenStream;
use syn::Token;

use quote::ToTokens;

use crate::model::tree::ImplHeader;

// ⟨async⟩ impl⟨generics⟩ ⟨self_type⟩ {
//   ⟨impl_body⟩
// }
pub fn produce_root_impl(
    asyncness: &Option<Token![async]>,
    header: &ImplHeader,
    impl_body_tokens: &TokenStream,
    tokens: &mut TokenStream,
) {
    asyncness.to_tokens(tokens);
    header.impl_token.to_tokens(tokens);
    header.generics.to_tokens(tokens);
    header.self_ty.to_tokens(tokens);
    header
        .brace_token
        .surround(tokens, |tokens| impl_body_tokens.to_tokens(tokens));
}
