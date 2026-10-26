// Independent hand-written behavior reference; not an a2r golden.
fn main() {
    let observed: i32 = { let mut calls = 0; let b = false && { calls += 1; true }; if !b && calls == 0 { 42 } else { 1 } };
    assert_eq!(observed, 42);
    println!("i32:{}", observed);
}
