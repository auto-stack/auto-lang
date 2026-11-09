// PLAN-941 independent Rust reference: original source semantics, no emitted code.
struct P{v:i32}
impl P{fn eat(&self,a:String,k:i32,b:String)->i32{self.v+a.chars().count() as i32*10+k+b.chars().count() as i32}}
fn calc()->i32{let p=P{v:17};let b="Z".to_owned();let k=4;let a="A中".to_owned();p.eat(a,k,b)}
fn main(){println!("value:{}",calc());}
