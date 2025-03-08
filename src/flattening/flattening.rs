use crate::model::tree::Tree;
use syn::parse_macro_input;

use super::trunk_browser::TrunkBrowser;

pub fn flatten(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let tree = parse_macro_input!(input as Tree);

    let stream = proc_macro::TokenStream::new();
    for trunk in &tree.roots {
        let browser = TrunkBrowser::new(trunk);
        let res = browser.browse(|_trunk, _chain, _tail| {});
        if let Err(error) = res {
            return error.into();
        }
    }
    stream
}
