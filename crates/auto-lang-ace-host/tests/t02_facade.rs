//! PLAN-969 T-02 permanent fixtures (V-DETAIL-01..04 anchors at the
//! public door + the frozen old-case mappings):
//! - `ace.m2.p969.source01` (B01): file input through the door, failure
//!   → exact fix → recovery through the SAME door.
//! - `ace.m2.p969.source02` (B18): the immutable-write invariant — a
//!   TYPED rejection at the original span, never an arbitrary non-zero.
//! - budget states: wall/cancel observed as AV terminal states on a
//!   poll-spanning program; the sub-interval completion cadence is
//!   pinned as the frozen contract (CANCEL_CHECK_INTERVAL = 1024 steps).
//! - refusal discipline: unknown engine names never dispatch.

use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

use auto_lang_core::{run_ace, run_ace_named, AceEngineChoice, AceOutcome, AceRequest, AceValue};

/// Write a source file and run it through the public door (the
/// file-producer shape the facade serves).
fn run_file(path: &std::path::Path) -> auto_lang_core::AceRunReport {
    let source = std::fs::read_to_string(path).expect("fixture file reads");
    run_ace(AceEngineChoice::AceAV, &AceRequest::simple(&source))
}

fn scratch_file(name: &str, source: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("p969-t02");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    std::fs::write(&path, source).unwrap();
    path
}

/// V-DETAIL-01 / mapped `ace.m1.b01` → `ace.m2.p969.source01`: the B01
/// file through the explicit door returns 14; a broken edit diagnoses at
/// the original span; the exact fix serves again through the SAME door.
#[test]
fn p969_t02_source01_b01_file_door() {
    auto_lang_ace_host::install().expect("executor installs");
    let good = scratch_file("b01.at", "fn main() int { return 2 + 3 * 4 }\n");
    let report = run_file(&good);
    match report.outcome {
        AceOutcome::Success {
            value: AceValue::I32(v),
            steps,
        } => {
            assert_eq!(v, 14);
            assert!(steps > 0, "the run executed real steps");
        }
        other => panic!("B01 through the door must succeed, got {other:?}"),
    }

    // The broken edit (same file slot) diagnoses with the original span.
    let broken = scratch_file("b01.at", "fn main() int { return 2 + }\n");
    let report = run_file(&broken);
    match report.outcome {
        AceOutcome::Diagnostic {
            stage, code, span, ..
        } => {
            assert_eq!((stage.as_str(), code.as_str()), ("text", "source.syntax"));
            assert_eq!(span.map(|s| (s.start, s.end)), Some((27, 28)));
        }
        other => panic!("broken edit must diagnose, got {other:?}"),
    }

    // The fix lands through the same door.
    let fixed = scratch_file("b01.at", "fn main() int { return 2 + 3 * 4 }\n");
    let report = run_file(&fixed);
    match report.outcome {
        AceOutcome::Success {
            value: AceValue::I32(v),
            ..
        } => assert_eq!(v, 14),
        other => panic!("fixed source must succeed, got {other:?}"),
    }
}

/// Mapped `ace.m1.ia920.b18` → `ace.m2.p969.source02`: the immutable
/// local write is a TYPED verify-stage rejection at the original span —
/// the invariant behind the legacy `auto_name_E0203` oracle, never an
/// arbitrary non-zero or a silent success.
#[test]
fn p969_t02_source02_b18_immutable_write() {
    auto_lang_ace_host::install().expect("executor installs");
    let b18 = scratch_file(
        "b18.at",
        "fn main() int {\n    let x int = 1\n    x = 2\n    return x\n}\n",
    );
    let report = run_file(&b18);
    match report.outcome {
        AceOutcome::Diagnostic {
            stage, code, span, ..
        } => {
            assert_eq!(
                stage, "verify",
                "immutable write rejects at the verify stage"
            );
            assert_eq!(code, "verify.immutable-write");
            // The span covers the offending assignment (`x = 2`).
            assert_eq!(span.map(|s| (s.start, s.end)), Some((38, 43)));
        }
        other => panic!("immutable write must be a typed rejection, got {other:?}"),
    }
}

