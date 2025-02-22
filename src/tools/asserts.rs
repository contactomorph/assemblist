use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{parse::Parse, parse2};

pub fn assert_tokens_are_matching<T>(tokens: TokenStream, expected_text: &str)
where
    T: Parse + ToTokens,
{
    match parse2::<T>(tokens) {
        Ok(item) => {
            let s = item.to_token_stream().to_string();
            assert_eq!(expected_text, s.as_str())
        }
        Err(_) => {
            panic!("Failed to parse the appropriate type.");
        }
    }
}
