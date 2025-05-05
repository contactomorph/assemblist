use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::discouraged::Speculative;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::{Comma, Paren};
use syn::{
    parenthesized, Attribute, Error, FnArg, Generics, Ident, Pat, PatType, Receiver, Result, Token,
};

pub struct Section {
    pub ident: Ident,
    pub generics: Generics,
    pub paren_token: Paren,
    pub inputs: Punctuated<FnArg, Comma>,
}

impl Parse for Section {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident: Ident = input.parse()?;
        let generics: Generics = input.parse()?;

        let content;
        let paren_token = parenthesized!(content in input);
        let inputs = parse_fn_args(&content)?;

        Ok(Section {
            ident,
            generics,
            paren_token,
            inputs,
        })
    }
}

impl ToTokens for Section {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.ident.to_tokens(tokens);
        self.generics.to_tokens(tokens);
        self.paren_token
            .surround(tokens, |tokens| self.inputs.to_tokens(tokens));
        self.generics.where_clause.to_tokens(tokens);
    }
}

fn parse_fn_args(input: ParseStream) -> Result<Punctuated<FnArg, Token![,]>> {
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

fn parse_fn_arg(input: ParseStream, attrs: Vec<Attribute>) -> Result<FnArg> {
    let ahead = input.fork();
    if let Ok(mut receiver) = ahead.parse::<Receiver>() {
        input.advance_to(&ahead);
        receiver.attrs = attrs;
        return Ok(FnArg::Receiver(receiver));
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

#[cfg(test)]
mod tests {
    use super::Section;
    use quote::quote;

    #[test]
    fn parse_section() {
        let tokens = quote!(naked());

        asserts::tokens_are_matching!(Section, tokens, "naked ()");

        let tokens = quote!(get_first<T: Debug>(vec: Vec<T>));

        asserts::tokens_are_matching!(Section, tokens, "get_first < T : Debug > (vec : Vec < T >)");

        let tokens = quote!(find<'a>(collection: &'a Collection));

        asserts::tokens_are_matching!(
            Section,
            tokens,
            "find < 'a > (collection : & 'a Collection)"
        );

        let tokens = quote!(f(,));

        asserts::tokens_are_not_matching!(
            Section,
            tokens,
            "expected one of: identifier, `::`, `<`, `_`, literal, `const`, `ref`, `mut`, `&`, parentheses, square brackets, `..`, `const`"
        );

        let tokens = quote!(f(x:));

        asserts::tokens_are_not_matching!(
            Section,
            tokens,
            "unexpected end of input, expected one of: `for`, parentheses, `fn`, `unsafe`, `extern`, identifier, `::`, `<`, `dyn`, square brackets, `*`, `&`, `!`, `impl`, `_`, lifetime"
        );
    }
}
