//! A macro to easily create immutable builders.
//!
//! The [builder pattern](https://en.wikipedia.org/wiki/Builder_pattern) is encouraged
//! in the Rust language, in particular as a strategy to emulate named and optional arguments,
//! which are intentionally not supported by Rust.
//! However creating all the builder machinery can lead to boilerplate code, in particular when the
//! generated data is complex and multi-layered.
//! The usual builder pattern is based on mutation and generally turns compile-time checks that
//! the final object is complete to a runtime verification. Assemblist allows you to create fluent
//! immutable builders structured as method chains like in:
//! ```rust
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
//! You can then just call it as it was declared:
//! ```rust
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
 * The point of this crate. Allows to generate fluent immutable builders based on method chains.
 *
 * The argument of the `assemblist!` macro is a scope containing one or more method chains.
 * A method chain is similar to a function except that its name and argument list are split into multiple sections.
 * Behind the scene, `assemblist!` actually creates as many distinct methods and generates their respective result
 * type automatically.
 * ```rust
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
 *
 * You can then just call method chains as they are declared:
 * ```rust
 * let movie = define_movie("The Lobster")
 *     .released_in(2015)
 *     .directed_by("Yorgos Lanthimos");
 * ```
 *
 * Multiple method chains can be declared inside the same `assemblist!{ … }` block:
 * ```rust
 * assemblist! {
 *     fn f1(/*args*/).f2(/*args*/).f3(/*args*/) { /* code */ }
 *     fn g1(/*args*/).g2(/*args*/) { /* code */ }
 *     fn h1(/*args*/).h2(/*args*/).h3(/*args*/).h4(/*args*/) { /* code */ }
 * }
 * ```
 *
 * ## Use builders with any function body
 *
 * Contrary to many alternative crates, in Assemblist, builder patterns are not generated from
 * annotations on a struct to build, but by directly declaring method chains as if they were basic
 * constructions of the Rust language. In * addition to making their use obvious, this pattern
 * allows for much more flexibility in the implementation. In fact, you do not even need to build
 * something. For example you may just want to decompose an existing function in order to clarify
 * the purpose of its parameters:
 * ```rust
 * pub fn replace_in<'a>(text: &'a str)
 *     .occurrences_of(pattern: &'a str)
 *     .with(replacement: &'a str)
 *     .at_most(n: usize)
 *     .times() -> String
 * {
 *     text.replacen(pattern, replacement, n)
 * }
 * ```
 * You can actually include arbitrary complex code.
 *
 * ## Declare alternatives
 *
 * The builder pattern is a very expressive method to offer alternatives to users of a library.
 * They can start with a common function name and then choose which subsequent method makes sense for
 * their specific case. Assemblist aknowledges this possibility by offering the alternative syntax:
 * ```rust
 * fn new_http_request_to(url: Uri)
 *     .from<'a>(user_agent: &'a str)
 *     .with_authorization(authorization: HttpAuthorization).{
 *
 *     fn as_get() -> GetHttpRequest {
 *         GetHttpRequest {
 *             url,
 *             user_agent: user_agent.to_string(),
 *             authorization,
 *         }
 *     }
 *
 *     fn as_post().{
 *         fn with_text_body(body: String) -> PostHttpRequest {
 *             PostHttpRequest {
 *                 url,
 *                 user_agent: user_agent.to_string(),
 *                 authorization,
 *                 body: HttpBody::Text(body),
 *             }
 *         }
 *
 *         fn with_json_body(json: JsonValue) -> PostHttpRequest {
 *             PostHttpRequest {
 *                 url,
 *                 user_agent: user_agent.to_string(),
 *                 authorization,
 *                 body: HttpBody::Json(json),
 *             }
 *         }
 *     }
 * }
 * ```
 * Chaining with a `.{ … }` block gives you the possibility to define alternatives. Inside such a block,
 * each possible continuation starts with the `fn` keyword and can itself be a method chain, possibly
 * including other alternatives recursively. Each branch of the corresponding tree of method chains
 * can provide a distinct implementation and even return a distinct type:
 * ```rust
 * let get_request = new_http_request_to(Uri::from_static("http://www.croco-paradise.tv"))
 *     .from("FireFox")
 *     .with_authorization(HttpAuthorization::None)
 *     .as_get();
 *
 * let post_request = new_http_request_to(Uri::from_static("http://www.croco-paradise.tv"))
 *     .from("FireFox")
 *     .with_authorization(HttpAuthorization::Bearer("sometoken3456=".to_string()))
 *     .as_post()
 *     .with_text_body("Hello world".to_string());
 * ```
 *
 * ## Use method chains in inherent implementations
 *
 * You can either declare method chains as root items, as shown in previous examples, or declare them
 * inside inherent implementations:
 * ```rust
 * struct Calculation;
 *
 * assemblist! {
 *     impl Calculation {
 *         fn add(a: isize).to(b: isize) -> isize { a + b }
 *         fn remove(a: isize).from(b: isize) -> isize { a - b }
 *     }
 * }
 * ```
 * It is even possible to declare multiple inherent implementations inside the same `assemblist!`
 * macro invocation and to mix them with root method chains.
 *
 * ## Current limitations
 *
 * ### No implicit lifetimes
 *
 * Assemblist does not handle implicit lifetimes for now so you must declare them explicitely. Each method
 * inside the method chain can carry its own lifetimes:
 * ```rust
 * fn pour_items_from<'a, T>(source: &'a mut Vec<T>)
 *     .starting_at(n: usize)
 *     .into<'b>(destination: &'b mut Vec<T>)
 * {
 *     for item in source.drain(n..) {
 *         destination.push(item);
 *     }
 * }
 * ```
 *
 * ### No patterns for arguments
 *
 * You cannot use patterns for methods arguments as in `f((a, b): (usize, bool))` or in `g(ref r: f64)`. If
 * you need to do this, just use plain argument names and destructure them inside the body:
 * ```rust
 * fn f(pair: (usize, bool)).g(x: f64) {
 *     let (a, b) = pair;
 *     let ref r = x;
 * }
 * ```
 *
 * ### No `self` arguments
 *
 * For now, method chains inside inherent implementations cannot declare a `self` argument. Only "static"
 * methods are allowed. So this example fails to compile:
 * ```compile_fail
 * struct MyInt(isize);
 *
 * assemblist! {
 *     impl MyInt {
 *         fn is_between(&self, a: isize).and(b: isize) { a <= self.0 && self.0 <= b }
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
