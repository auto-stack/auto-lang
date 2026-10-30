//! PLAN-822 T-05 cache/compatibility tests: same-store restore identity,
//! tampered-payload rejection (a good envelope never rescues bad bytes),
//! dependency-axis clean misses, and the wire-generation isolation the
//! old consumers rely on (V-08/V-09 axes).
//! Prefix `av_collections_822` (contract-decision §6/§8).

use ace_artifacts::codec::ChunkKind;
use ace_artifacts::store::{ManagedStore, PutRequest};
use ace_av::cache::envelope_wrap;
use ace_av::collections_fixture::{
    collections_envelope, collections_module_key, collections_types_fp_of, first_case_module,
    request_collections, restore_collections, COLLECTIONS_ABC_LAYER,
};
use ace_av_core::abc::{decode_abc, encode_abc};
use ace_av_core::abc_verify::verify_abc_bytes;
use ace_av_core::av::{execute, CancelFlag, ExecLimits};
use ace_av_core::contract::{ExecOutcome, ABC_COLLECTIONS_VERSION, ABC_OWNED_SCHEMA};
use ace_av_core::mir_verify::verify_mir;
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
fn av_collections_cache_restore_identity_and_miss() {
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
fn av_collections_cache_good_envelope_bad_payload_rejected() {
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
fn av_collections_compat_wire_generation_isolation() {
    // v3 bytes under the owned consumer's owner scope: rejected.
    let m = first_case_module();
    verify_mir(&m).expect("fixture verifies");
    let v3 = encode_abc(&m).expect("encode").to_flat();
    assert_eq!(&v3[4..6], &ABC_COLLECTIONS_VERSION.to_le_bytes());
    let owned_owner = ace_av_core::mir::OwnerScope {
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
        ace_av_core::contract::AV_COLLECTIONS_PROFILE
    );
}

/// V-09 layer separation: the collections artifacts live under their own
/// layer id (A22) with the plan822 producer — the 820/821 layers never
/// observe them (store-level no-confusion).
#[test]
fn av_collections_compat_layer_separation() {
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
    let owned_key = ace_artifacts::identity::QueryKey::new(
        &ace_av::owned_driver::OWNED_ABC_LAYER,
        vec![("producer".into(), "plan822/av-collections".into())],
    );
    let _ = owned_key;
    assert!(
        s.get_artifact(
            ace_av::owned_driver::OWNED_ABC_LAYER.id,
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
fn av_collections_cache_deterministic_replay_steps() {
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

/// F822-R14-01: restore binds the PAYLOAD's actual type-table fingerprint
/// to the requested identity. A structurally legal, self-consistent ABC
/// whose type table differs (one unused I64 appended — MIR/ABC both
/// admit it) is stored in a REAL ManagedStore under the ORIGINAL
/// types-fp key with a fully matching envelope (owner/entry/snapshot all
/// consistent): the re-verification gates alone would admit it; the
/// identity check must refuse to dispatch BEFORE execution (V-08:
/// dispatch=0 — the restore returns Err and no guest instruction runs).
#[test]
fn av_collections_cache_restore_payload_types_fp_mismatch_rejected() {
    let r = request_collections(None, "cache822", "first-case").unwrap();
    // The foreign module: same body, same snapshot, one MORE type — legal
    // everywhere, but a different type-table fingerprint.
    let mut m = first_case_module();
    m.funcs[0].source_snapshot = r.snapshot;
    m.types.push(ace_av_core::mir::MirType::I64);
    verify_mir(&m).expect("foreign type table remains structurally valid");
    let actual_fp = collections_types_fp_of(&m);
    assert_ne!(actual_fp, r.types_fp, "the fixture needs a real fp delta");
    let wire = encode_abc(&m).unwrap();
    verify_abc_bytes(&wire.to_flat(), &m.owner).expect("self-consistent valid ABC");
    // Store under the ORIGINAL identity (the attacker-controlled axis):
    // original key + envelope carrying the ORIGINAL owner/entry/snapshot.
    let store = std::env::temp_dir().join(format!(
        "cache822-r14-fp-binding-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let s = ManagedStore::open(&store).unwrap();
    s.put_artifact(PutRequest {
        layer: COLLECTIONS_ABC_LAYER.id,
        key: collections_module_key(&r.owner, &r.entry, &r.types_fp, r.snapshot),
        kind: ChunkKind::LayoutData,
        payload: envelope_wrap(
            &wire.to_flat(),
            &collections_envelope(&r.owner, &r.entry, r.snapshot),
        ),
        edges: vec![],
        project: Some("cache822".into()),
    })
    .unwrap();
    drop(s);
    // The mismatch is refused BEFORE dispatch — the error names both
    // fingerprints, and the outcome is an Err (never a Success).
    let out = restore_collections(&store, &r.owner, &r.entry, r.snapshot, &r.types_fp);
    let msg = out.expect_err("a foreign type table under an expected key must not dispatch");
    assert!(
        msg.contains("types-fp mismatch"),
        "the refusal names the identity gap: {msg}"
    );
    assert!(
        msg.contains(&r.types_fp) && msg.contains(&actual_fp),
        "{msg}"
    );
    // The positive control: an HONEST store (its own directory — the
    // poisoned key above is content-addressed and refuses overwrites)
    // re-serving the ORIGINAL payload still restores and executes.
    let honest_store = std::env::temp_dir().join(format!(
        "cache822-r14-fp-honest-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let r2 = request_collections(Some(&honest_store), "cache822", "first-case").unwrap();
    assert_eq!(r2.types_fp, r.types_fp);
    let ok = restore_collections(
        &honest_store,
        &r2.owner,
        &r2.entry,
        r2.snapshot,
        &r2.types_fp,
    );
    assert!(
        matches!(&ok, Ok(ExecOutcome::Success { .. })),
        "an honest restore still executes: {ok:?}"
    );
    let _ = std::fs::remove_dir_all(&store);
    let _ = std::fs::remove_dir_all(&honest_store);
}

/// F822-R15-01: restore binds the payload's ACTUAL entry identity — the
/// entry function NAME and its `source_snapshot` (the envelope never
/// carried the entry; its snapshot is outer metadata, not decoded
/// content). Two same-fingerprint payloads (the ORIGINAL type table, so
/// the R14 types-fp check passes), each legally admitted by MIR/ABC, are
/// stored under the ORIGINAL key with a fully matching envelope:
///  (a) the entry function renamed to `foreign_entry`;
///  (b) the entry function's source_snapshot bumped +1.
/// Both must be refused BEFORE dispatch; the honest store still restores.
#[test]
fn av_collections_cache_restore_payload_entry_identity_rejected() {
    let r = request_collections(None, "cache822", "first-case").unwrap();
    for mode in ["entry", "snapshot"] {
        let mut m = first_case_module();
        m.funcs[0].source_snapshot = r.snapshot;
        match mode {
            "entry" => m.funcs[0].name = "foreign_entry".into(),
            "snapshot" => m.funcs[0].source_snapshot = r.snapshot.wrapping_add(1),
            _ => unreachable!(),
        }
        verify_mir(&m).expect("foreign payload remains structurally valid");
        assert_eq!(collections_types_fp_of(&m), r.types_fp, "same type table");
        let wire = encode_abc(&m).unwrap();
        verify_abc_bytes(&wire.to_flat(), &m.owner).expect("self-consistent valid ABC");
        let store = std::env::temp_dir().join(format!(
            "cache822-r15-entry-binding-{}-{}-{mode}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let s = ManagedStore::open(&store).unwrap();
        s.put_artifact(PutRequest {
            layer: COLLECTIONS_ABC_LAYER.id,
            key: collections_module_key(&r.owner, &r.entry, &r.types_fp, r.snapshot),
            kind: ChunkKind::LayoutData,
            payload: envelope_wrap(
                &wire.to_flat(),
                &collections_envelope(&r.owner, &r.entry, r.snapshot),
            ),
            edges: vec![],
            project: Some("cache822".into()),
        })
        .unwrap();
        drop(s);
        let out = restore_collections(&store, &r.owner, &r.entry, r.snapshot, &r.types_fp);
        let msg =
            out.expect_err("a foreign entry identity under an expected key must not dispatch");
        assert!(
            msg.contains(mode) || msg.contains("mismatch"),
            "the refusal names the identity gap ({mode}): {msg}"
        );
        let _ = std::fs::remove_dir_all(&store);
    }
    // The positive control: an honest store's ORIGINAL payload (same
    // entry name, same snapshot) still restores and executes.
    let honest_store = std::env::temp_dir().join(format!(
        "cache822-r15-entry-honest-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let r2 = request_collections(Some(&honest_store), "cache822", "first-case").unwrap();
    let ok = restore_collections(
        &honest_store,
        &r2.owner,
        &r2.entry,
        r2.snapshot,
        &r2.types_fp,
    );
    assert!(
        matches!(&ok, Ok(ExecOutcome::Success { .. })),
        "an honest restore still executes: {ok:?}"
    );
    let _ = std::fs::remove_dir_all(&honest_store);
}

/// F822-R16-01: restore binds the payload's ACTUAL wire profile. The
/// generic admission validates every payload against the version tuple
/// of its OWN decoded profile (core-i32 included) — never the
/// requester's — so a legal core-i32 AVAB whose types/entry/snapshot
/// all match (passing the R14/R15 bindings) executed under the
/// collections identity. A scalar collections module (types=[I32, Fn];
/// main returns 7) is re-stamped as a legal core-i32 profile/schema
/// module — SAME types_fp — encoded as AVAB and stored under the
/// collections key + collections envelope: it must be refused BEFORE
/// dispatch. The honest control: the SAME module kept on the
/// collections profile restores and executes (the fix rejects foreign
/// WIRE, not legal scalar-only collections programs).
#[test]
fn av_collections_cache_restore_payload_profile_rejected() {
    let build = |as_core: bool| -> (String, Vec<u8>) {
        let owner = ace_av::collections_fixture::collections_owner_scope();
        let mut m = ace_av_core::mir::MirModule::new_collections(owner.clone());
        let ti = m.intern_type(ace_av_core::mir::MirType::I32);
        let sig = m.intern_type(ace_av_core::mir::MirType::Fn(ace_av_core::mir::FnSig {
            params: vec![],
            result: ti,
        }));
        let c = m.intern_const(ace_av_core::mir::MirConst::I32(7));
        m.funcs.push(ace_av_core::mir::MirFunction {
            owner: owner.clone(),
            name: "main".into(),
            sig,
            slots: vec![ace_av_core::mir::MirSlot {
                kind: ace_av_core::mir::SlotKind::Local,
                ty: ti,
                name: None,
            }],
            blocks: vec![ace_av_core::mir::MirBlock {
                instrs: vec![ace_av_core::mir::MirInstr::Const { dst: 0, value: c }],
                term: ace_av_core::mir::Terminator::Return { value: 0 },
            }],
            entry: 0,
            source_snapshot: 123,
            spans: vec![],
            env: None,
        });
        verify_mir(&m).unwrap();
        let fp = collections_types_fp_of(&m);
        if as_core {
            m.profile = ace_av_core::contract::AV_PROFILE.into();
            m.owner.schema = ace_av_core::contract::MIR_SCHEMA.into();
            m.funcs[0].owner = m.owner.clone();
            verify_mir(&m).unwrap();
            assert_eq!(collections_types_fp_of(&m), fp, "same types table");
        }
        let wire = encode_abc(&m).unwrap().to_flat();
        (fp, wire)
    };
    for (as_core, must_err) in [(true, true), (false, false)] {
        let (fp, wire) = build(as_core);
        let root = std::env::temp_dir().join(format!(
            "cache822-r16-profile-{}-{}-{as_core}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let store = ManagedStore::open(&root).unwrap();
        store
            .put_artifact(PutRequest {
                layer: COLLECTIONS_ABC_LAYER.id,
                key: collections_module_key("collections822", "main", &fp, 123),
                kind: ChunkKind::LayoutData,
                payload: envelope_wrap(&wire, &collections_envelope("collections822", "main", 123)),
                edges: vec![],
                project: Some("cache822".into()),
            })
            .unwrap();
        drop(store);
        let out = restore_collections(&root, "collections822", "main", 123, &fp);
        if must_err {
            let msg = out
                .expect_err("a foreign core wire under the collections identity must not dispatch");
            assert!(
                msg.contains("profile mismatch"),
                "the refusal names the identity gap: {msg}"
            );
        } else {
            assert!(
                matches!(&out, Ok(ExecOutcome::Success { .. })),
                "an honest collections scalar module still restores: {out:?}"
            );
        }
        let _ = std::fs::remove_dir_all(&root);
    }
}
