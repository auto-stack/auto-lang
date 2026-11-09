// Independent PLAN-941 source reference; no emitted code.
struct Counter{value:i32}impl Counter{fn new(n:i32)->Self{Self{value:n}}fn read(&self)->i32{self.value}}fn main(){let x=Counter::new(42);println!("value:{}",x.read());}
