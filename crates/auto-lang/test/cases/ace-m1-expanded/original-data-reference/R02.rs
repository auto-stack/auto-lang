// PLAN-941 independent Rust reference: original source semantics, no emitted code.
struct Inner{value:i32}struct Outer{inner:Inner,extra:i32}
fn calc()->i32{let mut p=Outer{inner:Inner{value:3},extra:2};p.inner.value=7;p.inner.value+p.extra}
fn main(){println!("value:{}",calc());}
