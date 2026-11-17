//! PLAN-971 B (T-02/T-08): the compile-only project CHECK face and the
//! producer-backed test discovery gate — the B batch's tool-facing facts.
//!
//! Oracle:
//! - a clean project check returns the producer's DECLARATION table (original
//!   name, owning module `pkg::mod`, arity, declared result type and a
//!   module-relative span) and its DISCOVERED test candidates (zero-parameter
//!   `int` declarations, entry `main` excluded);
//! - a bad module source is reported with the ORIGINAL module identity and a
//!   span that slices the offending text INSIDE that module's own source, and
//!   the diagnostics disappear after recovery;
//! - ZERO discovered test candidates is a NAMED failure with exit 1 — a test
//!   claim without a test is never a pass (V-03);
//! - discovered tests run through the project lane: passing declarations exit
//!   0, a failing declaration exits 1;
//! - unknown/legacy engine names are refused on both the check and the
//!   discovered-test faces (zero dispatch; the legacy AutoVM is unreachable
//!   from this contract by construction).
//!
//! Nothing here executes the project on the check leg: the check face is
//! compile-only. Platform: Windows x64 (this crate is Windows-only).

#![cfg(windows)]

use auto_lang_ace_host::{install, ENGINE_ID};
use auto_lang_core::{
    check_ace_project, check_ace_project_named, packages_capabilities, run_ace_project_tests,
    run_ace_project_tests_named, AceCheckOutcome, AceCheckRequest, AceEngineChoice, AceModuleSpec,
    AceTestRunOutcome, AceTestRunRequest,
};

/// math::core — one library module of the check project.
const MATH_CORE: &str = "pub fn eval(a int, b int, c int) int { return a + b * c }";

/// app::main — the entry module; it also carries one test-contract
/// declaration (`check_two`) that discovery must find.
const APP_MAIN: &str =
    "use math.core.eval\nfn main() int { return eval(2, 3, 4) }\nfn check_two() int { return 0 }";

fn spec<'a>(package: &'a str, id: &'a str, source: &'a str) -> AceModuleSpec<'a> {
    AceModuleSpec {
        package,
        version: "1.0.0",
        id,
        source,
    }
}

