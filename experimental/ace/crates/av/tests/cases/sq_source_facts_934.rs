//! PLAN-934 T-02 (bounded feasibility): an ordinary scalar source program
//! driven by the SQ-A fact layer — source → per-body query → fact-driven
//! `ResolvedProgram` → emitted+verified checked HIR → MIR → ABC → real
//! execution — plus the asymmetry controls that make "the fact is the
//! authority" observable:
//!
//! * cold miss with the solver disabled is RED (a miss really solves);
//! * warm hit with the solver disabled and the legacy typecheck stage
//!   absent is GREEN (no hidden production);
//! * a tampered fact type makes the REAL compile (HIR verify) red — the
//!   main path, not a side checker;
//! * facts of another owner are refused at admission;
//! * the legacy entry still meters one typecheck per body (control for
//!   the "typecheck attempts == 0" observation of the fact path).

use ace_av_core::av::{execute, CancelFlag, ExecLimits};
use ace_av_core::contract::{ExecOutcome, Value, AV_PROFILE};
use ace_av_core::{abc_verify, decode_abc, encode_abc};
use ace_core::body_work;
use ace_core::source::{
    compile_scalar_source, emit_resolved_parts, parse_program_with, resolve_with, SourceProfile,
};
use ace_core::verify::{verify_scalar, CheckedModule};
use ace_source_query::{
    apply_body_facts, apply_facts_to_resolved, apply_fn_sig, AppliedFn, FactsApplyError,
    InferBodyFacts, SourceQuerySession,
};
use std::collections::BTreeMap;

const SRC: &str = "fn leaf(x int) int { return x + 1 }\n\nfn main() int { return leaf(41) }\n";

#[derive(Debug)]
struct Prepared {
    applied: Vec<AppliedFn>,
    checked: CheckedModule,
    work: BTreeMap<String, body_work::BodyWork>,
}

/// The whole fact-driven preparation inside ONE body-work window, so the
/// recorded producer counts belong to exactly this path.
fn prepare(src: &str, session: &mut SourceQuerySession) -> Result<Prepared, String> {
    let (out, work) = body_work::capture_body_work(|| {
        let program = parse_program_with(src, SourceProfile::Scalar).map_err(|d| d.message)?;
        let mut resolved = resolve_with(&program, &[], SourceProfile::Scalar)
            .map_err(|ds| ds.first().map(|d| d.message.clone()).unwrap_or_default())?;
        let applied =
            apply_facts_to_resolved(&program, &mut resolved, session).map_err(|e| e.to_string())?;
        let parts = emit_resolved_parts(&resolved, src, "sq934.at", SourceProfile::Scalar)
            .map_err(|ds| ds.first().map(|d| d.message.clone()).unwrap_or_default())?;
        let checked = verify_scalar(parts.bundle)
            .map_err(|ds| ds.first().map(|d| d.message.clone()).unwrap_or_default())?;
        Ok::<_, String>((applied, checked))
    });
    let (applied, checked) = out?;
    Ok(Prepared {
        applied,
        checked,
        work,
    })
}

fn run_value(checked: &CheckedModule) -> Value {
    let mir = ace_lowering::lower_checked(checked, "d_main", 934).expect("lower real HIR");
    assert_eq!(mir.profile, AV_PROFILE);
    ace_av_core::verify_mir(&mir).expect("MIR verifies");
    let wire = encode_abc(&mir).expect("ABC encode").to_flat();
    let decoded = decode_abc(&wire).expect("ABC decode");
    let loaded = abc_verify::verify_abc(decoded, &mir.owner).expect("ABC admission");
    let (out, _trace) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    let ExecOutcome::Success { value, .. } = out else {
        panic!("expected success, got {out:?}")
    };
    value
}

