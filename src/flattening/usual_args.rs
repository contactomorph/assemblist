use proc_macro2::{Span, TokenStream};
use quote::{quote_spanned, ToTokens};
use std::result::Result;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{Attribute, FnArg, Ident, Lifetime, Pat, PatType, Receiver, Token, Type};

enum UsualArgAlternative {
    Identified {
        ident: Ident,
        colon_token: Token![:],
        ty: Box<Type>,
    },
    Receiver {
        reference: Option<(Token![&], Option<Lifetime>)>,
        mutability: Option<Token![mut]>,
        self_token: Token![self],
    },
}

pub struct UsualArg {
    attrs: Vec<Attribute>,
    alt: UsualArgAlternative,
}

impl UsualArg {
    pub fn is_receiver(&self) -> bool {
        matches!(&self.alt, UsualArgAlternative::Receiver { .. })
    }

    pub fn push_ident_to_tokens(&self, tokens: &mut TokenStream) {
        match &self.alt {
            UsualArgAlternative::Identified { ident, .. } => ident.to_tokens(tokens),
            UsualArgAlternative::Receiver { .. } => {
                let span = Span::call_site();
                Ident::new("self_", span).to_tokens(tokens);
            }
        }
    }

    pub fn push_type_to_tokens(&self, root_impl_type: Option<&Type>, tokens: &mut TokenStream) {
        match &self.alt {
            UsualArgAlternative::Identified { ty, .. } => ty.to_tokens(tokens),
            UsualArgAlternative::Receiver {
                reference,
                mutability,
                ..
            } => match root_impl_type {
                Some(ty) => {
                    if let Some((ampersand, lifetime)) = reference {
                        ampersand.to_tokens(tokens);
                        lifetime.to_tokens(tokens);
                    }
                    mutability.to_tokens(tokens);
                    ty.to_tokens(tokens);
                }
                None => {
                    syn::Type::Tuple(syn::TypeTuple {
                        paren_token: syn::token::Paren::default(),
                        elems: Punctuated::<_, _>::new(),
                    })
                    .to_tokens(tokens);
                }
            },
        }
    }

    #[cfg(test)]
    fn name(&self) -> String {
        match &self.alt {
            UsualArgAlternative::Identified { ident, .. } => ident.to_string(),
            UsualArgAlternative::Receiver { .. } => "self_".to_string(),
        }
    }
}

impl ToTokens for UsualArg {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for attr in &self.attrs {
            attr.to_tokens(tokens);
        }
        match &self.alt {
            UsualArgAlternative::Identified {
                ident,
                colon_token,
                ty,
            } => {
                ident.to_tokens(tokens);
                colon_token.to_tokens(tokens);
                ty.to_tokens(tokens);
            }
            UsualArgAlternative::Receiver {
                reference,
                mutability,
                self_token,
            } => {
                if let Some((ampersand, lifetime)) = reference {
                    ampersand.to_tokens(tokens);
                    lifetime.to_tokens(tokens);
                }
                mutability.to_tokens(tokens);
                self_token.to_tokens(tokens);
            }
        }
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
                    let arg = Self::extract_receiver(receiver)?;
                    output_args.push(arg);
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
                    alt: UsualArgAlternative::Identified {
                        ident: pat_ident.ident.clone(),
                        colon_token: typed_arg.colon_token,
                        ty: typed_arg.ty.clone(),
                    },
                })
            }
            _ => {
                let message = "Only basic identifier pattern is supported";
                let span = typed_arg.colon_token.span;
                Err(quote_spanned! { span => compile_error!(#message); })
            }
        }
    }

    fn extract_receiver(receiver: &Receiver) -> Result<UsualArg, TokenStream> {
        let alt = UsualArgAlternative::Receiver {
            reference: receiver.reference.clone(),
            mutability: receiver.mutability,
            self_token: receiver.self_token,
        };
        if let Some(colon_token) = receiver.colon_token {
            let message = "Complex receivers type are not supported";
            let span = colon_token.span;
            Err(quote_spanned! { span => compile_error!(#message); })
        } else {
            Ok(UsualArg {
                attrs: receiver.attrs.clone(),
                alt,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use quote::quote;
    use syn::{token::Comma, FnArg};

    use super::UsualArg;

    #[test]
    fn parse_usual_args() {
        let tokens = quote!(text: &'a str, n: i32);

        let punctuated = asserts::tokens_are_parsable_punctuated_as::<FnArg, Comma>(tokens);

        let args =
            UsualArg::extract_usual_args(&punctuated).expect("Should not have conversion issue");

        assert_eq!(2, args.len());
        assert_eq!("text", args[0].name().as_str());
        assert_eq!("n", args[1].name().as_str());

        let tokens = quote!(pair: (usize, String), dates: Vec<Date>,);

        let punctuated = asserts::tokens_are_parsable_punctuated_as::<FnArg, Comma>(tokens);

        let args =
            UsualArg::extract_usual_args(&punctuated).expect("Should not have conversion issue");

        assert_eq!(2, args.len());
        assert_eq!("pair", args[0].name().as_str());
        assert_eq!("dates", args[1].name().as_str());

        let tokens = quote!(text: &'a str, n: i32;);

        asserts::tokens_are_not_matching_punctuated::<FnArg, Comma>(tokens, "unexpected token");

        let tokens = quote!(&'a self, text: &'a str);

        let punctuated = asserts::tokens_are_parsable_punctuated_as::<FnArg, Comma>(tokens);

        let args =
            UsualArg::extract_usual_args(&punctuated).expect("Should not have conversion issue");

        assert_eq!(2, args.len());
        assert_eq!("self_", args[0].name().as_str());
        assert_eq!("text", args[1].name().as_str());
    }
}
