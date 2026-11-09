// Independent PLAN-941 standard-library reference; no generated source.
fn main() { match char::from_u32(20013) { Some(c) => println!("value:{}", c as u32), None => println!("trap:InvalidScalar") } }
