// PLAN-941 independent Rust reference: original source semantics, no emitted code.
struct R{n:i32}
impl R{fn down(k:i32)->i32{if k<=0{42}else{Self::down(k-1)}}fn vdown(&self,k:i32)->i32{if k<=0{self.n}else{self.vdown(k-1)}}}
fn calc()->i32{let r=R{n:42};R::down(9)+r.vdown(9)}
fn main(){println!("value:{}",calc());}
