// Independent hand-written behavior reference; not an a2r golden.
fn main() {
    let observed: i32 = { struct Item{name:String,code:i32} fn relay(x:Item)->Item{x} let b=relay(Item{name:"x".into(),code:41}); b.code+b.name.len() as i32 };
    assert_eq!(observed, 42);
    println!("i32:{}", observed);
}
