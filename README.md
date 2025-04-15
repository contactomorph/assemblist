# Easily create fluent immutable builders in Rust.

The [builder pattern](https://en.wikipedia.org/wiki/Builder_pattern) is encouraged in the Rust language, in particular as a strategy to emulate named and optional arguments, which are intentionally not supported by Rust. However creating all the builder machinery can lead to boilerplate code, in particular when the generated data is complex and multi-layered. The usual builder pattern is based on mutation and generally turns compile-time checks that the final object is complete to a runtime verification. Assemblist allows you to create fluent immutable builders structured as method chains like in:
```rust
assemblist!{
    fn define_movie<'a>(name: &'a str)
        .released_in(release_year: usize)
        .directed_by(director_name: &'a str) -> Movie
    {
        Movie {
            name: name.to_string(),
            release_year,
            director_name: director_name.to_string(),
        }
    }
}
```
You just need to declare such items inside the `assemblist!{ … }` macro.

You can then just call method chains as they are declared:
```rust
let movie = define_movie("The Lobster")
    .released_in(2015)
    .directed_by("Yorgos Lanthimos");
```

## Use builders with any function body

Contrary to many alternative crates, in Assemblist, builder patterns are not generated from annotations on a struct to build, but by directly declaring method chains as if they were basic constructions of the Rust language. In addition to making their use obvious, this pattern allows for much more flexibility in the implementation. In fact, you do not even need to build something. For example you may just want to decompose an existing function in order to clarify the purpose of its parameters:
```rust
assemblist!{
    pub fn replace_in<'a>(text: &'a str)
        .occurrences_of(pattern: &'a str)
        .with(replacement: &'a str)
        .at_most(n: usize)
        .times() -> String
    {
        text.replacen(pattern, replacement, n)
    }
}
```
You can actually include arbitrary complex code.

## Alternatives

The builder pattern is a very expressive method to offer alternatives to users of a library. They can start with a common function name and then choose which subsequent method makes sense for their specific case. Assemblist aknowledges this possibility by offering the alternative syntax:
```rust
assemblist!{
    fn new_http_request_to(url: Uri)
        .from<'a>(user_agent: &'a str)
        .with_authorization(authorization: HttpAuthorization).{

        fn as_get() -> GetHttpRequest {
            GetHttpRequest {
                url,
                user_agent: user_agent.to_string(),
                authorization,
            }
        }

        fn as_post().{
            fn with_text(body: String) -> PostHttpRequest {
                PostHttpRequest {
                    url,
                    user_agent: user_agent.to_string(),
                    authorization,
                    body: HttpBody::Text(body),
                }
            }

            fn with_json(json: JsonValue) -> PostHttpRequest {
                PostHttpRequest {
                    url,
                    user_agent: user_agent.to_string(),
                    authorization,
                    body: HttpBody::Json(json),
                }
            }
        }
    }
}
```
Chaining with a `.{ … }` block gives you the possibility to define alternatives. Inside such a block, each possible continuation starts with the `fn` keyword and can itself be a method chain, possibly including other alternatives recursively. Each branch of the corresponding tree of method chains can provide a distinct implementation and even return a distinct type.

## Also works for inherent implementations

You can either declare method chains as root items, as shown in previous examples, or declare them inside inherent implementations:
```rust
struct Calculation;

assemblist! {
    impl Calculation {
        fn add(a: isize).to(b: isize) -> isize { a + b }
        fn remove(a: isize).from(b: isize) -> isize { a - b }
    }
}
```
