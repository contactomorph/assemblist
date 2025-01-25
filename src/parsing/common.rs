use proc_macro2::TokenTree;

use crate::tools::{joining_spans::join_spans_of, localized_failure::LocalizedFailure};

pub const FN_IDENT: &'static str = "fn";
pub const IMPL_IDENT: &'static str = "impl";
pub const FOR_IDENT: &'static str = "for";
pub const ONLY_FN_MESSAGE: &'static str = "Only functions should be declared in this scope.";
pub const ONLY_FN_IMPL_MESSAGE: &'static str =
    "Only functions and implementations should be declared in this scope.";

pub fn create_invalid_item_error<T>(
    prelude: &Vec<TokenTree>,
    message: &'static str,
) -> Result<Vec<T>, LocalizedFailure> {
    let span = join_spans_of(&prelude[0], prelude.last().unwrap());
    LocalizedFailure::new_err(span, message)
}
