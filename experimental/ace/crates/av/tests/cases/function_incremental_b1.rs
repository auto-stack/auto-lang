//! PLAN-828 T-03 — batch-1 review (B1-C01..C05) permanent controls.
//!
//! Collected from the first-batch review's `artifact-controls.rs` and
//! `roundtrip-controls.rs` (same inputs, same oracles) plus the new
//! body-fact query probe and the cold-bundle close-release oracle:
//!
//! - `a07_artifact_admission_controls`: an artifact row that decodes
//!   cleanly but carries a wrong owner / param index / span / forged
//!   intrinsic facts is rejected BEFORE adoption — the full semantic
//!   admission (`verify_function` over the current cold bundle) and the
//!   DECODED body's span digest decide; clean rows and a legal non-zero
//!   owner stay adopted; a rejected function falls back to the cold body
//!   and later content hits never resurrect the corrupted pair.
//! - `a07_roundtrip_plain_and_mixed_intrinsic_adoption`: a mixed
//!   plain+intrinsic caller persists and restores with ALL artifacts
//!   adopted (stable id-text reference tables — B1-C04), second restore,
//!   later edit/hit identity, source/trace no-drift.
//! - `body_fact_query_dependency_invalidation`: a consumer's recorded
//!   body-fact read invalidates exactly when the READ body changes
//!   (C01-03); unread functions and interface-only callers never
//!   invalidate.
//! - `close_releases_cold_bundle_owner`: close() releases the
//!   session-owned cold HIR bundle without drop(session) (B1-C05).

use ace_av_core::av::{execute_program, CancelFlag, ExecLimits};
use ace_av_core::contract::{ExecOutcome, Value};
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
    use ace_av::cache::{
        canonical_hir_function, caps_fingerprint, open_store, put_enveloped, EnvelopeIdentity,
    };
    use ace_av::fn_hir_codec::{
        decode_fn_hir, encode_fn_hir, fn_hir_key, fn_hir_span_digest, normalize_fn_hir_body,
        rebind_fn_hir_body, FnHirArtifact,
    };
    use ace_av::semrefs::content_version_of;
    use ace_av::session::{restore, warmup_persisted};
    use ace_core::hir;
    use ace_core::source::compile_scalar_source;
    use ace_core::verify::{declaration_context_fingerprint, verify_function};

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
fn a07_artifact_admission_controls() {
    // (mode, expected adopted, expected rejecting gate). Since can3 the
    // canonical renders the owner's def id TEXT and the locals' slot
    // indices — owner/param tampering is caught at the CONTENT gate
    // (earlier than, never weaker than, the R1 verify-gate rejection
    // proven by the verifier premise below).
    let cases: &[(&str, u64, Option<&str>)] = &[
        ("clean", 2, None),
        ("nonzero_owner", 2, None),
        ("context", 1, Some("header")),
        ("literal", 1, Some("canonical")),
        ("entry_oob", 1, Some("codec")),
        ("owner", 1, Some("canonical")),
        ("owner_inrange", 1, Some("canonical")),
        ("span", 1, Some("span")),
        ("param_index", 1, Some("canonical")),
        ("intrinsics", 1, Some("intrinsics")),
    ];
    for (mode, adopted, skip) in cases {
        t03_a07_admission_exercise(mode, *adopted, *skip);
    }
}

