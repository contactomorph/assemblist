use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    Result,
    parse::{Parse, ParseStream},
    parse2,
    punctuated::Punctuated,
};

pub fn tokens_are_matching<T>(tokens: TokenStream, expected_text: &str)
where
    T: Parse + ToTokens,
{
    match parse2::<T>(tokens) {
        Ok(item) => {
            let s = item.to_token_stream().to_string();
            equivalent(expected_text, s.as_str())
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

pub fn tokens_are_not_matching<T>(tokens: TokenStream, expected_error_text: &str)
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

pub fn tokens_are_parsable_as<T>(tokens: TokenStream) -> T
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
    fn into_punctuated(self) -> Punctuated<T, P> {
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

pub fn tokens_are_parsable_punctuated_as<T, P>(tokens: TokenStream) -> Punctuated<T, P>
where
    T: Parse,
    P: Parse,
{
    match parse2::<InnerPunctuaded<T, P>>(tokens) {
        Ok(inner) => inner.into_punctuated(),
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

pub fn tokens_are_not_matching_punctuated<T, P>(tokens: TokenStream, expected_error_text: &str)
where
    T: Parse,
    P: Parse,
{
    match parse2::<InnerPunctuaded<T, P>>(tokens) {
        Ok(_) => {
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

pub fn failure<T>(result: std::result::Result<T, TokenStream>, expected_error_text: &str) {
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

pub fn equivalent(a: &str, b: &str) {
    let mut a = simplify(a);
    let mut b = simplify(b);

    let mut common = Vec::<char>::new();
    let mut pair;
    loop {
        pair = (a.next(), b.next());
        match pair {
            (None, None) => return,
            (Some(x), Some(y)) if x == y => common.push(x),
            _ => break,
        }
    }

    let mut left = last_to_string(&common, 20);
    let mut right = left.clone();
    let common_len = left.len();
    let remaining_len = 70 - common_len;
    let mut div = String::with_capacity(common_len + 1);
    for _ in 0..common_len {
        div.push(' ')
    }
    div.push('v');
    if let Some(c) = pair.0 {
        left.push(c)
    }
    if let Some(c) = pair.1 {
        right.push(c)
    }
    for c in a.take(remaining_len) {
        left.push(c)
    }
    for c in b.take(remaining_len) {
        right.push(c)
    }
    panic!(
        "assertion `left == right` failed\n   div: {}\n  left: {}\n right: {}\n",
        div, left, right
    )
}

fn last_to_string(chars: &[char], max_len: usize) -> String {
    let len = chars.len();
    if chars.len() <= max_len {
        chars.iter().collect()
    } else {
        chars[len - max_len..len].iter().collect()
    }
}

fn simplify(text: &str) -> impl Iterator<Item = char> + '_ {
    let mut whitespace_found = false;
    let mut starting = true;
    text.chars()
        .flat_map(move |c| {
            let chars: [char; 2];
            if c.is_whitespace() {
                whitespace_found = true;
                chars = ['\0', '\0'];
            } else if whitespace_found && !starting {
                whitespace_found = false;
                starting = false;
                chars = [' ', c];
            } else {
                whitespace_found = false;
                starting = false;
                chars = ['\0', c];
            }
            chars
        })
        .filter(|c| *c != '\0')
}
