// Independent PLAN-941 query/instance reference; no emitted source.
fn mark_a()->i32{eprintln!("mark_a");1}
fn mark_b()->i32{eprintln!("mark_b");2}
fn sub(a:i32,b:i32)->i32{a*3+b*2}
fn main(){let x=mark_a();let b=mark_b();let a=x;println!("value:{}",sub(a,b));}
