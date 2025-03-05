use proc_macro2::TokenStream;

pub type FlatteningResult = std::result::Result<(), TokenStream>;

pub trait Flattener {
    type Context;

    fn to_flat_representation(&self, context: &mut Self::Context, tokens: &mut TokenStream) -> FlatteningResult;
}