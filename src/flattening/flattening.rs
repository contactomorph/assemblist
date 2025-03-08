use crate::{flattening::trunk_browsing::browse_trunk, model::tree::Tree};
use syn::parse_macro_input;

pub fn flatten(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let tree = parse_macro_input!(input as Tree);

    let stream = proc_macro::TokenStream::new();
    for trunk in &tree.roots {
        let res = browse_trunk(trunk, |_trunk, _chain, _tail| {
            Ok(())
        });
        if let Err(error) = res {
            return error.into();
        }
    }
    stream
}
