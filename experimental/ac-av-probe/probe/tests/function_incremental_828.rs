//! PLAN-828 T-01 — the bounded full-path first case (V-01/V-04 minimal
//! positive and negative controls).
//!
//! Real source, two functions `main → helper`, warmed up once through the
//! whole-module path. A single body edit of `helper` must re-run ONLY
//! helper's chain (syntax rescan/graft → per-function frontend → HIR verify
//! → unit lower → MIR verify → ABC emit/decode/verify → install) and
//! re-link the snapshot; `main` keeps its HIR body object and FunctionCode
//! `Arc` identity, every stage counter for it stays zero, and no
//! whole-module frontend/lower/closure-roundtrip runs. A rejected
//! candidate (type error) leaves the current revision runnable and
//! unchanged.
//!
//! Expected values are INDEPENDENT of the pipeline (hand-evaluated):
//! helper(5) = 5*3+7 = 22, main = 23; after `x * 3 → x * 4`:
//! helper(5) = 27, main = 28.

use auto_av_core::av::{execute_program, CancelFlag, ExecLimits};
use auto_av_core::contract::{ExecOutcome, Value};
use auto_av_probe::load256;
use auto_av_probe::session::{warmup, STAGES};
use std::sync::Arc;

// helper precedes main: the frozen splice convergence needs ≥8 aligned
// tokens after the edit region (helper-first keeps main's whole token tail
// available; the frozen load's edit sites satisfy this structurally).
const SRC: &str = "fn helper(x int) int { let r = x * 3; return r + 7 }\nfn main() int { let a = helper(5); return a + 1 }\n";

fn assert_i32(out: &ExecOutcome, want: i32, ctx: &str) {
    match out {
        ExecOutcome::Success {
            value: Value::I32(v), ..
        } => assert_eq!(*v, want, "{ctx}"),
        other => panic!("{ctx}: expected Success(I32({want})), got {other:?}"),
    }
}

#[test]
fn t01_v01_single_body_edit_full_path_incremental() {
    let mut s = warmup(SRC, "d_main", &[]).expect("warm-up");
    let cancel = CancelFlag::new(false);

    // Baseline: the warm-up snapshot runs the independent expected value.
    let (out, _) = s.run(&ExecLimits::default(), &cancel);
    assert_i32(&out, 23, "warm-up run");

    // Identities before the edit.
    let (_, main_hir0, main_code0) = s.resident("main").expect("main resident");
    let (_, helper_hir0, helper_code0) = s.resident("helper").expect("helper resident");
    let snap0 = s.current();
    assert_eq!(snap0.revision(), 0);
    let idx_of = |snap: &Arc<auto_av_core::program_snapshot::ProgramSnapshot>, name: &str| {
        snap.code()
            .iter()
            .position(|c| c.name().ends_with(name))
            .unwrap_or_else(|| panic!("no code unit ending in `{name}`"))
    };
    let main0_in_snap = idx_of(&snap0, "main");

    // Single body edit inside helper: `x * 3` → `x * 4`.
    let star3 = SRC.find("x * 3").expect("target");
    let pos = (star3 + 4) as u64; // the '3'
    let outcome = s.edit(pos, pos + 1, b"4".to_vec()).expect("edit applies");
    assert!(
        outcome.ok,
        "fast path must run: {:?}",
        outcome.non_fast_path_reason
    );
    assert_eq!(outcome.changed_fn.as_deref(), Some("helper"));
    assert_eq!(outcome.install_set.len(), 1, "exactly one install");
    assert!(
        outcome.install_set[0].ends_with("helper"),
        "install set targets helper's def, got {:?}",
        outcome.install_set
    );
    assert_eq!(outcome.revision, 1);

    // New revision produces the new independent expected value.
    let (out1, _) = s.run(&ExecLimits::default(), &cancel);
    assert_i32(&out1, 28, "post-edit run");

    // Unchanged function object identity across the revision.
    let (_, main_hir1, main_code1) = s.resident("main").expect("main resident");
    assert!(
        Arc::ptr_eq(&main_hir0, &main_hir1),
        "main's HIR body object must be reused"
    );
    assert!(
        Arc::ptr_eq(&main_code0, &main_code1),
        "main's FunctionCode must be reused"
    );
    let snap1 = s.current();
    let main1_in_snap = idx_of(&snap1, "main");
    assert!(
        Arc::ptr_eq(&snap0.code()[main0_in_snap], &snap1.code()[main1_in_snap]),
        "snapshot keeps main's code Arc"
    );
    let (_, _, helper_code1) = s.resident("helper").expect("helper resident");
    assert!(
        !Arc::ptr_eq(&helper_code0, &helper_code1),
        "helper's code must be a new object"
    );
    let _ = helper_hir0;

    // Structural gate: every stage of the UNCHANGED function is zero.
    for (stage, count) in s.counters().stages_of("main") {
        assert_eq!(count, 0, "main stage `{stage}` must be zero");
    }
    // And every whole-module stage is zero.
    for (stage, count) in s.counters().whole_module() {
        assert_eq!(count, 0, "{stage} must be zero");
    }
    // The changed function ran its whole chain exactly once.
    for stage in STAGES {
        assert_eq!(
            s.counters().stage_of(stage, "helper"),
            1,
            "helper stage `{stage}`"
        );
    }
    // Syntax increment is real: exactly the edited decl re-parsed, the
    // other grafted.
    assert_eq!(s.counters().get("syntax_reparsed_decls"), 1);
    assert_eq!(s.counters().get("syntax_reused_decls"), 1);

    // Snapshot visibility (V-05 seed): a caller that pinned the old
    // snapshot still observes the OLD semantics after the install.
    let (old_out, _) = execute_program(&snap0, &ExecLimits::default(), &cancel);
    assert_i32(&old_out, 23, "pinned old snapshot run");
    let (new_out, _) = execute_program(&snap1, &ExecLimits::default(), &cancel);
    assert_i32(&new_out, 28, "new snapshot run");
}

