use proc_macro2::{Group, Ident, Span, TokenStream, TokenTree};

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

#[derive(Clone)]
pub struct AssemblistArgumentDeck {
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
}

impl AssemblistArgumentDeck {
    pub fn new(generics: Vec<AssemblistGenericArg>, args: Group) -> AssemblistArgumentDeck {
        AssemblistArgumentDeck { generics, args }
    }

    pub fn args_span(&self) -> Span {
        self.args.span()
    }

    pub fn args_stream(&self) -> TokenStream {
        self.args.stream()
    }

    pub fn generic_args_span(&self) -> Option<Span> {
        self.generics.last().map(|a| a.name.span())
    }
}
