//! PLAN-865 source-to-session evidence: compile source with the closures
//! descriptor, lower each function as a unit, then execute through the
//! existing 823 update/link/runtime path.

use auto_ac_prototype::hir::DefKind;
use auto_ac_prototype::source::compile_closures_source;
use auto_av_core::mir::MirType;
use auto_av_probe::closures_865_support::{ClosuresSourceSession, CompileAndRunError};
use auto_av_probe::closures_session::ClosuresUpdateSession;
use auto_mir_lower::lower_function_closures_unit;
use std::sync::Arc;

fn closure_units(source: &str) -> Vec<(String, auto_av_core::mir::MirModule)> {
    let compiled = compile_closures_source(source, "closures-865-session.at", &[])
        .unwrap_or_else(|diagnostics| panic!("source compilation failed: {diagnostics:#?}"));
    let bundle = compiled.checked.bundle();
    let module = &bundle.modules[0];
    module
        .defs
        .iter()
        .filter(|def| matches!(def.kind, DefKind::Function { .. }))
        .map(|def| {
            let unit = lower_function_closures_unit(bundle, &def.id_text, &[], 1)
                .unwrap_or_else(|error| panic!("lower {}: {error:?}", def.id_text));
            (unit.funcs[0].name.clone(), unit)
        })
        .collect()
}

fn submitted(
    units: &[(String, auto_av_core::mir::MirModule)],
) -> Vec<(&str, auto_av_core::mir::MirModule)> {
    units
        .iter()
        .map(|(name, module)| (name.as_str(), module.clone()))
        .collect()
}

fn run_source(source: &str, expected: i32) {
    let units = closure_units(source);
    let mut session = ClosuresUpdateSession::new(None, "d_main");
    let outcome = session
        .apply_revision(&submitted(&units))
        .expect("source closure units link and execute");
    assert_eq!(outcome.value, Some(expected));
    assert_eq!(outcome.live_after_release, 0);
    assert_eq!(outcome.allocs, outcome.frees);
    assert!(outcome.gate.pass());
    session.close().expect("session roots close cleanly");
}

#[test]
fn av_closures_865_k01_k03_run_source_through_session() {
    run_source(
        "fn main() int { let base int = 40; let f = move (n int) => { return base + n }; return f(2) }",
        42,
    );
    run_source(
        "fn main() int { let base int = 40; let f = (n int) => { return base + n }; return f(2) }",
        42,
    );
}

#[test]
fn av_closures_865_runs_zero_multi_and_direct_lambda_calls() {
    run_source(
        "fn main() int { let answer = () => { return 42 }; return answer() }",
        42,
    );
    run_source(
        "fn main() int { let add = (left int, right int) => left + right; return add(20, 22) }",
        42,
    );
    run_source("fn main() int { return ((n int) => n + 1)(41) }", 42);
}

#[test]
fn av_closures_865_passes_closure_to_function_typed_parameter() {
    run_source(
        "fn apply(f fn(int) -> int, value int) int { return f(value) } fn make(move s Str) fn(int) -> int { return move (n int) => { return s.len() + n } } fn main() int { let s Str = \"hello\"; let f = make(s.move); return apply(f, 37) }",
        42,
    );
}

#[test]
fn av_closures_865_k02_k05_run_owned_return_and_clone_source() {
    run_source(
        "fn make(move s Str) fn(int) int { return move (n int) => { return s.len() + n } } fn main() int { let s Str = \"hello\"; let f = make(s.move); return f(37) }",
        42,
    );
    run_source(
        "fn make(move s Str) fn(int) int { return move (n int) => { return s.len() + n } } fn main() int { let s Str = \"hello\"; let f = make(s.move); let g = f.clone(); return f(10) + g(22) }",
        42,
    );
}

#[test]
fn av_closures_865_source_clone_deep_copies_mutable_owned_list_env() {
    run_source(
        "fn main() int { let values = List<int>.new(); let push_and_len = move (n int) => { values.push(n); return values.len() }; let duplicate = push_and_len.clone(); return push_and_len(2) * 10 + duplicate(3) }",
        11,
    );
}

