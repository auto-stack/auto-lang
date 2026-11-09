// Independent PLAN-941 standard-library reference; no generated source.
fn main() { let s = "abc"; let index: i32 = -1; if index < 0 { println!("trap:TextIndex"); } else { match s.chars().nth(index as usize) { Some(c) => println!("value:{}", c as u32), None => println!("trap:TextIndex") } } }
