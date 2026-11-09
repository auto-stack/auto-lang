// Independent PLAN-941 source reference; no emitted code.
#[derive(Clone,Copy)]struct Counter{value:i32}fn bump(x:&mut Counter,d:i32)->i32{x.value+=d;x.value}
fn main(){let mut x=Counter{value:20};let snap=x;let now=bump(&mut x,1);println!("value:{}",snap.value*10+now);}
