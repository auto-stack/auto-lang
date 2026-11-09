// PLAN-941 independent Rust reference: original source semantics, no emitted code.
struct P{v:i32}
fn mixed(ro:&P,a:&mut P,k:i32,b:&mut P)->i32{a.v+=k;b.v+=2*k;ro.v*100+a.v*10+b.v}
fn calc()->i32{let ro=P{v:1};let mut a=P{v:2};let mut b=P{v:3};let result=mixed(&ro,&mut a,1,&mut b);
if ro.v*100+a.v*10+b.v-result!=0{return 60}if ro.v!=1{return 70+ro.v}result}
fn main(){println!("value:{}",calc());}
