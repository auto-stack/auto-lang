// PLAN-783 r1 final-review R1; standalone consumer, no implementation edits.
// Compile against the frozen probe release rlib; see review-r1.md.
use ac_artifact_probe::{codec::{self, ChunkKind}, identity::{self, QueryKey}, layout::{self, LayoutStore, LayoutView, Manifest, PAGE_SIZE}, schema, store::{self, ManagedStore, PutRequest, GcConfig}};
use std::{fs, path::Path};

fn key(s: &str) -> QueryKey {
    QueryKey::new(schema::layer("A04").unwrap(), vec![("case".into(), s.into())])
}
fn put(s: &ManagedStore, k: &QueryKey, bytes: &[u8]) -> String {
    s.put_artifact(PutRequest {layer:"A04", key:k.clone(), kind:ChunkKind::DeclSummary,
        payload:bytes.to_vec(), edges:vec![], project:None}).unwrap()
}
fn manifest(record: String, payload: String) -> Manifest {
    Manifest {layout:"A".into(), version:1, record_count:1, record_pages:vec![record],
        payload_pages:vec![payload], payload_len:8, data_pages:vec![], dir_pages:vec![]}
}
fn main() {
    let root = std::env::args().nth(1).expect("unique NEW scratch root");
    let root = Path::new(&root);
    fs::create_dir(root).expect("must be new; this harness never recursively deletes");

    let s = ManagedStore::open(&root.join("republish")).unwrap();
    let k = key("republish");
    put(&s,&k,b"original");
    assert!(s.get_artifact("A04",&k).unwrap().is_some());
    let gc = store::run_gc(&s, GcConfig{high:1, low:0}).unwrap();
    assert_eq!(gc.deleted_chunks,1);
    put(&s,&k,b"original");
    println!("R1 republish successful but readable={}", s.get_artifact("A04",&k).unwrap().is_some());

    let s = ManagedStore::open(&root.join("cas")).unwrap();
    let k1=key("one"); let k2=key("two");
    put(&s,&k1,b"first"); put(&s,&k2,b"other");
    let a=s.get_artifact("A04",&k1).unwrap().unwrap();
    let b=s.get_artifact("A04",&k2).unwrap().unwrap();
    let path=|cid:&str| s.root.join("blocks/blobs").join(&cid[..2]).join(cid);
    fs::copy(path(&b.chunk_id),path(&a.chunk_id)).unwrap();
    let loaded=s.get_artifact("A04",&k1).unwrap().unwrap();
    println!("R2 CAS swapped same-kind frame accepted: expected=first actual={}", String::from_utf8_lossy(&loaded.payload));

    let ls=LayoutStore::new(&root.join("layout"));
    let mut record=vec![0u8;PAGE_SIZE];
    record[14..16].copy_from_slice(&8u16.to_le_bytes());
    let (rid,_)=ls.write_chunk(&record).unwrap();
    let (pid,_)=ls.write_chunk(&vec![7u8;PAGE_SIZE]).unwrap();
    let m=manifest(rid.clone(),pid);
    let mut v=LayoutView::open(&ls,m.clone());
    assert_eq!(v.record(0).unwrap().kind,0);
    let rp=ls.page_path(&rid);
    let mut bytes=fs::read(&rp).unwrap(); bytes[codec::CHUNK_HEADER_LEN]=99;
    fs::write(&rp,&bytes).unwrap();
    assert!(layout::validate_page_chunk(&ls,&rid).is_err());
    let mut v=LayoutView::open(&ls,m);
    println!("R3 corrupt page reader accepted kind={}",v.record(0).unwrap().kind);
    // A checksum-valid but structurally invalid offset must be Err, never panic.
    record[16..24].copy_from_slice(&((PAGE_SIZE-1) as u64).to_le_bytes());
    let (badid,_)=ls.write_chunk(&record).unwrap();
    let (pid,_)=ls.write_chunk(&vec![7u8;PAGE_SIZE]).unwrap();
    let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        LayoutView::open(&ls,manifest(badid,pid)).record(0)
    }));
    println!("R3 malformed offset panicked={}",result.is_err());

    let s = ManagedStore::open(&root.join("orphan-row")).unwrap();
    let k=key("conflict"); put(&s,&k,b"winner");
    assert!(s.put_artifact(PutRequest{layer:"A04",key:k,kind:ChunkKind::DeclSummary,
        payload:vec![9u8;1024],edges:vec![],project:None}).is_err());
    let gc=store::run_gc(&s,GcConfig{high:1,low:0}).unwrap();
    let rec=store::recover(&s,true).unwrap();
    println!("R4 conflict after full GC+recovery blocks={} deleted={} orphan_removed={}",s.space().blocks_count,gc.deleted_chunks,rec.orphan_chunks_removed);

    let s=ManagedStore::open(&root.join("trash")).unwrap();
    fs::write(s.root.join("trash/pending-delete"),vec![0u8;1000]).unwrap();
    store::recover(&s,true).unwrap();
    println!("R4 recover retains trash bytes={}",s.space().trash);

    let s=ManagedStore::open(&root.join("contention")).unwrap();
    let payload=b"same chunk";
    let cid=identity::ChunkId::from_parts(&[&codec::frame_chunk(ChunkKind::DeclSummary,payload)]).0;
    let held=store::FileGuard::try_lock(&s.root.join(format!("locks/chunk-{}.lock",cid)),true).unwrap().unwrap();
    let result=s.put_artifact(PutRequest{layer:"A04",key:key("busy"),kind:ChunkKind::DeclSummary,payload:payload.to_vec(),edges:vec![],project:None});
    println!("R5 same-chunk contender result={:?}",result);
    drop(held);

    // Lease acquired after the GC snapshot but before this candidate's turn.
    let sr=root.join("lease-race");
    let s=ManagedStore::open(&sr).unwrap();
    let mut keys=Vec::new();
    for i in 0u32..150 {
        let k=key(&format!("junk-{i}"));
        let mut p=vec![0u8;65536]; p[..4].copy_from_slice(&i.to_le_bytes());
        put(&s,&k,&p); keys.push(k);
    }
    std::thread::sleep(std::time::Duration::from_millis(1100));
    let target=key("lease-target");
    let aid=put(&s,&target,b"lease target survives");
    let gcroot=sr.clone();
    let worker=std::thread::spawn(move || {
        store::run_gc(&ManagedStore::open(&gcroot).unwrap(),GcConfig{high:1,low:0}).unwrap()
    });
    let limit=std::time::Instant::now()+std::time::Duration::from_secs(15);
    loop {
        if keys.iter().any(|k|s.get_artifact("A04",k).unwrap().is_none()) {break;}
        assert!(std::time::Instant::now()<limit,"GC did not start");
        std::thread::yield_now();
    }
    assert!(s.get_artifact("A04",&target).unwrap().is_some(),"target not yet processed");
    let lease=s.acquire_lease("during-gc",60,Some(&aid)).unwrap();
    s.register_root("new-live-project",&aid,"hard").unwrap();
    let gc=worker.join().unwrap();
    println!("R6 lease acquired+hard root during GC: lease_live={} target_readable={} deleted={}",
        s.lease_live(&lease.lease_id).unwrap(),s.get_artifact("A04",&target).unwrap().is_some(),gc.deleted_chunks);
}
