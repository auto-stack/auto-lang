//! PLAN-828 T-03 — batch-1 review (B1-C01..C05) permanent controls.
//!
//! Collected from the first-batch review's `artifact-controls.rs` and
//! `roundtrip-controls.rs` (same inputs, same oracles) plus the new
//! body-fact query probe and the cold-bundle close-release oracle:
//!
//! - `t03_a07_artifact_admission_controls`: an artifact row that decodes
//!   cleanly but carries a wrong owner / param index / span / forged
//!   intrinsic facts is rejected BEFORE adoption — the full semantic
//!   admission (`verify_function` over the current cold bundle) and the
//!   DECODED body's span digest decide; clean rows and a legal non-zero
//!   owner stay adopted; a rejected function falls back to the cold body
//!   and later content hits never resurrect the corrupted pair.
//! - `t03_a07_roundtrip_plain_and_mixed_intrinsic_adoption`: a mixed
//!   plain+intrinsic caller persists and restores with ALL artifacts
//!   adopted (stable id-text reference tables — B1-C04), second restore,
//!   later edit/hit identity, source/trace no-drift.
//! - `t03_body_fact_query_dependency_invalidation`: a consumer's recorded
//!   body-fact read invalidates exactly when the READ body changes
//!   (C01-03); unread functions and interface-only callers never
//!   invalidate.
//! - `t03_close_releases_cold_bundle_owner`: close() releases the
//!   session-owned cold HIR bundle without drop(session) (B1-C05).

use auto_av_core::av::{execute_program, CancelFlag, ExecLimits};
use auto_av_core::contract::{ExecOutcome, Value};
use std::sync::Arc;

const SRC: &str = "fn helper(x int) int { let r = x * 3; return r + 7 }\nfn main() int { let a = helper(5); return a + 1 }\n";

fn assert_i32(out: &ExecOutcome, want: i32, ctx: &str) {
    match out {
        ExecOutcome::Success {
            value: Value::I32(v),
            ..
        } => assert_eq!(*v, want, "{ctx}"),
        other => panic!("{ctx}: expected Success(I32({want})), got {other:?}"),
    }
}

