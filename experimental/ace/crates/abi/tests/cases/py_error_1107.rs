//! PLAN-1107 V-04 (AV leg): an uncaught Python exception is a NAMED failure
//! with a source position — never a silent success, never a partial output.
#![cfg(windows)]

use std::path::Path;

use ace_abi::py_session::PySession;
use ace_av_core::av::ExecLimits;

const CORPUS: &str = "../../../../crates/auto-lang/test/cases/ace-m2/python-1107/error_uncaught_1107.at";

fn corpus() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(CORPUS);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

#[test]
fn p1107_v04_uncaught_python_error_is_a_named_trap_with_span() {
    let src = corpus();
    let mut session = PySession::open().expect("session opens");
    let run = session
        .run_with(
            &src,
            &ExecLimits {
                max_steps: 5_000_000,
                wall: std::time::Duration::from_secs(60),
                ..Default::default()
            },
        )
        .expect("the lane reports the failure as an outcome, not a hang");
    match &run.outcome {
        ace_av_core::contract::ExecOutcome::Trap {
            kind: ace_av_core::contract::TrapKind::ForeignStatus,
            span: Some(span),
        } => {
            // the position names the failing call (`l.pop()`), not the entry
            let text = &src;
            let at = &text[span.start as usize..span.end as usize];
            assert!(
                at.contains("pop"),
                "the trap span must name the failing call, got `{at}`"
            );
        }
        other => panic!("expected a named foreign-status trap with a span, got {other:?}"),
    }
    assert!(
        run.success_i32().is_none(),
        "an uncaught Python error never counts as a successful run"
    );
    assert_eq!(
        run.stdout_text(),
        "",
        "no partial output is published as program output"
    );
}