use proc_macro2::{Span, TokenStream};
use quote::{quote_spanned, ToTokens};
use syn::{punctuated::Punctuated, token::{Brace, Comma}, Attribute, FnArg, GenericParam, Ident, Item, Pat, PatType, ReturnType, Token, Type, Visibility};

use crate::model::{section::Section, tree::{Branch, Tree, Trunk}};

struct UsualArg {
    attrs: Vec<Attribute>,
    ident: Ident,
    colon_token: Token![:],
    ty: Box<Type>,
}

struct Step<'a, 'b> {
    section: &'a Section,
    args: &'b Vec<UsualArg>,
}

struct Introduction<'a> {
    attrs: &'a Vec<Attribute>,
    vis: &'a Visibility,
    asyncness: Option<Token![async]>,
}

pub struct Conclusion<'a> {
    output: &'a ReturnType,
    brace: Brace,
    body: &'a TokenStream,
}

struct MultiStep<'a, 'b> {
    introduction: Introduction<'a>,
    fn_token: Token![fn],
    previous: Option<&'a MultiStep<'a, 'b>>,
    current: Step<'a, 'b>,
    conclusion: Option<Conclusion<'a>>,
}

impl<'a, 'b> MultiStep<'a, 'b> {
    pub fn count_generics(&self) -> usize {
        let n = self.current.section.generics.params.len();
        match self.previous {
            Some(previous) => n + previous.count_generics(),
            None => n,
        }
    }

    pub fn generics_iter(&self) -> impl Iterator<Item=GenericParam> {
        for param in self.current.section.generics.params.iter() {
            yield param;
        }
    }
}

