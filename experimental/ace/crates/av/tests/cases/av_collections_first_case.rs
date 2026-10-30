//! PLAN-822 T-02 first-case tests: the collections whole path over the
//! REAL gates (verify → encode → abc-admit → execute), wire round trip,
//! store restore (nothing on disk trusted), and the untrusted-builder
//! admission negatives. Prefix `av_collections_822` (contract-decision §6).

use ace_av::collections_fixture::{
    collections_module_key, collections_types_fp_of, first_case_module, request_collections,
    restore_collections, run_collections_fixture,
};
use ace_av_core::abc::decode_abc;
use ace_av_core::abc::encode_abc;
use ace_av_core::contract::{ExecOutcome, ABC_COLLECTIONS_SCHEMA, AV_COLLECTIONS_PROFILE};
use ace_av_core::mir_verify::verify_mir;

fn tmp_store(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("av822-first-{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create scratch store");
    dir
}

/// The first case executes to Success over the real pipeline and the
/// canonical face carries the frozen axes (result + ledger + live=0).
#[test]
fn av_collections_first_case_executes_success() {
    let face = run_collections_fixture("first-case", None, "first-case-probe")
        .expect("the first case runs the whole path");
    assert_eq!(face["result"]["kind"], "i32", "face: {face}");
    assert_eq!(face["result"]["bits"], "0");
    assert_eq!(face["live_after_release"], 0, "face: {face}");
    // Ledger shapes: two Str materializations + the list buffer, all freed.
    assert_eq!(face["allocs"].as_array().unwrap().len(), 3, "face: {face}");
    assert_eq!(face["frees"].as_array().unwrap().len(), 3, "face: {face}");
}

/// Builder output is UNTRUSTED input: the module only acquires the
/// Checked credential through `verify_mir`, and execution only through
/// the ABC admission — never a direct builder→execute path.
#[test]
fn av_collections_first_case_requires_real_gates() {
    let m = first_case_module();
    assert_eq!(m.profile, AV_COLLECTIONS_PROFILE);
    let checked = verify_mir(&m).expect("the first case passes the real verify gate");
    assert_eq!(checked.profile(), AV_COLLECTIONS_PROFILE);
    // The wire family is AVAD v3: mutually unreadable with v1/v2.
    let wire = encode_abc(&m).expect("encode");
    let flat = wire.to_flat();
    let decoded = decode_abc(&flat).expect("decode");
    assert_eq!(
        decoded.profile,
        ace_av_core::contract::ABC_PROFILE_COLLECTIONS
    );
    assert_eq!(
        decoded.version,
        ace_av_core::contract::ABC_COLLECTIONS_VERSION
    );
    assert_eq!(decoded.module.owner.schema, ABC_COLLECTIONS_SCHEMA);
    // decode→re-encode is byte-stable (deterministic wire).
    let wire2 = encode_abc(&decoded.module).expect("re-encode");
    assert_eq!(flat, wire2.to_flat());
}

/// Same-store new-worker restore: the persisted artifact re-verifies and
/// reproduces the same Success; a tampered types-fp axis is a clean miss.
#[test]
fn av_collections_first_case_restore_reverifies() {
    let store = tmp_store("restore");
    let r = request_collections(Some(&store), "restore-probe", "first-case")
        .expect("persisted request");
    let ExecOutcome::Success { value, .. } = r.av.clone().expect("av set") else {
        panic!("first case must succeed: {:?}", r.av);
    };
    let owner = r.owner.clone();
    let entry = r.entry.clone();
    let snapshot = r.snapshot;
    let types_fp = collections_types_fp_of(&first_case_module());
    let key = collections_module_key(&owner, &entry, &types_fp, snapshot);

    // The restore seam: envelope admission + full re-verification; disk
    // content is never trusted.
    let out = restore_collections(&store, &owner, &entry, snapshot, &types_fp)
        .expect("restore re-verifies and executes");
    match out {
        ExecOutcome::Success { value: v, .. } => assert_eq!(v, value),
        other => panic!("restored run must match: {other:?}"),
    }

    // A changed dependency axis (types fingerprint) is a CLEAN miss —
    // never a reuse of the stale artifact.
    let miss = restore_collections(&store, &owner, &entry, snapshot, "deadbeefdeadbeef");
    assert!(miss.is_err(), "a foreign types-fp must miss");

    // The store carries exactly the one collections artifact under the
    // right key (no layer confusion with 820/821).
    let store2 = ace_artifacts::store::ManagedStore::open(&store).expect("reopen store");
    assert!(
        store2
            .get_artifact(ace_av::collections_fixture::COLLECTIONS_ABC_LAYER.id, &key)
            .expect("lookup")
            .is_some(),
        "the collections artifact persists under its own key"
    );
    drop(store2);
    let _ = std::fs::remove_dir_all(&store);
}

/// Admission negatives: builder-side rejections the REAL gate catches
/// before execution (V-01 admission flavor — dispatch=0 equivalent at
/// the probe layer: `verify_mir` fails, nothing executes).
#[test]
fn av_collections_first_case_admission_negatives() {
    // 1. A borrowed-view element type is outside the frozen domain.
    let mut m = first_case_module();
    m.types.push(ace_av_core::mir::MirType::StrView);
    let bad_list_ty = m.types.len() as u32 - 1;
    m.types[4] =
        ace_av_core::mir::MirType::List(ace_av_core::mir::ListTypeRef { elem: bad_list_ty });
    let err = verify_mir(&m).expect_err("StrView elements must be rejected");
    assert!(format!("{err:?}").contains("mir.type.list-elem"), "{err:?}");

    // 2. A record field carrying a container is rejected.
    let mut m2 = first_case_module();
    m2.types[3] = ace_av_core::mir::MirType::Record(ace_av_core::mir::RecordTypeRef {
        owner_module: "collections822".to_string(),
        name: "Note".to_string(),
        fields: vec![ace_av_core::mir::MirFieldDef {
            name: "text".to_string(),
            ty: 4, // List<...> as a field — nested-container rejection
        }],
    });
    let err2 = verify_mir(&m2).expect_err("container record fields must be rejected");
    assert!(
        format!("{err2:?}").contains("mir.type.nested-container"),
        "{err2:?}"
    );

    // 3. A projection without a guard fact never verifies.
    let mut m3 = first_case_module();
    // Drop the guard instruction (index 10 in block 0's flat list).
    m3.funcs[0].blocks[0].instrs.remove(10);
    let err3 = verify_mir(&m3).expect_err("guard-less projection must be rejected");
    assert!(
        format!("{err3:?}").contains("mir.variant.guard-missing"),
        "{err3:?}"
    );

    // 4. The collections instructions are collections-profile-only: the
    //    same module under the owned profile is rejected.
    let mut m4 = first_case_module();
    m4.profile = ace_av_core::contract::AV_OWNED_PROFILE.to_string();
    m4.owner.schema = ace_av_core::contract::MIR_OWNED_SCHEMA.to_string();
    for f in m4.funcs.iter_mut() {
        f.owner = m4.owner.clone();
    }
    let err4 = verify_mir(&m4).expect_err("collections instructions need the collections profile");
    assert!(
        format!("{err4:?}").contains("mir.profile")
            || format!("{err4:?}").contains("mir.schema-profile")
            || format!("{err4:?}").contains("mir.profile-type"),
        "{err4:?}"
    );
}
