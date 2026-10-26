// Independent hand-written behavior reference; not an a2r golden.
fn main() {
    let observed: i32 = { fn fact(n: i32) -> i32 { if n <= 1 { 1 } else { n * fact(n - 1) } } fact(5) };
    assert_eq!(observed, 120);
    println!("i32:{}", observed);
}
