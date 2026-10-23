//! cte_admit_ — admission/trust family (V-04 core): model results can
//! never mint execution, capability gates reject before any AV call
//! (ct_execute stays 0), binding/owner/type/placeholder faults are
//! controlled language-class diagnostics, and the legal control passes.

use auto_cte_probe::driver::{CteDriver, StepStatus};
use auto_cte_probe::store::{self, CteStore};

fn deck_with(src: &str, tail: &str) -> String {
    format!("deck cte817/1\nmodule main\nsource <<\n{src}\n>>\n{tail}\nentry main\n")
}

#[test]
fn cte_admit_model_results_stay_in_model_domain() {
    // The 796 model seam stays intact: model output is model_compute and
    // a hand-forged vm_execute marking is rejected THERE — and the 817
    // frozen layer never consults 796 results at all (its producer field
    // is bound to real AV success, see the wrong-producer test below).
    use ac_sema_core::reflect::{CtAdmission, CtResult, PortableValue};
    let model = CtResult::model(PortableValue::I32(42), vec![]);
    assert!(model.admit_as_model().is_ok());
    let forged = CtResult {
        schema: model.schema,
        value: PortableValue::I32(42),
        marking: ac_sema_core::reflect::ComputeMarking::VmExecute,
        deps: vec![],
    };
    assert!(matches!(
        forged.admit_as_model(),
        Err(CtAdmission::VmExecuteFromModel)
    ));
}

#[test]
fn cte_admit_unknown_capability_zero_av() {
    let bad = deck_with(
        "fn main() int {\n    return 1\n}\n",
        "caps not.a.cap\nquery q1 = call main()",
    );
    let mut d = CteDriver::ephemeral();
    let out = d.query_step(&bad, None);
    assert_eq!(out.status, StepStatus::Diag);
    assert_eq!(
        out.code.as_deref(),
        Some("cte.request.schema"),
        "{:?}",
        out.message
    );
    assert_eq!(out.ct_execute, 0);
}

#[test]
fn cte_admit_trace_capability_denied_zero_av() {
    // The callee closure requires hir.test.trace.v1 (mark intrinsics) —
    // the CTE profile never grants it: refusal happens BEFORE any AV
    // call (ct_execute=0).
    let deck = deck_with(
        "fn noisy() int {\n    let m = mark_a()\n    return m\n}\n\nfn main() int {\n    let ct = 0\n    return ct\n}\n",
        "query q1 = call noisy()\nbind main.ct = q1 expect i32 placeholder 0",
    );
    let mut d = CteDriver::ephemeral();
    let out = d.query_step(&deck, None);
    assert_eq!(out.status, StepStatus::Diag, "{:?}", out.message);
    assert!(
        out.message
            .as_deref()
            .unwrap_or_default()
            .contains("capability")
            || out.code == Some("cte.capability.denied".to_string()),
        "capability-class refusal, got {:?}",
        out.message
    );
    assert_eq!(out.ct_execute, 0, "denial happens before any AV execution");
}

#[test]
fn cte_admit_wrong_expected_type() {
    // Query yields i32 but the binding expects bool.
    let deck = deck_with(
        "fn cte_sum(a int, b int) int {\n    return a + b\n}\n\nfn main() int {\n    let ct = 0\n    return ct\n}\n",
        "query q1 = call cte_sum(1, 2)\nbind main.ct = q1 expect bool placeholder 0",
    );
    let mut d = CteDriver::ephemeral();
    let out = d.query_step(&deck, None);
    assert_eq!(out.status, StepStatus::Diag);
    assert_eq!(out.code.as_deref(), Some("cte.consumer.type-mismatch"));
    assert_eq!(
        out.ct_execute, 1,
        "the value DID execute; the consumer check failed"
    );
}

#[test]
fn cte_admit_unknown_owner() {
    let deck = deck_with(
        "fn main() int {\n    return 1\n}\n",
        "query q1 = call main()\nbind nope.ct = q1 expect i32 placeholder 0",
    );
    let mut d = CteDriver::ephemeral();
    let out = d.query_step(&deck, None);
    assert_eq!(out.status, StepStatus::Diag);
    assert_eq!(out.code.as_deref(), Some("cte.consumer.binding-owner"));
    assert_eq!(out.ct_execute, 0, "binding faults reject BEFORE execution");
}

#[test]
fn cte_admit_duplicate_binding() {
    let deck = deck_with(
        "fn f() int {\n    return 3\n}\n\nfn main() int {\n    let a = 0\n    let b = 0\n    return a + b\n}\n",
        "query q1 = call f()\nquery q2 = call f()\nbind main.a = q1 expect i32 placeholder 0\nbind main.b = q2 expect i32 placeholder 0",
    );
    // Both bindings use the SAME ct_ref "main.b"? No — ct_ref is
    // owner.binding; duplicates need an explicit same-ref deck, which
    // the deck parser rejects first. Simulate at the binding layer
    // instead via two binds to the same anchor name.
    let mut d = CteDriver::ephemeral();
    let out = d.query_step(&deck, None);
    // Two DIFFERENT ct_refs to two different anchors is legal...
    assert_eq!(out.status, StepStatus::Ok, "{:?}", out.message);
    // ...but the same anchor twice is ambiguous:
    let amb = deck.replace(
        "bind main.b = q2 expect i32 placeholder 0",
        "bind main.a = q2 expect i32 placeholder 0",
    );
    let out2 = d.query_step(&amb, None);
    assert_eq!(out2.status, StepStatus::Diag);
    assert!(
        out2.message
            .as_deref()
            .unwrap_or_default()
            .contains("duplicate"),
        "duplicate/ambiguous anchor rejected: {:?}",
        out2.message
    );
}

