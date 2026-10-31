//! PLAN-891 typed-entry tier: the six-scalar typed face through the REAL
//! chain — file-ized fixture → shared verify_mir → split → typed admission
//! → per-unit COFF → cached signature wrapper → one real PE per run, fed
//! the one-shot request frame on stdin and judged from the single response
//! frame on stdout. Oracles are HAND-COMPUTED bit patterns (the fixture's
//! constant table is input, never a predicted result); the same bits are
//! cross-checked against the AV leg's frozen typed table only as a
//! supplementary equality observation.

use ace_artifacts::store::ManagedStore;
use ace_native::typed_entry::{
    self, TypedArg, TypedOutcome, TypedSigPlan,
};
use ace_native::typed_fixture;
use ace_native::typed_pipeline::{self, TypedCacheMode};

fn tools_available() -> bool {
    ace_core::link::find_rust_lld().is_ok() && ace_core::link::find_sdk_um_dir().is_ok()
}

fn workdir(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("mn891-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("workdir");
    dir
}

fn fixture(json: &str) -> ace_av_core::mir::MirModule {
    let doc = typed_fixture::parse_fixture(json.as_bytes()).expect("fixture decodes");
    match typed_fixture::build_verified_module(&doc) {
        Ok(m) => m,
        Err(e) => panic!("fixture rejected: {e}"),
    }
}

fn arg(kind: &str, bits: &str) -> TypedArg {
    let k = typed_entry::TypedKind::from_label(kind).expect("kind");
    typed_entry::parse_arg(k, bits).expect("arg")
}

/// The frozen five W-row constant fixtures (binding.tsv W01–W05) plus the
/// bool/i32 faces — each returns its constant from a REAL typed PE run.
#[test]
#[cfg(windows)]
fn typed_probe_const_return_all_scalars_real_pe() {
    if !tools_available() {
        eprintln!("infra-blocked: rust-lld/SDK unavailable — typed probe skipped");
        return;
    }
    // (kind, const bits, expected canonical bits) — hand-computed oracle.
    let rows: &[(&str, &str, &str)] = &[
        ("i64", "8000000000000000", "8000000000000000"), // W01 i64-min
        ("u64", "ffffffffffffffff", "ffffffffffffffff"), // W02 u64-max
        ("f64", "8000000000000000", "8000000000000000"), // W03 negative zero
        ("f64", "fff80000deadbeef", "fff80000deadbeef"), // W04 NaN payload
        ("f32", "7f80ad00", "7f80ad00"),                 // W05 sNaN payload
        ("bool", "0", "0"),
        ("i32", "2a", "2a"),
    ];
    let dir = workdir("const");
    for (i, (kind, bits, want)) in rows.iter().enumerate() {
        let json = format!(
            r#"{{"schema":"mn-typed-fixture/1","module":"m891_c{i}","units":[{{"name":"main","params":[],"result":"{kind}","body":[{{"op":"const","dst":"t0","kind":"{kind}","bits":"{bits}"}}],"ret":"t0"}}],"entry":"main"}}"#
        );
        let m = fixture(&json);
        let run = typed_pipeline::run_typed(
            &m,
            "main",
            &[],
            0x8910 + i as u64,
            &dir,
            None,
            TypedCacheMode::Reuse,
            false,
            None,
        )
        .expect("typed run");
        match &run.outcome {
            TypedOutcome::Value { kind_code, bits } => {
                assert_eq!(
                    typed_entry::canonical_hex(*kind_code, *bits),
                    *want,
                    "{kind} const-return bits"
                );
                assert_eq!(run.exit_code, Some(0), "value frame exits 0");
            }
            other => panic!("{kind} const-return produced {other:?}"),
        }
    }
}

/// V-02 first half: identity(input) through the REAL typed entry — two
/// i64 inputs differing only in the HIGH bits must come back distinct
/// (a truncating or wrong-width channel cannot pass).
#[test]
#[cfg(windows)]
fn typed_probe_identity_distinguishes_high_bits_real_pe() {
    if !tools_available() {
        eprintln!("infra-blocked: rust-lld/SDK unavailable — typed probe skipped");
        return;
    }
    let json = r#"{"schema":"mn-typed-fixture/1","module":"m891_ident","units":[{"name":"main","params":[["a","i64"]],"result":"i64","body":[{"op":"copy","dst":"t0","src":"a"}],"ret":"t0"}],"entry":"main"}"#;
    let m = fixture(json);
    let dir = workdir("ident");
    for (i, bits) in ["0000000100000001", "0000000200000001"].iter().enumerate() {
        let run = typed_pipeline::run_typed(
            &m,
            "main",
            &[arg("i64", bits)],
            0x8920 + i as u64,
            &dir,
            None,
            TypedCacheMode::Reuse,
            false,
            None,
        )
        .expect("typed run");
        match &run.outcome {
            TypedOutcome::Value { kind_code, bits: got } => {
                assert_eq!(typed_entry::canonical_hex(*kind_code, *got), *bits);
            }
            other => panic!("identity produced {other:?}"),
        }
    }
}

/// V-02: six mixed parameters crossing the Win64 register/stack boundary;
/// each selection returns exactly the chosen parameter's original bits
/// (position/truncation errors cannot pass).
#[test]
#[cfg(windows)]
fn typed_probe_mixed_params_each_selection_real_pe() {
    if !tools_available() {
        eprintln!("infra-blocked: rust-lld/SDK unavailable — typed probe skipped");
        return;
    }
    let params: [(&str, &str); 6] = [
        ("i32", "1234567f"),
        ("i64", "1122334455667788"),
        ("f32", "7f80ad00"),
        ("bool", "1"),
        ("u64", "ffffffffffffffff"),
        ("f64", "fff80000deadbeef"),
    ];
    let dir = workdir("mixed");
    for (sel, (kind, bits)) in params.iter().enumerate() {
        let plist = params
            .iter()
            .enumerate()
            .map(|(i, (k, _))| format!("[\"p{i}\",\"{k}\"]"))
            .collect::<Vec<_>>()
            .join(",");
        let json = format!(
            r#"{{"schema":"mn-typed-fixture/1","module":"m891_mix{sel}","units":[{{"name":"main","params":[{plist}],"result":"{kind}","body":[{{"op":"copy","dst":"t0","src":"p{sel}"}}],"ret":"t0"}}],"entry":"main"}}"#
        );
        let m = fixture(&json);
        let args: Vec<TypedArg> = params.iter().map(|(k, b)| arg(k, b)).collect();
        let run = typed_pipeline::run_typed(
            &m,
            "main",
            &args,
            0x8930 + sel as u64,
            &dir,
            None,
            TypedCacheMode::Reuse,
            false,
            None,
        )
        .expect("typed run");
        match &run.outcome {
            TypedOutcome::Value { kind_code, bits: got } => {
                assert_eq!(
                    typed_entry::canonical_hex(*kind_code, *got),
                    *bits,
                    "selection {sel} ({kind})"
                );
            }
            other => panic!("selection {sel} produced {other:?}"),
        }
    }
}

/// V-02: imported callee in a SEPARATE COFF object — the caller forwards
/// its parameter through a real cross-object call and returns the callee's
/// machine result.
#[test]
#[cfg(windows)]
fn typed_probe_cross_function_separate_coff_real_pe() {
    if !tools_available() {
        eprintln!("infra-blocked: rust-lld/SDK unavailable — typed probe skipped");
        return;
    }
    let json = r#"{"schema":"mn-typed-fixture/1","module":"m891_call","units":[
        {"name":"helper","params":[["a","i64"]],"result":"i64","body":[{"op":"copy","dst":"t0","src":"a"}],"ret":"t0"},
        {"name":"main","params":[["a","i64"]],"result":"i64","body":[{"op":"call","dst":"t0","callee":"helper","args":["a"]}],"ret":"t0"}],
        "entry":"main"}"#;
    let m = fixture(json);
    let dir = workdir("call");
    let run = typed_pipeline::run_typed(
        &m,
        "main",
        &[arg("i64", "8000000000000000")],
        0x8940,
        &dir,
        None,
        TypedCacheMode::Reuse,
        false,
        None,
    )
    .expect("typed run");
    match &run.outcome {
        TypedOutcome::Value { kind_code, bits } => {
            assert_eq!(
                typed_entry::canonical_hex(*kind_code, *bits),
                "8000000000000000"
            );
        }
        other => panic!("cross-function produced {other:?}"),
    }
}

