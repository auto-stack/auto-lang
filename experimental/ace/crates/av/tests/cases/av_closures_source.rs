//! PLAN-865 source-to-session evidence: compile source with the closures
//! descriptor, lower each function as a unit, then execute through the
//! existing 823 update/link/runtime path.

use ace_av::closures_865_support::{ClosuresSourceSession, CompileAndRunError};
use ace_av::closures_session::ClosuresUpdateSession;
use ace_av_core::mir::MirType;
use ace_core::hir::DefKind;
use ace_core::source::compile_closures_source;
use ace_lowering::lower_function_closures_unit;
use std::sync::Arc;

fn closure_units(source: &str) -> Vec<(String, ace_av_core::mir::MirModule)> {
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
    units: &[(String, ace_av_core::mir::MirModule)],
) -> Vec<(&str, ace_av_core::mir::MirModule)> {
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
fn av_closures_k01_k03_run_source_through_session() {
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
fn av_closures_runs_zero_multi_and_direct_lambda_calls() {
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
fn av_closures_passes_closure_to_function_typed_parameter() {
    run_source(
        "fn apply(f fn(int) -> int, value int) int { return f(value) } fn make(move s Str) fn(int) -> int { return move (n int) => { return s.len() + n } } fn main() int { let s Str = \"hello\"; let f = make(s.move); return apply(f, 37) }",
        42,
    );
}

#[test]
fn av_closures_k02_k05_run_owned_return_and_clone_source() {
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
fn av_closures_source_clone_deep_copies_mutable_owned_list_env() {
    run_source(
        "fn main() int { let values = List<int>.new(); let push_and_len = move (n int) => { values.push(n); return values.len() }; let duplicate = push_and_len.clone(); return push_and_len(2) * 10 + duplicate(3) }",
        11,
    );
}

#[test]
fn av_closures_k04_rejects_borrow_escape_and_non_scalar_shared_write() {
    let escape = "fn bad() fn(int) int { let s Str = \"local\"; return (n int) => { return s.len() + n } } fn main() int { let f = bad(); return f(37) }";
    let diagnostics = compile_closures_source(escape, "K04.at", &[])
        .expect_err("a borrow-capturing return must fail before execution");
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "source.closure-borrow-escape"),
        "unexpected K04 diagnostics: {diagnostics:#?}"
    );

    // PLAN-909 named unsupported domain: a non-scalar shared write (owned
    // Str) keeps the `source.closure-shared-mutable` rejection — the cell
    // contract covers scalar int/bool/char vars only and never silently
    // clones or shares an owned aggregate.
    let owned_write = "fn main() int { var s Str = \"a\"; var f = (n int) => { s = \"b\"; return n }; return f(42) }";
    let diagnostics = compile_closures_source(owned_write, "shared-str.at", &[])
        .expect_err("a non-scalar shared write is outside the cell domain");
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "source.closure-shared-mutable"),
        "unexpected non-scalar shared-write diagnostics: {diagnostics:#?}"
    );

    // A written `let` capture stays the static immutable-assignment
    // rejection (the body local mirrors the source's immutability).
    let let_write = "fn main() int { let x int = 1; var f = (n int) => { x = x + n; return x }; return f(41) }";
    let diagnostics = compile_closures_source(let_write, "shared-let.at", &[])
        .expect_err("writing a captured let is an immutable write");
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "verify.immutable-write"),
        "unexpected let-write diagnostics: {diagnostics:#?}"
    );
}

// -- PLAN-909 shared mutable captures (T-02 minimal chain) -----------------

/// V-01: the UNCHANGED corpus K06 source runs to 42 through the real
/// source→HIR→lowering→MIR/ABC→AV chain (private writable move env).
#[test]
fn av_closures_909_k06_private_env_runs_42() {
    let source = include_str!("../../../../../../crates/auto-lang/test/cases/ace-m1/src/K06.at");
    run_source(source, 42);
}

/// V-02: a move capture's writes stay PRIVATE — the creator's `base` is
/// never written back (43; a mis-shared cell would yield 63).
#[test]
fn av_closures_909_move_capture_writes_do_not_write_back() {
    run_source(
        "fn main() int { var base int = 1; var f = move (n int) => { base = base + n; return base }; return base + f(20) + f(0) }",
        43,
    );
}

/// V-03: two non-move writers of one `var` share ONE cell (42; two
/// snapshot copies would yield 22).
#[test]
fn av_closures_909_sibling_writers_share_one_cell() {
    run_source(
        "fn main() int { var x int = 1; var f = (n int) => { x = x + n; return x }; var g = (n int) => { x = x + n; return x }; return f(20) + g(0) }",
        42,
    );
}

