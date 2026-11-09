// Independent PLAN-941 source reference; no emitted code.
fn tail(s:&str)->&str{&s[1..3]}fn main(){let a=String::from("wxyz");let t=tail(&a);println!("value:{}",t.chars().count());}
