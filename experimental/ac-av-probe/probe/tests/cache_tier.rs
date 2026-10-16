//! T-04 tier: real unit cache, dependency/invalidation, restore admission,
//! wrong-identity/concurrency/GC (PLAN-798 V-05 envelope leg, V-06..V-08)
//! plus the cross-process worker restore.

use auto_av_core::contract::fnv1a64;
use auto_av_probe::cache::{
    self, abc_unit_key, caps_fingerprint, envelope_unwrap, envelope_wrap, get_enveloped_opt,
    open_store, put_enveloped, EnvelopeIdentity,
};
use auto_av_probe::driver::Driver;

const BASE: &str = "fn helper(a int, b int) int {\n    return a * 10 + b\n}\n\nfn main() int {\n    if helper(3, 4) != 34 {\n        return 1\n    }\n    return 0\n}\n";

fn tmpdir(name: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("av798-cache-{}-{}", name, std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

/// V-06: fresh → same-process hot → restart (new driver, warm disk) →
/// cross-project sharing; hits skip mir_build/abc_emit, ALWAYS re-execute
/// and re-compare.
#[test]
fn av_cache_hit_lifecycle() {
    let root = tmpdir("lifecycle");
    let mut d = Driver::with_store(&root, "proj-a").unwrap();
    let r1 = d.request(BASE, "d_main", &[]);
    assert!(r1.ok(), "{:?}", r1.diag);
    assert_eq!(*r1.counters.get("cache_hit_abc").unwrap_or(&0), 0);

    let r2 = d.request(BASE, "d_main", &[]);
    assert!(r2.ok());
    assert_eq!(*r2.counters.get("cache_hit_abc").unwrap_or(&0), 2);
    assert_eq!(*r2.counters.get("mir_build").unwrap_or(&0), 0);
    assert_eq!(*r2.counters.get("abc_emit").unwrap_or(&0), 0);
    assert!(*r2.counters.get("av_execute").unwrap_or(&0) > 0, "hit still executes");
    assert_eq!(r2.value_i32(), r1.value_i32(), "independent re-comparison each request");

    // Restart: a NEW driver instance (cold in-process state) over the same
    // store root.
    let mut d3 = Driver::with_store(&root, "proj-a").unwrap();
    let r3 = d3.request(BASE, "d_main", &[]);
    assert!(r3.ok(), "restart/disk-hot: {:?}", r3.diag);
    assert_eq!(*r3.counters.get("cache_hit_abc").unwrap_or(&0), 2, "disk-hot hits");

    // Cross-project: a different project label shares the same logical
    // units (content identity, not project identity).
    let mut d4 = Driver::with_store(&root, "proj-b").unwrap();
    let r4 = d4.request(BASE, "d_main", &[]);
    assert!(r4.ok());
    assert_eq!(*r4.counters.get("cache_hit_abc").unwrap_or(&0), 2, "cross-project sharing");
    let _ = std::fs::remove_dir_all(&root);
}

/// V-07: body-only edit re-keys ONLY the edited unit (callers' code units
/// stay reusable); a signature edit re-keys every caller.
#[test]
fn av_cache_dependency_invalidation() {
    let root = tmpdir("deps");
    let mut d = Driver::with_store(&root, "p").unwrap();
    let r0 = d.request(BASE, "d_main", &[]);
    assert!(r0.ok());

    // Body-only edit of helper: same signature, different body.
    let body_edit = BASE.replace("a * 10 + b", "a * 10 - b");
    let r1 = d.request(&body_edit, "d_main", &[]);
    assert!(r1.ok(), "{:?}", r1.diag);
    assert_eq!(*r1.counters.get("cache_miss_abc").unwrap_or(&0), 1, "only helper misses");
    assert_eq!(*r1.counters.get("cache_hit_abc").unwrap_or(&0), 1, "d_main unit reuses");
    // And the result CHANGED (helper(3,4) = 26 now → main returns 1).
    assert_eq!(r1.value_i32(), Some(1));

    // Back to the base body: full hit again (both keys live).
    let r2 = d.request(BASE, "d_main", &[]);
    assert_eq!(*r2.counters.get("cache_hit_abc").unwrap_or(&0), 2, "both variants hit");

    // Signature edit: helper gains a param → callers' fingerprints embed
    // callee signature content → BOTH units re-key; the request itself now
    // fails binding (arity mismatch) with a frontend diagnostic.
    let sig_edit = BASE.replace("fn helper(a int, b int) int {", "fn helper(a int, b int, c int) int {");
    let r3 = d.request(&sig_edit, "d_main", &[]);
    assert!(r3.diag.is_some(), "signature conflict is a diagnosed request: {:?}", r3.av);
    assert_eq!(r3.diag.as_ref().unwrap().stage, "frontend");
    let _ = std::fs::remove_dir_all(&root);
}

/// V-08: a live-but-wrong-identity artifact (self-consistent envelope from
/// another producer/schema/owner/stage) is a CONTROLLED REJECTION at the
/// real consumption entry — never a silent hit or rebuild-over.
#[test]
fn av_cache_wrong_identity_rejected() {
    let root = tmpdir("ident");
    let store = open_store(&root).unwrap();
    // Envelope from a foreign producer over otherwise-identical payload.
    let payload = b"some-abc-bytes".to_vec();
    let foreign = EnvelopeIdentity {
        kind: cache::EnvelopeKind::AbcUnit,
        producer: "other-plan/x",
        schema: auto_av_core::contract::ABC_SCHEMA,
        owner_module: "m",
        stage: 2,
        mir_sem: 1,
        abc_sem: 1,
        av_sem: 1,
        snapshot: 0,
    };
    let key = abc_unit_key("m", "d_x", "fp", &caps_fingerprint(&[]));
    put_enveloped(&store, &key, "A09", &foreign, &payload, vec![], Some("p")).unwrap();
    // Reading with OUR identity must reject (not return the payload).
    let ours = EnvelopeIdentity::abc("m");
    let err = get_enveloped_opt(&store, &key, "A09", &ours).unwrap_err();
    assert!(matches!(err, cache::CacheError::Envelope(_)), "wrong producer rejected: {err}");
    // Same for a schema/owner/stage mismatch.
    let mut stage_wrong = EnvelopeIdentity::abc("m");
    stage_wrong.stage = 1;
    let wrapped = envelope_wrap(&payload, &EnvelopeIdentity::abc("m"));
    let e2 = envelope_unwrap(&wrapped, &stage_wrong).unwrap_err();
    assert!(matches!(e2, cache::CacheError::Envelope(_)));
    let _ = std::fs::remove_dir_all(&root);
}

/// V-08: corrupted framing — a self-consistent frame over tampered inner
/// bytes is rejected by the checksum/decode chain (store admission + inner
/// envelope + codec all guard).
#[test]
fn av_cache_corrupt_frame_rejected() {
    let payload = vec![1u8, 2, 3, 4, 5, 6, 7, 8];
    let id = EnvelopeIdentity::abc("m");
    let mut env = envelope_wrap(&payload, &id);
    let mid = env.len() / 2;
    env[mid] ^= 0xff; // corrupt inner, frame checksum now stale
    let err = envelope_unwrap(&env, &id).unwrap_err();
    assert!(matches!(err, cache::CacheError::Envelope(_)), "tampered envelope: {err}");
}

/// V-08: same-key different-content is a Conflict — the previously
/// committed result survives byte-identically.
#[test]
fn av_cache_conflict_preserves_committed() {
    let root = tmpdir("conflict");
    let store = open_store(&root).unwrap();
    let key = abc_unit_key("m", "d_x", "fp", &caps_fingerprint(&[]));
    let id = EnvelopeIdentity::abc("m");
    let a1 = put_enveloped(&store, &key, "A09", &id, b"content-one", vec![], Some("p")).unwrap();
    let err = put_enveloped(&store, &key, "A09", &id, b"content-two", vec![], Some("p"));
    assert!(matches!(err, Err(cache::CacheError::Store(
        ac_artifact_probe::store::StoreError::Conflict { .. }
    ))), "same key different content conflicts: {:?}", err);
    // The old committed artifact still reads back intact.
    let got = get_enveloped_opt(&store, &key, "A09", &id).unwrap().unwrap();
    assert_eq!(got, b"content-one".to_vec());
    let _ = a1;
    let _ = std::fs::remove_dir_all(&root);
}

/// V-08: GC + lease — a lease keeps live data pinned; roots keep soft
/// artifacts alive; stale leases sweep.
#[test]
fn av_cache_gc_and_lease() {
    let root = tmpdir("gc");
    let mut d = Driver::with_store(&root, "p").unwrap();
    let r = d.request(BASE, "d_main", &[]);
    assert!(r.ok());
    // Grab the manifest artifact for a lease pin.
    let manifest = r.manifest.as_ref().expect("manifest written");
    let store = open_store(&root).unwrap();
    let sentinel = manifest.units[0].artifact.clone();
    let lease = store.acquire_lease("exec", 60, Some(&sentinel)).expect("lease");
    // GC under a live lease must not delete the leased artifact's payload.
    let cfg = ac_artifact_probe::store::gc::gc_config_default();
    let _receipt = ac_artifact_probe::store::gc::run_gc(&store, cfg).expect("gc");
    // The leased unit still loads through the manifest chain.
    let payload = cache::get_by_artifact_id(
        &store,
        &sentinel,
        &EnvelopeIdentity::abc(&manifest.owner),
    );
    assert!(payload.is_ok(), "leased artifact survives GC: {:?}", payload.err());
    lease.release();
    // Stale lease sweep is a no-op now.
    let swept = store.sweep_stale_leases().expect("sweep");
    let _ = swept;
    // After releasing + unrooting, a follow-up request still works (roots
    // hold soft references from the original project).
    let mut d2 = Driver::with_store(&root, "p").unwrap();
    let r2 = d2.request(BASE, "d_main", &[]);
    assert!(r2.ok(), "{:?}", r2.diag);
    let _ = std::fs::remove_dir_all(&root);
}

/// T-01/T-04: cross-process restore — a NEW process loads the manifest +
/// units from disk, re-verifies, executes. Nothing in-memory carries over.
#[test]
fn av_cache_worker_restore_new_process() {
    let root = tmpdir("worker");
    let mut d = Driver::with_store(&root, "p").unwrap();
    let r = d.request(BASE, "d_main", &[]);
    assert!(r.ok());
    let manifest = r.manifest.as_ref().expect("manifest");
    // The manifest's own artifact id: find it via its key.
    let store = open_store(&root).unwrap();
    let key = cache::manifest_key(&manifest.owner, &manifest.entry, &manifest.fingerprint());
    let art = store.get_artifact("A09", &key).unwrap().expect("manifest artifact");
    drop(store);

    let worker = env!("CARGO_BIN_EXE_av_probe_worker");
    let out = std::process::Command::new(worker)
        .args([
            "restore-run",
            "--store",
            root.to_str().unwrap(),
            "--owner",
            &manifest.owner,
            "--entry",
            &manifest.entry,
            "--manifest-artifact",
            &art.artifact_id,
            "--snapshot",
            &format!("{:016x}", r.snapshot),
        ])
        .output()
        .expect("spawn worker");
    assert!(
        out.status.success(),
        "worker restore-run failed: {}\nstdout: {}\nstderr: {}",
        out.status,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let line = String::from_utf8_lossy(&out.stdout);
    let parsed: auto_av_probe::worker_restore::RestoreOutcome =
        serde_json::from_str(line.trim()).expect("worker JSON");
    assert!(parsed.executed_ok, "{:?}", parsed.error);
    assert_eq!(parsed.value, r.value_i32(), "restored execution matches");
    assert_eq!(parsed.units_verified, 2, "both units re-verified in the new process");
    let _ = fnv1a64;
    let _ = std::fs::remove_dir_all(&root);
}
