// R5: an exhausted CAS rename retry must not evade the total transient cap.
use ac_artifact_probe::{codec::{frame_chunk,ChunkKind},identity::{ChunkId,QueryKey},schema,store::{self,ManagedStore,PutRequest}};
use std::{fs,path::Path,sync::Arc,time::Duration};
use std::os::windows::fs::OpenOptionsExt;
fn key(s:&str)->QueryKey{QueryKey::new(schema::layer("A04").unwrap(),vec![("case".into(),s.into())])}
fn put(s:&ManagedStore,k:&str,p:&[u8])->Result<String,store::StoreError>{s.put_artifact(PutRequest{layer:"A04",key:key(k),kind:ChunkKind::DeclSummary,payload:p.to_vec(),edges:vec![],project:None})}
fn held_dest(s:&ManagedStore,p:&[u8])->fs::File{let f=frame_chunk(ChunkKind::DeclSummary,p);let c=ChunkId::from_parts(&[&f]).0;let b=s.root.join("blocks/blobs").join(&c[..2]).join(c);fs::create_dir_all(b.parent().unwrap()).unwrap();fs::write(&b,&f).unwrap();fs::OpenOptions::new().read(true).share_mode(0).open(b).unwrap()}
fn bytes(p:&Path,only_tmp:bool)->u64{fs::read_dir(p).map(|r|r.flatten().map(|e|if e.path().is_dir(){bytes(&e.path(),only_tmp)}else if !only_tmp || e.path().extension().is_some_and(|x|x=="tmp"){e.metadata().unwrap().len()}else{0}).sum()).unwrap_or(0)}
fn main(){
 let root=std::env::args().nth(1).expect("unique unused scratch root");let s=Arc::new(ManagedStore::open(Path::new(&root)).unwrap());
 let a=vec![31u8;3*1024*1024];let b=vec![32u8;3*1024*1024];
 let h1=held_dest(&s,&a);let first=put(&s,"failed-first",&a);assert!(first.is_err());drop(h1);
 println!("FIRST_FAILED error={first:?} staging={} residual_cas_tmp={}",bytes(&s.root.join("staging"),false),bytes(&s.root.join("blocks"),true));
 let h2=held_dest(&s,&b);let ss=s.clone();let worker=std::thread::spawn(move||put(&ss,"second",&b));
 std::thread::sleep(Duration::from_millis(350));
 let staged=bytes(&s.root.join("staging"),false);let tmp=bytes(&s.root.join("blocks"),true);
 println!("FAILED_THEN_NEXT staging={staged} cas_tmp={tmp} total={} cap={} within_cap={} reported_reservation_peak={}",staged+tmp,ManagedStore::STAGING_BUDGET,staged+tmp<=ManagedStore::STAGING_BUDGET,store::PEAK_TRANSIENT.load(std::sync::atomic::Ordering::Relaxed));
 drop(h2);println!("SECOND_RESULT {:?}",worker.join().unwrap());
}
