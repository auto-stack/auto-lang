// Independent PLAN-941 source reference; no emitted code.
fn main(){let a=String::from("ab");let b=a.clone();let mut c=b;c.push('中');println!("value:{}",a.chars().count()*10+c.chars().count());}
