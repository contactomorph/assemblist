use crate::flattening::trunk::{flatten_trunk, FlatteningResult};
use crate::model::tree::{BranchTail, Trunk};
use proc_macro2::TokenStream;
use quote::{quote_spanned, ToTokens};
use syn::parse_macro_input;

use super::chain::BrowsingChain;

fn flatten_function(
    stream: &mut TokenStream,
    trunk: &Trunk,
    chain: &BrowsingChain,
    tail: &BranchTail,
    _f: fn(&mut TokenStream, &Trunk, &BrowsingChain, &BranchTail) -> FlatteningResult,
) -> FlatteningResult {
    match tail {
        BranchTail::Alternative { rest, .. } => {}
        BranchTail::Leaf {
            output,
            brace,
            body,
        } => {}
    }
    Ok(())
}

pub fn flatten(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // let tree = parse_macro_input!(input as Tree);

    // let mut stream = proc_macro2::TokenStream::new();
    // for trunk in &tree.roots {
    //     let res = flatten_trunk(&mut stream, trunk, |_stream, _trunk, _chain, _tail| {
    //         // flatten_xxx()
    //         Ok(())
    //     });
    //     if let Err(error) = res {
    //         return error.into();
    //     }
    // }
    // stream.into()
    todo!()
}
