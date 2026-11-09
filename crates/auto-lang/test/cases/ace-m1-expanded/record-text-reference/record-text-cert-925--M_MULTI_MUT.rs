// Independent PLAN-941 source reference; no emitted code.
struct Cell{v:i32}fn pair(a:&mut Cell,b:&mut Cell)->i32{a.v=1;b.v=2;a.v*10+b.v}
fn main(){let mut x=Cell{v:0};let mut y=Cell{v:0};println!("value:{}",pair(&mut x,&mut y));}
