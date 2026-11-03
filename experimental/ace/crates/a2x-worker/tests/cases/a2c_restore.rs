//! PLAN-921 T-06/T-07 (V-05/06/07): the source-bound pure-restore chain
//! of the C scalar channel — anchor publish (cold + ranged warm rebind),
//! NEW-SESSION pure restore with P0/R>0 accounting, named rejections for
//! bad sources / forged anchors / tampered payloads (nothing installed on
//! failure), repair-then-restore, function-level warm-edit counters, and
//! lease+GC liveness of the anchored set.
//!
//! The new-PROCESS witness of the same chain (real cross-process restore
//! + next body edit) is the `a2c-formal` formal leg; this file proves the
//! session/store contracts in-process with fresh sessions per store.

use ace_a2x_c::CTargetProvider;
use ace_a2x_rust::RustTargetProvider;
use ace_a2x_store::A2xSession;
use ace_core::source::SourceProfile;
use std::path::PathBuf;

fn root(tag: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../target/a2x-out-c/restore-{}", tag))
}

const SRC: &str = "fn leaf(x int) int { return x * 2 }\nfn mid(x int) int { return leaf(x) + 1 }\nfn other(x int) int { return x - 1 }\nfn main() int { return mid(other(20)) }";

fn open_c(root: &PathBuf) -> A2xSession {
    A2xSession::open(
        root,
        "a2c-restore",
        "probe",
        128 * 1024 * 1024,
        CTargetProvider::v1(),
    )
    .unwrap()
}

/// V-06: cold publish → FRESH session restore is PURE (P0: zero
/// lower/print/persist; R>0: binding + payload + receipt verifies) and
/// byte-identical to the cold assembly; the next body edit works (env
/// rebuild counted, edited-only production); restore of the EDITED
/// source is pure again (the rebind chain).
#[test]
fn c_restore_pure_p0_then_edit_then_restore_again() {
    let root = root("p0");
    let _ = std::fs::remove_dir_all(&root);
    // process 1 (writer): cold publish
    let mut w = open_c(&root);
    let cold = w
        .cold_assemble_profile(SRC, SourceProfile::Scalar, &[], "main", false)
        .unwrap();
    assert!(w.last_anchor().is_some(), "cold publishes the anchor");

    // process 2 (restorer): fresh session over the same store
    let mut r = open_c(&root);
    let before = r.stats().clone();
    let restored = r
        .restore_source_bound(SRC, SourceProfile::Scalar, &[], "main")
        .expect("pure restore");
    let after = r.stats().clone();
    // P0: NO production counters moved
    assert_eq!(after.lower, before.lower, "restore must not lower");
    assert_eq!(after.print, before.print, "restore must not print");
    assert_eq!(after.persist, before.persist, "restore must not persist");
    assert_eq!(after.parse, before.parse, "restore must not parse");
    assert_eq!(after.whole_fallback, before.whole_fallback);
    // R>0: real verification work happened
    assert!(after.restore_lookup > before.restore_lookup);
    assert!(
        after.restore_binding_checks >= before.restore_binding_checks + 4,
        "one binding check per function"
    );
    assert!(after.restore_payload_verifies > before.restore_payload_verifies);
    assert!(after.restore_fileset_verifies >= before.restore_fileset_verifies + 3);
    // byte-identical assembly
    assert_eq!(restored.files, cold.files, "restored files == cold files");
    assert_eq!(restored.manifest, cold.manifest);

    // next body edit works: leaf's body changes (ranged authorization)
    let leaf_frag = "fn leaf(x int) int { return x * 3 }";
    let (start, end) = (0, SRC.find('\n').unwrap());
    let edited = format!("{}{}", leaf_frag, &SRC[end..]);
    let mut e = r;
    let before_edit = e.stats().clone();
    let asm = e
        .warm_edit_ranged("leaf", SRC, (start, end), leaf_frag, "main", false)
        .expect("warm edit after restore");
    let after_edit = e.stats().clone();
    assert_eq!(
        after_edit.env_rebuilds,
        before_edit.env_rebuilds + 1,
        "one env rebuild enables post-restore edits"
    );
    assert_eq!(
        after_edit.lower,
        before_edit.lower + 1,
        "exactly the edited function is produced"
    );
    assert_eq!(after_edit.print, before_edit.print + 1);
    assert_eq!(
        after_edit.persist,
        before_edit.persist + 3,
        "semantic + fragment + rebound anchor"
    );
    // sanity: the edit really took effect (60 vs 40)
    let dir = root.join("run-edit");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for (p, c) in &asm.files {
        std::fs::write(dir.join(p), c).unwrap();
    }
    let (out, _) = ace_a2x_worker::crun::compile_and_run_c(&dir, "edit.exe").unwrap();
    assert_eq!(out, "value:58\n", "mid(other(20)) == leaf(19)*3 + 1 == 58");

    // restore of the EDITED source is pure again (the rebind chain)
    let mut r2 = open_c(&root);
    let restored2 = r2
        .restore_source_bound(&edited, SourceProfile::Scalar, &[], "main")
        .expect("restore of the edited source");
    assert_eq!(r2.stats().lower, 0, "still P0");
    assert!(r2.stats().restore_binding_checks >= 4);
    assert_eq!(restored2.files, asm.files, "restored == edited assembly");
}

