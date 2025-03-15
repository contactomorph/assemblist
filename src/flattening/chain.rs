use proc_macro2::TokenStream;
use std::result::Result;

use super::{ordered_gens::OrderedGenericList, usual_args::UsualArg};
use crate::model::section::Section;

pub struct BrowsingChain<'a> {
    depth: usize,
    section: &'a Section,
    args: Vec<UsualArg>,
    gen_list: OrderedGenericList,
    previous: Option<&'a BrowsingChain<'a>>,
}

impl<'a> BrowsingChain<'a> {
    pub fn new(section: &'a Section) -> Result<BrowsingChain<'a>, TokenStream> {
        Self::create(None, section)
    }

    pub fn concat(&'a self, section: &'a Section) -> Result<BrowsingChain<'a>, TokenStream> {
        Self::create(Some(self), section)
    }

    pub fn create(
        previous: Option<&'a BrowsingChain<'a>>,
        section: &'a Section,
    ) -> Result<BrowsingChain<'a>, TokenStream> {
        let args = UsualArg::extract_usual_args(&section.inputs)?;
        let depth = match previous {
            Some(previous) => previous.depth + 1,
            None => 0,
        };
        let gen_list = match previous {
            Some(previous) => {
                OrderedGenericList::augment(Some(&previous.gen_list), &section.generics)
            }
            None => OrderedGenericList::augment(None, &section.generics),
        };
        let chain = BrowsingChain {
            previous,
            section,
            args,
            gen_list,
            depth,
        };
        Ok(chain)
    }

    pub fn depth(&self) -> usize {
        self.depth
    }

    pub fn section(&self) -> &Section {
        self.section
    }

    pub fn args(&self) -> &Vec<UsualArg> {
        &self.args
    }

    pub fn generics(&self) -> &OrderedGenericList {
        &self.gen_list
    }

    pub fn previous(&'a self) -> Option<&'a BrowsingChain<'a>> {
        self.previous
    }

    pub fn is_last(&self) -> bool {
        self.previous.is_none()
    }
}

impl<'a> IntoIterator for &'a BrowsingChain<'a> {
    type Item = &'a BrowsingChain<'a>;

    type IntoIter = BrowsingChainIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        BrowsingChainIterator { chain: Some(self) }
    }
}

pub struct BrowsingChainIterator<'a> {
    chain: Option<&'a BrowsingChain<'a>>,
}

impl<'a> Iterator for BrowsingChainIterator<'a> {
    type Item = &'a BrowsingChain<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.chain {
            Some(current) => {
                self.chain = current.previous;
                Some(current)
            }
            None => None,
        }
    }
}
