// Independent PLAN-941 source reference; no emitted code.
fn main(){match char::from_u32(55296){Some(c)=>println!("value:{}",c as u32),None=>println!("trap:InvalidScalar")}}
