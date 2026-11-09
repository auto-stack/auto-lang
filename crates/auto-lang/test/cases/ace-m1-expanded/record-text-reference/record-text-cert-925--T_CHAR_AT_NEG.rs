// Independent PLAN-941 source reference; no emitted code.
fn main(){let s=String::from("abc");let chars:Vec<char>=s.chars().collect();let i=-1_i32;
if i<0||i as usize>=chars.len(){println!("trap:TextIndex");return;}println!("value:{}",chars[i as usize] as u32);}