/// V-04: a pure record with a wide field — aggregate then fieldget must
/// round-trip the EXACT field bits (declaration-order layout, no f32→f64
/// widening anywhere).
#[test]
#[cfg(windows)]
fn typed_probe_record_field_roundtrip_real_pe() {
    if !tools_available() {
        eprintln!("infra-blocked: rust-lld/SDK unavailable — typed probe skipped");
        return;
    }
    let json = r#"{"schema":"mn-typed-fixture/1","module":"m891_rec","records":[
        {"name":"Pt","fields":[["x","f64"],["tag","i32"]]}],
        "units":[{"name":"main","params":[["a","f64"]],"result":"f64","body":[
            {"op":"const","dst":"t0","kind":"i32","bits":"2a"},
            {"op":"aggregate","dst":"r0","record":"Pt","args":["a","t0"]},
            {"op":"field-get","dst":"t1","base":"r0","path":[0]}],
            "ret":"t1"}],"entry":"main"}"#;
    let m = fixture(json);
    let dir = workdir("rec");
    // W04's negative-NaN payload through a record field.
    let run = typed_pipeline::run_typed(
        &m,
        "main",
        &[arg("f64", "fff80000deadbeef")],
        0x8950,
        &dir,
        None,
        TypedCacheMode::Reuse,
        false,
        None,
    )
    .expect("typed run");
    match &run.outcome {
        TypedOutcome::Value { kind_code, bits } => {
            assert_eq!(
                typed_entry::canonical_hex(*kind_code, *bits),
                "fff80000deadbeef"
            );
        }
        other => panic!("record round-trip produced {other:?}"),
    }
}

