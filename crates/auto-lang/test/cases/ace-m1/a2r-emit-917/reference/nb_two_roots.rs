// Hand-written reference for nb_two_roots (Plan 917): two distinct &mut roots.
#[derive(Clone, Debug)]
struct Counter { value: i64 }
fn bump(x: &mut Counter, delta: i64) -> i64 { x.value += delta; x.value }
fn main() -> i64 {
    let mut x = Counter { value: 20 };
    let mut y = Counter { value: 30 };
    bump(&mut x, 1);
    bump(&mut y, 2);
    x.value + y.value
}