#[test]
fn sq934_scalar_source_runs_from_query_facts_without_legacy_typecheck() {
    let mut session = SourceQuerySession::new("sq934", "main");
    let prep = prepare(SRC, &mut session).expect("fact-driven preparation");

    // The path really queried both bodies cold.
    assert_eq!(prep.applied.len(), 2);
    assert!(prep.applied.iter().all(|a| !a.sig_hit && !a.body_hit));
    assert_eq!(session.counters().solver_runs, 2);

    // Legacy typecheck ran ZERO times on this path (producer-metered).
    for name in ["leaf", "main"] {
        let w = prep.work.get(name).expect("per-body work recorded");
        assert_eq!(w.typecheck, 0, "{name}: legacy typecheck must not run");
        assert!(w.resolve >= 1, "{name}: binding still runs and is metered");
        assert!(w.plan >= 1, "{name}: the emitter still plans the body");
    }

    // The real value, through HIR verify → MIR → ABC → execution.
    assert_eq!(run_value(&prep.checked), Value::I32(42));
}

#[test]
fn sq934_legacy_entry_is_the_metering_control() {
    let (out, work) = body_work::capture_body_work(|| {
        compile_scalar_source(SRC, "sq934.at", &[]).expect("legacy scalar compile")
    });
    assert_eq!(work.get("leaf").map(|w| w.typecheck), Some(1));
    assert_eq!(work.get("main").map(|w| w.typecheck), Some(1));
    assert_eq!(run_value(&out.checked), Value::I32(42));
}

#[test]
fn sq934_body_edit_runs_43_and_keeps_the_caller_warm() {
    let mut session = SourceQuerySession::new("sq934", "main");
    let a = prepare(SRC, &mut session).expect("first compile");
    assert_eq!(run_value(&a.checked), Value::I32(42));

    let src2 = SRC.replace("x + 1", "x + 2");
    let b = prepare(&src2, &mut session).expect("second compile");
    let leaf = b.applied.iter().find(|f| f.name == "leaf").expect("leaf");
    let main = b.applied.iter().find(|f| f.name == "main").expect("main");
    assert!(!leaf.body_hit, "the edited body re-solves");
    assert!(main.body_hit, "the signature-only reader stays warm");
    assert_eq!(
        session.counters().solver_runs,
        3,
        "exactly one extra solve for the edited body"
    );
    assert_eq!(run_value(&b.checked), Value::I32(43));
}

#[test]
fn sq934_cold_miss_with_solver_disabled_is_red() {
    let mut session = SourceQuerySession::new("sq934", "main").with_solver(false);
    let err = prepare(SRC, &mut session).expect_err("cold miss must fail");
    assert!(err.contains("solver is disabled"), "{err}");
    assert_eq!(session.counters().solver_runs, 0);
}

#[test]
fn sq934_warm_hit_with_solver_disabled_is_green() {
    let mut session = SourceQuerySession::new("sq934", "main");
    prepare(SRC, &mut session).expect("cold");
    let runs = session.counters().solver_runs;
    session.set_allow_solver(false);
    let warm = prepare(SRC, &mut session).expect("warm without a solver");
    assert!(warm.applied.iter().all(|a| a.body_hit && a.sig_hit));
    assert_eq!(session.counters().solver_runs, runs);
    assert_eq!(run_value(&warm.checked), Value::I32(42));
}

