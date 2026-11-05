//! PLAN-937 CA-D1 foreign-contract conformance: the end-to-end neutral
//! chain typed-decls → facts → physical plan → strict codecs → restore,
//! plus the tamper matrix every persisted consumer replays.

#![cfg(windows)]

use ace_foreign_contract::codec::{facts_codec, plan_codec, typed_decls_codec};
use ace_foreign_contract::lane::{LaneKind, LaneValue};
use ace_foreign_contract::plan::{PhysicalPlan, SlotClass};
use ace_foreign_contract::typed::{TotalityEvidence, TypedBodyFamily, TypedDecl, TypedDecls};

fn checksum_decls() -> TypedDecls {
    TypedDecls {
        format: ace_foreign_contract::typed::TYPED_DECLS_FORMAT.into(),
        id: "t937".into(),
        decls: vec![TypedDecl {
            symbol: "t937_mix".into(),
            params: vec![
                LaneKind::I32,
                LaneKind::U32,
                LaneKind::I64,
                LaneKind::U64,
                LaneKind::F64,
                LaneKind::Bool,
                LaneKind::Char,
                LaneKind::I64,
            ],
            ret: LaneKind::I64,
            body: TypedBodyFamily::ChecksumV1,
            totality: Some(TotalityEvidence::frozen_family("wrapping mix only")),
            error_model: None,
            ctx: None,
            outs: vec![],
        }],
    }
}

/// The full neutral chain survives encode → decode → re-project with
/// identical digests (the bridge restore invariant's model half).
#[test]
fn neutral_chain_roundtrips_with_stable_digests() {
    let decls = checksum_decls();
    let facts = decls.validate().unwrap();
    let plan = PhysicalPlan::project(&facts).unwrap();

    let d_bytes = typed_decls_codec::encode(&decls).unwrap();
    let f_bytes = facts_codec::encode(&facts).unwrap();
    let p_bytes = plan_codec::encode(&plan).unwrap();

    let decls2 = typed_decls_codec::decode(&d_bytes).unwrap();
    let facts2 = facts_codec::decode(&f_bytes).unwrap();
    let plan2 = plan_codec::decode(&p_bytes).unwrap();

    assert_eq!(decls, decls2);
    assert_eq!(facts.digest().unwrap(), facts2.digest().unwrap());
    assert_eq!(plan.digest().unwrap(), plan2.digest().unwrap());
    // Re-projection from restored facts is byte-identical: the plan is a
    // pure function of the forms.
    let plan3 = PhysicalPlan::project(&facts2).unwrap();
    assert_eq!(plan.digest().unwrap(), plan3.digest().unwrap());
}

/// Tamper matrix: every mutated persisted field is a named rejection at
/// decode or re-validation — restore never "repairs" foreign edits.
#[test]
fn persisted_tamper_matrix_is_fail_closed() {
    let facts = checksum_decls().validate().unwrap();
    let bytes = facts_codec::encode(&facts).unwrap();
    let doc: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

    let mutations: Vec<(&str, serde_json::Value)> = vec![
        ("format", serde_json::json!("autoabi.contract.facts/2")),
        ("content_version", serde_json::json!("")),
        ("logical", serde_json::json!("")),
        ("functions", serde_json::json!([])),
    ];
    for (field, value) in mutations {
        let mut d = doc.clone();
        d[field] = value;
        let bad = serde_json::to_vec(&d).unwrap();
        let err = facts_codec::decode(&bad).unwrap_err().to_string();
        assert!(!err.is_empty(), "field {field} must reject");
    }
    // symbol tamper inside functions
    let mut d = doc.clone();
    d["functions"][0]["symbol"] = serde_json::json!("t937_ MIX");
    let bad = serde_json::to_vec(&d).unwrap();
    assert!(facts_codec::decode(&bad).is_err());
    // truncation
    assert!(facts_codec::decode(&bytes[..bytes.len() / 2]).is_err());
}

/// The public-lane rules survive the whole chain: no f32 anywhere,
/// bool→u8 / char→u32 projection visible in the plan, stack
/// classification for the 8-lane mixed signature.
#[test]
fn lane_projection_survives_chain() {
    let facts = checksum_decls().validate().unwrap();
    let plan = PhysicalPlan::project(&facts).unwrap();
    let f = plan.function("t937_mix").unwrap();
    if let ace_foreign_contract::PhysForm::Pure { params, .. } = &f.form {
        assert_eq!(params[5].lane, ace_foreign_contract::PhysLane::BoolU8);
        assert_eq!(params[5].lane.c_spell(), "bool");
        assert_eq!(params[6].lane, ace_foreign_contract::PhysLane::CharU32);
        assert!(matches!(params[7].slot, SlotClass::Stack { .. }));
    } else {
        panic!("pure form");
    }
    assert!(facts
        .validate_call_values(
            "t937_mix",
            None,
            &[
                LaneValue::I32(-1),
                LaneValue::U32(u32::MAX),
                LaneValue::I64(i64::MIN),
                LaneValue::U64(u64::MAX),
                LaneValue::F64Bits(f64::NEG_INFINITY.to_bits()),
                LaneValue::Bool(false),
                LaneValue::CharScalar(0),
                LaneValue::I64(7),
            ],
        )
        .is_ok());
}
