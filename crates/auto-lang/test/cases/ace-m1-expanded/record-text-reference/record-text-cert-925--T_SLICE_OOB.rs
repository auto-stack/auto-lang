// Independent PLAN-941 source reference; no emitted code.
fn main(){let s=String::from("héllo");let chars:Vec<char>=s.chars().collect();let start=0_usize;let end=6_usize;
if start>end||end>chars.len(){println!("trap:TextIndex");return;}let t:String=chars[start..end].iter().collect();println!("value:{}",t.chars().count());}
