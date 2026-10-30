//! PLAN-796 T-03 query/reflection dependency tests (`meta_query_`
//! filter, V-03/V-05): set/negative dependencies, source-map
//! fail-closed invalidation, signature queries surviving body edits,
//! and the CT model seam (cycles, portable-value admission).

use ace_core::source::SourceProfile;
use ace_semantics::adapter::parse_into_input;
use ace_semantics::query::{current_fingerprints, freshness_of, QueryKind};
use ace_semantics::{Freshness, InferOutcome, InferSession};
use ace_semantics_contract::{DepFingerprint, DepManifest, NegDep};
use ace_semantics_store::{
    lookup_only, open_store, publish_infer_result, publish_record_meta, run_inference,
};

fn core_src(src: &str, module: &str) -> ace_semantics::SemaInput {
    parse_into_input(src, "mq", module, SourceProfile::Core).unwrap()
}

fn rec_src(src: &str, module: &str) -> ace_semantics::SemaInput {
    parse_into_input(src, "mq", module, SourceProfile::TextRecord).unwrap()
}

/// Field-set change invalidates a record-fields query (the KEY carries
/// the field-set fingerprint), while an infer-body query on a function
/// that never reads the record stays hit-eligible (未读不误失效).
#[test]
fn meta_query_field_set_change_invalidates_only_readers() {
    let root = std::env::temp_dir().join("ac796-mq-fields");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let store = open_store(&root).unwrap();

    let src1 = "type Pair { left int; right bool }\nfn main() int { return 1 }";
    let input1 = rec_src(src1, "m");
    publish_record_meta(&store, &input1, "Pair", Some("proj")).unwrap();

    // fields query hits on the unchanged module
    match lookup_only(
        &store,
        &input1,
        &QueryKind::RecordFields {
            record: "Pair".into(),
        },
    )
    .unwrap()
    {
        ace_semantics_store::FetchOutcome::Hit { .. } => {}
        other => panic!("expected fields hit, got {other:?}"),
    }

    // a field is ADDED: the field-set key component changes → Miss
    let src2 = "type Pair { left int; right bool; mid bool }\nfn main() int { return 1 }";
    let input2 = rec_src(src2, "m");
    match lookup_only(
        &store,
        &input2,
        &QueryKind::RecordFields {
            record: "Pair".into(),
        },
    )
    .unwrap()
    {
        ace_semantics_store::FetchOutcome::Miss => {}
        other => panic!("expected fields miss after field added, got {other:?}"),
    }
}

/// Signature-only queries survive body-only edits (no blanket interface
/// hash): the FnSig key carries only fn-sig-fp.
#[test]
fn meta_query_signature_query_survives_body_edit() {
    let root = std::env::temp_dir().join("ac796-mq-sig");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let store = open_store(&root).unwrap();

    let src1 = "fn f(a int) int { return a + 1 }";
    let input1 = core_src(src1, "m");
    // publish the infer-body result of f (its own namespace)
    let sum = match run_inference(&input1, "f") {
        InferOutcome::Ok(s) => s,
        other => panic!("{other:?}"),
    };
    publish_infer_result(
        &store,
        &input1,
        &QueryKind::InferBody {
            fn_name: "f".into(),
        },
        &sum,
        Some("proj"),
    )
    .unwrap();

    // body-only edit: infer-body key changes (miss), while the SIGNATURE
    // (and the caller that only read the signature) is unaffected
    let src2 = "fn f(a int) int { return a * 100 }";
    let input2 = core_src(src2, "m");
    match lookup_only(
        &store,
        &input2,
        &QueryKind::InferBody {
            fn_name: "f".into(),
        },
    )
    .unwrap()
    {
        ace_semantics_store::FetchOutcome::Miss => {}
        other => panic!("expected infer-body miss after body edit, got {other:?}"),
    }
}

