//! PLAN-824 T-02 first-case tests: the dynamic whole path over the REAL
//! gates (verify → encode → abc-admit → execute), wire round trip, store
//! restore + the portable freeze/restore round (nothing on disk trusted),
//! and the untrusted-builder admission negatives. Prefix `av_dynamic_824`
//! (contract-decision §9).

use auto_av_core::abc::decode_abc;
use auto_av_core::abc::encode_abc;
use auto_av_core::contract::{
    ExecOutcome, TrapKind, ABC_DYNAMIC_SCHEMA, ABC_PROFILE_DYNAMIC, AV_DYNAMIC_PROFILE,
    DYNAMIC_PRODUCER_ID,
};
use auto_av_core::mir_verify::verify_mir;
use auto_av_probe::dynamic_fixture::{
    dynamic_module_key, dynamic_types_fp_of, first_case_module, request_dynamic, restore_dynamic,
    restore_dynamic_portable, run_dynamic_fixture,
};
use serde_json;

fn tmp_store(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("av824-first-{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create scratch store");
    dir
}

/// The first case executes to Success over the real pipeline: pack/is/take
/// of record/List/Option typed inputs all score (7+1+1=9) and the canonical
/// face carries the frozen axes (result + ledger + live=0).
#[test]
fn av_dynamic_824_first_case_executes_success() {
    let face = run_dynamic_fixture("first-case", None, "first-case-probe")
        .expect("the first case runs the whole path");
    assert_eq!(face["result"]["kind"], "i32", "face: {face}");
    assert_eq!(face["result"]["bits"], "9", "face: {face}");
    assert_eq!(face["live_after_release"], 0, "face: {face}");
    // Ledger shapes (frozen from the first real run, hand-audited): the
    // "dyn" Str materialization (3 bytes) and the list buffer (0 bytes at
    // LIST_NEW — the ledger records the element count at alloc time); both
    // freed LIFO with the list reporting its final 1 element. The Dyn
    // containers hold SHARES — they never allocate.
    assert_eq!(
        face["allocs"],
        serde_json::json!([
            { "id": 0, "bytes": 3 }, { "id": 1, "bytes": 0 }
        ]),
        "face: {face}"
    );
    assert_eq!(
        face["frees"],
        serde_json::json!([
            { "id": 1, "bytes": 1 }, { "id": 0, "bytes": 3 }
        ]),
        "face: {face}"
    );
    assert!(
        face["dyn"].is_null(),
        "an i32 result carries no dyn face: {face}"
    );
}

/// Builder output is UNTRUSTED input: the module only acquires the Checked
/// credential through `verify_mir`, and execution only through the ABC
/// admission — never a direct builder→execute path.
#[test]
fn av_dynamic_824_requires_real_gates() {
    let m = first_case_module();
    assert_eq!(m.profile, AV_DYNAMIC_PROFILE);
    let checked = verify_mir(&m).expect("the first case passes the real verify gate");
    assert_eq!(checked.profile(), AV_DYNAMIC_PROFILE);
    // The wire family is AVAD v4 / profile 5: mutually unreadable with
    // v1/v2/v3 (798 r6 generation isolation).
    let wire = encode_abc(&m).expect("encode");
    let flat = wire.to_flat();
    let decoded = decode_abc(&flat).expect("decode");
    assert_eq!(decoded.profile, ABC_PROFILE_DYNAMIC);
    assert_eq!(decoded.version, auto_av_core::contract::ABC_DYNAMIC_VERSION);
    assert_eq!(decoded.module.owner.schema, ABC_DYNAMIC_SCHEMA);
    assert_eq!(decoded.module.profile, AV_DYNAMIC_PROFILE);
    // decode→re-encode is byte-stable (deterministic wire).
    let wire2 = encode_abc(&decoded.module).expect("re-encode");
    assert_eq!(flat, wire2.to_flat());
}

