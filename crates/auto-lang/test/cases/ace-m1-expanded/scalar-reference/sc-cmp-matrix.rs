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
fn calc()->i32 {let a=r_sub(0,7);
if r_div(a,3)!=-2{return 1;}if r_rem(a,3)!=-1{return 2;}
if r_div(7,-3)!=-2{return 3;}if r_rem(7,-3)!=1{return 4;}
if r_div(-7,-3)!=2{return 5;}if r_rem(-7,-3)!=-1{return 6;}
if r_min()>=r_max(){return 7;}if r_min()>=r_max(){return 8;}
if r_min()>r_min(){return 9;}if r_max()<=r_min(){return 10;}
let t=true;let f=false;if t==f{return 11;}if t==f{return 12;}
if !(t&&!f){return 13;}if 1==2||2!=2{return 14;}
if r_add(1,r_mul(2,3))!=7{return 15;}
if !(r_add(2,3)<6&&4>2){return 16;}
if false||true&&false{return 17;}if !(1<2){return 18;}
let neg=r_sub(0,r_add(a,3));if neg!=4{return 19;}
let nn=r_sub(0,-5);if nn!=5{return 20;}
if !!t!=t{return 21;}if !(r_rem(10,4)==2&&r_div(9,2)==4){return 22;}0}
fn main(){println!("value:{}",calc());}
