use assemblist::assemblist;
use std::fmt::Debug;

#[derive(Debug, PartialEq, Eq)]
pub struct Movie {
    name: String,
    release_year: usize,
    director_name: String,
}

#[test]
fn convert_method_chain_for_movies() {
    assemblist! {
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

        fn f(i: usize) -> usize {
            i + 5
        }
    }

    let movie = define_movie("The Lobster")
        .released_in(2015)
        .directed_by("Yorgos Lanthimos");
    
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

// #[test]
// fn convert_method_alternative() {
//     assemblist! {
//         #[doc(alias = "docs")]
//         pub(crate) fn define_movie(name: String)
//             .released_in(release_year: usize)
//             .directed_by(director_name: String) -> Movie
//         {
//             Movie { name, release_year, director_name }
//         }
//     };
//     let movie1 = define_movie("The Lobster".to_string())
//         .released_in(2015)
//         .directed_by("Yorgos Lanthimos".to_string());
//     assert_eq!(
//         movie1,
//         Movie {
//             name: "The Lobster".to_string(),
//             release_year: 2015,
//             director_name: "Yorgos Lanthimos".to_string()
//         }
//     );
// }

// mod nested_module {
//     use super::Movie;
//     use assemblist::assemblist;

//     pub struct MovieMaker;

//     assemblist! {
//         impl MovieMaker {
//             pub fn define_movie(name: String)
//                 .released_in(release_year: usize)
//                 .directed_by(director_name: String) -> Movie
//             {
//                 Movie { name, release_year, director_name }
//             }
//         }
//     }
// }

// #[test]
// fn convert_impl() {
//     let movie = nested_module::MovieMaker::define_movie("The Lobster".to_string())
//         .released_in(2015)
//         .directed_by("Yorgos Lanthimos".to_string());
//     assert_eq!(
//         movie,
//         Movie {
//             name: "The Lobster".to_string(),
//             release_year: 2015,
//             director_name: "Yorgos Lanthimos".to_string()
//         }
//     );
// }

