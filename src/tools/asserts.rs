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
        Err(error) => {
            panic!("Failed to parse type `{}`: {}", std::any::type_name::<T>(), error);
        }
    }
}

pub fn assert_tokens_are_not_matching<T>(tokens: TokenStream, expected_error_text: &str)
where
    T: Parse + ToTokens,
{
    match parse2::<T>(tokens) {
        Ok(_) => {
            panic!("Should not be able to parse type `{}`", std::any::type_name::<T>());
        }
        Err(error) => {
            assert_eq!(expected_error_text, error.to_string().as_str());
        }
    }
}