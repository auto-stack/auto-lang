// Independent hand-written behavior reference; not an a2r golden.
fn main() {
    let observed: i32 = { let mut xs=vec![10_i32,30];xs.insert(1,20);xs.remove(1);xs[0]+xs[1]+xs.len() as i32 };
    assert_eq!(observed, 42);
    println!("i32:{}", observed);
}
