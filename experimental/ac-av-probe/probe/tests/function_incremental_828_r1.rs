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
