// PLAN-941 independent Rust reference: original source semantics, no emitted code.
fn calc()->i32{let text="A中😀";text.chars().count() as i32*10+text.len() as i32}
fn main(){println!("value:{}",calc());}