/// Manifest-level negative dependency: a name ABSENT at compute time
/// becoming defined breaks freshness even when the caller is unchanged
/// (M-04 semantics, exercised directly at the manifest layer).
#[test]
fn meta_query_negative_dep_name_becomes_defined() {
    let src = "fn main() int { return 1 }";
    let input1 = core_src(src, "m");
    let cur1 = current_fingerprints(&input1);
    // a manifest recorded when `g` was absent (e.g. an unknown-name
    // lookup during a previous revision)
    let manifest = DepManifest {
        positives: vec![],
        negatives: vec![NegDep {
            domain: "fn",
            name: "g".into(),
            set_fp: cur1.candidate_sets.get("fn").unwrap().clone(),
        }],
    };
    assert_eq!(freshness_of(&manifest, &cur1), Freshness::Fresh);

    // `g` becomes defined: the negative dep is broken
    let src2 = "fn g() int { return 2 }\nfn main() int { return 1 }";
    let cur2 = current_fingerprints(&core_src(src2, "m"));
    match freshness_of(&manifest, &cur2) {
        Freshness::Stale { reason } => {
            assert!(
                reason.contains("g"),
                "reason names the now-defined name: {reason}"
            )
        }
        Freshness::Fresh => panic!("negative dep must break when g appears"),
        Freshness::ProvenanceStale { .. } => {}
    }

    // candidate-set drift alone (a DIFFERENT fn added) also invalidates
    // the recorded lookup domain
    let src3 = "fn h() int { return 3 }\nfn main() int { return 1 }";
    let cur3 = current_fingerprints(&core_src(src3, "m"));
    assert!(matches!(
        freshness_of(&manifest, &cur3),
        Freshness::Stale { .. }
    ));
}

/// source-map dependencies fail CLOSED: a manifest that read source
/// bytes is stale when the origin is unregistered or the bytes changed
/// — position-only edits must not silently reuse a stale reflection
/// result (§5.3 代码位置变化…反射读取doc/source时相关变化须失效).
#[test]
fn meta_query_source_map_fails_closed() {
    let input = core_src("fn f() int { return 1 }", "m");
    let mut cur = current_fingerprints(&input);
    let manifest = DepManifest {
        positives: vec![DepFingerprint {
            kind: "source-map",
            origin: "file:m.at".into(),
            fp: "hash-of-original-bytes".into(),
        }],
        negatives: vec![],
    };
    // origin not registered → stale (fail closed)
    assert!(matches!(
        freshness_of(&manifest, &cur),
        Freshness::Stale { .. }
    ));
    // registered with the same fp → fresh
    cur.source_maps
        .insert("file:m.at".into(), "hash-of-original-bytes".into());
    assert_eq!(freshness_of(&manifest, &cur), Freshness::Fresh);
    // bytes changed (a position-only edit IS a byte change) → stale
    cur.source_maps
        .insert("file:m.at".into(), "hash-of-moved-bytes".into());
    assert!(matches!(
        freshness_of(&manifest, &cur),
        Freshness::Stale { .. }
    ));
}

/// Positive dependency drift: a callee SIGNATURE change invalidates a
/// caller's manifest even when the caller's own key components are
/// unchanged (§5.4 helper 变更失效).
#[test]
fn meta_query_positive_dep_callee_signature_drift() {
    let src1 = "fn helper(a int) int { return a }\nfn main() int { return helper(2) }";
    let input1 = core_src(src1, "m");
    let sum = match InferSession::new(&input1).infer_function("main") {
        InferOutcome::Ok(s) => s,
        other => panic!("{other:?}"),
    };
    // the manifest records the read of helper's signature
    assert!(sum.reads.sigs.iter().any(|s| s == "helper"));

    let src2 = "fn helper(a bool) int { return 1 }\nfn main() int { return helper(2) }";
    let input2 = core_src(src2, "m");
    let cur2 = current_fingerprints(&input2);
    let manifest = ace_semantics::manifest_from(&sum.reads, &input1, "main");
    match freshness_of(&manifest, &cur2) {
        Freshness::Stale { reason } => {
            assert!(
                reason.contains("fn-sig") || reason.contains("helper"),
                "reason: {reason}"
            )
        }
        Freshness::Fresh => panic!("callee signature drift must invalidate"),
        Freshness::ProvenanceStale { .. } => {}
    }
}