#[test]
fn t01_v04_rejected_candidate_keeps_current_version() {
    let mut s = warmup(SRC, "d_main", &[]).expect("warm-up");
    let cancel = CancelFlag::new(false);
    let (out, _) = s.run(&ExecLimits::default(), &cancel);
    assert_i32(&out, 23, "warm-up run");

    let (_, main_hir0, main_code0) = s.resident("main").expect("main resident");
    let (_, _, helper_code0) = s.resident("helper").expect("helper resident");

    // Broken candidate: `return r + 7` → `return true` (bool in an int
    // function — a real typecheck rejection, not a synthetic flag).
    let target = SRC.find("r + 7").expect("target");
    let outcome = s
        .edit(target as u64, (target + 5) as u64, b"true".to_vec())
        .expect("edit applies");
    assert!(!outcome.ok, "candidate must be rejected");
    assert!(outcome.install_set.is_empty(), "nothing installed");
    assert!(
        !outcome.diagnostics.is_empty(),
        "the rejection carries real diagnostics: {:?}",
        outcome.diagnostics
    );

    // The current revision is untouched and still runs the old result.
    assert_eq!(s.revision(), 0);
    let (out1, _) = s.run(&ExecLimits::default(), &cancel);
    assert_i32(&out1, 23, "post-rejection run keeps old version");
    let (_, main_hir1, main_code1) = s.resident("main").expect("main resident");
    assert!(Arc::ptr_eq(&main_hir0, &main_hir1));
    assert!(Arc::ptr_eq(&main_code0, &main_code1));
    let (_, _, helper_code1) = s.resident("helper").expect("helper resident");
    assert!(
        Arc::ptr_eq(&helper_code0, &helper_code1),
        "rejected candidate must not replace helper's code"
    );
    // No stage ran for the UNCHANGED function; the changed function's
    // PROJECTION legitimately ran (the edit is syntactically valid — the
    // rejection is semantic), but nothing past the frontend completed.
    for (stage, count) in s.counters().stages_of("main") {
        assert_eq!(count, 0, "main stage `{stage}`");
    }
    assert_eq!(s.counters().stage_of("project", "helper"), 1);
    for stage in STAGES.iter().filter(|s| **s != "project") {
        assert_eq!(
            s.counters().stage_of(stage, "helper"),
            0,
            "helper stage `{stage}` after rejected candidate"
        );
    }

    // Recovery: a correct edit relative to the still-accepted source runs
    // the fast path and installs the new version.
    let star3 = SRC.find("x * 3").expect("target");
    let pos = (star3 + 4) as u64;
    let outcome = s.edit(pos, pos + 1, b"4".to_vec()).expect("edit applies");
    assert!(
        outcome.ok,
        "recovery edit must run: {:?}",
        outcome.non_fast_path_reason
    );
    assert_eq!(s.revision(), 1);
    let (out2, _) = s.run(&ExecLimits::default(), &cancel);
    assert_i32(&out2, 28, "recovery run");
    for stage in STAGES.iter().filter(|s| **s != "project") {
        assert_eq!(
            s.counters().stage_of(stage, "helper"),
            1,
            "helper stage `{stage}` after recovery"
        );
    }
    // project ran for both the rejected candidate and the recovery edit.
    assert_eq!(s.counters().stage_of("project", "helper"), 2);
}

