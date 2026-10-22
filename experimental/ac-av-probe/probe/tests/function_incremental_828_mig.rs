//! PLAN-828 T07-R3-01 permanent controls — the legacy→required-edge
//! manifest identity migration (collected from the R3 review's
//! migration-controls.rs, same fixture recipe and oracles).
//!
//! The shared store's artifact id includes the edge set and one layer+key
//! admits exactly one artifact identity: a required-edge manifest (key
//! format "2") never collides with the pre-R2 edge-free row (key format
//! "1", same body). Legacy rows stay readable through their ROOT ids,
//! still gate warm restores, and retire when the new graph registers;
//! every legacy path — partial cold fallback, full warm read, reopen,
//! same-content edit, then GC and another restore — keeps persisting.

use ac_artifact_probe::identity::QueryKey;
use auto_av_probe::cache::{
    abc_unit_key, caps_fingerprint, get_by_artifact_id, get_enveloped_opt, open_store,
    payload_fingerprint, put_enveloped, unit_span_sidecar_key, EnvelopeIdentity, ABC_LAYER,
};
use auto_av_probe::session::{restore, warmup_persisted};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const SRC: &str =
    "fn helper() int { return 7 }\nfn main() int { return helper() + 2 }\nfn tail() int { return 9 }\n";
const P: &str = "p828-r3-perm-migration";

