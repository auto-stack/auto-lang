// Independent hand-written behavior reference; not an a2r golden.
fn main() {
    let observed: i32 = { let mut xs:Vec<i32>=Vec::new();assert_eq!(xs.len(),0);xs.push(42);xs.clear();xs.len() as i32 };
    assert_eq!(observed, 0);
    println!("i32:{}", observed);
}
