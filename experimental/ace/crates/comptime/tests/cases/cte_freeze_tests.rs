//! cte_freeze_ — frozen-layer trust & lifecycle family (PLAN-817 T-04,
//! V-04/V-07 core): bad-block restore admission (wrong schema/producer/
//! engine/identity/carrier are CONTROLLED REJECTIONS — never a miss,
//! never a green reuse), same-key conflict leaves the old commit intact,
//! failed/cancelled requests publish nothing, lease keep-alive across
//! GC, release-then-collect, cross-project root removal keeps the other
//! project's hits, and the user sentinel survives every pass unchanged.

use ace_artifacts::codec::ChunkKind;
use ace_artifacts::identity::QueryKey;
use ace_artifacts::store::{GcConfig, PutRequest};
use ace_comptime::driver::{CteDriver, StepStatus};
use ace_comptime::store::{self, envelope_key, CteStore, CTE_LAYER_SPEC};
use ace_comptime_contract::CTE_FROZEN_LAYER;
use ace_comptime_contract::{
    CteDepsGranularity, CteDepsManifestV1, CteEngineId, CteExecutedV1, CteExpectType,
    CteRequestIdentityV1, CteValue, CTE_EXECUTED_SCHEMA, CTE_PRODUCER, CTE_QUERY_ALGO_ID,
    CTE_SEMA_PACKAGE,
};

fn deck_with(src: &str, tail: &str) -> String {
    format!("deck cte817/1\nmodule main\nsource <<\n{src}\n>>\n{tail}entry main\n")
}

const SUM_DECK: &str = "deck cte817/1\nmodule main\nsource <<\nfn cte_sum(a int, b int) int {\n    return a + b\n}\n\nfn main() int {\n    let ct = 0\n    return ct\n}\n>>\nquery q1 = call cte_sum(20, 22)\nbind main.ct = q1 expect i32 placeholder 0\nentry main\n";

/// An identity the sum-deck's first request would mint (module/callee/
/// args/algo match; the manifest fp is opaque to the attacker model).
fn sum_identity(manifest_fp: &str) -> CteRequestIdentityV1 {
    CteRequestIdentityV1 {
        module: "main".into(),
        package: CTE_SEMA_PACKAGE.into(),
        callee: "cte_sum".into(),
        args_canonical: "i32:20,i32:22".into(),
        query_algo: CTE_QUERY_ALGO_ID.into(),
        caps: vec![],
        engine: current_engine(),
        manifest_fp: manifest_fp.into(),
    }
}

fn current_engine() -> CteEngineId {
    CteEngineId {
        av_profile: "av.core-i32/1".into(),
        mir_schema: "auto.mir.av-probe/1".into(),
        abc_schema: "auto.abc.av-probe/2".into(),
        mir_semantics: 1,
        abc_semantics: 2,
        av_semantics: 1,
        exec: "closure".into(),
    }
}

fn envelope_for(identity: &CteRequestIdentityV1, value: i32) -> CteExecutedV1 {
    CteExecutedV1 {
        schema: CTE_EXECUTED_SCHEMA.to_string(),
        recipe: "cte/scalar-query-v1".into(),
        request_identity: identity.clone(),
        value: CteValue::I32(value),
        declared_type: CteExpectType::I32,
        producer: CTE_PRODUCER.into(),
        engine: identity.engine.clone(),
        exec_steps: 1,
        exec_wall_ms: 0,
        deps: CteDepsManifestV1 {
            granularity: CteDepsGranularity::ClosureBodies,
            callee: identity.callee.clone(),
            closure: vec![identity.callee.clone()],
            fn_fingerprints: vec![],
            meta_records: vec![],
            meta_probes: vec![],
            meta_facts: vec![],
            fn_candidates_fp: "x".into(),
            record_candidates_fp: "y".into(),
            caps: vec![],
        },
        source_fp: "sfp".into(),
    }
}

// ---------------------------------------------------------------------------
// V-04: bad blocks — every fault class is a CONTROLLED REJECTION
// ---------------------------------------------------------------------------

