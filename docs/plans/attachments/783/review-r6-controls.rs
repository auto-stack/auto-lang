// Independent sibling: CAS residue cannot be removed, so it must consume quota.
use ac_artifact_probe::{codec::{frame_chunk,ChunkKind},identity::{ChunkId,QueryKey},schema,store::{self,ManagedStore,PutRequest}};
use std::{fs,path::Path};
use std::os::windows::fs::OpenOptionsExt;
fn put(s:&ManagedStore,k:&str,p:&[u8])->Result<String,store::StoreError>{s.put_artifact(PutRequest{layer:"A04",key:QueryKey::new(schema::layer("A04").unwrap(),vec![("case".into(),k.into())]),kind:ChunkKind::DeclSummary,payload:p.to_vec(),edges:vec![],project:None})}
fn bytes(p:&Path,only_tmp:bool)->u64{fs::read_dir(p).map(|r|r.flatten().map(|e|if e.path().is_dir(){bytes(&e.path(),only_tmp)}else if !only_tmp || e.path().extension().is_some_and(|x|x=="tmp"){e.metadata().unwrap().len()}else{0}).sum()).unwrap_or(0)}
fn main(){
 let root=std::env::args().nth(1).unwrap();let s=ManagedStore::open(Path::new(&root)).unwrap();
 let payload=vec![11u8;3*1024*1024];let framed=frame_chunk(ChunkKind::DeclSummary,&payload);let cid=ChunkId::from_parts(&[&framed]).0;
 let dest=s.root.join("blocks/blobs").join(&cid[..2]).join(cid);fs::create_dir_all(dest.parent().unwrap()).unwrap();fs::write(&dest,&framed).unwrap();
 let h=fs::OpenOptions::new().read(true).share_mode(0).open(&dest).unwrap();
 let tmp=dest.with_extension("tmp");fs::write(&tmp,&framed).unwrap();
 // Allow BlobStore to rewrite the same tmp, while denying rename and deletion.
 let ht=fs::OpenOptions::new().read(true).share_mode(3).open(&tmp).unwrap();
 assert!(put(&s,"failed",&payload).is_err());
 let leftover=bytes(&s.root.join("blocks"),true);assert_eq!(leftover,framed.len() as u64);
 let next=put(&s,"too-large",&vec![12u8;3*1024*1024]);assert!(matches!(next,Err(store::StoreError::ResourceExhausted(_))));
 assert!(put(&s,"small",b"small legal write").is_ok());
 let total=bytes(&s.root.join("staging"),false)+bytes(&s.root.join("blocks"),true);assert!(total<=ManagedStore::STAGING_BUDGET);
 println!("BLOCKED_CLEANUP residual={leftover} large_rejected=true small_succeeded=true total={total} cap={}",ManagedStore::STAGING_BUDGET);
 drop(ht);drop(h);
 let recovered=store::recover(&s,true).unwrap();
 assert!(put(&s,"after-recover",&vec![13u8;3*1024*1024]).is_ok());
 println!("AFTER_RECOVER tmp={} ordinary_3mib_succeeded=true recovery={recovered:?}",bytes(&s.root.join("blocks"),true));
}
