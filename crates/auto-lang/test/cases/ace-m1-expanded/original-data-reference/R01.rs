// PLAN-941 independent Rust reference: original source semantics, no emitted code.
#[derive(Clone,Copy)]struct Pair{left:i32,_right:i32}
fn calc()->i32{let mut p=Pair{left:1,_right:2};let q=p;p.left=9;q.left*10+p.left}
fn main(){println!("value:{}",calc());}
