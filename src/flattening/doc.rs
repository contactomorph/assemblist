use proc_macro2::TokenStream;

use crate::model::branch::{Branch, BranchTail};

use super::chain::BrowsingChain;
use quote::{quote, ToTokens};

pub fn produce_linked_doc_for_module<'a>(
    chain: &'a BrowsingChain<'a>,
    tail: &BranchTail,
    tokens: &mut TokenStream,
) {
    let mut fn_names = Vec::<String>::new();
    let root_type = collect_fn_names_and_root_type(chain, &mut fn_names);

    let mut intro = "Intermediary module for partial method chain ".to_string();
    let localization = fn_names.len() - 1;
    produce_doc_for_sequence(root_type, fn_names.as_slice(), localization, &mut intro);

    quote! {
        #[doc = #intro]
        #[doc = ""]
        #[doc = "Following method chains are supported:"]
    }
    .to_tokens(tokens);

    if let BranchTail::Alternative { rest, .. } = tail {
        produce_doc_for_all_sequences(&rest.0, root_type, &mut fn_names, localization, tokens);
        for branch in &rest.1 {
            produce_doc_for_all_sequences(branch, root_type, &mut fn_names, localization, tokens);
        }
    }
}

pub fn produce_linked_doc_for_output<'a>(chain: &'a BrowsingChain<'a>, tokens: &mut TokenStream) {
    let mut fn_names = Vec::<String>::new();
    let root_type = collect_fn_names_and_root_type(chain, &mut fn_names);

    let mut comment = "Intermediary type returned by partial method chain ".to_string();
    produce_doc_for_sequence(root_type, fn_names.as_slice(), fn_names.len(), &mut comment);

    quote! { #[doc = #comment] }.to_tokens(tokens);
}

fn collect_fn_names_and_root_type<'a>(
    chain: &'a BrowsingChain<'a>,
    fn_names: &mut Vec<String>,
) -> Option<&'a syn::Type> {
    let root_type;
    if let Some(previous) = chain.previous() {
        root_type = collect_fn_names_and_root_type(previous, fn_names);
    } else if let Some(ty) = chain.root_type() {
        root_type = Some(ty);
    } else {
        root_type = None;
    }
    fn_names.push(chain.section().ident.to_string());
    root_type
}

fn produce_doc_for_all_sequences(
    branch: &Branch,
    root_type: Option<&syn::Type>,
    fn_names: &mut Vec<String>,
    localization: usize,
    tokens: &mut TokenStream,
) {
    let fn_name = branch.section.ident.to_string();
    fn_names.push(fn_name);

    match &branch.tail {
        BranchTail::Alternative { rest, .. } => {
            produce_doc_for_all_sequences(&rest.0, root_type, fn_names, localization, tokens);
            for branch in &rest.1 {
                produce_doc_for_all_sequences(branch, root_type, fn_names, localization, tokens);
            }
        }
        BranchTail::Leaf { .. } => {
            let mut item = "- ".to_string();
            produce_doc_for_sequence(root_type, fn_names.as_slice(), localization, &mut item);
            quote! { #[doc = #item] }.to_tokens(tokens);
        }
    }

    fn_names.pop();
}

fn produce_doc_for_sequence(
    root_type: Option<&syn::Type>,
    fn_names: &[String],
    localisation: usize,
    doc: &mut String,
) {
    let mut root_type_name: Option<String> = None;
    if let Some(syn::Type::Path(p)) = root_type {
        if let Some(ident) = p.path.get_ident() {
            root_type_name = Some(ident.to_string());
        }
    }
    for (n, fn_name) in fn_names.iter().enumerate() {
        let is_function = n == 0 && root_type.is_none();

        if 0 < n {
            doc.push_str(".`");
        } else if let Some(root_type_name) = &root_type_name {
            doc.push_str("[`");
            doc.push_str(root_type_name.as_str());
            doc.push_str("`]`::`");
        }

        doc.push_str("[`");
        doc.push_str(fn_name.as_str());
        doc.push_str("`](");
        doc.push_str(if is_function { "fn@" } else { "method@" });

        if n <= localisation {
            for _ in 0..(localisation - n) {
                doc.push_str("super::")
            }
        } else {
            for fn_name in &fn_names[localisation..n] {
                doc.push_str(fn_name.as_str());
                doc.push_str("::");
            }
        }

        if 0 < n {
            doc.push_str("Output::");
        } else if let Some(root_type_name) = &root_type_name {
            doc.push_str(root_type_name.as_str());
            doc.push_str("::")
        }

        doc.push_str(fn_name.as_str());
        doc.push_str(")`(…)");
    }

    if localisation + 1 < fn_names.len() {
        doc.push('`');
    } else {
        doc.push_str(".…`");
    }
}
