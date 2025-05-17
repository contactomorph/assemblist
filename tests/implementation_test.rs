use assemblist::{assemblist, assemblist_text};

struct Calculation;
struct StringHandling;

assemblist! {
    impl Calculation {
        fn add(a: isize).to(b: isize) -> isize { a + b }
        fn remove(a: isize).from(b: isize) -> isize { a - b }
    }
    impl StringHandling {
        fn concat<'a>(a: &'a str).{
            fn with(b: &'a str) -> String {
                let mut result = String::new();
                result.push_str(a);
                result.push_str(b);
                result
            }
            fn with_int(b: isize) -> String {
                let mut result = String::new();
                result.push_str(a);
                result.push_str(format!("{}", b).as_str());
                result
            }
        }
    }
}

#[test]
pub fn decompose_implementations() {
    assert_eq!(34, Calculation::add(29).to(5));
    assert_eq!(24, Calculation::remove(29).from(5));
    assert_eq!(
        "Hello moon!".to_string(),
        StringHandling::concat("Hello ").with("moon!")
    );
    assert_eq!("CH4".to_string(), StringHandling::concat("CH").with_int(4));
}

#[test]
pub fn document_implementations() {
    let text = assemblist_text! {
        impl Calculation {
            /// Provide an integer to be added.
            ///---
            /// Provide a second integer to be added to the first.
            fn add(a: isize).to(b: isize) -> isize { a + b }
            /// Provide an integer to be removed.
            ///---
            /// Provide a second integer to be removed from the first.
            fn remove(a: isize).from(b: isize) -> isize { a - b }
        }
        impl StringHandling {
            /// Provide a string to be concatenated.
            fn concat<'a>(a: &'a str).{
                /// Provide the second string to be concatenated.
                fn with(b: &'a str) -> String {
                    let mut result = String::new();
                    result.push_str(a);
                    result.push_str(b);
                    result
                }
                /// Provide an integer as the second string to be concatenated.
                fn with_int(b: isize) -> String {
                    let mut result = String::new();
                    result.push_str(a);
                    result.push_str(format!("{}", b).as_str());
                    result
                }
            }
        }
    };

    asserts::equivalent!(
        text,
        "#[doc = \"Intermediary module for partial method chain [`Calculation`]`::`[`add`](method@Calculation::add)`(…).…`\"]
        #[doc = \"\"]
        #[doc = \"Following method chains are supported:\"]
        #[doc = \"- [`Calculation`]`::`[`add`](method@Calculation::add)`(…).`[`to`](method@add::Output::to)`(…)`\"]
        mod add {
            # ! [allow(unused_imports)]
            use super :: * ;
            #[doc = \"Intermediary type returned by partial method chain [`Calculation`]`::`[`add`](method@super::Calculation::add)`(…).…`\"]
            pub struct Output { pub(super) a : isize, }
            impl Output {
                #[doc = \" Provide a second integer to be added to the first.\"] 
                pub fn to(self, b : isize) -> isize { let a = self.a; a + b }
            }
        }
        #[doc = \"Intermediary module for partial method chain [`Calculation`]`::`[`remove`](method@Calculation::remove)`(…).…`\"]
        #[doc = \"\"]
        #[doc = \"Following method chains are supported:\"]
        #[doc = \"- [`Calculation`]`::`[`remove`](method@Calculation::remove)`(…).`[`from`](method@remove::Output::from)`(…)`\"]
        mod remove {
            # ! [allow(unused_imports)]
            use super :: * ;
            #[doc = \"Intermediary type returned by partial method chain [`Calculation`]`::`[`remove`](method@super::Calculation::remove)`(…).…`\"]
            pub struct Output { pub(super) a : isize, }
            impl Output {
                #[doc = \" Provide a second integer to be removed from the first.\"] 
                pub fn from(self, b : isize) -> isize { let a = self.a; a - b }
            }
        }
        impl Calculation {
            #[doc = \" Provide an integer to be added.\"]
            fn add(a : isize) -> add :: Output { add :: Output { a, } } 
            #[doc = \" Provide an integer to be removed.\"]
            fn remove(a : isize) -> remove :: Output { remove :: Output { a, } }
        }
        #[doc = \"Intermediary module for partial method chain [`StringHandling`]`::`[`concat`](method@StringHandling::concat)`(…).…`\"]
        #[doc = \"\"]
        #[doc = \"Following method chains are supported:\"]
        #[doc = \"- [`StringHandling`]`::`[`concat`](method@StringHandling::concat)`(…).`[`with`](method@concat::Output::with)`(…)`\"]
        #[doc = \"- [`StringHandling`]`::`[`concat`](method@StringHandling::concat)`(…).`[`with_int`](method@concat::Output::with_int)`(…)`\"]
        mod concat {
            # ! [allow(unused_imports)]
            use super :: * ;
            #[doc = \"Intermediary type returned by partial method chain [`StringHandling`]`::`[`concat`](method@super::StringHandling::concat)`(…).…`\"]
            pub struct Output < 'a > { pub(super) a : & 'a str, }
            impl < 'a > Output < 'a > {
                #[doc = \" Provide the second string to be concatenated.\"]
                pub fn with(self, b : & 'a str) -> String {
                    let a = self.a;
                    let mut result = String :: new();
                    result.push_str(a);
                    result.push_str(b);
                    result
                }
                #[doc = \" Provide an integer as the second string to be concatenated.\"]
                pub fn with_int(self, b : isize) -> String {
                    let a = self.a;
                    let mut result = String :: new();
                    result.push_str(a);
                    result.push_str(format!(\"{}\", b).as_str());
                    result
                }
            }
        }
        impl StringHandling {
            #[doc = \" Provide a string to be concatenated.\"]
            fn concat < 'a > (a : & 'a str) -> concat :: Output :: < 'a > {
                concat :: Output :: < 'a > { a, }
            }
        }"
    );
}
