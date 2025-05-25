use proc_macro2::TokenStream;
use std::result::Result;

use super::{ordered_gens::OrderedGenericList, usual_args::UsualArg};
use crate::model::section::Section;

#[derive(Clone, Copy)]
pub struct RootImplHeader<'a> {
    pub generics: &'a syn::Generics,
    pub root_type: &'a syn::Type,
}

enum BrowsingChainLink<'a> {
    Previous(&'a BrowsingChain<'a>),
    RootImpl(RootImplHeader<'a>),
    Beginning,
}

pub struct BrowsingChain<'a> {
    depth: usize,
    section: &'a Section,
    args: Vec<UsualArg>,
    gen_list: OrderedGenericList,
    link: BrowsingChainLink<'a>,
}

impl<'a> BrowsingChain<'a> {
    pub fn new(section: &'a Section) -> Result<BrowsingChain<'a>, TokenStream> {
        Self::create(BrowsingChainLink::Beginning, section)
    }

    pub fn new_root_impl(
        generics: &'a syn::Generics,
        root_type: &'a syn::Type,
        section: &'a Section,
    ) -> Result<BrowsingChain<'a>, TokenStream> {
        Self::create(
            BrowsingChainLink::RootImpl(RootImplHeader {
                generics,
                root_type,
            }),
            section,
        )
    }

    pub fn concat(&'a self, section: &'a Section) -> Result<BrowsingChain<'a>, TokenStream> {
        Self::create(BrowsingChainLink::Previous(self), section)
    }

    fn create(
        link: BrowsingChainLink<'a>,
        section: &'a Section,
    ) -> Result<BrowsingChain<'a>, TokenStream> {
        let args = UsualArg::extract_usual_args(&section.inputs)?;
        let depth = match link {
            BrowsingChainLink::Previous(previous) => previous.depth + 1,
            _ => 0,
        };
        let gen_list = match link {
            BrowsingChainLink::Previous(previous) => {
                OrderedGenericList::augment(Some(&previous.gen_list), &section.generics)
            }
            BrowsingChainLink::RootImpl(header) => {
                let previous = OrderedGenericList::augment(None, header.generics);
                OrderedGenericList::augment(Some(&previous), &section.generics)
            }
            _ => OrderedGenericList::augment(None, &section.generics),
        };
        let chain = BrowsingChain {
            link,
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
        match self.link {
            BrowsingChainLink::Previous(previous) => Some(previous),
            _ => None,
        }
    }

    pub fn root_header(&'a self) -> Option<RootImplHeader<'a>> {
        match self.link {
            BrowsingChainLink::Previous(previous) => previous.root_header(),
            BrowsingChainLink::RootImpl(header) => Some(header),
            _ => None,
        }
    }

    pub fn is_last(&self) -> bool {
        !matches!(&self.link, BrowsingChainLink::Previous(..))
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
                self.chain = current.previous();
                Some(current)
            }
            None => None,
        }
    }
}
