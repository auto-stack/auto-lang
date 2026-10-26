// Independent hand-written behavior reference; not an a2r golden.
fn main() {
    let observed: i32 = { #[derive(Clone,Copy)] struct Pair { left: i32, right: i32 } let mut p=Pair{left:1,right:2}; let q=p; p.left=9; assert_eq!(q.right,2); q.left*10+p.left };
    assert_eq!(observed, 19);
    println!("i32:{}", observed);
}
