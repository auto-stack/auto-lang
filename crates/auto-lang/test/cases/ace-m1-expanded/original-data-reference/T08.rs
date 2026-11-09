// PLAN-941 independent Rust reference: original source semantics, no emitted code.
fn main(){let text="中";let start=std::hint::black_box(0_usize);let end=std::hint::black_box(2_usize);
if start>end || end>text.chars().count(){println!("trap:TextIndex");return}
let selected:String=text.chars().skip(start).take(end-start).collect();println!("value:{}",selected.chars().count());}
