//! V-02 admission/trust tests: tampered HIR payloads, unknown versions,
//! corrupt chunks and out-of-subset features are refused by the real
//! reader/verifier; the legal sibling passes. A persisted payload NEVER
//! becomes trust.

mod common;

use ac_artifact_probe::codec::{
    self, ChunkKind, CodecError,
};
use ac_artifact_probe::driver::{vertical_and_run, VerticalCfg};
use ac_artifact_probe::store::{ManagedStore, PutRequest};
use ac_artifact_probe::{identity, schema};
use auto_ac_prototype::hir::{ExprKind, TypeUse};
use common::scratch;

fn core_bundle() -> auto_ac_prototype::hir::Bundle {
    let src = "fn main() int { var i = 0; while i < 3 { i = i + 1 }; return i }";
    auto_ac_prototype::source::compile_parts_with(
        src,
        "m",
        &[],
        auto_ac_prototype::source::SourceProfile::Core,
    )
    .unwrap()
    .bundle
}

#[test]
fn admission_corrupt_hir_chunk_rejected_by_reader() {
    let bundle = core_bundle();
    let chunk = codec::encode_bundle_chunk(&bundle).unwrap();
    // Flip one payload byte, keeping the framing hash stale → checksum.
    let mut bad = chunk.clone();
    let last = bad.len() - 1;
    bad[last] ^= 0x01;
    assert!(matches!(
        codec::decode_bundle_chunk(&bad),
        Err(CodecError::ChecksumMismatch { .. })
    ));
    // Re-frame the tampered payload with a VALID checksum: the reader must
    // still reject structurally (or the verifier semantically) — here the
    // flip lands in span bytes and stays well-formed, so prove the reader
    // path at the structural level with a known-bad reframe instead:
    let (_, payload) = codec::unframe_chunk(&chunk).unwrap();
    let mut p = payload.to_vec();
    p[0] = 99; // unknown codec version
    let reframed = codec::frame_chunk(ChunkKind::Hir, &p);
    assert!(matches!(
        codec::decode_bundle_chunk(&reframed),
        Err(CodecError::UnsupportedVersion(99))
    ));
    // Wrong chunk kind for the HIR decoder.
    let wrong = codec::frame_chunk(ChunkKind::Object, &p);
    assert!(matches!(
        codec::decode_bundle_chunk(&wrong),
        Err(CodecError::UnknownChunkKind(_))
    ));
}

#[test]
fn admission_semantic_tamper_rejected_by_real_verifier() {
    let mut bundle = core_bundle();
    // Well-formed but semantically broken: retype a ReadLocal of an i32
    // local as bool — the REAL verifier must reject (type mismatch).
    let body = &mut bundle.modules[0].bodies[0];
    for e in body.exprs.iter_mut() {
        if let ExprKind::ReadLocal { .. } = e.kind {
            e.ty = TypeUse::Builtin(auto_ac_prototype::hir::BuiltinType::Bool);
            break;
        }
    }
    let chunk = codec::encode_bundle_chunk(&bundle).unwrap();
    let restored = codec::decode_bundle_chunk(&chunk).unwrap(); // reader OK
    let err = auto_ac_prototype::verify::verify(restored).unwrap_err();
    assert!(
        err.iter().any(|d| d.code.starts_with("verify.")),
        "real verifier must reject the tampered bundle: {:?}",
        err.iter().map(|d| d.code.clone()).collect::<Vec<_>>()
    );
    // Legal sibling: the untampered bundle round-trips AND verifies.
    let clean = core_bundle();
    let chunk = codec::encode_bundle_chunk(&clean).unwrap();
    let restored = codec::decode_bundle_chunk(&chunk).unwrap();
    assert!(auto_ac_prototype::verify::verify(restored).is_ok());
}

#[test]
fn admission_owner_tamper_rejected_by_real_verifier() {
    let mut bundle = core_bundle();
    // Re-point a Binary's rhs at an expr of a different kind/owner
    // semantics: use an out-of-table index → reader refuses; in-table but
    // type-wrong → verifier refuses. Take an if-cond bool expr and feed it
    // to an i32 add: reader accepts (well-formed), verifier rejects.
    let body = &mut bundle.modules[0].bodies[0];
    // find a Binary add
    let mut add = None;
    for (i, e) in body.exprs.iter().enumerate() {
        if let ExprKind::Binary { op: auto_ac_prototype::hir::BinOp::AddI32, .. } = e.kind {
            add = Some(i);
            break;
        }
    }
    let Some(add) = add else { panic!("fixture must contain an add") };
    // find a bool-PRODUCING expr: a comparison (the core surface's while
    // condition lowers to a Lt binary; bool arrives as a named type ref).
    let bool_expr = body
        .exprs
        .iter()
        .position(|e| {
            matches!(
                e.kind,
                ExprKind::Binary { op: auto_ac_prototype::hir::BinOp::LtI32, .. }
            )
        })
        .expect("fixture has a comparison");
    if let ExprKind::Binary { rhs, .. } = &mut body.exprs[add].kind {
        *rhs = auto_ac_prototype::hir::ExprId(bool_expr as u32);
    }
    let chunk = codec::encode_bundle_chunk(&bundle).unwrap();
    let restored = codec::decode_bundle_chunk(&chunk).unwrap();
    assert!(
        auto_ac_prototype::verify::verify(restored).is_err(),
        "type-crossing operand must fail the real verifier"
    );
}

