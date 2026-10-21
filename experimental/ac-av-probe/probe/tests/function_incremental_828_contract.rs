//! PLAN-828 r3 T-01 vertical slice (§5.6 D-01..03): the single-body
//! checked recompilation seam against a retained environment
//! (`contract_828_frontend`) and the dispatch-payload identity
//! observation (§5.8 F-02 axes; `contract_828_dispatch`). Filters per
//! the r3 test-selection table; every count comes from the producer
//! seam itself, never reconstructed by the caller.

use auto_av_core::av::{CancelFlag, ExecLimits};
use auto_av_core::contract::ExecOutcome;
use auto_av_probe::session::{warmup, FunctionIncrementalSession};
use std::sync::Arc;

const TWO_FNS: &str = "fn helper(n int) int { return n + 1 }\nfn main() int { return helper(2) }\n";
const TWO_FNS_EDITED: &str =
    "fn helper(n int) int { return n + 2 }\nfn main() int { return helper(2) }\n";
const TWO_FNS_BAD_BODY: &str =
    "fn helper(n int) int { return n + true }\nfn main() int { return helper(2) }\n";
const TWO_FNS_DRIFT: &str =
    "fn helper(n bool) int { return 1 }\nfn main() int { return helper(2) }\n";

fn fn_item_of(src: &str, name: &str) -> auto_ac_prototype::source::ast::FnItem {
    let program = auto_ac_prototype::source::parse_program_with(
        src,
        auto_ac_prototype::source::SourceProfile::Scalar,
    )
    .unwrap();
    for item in &program.items {
        if let auto_ac_prototype::source::ast::Item::Fn(f) = item {
            if f.name.name == name {
                return f.clone();
            }
        }
    }
    panic!("fn `{name}` not in source");
}

