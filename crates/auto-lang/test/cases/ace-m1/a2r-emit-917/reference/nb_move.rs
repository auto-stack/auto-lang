// Hand-written reference for nb_move (Plan 917): move into a by-value callee.
#[derive(Clone, Debug)]
struct Counter { value: i64 }
fn consume(m: Counter) -> i64 { m.value }
fn main() -> i64 {
    let x = Counter { value: 20 };
    consume(x)
}
