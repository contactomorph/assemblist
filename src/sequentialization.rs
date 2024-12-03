use crate::types::{AssemblistDefinition, AssemblistSignature, AssemblistTree};
use proc_macro2::TokenStream;
use quote::quote_spanned;

fn sequentialize_leaf(
    level: usize,
    signature: AssemblistSignature,
    definition: AssemblistDefinition,
) -> TokenStream {
    let span = signature.span();
    let result_data = definition.result_data;
    let body = definition.body.stream();
    if level == 0 {
        let signature = signature.as_declaration();
        quote_spanned! {
            span => pub fn #signature #result_data { #body }
        }
    } else {
        let variable_decl = signature.as_variable_declaration();
        let signature = signature.as_declaration_with_self();
        quote_spanned! {
            span => impl ResultType { pub fn #signature #result_data { #variable_decl #body } }
        }
    }
}

fn sequentialize_branch(
    level: usize,
    signature: AssemblistSignature,
    values: Vec<TokenStream>,
) -> TokenStream {
    let span = signature.span();
    let name = signature.name();
    let type_content = signature.as_type_content();
    let field_assignments = signature.as_field_assignments();

    if level == 0 {
        let signature = signature.as_declaration();
        quote_spanned! {
            span =>
                pub mod #name {
                    pub struct ResultType {
                        #type_content
                    }
                    #(#values)*
                }
                pub fn #signature -> #name::ResultType {
                    #name::ResultType { #field_assignments }
                }
        }
    } else {
        let signature = signature.as_declaration_with_self();
        quote_spanned! {
            span =>
                pub mod #name {
                    pub struct ResultType {
                        #type_content
                    }
                    #(#values)*
                }
                impl ResultType {
                    pub fn #signature -> #name::ResultType {
                        #name::ResultType { #field_assignments }
                    }
                }
        }
    }
}

pub fn sequentialize_trees(trees: Vec<AssemblistTree>) -> TokenStream {
    let mut output = TokenStream::new();
    for tree in trees {
        let stream = tree.visit(
            &mut |level, signature, definition| sequentialize_leaf(level, signature, definition),
            &mut |level, signature, values| sequentialize_branch(level, signature, values),
        );
        output.extend(stream);
    }
    output
}