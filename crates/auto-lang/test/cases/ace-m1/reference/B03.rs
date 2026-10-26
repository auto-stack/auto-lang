// Independent hand-written behavior reference; not an a2r golden.
fn main() {
    let observed: i32 = { if !(3 >= 4) && 4 <= 4 && 3 != 4 && 3 == 3 { 42 } else { 1 } };
    assert_eq!(observed, 42);
    println!("i32:{}", observed);
}
