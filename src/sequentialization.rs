use crate::fn_tree::AssemblistFnDefinition;
use crate::item_tree::AssemblistItemTree;
use crate::prelude::AssemblistPrelude;
use crate::signature::AssemblistFnSignature;
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};

fn sequentialize_leaf(
    depth: usize,
    prelude: AssemblistPrelude,
    signature: AssemblistFnSignature,
    definition: AssemblistFnDefinition,
) -> TokenStream {
    let span = signature.span();
    let result_data = TokenStream::from_iter(definition.result_data);
    let body = definition.body.stream();
    if depth == 0 {
        let prelude = prelude.as_complete_declaration();
        let signature = signature.as_declaration();
        quote_spanned! {
            span => #prelude fn #signature #result_data { #body }
        }
    } else {
        let variable_decl = signature.as_variable_declaration();
        let signature = signature.as_declaration_with_self();
        quote_spanned! {
            span => impl Output { pub fn #signature #result_data { #variable_decl #body } }
        }
    }
}

fn sequentialize_branch(
    depth: usize,
    prelude: AssemblistPrelude,
    signature: AssemblistFnSignature,
    values: Vec<TokenStream>,
) -> TokenStream {
    let span = signature.span();
    let name = signature.name();
    let type_content = signature.as_type_content();
    let field_assignments = signature.as_field_assignments();

    if depth == 0 {
        let visibility = prelude.get_visibility_declaration();
        let prelude = prelude.as_complete_declaration();
        let signature = signature.as_declaration();
        quote_spanned! {
            span =>
                #visibility mod #name {
                    pub struct Output {
                        #type_content
                    }
                    #(#values)*
                }
                #prelude fn #signature -> #name::Output {
                    #name::Output { #field_assignments }
                }
        }
    } else {
        let signature = signature.as_declaration_with_self();
        quote_spanned! {
            span =>
                pub mod #name {
                    pub struct Output {
                        #type_content
                    }
                    #(#values)*
                }
                impl Output {
                    pub fn #signature -> #name::Output {
                        #name::Output { #field_assignments }
                    }
                }
        }
    }
}

pub fn sequentialize_trees(trees: Vec<AssemblistItemTree>) -> TokenStream {
    let mut output = TokenStream::new();
    for tree in trees {
        let stream = match tree {
            AssemblistItemTree::Fn(fn_tree) => fn_tree.visit(
                &mut |depth, prelude, signature, definition| {
                    sequentialize_leaf(depth, prelude, signature, definition)
                },
                &mut |depth, prelude, signature, values| {
                    sequentialize_branch(depth, prelude, signature, values)
                },
            ),
            AssemblistItemTree::Impl(_impl_tree) => {
                todo!()
            }
        };
        output.extend(stream);
    }
    output
}

pub fn format_trees(trees: Vec<AssemblistItemTree>) -> TokenStream {
    let tokens = trees.into_iter().map(|tree| {
        let str = match tree {
            AssemblistItemTree::Fn(fn_tree) => format!("{:?}", fn_tree),
            AssemblistItemTree::Impl(_impl_tree) => {
                todo!()
            }
        };
        proc_macro2::TokenTree::Literal(proc_macro2::Literal::string(str.as_str()))
    });
    let output: proc_macro2::TokenStream = quote! { #(#tokens)* };
    output.into()
}
