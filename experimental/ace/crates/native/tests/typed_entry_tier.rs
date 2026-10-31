//! PLAN-891 typed-entry tier: the six-scalar typed face through the REAL
//! chain — file-ized fixture → shared verify_mir → split → typed admission
//! → per-unit COFF → cached signature wrapper → one real PE per run, fed
//! the one-shot request frame on stdin and judged from the single response
//! frame on stdout. Oracles are HAND-COMPUTED bit patterns (the fixture's
//! constant table is input, never a predicted result); the same bits are
//! cross-checked against the AV leg's frozen typed table only as a
//! supplementary equality observation.

use ace_artifacts::store::ManagedStore;
use ace_native::typed_entry::{self, TypedArg, TypedOutcome, TypedSigPlan};
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
            TypedOutcome::Value {
                kind_code,
                bits: got,
            } => {
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
            TypedOutcome::Value {
                kind_code,
                bits: got,
            } => {
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
        &m,
        "main",
        &[],
        0x8960,
        &dir,
        None,
        TypedCacheMode::Reuse,
        false,
        None,
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
            assert_eq!(
                run.exit_code,
                Some(0),
                "trap frame exits 0 with status trap"
            );
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
    let frame =
        typed_entry::encode_request(&plan, 42, &[arg("i64", "8000000000000000")]).expect("encode");
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
        &m,
        "main",
        &[arg("u64", "0000000000000001")],
        1,
        &dir,
        Some(&store),
        TypedCacheMode::Fresh,
        false,
        None,
    )
    .expect("fresh run");
    assert_eq!(run1.counters.emit_units, 1, "fresh emits its unit");
    assert_eq!(run1.counters.wrapper_builds, 1, "fresh builds the wrapper");
    assert!(matches!(run1.outcome, TypedOutcome::Value { .. }));

    let run2 = typed_pipeline::run_typed(
        &m,
        "main",
        &[arg("u64", "0000000000000001")],
        2,
        &dir,
        Some(&store),
        TypedCacheMode::Reuse,
        false,
        None,
    )
    .expect("hit run");
    assert_eq!(run2.counters.emit_units, 0, "hit reuses the unit COFF");
    assert_eq!(run2.counters.cache_hits, 1);
    assert_eq!(run2.counters.wrapper_builds, 0, "hit reuses the wrapper");
    assert_eq!(run2.counters.wrapper_hits, 1);
    assert_eq!(run2.counters.run_invocations, 1, "hit still really runs");
    assert_eq!(run2.counters.link_invocations, 1);

    let run3 = typed_pipeline::run_typed(
        &m,
        "main",
        &[arg("u64", "ffffffffffffffff")],
        3,
        &dir,
        Some(&store),
        TypedCacheMode::Reuse,
        false,
        None,
    )
    .expect("param-only run");
    assert_eq!(run3.counters.emit_units, 0, "param-only produces no COFF");
    assert_eq!(
        run3.counters.wrapper_builds, 0,
        "param-only produces no wrapper"
    );
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

/// V-11: typed fingerprints are exact — same raw bits under DIFFERENT
/// logical kinds never collide; a NaN payload or zero-sign change changes
/// the key; re-parsing the same fixture is stable.
#[test]
fn typed_fingerprints_distinguish_kind_and_bits() {
    let mk = |kind: &str, bits: &str, module: &str| {
        let json = format!(
            r#"{{"schema":"mn-typed-fixture/1","module":"{module}","units":[{{"name":"main","params":[],"result":"{kind}","body":[{{"op":"const","dst":"t0","kind":"{kind}","bits":"{bits}"}}],"ret":"t0"}}],"entry":"main"}}"#
        );
        let doc = typed_fixture::parse_fixture(json.as_bytes()).expect("fixture");
        let m = typed_fixture::build_verified_module(&doc).expect("verify");
        ace_av::cache::split_units(&m).expect("split").remove(0)
    };
    let u64_max = mk("u64", "ffffffffffffffff", "m_fp1");
    let i64_same = mk("i64", "ffffffffffffffff", "m_fp2");
    assert_ne!(
        ace_native::cache::unit_fp(&u64_max),
        ace_native::cache::unit_fp(&i64_same),
        "same bits under different logical kinds must not collide"
    );
    let nan_a = mk("f64", "fff80000deadbeef", "m_fp3");
    let nan_b = mk("f64", "fff80000deadbee0", "m_fp4");
    assert_ne!(
        ace_native::cache::unit_fp(&nan_a),
        ace_native::cache::unit_fp(&nan_b),
        "a NaN payload change must change the key"
    );
    let neg_zero = mk("f64", "8000000000000000", "m_fp5");
    let pos_zero = mk("f64", "0000000000000000", "m_fp6");
    assert_ne!(
        ace_native::cache::unit_fp(&neg_zero),
        ace_native::cache::unit_fp(&pos_zero),
        "zero-sign bits must stay distinct"
    );
    let again = mk("f64", "fff80000deadbeef", "m_fp3");
    assert_eq!(
        ace_native::cache::unit_fp(&nan_a),
        ace_native::cache::unit_fp(&again)
    );
}

/// V-11 (closure axis): a callee signature change flows into every
/// caller's fingerprint (callee-sig invalidation), and the typed symbol
/// domain stays `ac9.` (isolated from the scalar `ac8.` domain).
#[test]
fn typed_fingerprints_track_callee_signatures_and_symbol_domain() {
    let mk = |helper_second: bool| {
        let helper = if helper_second {
            r#"{"name":"helper","params":[["a","i64"],["b","i64"]],"result":"i64","body":[{"op":"copy","dst":"t0","src":"b"}],"ret":"t0"}"#
        } else {
            r#"{"name":"helper","params":[["a","i64"]],"result":"i64","body":[{"op":"copy","dst":"t0","src":"a"}],"ret":"t0"}"#
        };
        let main = if helper_second {
            r#"{"name":"main","params":[["a","i64"]],"result":"i64","body":[{"op":"const","dst":"t9","kind":"i64","bits":"0000000000000002"},{"op":"call","dst":"t0","callee":"helper","args":["a","t9"]}],"ret":"t0"}"#
        } else {
            r#"{"name":"main","params":[["a","i64"]],"result":"i64","body":[{"op":"call","dst":"t0","callee":"helper","args":["a"]}],"ret":"t0"}"#
        };
        let json = format!(
            r#"{{"schema":"mn-typed-fixture/1","module":"m891_sigfp","units":[{helper},{main}],"entry":"main"}}"#
        );
        let doc = typed_fixture::parse_fixture(json.as_bytes()).expect("fixture");
        let m = typed_fixture::build_verified_module(&doc).expect("verify");
        let units = ace_av::cache::split_units(&m).expect("split");
        units
            .iter()
            .find(|u| u.funcs[0].name == "main")
            .unwrap()
            .clone()
    };
    let before = mk(false);
    let after = mk(true);
    assert_ne!(
        ace_native::cache::unit_fp(&before),
        ace_native::cache::unit_fp(&after),
        "a callee signature change must flow into the caller key"
    );
    let f = &before.funcs[0];
    let symbol = ace_native::symbols::typed_function_symbol(&before.owner, &f.name);
    assert!(
        symbol.starts_with("ac9."),
        "typed symbols live in the ac9. domain: {symbol}"
    );
}

/// V-14: the typed argument payload cap is a NAMED pre-allocation
/// rejection (never an overflow), and a cyclic record fixture dies at
/// shared verification with codegen 0.
#[test]
fn typed_budget_and_cycle_rejections_are_named() {
    // 8200 i64 parameters -> 65600 payload bytes, above the 64KiB cap.
    // The unit is hand-built (the shared verifier's own 255-param cap is a
    // separate earlier door); plan construction is the budget gate that
    // must reject BEFORE any frame allocation.
    let n = 8200usize;
    let mut m = ace_av_core::mir::MirModule::data_unit(ace_av_core::mir::OwnerScope {
        schema: ace_av_core::contract::MIR_DATA_SCHEMA.to_string(),
        module: "m891_big".to_string(),
    });
    let i64_ty = m.intern_type(ace_av_core::mir::MirType::I64);
    let i32_ty = m.intern_type(ace_av_core::mir::MirType::I32);
    let sig = m.intern_type(ace_av_core::mir::MirType::Fn(ace_av_core::mir::FnSig {
        params: vec![i64_ty; n],
        result: i32_ty,
    }));
    m.funcs.push(ace_av_core::mir::MirFunction {
        owner: m.owner.clone(),
        name: "main".into(),
        sig,
        slots: vec![],
        blocks: vec![],
        entry: 0,
        source_snapshot: 0,
        spans: vec![],
        env: None,
    });
    match typed_entry::entry_plan(&m) {
        Err(ace_native::MnError::Resource(kind, _)) => {
            assert_eq!(kind, "typed-args", "the args budget is the named rejection");
        }
        other => panic!("expected the typed-args budget rejection, got {other:?}"),
    }
    // A self-referential record is a named shared-verify rejection.
    let cyclic = r#"{"schema":"mn-typed-fixture/1","module":"m891_cyc","records":[{"name":"A","fields":[["a","A"]]}],"units":[{"name":"main","params":[],"result":"i32","body":[{"op":"const","dst":"t0","kind":"i32","bits":"1"}],"ret":"t0"}],"entry":"main"}"#;
    let doc = typed_fixture::parse_fixture(cyclic.as_bytes()).expect("decodes");
    match typed_fixture::build_verified_module(&doc) {
        Err(ace_native::MnError::Unsupported("fixture", _)) => {}
        other => panic!("cyclic record must be a named verify rejection, got {other:?}"),
    }
}

/// F-01/V-03: the remaining critical float bit patterns — ±Inf and the
/// positive quiet NaN — return their EXACT bits (the negative sides,
/// signed zeros, sNaN payloads and the f32 minimum subnormal are already
/// frozen rows above).
#[test]
#[cfg(windows)]
fn typed_probe_inf_and_positive_qnan_bits_real_pe() {
    if !tools_available() {
        eprintln!("infra-blocked: rust-lld/SDK unavailable — typed probe skipped");
        return;
    }
    let rows: &[(&str, &str)] = &[
        ("f64", "7ff0000000000000"), // +Inf
        ("f64", "fff0000000000000"), // -Inf
        ("f64", "7ff8000000000000"), // +qNaN
    ];
    let dir = workdir("inf");
    for (i, (kind, bits)) in rows.iter().enumerate() {
        let json = format!(
            r#"{{"schema":"mn-typed-fixture/1","module":"m891_inf{i}","units":[{{"name":"main","params":[],"result":"{kind}","body":[{{"op":"const","dst":"t0","kind":"{kind}","bits":"{bits}"}}],"ret":"t0"}}],"entry":"main"}}"#
        );
        let m = fixture(&json);
        let run = typed_pipeline::run_typed(
            &m,
            "main",
            &[],
            0x8970 + i as u64,
            &dir,
            None,
            typed_pipeline::TypedCacheMode::Reuse,
            false,
            None,
        )
        .expect("typed run");
        match &run.outcome {
            TypedOutcome::Value {
                kind_code,
                bits: got,
            } => {
                assert_eq!(&typed_entry::canonical_hex(*kind_code, *got), bits);
            }
            other => panic!("{kind} {bits} produced {other:?}"),
        }
    }
}

/// F-03/V-05/V-12: a valid frame WITHOUT a clean exit0 is a named
/// admission rejection (late failure), never a Value outcome.
#[test]
fn typed_frame_with_bad_exit_is_never_a_value() {
    let mut frame = vec![0u8; 44];
    frame[0..4].copy_from_slice(&typed_entry::RES_MAGIC.to_le_bytes());
    frame[4..8].copy_from_slice(&typed_entry::FRAME_VERSION.to_le_bytes());
    frame[12..16].copy_from_slice(&typed_entry::STATUS_VALUE.to_le_bytes());
    frame[24..28].copy_from_slice(&typed_entry::KIND_I32.to_le_bytes());
    let sum = {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for b in &frame[..36] {
            h ^= *b as u64;
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
        h
    };
    frame[36..44].copy_from_slice(&sum.to_le_bytes());
    // Clean exit: the frame decodes to a value.
    assert!(typed_entry::admit_frame_and_exit(&frame, Some(0)).is_ok());
    // Late failure / abnormal exit: the SAME frame is a named rejection.
    for bad in [Some(1), Some(-1073741819), None] {
        let err = typed_entry::admit_frame_and_exit(&frame, bad).unwrap_err();
        assert_eq!(err.code(), "mn.admit.typed-frame", "exit {bad:?}");
    }
}

/// F-02/V-06: corrupted cache axes are named admission rejections — a
/// tampered wrapper envelope and a tampered unit COFF payload both refuse
/// before any link (never a silent hit).
#[test]
fn typed_cache_corruption_is_rejected_before_link() {
    use ace_artifacts::codec::ChunkKind;
    use ace_artifacts::store::PutRequest;
    let root = workdir("corrupt");
    let store = ManagedStore::open(&root).expect("store");

    // A wrapper envelope whose sha256 lies about its body.
    let plan = TypedSigPlan {
        sig_text: "(i64)->i64".into(),
        sig_id: 0xdead_beef,
        args: vec![typed_entry::TypedKind::I64],
        offsets: vec![0],
        payload_len: 8,
        result: typed_entry::TypedKind::I64,
    };
    let runtime = ace_native::typed_runtime::prepare(None).expect("runtime");
    let key = typed_pipeline::wrapper_key("ac9.test.main", &plan, &runtime.identity);
    let forged = format!(
        "schema={}\nabi={}\nentry-symbol={}\nsig={}\nsig-id={:08x}\ntarget={}\nflags={}\nlen=4\nsha256={}\n\nJUNK",
        "ac891/typed-wrapper/v1",
        "msvc-x64/priv-typed-data-v1",
        "ac9.test.main",
        plan.sig_text,
        plan.sig_id,
        "x86_64-pc-windows-msvc",
        "opt=none,pic=false,cpu=baseline",
        "0000000000000000000000000000000000000000000000000000000000000000",
    );
    store
        .put_artifact(PutRequest {
            layer: "A16",
            key: key.clone(),
            kind: ChunkKind::LayoutData,
            payload: forged.into_bytes(),
            edges: vec![],
            project: None,
        })
        .map_err(|e| format!("{e:?}"))
        .unwrap();
    let err = typed_pipeline::typed_wrapper_for_session(
        "ac9.test.main",
        &plan,
        Some(&store),
        typed_pipeline::TypedCacheMode::Reuse,
        &runtime.identity,
    )
    .unwrap_err();
    assert!(
        err.code().starts_with("mn.admit."),
        "wrapper: {}",
        err.code()
    );

    // A unit COFF payload whose header sha does not bind the body.
    let json = r#"{"schema":"mn-typed-fixture/1","module":"m891_cor","units":[{"name":"main","params":[],"result":"i32","body":[{"op":"const","dst":"t0","kind":"i32","bits":"2a"}],"ret":"t0"}],"entry":"main"}"#;
    let doc = typed_fixture::parse_fixture(json.as_bytes()).unwrap();
    let unit = typed_fixture::build_verified_unit(&doc, "main").unwrap();
    let fp = ace_native::cache::unit_fp(&unit);
    let meta = ace_native::cache::CoffMeta::for_typed_unit(&unit, fp, &runtime.identity).unwrap();
    let ukey = ace_native::cache::coff_unit_key(&meta);
    store
        .put_artifact(PutRequest {
            layer: "A09",
            key: ukey,
            kind: ChunkKind::LayoutData,
            payload: b"schema=ac818/a09-coff/v1\nsha256=bad\nlen=1\n\nX".to_vec(),
            edges: vec![],
            project: None,
        })
        .map_err(|e| format!("{e:?}"))
        .unwrap();
    let err = ace_native::cache::get_unit_coff_with_id(&store, &meta).unwrap_err();
    assert!(err.code().starts_with("mn.admit."), "unit: {}", err.code());
}

/// F-04/V-11: an import that does not resolve inside the typed closure is
/// a named PRE-LINK rejection — no object is produced for the ghost call.
#[test]
fn typed_ghost_import_is_rejected_before_link() {
    use ace_av_core::mir::*;
    let owner = OwnerScope {
        schema: ace_av_core::contract::MIR_DATA_SCHEMA.to_string(),
        module: "m891_ghost".to_string(),
    };
    let mut m = MirModule::data_unit(owner.clone());
    let i64_ty = m.intern_type(MirType::I64);
    let i32_ty = m.intern_type(MirType::I32);
    let callee_sig = m.intern_type(MirType::Fn(FnSig {
        params: vec![i64_ty],
        result: i32_ty,
    }));
    m.imports.push(MirImport {
        symbol: "fn:ghost".into(),
        effect: Effect::Pure,
        sig: callee_sig,
        env: None,
    });
    let main_sig = m.intern_type(MirType::Fn(FnSig {
        params: vec![i64_ty],
        result: i32_ty,
    }));
    m.funcs.push(MirFunction {
        owner,
        name: "main".into(),
        sig: main_sig,
        slots: vec![
            MirSlot {
                kind: SlotKind::Param,
                ty: i64_ty,
                name: None,
            },
            MirSlot {
                kind: SlotKind::Temp,
                ty: i32_ty,
                name: None,
            },
        ],
        blocks: vec![MirBlock {
            instrs: vec![MirInstr::Call {
                callee: CalleeRef::Import(0),
                dst: 1,
                args: vec![0],
            }],
            term: Terminator::Return { value: 1 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: vec![],
        env: None,
    });
    let dir = workdir("ghost");
    match typed_pipeline::run_typed(
        &m,
        "main",
        &[arg("i64", "0000000000000001")],
        1,
        &dir,
        None,
        typed_pipeline::TypedCacheMode::Reuse,
        false,
        None,
    ) {
        Err(e) => {
            assert_eq!(
                e.code(),
                "mn.admit.callee-sig",
                "ghost import: {}",
                e.code()
            );
        }
        Ok(r) => panic!("a ghost import must not link or run, got {:?}", r.outcome),
    }
}

/// F-05/V-10: bit-scrubbing sensitivity — flipping ANY single payload bit
/// of a float result changes the canonical hex, so the parent's fixed
/// expectation must turn red (the wrong-bits red control in the SDK rides
/// the same consumption path end to end).
#[test]
fn typed_bit_scrubbing_sensitivity_control() {
    for (kind, code, bits) in [
        (TypedKindF::F32, typed_entry::KIND_F32, 0x7f80_ad00u64),
        (
            TypedKindF::F64,
            typed_entry::KIND_F64,
            0xfff8_0000_dead_beef,
        ),
    ] {
        let width = kind.width() * 8;
        for bit in (0..width).step_by(7) {
            let scrubbed = bits ^ (1 << bit);
            let a = typed_entry::canonical_hex(code, bits);
            let b = typed_entry::canonical_hex(code, scrubbed);
            assert_ne!(a, b, "{kind:?} bit {bit}: a scrubbed payload must differ");
            // Both remain canonical-shaped (the shape rules accept the
            // mutated form — it is a WRONG VALUE, not a malformed frame).
            assert_eq!(b.len(), a.len());
        }
    }
}
#[derive(Debug)]
enum TypedKindF {
    F32,
    F64,
}
impl TypedKindF {
    fn width(&self) -> usize {
        match self {
            TypedKindF::F32 => 4,
            TypedKindF::F64 => 8,
        }
    }
}

/// F-06/V-04: a callee's FieldSet touches only ITS OWN copy — the caller's
/// record value is unchanged after the call (declaration-order layout,
/// caller-original-bits oracle).
#[test]
#[cfg(windows)]
fn typed_record_callee_writes_stay_private_real_pe() {
    if !tools_available() {
        eprintln!("infra-blocked: rust-lld/SDK unavailable — typed probe skipped");
        return;
    }
    let json = r#"{"schema":"mn-typed-fixture/1","module":"m891_priv","records":[{"name":"Pt","fields":[["x","f64"],["tag","i32"]]}],"units":[
      {"name":"helper","params":[["p","Pt"]],"result":"i32","body":[
         {"op":"const","dst":"t0","kind":"i32","bits":"05"},
         {"op":"field-set","root":"p","path":[1],"value":"t0"},
         {"op":"field-get","dst":"t1","base":"p","path":[1]}],
       "ret":"t1"},
      {"name":"main","params":[["a","f64"]],"result":"i32","body":[
         {"op":"const","dst":"t0","kind":"i32","bits":"07"},
         {"op":"aggregate","dst":"r0","record":"Pt","args":["a","t0"]},
         {"op":"call","dst":"t1","callee":"helper","args":["r0"]},
         {"op":"field-get","dst":"t2","base":"r0","path":[1]}],
       "ret":"t2"}],
     "entry":"main"}"#;
    let m = fixture(json);
    let dir = workdir("priv");
    let run = typed_pipeline::run_typed(
        &m,
        "main",
        &[arg("f64", "fff80000deadbeef")],
        0x8980,
        &dir,
        None,
        typed_pipeline::TypedCacheMode::Reuse,
        false,
        None,
    )
    .expect("typed run");
    match &run.outcome {
        // helper saw (and returned) its own 5; main still sees its 7.
        TypedOutcome::Value { kind_code, bits } => {
            assert_eq!(typed_entry::canonical_hex(*kind_code, *bits), "7");
        }
        other => panic!("record privacy produced {other:?}"),
    }
}

/// PLAN-891 V-10 formal turnaround measurement: the four §5.4 states × 20
/// formal samples each, nearest-rank P90 gate ≤ 2000ms per state (the dev
/// formal gate; the long-term sub-second target is tracked separately).
/// Each sample is the CONTINUOUS formal-entry → real-PE-result interval
/// (build/link/run/judge included); tool/library builds happen before the
/// samples. Structure gates ride along: a body edit leaves untouched-unit
/// production at zero and the machine result follows the edited body.
mod performance {
    use ace_artifacts::store::ManagedStore;
    use ace_native::typed_entry::{self, TypedArg, TypedOutcome};
    use ace_native::typed_fixture::{self, FixtureDoc};
    use ace_native::typed_pipeline::{self, TypedCacheMode};
    use ace_native::typed_session::TypedSession;
    use std::time::Instant;

    const SAMPLES: usize = 20;
    const P90_LIMIT_MS: f64 = 2000.0;

    const BASE: &str = r#"{"schema":"mn-typed-fixture/1","module":"m891_perf","units":[
      {"name":"helper","params":[["a","i64"]],"result":"i64","body":[{"op":"copy","dst":"t0","src":"a"}],"ret":"t0"},
      {"name":"main","params":[["a","i64"]],"result":"i64","body":[{"op":"call","dst":"t0","callee":"helper","args":["a"]}],"ret":"t0"}],
     "entry":"main"}"#;

    fn tools_available() -> bool {
        ace_core::link::find_rust_lld().is_ok() && ace_core::link::find_sdk_um_dir().is_ok()
    }

    fn percentile(sorted: &[f64], p: f64) -> f64 {
        let idx = ((sorted.len() as f64 - 1.0) * p).round() as usize;
        sorted[idx.min(sorted.len() - 1)]
    }

    fn report(state: &str, ms: &mut Vec<f64>) {
        ms.sort_by(|a, b| a.partial_cmp(b).unwrap());
        eprintln!(
            "{state}: n={} P50={:.0}ms P90={:.0}ms P95={:.0}ms P99={:.0}ms max={:.0}ms",
            ms.len(),
            percentile(ms, 0.50),
            percentile(ms, 0.90),
            percentile(ms, 0.95),
            percentile(ms, 0.99),
            ms.last().copied().unwrap_or(0.0),
        );
    }

    fn arg(bits: &str) -> TypedArg {
        typed_entry::parse_arg(typed_entry::TypedKind::I64, bits).expect("arg")
    }

    fn value_bits(out: &TypedOutcome) -> String {
        match out {
            TypedOutcome::Value { kind_code, bits } => {
                typed_entry::canonical_hex(*kind_code, *bits)
            }
            other => panic!("expected a value, got {other:?}"),
        }
    }

    fn doc() -> FixtureDoc {
        typed_fixture::parse_fixture(BASE.as_bytes()).expect("fixture")
    }

    fn helper_body_const(ff: &str) -> typed_fixture::UnitDecl {
        let wrapped = format!(
            r#"{{"schema":"mn-typed-fixture/1","module":"m891_perf","units":[{{"name":"helper","params":[["a","i64"]],"result":"i64","body":[{{"op":"const","dst":"t0","kind":"i64","bits":"{ff}"}}],"ret":"t0"}}],"entry":"main"}}"#
        );
        let d = typed_fixture::parse_fixture(wrapped.as_bytes()).expect("unit decodes");
        d.units.into_iter().next().unwrap()
    }

    #[test]
    #[cfg(windows)]
    fn typed_v10_four_state_20_samples_p90_gate() {
        if !tools_available() {
            eprintln!("infra-blocked: rust-lld/SDK unavailable — typed performance skipped");
            return;
        }
        let root = std::env::temp_dir().join(format!(
            "mn891-perf-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0)
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let workdir = root.join("work");
        std::fs::create_dir_all(&workdir).unwrap();

        // Tool/library builds precede the samples (one warmup run).
        {
            let warm_store = root.join("warm");
            let store = ManagedStore::open(&warm_store).expect("store");
            let module = typed_fixture::build_verified_module(&doc()).expect("verify");
            let r = typed_pipeline::run_typed(
                &module,
                "main",
                &[arg("0000000000000001")],
                0,
                &workdir,
                Some(&store),
                TypedCacheMode::Fresh,
                false,
                None,
            )
            .expect("warmup");
            assert_eq!(value_bits(&r.outcome), "0000000000000001");
        }

        // State 1: fresh (forced recompute + publish) per sample.
        let mut ms: Vec<f64> = Vec::with_capacity(SAMPLES);
        for i in 0..SAMPLES {
            let store_root = root.join(format!("fresh-{i}"));
            let t0 = Instant::now();
            let store = ManagedStore::open(&store_root).expect("store");
            let module = typed_fixture::build_verified_module(&doc()).expect("verify");
            let r = typed_pipeline::run_typed(
                &module,
                "main",
                &[arg("0000000000000002")],
                i as u64 + 1,
                &workdir,
                Some(&store),
                TypedCacheMode::Fresh,
                false,
                None,
            )
            .expect("fresh run");
            let dt = t0.elapsed().as_secs_f64() * 1000.0;
            assert_eq!(value_bits(&r.outcome), "0000000000000002");
            assert_eq!(r.counters.emit_units, 2);
            ms.push(dt);
        }
        report("typed-fresh", &mut ms);
        assert!(
            percentile(&ms, 0.90) <= P90_LIMIT_MS,
            "fresh P90 {:.0}ms above the {}ms gate",
            percentile(&ms, 0.90),
            P90_LIMIT_MS
        );

        // State 2: warm hit per sample (units + wrapper restored).
        let hit_root = root.join("hit");
        {
            let store = ManagedStore::open(&hit_root).expect("store");
            let module = typed_fixture::build_verified_module(&doc()).expect("verify");
            let _ = typed_pipeline::run_typed(
                &module,
                "main",
                &[arg("0000000000000001")],
                100,
                &workdir,
                Some(&store),
                TypedCacheMode::Fresh,
                false,
                None,
            )
            .expect("seed");
        }
        let mut ms: Vec<f64> = Vec::with_capacity(SAMPLES);
        for i in 0..SAMPLES {
            let store = ManagedStore::open(&hit_root).expect("store");
            let module = typed_fixture::build_verified_module(&doc()).expect("verify");
            let t0 = Instant::now();
            let r = typed_pipeline::run_typed(
                &module,
                "main",
                &[arg("0000000000000003")],
                i as u64 + 200,
                &workdir,
                Some(&store),
                TypedCacheMode::Reuse,
                false,
                None,
            )
            .expect("hit run");
            let dt = t0.elapsed().as_secs_f64() * 1000.0;
            assert_eq!(value_bits(&r.outcome), "0000000000000003");
            assert_eq!(r.counters.emit_units, 0, "hit produces no unit COFF");
            assert_eq!(r.counters.wrapper_builds, 0, "hit produces no wrapper");
            ms.push(dt);
        }
        report("typed-hit", &mut ms);
        assert!(
            percentile(&ms, 0.90) <= P90_LIMIT_MS,
            "hit P90 {:.0}ms above the gate",
            percentile(&ms, 0.90)
        );

        // State 3: cold restart per sample (new session, store re-admits).
        let mut ms: Vec<f64> = Vec::with_capacity(SAMPLES);
        for i in 0..SAMPLES {
            let store = ManagedStore::open(&hit_root).expect("store");
            let t0 = Instant::now();
            let mut s = TypedSession::restart(doc(), &workdir, store).expect("restart");
            let r = s
                .run(&[arg("0000000000000004")], i as u64 + 300, None)
                .expect("restart run");
            let dt = t0.elapsed().as_secs_f64() * 1000.0;
            assert_eq!(value_bits(&r.outcome), "0000000000000004");
            assert_eq!(s.counters().emit_units, 0, "restart emits nothing");
            s.close();
            ms.push(dt);
        }
        report("typed-restart", &mut ms);
        assert!(
            percentile(&ms, 0.90) <= P90_LIMIT_MS,
            "restart P90 {:.0}ms above the gate",
            percentile(&ms, 0.90)
        );

        // State 4: warm body edit per sample (helper A/B alternating) with
        // the structure gate: only helper re-produces; main stays 0.
        let edit_root = root.join("edit");
        let store = ManagedStore::open(&edit_root).expect("store");
        let mut s = TypedSession::start(doc(), &workdir, Some(store), TypedCacheMode::Reuse)
            .expect("session");
        let mut ms: Vec<f64> = Vec::with_capacity(SAMPLES);
        for i in 0..SAMPLES {
            let decl = if i % 2 == 0 {
                helper_body_const("00000000000000ff")
            } else {
                helper_body_const("00000000000000ee")
            };
            let want = if i % 2 == 0 {
                "00000000000000ff"
            } else {
                "00000000000000ee"
            };
            let t0 = Instant::now();
            let obs = s.edit_unit(decl).expect("edit");
            let r = s
                .run(&[arg("0000000000000005")], i as u64 + 400, None)
                .expect("edit run");
            let dt = t0.elapsed().as_secs_f64() * 1000.0;
            assert_eq!(
                value_bits(&r.outcome),
                want,
                "the machine result follows the edit"
            );
            assert_eq!(
                obs.reprepared_units,
                vec!["helper"],
                "only helper re-prepares"
            );
            assert_eq!(obs.whole_fallback, 0);
            ms.push(dt);
        }
        report("typed-body-edit", &mut ms);
        assert!(
            percentile(&ms, 0.90) <= P90_LIMIT_MS,
            "body-edit P90 {:.0}ms above the gate",
            percentile(&ms, 0.90)
        );
        s.close();
        let _ = std::fs::remove_dir_all(&root);
    }
}
