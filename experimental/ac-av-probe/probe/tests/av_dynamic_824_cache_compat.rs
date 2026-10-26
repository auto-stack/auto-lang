//! PLAN-824 T-05 cache/compatibility tests: same-store restore identity
//! (code A23 and portable A24 domains), tampered-payload rejection (a good
//! envelope never rescues bad bytes — and a good CHECKSUM never rescues
//! bad structure), the R14/R15/R16 identity bindings, wire-generation
//! isolation against the old consumers, and layer separation
//! (V-08/V-09 axes).
//! Prefix `av_dynamic_824` (contract-decision §9).

use ac_artifact_probe::codec::ChunkKind;
use ac_artifact_probe::store::{ManagedStore, PutRequest};
use auto_av_core::abc::{decode_abc, encode_abc};
use auto_av_core::abc_verify::verify_abc_bytes;
use auto_av_core::contract::{ExecOutcome, ABC_DYNAMIC_VERSION, ABC_OWNED_SCHEMA};
use auto_av_core::mir_verify::verify_mir;
use auto_av_probe::cache::envelope_wrap;
use auto_av_probe::dynamic_fixture::{
    dynamic_envelope, dynamic_module_key, dynamic_portable_envelope, dynamic_portable_key,
    dynamic_types_fp_of, fixture_module, request_dynamic, restore_dynamic,
    restore_dynamic_portable, DYNAMIC_ABC_LAYER, DYNAMIC_PORTABLE_LAYER,
};
use std::path::PathBuf;

fn tmp_store(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("av824-cache-{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch store");
    dir
}

/// V-08: the persisted code artifact (A23) restores with the SAME success
/// value and the same step count; every changed dependency axis is a
/// CLEAN miss — never a stale reuse.
#[test]
fn av_dynamic_824_cache_restore_identity_and_miss() {
    let store = tmp_store("identity");
    let mut r = request_dynamic(Some(&store), "cache-probe", "first-case").expect("persist");
    let ExecOutcome::Success { value, steps } = r.av.clone().expect("av") else {
        panic!("first case must succeed")
    };
    let owner = r.owner.clone();
    let entry = r.entry.clone();
    let snapshot = r.snapshot;
    let types_fp = dynamic_types_fp_of(&fixture_module("first-case").unwrap());

    // The new-worker seam re-verifies everything from disk.
    let out = restore_dynamic(&store, &owner, &entry, snapshot, &types_fp).expect("restore");
    match out {
        ExecOutcome::Success {
            value: v,
            steps: s2,
        } => {
            assert_eq!(v, value);
            assert_eq!(s2, steps, "deterministic replay");
        }
        other => panic!("restored value must match: {other:?}"),
    }

    // Changed dependency axes are clean misses.
    let miss = restore_dynamic(&store, &owner, &entry, snapshot + 1, &types_fp);
    assert!(miss.is_err(), "a changed snapshot must miss, not reuse");
    let miss = restore_dynamic(&store, &owner, &entry, snapshot, "deadbeefdeadbeef");
    assert!(miss.is_err(), "a foreign types-fp must miss");
    let miss = restore_dynamic(&store, &owner, "foreign_entry", snapshot, &types_fp);
    assert!(miss.is_err(), "a foreign entry must miss");

    let _ = std::fs::remove_dir_all(&store);
}

