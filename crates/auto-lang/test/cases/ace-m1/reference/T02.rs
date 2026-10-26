// Independent hand-written behavior reference; not an a2r golden.
fn main() {
    let observed: i32 = { let t: String = "A中😀".chars().skip(1).take(2).collect(); t.chars().count() as i32 * 10 + t.len() as i32 };
    assert_eq!(observed, 27);
    println!("i32:{}", observed);
}
