// Independent PLAN-941 source reference; no emitted code.
struct Counter{value:i32}impl Counter{fn read(&self)->i32{self.value}}struct Gauge{value:i32}impl Gauge{fn read(&self)->i32{self.value+1}}fn read()->i32{7}
fn main(){let c=Counter{value:1};let g=Gauge{value:2};println!("value:{}",c.read()*100+g.read()*10+read());}