/// V-08 (the portable domain A24): the persisted frozen payload restores
/// to the EQUAL value; a foreign value-fp or types-fp axis is a clean
/// miss.
#[test]
fn av_dynamic_824_cache_portable_restore_identity_and_miss() {
    let store = tmp_store("portable");
    let mut r = request_dynamic(Some(&store), "portable-probe", "dyn-out").expect("persist");
    assert!(r.portable_artifact.is_some(), "the A24 payload persisted");
    let owner = r.owner.clone();
    let entry = r.entry.clone();
    let snapshot = r.snapshot;
    let ctx = fixture_module("dyn-out").unwrap();
    let types_fp = dynamic_types_fp_of(&ctx);
    let bytes = r.portable_bytes.clone();
    let value_fp = format!("{:016x}", auto_av_core::contract::fnv1a64(&bytes));

    let back = restore_dynamic_portable(&store, &owner, &entry, &types_fp, &value_fp, snapshot, &ctx)
        .expect("portable restore");
    let ExecOutcome::Success {
        value: auto_av_core::contract::Value::Dyn(d),
        ..
    } = r.av.take().expect("av")
    else {
        panic!("dyn-out returns a Dyn")
    };
    assert_eq!(back, *d, "the restored payload equals the original");

    // Foreign axes: clean misses, never a reuse.
    let miss = restore_dynamic_portable(
        &store, &owner, &entry, &types_fp, "ffffffffffffffff", snapshot, &ctx,
    );
    assert!(miss.is_err(), "a foreign value-fp must miss");
    let miss = restore_dynamic_portable(
        &store, &owner, &entry, "deadbeefdeadbeef", &value_fp, snapshot, &ctx,
    );
    assert!(miss.is_err(), "a foreign types-fp must miss");

    let _ = std::fs::remove_dir_all(&store);
}

/// V-08 tamper axis (code domain): a GOOD envelope carrying BAD bytes is
/// rejected at the decode/re-verify re-run — the restore path never
/// trusts disk content, and the tampered artifact never dispatches.
#[test]
fn av_dynamic_824_cache_good_envelope_bad_payload_rejected() {
    let store = tmp_store("tamper");
    // Forge into a FRESH store: good identity, bad bytes (the 783 store's
    // own same-key conflict defense is not the axis under test).
    let r = request_dynamic(None, "tamper-probe", "first-case").expect("derive identity");
    let mut bad = encode_abc(&fixture_module("first-case").unwrap())
        .expect("encode")
        .to_flat();
    // Corrupt the interior (keep the header intact so the rejection is
    // content-level: checksum / re-verify).
    let mid = bad.len() / 2;
    bad[mid] ^= 0xff;
    let types_fp = dynamic_types_fp_of(&fixture_module("first-case").unwrap());
    let key = dynamic_module_key(&r.owner, &r.entry, &types_fp, r.snapshot);
    let env = envelope_wrap(&bad, &dynamic_envelope(&r.owner, r.snapshot));
    let s = ManagedStore::open(&store).expect("open");
    s.put_artifact(PutRequest {
        layer: DYNAMIC_ABC_LAYER.id,
        key,
        kind: ChunkKind::LayoutData,
        payload: env,
        edges: vec![],
        project: Some("tamper-probe".into()),
    })
    .expect("put the tampered artifact");
    drop(s);

    let res = restore_dynamic(&store, &r.owner, &r.entry, r.snapshot, &types_fp);
    match res {
        Err(e) => assert!(
            e.contains("re-verify") || e.contains("checksum") || e.contains("decode"),
            "the tampered payload must be rejected at re-verification: {e}"
        ),
        Ok(ExecOutcome::Success { .. }) => panic!("tampered bytes must NEVER execute to success"),
        Ok(other) => panic!("unexpected outcome for tampered bytes: {other:?}"),
    }
    let _ = std::fs::remove_dir_all(&store);
}

