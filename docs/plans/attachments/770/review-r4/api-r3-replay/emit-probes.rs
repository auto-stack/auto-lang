use auto_ac_prototype::{source, verify};
fn main() {
 let dir = std::env::args().nth(1).unwrap();
 for ent in std::fs::read_dir(&dir).unwrap() {
  let p=ent.unwrap().path();
  if p.extension().and_then(|s|s.to_str())!=Some("at") {continue;}
  let src=std::fs::read_to_string(&p).unwrap();
  let caps=if src.contains("mark_"){vec!["hir.test.trace.v1".to_string()]}else{vec![]};
  let parts=source::compile_parts_with(&src,"r3",&caps,source::SourceProfile::Text).unwrap();
  std::fs::write(p.with_extension("atom"), &parts.atom_text).unwrap();
  match verify::verify_text(parts.bundle) {
   Ok(_) => println!("{}: Checked",p.file_name().unwrap().to_string_lossy()),
   Err(ds) => println!("{}: {:?}",p.file_name().unwrap().to_string_lossy(),ds.iter().map(|d|&d.code).collect::<Vec<_>>())
  }
 }
 let src="fn main() int { let x Str = \"abc\"; if true { let n int = x.len() }; return 0 }";
 let mut b=source::compile_parts_with(src,"drop",&[],source::SourceProfile::Text).unwrap().bundle;
 let body=&mut b.modules[0].bodies[0];
 let owner=body.locals.iter().position(|l|l.id_text=="l0").unwrap() as u32;
 let mut changed=0;
 for block in &mut body.blocks { let mut in_block=false;
  for stmt in &mut block.stmts {
   if let auto_ac_prototype::hir::Stmt::Let{value,..}=stmt {
    if matches!(body.exprs[value.0 as usize].kind, auto_ac_prototype::hir::ExprKind::TextLen{..}) {
     in_block=true; changed+=1;
    }
   }
  }
  if in_block { block.stmts.push(auto_ac_prototype::hir::Stmt::Drop{local:auto_ac_prototype::hir::LocalId(owner)}); }
 }
 assert_eq!(changed,1);
 println!("q04 conditional Drop: verify={}",verify::verify_text(b.clone()).is_ok());
 std::fs::write(std::path::Path::new(&dir).join("q04-conditional-drop.atom"),auto_ac_prototype::descriptor::write_canonical(&b, auto_ac_prototype::text_descriptor()).unwrap()).unwrap();
 let src="fn unused(move s Str) int { return 0 }
fn main() int { return 0 }";
 let mut b=source::compile_parts_with(src,"cleanup",&[],source::SourceProfile::Text).unwrap().bundle;
 b.requires.clear(); assert!(b.requires.is_empty());
 match verify::verify_text(b.clone()) {
  Ok(c)=>println!("q05 cleanup-only: Checked runtime-needed={} capability-empty={}",c.requires_runtime_text(),auto_ac_prototype::native::capabilities_check(&c,&[]).is_ok()),
  Err(ds)=>println!("q05 rejected {:?}",ds.iter().map(|d|&d.code).collect::<Vec<_>>())
 }
 std::fs::write(std::path::Path::new(&dir).join("q05-cleanup-no-requires.atom"),auto_ac_prototype::descriptor::write_canonical(&b,auto_ac_prototype::text_descriptor()).unwrap()).unwrap();
 let src="fn main() int { if \"a\" == \"b\" { return 1 }; return 0 }";
 let mut b=source::compile_parts_with(src,"bad-ref",&[],source::SourceProfile::Text).unwrap().bundle;
 let mut changed=0;
 for e in &mut b.modules[0].bodies[0].exprs { if let auto_ac_prototype::hir::ExprKind::Binary{lhs,..}=&mut e.kind { lhs.0=u32::MAX; changed+=1; } }
 assert_eq!(changed,1);
 let result=std::panic::catch_unwind(||verify::verify_text(b));
 match result { Ok(Err(ds))=>println!("q06 controlled Err {:?}",ds.iter().map(|d|&d.code).collect::<Vec<_>>()), Ok(Ok(_))=>println!("q06 ACCEPTED"), Err(_)=>println!("q06 PANIC") };
}