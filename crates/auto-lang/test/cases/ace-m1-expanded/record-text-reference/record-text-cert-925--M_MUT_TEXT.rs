// Independent PLAN-941 source reference; no emitted code.
fn twice(s:&mut str)->usize{let t=format!("{}{}",s,s);t.chars().count()}fn main(){let mut a=String::from("ab");println!("value:{}",twice(&mut a));}