/// V-08 tamper axis (portable domain): a structurally corrupted payload
/// with a RECOMPUTED (good) checksum is still rejected — a good hash never
/// bypasses the structural checks (§5.5 好hash不得绕过).
#[test]
fn av_dynamic_824_cache_portable_good_checksum_bad_structure_rejected() {
    let store = tmp_store("portable-tamper");
    let mut r = request_dynamic(None, "ptamper-probe", "dyn-out").expect("derive identity");
    let mut bytes = r.portable_bytes.clone();
    // Corrupt the FIRST type-descriptor tag (byte 16 = the root tydesc)
    // and RECOMPUTE the FNV trailer so the checksum is honest for the
    // corrupted bytes.
    bytes[16] = 0xff;
    let body_len = bytes.len() - 8;
    let sum = auto_av_core::contract::fnv1a64(&bytes[..body_len]);
    bytes[body_len..].copy_from_slice(&sum.to_le_bytes());

    let ctx = fixture_module("dyn-out").unwrap();
    let types_fp = dynamic_types_fp_of(&ctx);
    let value_fp = format!("{:016x}", auto_av_core::contract::fnv1a64(&bytes));
    let key = dynamic_portable_key(&r.owner, &r.entry, &types_fp, &value_fp, r.snapshot);
    let env = envelope_wrap(&bytes, &dynamic_portable_envelope(&r.owner, r.snapshot));
    let s = ManagedStore::open(&store).expect("open");
    s.put_artifact(PutRequest {
        layer: DYNAMIC_PORTABLE_LAYER.id,
        key,
        kind: ChunkKind::LayoutData,
        payload: env,
        edges: vec![],
        project: Some("ptamper-probe".into()),
    })
    .expect("put the tampered portable payload");
    drop(s);

    let res = restore_dynamic_portable(
        &store,
        &r.owner,
        &r.entry,
        &types_fp,
        &value_fp,
        r.snapshot,
        &ctx,
    );
    match res {
        Err(e) => assert!(
            e.contains("portable.invalid") || e.contains("tydesc") || e.contains("unfreeze"),
            "corrupted structure must be rejected despite the good checksum: {e}"
        ),
        Ok(_) => panic!("corrupted structure must NEVER restore"),
    }
    let _ = std::fs::remove_dir_all(&store);
}

/// V-08/R14: a SELF-CONSISTENT, gate-legal foreign payload (an extra
/// unused type entry — a different types-fp) stored under the ORIGINAL
/// key/envelope is refused BEFORE dispatch with both fingerprints named;
/// the honest module still restores.
#[test]
fn av_dynamic_824_cache_restore_payload_types_fp_mismatch_rejected() {
    let honest_store = tmp_store("r14-honest");
    let forge_store = tmp_store("r14-forge");
    let r = request_dynamic(Some(&honest_store), "r14-probe", "first-case").expect("persist");
    let types_fp = dynamic_types_fp_of(&fixture_module("first-case").unwrap());

    // The foreign module: legal, admitted, but a different type table.
    let mut foreign = fixture_module("first-case").unwrap();
    foreign.intern_type(auto_av_core::mir::MirType::I64);
    verify_mir(&foreign).expect("the foreign module is gate-legal");
    let foreign_fp = dynamic_types_fp_of(&foreign);
    assert_ne!(foreign_fp, types_fp, "the axis actually differs");
    let payload = encode_abc(&foreign).expect("encode").to_flat();

    let key = dynamic_module_key(&r.owner, &r.entry, &types_fp, r.snapshot);
    let env = envelope_wrap(&payload, &dynamic_envelope(&r.owner, r.snapshot));
    let s = ManagedStore::open(&forge_store).expect("open");
    s.put_artifact(PutRequest {
        layer: DYNAMIC_ABC_LAYER.id,
        key,
        kind: ChunkKind::LayoutData,
        payload: env,
        edges: vec![],
        project: Some("r14-probe".into()),
    })
    .expect("put");
    drop(s);

    let err = restore_dynamic(&forge_store, &r.owner, &r.entry, r.snapshot, &types_fp)
        .expect_err("the types-fp mismatch must refuse");
    assert!(
        err.contains("types-fp mismatch") && err.contains(&types_fp) && err.contains(&foreign_fp),
        "both fingerprints named: {err}"
    );
    // Honest control: the ORIGINAL artifact still restores.
    let out =
        restore_dynamic(&honest_store, &r.owner, &r.entry, r.snapshot, &types_fp).expect("honest");
    assert!(matches!(out, ExecOutcome::Success { .. }));
    let _ = std::fs::remove_dir_all(&honest_store);
    let _ = std::fs::remove_dir_all(&forge_store);
}

