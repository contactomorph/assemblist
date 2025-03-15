use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{ConstParam, GenericParam, Generics, LifetimeParam, TypeParam};

pub struct OrderedGenericList {
    lifetime_gens: Vec<LifetimeParam>,
    const_gens: Vec<ConstParam>,
    type_gens: Vec<TypeParam>,
    last_gens: Vec<GenericParam>,
}

impl OrderedGenericList {
    pub fn augment(
        previous: Option<&OrderedGenericList>,
        generics: &Generics,
    ) -> OrderedGenericList {
        let lifetime_gens = previous
            .map(|l| l.lifetime_gens.clone())
            .unwrap_or(Vec::new());
        let const_gens = previous.map(|l| l.const_gens.clone()).unwrap_or(Vec::new());
        let type_gens = previous.map(|l| l.type_gens.clone()).unwrap_or(Vec::new());
        let last_gens = generics.params.iter().map(|g| g.clone()).collect();
        let mut list = OrderedGenericList {
            lifetime_gens,
            const_gens,
            type_gens,
            last_gens,
        };
        for g in generics.params.iter() {
            match g {
                GenericParam::Lifetime(lt) => list.lifetime_gens.push(lt.clone()),
                GenericParam::Const(cst) => list.const_gens.push(cst.clone()),
                GenericParam::Type(tp) => list.type_gens.push(tp.clone()),
            }
        }
        list
    }

    pub fn count(&self) -> usize {
        self.lifetime_gens.len() + self.const_gens.len() + self.type_gens.len()
    }

    // :: <⟨generic1⟩, …, ⟨genericN⟩>
    pub fn produce_complete_generics(&self, must_prefix: bool, tokens: &mut TokenStream) {
        if self.count() == 0 {
            return;
        }

        let span = Span::call_site();

        if must_prefix {
            syn::token::PathSep {
                spans: [span, span],
            }
            .to_tokens(tokens);
        }

        syn::token::Lt { spans: [span] }.to_tokens(tokens);
        let mut first = true;
        Self::separate_with_comma(self.lifetime_gens.iter(), &mut first, tokens);
        Self::separate_with_comma(self.const_gens.iter(), &mut first, tokens);
        Self::separate_with_comma(self.type_gens.iter(), &mut first, tokens);
        syn::token::Gt { spans: [span] }.to_tokens(tokens);
    }

    // <⟨generic1⟩, …, ⟨genericN⟩>
    pub fn produce_last_generics(&self, tokens: &mut TokenStream) {
        if self.last_gens.len() == 0 {
            return;
        }

        let span = Span::call_site();

        syn::token::Lt { spans: [span] }.to_tokens(tokens);
        let mut first = true;
        Self::separate_with_comma(self.last_gens.iter(), &mut first, tokens);
        syn::token::Gt { spans: [span] }.to_tokens(tokens);
    }

    fn separate_with_comma<'a, T: 'a + ToTokens>(
        iterator: impl Iterator<Item = &'a T>,
        first: &mut bool,
        tokens: &mut TokenStream,
    ) {
        let span = Span::call_site();
        let comma = syn::token::Comma { spans: [span] };
        for item in iterator {
            if *first {
                *first = false
            } else {
                comma.to_tokens(tokens)
            }
            item.to_tokens(tokens);
        }
    }
}