fn extract_usual_arg(typed_arg: &PatType) -> std::result::Result<UsualArg, TokenStream> {
    match &*typed_arg.pat {
        Pat::Ident(pat_ident) => {
            if let Some(subpat) = &pat_ident.subpat {
                let message = "Subpatterns are not supported";
                let span = subpat.0.span;
                return Err(quote_spanned! { span => compile_error!(#message); });
            }
            if let Some(by_ref) = &pat_ident.by_ref {
                let message = "By ref parameter are not supported";
                let span = by_ref.span;
                return Err(quote_spanned! { span => compile_error!(#message); });
            }
            if let Some(mutability) = &pat_ident.mutability {
                let message = "Mutability is not supported";
                let span = mutability.span;
                return Err(quote_spanned! { span => compile_error!(#message); });
            }
            Ok(UsualArg {
                attrs: typed_arg.attrs.clone(),
                ident: pat_ident.ident.clone(),
                colon_token: typed_arg.colon_token,
                ty: typed_arg.ty.clone(),
            })
        }
        _ => {
            let message = "Only basic identifier pattern is supported";
            let span = typed_arg.colon_token.span;
            Err(quote_spanned! { span => compile_error!(#message); })
        }
    }
}

fn extract_usual_args(args: &Punctuated<FnArg, Comma>) -> std::result::Result<Vec<UsualArg>, TokenStream> {
    let mut output_args: Vec<UsualArg> = Vec::new();
    for input in args.iter() {
        match input {
            FnArg::Typed(typed_arg) => {
                let arg = extract_usual_arg(typed_arg)?;
                output_args.push(arg);
            }
            FnArg::Receiver(receiver) => {
                let message = "self receiver is not supported";
                let span = receiver.self_token.span;
                return Err(quote_spanned! { span => compile_error!(#message); });
            }
        }
    }
    Ok(output_args)
}

pub type FlatteningResult = std::result::Result<(), TokenStream>;

pub trait Flattener {
    fn to_flat_representation(&self, tokens: &mut TokenStream) -> FlatteningResult;
}

fn to_mod_common_imports(span: Span, tokens: &mut TokenStream) {
    // #![allow(unused_imports)]
    // use super::*;
    syn::token::Use { span }.to_tokens(tokens);
    syn::token::Super { span }.to_tokens(tokens);
    syn::token::PathSep { spans: [span, span] }.to_tokens(tokens);
    syn::token::Star { spans: [span] }.to_tokens(tokens);
    syn::token::Semi { spans: [span] }.to_tokens(tokens);
}

// fn to_output_definition(section: &Section, arg_set: &UsualArgSet, tokens: &mut TokenStream) {
//     let span = section.ident.span();
//     // pub struct Output ⟨generics⟩ {
//     //      pub ⟨field1⟩: ⟨ty1⟩;
//     //      …
//     //      pub ⟨fieldN⟩: ⟨tyN⟩;
//     // }
//     syn::token::Pub { span }.to_tokens(tokens);
//     syn::token::Struct { span }.to_tokens(tokens);
//     Ident::new("Output", span).to_tokens(tokens);
//     section.generics.to_tokens(tokens);
//     Brace::default().surround(tokens, |tokens| {
//         for arg in arg_set.args.iter() {
//             let span = arg.colon_token.span;
//             syn::token::Pub { span }.to_tokens(tokens);
//             for attr in &arg.attrs { attr.to_tokens(tokens) }
//             arg.ident.to_tokens(tokens);
//             arg.colon_token.to_tokens(tokens);
//             arg.ty.to_tokens(tokens);
//             syn::token::Semi { spans: [span] }.to_tokens(tokens);
//         }
//     });
// }

fn to_complete_generics(multi: &MultiStep, tokens: &mut TokenStream) {
    let generics = &multi.current.section.generics;
    let spans = [Span::call_site()];
    let count = multi.count_generics();

    if 0 < count {
        generics.lt_token.to_tokens(tokens);

        for step in &multi.previous {
            for param in step.section.generics.params.iter() {
                param.to_tokens(tokens);
                syn::token::Comma { spans }.to_tokens(tokens)
            }
        }

        for param in generics.params.iter() {
            param.to_tokens(tokens);
            syn::token::Comma { spans }.to_tokens(tokens)
        }

        generics.gt_token.to_tokens(tokens);
    }
}

fn to_output_instance(multi: &MultiStep, tokens: &mut TokenStream) {
    let section = multi.current.section;
    let span = Span::call_site();
    let spans = [span];
    // ⟨name⟩::Output ⟨generics⟩ { ⟨field1⟩, …, ⟨fieldN⟩, }
    section.ident.to_tokens(tokens);
    syn::token::PathSep { spans: [span, span] }.to_tokens(tokens);
    Ident::new("Output", span).to_tokens(tokens);
    to_complete_generics(multi, tokens);
    Brace::default().surround(tokens, |tokens| {
        for step in &multi.previous {
            for arg in step.args.iter() {
                arg.ident.to_tokens(tokens);
                syn::token::Comma { spans }.to_tokens(tokens)
            }
        }
        for arg in multi.current.args.iter() {
            arg.ident.to_tokens(tokens);
            syn::token::Comma { spans }.to_tokens(tokens);
        }
    })
}

fn to_introduction(introduction: &Introduction, tokens: &mut TokenStream) {
    // ⟨attr⟩ ⟨visibility⟩ ⟨async⟩ 
    for attr in introduction.attrs { attr.to_tokens(tokens) }
    introduction.vis.to_tokens(tokens);
    introduction.asyncness.to_tokens(tokens);
}

fn to_conclusion(conclusion: &Conclusion, tokens: &mut TokenStream) {
    conclusion.output.to_tokens(tokens);
    conclusion
        .brace
        .surround(tokens, |tokens| conclusion.body.to_tokens(tokens));
}

fn to_output_creation(multi: &MultiStep, tokens: &mut TokenStream) {
    let span = Span::call_site();
    syn::token::RArrow { spans: [span, span] }.to_tokens(tokens);
    multi.current.section.ident.to_tokens(tokens);
    syn::token::PathSep { spans: [span, span] }.to_tokens(tokens);
    Ident::new("Output", span).to_tokens(tokens);
    to_complete_generics(multi, tokens);
    Brace::default().surround(tokens, |tokens| {
        to_output_instance(multi, tokens)
    });
}

fn to_section_function(multi: &MultiStep, tokens: &mut TokenStream) -> FlatteningResult {
    // ⟨introduction⟩ fn ⟨signature⟩ -> ⟨name⟩::Output ⟨generics⟩ {
    //   ⟨output_instance⟩
    // }
    // ⟨introduction⟩ fn ⟨name⟩⟨generics⟩(⟨inputs⟩) ⟨conclusion⟩
    let section = multi.current.section;
    let span = section.ident.span();

    if multi.previous.is_empty() { 
        to_introduction(&multi.introduction, tokens);
    } else {
        syn::token::Pub { span: Span::call_site() }.to_tokens(tokens);
    }
    multi.fn_token.to_tokens(tokens);
    section.ident.to_tokens(tokens);
    multi.current.section.generics.to_tokens(tokens);
    section.paren_token
        .surround(tokens, |tokens| 
            section.inputs.to_tokens(tokens)
        );

    match &multi.conclusion {
        Some(conclusion) => to_conclusion(conclusion, tokens),
        None => to_output_creation(multi, tokens),
    }
    Ok(())
}

// fn to_base_section_module(trunk: &Trunk, section: &Section, sub: &(Branch, Vec<Branch>), arg_set: &UsualArgSet, tokens: &mut TokenStream) -> FlatteningResult {
//     let span = trunk.fn_token.span;
//     // ⟨visibility⟩ mod ⟨name⟩ {
//     //     ⟨common_imports⟩
//     //     ⟨output_definition⟩
//     //     ⟨functions⟩
//     // }
//     trunk.vis.to_tokens(tokens);
//     syn::token::Mod { span }.to_tokens(tokens);
//     section.ident.to_tokens(tokens);
//     let mut result: FlatteningResult = Ok(());
//     Brace::default().surround(tokens, |tokens| {
//         if result.is_ok() {
//             to_mod_common_imports(span, tokens);
//             to_output_definition(section, arg_set, tokens);
//             //result = to_nested_branch(&sub.0, &sub.1, tokens);
//         }
//     });
//     result
// }

// fn to_base_section_function(trunk: &Trunk, section: &Section, arg_set: &UsualArgSet, tokens: &mut TokenStream) -> FlatteningResult {
//     let span = trunk.fn_token.span;
//     // ⟨attr⟩ ⟨visibility⟩ ⟨async⟩ fn ⟨signature⟩ -> ⟨name⟩::Output ⟨generics⟩ {
//     //   ⟨output_instance⟩
//     // }
//     for attr in &trunk.attrs { attr.to_tokens(tokens) }
//     trunk.vis.to_tokens(tokens);
//     trunk.asyncness.to_tokens(tokens);
//     trunk.fn_token.to_tokens(tokens);
//     section.ident.to_tokens(tokens);
//     section.generics.to_tokens(tokens);
//     section.paren_token
//         .surround(tokens, |tokens| section.inputs.to_tokens(tokens));

//     syn::token::RArrow { spans: [span, span] }.to_tokens(tokens);
//     section.ident.to_tokens(tokens);
//     syn::token::PathSep { spans: [span, span] }.to_tokens(tokens);
//     Ident::new("Output", span).to_tokens(tokens);
//     section.generics.to_tokens(tokens);
//     Brace::default().surround(tokens, |tokens| {
//         //to_output_instance(section, arg_set, tokens)
//     });
//     Ok(())
// }

fn to_function_chain(branch: &Branch, multi: Option<&MultiStep>, tokens: &mut TokenStream) -> FlatteningResult {
    match (branch, multi) {
        (Branch::Final(final_sec), None) => {
            todo!()
        }
        (Branch::Alternative(continuing, branches), _) => {
            let section = &continuing.section;
            let arg_set = extract_usual_args(&continuing.section.inputs)?;
            // to_base_section_module(self, section, &*branches, &arg_set, tokens)?;
            // to_base_section_function(self, section, &arg_set, tokens)?;
        }
    }
    Ok(())
}

impl Flattener for Trunk {
    fn to_flat_representation<'a>(&'a self, tokens: &mut TokenStream) -> FlatteningResult {
        match &self.branch {
            Branch::Final(final_sec) => {
                let introduction = Introduction::<'a> {
                    attrs: &self.attrs,
                    asyncness: self.asyncness,
                    vis: &self.vis,
                };
                let args = extract_usual_args(&final_sec.section.inputs)?;
                let current = Step {
                    section: &final_sec.section,
                    args: &args,
                };
                let conclusion = Conclusion::<'a> {
                    output: &final_sec.output,
                    brace: final_sec.brace,
                    body: &final_sec.body,
                };
                let multi = MultiStep::<'a> {
                    introduction,
                    fn_token: self.fn_token,
                    previous: None,
                    current,
                    conclusion: Some(conclusion),
                };
                to_section_function(&multi, tokens);
            }
            Branch::Alternative(continuing, branches) => {
                let section = &continuing.section;
                let arg_set = extract_usual_args(&continuing.section.inputs)?;
                // to_base_section_module(self, section, &*branches, &arg_set, tokens)?;
                // to_base_section_function(self, section, &arg_set, tokens)?;
            }
        }
        Ok(())
    }
}

impl Flattener for Tree {
    fn to_flat_representation(&self, tokens: &mut TokenStream) -> FlatteningResult {
        for trunk in &self.roots {
            trunk.to_flat_representation(tokens)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{model::tree::Tree, tools::asserts::assert_tokens_are_parsable_as};
    use proc_macro2::TokenStream;
    use quote::quote;

    use super::Flattener;

    #[test]
    fn flatten_tree() {
        let tokens = quote!(pub(crate) fn first<'a>(text: &'a str).second(n: &'a mut i32) {});

        let tree = assert_tokens_are_parsable_as::<Tree>(tokens);

        let mut otokens = TokenStream::new();
        let res: std::result::Result::<_,_> = tree.to_flat_representation(&mut otokens);
        assert!(res.is_ok());
        assert_eq!(
            otokens.to_string().as_str(),
            "pub (crate) mod first { \
                use super :: * ; \
                pub struct Output < 'a > { pub text : & 'a str ; } \
            } \
            pub (crate) fn first < 'a > (text : & 'a str) -> first :: Output < 'a > { \
                first :: Output < 'a > { text , } \
            }"
        );
    }
}