/// V-03: a reader created BEFORE the writer still observes shared writes
/// (static cell grouping; 82 — a snapshot reader would yield 42).
#[test]
fn av_closures_909_reader_created_first_observes_shared_writes() {
    run_source(
        "fn main() int { var x int = 1; var r = () => { return x }; var w = (n int) => { x = x + n; return x }; return w(40) + r() }",
        82,
    );
}

/// V-03: external assignment and closure writes land in the same cell
/// (182); bool/char cells share the same way.
#[test]
fn av_closures_909_external_writes_and_closure_writes_share() {
    run_source(
        "fn main() int { var x int = 1; var f = () => { return x }; var g = () => { x = 100; return 0 }; x = 41; return f() + x + g() + f() }",
        182,
    );
    run_source(
        "fn main() int { var flag bool = false; var set = () => { flag = true; return 0 }; let a = set(); if flag == true { return 42 }; return a }",
        42,
    );
}

/// V-05: a factory's shared cell escapes with the returned closure — the
/// creator has returned, both calls still mutate one live cell (42).
#[test]
fn av_closures_909_factory_cell_escapes_with_closure() {
    run_source(
        "fn make() fn(int) int { var count int = 1; var inc = (n int) => { count = count + n; return count }; return inc } fn main() int { let f = make(); return f(20) + f(0) }",
        42,
    );
}

/// V-06: an explicit clone deep-copies the cell — original and copy never
/// write back to each other (44; a shared-cell clone would yield 65).
#[test]
fn av_closures_909_clone_detaches_the_cell() {
    run_source(
        "fn main() int { var x int = 1; var f = (n int) => { x = x + n; return x }; let g = f.clone(); let a = f(20); let b = g(1); let c = f(0); return a + b + c }",
        44,
    );
}

/// V-04: each factory ACTIVATION owns an independent cell (42; a single
/// global cell would yield 63).
#[test]
fn av_closures_909_independent_activations_own_independent_cells() {
    run_source(
        "fn make() fn(int) int { var c int = 0; var inc = (n int) => { c = c + n; return c }; return inc } fn main() int { let a = make(); let b = make(); let x = a(20); let y = b(1); let z = a(1); return x + y + z }",
        42,
    );
}

/// V-04: a nested lambda writes the grand-outer `var` THROUGH the enclosing
/// closure's capture — one cell across three lexical levels (82; a snapshot
/// at any level would yield 42).
#[test]
fn av_closures_909_nested_lexical_write_reaches_outer_cell() {
    run_source(
        "fn main() int { var x int = 1; var outer = () => { var inner = (n int) => { x = x + n; return x }; return inner(40) }; return outer() + x }",
        82,
    );
}

/// V-04: the shared group is STATIC — a writer in an untaken branch still
/// makes the binding a cell, so the reader created first observes the
/// external write (41; a snapshot reader would yield 1), and the
/// initializer runs exactly once.
#[test]
fn av_closures_909_writer_in_untaken_branch_still_shares() {
    run_source(
        "fn main() int { var x int = 1; let r = () => { return x }; if 1 > 2 { let w = () => { x = 100; return 0 }; let t = w() }; x = 41; return r() }",
        41,
    );
}

/// V-04: `let` and `var` neighbours — the read-only `let` stays a 865
/// snapshot while the written `var` is a shared cell (81).
#[test]
fn av_closures_909_let_and_var_neighbours() {
    run_source(
        "fn main() int { let a int = 1; var b int = 2; let f = () => { return a + b }; var g = () => { b = b + 38; return b }; return g() + f() }",
        81,
    );
}

/// Mixed row: a move capture of a shared cell snapshots the CURRENT value
/// at creation and detaches; the non-move sibling keeps sharing.
#[test]
fn av_closures_909_move_snapshot_of_shared_cell_detaches() {
    // x: cell (w writes it). m is created when x == 1 (snapshot 1, private).
    // w(40) -> x = 41. m(1) -> private 2. Result 41 + 2 + x(41) = 84.
    run_source(
        "fn main() int { var x int = 1; var m = move (n int) => { x = x + n; return x }; var w = (n int) => { x = x + n; return x }; return w(40) + m(1) + x }",
        84,
    );
}

#[test]
fn av_closures_env_drift_refuses_revision_and_preserves_installed_creator() {
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
fn av_closures_body_edit_produces_only_the_changed_function() {
    let source = include_str!("../../../../../../crates/auto-lang/test/cases/ace-m1/src/K01.at");
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
fn av_closures_capture_and_signature_edits_rebuild_exact_units() {
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