#[test]
fn p971_check_face_reports_producer_declarations_and_diagnostics() {
    install().expect("install the reference ACE host");
    let caps = packages_capabilities();
    let started = std::time::Instant::now();

    // ---- 1. Clean project: declarations + discovered tests --------------
    let modules = [
        spec("math", "core", MATH_CORE),
        spec("app", "main", APP_MAIN),
    ];
    let req = AceCheckRequest::new(&modules, "app::main", &caps);
    let report = check_ace_project(AceEngineChoice::AceAV, &req);
    assert_eq!(report.engine, ENGINE_ID);
    assert!(
        matches!(report.outcome, AceCheckOutcome::Clean),
        "the clean project must check clean, got {:?}",
        report.outcome
    );
    // The declaration table carries the producer's facts: the cross-module
    // library fn keeps its module identity, arity and rendered result type.
    let eval = report
        .declarations
        .iter()
        .find(|d| d.name == "eval" && d.module == "math::core")
        .unwrap_or_else(|| panic!("math::core::eval missing: {:?}", report.declarations));
    assert_eq!(eval.params, 3, "declared arity is a producer fact");
    assert_eq!(eval.ret, "int");
    let eval_span = eval.span.expect("the declaration carries its span");
    assert!(
        MATH_CORE
            .get(eval_span.start as usize..eval_span.end as usize)
            .is_some_and(|s| s.contains("eval")),
        "the declaration span is module-relative: {:?}",
        eval_span
    );
    // Discovery: exactly the zero-parameter `int` declaration; the entry
    // (`main`) and parameterized library fns are NOT test candidates.
    let names: Vec<&str> = report.tests.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(names, vec!["check_two"], "discovery: {:?}", report.tests);
    assert_eq!(report.tests[0].module, "app::main");
    assert_eq!(
        report.tests[0].module, "app::main",
        "a discovered test keeps its original module"
    );

    // ---- 2. Diagnostics leg: the ORIGINAL module + original span --------
    let bad_math = "pub fn eval(a int, b int, c int) int { return nosuch }";
    let bad_modules = [
        spec("math", "core", bad_math),
        spec("app", "main", APP_MAIN),
    ];
    let bad_req = AceCheckRequest::new(&bad_modules, "app::main", &caps);
    let bad = check_ace_project(AceEngineChoice::AceAV, &bad_req);
    match &bad.outcome {
        AceCheckOutcome::Diagnostics(diags) => {
            assert!(!diags.is_empty(), "a rejected project carries diagnostics");
            let d = diags
                .iter()
                .find(|d| d.module == "math::core")
                .unwrap_or_else(|| panic!("no math::core diagnostic: {diags:?}"));
            let span = d.span.expect("a diagnostics span");
            assert!(
                bad_math
                    .get(span.start as usize..span.end as usize)
                    .is_some_and(|s| s.contains("nosuch")),
                "the diagnostic span must point into math::core's own source: {d:?}"
            );
            assert!(!d.code.is_empty() && !d.message.is_empty());
        }
        other => panic!("bad source must be a diagnostic, got {other:?}"),
    }
    // A rejected project admits no declarations (no half-published facts).
    assert!(bad.declarations.is_empty() && bad.tests.is_empty());

    // Two independent bad modules: every reported diagnostic keeps a module
    // identity and a span inside THAT module's own text.
    let bad_main = "use math.core.eval\nfn main() int { return eval(2, 3, 4) }\nfn check_two() int { return nosuch_two }";
    let two_bad = [
        spec("math", "core", bad_math),
        spec("app", "main", bad_main),
    ];
    let two = check_ace_project(
        AceEngineChoice::AceAV,
        &AceCheckRequest::new(&two_bad, "app::main", &caps),
    );
    match &two.outcome {
        AceCheckOutcome::Diagnostics(diags) => {
            for d in diags {
                let text = if d.module == "math::core" {
                    bad_math
                } else {
                    assert_eq!(d.module, "app::main", "diagnostic module: {d:?}");
                    bad_main
                };
                let span = d.span.expect("span");
                assert!(
                    text.get(span.start as usize..span.end as usize).is_some(),
                    "span {span:?} out of that module's text"
                );
            }
        }
        other => panic!("two bad modules must be diagnosed, got {other:?}"),
    }

    // ---- 3. Recovery leg: the clean project checks clean again ----------
    let recovered = check_ace_project(AceEngineChoice::AceAV, &req);
    assert!(matches!(recovered.outcome, AceCheckOutcome::Clean));
    assert_eq!(recovered.tests.len(), 1);

    // Check leg is compile-only: a project whose entry would TRAP checks
    // clean (nothing ran) but its discovered tests report the real outcome.
    let trap_main = "fn main() int { return 1 / 0 }\nfn check_two() int { return 0 }";
    let trap_modules = [spec("app", "main", trap_main)];
    let trap = check_ace_project(
        AceEngineChoice::AceAV,
        &AceCheckRequest::new(&trap_modules, "app::main", &caps),
    );
    assert!(
        matches!(trap.outcome, AceCheckOutcome::Clean),
        "the check face must not execute the project: {:?}",
        trap.outcome
    );

    // ---- 4. Engine discipline on the check face ------------------------
    for bad_engine in ["autovm", "vm", "evaluator", "ace-native", ""] {
        let refused = check_ace_project_named(bad_engine, &req);
        assert!(
            matches!(&refused.outcome, AceCheckOutcome::Denied { code, .. } if code == "ace.engine/unknown"),
            "engine `{bad_engine}` must be refused on the check face, got {:?}",
            refused.outcome
        );
    }

    let elapsed = started.elapsed();
    assert!(
        elapsed.as_secs() < 10,
        "the check-face fixture must stay under the 10s case gate, took {elapsed:?}"
    );
}

