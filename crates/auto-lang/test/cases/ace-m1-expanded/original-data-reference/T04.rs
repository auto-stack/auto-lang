// PLAN-941 independent Rust reference: original source semantics, no emitted code.
fn calc()->i32{let character='😀';character as u32 as i32}
fn main(){println!("value:{}",calc());}
