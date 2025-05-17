use crate::flattening::trunk::FlatteningResult;
use crate::model::attribute::DocumentationBlockView;
use crate::model::branch::{BranchTail, DocumentedBranch};
use crate::model::prelude::Prelude;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::token::Brace;

use super::chain::BrowsingChain;
use super::doc::produce_linked_doc_for_module;
use super::method::produce_method;
use super::output::{produce_inherent_impl_header_for_output, produce_output_definition};
use super::prelude::produce_module_prelude;

// #![allow(unused_imports)]
// use super::*;
fn produce_common_imports(tokens: &mut TokenStream) {
    let use_stream = quote! { #![allow(unused_imports)] use super::*; };
    tokens.extend(use_stream);
}

// ⟨prelude⟩ mod ⟨name⟩
//
// ∨
//
// pub mod ⟨name⟩
fn produce_module_header(prelude: &Prelude, chain: &BrowsingChain, tokens: &mut TokenStream) {
    produce_module_prelude(prelude, tokens, chain.depth());
    syn::token::Mod {
        span: Span::call_site(),
    }
    .to_tokens(tokens);
    chain.section().ident.to_tokens(tokens);
}

// ⟨common_imports⟩
// ⟨output_definition⟩
// ⟨impl_header⟩ {
//   ⟨method1⟩
//   …
//   ⟨methodN⟩
// }
// ⟨sub_module1⟩
// …
// ⟨sub_moduleN⟩
fn produce_module_body(
    prelude: &Prelude,
    view: &DocumentationBlockView,
    rest: &(DocumentedBranch, Vec<DocumentedBranch>),
    chain: &BrowsingChain,
    tokens: &mut TokenStream,
) -> FlatteningResult {
    produce_common_imports(tokens);
    produce_output_definition(chain, tokens);

    let mut continuations = Vec::<(DocumentationBlockView, BrowsingChain, &BranchTail)>::new();

    let depth = chain.depth() + 1;

    let branch_view = rest.0.doc_block.create_view_starting_at(depth);
    let view = if branch_view.is_empty() {
        *view
    } else {
        branch_view
    };
    let first_chain = chain.concat(&rest.0.branch.section)?;
    let first_tail = &rest.0.branch.tail;
    continuations.push((view, first_chain, first_tail));

    for branch in &rest.1 {
        let branch_view = branch.doc_block.create_view_starting_at(depth);
        let next_chain = chain.concat(&branch.branch.section)?;
        let next_tail = &branch.branch.tail;
        continuations.push((branch_view, next_chain, next_tail));
    }

    produce_inherent_impl_header_for_output(chain, tokens);
    Brace::default().surround(tokens, |tokens| {
        for (view, next_chain, next_tail) in &continuations {
            produce_method(prelude, view, next_chain, next_tail, tokens);
        }
    });

    for (view, next_chain, next_tail) in continuations {
        produce_module(tokens, prelude, &view, &next_chain, next_tail)?
    }
    Ok(())
}

// ⟨module_header⟩ {
//     ⟨module_body⟩
// }
pub fn produce_module(
    tokens: &mut TokenStream,
    prelude: &Prelude,
    view: &DocumentationBlockView,
    chain: &BrowsingChain,
    tail: &BranchTail,
) -> FlatteningResult {
    if let BranchTail::Alternative { rest, .. } = tail {
        produce_linked_doc_for_module(chain, tail, tokens);
        produce_module_header(prelude, chain, tokens);
        let mut result: FlatteningResult = Ok(());
        Brace::default().surround(tokens, |tokens| {
            result = produce_module_body(prelude, view, rest, chain, tokens);
        });
        result
    } else {
        Ok(())
    }
}