#[test]
fn av_closures_865_k04_k06_reject_unsupported_borrow_escape_and_mutation() {
    let escape = "fn bad() fn(int) int { let s Str = \"local\"; return (n int) => { return s.len() + n } } fn main() int { let f = bad(); return f(37) }";
    let diagnostics = compile_closures_source(escape, "K04.at", &[])
        .expect_err("a borrow-capturing return must fail before execution");
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "source.closure-borrow-escape"),
        "unexpected K04 diagnostics: {diagnostics:#?}"
    );

    let mutable = "fn main() int { var base int = 1; var f = move (n int) => { base = base + n; return base }; return f(20) + f(0) }";
    let diagnostics = compile_closures_source(mutable, "K06.at", &[])
        .expect_err("mutable closure state is outside the frozen profile");
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "source.closure-shared-mutable"),
        "unexpected K06 diagnostics: {diagnostics:#?}"
    );
}

#[test]
fn av_closures_865_env_drift_refuses_revision_and_preserves_installed_creator() {
    let source = "fn main() int { let base int = 40; let f = move (n int) => { return base + n }; return f(2) }";
    let units = closure_units(source);
    let creator = units
        .iter()
        .find(|(name, _)| name == "d_main")
        .expect("creator unit");
    let closure = units
        .iter()
        .find(|(name, _)| name != "d_main")
        .expect("synthetic closure body unit");
    let mut session = ClosuresUpdateSession::new(None, "d_main");
    let first = session
        .apply_revision(&submitted(&units))
        .expect("initial source revision");
    assert_eq!(first.value, Some(42));
    let installed_creator = Arc::clone(session.unit("d_main").expect("installed creator"));

    let mut drifted = creator.1.clone();
    let bool_ty = drifted.intern_type(MirType::Bool);
    let import = drifted
        .imports
        .iter_mut()
        .find(|import| import.symbol == format!("fn:{}", closure.0))
        .expect("creator has a closure-body import");
    let env = import.env.as_mut().expect("closure import declares env");
    assert_eq!(env.len(), 1);
    env[0] = bool_ty;
    let error = session
        .apply_revision(&[(creator.0.as_str(), drifted)])
        .expect_err("env drift must fail before publishing a candidate");
    assert!(error.contains("env-contract"), "{error}");
    assert!(Arc::ptr_eq(
        &installed_creator,
        session
            .unit("d_main")
            .expect("old creator remains installed")
    ));

    let body_only = closure_units(
        "fn main() int { let base int = 40; let f = move (n int) => { return base + n + 1 }; return f(2) }",
    );
    let replacement_body = body_only
        .iter()
        .find(|(name, _)| name == &closure.0)
        .expect("stable synthetic closure identity");
    let outcome = session
        .apply_revision(&[(replacement_body.0.as_str(), replacement_body.1.clone())])
        .expect("body-only replacement reuses the installed creator");
    assert_eq!(outcome.value, Some(43));
    assert_eq!(outcome.production.verified, 1);
    assert_eq!(outcome.gate.unchanged_degraded, 0);
    assert!(Arc::ptr_eq(
        &installed_creator,
        session.unit("d_main").expect("creator carried unchanged")
    ));
    assert_eq!(outcome.live_after_release, 0);
    session.close().expect("session roots close cleanly");
}

