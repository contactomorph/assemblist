use assemblist::assemblist;

#[test]
fn convert() {
    assemblist! {
        fn a(s: String).b(j: usize).c(ok: bool) -> usize {
            if ok { s.len() + j } else { 42 }
        }

        fn x(i: usize) -> usize {
            i + 5
        }
    }

    let _v = a("ee".to_string()).b(23).c(true);
}
