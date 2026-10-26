// Independent hand-written behavior reference; not an a2r golden.
fn main() {
    let observed: i32 = { fn make(s:String)->impl Fn(i32)->i32 + Clone{move |n|s.len() as i32+n} let f=make("hello".into());let g=f.clone();f(10)+g(22) };
    assert_eq!(observed, 42);
    println!("i32:{}", observed);
}
