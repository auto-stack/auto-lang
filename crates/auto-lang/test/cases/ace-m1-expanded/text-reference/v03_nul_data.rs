// Independent PLAN-941 standard-library reference; no generated source.
fn main() { let s = "a\0b"; println!("value:{}", s.chars().count() * 10 + s.len()); }
