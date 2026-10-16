use ac_syntax_artifact_adapter::SyntaxArtifactStore;
use ac_syntax_rt::snapshot::{Snapshot,PageStore};
use std::rc::Rc;
#[test]
fn produced_source_page_is_readable_through_artifact_api(){
 let root=std::env::temp_dir().join(format!("ac787-r7-source-page-{}",std::process::id()));let _=std::fs::remove_dir_all(&root);let sas=SyntaxArtifactStore::open(&root,"source-read").unwrap();
 let p=ac_syntax_grammar::compile(&std::fs::read_to_string("D:/autostack/.wt/lang-787/auto-lang/experimental/ac-syntax/grammars/interface-mini-v1.at").unwrap()).unwrap();let grammar=sas.put_grammar(&p).unwrap();assert!(sas.inner().get_artifact_by_id(&grammar).unwrap().is_some(),"A01 kind=6 valid control");
 let source="api f() -> i32;\n";let snap=Snapshot::from_text(&Rc::new(PageStore::new()),source);let(arts,_,_)=sas.put_source_pages(&snap).unwrap();assert_eq!(arts.len(),1);let loaded=sas.inner().get_artifact_by_id(&arts[0]);let hash=ac_artifact_probe::identity::domain_hash("src-page",&[source.as_bytes()]);let key=ac_artifact_probe::identity::QueryKey::new(ac_artifact_probe::schema::layer("A02").unwrap(),vec![("page-hash".into(),hash)]);let by_key=sas.inner().get_artifact("A02",&key);eprintln!("produced SourcePage by-id read={loaded:?}; by-key read={by_key:?}");assert!(by_key.is_ok(),"normally produced live SourcePage must also be readable through original by-key API: {by_key:?}");let loaded=loaded.expect("a normally produced live SourcePage must be readable by its returned artifact id").unwrap();assert_eq!(loaded.kind,ac_artifact_probe::codec::ChunkKind::SourcePage);assert_eq!(loaded.payload,source.as_bytes());
}
