use assemblist::assemblist;
use http::Uri;
use json::JsonValue;
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
    };

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

pub struct MultiPtr<'a, 'b, T, U> {
    ptr1: &'a T,
    ptr2: &'b U,
}

#[test]
fn convert_method_chain_for_multi_ptr() {
    assemblist! {
        pub fn mutli_ptr_with<'a, T>(ptr1: &'a T).and<'b, U>(ptr2: &'b U) -> MultiPtr<'a, 'b, T, U>
        {
            MultiPtr { ptr1, ptr2 }
        }
    };

    let a = vec![4, 54, 6];
    let b = "Hello".to_string();

    let multi = mutli_ptr_with(&a).and(&b);
    assert_eq!(&a as *const _, multi.ptr1 as *const _);
    assert_eq!(&b as *const _, multi.ptr2 as *const _);
}

#[test]
fn convert_method_chain_for_multi_ptr_in_3_steps() {
    assemblist! {
        pub fn mutli_ptr_with<'a, T>(ptr1: &'a T)
            .and<'b, U>(ptr2: &'b U)
            .closed() -> MultiPtr<'a, 'b, T, U>
        {
            MultiPtr { ptr1, ptr2 }
        }
    };

    let a = vec![4, 54, 6];
    let b = "Hello".to_string();

    let multi = mutli_ptr_with(&a).and(&b).closed();
    assert_eq!(&a as *const _, multi.ptr1 as *const _);
    assert_eq!(&b as *const _, multi.ptr2 as *const _);
}

#[derive(PartialEq, Eq, Debug)]
pub enum HttpBody {
    Text(String),
    Json(JsonValue),
}

#[derive(PartialEq, Eq, Debug)]
pub enum HttpAuthorization {
    None,
    Basic(String),
    Bearer(String),
}

pub struct PostHttpRequest {
    url: Uri,
    user_agent: String,
    authorization: HttpAuthorization,
    body: HttpBody,
}

pub struct GetHttpRequest {
    url: Uri,
    user_agent: String,
    authorization: HttpAuthorization,
}

#[test]
fn convert_method_chain_for_http_requests() {
    assemblist! {
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
    };

    let get_request = new_http_request_to(Uri::from_static("http://www.croco-paradise.tv"))
        .from("FireFox")
        .with_authorization(HttpAuthorization::None)
        .as_get();

    assert_eq!(
        get_request.url.to_string(),
        "http://www.croco-paradise.tv/".to_string()
    );
    assert_eq!(get_request.user_agent, "FireFox".to_string());
    assert_eq!(get_request.authorization, HttpAuthorization::None);

    let post_request = new_http_request_to(Uri::from_static("http://www.croco-paradise.tv"))
        .from("FireFox")
        .with_authorization(HttpAuthorization::Bearer("AEKZEFOEZ".to_string()))
        .as_post()
        .with_text("Hello world".to_string());

    assert_eq!(
        post_request.url.to_string(),
        "http://www.croco-paradise.tv/".to_string()
    );
    assert_eq!(post_request.user_agent, "FireFox".to_string());
    assert_eq!(
        post_request.authorization,
        HttpAuthorization::Bearer("AEKZEFOEZ".to_string())
    );
    assert_eq!(post_request.body, HttpBody::Text("Hello world".to_string()));
}
