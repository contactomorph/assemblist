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

    asserts::equivalent!(
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
        }"
    )
}