#[test]
fn freeze_bad_blocks_all_controlled_rejections() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("store");
    let st = CteStore::open(&root, "bad-blocks").unwrap();

    let expected = sum_identity("fp-real");
    // (a) wrong schema under the right key — a controlled rejection.
    let mut e = envelope_for(&expected, 42);
    e.schema = "ac817/wrong/v9".into();
    plant_at(&st, &e, &envelope_key(&expected));
    let err = store::lookup_frozen(&st, &expected).unwrap_err();
    assert!(err.message.contains("schema"), "{err}");

    // (b) wrong producer (a model_compute world "minting" execution):
    // re-planting the same key with different content conflicts, so use
    // a fresh store per fault class — the helper below reopens.
    let fresh = |name: &str| CteStore::open(&tmp.path().join(name), name).unwrap();

    let st = fresh("b2");
    let mut e = envelope_for(&expected, 42);
    e.producer = "ac796/model".into();
    plant_at(&st, &e, &envelope_key(&expected));
    let err = store::lookup_frozen(&st, &expected).unwrap_err();
    assert!(err.message.contains("producer"), "{err}");
    assert!(
        err.message
            .contains("model_compute output cannot mint execution"),
        "the isolation rule is named: {err}"
    );

    // (c) wrong engine semantic versions under the same identity fields.
    let st = fresh("b3");
    let mut e = envelope_for(&expected, 42);
    e.engine.abc_semantics = 1;
    plant_at(&st, &e, &envelope_key(&expected));
    let err = store::lookup_frozen(&st, &expected).unwrap_err();
    assert!(err.message.contains("engine"), "{err}");

    // (d) 自洽异址: a fully self-consistent envelope of a DIFFERENT
    // request planted under this request's key — the identity comparison
    // is the only thing that can catch it.
    let st = fresh("b4");
    let mut other = expected.clone();
    other.args_canonical = "i32:7,i32:8".into();
    let e = envelope_for(&other, 99);
    plant_at(&st, &e, &envelope_key(&expected));
    let err = store::lookup_frozen(&st, &expected).unwrap_err();
    assert!(
        err.message.contains("request identity"),
        "good frame, wrong inner identity: {err}"
    );

    // (e) 好frame错载体: the wrong ChunkKind under the right key.
    let st = fresh("b5");
    let e = envelope_for(&expected, 42);
    st.raw()
        .put_artifact(PutRequest {
            layer: CTE_FROZEN_LAYER,
            key: envelope_key(&expected),
            kind: ChunkKind::Hir,
            payload: serde_json::to_vec(&e).unwrap(),
            edges: vec![],
            project: Some("attacker".into()),
        })
        .map(|_| ())
        .unwrap();
    let err = store::lookup_frozen(&st, &expected).unwrap_err();
    assert!(err.message.contains("LayoutData carrier"), "{err}");
}

fn plant_at(st: &CteStore, env: &CteExecutedV1, key: &QueryKey) {
    st.raw()
        .put_artifact(PutRequest {
            layer: CTE_FROZEN_LAYER,
            key: key.clone(),
            kind: ChunkKind::LayoutData,
            payload: serde_json::to_vec(env).unwrap(),
            edges: vec![],
            project: Some("attacker".into()),
        })
        .map(|_| ())
        .unwrap();
}

#[test]
fn freeze_same_key_conflict_keeps_old_commit() {
    let tmp = tempfile::tempdir().unwrap();
    let st = CteStore::open(&tmp.path().join("store"), "conflict").unwrap();
    let id = sum_identity("fp1");
    let a = envelope_for(&id, 42);
    let b = envelope_for(&id, 43);
    store::publish_frozen(&st, &a).unwrap();
    assert!(
        store::publish_frozen(&st, &b).is_err(),
        "same key + different content = Conflict, never overwrite"
    );
    let got = store::lookup_frozen(&st, &id).unwrap().expect("present");
    assert_eq!(got.value, CteValue::I32(42), "the old commit is intact");
}