/// V-05: a typed i32 value of 70 is a VALUE (exit 0 + frame), never a
/// trap; an actual division-by-zero is a TRAP frame with a bounded reason
/// and no value payload — the two are mutually exclusive.
#[test]
#[cfg(windows)]
fn typed_probe_value70_and_div_trap_are_distinct_real_pe() {
    if !tools_available() {
        eprintln!("infra-blocked: rust-lld/SDK unavailable — typed probe skipped");
        return;
    }
    let dir = workdir("trap");
    let val70 = r#"{"schema":"mn-typed-fixture/1","module":"m891_v70","units":[{"name":"main","params":[],"result":"i32","body":[{"op":"const","dst":"t0","kind":"i32","bits":"46"}],"ret":"t0"}],"entry":"main"}"#;
    let m = fixture(val70);
    let run = typed_pipeline::run_typed(
        &m, "main", &[], 0x8960, &dir, None, TypedCacheMode::Reuse, false, None,
    )
    .expect("typed run");
    match &run.outcome {
        TypedOutcome::Value { kind_code, bits } => {
            assert_eq!(typed_entry::canonical_hex(*kind_code, *bits), "46");
            assert_eq!(run.exit_code, Some(0), "value 70 exits 0 with a frame");
        }
        other => panic!("value 70 produced {other:?}"),
    }

    let divz = r#"{"schema":"mn-typed-fixture/1","module":"m891_divz","units":[{"name":"main","params":[["a","i32"],["b","i32"]],"result":"i32","body":[{"op":"bin","dst":"t0","oper":"div","lhs":"a","rhs":"b"}],"ret":"t0"}],"entry":"main"}"#;
    let m = fixture(divz);
    let run = typed_pipeline::run_typed(
        &m,
        "main",
        &[arg("i32", "1"), arg("i32", "0")],
        0x8961,
        &dir,
        None,
        TypedCacheMode::Reuse,
        false,
        None,
    )
    .expect("typed run");
    match &run.outcome {
        TypedOutcome::Trap { reason } => {
            assert_eq!(*reason, typed_entry::TRAP_DIV_ZERO, "named div-by-zero");
            assert_eq!(run.exit_code, Some(0), "trap frame exits 0 with status trap");
        }
        other => panic!("division by zero produced {other:?}"),
    }
}

