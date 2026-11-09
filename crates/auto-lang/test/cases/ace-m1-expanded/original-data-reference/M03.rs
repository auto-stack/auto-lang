// PLAN-941 independent Rust reference: original source semantics, no emitted code.
struct P{v:i32}
impl P{fn mix(&self,q:&mut P,d:i32)->i32{q.v+=d;self.v*10+q.v}}
fn calc()->i32{let p=P{v:7};let mut q=P{v:2};let result=p.mix(&mut q,3);if p.v!=7{return 90+p.v}if q.v!=5{return 80+q.v}result}
fn main(){println!("value:{}",calc());}
