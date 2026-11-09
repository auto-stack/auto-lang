// Independent PLAN-941 standard-library reference; no generated source.
fn main() { let a = String::from("héllo"); let b = a.clone(); let c = a + &b; println!("value:{}", c.chars().count()*10+c.len()); }
