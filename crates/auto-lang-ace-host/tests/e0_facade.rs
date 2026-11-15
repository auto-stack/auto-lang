//! PLAN-969 E0 (T-E0-I/V): the public facade cases V-E0-DISPATCH /
//! V-E0-CLOSE / V-E0-SOURCE, driven through the REAL public door
//! (`auto_lang_core::run_ace` + this bridge's executor) — no label
//! checks, every assertion observes the dispatched result, counters,
//! authorization state or named refusal.
//!
//! The provider cases build the frozen F0 text provider once per test
//! process (explicit integration lane, not a daily union member; the
//! 10-second per-case budget is measured and reported by T-E0-V).

use std::sync::atomic::AtomicBool;
use std::time::Instant;

use auto_lang_core::{
    run_ace, run_ace_named, AceEngineChoice, AceOutcome, AceRequest, AceRunReport, AceSpan,
    AceValue,
};

/// Assert a successful i32 run, panicking with the FULL outcome on any
/// other state (diagnosis discipline: failures show their real state).
fn expect_success_i32(report: &AceRunReport, expected: i32) {
    match &report.outcome {
        AceOutcome::Success {
            value: AceValue::I32(v),
            steps,
        } => {
            assert_eq!(*v, expected, "success value (steps={steps})");
        }
        other => panic!("expected Success(I32({expected})), got {other:?}"),
    }
}

fn install() {
    auto_lang_ace_host::install().expect("the host executor installs");
}

fn pure_request<'a>(source: &'a str) -> AceRequest<'a> {
    AceRequest::simple(source)
}

fn provider_request<'a>(source: &'a str, name: &'a str) -> AceRequest<'a> {
    AceRequest {
        provider: Some(name),
        ..AceRequest::simple(source)
    }
}

/// V-E0-DISPATCH: an explicit ACE request runs plain Auto against the F0
/// Rust provider — real dispatch (value + callee counter), refusal
/// discipline for unknown/legacy engines, wrong signatures and revoked
/// authority, and the exact-fix rerun through the same door.
#[test]
fn p969_e0_dispatch() {
    let started = Instant::now();
    install();
    let scratch = tempfile::tempdir().expect("scratch dir");
    auto_lang_ace_host::build_f0_text_provider("f0_text", scratch.path())
        .expect("the frozen F0 text provider builds");

    // The §5 r6 oracle: `text_byte_len("A中😀\0")` = 9 UTF-8 bytes, NUL
    // intact, through ONE real dispatch.
    let good = "use.rs f0_text\nfn main() int { return text_byte_len(\"A中😀\\0\") }";
    let report = run_ace(AceEngineChoice::AceAV, &provider_request(good, "f0_text"));
    expect_success_i32(&report, 9); // explicit ACE dispatch returns the UTF-8 byte length
    assert_eq!(
        report.callee_count,
        Some(1),
        "exactly one real provider call"
    );
    assert_eq!(report.rejected_count, Some(0));

    // Unknown engine on the SAME input: a named refusal, zero dispatch,
    // never a value and never a default-engine switch.
    let report = run_ace_named("ace-m2", &provider_request(good, "f0_text"));
    assert!(matches!(
        &report.outcome,
        AceOutcome::Denied { code, .. } if code == "ace.engine/unknown"
    ));

    // Legacy engine names are refused by the ACE door (legacy dispatch=0:
    // the legacy VM is structurally unreachable from run_ace).
    for legacy in ["autovm", "vm", "evaluator"] {
        let report = run_ace_named(legacy, &provider_request(good, "f0_text"));
        assert!(
            matches!(&report.outcome, AceOutcome::Denied { code, .. } if code == "ace.engine/unknown"),
            "legacy engine `{legacy}` must be a named refusal, got {:?}",
            report.outcome
        );
    }

    // Wrong signature: a source diagnostic with the ORIGINAL span/stage,
    // produced before any provider load or dispatch.
    let bad_sig = "use.rs f0_text\nfn main() int { return text_byte_len(1, 2) }";
    let report = run_ace(
        AceEngineChoice::AceAV,
        &provider_request(bad_sig, "f0_text"),
    );
    match report.outcome {
        AceOutcome::Diagnostic {
            stage, code, span, ..
        } => {
            // Pinned oracle (T-E0-R F-E0R-03): the fact lane binds the
            // call at the ORIGINAL stage/span — `text_byte_len(1, 2)`.
            assert_eq!(stage, "bind", "wrong-signature diagnoses at the bind stage");
            assert_eq!(code, "source.argument");
            assert_eq!(
                span,
                Some(AceSpan { start: 38, end: 57 }),
                "the span covers the original call site"
            );
        }
        other => panic!("wrong signature must be a diagnostic, got {other:?}"),
    }
    assert_eq!(
        report.callee_count, None,
        "compile failure dispatches nothing"
    );

    // Foreign-lane capabilities are explicit too: a capability outside the
    // frozen F0 set is refused before admission, never silently dropped
    // (T-E0-R F-E0R-04).
    let foreign_cap = AceRequest {
        capabilities: &[std::string::String::from("console")],
        ..provider_request(good, "f0_text")
    };
    let report = run_ace(AceEngineChoice::AceAV, &foreign_cap);
    assert!(
        matches!(&report.outcome, AceOutcome::Denied { code, .. } if code == "ace.capability/unsupported"),
        "unfrozen capability refuses before admission, got {:?}",
        report.outcome
    );

    // Revoked authority: pre-dispatch refusal, then the exact fix (readmit)
    // serves the SAME door again.
    assert!(auto_lang_ace_host::revoke_provider("f0_text"));
    let report = run_ace(AceEngineChoice::AceAV, &provider_request(good, "f0_text"));
    assert!(
        matches!(&report.outcome, AceOutcome::Denied { code, .. } if code == "ace.provider/revoked"),
        "revoked provider refuses before dispatch, got {:?}",
        report.outcome
    );
    assert_eq!(report.callee_count, None, "revocation dispatches nothing");
    assert!(auto_lang_ace_host::readmit_provider("f0_text"));
    let report = run_ace(AceEngineChoice::AceAV, &provider_request(good, "f0_text"));
    expect_success_i32(&report, 9);
    assert_eq!(
        report.callee_count,
        Some(1),
        "readmission restores the real dispatch"
    );

    println!(
        "p969_e0_dispatch elapsed: {:.3}s",
        started.elapsed().as_secs_f64()
    );
}

