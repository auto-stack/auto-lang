// Independent hand-written behavior reference; not an a2r golden.
fn main() {
    let observed: i32 = { let mut r:Result<i32,String>=Err("old".into());assert!(r.is_err());r=Ok(42);match r{Ok(v)=>v,Err(e)=>e.len() as i32} };
    assert_eq!(observed, 42);
    println!("i32:{}", observed);
}