fn t03_a07_admission_exercise(mode: &str, expected_adopted: u64, expect_skip: Option<&str>) {
    use auto_ac_prototype::hir;
    use auto_ac_prototype::source::compile_scalar_source;
    use auto_ac_prototype::verify::{declaration_context_fingerprint, verify_function};
    use auto_av_probe::cache::{
        canonical_hir_function, caps_fingerprint, open_store, put_enveloped, EnvelopeIdentity,
    };
    use auto_av_probe::fn_hir_codec::{
        decode_fn_hir, encode_fn_hir, fn_hir_key, fn_hir_span_digest, normalize_fn_hir_body,
        rebind_fn_hir_body, FnHirArtifact,
    };
    use auto_av_probe::semrefs::content_version_of;
    use auto_av_probe::session::{restore, warmup_persisted};

    let src: &str = if mode == "nonzero_owner" {
        "fn main() int { let a = helper(5); return a + 1 }\nfn helper(x int) int { let r = x * 3; return r + 7 }\n"
    } else {
        SRC
    };
    let c = compile_scalar_source(src, "probe.at", &[]).unwrap();
    let bundle = c.checked.bundle();
    let m = &bundle.modules[0];
    let di = m.defs.iter().position(|d| d.id_text == "d_helper").unwrap();
    let bid = match m.defs[di].kind {
        hir::DefKind::Function { body, .. } => body,
        _ => panic!("not a function"),
    };
    let original = &m.bodies[bid.0 as usize];
    let cv = content_version_of(&canonical_hir_function(m, bid));
    let caps = caps_fingerprint(&[]);
    // The clean artifact is exactly what the producer deposits: the
    // body normalized through the producing (cold) tables into stable
    // id-text reference tables (codec v2 coordinates).
    let cold_defs: Vec<String> = m.defs.iter().map(|d| d.id_text.clone()).collect();
    let cold_types: Vec<String> = m.types.iter().map(|t| t.id_text.clone()).collect();
    let (norm_body, callee_table, type_table) =
        normalize_fn_hir_body(&cold_defs, &cold_types, original).expect("normalize");
    let mut a = FnHirArtifact {
        def_id: "d_helper".into(),
        content_version: cv.clone(),
        span_digest: fn_hir_span_digest(original),
        context_fingerprint: declaration_context_fingerprint(m),
        caps_fp: caps.clone(),
        intrinsics: vec![],
        callee_table,
        type_table,
        body: norm_body,
    };
    match mode {
        "clean" | "nonzero_owner" => {}
        "context" => a.context_fingerprint = "forged".into(),
        "owner" => a.body.owner = hir::DefId(u32::MAX),
        "owner_inrange" => {
            a.body.owner =
                hir::DefId(m.defs.iter().position(|d| d.id_text == "d_main").unwrap() as u32)
        }
        "span" => {
            a.body.span.start = 999_000;
            a.body.span.end = 999_010;
            a.body.exprs[0].span = a.body.span;
        }
        "param_index" => {
            a.body
                .locals
                .iter_mut()
                .find(|l| matches!(l.role, hir::LocalRole::Param))
                .unwrap()
                .index = Some(u32::MAX)
        }
        "intrinsics" => a.intrinsics = vec!["nonexistent.symbol".into()],
        "literal" => {
            let e = a
                .body
                .exprs
                .iter_mut()
                .find(|e| matches!(e.kind, hir::ExprKind::Constant { .. }))
                .unwrap();
            e.kind = hir::ExprKind::Constant { value: 999 };
        }
        "entry_oob" => a.body.entry = hir::BlockId(u32::MAX),
        _ => panic!("unknown mode {mode}"),
    }
    let bytes = encode_fn_hir(&a).unwrap();
    assert_eq!(
        decode_fn_hir(&bytes).is_ok(),
        mode != "entry_oob",
        "mode={mode}: codec bound — only an out-of-range entry block fails decode (err={:?})",
        decode_fn_hir(&bytes).err()
    );
    // The production verifier's own verdict on the same tampered body —
    // what admission must not be weaker than.
    let rebound = rebind_fn_hir_body(&a, m).expect("rebind premise");
    let mut v = bundle.clone();
    v.modules[0].bodies[bid.0 as usize] = rebound;
    let verifier_accepts = verify_function(v, di).is_ok();
    assert_eq!(
        verifier_accepts,
        matches!(
            mode,
            "clean" | "nonzero_owner" | "span" | "intrinsics" | "literal" | "context"
        ),
        "mode={mode}: control premise — which bodies the verifier itself rejects"
    );

    let dir = std::env::temp_dir().join(format!("p828-b1-adm-{mode}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    {
        let st = open_store(&dir).unwrap();
        let key = fn_hir_key(&m.id_text, "d_helper", &cv, &caps);
        let id = EnvelopeIdentity::fn_hir_bound(&m.id_text, "d_helper", &cv);
        put_enveloped(&st, &key, "A07", &id, &bytes, vec![], Some("p828-b1")).unwrap();
    }
    drop(warmup_persisted(src, "d_main", &[], &dir, "p828-b1").unwrap());
    let mut s = restore(src, "d_main", &[], &dir, "p828-b1").unwrap();
    let cancel = CancelFlag::new(false);
    let (result, _) = s.run(&ExecLimits::default(), &cancel);
    assert_i32(&result, 23, "mode={mode}: execution stays correct");
    let adopted = s.counters().get("restore_hir_adopted");
    // The resident helper equals the COLD truth field-by-field — a
    // corrupted pair never becomes resident (and never rides a hit).
    let (_, restored, _) = s.resident("helper").unwrap();
    let cold_body = &m.bodies[bid.0 as usize];
    assert_eq!(restored.owner, cold_body.owner, "mode={mode}: owner");
    assert_eq!(restored.span, cold_body.span, "mode={mode}: span");
    assert_eq!(
        restored
            .locals
            .iter()
            .find(|l| matches!(l.role, hir::LocalRole::Param))
            .unwrap()
            .index,
        cold_body
            .locals
            .iter()
            .find(|l| matches!(l.role, hir::LocalRole::Param))
            .unwrap()
            .index,
        "mode={mode}: param index"
    );
    if matches!(mode, "owner" | "span" | "param_index") {
        let at = src.find("x * 3").unwrap() as u64 + 4;
        let out4 = s.edit(at, at + 1, b"4".to_vec()).unwrap();
        assert!(out4.ok, "mode={mode}");
        let at4 = s.src().find("x * 4").unwrap() as u64 + 4;
        let out3 = s.edit(at4, at4 + 1, b"3".to_vec()).unwrap();
        assert!(out3.ok, "mode={mode}");
        let (_, hit, _) = s.resident("helper").unwrap();
        assert!(
            Arc::ptr_eq(&restored, &hit),
            "mode={mode}: the content hit reuses the clean body object"
        );
        let (again, _) = s.run(&ExecLimits::default(), &cancel);
        assert_i32(&again, 23, "mode={mode}: post-edit run");
    }
    if let Some(skip) = expect_skip {
        assert_eq!(
            s.counters().get(&format!("restore_hir_skip:{skip}")),
            1,
            "mode={mode}: the {skip} gate rejected the artifact"
        );
    }
    assert_eq!(
        adopted, expected_adopted,
        "mode={mode}: invalid artifact skipped; clean artifact adopted"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn t03_a07_artifact_admission_controls() {
    // (mode, expected adopted, expected rejecting gate)
    let cases: &[(&str, u64, Option<&str>)] = &[
        ("clean", 2, None),
        ("nonzero_owner", 2, None),
        ("context", 1, Some("header")),
        ("literal", 1, Some("canonical")),
        ("entry_oob", 1, Some("codec")),
        ("owner", 1, Some("verify")),
        ("owner_inrange", 1, Some("verify")),
        ("span", 1, Some("span")),
        ("param_index", 1, Some("verify")),
        ("intrinsics", 1, Some("intrinsics")),
    ];
    for (mode, adopted, skip) in cases {
        t03_a07_admission_exercise(mode, *adopted, *skip);
    }
}

fn t03_a07_roundtrip_exercise(name: &str, src: &str, caps: &[String], expected: i32, mixed: bool) {
    use auto_av_probe::session::{restore, warmup_persisted};
    let dir = std::env::temp_dir().join(format!("p828-b1-rt-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let cancel = CancelFlag::new(false);
    let mut s = warmup_persisted(src, "d_main", caps, &dir, "p828-b1").unwrap();
    let at = src.find("+ 1").unwrap() + 2;
    let out = s.edit(at as u64, at as u64 + 1, b"2".to_vec()).unwrap();
    assert!(out.ok, "{out:?}");
    let edited = s.src().to_owned();
    let count = s.function_names().len() as u64;
    let (pre_out, pre_trace) = s.run(&ExecLimits::default(), &cancel);
    assert_i32(&pre_out, expected, "edited run");

    // Persist-side fact (mixed case): the STORED main artifact's callee
    // table resolves through stable id texts and its declared intrinsics
    // are exactly what main's body reads — mark_b, not helper's mark_a.
    if mixed {
        use auto_ac_prototype::hir::DefKind;
        use auto_ac_prototype::source::compile_scalar_source;
        use auto_av_probe::cache::{
            canonical_hir_function, caps_fingerprint, get_enveloped_opt, open_store,
            EnvelopeIdentity,
        };
        use auto_av_probe::fn_hir_codec::{decode_fn_hir, fn_hir_key};
        use auto_av_probe::semrefs::content_version_of;
        let compiled = compile_scalar_source(&edited, "probe.at", caps).unwrap();
        let m = &compiled.checked.bundle().modules[0];
        let mark_b_def = m
            .defs
            .iter()
            .find(|d| matches!(&d.kind, DefKind::Intrinsic { symbol, .. } if symbol == "hir.test.mark_b"))
            .map(|d| d.id_text.clone())
            .expect("mark_b intrinsic def");
        let di = m.defs.iter().position(|d| d.id_text == "d_main").unwrap();
        let bid = match m.defs[di].kind {
            DefKind::Function { body, .. } => body,
            _ => panic!(),
        };
        let cv = content_version_of(&canonical_hir_function(m, bid));
        let st = open_store(&dir).unwrap();
        let key = fn_hir_key(&m.id_text, "d_main", &cv, &caps_fingerprint(caps));
        let id = EnvelopeIdentity::fn_hir_bound(&m.id_text, "d_main", &cv);
        let bytes = get_enveloped_opt(&st, &key, "A07", &id)
            .unwrap()
            .expect("edited main artifact row exists");
        let a = decode_fn_hir(&bytes).unwrap();
        assert!(
            a.callee_table.iter().any(|c| c == &mark_b_def),
            "stored callee table carries mark_b's stable def id ({mark_b_def}): {:?}",
            a.callee_table
        );
        assert_eq!(a.intrinsics, vec!["hir.test.mark_b".to_string()]);
        drop(st);
    }

    // First restore: ALL legitimate current-context artifacts adopt (the
    // B1-C04 oracle — the mixed case used to adopt only 2/3).
    let r = restore(&edited, "d_main", caps, &dir, "p828-b1").unwrap();
    assert_eq!(r.counters().get("restore_hir_adopted"), count);
    assert_eq!(r.counters().get("restore_hir_verified"), count);
    let (out, trace) = r.run(&ExecLimits::default(), &cancel);
    assert_i32(&out, expected, "restored run");
    assert_eq!(trace.events, pre_trace.events, "trace does not drift");

    // Second restore over the same store: still all adopted, same results.
    drop(r);
    let mut r2 = restore(&edited, "d_main", caps, &dir, "p828-b1").unwrap();
    assert_eq!(
        r2.counters().get("restore_hir_adopted"),
        count,
        "second restore adopts all artifacts again"
    );
    let (_, helper_r2, helper_code_r2) = r2.resident("helper").unwrap();
    let (_, tail_r2, tail_code_r2) = r2.resident("tail").unwrap();
    let (out2, trace2) = r2.run(&ExecLimits::default(), &cancel);
    assert_i32(&out2, expected, "second restored run");
    assert_eq!(trace2.events, pre_trace.events);

    // Subsequent edit + revert: the content hit keeps every UNCHANGED
    // function's adopted object; the source returns to the exact text.
    let plus2 = edited.find("+ 2").unwrap() + 2;
    let out3_ = r2
        .edit(plus2 as u64, plus2 as u64 + 1, b"3".to_vec())
        .unwrap();
    assert!(out3_.ok);
    let src3 = r2.src().to_owned();
    let plus3 = src3.find("+ 3").unwrap() + 2;
    let out2_ = r2
        .edit(plus3 as u64, plus3 as u64 + 1, b"2".to_vec())
        .unwrap();
    assert!(out2_.ok);
    assert_eq!(r2.src(), &edited, "source is back to the edited text");
    let (_, helper_now, helper_code_now) = r2.resident("helper").unwrap();
    let (_, tail_now, tail_code_now) = r2.resident("tail").unwrap();
    assert!(Arc::ptr_eq(&helper_r2, &helper_now), "helper HIR identity");
    assert!(Arc::ptr_eq(&tail_r2, &tail_now), "tail HIR identity");
    assert!(
        Arc::ptr_eq(&helper_code_r2, &helper_code_now),
        "helper code identity"
    );
    assert!(
        Arc::ptr_eq(&tail_code_r2, &tail_code_now),
        "tail code identity"
    );
    let (final_out, final_trace) = r2.run(&ExecLimits::default(), &cancel);
    assert_i32(&final_out, expected, "post-hit run");
    assert_eq!(final_trace.events, pre_trace.events);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn t03_a07_roundtrip_plain_and_mixed_intrinsic_adoption() {
    t03_a07_roundtrip_exercise(
        "plain",
        "fn helper() int { return 1 }\nfn main() int { return helper() + 1 }\nfn tail() int { return 9 }\n",
        &[],
        3,
        false,
    );
    t03_a07_roundtrip_exercise(
        "mixed",
        "fn helper() int { return mark_a() }\nfn main() int { return helper() + mark_b() + 1 }\nfn tail() int { return 9 }\n",
        &["hir.test.trace.v1".to_string()],
        5,
        true,
    );
}

/// C01-03 body-fact query core: a consumer that READ another function's
/// body fact is invalidated exactly when THAT body changes — an
/// identical-content hit is forced onto the full re-verify chain; a
/// re-minted artifact (which reads no body facts) and UNREAD functions'
/// changes never invalidate. Interface-only callers keep identity and
/// zero stage work throughout (the already-proven half of C01-03).
#[test]
fn t03_body_fact_query_dependency_invalidation() {
    use auto_av_probe::session::warmup;
    // `extra` keeps `tail` off the file END: an edit in the LAST
    // declaration hits the frozen splice's explicit NoConvergence
    // fallback (contracted non-fast-path, not this test's subject).
    let src = "fn helper(x int) int { let r = x * 3; return r + 7 }\nfn main() int { let a = helper(5); return a + 1 }\nfn tail() int { return 9 }\nfn extra() int { return 6 }\n";
    let mut s = warmup(src, "d_main", &[]).expect("warm-up");
    let cancel = CancelFlag::new(false);
    let x3 = src.find("x * 3").unwrap() as u64 + 4;
    let plus1 = src.find("+ 1").unwrap() as u64 + 2;

    // Interface-only axis: helper body edit leaves main at ZERO stage
    // work with the same objects.
    let (_, main_h0, _) = s.resident("main").unwrap();
    let out = s.edit(x3, x3 + 1, b"4".to_vec()).unwrap();
    assert!(out.ok);
    for (stage, count) in s.counters().stages_of("main") {
        assert_eq!(count, 0, "main stage `{stage}` after helper edit");
    }
    let (_, main_h1, _) = s.resident("main").unwrap();
    assert!(Arc::ptr_eq(&main_h0, &main_h1));
    let _ = s.edit(x3, x3 + 1, b"3".to_vec()).unwrap(); // back to x*3

    // Populate main's content history FIRST (a re-mint commit CLEARS the
    // consumer's records, so the recorded read must come after this):
    // main 1→2 (miss) then 2→1 (hit on the original content).
    let out = s.edit(plus1, plus1 + 1, b"2".to_vec()).unwrap();
    assert!(out.ok);
    let out = s.edit(plus1, plus1 + 1, b"1".to_vec()).unwrap();
    assert!(out.ok);
    assert_eq!(s.counters().get("reuse_hit:main"), 1);
    assert_eq!(s.counters().get("verify_hir:main"), 1);

    // Body-fact read: main consumes helper's CURRENT body fact through
    // the production query core.
    let (hdef, _v) = s
        .query_body_fact("main", "helper")
        .expect("helper's body fact");
    assert_eq!(hdef, "d_helper");
    assert_eq!(s.counters().get("body_fact_query:helper"), 1);

    // helper's body changes → main's recorded read is STALE. The next
    // edit lands on content ALREADY in history (2), yet must NOT hit:
    // the stale body-fact read forces the full re-verify chain.
    let out = s.edit(x3, x3 + 1, b"4".to_vec()).unwrap();
    assert!(out.ok);
    let out = s.edit(plus1, plus1 + 1, b"2".to_vec()).unwrap();
    assert!(out.ok, "{:?}", out.diagnostics);
    assert_eq!(s.counters().get("body_fact_invalidated:main"), 1);
    assert_eq!(
        s.counters().get("reuse_hit:main"),
        1,
        "the stale-read content hit was refused"
    );
    assert_eq!(s.counters().get("verify_hir:main"), 2);

    // The re-mint clears main's records: reverting now HITS again (the
    // re-minted artifact's check reads interfaces only).
    let out = s.edit(plus1, plus1 + 1, b"1".to_vec()).unwrap();
    assert!(out.ok);
    assert_eq!(s.counters().get("reuse_hit:main"), 2);

    // helper changing again no longer invalidates the re-minted main.
    let out = s.edit(x3, x3 + 1, b"5".to_vec()).unwrap();
    assert!(out.ok);
    let out = s.edit(plus1, plus1 + 1, b"2".to_vec()).unwrap();
    assert!(out.ok);
    assert_eq!(
        s.counters().get("reuse_hit:main"),
        3,
        "helper's later change does not invalidate the re-minted consumer"
    );
    assert_eq!(s.counters().get("body_fact_invalidated:main"), 1);

    // Unread axis: re-read helper's fact, then edit TAIL — main's read
    // stays current and the content hit survives.
    s.query_body_fact("main", "helper").expect("re-read");
    let t9 = src.find("9").unwrap() as u64;
    let out = s.edit(t9, t9 + 1, b"8".to_vec()).unwrap();
    assert!(
        out.ok,
        "tail edit: {:?} {:?}",
        out.non_fast_path_reason, out.diagnostics
    );
    let out = s.edit(plus1, plus1 + 1, b"1".to_vec()).unwrap();
    assert!(
        out.ok,
        "main revert: {:?} {:?}",
        out.non_fast_path_reason, out.diagnostics
    );
    assert_eq!(
        s.counters().get("reuse_hit:main"),
        4,
        "an unread function's body change never invalidates the consumer"
    );
    assert_eq!(s.counters().get("body_fact_invalidated:main"), 1);

    // End state: helper x*5, main +1, tail 8 — independently expected.
    let (out, _) = s.run(&ExecLimits::default(), &cancel);
    assert_i32(&out, 5 * 5 + 7 + 1, "final value helper(5)=5*5+7, main +1");
}

/// B1-C05: the cold HIR bundle owned by the session (defs, types and
/// every body's locals/places/exprs/blocks) is released BY close(),
/// without drop(session) — `resident_cold_hir_nodes` is the observable
/// oracle of this owner. An externally pinned snapshot keeps running.
#[test]
fn t03_close_releases_cold_bundle_owner() {
    use auto_av_probe::session::warmup;
    let mut s = warmup(SRC, "d_main", &[]).expect("warm-up");
    let cancel = CancelFlag::new(false);
    assert!(
        s.resident_cold_hir_nodes() > 0,
        "the session owns a non-trivial cold HIR payload"
    );
    let pin = s.current();
    s.close();
    assert!(s.closed());
    assert_eq!(
        s.resident_cold_hir_nodes(),
        0,
        "close() releases the owned cold HIR bundle — no drop needed"
    );
    // The externally pinned revision keeps running (V-05); a second
    // close stays released (idempotent terminal state).
    let (out, _) = execute_program(&pin, &ExecLimits::default(), &cancel);
    assert_i32(&out, 23, "pinned snapshot keeps its program");
    s.close();
    assert_eq!(s.resident_cold_hir_nodes(), 0);
}
