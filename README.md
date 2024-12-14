# Easily create immutable builders in Rust.

The [builder pattern](https://en.wikipedia.org/wiki/Builder_pattern) is encouraged in the Rust language, in particular as a strategy to emulate named and optional arguments, which are intentionally not supported by Rust. However creating all the builder machinery can lead to boilerplate code, in particular when the generated data is complex and multi-layered. The usual builder pattern is based on mutation and generally turns compile-time checks that the final object is complete to a runtime verification. Assemblist allows you to create immutable builders structured as method chains like in
```rust
fn define_movie(name: String)
    .released_in(release_year: usize)
    .directed_by(director_name: String) -> Movie
{
    Movie { name, release_year, director_name }
}
```