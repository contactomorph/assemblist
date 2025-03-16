use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::WhereClause;
use syn::{ConstParam, GenericParam, Generics, LifetimeParam, TypeParam};

pub struct OrderedGenericList {
    lifetime_gens: Vec<LifetimeParam>,
    const_gens: Vec<ConstParam>,
    type_gens: Vec<TypeParam>,
    last_gens: Vec<GenericParam>,
    where_clause: Option<WhereClause>,
}

impl OrderedGenericList {
    pub fn augment(
        previous: Option<&OrderedGenericList>,
        generics: &Generics,
    ) -> OrderedGenericList {
        let lifetime_gens = previous
            .map(|l| l.lifetime_gens.clone())
            .unwrap_or_default();
        let const_gens = previous.map(|l| l.const_gens.clone()).unwrap_or_default();
        let type_gens = previous.map(|l| l.type_gens.clone()).unwrap_or_default();
        let last_gens = generics.params.iter().cloned().collect();
        let mut list = OrderedGenericList {
            lifetime_gens,
            const_gens,
            type_gens,
            last_gens,
            where_clause: generics.where_clause.clone(),
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

    // :: <⟨name1⟩, …, ⟨nameN⟩>
    pub fn produce_complete_generic_names(&self, must_prefix: bool, tokens: &mut TokenStream) {
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
        Self::separate_with_comma(
            self.lifetime_gens.iter().map(|lt| &lt.lifetime),
            &mut first,
            tokens,
        );
        Self::separate_with_comma(self.const_gens.iter().map(|c| &c.ident), &mut first, tokens);
        Self::separate_with_comma(self.type_gens.iter().map(|t| &t.ident), &mut first, tokens);
        syn::token::Gt { spans: [span] }.to_tokens(tokens);
    }

    // <⟨generic1⟩, …, ⟨genericN⟩>
    pub fn produce_complete_constrained_generics(&self, tokens: &mut TokenStream) {
        if self.count() == 0 {
            return;
        }

        let span = Span::call_site();

        syn::token::Lt { spans: [span] }.to_tokens(tokens);
        let mut first = true;
        Self::separate_with_comma(self.lifetime_gens.iter(), &mut first, tokens);
        Self::separate_with_comma(self.const_gens.iter(), &mut first, tokens);
        Self::separate_with_comma(self.type_gens.iter(), &mut first, tokens);
        syn::token::Gt { spans: [span] }.to_tokens(tokens);
    }

    // <⟨generic1⟩, …, ⟨genericN⟩>
    pub fn produce_last_contrained_generics(&self, tokens: &mut TokenStream) {
        if self.last_gens.is_empty() {
            return;
        }

        let span = Span::call_site();

        syn::token::Lt { spans: [span] }.to_tokens(tokens);
        let mut first = true;
        Self::separate_with_comma(self.last_gens.iter(), &mut first, tokens);
        syn::token::Gt { spans: [span] }.to_tokens(tokens);
    }

    // where ⟨constraints⟩
    pub fn produce_where_clause(&self, tokens: &mut TokenStream) {
        self.where_clause.as_ref().to_tokens(tokens)
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
