mod parsing;
mod sequentialization;
mod types;

#[proc_macro]
pub fn assemblist(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match parsing::parse(input) {
        Ok(trees) => sequentialization::sequentialize_trees(trees).into(),
        Err(failure) => failure.to_stream().into(),
    }
}
