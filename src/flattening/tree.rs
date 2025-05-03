use crate::flattening::trunk::flatten_trunk;
use crate::model::tree::Tree;
use proc_macro2::TokenStream;

use super::module::produce_module;

pub fn flatten(tree: Tree) -> TokenStream {
    let mut tokens = TokenStream::new();
    for trunk in tree.roots {
        if let Err(error) = flatten_trunk(&mut tokens, &trunk, produce_module) {
            return error;
        }
    }
    tokens
}

#[cfg(test)]
mod tests {
    use quote::quote;

    use crate::{flattening::tree::flatten, model::tree::Tree};

    #[test]
    fn test_flatten_all() {
        let tokens = quote!(pub(crate) fn first<'a>(text: &'a str, uuid: Uuid).second<T>(n: &'a mut T).third(l: usize) -> i64 { compose(l, uuid, combine(text, n)) });

        let tree = asserts::tokens_are_parsable_as::<Tree>(tokens);

        let output = flatten(tree);

        asserts::equivalent(
            output.to_string().as_str(),
            "pub (crate) fn first < 'a > (text : & 'a str , uuid : Uuid) -> first :: Output :: < 'a > {
                first :: Output :: < 'a > { text , uuid , }
            }
            pub (crate) mod first {
                # ! [allow (unused_imports)]
                use super :: * ;
                pub struct Output < 'a > {
                    pub (super) text : & 'a str ,
                    pub (super) uuid : Uuid ,
                }
                impl < 'a > Output < 'a > {
                    pub fn second < T > (self , n : & 'a mut T) -> second :: Output :: < 'a , T > {
                        let text = self . text ;
                        let uuid = self . uuid ;
                        second :: Output :: < 'a , T > { n , text , uuid , }
                    }
                }
                pub mod second {
                    # ! [allow (unused_imports)]
                    use super :: * ;
                    pub struct Output < 'a , T > {
                        pub (super) n : & 'a mut T ,
                        pub (super) text : & 'a str ,
                        pub (super) uuid : Uuid ,
                    }
                    impl < 'a , T > Output < 'a , T > {
                        pub fn third (self , l : usize) -> i64 {
                            let n = self . n ;
                            let text = self . text ;
                            let uuid = self . uuid ;
                            compose (l , uuid , combine (text , n))
                        }
                    }
                }
            }",
        );
    }
}
