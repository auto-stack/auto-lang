use ac_syntax_rt::{snapshot::{Snapshot,PageStore},lexer::CompiledScanner,parse::parse_snapshot,program::Budget};
use ac_syntax_artifact_adapter::SyntaxArtifactStore;
use std::rc::Rc;
const ROOT:&str="D:/autostack/.wt/lang-787/auto-lang";
#[test]
fn producer_trees_remain_readable_after_subtree_admission(){
 let mut failures=Vec::new();let mut files=0;let mut decls=0;
 for profile in ["auto-core","interface-mini-v1"]{
 let p=ac_syntax_grammar::compile(&std::fs::read_to_string(format!("{ROOT}/experimental/ac-syntax/grammars/{profile}.at")).unwrap()).unwrap();
 let cs=CompiledScanner::compile(&p).unwrap();
 let mut cases=Vec::new();
 if profile=="auto-core" {for sub in ["valid","invalid"]{for e in std::fs::read_dir(format!("{ROOT}/experimental/ac-core/fixtures/source/{sub}")).unwrap(){let f=e.unwrap().path();if f.extension().is_some_and(|x|x=="at"){cases.push((format!("{sub}/{}",f.file_name().unwrap().to_string_lossy()),std::fs::read_to_string(f).unwrap()));}}}}
 else{for (i,s) in ["api f() -> i32;","api f( -> i32; api g() -> i32;","api f(x: ) -> i32;","note \"abc\";","note \"abc","api f() -> i32","; ? ; api g() -> i32;","/*x*/ api f() -> i32;", "api f() -> i32; api", "api f() -> i32; /* open"].iter().enumerate(){cases.push((format!("mini{i}"),s.to_string()));}}
 let root=std::env::temp_dir().join(format!("ac787-r5-matrix-{profile}-{}",std::process::id()));
 let _=std::fs::remove_dir_all(&root);let sas=SyntaxArtifactStore::open(&root,"matrix").unwrap();
 for (name,text) in cases {files+=1;let s=Snapshot::from_text(&Rc::new(PageStore::new()),&text);let out=parse_snapshot(&p,&cs,&s,&Budget::default()).unwrap();let t=sas.put_tokens(&p,&s,&out.tokens).unwrap();
 if let Err(e)=sas.put_cst(&p,&s,&out.tree,&t){failures.push(format!("{profile}/{name} PUT: {e:?}"));continue;}
 let v=sas.open_cst(&p,&s).unwrap().unwrap();assert_eq!(v.decl_count(),out.tree.decls.len());
 for i in 0..v.decl_count(){decls+=1;if let Err(e)=v.decl_subtree(i){failures.push(format!("{profile}/{name} decl{i}: {e:?}"));}}
 }
 let _=std::fs::remove_dir_all(root);
 }
 eprintln!("producer matrix: files={files} decls={decls} failures={}",failures.len());
 for e in &failures{eprintln!("{e}");}assert!(failures.is_empty());
}

#[test]
fn empty_call_argument_rule_is_accepted_by_recovery_admission(){
 let p=ac_syntax_grammar::compile(&std::fs::read_to_string(format!("{ROOT}/experimental/ac-syntax/grammars/auto-core.at")).unwrap()).unwrap();let cs=CompiledScanner::compile(&p).unwrap();
 for (tag,txt) in [("one-arg","fn main() int { return f(1) }\n"),("no-args","fn main() int { return f() }\n")]{
 assert!(auto_ac_prototype::source::parse_program(txt).is_ok(),"existing core syntax oracle");
 let s=Snapshot::from_text(&Rc::new(PageStore::new()),txt);let out=parse_snapshot(&p,&cs,&s,&Budget::default()).unwrap();
 for blk in &out.tree.blocks{for nd in &blk.nodes{if nd.len==0&&nd.kind>=ac_syntax_rt::program::RULE_KIND_BASE{let r=(nd.kind-ac_syntax_rt::program::RULE_KIND_BASE)as usize;if let Some(rule)=p.rules.get(r){eprintln!("{tag} zero rule={} start={} decl={}",rule.name,nd.start,nd.decl);}}}}
 let root=std::env::temp_dir().join(format!("ac787-r5-epsilon-{tag}-{}",std::process::id()));let _=std::fs::remove_dir_all(&root);let sas=SyntaxArtifactStore::open(&root,"epsilon").unwrap();let t=sas.put_tokens(&p,&s,&out.tokens).unwrap();sas.put_cst(&p,&s,&out.tree,&t).unwrap();let v=sas.open_cst(&p,&s).unwrap().unwrap();let result=v.decl_subtree(0);eprintln!("{tag} recovered={:?}",result.as_ref().map(|t|t.node_total()));assert!(result.is_ok(),"accepted core no-argument calls must survive actual subtree admission: {result:?}");
 }
}

