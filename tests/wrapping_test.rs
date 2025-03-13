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
