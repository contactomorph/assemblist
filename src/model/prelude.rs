use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    Result, Token, Visibility,
};

use super::attribute::{AttributeBlock, DocumentationBlock};

pub struct Prelude {
    pub attr_block: AttributeBlock,
    pub vis: Visibility,
    pub asyncness: Option<Token![async]>,
}

pub struct Intro {
    pub doc_block: DocumentationBlock,
    pub attr_block: AttributeBlock,
    pub vis: Visibility,
    pub asyncness: Option<Token![async]>,
}

impl Parse for Intro {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut attr_block: AttributeBlock = input.parse()?;
        let doc_block = DocumentationBlock::extract_from(&mut attr_block);
        let vis: Visibility = input.parse()?;
        let asyncness: Option<Token![async]> = input.parse()?;
        Ok(Self {
            doc_block,
            attr_block,
            vis,
            asyncness,
        })
    }
}

impl ToTokens for Prelude {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.attr_block.to_tokens(tokens);
        self.vis.to_tokens(tokens);
        self.asyncness.to_tokens(tokens);
    }
}

impl Intro {
    pub fn split(self) -> (Prelude, DocumentationBlock) {
        let prelude = Prelude {
            attr_block: self.attr_block,
            vis: self.vis,
            asyncness: self.asyncness,
        };
        (prelude, self.doc_block)
    }
}
