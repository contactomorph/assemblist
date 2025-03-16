use std::format;

use assemblist::assemblist;

#[derive(PartialEq, Eq, Debug)]
pub struct Date(u32);

#[derive(PartialEq, Eq)]
pub struct Nat32(u32);

impl<'a> Into<Nat32> for &'a Date {
    fn into(self) -> Nat32 {
        Nat32(self.0)
    }
}

impl std::fmt::Debug for Nat32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[test]
fn convert_method_chain_for_movies() {
    assemblist! {
        pub fn consume<U>(items: Vec<U>)
            .to_feed<'b, V: std::fmt::Debug>(output: &'b mut Vec<V>)
            .and_return() -> Vec<(U, String)>
        where for<'a> &'a U: Into<V>
        {
            let mut res = Vec::<(U, String)>::new();
            for item in items {
                let item2: V = (&item).into();
                let text = format!("{:?}", item2);
                res.push((item, text));
                output.push(item2);
            }
            res
        }
    }

    let dates = vec![Date(20240227), Date(20230410)];
    let mut numbers = Vec::<Nat32>::new();
    let pairs = consume(dates).to_feed(&mut numbers).and_return();

    assert_eq!(numbers, vec![Nat32(20240227), Nat32(20230410)]);
    assert_eq!(
        pairs,
        vec![
            (Date(20240227), "20240227".to_string()),
            (Date(20230410), "20230410".to_string())
        ]
    );
}
