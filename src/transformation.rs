
use std::fmt::Debug;

use syn::parse::discouraged::Speculative;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::{Comma, Paren};
use syn::{parenthesized, parse_macro_input, Attribute, Error, FnArg, Generics, Ident, Pat, PatType, PatWild, Receiver, ReturnType, Token};
use syn::Result;
use quote::quote;

pub struct SectionSignature {
    pub ident: Ident,
    pub generics: Generics,
    pub paren_token: Paren,
    pub inputs: Punctuated<FnArg, Comma>,
    pub output: ReturnType,
}

impl Parse for SectionSignature {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident: Ident = input.parse()?;
        let mut generics: Generics = input.parse()?;

        let content;
        let paren_token = parenthesized!(content in input);
        let inputs = parse_fn_args(&content)?;

        let output: ReturnType = input.parse()?;
        generics.where_clause = input.parse()?;

        Ok(SectionSignature {
            ident,
            generics,
            paren_token,
            inputs,
            output,
        })
    }
}

impl ::quote::ToTokens for SectionSignature {
    fn to_tokens(&self, tokens: &mut ::proc_macro2::TokenStream) {
        self.ident.to_tokens(tokens);
        self.generics.to_tokens(tokens);
        self.paren_token.surround(tokens, |tokens| self.inputs.to_tokens(tokens));
        self.output.to_tokens(tokens);
        self.generics.where_clause.to_tokens(tokens);
    }
}

impl Debug for SectionSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f
            .debug_struct("SectionSignature")
            .field("ident", &self.ident)
            // .field("generics", &self.generics)
            // .field("inputs", &self.inputs)
            // .field("output", &self.output)
            .finish()
    }
}

fn parse_fn_args(
    input: ParseStream,
) -> Result<Punctuated<FnArg, Token![,]>> {
    let mut args = Punctuated::new();
    let mut has_receiver = false;

    while !input.is_empty() {
        let attrs = input.call(Attribute::parse_outer)?;

        let arg = parse_fn_arg(input, attrs)?;

        match &arg {
            FnArg::Receiver(receiver) if has_receiver => {
                return Err(Error::new(
                    receiver.self_token.span,
                    "unexpected second method receiver",
                ));
            }
            FnArg::Receiver(receiver) if !args.is_empty() => {
                return Err(Error::new(
                    receiver.self_token.span,
                    "unexpected method receiver",
                ));
            }
            FnArg::Receiver(_) => has_receiver = true,
            FnArg::Typed(_) => {}
        }
        args.push_value(arg);

        if input.is_empty() {
            break;
        }

        let comma: Token![,] = input.parse()?;
        args.push_punct(comma);
    }

    Ok(args)
}


fn parse_fn_arg(
    input: ParseStream,
    attrs: Vec<Attribute>,
) -> Result<FnArg> {
    let ahead = input.fork();
    if let Ok(mut receiver) = ahead.parse::<Receiver>() {
        input.advance_to(&ahead);
        receiver.attrs = attrs;
        return Ok(FnArg::Receiver(receiver));
    }

    // Hack to parse pre-2018 syntax in
    // test/ui/rfc-2565-param-attrs/param-attrs-pretty.rs
    // because the rest of the test case is valuable.
    if input.peek(Ident) && input.peek2(Token![<]) {
        let span = input.fork().parse::<Ident>()?.span();
        return Ok(FnArg::Typed(PatType {
            attrs,
            pat: Box::new(Pat::Wild(PatWild {
                attrs: Vec::new(),
                underscore_token: Token![_](span),
            })),
            colon_token: Token![:](span),
            ty: input.parse()?,
        }));
    }

    let pat = Box::new(Pat::parse_single(input)?);
    let colon_token: Token![:] = input.parse()?;

    Ok(FnArg::Typed(PatType {
        attrs,
        pat,
        colon_token,
        ty: input.parse()?,
    }))
}

// /// Parses the following syntax
// ///
// ///     $VISIBILITY $($SIGNATURE)+
// ///
// /// For example:
// ///     pub fn 
// struct _FnHeader {
//     pub sig: Signature,
// }

pub fn transform(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let signature = parse_macro_input!(input as SectionSignature);
    quote!{ fn #signature { todo!() } }.into()
}

#[cfg(test)]
mod tests {
    use proc_macro2::TokenStream;
    use quote::quote;
    use std::fmt::Debug;
    use super::SectionSignature;
    use syn::{parse::Parse, parse2};

    fn assert_tokens_matching<T>(tokens: TokenStream, expected_text: &str) where T: Parse + Debug {
        match parse2::<T>(tokens) {
            Ok(item) => {
                let s = format!("{:?}", item);
                assert_eq!(expected_text, s.as_str())
            }
            Err(_) => {
                panic!("Failed to parse the appropriate type.");
            }
        }
    }

    #[test]
    fn parse_section() {
        let tokens: TokenStream = quote!(dodo<'b>() -> usize);
        assert_tokens_matching::<SectionSignature>(
            tokens,
            r##"SectionSignature { ident: Ident(dodo) }"##
        )
    }
}