/// V-04 anchor at the door: a wall budget is observed as the AV's own
/// terminal Resource state (the real sub-task ended), and a pre-set
/// cancel flag on a poll-spanning program yields Cancelled. The
/// sub-interval completion (short program finishing before the first
/// poll) is pinned as the FROZEN contract, not a gap.
#[test]
fn p969_t02_budget_states() {
    auto_lang_ace_host::install().expect("executor installs");
    let busy = "fn main() int { var i = 0\nwhile i < 100000000 { i = i + 1 }\nreturn i }";

    // Wall: the AV terminal state, not a parent-side timeout.
    let started = Instant::now();
    let req = AceRequest {
        wall: Some(Duration::from_millis(1)),
        ..AceRequest::simple(busy)
    };
    let report = run_ace(AceEngineChoice::AceAV, &req);
    match report.outcome {
        AceOutcome::ResourceDenied { kind, .. } => {
            assert_eq!(kind, "ExecutionWall");
        }
        other => panic!("wall budget must be the AV resource state, got {other:?}"),
    }
    assert!(
        started.elapsed() < Duration::from_secs(5),
        "the denial is prompt, not a hang"
    );

    // Cancel: observed on a poll-spanning program.
    let flag = AtomicBool::new(true);
    let req = AceRequest {
        cancel: Some(&flag),
        ..AceRequest::simple(busy)
    };
    assert_eq!(
        run_ace(AceEngineChoice::AceAV, &req).outcome,
        AceOutcome::Cancelled
    );

    // Frozen cadence: a sub-interval program completes (the poll at
    // CANCEL_CHECK_INTERVAL never fires) — pinned so a routing change
    // that "fixes" this into an abort is caught.
    let short = "fn main() int { return 2 + 3 * 4 }";
    let flag = AtomicBool::new(true);
    let req = AceRequest {
        cancel: Some(&flag),
        wall: Some(Duration::from_nanos(1)),
        ..AceRequest::simple(short)
    };
    match run_ace(AceEngineChoice::AceAV, &req).outcome {
        AceOutcome::Success {
            value: AceValue::I32(v),
            ..
        } => assert_eq!(v, 14),
        other => {
            panic!("sub-interval program completes per the frozen poll cadence, got {other:?}")
        }
    }
}

/// V-03 anchor at the door: unknown engine names (including every legacy
/// engine name) are named refusals with zero dispatch.
#[test]
fn p969_t02_unknown_engine_refuses() {
    auto_lang_ace_host::install().expect("executor installs");
    let source = "fn main() int { return 2 + 3 * 4 }";
    for name in ["ace-native", "autovm", "vm", "evaluator", "", "AV"] {
        let report = run_ace_named(name, &AceRequest::simple(source));
        assert!(
            matches!(&report.outcome, AceOutcome::Denied { code, .. } if code == "ace.engine/unknown"),
            "engine `{name}` must refuse, got {:?}",
            report.outcome
        );
    }
}

/// T-02 console lane: `print` executes under per-run console grants —
/// the captured stream comes back on the report; WITHOUT the grant the
/// run refuses before any dispatch (captured empty, dispatches 0); the
/// stderr grant is its own authority axis; the main-only entry policy
/// surfaces the adapter's own named code.
#[test]
fn p969_t02_console_lane_grants_and_capture() {
    auto_lang_ace_host::install().expect("executor installs");
    let src = "fn main() int { print(7); return 42 }";
    let grants = [std::string::String::from("console.stdout")];

    // Granted: success + the captured bytes.
    let req = AceRequest {
        console: true,
        grants: &grants,
        ..AceRequest::simple(src)
    };
    let report = run_ace(AceEngineChoice::AceAV, &req);
    match &report.outcome {
        AceOutcome::Success {
            value: AceValue::I32(v),
            ..
        } => assert_eq!(*v, 42),
        other => panic!("granted console run succeeds, got {other:?}"),
    }
    let console = report.console.as_ref().expect("console report present");
    assert_eq!(
        console.stdout, b"7\n",
        "the print lands in the captured stdout"
    );
    assert_eq!(console.stderr, b"");
    assert!(console.dispatches >= 1, "a real console dispatch happened");

    // Ungranted: pre-dispatch refusal with a NAMED code; nothing written.
    let req = AceRequest {
        console: true,
        ..AceRequest::simple(src)
    };
    let report = run_ace(AceEngineChoice::AceAV, &req);
    assert!(
        matches!(&report.outcome, AceOutcome::Denied { code, .. } if code == "host.unauthorized"),
        "missing console grants refuse before dispatch, got {:?}",
        report.outcome
    );
    let console = report
        .console
        .as_ref()
        .expect("refusal still reports the console state");
    assert_eq!(console.stdout, b"", "an unauthorized run writes nothing");
    assert_eq!(console.dispatches, 0, "and dispatches nothing");

    // The stderr grant is its own authority axis.
    let stderr_grants = [std::string::String::from("console.stderr")];
    let req = AceRequest {
        console: true,
        grants: &stderr_grants,
        ..AceRequest::simple("fn main() int { eprint(9); return 1 }")
    };
    let report = run_ace(AceEngineChoice::AceAV, &req);
    assert!(
        matches!(report.outcome, AceOutcome::Success { .. }),
        "{:?}",
        report.outcome
    );
    let console = report.console.as_ref().unwrap();
    assert_eq!(console.stderr, b"9\n");
    assert_eq!(console.stdout, b"");

    // The console front door's main-only entry policy keeps its own
    // named diagnostic (the adapter code, not a generic lower failure).
    let req = AceRequest {
        console: true,
        grants: &grants,
        entry_def_id: "d_helper",
        ..AceRequest::simple("fn helper() int { return 1 }\nfn main() int { print(7); return 42 }")
    };
    let report = run_ace(AceEngineChoice::AceAV, &req);
    match report.outcome {
        AceOutcome::Diagnostic { stage, code, .. } => {
            assert_eq!(stage, "lower");
            assert_eq!(
                code, "lower.entry",
                "the console entry policy keeps its named code"
            );
        }
        other => panic!("wrong console entry is a typed rejection, got {other:?}"),
    }
}
