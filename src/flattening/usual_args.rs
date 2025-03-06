use proc_macro2::TokenStream;
use quote::{quote_spanned, ToTokens};
use std::result::Result;
use syn::{
    punctuated::Punctuated, token::Comma, Attribute, FnArg, Ident, Pat, PatType, Token, Type,
};

pub struct UsualArg {
    pub attrs: Vec<Attribute>,
    pub ident: Ident,
    pub colon_token: Token![:],
    pub ty: Box<Type>,
}

impl ToTokens for UsualArg {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for attr in &self.attrs {
            attr.to_tokens(tokens);
        }
        self.ident.to_tokens(tokens);
        self.colon_token.to_tokens(tokens);
        self.ty.to_tokens(tokens);
    }
}

pub type UsualArgExtractionResult = Result<Vec<UsualArg>, TokenStream>;

impl UsualArg {
    pub fn extract_usual_args(args: &Punctuated<FnArg, Comma>) -> UsualArgExtractionResult {
        let mut output_args: Vec<Self> = Vec::new();
        for input in args.iter() {
            match input {
                FnArg::Typed(typed_arg) => {
                    let arg = Self::extract_usual_arg(typed_arg)?;
                    output_args.push(arg);
                }
                FnArg::Receiver(receiver) => {
                    let message = "self receiver is not supported";
                    let span = receiver.self_token.span;
                    return Err(quote_spanned! { span => compile_error!(#message); });
                }
            }
        }
        Ok(output_args)
    }

    fn extract_usual_arg(typed_arg: &PatType) -> Result<UsualArg, TokenStream> {
        match &*typed_arg.pat {
            Pat::Ident(pat_ident) => {
                if let Some(subpat) = &pat_ident.subpat {
                    let message = "Subpatterns are not supported";
                    let span = subpat.0.span;
                    return Err(quote_spanned! { span => compile_error!(#message); });
                }
                if let Some(by_ref) = &pat_ident.by_ref {
                    let message = "By ref parameter are not supported";
                    let span = by_ref.span;
                    return Err(quote_spanned! { span => compile_error!(#message); });
                }
                if let Some(mutability) = &pat_ident.mutability {
                    let message = "Mutability is not supported";
                    let span = mutability.span;
                    return Err(quote_spanned! { span => compile_error!(#message); });
                }
                Ok(UsualArg {
                    attrs: typed_arg.attrs.clone(),
                    ident: pat_ident.ident.clone(),
                    colon_token: typed_arg.colon_token,
                    ty: typed_arg.ty.clone(),
                })
            }
            _ => {
                let message = "Only basic identifier pattern is supported";
                let span = typed_arg.colon_token.span;
                Err(quote_spanned! { span => compile_error!(#message); })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tools::asserts::{
        assert_failure, assert_tokens_are_not_matching_punctuated,
        assert_tokens_are_parsable_punctuated_as,
    };
    use quote::quote;
    use syn::{punctuated::Punctuated, token::Comma, FnArg};

    use super::UsualArg;

    #[test]
    fn parse_usual_args() {
        let tokens = quote!(text: &'a str, n: i32);

        let punctuated = assert_tokens_are_parsable_punctuated_as::<FnArg, Comma>(tokens);

        let args =
            UsualArg::extract_usual_args(&punctuated).expect("Should not have conversion issue");

        assert_eq!(2, args.len());
        assert_eq!("text", args[0].ident.to_string().as_str());
        assert_eq!("n", args[1].ident.to_string().as_str());

        let tokens = quote!(pair: (usize, String), dates: Vec<Date>,);

        let punctuated = assert_tokens_are_parsable_punctuated_as::<FnArg, Comma>(tokens);

        let args =
            UsualArg::extract_usual_args(&punctuated).expect("Should not have conversion issue");

        assert_eq!(2, args.len());
        assert_eq!("pair", args[0].ident.to_string().as_str());
        assert_eq!("dates", args[1].ident.to_string().as_str());

        let tokens = quote!(text: &'a str, n: i32;);

        assert_tokens_are_not_matching_punctuated::<FnArg, Comma>(tokens, "unexpected token");

        let tokens = quote!(text: &'a str, self: Box<Self>);

        let punctuated = assert_tokens_are_parsable_punctuated_as::<FnArg, Comma>(tokens);

        let args = UsualArg::extract_usual_args(&punctuated);

        assert_failure(
            args,
            "compile_error ! (\"self receiver is not supported\") ;",
        );
    }
}
