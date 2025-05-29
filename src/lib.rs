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

/**
 * The point of this crate. Allows to generate fluent immutable builders based on method chains.
 *
 * The argument of the `assemblist!` macro is a scope containing one or more method chains.
 * A method chain is similar to a function except that its name and argument list are split into multiple sections.
 * Behind the scene, `assemblist!` automatically generates distinct normal methods based on your declared method chains,
 * along with their respective return types.
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
 * # Use builders with any function body
 *
 * Contrary to many alternative crates, in Assemblist, builder patterns are not generated from
 * annotations on a struct to build, but by directly declaring method chains as if they were basic
 * constructions of the Rust language. In addition to making their use obvious, this pattern
 * allows for much more flexibility in the implementation. In fact, you do not even need to build
 * something. For example you may just want to decompose an existing function in order to clarify
 * the purpose of its parameters (enclosing macro is omitted):
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
 * # Declare alternatives
 *
 * The builder pattern is a very expressive method to offer alternatives to users of a library.
 * They can start with a common function name and then choose which subsequent method makes sense for
 * their specific case. Assemblist enhances flexibility by letting them define alternative paths
 * within method chains. Using `.{ … }` blocks, you can specify multiple continuations, each returning different types
 * based on the user's choice.
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
 * Inside each alternative block, continuations start with the `fn` keyword and can themselves be method chains,
 * possibly including other alternative block recursively.
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
 * # Use method chains in inherent implementations
 *
 * You can either declare method chains as root items, as shown in previous examples, or declare them
 * inside inherent implementations:
 * ```rust
 * struct Calculation;
 *
 * assemblist! {
 *     impl Calculation {
 *         pub fn add(a: isize).to(b: isize) -> isize { a + b }
 *         pub fn remove(a: isize).from(b: isize) -> isize { a - b }
 *     }
 * }
 * ```
 *
 * Assemblist also supports the use of the self receiver in three standard forms: `self`, `&'a self`, and `&'a mut self`.
 * However, there are a couple of important rules to follow:
 *  - The self parameter must appear as the first argument in the first method of the chain.
 *  - Inside method bodies, the receiver is referred to using `self_` instead of the conventional `self`.
 *
 * Here’s an example showcasing this convention:
 * ```rust
 * struct MyVec<T> { _inner: Vec<T> }
 *
 * assemblist! {
 *    impl<T> MyVec<T> where T: Debug {
 *         pub fn at<'a>(&'a mut self, index: usize).{
 *             fn insert(element: T) {
 *                 self_._inner.insert(index, element)
 *             }
 *             fn remove() -> T {
 *                 self_._inner.remove(index)
 *             }
 *         }
 *     }
 * }
 * ```
 * On the user side, method chains behave just as expected:
 * ```rust
 * let v = MyVec<usize> { _inner: vec![4, 6, 3, 8, 2002] };
 * v.at(1).insert(42);
 * v.at(3).remove();
 * ```
 *
 * Note that it is possible to declare multiple inherent implementations inside the same `assemblist!`
 * macro invocation and to mix them with root method chains.
 *
 * # How to document your chains
 *
 * Assemblist makes method-chain documentation simple. Just separate descriptions with `---`, ensuring each
 * corresponds to its respective method in order.
 * ```rust
 * /// Start creating a movie by providing its title.
 * ///---
 * /// Provide the movies release year.
 * ///---
 * /// Provide the director's name and return the complete movie.
 * fn define_movie<'a>(name: &'a str)
 *     .released_in(release_year: usize)
 *     .directed_by(director_name: &'a str) -> Movie { /* code */ }
 * ```
 *
 * In case of alternative blocks, you can insert documentation just in front of
 * each possible `fn` continuation. The mechanism described earlier applies
 * recursively so you can include consecutive descriptions separated by sequences `---`
 * whenever your continuation contains multiple methods.
 *
 * ```rust
 * /// Start creating an http request by providing an uri.
 * ///---
 * /// Provide a user agent.
 * ///---
 * /// Provide the authorization.
 * fn new_http_request_to(url: Uri)
 *     .from<'a>(user_agent: &'a str)
 *     .with_authorization(authorization: HttpAuthorization).{
 *
 *     /// Specify the request is a PATCH.
 *     ///---
 *     /// Provide a json body and return the PATCH request.
 *     fn as_patch().with_json_body(json: JsonValue) -> PatchHttpRequest { /* code */ }
 *
 *     /// Specify the request is a POST.
 *     fn as_post().{
 *
 *         /// Provide a text body and return the POST request.
 *         fn with_text_body(body: String) -> PostHttpRequest { /* code */ }
 *
 *         /// Provide a json body and return the POST request.
 *         fn with_json_body(json: JsonValue) -> PostHttpRequest { /* code */ }
 *     }
 * }
 * ```
 *
 * # Current limitations
 *
 * ## No implicit lifetimes
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
 * ## No patterns for arguments
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
 * ## No complex receiver arguments
 *
 * For now, method chains inside inherent implementations cannot declare a complex receiver argument. Only `self`,
 * `&'a self` and `&'a mut self` are allowed. So this example fails to compile:
 * ```compile_fail
 * struct MyInt(isize);
 *
 * assemblist! {
 *     impl MyInt {
 *         fn is_between(self: Box<Self>, a: isize).and(b: isize) { a <= self.0 && self.0 <= b }
 *     }
 * }
 * ```
 *
 * ## Avoid method name clash
 *
 * The `assemblist!` macro generates a tree of inner modules, each containing custom types and
 * implementations. These modules are named based on the sections of your method chains.
 *
 * Because of this structure, multiple method chains within the same root module **cannot
 * start with the same method name** --- whether they belong to inherent implementations or not.
 * This restriction prevents naming conflicts in the generated modules.
 *
 * ```compile_fail
 * assemblist! {
 *     impl MyInt {
 *         fn f(/*args*/).y(/*args*/) { /* code */ }
 *     }
 *
 *     /* conflict: other method chain already starts with `f` */
 *     fn f(/*args*/).x(/*args*/) { /* code */ }
 *
 *     impl MyVec<T> {
 *         /* conflict: other method chain already starts with `f` */
 *         fn f(/*args*/).z(/*args*/) { /* code */ }
 *     }
 * }
 * ```
 * If you need multiple method chains with the same starting name, declare them inside
 * separate modules to avoid conflicts.
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
