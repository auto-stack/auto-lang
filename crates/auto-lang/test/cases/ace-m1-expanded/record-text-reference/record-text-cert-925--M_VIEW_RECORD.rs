// Independent PLAN-941 source reference; no emitted code.
struct Counter{value:i32}fn scale(x:&Counter,k:i32)->i32{x.value*k}fn main(){let x=Counter{value:6};println!("value:{}",scale(&x,7));}