/// Host-side frame codec negatives (V-05/V-12 axis): every shape
/// violation is a NAMED rejection — a success-looking fragment never
/// passes.
#[test]
fn typed_frame_codec_rejects_protocol_violations() {
    let plan = TypedSigPlan {
        sig_text: "(i64)->i64".into(),
        sig_id: 0x1234_5678,
        args: vec![typed_entry::TypedKind::I64],
        offsets: vec![0],
        payload_len: 8,
        result: typed_entry::TypedKind::I64,
    };
    let frame = typed_entry::encode_request(&plan, 42, &[arg("i64", "8000000000000000")])
        .expect("encode");
    assert_eq!(frame.len(), plan.frame_len());
    // Argument-count mismatch.
    assert!(typed_entry::encode_request(&plan, 42, &[]).is_err());
    // Argument-kind mismatch.
    assert!(typed_entry::encode_request(&plan, 42, &[arg("i32", "2a")]).is_err());
    // Non-canonical / wrong-width / bool≠0,1 arguments.
    assert!(typed_entry::parse_arg(typed_entry::TypedKind::I64, "123").is_err());
    assert!(typed_entry::parse_arg(typed_entry::TypedKind::Bool, "2").is_err());
    assert!(typed_entry::parse_arg(typed_entry::TypedKind::F32, "deadbeefdeadbeef").is_err());

    // Response decode: a good frame passes; each corruption is named.
    let mut good = vec![0u8; 44];
    good[0..4].copy_from_slice(&typed_entry::RES_MAGIC.to_le_bytes());
    good[4..8].copy_from_slice(&typed_entry::FRAME_VERSION.to_le_bytes());
    good[8..12].copy_from_slice(&plan.sig_id.to_le_bytes());
    good[12..16].copy_from_slice(&typed_entry::STATUS_VALUE.to_le_bytes());
    good[16..24].copy_from_slice(&42u64.to_le_bytes());
    good[24..28].copy_from_slice(&typed_entry::KIND_I64.to_le_bytes());
    good[28..36].copy_from_slice(&0x8000_0000_0000_0000u64.to_le_bytes());
    let sum = ace_native::typed_entry::decode_response(&good); // checksum missing yet
    assert!(sum.is_err(), "checksum must close the frame");
    let checksum = {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for b in &good[..36] {
            h ^= *b as u64;
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
        h
    };
    good[36..44].copy_from_slice(&checksum.to_le_bytes());
    let (sig, nonce, out) = typed_entry::decode_response(&good).expect("good frame decodes");
    assert_eq!((sig, nonce), (plan.sig_id, 42));
    assert!(matches!(out, TypedOutcome::Value { .. }));
    // Trailing byte / short frame / magic corruption all reject.
    let mut trailing = good.clone();
    trailing.push(0);
    assert!(typed_entry::decode_response(&trailing).is_err());
    assert!(typed_entry::decode_response(&good[..43]).is_err());
    let mut bad_magic = good.clone();
    bad_magic[0] ^= 0xFF;
    assert!(typed_entry::decode_response(&bad_magic).is_err());
    let mut bad_sum = good.clone();
    bad_sum[40] ^= 0xFF;
    assert!(typed_entry::decode_response(&bad_sum).is_err());
}

/// Cache discipline (V-06/V-07 first slice): fresh build produces units;
/// the SAME module again is a full hit (emit 0) that still actually RUNS;
/// a param-only change produces neither unit COFFs nor the wrapper while
/// the machine result follows the new arguments.
#[test]
#[cfg(windows)]
fn typed_cache_fresh_hit_and_param_only_rerun() {
    if !tools_available() {
        eprintln!("infra-blocked: rust-lld/SDK unavailable — typed cache tier skipped");
        return;
    }
    let json = r#"{"schema":"mn-typed-fixture/1","module":"m891_cache","units":[{"name":"main","params":[["a","u64"]],"result":"u64","body":[{"op":"copy","dst":"t0","src":"a"}],"ret":"t0"}],"entry":"main"}"#;
    let m = fixture(json);
    let root = workdir("cache-store");
    let store = ManagedStore::open(&root).expect("store");
    let dir = workdir("cache");

    let run1 = typed_pipeline::run_typed(
        &m, "main", &[arg("u64", "0000000000000001")], 1, &dir,
        Some(&store), TypedCacheMode::Fresh, false, None,
    )
    .expect("fresh run");
    assert_eq!(run1.counters.emit_units, 1, "fresh emits its unit");
    assert_eq!(run1.counters.wrapper_builds, 1, "fresh builds the wrapper");
    assert!(matches!(run1.outcome, TypedOutcome::Value { .. }));

    let run2 = typed_pipeline::run_typed(
        &m, "main", &[arg("u64", "0000000000000001")], 2, &dir,
        Some(&store), TypedCacheMode::Reuse, false, None,
    )
    .expect("hit run");
    assert_eq!(run2.counters.emit_units, 0, "hit reuses the unit COFF");
    assert_eq!(run2.counters.cache_hits, 1);
    assert_eq!(run2.counters.wrapper_builds, 0, "hit reuses the wrapper");
    assert_eq!(run2.counters.wrapper_hits, 1);
    assert_eq!(run2.counters.run_invocations, 1, "hit still really runs");
    assert_eq!(run2.counters.link_invocations, 1);

    let run3 = typed_pipeline::run_typed(
        &m, "main", &[arg("u64", "ffffffffffffffff")], 3, &dir,
        Some(&store), TypedCacheMode::Reuse, false, None,
    )
    .expect("param-only run");
    assert_eq!(run3.counters.emit_units, 0, "param-only produces no COFF");
    assert_eq!(run3.counters.wrapper_builds, 0, "param-only produces no wrapper");
    assert_eq!(run3.counters.run_invocations, 1);
    match &run3.outcome {
        TypedOutcome::Value { kind_code, bits } => {
            assert_eq!(
                typed_entry::canonical_hex(*kind_code, *bits),
                "ffffffffffffffff",
                "the machine result follows the NEW arguments"
            );
        }
        other => panic!("param-only rerun produced {other:?}"),
    }
}
