//! PLAN-823 T-02 first-case tests: the closures whole path over the REAL
//! gates (verify → encode → abc-admit → execute), wire round trip, store
//! restore (code cache — the env rebuilds during execution), and the
//! untrusted-builder admission negatives. Prefix `av_closures_823`
//! (contract-decision r2 §5).

use auto_av_core::abc::decode_abc;
use auto_av_core::abc::encode_abc;
use auto_av_core::contract::{ExecOutcome, ABC_CLOSURES_SCHEMA, AV_CLOSURES_PROFILE};
use auto_av_core::mir_verify::verify_mir;
use auto_av_probe::closures_fixture::{
    closures_module_key, closures_types_fp_of, first_case_module, request_closures,
    restore_closures, run_closures_fixture,
};

fn tmp_store(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("av823-first-{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create scratch store");
    dir
}

/// The first case executes to Success over the real pipeline and carries
/// the frozen oracle axes (V-01: creator-returned closure stays legal;
/// result matches the hand oracle; live=0 after host release).
/// Hand oracle: env text "auto" (slen 4), env list len 2 after the body's
/// push, bonus 5 → 2*10 + 4 + 5 = **29**.
#[test]
fn av_closures_823_first_case_executes_success() {
    let face = run_closures_fixture("first-case", None, "first-case-probe")
        .expect("the first case runs the whole path");
    assert_eq!(face["result"]["kind"], "i32", "face: {face}");
    assert_eq!(face["result"]["bits"], "1d", "hex(29), face: {face}");
    assert_eq!(face["live_after_release"], 0, "face: {face}");
    // Ledger shapes (frozen): two "auto" materializations (the capture
    // source and the note's own field), the list buffer, the capture
    // clone, the body's pushed note text = 5 allocs, all freed.
    assert_eq!(face["allocs"].as_array().unwrap().len(), 5, "face: {face}");
    assert_eq!(face["frees"].as_array().unwrap().len(), 5, "face: {face}");
}

/// Builder output is UNTRUSTED input: the module only acquires the
/// Checked credential through `verify_mir`, and execution only through
/// the ABC admission — plus the deterministic-wire round trip (V-01
/// admission flavor).
#[test]
fn av_closures_823_first_case_requires_real_gates() {
    let m = first_case_module();
    assert_eq!(m.profile, AV_CLOSURES_PROFILE);
    let checked = verify_mir(&m).expect("the first case passes the real verify gate");
    assert_eq!(checked.profile(), AV_CLOSURES_PROFILE);
    // The wire family is AVAD v5: mutually unreadable with v1/v2/v3/v4.
    let wire = encode_abc(&m).expect("encode");
    let flat = wire.to_flat();
    let decoded = decode_abc(&flat).expect("decode");
    assert_eq!(
        decoded.profile,
        auto_av_core::contract::ABC_PROFILE_CLOSURES
    );
    assert_eq!(
        decoded.version,
        auto_av_core::contract::ABC_CLOSURES_VERSION
    );
    assert_eq!(decoded.module.owner.schema, ABC_CLOSURES_SCHEMA);
    // decode→re-encode is byte-stable (deterministic wire).
    let wire2 = encode_abc(&decoded.module).expect("re-encode");
    assert_eq!(flat, wire2.to_flat());
}

