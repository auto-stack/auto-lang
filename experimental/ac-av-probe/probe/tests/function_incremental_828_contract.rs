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
    // Pinned BEFORE the edit — the identity assertions below compare across
    // a REAL revision boundary (T-01 r3 review: capturing `current()` after
    // the edit compared the snapshot with itself).
    let before = s.current();
    let at = s.src().find("return 1").unwrap() + 7;
    let out = s.edit(at as u64, (at + 1) as u64, b"2".to_vec()).unwrap();
    assert!(out.ok, "{out:?}");
    let snap = s.current();
    assert_eq!(value_of(&s), 3);
    assert!(
        !Arc::ptr_eq(&before, &snap),
        "the edit produced a new snapshot root"
    );

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

    // Identity through the ACTUAL execution path, across the revision
    // boundary: the UNCHANGED function's code object and the unit payload
    // the unit view hands the interpreter are the SAME immutable objects
    // before and after the edit (the C02-01 positive the merged view
    // cannot provide today).
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

    // Copy-execution negative (adopted from the r3 T-01 review control):
    // rebuild a snapshot from a content-equal CLONE of the unchanged
    // function's unit. Execution still computes the correct value, but the
    // dispatched payload is a DIFFERENT allocation — copies are detectable
    // in the execution path, not only by wrapper identity.
    let copied = Arc::new(b_main.as_ref().clone());
    assert!(!Arc::ptr_eq(&copied, b_main));
    let copy_snap = auto_av_core::program_snapshot::ProgramSnapshot::link(
        vec![
            snap.find_code("d_tail").unwrap().clone(),
            snap.find_code("d_helper").unwrap().clone(),
            copied.clone(),
        ],
        "d_main",
        snap.revision(),
    )
    .unwrap();
    assert!(
        !std::ptr::eq(
            b_main.unit().module().funcs.first().unwrap(),
            copied.unit().module().funcs.first().unwrap()
        ),
        "the forced copy is a real new payload object"
    );
    let (copy_run, _) = auto_av_core::av::execute_program_units(
        &copy_snap,
        &ExecLimits::default(),
        &CancelFlag::new(false),
    );
    assert!(
        matches!(copy_run, ExecOutcome::Success { .. }),
        "the copy still computes correctly: {copy_run:?}"
    );

    // Both views agree on the result (merged vs unit dispatch).
    let (merged_run, _) =
        auto_av_core::av::execute_program(&snap, &ExecLimits::default(), &CancelFlag::new(false));
    assert!(matches!(merged_run, ExecOutcome::Success { .. }));
}

/// D-02 (adopted from the r3 T-01 review control): the only way a public
/// binding comes to exist is snapshot CONSTRUCTION — a missing import
/// target or a real signature mismatch is refused when the snapshot is
/// linked, before any dispatch.
#[test]
fn contract_828_dispatch_real_link_rejections() {
    use auto_av_core::program_snapshot::ProgramSnapshot;
    use auto_av_probe::session::warmup as warm;
    let a = warm(
        "fn helper() int { return 1 }\nfn main() int { return helper() }\n",
        "d_main",
        &[],
    )
    .unwrap()
    .current();
    let b = warm(
        "fn helper() bool { return true }\nfn main() int { return 1 }\n",
        "d_main",
        &[],
    )
    .unwrap()
    .current();
    let main = a.find_code("d_main").unwrap().clone();
    // main's import `fn:helper` resolves to no function in this unit set.
    assert!(ProgramSnapshot::link(vec![main.clone()], "d_main", 1)
        .unwrap_err()
        .contains("does not resolve"));
    // helper's REAL unit type (bool) mismatches main's call-site type (int).
    let helper = b.find_code("d_helper").unwrap().clone();
    assert!(ProgramSnapshot::link(vec![helper, main], "d_main", 1)
        .unwrap_err()
        .contains("signature mismatch"));
}

// ---------------------------------------------------------------------------
// T01-C04 admission controls (`contract_828_admission`): the schema
// relaxation that lets single-body documents BIND body-less callee
// scaffolds must not open the WHOLE-MODULE credential. A document whose
// function carries no body is a controlled verify rejection in every
// profile — mixed documents, uncalled body-less declarations included —
// while the function-level seam (single-body documents through
// verify_function) stays legal. Adopted from the r3 T-01 review's
// `boundary_missing_body_*` inputs and controls.
// ---------------------------------------------------------------------------

