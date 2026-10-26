// Independent hand-written behavior reference; not an a2r golden.
fn main() {
    let observed: i32 = { fn fail()->Result<i32,String>{Err("bad".into())} match fail(){Ok(v)=>v,Err(e)=>e.len() as i32} };
    assert_eq!(observed, 3);
    println!("i32:{}", observed);
}
