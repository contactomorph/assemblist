use proc_macro2::{Group, Ident, Punct, Spacing, Span, TokenStream, TokenTree};
use quote::quote_spanned;

struct AssemblistField {
    name: Ident,
    ty: Vec<TokenTree>,
    late: bool,
}

pub struct AssemblistSignature {
    name: Ident,
    argument_group: TokenStream,
    fields: Vec<AssemblistField>,
}

impl AssemblistSignature {
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
            if !field.late {
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
            if !field.late {
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
}

pub struct AssemblistTree {
    name: Ident,
    fields: Vec<AssemblistField>,
    argument_group: TokenStream,
    content: AssemblistTreeContent,
}

pub struct AssemblistDefinition {
    pub result_data: TokenStream,
    pub body: Group,
}

enum AssemblistTreeContent {
    Definition(AssemblistDefinition),
    SubTrees(Vec<AssemblistTree>),
}

enum Step {
    Starting,
    NameFound(Ident),
    NameFoundAndTypeStarting(Ident, Vec<TokenTree>),
}

impl AssemblistTree {
    pub fn from_sub_tree(
        name: Ident,
        cumulated_arguments: Vec<Group>,
        sub_tree: AssemblistTree,
    ) -> Self {
        let mut sub_trees = Vec::new();
        sub_trees.push(sub_tree);
        Self::from_sub_trees(name, cumulated_arguments, sub_trees)
    }

    pub fn from_sub_trees(
        name: Ident,
        cumulated_arguments: Vec<Group>,
        sub_trees: Vec<AssemblistTree>,
    ) -> Self {
        Self {
            name,
            argument_group: cumulated_arguments.last().unwrap().stream(),
            fields: Self::generate_fields(cumulated_arguments),
            content: AssemblistTreeContent::SubTrees(sub_trees),
        }
    }

    pub fn from_function(
        name: Ident,
        cumulated_arguments: Vec<Group>,
        definition: AssemblistDefinition,
    ) -> Self {
        Self {
            name,
            argument_group: cumulated_arguments.last().unwrap().stream(),
            fields: Self::generate_fields(cumulated_arguments),
            content: AssemblistTreeContent::Definition(definition),
        }
    }

    pub fn visit<T>(
        self,
        f_leaf: &mut impl FnMut(usize, AssemblistSignature, AssemblistDefinition) -> T,
        f_branch: &mut impl FnMut(usize, AssemblistSignature, Vec<T>) -> T,
    ) -> T {
        self.visit_with_level(0, f_leaf, f_branch)
    }

    fn generate_fields_from_group(
        argument_group: Group,
        late: bool,
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
                        fields.push(AssemblistField { name, ty, late })
                    }
                }
                (Step::NameFoundAndTypeStarting(name, mut ty), token) => {
                    ty.push(token);
                    step = Step::NameFoundAndTypeStarting(name, ty);
                }
            }
        }
        if let Step::NameFoundAndTypeStarting(name, ty) = step {
            fields.push(AssemblistField { name, ty, late });
        }
    }

    fn generate_fields(cumulated_arguments: Vec<Group>) -> Vec<AssemblistField> {
        let mut fields = Vec::<AssemblistField>::new();
        let group_count = cumulated_arguments.len();
        for (i, argument_group) in cumulated_arguments.into_iter().enumerate() {
            let late = i + 1 == group_count;
            Self::generate_fields_from_group(argument_group, late, &mut fields);
        }
        fields
    }

    fn visit_with_level<T>(
        self,
        level: usize,
        f_leaf: &mut impl FnMut(usize, AssemblistSignature, AssemblistDefinition) -> T,
        f_branch: &mut impl FnMut(usize, AssemblistSignature, Vec<T>) -> T,
    ) -> T {
        let signature = AssemblistSignature {
            name: self.name,
            argument_group: self.argument_group,
            fields: self.fields,
        };
        match self.content {
            AssemblistTreeContent::Definition(definition) => f_leaf(level, signature, definition),
            AssemblistTreeContent::SubTrees(sub_trees) => {
                let values = sub_trees
                    .into_iter()
                    .map(|tree| tree.visit_with_level(level + 1, f_leaf, f_branch))
                    .collect::<Vec<_>>();
                f_branch(level, signature, values)
            }
        }
    }
}

pub struct LocalizedFailure {
    span: Span,
    message: &'static str,
}

impl LocalizedFailure {
    pub fn new_err<T>(span: Span, message: &'static str) -> Result<T, Self> {
        Err(Self { span, message })
    }

    pub fn to_stream(self) -> TokenStream {
        let message = self.message;
        quote_spanned! { self.span => compile_error!(#message) }
    }
}
