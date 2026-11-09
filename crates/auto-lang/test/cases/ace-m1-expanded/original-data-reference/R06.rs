// PLAN-941 independent Rust reference: original source semantics, no emitted code.
struct Pair{left:i32,right:i32}
fn calc()->i32{let mut p=Pair{left:1,right:2};p=Pair{left:p.right,right:p.left};p.left*10+p.right}
fn main(){println!("value:{}",calc());}
