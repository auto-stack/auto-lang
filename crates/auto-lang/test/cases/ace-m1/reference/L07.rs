// Independent hand-written behavior reference; not an a2r golden.
fn main() {
    let observed: i32 = { let mut xs=vec![20_i32,1];xs[1]=22;xs[0]+xs[1] };
    assert_eq!(observed, 42);
    println!("i32:{}", observed);
}