#[test]
fn av_closures_865_body_edit_produces_only_the_changed_function() {
    let source = include_str!("../../../../crates/auto-lang/test/cases/ace-m1/src/K01.at");
    let edited = source.replace("return base + n", "return base + n + 1");
    assert_ne!(
        source.len(),
        edited.len(),
        "exercise a changed closure span"
    );
    let store_root = std::env::temp_dir().join(format!(
        "plan865-source-edit-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    std::fs::create_dir_all(&store_root).expect("create isolated closure store");
    let mut session = ClosuresSourceSession::new(&store_root);
    let first = session
        .apply_source(source, "K01.at")
        .expect("initial source applies");
    assert_eq!(first.value, 42);
    let next = session
        .apply_source(&edited, "K01.at")
        .expect("changed closure body applies");
    assert_eq!(next.value, 43);
    assert_eq!(next.submitted_units, 1);
    assert_eq!(next.submitted_unit_names.len(), 1);
    assert_ne!(next.submitted_unit_names[0], "d_main");
    assert_eq!(next.production.verified, 1);
    assert_eq!(next.production.encoded, 1);
    assert_eq!(next.production.admitted, 1);
    session.close().expect("release isolated session roots");
    std::fs::remove_dir_all(&store_root).expect("remove isolated closure store");
}

#[test]
fn av_closures_865_capture_and_signature_edits_rebuild_exact_units() {
    let initial = "fn untouched() int { return 7 } fn main() int { let base int = 40; let bonus int = 1; let f = move (n int) => { return base + n }; return f(2) }";
    let capture_edit = initial.replace("return base + n", "return base + bonus + n");
    let signature_edit = "fn untouched() int { return 7 } fn main() int { let base int = 40; let bonus int = 1; let f = move (n int, extra int) => { return base + bonus + n + extra }; return f(1, 1) }";
    let store_root = std::env::temp_dir().join(format!(
        "plan865-capture-signature-edit-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    std::fs::create_dir_all(&store_root).expect("create isolated closure store");
    let mut session = ClosuresSourceSession::new(&store_root);

    let first = session
        .apply_source(initial, "capture-signature.at")
        .expect("initial source applies");
    assert_eq!(first.value, 42);
    assert!(first.submitted_unit_names.contains(&"d_main".to_string()));
    assert!(first
        .submitted_unit_names
        .contains(&"d_untouched".to_string()));

    let captures_changed = session
        .apply_source(&capture_edit, "capture-signature.at")
        .expect("capture-set edit applies");
    assert_eq!(captures_changed.value, 43);
    assert_eq!(
        captures_changed.submitted_units, 2,
        "capture edit submitted units: {:?}",
        captures_changed.submitted_unit_names
    );
    assert_eq!(captures_changed.production.verified, 2);
    assert_eq!(captures_changed.production.encoded, 2);
    assert_eq!(captures_changed.production.admitted, 2);
    assert!(captures_changed
        .submitted_unit_names
        .contains(&"d_main".to_string()));
    assert!(captures_changed
        .submitted_unit_names
        .iter()
        .any(|name| name != "d_main" && name != "d_untouched"));
    assert!(!captures_changed
        .submitted_unit_names
        .contains(&"d_untouched".to_string()));
    assert_eq!(captures_changed.live_after_release, 0);

    let signature_changed = session
        .apply_source(signature_edit, "capture-signature.at")
        .expect("closure signature edit applies");
    assert_eq!(signature_changed.value, 43);
    assert_eq!(
        signature_changed.submitted_units, 2,
        "signature edit submitted units: {:?}",
        signature_changed.submitted_unit_names
    );
    assert_eq!(signature_changed.production.verified, 2);
    assert_eq!(signature_changed.production.encoded, 2);
    assert_eq!(signature_changed.production.admitted, 2);
    assert!(signature_changed
        .submitted_unit_names
        .contains(&"d_main".to_string()));
    assert!(signature_changed
        .submitted_unit_names
        .iter()
        .any(|name| name != "d_main" && name != "d_untouched"));
    assert!(!signature_changed
        .submitted_unit_names
        .contains(&"d_untouched".to_string()));
    assert_eq!(signature_changed.live_after_release, 0);

    let invalid_candidate = "fn untouched() int { return 7 } fn main() int { let base int = 40; let bonus int = 1; let f = move (n int, extra int) => { return base + bonus + n + extra }; return f(true, 1) }";
    let diagnostics = match session.apply_source(invalid_candidate, "capture-signature.at") {
        Err(CompileAndRunError::Diagnostics(diagnostics)) => diagnostics,
        Err(CompileAndRunError::Pipeline(message)) => {
            panic!("invalid signature candidate reached the runtime pipeline: {message}")
        }
        Ok(_) => panic!("invalid signature candidate must be rejected before publication"),
    };
    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "source.type-mismatch"));

    let retained = session
        .apply_source(signature_edit, "capture-signature.at")
        .expect("previously installed source remains usable");
    assert_eq!(retained.value, 43);
    assert_eq!(retained.submitted_units, 0);
    assert_eq!(retained.production.verified, 0);
    assert_eq!(retained.live_after_release, 0);
    session.close().expect("release isolated session roots");
    std::fs::remove_dir_all(&store_root).expect("remove isolated closure store");
}
