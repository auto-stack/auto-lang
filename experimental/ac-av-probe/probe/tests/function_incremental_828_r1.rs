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
            value: Value::I32(v), ..
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
