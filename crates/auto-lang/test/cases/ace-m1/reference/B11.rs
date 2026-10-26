// Independent hand-written behavior reference; not an a2r golden.
fn main() {
    let observed: i32 = { fn sub(a: i32,b: i32)->i32 { a-b } sub(10,3) };
    assert_eq!(observed, 7);
    println!("i32:{}", observed);
}