#[test]
fn freeze_failed_and_cancelled_steps_publish_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("store");
    // A trapping callee publishes NOTHING (the producer path binds to
    // real AV success only).
    let trap_deck = deck_with(
        "fn boom() int {\n    return 2147483647 + 1\n}\n\nfn main() int {\n    let ct = 0\n    return ct\n}\n",
        "query q1 = call boom()\nbind main.ct = q1 expect i32 placeholder 0\n",
    );
    let mut d = CteDriver::with_store(&root, "no-publish").unwrap();
    let out = d.query_step(&trap_deck, None);
    assert_eq!(out.status, StepStatus::Diag);
    assert_eq!(out.ct_execute, 1);
    // A cancelled request publishes nothing either.
    d.cancel();
    let c = d.query_step(SUM_DECK, None);
    assert_eq!(c.code.as_deref(), Some("cte.cancelled"));
    // The very next clean request MISSES — nothing was restored and
    // nothing was planted by the failed/cancelled requests.
    let mut clean = CteDriver::with_store(&root, "no-publish").unwrap();
    let ok = clean.query_step(SUM_DECK, None);
    assert_eq!(ok.status, StepStatus::Ok, "{:?}", ok.message);
    assert_eq!(ok.cache, Some("miss"), "no artifact survived the failures");
}

// ---------------------------------------------------------------------------
// V-07: roots / lease / GC lifecycle
// ---------------------------------------------------------------------------

#[test]
fn freeze_lease_keeps_entry_alive_across_gc() {
    let tmp = tempfile::tempdir().unwrap();
    let st = CteStore::open(&tmp.path().join("store"), "lease").unwrap();
    let id = sum_identity("fp-lease");
    let env = envelope_for(&id, 42);
    let art = store::publish_frozen(&st, &env).unwrap();

    // A live lease pins the artifact: an aggressive GC pass (high=1 →
    // always due; low=0 → delete everything unreachable) must NOT touch
    // it while the lease is held.
    let lease = st
        .raw()
        .acquire_lease("cte817-active-consumer", 300, Some(&art))
        .unwrap();
    let receipt = ace_artifacts::store::run_gc(&st.raw(), GcConfig { high: 1, low: 0 }).unwrap();
    assert!(receipt.ran, "the aggressive pass ran");
    let got = store::lookup_frozen(&st, &id)
        .unwrap()
        .expect("lease keeps it alive");
    assert_eq!(got.value, CteValue::I32(42));

    // Released: the same pass collects it (its project soft root is the
    // only pin — remove it first so the release is observable).
    st.raw().remove_project_roots("lease").unwrap();
    lease.release();
    ace_artifacts::store::run_gc(st.raw(), GcConfig { high: 1, low: 0 }).unwrap();
    assert!(
        store::lookup_frozen(&st, &id).unwrap().is_none(),
        "released + unrooted entries are collected"
    );
}

#[test]
fn freeze_cross_project_roots_independent() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("store");
    // Each project publishes its OWN entry (different helper bodies →
    // different manifests → different keys → distinct soft roots).
    let deck_a = SUM_DECK;
    let deck_b = SUM_DECK.replace("return a + b", "return a + b + 0");
    let mut a = CteDriver::with_store(&root, "proj-a").unwrap();
    let mut b = CteDriver::with_store(&root, "proj-b").unwrap();
    assert_eq!(a.query_step(deck_a, None).cache, Some("miss"));
    assert_eq!(b.query_step(&deck_b, None).cache, Some("miss"));

    // proj-a drops its roots; a GC pass keeps proj-b's entry alive
    // through proj-b's own root.
    let st = CteStore::open(&root, "gc-runner").unwrap();
    st.raw().remove_project_roots("proj-a").unwrap();
    ace_artifacts::store::run_gc(st.raw(), GcConfig { high: 1, low: 0 }).unwrap();
    let mut b2 = CteDriver::with_store(&root, "proj-b").unwrap();
    let hit = b2.query_step(&deck_b, None);
    assert_eq!(hit.status, StepStatus::Ok, "{:?}", hit.message);
    assert_eq!(hit.cache, Some("hit"), "proj-b's root keeps its entry");

    // When proj-b drops its root too, its entry is collectable.
    st.raw().remove_project_roots("proj-b").unwrap();
    ace_artifacts::store::run_gc(st.raw(), GcConfig { high: 1, low: 0 }).unwrap();
    let mut b3 = CteDriver::with_store(&root, "proj-b").unwrap();
    let miss = b3.query_step(&deck_b, None);
    assert_eq!(miss.cache, Some("miss"), "unrooted entries are gone");
}