fn dir(t: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!(
        "p828-r3perm-{t}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&p).unwrap();
    p
}

/// The exact pre-R2 producer recipe: an edge-free manifest row under the
/// LEGACY key (format "1") with a registered root — what an old cache
/// holds on disk. Everything through public store APIs.
fn legacy_manifest(t: &str) -> PathBuf {
    let seed = dir("current-recipe");
    let s = warmup_persisted(SRC, "d_main", &[], &seed, P).unwrap();
    let owner = s.current().code()[0].unit().module().owner.module.clone();
    drop(s);
    let st = open_store(&seed).unwrap();
    let id = st
        .root_artifacts(P, "session-manifest")
        .unwrap()
        .pop()
        .unwrap();
    let body = get_by_artifact_id(&st, &id, &EnvelopeIdentity::manifest(&owner, 0)).unwrap();
    let target = dir(t);
    let st = open_store(&target).unwrap();
    let key = QueryKey::new(
        &ABC_LAYER,
        vec![
            (
                "producer".into(),
                auto_av_core::contract::PRODUCER_ID.into(),
            ),
            ("owner".into(), owner.clone()),
            ("kind".into(), "session-manifest".into()),
            ("format".into(), "1".into()),
            ("entry".into(), "d_main".into()),
            ("fp".into(), payload_fingerprint(&body)),
        ],
    );
    let old = put_enveloped(
        &st,
        &key,
        "A09",
        &EnvelopeIdentity::manifest(&owner, 0),
        &body,
        vec![],
        Some(P),
    )
    .unwrap();
    st.register_root(P, &old, "session-manifest").unwrap();
    target
}

/// A legacy manifest plus the full legacy ABC/sidecar closure (rows
/// copied with the pre-R2 recipe: no edges, per-row soft roots).
fn full_legacy_store() -> PathBuf {
    let p = legacy_manifest("full-legacy");
    let seed = dir("full-copy");
    let s = warmup_persisted(SRC, "d_main", &[], &seed, P).unwrap();
    let owner = s.current().code()[0].unit().module().owner.module.clone();
    let from = open_store(&seed).unwrap();
    let to = open_store(&p).unwrap();
    let root = from
        .root_artifacts(P, "session-manifest")
        .unwrap()
        .pop()
        .unwrap();
    let body = get_by_artifact_id(&from, &root, &EnvelopeIdentity::manifest(&owner, 0)).unwrap();
    let manifest: serde_json::Value = serde_json::from_slice(&body).unwrap();
    for name in ["helper", "main", "tail"] {
        let def = format!("d_{name}");
        let cv = manifest["units"]
            .as_array()
            .unwrap()
            .iter()
            .find(|u| u["def_id"].as_str() == Some(&def))
            .unwrap()["cv"]
            .as_str()
            .unwrap();
        let key = abc_unit_key(&owner, &def, cv, &caps_fingerprint(&[]));
        let id = EnvelopeIdentity::abc_bound(&owner, &def, cv);
        let b = get_enveloped_opt(&from, &key, "A09", &id).unwrap().unwrap();
        put_enveloped(&to, &key, "A09", &id, &b, vec![], Some(P)).unwrap();
        let key = unit_span_sidecar_key(&owner, &def, cv);
        let id = EnvelopeIdentity::unit_span_sidecar(&owner, &def, cv);
        let b = get_enveloped_opt(&from, &key, "A09", &id).unwrap().unwrap();
        put_enveloped(&to, &key, "A09", &id, &b, vec![], Some(P)).unwrap();
    }
    p
}

/// R3 control 1: a legacy manifest without units keeps the cold-fallback
/// self-healing path — and the healed store restores warm afterwards.
#[test]
fn migration_legacy_manifest_missing_units_cold_fallback_self_heals() {
    let p = legacy_manifest("legacy-restore");
    let r = restore(SRC, "d_main", &[], &p, P)
        .expect("a partial legacy store must retain the cold fallback");
    drop(r);
    let r2 = restore(SRC, "d_main", &[], &p, P).unwrap();
    assert_eq!(
        r2.counters().get("restore_units_admitted"),
        3,
        "the healed store restores warm on the next open"
    );
}

/// R3 control 2: warmup_persisted directly over a legacy row succeeds
/// (the new required graph takes its own key; the old row is untouched).
#[test]
fn migration_legacy_manifest_republishes_without_same_key_edge_conflict() {
    let p = legacy_manifest("legacy-warmup");
    warmup_persisted(SRC, "d_main", &[], &p, P)
        .expect("an old edge-free v1 row must be isolated without blocking persistence");
}

/// R3 control 3: a COMPLETE legacy closure warms (the legacy row is a
/// valid directory), and a reopen over the same store persists fine.
#[test]
fn migration_complete_legacy_store_warm_reads_and_reopens() {
    let p = full_legacy_store();
    let r = restore(SRC, "d_main", &[], &p, P).unwrap();
    assert_eq!(
        r.counters().get("restore_units_admitted"),
        3,
        "premise: the exact legacy ABC/sidecar rows form a readable warm store"
    );
    drop(r);
    warmup_persisted(SRC, "d_main", &[], &p, P)
        .expect("the new required graph must not conflict with the legacy row");
}

/// R3 control 4: after a warm read of a complete legacy store, a REAL
/// same-content edit (history hit) publishes the new required graph —
/// persist_error stays 0.
#[test]
fn migration_complete_legacy_store_same_content_edit_keeps_persistence() {
    let p = full_legacy_store();
    let mut s = restore(SRC, "d_main", &[], &p, P).unwrap();
    assert_eq!(s.counters().get("restore_units_admitted"), 3);
    let at = SRC.find("return 7").unwrap() as u64 + 7;
    let out = s.edit(at, at, b"   ".to_vec()).unwrap();
    assert!(out.ok, "{out:?}");
    assert_eq!(s.counters().get("reuse_hit:helper"), 1);
    assert_eq!(
        s.counters().get("persist_error"),
        0,
        "same-content edits must publish the new required graph, not strand it"
    );
}

/// R3 control 5: the fresh required graph is idempotent — reopen and a
/// same-content edit keep persist_error 0.
#[test]
fn migration_fresh_required_manifest_reopen_and_same_content_edit_positive() {
    let p = dir("fresh-positive");
    drop(warmup_persisted(SRC, "d_main", &[], &p, P).unwrap());
    let mut s = warmup_persisted(SRC, "d_main", &[], &p, P).unwrap();
    let at = SRC.find("return 7").unwrap() as u64 + 7;
    assert!(s.edit(at, at, b"   ".to_vec()).unwrap().ok);
    assert_eq!(s.counters().get("reuse_hit:helper"), 1);
    assert_eq!(s.counters().get("persist_error"), 0);
}

/// The full migration PATH the task list requires: legacy store → warm
/// read → same-content edit (new graph published, legacy root retired) →
/// REAL GC → restore again with correct value and source coordinates.
#[test]
fn migration_legacy_to_new_graph_through_gc_then_restore() {
    use ac_artifact_probe::store::{run_gc, GcConfig};
    let p = full_legacy_store();
    let mut s = restore(SRC, "d_main", &[], &p, P).unwrap();
    assert_eq!(s.counters().get("restore_units_admitted"), 3);
    let at = SRC.find("return 7").unwrap() as u64 + 7;
    let out = s.edit(at, at, b"   ".to_vec()).unwrap();
    assert!(out.ok);
    assert_eq!(s.counters().get("persist_error"), 0);
    let edited = s.src().to_string();
    drop(s);
    {
        let st = open_store(&p).unwrap();
        assert_eq!(
            st.root_artifacts(P, "session-manifest").unwrap().len(),
            1,
            "the legacy manifest root retired when the new graph registered"
        );
        let receipt = run_gc(&st, GcConfig { high: 1, low: 0 }).unwrap();
        assert!(receipt.ran);
    }
    let r = restore(&edited, "d_main", &[], &p, P).unwrap();
    assert_eq!(r.counters().get("restore_units_admitted"), 3);
    let (out, _) = r.run(
        &auto_av_core::av::ExecLimits::default(),
        &auto_av_core::av::CancelFlag::new(false),
    );
    assert!(matches!(
        out,
        auto_av_core::contract::ExecOutcome::Success {
            value: auto_av_core::contract::Value::I32(9),
            ..
        }
    ));
}

/// T07-R3-01 residual (collected from the R4 review's
/// rebuilt-closure-controls.rs, same premises/oracles): a MISSING cache
/// block passed through the real store `recover` is a normal cold-fallback
/// input. The positive keeps the source unchanged; the negative inserts
/// five spaces into `return 7` — same content semantics, same CVs, a
/// fresh reference store's manifest body is BYTE-IDENTICAL — but the
/// rebuilt primary sidecar has a DIFFERENT artifact id, so the required
/// closure differs: the manifest key must fingerprint the ACTUAL closure
/// (a fixed format slot cannot), the new graph publishes its own
/// identity, the old row is never overwritten, and cold self-healing
/// succeeds.
fn rebuilt_closure_check(reposition: bool) {
    use ac_artifact_probe::store::recover;
    use auto_av_probe::cache::unit_span_sidecar_key;
    fn manifest(p: &PathBuf, owner: &str) -> Vec<u8> {
        let st = open_store(p).unwrap();
        let id = st
            .root_artifacts(P, "session-manifest")
            .unwrap()
            .pop()
            .unwrap();
        get_by_artifact_id(&st, &id, &EnvelopeIdentity::manifest(owner, 0)).unwrap()
    }
    let p = dir("owned-store");
    let s = warmup_persisted(SRC, "d_main", &[], &p, P).unwrap();
    let owner = s.current().code()[0].unit().module().owner.module.clone();
    drop(s);
    let before = manifest(&p, &owner);
    let v: serde_json::Value = serde_json::from_slice(&before).unwrap();
    let cv = v["units"]
        .as_array()
        .unwrap()
        .iter()
        .find(|u| u["def_id"] == "d_helper")
        .unwrap()["cv"]
        .as_str()
        .unwrap();
    // Delete exactly one blob ONLY inside this test-created,
    // canonicalized scratch root (never an arbitrary user cache).
    let st = open_store(&p).unwrap();
    let row = st
        .get_artifact("A09", &unit_span_sidecar_key(&owner, "d_helper", cv))
        .unwrap()
        .unwrap();
    let file = p
        .join("blocks")
        .join("blobs")
        .join(&row.chunk_id[..2])
        .join(&row.chunk_id);
    let actual = std::fs::canonicalize(&file).unwrap();
    let own = std::fs::canonicalize(&p).unwrap();
    assert!(actual.starts_with(&own));
    std::fs::remove_file(&actual).unwrap();
    let _ = recover(&st, true).unwrap();
    drop(st);
    let current = if reposition {
        SRC.replacen("return 7", "return      7", 1)
    } else {
        SRC.to_string()
    };
    // Premise: the repositioned source yields a BYTE-IDENTICAL manifest
    // body (same CVs/directory) — only the required sidecar identity
    // changes.
    let reference = dir("fresh-reference");
    drop(warmup_persisted(&current, "d_main", &[], &reference, P).unwrap());
    assert_eq!(
        manifest(&reference, &owner),
        before,
        "premise: same body directory; only required sidecar identity changes"
    );
    let result = restore(&current, "d_main", &[], &p, P);
    if reposition {
        let st = open_store(&p).unwrap();
        let rebuilt = st
            .get_artifact("A09", &unit_span_sidecar_key(&owner, "d_helper", cv))
            .unwrap()
            .unwrap();
        assert_ne!(
            row.artifact_id, rebuilt.artifact_id,
            "premise: the missing primary was genuinely rebuilt before any manifest publish"
        );
    }
    assert!(
        result.is_ok(),
        "a changed required closure must publish a distinct identity instead of conflicting"
    );
    drop(result);
    let again = restore(&current, "d_main", &[], &p, P).unwrap();
    assert_eq!(again.counters().get("restore_units_admitted"), 3);
}

#[test]
fn rebuild_same_source_missing_sidecar_recovers_positive() {
    rebuilt_closure_check(false);
}

#[test]
fn rebuild_repositioned_source_same_body_new_sidecar_does_not_conflict() {
    rebuilt_closure_check(true);
}

/// Distinguishing control through the REAL publisher: the same closure
/// reopens IDEMPOTENTLY (one manifest root, no conflict, persist_error
/// stays 0) — the fingerprint is over the normalized SET, not the
/// enumeration order.
#[test]
fn rebuild_same_closure_reopens_idempotently_through_publisher() {
    let p = dir("idempotent");
    drop(warmup_persisted(SRC, "d_main", &[], &p, P).unwrap());
    let mut s = warmup_persisted(SRC, "d_main", &[], &p, P).unwrap();
    {
        let st = open_store(&p).unwrap();
        assert_eq!(
            st.root_artifacts(P, "session-manifest").unwrap().len(),
            1,
            "the identical closure keeps one manifest root"
        );
    }
    let at = SRC.find("return 7").unwrap() as u64 + 7;
    assert!(s.edit(at, at, b"  ".to_vec()).unwrap().ok);
    assert_eq!(s.counters().get("persist_error"), 0);
}