/// V-08/R15: a payload whose ENTRY identity differs (renamed entry / bumped
/// source-snapshot) under the original key/envelope is refused BEFORE
/// dispatch; the honest module still restores.
#[test]
fn av_dynamic_824_cache_restore_payload_entry_identity_rejected() {
    for variant in ["rename", "snapshot"] {
        let honest_store = tmp_store(&format!("r15-honest-{variant}"));
        let forge_store = tmp_store(&format!("r15-forge-{variant}"));
        let r =
            request_dynamic(Some(&honest_store), "r15-probe", "first-case").expect("persist");
        let types_fp = dynamic_types_fp_of(&fixture_module("first-case").unwrap());

        let mut foreign = fixture_module("first-case").unwrap();
        match variant {
            "rename" => foreign.funcs[foreign.entry as usize].name = "foreign_entry".to_string(),
            _ => foreign.funcs[foreign.entry as usize].source_snapshot = r.snapshot + 1,
        }
        verify_mir(&foreign).expect("the foreign module is gate-legal");
        let payload = encode_abc(&foreign).expect("encode").to_flat();

        let key = dynamic_module_key(&r.owner, &r.entry, &types_fp, r.snapshot);
        let env = envelope_wrap(&payload, &dynamic_envelope(&r.owner, r.snapshot));
        let s = ManagedStore::open(&forge_store).expect("open");
        s.put_artifact(PutRequest {
            layer: DYNAMIC_ABC_LAYER.id,
            key,
            kind: ChunkKind::LayoutData,
            payload: env,
            edges: vec![],
            project: Some("r15-probe".into()),
        })
        .expect("put");
        drop(s);

        let err = restore_dynamic(&forge_store, &r.owner, &r.entry, r.snapshot, &types_fp)
            .expect_err("the entry mismatch must refuse");
        assert!(
            err.contains("entry mismatch") || err.contains("source-snapshot mismatch"),
            "{variant}: {err}"
        );
        let out = restore_dynamic(&honest_store, &r.owner, &r.entry, r.snapshot, &types_fp)
            .expect("honest");
        assert!(matches!(out, ExecOutcome::Success { .. }));
        let _ = std::fs::remove_dir_all(&honest_store);
        let _ = std::fs::remove_dir_all(&forge_store);
    }
}

/// V-08/R16: a legal payload of a DIFFERENT profile generation (the
/// collections AVAD v3 wire) stored under a dynamic key never dispatches —
/// the profile bind fires at decode; the honest dynamic module restores.
#[test]
fn av_dynamic_824_cache_restore_payload_profile_rejected() {
    let honest_store = tmp_store("r16-honest");
    let forge_store = tmp_store("r16-forge");
    let r = request_dynamic(Some(&honest_store), "r16-probe", "first-case").expect("persist");
    let types_fp = dynamic_types_fp_of(&fixture_module("first-case").unwrap());

    // A gate-legal collections-profile module (the previous generation).
    let collections = auto_av_probe::collections_fixture::first_case_module();
    verify_mir(&collections).expect("the collections module is gate-legal");
    let payload = encode_abc(&collections).expect("encode").to_flat();

    let key = dynamic_module_key(&r.owner, &r.entry, &types_fp, r.snapshot);
    let env = envelope_wrap(&payload, &dynamic_envelope(&r.owner, r.snapshot));
    let s = ManagedStore::open(&forge_store).expect("open");
    s.put_artifact(PutRequest {
        layer: DYNAMIC_ABC_LAYER.id,
        key,
        kind: ChunkKind::LayoutData,
        payload: env,
        edges: vec![],
        project: Some("r16-probe".into()),
    })
    .expect("put");
    drop(s);

    let err = restore_dynamic(&forge_store, &r.owner, &r.entry, r.snapshot, &types_fp)
        .expect_err("a foreign-generation payload must refuse");
    assert!(err.contains("profile mismatch"), "{err}");
    let out =
        restore_dynamic(&honest_store, &r.owner, &r.entry, r.snapshot, &types_fp).expect("honest");
    assert!(matches!(out, ExecOutcome::Success { .. }));
    let _ = std::fs::remove_dir_all(&honest_store);
    let _ = std::fs::remove_dir_all(&forge_store);
}

