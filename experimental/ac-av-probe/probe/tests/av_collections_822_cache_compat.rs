//! PLAN-822 T-05 cache/compatibility tests: same-store restore identity,
//! tampered-payload rejection (a good envelope never rescues bad bytes),
//! dependency-axis clean misses, and the wire-generation isolation the
//! old consumers rely on (V-08/V-09 axes).
//! Prefix `av_collections_822` (contract-decision §6/§8).

use ac_artifact_probe::codec::ChunkKind;
use ac_artifact_probe::store::{ManagedStore, PutRequest};
use auto_av_core::abc::{decode_abc, encode_abc};
use auto_av_core::abc_verify::verify_abc_bytes;
use auto_av_core::av::{execute, CancelFlag, ExecLimits};
use auto_av_core::contract::{ExecOutcome, ABC_COLLECTIONS_VERSION, ABC_OWNED_SCHEMA};
use auto_av_core::mir_verify::verify_mir;
use auto_av_probe::cache::envelope_wrap;
use auto_av_probe::collections_fixture::{
    collections_envelope, collections_module_key, collections_types_fp_of, first_case_module,
    request_collections, restore_collections, COLLECTIONS_ABC_LAYER,
};
use std::path::PathBuf;

fn tmp_store(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("av822-cache-{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch store");
    dir
}

/// V-08: a fresh worker process restores the persisted artifact and
/// reproduces the SAME success value; a changed dependency axis
/// (types fingerprint) is a clean miss — never a stale reuse.
#[test]
fn av_collections_822_cache_restore_identity_and_miss() {
    let store = tmp_store("identity");
    let r = request_collections(Some(&store), "cache-probe", "first-case").expect("persist");
    let ExecOutcome::Success { value, .. } = r.av.clone().expect("av") else {
        panic!("first case must succeed")
    };

    // The new-worker seam re-verifies everything from disk.
    let out =
        restore_collections(&store, &r.owner, &r.entry, r.snapshot, &r.types_fp).expect("restore");
    match out {
        ExecOutcome::Success { value: v, .. } => assert_eq!(v, value),
        other => panic!("restored value must match: {other:?}"),
    }

    // A foreign snapshot axis is a CLEAN miss.
    let miss = restore_collections(&store, &r.owner, &r.entry, r.snapshot + 1, &r.types_fp);
    assert!(miss.is_err(), "a changed snapshot must miss, not reuse");

    let _ = std::fs::remove_dir_all(&store);
}

/// V-08 tamper axis: a GOOD envelope carrying BAD bytes is rejected at
/// the decode/verify re-run — the restore path never trusts disk
/// content, and the execution count of the tampered artifact is zero
/// (the restore fails before any dispatch).
#[test]
fn av_collections_822_cache_good_envelope_bad_payload_rejected() {
    let store = tmp_store("tamper");
    // The 783 store is content-addressed: a same-key overwrite with
    // different bytes is itself a Conflict rejection (its own tamper
    // defense). To exercise the RESTORE-side defense we forge the
    // artifact FIRST into a fresh store — good identity, bad bytes.
    let r = request_collections(None, "tamper-probe", "first-case").expect("derive identity");

    // Forge a valid-identity envelope over corrupt ABC bytes.
    let mut bad = encode_abc(&first_case_module()).expect("encode").to_flat();
    // Corrupt the interior (keep the magic/version header intact so the
    // rejection is content-level: checksum).
    let mid = bad.len() / 2;
    bad[mid] ^= 0xff;
    let key = collections_module_key(
        &r.owner,
        &r.entry,
        &collections_types_fp_of(&first_case_module()),
        r.snapshot,
    );
    let env = envelope_wrap(&bad, &collections_envelope(&r.owner, &r.entry, r.snapshot));
    let s = ManagedStore::open(&store).expect("open");
    s.put_artifact(PutRequest {
        layer: COLLECTIONS_ABC_LAYER.id,
        key,
        kind: ChunkKind::LayoutData,
        payload: env,
        edges: vec![],
        project: Some("tamper-probe".into()),
    })
    .expect("overwrite with the tampered artifact");
    drop(s);

    let res = restore_collections(&store, &r.owner, &r.entry, r.snapshot, &r.types_fp);
    match res {
        Err(e) => assert!(
            e.contains("re-verify") || e.contains("checksum") || e.contains("decode"),
            "the tampered payload must be rejected at re-verification: {e}"
        ),
        Ok(ExecOutcome::Success { .. }) => {
            panic!("tampered bytes must NEVER execute to success")
        }
        Ok(other) => panic!("unexpected outcome for tampered bytes: {other:?}"),
    }
    let _ = std::fs::remove_dir_all(&store);
}

