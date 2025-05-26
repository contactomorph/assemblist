use assemblist::{assemblist, assemblist_text};
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

#[test]
pub fn document_implementations() {
    let text = assemblist_text! {
        /// Provide url
        ///---
        /// Provide user agent
        ///---
        /// Provide authorization
        ///---
        /// Should not appear
        fn new_http_request_to(url: Uri)
            .from<'a>(user_agent: &'a str)
            .with_authorization(authorization: HttpAuthorization).{

            /// Create get request
            ///---
            /// Should not appear
            fn as_get() -> GetHttpRequest {
                GetHttpRequest {
                    url,
                    user_agent: user_agent.to_string(),
                    authorization,
                }
            }

            /// Create post request
            ///---
            /// Should not appear
            fn as_post().{
                /// provide string body
                fn with_text(body: String) -> PostHttpRequest {
                    PostHttpRequest {
                        url,
                        user_agent: user_agent.to_string(),
                        authorization,
                        body: HttpBody::Text(body),
                    }
                }

                /// provide json body
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

    asserts::equivalent!(
        text,
        "#[doc = \" Provide url\"]
        fn new_http_request_to(url : Uri) -> new_http_request_to :: Output {
            new_http_request_to :: Output { url, }
        }
        #[doc = \"Intermediary module for partial method chain [`new_http_request_to`](fn@new_http_request_to)`(…).…`\"]
        #[doc = \"\"] #[doc = \"Following method chains are supported:\"]
        #[doc = \"- [`new_http_request_to`](fn@new_http_request_to)`(…).`[`from`](method@new_http_request_to::Output::from)`(…).`[`with_authorization`](method@new_http_request_to::from::Output::with_authorization)`(…).`[`as_get`](method@new_http_request_to::from::with_authorization::Output::as_get)`(…)`\"]
        #[doc = \"- [`new_http_request_to`](fn@new_http_request_to)`(…).`[`from`](method@new_http_request_to::Output::from)`(…).`[`with_authorization`](method@new_http_request_to::from::Output::with_authorization)`(…).`[`as_post`](method@new_http_request_to::from::with_authorization::Output::as_post)`(…).`[`with_text`](method@new_http_request_to::from::with_authorization::as_post::Output::with_text)`(…)`\"]
        #[doc = \"- [`new_http_request_to`](fn@new_http_request_to)`(…).`[`from`](method@new_http_request_to::Output::from)`(…).`[`with_authorization`](method@new_http_request_to::from::Output::with_authorization)`(…).`[`as_post`](method@new_http_request_to::from::with_authorization::Output::as_post)`(…).`[`with_json`](method@new_http_request_to::from::with_authorization::as_post::Output::with_json)`(…)`\"]
        mod new_http_request_to {
            # ! [allow(unused_imports)]
            use super :: * ;
            #[doc = \"Intermediary type returned by partial method chain [`new_http_request_to`](fn@super::new_http_request_to)`(…).…`\"]
            pub struct Output { pub(super) url : Uri, }
            impl Output {
                #[doc = \" Provide user agent\"]
                pub fn from < 'a > (self, user_agent : & 'a str) -> from :: Output :: < 'a > {
                    let url = self.url;
                    from :: Output :: < 'a > { user_agent, url, }
                }
            }
            #[doc = \"Intermediary module for partial method chain [`new_http_request_to`](fn@super::new_http_request_to)`(…).`[`from`](method@Output::from)`(…).…`\"]
            #[doc = \"\"]
            #[doc = \"Following method chains are supported:\"]
            #[doc = \"- [`new_http_request_to`](fn@super::new_http_request_to)`(…).`[`from`](method@Output::from)`(…).`[`with_authorization`](method@from::Output::with_authorization)`(…).`[`as_get`](method@from::with_authorization::Output::as_get)`(…)`\"]
            #[doc = \"- [`new_http_request_to`](fn@super::new_http_request_to)`(…).`[`from`](method@Output::from)`(…).`[`with_authorization`](method@from::Output::with_authorization)`(…).`[`as_post`](method@from::with_authorization::Output::as_post)`(…).`[`with_text`](method@from::with_authorization::as_post::Output::with_text)`(…)`\"]
            #[doc = \"- [`new_http_request_to`](fn@super::new_http_request_to)`(…).`[`from`](method@Output::from)`(…).`[`with_authorization`](method@from::Output::with_authorization)`(…).`[`as_post`](method@from::with_authorization::Output::as_post)`(…).`[`with_json`](method@from::with_authorization::as_post::Output::with_json)`(…)`\"]
            pub mod from {
                # ! [allow(unused_imports)]
                use super :: * ;
                #[doc = \"Intermediary type returned by partial method chain [`new_http_request_to`](fn@super::super::new_http_request_to)`(…).`[`from`](method@super::Output::from)`(…).…`\"]
                pub struct Output < 'a > {
                    pub(super) user_agent : & 'a str,
                    pub(super) url : Uri,
                }
                impl < 'a > Output < 'a > {
                    #[doc = \" Provide authorization\"]
                    pub fn with_authorization(self, authorization : HttpAuthorization) -> with_authorization :: Output :: < 'a > {
                        let user_agent = self.user_agent; let url = self.url;
                        with_authorization :: Output :: < 'a > { authorization, user_agent, url, }
                    }
                }
                #[doc = \"Intermediary module for partial method chain [`new_http_request_to`](fn@super::super::new_http_request_to)`(…).`[`from`](method@super::Output::from)`(…).`[`with_authorization`](method@Output::with_authorization)`(…).…`\"]
                #[doc = \"\"]
                #[doc = \"Following method chains are supported:\"]
                #[doc = \"- [`new_http_request_to`](fn@super::super::new_http_request_to)`(…).`[`from`](method@super::Output::from)`(…).`[`with_authorization`](method@Output::with_authorization)`(…).`[`as_get`](method@with_authorization::Output::as_get)`(…)`\"]
                #[doc = \"- [`new_http_request_to`](fn@super::super::new_http_request_to)`(…).`[`from`](method@super::Output::from)`(…).`[`with_authorization`](method@Output::with_authorization)`(…).`[`as_post`](method@with_authorization::Output::as_post)`(…).`[`with_text`](method@with_authorization::as_post::Output::with_text)`(…)`\"]
                #[doc = \"- [`new_http_request_to`](fn@super::super::new_http_request_to)`(…).`[`from`](method@super::Output::from)`(…).`[`with_authorization`](method@Output::with_authorization)`(…).`[`as_post`](method@with_authorization::Output::as_post)`(…).`[`with_json`](method@with_authorization::as_post::Output::with_json)`(…)`\"]
                pub mod with_authorization {
                    # ! [allow(unused_imports)] use super :: * ;
                    #[doc = \"Intermediary type returned by partial method chain [`new_http_request_to`](fn@super::super::super::new_http_request_to)`(…).`[`from`](method@super::super::Output::from)`(…).`[`with_authorization`](method@super::Output::with_authorization)`(…).…`\"]
                    pub struct Output < 'a > {
                        pub(super) authorization : HttpAuthorization,
                        pub(super) user_agent : & 'a str,
                        pub(super) url : Uri,
                    }
                    impl < 'a > Output < 'a > {
                        #[doc = \" Create get request\"]
                        pub fn as_get(self,) -> GetHttpRequest {
                            let authorization = self.authorization;
                            let user_agent = self.user_agent;
                            let url = self.url;
                            GetHttpRequest { url, user_agent: user_agent.to_string(), authorization, }
                        }
                        #[doc = \" Create post request\"]
                        pub fn as_post(self,) -> as_post :: Output :: < 'a > {
                            let authorization = self.authorization;
                            let user_agent = self.user_agent;
                            let url = self.url;
                            as_post :: Output :: < 'a > { authorization, user_agent, url, }
                        }
                    }
                    #[doc = \"Intermediary module for partial method chain [`new_http_request_to`](fn@super::super::super::new_http_request_to)`(…).`[`from`](method@super::super::Output::from)`(…).`[`with_authorization`](method@super::Output::with_authorization)`(…).`[`as_post`](method@Output::as_post)`(…).…`\"]
                    #[doc = \"\"]
                    #[doc = \"Following method chains are supported:\"]
                    #[doc = \"- [`new_http_request_to`](fn@super::super::super::new_http_request_to)`(…).`[`from`](method@super::super::Output::from)`(…).`[`with_authorization`](method@super::Output::with_authorization)`(…).`[`as_post`](method@Output::as_post)`(…).`[`with_text`](method@as_post::Output::with_text)`(…)`\"]
                    #[doc = \"- [`new_http_request_to`](fn@super::super::super::new_http_request_to)`(…).`[`from`](method@super::super::Output::from)`(…).`[`with_authorization`](method@super::Output::with_authorization)`(…).`[`as_post`](method@Output::as_post)`(…).`[`with_json`](method@as_post::Output::with_json)`(…)`\"]
                    pub mod as_post {
                        # ! [allow(unused_imports)]
                        use super :: * ;
                        #[doc = \"Intermediary type returned by partial method chain [`new_http_request_to`](fn@super::super::super::super::new_http_request_to)`(…).`[`from`](method@super::super::super::Output::from)`(…).`[`with_authorization`](method@super::super::Output::with_authorization)`(…).`[`as_post`](method@super::Output::as_post)`(…).…`\"]
                        pub struct Output < 'a > {
                            pub(super) authorization : HttpAuthorization,
                            pub(super) user_agent : & 'a str,
                            pub(super) url : Uri,
                        }
                        impl < 'a > Output < 'a > {
                            #[doc = \" provide string body\"]
                            pub fn with_text(self, body : String) -> PostHttpRequest {
                                let authorization = self.authorization;
                                let user_agent = self.user_agent;
                                let url = self.url;
                                PostHttpRequest {
                                    url,
                                    user_agent: user_agent.to_string(),
                                    authorization,
                                    body: HttpBody::Text(body),
                                }
                            }
                            #[doc = \" provide json body\"]
                            pub fn with_json(self, json : JsonValue) -> PostHttpRequest {
                                let authorization = self.authorization;
                                let user_agent = self.user_agent;
                                let url = self.url;
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
            }
        }"
    );
}
