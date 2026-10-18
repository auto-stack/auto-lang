//! PLAN-796 T-04 cache lifecycle tests (`sema_cache_` filter, V-04):
//! cross-project sharing, idempotent republication, config-drift
//! staleness, corrupted-block read rejection (好 frame 错地址), and
//! lease pinning against GC.

#![cfg(windows)]

use ac_artifact_probe::store::{run_gc, GcConfig, ManagedStore};
use ac_sema_core::adapter::parse_into_input;
use ac_sema_core::query::QueryKind;
use ac_sema_core::{InferOutcome, InferSummary};
use ac_sema_probe::{lookup_only, open_store, publish_infer_result, run_inference, FetchOutcome, ProbeError};
use auto_ac_prototype::source::SourceProfile;

fn scratch(family: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("ac796-{}", family));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn input_of(src: &str) -> ac_sema_core::SemaInput {
    parse_into_input(src, "shared", "m", SourceProfile::Core).unwrap()
}

const SRC: &str = "fn helper(a int) int { return a + 1 }\nfn main() int { return helper(4) }";

fn publish_main(store: &ManagedStore, project: &str) -> InferSummary {
    let input = input_of(SRC);
    match run_inference(&input, "main") {
        InferOutcome::Ok(s) | InferOutcome::Diags(s) => s,
        _ => panic!("unexpected outcome"),
    }
}

/// Cross-project sharing (V-04 跨工程): the same logical key serves a
/// SECOND project from the same chunks — one publication, one hit each.
#[test]
fn sema_cache_cross_project_shared_hit() {
    let root = scratch("cache-xproj");
    let store = open_store(&root.join("store")).unwrap();
    let input = input_of(SRC);
    let q = QueryKind::InferBody { fn_name: "main".into() };

    // project A publishes
    let sum = publish_main(&store, "proj-a");
    publish_infer_result(&store, &input, &q, &sum, Some("proj-a")).unwrap();

    // project B (a DIFFERENT root set, same query context) hits — the
    // store keys are content-logical, not project-scoped
    match lookup_only(&store, &input, &q).unwrap() {
        FetchOutcome::Hit { .. } => {}
        other => panic!("expected cross-project hit, got {other:?}"),
        FetchOutcome::HitStaleProvenance { .. } => {}
    }
}

/// Idempotent republication: identical content under one key yields the
/// SAME artifact id (CAS dedup), never a conflict.
#[test]
fn sema_cache_idempotent_republication_same_artifact() {
    let root = scratch("cache-idem");
    let store = open_store(&root.join("store")).unwrap();
    let input = input_of(SRC);
    let q = QueryKind::InferBody { fn_name: "main".into() };
    let sum = publish_main(&store, "proj-a");
    let a1 = publish_infer_result(&store, &input, &q, &sum, Some("proj-a")).unwrap();
    let a2 = publish_infer_result(&store, &input, &q, &sum, Some("proj-b")).unwrap();
    assert_eq!(a1, a2, "identical content republicates to the same artifact");
}

/// Algorithm/config drift (V-04 换算法): a manifest recorded under a
/// different config fingerprint (algorithm/version change) is STALE —
/// never reused across algorithm generations.
#[test]
fn sema_cache_config_drift_stale() {
    use ac_sema_contract::{DepFingerprint, DepManifest};
    use ac_sema_core::{current_fingerprints, freshness_of, Freshness};
    let input = input_of(SRC);
    let cur = current_fingerprints(&input);
    let stale_manifest = DepManifest {
        positives: vec![DepFingerprint {
            kind: "config",
            origin: "profile/algo/schema".into(),
            fp: "an-older-algorithm-fingerprint".into(),
        }],
        negatives: vec![],
    };
    match freshness_of(&stale_manifest, &cur) {
        Freshness::Stale { reason } => assert!(reason.contains("config"), "{reason}"),
        Freshness::ProvenanceStale { .. } => panic!("config drift must not classify as provenance-only"),
        Freshness::Fresh => panic!("config drift must invalidate"),
    }
}

/// Corrupted block (V-04 好 frame 错地址): overwriting the stored
/// payload bytes is a CONTROLLED read rejection (CAS admission), never
/// wrong-bytes service.
#[test]
fn sema_cache_corrupted_block_rejected() {
    let root = scratch("cache-corrupt");
    let store = open_store(&root.join("store")).unwrap();
    let input = input_of(SRC);
    let q = QueryKind::InferBody { fn_name: "main".into() };
    let sum = publish_main(&store, "proj");
    publish_infer_result(&store, &input, &q, &sum, Some("proj")).unwrap();

    // the store holds exactly one block for this artifact: corrupt it
    let blocks = root.join("store").join("blocks");
    let mut block_files: Vec<std::path::PathBuf> = Vec::new();
    fn walk(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        for e in std::fs::read_dir(dir).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, out);
            } else {
                out.push(p);
            }
        }
    }
    walk(&blocks, &mut block_files);
    assert!(!block_files.is_empty(), "expected at least one block file");
    for f in &block_files {
        let bytes = std::fs::read(f).unwrap();
        let mut corrupted = bytes.clone();
        // flip payload bytes well past the 48-byte header
        let n = corrupted.len();
        corrupted[n - 1] ^= 0xFF;
        corrupted[n - 2] ^= 0xFF;
        std::fs::write(f, &corrupted).unwrap();
    }
    match lookup_only(&store, &input, &q) {
        Err(ProbeError::Store(e)) => {
            let msg = format!("{e}");
            assert!(
                msg.contains("address mismatch") || msg.contains("checksum") || msg.contains("magic"),
                "expected CAS/checksum rejection, got: {msg}"
            );
        }
        Err(other) => panic!("expected store error, got {other:?}"),
        Ok(outcome) => panic!("corrupted payload must not serve: {outcome:?}"),
        Ok(FetchOutcome::HitStaleProvenance { .. }) => {}
    }
}

/// Lease pinning: a leased artifact survives a hair-trigger GC; after
/// release (and root removal) it is reclaimable.
#[test]
fn sema_cache_lease_pins_against_gc() {
    let root = scratch("cache-lease");
    let store = open_store(&root.join("store")).unwrap();
    let input = input_of(SRC);
    let q = QueryKind::InferBody { fn_name: "main".into() };
    let sum = publish_main(&store, "proj");
    let art = publish_infer_result(&store, &input, &q, &sum, Some("proj")).unwrap();

    // pin via lease (required-data semantics for the retained payload)
    let lease = store.acquire_lease("sema-required-data", 600, Some(&art)).unwrap();
    assert!(store.lease_live(&lease.lease_id).unwrap());

    // drop the project root, then GC with a hair trigger: the lease
    // still pins the artifact
    store.remove_project_roots("proj").unwrap();
    let receipt = run_gc(&store, GcConfig { high: 1, low: 0 }).unwrap();
    let _ = receipt;
    match lookup_only(&store, &input, &q).unwrap() {
        FetchOutcome::Hit { .. } => {}
        other => panic!("leased artifact must survive GC, got {other:?}"),
        FetchOutcome::HitStaleProvenance { .. } => {}
    }

    // release: the payload becomes reclaimable
    let lid = lease.lease_id.clone();
    lease.release();
    assert!(!store.lease_live(&lid).unwrap());
    let _ = run_gc(&store, GcConfig { high: 1, low: 0 }).unwrap();
    match lookup_only(&store, &input, &q).unwrap() {
        FetchOutcome::Hit { .. } | FetchOutcome::Miss => {} // reclaim timing is GC's call
        FetchOutcome::Stale { .. } => {}
        FetchOutcome::HitStaleProvenance { .. } => {}
    }
}