/// V-E0-CLOSE: twenty create/close rounds over two isolated request
/// identities — results and per-run counters never cross; the closed
/// provider refuses; host failure and cancellation are separately
/// observable states.
#[test]
fn p969_e0_close() {
    let started = Instant::now();
    install();
    let scratch = tempfile::tempdir().expect("scratch dir");
    // One build, TWO registered identities: revoking one must not affect
    // the other (authorization isolation).
    auto_lang_ace_host::build_f0_text_providers(&["f0_a", "f0_b"], scratch.path())
        .expect("the frozen F0 provider builds for both identities");

    let src_a = "use.rs f0_text\nfn main() int { return text_byte_len(\"A中😀\\0\") }";
    let src_b = "use.rs f0_text\nfn main() int { return text_byte_len(\"\") }";

    for round in 0..20 {
        // Session A: 9, exactly one call THIS run (a fresh table per
        // request — counters must not accumulate across rounds).
        let report = run_ace(AceEngineChoice::AceAV, &provider_request(src_a, "f0_a"));
        expect_success_i32(&report, 9); // round {round}: session A keeps its result
        assert_eq!(
            report.callee_count,
            Some(1),
            "round {round}: session A own counter"
        );
        // Session B: 0 through the same provider but a different request.
        let report = run_ace(AceEngineChoice::AceAV, &provider_request(src_b, "f0_b"));
        expect_success_i32(&report, 0); // round {round}: session B keeps its result
        assert_eq!(
            report.callee_count,
            Some(1),
            "round {round}: session B own counter"
        );
    }

    // Authorization isolation: revoking A leaves B serving.
    assert!(auto_lang_ace_host::revoke_provider("f0_a"));
    let report = run_ace(AceEngineChoice::AceAV, &provider_request(src_a, "f0_a"));
    assert!(
        matches!(&report.outcome, AceOutcome::Denied { code, .. } if code == "ace.provider/revoked")
    );
    let report = run_ace(AceEngineChoice::AceAV, &provider_request(src_b, "f0_b"));
    expect_success_i32(&report, 0);
    assert!(auto_lang_ace_host::readmit_provider("f0_a"));

    // Host close: after teardown the provider identity refuses (host
    // failure with a named code) — a DISTINCT state from denial.
    assert!(auto_lang_ace_host::teardown_provider("f0_a"));
    let report = run_ace(AceEngineChoice::AceAV, &provider_request(src_a, "f0_a"));
    assert!(
        matches!(&report.outcome, AceOutcome::HostFailure { code, .. } if code == "ace.provider/unknown"),
        "a closed provider is a named host failure, got {:?}",
        report.outcome
    );

    // Cancellation is its own observable state (host sets the flag; the
    // run reports Cancelled, distinct from denial/host failure).
    let flag = AtomicBool::new(true);
    let cancelled = AceRequest {
        cancel: Some(&flag),
        ..provider_request(src_b, "f0_b")
    };
    let report = run_ace(AceEngineChoice::AceAV, &cancelled);
    assert_eq!(report.outcome, AceOutcome::Cancelled);

    println!(
        "p969_e0_close elapsed: {:.3}s",
        started.elapsed().as_secs_f64()
    );
}

