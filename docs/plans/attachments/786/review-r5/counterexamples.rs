#[path = "D:/autostack/.wt/lang-786/auto-lang/crates/auto-man/src/project_display_reader.rs"]
mod project_display_reader;
use project_display_reader::{read,ReadPath,CompatReason,ModelState};
fn main(){
 let a:Vec<_>=std::env::args().collect(); let n:usize=a[2].parse().unwrap();
 let body=match a[1].as_str(){
  "object"=>format!("extra: {}0{}\n","{x:".repeat(n),"}".repeat(n)),
  "nodes"=>format!("{}v: 0\n{}","node {{\n".replace("{{","{").repeat(n),"}\n".repeat(n)),
  "mixed"=>{let mut s=String::from("extra: ");for i in 0..n{s.push_str(if i%2==0{"["}else{"{x:"});}s.push('0');for i in (0..n).rev(){s.push(if i%2==0{']'}else{'}'});}s.push('\n');s},
  "flat"=>"extra: []\n".repeat(n),
  "quoted"=>format!("extra: \"{}\"\n","[".repeat(n)),
  "comment"=>format!("/*{}*/\n","{".repeat(n)),
  "args"=>format!("node({}0{}) {{}}\n","(".repeat(n),")".repeat(n)),
  _=>panic!("case")};
 let s=format!("name: \"n\"\ntitle: \"ok\"\n{body}");
 std::fs::write(&a[3],&s).unwrap();
 let r=read(&s);assert_eq!(r.name.as_deref(),Some("n"));assert_eq!(r.title.as_deref(),Some("ok"));assert_eq!(r.title_zh,None);
 let count=s.chars().filter(|c|matches!(c,'['|'{')).count();
 if count>64{assert_eq!(r.title_path,ReadPath::Compat(CompatReason::DeepNesting));}
 else{assert_eq!(r.title_path,ReadPath::Model(ModelState::Value));}
 println!("{}-{n} opens={count} PASS {:?}",a[1],r.title_path);
}
