// Independent PLAN-941 standard-library reference; no generated source.
fn main() { let s = "héllo"; match s.chars().nth(1) { Some(c) => println!("value:{}", c as u32), None => println!("trap:TextIndex") } }
