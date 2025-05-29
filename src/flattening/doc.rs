use proc_macro2::TokenStream;

use crate::model::branch::{BranchTail, DocumentedBranch};

use super::chain::{BrowsingChain, RootImplHeader};
use quote::{quote, ToTokens};

pub fn produce_linked_doc_for_module<'a>(
    chain: &'a BrowsingChain<'a>,
    tail: &BranchTail,
    tokens: &mut TokenStream,
) {
    let mut fn_names = Vec::<String>::new();
    let root_header = collect_fn_names_and_root_header(chain, &mut fn_names);

    let mut intro = "Intermediary module for partial method chain ".to_string();
    let localization = fn_names.len() - 1;
    produce_doc_for_sequence(root_header, fn_names.as_slice(), localization, &mut intro);

    quote! {
        #[doc = #intro]
        #[doc = ""]
        #[doc = "Following method chains are supported:"]
    }
    .to_tokens(tokens);

    if let BranchTail::Alternative { rest, .. } = tail {
        produce_doc_for_all_sequences(&rest.0, root_header, &mut fn_names, localization, tokens);
        for branch in &rest.1 {
            produce_doc_for_all_sequences(branch, root_header, &mut fn_names, localization, tokens);
        }
    }
}

pub fn produce_linked_doc_for_output<'a>(chain: &'a BrowsingChain<'a>, tokens: &mut TokenStream) {
    let mut fn_names = Vec::<String>::new();
    let root_header = collect_fn_names_and_root_header(chain, &mut fn_names);

    let mut comment = "Intermediary type returned by partial method chain ".to_string();
    produce_doc_for_sequence(
        root_header,
        fn_names.as_slice(),
        fn_names.len(),
        &mut comment,
    );

    quote! { #[doc = #comment] }.to_tokens(tokens);
}

fn collect_fn_names_and_root_header<'a>(
    chain: &'a BrowsingChain<'a>,
    fn_names: &mut Vec<String>,
) -> Option<RootImplHeader<'a>> {
    let root_header;
    if let Some(previous) = chain.previous() {
        root_header = collect_fn_names_and_root_header(previous, fn_names);
    } else if let Some(header) = chain.root_header() {
        root_header = Some(header);
    } else {
        root_header = None;
    }
    fn_names.push(chain.section().ident.to_string());
    root_header
}

fn produce_doc_for_all_sequences(
    branch: &DocumentedBranch,
    root_header: Option<RootImplHeader>,
    fn_names: &mut Vec<String>,
    localization: usize,
    tokens: &mut TokenStream,
) {
    let fn_name = branch.branch.section.ident.to_string();
    fn_names.push(fn_name);

    match &branch.branch.tail {
        BranchTail::Alternative { rest, .. } => {
            produce_doc_for_all_sequences(&rest.0, root_header, fn_names, localization, tokens);
            for branch in &rest.1 {
                produce_doc_for_all_sequences(branch, root_header, fn_names, localization, tokens);
            }
        }
        BranchTail::Leaf { .. } => {
            let mut item = "- ".to_string();
            produce_doc_for_sequence(root_header, fn_names.as_slice(), localization, &mut item);
            quote! { #[doc = #item] }.to_tokens(tokens);
        }
    }

    fn_names.pop();
}

fn produce_doc_for_sequence(
    root_header: Option<RootImplHeader>,
    fn_names: &[String],
    localisation: usize,
    doc: &mut String,
) {
    let mut root_type_name: Option<String> = None;
    if let Some(RootImplHeader {
        root_type: syn::Type::Path(p),
        ..
    }) = root_header
    {
        if let Some(segment) = p.path.segments.last() {
            root_type_name = Some(segment.ident.to_string());
        }
    }
    for (n, fn_name) in fn_names.iter().enumerate() {
        let is_function = n == 0 && root_header.is_none();

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
