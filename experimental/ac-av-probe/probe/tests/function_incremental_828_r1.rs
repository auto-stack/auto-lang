//! PLAN-828 review-r1 counterexample regressions (F-03/05/06/07/08/09).
//! Each test mirrors the reviewer's independent replay case verbatim in
//! inputs and oracle; they are the permanent closure cases for their
//! findings. The F-04 replay (unchanged-code span mapping) lands with its
//! fix.

use auto_av_core::av::{CancelFlag, ExecLimits};
use auto_av_core::contract::{ExecOutcome, Value};
use auto_av_probe::session::{restore, warmup, warmup_persisted};
use std::sync::Arc;

fn value(s: &auto_av_probe::session::FunctionIncrementalSession) -> i32 {
    let (out, _) = s.run(&ExecLimits::default(), &CancelFlag::new(false));
    match out {
        ExecOutcome::Success {
            value: Value::I32(v),
            ..
        } => v,
        other => panic!("expected i32 success, got {other:?}"),
    }
}

/// F-828-R1-03: a legal body edit of a NON-FIRST RECURSIVE function must
/// install; the unit's self-reference rides the import convention.
#[test]
fn r1_recursion_in_nonfirst_function() {
    let src = "fn main() int { return recur(3) }\nfn recur(n int) int { if n > 0 { return recur(n - 1) + 1 } else { return 0 } }\nfn tail() int { return 9 }\n";
    let mut s = warmup(src, "d_main", &[]).unwrap();
    assert_eq!(value(&s), 3, "cold accepted control");
    let edited = src.replacen("+ 1", "+ 2", 1);
    let cold = warmup(&edited, "d_main", &[]).unwrap();
    assert_eq!(value(&cold), 6, "cold edited-source oracle");
    let at = src.find("+ 1").unwrap() + 2;
    let out = s.edit(at as u64, (at + 1) as u64, b"2".to_vec()).unwrap();
    assert!(out.ok, "{out:?}");
    let cold = warmup(s.src(), "d_main", &[]).unwrap();
    assert_eq!(value(&cold), 6, "cold oracle");
    assert_eq!(value(&s), 6, "incremental recursion must agree");
}

/// F-828-R1-05: a prepared candidate is bound to its minting session — a
/// cross-session commit is refused even at equal revision numbers, and the
/// target session's source and result stay consistent.
#[test]
fn r1_prepared_candidate_cannot_cross_sessions() {
    let a = "fn helper() int { return 1 }\nfn main() int { return helper() + 10 }\n";
    let b = "fn helper() int { return 2 }\nfn main() int { return helper() + 100 }\n";
    let mut sa = warmup(a, "d_main", &[]).unwrap();
    let mut sb = warmup(b, "d_main", &[]).unwrap();
    assert_eq!(value(&sa), 11);
    assert_eq!(value(&sb), 102);
    let at = a.find("return 1").unwrap() + 7;
    let candidate = sa
        .prepare_edit(at as u64, (at + 1) as u64, b"3".to_vec())
        .unwrap()
        .unwrap();
    let out = sb.commit(candidate);
    assert!(!out.ok, "a different session's candidate must be rejected");
    assert_eq!(value(&sb), 102);
    // The minting session can still commit a fresh candidate of its own.
    let own = sa
        .prepare_edit(at as u64, at as u64, Vec::new())
        .unwrap()
        .unwrap();
    assert!(sa.commit(own).ok, "own-session candidate commits");
}

/// F-828-R1-06: credential currency is decided by the ENVIRONMENT from its
/// own bundle — a caller-supplied fingerprint cannot substitute for it.
#[test]
fn r1_credential_must_bind_actual_environment() {
    use auto_ac_prototype::source::compile_scalar_source;
    use auto_ac_prototype::verify::verify_function;
    use auto_av_probe::function_queries::FnEnv;
    let a = compile_scalar_source(
        "fn helper(x int) int { return x }\nfn main() int { return helper(1) }",
        "a.at",
        &[],
    )
    .unwrap();
    let b = compile_scalar_source(
        "fn helper(x int, y int) int { return x + y }\nfn main() int { return helper(1, 2) }",
        "b.at",
        &[],
    )
    .unwrap();
    let cf = verify_function(a.checked.bundle().clone(), 0).unwrap();
    let ea = FnEnv::from_bundle(a.checked.bundle()).unwrap();
    let eb = FnEnv::from_bundle(b.checked.bundle()).unwrap();
    assert!(ea.credential_current(&cf));
    assert!(
        !eb.credential_current(&cf),
        "the credential of env A is not current in env B — the ENVIRONMENT decides"
    );
}

