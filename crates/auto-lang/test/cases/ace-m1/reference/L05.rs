// Independent hand-written behavior reference; not an a2r golden.
fn main() {
    let observed: i32 = { struct Item{name:String,code:i32} let mut xs=vec![Item{name:"hello".into(),code:37}];let x=xs.remove(0);x.name.len() as i32+x.code+xs.len() as i32 };
    assert_eq!(observed, 42);
    println!("i32:{}", observed);
}
