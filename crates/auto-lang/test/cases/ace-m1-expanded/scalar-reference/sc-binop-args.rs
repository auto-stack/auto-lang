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
fn add(a:i32,b:i32)->i32 {r_add(a,b)} fn calc()->i32 {
if add(r_sub(1,2),r_mul(3,4))!=11{return 1;}
let b=r_div(10,2);let a=r_rem(7,4);if add(a,b)!=8{return 2;}
if add(add(1,2),add(r_rem(3,2),r_sub(4,1)))!=7{return 3;}
if add(r_add(1,2),3)!=6{return 4;}
let x=r_mul(r_add(1,2),r_sub(10,7));if x!=9{return 5;}0}
fn main(){println!("value:{}",calc());}