/// V-06 negative controls: a BAD SOURCE (no anchor for its sha) and a
/// CONTEXT MISMATCH (caps/entry) are named rejections that install
/// NOTHING. Same-key anchor tampering is structurally impossible (the
/// store refuses different content under one key — content-addressed
/// immutability), which this test re-proves; the repair leg shows re-cold
/// republication stays idempotent and restore pure.
#[test]
fn c_restore_rejects_and_repairs() {
    let root = root("neg");
    let _ = std::fs::remove_dir_all(&root);
    let mut w = open_c(&root);
    w.cold_assemble_profile(SRC, SourceProfile::Scalar, &[], "main", false)
        .unwrap();

    // bad source: a different source has no anchor
    let bad_src = "fn main() int { return 1 }";
    let mut r = open_c(&root);
    let err = r
        .restore_source_bound(bad_src, SourceProfile::Scalar, &[], "main")
        .expect_err("no anchor for a different source");
    match err {
        ace_a2x_store::A2xRejectOrResource::Reject(j) => {
            assert_eq!(j.code, "a2x.restore.no-anchor", "got: {}", j);
        }
        other => panic!("expected Reject, got {:?}", other),
    }
    // nothing was installed: a warm edit cannot proceed (no env, no
    // residents bound to the bad source)
    let err = r
        .warm_edit_ranged(
            "leaf",
            bad_src,
            (0, 17),
            "fn leaf(x int) int { return 9 }",
            "main",
            false,
        )
        .expect_err("nothing installed after a failed restore");
    assert!(matches!(err, ace_a2x_store::A2xRejectOrResource::Reject(_)));

    // context mismatch: same source, different caps → named context reject
    let mut rc = open_c(&root);
    let err = rc
        .restore_source_bound(
            SRC,
            SourceProfile::Scalar,
            &["trace.hir.test".to_string()],
            "main",
        )
        .expect_err("caps mismatch must reject");
    match err {
        ace_a2x_store::A2xRejectOrResource::Reject(j) => {
            assert_eq!(j.code, "a2x.restore.context", "got: {}", j);
        }
        other => panic!("expected Reject, got {:?}", other),
    }
    // entry mismatch: a different entry is a different anchor key
    let err = rc
        .restore_source_bound(SRC, SourceProfile::Scalar, &[], "leaf")
        .expect_err("entry mismatch must reject");
    match err {
        ace_a2x_store::A2xRejectOrResource::Reject(j) => {
            assert_eq!(j.code, "a2x.restore.no-anchor", "got: {}", j);
        }
        other => panic!("expected Reject, got {:?}", other),
    }

    // anchor immutability: different content under the SAME key is a
    // store-level conflict (never overwrite); identical republication is
    // idempotent — the anchor cannot be silently tampered through the API
    {
        let anchored = w.last_anchor().unwrap().clone();
        let store = ace_artifacts::store::ManagedStore::open(&root).unwrap();
        let key = ace_artifacts::identity::QueryKey::new(
            &ace_a2x_store::LAYER_SOURCE_ANCHOR,
            vec![
                ("owner".into(), "probe".into()),
                ("profile".into(), "scalar".into()),
                ("source_sha256".into(), anchored.source_sha256.clone()),
                ("backend".into(), "auto-a2x-c".into()),
                ("entry".into(), "main".into()),
            ],
        );
        let mut tampered = anchored.clone();
        tampered.functions[0].slice_sha256 = "0".repeat(64);
        let conflict = store
            .put_artifact(ace_artifacts::store::PutRequest {
                layer: "AXP1",
                key: key.clone(),
                kind: ace_artifacts::codec::ChunkKind::ObjectManifest,
                payload: tampered.encode().into_bytes(),
                edges: vec![],
                project: Some("a2c-restore".into()),
            })
            .expect_err("different content under one key must conflict");
        assert!(format!("{:?}", conflict).contains("Conflict"));
        // (byte-identical republication idempotency — same payload AND
        // same edges — is proven by the repair leg below: re-cold
        // republishes the same anchor without conflict)
    }

    // repair leg: re-cold republication is safe; restore stays pure
    let mut w3 = open_c(&root);
    w3.cold_assemble_profile(SRC, SourceProfile::Scalar, &[], "main", false)
        .unwrap();
    let mut r4 = open_c(&root);
    let ok = r4
        .restore_source_bound(SRC, SourceProfile::Scalar, &[], "main")
        .expect("repair keeps restore pure");
    assert_eq!(r4.stats().lower, 0);
    assert!(!ok.files.is_empty());
}