/// Same-store new-worker restore (V-08 first-case axis): the persisted
/// CODE artifact re-verifies and reproduces the same Success (the env
/// NEVER persists — it rebuilds during execution); a tampered types-fp
/// axis is a clean miss.
#[test]
fn av_closures_823_first_case_restore_reverifies() {
    let store = tmp_store("restore");
    let r =
        request_closures(Some(&store), "restore-probe", "first-case").expect("persisted request");
    let ExecOutcome::Success { value, .. } = r.av.clone().expect("av set") else {
        panic!("first case must succeed: {:?}", r.av);
    };
    let owner = r.owner.clone();
    let entry = r.entry.clone();
    let snapshot = r.snapshot;
    let types_fp = closures_types_fp_of(&first_case_module());
    let key = closures_module_key(&owner, &entry, &types_fp, snapshot);

    // The restore seam: envelope admission + full re-verification; disk
    // content is never trusted.
    let out = restore_closures(&store, &owner, &entry, snapshot, &types_fp)
        .expect("restore re-verifies and executes");
    match out {
        ExecOutcome::Success { value: v, .. } => assert_eq!(v, value),
        other => panic!("restored run must match: {other:?}"),
    }

    // A changed dependency axis (types fingerprint) is a CLEAN miss —
    // never a reuse of the stale artifact.
    let miss = restore_closures(&store, &owner, &entry, snapshot, "deadbeefdeadbeef");
    assert!(miss.is_err(), "a foreign types-fp must miss");

    // The store carries exactly the one closures artifact under the right
    // key (no layer confusion with 820/821/822).
    let store2 = ac_artifact_probe::store::ManagedStore::open(&store).expect("reopen store");
    assert!(
        store2
            .get_artifact(auto_av_probe::closures_fixture::CLOSURES_ABC_LAYER.id, &key)
            .expect("lookup")
            .is_some(),
        "the closures artifact persists under its own key"
    );
    drop(store2);
    let _ = std::fs::remove_dir_all(&store);
}

/// Admission negatives: builder-side rejections the REAL gate catches
/// before execution (dispatch=0 equivalent at the probe layer —
/// `verify_mir` fails, nothing executes).
#[test]
fn av_closures_823_first_case_admission_negatives() {
    // 1. ENV_GET outside a closure body never verifies.
    let mut m = first_case_module();
    m.funcs[2].blocks[0]
        .instrs
        .insert(3, auto_av_core::mir::MirInstr::EnvGet { dst: 2, field: 0 });
    let err = verify_mir(&m).expect_err("env-get outside a closure body must be rejected");
    assert!(
        format!("{err:?}").contains("mir.env.not-closure-body"),
        "{err:?}"
    );

    // 2. A borrowed-view capture is outside the escape domain.
    let mut m2 = first_case_module();
    m2.funcs[0].blocks[0].instrs[6] = auto_av_core::mir::MirInstr::CreateClosure {
        dst: 5,
        callee: auto_av_core::mir::CalleeRef::Func(1),
        captures: vec![auto_av_core::mir::CaptureOperand {
            kind: auto_av_core::mir::CaptureKind::Clone,
            src: 6, // out-of-range capture source
        }],
    };
    let err2 = verify_mir(&m2).expect_err("bogus capture source must be rejected");
    assert!(
        format!("{err2:?}").contains("mir.slot-range") || format!("{err2:?}").contains("capture"),
        "{err2:?}"
    );

    // 3. A second CREATE_CLOSURE site for the same callee violates the
    // first-case unique-site rule.
    let mut m3 = first_case_module();
    let site = m3.funcs[0].blocks[0].instrs[6].clone();
    m3.funcs[0].blocks[0].instrs.push(site);
    let err3 = verify_mir(&m3).expect_err("a duplicate closure site must be rejected");
    assert!(
        format!("{err3:?}").contains("mir.closure.site-duplicate"),
        "{err3:?}"
    );

    // 4. The closure instructions are closures-profile-only: the same
    //    module under the collections profile is rejected.
    let mut m4 = first_case_module();
    m4.profile = auto_av_core::contract::AV_COLLECTIONS_PROFILE.to_string();
    m4.owner.schema = auto_av_core::contract::MIR_COLLECTIONS_SCHEMA.to_string();
    for f in m4.funcs.iter_mut() {
        f.owner = m4.owner.clone();
    }
    let err4 = verify_mir(&m4).expect_err("closure instructions need the closures profile");
    assert!(
        format!("{err4:?}").contains("mir.profile-type")
            || format!("{err4:?}").contains("mir.profile-instr"),
        "{err4:?}"
    );
}
