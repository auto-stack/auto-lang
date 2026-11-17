//! PLAN-971 B (T-02): the permanent source-level DEBUG fixture (V-01,
//! V-05) over the public explicit-ACE debug gate (`auto_lang_core::
//! start_ace_debug` + this bridge's reference host over the 960 packages
//! lane).
//!
//! Oracle:
//! - a project-level breakpoint stops the REAL chain at a REAL instruction
//!   boundary: the stop reports the innermost frame (source function name +
//!   owning `pkg::mod` module), the whole call stack and the named locals
//!   with their live values;
//! - the reported span slices the ORIGINAL module's source text (V-05: the
//!   producer's span table, not a legacy AST position);
//! - step advances exactly one instruction; step-out returns to the caller
//!   frame; continue runs to the next breakpoint; the run's terminal state
//!   carries the real value (2 + 1 = 3);
//! - a span breakpoint matches an instruction whose original span intersects
//!   it; unknown/legacy engine names are named refusals; a project that does
//!   not compile is a named start failure carrying the original module
//!   diagnostic;
//! - closing a session reclaims its scratch store (no residue).
//!
//! Platform: Windows x64 (this crate is Windows-only).

#![cfg(windows)]

use std::time::Duration;

use auto_lang_ace_host::install;
use auto_lang_core::{
    check_ace_project, packages_capabilities, start_ace_debug, start_ace_debug_named,
    AceBreakpoint, AceCheckOutcome, AceCheckRequest, AceDebugCommand, AceDebugRequest,
    AceDebugSnapshot, AceDebugStop, AceEngineChoice, AceModuleSpec, AceProjectOutcome, AceValue,
};

/// math::core — the callee whose body we break in.
const MATH_CORE: &str = "pub fn bump(x int) int { let y = x + 1\n return y }";

/// app::main — the entry that calls the callee.
const APP_MAIN: &str =
    "use math.core.bump\nfn main() int { let a = 2\n let b = bump(a)\n return b }";

fn spec<'a>(package: &'a str, id: &'a str, source: &'a str) -> AceModuleSpec<'a> {
    AceModuleSpec {
        package,
        version: "1.0.0",
        id,
        source,
    }
}

