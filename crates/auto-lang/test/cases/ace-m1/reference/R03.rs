// Independent hand-written behavior reference; not an a2r golden.
fn main() {
    let observed: i32 = { struct Pair{left:i32,right:i32} let p=Pair{right:3,left:2}; p.left*10+p.right };
    assert_eq!(observed, 23);
    println!("i32:{}", observed);
}
