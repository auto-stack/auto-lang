// Independent PLAN-941 query/instance reference; no emitted source.
fn pick(a:i32,b:bool)->i32{if b{return a} 0}
fn main(){let b=true;let a=7;println!("value:{}",pick(a,b));}
