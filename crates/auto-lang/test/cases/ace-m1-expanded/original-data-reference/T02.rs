// PLAN-941 independent Rust reference: original source semantics, no emitted code.
fn calc()->i32{let text="A中😀";let selected:String=text.chars().skip(1).take(2).collect();selected.chars().count() as i32*10+selected.len() as i32}
fn main(){println!("value:{}",calc());}
