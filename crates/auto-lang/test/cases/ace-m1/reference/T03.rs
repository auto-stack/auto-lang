// Independent hand-written behavior reference; not an a2r golden.
fn main() {
    let observed: i32 = { let a = String::from("ab"); let b = a.clone(); let mut c = b; c.push_str("中"); a.chars().count() as i32 * 10 + c.chars().count() as i32 };
    assert_eq!(observed, 23);
    println!("i32:{}", observed);
}
