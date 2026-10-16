use ac_syntax_rt::{snapshot::{Snapshot,PageStore,Edit,EditRequest},lexer::{CompiledScanner,lex_all},parse::parse_snapshot,program::{GrammarProgram,Budget}};
use ac_syntax_artifact_adapter::SyntaxArtifactStore;
use std::rc::Rc;
fn grammar()->GrammarProgram {ac_syntax_grammar::compile(&std::fs::read_to_string("D:/autostack/.wt/lang-787/auto-lang/experimental/ac-syntax/grammars/interface-mini-v1.at").unwrap()).unwrap()}
fn scratch(s:&str)->std::path::PathBuf {std::env::temp_dir().join(format!("review787-r3-{s}-{}",std::process::id()))}
fn put(sas:&SyntaxArtifactStore,p:&GrammarProgram,s:&Snapshot)->Result<String,ac_syntax_artifact_adapter::AdapterError>{let cs=CompiledScanner::compile(p).unwrap();let out=parse_snapshot(p,&cs,s,&Budget::default()).unwrap();let t=sas.put_tokens(p,s,&out.tokens)?;sas.put_cst(p,s,&out.tree,&t)}

#[test]
fn identical_content_at_new_revision_is_reusable() {
    let p=grammar();let text="api f() -> i32;\n";let s=Snapshot::from_text(&Rc::new(PageStore::new()),text);
    let s2=s.edit(&EditRequest{base_revision:0,edits:vec![Edit{start:0,end:s.len(),insert:text.as_bytes().to_vec()}]}).unwrap();
    assert_eq!(s.id(),s2.id());assert_eq!(s2.revision(),1);
    let sas=SyntaxArtifactStore::open(&scratch("same-source"),"project").unwrap();put(&sas,&p,&s).unwrap();
    put(&sas,&p,&s2).expect("same source at revision1 must reuse content, not conflict because revision was embedded in immutable payload");
}

fn project_child(n:usize,shape:&str)->ac_artifact_probe::subproc::RunResult{
    let mut c=std::process::Command::new(env!("CARGO_BIN_EXE_projection-normal-stack"));c.arg(n.to_string()).arg(shape);
    ac_artifact_probe::subproc::run_deadlined(c,std::time::Duration::from_secs(30)).unwrap()
}
#[test]
fn projection_small_normal_stack_control(){let r=project_child(32,"chain");eprintln!("code={:?} timeout={} stderr={}",r.code,r.timed_out,r.stderr);assert!(!r.timed_out);assert_eq!(r.code,Some(0));}
#[test]
fn projection_deep_chain_normal_stack(){let r=project_child(2200,"chain");eprintln!("code={:?} timeout={} stderr={}",r.code,r.timed_out,r.stderr);assert!(!r.timed_out);assert_eq!(r.code,Some(0),"valid deep input must not abort process");}
#[test]
fn projection_nested_blocks_normal_stack(){let r=project_child(512,"blocks");eprintln!("code={:?} timeout={} stderr={}",r.code,r.timed_out,r.stderr);assert!(!r.timed_out);assert_eq!(r.code,Some(0),"nested blocks must not abort process");}

#[test]
fn multiline_checkpoint_resume_preserves_original_token_line(){
    let p=grammar();let cs=CompiledScanner::compile(&p).unwrap();
    let base="note \"first line\nsecond line";let ext=" closes\";\napi f() -> i32;\n";
    let pages=Rc::new(PageStore::new());let s=Snapshot::from_text(&pages,base);let lx=lex_all(&p,&cs,&s).unwrap();
    let sas=SyntaxArtifactStore::open(&scratch("multiline-checkpoint"),"project").unwrap();sas.put_tokens(&p,&s,&lx.tokens).unwrap();
    let (_,cp)=sas.get_tokens_with_checkpoint(&p,&s).unwrap().unwrap();
    let full=Snapshot::from_text(&pages,&format!("{base}{ext}"));let fresh=lex_all(&p,&cs,&full).unwrap();
    let mut sc=ac_syntax_rt::lexer::Scanner::start(&p,&cs,&full,&cp).unwrap();let mut resumed=Vec::new();while let Some(t)=sc.next_token().unwrap(){resumed.push(t);}
    let start=fresh.tokens.iter().position(|t|t.start==resumed[0].start).unwrap();
    assert_eq!(resumed,fresh.tokens[start..],"resumed multiline token must keep the original starting line and following token line numbers");
}

