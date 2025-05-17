use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse::Parse, Attribute, Expr, ExprLit, Lit, MetaNameValue};

const DOC_ATTRIBUTE_NAME: &str = "doc";
const SEPARATION: &str = "\"---\"";

pub struct AttributeBlock {
    attrs: Vec<Attribute>,
}

pub struct DocumentationSection {
    lines: Vec<Attribute>,
}

pub struct DocumentationBlock {
    sections: Vec<DocumentationSection>,
}

#[derive(Clone, Copy)]
pub struct DocumentationBlockView<'a> {
    depth: usize,
    sections: &'a [DocumentationSection],
}

enum AttributeKind {
    Doc,
    DocSeparation,
    Other,
}

fn classify_doc(nvm: &MetaNameValue) -> AttributeKind {
    if let Expr::Lit(ExprLit {
        lit: Lit::Str(str), ..
    }) = &nvm.value
    {
        let mut tokens = TokenStream::new();
        str.to_tokens(&mut tokens);
        if tokens.to_string() == SEPARATION {
            AttributeKind::DocSeparation
        } else {
            AttributeKind::Doc
        }
    } else {
        AttributeKind::Doc
    }
}

fn classify(attr: &Attribute) -> AttributeKind {
    if let syn::Meta::NameValue(name_value) = &attr.meta {
        if let Some(name) = name_value.path.get_ident() {
            if name == DOC_ATTRIBUTE_NAME {
                return classify_doc(name_value);
            }
        }
    }
    AttributeKind::Other
}

impl Parse for AttributeBlock {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        Ok(Self { attrs })
    }
}

impl ToTokens for DocumentationSection {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for line in &self.lines {
            line.to_tokens(tokens);
        }
    }
}

impl ToTokens for DocumentationBlock {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut first = true;
        for section in &self.sections {
            section.to_tokens(tokens);
            if first {
                first = true;
            } else {
                quote! { #[doc = "---"] }.to_tokens(tokens);
            }
        }
    }
}

impl ToTokens for AttributeBlock {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for attr in &self.attrs {
            attr.to_tokens(tokens);
        }
    }
}

impl AttributeBlock {
    pub fn is_empty(&self) -> bool {
        self.attrs.is_empty()
    }
}

impl DocumentationBlock {
    pub fn new() -> Self {
        Self {
            sections: Vec::new(),
        }
    }

    pub fn extract_from(attr_block: &mut AttributeBlock) -> Self {
        let mut attrs = Vec::<Attribute>::new();
        std::mem::swap(&mut attrs, &mut attr_block.attrs);
        let mut sections = Vec::<DocumentationSection>::new();
        let mut lines = Vec::<Attribute>::new();
        for attr in attrs {
            match classify(&attr) {
                AttributeKind::DocSeparation => {
                    sections.push(DocumentationSection { lines });
                    lines = Vec::<Attribute>::new();
                }
                AttributeKind::Doc => lines.push(attr),
                AttributeKind::Other => attr_block.attrs.push(attr),
            }
        }
        if !lines.is_empty() {
            sections.push(DocumentationSection { lines });
        }
        Self { sections }
    }

    pub fn create_view_starting_at(&self, depth: usize) -> DocumentationBlockView<'_> {
        DocumentationBlockView {
            depth,
            sections: &self.sections,
        }
    }
}

impl DocumentationBlockView<'_> {
    #[cfg(test)]
    pub fn new() -> DocumentationBlockView<'static> {
        DocumentationBlockView {
            depth: 0,
            sections: &[],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.sections.is_empty()
    }

    pub fn section_at(&self, depth: usize) -> Option<&DocumentationSection> {
        if self.depth <= depth {
            let relative_depth = depth - self.depth;
            self.sections.get(relative_depth)
        } else {
            None
        }
    }
}