/// V-05: function-level warm editing — ≥3 unchanged functions keep
/// ZERO production (resident reuse), A→B→A revisits hit the store with
/// real historical bytes, and the whole-session accounting is per-edit.
#[test]
fn c_warm_edit_counters_and_aba() {
    let root = root("warm");
    let _ = std::fs::remove_dir_all(&root);
    let mut s = open_c(&root);
    s.cold_assemble_profile(SRC, SourceProfile::Scalar, &[], "main", false)
        .unwrap();
    let base = s.stats().clone();

    // A→B: edit leaf's body (ranged)
    let leaf_b = "fn leaf(x int) int { return x * 3 }";
    let (start, end) = (0, SRC.find('\n').unwrap());
    s.warm_edit_ranged("leaf", SRC, (start, end), leaf_b, "main", false)
        .unwrap();
    let after_b = s.stats().clone();
    assert_eq!(after_b.lower - base.lower, 1);
    assert_eq!(after_b.print - base.print, 1);
    assert_eq!(after_b.persist - base.persist, 3);

    // B→A: back to the original body — a STORE hit (A's bytes are real
    // history), zero lowering
    let src_b = format!("{}{}", leaf_b, &SRC[end..]);
    let end_b = src_b.find('\n').unwrap();
    let asm_a = s
        .warm_edit_ranged("leaf", &src_b, (0, end_b), &SRC[..end], "main", false)
        .unwrap();
    let after_a = s.stats().clone();
    assert_eq!(
        after_a.lower, after_b.lower,
        "A→B→A revisits hit the store, no re-lowering"
    );
    assert!(after_a.store_hit_fragment > after_b.store_hit_fragment);
    // the A assembly is byte-identical to the cold one
    let mut cold = open_c(&root.join("cold-check"));
    let cold_asm = cold
        .cold_assemble_profile(SRC, SourceProfile::Scalar, &[], "main", true)
        .unwrap();
    assert_eq!(asm_a.files, cold_asm.files);
}