/// F-828-R1-07: after a warm restore, the first edit keeps every UNCHANGED
/// function's restored code object (snapshot identity).
#[test]
fn r1_restored_unchanged_code_identity_survives_edit() {
    let src = "fn helper() int { return 1 }\nfn main() int { return helper() + 10 }\n";
    let dir = std::env::temp_dir().join(format!("plan828-review-restore-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let _seed = warmup_persisted(src, "d_main", &[], &dir, "review-828").unwrap();
    drop(_seed);
    let mut s = restore(src, "d_main", &[], &dir, "review-828").unwrap();
    assert_eq!(s.counters().get("restore_units_admitted"), 2);
    assert_eq!(value(&s), 11);
    let old = s.current().find_code("d_main").unwrap().clone();
    let at = src.find("return 1").unwrap() + 7;
    let out = s.edit(at as u64, (at + 1) as u64, b"2".to_vec()).unwrap();
    assert!(out.ok, "{out:?}");
    assert_eq!(value(&s), 12);
    let current = s.current();
    assert!(
        Arc::ptr_eq(&old, current.find_code("d_main").unwrap()),
        "restore must synchronize resident records with restored dispatch code"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// F-828-R1-08: the trace-intrinsic support surface survives the resident
/// path (warm-up with the trace capability).
#[test]
fn r1_trace_intrinsic_support_is_preserved() {
    let src = "fn helper() int { return mark_a() }\nfn main() int { return helper() }\n";
    let caps = vec!["hir.test.trace.v1".to_string()];
    auto_ac_prototype::source::compile_scalar_source(src, "probe.at", &caps)
        .expect("old source support control");
    let s = warmup(src, "d_main", &caps).expect("resident path must preserve trace support");
    assert_eq!(value(&s), 1);
}

/// F-828-R1-09: the session runs the SAME frozen input front door — a
/// source beyond MAX_INPUT_BYTES is refused before any allocation.
#[test]
fn r1_session_preserves_frozen_input_cap() {
    let mut src = String::from("fn main() int { return 1 }\n//");
    src.push_str(&"x".repeat(auto_av_core::contract::MAX_INPUT_BYTES));
    assert!(src.len() > auto_av_core::contract::MAX_INPUT_BYTES);
    let out = warmup(&src, "d_main", &[]);
    assert!(
        out.is_err(),
        "resident warmup accepted {} bytes beyond the inherited {} byte cap",
        src.len(),
        auto_av_core::contract::MAX_INPUT_BYTES
    );
}

/// F-828-R1-04 / R2: an edit in an EARLIER function shifts a LATER
/// unchanged function's trap span — mapped into current-source
/// coordinates without touching the code payload.
#[test]
fn r2_unchanged_trap_location_after_prefix_edit() {
    let src = "fn helper() int { return 1 }
fn main() int { var x = helper(); return 1 / 0 }
fn tail() int { return 9 }
";
    let mut s = warmup(src, "d_main", &[]).unwrap();
    let trap = |s: &auto_av_probe::session::FunctionIncrementalSession| {
        let (out, _) = s.run(&ExecLimits::default(), &CancelFlag::new(false));
        match out {
            ExecOutcome::Trap { span: Some(sp), .. } => sp,
            other => panic!("expected trap, got {other:?}"),
        }
    };
    let before = trap(&s);
    let at = src.find("return 1").unwrap();
    let insert = "var pad = 2;
";
    let out = s
        .edit(at as u64, at as u64, insert.as_bytes().to_vec())
        .unwrap();
    assert!(out.ok, "{out:?}");
    let cold = warmup(s.src(), "d_main", &[]).unwrap();
    let expected = trap(&cold);
    assert_eq!(expected.start, before.start + insert.len() as u32);
    assert_eq!(
        trap(&s),
        expected,
        "unchanged code needs snapshot-specific source mapping"
    );
}

/// F-828-R1-06 / R2: a Scalar credential is never current in a Core
/// environment even when the module declarations read identically.
#[test]
fn r2_credential_cannot_cross_hir_domains() {
    use auto_ac_prototype::source::{compile_scalar_source, compile_source};
    use auto_ac_prototype::verify::verify_function;
    use auto_av_probe::function_queries::FnEnv;
    let src = "fn helper() int { return 1 }
fn main() int { return helper() }";
    let scalar = compile_scalar_source(src, "same.at", &[]).unwrap();
    let core = compile_source(src, "same.at", &[]).unwrap();
    let cf = verify_function(scalar.checked.bundle().clone(), 0).unwrap();
    let env_scalar = FnEnv::from_bundle(scalar.checked.bundle()).unwrap();
    let env_core = FnEnv::from_bundle(core.checked.bundle()).unwrap();
    assert_ne!(env_scalar.domain(), env_core.domain());
    assert!(env_scalar.credential_current(&cf), "same-domain control");
    assert!(
        !env_core.credential_current(&cf),
        "a Scalar credential must not flow into a Core environment"
    );
}

/// F-828-R1-08 / R2: a trace intrinsic in an UNCHANGED callee survives a
/// legal body edit of its CALLER; without the capability the session
/// still refuses (authorization negative control).
#[test]
fn r2_trace_in_unchanged_function_survives_body_edit() {
    let src = "fn helper() int { return mark_a() }
fn main() int { return helper() + 1 }
fn tail() int { return 9 }
";
    let caps = vec!["hir.test.trace.v1".to_string()];
    // Authorization negative control: without the capability, warm-up
    // refuses (the compile front door rejects the unresolved mark).
    assert!(warmup(src, "d_main", &[]).is_err());
    let mut s = warmup(src, "d_main", &caps).expect("trace-capable warm-up");
    assert_eq!(value(&s), 2, "mark_a() = 1, +1");
    let at = src.find("+ 1").unwrap();
    let out = s
        .edit(at as u64, (at + 3) as u64, b"+ 2".to_vec())
        .expect("edit applies");
    assert!(
        out.ok,
        "editing the caller of an intrinsic-using unchanged fn must install: {out:?}"
    );
    assert_eq!(value(&s), 3, "mark_a() = 1, +2");
}

const R2A_SRC: &str = "fn helper() int { return 1 }
fn main() int { var x = helper(); return 1 / 0 }
fn tail() int { return 9 }
";

fn trap_span(
    s: &auto_av_probe::session::FunctionIncrementalSession,
) -> auto_av_core::contract::SpanInfo {
    let (out, _) = s.run(&ExecLimits::default(), &CancelFlag::new(false));
    match out {
        ExecOutcome::Trap { span: Some(sp), .. } => sp,
        other => panic!("expected trap, got {other:?}"),
    }
}

/// F-828-R1-04/R2a F04-a: the PUBLIC snapshot execution path maps spans
/// through the snapshot's own immutable shifts.
#[test]
fn r2a_current_snapshot_public_execution_uses_its_source_revision() {
    let mut s = warmup(R2A_SRC, "d_main", &[]).unwrap();
    let at = R2A_SRC.find("return 1").unwrap();
    let insert = "var pad = 2;
";
    let out = s
        .edit(at as u64, at as u64, insert.as_bytes().to_vec())
        .unwrap();
    assert!(out.ok, "{out:?}");
    let cold = warmup(s.src(), "d_main", &[]).unwrap();
    let expected = trap_span(&cold);
    assert_eq!(expected.start, 83);
    let snap = s.current();
    let (pub_out, _) =
        auto_av_core::av::execute_program(&snap, &ExecLimits::default(), &CancelFlag::new(false));
    match pub_out {
        ExecOutcome::Trap { span: Some(sp), .. } => assert_eq!(
            (sp.start, sp.end),
            (expected.start, expected.end),
            "public snapshot path"
        ),
        other => panic!("expected trap, got {other:?}"),
    }
}

/// F-828-R1-04/R2a F04-b: content hits keep the ORIGINAL mint baseline.
#[test]
fn r2a_history_hit_preserves_mint_source_for_span_mapping() {
    let mut s = warmup(R2A_SRC, "d_main", &[]).unwrap();
    let at = R2A_SRC.find("return 1").unwrap();
    let out = s
        .edit(
            at as u64,
            at as u64,
            b"var pad = 2;
"
            .to_vec(),
        )
        .unwrap();
    assert!(out.ok, "{out:?}");
    // Content churn in the TRAPPING function: 1/0 -> 2/0 -> 1/0.
    let div = R2A_SRC.find("1 / 0").unwrap() + 13 + 0;
    let o1 = s.edit(div as u64, (div + 1) as u64, b"2".to_vec()).unwrap();
    assert!(o1.ok, "{o1:?}");
    let o2 = s.edit(div as u64, (div + 1) as u64, b"1".to_vec()).unwrap();
    assert!(o2.ok, "{o2:?}");
    assert!(
        s.counters().stage_of("reuse_hit", "main") >= 1,
        "real content hit"
    );
    let cold = warmup(s.src(), "d_main", &[]).unwrap();
    assert_eq!(trap_span(&s), trap_span(&cold), "hit keeps mint mapping");
}

/// F-828-R1-04/R2a F04-c: a semantic no-op newline insert INSIDE the
/// trapping function is a content hit whose body map still updates.
#[test]
fn r2a_semantic_noop_edit_inside_trapping_function_updates_source_map() {
    let mut s = warmup(R2A_SRC, "d_main", &[]).unwrap();
    let at = R2A_SRC.find("return 1 / 0").unwrap();
    let out = s
        .edit(
            at as u64,
            at as u64,
            b"
"
            .to_vec(),
        )
        .unwrap();
    assert!(out.ok, "{out:?}");
    assert!(
        s.counters().stage_of("reuse_hit", "main") >= 1,
        "newline-only edit is a content hit"
    );
    let cold = warmup(s.src(), "d_main", &[]).unwrap();
    assert_eq!(
        trap_span(&s),
        trap_span(&cold),
        "body map follows the insert"
    );
}

/// F-828-R1-06/R2a: a credential whose body READS an intrinsic binds that
/// binding; changing the referenced intrinsic invalidates it (unread
/// intrinsics stay excluded from global identity).
#[test]
fn r2a_credential_invalidates_when_referenced_intrinsic_changes() {
    use auto_ac_prototype::source::compile_scalar_source;
    use auto_ac_prototype::verify::verify_function;
    use auto_av_probe::function_queries::FnEnv;
    let caps = vec!["hir.test.trace.v1".to_string()];
    let a = compile_scalar_source(
        "fn helper() int { return mark_a() }
fn main() int { return helper() }",
        "t.at",
        &caps,
    )
    .unwrap();
    let b = compile_scalar_source(
        "fn helper() int { return mark_b() }
fn main() int { return helper() }",
        "t.at",
        &caps,
    )
    .unwrap();
    let idx = a.checked.bundle().modules[0]
        .defs
        .iter()
        .position(|d| d.id_text == "d_helper")
        .unwrap();
    let cf = verify_function(a.checked.bundle().clone(), idx).unwrap();
    let env_a = FnEnv::from_bundle(a.checked.bundle()).unwrap();
    let env_b = FnEnv::from_bundle(b.checked.bundle()).unwrap();
    assert!(env_a.credential_current(&cf), "same-binding control");
    assert!(
        !env_b.credential_current(&cf),
        "the read intrinsic binding changed - the credential must invalidate"
    );
}

/// F-828-R1-04/R3a a: body-local refinement available on the PUBLIC
/// snapshot path (insert inside main before the trap).
#[test]
fn r3a_public_snapshot_preserves_body_local_source_refinement() {
    let mut s = warmup(R2A_SRC, "d_main", &[]).unwrap();
    let at = R2A_SRC.find("helper();").unwrap() + "helper();".len(); // inside main, before the div
    let out = s
        .edit(at as u64, at as u64, b"var q = 1; ".to_vec())
        .unwrap();
    assert!(out.ok, "{out:?}");
    let cold = warmup(s.src(), "d_main", &[]).unwrap();
    let expected = trap_span(&cold);
    let snap = s.current();
    let (o, _) =
        auto_av_core::av::execute_program(&snap, &ExecLimits::default(), &CancelFlag::new(false));
    match o {
        ExecOutcome::Trap { span: Some(sp), .. } => {
            assert_eq!(
                (sp.start, sp.end),
                (expected.start, expected.end),
                "public path body-local map"
            )
        }
        other => panic!("expected trap, got {other:?}"),
    }
}

/// F-828-R1-04/R3a b: successive body-local edits COMPOSE.
#[test]
fn r3a_successive_body_local_edits_compose_source_mapping() {
    let mut s = warmup(R2A_SRC, "d_main", &[]).unwrap();
    let at = R2A_SRC.find("helper();").unwrap() + "helper();".len();
    let o1 = s
        .edit(at as u64, at as u64, b"var q = 1; ".to_vec())
        .unwrap();
    assert!(o1.ok);
    let o2 = s
        .edit(at as u64, at as u64, b"var w = 2; ".to_vec())
        .unwrap();
    assert!(o2.ok);
    let cold = warmup(s.src(), "d_main", &[]).unwrap();
    assert_eq!(trap_span(&s), trap_span(&cold), "composed body-local maps");
}

/// F-828-R1-04/R3a c: an edit INSIDE the trap range moves the END only.
#[test]
fn r3a_whitespace_inside_trap_range_updates_end_independently() {
    let mut s = warmup(R2A_SRC, "d_main", &[]).unwrap();
    let before = trap_span(&s);
    let mid = before.start as usize + 2; // inside "1 / 0"
    let out = s.edit(mid as u64, mid as u64, b" ".to_vec()).unwrap();
    assert!(out.ok, "{out:?}");
    let cold = warmup(s.src(), "d_main", &[]).unwrap();
    let expected = trap_span(&cold);
    assert_eq!(
        expected.start, before.start,
        "start before the insert point"
    );
    assert_eq!(expected.end, before.end + 1, "end after it shifts");
    assert_eq!(trap_span(&s), expected);
}

/// F-828-R1-06/R3a d: per-REFERENCE identity — swapping two intrinsic
/// symbols between defs keeps the symbol SET intact but must invalidate
/// the credential.
#[test]
fn r3a_credential_tracks_each_intrinsic_reference_not_just_symbol_set() {
    use auto_ac_prototype::hir::DefKind;
    use auto_ac_prototype::source::compile_scalar_source;
    use auto_ac_prototype::verify::verify_function;
    use auto_av_probe::function_queries::FnEnv;
    let caps = vec!["hir.test.trace.v1".to_string()];
    let src = "fn helper() int { return mark_a() + mark_b() }
fn main() int { return helper() }
";
    let compiled = compile_scalar_source(src, "t.at", &caps).unwrap();
    let bundle = compiled.checked.bundle();
    let idx = bundle.modules[0]
        .defs
        .iter()
        .position(|d| d.id_text == "d_helper")
        .unwrap();
    let cf = verify_function(bundle.clone(), idx).unwrap();
    assert!(FnEnv::from_bundle(&bundle).unwrap().credential_current(&cf));
    // Swap the two intrinsic symbols between the def slots.
    let mut swapped = bundle.clone();
    let mut mutations = 0;
    for d in swapped.modules[0].defs.iter_mut() {
        if let DefKind::Intrinsic { symbol, .. } = &mut d.kind {
            if symbol == "hir.test.mark_a" {
                *symbol = "hir.test.mark_b".into();
                mutations += 1;
            } else if symbol == "hir.test.mark_b" {
                *symbol = "hir.test.mark_a".into();
                mutations += 1;
            }
        }
    }
    assert_eq!(mutations, 2);
    assert!(
        !FnEnv::from_bundle(&swapped)
            .unwrap()
            .credential_current(&cf),
        "swapped per-reference bindings must invalidate the credential"
    );
}

/// F-828-R1-06/R3a e: the consulted intrinsic's EFFECT fact binds too.
#[test]
fn r3a_credential_tracks_effect_fact_of_consulted_intrinsic() {
    use auto_ac_prototype::hir::{DefKind, Effect};
    use auto_ac_prototype::source::compile_scalar_source;
    use auto_ac_prototype::verify::verify_function;
    use auto_av_probe::function_queries::FnEnv;
    let caps = vec!["hir.test.trace.v1".to_string()];
    let compiled = compile_scalar_source(
        "fn helper() int { return mark_a() }
fn main() int { return helper() }",
        "t.at",
        &caps,
    )
    .unwrap();
    let bundle = compiled.checked.bundle();
    let idx = bundle.modules[0]
        .defs
        .iter()
        .position(|d| d.id_text == "d_helper")
        .unwrap();
    let cf = verify_function(bundle.clone(), idx).unwrap();
    assert!(FnEnv::from_bundle(&bundle).unwrap().credential_current(&cf));
    let mut flipped = bundle.clone();
    for d in flipped.modules[0].defs.iter_mut() {
        if let DefKind::Intrinsic { effect, .. } = &mut d.kind {
            *effect = match *effect {
                Effect::Observable => Effect::Pure,
                Effect::Pure => Effect::Observable,
            };
        }
    }
    assert!(
        !FnEnv::from_bundle(&flipped)
            .unwrap()
            .credential_current(&cf),
        "consulted effect fact changed - the credential must invalidate"
    );
}

/// F-828-R1-04/R4a: two consecutive 32-space inserts at the (moving)
/// "return 1 / 0" position compose in MINT coordinates.
#[test]
fn r4a_two_long_whitespace_edits_compose_in_code_source_coordinates() {
    let mut s = warmup(R2A_SRC, "d_main", &[]).unwrap();
    for revision in 1..=2 {
        let at = s.src().find("return 1 / 0").unwrap();
        let out = s.edit(at as u64, at as u64, vec![b' '; 32]).unwrap();
        assert!(out.ok, "{out:?}");
        let cold = warmup(s.src(), "d_main", &[]).unwrap();
        assert_eq!(
            trap_span(&s),
            trap_span(&cold),
            "revision {revision}: current edit offset is not a mint-source offset"
        );
    }
}

/// F-828-R1-04/R4a: a LATER-in-range edit then an EARLIER edit keep BOTH
/// range-end mappings.
#[test]
fn r4a_later_then_earlier_edits_preserve_both_range_end_mappings() {
    let mut s = warmup(R2A_SRC, "d_main", &[]).unwrap();
    let at = s.src().find(" / 0").unwrap();
    let out = s.edit(at as u64, at as u64, b" ".to_vec()).unwrap();
    assert!(out.ok, "{out:?}");
    let cold = warmup(s.src(), "d_main", &[]).unwrap();
    assert_eq!(
        trap_span(&s),
        trap_span(&cold),
        "first intra-range edit control"
    );
    let at = s.src().find("return 1").unwrap();
    let out = s.edit(at as u64, at as u64, vec![b' '; 32]).unwrap();
    assert!(out.ok, "{out:?}");
    let cold = warmup(s.src(), "d_main", &[]).unwrap();
    assert_eq!(
        trap_span(&s),
        trap_span(&cold),
        "second earlier edit must not erase the first range-end change"
    );
}

/// F-828-R1-06/R4a: binding identity is the def ID TEXT — a caller edit
/// stays installable even when the synth program drops another function's
/// intrinsic (def indices shift, id texts do not).
#[test]
fn r4a_mixed_intrinsics_keep_binding_across_synth_def_index_shift() {
    let caps = vec!["hir.test.trace.v1".to_string()];
    let src = "fn helper() int { return mark_a() }
fn main() int { return helper() + mark_b() + 1 }
fn tail() int { return 9 }
";
    let mut s = warmup(src, "d_main", &caps).expect("trace warm-up");
    let cold = warmup(src, "d_main", &caps).unwrap();
    assert_eq!(value(&cold), value(&s), "cold parity control");
    let at = s.src().find("+ 1").unwrap() + 2;
    let out = s.edit(at as u64, (at + 1) as u64, b"2".to_vec()).unwrap();
    assert!(
        out.ok,
        "binding unchanged while synth-table index moved: {out:?}"
    );
    assert_eq!(value(&s), value(&warmup(s.src(), "d_main", &caps).unwrap()));
}

/// F-828-R1-04/R5a: whitespace inserted EXACTLY at the span start pushes
/// the token right.
#[test]
fn r5a_whitespace_inserted_exactly_at_span_start_tracks_token_start() {
    let mut s = warmup(R2A_SRC, "d_main", &[]).unwrap();
    let at = s.src().find("1 / 0").unwrap();
    let out = s.edit(at as u64, at as u64, b" ".to_vec()).unwrap();
    assert!(out.ok, "{out:?}");
    let cold = warmup(s.src(), "d_main", &[]).unwrap();
    assert_eq!(
        trap_span(&s),
        trap_span(&cold),
        "the token moves right when whitespace is inserted exactly before it"
    );
}

/// F-828-R1-04/R5a: after a DELETION, an edit strictly after the
/// expression leaves the earlier span intact (the inverse's mint preimage
/// can exceed the current offset).
#[test]
fn r5a_deletion_then_edit_after_trap_does_not_move_earlier_span() {
    let src = R2A_SRC.replace(
        "var x = helper()",
        &(format!("{}var x = helper()", " ".repeat(32))),
    );
    let mut s = warmup(&src, "d_main", &[]).unwrap();
    let at = s.src().find(&" ".repeat(32)).unwrap();
    let out = s.edit(at as u64, (at + 32) as u64, Vec::new()).unwrap();
    assert!(out.ok, "{out:?}");
    let cold = warmup(s.src(), "d_main", &[]).unwrap();
    assert_eq!(
        trap_span(&s),
        trap_span(&cold),
        "deletion-only accepted control"
    );
    let expected = trap_span(&cold);
    let at = s.src().find("1 / 0").unwrap() + "1 / 0".len();
    let out = s.edit(at as u64, at as u64, b" ".to_vec()).unwrap();
    assert!(out.ok, "{out:?}");
    let cold = warmup(s.src(), "d_main", &[]).unwrap();
    assert_eq!(
        trap_span(&cold),
        expected,
        "edit strictly after the expression leaves its token span intact"
    );
    assert_eq!(trap_span(&s), expected);
}

/// F-828-R1-04/R5a: editing INSIDE previously inserted whitespace
/// inverse-maps to the original insertion anchor.
#[test]
fn r5a_editing_inside_prior_inserted_whitespace_keeps_nearby_anchor() {
    let mut s = warmup(R2A_SRC, "d_main", &[]).unwrap();
    let start = s.src().find("1 / 0").unwrap() - 1;
    let out = s.edit(start as u64, start as u64, vec![b' '; 32]).unwrap();
    assert!(out.ok, "{out:?}");
    let cold = warmup(s.src(), "d_main", &[]).unwrap();
    assert_eq!(
        trap_span(&s),
        trap_span(&cold),
        "first insertion accepted control"
    );
    let inside = start + 16;
    let out = s.edit(inside as u64, inside as u64, b" ".to_vec()).unwrap();
    assert!(out.ok, "{out:?}");
    let cold = warmup(s.src(), "d_main", &[]).unwrap();
    assert_eq!(
        trap_span(&s),
        trap_span(&cold),
        "inserted text must inverse-map to its original insertion anchor"
    );
}

const R6A_SRC: &str = "fn helper() int { return 1 }
fn main() int { var x = helper(); return 1 / 0 }
fn tail() int { return 9 }
";

/// F-828-R1-04/R6a: insert-then-undo restores IDENTITY mapping (the
/// deleted insertion leaves no region); a pinned pre-undo snapshot keeps
/// its own map.
#[test]
fn r6a_undo_and_paused_snapshot_keep_their_distinct_source_maps() {
    use auto_av_core::av::{execute_program_steps, resume_program, SteppedOutcome};
    let mut s = warmup(R6A_SRC, "d_main", &[]).unwrap();
    let at = s.src().find("return 1 / 0").unwrap();
    let out = s.edit(at as u64, at as u64, vec![b' '; 32]).unwrap();
    assert!(out.ok, "{out:?}");
    let expected_pinned = trap_span(&s);
    let pinned = s.current();
    let paused =
        match execute_program_steps(&pinned, &ExecLimits::default(), &CancelFlag::new(false), 1) {
            SteppedOutcome::Paused(p) => p,
            SteppedOutcome::Finished(out, _) => panic!("expected an in-flight chain: {out:?}"),
        };
    let out = s.edit(at as u64, (at + 32) as u64, Vec::new()).unwrap();
    assert!(out.ok, "{out:?}");
    assert_eq!(s.src(), R6A_SRC, "real undo restored source bytes");
    let cold = warmup(R6A_SRC, "d_main", &[]).unwrap();
    assert_eq!(trap_span(&s), trap_span(&cold), "undo control");
    match resume_program(paused, &CancelFlag::new(false), None) {
        SteppedOutcome::Finished(
            ExecOutcome::Trap {
                span: Some(actual), ..
            },
            _,
        ) => {
            assert_eq!(actual, expected_pinned)
        }
        _ => panic!("expected pinned old revision trap"),
    }
}

/// F-828-R1-04/R6a: deleting PART of previously inserted whitespace
/// records a mint-length-0 trim (the current length is not the mint
/// length).
#[test]
fn r6a_deleting_part_of_inserted_whitespace_does_not_delete_mint_tokens() {
    let mut s = warmup(R6A_SRC, "d_main", &[]).unwrap();
    let at = s.src().find("return 1 / 0").unwrap();
    let out = s.edit(at as u64, at as u64, vec![b' '; 32]).unwrap();
    assert!(out.ok, "{out:?}");
    let cold = warmup(s.src(), "d_main", &[]).unwrap();
    assert_eq!(trap_span(&s), trap_span(&cold), "insertion control");
    let out = s
        .edit((at + 8) as u64, (at + 24) as u64, Vec::new())
        .unwrap();
    assert!(out.ok, "{out:?}");
    let cold = warmup(s.src(), "d_main", &[]).unwrap();
    assert_eq!(
        trap_span(&s),
        trap_span(&cold),
        "removing inserted bytes must not treat old_len as mint length"
    );
}

/// F-828-R1-04/R6a: editing inside a REPLACEMENT's expanded output must
/// not back-map into an unrelated EARLIER function's span.
#[test]
fn r6a_expanded_whitespace_replacement_cannot_move_an_unmodified_earlier_trap() {
    let src = "fn helper() int { return 1 / 0 }
fn main() int { return helper() }
fn tail() int { return 9 }
";
    let mut s = warmup(src, "d_main", &[]).unwrap();
    let original = trap_span(&s);
    let at = s.src().find("return helper()").unwrap() - 1;
    assert_eq!(&s.src()[at..at + 1], " ");
    let out = s.edit(at as u64, (at + 1) as u64, vec![b' '; 64]).unwrap();
    assert!(out.ok, "{out:?}");
    let cold = warmup(s.src(), "d_main", &[]).unwrap();
    assert_eq!(trap_span(&cold), original, "edit is after helper");
    assert_eq!(trap_span(&s), original, "expanded replacement control");
    let out = s
        .edit((at + 32) as u64, (at + 48) as u64, Vec::new())
        .unwrap();
    assert!(out.ok, "{out:?}");
    let cold = warmup(s.src(), "d_main", &[]).unwrap();
    assert_eq!(trap_span(&cold), original);
    assert_eq!(
        trap_span(&s),
        original,
        "editing inside replacement output must not back-map into an earlier function"
    );
}

/// r7a-check: named combination controls + the 186-cell interval matrix,
/// verbatim from the review replay (cold compilation is the oracle).
fn r7a_edit_and_compare(
    s: &mut auto_av_probe::session::FunctionIncrementalSession,
    a: usize,
    b: usize,
    n: usize,
    label: &str,
) {
    let out = s.edit(a as u64, b as u64, vec![b' '; n]).unwrap();
    assert!(out.ok, "{label}: {out:?}");
    let cold = warmup(s.src(), "d_main", &[]).unwrap();
    assert_eq!(
        trap_span(s),
        trap_span(&cold),
        "{label}: edit=({a},{b},{n})"
    );
}

#[test]
fn r7a_replacing_entire_insertion_with_nonempty_output_keeps_new_output() {
    let mut s = warmup(R6A_SRC, "d_main", &[]).unwrap();
    let at = s.src().find("return 1 / 0").unwrap();
    r7a_edit_and_compare(&mut s, at, at, 32, "insert 32");
    r7a_edit_and_compare(&mut s, at, at + 32, 16, "replace all inserted bytes");
}

#[test]
fn r7a_spanning_replacement_restores_original_source() {
    let mut s = warmup(R6A_SRC, "d_main", &[]).unwrap();
    let at = s.src().find("1 / 0").unwrap() - 1;
    r7a_edit_and_compare(&mut s, at, at, 32, "insert before original separator");
    let out = s.edit(at as u64, (at + 33) as u64, b" ".to_vec()).unwrap();
    assert!(out.ok, "{out:?}");
    assert_eq!(s.src(), R6A_SRC, "33-to-1 restores exact original bytes");
    let cold = warmup(R6A_SRC, "d_main", &[]).unwrap();
    assert_eq!(
        trap_span(&s),
        trap_span(&cold),
        "delta discarded with the insertion"
    );
}

#[test]
fn r7a_partial_delete_then_delete_rest_recovers_identity() {
    let mut s = warmup(R6A_SRC, "d_main", &[]).unwrap();
    let at = s.src().find("return 1 / 0").unwrap();
    r7a_edit_and_compare(&mut s, at, at, 32, "insert 32");
    r7a_edit_and_compare(&mut s, at + 8, at + 24, 0, "delete middle 16");
    let out = s.edit(at as u64, (at + 16) as u64, Vec::new()).unwrap();
    assert!(out.ok, "{out:?}");
    assert_eq!(s.src(), R6A_SRC);
    let cold = warmup(R6A_SRC, "d_main", &[]).unwrap();
    assert_eq!(trap_span(&s), trap_span(&cold), "undo remaining insertion");
}

#[test]
fn r7a_replacing_expansion_output_recovers_identity() {
    let mut s = warmup(R6A_SRC, "d_main", &[]).unwrap();
    let at = s.src().find("1 / 0").unwrap() - 1;
    r7a_edit_and_compare(&mut s, at, at + 1, 64, "expand original separator");
    let out = s.edit(at as u64, (at + 64) as u64, b" ".to_vec()).unwrap();
    assert!(out.ok, "{out:?}");
    assert_eq!(s.src(), R6A_SRC);
    let cold = warmup(R6A_SRC, "d_main", &[]).unwrap();
    assert_eq!(
        trap_span(&s),
        trap_span(&cold),
        "undo whitespace replacement"
    );
}

fn r7a_edge_delete_then_insert(a: usize, b: usize) {
    let mut s = warmup(R6A_SRC, "d_main", &[]).unwrap();
    let at = s.src().find("1 / 0").unwrap();
    r7a_edit_and_compare(&mut s, at, at, 32, "insert at token boundary");
    r7a_edit_and_compare(&mut s, at + a, at + b, 0, "delete edge of insertion");
    let at2 = s.src().find("1 / 0").unwrap();
    r7a_edit_and_compare(&mut s, at2, at2, 8, "insert again at current token start");
}

#[test]
fn r7a_insertion_prefix_deletion_then_insertion() {
    r7a_edge_delete_then_insert(0, 16);
}

#[test]
fn r7a_insertion_suffix_deletion_then_insertion() {
    r7a_edge_delete_then_insert(16, 32);
}

#[test]
fn r7a_replacement_internal_delete_then_reinsert() {
    let mut s = warmup(R6A_SRC, "d_main", &[]).unwrap();
    let at = s.src().find("1 / 0").unwrap() - 1;
    r7a_edit_and_compare(&mut s, at, at + 1, 64, "expand before trap");
    r7a_edit_and_compare(&mut s, at + 32, at + 48, 0, "delete inside output");
    r7a_edit_and_compare(&mut s, at + 12, at + 12, 7, "insert inside trimmed output");
}

#[test]
fn r7a_disjoint_insertions_then_covering_replacement() {
    let source = R6A_SRC.replacen("return 1 / 0", "return    1 / 0", 1);
    let mut s = warmup(&source, "d_main", &[]).unwrap();
    let at = s.src().find("1 / 0").unwrap() - 4;
    r7a_edit_and_compare(&mut s, at, at, 8, "insert before four mint spaces");
    r7a_edit_and_compare(&mut s, at + 12, at + 12, 8, "second disjoint insertion");
    let out = s
        .edit(at as u64, (at + 20) as u64, b"    ".to_vec())
        .unwrap();
    assert!(out.ok, "{out:?}");
    assert_eq!(s.src(), source);
    let cold = warmup(&source, "d_main", &[]).unwrap();
    assert_eq!(trap_span(&s), trap_span(&cold), "covering replacement");
}

#[test]
fn r7a_bounded_whitespace_interval_composition_matrix() {
    let source = R6A_SRC.replacen("return 1 / 0", "return    1 / 0", 1);
    let gap = source.find("1 / 0").unwrap() - 4;
    let mut cases = 0usize;
    let mut failures = Vec::new();
    for (first_a, first_b, first_n) in [(0usize, 0usize, 8usize), (4, 4, 8), (0, 4, 12)] {
        let boundaries = [0usize, 1, 4, 6, 11, 12];
        for (i, &a) in boundaries.iter().enumerate() {
            for &b in &boundaries[i..] {
                for n in [0usize, 1, 8] {
                    if 12 - (b - a) + n == 0 {
                        continue;
                    }
                    cases += 1;
                    let mut s = warmup(&source, "d_main", &[]).unwrap();
                    let out = s
                        .edit(
                            (gap + first_a) as u64,
                            (gap + first_b) as u64,
                            vec![b' '; first_n],
                        )
                        .unwrap();
                    assert!(out.ok, "first {out:?}");
                    let cold = warmup(s.src(), "d_main", &[]).unwrap();
                    assert_eq!(trap_span(&s), trap_span(&cold), "matrix initial control");
                    let out = s
                        .edit((gap + a) as u64, (gap + b) as u64, vec![b' '; n])
                        .unwrap();
                    assert!(out.ok, "second {out:?}");
                    let cold = warmup(s.src(), "d_main", &[]).unwrap();
                    let (actual, expected) = (trap_span(&s), trap_span(&cold));
                    if actual != expected {
                        failures.push(format!(
                            "first=({first_a},{first_b},{first_n}) second=({a},{b},{n}) actual={actual:?} expected={expected:?}"
                        ));
                    }
                }
            }
        }
    }
    eprintln!("interval_matrix cases={cases} failed={}", failures.len());
    assert!(
        failures.is_empty(),
        "{} / {cases} interval compositions disagree with cold compilation: {:#?}",
        failures.len(),
        failures
    );
}
