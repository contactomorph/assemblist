use std::{fmt::Debug, marker::PhantomData};

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
            #[inline]
            fn add(a : isize) -> add :: Output { add :: Output { a, } }
            #[doc = \" Provide an integer to be removed.\"]
            #[inline]
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
            #[inline]
            fn concat < 'a > (a : & 'a str) -> concat :: Output :: < 'a > {
                concat :: Output :: < 'a > { a, }
            }
        }"
    );
}

struct VecHandling<T> {
    ph: PhantomData<T>,
}

assemblist! {
    impl<T> VecHandling<T> {
        fn compute_len<'a>(v: &'a Vec<T>).and_return_it() -> usize { v.len() }
    }
}

#[test]
pub fn decompose_generic_implementations() {
    let vec = vec![
        "Alpha".to_string(),
        "Beta".to_string(),
        "Gamma".to_string(),
        "Delta".to_string(),
    ];
    assert_eq!(4, VecHandling::compute_len(&vec).and_return_it());
    let vec = vec![4.5, 56.0, -0.4, -2323.12, 89.03];
    assert_eq!(5, VecHandling::compute_len(&vec).and_return_it());
}

struct DoubleWhere<T>
where
    T: Debug,
{
    ph: PhantomData<T>,
}

assemblist! {
    impl<T> DoubleWhere<T> where T: Debug {
        pub fn consider<'a>(x: &'a T).as_well_as<U>(y: &'a U).and_display_them() -> String where U: Debug {
            format!("x:{:?}, y:{:?}", x, y)
        }
    }
}

#[test]
pub fn decompose_generic_implementations_with_where_clause() {
    let s = "Message".to_string();
    let x = 99.99;
    let result = DoubleWhere::consider(&s).as_well_as(&x).and_display_them();
    assert_eq!(result.as_str(), "x:\"Message\", y:99.99")
}

#[test]
pub fn verify_generic_implementations_with_where_clause() {
    let text = assemblist_text! {
        impl<T> DoubleWhere<T> where T: Debug {
            pub fn consider<'a>(x: &'a T).as_well_as<U>(y: &'a U).and_display_them() -> String where U: Debug {
                format!("x:{:?}, y:{:?}", x, y)
            }
        }
    };

    asserts::equivalent!(
        text,
        "#[doc = \"Intermediary module for partial method chain [`DoubleWhere`]`::`[`consider`](method@DoubleWhere::consider)`(…).…`\"]
        #[doc = \"\"]
        #[doc = \"Following method chains are supported:\"]
        #[doc = \"- [`DoubleWhere`]`::`[`consider`](method@DoubleWhere::consider)`(…).`[`as_well_as`](method@consider::Output::as_well_as)`(…).`[`and_display_them`](method@consider::as_well_as::Output::and_display_them)`(…)`\"]
        pub mod consider {
            # ! [allow(unused_imports)] use super :: * ;
            #[doc = \"Intermediary type returned by partial method chain [`DoubleWhere`]`::`[`consider`](method@super::DoubleWhere::consider)`(…).…`\"]
            pub struct Output < 'a, T > where T : Debug { pub(super) x : & 'a T, }
            impl < 'a, T > Output < 'a, T > where T : Debug {
                #[inline]
                pub fn as_well_as < U > (self, y : & 'a U) -> as_well_as :: Output :: < 'a, T, U > {
                    let x = self.x;
                    as_well_as :: Output :: < 'a, T, U > { y, x, }
                }
            }
            #[doc = \"Intermediary module for partial method chain [`DoubleWhere`]`::`[`consider`](method@super::DoubleWhere::consider)`(…).`[`as_well_as`](method@Output::as_well_as)`(…).…`\"]
            #[doc = \"\"]
            #[doc = \"Following method chains are supported:\"]
            #[doc = \"- [`DoubleWhere`]`::`[`consider`](method@super::DoubleWhere::consider)`(…).`[`as_well_as`](method@Output::as_well_as)`(…).`[`and_display_them`](method@as_well_as::Output::and_display_them)`(…)`\"]
            pub mod as_well_as {
                # ! [allow(unused_imports)] use super :: * ;
                #[doc = \"Intermediary type returned by partial method chain [`DoubleWhere`]`::`[`consider`](method@super::super::DoubleWhere::consider)`(…).`[`as_well_as`](method@super::Output::as_well_as)`(…).…`\"]
                pub struct Output < 'a, T, U > where T : Debug {
                    pub(super) y : & 'a U,
                    pub(super) x : & 'a T,
                }
                impl < 'a, T, U > Output < 'a, T, U > where T : Debug {
                    pub fn and_display_them(self,) -> String where U : Debug {
                        let y = self.y; let x = self.x;
                        format! (\"x:{:?}, y:{:?}\", x, y)
                    }
                }
            }
        }
        impl < T > DoubleWhere < T > where T : Debug {
            #[inline]
            pub fn consider < 'a > (x : & 'a T) -> consider :: Output :: < 'a, T > {
                consider :: Output :: < 'a, T > { x, }
            }
        }"
    );
}

struct MyVec<T> {
    _inner: Vec<T>,
}

impl<T> MyVec<T>
where
    T: Debug,
{
    pub fn new(data: Vec<T>) -> Self {
        Self { _inner: data }
    }
}

