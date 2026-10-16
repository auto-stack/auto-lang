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

#[test]
fn stream_source_and_grammar_survive_current_root_gc() {
    let p=grammar();let text="api current(a: i32) -> i32;\n";
    let sas=SyntaxArtifactStore::open(&scratch("r2-source-gc"),"project-a").unwrap();
    let mut b=StreamingCstBuilder::new(&sas,&p);b.push_chunk(text).unwrap();b.finish(text.len() as u64).unwrap();
    let id=b.stream_snapshot_id();
    ac_artifact_probe::store::gc::run_gc(sas.inner(),ac_artifact_probe::store::gc::GcConfig{high:0,low:0}).unwrap();
    let v=sas.open_cst_stream(&p,&id,text.len() as u64).unwrap().unwrap();
    assert_eq!(v.source_bytes().unwrap(),text.as_bytes(),"current root must retain source");
    assert!(sas.get_grammar(&p).unwrap().is_some(),"current root must retain grammar");
}

#[test]
fn independent_streams_can_share_store() {
    let p=grammar();let sas=SyntaxArtifactStore::open(&scratch("r2-two-streams"),"project-a").unwrap();
    let mut a=StreamingCstBuilder::new(&sas,&p);a.push_chunk("api f() -> i32;\n").unwrap();a.finish(16).unwrap();
    let mut b=StreamingCstBuilder::new(&sas,&p);
    b.push_chunk("note \"second\";\n").expect("different document must not reuse incomplete prefix key");
}

fn plant_manifest(sas:&SyntaxArtifactStore,p:&GrammarProgram,s:&Snapshot,m:&serde_json::Value,key_id:&str) {
    use ac_artifact_probe::{identity::QueryKey,schema,store::PutRequest,codec::ChunkKind};
    let key=QueryKey::new(schema::layer("A03").unwrap(),vec![("source-snapshot".into(),key_id.into()),("grammar-id".into(),p.grammar_id().as_str().into()),("parser-version".into(),ac_syntax_rt::PARSER_ABI_VERSION.to_string()),("page".into(),"manifest".into())]);
    let _=s;
    sas.inner().put_artifact(PutRequest{layer:"A03",key,kind:ChunkKind::CstManifest,payload:serde_json::to_vec(m).unwrap(),edges:vec![],project:None}).unwrap();
}

#[test]
fn valid_manifest_ref_below_page_base_is_controlled() {
    let p=grammar();let s=Snapshot::from_text(&Rc::new(PageStore::new()),"x");
    let sas=SyntaxArtifactStore::open(&scratch("r2-underflow"),"review").unwrap();
    let m=serde_json::json!({"snapshot":s.id().as_str(),"grammar":p.grammar_id().as_str(),"total_len":1,"diags":[],"root_edges":[],"pages":[{"chunk":"a","first_node":0,"node_count":1},{"chunk":"b","first_node":1,"node_count":1}],"decl_index":[],"decl_count":1,"decls":[{"origin":0,"node":0,"node_lo":0,"page":1}],"source_pages":[]});
    plant_manifest(&sas,&p,&s,&m,s.id().as_str());
    let r=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||sas.open_cst(&p,&s).is_err()));
    assert_eq!(r.ok(),Some(true),"bad lower-bound reference must reject without panic");
}

#[test]
fn stream_manifest_identity_is_checked() {
    let p=grammar();let s=Snapshot::from_text(&Rc::new(PageStore::new()),"x");
    let sas=SyntaxArtifactStore::open(&scratch("r2-stream-id"),"review").unwrap();
    let m=serde_json::json!({"snapshot":"other-snapshot","grammar":"other-grammar","total_len":1,"diags":[],"root_edges":[],"pages":[],"decl_index":[],"decl_count":0,"decls":[],"source_pages":[]});
    plant_manifest(&sas,&p,&s,&m,"requested-stream");
    assert!(sas.open_cst_stream(&p,"requested-stream",1).is_err(),"wrong grammar/snapshot content must reject under valid lookup key");
}

