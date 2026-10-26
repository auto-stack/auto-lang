// Independent hand-written behavior reference; not an a2r golden.
fn main() {
    let observed: i32 = { let x = 7; { let x = 10; assert_eq!(x, 10); } x };
    assert_eq!(observed, 7);
    println!("i32:{}", observed);
}
