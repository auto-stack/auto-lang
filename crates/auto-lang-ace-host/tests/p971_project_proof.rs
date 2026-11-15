//! PLAN-971 B-PROOF-I/V (A batch): the permanent V-A-PROOF fixture — one
//! REAL multi-file project roundtrip through the public explicit-ACE
//! PROJECT gate (`auto_lang_core::run_ace_project` + this bridge's
//! reference host over the 960 packages lane).
//!
//! Oracle (frozen by the T-01 freeze record, PLAN-971 §5): values of a
//! real cross-module project; a bad module source is diagnosed in the
//! ORIGINAL file (`pkg::mod` identity) at its original span, and the
//! diagnostics disappear after recovery; the test lane reports per-test
//! outcomes with a NONZERO protocol exit when any test fails; a set
//! cancel flag stops the request and every scratch store is reclaimed
//  (observable: no residue under the temp root); unknown/legacy engine
//! names are named refusals with zero dispatch; a legal neighbor project
//! still runs after every negative leg. The legacy AutoVM is never
//! dispatched (structural: the neutral contract has no path to it).
//!
//! A-batch scope: breakpoints/stack/locals and in-run step granularity
//! are the B batch (T-02); this fixture proves the protocol capability
//! only. Platform: Windows x64 (this crate is Windows-only).

#![cfg(windows)]

use std::sync::atomic::AtomicBool;

use auto_lang_ace_host::{install, ENGINE_ID};
use auto_lang_core::{
    packages_capabilities, run_ace_project, run_ace_project_named, AceEngineChoice, AceModuleSpec,
    AceProjectOutcome, AceProjectRequest, AceValue,
};

/// math::core — one library module of the proof project.
const MATH_CORE: &str = "pub fn eval(a int, b int, c int) int { return a + b * c }";

/// app::main — the entry module consuming the library cross-module.
const APP_MAIN: &str = "use math.core.eval\nfn main() int { return eval(2, 3, 4) }";

fn spec<'a>(package: &'a str, id: &'a str, source: &'a str) -> AceModuleSpec<'a> {
    AceModuleSpec {
        package,
        version: "1.0.0",
        id,
        source,
    }
}

fn project<'a>(
    modules: &'a [AceModuleSpec<'a>],
    entry_module: &'a str,
    caps: &'a [String],
) -> AceProjectRequest<'a> {
    AceProjectRequest::simple_project(modules, entry_module, caps)
}

/// No `ace-p971-project-<pid>-*` scratch root may survive any request
/// (stop → reclaim, observed on the real temp root).
fn assert_no_scratch_residue() {
    let prefix = format!("ace-p971-project-{}-", std::process::id());
    let residue: Vec<_> = std::fs::read_dir(std::env::temp_dir())
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().starts_with(&prefix))
        .collect();
    assert!(residue.is_empty(), "scratch store residue: {residue:?}");
}

