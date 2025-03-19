//! A macro to easily create immutable builders.
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
//! fn define_movie<'a>(name: &'a str)
//!     .released_in(release_year: usize)
//!     .directed_by(director_name: &'a str) -> Movie
//! {
//!    Movie {
//!        name: name.to_string(),
//!        release_year,
//!        director_name: director_name.to_string(),
//!    }
//! }
//! ```
//! You can then just call it as it was declared
//! ```ignore
//! let movie = define_movie("The Lobster")
//!     .released_in(2015)
//!     .directed_by("Yorgos Lanthimos");
//! ```

use model::tree::Tree;
use proc_macro::{Literal, TokenStream, TokenTree};
use syn::parse_macro_input;
mod flattening;
mod model;
mod tools;

/**
 * The point of this crate. Generate immutable builders based on chains of methods.
 *
 * The argument of the `assemblist!` macro is a scope containing one or more method chains.
 * A method chain is similar to a function except that its name and argument list are split into multiple sections.
 * Behind the scene, `assemblist!` actually creates as many distinct methods and generates their respective result
 * type automatically.
 * ```ignore
 * assemblist! {
 *     fn define_movie<'a>(name: &'a str)
 *         .released_in(release_year: usize)
 *         .directed_by(director_name: &'a str) -> Movie
 *     {
 *         Movie {
 *             name: name.to_string(),
 *             release_year,
 *             director_name: director_name.to_string(),
 *         }
 *     }
 * }
 * ```
 * # For building objects ... among others
 *
 * Note that you can use `assemblist!` with valid rust code for the body content. It does not have to
 * be a constructor like in the previous example.
 * ```ignore
 * assemblist! {
 *     pub fn replace_in<'a>(string: &'a str)
 *         .occurrences_of(pattern: &'a str)
 *         .with(to: &'a str)
 *         .at_most(n: usize)
 *         .times() -> String
 *     {
 *         string.replacen(pattern, to, n)
 *     }
 * }
 * ```
 */
#[proc_macro]
pub fn assemblist(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let tree = parse_macro_input!(input as Tree);
    flattening::tree::flatten(tree).into()
}

#[doc(hidden)]
#[proc_macro]
pub fn assemblist_text(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let tree = parse_macro_input!(input as Tree);
    let text = flattening::tree::flatten(tree).to_string();
    let text = Literal::string(text.as_str());
    let value = TokenTree::Literal(text);
    TokenStream::from(value)
}