#[test]
fn range_query_preserves_outline_for_large_legal_trivia(){
 let p=ac_syntax_grammar::compile(&std::fs::read_to_string(format!("{ROOT}/experimental/ac-syntax/grammars/interface-mini-v1.at")).unwrap()).unwrap();
 for prefix in [0,(2<<20)+10]{let text=format!("{}api f() -> i32;\n"," ".repeat(prefix));let root=std::env::temp_dir().join(format!("ac787-r5-window-{prefix}-{}",std::process::id()));let _=std::fs::remove_dir_all(&root);let sas=SyntaxArtifactStore::open(&root,"window").unwrap();let mut b=ac_syntax_artifact_adapter::StreamingCstBuilder::new(&sas,&p);b.push_chunk(&text).unwrap();b.finish(text.len()as u64).unwrap();let id=b.stream_snapshot_id();let v=sas.open_cst_stream(&p,&id,text.len()as u64).unwrap().unwrap();let st=v.decl_subtree(0).unwrap();let snap=Snapshot::from_text(&Rc::new(PageStore::new()),&text);let expected=ac_syntax_rt::outline::outline(&p,&st,&snap).unwrap();assert_eq!(expected[0].name,"f");
 let mut cmd=std::process::Command::new(env!("CARGO_BIN_EXE_range-query"));cmd.arg(&root).arg(p.grammar_id().as_str()).arg(p.revision.to_string()).arg(&id).arg(text.len().to_string()).arg("0");let r=ac_artifact_probe::subproc::run_deadlined(cmd,std::time::Duration::from_secs(30)).unwrap();eprintln!("window prefix={prefix} code={:?} stdout={} stderr={}",r.code,r.stdout,r.stderr);assert!(!r.timed_out);assert_eq!(r.code,Some(0));assert!(r.stdout.contains("name=f"),"successful query must retain exact name despite legal trivia");
 }
}

#[test]
fn inventory_returned_parse_allocations_omitted_by_counter(){
 let p=ac_syntax_grammar::compile(&std::fs::read_to_string(format!("{ROOT}/experimental/ac-syntax/grammars/interface-mini-v1.at")).unwrap()).unwrap();let cs=CompiledScanner::compile(&p).unwrap();let mut chunk=String::new();let mut i=0;while chunk.len()<(1<<16){chunk.push_str(&format!("api f{i}(a: i32, b: i32) -> i32;\nnote \"n{i}\";\n"));i+=1;}
 let s=Snapshot::from_text(&Rc::new(PageStore::new()),&chunk);let out=parse_snapshot(&p,&cs,&s,&Budget::default()).unwrap();
 let blocks=out.tree.blocks.capacity()*std::mem::size_of::<Rc<ac_syntax_rt::cst::CstBlock>>();let roots=out.tree.root_edges.capacity()*std::mem::size_of::<(u32,u32)>();let diags=out.tree.diags.capacity()*std::mem::size_of::<ac_syntax_rt::cst::SynDiag>()+out.tree.diags.iter().map(|d|d.message.capacity()).sum::<usize>();
 let source_bytes=s.page_refs().iter().map(|p|p.bytes().len()).sum::<usize>();
 eprintln!("RETURNED-ALLOCATION-OMISSIONS chunk_len={} blocks_vec={} root_edges_vec={} tree_diags={} source_page_bytes_lower_bound={} root_plus_blocks_lower_bound={}",chunk.len(),blocks,roots,diags,source_bytes,blocks+roots+diags);assert!(blocks>0&&roots>0&&source_bytes>0);
}
