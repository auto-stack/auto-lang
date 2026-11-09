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
fn lhs_a()->bool{let _x=trace_a();true}fn rhs_b()->bool{let _x=trace_b();true}
fn of_bool(x:bool)->i32{if x {1}else{0}}fn named2(a:i32,b:bool)->i32{if b {a}else{0}}
fn calc()->i32{let l=lhs_a()&&rhs_b();let mut v=false;std::hint::black_box(v);v=lhs_a()&&rhs_b();std::hint::black_box(v);
if lhs_a()&&rhs_b(){v=true;}else{v=false;}let mut n=0;while lhs_a()&&n<2{n=r_add(n,1);}
let p=of_bool(lhs_a()&&rhs_b());let b=lhs_a()&&rhs_b();let a=1;let q=named2(a,b);
if !l{return 1;}if !v{return 2;}if n!=2{return 3;}if p!=1{return 4;}if q!=1{return 5;}0}
fn main(){println!("value:{}",calc());}
