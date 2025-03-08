use crate::{
    flattening::{flattener::FlatteningResult, usual_args::UsualArg},
    model::{
        section::Section,
        tree::{Branch, BranchTail, Tree},
    },
    tools::cumulative::{CumulativeFn, CumulativeList},
};
use syn::parse_macro_input;

pub fn flatten(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let tree = parse_macro_input!(input as Tree);

    let mut stream = proc_macro::TokenStream::new();
    for trunk in &tree.roots {
        let f = CumulativeFn::<proc_macro::TokenStream, &Branch>::make(
            |handler, stream, branch, agg| {
                let _list: &CumulativeList<'_, Vec<UsualArg>> = agg;
                let args = UsualArg::extract_usual_args(&branch.section.inputs)?;
                if let BranchTail::Alternative { rest, .. } = &branch.tail {
                    let sub_branch: &Branch = &rest.0;
                    handler.call(stream, sub_branch, &args)?;
                }
                let result: FlatteningResult = Ok(());
                result
            },
        );
        if let Err(error) = f.call(&mut stream, &trunk.branch) {
            return error.into();
        }
    }
    stream
}
