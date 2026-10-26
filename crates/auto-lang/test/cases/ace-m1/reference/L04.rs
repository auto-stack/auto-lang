// Independent hand-written behavior reference; not an a2r golden.
fn main() {
    let observed: i32 = { struct Item{name:String,code:i32} let mut xs=Vec::new();xs.push(Item{name:"hello".into(),code:37});xs[0].name.len() as i32+xs[0].code };
    assert_eq!(observed, 42);
    println!("i32:{}", observed);
}
