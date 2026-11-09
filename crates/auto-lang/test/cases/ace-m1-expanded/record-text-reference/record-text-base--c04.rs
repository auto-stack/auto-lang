// Independent PLAN-941 source reference; no emitted code.
struct Counter{value:i32}impl Counter{fn new(n:i32)->Self{Self{value:n}}fn read(&self)->i32{self.value}}fn main(){let c=Counter::new(40);let s=String::from("x");println!("value:{}",c.read()+s.chars().count() as i32+1);}
