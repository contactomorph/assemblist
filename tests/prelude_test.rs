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
        #[doc = \"Intermediary module for partial method chain [`who`](fn@who)`(…).…`\"]
        #[doc = \"\"]
        #[doc = \"Following method chains are supported:\"]
        #[doc = \"- [`who`](fn@who)`(…).`[`are`](method@who::Output::are)`(…).`[`you`](method@who::are::Output::you)`(…)`\"]
        #[warn(dead_code)]
        pub(crate) mod who {
            # ! [allow(unused_imports)]
            use super :: * ;
            #[doc = \"Intermediary type returned by partial method chain [`who`](fn@super::who)`(…).…`\"] 
            pub struct Output {}
            impl Output {
                pub fn are(self,) -> are :: Output { are :: Output {} }
            }
            #[doc = \"Intermediary module for partial method chain [`who`](fn@super::who)`(…).`[`are`](method@Output::are)`(…).…`\"]
            #[doc = \"\"]
            #[doc = \"Following method chains are supported:\"]
            #[doc = \"- [`who`](fn@super::who)`(…).`[`are`](method@Output::are)`(…).`[`you`](method@are::Output::you)`(…)`\"]
            pub mod are {
                # ! [allow(unused_imports)]
                use super :: * ;
                #[doc = \"Intermediary type returned by partial method chain [`who`](fn@super::super::who)`(…).`[`are`](method@super::Output::are)`(…).…`\"] 
                pub struct Output {}
                impl Output {
                    pub async fn you(self,) {}
                }
            }
        }
        #[allow(dead_code)]
        pub(crate) async fn single() {}
        #[doc = \"Intermediary module for partial method chain [`Foo`]`::`[`we`](method@Foo::we)`(…).…`\"]
        #[doc = \"\"]
        #[doc = \"Following method chains are supported:\"]
        #[doc = \"- [`Foo`]`::`[`we`](method@Foo::we)`(…).`[`cannot`](method@we::Output::cannot)`(…).`[`talk`](method@we::cannot::Output::talk)`(…)`\"]
        #[error(dead_code)]
        pub(self) mod we {
            # ! [allow(unused_imports)]
            use super :: * ;
            #[doc = \"Intermediary type returned by partial method chain [`Foo`]`::`[`we`](method@super::Foo::we)`(…).…`\"]
            pub struct Output {}
            impl Output {
                pub fn cannot(self,) -> cannot :: Output { cannot :: Output {} }
            }
            #[doc = \"Intermediary module for partial method chain [`Foo`]`::`[`we`](method@super::Foo::we)`(…).`[`cannot`](method@Output::cannot)`(…).…`\"]
            #[doc = \"\"]
            #[doc = \"Following method chains are supported:\"]
            #[doc = \"- [`Foo`]`::`[`we`](method@super::Foo::we)`(…).`[`cannot`](method@Output::cannot)`(…).`[`talk`](method@cannot::Output::talk)`(…)`\"]
            pub mod cannot {
                # ! [allow(unused_imports)]
                use super :: * ;
                #[doc = \"Intermediary type returned by partial method chain [`Foo`]`::`[`we`](method@super::super::Foo::we)`(…).`[`cannot`](method@super::Output::cannot)`(…).…`\"]
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
