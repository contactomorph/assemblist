use std::marker::PhantomData;

use syn::parse::End;

enum CumulativeAlt<'a, A: ?Sized> {
    End,
    Prev {
        item: &'a A,
        prev: &'a CumulativeList<'a, A>,
    },
}

/**
 * A linked list of stack allocated items
 */
pub struct CumulativeList<'a, A: ?Sized> {
    rank: usize,
    alt: CumulativeAlt<'a, A>,
}

pub type CumulativeListRef<'a, A> = &'a CumulativeList<'a, A>;

impl<'a, A: ?Sized> CumulativeList<'a, A> {
    /**
     * Get list size.
     */
    pub fn size(&self) -> usize {
        self.rank
    }

    /**
     * Get previous list.
     */
    pub fn previous(&self) -> Option<&CumulativeList<'a, A>> {
        match &self.alt {
            CumulativeAlt::End => None,
            CumulativeAlt::Prev { prev, .. } => Some(*prev),
        }
    }

    /**
     * Get current item.
     */
    pub fn item(&self) -> Option<&A> {
        match &self.alt {
            CumulativeAlt::End => None,
            CumulativeAlt::Prev { item, .. } => Some(item),
        }
    }

    pub fn item_and_previous(&self) -> Option<(&A, &CumulativeList<'a, A>)> {
        match &self.alt {
            CumulativeAlt::End => None,
            CumulativeAlt::Prev { item, prev, .. } => Some((item, *prev)),
        }
    }
    
    pub fn concat(&'a self, item: &'a A) -> CumulativeList<'a, A> {
        CumulativeList {
            rank: self.size() + 1,
            alt: CumulativeAlt::Prev { item, prev: self },
        }
    }
}

pub struct CumulativeIterator<'a, A: ?Sized> {
    ptr: &'a CumulativeList<'a, A>,
}

impl<'a, A: ?Sized> Iterator for CumulativeIterator<'a, A> {
    type Item = &'a A;
    fn next(&mut self) -> Option<Self::Item> {
        match &self.ptr.alt {
            CumulativeAlt::End => None,
            CumulativeAlt::Prev { item, prev } => {
                self.ptr = prev;
                Some(item)
            }
        }
    }
}

impl<'a, A: ?Sized> IntoIterator for &'a CumulativeList<'a, A> {
    type Item = &'a A;
    type IntoIter = CumulativeIterator<'a, A>;
    fn into_iter(self) -> Self::IntoIter {
        CumulativeIterator::<'a, A> { ptr: self }
    }
}

pub type CumulativeFnHandlerRef<'h, S, I, A, O> = &'h CumulativeFnHandler<'h, S, I, A, O>;

pub type CumulativeLambdaRef<S, I, A, O> = for<'h, 's, 'a> fn(
    CumulativeFnHandlerRef<'h, S, I, A, O>,
    &'s mut S,
    I,
    CumulativeListRef<'a, A>,
) -> O;

pub struct CumulativeFnHandler<'a, S, I, A: ?Sized, O> {
    f: CumulativeLambdaRef<S, I, A, O>,
    agg: &'a CumulativeList<'a, A>,
}

pub struct CumulativeFn<S, I> {
    _ph: PhantomData<(S, I)>,
}

pub struct CumulativeFnCaller<S, I, A: ?Sized, O> {
    f: CumulativeLambdaRef<S, I, A, O>,
}

impl<S, I> CumulativeFn<S, I> {
    fn make<A: ?Sized, O>(f: CumulativeLambdaRef<S, I, A, O>) -> CumulativeFnCaller<S, I, A, O> {
        CumulativeFnCaller { f }
    }
}

impl<S, I, A: ?Sized, O> CumulativeFnCaller<S, I, A, O> {
    fn call(&self, state: &mut S, input: I) -> O {
        let agg: CumulativeList<'_, A> = CumulativeList {
            rank: 0,
            alt: CumulativeAlt::End,
        };
        let handler = CumulativeFnHandler::<'_, S, I, A, O> {
            f: self.f,
            agg: &agg,
        };
        (self.f)(&handler, state, input, &agg)
    }
}

impl<'a, S, I, A: ?Sized, O> CumulativeFnHandler<'a, S, I, A, O> {
    fn call(&'a self, state: &mut S, input: I, item: &'a A) -> O {
        let agg: CumulativeList<'_, A> = self.agg.concat(item);
        let handler = CumulativeFnHandler::<'_, S, I, A, O> {
            f: self.f,
            agg: &agg,
        };
        (self.f)(&handler, state, input, &agg)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CumulativeFn, CumulativeFnHandler, CumulativeFnHandlerRef, CumulativeIterator,
        CumulativeList, CumulativeListRef,
    };

    #[test]
    fn cummulative() {
        let f = CumulativeFn::<Vec<String>, (&Vec<&'static str>, usize)>::make(
            |handler, state, (names, index), aggregation| {
                if index >= names.len() {
                    return;
                }
                let inverted_name: String = names[index].to_string().chars().rev().collect();
                let mut text = String::new();
                let mut first = true;
                for a in aggregation.into_iter() {
                    if first {
                        first = false
                    } else {
                        text.push(' ')
                    }
                    text.push_str(a)
                }
                state.push(text);
                handler.call(state, (names, index + 1), inverted_name.as_str());
                state.push(inverted_name);
            },
        );

        let mut state = Vec::<String>::new();
        let names = vec![
            "Nous",
            "autres",
            "civilisations",
            "nous",
            "savons",
            "maintenant",
            "que",
            "nous",
            "sommes",
            "mortelles",
        ];
        f.call(&mut state, (&names, 0));

        assert_eq!(20, state.len());
        assert_eq!("", state[0].as_str());
        assert_eq!("suoN", state[1].as_str());
        assert_eq!("sertua suoN", state[2].as_str());
        assert_eq!("snoitasilivic sertua suoN", state[3].as_str());
        assert_eq!("selletrom", state[10].as_str());
        assert_eq!("semmos", state[11].as_str());
        assert_eq!("suon", state[12].as_str());
        assert_eq!("snovas", state[13].as_str());
    }
}
