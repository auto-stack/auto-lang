// R3 full-invariant controls against frozen e04de7b96; no implementation edits.
use ac_artifact_probe::{codec::{frame_chunk,ChunkKind},identity::{ChunkId,QueryKey},schema,store::{self,FileGuard,GcConfig,ManagedStore,PutRequest}};
use std::{fs,path::Path,sync::{Arc,Barrier},time::Duration};
fn key(s:&str)->QueryKey{QueryKey::new(schema::layer("A04").unwrap(),vec![("case".into(),s.into())])}
fn dir_bytes(root:&Path)->u64 {fs::read_dir(root).map(|r|r.flatten().map(|e|if e.path().is_dir(){dir_bytes(&e.path())}else{e.metadata().unwrap().len()}).sum()).unwrap_or(0)}
fn main(){
 let root=std::env::args().nth(1).expect("new scratch root");let root=Path::new(&root);fs::create_dir(root).unwrap();
 let s=ManagedStore::open(&root.join("orphan-accounting")).unwrap();
 let framed=frame_chunk(ChunkKind::DeclSummary,&vec![7u8;3*1024*1024]);let cid=ChunkId::from_parts(&[&framed]).0;
 let shard=s.root.join("blocks/blobs").join(&cid[..2]);fs::create_dir_all(&shard).unwrap();fs::write(shard.join(&cid),&framed).unwrap();
 let gc=store::run_gc(&s,GcConfig{high:1024*1024,low:512*1024}).unwrap();
 println!("GC_ORPHAN before={} after={} low={} backoff={:?} deleted={}",gc.total_before,gc.total_after,512*1024,gc.backoff,gc.deleted_chunks);
 // Reusing ordinary live metadata cannot hide a crash orphan physically on disk.
 println!("GC_ORPHAN actual_blocks={} block_count={}",s.space().blocks,s.space().blocks_count);

 let mut over_budget=false;
 for round in 0..12 {
  let s=Arc::new(ManagedStore::open(&root.join(format!("budget-{round}"))).unwrap());
  let mut held=Vec::new();let mut payloads=Vec::new();
  for i in 0..4 {let p=vec![i as u8;3*1024*1024];let cid=ChunkId::from_parts(&[&frame_chunk(ChunkKind::DeclSummary,&p)]).0;held.push(FileGuard::try_lock(&s.root.join(format!("locks/chunk-{cid}.lock")),true).unwrap().unwrap());payloads.push(p);}
  let barrier=Arc::new(Barrier::new(5));let mut workers=Vec::new();
  for (i,p) in payloads.into_iter().enumerate(){let ss=s.clone();let bb=barrier.clone();workers.push(std::thread::spawn(move||{bb.wait();ss.put_artifact(PutRequest{layer:"A04",key:key(&format!("r{round}-t{i}")),kind:ChunkKind::DeclSummary,payload:p,edges:vec![],project:None})}));}
  barrier.wait();std::thread::sleep(Duration::from_millis(200));
  let actual=dir_bytes(&s.root.join("staging"));let reported=store::PEAK_STAGING.load(std::sync::atomic::Ordering::Relaxed);
  drop(held);let mut oks=0;let mut rejected=0;for w in workers{match w.join().unwrap(){Ok(_)=>oks+=1,Err(_)=>rejected+=1}}
  println!("STAGING_BUDGET round={} actual={} cap={} reported={} success={} rejected={}",round,actual,ManagedStore::STAGING_BUDGET,reported,oks,rejected);
  if actual>ManagedStore::STAGING_BUDGET{over_budget=true;break;}
 }
 println!("STAGING_BUDGET violated={}",over_budget);
}