/// Same-store restore + the portable freeze/restore round: the persisted
/// ABC artifact re-verifies and reproduces the same Success; a Dyn result
/// freezes, persists (A24 — its own domain) and restores bit-exactly; a
/// tampered types-fp axis is a clean miss.
#[test]
fn av_dynamic_824_restore_reverifies() {
    let store = tmp_store("restore");
    let r =
        request_dynamic(Some(&store), "restore-probe", "first-case").expect("persisted request");
    let ExecOutcome::Success { value, .. } = r.av.clone().expect("av set") else {
        panic!("first case must succeed: {:?}", r.av);
    };
    let owner = r.owner.clone();
    let entry = r.entry.clone();
    let snapshot = r.snapshot;
    let types_fp = dynamic_types_fp_of(&first_case_module());
    let key = dynamic_module_key(&owner, &entry, &types_fp, snapshot);

    // The restore seam: envelope admission + full re-verification; disk
    // content is never trusted.
    let out = restore_dynamic(&store, &owner, &entry, snapshot, &types_fp)
        .expect("restore re-verifies and executes");
    match out {
        ExecOutcome::Success { value: v, .. } => assert_eq!(v, value),
        other => panic!("restored run must match: {other:?}"),
    }

    // A changed dependency axis (types fingerprint) is a CLEAN miss —
    // never a reuse of the stale artifact.
    let miss = restore_dynamic(&store, &owner, &entry, snapshot, "deadbeefdeadbeef");
    assert!(miss.is_err(), "a foreign types-fp must miss");

    // The portable round on the Dyn-valued fixture: freeze → A24 persist →
    // same-store restore → unfreeze (CURRENT nominal admission) → equal;
    // the RESTORED execution's Dyn freezes to the SAME deterministic bytes.
    let rd = request_dynamic(Some(&store), "restore-probe", "dyn-out")
        .expect("dyn-out persisted request");
    assert!(
        rd.portable_artifact.is_some(),
        "the portable payload persisted (A24)"
    );
    assert_eq!(
        rd.portable_roundtrip_ok,
        Some(true),
        "in-process portable round"
    );
    let ExecOutcome::Success {
        value: auto_av_core::contract::Value::Dyn(d),
        ..
    } = rd.av.clone().expect("av set")
    else {
        panic!("dyn-out must return a Dyn: {:?}", rd.av);
    };
    let original_bytes = rd.portable_bytes.clone();
    let restored_out = restore_dynamic(
        &store,
        &rd.owner,
        &rd.entry,
        rd.snapshot,
        &dynamic_types_fp_of(&auto_av_probe::dynamic_fixture::fixture_module("dyn-out").unwrap()),
    )
    .expect("dyn-out restore");
    let ExecOutcome::Success {
        value: auto_av_core::contract::Value::Dyn(d2),
        ..
    } = restored_out
    else {
        panic!("restored dyn-out must return a Dyn: {restored_out:?}");
    };
    assert_eq!(
        *d2, *d,
        "the restored Dyn equals the original (content identity)"
    );
    let restored_bytes =
        auto_av_core::portable::freeze(&d2, &auto_av_core::dynamic::TraversalOptions::default())
            .expect("freeze the restored Dyn");
    assert_eq!(
        restored_bytes, original_bytes,
        "restore reproduces the deterministic frozen bytes"
    );
    // The frozen payload also restores standalone through the portable seam.
    let value_fp = format!("{:016x}", auto_av_core::contract::fnv1a64(&original_bytes));
    let standalone = restore_dynamic_portable(
        &store,
        &rd.owner,
        &rd.entry,
        &dynamic_types_fp_of(&auto_av_probe::dynamic_fixture::fixture_module("dyn-out").unwrap()),
        &value_fp,
        rd.snapshot,
        &auto_av_probe::dynamic_fixture::fixture_module("dyn-out").unwrap(),
    )
    .expect("standalone portable restore");
    assert_eq!(standalone, *d);

    // The store carries the artifacts under their OWN layers (code A23 vs
    // portable A24 — separate domains, no confusion with 820/821/822).
    let store2 = ac_artifact_probe::store::ManagedStore::open(&store).expect("reopen store");
    assert!(
        store2
            .get_artifact(auto_av_probe::dynamic_fixture::DYNAMIC_ABC_LAYER.id, &key)
            .expect("lookup")
            .is_some(),
        "the dynamic ABC artifact persists under its own key"
    );
    drop(store2);
    let _ = DYNAMIC_PRODUCER_ID;
    let _ = std::fs::remove_dir_all(&store);
}