#[test]
fn freeze_user_sentinel_survives_every_pass() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("store");
    // A foreign "user sentinel" artifact lives in its own layer under
    // its own project root.
    let sentinel = CteStore::open(&root, "user-sentinel").unwrap();
    let key = QueryKey::new(
        &CTE_LAYER_SPEC,
        vec![
            ("producer".into(), "user".into()),
            ("owner".into(), "sentinel".into()),
            ("callee".into(), "s".into()),
            ("args".into(), "i32:0".into()),
            ("algo".into(), "user".into()),
            ("caps".into(), "".into()),
            ("engine".into(), "user".into()),
            ("manifest-fp".into(), "user".into()),
        ],
    );
    let sent_id = sentinel
        .raw()
        .put_artifact(PutRequest {
            layer: CTE_FROZEN_LAYER,
            key: key.clone(),
            kind: ChunkKind::LayoutData,
            payload: b"sentinel-bytes".to_vec(),
            edges: vec![],
            project: Some("user-sentinel".into()),
        })
        .unwrap();

    // CTE traffic + aggressive GC passes come and go.
    let mut d = CteDriver::with_store(&root, "traffic").unwrap();
    assert_eq!(d.query_step(SUM_DECK, None).cache, Some("miss"));
    d.store()
        .unwrap()
        .raw()
        .remove_project_roots("traffic")
        .unwrap();
    ace_artifacts::store::run_gc(d.store().unwrap().raw(), GcConfig { high: 1, low: 0 }).unwrap();

    // The sentinel is untouched — same id, same bytes.
    let got = sentinel
        .raw()
        .get_artifact_by_id(&sent_id)
        .unwrap()
        .expect("sentinel alive");
    assert_eq!(got.payload, b"sentinel-bytes");
    assert!(
        sentinel
            .raw()
            .get_artifact(CTE_FROZEN_LAYER, &key)
            .unwrap()
            .is_some(),
        "still addressable under its own key"
    );
}

// ---------------------------------------------------------------------------
// V-04 completion: unknown callee / no-import profile
// ---------------------------------------------------------------------------

#[test]
fn freeze_unknown_callee_language_diag_zero_av() {
    // The profile has no imports; an undeclared name is a callee-side
    // source failure — a LANGUAGE diagnostic before any AV call.
    let d = deck_with(
        "fn main() int {\n    let ct = 0\n    return ct\n}\n",
        "query q1 = call not_declared()\nbind main.ct = q1 expect i32 placeholder 0\n",
    );
    let mut drv = CteDriver::ephemeral();
    let out = drv.query_step(&d, None);
    assert_eq!(out.status, StepStatus::Diag);
    // An undeclared callee rejects at the manifest admission — still a
    // language-class refusal before any AV call.
    assert_eq!(
        out.code.as_deref(),
        Some("cte.query.unknown-ref"),
        "{:?}",
        out.code
    );
    assert!(out.language_class);
    assert_eq!(out.ct_execute, 0);

    // A DECLARED callee whose body fails type checking surfaces the
    // callee-side frontend diagnostic (language class, pre-AV).
    let bad = deck_with(
        "fn bad() int {
    return true
}

fn main() int {
    let ct = 0
    return ct
}
",
        "query q1 = call bad()
bind main.ct = q1 expect i32 placeholder 0
",
    );
    let out2 = drv.query_step(&bad, None);
    assert_eq!(out2.status, StepStatus::Diag);
    assert_eq!(
        out2.code.as_deref(),
        Some("cte.callee.frontend"),
        "{:?}",
        out2.code
    );
    assert!(out2.language_class);
    assert_eq!(out2.ct_execute, 0);
}