/// V-07: lease + GC keep the anchored set alive; restore works after a
/// GC pass under a lease. Read-only colds publish nothing.
#[test]
fn c_readonly_lease_gc_liveness() {
    let root = root("gc");
    let _ = std::fs::remove_dir_all(&root);
    // read-only: no anchor published
    let mut ro = open_c(&root.join("ro"));
    ro.cold_assemble_profile(SRC, SourceProfile::Scalar, &[], "main", true)
        .unwrap();
    assert!(ro.last_anchor().is_none(), "read-only publishes no anchor");
    assert_eq!(ro.stats().persist, 0);

    let mut s = open_c(&root);
    s.cold_assemble_profile(SRC, SourceProfile::Scalar, &[], "main", false)
        .unwrap();
    let lease = s
        .lease_current("restore-witness")
        .expect("lease the anchor");
    let receipt = s.gc().expect("gc runs");
    assert!(!receipt.is_empty());
    let mut r = open_c(&root);
    r.restore_source_bound(SRC, SourceProfile::Scalar, &[], "main")
        .expect("anchored set survives GC under the lease");
    drop(lease);
}

/// V-01 (mixed-target slice): the SAME session and store serve BOTH
/// targets — C and Rust fragments/anchors stay isolated per backend, and
/// a scalar Rust assembly also restores purely (the chain is shared, the
/// identities are not).
#[test]
fn c_and_rust_same_session_store_isolation() {
    let root = root("mixed");
    let _ = std::fs::remove_dir_all(&root);
    let mut s = A2xSession::open(
        &root,
        "mixed",
        "probe",
        128 * 1024 * 1024,
        CTargetProvider::v1(),
    )
    .unwrap();
    let c_asm = s
        .cold_assemble_profile(SRC, SourceProfile::Scalar, &[], "main", false)
        .unwrap();
    assert_eq!(s.current_backend(), "auto-a2x-c");
    let c_anchor = s.last_anchor().unwrap().clone();

    // same session, same store: switch to Rust
    s.set_target(RustTargetProvider::v1());
    assert_eq!(s.current_backend(), "auto-a2x-rust");
    let r_asm = s
        .cold_assemble_profile(SRC, SourceProfile::Scalar, &[], "main", false)
        .unwrap();
    let r_anchor = s.last_anchor().unwrap().clone();
    assert_ne!(c_anchor.backend, r_anchor.backend);
    assert_ne!(c_anchor.source_sha256, ""); // both bound the same source
    assert_eq!(c_anchor.source_sha256, r_anchor.source_sha256);
    // the C file set is C; the Rust file set is Rust — never cross-hit
    let c_names: Vec<&str> = c_asm.files.iter().map(|(p, _)| p.as_str()).collect();
    let r_names: Vec<&str> = r_asm.files.iter().map(|(p, _)| p.as_str()).collect();
    assert_eq!(c_names, vec!["main.c", "a2x_c_runtime.h", "generated.c"]);
    assert_eq!(r_names, vec!["main.rs", "a2x_runtime.rs", "generated.rs"]);

    // fresh session per backend restores ITS OWN anchor (never the other's)
    let mut rc = A2xSession::open(
        &root,
        "mixed",
        "probe",
        128 * 1024 * 1024,
        CTargetProvider::v1(),
    )
    .unwrap();
    rc.restore_source_bound(SRC, SourceProfile::Scalar, &[], "main")
        .expect("C restores its own anchor");
    let mut rr = A2xSession::open(
        &root,
        "mixed",
        "probe",
        128 * 1024 * 1024,
        RustTargetProvider::v1(),
    )
    .unwrap();
    rr.restore_source_bound(SRC, SourceProfile::Scalar, &[], "main")
        .expect("Rust restores its own anchor");
    assert_eq!(rc.stats().lower, 0);
    assert_eq!(rr.stats().lower, 0);
}
