// Independent PLAN-941 source reference; no emitted code.
fn dupe(s:&str)->String{s.to_owned()}fn main(){let a=String::from("ab");let b=dupe(&a);println!("value:{}",b.chars().count()*10+a.chars().count());}