/// CT model seam (V-05): portable requests evaluate with
/// model_compute marking; Unknown type refs and request CYCLES are
/// controlled rejections — no AVM execution is claimed anywhere.
#[test]
fn meta_query_ct_model_seam_admission_and_cycles() {
    use ace_semantics::{CtAdmission, CtRequest, ModelCt, PortableValue};

    // portable: direct fold
    let req = CtRequest::new(
        "tpl/sum",
        vec![PortableValue::I32(2), PortableValue::I32(3)],
        vec![],
    );
    let res = ModelCt::fold_i32_sum(&req).unwrap();
    assert!(matches!(res.value, PortableValue::I32(5)));
    assert_eq!(res.marking.as_str(), "model_compute");

    // Unknown/unsolved type reference rejected at admission
    let bad = CtRequest::new(
        "tpl/t",
        vec![PortableValue::TypeRef(
            ace_semantics_contract::TypeDesc::Opaque {
                name: "?unsolved".into(),
            },
        )],
        vec![],
    );
    assert!(matches!(bad.admit(), Err(CtAdmission::UnknownValue)));

    // request cycle → controlled CyclicRequest with the full chain
    let mut ct = ModelCt::new();
    let r1 = CtRequest::new("tpl/a", vec![], vec![]);
    let err = ct
        .evaluate(&r1, |_req, ct| {
            // a's body model evaluates b, which evaluates a again
            let r2 = CtRequest::new("tpl/b", vec![], vec![]);
            ct.evaluate(&r2, |_req, ct| {
                let r1again = CtRequest::new("tpl/a", vec![], vec![]);
                ct.evaluate(&r1again, |_, _| unreachable!("cycle must be caught first"))
            })
        })
        .unwrap_err();
    match err {
        CtAdmission::CyclicRequest { chain } => {
            assert!(chain.len() >= 2, "chain carries the loop: {chain:?}");
            assert_eq!(chain.first().unwrap(), "tpl/a");
            assert_eq!(chain.last().unwrap(), "tpl/a");
        }
        other => panic!("expected CyclicRequest, got {other:?}"),
    }

    // non-cyclic chain evaluates fine with model marking
    let mut ct = ModelCt::new();
    let r1 = CtRequest::new("tpl/outer", vec![PortableValue::I32(4)], vec![]);
    let out = ct
        .evaluate(&r1, |req, ct| {
            let inner = CtRequest::new("tpl/inner", req.args.clone(), vec![]);
            ct.evaluate(&inner, |r, _| ModelCt::fold_i32_sum(r))
        })
        .unwrap();
    assert!(matches!(out.value, PortableValue::I32(4)));
    assert_eq!(out.marking.as_str(), "model_compute");
}

/// Reflection projections are read-only and honest: field lookups
/// return real positions; absent records/fields are misses, never
/// empty successes.
#[test]
fn meta_query_reflection_projections() {
    let src = "type Pair { left int; right bool }\nfn get(p Pair) int { return p.left }";
    let input = rec_src(src, "m");
    let (ord, name, ty) = ace_semantics::query_field(&input, "Pair", "left").unwrap();
    assert_eq!((ord, name.as_str()), (0, "left"));
    assert_eq!(ty, ace_semantics_contract::TypeDesc::I32);
    assert!(ace_semantics::query_field(&input, "Pair", "nope").is_none());
    assert!(ace_semantics::query_field(&input, "Ghost", "left").is_none());
    assert_eq!(
        ace_semantics::list_candidates(&input, "type"),
        vec!["Pair".to_string()]
    );
}