#[test]
fn cyclic_page_is_rejected_before_tree_exposure() {
    use ac_artifact_probe::{identity::QueryKey,schema,store::PutRequest,codec::{ChunkKind,framed_chunk_id}};
    let p=grammar();let s=Snapshot::from_text(&Rc::new(PageStore::new()),"x");
    let sas=SyntaxArtifactStore::open(&scratch("r2-cycle"),"review").unwrap();
    let mut payload=Vec::new();payload.extend_from_slice(&1u32.to_le_bytes());payload.extend_from_slice(&1u32.to_le_bytes());
    // One node, one self-edge. All offsets/counts fit, but this is not a tree.
    for v in [1u32,0,1,0,1,0,1,0] {payload.extend_from_slice(&v.to_le_bytes());}
    payload.extend_from_slice(&0u32.to_le_bytes());payload.extend_from_slice(&u32::MAX.to_le_bytes());
    let cid=framed_chunk_id(ChunkKind::CstPage,&payload).to_string();
    sas.inner().put_artifact(PutRequest{layer:"A03",key:QueryKey::new(schema::layer("A03").unwrap(),vec![("page".into(),"cycle".into())]),kind:ChunkKind::CstPage,payload,edges:vec![],project:None}).unwrap();
    let m=serde_json::json!({"snapshot":s.id().as_str(),"grammar":p.grammar_id().as_str(),"total_len":1,"diags":[],"root_edges":[],"pages":[{"chunk":cid,"first_node":0,"node_count":1}],"decl_index":[],"decl_count":1,"decls":[{"origin":0,"node":0,"node_lo":0,"page":0}],"source_pages":[]});
    plant_manifest(&sas,&p,&s,&m,s.id().as_str());
    let v=sas.open_cst(&p,&s).unwrap().unwrap();
    assert!(v.decl_subtree(0).is_err(),"cyclic graph must reject, not return a tree that loops during canonical/outline");
}

#[test]
fn nested_unclosed_comment_checkpoint_preserves_depth() {
    use ac_syntax_rt::lexer::MidToken;
    let p=grammar();let cs=CompiledScanner::compile(&p).unwrap();
    let s=Snapshot::from_text(&Rc::new(PageStore::new()),"/* outer /* inner");
    let lx=lex_all(&p,&cs,&s).unwrap();
    let sas=SyntaxArtifactStore::open(&scratch("r2-checkpoint"),"review").unwrap();
    sas.put_tokens(&p,&s,&lx.tokens).unwrap();
    let (_,cp)=sas.get_tokens_with_checkpoint(&p,&s).unwrap().unwrap();
    match cp.mid {Some(MidToken::Block{depth,..})=>assert_eq!(depth,2,"both nested comment openers remain active"),other=>panic!("expected block checkpoint: {other:?}")}
}

#[test]
fn foreign_session_handle_is_rejected() {
    use ac_syntax_rt::handle::{TreeSession,TreeEntry,TreeMeta,NodeHandle};
    let p=grammar();let s=Snapshot::from_text(&Rc::new(PageStore::new()),"x");
    let a=TreeSession::new();let b=TreeSession::new();
    let entry=||TreeEntry{tree:ac_syntax_rt::cst::CstTree::empty(),meta:TreeMeta{snapshot_id:s.id().clone(),grammar_id:p.grammar_id().clone(),revision:0}};
    let id=a.publish(entry());b.publish(entry());
    assert!(b.resolve(NodeHandle{tree:id,node:0}).is_err(),"a handle from a distinct tree session must not resolve to another tree");
}

#[test]
fn repaired_declaration_has_no_unreachable_old_error_flags() {
    let p=grammar();let cs=CompiledScanner::compile(&p).unwrap();
    let text="api bad(a: i32 -> i32;\napi good() -> i32;\napi more() -> i32;\napi end() -> i32;\n";
    let pages=Rc::new(PageStore::new());let s=Snapshot::from_text(&pages,text);
    let old=parse_snapshot(&p,&cs,&s,&Budget::default()).unwrap();
    let pos=text.find(" ->").unwrap() as u64;
    let ed=Edit{start:pos,end:pos,insert:b")".to_vec()};
    let ns=s.edit(&EditRequest{base_revision:0,edits:vec![ed.clone()]}).unwrap();
    let inc=ac_syntax_rt::edit::incremental_reparse(&p,&cs,&old,&s,&ns,&[ed],&Budget::default()).unwrap();
    let fresh=parse_snapshot(&p,&cs,&ns,&Budget::default()).unwrap();
    assert_eq!(inc.tree.canonical(),fresh.tree.canonical(),"reachable trees match");
    assert!(!fresh.tree.has_error_nodes());
    assert_eq!(inc.tree.has_error_nodes(),fresh.tree.has_error_nodes(),"shared unreachable old errors must not make repaired tree fail strict");
}

