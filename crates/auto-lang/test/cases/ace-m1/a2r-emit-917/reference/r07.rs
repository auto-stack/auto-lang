// Hand-written reference for corpus R07 (Plan 917).
// Independent Rust encoding of the frozen Auto program: bump mutates the
// caller's Counter through &mut; main observes now=21, x.value=21, total 42.
#[derive(Clone, Debug)]
struct Counter {
    value: i64,
}

fn bump(x: &mut Counter, delta: i64) -> i64 {
    x.value += delta;
    x.value
}

fn main() -> i64 {
    let mut x = Counter { value: 20 };
    let now = bump(&mut x, 1);
    now + x.value
}
