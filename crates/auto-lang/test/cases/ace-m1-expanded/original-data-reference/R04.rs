// PLAN-941 independent Rust reference: original source semantics, no emitted code.
#[derive(Clone)]struct Item{name:String,_code:i32}
fn calc()->i32{let mut a=Item{name:"hello".to_owned(),_code:1};let b=a.clone();a.name="x".to_owned();b.name.chars().count() as i32*10+a.name.chars().count() as i32}
fn main(){println!("value:{}",calc());}