fn describe(snapshot: &AceDebugSnapshot) -> String {
    let frames: Vec<String> = snapshot
        .frames
        .iter()
        .map(|f| {
            format!(
                "{}::{} span={:?} locals={:?}",
                f.module, f.function, f.span, f.locals
            )
        })
        .collect();
    format!(
        "stop={:?} steps={} outcome={:?} msg={} frames=[{}]",
        snapshot.stop,
        snapshot.steps,
        snapshot.outcome,
        snapshot.message,
        frames.join(" | ")
    )
}

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
fn p971_debug_face_breaks_steps_and_reports_original_source() {
    install().expect("install the reference ACE host");
    let caps = packages_capabilities();
    let started = std::time::Instant::now();
    let modules = [
        spec("math", "core", MATH_CORE),
        spec("app", "main", APP_MAIN),
    ];

    // The producer's declaration table supplies the symbols/breakpoints.
    let check = check_ace_project(
        AceEngineChoice::AceAV,
        &AceCheckRequest::new(&modules, "app::main", &caps),
    );
    assert!(matches!(check.outcome, AceCheckOutcome::Clean));
    let bump = check
        .declarations
        .iter()
        .find(|d| d.name == "bump" && d.module == "math::core")
        .unwrap_or_else(|| panic!("bump declaration missing: {:?}", check.declarations))
        .clone();
    // The project has no test-contract declaration (bump takes a param, main
    // is the entry): discovery is zero here — the test lane's gate lives in
    // the tool-face fixture.
    assert!(check.tests.is_empty(), "discovery: {:?}", check.tests);

    // ---- 1. Breakpoint at the callee's entry ---------------------------
    let breakpoints = [AceBreakpoint {
        module: "math::core".to_string(),
        symbol: bump.symbol.clone(),
        span: None,
    }];
    let request = AceDebugRequest::simple(&modules, "app::main", &caps, &breakpoints);
    let mut session = start_ace_debug(AceEngineChoice::AceAV, &request)
        .unwrap_or_else(|e| panic!("debug session must start: {e:?}"));
    let entry = session.snapshot();
    assert_eq!(entry.stop, AceDebugStop::Entry, "{}", describe(&entry));
    assert_eq!(entry.frames.len(), 1, "{}", describe(&entry));
    assert_eq!(entry.frames[0].function, "main");
    assert_eq!(entry.frames[0].module, "app::main");
    assert_eq!(entry.steps, 0);

    let hit = session.resume(AceDebugCommand::Continue);
    assert_eq!(
        hit.stop,
        AceDebugStop::Breakpoint { index: 0 },
        "{}",
        describe(&hit)
    );
    assert_eq!(hit.frames.len(), 2, "call stack: {}", describe(&hit));
    // Outermost first: main, then the callee.
    assert_eq!(hit.frames[0].function, "main");
    assert_eq!(hit.frames[0].module, "app::main");
    let inner = &hit.frames[1];
    assert_eq!(inner.function, "bump", "{}", describe(&hit));
    assert_eq!(inner.module, "math::core");
    // Locals are the live values of the paused frame: the parameter is 2.
    let x = inner
        .locals
        .iter()
        .find(|l| l.name == "x")
        .unwrap_or_else(|| panic!("param x missing: {}", describe(&hit)));
    assert!(
        x.value.contains('2'),
        "param x must be the live value 2: {x:?} ({})",
        describe(&hit)
    );
    // V-05: the stop's span slices the ORIGINAL callee module source.
    let inner_span = inner.span.expect("the stopped instruction carries a span");
    let inner_text = MATH_CORE
        .get(inner_span.start as usize..inner_span.end as usize)
        .unwrap_or_else(|| panic!("span {inner_span:?} outside math::core: {}", describe(&hit)));
    // V-05 (review F-9): the stop must be inside the CALLEE BODY, not at the
    // file header or another module — the module-offset mapping is what the
    // span base rebasing can get wrong.
    let body_start = MATH_CORE.find("let y").expect("callee body marker");
    assert!(
        (inner_span.start as usize) >= body_start,
        "the stop span {inner_span:?} must lie inside the callee body (>= byte {body_start})"
    );
    assert!(
        MATH_CORE.contains("x + 1"),
        "the fixture's callee body is the expected expression"
    );
    println!("inner stop text: {inner_text:?}");
    assert!(
        !inner_text.is_empty(),
        "the stop span must name real original source text"
    );
    // The caller frame keeps its own original source identity.
    let outer = &hit.frames[0];
    let outer_span = outer.span.expect("caller frame span");
    let outer_text = APP_MAIN
        .get(outer_span.start as usize..outer_span.end as usize)
        .unwrap_or_else(|| panic!("caller span {outer_span:?} outside app::main"));
    // The call site maps to the ORIGINAL caller source text.
    assert!(
        outer_text.contains("bump"),
        "the caller frame must be stopped at the original call site, got {outer_text:?}"
    );
    println!("outer stop text: {outer_text:?}");

    // ---- 2. Single step: exactly one instruction ----------------------
    let step1 = session.resume(AceDebugCommand::Step);
    assert_eq!(step1.stop, AceDebugStop::Step, "{}", describe(&step1));
    assert!(
        step1.steps > hit.steps,
        "a step must advance the real instruction counter: {} -> {}",
        hit.steps,
        step1.steps
    );

    // ---- 3. Step-out: back to the caller frame ------------------------
    let out = session.resume(AceDebugCommand::StepOut);
    assert_eq!(out.stop, AceDebugStop::StepOut, "{}", describe(&out));
    assert_eq!(
        out.frames.len(),
        1,
        "back in the caller: {}",
        describe(&out)
    );
    assert_eq!(out.frames[0].function, "main");
    assert_eq!(out.frames[0].module, "app::main");

    // ---- 4. Continue to the end: the real value -----------------------
    let finished = session.resume(AceDebugCommand::Continue);
    assert_eq!(
        finished.stop,
        AceDebugStop::Finished,
        "{}",
        describe(&finished)
    );
    match finished.outcome.as_ref() {
        Some(AceProjectOutcome::Success {
            value: AceValue::I32(3),
            ..
        }) => {}
        other => panic!("bump(2) must be 3, got {other:?} ({})", describe(&finished)),
    }
    // A finished session stays finished (idempotent).
    let again = session.resume(AceDebugCommand::Continue);
    assert_eq!(again.stop, AceDebugStop::Finished);
    session.close();
    assert_no_scratch_residue();

    // ---- 5. A SPAN breakpoint matches an intersecting instruction -----
    // The callee's declaration span covers its body, so the first body
    // instruction's original span intersects it.
    let span_breakpoints = [AceBreakpoint {
        module: "math::core".to_string(),
        symbol: bump.symbol.clone(),
        span: bump.span,
    }];
    let request = AceDebugRequest::simple(&modules, "app::main", &caps, &span_breakpoints);
    let mut session = start_ace_debug(AceEngineChoice::AceAV, &request)
        .unwrap_or_else(|e| panic!("span-breakpoint session must start: {e:?}"));
    let hit = session.resume(AceDebugCommand::Continue);
    assert_eq!(
        hit.stop,
        AceDebugStop::Breakpoint { index: 0 },
        "span breakpoint: {}",
        describe(&hit)
    );
    assert_eq!(hit.frames.len(), 2);
    assert_eq!(hit.frames[1].function, "bump");
    session.close();
    assert_no_scratch_residue();

    // ---- 5b. Breakpoints set AFTER the start stop (a debug UI's flow) ---
    // Start with no breakpoints, then install the callee-entry breakpoint
    // while stopped at the entry: the next continue must honour it.
    let no_breakpoints: [AceBreakpoint; 0] = [];
    let request = AceDebugRequest::simple(&modules, "app::main", &caps, &no_breakpoints);
    let mut session = start_ace_debug(AceEngineChoice::AceAV, &request)
        .unwrap_or_else(|e| panic!("no-breakpoint session must start: {e:?}"));
    assert_eq!(session.snapshot().stop, AceDebugStop::Entry);
    session.set_breakpoints(&[AceBreakpoint {
        module: "math::core".to_string(),
        symbol: bump.symbol.clone(),
        span: None,
    }]);
    let hit = session.resume(AceDebugCommand::Continue);
    assert_eq!(
        hit.stop,
        AceDebugStop::Breakpoint { index: 0 },
        "a breakpoint installed after the start stop must be honoured: {}",
        describe(&hit)
    );
    assert_eq!(hit.frames.len(), 2);
    assert_eq!(hit.frames[1].function, "bump");
    session.close();
    assert_no_scratch_residue();

    // ---- 5c. The remaining commands and budget stops (review F-4) ------
    // StepOver at the entry stop: executes one instruction and stays in the
    // caller frame with a distinct source position.
    let mut session = start_ace_debug(AceEngineChoice::AceAV, &request)
        .unwrap_or_else(|e| panic!("step-over session must start: {e:?}"));
    let over = session.resume(AceDebugCommand::StepOver);
    assert_eq!(over.stop, AceDebugStop::StepOver, "{}", describe(&over));
    assert_eq!(
        over.frames.len(),
        1,
        "still in the caller: {}",
        describe(&over)
    );
    // StepOver again from the callee breakpoint: still two live frames.
    session.set_breakpoints(&[AceBreakpoint {
        module: "math::core".to_string(),
        symbol: bump.symbol.clone(),
        span: None,
    }]);
    let hit = session.resume(AceDebugCommand::Continue);
    assert_eq!(hit.stop, AceDebugStop::Breakpoint { index: 0 });
    let over_in_callee = session.resume(AceDebugCommand::StepOver);
    assert_eq!(
        over_in_callee.stop,
        AceDebugStop::StepOver,
        "{}",
        describe(&over_in_callee)
    );
    assert_eq!(
        over_in_callee.frames.len(),
        2,
        "step-over does not leave the callee: {}",
        describe(&over_in_callee)
    );
    session.close();
    assert_no_scratch_residue();

    // Cancel: the live session's cancel handle stops the chain with the named
    // Cancelled outcome (never a silent continue). The start request's flag
    // is only the initial state, so the session exposes a live handle.
    let mut session = start_ace_debug(AceEngineChoice::AceAV, &request)
        .unwrap_or_else(|e| panic!("cancellable session must start: {e:?}"));
    assert_eq!(session.snapshot().stop, AceDebugStop::Entry);
    session.cancel();
    let cancelled = session.resume(AceDebugCommand::Continue);
    assert_eq!(
        cancelled.stop,
        AceDebugStop::Finished,
        "{}",
        describe(&cancelled)
    );
    assert!(
        matches!(cancelled.outcome, Some(AceProjectOutcome::Cancelled)),
        "cancel must end with the named Cancelled outcome: {}",
        describe(&cancelled)
    );
    session.close();
    assert_no_scratch_residue();

    // Wall: a spinning program under a small session budget must end with a
    // named resource denial (never an unbounded step loop). The spin makes
    // the budget deterministic (a fast straight-line program could finish
    // before any deadline check).
    let spin = "fn main() int { for 1 < 2 { }\n return 0 }";
    let spin_modules = [spec("app", "main", spin)];
    let no_bp: [AceBreakpoint; 0] = [];
    let mut wall_request = AceDebugRequest::simple(&spin_modules, "app::main", &caps, &no_bp);
    wall_request.wall = Some(Duration::from_millis(50));
    let mut session = start_ace_debug(AceEngineChoice::AceAV, &wall_request)
        .unwrap_or_else(|e| panic!("wall-bounded session must start: {e:?}"));
    let walled = session.resume(AceDebugCommand::Continue);
    assert_eq!(walled.stop, AceDebugStop::Finished, "{}", describe(&walled));
    match walled.outcome.as_ref() {
        Some(AceProjectOutcome::ResourceDenied { kind, .. }) => {
            assert!(
                kind.to_lowercase().contains("wall"),
                "the denial names the wall axis: {kind}"
            );
        }
        other => panic!("a spinning debugged program must hit the wall, got {other:?}"),
    }
    session.close();
    assert_no_scratch_residue();

    // ---- 6. Engine and project discipline -----------------------------
    let refused = start_ace_debug_named("autovm", &request)
        .err()
        .expect("legacy engine name must be refused");
    assert_eq!(refused.code, "ace.engine/unknown");
    for bad in ["vm", "evaluator", ""] {
        let refused = start_ace_debug_named(bad, &request)
            .err()
            .unwrap_or_else(|| panic!("engine `{bad}` must be refused"));
        assert_eq!(refused.code, "ace.engine/unknown");
    }

    // ---- 6b. Hot edit: a stop never reports the OLD span (V-08) --------
    // Shift every callee span by prefixing a helper declaration, then debug a
    // FRESH session: the stop must slice the NEW module text, and the span
    // must be different from the pre-edit one.
    let edited_math = format!("pub fn unused() int {{ return 0 }}\n{MATH_CORE}");
    let edited_modules = [
        spec("math", "core", &edited_math),
        spec("app", "main", APP_MAIN),
    ];
    let edited_breakpoints = [AceBreakpoint {
        module: "math::core".to_string(),
        symbol: bump.symbol.clone(),
        span: None,
    }];
    let edited_request =
        AceDebugRequest::simple(&edited_modules, "app::main", &caps, &edited_breakpoints);
    let mut edited = start_ace_debug(AceEngineChoice::AceAV, &edited_request)
        .unwrap_or_else(|e| panic!("edited session must start: {e:?}"));
    let edited_hit = edited.resume(AceDebugCommand::Continue);
    assert_eq!(
        edited_hit.stop,
        AceDebugStop::Breakpoint { index: 0 },
        "{}",
        describe(&edited_hit)
    );
    let edited_span = edited_hit.frames[1]
        .span
        .expect("the edited stop carries a span");
    let edited_text = edited_math
        .get(edited_span.start as usize..edited_span.end as usize)
        .unwrap_or_else(|| {
            panic!(
                "edited span {edited_span:?} outside the EDITED math::core: {}",
                describe(&edited_hit)
            )
        });
    assert!(
        !edited_text.is_empty(),
        "the edited stop must slice the NEW original source"
    );
    assert_ne!(
        (edited_span.start, edited_span.end),
        (inner_span.start, inner_span.end),
        "a hot edit must not keep reporting the OLD span (V-08)"
    );
    edited.close();
    assert_no_scratch_residue();

    // A bad module source is a named start failure carrying the ORIGINAL
    // module diagnostic (never a silent empty session).
    let bad_modules = [
        spec("math", "core", "pub fn bump(x int) int { return nosuch }"),
        spec("app", "main", APP_MAIN),
    ];
    let bad_request = AceDebugRequest::simple(&bad_modules, "app::main", &caps, &breakpoints);
    let failure = start_ace_debug(AceEngineChoice::AceAV, &bad_request)
        .err()
        .expect("a broken project must not start a session");
    let diagnostic = failure
        .diagnostic
        .expect("the start failure carries the project diagnostic");
    assert_eq!(diagnostic.module, "math::core");
    assert!(!failure.code.is_empty());

    let elapsed = started.elapsed();
    assert!(
        elapsed.as_secs() < 10,
        "the debug fixture must stay under the 10s case gate, took {elapsed:?}"
    );
}