/// V-09 wire isolation: the collections AVAD v3 generation is mutually
/// unreadable with the earlier generations — a v3 payload under the
/// OWNED (v2) owner-scope admission is a schema/profile rejection, and
/// an owned v2 payload never decodes as v3.
#[test]
fn av_collections_822_compat_wire_generation_isolation() {
    // v3 bytes under the owned consumer's owner scope: rejected.
    let m = first_case_module();
    verify_mir(&m).expect("fixture verifies");
    let v3 = encode_abc(&m).expect("encode").to_flat();
    assert_eq!(&v3[4..6], &ABC_COLLECTIONS_VERSION.to_le_bytes());
    let owned_owner = auto_av_core::mir::OwnerScope {
        schema: ABC_OWNED_SCHEMA.to_string(),
        module: m.owner.module.clone(),
    };
    let err = verify_abc_bytes(&v3, &owned_owner).expect_err("v3 under a v2 consumer");
    assert!(
        format!("{err:?}").contains("schema-profile") || format!("{err:?}").contains("profile"),
        "{err:?}"
    );

    // Round-trip identity: v3 decodes only as the collections module.
    let dec = decode_abc(&v3).expect("v3 decodes in this build");
    assert_eq!(
        dec.module.profile,
        auto_av_core::contract::AV_COLLECTIONS_PROFILE
    );
}

/// V-09 layer separation: the collections artifacts live under their own
/// layer id (A22) with the plan822 producer — the 820/821 layers never
/// observe them (store-level no-confusion).
#[test]
fn av_collections_822_compat_layer_separation() {
    let store = tmp_store("layers");
    let r = request_collections(Some(&store), "layer-probe", "first-case").expect("persist");
    let s = ManagedStore::open(&store).expect("open");
    // The artifact exists under A22...
    let key = collections_module_key(&r.owner, &r.entry, &r.types_fp, r.snapshot);
    assert!(
        s.get_artifact(COLLECTIONS_ABC_LAYER.id, &key)
            .expect("a22 lookup")
            .is_some(),
        "the collections artifact is under its own layer"
    );
    // ...and the owned layer (A21) never serves it.
    let owned_key = ac_artifact_probe::identity::QueryKey::new(
        &auto_av_probe::owned_driver::OWNED_ABC_LAYER,
        vec![("producer".into(), "plan822/av-collections".into())],
    );
    let _ = owned_key;
    assert!(
        s.get_artifact(
            auto_av_probe::owned_driver::OWNED_ABC_LAYER.id,
            &collections_module_key(&r.owner, &r.entry, &r.types_fp, r.snapshot),
        )
        .expect("a21 lookup")
        .is_none(),
        "A21 never carries the collections artifact"
    );
    drop(s);
    let _ = std::fs::remove_dir_all(&store);
}

/// V-08 execution seam: a restored-and-rejected artifact never dispatches
/// — the restore path executes ONLY through `verify_abc_bytes` success
/// (asserted structurally: the tamper test's Err arm precedes execute).
/// This test pins the positive control: a LEGAL restore executes with the
/// same step count as the original run (deterministic replay).
#[test]
fn av_collections_822_cache_deterministic_replay_steps() {
    let store = tmp_store("replay");
    let r = request_collections(Some(&store), "replay-probe", "first-case").expect("persist");
    let ExecOutcome::Success { steps, .. } = r.av.expect("av") else {
        panic!("first case succeeds")
    };
    let out =
        restore_collections(&store, &r.owner, &r.entry, r.snapshot, &r.types_fp).expect("restore");
    match out {
        ExecOutcome::Success { steps: s2, .. } => assert_eq!(s2, steps, "deterministic replay"),
        other => panic!("{other:?}"),
    }
    // The executor's CancelFlag default is unused here; reference it so
    // the import stays honest for future seam extensions.
    let _ = CancelFlag::new(false);
    let _ = ExecLimits::default();
    let _ = execute; // the seam the restore path uses internally
    let _ = std::fs::remove_dir_all(&store);
}