fn verify_profile(
    b: auto_ac_prototype::hir::Bundle,
    p: auto_ac_prototype::source::SourceProfile,
) -> Result<auto_ac_prototype::verify::CheckedModule, Vec<auto_ac_prototype::atom_text::Diagnostic>>
{
    use auto_ac_prototype::source::SourceProfile;
    use auto_ac_prototype::verify;
    match p {
        SourceProfile::Core => verify::verify(b),
        SourceProfile::Scalar => verify::verify_scalar(b),
        SourceProfile::Text => verify::verify_text(b),
        SourceProfile::Record => verify::verify_record(b),
        SourceProfile::TextRecord => verify::verify_text_record(b),
    }
}

/// Remove a function's `; body: "<id>"` declaration field and its
/// `body <id> { ... }` section from a bound Atom document — every other
/// identity/type/declaration byte stays (the review's exact mutation).
fn strip_body_field_and_section(text: &str, body_id: &str) -> String {
    let mut text = text.replace(&format!("; body: \"{body_id}\""), "");
    let begin = text.find(&format!("body {body_id} {{")).unwrap();
    let open = begin + text[begin..].find('{').unwrap();
    let mut depth = 0i32;
    let mut end = None;
    for (off, c) in text[open..].char_indices() {
        if c == '{' {
            depth += 1;
        }
        if c == '}' {
            depth -= 1;
            if depth == 0 {
                end = Some(open + off + 1);
                break;
            }
        }
    }
    text.replace_range(begin..end.unwrap(), "");
    text
}

#[test]
fn contract_828_admission_missing_body_rejected_all_profiles() {
    use auto_ac_prototype::source::SourceProfile;
    let old_schemas = [
        (
            SourceProfile::Core,
            include_str!("fixtures_828/old-core-i32.atom"),
        ),
        (
            SourceProfile::Scalar,
            include_str!("fixtures_828/old-scalar-i32.atom"),
        ),
        (
            SourceProfile::Text,
            include_str!("fixtures_828/old-text.atom"),
        ),
        (
            SourceProfile::Record,
            include_str!("fixtures_828/old-record.atom"),
        ),
        (
            SourceProfile::TextRecord,
            include_str!("fixtures_828/old-text-record.atom"),
        ),
    ];
    for (profile, old_schema) in old_schemas {
        let parts = auto_ac_prototype::source::compile_parts_with(
            "fn main() int { return 1 }",
            "probe.at",
            &[],
            profile,
        )
        .unwrap();
        // Legal complete document control: the FULL original text binds
        // and whole-module-verifies in this profile.
        let valid = auto_ac_prototype::bind_document(&parts.atom_text).unwrap();
        assert!(
            verify_profile(valid, profile).is_ok(),
            "{profile:?}: legal complete document control"
        );

        let text = strip_body_field_and_section(&parts.atom_text, "b_main");
        assert_ne!(text, parts.atom_text, "removed exactly the body");

        // Pre-repair descriptor control: the SAME text is rejected by the
        // historical schema that required `body` — proving the acceptance
        // was introduced by the scaffold relaxation, not pre-existing.
        let old = auto_ac_prototype::descriptor::Descriptor::parse(
            old_schema.trim_start_matches('\u{feff}'),
        )
        .unwrap();
        assert!(
            auto_ac_prototype::descriptor::bind(&old, &text).is_err(),
            "{profile:?}: same input rejected under the pre-repair descriptor"
        );

        // The public binder still ACCEPTS body-less declarations (the
        // function-level scaffold needs them) — so the rejection must land
        // at the whole-module credential boundary, never in native.
        let bundle = auto_ac_prototype::bind_document(&text).unwrap();
        assert!(
            bundle.modules[0].bodies.is_empty(),
            "{profile:?}: the stripped document carries no body"
        );
        let diags = verify_profile(bundle, profile).unwrap_err();
        assert!(
            diags.iter().any(|d| d.code == "verify.missing-body"),
            "{profile:?}: controlled missing-body rejection, got {diags:?}"
        );
    }
}

#[test]
fn contract_828_admission_mixed_and_uncalled_bodyless_rejected() {
    use auto_ac_prototype::source as ac;
    // Mixed document: main keeps its REAL body and CALLS helper, helper's
    // body is stripped — still a whole-module rejection.
    let cold = ac::compile_parts_with(TWO_FNS, "probe.at", &[], ac::SourceProfile::Scalar).unwrap();
    let mixed = strip_body_field_and_section(&cold.atom_text, "b_helper");
    assert!(auto_ac_prototype::bind_document(&mixed).is_ok());
    let diags = verify_profile(
        auto_ac_prototype::bind_document(&mixed).unwrap(),
        ac::SourceProfile::Scalar,
    )
    .unwrap_err();
    assert!(
        diags.iter().any(|d| d.code == "verify.missing-body"),
        "mixed document rejected: {diags:?}"
    );

    // Uncalled body-less declaration: helper is never called — still a
    // rejection (the walk is per-def, not call-driven).
    let src = "fn helper() int { return 1 }\nfn main() int { return 2 }\n";
    let cold = ac::compile_parts_with(src, "probe.at", &[], ac::SourceProfile::Scalar).unwrap();
    let uncalled = strip_body_field_and_section(&cold.atom_text, "b_helper");
    let diags = verify_profile(
        auto_ac_prototype::bind_document(&uncalled).unwrap(),
        ac::SourceProfile::Scalar,
    )
    .unwrap_err();
    assert!(
        diags.iter().any(|d| d.code == "verify.missing-body"),
        "uncalled body-less declaration rejected: {diags:?}"
    );
}