/// T-02 frozen-load parity: the 256 KiB scalar load (115 functions, the
/// full 798 surface incl. binary `-`, `>`, conditional `for`) must project
/// through the auto-scalar grammar to the EXACT ac-core scalar-parser AST
/// (structure + spans, Debug equality) — and so must an edited revision.
#[test]
fn t02_frozen_load_projection_parity() {
    let gs = std::fs::read_to_string(format!(
        "{}/../../ac-syntax/grammars/auto-scalar.at",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    let p = ac_syntax_grammar::compile(&gs).unwrap();
    let cs = ac_syntax_rt::lexer::CompiledScanner::compile(&p).unwrap();
    let load = load256::generate_revision(0);
    let load_edited = load256::generate_revision(3);
    for (tag, src) in [("base", load.source.as_str()), ("rev3", load_edited.source.as_str())] {
        let snap =
            ac_syntax_rt::snapshot::Snapshot::from_text(&std::rc::Rc::new(ac_syntax_rt::snapshot::PageStore::new()), src);
        let out = ac_syntax_rt::parse::parse_snapshot(
            &p,
            &cs,
            &snap,
            &ac_syntax_rt::program::Budget::default(),
        )
        .unwrap();
        assert!(out.tree.diags.is_empty(), "{tag}: syntax diags");
        let k = ac_syntax_core_adapter::CoreKinds::from_program(&p);
        let projected =
            ac_syntax_core_adapter::project_strict(&p, &k, &out.tree, &snap).unwrap();
        let oracle = auto_ac_prototype::source::parse_program_with(
            src,
            auto_ac_prototype::source::SourceProfile::Scalar,
        )
        .unwrap();
        assert_eq!(
            format!("{projected:?}"),
            format!("{oracle:?}"),
            "{tag}: frozen-load AST+span parity"
        );
    }
}

/// T-02 syntax-error control: an edit that breaks the changed decl's syntax
/// is rejected at projection (strict HasErrors) and the current revision
/// stays runnable — a refused fast path, never a silent whole-module
/// rebuild.
#[test]
fn t02_syntax_error_candidate_rejected_keeps_old() {
    let mut s = warmup(SRC, "d_main", &[]).expect("warm-up");
    let cancel = CancelFlag::new(false);
    let (out, _) = s.run(&ExecLimits::default(), &cancel);
    assert_i32(&out, 23, "warm-up run");
    // Break the body: `x * 3` -> `x *` (dangling operator).
    let star3 = SRC.find("x * 3").expect("target");
    let pos = (star3 + 4) as u64;
    let outcome = s
        .edit(pos, pos + 1, b"".to_vec())
        .expect("edit applies");
    assert!(!outcome.ok, "syntax-broken candidate must be rejected");
    assert!(outcome.install_set.is_empty());
    assert_eq!(s.revision(), 0);
    let (out1, _) = s.run(&ExecLimits::default(), &cancel);
    assert_i32(&out1, 23, "old version still runs");
    // main untouched at every stage.
    for (stage, count) in s.counters().stages_of("main") {
        assert_eq!(count, 0, "main stage `{stage}`");
    }
}
