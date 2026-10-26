// Independent hand-written behavior reference; not an a2r golden.
fn main() {
    let observed: i32 = { fn maybe(x:i32)->Option<i32>{if x>0{Some(x)}else{None}} match maybe(42){Some(v)=>v,None=>1} };
    assert_eq!(observed, 42);
    println!("i32:{}", observed);
}
