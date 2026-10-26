// Independent hand-written behavior reference; not an a2r golden.
fn main() {
    let observed: i32 = { struct Counter{value:i32} fn bump(x:&mut Counter,d:i32)->i32{x.value+=d;x.value} let mut x=Counter{value:20}; let now=bump(&mut x,1); now+x.value };
    assert_eq!(observed, 42);
    println!("i32:{}", observed);
}
