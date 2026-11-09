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
fn f_false()->bool{let _x=trace_a();2<1}fn t_true()->bool{let _x=trace_a();1<2}
fn rhs_b(x:bool)->bool{if x{let _y=trace_b();true}else{false}}
fn calc()->i32{let r1=f_false()&&rhs_b(true);if r1{return 1;}let r2=t_true()||rhs_b(false);if !r2{return 2;}
let r3=t_true()&&rhs_b(true);if !r3{return 3;}let r4=f_false()||rhs_b(false);if r4{return 4;}0}
fn main(){println!("value:{}",calc());}
