// PLAN-941 independently authored scalar reference, never emitted source.
// Checked arithmetic is implemented separately using Rust standard operations.
#![allow(dead_code)]
fn r_trap(name:&str)->!{println!("trap:{}",name);std::process::exit(0)}
fn r_add(a:i32,b:i32)->i32{a.checked_add(b).unwrap_or_else(||r_trap("Overflow"))}
fn r_sub(a:i32,b:i32)->i32{a.checked_sub(b).unwrap_or_else(||r_trap("Overflow"))}
fn r_mul(a:i32,b:i32)->i32{a.checked_mul(b).unwrap_or_else(||r_trap("Overflow"))}
fn r_div(a:i32,b:i32)->i32{if b==0{r_trap("DivByZero")}a.checked_div(b).unwrap_or_else(||r_trap("DivOverflow"))}
fn r_rem(a:i32,b:i32)->i32{if b==0{r_trap("RemByZero")}a.checked_rem(b).unwrap_or_else(||r_trap("RemOverflow"))}
fn r_min()->i32{i32::MIN}fn r_max()->i32{i32::MAX}
fn trace_a()->i32{eprintln!("mark_a");1}fn trace_b()->i32{eprintln!("mark_b");2}
fn calc()->i32 {let a=1;let b=r_sub(a,2);let mut c=r_mul(r_add(b,4),3);if c!=9{return 1;}while c>0{c=r_sub(c,3);}if c!=0{return 2;}0}
fn main(){println!("value:{}",calc());}
