use ac_syntax_rt::{snapshot::{PageStore,Snapshot},lexer::CompiledScanner,parse::parse_snapshot,program::Budget};
use ac_syntax_artifact_adapter::{SyntaxArtifactStore,StreamingCstBuilder};
use std::rc::Rc;
const ROOT:&str="D:/autostack/.wt/lang-787/auto-lang";
fn grammar(name:&str)->ac_syntax_rt::program::GrammarProgram{ac_syntax_grammar::compile(&std::fs::read_to_string(format!("{ROOT}/experimental/ac-syntax/grammars/{name}.at")).unwrap()).unwrap()}
#[test]
fn epsilon_coordinates_work_across_whitespace_and_recovery(){
 let p=grammar("auto-core");let cs=CompiledScanner::compile(&p).unwrap();
 for (i,text) in ["fn main() int { return f(\n\n) }\n", "fn main() int { return f(// hello\n) }\n", "fn main() int { return f(g(), h()) }\n", "fn main() int { if f() { return g() } else { return h() } }\n", "fn a() int { return f() }\nfn b() int { return g() }\n", "fn main() int { return f( }\nfn b() int { return 1 }\n"].iter().enumerate(){
 let s=Snapshot::from_text(&Rc::new(PageStore::new()),text);let out=parse_snapshot(&p,&cs,&s,&Budget::default()).unwrap();let root=std::env::temp_dir().join(format!("ac787-r6-epsilon-{i}-{}",std::process::id()));let _=std::fs::remove_dir_all(&root);let sas=SyntaxArtifactStore::open(&root,"epsilon").unwrap();let t=sas.put_tokens(&p,&s,&out.tokens).unwrap();sas.put_cst(&p,&s,&out.tree,&t).unwrap();let v=sas.open_cst(&p,&s).unwrap().unwrap();for d in 0..v.decl_count(){v.decl_subtree(d).unwrap_or_else(|e|panic!("case {i} decl{d}: {e:?}"));}eprintln!("epsilon case{i} decls={} OK",v.decl_count());
 }
}
#[test]
fn query_window_limit_is_checked_before_materializing_all_name_pages(){
 let p=grammar("interface-mini-v1");
 for n in [8,8<<20]{let text=format!("api {}() -> i32;\n","x".repeat(n));let root=std::env::temp_dir().join(format!("ac787-r6-name-{n}-{}",std::process::id()));let _=std::fs::remove_dir_all(&root);let sas=SyntaxArtifactStore::open(&root,"window").unwrap();let mut b=StreamingCstBuilder::new(&sas,&p);b.push_chunk(&text).unwrap();b.finish(text.len()as u64).unwrap();let id=b.stream_snapshot_id();
 let mut cmd=std::process::Command::new(env!("CARGO_BIN_EXE_range-query"));cmd.arg(&root).arg(p.grammar_id().as_str()).arg(p.revision.to_string()).arg(id).arg(text.len().to_string()).arg("0");let r=ac_artifact_probe::subproc::run_deadlined(cmd,std::time::Duration::from_secs(30)).unwrap();assert!(!r.timed_out);eprintln!("name_len={n} code={:?} stderr={}",r.code,r.stderr);
 if n==8 {assert_eq!(r.code,Some(0));assert!(r.stdout.contains("name=xxxxxxxx"));}else{assert_eq!(r.code,Some(4));let materialized:usize=r.stderr.split("window ").nth(1).unwrap().split_whitespace().next().unwrap().parse().unwrap();assert!(materialized<=(2<<20)+(256<<10),"window limit must reject while accumulating, allowing at most one source page overshoot, observed {materialized} bytes");}
 }
}

#[test]
fn sequential_released_view_leases_do_not_collide(){
 let root=std::env::temp_dir().join(format!("ac787-r6-lease-{}",std::process::id()));let _=std::fs::remove_dir_all(&root);let sas=SyntaxArtifactStore::open(&root,"leases").unwrap();
 {let first=sas.inner().acquire_lease("control",3600,None).expect("single lease positive control");drop(first);}
 let mut failures=Vec::new();for i in 0..128{match sas.inner().acquire_lease("sequential",3600,None){Ok(l)=>drop(l),Err(e)=>failures.push(format!("iteration{i}: {e:?}"))}}
 eprintln!("released sequential lease attempts=128 failures={}",failures.len());for e in failures.iter().take(4){eprintln!("{e}");}assert!(failures.is_empty(),"ordinary sequential lease grants must have unique ids even in one millisecond");
}

#[test]
fn source_put_peak_covers_buffer_payload_and_framed_copy(){
 let text="x".repeat(256<<10);let snap=Snapshot::from_text(&Rc::new(PageStore::new()),&text);let root=std::env::temp_dir().join(format!("ac787-r6-putpeak-{}",std::process::id()));let _=std::fs::remove_dir_all(&root);let sas=SyntaxArtifactStore::open(&root,"putpeak").unwrap();let (_,_,reported)=sas.put_source_pages(&snap).unwrap();let framed=ac_artifact_probe::codec::frame_chunk(ac_artifact_probe::codec::ChunkKind::SourcePage,text.as_bytes());let minimum=(256<<10)+text.len()+framed.len();eprintln!("source put reported={reported} concurrent_lower_bound={minimum} buffer={} payload={} framed={}",256<<10,text.len(),framed.len());assert!(reported>=minimum,"source put must count original buffer, request payload clone, and separately allocated framed bytes concurrently");
}
