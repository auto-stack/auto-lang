// R4 sibling controls: crash republish metadata states and total temporary bytes.
use ac_artifact_probe::{codec::{frame_chunk,ChunkKind},identity::{ChunkId,QueryKey},schema,store::{self,GcConfig,ManagedStore,PutRequest}};
use std::{fs,path::Path,sync::{Arc,Barrier},time::Duration};
use std::os::windows::fs::OpenOptionsExt;
fn key(s:&str)->QueryKey{QueryKey::new(schema::layer("A04").unwrap(),vec![("case".into(),s.into())])}
fn put(s:&ManagedStore,k:&QueryKey,p:&[u8])->Result<String,store::StoreError>{s.put_artifact(PutRequest{layer:"A04",key:k.clone(),kind:ChunkKind::DeclSummary,payload:p.to_vec(),edges:vec![],project:None})}
fn blob(s:&ManagedStore,p:&[u8])->std::path::PathBuf{let f=frame_chunk(ChunkKind::DeclSummary,p);let c=ChunkId::from_parts(&[&f]).0;let b=s.root.join("blocks/blobs").join(&c[..2]).join(c);fs::create_dir_all(b.parent().unwrap()).unwrap();fs::write(&b,&f).unwrap();b}
fn bytes(p:&Path)->u64 {fs::read_dir(p).map(|r|r.flatten().map(|e|if e.path().is_dir(){bytes(&e.path())}else{e.metadata().unwrap().len()}).sum()).unwrap_or(0)}
fn main(){
 let root=std::env::args().nth(1).expect("unique new scratch root");let root=Path::new(&root);fs::create_dir(root).unwrap();
 let s=ManagedStore::open(&root.join("deleted-row-orphan")).unwrap();let p=vec![4u8;3*1024*1024];let k=key("republish");put(&s,&k,&p).unwrap();
 let g=store::run_gc(&s,GcConfig{high:1,low:0}).unwrap();assert_eq!(g.deleted_chunks,1);
 blob(&s,&p); // Real F2 crash-republish state: CAS restored, old chunk row is deleted.
 let g=store::run_gc(&s,GcConfig{high:1024*1024,low:512*1024}).unwrap();
 let r=store::recover(&s,true).unwrap();
 println!("DELETED_ROW_ORPHAN after_gc={} backoff={} orphans_removed={} after_recover={} recover_removed={}",g.total_after,g.backoff.is_some(),g.orphans_removed,s.space().total,r.orphan_chunks_removed+r.orphan_rows_removed);
 let s=ManagedStore::open(&root.join("live-row-orphan")).unwrap();let k=key("conflict");put(&s,&k,b"winner").unwrap();assert!(matches!(put(&s,&k,&p),Err(store::StoreError::Conflict{..})));
 let g=store::run_gc(&s,GcConfig{high:1024*1024,low:512*1024}).unwrap();let after=g.total_after;let r=store::recover(&s,true).unwrap();
 println!("LIVE_ROW_ORPHAN after_gc={} backoff={} deleted={} orphan_gc={} after_recover={} recover_removed={}",after,g.backoff.is_some(),g.deleted_chunks,g.orphans_removed,s.space().total,r.orphan_rows_removed);
 // Two ordinary producers; an OS share conflict holds their CAS rename in
 // bounded retry so the temporary copy and staging can be measured together.
 let s=Arc::new(ManagedStore::open(&root.join("temporary-cap")).unwrap());let mut held=Vec::new();let mut payloads=Vec::new();
 for i in 0..2 {let p=vec![i as u8;3*1024*1024];let b=blob(&s,&p);held.push(fs::OpenOptions::new().read(true).share_mode(0).open(b).unwrap());payloads.push(p);}
 let barrier=Arc::new(Barrier::new(3));let mut workers=Vec::new();
 for (i,p) in payloads.into_iter().enumerate(){let ss=s.clone();let bb=barrier.clone();workers.push(std::thread::spawn(move||{bb.wait();put(&ss,&key(&format!("t{i}")),&p)}));}
 barrier.wait();std::thread::sleep(Duration::from_millis(350));
 let staged=bytes(&s.root.join("staging"));let tmp:u64=fs::read_dir(s.root.join("blocks/blobs/ch")).unwrap().flatten().filter(|e|e.path().extension().is_some_and(|x|x=="tmp")).map(|e|e.metadata().unwrap().len()).sum();
 println!("TEMPORARY_CAP staging={} cas_tmp={} combined={} cap={}",staged,tmp,staged+tmp,ManagedStore::STAGING_BUDGET);
 drop(held);let mut ok=0;let mut failed=0;for w in workers{if w.join().unwrap().is_ok(){ok+=1}else{failed+=1}}println!("TEMPORARY_CAP producers_ok={} failed={}",ok,failed);
}
