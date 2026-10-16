use ac_syntax_rt::{snapshot::{Snapshot,PageStore,PAGE_BYTES,Edit,EditRequest},position::LineIndex,lexer::{CompiledScanner,lex_all},parse::parse_snapshot,program::{GrammarProgram,Budget}};
use ac_syntax_artifact_adapter::{SyntaxArtifactStore,StreamingCstBuilder};
use std::rc::Rc;
fn grammar() -> GrammarProgram {
    ac_syntax_grammar::compile(&std::fs::read_to_string("D:/autostack/.wt/lang-787/auto-lang/experimental/ac-syntax/grammars/interface-mini-v1.at").unwrap()).unwrap()
}
fn scratch(s:&str)->std::path::PathBuf {
    std::env::temp_dir().join(format!("review787-r1-{s}-{}",std::process::id()))
}
#[test]
fn live_snapshot_survives_other_document_pressure() {
    let pages=Rc::new(PageStore::with_budget(PAGE_BYTES));
    let old=Snapshot::from_text(&pages,&"a".repeat(PAGE_BYTES));
    let _new=Snapshot::from_text(&pages,&"b".repeat(PAGE_BYTES));
    assert_eq!(old.read_str(0,1).unwrap(),"a");
}
#[test]
fn line_column_inside_cross_page_line() {
    let s=Snapshot::from_text(&Rc::new(PageStore::new()),&"x".repeat(PAGE_BYTES+10));
    let ix=LineIndex::build(&s);
    assert_eq!(ix.line_col_utf8(&s,PAGE_BYTES as u64+5),(1,PAGE_BYTES as u64+6));
}
#[test]
fn grammar_variant_tokens_do_not_hit_old_grammar() {
    let p=grammar();
    // Use a genuine recompile to establish a new canonical grammar identity.
    let text=std::fs::read_to_string("D:/autostack/.wt/lang-787/auto-lang/experimental/ac-syntax/grammars/interface-mini-v1.at").unwrap();
    let p2=ac_syntax_grammar::compile(&text.replace("literal: \"api\"","literal: \"endpoint\"")).unwrap();
    assert_ne!(p.grammar_id(),p2.grammar_id());
    let s=Snapshot::from_text(&Rc::new(PageStore::new()),"api f() -> i32;");
    let cs=CompiledScanner::compile(&p).unwrap();
    let out=lex_all(&p,&cs,&s).unwrap();
    let sas=SyntaxArtifactStore::open(&scratch("grammar"),"review").unwrap();
    sas.put_tokens(&p,&s,&out.tokens).unwrap();
    assert!(sas.get_tokens(&p2,&s).unwrap().is_none(),"A02 must miss for different grammar");
}
#[test]
fn local_lexer_real_reads_are_bounded() {
    let p=grammar(); let cs=CompiledScanner::compile(&p).unwrap();
    let text=(0..40000).map(|i|format!("api f{i}(a: i32) -> i32;\n")).collect::<String>();
    let pages=Rc::new(PageStore::new()); let s=Snapshot::from_text(&pages,&text);
    let old=lex_all(&p,&cs,&s).unwrap();
    let pos=text.find("f20000").unwrap() as u64;
    let ed=Edit{start:pos,end:pos+1,insert:b"g".to_vec()};
    let ns=s.edit(&EditRequest{base_revision:0,edits:vec![ed.clone()]}).unwrap();
    pages.read_bytes.set(0);
    let sp=ac_syntax_rt::edit::incremental_lex(&p,&cs,&old.tokens,&s,&ns,&[ed]).unwrap();
    let actual=pages.read_bytes.get();
    eprintln!("reported_rescan={} actual_source_page_bytes={actual}",sp.rescan_bytes);
    assert!(actual<=65536,"local scanning read {actual} bytes, reported {}",sp.rescan_bytes);
}
#[test]
fn rooted_stream_directory_survives_gc() {
    let p=grammar();let sas=SyntaxArtifactStore::open(&scratch("gc"),"review").unwrap();
    let text="api f() -> i32;\n";
    let mut b=StreamingCstBuilder::new(&sas,&p);b.push_chunk(text).unwrap();b.finish(text.len() as u64).unwrap();
    let id=b.stream_snapshot_id();
    assert!(sas.open_cst_stream(&p,&id,text.len() as u64).unwrap().unwrap().decl_subtree(0).is_ok());
    ac_artifact_probe::store::gc::run_gc(sas.inner(),ac_artifact_probe::store::gc::GcConfig{high:0,low:0}).unwrap();
    let v=sas.open_cst_stream(&p,&id,text.len() as u64).unwrap().unwrap();
    assert!(v.decl_subtree(0).is_ok(),"rooted declaration index must survive GC");
}
#[test]
fn paged_multi_page_tree_roundtrip() {
    let p=grammar();let cs=CompiledScanner::compile(&p).unwrap();
    let text=(0..100).map(|i|format!("api f{i}(a: i32) -> i32;\n")).collect::<String>();
    let s=Snapshot::from_text(&Rc::new(PageStore::new()),&text);
    let out=parse_snapshot(&p,&cs,&s,&Budget::default()).unwrap();
    let sas=SyntaxArtifactStore::open(&scratch("multipage"),"review").unwrap();
    let tok=sas.put_tokens(&p,&s,&out.tokens).unwrap();
    sas.put_cst(&p,&s,&out.tree,&tok).unwrap();
    let v=sas.open_cst(&p,&s).unwrap().unwrap();
    for i in 0..100 { assert!(v.decl_subtree(i).is_ok()); }
}
#[test]
fn valid_framed_bad_manifest_ref_is_controlled() {
    use ac_artifact_probe::{identity::QueryKey,schema,store::PutRequest,codec::ChunkKind};
    let p=grammar();let s=Snapshot::from_text(&Rc::new(PageStore::new()),"api f() -> i32;");
    let sas=SyntaxArtifactStore::open(&scratch("bad-ref"),"review").unwrap();
    let manifest=serde_json::json!({"snapshot":s.id().as_str(),"grammar":p.grammar_id().as_str(),"total_len":s.len(),"diags":[],"root_edges":[],"pages":[],"decl_index":[],"decl_count":1,"decls":[{"origin":0,"node":0,"node_lo":0,"page":999}]});
    let key=QueryKey::new(schema::layer("A03").unwrap(),vec![("source-snapshot".into(),s.id().as_str().into()),("grammar-id".into(),p.grammar_id().as_str().into()),("parser-version".into(),ac_syntax_rt::PARSER_ABI_VERSION.to_string()),("page".into(),"manifest".into())]);
    sas.inner().put_artifact(PutRequest{layer:"A03",key,kind:ChunkKind::CstManifest,payload:serde_json::to_vec(&manifest).unwrap(),edges:vec![],project:None}).unwrap();
    let outcome=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||{
        match sas.open_cst(&p,&s) {Err(_)=>true,Ok(None)=>true,Ok(Some(v))=>v.decl_subtree(0).is_err()}
    }));
    assert_eq!(outcome.ok(),Some(true),"bad ref must return error/miss, never panic");
}