fn t03_a07_roundtrip_exercise(name: &str, src: &str, caps: &[String], expected: i32, mixed: bool) {
    use ace_av::session::{restore, warmup_persisted};
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
        use ace_av::cache::{
            canonical_hir_function, caps_fingerprint, get_enveloped_opt, open_store,
            EnvelopeIdentity,
        };
        use ace_av::fn_hir_codec::{decode_fn_hir, fn_hir_key};
        use ace_av::semrefs::content_version_of;
        use ace_core::hir::DefKind;
        use ace_core::source::compile_scalar_source;
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
fn a07_roundtrip_plain_and_mixed_intrinsic_adoption() {
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

/// C01-03/R2 body-fact query core: dependencies belong to the PRODUCED
/// ARTIFACT (its history entry), not to a consumer-global slot. A read
/// binds the consumer's CURRENT entry (first-read-wins); a re-read
/// returns the fresh fact value but never refreshes the recorded
/// credential; a content hit checks THE TARGET ENTRY's own snapshot — a
/// read callee body going stale refuses exactly that entry's reuse (full
/// re-verify, then a fresh pair replaces the cached one); sibling
/// artifacts without the read stay reusable; unread functions' changes
/// and interface-only callers are never affected. A closed session
/// refuses the query outright (B1-R2-02).
#[test]
fn body_fact_query_dependency_invalidation() {
    use ace_av::session::warmup;
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

    // Populate main's content history: main 1→2 mints B (no body-fact
    // reads), 2→1 hits the warm entry A. Capture A's pair while A is
    // CURRENT (the artifact whose credential the read below attaches to).
    let out = s.edit(plus1, plus1 + 1, b"2".to_vec()).unwrap();
    assert!(out.ok);
    let out = s.edit(plus1, plus1 + 1, b"1".to_vec()).unwrap();
    assert!(out.ok);
    assert_eq!(s.counters().get("reuse_hit:main"), 1);
    assert_eq!(s.counters().get("verify_hir:main"), 1);
    let (a_hir, a_code) = {
        let (_, h, c) = s.resident("main").unwrap();
        (h, c)
    };

    // Body-fact read: binds main's CURRENT entry (A = "+ 1").
    let (hdef, _v) = s
        .query_body_fact("main", "helper")
        .expect("helper's body fact");
    assert_eq!(hdef, "d_helper");
    assert_eq!(s.counters().get("body_fact_query:helper"), 1);

    // A's read does NOT leak onto B: main 1→2 hits B (its own snapshot
    // has no reads) even though A carries a dependency.
    let out = s.edit(plus1, plus1 + 1, b"2".to_vec()).unwrap();
    assert!(out.ok);
    assert_eq!(s.counters().get("reuse_hit:main"), 2);
    assert_eq!(s.counters().get("body_fact_invalidated:main"), 0);

    // helper's body changes → A's recorded fact goes stale. Returning to
    // A's content must NOT reuse A's cached pair: the hit is refused and
    // the full re-verify chain runs; the fresh pair REPLACES the cached
    // entry (its credential is superseded).
    let out = s.edit(x3, x3 + 1, b"4".to_vec()).unwrap();
    assert!(out.ok);
    let out = s.edit(plus1, plus1 + 1, b"1".to_vec()).unwrap();
    assert!(out.ok, "{:?}", out.diagnostics);
    assert_eq!(s.counters().get("body_fact_invalidated:main"), 1);
    assert_eq!(
        s.counters().get("reuse_hit:main"),
        2,
        "the stale entry's content hit was refused"
    );
    let (_, fresh_hir, fresh_code) = s.resident("main").unwrap();
    assert!(!Arc::ptr_eq(&a_hir, &fresh_hir));
    assert!(
        !Arc::ptr_eq(&a_code, &fresh_code),
        "a re-verified content gets a fresh pair, never the stale cache"
    );

    // The re-verified entry has no reads: alternating back to B and the
    // re-verified content again hits BOTH (artifact-owned snapshots).
    let out = s.edit(plus1, plus1 + 1, b"2".to_vec()).unwrap();
    assert!(out.ok);
    let out = s.edit(plus1, plus1 + 1, b"1".to_vec()).unwrap();
    assert!(out.ok);
    assert_eq!(s.counters().get("reuse_hit:main"), 4);
    assert_eq!(s.counters().get("body_fact_invalidated:main"), 1);

    // Re-read does not refresh: attach helper@v4 to the current entry;
    // helper moves on; a re-query returns the NEW fact while the entry's
    // credential stays at v4 — the next return to this content is
    // refused exactly because the OLD credential is stale.
    s.query_body_fact("main", "helper").expect("attach v4");
    let out = s.edit(x3, x3 + 1, b"5".to_vec()).unwrap();
    assert!(out.ok);
    let _re_read = s.query_body_fact("main", "helper").expect("re-read");
    let out = s.edit(plus1, plus1 + 1, b"2".to_vec()).unwrap();
    assert!(out.ok);
    let out = s.edit(plus1, plus1 + 1, b"1".to_vec()).unwrap();
    assert!(out.ok, "{:?}", out.diagnostics);
    assert_eq!(s.counters().get("body_fact_invalidated:main"), 2);
    assert_eq!(s.counters().get("reuse_hit:main"), 5);

    // Unread axis: after the re-verify, re-attach helper@v5, edit TAIL
    // (never read) — the hits survive.
    s.query_body_fact("main", "helper").expect("attach v5");
    let t9 = src.find("9").unwrap() as u64;
    let out = s.edit(t9, t9 + 1, b"8".to_vec()).unwrap();
    assert!(out.ok);
    let out = s.edit(plus1, plus1 + 1, b"2".to_vec()).unwrap();
    assert!(out.ok);
    let out = s.edit(plus1, plus1 + 1, b"1".to_vec()).unwrap();
    assert!(out.ok);
    assert_eq!(
        s.counters().get("reuse_hit:main"),
        7,
        "an unread function's body change never invalidates the consumer"
    );
    assert_eq!(s.counters().get("body_fact_invalidated:main"), 2);

    // End state: helper x*5, main +1, tail 8 — independently expected.
    let (out, _) = s.run(&ExecLimits::default(), &cancel);
    assert_i32(&out, 5 * 5 + 7 + 1, "final value helper(5)=5*5+7, main +1");

    // B1-R2-02: a closed session refuses the query — no result, no new
    // dependency work, no counter movement.
    s.close();
    let queries = s.counters().get("body_fact_query:helper");
    assert!(s.query_body_fact("main", "helper").is_none());
    assert_eq!(
        s.counters().get("body_fact_query:helper"),
        queries,
        "a closed query core accepts no new work"
    );
    let (out, _) = s.run(&ExecLimits::default(), &cancel);
    assert!(
        matches!(out, ExecOutcome::Infra(_)),
        "closed run is refused"
    );
}

/// B1-C05: the cold HIR bundle owned by the session (defs, types and
/// every body's locals/places/exprs/blocks) is released BY close(),
/// without drop(session) — `resident_cold_hir_nodes` is the observable
/// oracle of this owner. An externally pinned snapshot keeps running.
#[test]
fn close_releases_cold_bundle_owner() {
    use ace_av::session::warmup;
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

/// B1-R2-01 P1 (collected from the R2 review's adjacent-controls.rs, same
/// inputs/oracles): changing an assignment's WRITE TARGET (`x = 3` →
/// `y = 3`) is a legal edit that changes the program's semantics — the
/// content identity must distinguish it (the pre-R2 canonical omitted
/// `Place.local`, so the edit hit the old code and served the WRONG
/// result 3 while the same source cold-compiles to 1). The value-control
/// sibling (3→4) proves the input reaches the real path.
#[test]
fn r2_assignment_target_change_is_new_content_not_old_code() {
    use ace_av::session::warmup;
    let src = "fn helper() int { var x = 1; var y = 2; x = 3; return x }\nfn main() int { return helper() }\nfn tail() int { return 9 }\n";
    let mut s = warmup(src, "d_main", &[]).expect("warm-up");
    let cancel = CancelFlag::new(false);
    let run = |s: &ace_av::session::FunctionIncrementalSession| match s
        .run(&ExecLimits::default(), &CancelFlag::new(false))
        .0
    {
        ExecOutcome::Success {
            value: Value::I32(v),
            ..
        } => v,
        other => panic!("{other:?}"),
    };
    assert_eq!(run(&s), 3);
    let (_, old_hir, old_code) = s.resident("helper").unwrap();
    let at = src.find("x = 3").unwrap() as u64;
    let out = s.edit(at, at + 1, b"y".to_vec()).unwrap();
    assert!(out.ok, "{out:?}");
    let cold = warmup(s.src(), "d_main", &[]).expect("cold compile");
    assert_eq!(run(&cold), 1, "cold expectation");
    assert_eq!(run(&s), 1, "the incremental result equals the cold compile");
    assert_eq!(
        s.counters().get("reuse_hit:helper"),
        0,
        "a different write target is different content — no hit"
    );
    let (_, h, code) = s.resident("helper").unwrap();
    assert!(!Arc::ptr_eq(&old_hir, &h));
    assert!(!Arc::ptr_eq(&old_code, &code));
    let _ = cancel;
}

/// Value sibling of the write-target control: editing the assigned VALUE
/// is genuinely new content, no hit, cold-equal result.
#[test]
fn r2_assignment_value_change_positive() {
    use ace_av::session::warmup;
    let src = "fn helper() int { var x = 1; var y = 2; x = 3; return x }\nfn main() int { return helper() }\nfn tail() int { return 9 }\n";
    let mut s = warmup(src, "d_main", &[]).expect("warm-up");
    let run = |s: &ace_av::session::FunctionIncrementalSession| match s
        .run(&ExecLimits::default(), &CancelFlag::new(false))
        .0
    {
        ExecOutcome::Success {
            value: Value::I32(v),
            ..
        } => v,
        other => panic!("{other:?}"),
    };
    let at = src.find("x = 3").unwrap() as u64 + 4;
    assert!(s.edit(at, at + 1, b"4".to_vec()).unwrap().ok);
    assert_eq!(run(&s), 4);
    assert_eq!(run(&warmup(s.src(), "d_main", &[]).unwrap()), 4);
    assert_eq!(s.counters().get("reuse_hit:helper"), 0);
}

/// B1-C03/R2 (collected from the R2 review's adjacent-controls.rs): a
/// re-read between the dependency going stale and the consumer's edit
/// must NOT renew the OLD artifact's recorded dependency — the same
/// content edit still re-verifies.
#[test]
fn r2_reread_does_not_refresh_old_dependency() {
    use ace_av::session::warmup;
    let src = "fn helper() int { return 1 }\nfn main() int { return helper() + 1 }\nfn tail() int { return 9 }\n";
    let mut s = warmup(src, "d_main", &[]).expect("warm-up");
    let at = src.find("return 1").unwrap() as u64 + 7;
    let plus = src.find("+ 1").unwrap() as u64 + 2;
    let first = s.query_body_fact("main", "helper").unwrap();
    assert!(s.edit(at, at + 1, b"2".to_vec()).unwrap().ok);
    let second = s.query_body_fact("main", "helper").unwrap();
    assert_ne!(first.1, second.1);
    assert!(s.edit(plus, plus + 1, b"1".to_vec()).unwrap().ok);
    assert_eq!(
        s.counters().get("body_fact_invalidated:main"),
        1,
        "a reread before rebuilding cannot renew an old artifact's recorded dependency"
    );
}

/// B1-C03/R2 (collected from the R2 review's adjacent-controls.rs):
/// reminting B cannot erase the dependencies of the still-cached A —
/// returning to A's content re-verifies instead of reusing A's pair.
#[test]
fn r2_history_retains_its_own_body_dependencies() {
    use ace_av::session::warmup;
    let src = "fn helper() int { return 1 }\nfn main() int { return helper() + 1 }\nfn tail() int { return 9 }\n";
    let mut s = warmup(src, "d_main", &[]).expect("warm-up");
    let h = src.find("return 1").unwrap() as u64 + 7;
    let m = src.find("+ 1").unwrap() as u64 + 2;
    let (_, old_hir, old_code) = s.resident("main").unwrap();
    let _original_read = s.query_body_fact("main", "helper").unwrap();
    assert!(s.edit(m, m + 1, b"2".to_vec()).unwrap().ok); // B: no body-fact reads.
    assert!(s.edit(h, h + 1, b"2".to_vec()).unwrap().ok); // A's read goes stale.
    assert!(s.edit(m, m + 1, b"1".to_vec()).unwrap().ok); // Back to A, not B.
    let (_, new_hir, new_code) = s.resident("main").unwrap();
    assert_eq!(s.counters().get("body_fact_invalidated:main"), 1);
    assert!(!Arc::ptr_eq(&old_hir, &new_hir));
    assert!(
        !Arc::ptr_eq(&old_code, &new_code),
        "reminting B cannot erase dependencies of the still-cached historical A"
    );
}

/// B1-R2-01 restore-side sibling (collected from the R2 review's
/// semantic-artifact-controls.rs, premises updated to the FIXED
/// behavior): a legally-verifiable HIR that writes a DIFFERENT target
/// (`Place.local` x→y) or enters at another block is NOT the current
/// source's content — the can3 canonical distinguishes it and the
/// artifact is skipped at the canonical gate; the cold body serves.
#[test]
fn r2_semantic_wrong_target_or_entry_not_adopted() {
    use ace_av::cache::{caps_fingerprint, open_store, put_enveloped, EnvelopeIdentity};
    use ace_av::fn_hir_codec::{
        encode_fn_hir, fn_hir_key, fn_hir_span_digest, normalize_fn_hir_body, FnHirArtifact,
    };
    use ace_av::semrefs::content_version_of;
    use ace_av::session::{restore, warmup_persisted};
    use ace_core::hir;
    use ace_core::source::compile_scalar_source;
    use ace_core::verify::{declaration_context_fingerprint, verify_function};

    fn exercise(mode: &str) {
        let src: &str = if mode == "entry" {
            "fn helper(x int) int { if x > 0 { return 11 } else { return 22 } }\nfn main() int { return helper(0) }\nfn tail() int { return 9 }\n"
        } else {
            "fn helper() int { var x = 1; var y = 2; x = 3; return x }\nfn main() int { return helper() }\nfn tail() int { return 9 }\n"
        };
        let c = compile_scalar_source(src, "probe.at", &[]).unwrap();
        let bundle = c.checked.bundle();
        let m = &bundle.modules[0];
        let di = m.defs.iter().position(|d| d.id_text == "d_helper").unwrap();
        let bid = match m.defs[di].kind {
            hir::DefKind::Function { body, .. } => body,
            _ => panic!(),
        };
        let original = &m.bodies[bid.0 as usize];
        let mut changed = original.clone();
        if mode == "entry" {
            let entry = original
                .blocks
                .iter()
                .position(|b| {
                    b.stmts.iter().any(|st| match st {
                        hir::Stmt::Return { value } => matches!(
                            original.exprs[value.0 as usize].kind,
                            hir::ExprKind::Constant { value: 11 }
                        ),
                        _ => false,
                    })
                })
                .unwrap();
            changed.entry = hir::BlockId(entry as u32);
            assert_ne!(changed.entry, original.entry);
        } else {
            let y = original.locals.iter().position(|l| l.name == "y").unwrap();
            assert!(!original.places.is_empty());
            changed.places[0].local = hir::LocalId(y as u32);
        }
        let mut altered_module = m.clone();
        altered_module.bodies[bid.0 as usize] = changed.clone();
        // FIXED premise: the can3 canonical DOES distinguish the altered
        // body (it renders Place.local and the entry block).
        assert_ne!(
            ace_av::cache::canonical_hir_function(m, bid),
            ace_av::cache::canonical_hir_function(&altered_module, bid),
            "can3 must distinguish a different write target / entry block"
        );
        let mut v = bundle.clone();
        v.modules[0] = altered_module;
        let verifier = verify_function(v, di);
        assert_eq!(
            verifier.is_ok(),
            mode != "entry",
            "entry mutation is a verifier-rejected negative; place mutation stays legal HIR"
        );
        let (body, callee_table, type_table) = normalize_fn_hir_body(
            &m.defs.iter().map(|d| d.id_text.clone()).collect::<Vec<_>>(),
            &m.types
                .iter()
                .map(|t| t.id_text.clone())
                .collect::<Vec<_>>(),
            &changed,
        )
        .unwrap();
        let cv = content_version_of(&ace_av::cache::canonical_hir_function(m, bid));
        let caps = caps_fingerprint(&[]);
        let a = FnHirArtifact {
            def_id: "d_helper".into(),
            content_version: cv.clone(),
            span_digest: fn_hir_span_digest(original),
            context_fingerprint: declaration_context_fingerprint(m),
            caps_fp: caps.clone(),
            intrinsics: vec![],
            callee_table,
            type_table,
            body,
        };
        let dir = std::env::temp_dir().join(format!("p828-r2-sem-{mode}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        {
            let st = open_store(&dir).unwrap();
            put_enveloped(
                &st,
                &fn_hir_key(&m.id_text, "d_helper", &cv, &caps),
                "A07",
                &EnvelopeIdentity::fn_hir_bound(&m.id_text, "d_helper", &cv),
                &encode_fn_hir(&a).unwrap(),
                vec![],
                Some("p828-r2"),
            )
            .unwrap();
        }
        drop(warmup_persisted(src, "d_main", &[], &dir, "p828-r2").unwrap());
        let s = restore(src, "d_main", &[], &dir, "p828-r2").unwrap();
        let (_, body, _) = s.resident("helper").unwrap();
        let execution = s.run(&ExecLimits::default(), &CancelFlag::new(false)).0;
        let expected = if mode == "entry" { 22 } else { 3 };
        assert!(
            matches!(execution, ExecOutcome::Success { value: Value::I32(v), .. } if v == expected)
        );
        assert_eq!(
            s.counters().get("restore_hir_skip:canonical"),
            1,
            "the semantically-different body is rejected at the content gate"
        );
        assert_eq!(
            s.counters().get("restore_hir_adopted"),
            2,
            "the wrong-content artifact is skipped; the cold body serves"
        );
        if mode == "entry" {
            assert_eq!(body.entry, original.entry);
        } else {
            assert_eq!(body.places[0].local, original.places[0].local);
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
    exercise("place");
    exercise("entry");
}

/// B1-R3-01 (collected from the R3 review's canonical-controls.rs, same
/// input/mutation/oracle): flipping a call binding's `.move` marker is a
/// SEMANTIC change (hir PartialEq) that the verifier ACCEPTS as legal
/// HIR — the can4 content identity must distinguish it before adoption
/// (the pre-R3 dump omitted `CallBinding.moved`, so the wrong-flag HIR
/// was adopted as the current source's content).
#[test]
fn r3_call_binding_moved_distinguished_at_restore() {
    use ace_av::cache::{caps_fingerprint, open_store, put_enveloped, EnvelopeIdentity};
    use ace_av::fn_hir_codec::{
        encode_fn_hir, fn_hir_key, fn_hir_span_digest, normalize_fn_hir_body, FnHirArtifact,
    };
    use ace_av::semrefs::content_version_of;
    use ace_av::session::{restore, warmup_persisted};
    use ace_core::hir;
    use ace_core::source::compile_scalar_source;
    use ace_core::verify::declaration_context_fingerprint;

    let src = "fn helper(x int) int { return x + 1 }\nfn main() int { return helper(5) }\nfn tail() int { return 9 }\n";
    let c = compile_scalar_source(src, "probe.at", &[]).unwrap();
    let bundle = c.checked.bundle();
    let m = &bundle.modules[0];
    let di = m.defs.iter().position(|d| d.id_text == "d_main").unwrap();
    let bid = match m.defs[di].kind {
        hir::DefKind::Function { body, .. } => body,
        _ => panic!(),
    };
    let original = &m.bodies[bid.0 as usize];
    let mut changed = original.clone();
    let call = changed
        .exprs
        .iter_mut()
        .find_map(|e| match &mut e.kind {
            hir::ExprKind::Call { bindings, .. } => Some(bindings),
            _ => None,
        })
        .unwrap();
    assert!(!call[0].moved);
    call[0].moved = true;
    assert_ne!(original, &changed, "moved participates in hir PartialEq");
    let mut v = bundle.clone();
    v.modules[0].bodies[bid.0 as usize] = changed.clone();
    // FIXED premise: can4 distinguishes the semantic marker.
    assert_ne!(
        ace_av::cache::canonical_hir_function(m, bid),
        ace_av::cache::canonical_hir_function(&v.modules[0], bid),
        "the .move marker must enter content identity"
    );
    let (body, callee_table, type_table) = normalize_fn_hir_body(
        &m.defs.iter().map(|d| d.id_text.clone()).collect::<Vec<_>>(),
        &m.types
            .iter()
            .map(|t| t.id_text.clone())
            .collect::<Vec<_>>(),
        &changed,
    )
    .unwrap();
    let cv = content_version_of(&ace_av::cache::canonical_hir_function(m, bid));
    let caps = caps_fingerprint(&[]);
    let a = FnHirArtifact {
        def_id: "d_main".into(),
        content_version: cv.clone(),
        span_digest: fn_hir_span_digest(original),
        context_fingerprint: declaration_context_fingerprint(m),
        caps_fp: caps.clone(),
        intrinsics: vec![],
        callee_table,
        type_table,
        body,
    };
    let dir = std::env::temp_dir().join(format!("p828-r3-moved-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    {
        let st = open_store(&dir).unwrap();
        put_enveloped(
            &st,
            &fn_hir_key(&m.id_text, "d_main", &cv, &caps),
            "A07",
            &EnvelopeIdentity::fn_hir_bound(&m.id_text, "d_main", &cv),
            &encode_fn_hir(&a).unwrap(),
            vec![],
            Some("p828-r3"),
        )
        .unwrap();
    }
    drop(warmup_persisted(src, "d_main", &[], &dir, "p828-r3").unwrap());
    let s = restore(src, "d_main", &[], &dir, "p828-r3").unwrap();
    let (_, restored, _) = s.resident("main").unwrap();
    let moved = restored
        .exprs
        .iter()
        .find_map(|e| match &e.kind {
            hir::ExprKind::Call { bindings, .. } => Some(bindings[0].moved),
            _ => None,
        })
        .unwrap();
    assert_eq!(
        s.counters().get("restore_hir_skip:canonical"),
        1,
        "the wrong-move artifact is rejected at the content gate"
    );
    assert_eq!(
        s.counters().get("restore_hir_adopted"),
        2,
        "the clean rows adopt; the wrong-flag row is skipped"
    );
    assert!(!moved, "resident main keeps the cold source's moved=false");
    let _ = std::fs::remove_dir_all(&dir);
}

/// B1-C03 sibling control (collected from the R3 review's
/// canonical-controls.rs): a CANCELLED candidate and a LATE (stale-base)
/// commit never refresh or bypass the recorded dependency — the history
/// entry's own snapshot still forces the re-verify on the next return.
#[test]
fn r3_dependencies_cancel_and_late_do_not_refresh_history() {
    use ace_av::session::warmup;
    let src = "fn helper() int { return 1 }\nfn main() int { return helper() + 1 }\nfn tail() int { return 9 }\n";
    let mut s = warmup(src, "d_main", &[]).unwrap();
    let hp = src.find("return 1").unwrap() as u64 + 7;
    let mp = src.find("+ 1").unwrap() as u64 + 2;
    s.query_body_fact("main", "helper").unwrap();
    let prepared = s
        .prepare_edit(mp, mp + 1, b"2".to_vec())
        .unwrap()
        .ok()
        .unwrap();
    let cancelled = s.cancel_prepared(prepared);
    assert!(!cancelled.ok);
    assert!(s.edit(hp, hp + 1, b"2".to_vec()).unwrap().ok);
    assert!(s.edit(mp, mp + 1, b"1".to_vec()).unwrap().ok);
    assert_eq!(s.counters().get("body_fact_invalidated:main"), 1);
    s.query_body_fact("main", "helper").unwrap();
    let late = s
        .prepare_edit(mp, mp + 1, b"2".to_vec())
        .unwrap()
        .ok()
        .unwrap();
    assert!(s.edit(hp, hp + 1, b"3".to_vec()).unwrap().ok);
    assert!(!s.commit(late).ok, "the stale-base candidate is refused");
    assert!(s.edit(mp, mp + 1, b"1".to_vec()).unwrap().ok);
    assert_eq!(s.counters().get("body_fact_invalidated:main"), 2);
}
