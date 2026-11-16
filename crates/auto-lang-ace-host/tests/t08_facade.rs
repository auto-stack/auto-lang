//! PLAN-969 T-08: the compatibility front-door audit at the ACE door —
//! V-08's controlled-rejection oracles plus V-01's producer consistency.
//!
//! Door mapping (frozen audit table; every legacy export keeps its
//! legacy owner — the ACE door neither replaces nor silently routes
//! them):
//! - `run`/`run_with_capture*`/`run_file*`/`test_file` — legacy AutoVM
//!   owners, unchanged (pinned by the existing core/auto-lang suites);
//!   the ACE answer to the same needs is the explicit `run_ace` door.
//! - `run_with_mode`/`run_file_with_auto_mode` — legacy compile-only
//!   info-string owners, unchanged (pinned by unified_api_tests).
//! - `run_abt`/`create_vm_from_abt` — legacy ABT (human-readable
//!   bytecode) owners; the ACE door takes SOURCE only and rejects ABT
//!   artifacts as source (controlled rejection — they can never pass as
//!   ACE instructions).
//! - `eval_config_with_vm` — legacy config VM-eval owner (pac.at).
//! - `test_file`/`test_code`/`test_code_with_deadline_and_input`/
//!   `test_code_with_path_deadline_and_input`, `execute_autovm*`,
//!   `extract_autovm_result`, `create_vm_from_source` — legacy AutoVM
//!   owners, unchanged (byte-identical across the PLAN-969 range).
//! - cwd / process exit-code mapping — legacy-owner process semantics
//!   (CLI layer, auto-man); NOT ACE-door surface in M2-A.
//! - args/stdin — explicit request inputs with NAMED refusals until the
//!   AV lanes exist (`ace.args/unsupported` / `ace.input/unsupported`).

use auto_lang_core::{run_ace, AceEngineChoice, AceOutcome, AceRequest, AceValue};

/// AC-06/V-08: a REAL legacy ABT program text through the ACE door is a
/// CONTROLLED source rejection — the door compiles Auto source only;
/// bytecode text can never execute as ACE instructions.
#[test]
fn p969_t08_abt_artifact_is_a_controlled_rejection() {
    auto_lang_ace_host::install().expect("executor installs");
    let abt = "\n.exports\n  main -> @main\n\n.code\nmain:\n  fn.prolog 0, 0\n  const.i32 42\n  call.nat nat#1\n  ret 0\n";
    let report = run_ace(AceEngineChoice::AceAV, &AceRequest::simple(abt));
    match report.outcome {
        AceOutcome::Diagnostic { stage, .. } => {
            assert_eq!(stage, "text", "ABT text rejects at the source front door");
        }
        AceOutcome::Success { .. } => {
            panic!("ABT bytecode must never execute through the ACE door")
        }
        other => panic!("ABT text is a controlled rejection, got {other:?}"),
    }
}

/// AC-06: argv/stdin are explicit inputs with NAMED refusals (the AV
/// lane has no input surface yet) — refused before any lane work, never
/// silently dropped, never routed to the legacy VM.
#[test]
fn p969_t08_args_and_input_refuse_by_name() {
    auto_lang_ace_host::install().expect("executor installs");
    let source = "fn main() int { return 2 + 3 * 4 }";

    let args = ["--flag", "value"];
    let req = AceRequest {
        args: &args,
        ..AceRequest::simple(source)
    };
    let report = run_ace(AceEngineChoice::AceAV, &req);
    assert!(
        matches!(&report.outcome, AceOutcome::Denied { code, .. } if code == "ace.args/unsupported"),
        "argv refuses by name, got {:?}",
        report.outcome
    );

    for input in [Some(""), Some("line\n")] {
        let req = AceRequest {
            input,
            ..AceRequest::simple(source)
        };
        let report = run_ace(AceEngineChoice::AceAV, &req);
        assert!(
            matches!(&report.outcome, AceOutcome::Denied { code, .. } if code == "ace.input/unsupported"),
            "stdin {input:?} refuses by name, got {:?}",
            report.outcome
        );
    }

    // The same door without args/input still serves (the refusal is
    // scoped to the unsupported inputs, not a door-wide failure).
    let report = run_ace(AceEngineChoice::AceAV, &AceRequest::simple(source));
    match report.outcome {
        AceOutcome::Success {
            value: AceValue::I32(v),
            ..
        } => assert_eq!(v, 14),
        other => panic!("the plain door still serves, got {other:?}"),
    }
}

/// V-01: the SAME source through the file producer and the string
/// producer yields the identical terminal state (producer consistency).
#[test]
fn p969_t08_file_and_string_producers_agree() {
    auto_lang_ace_host::install().expect("executor installs");
    let source = "fn main() int { return 2 + 3 * 4 }";
    let dir = std::env::temp_dir().join("p969-t08");
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("b01.at");
    std::fs::write(&file, source).unwrap();

    let via_string = run_ace(AceEngineChoice::AceAV, &AceRequest::simple(source));
    let file_source = std::fs::read_to_string(&file).unwrap();
    let via_file = run_ace(AceEngineChoice::AceAV, &AceRequest::simple(&file_source));

    assert_eq!(
        via_string.outcome, via_file.outcome,
        "file/string producers agree"
    );
    assert!(matches!(
        via_file.outcome,
        AceOutcome::Success {
            value: AceValue::I32(14),
            ..
        }
    ));
}