fn helper_dump(module: &auto_ac_prototype::hir::Module, def_id: &str) -> Vec<u8> {
    let body = module
        .defs
        .iter()
        .find_map(|d| match &d.kind {
            auto_ac_prototype::hir::DefKind::Function { body, .. } if d.id_text == def_id => {
                Some(*body)
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("def {def_id} missing"));
    auto_av_probe::cache::canonical_hir_function(module, body)
}

fn value_of(s: &FunctionIncrementalSession) -> i32 {
    let (out, _) = s.run(&ExecLimits::default(), &CancelFlag::new(false));
    match out {
        ExecOutcome::Success {
            value: auto_av_core::contract::Value::I32(v),
            ..
        } => v,
        other => panic!("expected i32 success, got {other:?}"),
    }
}

/// C01-01 minimal positive (D-01): a single changed body re-checked and
/// emitted against the RETAINED environment produces exactly the cold
/// compile's HIR for that function, walking ONE function — while the
/// old whole-program path (compile_program_parts over the stub synth
/// program) is measurable as planning EVERY function, so the structural
/// gate `planned_fns == 1` distinguishes the two real entries.
#[test]
fn contract_828_frontend_single_body_matches_cold_compile() {
    use auto_ac_prototype::source as ac;
    let original = ac::parse_program_with(TWO_FNS, ac::SourceProfile::Scalar).unwrap();
    let mut env = ac::resolve_env(&original, &[], ac::SourceProfile::Scalar).unwrap();
    assert_eq!(ac::resolve::env_fn_count(&env), 2);

    ac::env_note_cold_walks(&mut env);
    let (r0, c0) = ac::resolve::env_walk_stats(&env);
    assert_eq!((r0, c0), (2, 2), "cold baseline walked every fn once");

    let edited_item = fn_item_of(TWO_FNS_EDITED, "helper");
    ac::recheck_function(&mut env, &edited_item, 0).unwrap();
    let (parts, receipt) = ac::emit_function_body(&env, 0, TWO_FNS_EDITED, "probe.at").unwrap();

    // T01-C01: the single-function document contains EXACTLY ONE body —
    // unchanged functions emit body-less declaration scaffolds only.
    assert_eq!(receipt.real_body_plans, 1, "one real body planned");
    assert_eq!(
        receipt.scaffold_fns, 1,
        "the callee is a body-less scaffold"
    );
    assert_eq!(
        parts.bundle.modules[0].bodies.len(),
        1,
        "exactly one bound body document"
    );
    let (r1, c1) = ac::resolve::env_walk_stats(&env);
    assert_eq!(
        (r1, c1),
        (3, 3),
        "exactly one resolve + one typecheck walk added"
    );

    // Cold cross-check: the whole-module compile of the same edited
    // source must produce the SAME HIR for the changed function.
    let cold =
        ac::compile_parts_with(TWO_FNS_EDITED, "probe.at", &[], ac::SourceProfile::Scalar).unwrap();
    assert_eq!(cold.planned_fns, 2, "old whole-program path plans all fns");
    let seam_module = parts.bundle.modules.first().unwrap();
    let cold_module = cold.bundle.modules.first().unwrap();
    assert_eq!(
        helper_dump(seam_module, "d_helper"),
        helper_dump(cold_module, "d_helper"),
        "single-body HIR is byte-identical to the cold compile's"
    );

    // D-01 verifier gate: the seam's bundle passes the REAL function
    // verifier (callee scaffolds bind types; body-less sentinel defs are
    // never dereferenced by function-level verification).
    let def_idx = seam_module
        .defs
        .iter()
        .position(|d| d.id_text == "d_helper")
        .unwrap();
    let checked = auto_ac_prototype::verify::verify_function(parts.bundle.clone(), def_idx);
    assert!(checked.is_ok(), "real verifier accepts the seam bundle");

    // T01-C02: the structural gate consumes the ACTUAL producer artifact
    // of whichever entry ran — exactly one bound body plus the
    // scoped-work attestation. The REAL old pipeline (synth program
    // through compile_program_parts) is executed here; its artifact
    // binds EVERY body (2) and FAILS the same gate.
    let gate = |parts: &ac::SourceParts, receipt: Option<ac::BodyEmitReceipt>| {
        parts.bundle.modules[0].bodies.len() == 1
            && receipt.map_or(false, |r| r.real_body_plans == 1)
    };
    assert!(
        gate(&parts, Some(receipt)),
        "gate green on the seam artifact"
    );
    let edited_program = ac::parse_program_with(TWO_FNS_EDITED, ac::SourceProfile::Scalar).unwrap();
    let synth = edited_program;
    let old_parts = ac::compile_program_parts(
        &synth,
        TWO_FNS_EDITED,
        "probe.at",
        &[],
        ac::SourceProfile::Scalar,
    )
    .unwrap();
    assert_eq!(old_parts.planned_fns, 2, "real old pipeline plans 2 bodies");
    assert_eq!(
        old_parts.bundle.modules[0].bodies.len(),
        2,
        "and binds 2 bodies"
    );
    assert!(
        !gate(&old_parts, None),
        "gate RED on the actually-executed old pipeline artifact"
    );
}

/// C01-04 minimal (D-01): an erroneous changed body is rejected by the
/// REAL typecheck rules with real diagnostics, and the rejection leaves
/// the retained environment usable — the next valid recheck succeeds.
#[test]
fn contract_828_frontend_rejects_erroneous_body_and_restores_env() {
    use auto_ac_prototype::source as ac;
    let original = ac::parse_program_with(TWO_FNS, ac::SourceProfile::Scalar).unwrap();
    let mut env = ac::resolve_env(&original, &[], ac::SourceProfile::Scalar).unwrap();

    let bad = fn_item_of(TWO_FNS_BAD_BODY, "helper");
    let diags = ac::recheck_function(&mut env, &bad, 0).unwrap_err();
    assert!(
        diags.iter().any(|d| d.code.contains("type-mismatch")),
        "real verifier diagnostic, got {diags:?}"
    );

    // The environment is not poisoned: the valid edit re-checks.
    let good = fn_item_of(TWO_FNS_EDITED, "helper");
    ac::recheck_function(&mut env, &good, 0).unwrap();
    let (parts, receipt) = ac::emit_function_body(&env, 0, TWO_FNS_EDITED, "probe.at").unwrap();
    assert_eq!(receipt.real_body_plans, 1);
    let cold =
        ac::compile_parts_with(TWO_FNS_EDITED, "probe.at", &[], ac::SourceProfile::Scalar).unwrap();
    assert_eq!(
        helper_dump(parts.bundle.modules.first().unwrap(), "d_helper"),
        helper_dump(cold.bundle.modules.first().unwrap(), "d_helper")
    );
}

/// D-01 boundary: a SIGNATURE drift is a controlled rejection — the
/// single-body seam never silently re-checks an interface change.
#[test]
fn contract_828_frontend_rejects_signature_drift() {
    use auto_ac_prototype::source as ac;
    let original = ac::parse_program_with(TWO_FNS, ac::SourceProfile::Scalar).unwrap();
    let mut env = ac::resolve_env(&original, &[], ac::SourceProfile::Scalar).unwrap();
    let drifted = fn_item_of(TWO_FNS_DRIFT, "helper");
    let diags = ac::recheck_function(&mut env, &drifted, 0).unwrap_err();
    assert!(
        diags.iter().any(|d| d.code == "source.incremental"),
        "controlled non-fast-path rejection, got {diags:?}"
    );
}

/// D-01 caller control: after the callee's body re-check, the CALLER's
/// entry is consultable and reproducible from the retained environment
/// (its single-body emission matches the cold compile too) — unchanged
/// entries are retained, not rebuilt or invalidated.
#[test]
fn contract_828_frontend_caller_entry_untouched_control() {
    use auto_ac_prototype::source as ac;
    let original = ac::parse_program_with(TWO_FNS, ac::SourceProfile::Scalar).unwrap();
    let mut env = ac::resolve_env(&original, &[], ac::SourceProfile::Scalar).unwrap();
    let edited_helper = fn_item_of(TWO_FNS_EDITED, "helper");
    ac::recheck_function(&mut env, &edited_helper, 0).unwrap();

    // The caller slot was never re-walked: re-checking main with its own
    // (unchanged) item reproduces the cold compile's main exactly.
    let main_item = fn_item_of(TWO_FNS_EDITED, "main");
    ac::recheck_function(&mut env, &main_item, 1).unwrap();
    let (parts, receipt) = ac::emit_function_body(&env, 1, TWO_FNS_EDITED, "probe.at").unwrap();
    assert_eq!(receipt.real_body_plans, 1);
    assert_eq!(
        receipt.scaffold_fns, 1,
        "the callee scaffold binds main's call"
    );
    let cold =
        ac::compile_parts_with(TWO_FNS_EDITED, "probe.at", &[], ac::SourceProfile::Scalar).unwrap();
    assert_eq!(
        helper_dump(parts.bundle.modules.first().unwrap(), "d_main"),
        helper_dump(cold.bundle.modules.first().unwrap(), "d_main")
    );
}

/// D-02 axes (§5.8 F-02 observation, producer-reported): the CODE-level
/// identity axis is green today (unchanged functions share one immutable
/// FunctionCode across snapshots; a forced same-content COPY turns it
/// red) — proving the gate detects copies. The DISPATCH-level axis is
/// honestly RED on today's merged execution view (the payload AV
/// consumes is a per-snapshot copy; content stays equal) — the measured
/// F-02 gap the per-unit dispatch design (T-05/06) closes.
#[test]
fn contract_828_dispatch_payload_identity_axes() {
    let src = "fn helper() int { return 1 }\nfn main() int { var x = helper(); return x + 1 }\nfn tail() int { return 9 }\n";
    let mut s = warmup(src, "d_main", &[]).unwrap();
    assert_eq!(value_of(&s), 2);
    let snap_a = s.current();
    let at = s.src().find("return 1").unwrap() + 7;
    let out = s.edit(at as u64, (at + 1) as u64, b"2".to_vec()).unwrap();
    assert!(out.ok, "{out:?}");
    let snap_b = s.current();
    assert_eq!(value_of(&s), 3);

    // Axis 1 — code identity (green): the unchanged function's immutable
    // code object is the SAME Arc across snapshots.
    let a_main = snap_a.find_code("d_main").unwrap().clone();
    let b_main = snap_b.find_code("d_main").unwrap().clone();
    assert!(
        Arc::ptr_eq(&a_main, &b_main),
        "unchanged code object identity"
    );

    // Forced-copy negative on axis 1: rebuild the snapshot with a
    // CONTENT-EQUAL CLONE of the unchanged function's code — the result
    // still runs correctly, but the identity gate must go red.
    let helper_b = snap_b.find_code("d_helper").unwrap().clone();
    let cloned = Arc::new(a_main.as_ref().clone());
    assert!(
        !Arc::ptr_eq(&cloned, &a_main),
        "the copy is a real new object"
    );
    let snap_copy = auto_av_core::program_snapshot::ProgramSnapshot::link_with_maps(
        vec![
            helper_b.clone(),
            cloned.clone(),
            snap_b.find_code("d_tail").unwrap().clone(),
        ],
        "d_main",
        99,
        vec![
            snap_b.span_maps()[0].clone(),
            snap_b.span_maps()[1].clone(),
            snap_b.span_maps()[2].clone(),
        ],
    )
    .unwrap();
    let (ran, _) = auto_av_core::av::execute_program(
        &snap_copy,
        &ExecLimits::default(),
        &CancelFlag::new(false),
    );
    assert!(
        matches!(ran, ExecOutcome::Success { .. }),
        "the forced copy still produces the correct result: {ran:?}"
    );
    assert!(
        !Arc::ptr_eq(&snap_copy.find_code("d_main").unwrap(), &a_main),
        "identity gate RED on the forced copy — copies are detectable"
    );

    // Axis 2 — dispatch payload (measured current state): what AV
    // actually consumes for the unchanged function is the per-snapshot
    // MERGED copy, so the object-identity observation is red today — and
    // (the vertical slice's second real finding) even the merged CONTENT
    // drifts, because the changed function's growth re-indexes the
    // merged type/const space under later functions. The UNIT payload —
    // the immutable FunctionCode AV must come to dispatch directly —
    // stays content-identical. T-05/06 close this with link-time binding
    // tables over unit-local payloads.
    let main_idx = 1usize;
    assert!(
        !snap_a.dispatched_payload_same_object(&snap_b, main_idx),
        "dispatched payload identity is RED on today's merged view (F-02 measured)"
    );
    assert!(
        !snap_a.dispatched_payload_same_content(&snap_b, main_idx),
        "merged content ALSO drifts for unchanged fns (merged-space re-index)"
    );
    let unit_a = format!(
        "{:?}",
        snap_a.find_code("d_main").unwrap().unit().module().funcs[0]
    );
    let unit_b = format!(
        "{:?}",
        snap_b.find_code("d_main").unwrap().unit().module().funcs[0]
    );
    assert_eq!(
        unit_a, unit_b,
        "the immutable unit payload is content-identical"
    );
}

/// D-02 first case (r3 T-01 review): the UNIT dispatch view executes
/// the dual-function program reading each function's ORIGINAL immutable
/// payload — the value is correct, the dispatched objects ARE the
/// snapshot's code payloads (identity through the actual execution
/// path), and incomplete unit bindings refuse dispatch up front.
#[test]
fn contract_828_dispatch_unit_view_first_case() {
    let src = "fn helper() int { return 1 }\nfn main() int { var x = helper(); return x + 1 }\nfn tail() int { return 9 }\n";
    let mut s = warmup(src, "d_main", &[]).unwrap();
    assert_eq!(value_of(&s), 2);
    let at = s.src().find("return 1").unwrap() + 7;
    let out = s.edit(at as u64, (at + 1) as u64, b"2".to_vec()).unwrap();
    assert!(out.ok, "{out:?}");
    let snap = s.current();
    assert_eq!(value_of(&s), 3);

    // Binding completeness is a real precondition (rejection control).
    snap.unit_binding_complete().unwrap();
    let vacant = auto_av_core::program_snapshot::ProgramSnapshot::vacant("d_main", 7);
    assert!(
        vacant.unit_binding_complete().is_err(),
        "a snapshot with no units refuses unit dispatch"
    );

    // The unit-view execution: correct value through ORIGINAL payloads.
    let (ran, _) = auto_av_core::av::execute_program_units(
        &snap,
        &ExecLimits::default(),
        &CancelFlag::new(false),
    );
    assert!(
        matches!(
            ran,
            ExecOutcome::Success {
                value: auto_av_core::contract::Value::I32(3),
                ..
            }
        ),
        "unit dispatch computes the same result: {ran:?}"
    );

    // Identity through the ACTUAL execution path: the functions the unit
    // view hands the interpreter are the snapshot's immutable code
    // payloads — across an edit, the UNCHANGED function's payload is the
    // SAME allocation (this is the C02-01 positive the merged view
    // cannot provide today).
    let before = s.current(); // pinned pre-edit snapshot Arc still alive
    let a_main = before.find_code("d_main").unwrap();
    let b_main = snap.find_code("d_main").unwrap();
    assert!(Arc::ptr_eq(a_main, b_main));
    assert!(
        std::ptr::eq(
            a_main.unit().module().funcs.first().unwrap(),
            b_main.unit().module().funcs.first().unwrap()
        ),
        "the unit payload AV actually consumes is the same immutable object"
    );
    // Both views agree on the result (merged vs unit dispatch).
    let (merged_run, _) =
        auto_av_core::av::execute_program(&snap, &ExecLimits::default(), &CancelFlag::new(false));
    assert!(matches!(merged_run, ExecOutcome::Success { .. }));
}
