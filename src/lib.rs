//! Macros to easily create immutable builders.
//!
//! The [builder pattern](https://en.wikipedia.org/wiki/Builder_pattern) is encouraged
//! in the Rust language, in particular as a strategy to emulate named and optional arguments,
//! which are intentionally not supported by Rust.
//! However creating all the builder machinery can lead to boilerplate code, in particular when the
//! generated data is complex and multi-layered.
//! The usual builder pattern is based on mutation and generally turns compile-time checks that
//! the final object is complete to a runtime verification. Assemblist allows you to create immutable builders
//! structured as method chains like in
//! ```ignore
//! fn define_movie(name: String)
//!     .released_in(release_year: usize)
//!     .directed_by(director_name: String) -> Movie
//! {
//!     Movie { name, release_year, director_name }
//! }
//! ```
mod concepts;
mod model;
mod parsing;
mod sequentialization;
mod tools;
mod transformation;

use parsing::item_tree::parse;
use transformation::transform;

/**
 * A macro used to generate immutable builders for functions and methods.
 *
 * The argument of the assemblist! macro is a scope containing either method chains or implementations containing
 * method chains. A method chain looks like a function where the name and argument list are split into multiple parts.
 * Behind the scene assemblist actually creates as many disctinct methods and generates their result type automatically.
 * ```ignore
 * fn define_movie(name: String)
 *     .released_in(release_year: usize)
 *     .directed_by(director_name: String) -> Movie
 * {
 *   Movie { name, release_year, director_name }
 * }
 * ```
 * Here is a similar chain method declared inside an impl scope:
 * ```ignore
 * impl MovieMaker {
 *   fn define_movie(name: String)
 *       .released_in(release_year: usize)
 *       .directed_by(director_name: String) -> Movie
 *   {
 *     Movie { name, release_year, director_name }
 *   }
 * }
 * ```
 * Note that the latter pattern is only valid for inherent implementations.
 * You cannot add method chains to trait implementations.
 */
#[proc_macro]
pub fn assemblist(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match parse(input) {
        Ok(trees) => sequentialization::sequentialize_trees(trees).into(),
        Err(failure) => failure.to_stream().into(),
    }
}

#[doc(hidden)]
#[proc_macro]
pub fn assemblist_format(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match parse(input) {
        Ok(trees) => sequentialization::format_trees(trees).into(),
        Err(failure) => failure.to_stream().into(),
    }
}

#[proc_macro]
pub fn assemblist_trans(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    transform(input)
}