/// V-09 wire isolation: the dynamic AVAD v4 generation is mutually
/// unreadable with the earlier generations — a v4 payload under the OWNED
/// (v2) owner-scope admission is a schema/profile rejection, and v4 decodes
/// only as the dynamic profile.
#[test]
fn av_dynamic_824_compat_wire_generation_isolation() {
    let m = fixture_module("first-case").unwrap();
    verify_mir(&m).expect("fixture verifies");
    let v4 = encode_abc(&m).expect("encode").to_flat();
    assert_eq!(&v4[4..6], &ABC_DYNAMIC_VERSION.to_le_bytes());
    let owned_owner = auto_av_core::mir::OwnerScope {
        schema: ABC_OWNED_SCHEMA.to_string(),
        module: m.owner.module.clone(),
    };
    let err = verify_abc_bytes(&v4, &owned_owner).expect_err("v4 under a v2 consumer");
    assert!(
        format!("{err:?}").contains("schema-profile") || format!("{err:?}").contains("profile"),
        "{err:?}"
    );
    // Round-trip identity: v4 decodes only as the dynamic module.
    let dec = decode_abc(&v4).expect("v4 decodes in this build");
    assert_eq!(
        dec.module.profile,
        auto_av_core::contract::AV_DYNAMIC_PROFILE
    );
}

/// V-09 layer separation: the dynamic artifacts live under their OWN layer
/// ids (A23 code / A24 portable) with the plan824 producer — the owned
/// (A21) and collections (A22) layers never observe them, and the two
/// dynamic domains never serve each other's keys.
#[test]
fn av_dynamic_824_compat_layer_separation() {
    let store = tmp_store("layers");
    let mut r = request_dynamic(Some(&store), "layer-probe", "dyn-out").expect("persist");
    let types_fp = dynamic_types_fp_of(&fixture_module("dyn-out").unwrap());
    let value_fp =
        format!("{:016x}", auto_av_core::contract::fnv1a64(&r.portable_bytes));
    let s = ManagedStore::open(&store).expect("open");

    let code_key = dynamic_module_key(&r.owner, &r.entry, &types_fp, r.snapshot);
    assert!(
        s.get_artifact(DYNAMIC_ABC_LAYER.id, &code_key)
            .expect("a23 lookup")
            .is_some(),
        "the code artifact is under A23"
    );
    let portable_key = dynamic_portable_key(&r.owner, &r.entry, &types_fp, &value_fp, r.snapshot);
    assert!(
        s.get_artifact(DYNAMIC_PORTABLE_LAYER.id, &portable_key)
            .expect("a24 lookup")
            .is_some(),
        "the portable payload is under A24"
    );
    // The two dynamic domains never serve each other's keys.
    assert!(
        s.get_artifact(DYNAMIC_ABC_LAYER.id, &portable_key)
            .expect("a23 portable lookup")
            .is_none(),
        "A23 never carries the portable payload"
    );
    assert!(
        s.get_artifact(DYNAMIC_PORTABLE_LAYER.id, &code_key)
            .expect("a24 code lookup")
            .is_none(),
        "A24 never carries the code artifact"
    );
    // And the prior plans' layers never observe the dynamic artifacts.
    for (layer_id, name) in [
        (auto_av_probe::owned_driver::OWNED_ABC_LAYER.id, "A21"),
        (
            auto_av_probe::collections_fixture::COLLECTIONS_ABC_LAYER.id,
            "A22",
        ),
    ] {
        assert!(
            s.get_artifact(layer_id, &code_key).expect("lookup").is_none(),
            "{name} never carries the dynamic code artifact"
        );
        assert!(
            s.get_artifact(layer_id, &portable_key)
                .expect("lookup")
                .is_none(),
            "{name} never carries the portable payload"
        );
    }
    drop(s);
    let _ = std::fs::remove_dir_all(&store);
}
