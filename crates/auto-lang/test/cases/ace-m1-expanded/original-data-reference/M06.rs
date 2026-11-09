// PLAN-941 independent Rust reference: original source semantics, no emitted code.
struct P{v:i32}
fn update(q:&mut P)->String{q.v=5;"A中".to_owned()}
fn calc()->i32{let mut q=P{v:1};let result=update(&mut q);q.v*10+result.chars().count() as i32}
fn main(){println!("value:{}",calc());}
