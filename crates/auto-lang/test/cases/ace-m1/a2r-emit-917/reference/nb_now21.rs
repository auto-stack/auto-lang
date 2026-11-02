// Hand-written reference for nb_now21 (Plan 917): the callee-returned projection.
#[derive(Clone, Debug)]
struct Counter { value: i64 }
fn bump(x: &mut Counter, delta: i64) -> i64 { x.value += delta; x.value }
fn main() -> i64 {
    let mut x = Counter { value: 20 };
    bump(&mut x, 1)
}
