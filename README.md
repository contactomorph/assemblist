# Easily create immutable builders in Rust.

The [builder pattern](https://en.wikipedia.org/wiki/Builder_pattern) is encouraged in the Rust language, in particular as a strategy to emulate named and optional arguments, which are intentionally not supported by Rust. However creating all the builder machinery can lead to boilerplate code, in particular when the generated data is complex and multi-layered. The usual builder pattern is based on mutation and generally turns compile-time checks that the final object is complete to a runtime verification. Assemblist allows you to create immutable builders structured as method chains like in
```rust
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
```
You can then just call it as it was declared
```rust
let movie = define_movie("The Lobster")
    .released_in(2015)
    .directed_by("Yorgos Lanthimos");
```