#[test]
fn canonical_detects_field_mutation() {
    use ac_syntax_rt::cst::CstBlock;
    let p=grammar();let cs=CompiledScanner::compile(&p).unwrap();
    let s=Snapshot::from_text(&Rc::new(PageStore::new()),"api f(a: i32) -> i32;\n");
    let tree=parse_snapshot(&p,&cs,&s,&Budget::default()).unwrap().tree;
    let mut changed=tree.clone();let mut mutated=false;
    for b in &mut changed.blocks {
        if let Some(i)=b.fields.iter().position(|&f|f!=u32::MAX) {
            let mut fields=b.fields.clone();fields[i]=u32::MAX;
            *b=Rc::new(CstBlock{nodes:b.nodes.clone(),edges:b.edges.clone(),fields});mutated=true;break;
        }
    }
    assert!(mutated,"control must change a real edge field");
    assert_ne!(tree.canonical(),changed.canonical(),"fresh/incremental oracle must include fields");
}

#[test]
fn supported_core_chain_projects_without_fixed_syntax_depth_cutoff() {
    // A large explicit test-thread stack isolates the semantic depth cutoff;
    // it is not evidence that the adapter is stack-safe on normal stacks.
    std::thread::Builder::new().stack_size(64<<20).spawn(||{
        use ac_syntax_core_adapter::{CoreKinds,project_strict};
        let p=ac_syntax_grammar::compile(&std::fs::read_to_string("D:/autostack/.wt/lang-787/auto-lang/experimental/ac-syntax/grammars/auto-core.at").unwrap()).unwrap();
        let text=format!("fn f() int {{ return 1{} }}"," + 1".repeat(2200));
        assert!(auto_ac_prototype::source::parse_program(&text).is_ok(),"legacy core accepts input");
        let s=Snapshot::from_text(&Rc::new(PageStore::new()),&text);let cs=CompiledScanner::compile(&p).unwrap();
        let tree=parse_snapshot(&p,&cs,&s,&Budget::default()).unwrap().tree;
        assert!(!tree.has_error_nodes());
        let r=project_strict(&p,&CoreKinds::from_program(&p),&tree,&s);
        assert!(r.is_ok(),"valid core input must project, got {:?}",r.err());
    }).unwrap().join().unwrap();
}

#[test]
fn late_persistent_publication_does_not_replace_current_root() {
    let p=grammar();let cs=CompiledScanner::compile(&p).unwrap();
    let old=Snapshot::from_text(&Rc::new(PageStore::new()),"api a() -> i32;\n");
    let new=old.edit(&EditRequest{base_revision:0,edits:vec![Edit{start:4,end:5,insert:b"new_name".to_vec()}]}).unwrap();
    assert_eq!(new.revision(),1);
    let sas=SyntaxArtifactStore::open(&scratch("r2-late-persist"),"project").unwrap();
    // Deterministic completion order: current revision finishes first; old task arrives last.
    for snap in [&new,&old] {
        let out=parse_snapshot(&p,&cs,snap,&Budget::default()).unwrap();
        let t=sas.put_tokens(&p,snap,&out.tokens).unwrap();
        sas.put_cst(&p,snap,&out.tree,&t).unwrap();
    }
    ac_artifact_probe::store::gc::run_gc(sas.inner(),ac_artifact_probe::store::gc::GcConfig{high:0,low:0}).unwrap();
    assert!(sas.open_cst(&p,&new).unwrap().is_some(),"late old task must not replace the real current persisted root");
}
