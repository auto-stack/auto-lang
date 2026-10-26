// Independent hand-written behavior reference; not an a2r golden.
fn main() {
    let observed: i32 = { 2 + 3 * 4 };
    assert_eq!(observed, 14);
    println!("i32:{}", observed);
}
