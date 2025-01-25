use proc_macro2::{Span, TokenTree};

pub fn join_spans(span1: Span, span2: Span) -> Span {
    span1.join(span2).unwrap_or(span1)
}

pub fn join_spans_of(token1: &TokenTree, token2: &TokenTree) -> Span {
    join_spans(token1.span(), token2.span())
}
