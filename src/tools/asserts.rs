use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    parse::{self, Parse, ParseStream},
    parse2,
    punctuated::Punctuated,
    Result,
};

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
            panic!(
                "Failed to parse type `{}`: {}",
                std::any::type_name::<T>(),
                error
            );
        }
    }
}

pub fn assert_tokens_are_not_matching<T>(tokens: TokenStream, expected_error_text: &str)
where
    T: Parse + ToTokens,
{
    match parse2::<T>(tokens) {
        Ok(_) => {
            panic!(
                "Should not be able to parse type `{}`",
                std::any::type_name::<T>()
            );
        }
        Err(error) => {
            assert_eq!(expected_error_text, error.to_string().as_str());
        }
    }
}

pub fn assert_tokens_are_parsable_as<T>(tokens: TokenStream) -> T
where
    T: Parse,
{
    match parse2::<T>(tokens) {
        Ok(item) => item,
        Err(error) => {
            panic!(
                "Failed to parse type `{}`: {}",
                std::any::type_name::<T>(),
                error
            );
        }
    }
}

struct InnerPunctuaded<T, P> {
    inner: Vec<(T, P)>,
    last: Option<Box<T>>,
}

impl<T, P> Parse for InnerPunctuaded<T, P>
where
    T: Parse,
    P: Parse,
{
    fn parse(input: ParseStream) -> Result<Self> {
        let mut inner = Vec::<(T, P)>::new();
        while let Ok(value) = input.parse::<T>() {
            if let Ok(punct) = input.parse::<P>() {
                inner.push((value, punct));
            } else {
                return Ok(InnerPunctuaded {
                    inner,
                    last: Some(Box::new(value)),
                });
            }
        }
        Ok(InnerPunctuaded { inner, last: None })
    }
}

impl<T, P> InnerPunctuaded<T, P>
where
    T: Parse,
    P: Parse,
{
    fn to_punctuated(self) -> Punctuated<T, P> {
        let mut punctuated = Punctuated::<T, P>::new();
        for (value, punct) in self.inner.into_iter() {
            punctuated.push_value(value);
            punctuated.push_punct(punct);
        }
        if let Some(last) = self.last {
            punctuated.push_value(*last);
        }
        punctuated
    }
}

pub fn assert_tokens_are_parsable_punctuated_as<T, P>(tokens: TokenStream) -> Punctuated<T, P>
where
    T: Parse,
    P: Parse,
{
    match parse2::<InnerPunctuaded<T, P>>(tokens) {
        Ok(inner) => inner.to_punctuated(),
        Err(error) => {
            panic!(
                "Failed to parse type `{}` punctuated by `{}`: {}",
                std::any::type_name::<T>(),
                std::any::type_name::<P>(),
                error
            );
        }
    }
}

pub fn assert_tokens_are_not_matching_punctuated<T, P>(
    tokens: TokenStream,
    expected_error_text: &str,
) where
    T: Parse,
    P: Parse,
{
    match parse2::<InnerPunctuaded<T, P>>(tokens) {
        Ok(inner) => {
            panic!(
                "Should not be able to parse type `{}` punctuated by `{}`",
                std::any::type_name::<T>(),
                std::any::type_name::<P>()
            );
        }
        Err(error) => {
            assert_eq!(expected_error_text, error.to_string().as_str());
        }
    }
}

pub fn assert_failure<T>(result: std::result::Result<T, TokenStream>, expected_error_text: &str) {
    match result {
        Ok(_) => {
            panic!(
                "Should not find an instance of type `{}`",
                std::any::type_name::<T>()
            );
        }
        Err(error) => {
            assert_eq!(expected_error_text, error.to_string().as_str());
        }
    }
}