#[test]
fn failed_root_registration_preserves_previous_current_root(){
    let p=grammar();let old=Snapshot::from_text(&Rc::new(PageStore::new()),"api old() -> i32;\n");
    let new=old.edit(&EditRequest{base_revision:0,edits:vec![Edit{start:4,end:7,insert:b"new".to_vec()}]}).unwrap();
    let sas=SyntaxArtifactStore::open(&scratch("publication-failure"),"project").unwrap();let first=put(&sas,&p,&old).unwrap();
    sas.inner().db().execute_batch("CREATE TRIGGER review_fail_root BEFORE INSERT ON roots WHEN NEW.kind = 'cst-manifest' BEGIN SELECT RAISE(ABORT, 'review injected root registration failure'); END;").unwrap();
    let r=put(&sas,&p,&new);assert!(r.is_err(),"failure control must fire");
    let roots=sas.inner().root_artifacts("project","cst-manifest").unwrap();
    assert_eq!(roots,vec![first],"publication error must preserve the previous current root");
}

#[test]
fn fresh_document_session_revision_zero_can_be_current(){
    let p=grammar();let old=Snapshot::from_text(&Rc::new(PageStore::new()),"api old() -> i32;\n");
    let old1=old.edit(&EditRequest{base_revision:0,edits:vec![Edit{start:4,end:7,insert:b"old1".to_vec()}]}).unwrap();
    let root=scratch("restart-revision");{
        let sas=SyntaxArtifactStore::open(&root,"project").unwrap();put(&sas,&p,&old1).unwrap();
    }
    // New document/session after restart. Revision numbers are session-local.
    let new=Snapshot::from_text(&Rc::new(PageStore::new()),"api new() -> i32;\n");assert_eq!(new.revision(),0);
    let sas=SyntaxArtifactStore::open(&root,"project").unwrap();let a=put(&sas,&p,&new).unwrap();
    assert!(sas.inner().root_artifacts("project","cst-manifest").unwrap().contains(&a),"revision0 in a new session is not a late task from the old session");
}

#[test]
fn valid_framed_out_of_source_span_is_rejected(){
    use ac_artifact_probe::{identity::QueryKey,schema,store::PutRequest,codec::{ChunkKind,framed_chunk_id}};
    let p=grammar();let s=Snapshot::from_text(&Rc::new(PageStore::new()),"x");let sas=SyntaxArtifactStore::open(&scratch("span"),"project").unwrap();
    let mut payload=Vec::new();payload.extend_from_slice(&1u32.to_le_bytes());payload.extend_from_slice(&0u32.to_le_bytes());
    let kind=ac_syntax_rt::program::RULE_KIND_BASE+p.rules.iter().position(|r|r.name=="api-decl").unwrap() as u32;
    for v in [kind,9,1,0,0,0,1,0]{payload.extend_from_slice(&v.to_le_bytes());}
    let cid=framed_chunk_id(ChunkKind::CstPage,&payload).to_string();
    sas.inner().put_artifact(PutRequest{layer:"A03",key:QueryKey::new(schema::layer("A03").unwrap(),vec![("page".into(),"bad-span".into())]),kind:ChunkKind::CstPage,payload,edges:vec![],project:None}).unwrap();
    let m=serde_json::json!({"snapshot":s.id().as_str(),"grammar":p.grammar_id().as_str(),"revision":0,"total_len":1,"diags":[],"root_edges":[],"pages":[{"chunk":cid,"first_node":0,"node_count":1}],"decl_index":[],"decl_count":1,"decls":[{"origin":0,"node":0,"node_lo":0,"page":0}],"source_pages":[]});
    let key=QueryKey::new(schema::layer("A03").unwrap(),vec![("source-snapshot".into(),s.id().as_str().into()),("grammar-id".into(),p.grammar_id().as_str().into()),("parser-version".into(),ac_syntax_rt::PARSER_ABI_VERSION.to_string()),("page".into(),"manifest".into())]);
    sas.inner().put_artifact(PutRequest{layer:"A03",key,kind:ChunkKind::CstManifest,payload:serde_json::to_vec(&m).unwrap(),edges:vec![],project:None}).unwrap();
    match sas.open_cst(&p,&s){Err(_)|Ok(None)=>{},Ok(Some(v))=>assert!(v.decl_range(0).is_err(),"one-byte source must not expose a successful [9,10) declaration span")}
}
