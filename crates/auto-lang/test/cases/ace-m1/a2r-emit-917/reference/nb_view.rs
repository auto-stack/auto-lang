// Hand-written reference for nb_view (Plan 917): a read borrow does not move.
#[derive(Clone, Debug)]
struct Counter { value: i64 }
fn main() -> i64 {
    let x = Counter { value: 20 };
    let v: &Counter = &x;
    v.value + x.value
}
