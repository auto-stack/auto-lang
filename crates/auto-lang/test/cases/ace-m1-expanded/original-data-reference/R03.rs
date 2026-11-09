// PLAN-941 independent Rust reference: original source semantics, no emitted code.
struct Pair{left:i32,right:i32}
fn calc()->i32{let p=Pair{right:3,left:2};p.left*10+p.right}
fn main(){println!("value:{}",calc());}
