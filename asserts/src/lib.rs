use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    Result,
    parse::{Parse, ParseStream},
    parse2,
    punctuated::Punctuated,
};

#[macro_export]
macro_rules! equivalent {
    ($a: expr, $b: expr) => {
        match $crate::__equivalent($a, $b) {
            Ok(_) => {},
            Err(message) => panic!("{}", message),
        }
    }
}

#[macro_export]
macro_rules! tokens_are_matching {
    ($t: ty, $tokens: expr, $expected_text: expr) => {
        match $crate::__tokens_are_matching::<$t>($tokens, $expected_text) {
            Ok(_) => {},
            Err(message) => panic!("{}", message),
        }
    }
}


#[macro_export]
macro_rules! tokens_are_not_matching {
    ($t: ty, $tokens: expr, $expected_text: expr) => {
        match $crate::__tokens_are_not_matching::<$t>($tokens, $expected_text) {
            Ok(_) => {},
            Err(message) => panic!("{}", message),
        }
    }
}

pub type AssertResult = std::result::Result<(), String>;

pub fn __tokens_are_matching<T>(tokens: TokenStream, expected_text: &str) -> AssertResult
where
    T: Parse + ToTokens,
{
    match parse2::<T>(tokens) {
        Ok(item) => {
            let s = item.to_token_stream().to_string();
            __equivalent(expected_text, s.as_str())
        }
        Err(error) => {
            let message = format!(
                "Failed to parse type `{}`: {}",
                std::any::type_name::<T>(),
                error
            );
            Err(message)
        }
    }
}

pub fn __tokens_are_not_matching<T>(tokens: TokenStream, expected_error_text: &str) -> AssertResult
where
    T: Parse + ToTokens,
{
    match parse2::<T>(tokens) {
        Ok(_) => {
            let message = format!(
                "Should not be able to parse type `{}`",
                std::any::type_name::<T>()
            );
            Err(message)
        }
        Err(error) => {
            let error_message = error.to_string();
            if error_message.as_str() == expected_error_text {
                Ok(())
            } else {
                Err(format!(" Expected: {}\n   Actual: {}\n", expected_error_text, error_message))
            }
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

const MAX_PREFIX_LEN: usize = 20;
const LINE_LEN: usize = 100;
const LINE_COUNT: usize = 3;
const WHITE_SPACE: &'static str = "        ";

pub fn __equivalent(a: &str, b: &str) -> std::result::Result<(), String> {
    let mut a = simplify(a);
    let mut b = simplify(b);

    let mut common = Vec::<char>::new();
    let mut pair;
    loop {
        pair = (a.next(), b.next());
        match pair {
            (None, None) => return Ok(()),
            (Some(x), Some(y)) if x == y => common.push(x),
            _ => break,
        }
    }

    let (mut left, common_len) = last_to_string(&common, MAX_PREFIX_LEN);
    let mut right = left.clone();

    let mut div = String::with_capacity(common_len + 1);
    for _ in 0..common_len {
        div.push(' ')
    }
    div.push('↓');

    fill_side(&mut left, pair.0, common_len, a);
    fill_side(&mut right, pair.1, common_len, b);

    let message = format!(
        "assertion `left == right` failed\n  left: {}\n{}\n right: {}\n{}\n",
        div, left, div, right
    );

    Err(message)
}

fn fill_side(side: &mut String, first_char: Option<char>, common_len: usize, chars: impl Iterator<Item = char>) {
    if let Some(c) = first_char {
        side.push(c)
    }

    let remaining = LINE_LEN * LINE_COUNT - common_len;
    for (n, c) in chars.take(remaining).enumerate() {
        if 0 < (n + common_len + 1) % LINE_LEN {
            side.push(c)
        }
        else if n + 1 == remaining {
            side.push('…')
        }
        else {
            side.push('\n');
            side.push_str(WHITE_SPACE);
            side.push(c)
        }
    }
}

fn last_to_string(chars: &[char], max_len: usize) -> (String, usize) {
    let len = chars.len();
    if len <= max_len {
        let mut res = WHITE_SPACE.to_string();
        for c in chars.iter() {
            res.push(*c);
        }
        (res, len)
    } else {
        let mut res = WHITE_SPACE[..WHITE_SPACE.len() - 1].to_string();
        res.push('…');
        for c in chars[len - max_len..len].iter() {
            res.push(*c);
        }
        (res, max_len)
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