/// V-E0-SOURCE: plain source diagnostics keep the original span and the
/// correct stage; the wrong entry is an `entry`-stage diagnostic; the
/// same door serves the fixed source again. The B01 oracle (14) pins the
/// §5 r6 concrete input.
#[test]
fn p969_e0_source() {
    let started = Instant::now();
    install();

    // §5 r6: the B01 source returns 14 through an explicit ACE request.
    let b01 = "fn main() int { return 2 + 3 * 4 }";
    let report = run_ace(AceEngineChoice::AceAV, &pure_request(b01));
    expect_success_i32(&report, 14); // B01 through the explicit ACE door returns 14

    // A syntax-broken edit: original span + producing stage.
    let broken = "fn main() int { return 2 + }";
    let report = run_ace(AceEngineChoice::AceAV, &pure_request(broken));
    match report.outcome {
        AceOutcome::Diagnostic {
            stage, code, span, ..
        } => {
            // Pinned oracle (T-E0-R F-E0R-03): syntax diagnosis at the
            // ORIGINAL text stage, span on the offending `}`.
            assert_eq!(stage, "text");
            assert_eq!(code, "source.syntax");
            assert_eq!(span, Some(AceSpan { start: 27, end: 28 }));
        }
        other => panic!("broken source must diagnose, got {other:?}"),
    }

    // A profile-foreign literal: still the original span/stage, different
    // producer code (pinned: the string literal `"x"` at 23..26).
    let typed = "fn main() int { return \"x\" }";
    let report = run_ace(AceEngineChoice::AceAV, &pure_request(typed));
    match report.outcome {
        AceOutcome::Diagnostic {
            stage, code, span, ..
        } => {
            assert_eq!(stage, "text");
            assert_eq!(code, "source.unsupported");
            assert_eq!(span, Some(AceSpan { start: 23, end: 26 }));
        }
        other => panic!("profile-foreign literal must diagnose, got {other:?}"),
    }

    // The wrong entry id is an `entry`-stage diagnostic (correct stage,
    // not a lower-stage crash).
    let wrong_entry = AceRequest {
        entry_def_id: "d_missing",
        ..pure_request(b01)
    };
    let report = run_ace(AceEngineChoice::AceAV, &wrong_entry);
    match report.outcome {
        AceOutcome::Diagnostic { stage, code, .. } => {
            assert_eq!(stage, "entry");
            assert_eq!(code, "ace.entry/missing");
        }
        other => panic!("wrong entry must diagnose at the entry stage, got {other:?}"),
    }

    // The fix lands through the SAME door: the good candidate is kept
    // and the result changes back.
    let report = run_ace(AceEngineChoice::AceAV, &pure_request(b01));
    expect_success_i32(&report, 14);

    // Pure-lane grants are an explicit refusal (nothing is admitted to
    // grant against on this lane in E0).
    let granted = AceRequest {
        grants: &[std::string::String::from("foreign.scalar")],
        ..pure_request(b01)
    };
    let report = run_ace(AceEngineChoice::AceAV, &granted);
    assert!(
        matches!(&report.outcome, AceOutcome::Denied { code, .. } if code == "ace.grant/no-provider"),
        "pure-lane grants refuse explicitly, got {:?}",
        report.outcome
    );

    println!(
        "p969_e0_source elapsed: {:.3}s",
        started.elapsed().as_secs_f64()
    );
}
