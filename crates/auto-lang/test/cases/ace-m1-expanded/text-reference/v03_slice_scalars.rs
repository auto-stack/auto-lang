// Independent PLAN-941 standard-library reference; no generated source.
fn main() { let s = "héllo"; let (begin,end) = (2,5); if begin > end || end > s.chars().count() { println!("trap:TextIndex"); } else { println!("value:{}", s.chars().skip(begin).take(end-begin).count()); } }
