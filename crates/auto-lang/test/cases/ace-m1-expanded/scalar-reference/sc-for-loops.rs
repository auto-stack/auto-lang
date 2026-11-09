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
fn calc()->i32 {let mut zero=0;while 1>2{zero=r_add(zero,1);}if zero!=0{return 1;}
let mut total=0;let mut i=0;while i<3{let mut j=0;while j<3{j=r_add(j,1);if j==2{continue;}if j==3{break;}total=r_add(total,10);}i=r_add(i,1);}if total!=30{return 2;}0}
fn main(){println!("value:{}",calc());}