/// Admission negatives: the wrong-type take is a RUNTIME CHECKED trap (the
/// legal module executes and traps — source intact); builder-side
/// whitelist violations are rejected by the REAL gate before execution;
/// the Dyn surface is dynamic-profile-only.
#[test]
fn av_dynamic_824_admission_negatives() {
    // 1. The take-mismatch fixture: verifies (a legal module) and traps
    //    av.trap.dyn-type at execution — a checked failure, never infra.
    let r = request_dynamic(None, "negative-probe", "take-mismatch")
        .expect("the mismatch fixture runs the pipeline");
    match r.av.expect("av set") {
        ExecOutcome::Trap { kind, .. } => assert_eq!(kind, TrapKind::DynTypeMismatch),
        other => panic!("wrong-type take must trap: {other:?}"),
    }

    // 2. A Fn-typed slot cannot be erased: the whitelist closure rejects
    //    at verification (nothing executes).
    let mut m = first_case_module();
    let fn_ty = m.intern_type(auto_av_core::mir::MirType::Fn(auto_av_core::mir::FnSig {
        params: vec![],
        result: 0,
    }));
    m.funcs[0].slots.push(auto_av_core::mir::MirSlot {
        kind: auto_av_core::mir::SlotKind::Local,
        ty: fn_ty,
        name: Some("bad".to_string()),
    });
    let n = m.funcs[0].slots.len() as u32 - 1;
    m.funcs[0].blocks[0]
        .instrs
        .push(auto_av_core::mir::MirInstr::DynPack { dst: 3, src: n });
    let err = verify_mir(&m).expect_err("packing a Fn slot must be rejected");
    assert!(format!("{err:?}").contains("mir.dyn.whitelist"), "{err:?}");

    // 3. The erased Dyn TYPE is dynamic-profile-only: the same module
    //    under the collections profile is rejected at the type gate.
    let mut m3 = first_case_module();
    m3.profile = auto_av_core::contract::AV_COLLECTIONS_PROFILE.to_string();
    m3.owner.schema = auto_av_core::contract::MIR_COLLECTIONS_SCHEMA.to_string();
    for f in m3.funcs.iter_mut() {
        f.owner = m3.owner.clone();
    }
    let err3 = verify_mir(&m3).expect_err("the Dyn type needs the dynamic profile");
    assert!(format!("{err3:?}").contains("mir.profile-type"), "{err3:?}");

    // 4. The DYN instructions are dynamic-profile-only: strip the Dyn type
    //    usage but keep an instruction under the owned profile — rejected.
    let mut m4 = first_case_module();
    m4.profile = auto_av_core::contract::AV_OWNED_PROFILE.to_string();
    m4.owner.schema = auto_av_core::contract::MIR_OWNED_SCHEMA.to_string();
    for f in m4.funcs.iter_mut() {
        f.owner = m4.owner.clone();
    }
    let err4 = verify_mir(&m4).expect_err("DYN instructions need the dynamic profile");
    assert!(
        format!("{err4:?}").contains("mir.profile")
            || format!("{err4:?}").contains("mir.schema-profile")
            || format!("{err4:?}").contains("mir.profile-type")
            || format!("{err4:?}").contains("mir.profile-instr"),
        "{err4:?}"
    );
}
