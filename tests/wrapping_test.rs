use assemblist::assemblist;

#[test]
pub fn decompose_replacen() {
    assemblist! {
        pub fn replace_in<'a>(string: &'a str)
            .occurrences_of(pattern: &'a str)
            .with(to: &'a str)
            .at_most(n: usize)
            .times() -> String
        {
            string.replacen(pattern, to, n)
        }
    }

    let text = "Une souris verte qui courait dans l'herbe \
        Je l'attrape par la queue, je la montre à ces messieurs \
        Ces messieurs me disent 'trempez-la dans l'huile' \
        'Trempez-la dans l'eau, ça fera un escargot'";

    let result = replace_in(text)
        .occurrences_of("l'")
        .with("z'")
        .at_most(3)
        .times();

    assert_eq!(
        result,
        "Une souris verte qui courait dans z'herbe \
        Je z'attrape par la queue, je la montre à ces messieurs \
        Ces messieurs me disent 'trempez-la dans z'huile' \
        'Trempez-la dans l'eau, ça fera un escargot'");
}

#[test]
fn decompose_resize_with() {
    assemblist! {
        pub fn resize_vec<'a, T>(vec: &'a mut Vec<T>)
            .to(new_len: usize)
            .filling_with(f: impl FnMut() -> T)
        {
            Vec::<T>::resize_with(vec, new_len, f)
        }
    };

    let mut a = vec![12, 54, -9, 0, 3434];

    resize_vec(&mut a).to(8).filling_with(|| 42);

    assert_eq!(a, vec![12, 54, -9, 0, 3434, 42, 42, 42]);
}