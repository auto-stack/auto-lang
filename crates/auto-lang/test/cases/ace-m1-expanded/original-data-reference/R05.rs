// PLAN-941 independent Rust reference: original source semantics, no emitted code.
struct Item{name:String,code:i32}
fn relay(value:Item)->Item{value}
fn calc()->i32{let a=Item{name:"x".to_owned(),code:41};let b=relay(a);b.code+b.name.chars().count() as i32}
fn main(){println!("value:{}",calc());}
