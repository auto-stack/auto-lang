// PLAN-941 independent Rust reference: original source semantics, no emitted code.
struct Counter{value:i32}
impl Counter{fn new(value:i32)->Self{Self{value}}fn read(&self)->i32{self.value}}
fn calc()->i32{Counter::new(42).read()}
fn main(){println!("value:{}",calc());}