#[test]
fn cte_admit_placeholder_drift_rejected() {
    // Source has `let ct = 7` but the binding declares placeholder 0.
    let deck = deck_with(
        "fn f() int {\n    return 3\n}\n\nfn main() int {\n    let ct = 7\n    return ct\n}\n",
        "query q1 = call f()\nbind main.ct = q1 expect i32 placeholder 0",
    );
    let mut d = CteDriver::ephemeral();
    let out = d.query_step(&deck, None);
    assert_eq!(out.status, StepStatus::Diag);
    assert_eq!(
        out.code.as_deref(),
        Some("cte.consumer.binding-placeholder"),
        "{:?}",
        out.code
    );
    assert_eq!(out.ct_execute, 0, "drift rejects before any execution");
}

#[test]
fn cte_admit_wrong_producer_envelope_rejected() {
    // Publish a well-formed-but-wrong-producer envelope under the exact
    // request key: restore must be a CONTROLLED REJECTION, never a miss
    // and never a silent reuse.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("store");
    let st = CteStore::open(&root, "admit").unwrap();
    let identity = fake_identity("plan817/cte-probe");
    let mut env = fake_envelope(&identity, "not-the-real-producer");
    let _ = &mut env;
    store::publish_frozen(&st, &env).unwrap();
    match store::lookup_frozen(&st, &identity) {
        Err(e) => {
            assert!(
                e.message.contains("producer") || e.message.contains("identity"),
                "controlled rejection naming the fault: {e}"
            );
        }
        Ok(Some(other)) => panic!("wrong producer must not restore, got {:?}", other.producer),
        Ok(None) => panic!("wrong producer must REJECT, not miss"),
    }
    // Republication under the SAME key with different content is the
    // store's Conflict — the wrong row can never be overwritten in place.
    let real_env = fake_envelope(&identity, "plan817/cte-probe");
    assert!(
        store::publish_frozen(&st, &real_env).is_err(),
        "same-key different-content must conflict, not overwrite"
    );
    // A different key (different args) publishes and restores fine.
    let mut id2 = fake_identity("plan817/cte-probe");
    id2.args_canonical = "i32:7".into();
    let env2 = fake_envelope(&id2, "plan817/cte-probe");
    store::publish_frozen(&st, &env2).unwrap();
    let got = store::lookup_frozen(&st, &id2)
        .expect("identity-matched restore")
        .expect("present");
    assert_eq!(got.request_identity, id2);
}

#[test]
fn cte_admit_legal_control_passes() {
    let deck = deck_with(
        "fn cte_sum(a int, b int) int {\n    return a + b\n}\n\nfn main() int {\n    let ct = 0\n    return ct\n}\n",
        "query q1 = call cte_sum(20, 22)\nbind main.ct = q1 expect i32 placeholder 0",
    );
    let mut d = CteDriver::ephemeral();
    let out = d.query_step(&deck, None);
    assert_eq!(out.status, StepStatus::Ok, "{:?}", out.message);
    assert_eq!(out.ct_execute, 1);
}

// ---------------------------------------------------------------------------

fn fake_identity(producer: &str) -> auto_cte_contract::CteRequestIdentityV1 {
    use auto_cte_contract::*;
    let _ = producer;
    CteRequestIdentityV1 {
        module: "main".into(),
        package: CTE_SEMA_PACKAGE.into(),
        callee: "f".into(),
        args_canonical: "i32:1".into(),
        query_algo: CTE_QUERY_ALGO_ID.into(),
        caps: vec![],
        engine: CteEngineId {
            av_profile: "av.core-i32/1".into(),
            mir_schema: "auto.mir.av-probe/1".into(),
            abc_schema: "auto.abc.av-probe/2".into(),
            mir_semantics: 1,
            abc_semantics: 2,
            av_semantics: 1,
            exec: "closure".into(),
        },
        manifest_fp: "fp".into(),
    }
}

fn fake_envelope(
    identity: &auto_cte_contract::CteRequestIdentityV1,
    producer: &str,
) -> auto_cte_contract::CteExecutedV1 {
    use auto_cte_contract::*;
    CteExecutedV1 {
        schema: CTE_EXECUTED_SCHEMA.to_string(),
        recipe: CTE_RECIPE.to_string(),
        request_identity: identity.clone(),
        value: CteValue::I32(1),
        declared_type: CteExpectType::I32,
        producer: producer.to_string(),
        engine: identity.engine.clone(),
        exec_steps: 3,
        exec_wall_ms: 0,
        deps: CteDepsManifestV1 {
            granularity: CteDepsGranularity::ClosureBodies,
            callee: "f".into(),
            closure: vec!["f".into()],
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