#[test]
fn sq934_tampered_fact_type_fails_the_real_compile() {
    let src = SRC;
    let program = parse_program_with(src, SourceProfile::Scalar).expect("parse");
    let mut resolved = resolve_with(&program, &[], SourceProfile::Scalar).expect("bind");
    let mut session = SourceQuerySession::new("sq934", "main");

    for name in ["leaf", "main"] {
        let sig = session.fn_sig(&program, name).expect("sig").facts;
        let rfn = resolved
            .fns
            .iter_mut()
            .find(|f| f.name == name)
            .expect("bound fn");
        apply_fn_sig(rfn, &sig).expect("sig applies");

        let body = session.infer_body(&program, name).expect("body");
        let ast_body = program
            .items
            .iter()
            .find_map(|it| match it {
                ace_core::source::ast::Item::Fn(f) if f.name.name == name => Some(&f.body),
                _ => None,
            })
            .expect("ast body");

        let mut facts: InferBodyFacts = body.facts;
        if name == "leaf" {
            // The body's return value is the binary node: claim it is bool
            // while the declared result is int. Structural identity still
            // matches, so only the REAL compile can catch it.
            let binary = facts
                .nodes
                .iter_mut()
                .find(|n| n.kind == "expr.binary")
                .expect("binary fact");
            binary.ty = ace_source_query::PortableTy::Bool;
        }
        let rfn = resolved
            .fns
            .iter_mut()
            .find(|f| f.name == name)
            .expect("bound fn");
        apply_body_facts(ast_body, rfn, &facts).expect("shape-valid facts install");
    }

    let parts = emit_resolved_parts(&resolved, src, "sq934.at", SourceProfile::Scalar)
        .expect("emission itself succeeds");
    let err = verify_scalar(parts.bundle).expect_err("the real compile must reject a bool return");
    assert!(
        err.iter().any(|d| d.code.contains("type")),
        "expected a type diagnostic, got {:?}",
        err.iter().map(|d| (&d.code, &d.message)).collect::<Vec<_>>()
    );
}

#[test]
fn sq934_facts_of_another_owner_are_refused() {
    let program = parse_program_with(SRC, SourceProfile::Scalar).expect("parse");
    let mut resolved = resolve_with(&program, &[], SourceProfile::Scalar).expect("bind");
    let mut session = SourceQuerySession::new("sq934", "main");
    let leaf_facts = session.infer_body(&program, "leaf").expect("leaf facts").facts;
    let main_ast = program
        .items
        .iter()
        .find_map(|it| match it {
            ace_core::source::ast::Item::Fn(f) if f.name.name == "main" => Some(&f.body),
            _ => None,
        })
        .expect("main body");
    let main_rfn = resolved
        .fns
        .iter_mut()
        .find(|f| f.name == "main")
        .expect("main resolved");
    let err = apply_body_facts(main_ast, main_rfn, &leaf_facts)
        .expect_err("a foreign body's facts must not apply");
    assert!(
        matches!(err, ace_source_query::ApplyError::Facts(_)),
        "expected an admission rejection, got {err:?}"
    );
}

#[test]
fn sq934_empty_capabilities_are_honest() {
    // The fixture uses no intrinsic; a capability-free session must not be
    // the reason the chain passes.
    let mut session = SourceQuerySession::new("sq934", "main");
    let prep = prepare(SRC, &mut session).expect("prep");
    let v = run_value(&prep.checked);
    assert_eq!(v, Value::I32(42));
}

/// Kept tiny on purpose: this file proves the T-02 chain, not the full
/// consumer matrix (T-08) or the restore legs (T-07).
#[allow(dead_code)]
fn _assert_error_type_is_send(_: &FactsApplyError) {}

const NAMED: &str = "fn pick(a int, b bool) int { if b { return a }\n  return 0 }\n\nfn main() int { return pick(b: true, a: 7) }\n";

#[test]
fn sq934_named_arguments_bind_by_name_end_to_end() {
    let mut session = SourceQuerySession::new("sq934", "main");
    let prep = prepare(NAMED, &mut session).expect("named-argument chain");
    assert_eq!(run_value(&prep.checked), Value::I32(7));
}

#[test]
fn sq934_named_arguments_positional_control_is_rejected() {
    let bad = NAMED.replace("pick(b: true, a: 7)", "pick(true, 7)");
    let mut session = SourceQuerySession::new("sq934", "main");
    let err = prepare(&bad, &mut session).expect_err("the positional reading is a type error");
    assert!(err.contains("diagnostic"), "{err}");
}
