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
use ace_core::source::resolve::resolve_with_supply;
use ace_core::source::{
    compile_scalar_source, emit_resolved_parts, parse_program_with, resolve_with, SourceProfile,
};
use ace_core::verify::{verify_scalar, CheckedModule};
use ace_core::type_synth::TypeWork;
use ace_source_query::{
    apply_body_facts, bind_with_facts, AppliedFn, BodyTypeSupply, FactsApplyError, InferBodyFacts,
    PortableTy, SourceQuerySession,
};
use std::collections::BTreeMap;

const SRC: &str = "fn leaf(x int) int { return x + 1 }\n\nfn main() int { return leaf(41) }\n";

#[derive(Debug)]
struct Prepared {
    applied: Vec<AppliedFn>,
    checked: CheckedModule,
    work: BTreeMap<String, body_work::BodyWork>,
    type_work: BTreeMap<String, TypeWork>,
}

/// The whole fact-driven preparation inside ONE body-work window, so the
/// recorded producer counts belong to exactly this path. The type-work
/// window nests inside it and records where each body's types came from.
fn prepare(src: &str, session: &mut SourceQuerySession) -> Result<Prepared, String> {
    let (out, work) = body_work::capture_body_work(|| {
        let (inner, type_work) = ace_core::type_synth::capture(
            || -> Result<(Vec<AppliedFn>, CheckedModule), String> {
                let program =
                    parse_program_with(src, SourceProfile::Scalar).map_err(|d| d.message)?;
                let (resolved, applied, _supply) = bind_with_facts(
                    &program,
                    &[],
                    "sq934",
                    "main",
                    session,
                    SourceProfile::Scalar,
                )
                .map_err(|e| e.to_string())?;
                let parts = emit_resolved_parts(&resolved, src, "sq934.at", SourceProfile::Scalar)
                    .map_err(|ds| ds.first().map(|d| d.message.clone()).unwrap_or_default())?;
                let checked = verify_scalar(parts.bundle)
                    .map_err(|ds| ds.first().map(|d| d.message.clone()).unwrap_or_default())?;
                Ok((applied, checked))
            },
        );
        inner.map(|(applied, checked)| (applied, checked, type_work))
    });
    let (applied, checked, type_work) = out?;
    Ok(Prepared {
        applied,
        checked,
        work,
        type_work,
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
        // PLAN-934 T-04: the types came from the fact supply, never from
        // the resolver's own synthesis.
        let tw = prep.type_work.get(name).expect("type work recorded");
        assert_eq!(tw.synth, 0, "{name}: no legacy synthesis on the fact path");
        assert!(
            tw.fact_checks > 0,
            "{name}: every node's type was supplied and checked"
        );
        assert_eq!(tw.fact_missing, 0);
    }

    // The real value, through HIR verify → MIR → ABC → execution.
    assert_eq!(run_value(&prep.checked), Value::I32(42));
}

#[test]
fn sq934_legacy_entry_is_the_metering_control() {
    let (out, work) = body_work::capture_body_work(|| {
        let (compiled, type_work) = ace_core::type_synth::capture(|| {
            compile_scalar_source(SRC, "sq934.at", &[]).expect("legacy scalar compile")
        });
        (compiled, type_work)
    });
    let (out, type_work) = out;
    assert_eq!(work.get("leaf").map(|w| w.typecheck), Some(1));
    assert_eq!(work.get("main").map(|w| w.typecheck), Some(1));
    // The SAME window on the legacy entry records real synthesis, so the
    // fact path's zero is an observation, not a dead counter.
    assert!(type_work.get("leaf").map(|w| w.synth).unwrap_or(0) > 0);
    assert!(type_work.get("main").map(|w| w.synth).unwrap_or(0) > 0);
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
    let mut session = SourceQuerySession::new("sq934", "main");
    let mut supply = BodyTypeSupply::new();

    for name in ["leaf", "main"] {
        let ast_body = program
            .items
            .iter()
            .find_map(|it| match it {
                ace_core::source::ast::Item::Fn(f) if f.name.name == name => Some(&f.body),
                _ => None,
            })
            .expect("ast body");
        let mut facts: InferBodyFacts = session.infer_body(&program, name).expect("body").facts;
        if name == "leaf" {
            // The body's return value is the binary node: claim it is bool
            // while the declared result is int. Structural identity still
            // matches, so only core's own check can catch it.
            let binary = facts
                .nodes
                .iter_mut()
                .find(|n| n.kind == "expr.binary")
                .expect("binary fact");
            binary.ty = PortableTy::Bool;
        }
        supply
            .add_body(ast_body, &facts)
            .expect("shape-valid page admits");
    }

    // PLAN-934 T-04: the bad fact is rejected at BIND, before any HIR.
    let err = match resolve_with_supply(&program, &[], SourceProfile::Scalar, &supply) {
        Ok(_) => panic!("a bool fact on an i32 node must fail the bind"),
        Err(e) => e,
    };
    assert!(
        err.iter().any(|d| d.code == "source.fact-mismatch"),
        "expected a bind-stage fact mismatch, got {:?}",
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

#[test]
fn sq934_source_map_drift_keeps_facts_and_executes() {
    let drifted = format!("\n\n\n{}", SRC);
    let mut session = SourceQuerySession::new("sq934", "main");
    let cold = prepare(SRC, &mut session).expect("cold");
    assert_eq!(run_value(&cold.checked), Value::I32(42));

    let warm = prepare(&drifted, &mut session).expect("drifted source");
    assert!(
        warm.applied.iter().all(|a| a.body_hit && a.sig_hit),
        "prefix drift is a source-map change, not a semantic one"
    );
    assert_eq!(run_value(&warm.checked), Value::I32(42));
}

#[test]
fn sq934_restore_bind_executes_from_pages_only() {
    let program = parse_program_with(SRC, SourceProfile::Scalar).expect("parse");
    let mut producer = SourceQuerySession::new("sq934", "main");
    let mut pages: BTreeMap<String, InferBodyFacts> = BTreeMap::new();
    for name in ["leaf", "main"] {
        pages.insert(
            name.to_string(),
            producer.infer_body(&program, name).expect("facts").facts,
        );
    }
    // Restore lane: pages only, no solver involved.
    let (resolved, applied, supply) = ace_source_query::restore_bind(
        &program,
        &[],
        "sq934",
        "main",
        &pages,
        SourceProfile::Scalar,
    )
    .expect("restore bind from pages");
    assert_eq!(applied.len(), 2);
    assert_eq!(supply.admitted_bodies(), 2);
    let parts = emit_resolved_parts(&resolved, SRC, "sq934.at", SourceProfile::Scalar)
        .expect("emit restored program");
    let checked = verify_scalar(parts.bundle).expect("restored HIR verifies");
    assert_eq!(run_value(&checked), Value::I32(42));
}

#[test]
fn sq934_shared_consumer_entry_feeds_real_execution_without_typecheck() {
    let mut session = SourceQuerySession::new("sq934", "main");
    let (compiled, work) = body_work::capture_body_work(|| {
        ace_source_query::compile_via_facts(
            SRC,
            "sq934.at",
            &[],
            "sq934",
            "main",
            &mut session,
            SourceProfile::Scalar,
        )
    });
    let compiled = compiled.expect("shared fact entry");
    assert_eq!(compiled.applied.len(), 2);
    for name in ["leaf", "main"] {
        assert_eq!(
            work.get(name).map(|w| w.typecheck),
            Some(0),
            "{name}: the shared consumer entry never runs the legacy typecheck"
        );
    }
    assert_eq!(run_value(&compiled.checked), Value::I32(42));
}
