// Independent hand-written behavior reference; not an a2r golden.
fn main() {
    let observed: i32 = { struct Pair{left:i32,right:i32} let mut p=Pair{left:1,right:2}; p=Pair{left:p.right,right:p.left}; p.left*10+p.right };
    assert_eq!(observed, 21);
    println!("i32:{}", observed);
}