assemblist! {
    impl<T> MyVec<T> where T: Debug {
        pub fn take_at_most<'a>(&'a self, n: usize).comparing_to<'b, U>(other: &'b MyVec<U>).applying(f: impl Fn(&T, &U) -> bool) -> bool {
            let m = std::cmp::min(std::cmp::min(self_._inner.len(), other._inner.len()), n);
            for i in 0..m {
                if !f(&self_._inner[i], &other._inner[i]) { return false }
            }
            true
        }
        pub fn at<'a>(&'a mut self, index: usize).{
            fn insert(element: T) {
                self_._inner.insert(index, element)
            }
            fn remove() -> T {
                self_._inner.remove(index)
            }
        }
    }
}

#[test]
pub fn decompose_generic_implementations_with_self_receiver() {
    let mut x = MyVec::new(vec!["Blue", "Orange", "Red", "Lavender"]);
    let y = MyVec::new(vec![4, 6, 3, 8, 2002]);

    assert!(x.take_at_most(8).comparing_to(&y).applying(|t, n| t.len() == *n));

    x.at(1).insert("Yellow");
    assert_eq!(5, x._inner.len());
    x.at(2).remove();

    assert!(x.take_at_most(8).comparing_to(&y).applying(|t, n| t.len() == *n));
}

#[test]
pub fn verify_generic_implementations_with_self_receiver() {
    let text = assemblist_text! {
        impl<T> MyVec<T> where T: Debug {
            pub fn take_at_most<'a>(&'a self, n: usize)
                .comparing_to<'b, U>(other: &'b MyVec<U>)
                .applying(f: impl Fn(&T, &U) -> bool) -> bool
            {
                let m = std::cmp::min(std::cmp::min(self_._inner.len(), other._inner.len()), n);
                for i in 0..m {
                    if !f(&self_._inner[i], &other._inner[i]) { return false }
                }
                true
            }
        }
    };

    asserts::equivalent!(
        text,
        "#[doc = \"Intermediary module for partial method chain [`MyVec`]`::`[`take_at_most`](method@MyVec::take_at_most)`(…).…`\"]
        #[doc = \"\"]
        #[doc = \"Following method chains are supported:\"]
        #[doc = \"- [`MyVec`]`::`[`take_at_most`](method@MyVec::take_at_most)`(…).`[`comparing_to`](method@take_at_most::Output::comparing_to)`(…).`[`applying`](method@take_at_most::comparing_to::Output::applying)`(…)`\"]
        pub mod take_at_most {
            # ! [allow(unused_imports)]
            use super :: * ;
            #[doc = \"Intermediary type returned by partial method chain [`MyVec`]`::`[`take_at_most`](method@super::MyVec::take_at_most)`(…).…`\"]
            pub struct Output < 'a, T > where T : Debug {
                pub(super) self_ : & 'a MyVec < T > ,
                pub(super) n : usize,
            }
            impl < 'a, T > Output < 'a, T > where T : Debug {
                #[inline]
                pub fn comparing_to < 'b, U > (self, other : & 'b MyVec < U >) -> comparing_to :: Output :: < 'a, 'b, T, U > {
                    let self_ = self.self_;
                    let n = self.n;
                    comparing_to :: Output :: < 'a, 'b, T, U > { other, self_, n, }
                }
            }
            #[doc = \"Intermediary module for partial method chain [`MyVec`]`::`[`take_at_most`](method@super::MyVec::take_at_most)`(…).`[`comparing_to`](method@Output::comparing_to)`(…).…`\"]
            #[doc = \"\"]
            #[doc = \"Following method chains are supported:\"]
            #[doc = \"- [`MyVec`]`::`[`take_at_most`](method@super::MyVec::take_at_most)`(…).`[`comparing_to`](method@Output::comparing_to)`(…).`[`applying`](method@comparing_to::Output::applying)`(…)`\"]
            pub mod comparing_to {
                # ! [allow(unused_imports)]
                use super :: * ;
                #[doc = \"Intermediary type returned by partial method chain [`MyVec`]`::`[`take_at_most`](method@super::super::MyVec::take_at_most)`(…).`[`comparing_to`](method@super::Output::comparing_to)`(…).…`\"]
                pub struct Output < 'a, 'b, T, U > where T : Debug {
                    pub(super) other : & 'b MyVec < U > ,
                    pub(super) self_ : & 'a MyVec < T > ,
                    pub(super) n : usize,
                }
                impl < 'a, 'b, T, U > Output < 'a, 'b, T, U > where T : Debug {
                    pub fn applying(self, f : impl Fn(& T, & U) -> bool) -> bool where T : Debug {
                        let other = self.other;
                        let self_ = self.self_;
                        let n = self.n;
                        let m = std :: cmp :: min(std::cmp::min(self_._inner.len(), other._inner.len()), n);
                        for i in 0 .. m {
                            if !f(&self_._inner[i], &other._inner[i]) { return false }
                        }
                        true
                    }
                }
            }
        }
        impl < T > MyVec < T > where T : Debug {
            #[inline]
            pub fn take_at_most < 'a > (& 'a self, n : usize) -> take_at_most :: Output :: < 'a, T > {
                take_at_most :: Output :: < 'a, T > { self_ : self, n, }
            }
        }"
    );
}
