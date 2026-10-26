// Independent hand-written behavior reference; not an a2r golden.
fn main() {
    let observed: i32 = { -(-7_i32) + (-9) };
    assert_eq!(observed, -2);
    println!("i32:{}", observed);
}