#[test]
fn p971_discovery_gate_and_discovered_test_lane() {
    install().expect("install the reference ACE host");
    let caps = packages_capabilities();
    let started = std::time::Instant::now();

    // ---- 1. Zero discovery is a NAMED failure, never a pass (V-03) -----
    let no_tests = "fn main() int { return 0 }";
    let modules = [spec("app", "main", no_tests)];
    let req = AceTestRunRequest::simple(&modules, "app::main", &caps);
    let zero = run_ace_project_tests(AceEngineChoice::AceAV, &req);
    assert!(
        matches!(&zero.outcome, AceTestRunOutcome::NoTestsDiscovered { .. }),
        "zero discovered tests must be a named failure, got {:?}",
        zero.outcome
    );
    assert_eq!(zero.exit_code, 1, "zero discovery never exits zero");
    assert!(zero.tests.is_empty(), "nothing ran");
    assert!(zero.discovered.is_empty());

    // ---- 2. Discovered passing test: runs on the project lane, exit 0 ---
    let passing = "fn main() int { return 0 }\nfn check_sum() int { return 0 }";
    let modules = [
        spec("math", "core", MATH_CORE),
        spec("app", "main", passing),
    ];
    let req = AceTestRunRequest::simple(&modules, "app::main", &caps);
    let pass = run_ace_project_tests(AceEngineChoice::AceAV, &req);
    assert_eq!(
        pass.outcome,
        AceTestRunOutcome::Passed,
        "a passing discovered test exits 0"
    );
    assert_eq!(pass.exit_code, 0);
    assert_eq!(pass.discovered.len(), 1);
    assert_eq!(pass.discovered[0].name, "check_sum");
    assert_eq!(pass.discovered[0].module, "app::main");
    assert_eq!(pass.tests.len(), 1);
    assert!(pass.tests[0].passed);
    assert_eq!(pass.tests[0].name, "check_sum");

    // ---- 3. Failing discovered test: nonzero exit, named outcome -------
    let failing = "fn main() int { return 0 }\nfn check_sum() int { return 5 }";
    let modules = [spec("app", "main", failing)];
    let req = AceTestRunRequest::simple(&modules, "app::main", &caps);
    let fail = run_ace_project_tests(AceEngineChoice::AceAV, &req);
    assert!(
        matches!(&fail.outcome, AceTestRunOutcome::Failed { message } if message.contains("check_sum")),
        "a failing discovered test must be a named failure, got {:?}",
        fail.outcome
    );
    assert_eq!(fail.exit_code, 1);
    assert!(!fail.tests[0].passed);

    // ---- 4. Compile rejection reaches the run face with module identity --
    let bad = "fn main() int { return 0 }\nfn check_sum() int { return nosuch }";
    let modules = [spec("app", "main", bad)];
    let req = AceTestRunRequest::simple(&modules, "app::main", &caps);
    let diag = run_ace_project_tests(AceEngineChoice::AceAV, &req);
    match &diag.outcome {
        AceTestRunOutcome::Diagnostic(d) => {
            assert_eq!(d.module, "app::main");
            let span = d.span.expect("span");
            assert!(bad
                .get(span.start as usize..span.end as usize)
                .is_some_and(|s| s.contains("nosuch")));
        }
        other => panic!("a compile rejection must stay a diagnostic, got {other:?}"),
    }
    assert_eq!(diag.exit_code, 1);

    // ---- 4. A timing-out discovered test is distinguished from a pass ----
    // The test spins forever; the request wall budget is 50ms, so the test
    // generation must be stopped by the budget (a named resource failure on
    // that test), never reported as passed.
    let spin = "fn main() int { return 0 }\nfn check_spin() int { for 1 < 2 { }\n return 0 }";
    let modules = [spec("app", "main", spin)];
    let mut req = AceTestRunRequest::simple(&modules, "app::main", &caps);
    req.wall = Some(std::time::Duration::from_millis(50));
    let timeout = run_ace_project_tests(AceEngineChoice::AceAV, &req);
    assert!(
        !matches!(&timeout.outcome, AceTestRunOutcome::Passed),
        "a timing-out test must never pass, got {:?}",
        timeout.outcome
    );
    assert_eq!(timeout.exit_code, 1);
    // The spinning test itself is reported as a FAILED test whose cause names
    // the resource budget; the request-level wall denial (the entry run has
    // no budget left) is the distinguished terminal state.
    assert_eq!(
        timeout.tests.len(),
        1,
        "the test lane ran: {:?}",
        timeout.tests
    );
    assert!(!timeout.tests[0].passed);
    assert_eq!(timeout.tests[0].name, "check_spin");
    assert!(
        timeout.tests[0].message.to_lowercase().contains("resource")
            || timeout.tests[0].message.to_lowercase().contains("wall"),
        "the failing test names the budget cause: {}",
        timeout.tests[0].message
    );
    assert!(
        matches!(
            &timeout.outcome,
            AceTestRunOutcome::ResourceDenied { .. } | AceTestRunOutcome::Failed { .. }
        ),
        "a timing-out test is a named failure, never a pass: {:?}",
        timeout.outcome
    );

    // ---- 5. Engine and project discipline ------------------------------
    let bad_project = "fn main() int { return nosuch }";
    let modules = [spec("app", "main", bad_project)];
    let req = AceTestRunRequest::simple(&modules, "app::main", &caps);
    let diag = run_ace_project_tests(AceEngineChoice::AceAV, &req);
    assert!(matches!(
        &diag.outcome,
        AceTestRunOutcome::Diagnostic(d) if d.module == "app::main"
    ));

    // ---- 6. Engine discipline on the discovered-test face --------------
    for bad_engine in ["autovm", "vm", "evaluator", ""] {
        let refused = run_ace_project_tests_named(bad_engine, &req);
        assert!(
            matches!(&refused.outcome, AceTestRunOutcome::Denied { code, .. } if code == "ace.engine/unknown"),
            "engine `{bad_engine}` must be refused on the test face, got {:?}",
            refused.outcome
        );
        assert_eq!(refused.exit_code, 1);
    }

    // ---- 7. Discovery-contract consequence (review F-5, named) ---------
    // The frozen contract is a zero-parameter `int` DECLARATION (entry
    // `main` excluded), not the legacy `#[test]` attribute. A legitimate
    // zero-arg helper in a library module is therefore discovered too: the
    // lane executes it, its non-zero return fails the run. This leg records
    // the false-positive consequence instead of leaving it implicit; the
    // owner for narrowing discovery / restoring an attribute surface is the
    // language side (plan §5 B boundaries / KNOWN-DEBT).
    let helper = "fn main() int { return 0 }\nfn version() int { return 3 }";
    let modules = [spec("app", "main", helper)];
    let req = AceTestRunRequest::simple(&modules, "app::main", &caps);
    let helper_run = run_ace_project_tests(AceEngineChoice::AceAV, &req);
    assert_eq!(
        helper_run.discovered.len(),
        1,
        "a zero-arg int helper is discovered under the frozen contract: {:?}",
        helper_run.discovered
    );
    assert_eq!(helper_run.discovered[0].name, "version");
    assert_eq!(helper_run.discovered[0].params, 0);
    assert!(
        !matches!(&helper_run.outcome, AceTestRunOutcome::Passed),
        "the helper's non-zero return is not a pass: {:?}",
        helper_run.outcome
    );
    assert_eq!(helper_run.exit_code, 1);

    let elapsed = started.elapsed();
    assert!(
        elapsed.as_secs() < 10,
        "the discovery fixture must stay under the 10s case gate, took {elapsed:?}"
    );
}