/// The paired boundary: the SAME relaxation stays LEGAL at function
/// level — the seam's single-body document (one real body + body-less
/// scaffolds) passes `verify_function` for its changed function, while
/// the same bundle is refused the whole-module credential.
#[test]
fn contract_828_admission_scaffold_still_legal_at_function_level() {
    use auto_ac_prototype::source as ac;
    let original = ac::parse_program_with(TWO_FNS, ac::SourceProfile::Scalar).unwrap();
    let mut env = ac::resolve_env(&original, &[], ac::SourceProfile::Scalar).unwrap();
    let edited = fn_item_of(TWO_FNS_EDITED, "helper");
    ac::recheck_function(&mut env, &edited, 0).unwrap();
    let (parts, receipt) = ac::emit_function_body(&env, 0, TWO_FNS_EDITED, "probe.at").unwrap();
    assert_eq!(receipt.real_body_plans, 1);
    let module = parts.bundle.modules.first().unwrap();
    let def_idx = module
        .defs
        .iter()
        .position(|d| d.id_text == "d_helper")
        .unwrap();
    assert!(auto_ac_prototype::verify::verify_function(parts.bundle.clone(), def_idx).is_ok());
    // ...and the very same bundle carries body-less defs, so the
    // whole-module credential refuses it (one body for two functions).
    let diags = verify_profile(parts.bundle, ac::SourceProfile::Scalar).unwrap_err();
    assert!(
        diags.iter().any(|d| d.code == "verify.missing-body"),
        "whole-module credential refuses the single-body document: {diags:?}"
    );
}

// ---------------------------------------------------------------------------
// T01-C05 transaction first cases (`contract_828_transaction`): the
// frozen D-03 mechanism — recheck grafts the candidate and hands "current"
// back as a bounded single-slot save; failure/cancel restores, commit
// publishes once, close explicitly releases the environment's owned
// entries.
// ---------------------------------------------------------------------------

#[test]
fn contract_828_transaction_cancel_then_continue() {
    use auto_ac_prototype::source as ac;
    let parsed = ac::parse_program_with(TWO_FNS, ac::SourceProfile::Scalar).unwrap();
    let mut env = ac::resolve_env(&parsed, &[], ac::SourceProfile::Scalar).unwrap();
    let cold_original =
        ac::compile_parts_with(TWO_FNS, "probe.at", &[], ac::SourceProfile::Scalar).unwrap();
    let cold_edited =
        ac::compile_parts_with(TWO_FNS_EDITED, "probe.at", &[], ac::SourceProfile::Scalar).unwrap();

    // Candidate A on the shared base: recheck ok, candidate grafted, save
    // holds the displaced current entry — then CANCELLED.
    let save_a =
        ac::recheck_transactional(&mut env, &fn_item_of(TWO_FNS_EDITED, "helper"), 0).unwrap();
    assert_eq!(save_a.slot_name(), "helper");
    save_a.restore(&mut env).unwrap();

    // Current is preserved across the cancelled candidate: emitting the
    // ORIGINAL body from the same environment reproduces the cold compile.
    let (parts, _) = ac::emit_function_body(&env, 0, TWO_FNS, "probe.at").unwrap();
    assert_eq!(
        helper_dump(parts.bundle.modules.first().unwrap(), "d_helper"),
        helper_dump(cold_original.bundle.modules.first().unwrap(), "d_helper"),
        "cancelled candidate did not publish"
    );

    // Legitimate continuation: candidate B from the same base commits, and
    // the environment now publishes B.
    let save_b =
        ac::recheck_transactional(&mut env, &fn_item_of(TWO_FNS_EDITED, "helper"), 0).unwrap();
    save_b.commit(&env).unwrap();
    let (parts, _) = ac::emit_function_body(&env, 0, TWO_FNS_EDITED, "probe.at").unwrap();
    assert_eq!(
        helper_dump(parts.bundle.modules.first().unwrap(), "d_helper"),
        helper_dump(cold_edited.bundle.modules.first().unwrap(), "d_helper"),
        "committed candidate published exactly once"
    );

    // Foreign disposition guard: a save belonging to this environment's
    // slot is refused by another environment (identity, not index trust).
    let other_src = "fn main() int { return 1 }\n";
    let other = ac::parse_program_with(other_src, ac::SourceProfile::Scalar).unwrap();
    let mut other_env = ac::resolve_env(&other, &[], ac::SourceProfile::Scalar).unwrap();
    let save_c =
        ac::recheck_transactional(&mut env, &fn_item_of(TWO_FNS_EDITED, "helper"), 0).unwrap();
    // The rejected (consumed) disposition leaves the save dead — a foreign
    // restore is a rejection, never a silent fallback.
    assert!(save_c.restore(&mut other_env).is_err());
}

