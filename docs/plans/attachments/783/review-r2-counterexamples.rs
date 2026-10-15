// Independent R2 consumer of frozen 90a35f684. No implementation edits.
use ac_artifact_probe::{codec::{self,ChunkKind},identity::{ChunkId,QueryKey},layout::{LayoutStore,LayoutView,Manifest,PAGE_SIZE},schema,store::{self,FileGuard,ManagedStore,PutRequest}};
use std::{fs,path::Path,time::{Duration,Instant}};
fn key(s:&str)->QueryKey {QueryKey::new(schema::layer("A04").unwrap(),vec![("case".into(),s.into())])}
fn put(s:&ManagedStore,k:QueryKey,p:Vec<u8>)->Result<String,store::StoreError>{s.put_artifact(PutRequest{layer:"A04",key:k,kind:ChunkKind::DeclSummary,payload:p,edges:vec![],project:Some("current".into())})}
fn manifest(layout:&str,pages:Vec<String>,dirs:Vec<String>)->Manifest {Manifest{layout:layout.into(),version:1,record_count:1,record_pages:pages.clone(),payload_pages:pages.clone(),payload_len:8,data_pages:pages,dir_pages:dirs}}
fn main(){
 let root=std::env::args().nth(1).expect("new scratch root");let root=Path::new(&root);fs::create_dir(root).unwrap();
 let ls=LayoutStore::new(&root.join("layout"));
 for len in [13u16,14,15,21,22,23] {
  let mut data=vec![0u8;PAGE_SIZE];data[14]=8;
  let (did,_)=ls.write_chunk(&data).unwrap();
  let mut dir=vec![0u8;PAGE_SIZE];dir[8..10].copy_from_slice(&len.to_le_bytes());
  let (cid,_)=ls.write_chunk(&dir).unwrap();
  let got=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||LayoutView::open(&ls,manifest("B",vec![did],vec![cid])).unwrap().record(0)));
  println!("B_RECORD len={} outcome={}",len,match got {Err(_)=>"PANIC",Ok(Err(_))=>"controlled-error",Ok(Ok(_))=>"accepted"});
 }
 let mut a=vec![0u8;PAGE_SIZE];a[14..16].copy_from_slice(&8u16.to_le_bytes());a[16..24].copy_from_slice(&(1u64<<48).to_le_bytes());
 let (aid,_)=ls.write_chunk(&a).unwrap();let (pid,_)=ls.write_chunk(&vec![7u8;PAGE_SIZE]).unwrap();
 let mut am=manifest("A",vec![aid],vec![]);am.payload_pages=vec![pid];
 println!("A_OFFSET_2POW48 accepted={}",LayoutView::open(&ls,am).unwrap().record(0).is_ok());

 // An ordinary crash orphan is being republished while recover(false) owns
 // exclusive liveness. The publisher's CAS placement is outside that lock.
 let sr=root.join("recover-publish");let s=ManagedStore::open(&sr).unwrap();
 let payload=b"recover-publish-target".to_vec();let framed=codec::frame_chunk(ChunkKind::DeclSummary,&payload);let cid=ChunkId::from_parts(&[&framed]).0;
 let shard=sr.join("blocks/blobs/ch");fs::create_dir_all(&shard).unwrap();
 for i in 0..600 {fs::write(shard.join(format!("chk000000{:058x}",i)),b"orphan").unwrap();}
 fs::write(shard.join(&cid),&framed).unwrap();
 let cr=sr.clone();let rec=std::thread::spawn(move||store::recover(&ManagedStore::open(&cr).unwrap(),false));
 let lock=sr.join("locks/liveness.lock");let until=Instant::now()+Duration::from_secs(5);
 loop {if FileGuard::try_lock(&lock,false).unwrap().is_none(){break;}assert!(Instant::now()<until);std::thread::yield_now();}
 let pk=key("recover-target");let result=put(&s,pk.clone(),payload);
 let recovered=rec.join().unwrap().unwrap();
 println!("RECOVER_PUBLISH put_ok={} readable={} orphans_removed={}",result.is_ok(),s.get_artifact("A04",&pk).unwrap().is_some(),recovered.orphan_chunks_removed);

 // Same-process publishers use only millisecond+PID as their staging name.
 let sr=root.join("thread-publish");let s=std::sync::Arc::new(ManagedStore::open(&sr).unwrap());
 for round in 0..20 {
  let barrier=std::sync::Arc::new(std::sync::Barrier::new(5));let mut workers=vec![];
  for i in 0..4 {let ss=s.clone();let bb=barrier.clone();workers.push(std::thread::spawn(move||{let k=key(&format!("r{}-t{}",round,i));let p=vec![i as u8;65536];bb.wait();let result=put(&ss,k.clone(),p);(k,result)}));}
  barrier.wait();let mut failures=0;let mut bad=0;
  for w in workers {let (k,result)=w.join().unwrap();if result.is_err(){failures+=1;}else if !matches!(s.get_artifact("A04",&k),Ok(Some(_))){bad+=1;}}
  if failures>0||bad>0 {println!("THREAD_PUBLISH round={} errors={} successful_unreadable={}",round,failures,bad);break;}
 }
 // Recovery's new trash sweep must reject a reparse namespace root.
 let sr=root.join("trash-junction");let s=ManagedStore::open(&sr).unwrap();
 let external=root.join("external-owned-sentinel");fs::create_dir(&external).unwrap();
 let sentinel=external.join("precious.txt");fs::write(&sentinel,b"must survive").unwrap();
 let trash=sr.join("trash");fs::remove_dir(&trash).unwrap();
 let trash_arg=trash.display().to_string().replace('/',"\\");let external_arg=external.display().to_string().replace('/',"\\");
 let link=std::process::Command::new("cmd").args(["/C","mklink","/J",&trash_arg,&external_arg]).output().unwrap();assert!(link.status.success(),"junction setup failed: {:?}",link);
 assert!(store::is_reparse(&trash));
 let result=store::recover(&s,true);
 println!("TRASH_NAMESPACE_JUNCTION recover_ok={} external_sentinel_survived={}",result.is_ok(),sentinel.exists());
 fs::remove_dir(&trash).unwrap(); // Remove only the junction itself.
}
