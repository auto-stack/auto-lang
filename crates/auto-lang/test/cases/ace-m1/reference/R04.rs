// Independent hand-written behavior reference; not an a2r golden.
fn main() {
    let observed: i32 = { #[derive(Clone)] struct Item{name:String,code:i32} let mut a=Item{name:"hello".into(),code:1}; let b=a.clone(); a.name="x".into(); assert_eq!(b.code,1); b.name.len() as i32*10+a.name.len() as i32 };
    assert_eq!(observed, 51);
    println!("i32:{}", observed);
}