/// The minimal post-rejection transaction case (the review's demonstrated
/// primitive, now transactional): a body that passes re-check but is
/// rejected by the REAL verifier (`var x = 2` without return) restores the
/// current entry — the next emission from the same environment is the
/// ORIGINAL body, not the rejected one.
#[test]
fn contract_828_transaction_post_verify_rejection_keeps_current() {
    use auto_ac_prototype::source as ac;
    const NO_RETURN: &str =
        "fn helper(n int) int { var x = 2 }\nfn main() int { return helper(2) }\n";
    let parsed = ac::parse_program_with(TWO_FNS, ac::SourceProfile::Scalar).unwrap();
    let mut env = ac::resolve_env(&parsed, &[], ac::SourceProfile::Scalar).unwrap();
    let cold_original =
        ac::compile_parts_with(TWO_FNS, "probe.at", &[], ac::SourceProfile::Scalar).unwrap();

    // The prepare transaction runs recheck -> emit -> verify; the verify
    // stage rejects (missing return) and the environment is restored.
    let diags = match ac::prepare_function_candidate(
        &mut env,
        &fn_item_of(NO_RETURN, "helper"),
        0,
        NO_RETURN,
        "probe.at",
    ) {
        Err(ds) => ds,
        Ok(_) => panic!("the real function verifier rejects the control-flow body"),
    };
    assert!(
        !diags.is_empty(),
        "the real function verifier rejects the control-flow body"
    );

    // CURRENT is preserved: the same environment now emits the ORIGINAL
    // body — verifying cleanly and matching the cold compile exactly.
    let (parts, receipt) = ac::emit_function_body(&env, 0, TWO_FNS, "probe.at").unwrap();
    assert_eq!(receipt.real_body_plans, 1);
    let module = parts.bundle.modules.first().unwrap();
    let def_idx = module
        .defs
        .iter()
        .position(|d| d.id_text == "d_helper")
        .unwrap();
    assert!(auto_ac_prototype::verify::verify_function(parts.bundle.clone(), def_idx).is_ok());
    assert_eq!(
        helper_dump(module, "d_helper"),
        helper_dump(cold_original.bundle.modules.first().unwrap(), "d_helper"),
        "a post-recheck rejection never replaces current semantics"
    );
}

/// D-03 close seam: release of the environment's owned body entries is an
/// EXPLICIT call (never implied by field drops); post-close recheck, emit
/// and prepare are controlled rejections; close is idempotent.
#[test]
fn contract_828_transaction_close_releases_owned_entries() {
    use auto_ac_prototype::source as ac;
    let parsed = ac::parse_program_with(TWO_FNS, ac::SourceProfile::Scalar).unwrap();
    let mut env = ac::resolve_env(&parsed, &[], ac::SourceProfile::Scalar).unwrap();
    assert_eq!(ac::resolve::env_fn_count(&env), 2);

    // A live candidate save at close time.
    let (_, _, save) = ac::prepare_function_candidate(
        &mut env,
        &fn_item_of(TWO_FNS_EDITED, "helper"),
        0,
        TWO_FNS_EDITED,
        "probe.at",
    )
    .unwrap();

    env.close();
    assert!(env.closed());
    assert_eq!(
        ac::resolve::env_fn_count(&env),
        0,
        "owned resolved body entries explicitly released"
    );
    // Closed disposition: commit is a rejection.
    assert!(save.commit(&env).is_err());
    // Post-close rejections on every entry point.
    let recheck = ac::recheck_function(&mut env, &fn_item_of(TWO_FNS_EDITED, "helper"), 0);
    assert!(
        recheck
            .unwrap_err()
            .iter()
            .any(|d| d.code == "source.closed"),
        "closed recheck is a controlled rejection"
    );
    let emit = ac::emit_function_body(&env, 0, TWO_FNS, "probe.at");
    match emit {
        Err(ds) => assert!(
            ds.iter().any(|d| d.code == "source.closed"),
            "closed emission is a controlled rejection, got {ds:?}"
        ),
        Ok(_) => panic!("closed emission is a controlled rejection"),
    }
    env.close(); // idempotent
}
