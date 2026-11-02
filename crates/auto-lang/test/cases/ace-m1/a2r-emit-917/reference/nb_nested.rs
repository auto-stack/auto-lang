// Hand-written reference for nb_nested (Plan 917): nested direct call.
#[derive(Clone, Debug)]
struct Counter { value: i64 }
fn bump(x: &mut Counter, delta: i64) -> i64 { x.value += delta; x.value }
fn wrap(v: i64) -> i64 { v + 100 }
fn main() -> i64 {
    let mut x = Counter { value: 20 };
    wrap(bump(&mut x, 1))
}
