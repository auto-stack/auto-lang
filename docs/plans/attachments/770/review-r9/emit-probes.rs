use auto_ac_prototype::{source, verify};
fn main() {
 let dir=std::env::args().nth(1).unwrap();
 for ent in std::fs::read_dir(&dir).unwrap() {
  let p=ent.unwrap().path(); if p.extension().and_then(|s|s.to_str())!=Some("at") {continue;}
  let src=std::fs::read_to_string(&p).unwrap();
  let parts=source::compile_parts_with(&src,"r9",&[],source::SourceProfile::Text).unwrap();
  std::fs::write(p.with_extension("atom"),&parts.atom_text).unwrap();
  match verify::verify_text(parts.bundle) { Ok(_)=>println!("{}: Checked",p.display()),Err(ds)=>println!("{}: {:?}",p.display(),ds.iter().map(|d|&d.code).collect::<Vec<_>>()) }
 }
}
