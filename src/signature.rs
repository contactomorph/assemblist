use proc_macro2::{Group, Ident, Punct, Spacing, Span, TokenStream, TokenTree};
use quote::{quote, quote_spanned};
use std::fmt::Debug;

struct AssemblistField {
    name: Ident,
    ty: Vec<TokenTree>,
    depth: usize,
    max_depth: usize,
}

pub struct AssemblistSignature {
    name: Ident,
    argument_group: TokenStream,
    fields: Vec<AssemblistField>,
}

impl Debug for AssemblistSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}(", self.name)?;
        let mut first = true;
        for field in &self.fields {
            if first {
                first = false;
            } else {
                write!(f, ", ")?
            }
            let ty = &field.ty;
            write!(
                f,
                "{}: {}/{} {}",
                field.name,
                field.depth,
                field.max_depth,
                quote! {#(#ty)*}
            )?;
        }
        write!(f, ")")?;
        Ok(())
    }
}

enum Step {
    Starting,
    NameFound(Ident),
    NameFoundAndTypeStarting(Ident, Vec<TokenTree>),
}

impl AssemblistSignature {
    pub fn new(name: Ident, cumulated_arguments: (&Vec<Group>, Group)) -> AssemblistSignature {
        Self {
            name,
            argument_group: cumulated_arguments.1.stream(),
            fields: Self::generate_fields(cumulated_arguments),
        }
    }

    pub fn span(&self) -> Span {
        self.name.span()
    }

    pub fn name(&self) -> Ident {
        self.name.clone()
    }

    pub fn as_type_content(&self) -> TokenStream {
        let mut tokens = Vec::<TokenTree>::new();
        for field in &self.fields {
            tokens.push(TokenTree::Ident(Ident::new("pub", self.name.span())));
            tokens.push(TokenTree::Ident(field.name.clone()));
            tokens.push(TokenTree::Punct(Punct::new(':', Spacing::Alone)));
            tokens.extend(field.ty.iter().map(|t| t.clone()));
            tokens.push(TokenTree::Punct(Punct::new(',', Spacing::Alone)));
        }
        TokenStream::from_iter(tokens)
    }

    pub fn as_field_assignments(&self) -> TokenStream {
        let mut tokens = Vec::<TokenTree>::new();
        for field in &self.fields {
            tokens.push(TokenTree::Ident(field.name.clone()));
            tokens.push(TokenTree::Punct(Punct::new(':', Spacing::Alone)));
            if field.depth < field.max_depth {
                tokens.push(TokenTree::Ident(Ident::new("self", self.name.span())));
                tokens.push(TokenTree::Punct(Punct::new('.', Spacing::Alone)));
            }
            tokens.push(TokenTree::Ident(field.name.clone()));
            tokens.push(TokenTree::Punct(Punct::new(',', Spacing::Alone)));
        }
        TokenStream::from_iter(tokens)
    }

    pub fn as_variable_declaration(&self) -> TokenStream {
        let mut tokens = Vec::<TokenTree>::new();
        for field in &self.fields {
            if field.depth < field.max_depth {
                tokens.push(TokenTree::Ident(Ident::new("let", self.name.span())));
                tokens.push(TokenTree::Ident(field.name.clone()));
                tokens.push(TokenTree::Punct(Punct::new('=', Spacing::Alone)));
                tokens.push(TokenTree::Ident(Ident::new("self", self.name.span())));
                tokens.push(TokenTree::Punct(Punct::new('.', Spacing::Alone)));
                tokens.push(TokenTree::Ident(field.name.clone()));
                tokens.push(TokenTree::Punct(Punct::new(';', Spacing::Alone)));
            }
        }
        TokenStream::from_iter(tokens)
    }

    pub fn as_declaration(&self) -> TokenStream {
        let span = self.name.span();
        let name = self.name.clone();
        let argument_group = self.argument_group.clone();
        quote_spanned! { span => #name(#argument_group) }
    }

    pub fn as_declaration_with_self(&self) -> TokenStream {
        let span = self.name.span();
        let name = self.name.clone();
        let argument_group = self.argument_group.clone();
        quote_spanned! { span => #name(self, #argument_group) }
    }

    fn generate_fields_from_group(
        argument_group: &Group,
        depth: usize,
        max_depth: usize,
        fields: &mut Vec<AssemblistField>,
    ) {
        let mut step = Step::Starting;
        for token in argument_group.stream() {
            match (step, token) {
                (Step::Starting, TokenTree::Ident(ident)) => {
                    step = Step::NameFound(ident);
                }
                (Step::Starting, _) => {
                    step = Step::Starting;
                    break;
                }
                (Step::NameFound(name), TokenTree::Punct(punct)) if punct.as_char() == ':' => {
                    step = Step::NameFoundAndTypeStarting(name, Vec::new());
                }
                (Step::NameFound(_), _) => {
                    step = Step::Starting;
                    break;
                }
                (Step::NameFoundAndTypeStarting(name, ty), TokenTree::Punct(punct))
                    if punct.as_char() == ',' =>
                {
                    step = Step::Starting;
                    if 0 < ty.len() {
                        fields.push(AssemblistField {
                            name,
                            ty,
                            depth,
                            max_depth,
                        })
                    }
                }
                (Step::NameFoundAndTypeStarting(name, mut ty), token) => {
                    ty.push(token);
                    step = Step::NameFoundAndTypeStarting(name, ty);
                }
            }
        }
        if let Step::NameFoundAndTypeStarting(name, ty) = step {
            fields.push(AssemblistField {
                name,
                ty,
                depth,
                max_depth,
            });
        }
    }

    fn generate_fields(cumulated_arguments: (&Vec<Group>, Group)) -> Vec<AssemblistField> {
        let mut fields = Vec::<AssemblistField>::new();
        let max_depth = cumulated_arguments.0.len();
        for (depth, argument_group) in cumulated_arguments.0.into_iter().enumerate() {
            Self::generate_fields_from_group(argument_group, depth, max_depth, &mut fields);
        }
        Self::generate_fields_from_group(&cumulated_arguments.1, max_depth, max_depth, &mut fields);
        fields
    }
}
