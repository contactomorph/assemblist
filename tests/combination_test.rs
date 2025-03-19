use core::future::IntoFuture;
use std::format;
use std::future::Future;
use tokio::time::{timeout, Duration};

use assemblist::assemblist;
use futures::future::FutureExt;

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
fn convert_method_chain_with_complex_logic() {
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

#[tokio::test]
async fn convert_method_chain_with_async_logic() {
    assemblist! {
        pub async fn wait_for<F: IntoFuture>(future: F)
            .then_chain_for<G: Future, L: FnOnce(F::Output) -> G>(continuation: L)
            .at_most(duration: Duration) -> Result<G::Output, tokio::time::error::Elapsed>
        {
            let map = (<F as IntoFuture>::into_future(future)).then(continuation);
            timeout(duration, map).await
        }
    };

    let future = async {
        tokio::time::sleep(Duration::from_millis(100)).await;
        "Hello"
    };

    async fn continuation(input: &'static str) -> usize {
        tokio::time::sleep(Duration::from_millis(200)).await;
        input.len()
    }

    let result = wait_for(future)
        .then_chain_for(continuation)
        .at_most(Duration::from_millis(1000))
        .await;

    match result {
        Ok(size) => assert_eq!(5, size),
        Err(_) => panic!("Waited too long"),
    }
}
