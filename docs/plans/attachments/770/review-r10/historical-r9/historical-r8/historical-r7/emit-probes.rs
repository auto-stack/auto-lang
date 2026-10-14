use auto_ac_prototype::{source, verify};
fn main() {
 let dir=std::env::args().nth(1).unwrap();
 for ent in std::fs::read_dir(&dir).unwrap() {
  let p=ent.unwrap().path(); if p.extension().and_then(|s|s.to_str())!=Some("at") {continue;}
  let src=std::fs::read_to_string(&p).unwrap();
  let parts=source::compile_parts_with(&src,"r7",&[],source::SourceProfile::Text).unwrap();
  std::fs::write(p.with_extension("atom"),&parts.atom_text).unwrap();
  match verify::verify_text(parts.bundle) { Ok(_)=>println!("{}: Checked",p.display()),Err(ds)=>println!("{}: {:?}",p.display(),ds.iter().map(|d|&d.code).collect::<Vec<_>>()) }
 }
 let src="fn main() int { let x Str=\"abc\"; var v str=x.to(str); var i int=0; for i<1 { let n int=v.len(); let s Str=\"def\"; v=s.to(str); break }; return 0 }";
 let mut b=source::compile_parts_with(src,"break-only",&[],source::SourceProfile::Text).unwrap().bundle;
 let body=&mut b.modules[0].bodies[0];
 let entry=body.entry.0 as usize;
 let guard=body.blocks[entry].stmts.iter().find_map(|s|if let auto_ac_prototype::hir::Stmt::Loop{body,..}=s{Some(*body)}else{None}).unwrap();
 let actual=body.blocks[guard.0 as usize].stmts.iter().find_map(|s|if let auto_ac_prototype::hir::Stmt::If{then,..}=s{Some(*then)}else{None}).unwrap();
 let mut changed=0;
 for s in &mut body.blocks[entry].stmts {if let auto_ac_prototype::hir::Stmt::Loop{body,..}=s{*body=actual;changed+=1;}}
 assert_eq!(changed,1);assert_ne!(guard,actual);
 let user_block=body.blocks[actual.0 as usize].clone(); let mut root_block=body.blocks[entry].clone(); for s in &mut root_block.stmts { if let auto_ac_prototype::hir::Stmt::Loop{body,..}=s{*body=auto_ac_prototype::hir::BlockId(1)} }; body.blocks=vec![root_block,user_block]; body.entry=auto_ac_prototype::hir::BlockId(0);
 let mut used=std::collections::BTreeSet::new(); let mut todo=vec![];
 for block in &body.blocks {for stmt in &block.stmts {match stmt {auto_ac_prototype::hir::Stmt::Let{value,..}|auto_ac_prototype::hir::Stmt::Assign{value,..}|auto_ac_prototype::hir::Stmt::Return{value}=>todo.push(value.0), _=>{}}}}
 while let Some(id)=todo.pop(){if !used.insert(id){continue};match &body.exprs[id as usize].kind {auto_ac_prototype::hir::ExprKind::TextToBorrowed{operand}|auto_ac_prototype::hir::ExprKind::TextLen{operand}=>todo.push(operand.0),auto_ac_prototype::hir::ExprKind::Binary{lhs,rhs,..}=>{todo.push(lhs.0);todo.push(rhs.0)},_=>{}}}
 let map:std::collections::BTreeMap<u32,u32>=used.iter().enumerate().map(|(n,id)|(*id,n as u32)).collect();
 let mut exprs:Vec<_>=used.iter().map(|id|body.exprs[*id as usize].clone()).collect();
 for e in &mut exprs {match &mut e.kind{auto_ac_prototype::hir::ExprKind::TextToBorrowed{operand}|auto_ac_prototype::hir::ExprKind::TextLen{operand}=>operand.0=map[&operand.0],auto_ac_prototype::hir::ExprKind::Binary{lhs,rhs,..}=>{lhs.0=map[&lhs.0];rhs.0=map[&rhs.0]},_=>{}}}
 for block in &mut body.blocks {for stmt in &mut block.stmts {match stmt {auto_ac_prototype::hir::Stmt::Let{value,..}|auto_ac_prototype::hir::Stmt::Assign{value,..}|auto_ac_prototype::hir::Stmt::Return{value}=>value.0=map[&value.0],_=>{}}}}
 body.exprs=exprs;
 std::fs::write(std::path::Path::new(&dir).join("m01-direct-break-only.atom"),auto_ac_prototype::descriptor::write_canonical(&b,auto_ac_prototype::text_descriptor()).unwrap()).unwrap();
 let mut control=b.clone(); let cb=&mut control.modules[0].bodies[0]; for e in &mut cb.exprs {if let auto_ac_prototype::hir::ExprKind::ReadLocal{local}=&mut e.kind {if local.0==4 {local.0=0}}};
 std::fs::write(std::path::Path::new(&dir).join("m02-direct-break-live-owner-control.atom"),auto_ac_prototype::descriptor::write_canonical(&control,auto_ac_prototype::text_descriptor()).unwrap()).unwrap();
 println!("m02 view source redirected s->x: verify={}",verify::verify_text(control).is_ok());
 match verify::verify_text(b){Ok(_)=>println!("m01 actual mutation Loop.body {:?}->{:?}: Checked",guard,actual),Err(ds)=>println!("m01 actual mutation Loop.body {:?}->{:?}: {:?}",guard,actual,ds.iter().map(|d|&d.code).collect::<Vec<_>>())}}
