use proc_macro2::{Ident, Punct, Spacing, Span, TokenStream, TokenTree};
use quote::{quote, quote_spanned};

use crate::concepts::{
    fn_tree::AssemblistFnDefinition,
    item_tree::{AssemblistImplTree, AssemblistItemTree},
    prelude::AssemblistPrelude,
    signature::AssemblistFnSignature,
};

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

fn sequentialize_hierachical_part_of_root_branch(
    prelude: &AssemblistPrelude,
    signature: &AssemblistFnSignature,
    values: Vec<TokenStream>,
) -> TokenStream {
    let span = signature.span();
    let name = signature.name();
    let type_content = signature.as_type_content();

    let visibility = prelude.get_visibility_declaration();
    quote_spanned! {
        span =>
            #visibility mod #name {
                #![allow(unused_imports)]
                use super::*;
                pub struct Output {
                    #type_content
                }
                #(#values)*
            }
    }
}

fn sequentialize_flat_part_of_root_branch(
    prelude: AssemblistPrelude,
    signature: AssemblistFnSignature,
) -> TokenStream {
    let span = signature.span();
    let name = signature.name();
    let field_assignments = signature.as_field_assignments();

    let prelude = prelude.as_complete_declaration();
    let signature = signature.as_declaration();
    quote_spanned! {
        span =>
            #prelude fn #signature -> #name::Output {
                #name::Output { #field_assignments }
            }
    }
}

fn repeat_super_keyword(n: usize, span: Span) -> TokenStream {
    let mut tokens = Vec::<TokenTree>::new();
    if 0 < n {
        for _ in 0..n {
            tokens.push(TokenTree::Ident(Ident::new("super", span)));
            tokens.push(TokenTree::Punct(Punct::new(':', Spacing::Joint)));
            tokens.push(TokenTree::Punct(Punct::new(':', Spacing::Alone)));
        }
    }
    let mut stream = TokenStream::new();
    stream.extend(tokens);
    stream
}

fn sequentialize_upper_branch(
    signature: AssemblistFnSignature,
    values: Vec<TokenStream>,
) -> TokenStream {
    let span = signature.span();
    let name = signature.name();
    let super_stream = repeat_super_keyword(signature.depth(), span);
    let type_content = signature.as_type_content();
    let field_assignments = signature.as_field_assignments();

    let signature = signature.as_declaration_with_self();
    quote_spanned! {
        span =>
            pub mod #name {
                #![allow(unused_imports)]
                use #super_stream*;
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

pub fn sequentialize_impl(impl_tree: AssemblistImplTree) -> TokenStream {
    let span = impl_tree.span();
    let AssemblistImplTree {
        prelude,
        name,
        sub_trees,
        ..
    } = impl_tree;
    let prelude = prelude.as_complete_declaration();
    let name_start = name.0;
    let name_end = TokenStream::from_iter(name.1);
    let mut impl_body_stream = TokenStream::new();
    let mut external_stream = TokenStream::new();
    for fn_tree in sub_trees {
        let stream = fn_tree.visit(
            &mut |depth, prelude, signature, definition| {
                sequentialize_leaf(depth, prelude, signature, definition)
            },
            &mut |depth, prelude, signature, values| {
                if depth == 0 {
                    external_stream.extend(sequentialize_hierachical_part_of_root_branch(
                        &prelude, &signature, values,
                    ));
                    sequentialize_flat_part_of_root_branch(prelude, signature)
                } else {
                    sequentialize_upper_branch(signature, values)
                }
            },
        );
        impl_body_stream.extend(stream);
    }
    quote_spanned! { span => #external_stream #prelude impl #name_start #name_end { #impl_body_stream } }
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
                    if depth == 0 {
                        let mut stream = sequentialize_hierachical_part_of_root_branch(
                            &prelude, &signature, values,
                        );
                        stream.extend(sequentialize_flat_part_of_root_branch(prelude, signature));
                        stream
                    } else {
                        sequentialize_upper_branch(signature, values)
                    }
                },
            ),
            AssemblistItemTree::Impl(impl_tree) => sequentialize_impl(impl_tree),
        };
        output.extend(stream);
    }
    output
}

pub fn format_trees(trees: Vec<AssemblistItemTree>) -> TokenStream {
    let tokens = trees.into_iter().map(|tree| {
        let str = match tree {
            AssemblistItemTree::Fn(fn_tree) => format!("{:?}", fn_tree),
            AssemblistItemTree::Impl(impl_tree) => format!("{:?}", impl_tree),
        };
        proc_macro2::TokenTree::Literal(proc_macro2::Literal::string(str.as_str()))
    });
    let output: proc_macro2::TokenStream = quote! { #(#tokens)* };
    output.into()
}
