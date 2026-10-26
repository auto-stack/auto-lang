// Independent hand-written behavior reference; not an a2r golden.
fn main() {
    let observed: i32 = { (-7_i32 / 3) * 10 + (-7_i32 % 3) };
    assert_eq!(observed, -21);
    println!("i32:{}", observed);
}
