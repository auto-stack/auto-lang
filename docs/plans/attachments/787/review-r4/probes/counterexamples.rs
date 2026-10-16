use ac_syntax_rt::{snapshot::{Snapshot,PageStore,Edit,EditRequest},lexer::CompiledScanner,parse::parse_snapshot,program::{GrammarProgram,Budget}};
use ac_syntax_artifact_adapter::{SyntaxArtifactStore,StreamingCstBuilder};
use std::rc::Rc;
fn grammar()->GrammarProgram {ac_syntax_grammar::compile(&std::fs::read_to_string("D:/autostack/.wt/lang-787/auto-lang/experimental/ac-syntax/grammars/interface-mini-v1.at").unwrap()).unwrap()}
fn scratch(s:&str)->std::path::PathBuf {std::env::temp_dir().join(format!("review787-r4-{s}-{}",std::process::id()))}

#[test]
fn fresh_and_incremental_artifacts_for_same_snapshot_can_coexist(){
    let p=grammar();let cs=CompiledScanner::compile(&p).unwrap();
    let text=(0..40).map(|i|format!("api f{i}() -> i32;\n")).collect::<String>();
    let pages=Rc::new(PageStore::new());let s=Snapshot::from_text(&pages,&text);
    let old=parse_snapshot(&p,&cs,&s,&Budget::default()).unwrap();
    let pos=text.find("f20").unwrap() as u64;let ed=Edit{start:pos,end:pos,insert:b"new_".to_vec()};
    let ns=s.edit(&EditRequest{base_revision:0,edits:vec![ed.clone()]}).unwrap();
    let inc=ac_syntax_rt::edit::incremental_reparse(&p,&cs,&old,&s,&ns,&[ed],&Budget::default()).unwrap();
    let fresh=parse_snapshot(&p,&cs,&ns,&Budget::default()).unwrap();
    assert_eq!(inc.tree.canonical(),fresh.tree.canonical());assert!(inc.reused_decls>0);
    let sas=SyntaxArtifactStore::open(&scratch("incremental-cache"),"project").unwrap();
    let tok=sas.put_tokens(&p,&ns,&fresh.tokens).unwrap();
    sas.put_cst(&p,&ns,&fresh.tree,&tok).unwrap();
    sas.put_cst(&p,&ns,&inc.tree,&tok).expect("same source/grammar must not conflict because arena allocation differs");
}

#[test]
fn child_span_outside_source_is_rejected_by_subtree_entry(){
    use ac_artifact_probe::{identity::QueryKey,schema,store::PutRequest,codec::{ChunkKind,framed_chunk_id}};
    let p=grammar();let s=Snapshot::from_text(&Rc::new(PageStore::new()),"x");let sas=SyntaxArtifactStore::open(&scratch("child-span"),"project").unwrap();
    let rule=ac_syntax_rt::program::RULE_KIND_BASE+p.rules.iter().position(|r|r.name=="api-decl").unwrap() as u32;
    let tok=ac_syntax_rt::program::TOKEN_KIND_BASE+p.tokens.iter().position(|t|t.name=="api").unwrap() as u32;
    let mut payload=Vec::new();payload.extend_from_slice(&2u32.to_le_bytes());payload.extend_from_slice(&1u32.to_le_bytes());
    // Valid root [0,1), one unique child with invalid source/parent span [9,10).
    for v in [rule,0,1,0,1,0,1,0,tok,9,1,1,0,0,1,0,1,u32::MAX]{payload.extend_from_slice(&v.to_le_bytes());}
    let cid=framed_chunk_id(ChunkKind::CstPage,&payload).to_string();
    sas.inner().put_artifact(PutRequest{layer:"A03",key:QueryKey::new(schema::layer("A03").unwrap(),vec![("page".into(),"child-span".into())]),kind:ChunkKind::CstPage,payload,edges:vec![],project:None}).unwrap();
    let m=serde_json::json!({"snapshot":s.id().as_str(),"grammar":p.grammar_id().as_str(),"total_len":1,"diags":[],"root_edges":[],"pages":[{"chunk":cid,"first_node":0,"node_count":2}],"decl_index":[],"decl_count":1,"decls":[{"origin":0,"node":0,"node_lo":0,"page":0}],"source_pages":[]});
    let key=QueryKey::new(schema::layer("A03").unwrap(),vec![("source-snapshot".into(),s.id().as_str().into()),("grammar-id".into(),p.grammar_id().as_str().into()),("parser-version".into(),ac_syntax_rt::PARSER_ABI_VERSION.to_string()),("page".into(),"manifest".into())]);
    sas.inner().put_artifact(PutRequest{layer:"A03",key,kind:ChunkKind::CstManifest,payload:serde_json::to_vec(&m).unwrap(),edges:vec![],project:None}).unwrap();
    match sas.open_cst(&p,&s){Err(_)|Ok(None)=>{},Ok(Some(v))=>{
        assert_eq!(v.decl_range(0).unwrap(),(0,1),"root-only check is an accepted control");
        assert!(v.decl_subtree(0).is_err(),"invalid child span must not escape the actual subtree read entry");
    }}
}

#[test]
fn range_query_handles_leading_trivia_across_source_page(){
    let p=grammar();
    for prefix in [0,(256<<10)+10] {
        let text=format!("{}api f() -> i32;\n"," ".repeat(prefix));
        let root=scratch(&format!("range-{prefix}"));let sas=SyntaxArtifactStore::open(&root,"project").unwrap();
        let mut b=StreamingCstBuilder::new(&sas,&p);b.push_chunk(&text).unwrap();b.finish(text.len() as u64).unwrap();let id=b.stream_snapshot_id();
        let view=sas.open_cst_stream(&p,&id,text.len() as u64).unwrap().unwrap();
        let st=view.decl_subtree(0).unwrap();eprintln!("prefix={prefix} origin={} span={:?}",st.decls[0].origin,view.decl_range(0).unwrap());
        let mut cmd=std::process::Command::new(env!("CARGO_BIN_EXE_range-query"));cmd.arg(&root).arg(p.grammar_id().as_str()).arg(p.revision.to_string()).arg(&id).arg(text.len().to_string()).arg("0");
        let r=ac_artifact_probe::subproc::run_deadlined(cmd,std::time::Duration::from_secs(30)).unwrap();
        eprintln!("prefix={prefix} code={:?} timeout={} stdout={} stderr={}",r.code,r.timed_out,r.stdout,r.stderr);
        assert!(!r.timed_out);assert_eq!(r.code,Some(0),"legitimate leading trivia must not underflow subtree origin translation");assert!(r.stdout.contains("name=f"));
    }
}