#[test]
fn admission_store_chunk_corruption_is_controlled() {
    let root = scratch("admission-store");
    let store = ManagedStore::open(&root.join("store")).unwrap();
    let key = identity::QueryKey::new(
        schema::layer("A07").unwrap(),
        vec![("probe".into(), "corrupt".into())],
    );
    let payload = codec::encode_bundle_chunk(&core_bundle()).unwrap();
    store
        .put_artifact(PutRequest {
            layer: "A07",
            key: key.clone(),
            kind: ChunkKind::Hir,
            payload,
            edges: vec![],
            project: None,
        })
        .unwrap();
    // Corrupt the stored blob bytes on disk.
    let loaded = store.get_artifact("A07", &key).unwrap().unwrap();
    let blob_path = root
        .join("store")
        .join("blocks")
        .join("blobs")
        .join(&loaded.chunk_id[..2])
        .join(&loaded.chunk_id);
    let bytes = std::fs::read(&blob_path).unwrap();
    let mut bad = bytes.clone();
    let last = bad.len() - 1;
    bad[last] ^= 0x7F;
    std::fs::write(&blob_path, &bad).unwrap();
    // Reader must fail controlled (checksum), never return wrong bytes.
    match store.get_artifact("A07", &key) {
        Err(ac_artifact_probe::store::StoreError::Codec(
            CodecError::ChecksumMismatch { .. },
        )) => {}
        other => panic!("expected controlled checksum rejection, got {:?}", other.map(|o| o.map(|a| a.chunk_id))),
    }
    let _ = bytes;
}

#[test]
fn admission_unknown_schema_and_future_layers_refused() {
    // Unknown layer id is not in the registry.
    assert!(schema::layer("A99").is_none());
    // Future layers refuse production through probe machinery.
    for future in ["A01", "A02", "A05", "A06", "A08", "A11", "A12"] {
        let spec = schema::layer(future).unwrap();
        assert!(
            matches!(spec.coverage, schema::LayerCoverage::Future),
            "{} must be Future",
            future
        );
        assert!(schema::require_producible(spec).is_err());
    }
    // Text/record-domain bundles are outside the frozen codec subset.
    let text_src = "fn main() int { var s = \"ab\"; return s.len_bytes() }";
    let parts = auto_ac_prototype::source::compile_parts_with(
        text_src,
        "m",
        &["ac.runtime.text.v1".to_string()],
        auto_ac_prototype::source::SourceProfile::Text,
    )
    .unwrap();
    assert!(matches!(
        codec::encode_bundle(&parts.bundle),
        Err(CodecError::UnsupportedFeature(_))
    ));
}

#[test]
fn admission_restored_bundle_reverified_before_use_in_driver() {
    // End-to-end: force the HIR-hit path and prove the restored bundle
    // re-enters the real verifier before lowering (counters are the
    // structural proof; the reader-level rejection is covered above).
    let root = scratch("admission-driver");
    let store = ManagedStore::open(&root.join("store")).unwrap();
    let src_path = root.join("m.at");
    std::fs::write(&src_path, "fn main() int { return 6 * 7 }").unwrap();
    let cfg = VerticalCfg { project: "p".into(), source: src_path.clone(), work_dir: root.join("w"), ..Default::default() };
    let out = vertical_and_run(&store, &cfg).unwrap();
    assert_eq!(out.exit_code, Some(42));

    store.purge_layer("A09").unwrap();
    store.purge_layer("A10").unwrap();
    let out2 = vertical_and_run(&store, &cfg).unwrap();
    assert_eq!(out2.exit_code, Some(42));
    assert_eq!(out2.counters.hir_restore, 1, "HIR-hit path must restore");
    assert_eq!(out2.counters.verify, 1, "restored bundle must re-enter the real verifier");
    assert_eq!(out2.counters.parse, 0, "source front end must be skipped");
    assert_eq!(out2.counters.lower, 1, "object miss must re-lower");
}

#[test]
fn admission_probe_lowering_determinism() {
    let src = "fn main() int { var i = 0; while i < 3 { i = i + 1 }; return i }";
    let p1 = auto_ac_prototype::source::compile_parts_with(src, "main", &[], auto_ac_prototype::source::SourceProfile::Core).unwrap();
    let p2 = auto_ac_prototype::source::compile_parts_with(src, "main", &[], auto_ac_prototype::source::SourceProfile::Core).unwrap();
    let c1 = auto_ac_prototype::verify::verify(p1.bundle).unwrap();
    let c2 = auto_ac_prototype::verify::verify(p2.bundle).unwrap();
    let sel1 = auto_ac_prototype::native::find_entry(&c1, "d_main").unwrap();
    let sel2 = auto_ac_prototype::native::find_entry(&c2, "d_main").unwrap();
    let o1 = auto_ac_prototype::native::lower_object(&c1, &sel1).unwrap();
    let o2 = auto_ac_prototype::native::lower_object(&c2, &sel2).unwrap();
    println!("fresh-vs-fresh equal: {}", o1 == o2);
    // restored roundtrip
    let b = c2.into_bundle();
    let chunk = codec::encode_bundle_chunk(&b).unwrap();
    let restored = codec::decode_bundle_chunk(&chunk).unwrap();
    let cr = auto_ac_prototype::verify::verify(restored).unwrap();
    let selr = auto_ac_prototype::native::find_entry(&cr, "d_main").unwrap();
    let or_ = auto_ac_prototype::native::lower_object(&cr, &selr).unwrap();
    println!("fresh-vs-restored equal: {}", o1 == or_);
    assert_eq!(o1, o2, "fresh lowering must be deterministic");
    assert_eq!(o1, or_, "restored-bundle lowering must match fresh");
}
