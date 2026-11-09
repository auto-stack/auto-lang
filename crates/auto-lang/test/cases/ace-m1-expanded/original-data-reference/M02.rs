// PLAN-941 independent Rust reference: original source semantics, no emitted code.
#[derive(Clone)] struct Token{text:String,kind:i32,line:i32,at:i32,len:i32}
impl Token{fn ident(text:String,line:i32,at:i32)->Self{let len=text.chars().count() as i32;Self{text,kind:1,line,at,len}}fn is_ident(&self)->bool{self.kind==1}}
fn weigh(t:Token)->i32{t.len+t.line+t.at}
fn calc()->i32{let a=Token::ident("keyword".to_owned(),3,12);if !a.is_ident(){return 100}if a.len!=7{return 101}
let mut b=a.clone();b.at=40;if a.at!=12{return 102}weigh(b)+a.text.chars().count() as i32}
fn main(){println!("value:{}",calc());}
