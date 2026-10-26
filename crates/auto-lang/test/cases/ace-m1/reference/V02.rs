// Independent hand-written behavior reference; not an a2r golden.
fn main() {
    let observed: i32 = { fn divide(a:i32,b:i32)->Result<i32,String>{if b==0{Err("bad".into())}else{Ok(a/b)}} match divide(84,2){Ok(v)=>v,Err(e)=>e.len() as i32} };
    assert_eq!(observed, 42);
    println!("i32:{}", observed);
}
