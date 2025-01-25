use proc_macro2::{Group, Ident, Punct, Span, TokenStream, TokenTree};
use quote::quote_spanned;

#[derive(Clone)]
pub enum AssemblistGenericKind {
    Type,
    Lifetime,
    Const,
}

#[derive(Clone)]
pub struct AssemblistGenericArg {
    name: Ident,
    kind: AssemblistGenericKind,
    constraint: Vec<TokenTree>,
}

/**
 * Represent a list of generic arguments together with a list of function parameters.
 */
#[derive(Clone)]
pub struct AssemblistDeck {
    args: Group,
    generics: Vec<AssemblistGenericArg>,
}

impl AssemblistGenericArg {
    pub fn with_constraint(
        name: Ident,
        kind: AssemblistGenericKind,
        constraint: Vec<TokenTree>,
    ) -> AssemblistGenericArg {
        AssemblistGenericArg {
            name,
            kind,
            constraint,
        }
    }
    pub fn new(name: Ident, kind: AssemblistGenericKind) -> AssemblistGenericArg {
        AssemblistGenericArg {
            name,
            kind,
            constraint: Vec::new(),
        }
    }

    pub fn as_generic(&self) -> TokenStream {
        let span = self.name.span();
        let name = &self.name;
        let constraint = &self.constraint;
        let kind = match self.kind {
            AssemblistGenericKind::Type => {
                TokenTree::Group(Group::new(proc_macro2::Delimiter::None, TokenStream::new()))
            }
            AssemblistGenericKind::Const => TokenTree::Ident(Ident::new("const", span)),
            AssemblistGenericKind::Lifetime => {
                TokenTree::Punct(Punct::new('\'', proc_macro2::Spacing::Joint))
            }
        };
        quote_spanned! { span => #kind #name: #(#constraint)* }
    }
}

impl AssemblistDeck {
    pub fn new_generic(generics: Vec<AssemblistGenericArg>, args: Group) -> AssemblistDeck {
        AssemblistDeck { generics, args }
    }

    pub fn new_non_generic(args: Group) -> AssemblistDeck {
        AssemblistDeck {
            generics: Vec::new(),
            args,
        }
    }

    pub fn args_span(&self) -> Span {
        self.args.span()
    }

    pub fn args_stream(&self) -> TokenStream {
        self.args.stream()
    }

    pub fn generics_stream(&self) -> core::slice::Iter<'_, AssemblistGenericArg> {
        self.generics.iter()
    }

    pub fn generic_args_span(&self) -> Option<Span> {
        self.generics.last().map(|a| a.name.span())
    }

    pub fn as_generic_sequence(self) -> TokenStream {
        match self.generic_args_span() {
            None => TokenStream::new(),
            Some(span) => {
                quote_spanned! { span => <> }
            }
        }
    }
}
