// Independent hand-written behavior reference; not an a2r golden.
fn main() {
    let observed: i32 = { let base=40;let f=|n:i32|base+n;f(2) };
    assert_eq!(observed, 42);
    println!("i32:{}", observed);
}
