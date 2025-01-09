use assemblist::assemblist;
use std::fmt::Debug;

#[derive(Debug, PartialEq, Eq)]
pub struct Movie {
    name: String,
    release_year: usize,
    director_name: String,
}

#[test]
fn convert_method_chain() {
    assemblist! {
        fn define_movie(name: String)
            .released_in(release_year: usize)
            .directed_by(director_name: String) -> crate::Movie
        {
            crate::Movie { name, release_year, director_name }
        }

        fn f(i: usize) -> usize {
            i + 5
        }
    }

    let movie = define_movie("The Lobster".to_string())
        .released_in(2015)
        .directed_by("Yorgos Lanthimos".to_string());
    assert_eq!(
        movie,
        Movie {
            name: "The Lobster".to_string(),
            release_year: 2015,
            director_name: "Yorgos Lanthimos".to_string()
        }
    );

    assert_eq!(f(3), 8);
}

#[test]
fn convert_method_alternative() {
    assemblist! {
        #[doc(alias = "docs")]
        pub(crate) fn define_movie(name: String).released_in(release_year: usize).{
            fn directed_by(director_name: String) -> crate::Movie {
                crate::Movie { name, release_year, director_name }
            }
            fn directed_by_me() -> crate::Movie {
                crate::Movie { name, release_year, director_name: "me".to_string() }
            }
        }
    };
    let movie1 = define_movie("The Lobster".to_string())
        .released_in(2015)
        .directed_by("Yorgos Lanthimos".to_string());
    assert_eq!(
        movie1,
        Movie {
            name: "The Lobster".to_string(),
            release_year: 2015,
            director_name: "Yorgos Lanthimos".to_string()
        }
    );

    let movie2 = define_movie("Assemblist".to_string())
        .released_in(2024)
        .directed_by_me();
    assert_eq!(
        movie2,
        Movie {
            name: "Assemblist".to_string(),
            release_year: 2024,
            director_name: "me".to_string()
        }
    );
}

mod nested_module {
    use assemblist::assemblist;

    pub struct MovieMaker;

    assemblist! {
        impl MovieMaker {
            pub fn define_movie(name: String)
                .released_in(release_year: usize)
                .directed_by(director_name: String) -> crate::Movie
            {
                crate::Movie { name, release_year, director_name }
            }
        }
    }
}

#[test]
fn convert_impl() {
    let movie = nested_module::MovieMaker::define_movie("The Lobster".to_string())
        .released_in(2015)
        .directed_by("Yorgos Lanthimos".to_string());
    assert_eq!(
        movie,
        Movie {
            name: "The Lobster".to_string(),
            release_year: 2015,
            director_name: "Yorgos Lanthimos".to_string()
        }
    );
}
