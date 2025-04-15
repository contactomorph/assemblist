use assemblist::assemblist;

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
