// Independent hand-written behavior reference; not an a2r golden.
fn main() {
    let observed: i32 = { let s = "A中😀"; s.chars().count() as i32 * 10 + s.len() as i32 };
    assert_eq!(observed, 38);
    println!("i32:{}", observed);
}