#[test]
fn p971_project_proof_roundtrip() {
    install().expect("install the reference ACE host");

    let caps = packages_capabilities();
    let started = std::time::Instant::now();

    // ---- 1. Value leg: a real cross-module project (2 + 3 * 4 = 14) ----
    let modules = [
        spec("math", "core", MATH_CORE),
        spec("app", "main", APP_MAIN),
    ];
    let req = project(&modules, "app::main", &caps);
    let report = run_ace_project(AceEngineChoice::AceAV, &req);
    assert_eq!(report.engine, ENGINE_ID);
    assert!(
        matches!(&report.outcome, AceProjectOutcome::Success { value: AceValue::I32(14), steps } if *steps > 0),
        "cross-module entry must run through the explicit project gate, got {:?}",
        report.outcome
    );
    assert_eq!(report.exit_code, 0, "an all-success request exits zero");
    assert!(report.wall_ms >= 0.0);

    // ---- 2. Diagnostics leg: bad source is located in the ORIGINAL ----
    // file (module identity + module-relative span), not the entry file.
    let bad_math = "pub fn eval(a int, b int, c int) int { return nosuch }";
    let bad_modules = [
        spec("math", "core", bad_math),
        spec("app", "main", APP_MAIN),
    ];
    let bad_req = project(&bad_modules, "app::main", &caps);
    let bad_report = run_ace_project(AceEngineChoice::AceAV, &bad_req);
    match &bad_report.outcome {
        AceProjectOutcome::Diagnostic { module, span, .. } => {
            assert_eq!(
                module, "math::core",
                "the diagnostic must carry the ORIGINAL module identity"
            );
            let span = span.expect("the package-lane diagnostic carries a span");
            let text = &bad_math[span.start as usize..span.end as usize];
            assert!(
                text.contains("nosuch"),
                "span {span:?} must point at the offending name inside math::core's own source, got {text:?}"
            );
        }
        other => panic!("bad source must be a diagnostic, got {other:?}"),
    }
    assert_eq!(bad_report.exit_code, 1, "a diagnosed request exits nonzero");
    assert!(bad_report.diag_count >= 1);

    // ---- 3. Recovery leg: the same project with the fixed module runs ----
    // again — the diagnostics disappear.
    let fixed_report = run_ace_project(AceEngineChoice::AceAV, &req);
    assert!(matches!(
        fixed_report.outcome,
        AceProjectOutcome::Success {
            value: AceValue::I32(14),
            ..
        }
    ));

    // ---- 4. Test lane: per-test outcomes; failure is a NONZERO exit ----
    let test_main = concat!(
        "fn main() int { return 0 }\n",
        "fn ok_check() int { return 0 }\n",
        "fn bad_check() int { return 1 }\n",
        "fn trap_check() int { return 1 / 0 }\n",
    );
    let test_modules = [spec("app", "main", test_main)];
    let tests: Vec<&str> = vec!["ok_check", "bad_check", "trap_check", "missing_check"];
    let test_req = AceProjectRequest {
        modules: &test_modules,
        entry_module: "app::main",
        entry_def_id: "d_main",
        tests: &tests,
        capabilities: &caps,
        wall: None,
        cancel: None,
    };
    let test_report = run_ace_project(AceEngineChoice::AceAV, &test_req);
    assert_eq!(test_report.tests.len(), 4, "every requested test reports");
    let by_name = |n: &str| {
        test_report
            .tests
            .iter()
            .find(|t| t.name == n)
            .unwrap_or_else(|| panic!("test {n} missing"))
    };
    let ok = by_name("ok_check");
    assert!(ok.passed, "ok_check: {:?}", ok.message);
    assert_eq!(ok.module, "app::main");
    let bad = by_name("bad_check");
    assert!(!bad.passed, "a nonzero-returning test must fail");
    assert!(
        bad.message.contains("0"),
        "reason names the contract: {}",
        bad.message
    );
    let trap = by_name("trap_check");
    assert!(!trap.passed, "a trapping test must fail");
    let missing = by_name("missing_check");
    assert!(!missing.passed, "an unknown test entry must fail");
    assert!(missing.message.contains("not found"));
    assert!(missing.module.is_empty());
    // The entry still runs to completion — the protocol exit aggregates.
    assert!(matches!(
        test_report.outcome,
        AceProjectOutcome::Success {
            value: AceValue::I32(0),
            ..
        }
    ));
    assert_eq!(test_report.exit_code, 1, "test failure NEVER exits zero");

    // All-pass neighbor: same project, only the passing test.
    let pass_req = AceProjectRequest {
        modules: &test_modules,
        entry_module: "app::main",
        entry_def_id: "d_main",
        tests: &["ok_check"],
        capabilities: &caps,
        wall: None,
        cancel: None,
    };
    let pass_report = run_ace_project(AceEngineChoice::AceAV, &pass_req);
    assert!(pass_report.tests.iter().all(|t| t.passed));
    assert_eq!(pass_report.exit_code, 0);

    // ---- 5. Stop leg: a set cancel flag stops the request and the ----
    // scratch stores are reclaimed (observable on the temp root).
    let cancel = AtomicBool::new(true);
    let stop_req = AceProjectRequest {
        modules: &test_modules,
        entry_module: "app::main",
        entry_def_id: "d_main",
        tests: &tests,
        capabilities: &caps,
        wall: None,
        cancel: Some(&cancel),
    };
    let stop_report = run_ace_project(AceEngineChoice::AceAV, &stop_req);
    assert_eq!(stop_report.outcome, AceProjectOutcome::Cancelled);
    assert_eq!(stop_report.exit_code, 1);
    assert_no_scratch_residue();

    // ---- 6. Refusal leg: legacy/unknown engine names never dispatch ----
    for engine in ["autovm", "vm", "evaluator", "ace-native"] {
        let refused = run_ace_project_named(engine, &req);
        assert!(
            matches!(&refused.outcome, AceProjectOutcome::Denied { code, .. } if code == "ace.engine/unknown"),
            "engine `{engine}` refused on the project face, got {:?}",
            refused.outcome
        );
        assert_eq!(refused.exit_code, 1);
    }

    // ---- 7. Legal neighbor after every negative leg: the executor ----
    // still serves the original project (no poisoned state).
    let neighbor = run_ace_project(AceEngineChoice::AceAV, &req);
    assert!(matches!(
        neighbor.outcome,
        AceProjectOutcome::Success {
            value: AceValue::I32(14),
            ..
        }
    ));
    assert_no_scratch_residue();

    eprintln!(
        "p971_project_proof_roundtrip: 9 requests, wall {:.1}ms",
        started.elapsed().as_secs_f64() * 1000.0
    );
}
