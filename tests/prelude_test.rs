use assemblist::assemblist_text;

#[test]
fn verify_preludes() {
    let text = assemblist_text! {
        #[warn(dead_code)]
        pub(crate) async fn who().are().you() {}
        #[allow(dead_code)]
        pub(crate) async fn single() {}
        #[cfg(test)]
        impl Foo {
            #[error(dead_code)]
            pub(self) async fn we().cannot().talk() {}
            async fn nothing() {}
        }
    };

    asserts::equivalent(
        text,
        "#[warn(dead_code)]
        pub(crate) fn who() -> who :: Output { who :: Output {} }
        #[warn(dead_code)]
        pub(crate) mod who {
            # ! [allow(unused_imports)]
            use super :: * ;
            pub struct Output {}
            impl Output {
                pub fn are(self,) -> are :: Output { are :: Output {} }
            }
            pub mod are {
                # ! [allow(unused_imports)]
                use super :: * ;
                pub struct Output {}
                impl Output {
                    pub async fn you(self,) {}
                }
            }
        }
        #[allow(dead_code)]
        pub(crate) async fn single() {}
        #[error(dead_code)]
        pub(self) mod we {
            # ! [allow(unused_imports)]
            use super :: * ;
            pub struct Output {}
            impl Output {
                pub fn cannot(self,) -> cannot :: Output { cannot :: Output {} }
            }
            pub mod cannot {
                # ! [allow(unused_imports)]
                use super :: * ;
                pub struct Output {}
                impl Output {
                    pub async fn talk(self,) {}
                }
            }
        }
        #[cfg(test)]
        impl Foo {
            #[error(dead_code)]
            pub(self) fn we() -> we :: Output { we :: Output {} }
            async fn nothing() {}
        }",
    )
}

pub mod asserts {
    pub fn equivalent(a: &str, b: &str) {
        let mut a = simplify(a);
        let mut b = simplify(b);

        let mut common = Vec::<char>::new();
        let mut pair;
        loop {
            pair = (a.next(), b.next());
            match pair {
                (None, None) => return,
                (Some(x), Some(y)) if x == y => common.push(x),
                _ => break,
            }
        }

        let mut left = last_to_string(&common, 20);
        let mut right = left.clone();
        let common_len = left.len();
        let remaining_len = 70 - common_len;
        let mut div = String::with_capacity(common_len+1);
        for _ in 0..common_len { div.push(' ') }
        div.push('v');
        pair.0.map(|c| left.push(c));
        pair.1.map(|c| right.push(c));
        for c in a.take(remaining_len) { left.push(c) }
        for c in b.take(remaining_len) { right.push(c) }
        panic!("assertion `left == right` failed\n   div: {}\n  left: {}\n right: {}\n", div, left, right)
    }

    fn last_to_string(chars: &Vec<char>, max_len: usize) -> String {
        let len = chars.len();
        if chars.len() <= max_len {
            chars.iter().collect()
        } else {
            chars[len-max_len..len].iter().collect()
        }
    }

    fn simplify<'a>(text: &'a str) -> impl Iterator<Item = char> + 'a {
        let mut whitespace_found = false;
        let mut starting = true;
        text.chars()
            .flat_map(move |c| {
                let chars: [char; 2];
                if c.is_whitespace() {
                    whitespace_found = true;
                    chars = ['\0', '\0'];
                } else if whitespace_found && !starting {
                    whitespace_found = false;
                    starting = false;
                    chars = [' ', c];
                } else {
                    whitespace_found = false;
                    starting = false;
                    chars = ['\0', c];
                }
                chars
            })
            .filter(|c| *c != '\0')
    }
}
