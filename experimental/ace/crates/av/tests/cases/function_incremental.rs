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

use ace_av::load256;
use ace_av::session::{warmup, STAGES};
use ace_av_core::av::{execute_program, CancelFlag, ExecLimits};
use ace_av_core::contract::{ExecOutcome, Value};
use std::sync::Arc;

// helper precedes main: the frozen splice convergence needs ≥8 aligned
// tokens after the edit region (helper-first keeps main's whole token tail
// available; the frozen load's edit sites satisfy this structurally).
const SRC: &str = "fn helper(x int) int { let r = x * 3; return r + 7 }\nfn main() int { let a = helper(5); return a + 1 }\n";

fn assert_i32(out: &ExecOutcome, want: i32, ctx: &str) {
    match out {
        ExecOutcome::Success {
            value: Value::I32(v),
            ..
        } => assert_eq!(*v, want, "{ctx}"),
        other => panic!("{ctx}: expected Success(I32({want})), got {other:?}"),
    }
}

#[test]
fn v01_single_body_edit_full_path_incremental() {
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
    let idx_of = |snap: &Arc<ace_av_core::program_snapshot::ProgramSnapshot>, name: &str| {
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
fn v04_rejected_candidate_keeps_current_version() {
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
fn frozen_load_projection_parity() {
    let gs = std::fs::read_to_string(format!(
        "{}/../../grammars/auto-scalar.at",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    let p = ace_syntax_grammar::compile(&gs).unwrap();
    let cs = ace_syntax_runtime::lexer::CompiledScanner::compile(&p).unwrap();
    let load = load256::generate_revision(0);
    let load_edited = load256::generate_revision(3);
    for (tag, src) in [
        ("base", load.source.as_str()),
        ("rev3", load_edited.source.as_str()),
    ] {
        let snap = ace_syntax_runtime::snapshot::Snapshot::from_text(
            &std::rc::Rc::new(ace_syntax_runtime::snapshot::PageStore::new()),
            src,
        );
        let out = ace_syntax_runtime::parse::parse_snapshot(
            &p,
            &cs,
            &snap,
            &ace_syntax_runtime::program::Budget::default(),
        )
        .unwrap();
        assert!(out.tree.diags.is_empty(), "{tag}: syntax diags");
        let k = ace_syntax_core::CoreKinds::from_program(&p);
        let projected = ace_syntax_core::project_strict(&p, &k, &out.tree, &snap).unwrap();
        let oracle =
            ace_core::source::parse_program_with(src, ace_core::source::SourceProfile::Scalar)
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
fn syntax_error_candidate_rejected_keeps_old() {
    let mut s = warmup(SRC, "d_main", &[]).expect("warm-up");
    let cancel = CancelFlag::new(false);
    let (out, _) = s.run(&ExecLimits::default(), &cancel);
    assert_i32(&out, 23, "warm-up run");
    // Break the body: `x * 3` -> `x *` (dangling operator).
    let star3 = SRC.find("x * 3").expect("target");
    let pos = (star3 + 4) as u64;
    let outcome = s.edit(pos, pos + 1, b"".to_vec()).expect("edit applies");
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

/// T-03: interface environment gates + credential currency.
#[test]
fn signature_change_is_non_fast_path_and_keeps_old() {
    let mut s = warmup(SRC, "d_main", &[]).expect("warm-up");
    let cancel = CancelFlag::new(false);
    let (out, _) = s.run(&ExecLimits::default(), &cancel);
    assert_i32(&out, 23, "warm-up run");

    // Edit the SIGNATURE (helper gains a param): still one decl, still valid
    // code — the interface-environment gate must refuse it as non-fast-path.
    let sig_pos = SRC.find("(x int)").expect("sig");
    let outcome = s
        .edit(
            sig_pos as u64,
            (sig_pos + 7) as u64,
            b"(x int, y int)".to_vec(),
        )
        .expect("edit applies");
    assert!(!outcome.ok, "signature edit must not take the fast path");
    let reason = outcome.non_fast_path_reason.unwrap_or_default();
    assert!(
        reason.contains("interface") || reason.contains("context"),
        "rejection reason names the interface gate: {reason}"
    );
    assert_eq!(s.revision(), 0);
    let (out1, _) = s.run(&ExecLimits::default(), &cancel);
    assert_i32(&out1, 23, "old version still runs");
}

#[test]
fn body_credentials_are_context_bound() {
    use ace_av::function_queries::FnEnv;
    use ace_core::source::compile_scalar_source;
    use ace_core::verify::verify_function;

    let a = "fn helper(x int) int { return x }
fn main() int { return helper(1) }";
    let b = "fn helper(x int, y int) int { return x + y }
fn main() int { return helper(1, 2) }";
    let mint = |src: &str| -> (FnEnv, String) {
        let compiled = compile_scalar_source(src, "probe.at", &[]).expect("compiles");
        let bundle = compiled.checked.bundle();
        let env = FnEnv::from_bundle(bundle).expect("env");
        let idx = bundle.modules[0]
            .defs
            .iter()
            .position(|d| d.id_text == "d_helper")
            .expect("helper def");
        let cf = verify_function(bundle.clone(), idx).expect("verify");
        (env, cf.context_revision().to_string())
    };
    let (env_a, _rev_a) = mint(a);
    let (env_b, _rev_b) = mint(b);
    assert_ne!(_rev_a, _rev_b, "different declaration contexts");
    // A credential minted under context A is current in env A only — the
    // ENVIRONMENT decides from its own bundle (F-828-R1-06), no
    // caller-supplied revision string participates.
    let cf_a = {
        let compiled = compile_scalar_source(a, "probe.at", &[]).expect("compiles");
        let bundle = compiled.checked.bundle();
        let idx = bundle.modules[0]
            .defs
            .iter()
            .position(|d| d.id_text == "d_helper")
            .unwrap();
        verify_function(bundle.clone(), idx).expect("verify")
    };
    assert!(env_a.credential_current(&cf_a));
    assert!(!env_b.credential_current(&cf_a));
    // Interface queries are owner-scoped: unknown names are absent.
    assert!(env_a.interface_of("nope").is_none());
    assert_eq!(env_a.def_id_of("helper"), Some("d_helper"));
}

/// T-04: in-session content-addressed reuse — editing a function BACK to a
/// prior content reuses the stored verified objects (identity!) with zero
/// re-verification / re-lowering / re-admission.
#[test]
fn content_addressed_reuse_hits_previous_revision() {
    let mut s = warmup(SRC, "d_main", &[]).expect("warm-up");
    let cancel = CancelFlag::new(false);
    let (out, _) = s.run(&ExecLimits::default(), &cancel);
    assert_i32(&out, 23, "warm-up run");
    let (_, hir_a, code_a) = s.resident("helper").expect("helper resident");

    // A -> B (`3` -> `4`): full chain for the new content.
    let star3 = SRC.find("x * 3").expect("target");
    let pos = (star3 + 4) as u64;
    let out1 = s.edit(pos, pos + 1, b"4".to_vec()).expect("edit A->B");
    assert!(out1.ok);
    let (r1, _) = s.run(&ExecLimits::default(), &cancel);
    assert_i32(&r1, 28, "A->B run");

    // B -> A (revert; same byte position): content hit — the warm-up
    // objects come back by IDENTITY.
    let out2 = s.edit(pos, pos + 1, b"3".to_vec()).expect("edit B->A");
    assert!(out2.ok, "{:?}", out2.non_fast_path_reason);
    let (r2, _) = s.run(&ExecLimits::default(), &cancel);
    assert_i32(&r2, 23, "B->A run");
    let (_, hir_now, code_now) = s.resident("helper").expect("helper resident");
    assert!(
        Arc::ptr_eq(&hir_a, &hir_now),
        "reverted content reuses the warm-up HIR body object"
    );
    assert!(
        Arc::ptr_eq(&code_a, &code_now),
        "reverted content reuses the warm-up FunctionCode"
    );

    // Counters: exactly ONE lower/emit/verify chain ran (for B); the revert
    // was a content hit; main untouched throughout.
    assert_eq!(s.counters().stage_of("reuse_hit", "helper"), 1);
    assert_eq!(s.counters().stage_of("mir_lower", "helper"), 1);
    assert_eq!(s.counters().stage_of("verify_hir", "helper"), 1);
    assert_eq!(s.counters().stage_of("abc_emit", "helper"), 1);
    assert_eq!(s.counters().stage_of("load", "helper"), 1);
    for (stage, count) in s.counters().stages_of("main") {
        assert_eq!(count, 0, "main stage `{stage}`");
    }
}

/// T-04 admission negatives: an executable closure can never become
/// resident function code, and the snapshot link refuses signature-mismatched
/// call bindings (V-04 trust share).
#[test]
fn unit_admission_negatives() {
    use ace_av_core::abc::encode_abc;
    use ace_av_core::abc_verify::verify_abc_bytes;
    use ace_av_core::contract::MIR_SCHEMA;
    use ace_av_core::function_code::FunctionCode;
    use ace_av_core::mir::{
        FnSig, MirBlock, MirConst, MirFunction, MirInstr, MirModule, MirSlot, MirType, Terminator,
    };
    use ace_av_core::program_snapshot::ProgramSnapshot;

    let owner = ace_av_core::mir::OwnerScope {
        schema: MIR_SCHEMA.to_string(),
        module: "m828".to_string(),
    };
    // A COMPLETE (non-unit) module admits as an executable closure — which
    // FunctionCode must refuse.
    let mut whole = MirModule::new(owner.clone());
    let ti = whole.intern_type(MirType::I32);
    let sig = whole.intern_type(MirType::Fn(FnSig {
        params: vec![],
        result: ti,
    }));
    let c7 = whole.intern_const(MirConst::I32(7));
    whole.funcs.push(MirFunction {
        owner: owner.clone(),
        name: "d_main".into(),
        sig,
        slots: vec![MirSlot {
            kind: ace_av_core::mir::SlotKind::Local,
            ty: ti,
            name: None,
        }],
        blocks: vec![MirBlock {
            instrs: vec![MirInstr::Const { dst: 0, value: c7 }],
            term: Terminator::Return { value: 0 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: vec![],
        env: None,
    });
    whole.entry = 0;
    let bytes = encode_abc(&whole).unwrap().to_flat();
    let loaded = verify_abc_bytes(&bytes, &owner).unwrap();
    assert!(loaded.executable());
    assert!(
        FunctionCode::admit_unit(loaded, 0).is_err(),
        "executable closures are not resident function code"
    );

    // Signature-mismatched link: unit A imports `fn:d_b` as ()->i32 while
    // unit B (the only d_b) exports (i32)->i32 — link must refuse.
    let mut unit_a = MirModule::unit(owner.clone());
    let ti_a = unit_a.intern_type(MirType::I32);
    let wrong_sig = unit_a.intern_type(MirType::Fn(FnSig {
        params: vec![],
        result: ti_a,
    }));
    let call_sig = unit_a.intern_type(MirType::Fn(FnSig {
        params: vec![],
        result: ti_a,
    }));
    let imp = unit_a.intern_import(ace_av_core::mir::MirImport {
        symbol: "fn:d_b".to_string(),
        effect: ace_av_core::mir::Effect::Pure,
        sig: wrong_sig,
        env: None,
    });
    let _ = call_sig;
    let sig_a = unit_a.intern_type(MirType::Fn(FnSig {
        params: vec![],
        result: ti_a,
    }));
    unit_a.funcs.push(MirFunction {
        owner: owner.clone(),
        name: "d_main".into(),
        sig: sig_a,
        slots: vec![MirSlot {
            kind: ace_av_core::mir::SlotKind::Local,
            ty: ti_a,
            name: None,
        }],
        blocks: vec![MirBlock {
            instrs: vec![MirInstr::Call {
                callee: ace_av_core::mir::CalleeRef::Import(imp),
                dst: 0,
                args: vec![],
            }],
            term: Terminator::Return { value: 0 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: vec![],
        env: None,
    });
    unit_a.entry = 0;
    let mut unit_b = MirModule::unit(owner.clone());
    let ti_b = unit_b.intern_type(MirType::I32);
    let sig_b = unit_b.intern_type(MirType::Fn(FnSig {
        params: vec![ti_b],
        result: ti_b,
    }));
    let cb = unit_b.intern_const(MirConst::I32(1));
    unit_b.funcs.push(MirFunction {
        owner: owner.clone(),
        name: "d_b".into(),
        sig: sig_b,
        slots: vec![
            MirSlot {
                kind: ace_av_core::mir::SlotKind::Param,
                ty: ti_b,
                name: None,
            },
            MirSlot {
                kind: ace_av_core::mir::SlotKind::Local,
                ty: ti_b,
                name: None,
            },
        ],
        blocks: vec![MirBlock {
            instrs: vec![
                MirInstr::Copy { dst: 1, src: 0 },
                MirInstr::Const { dst: 1, value: cb },
            ],
            term: Terminator::Return { value: 1 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: vec![],
        env: None,
    });
    unit_b.entry = 0;
    for u in [&unit_a, &unit_b] {
        let b = encode_abc(u).unwrap().to_flat();
        let d = ace_av_core::abc::decode_abc(&b).unwrap();
        let loaded = ace_av_core::abc_verify::verify_abc(d, &u.owner).unwrap();
        assert!(!loaded.executable(), "units admit as non-executable");
    }
    let code_a = FunctionCode::admit_unit(
        verify_abc_bytes(&encode_abc(&unit_a).unwrap().to_flat(), &owner).unwrap(),
        0,
    )
    .unwrap();
    let code_b = FunctionCode::admit_unit(
        verify_abc_bytes(&encode_abc(&unit_b).unwrap().to_flat(), &owner).unwrap(),
        0,
    )
    .unwrap();
    let err = ProgramSnapshot::link(vec![code_a.into(), code_b.into()], "d_main", 1).unwrap_err();
    assert!(
        err.contains("signature mismatch"),
        "link refuses mismatched call bindings: {err}"
    );
}

/// T-05: stable cross-revision identity + generation axis. The def id is
/// the identity; unchanged functions keep object AND generation; a content
/// hit reuses the original object with its original generation.
#[test]
fn generations_and_stable_ids_across_revisions() {
    let mut s = warmup(SRC, "d_main", &[]).expect("warm-up");
    let cancel = CancelFlag::new(false);
    let snap0 = s.current();
    assert_eq!(snap0.generation_of("d_main"), Some(0));
    assert_eq!(snap0.generation_of("d_helper"), Some(0));
    assert!(snap0.find_code("d_nope").is_none());

    // A -> B: helper mints generation 1; main keeps 0.
    let star3 = SRC.find("x * 3").expect("target");
    let pos = (star3 + 4) as u64;
    let out1 = s.edit(pos, pos + 1, b"4".to_vec()).expect("edit");
    assert!(out1.ok);
    let snap1 = s.current();
    assert_eq!(snap1.generation_of("d_main"), Some(0), "main generation");
    assert_eq!(
        snap1.generation_of("d_helper"),
        Some(1),
        "helper generation"
    );
    // The warm-up snapshot is unaffected (immutable history).
    assert_eq!(snap0.generation_of("d_helper"), Some(0));

    // B -> A (content hit): helper reuses the WARM-UP object, so its
    // generation is the original 0, not a new mint.
    let out2 = s.edit(pos, pos + 1, b"3".to_vec()).expect("edit");
    assert!(out2.ok);
    let snap2 = s.current();
    assert_eq!(
        snap2.generation_of("d_helper"),
        Some(0),
        "content hit keeps the original generation"
    );
    assert_eq!(snap2.generation_of("d_main"), Some(0));
    let (r, _) = s.run(&ExecLimits::default(), &cancel);
    assert_i32(&r, 23, "reverted run");
}

/// T-06 / V-05: deterministic barrier. The chain pauses MID-EXECUTION (a
/// step count — no sleeps, no races), a new revision installs, and the
/// resumed chain must complete with the OLD semantics while new top-level
/// calls see the NEW revision. No callee-version mixing.
#[test]
fn v05_deterministic_barrier_keeps_inflight_chain_on_its_snapshot() {
    use ace_av_core::av::{execute_program_steps, resume_program, SteppedOutcome};
    let mut s = warmup(SRC, "d_main", &[]).expect("warm-up");
    let cancel = CancelFlag::new(false);
    let snap0 = s.current();

    // Pause A mid-execution (a handful of steps into main's body).
    let paused = match execute_program_steps(&snap0, &ExecLimits::default(), &cancel, 3) {
        SteppedOutcome::Paused(p) => p,
        SteppedOutcome::Finished(o, _) => panic!("expected a pause, got {o:?}"),
    };

    // Install B while A's chain is parked.
    let star3 = SRC.find("x * 3").expect("target");
    let pos = (star3 + 4) as u64;
    let outcome = s.edit(pos, pos + 1, b"4".to_vec()).expect("edit");
    assert!(outcome.ok, "{:?}", outcome.non_fast_path_reason);
    assert_eq!(s.revision(), 1);

    // The resumed chain completes on ITS snapshot: old semantics.
    match resume_program(paused, &cancel, None) {
        SteppedOutcome::Finished(out, _) => assert_i32(&out, 23, "resumed old chain"),
        SteppedOutcome::Paused(_) => panic!("resume-to-completion must finish"),
    }
    // New top-level calls see the new revision.
    let (r, _) = s.run(&ExecLimits::default(), &cancel);
    assert_i32(&r, 28, "new top-level call");
}

/// T-06 / V-06: out-of-order commits. Two candidates prepared on the same
/// base; the second commits first, and the slow first candidate is REFUSED
/// as stale — it must not overwrite the newer result.
#[test]
fn v06_stale_candidate_never_overwrites_newer_result() {
    let mut s = warmup(SRC, "d_main", &[]).expect("warm-up");
    let cancel = CancelFlag::new(false);
    let star3 = SRC.find("x * 3").expect("target");
    let pos = (star3 + 4) as u64;

    // r1 (slow): 3 -> 4 ; r2: 3 -> 5. Both prepared on revision 0.
    let r1 = s
        .prepare_edit(pos, pos + 1, b"4".to_vec())
        .unwrap()
        .ok()
        .unwrap();
    let r2 = s
        .prepare_edit(pos, pos + 1, b"5".to_vec())
        .unwrap()
        .ok()
        .unwrap();

    let out2 = s.commit(r2);
    assert!(out2.ok);
    assert_eq!(s.revision(), 1);

    // The late r1 arrives: refused as stale, current stays r2's result.
    let out1 = s.commit(r1);
    assert!(!out1.ok);
    assert!(
        out1.non_fast_path_reason
            .as_deref()
            .unwrap_or_default()
            .contains("stale"),
        "{}",
        out1.non_fast_path_reason.unwrap_or_default()
    );
    assert_eq!(s.revision(), 1, "stale commit does not bump the revision");
    let (r, _) = s.run(&ExecLimits::default(), &cancel);
    assert_i32(
        &r,
        33,
        "r2 result survives the late r1 (helper(5)=5*5+7=32, +1)",
    );

    // Fuel exhaustion is a run-level outcome, not an install: the snapshot
    // stays runnable under normal limits.
    let tiny = ExecLimits {
        max_steps: 1,
        ..ExecLimits::default()
    };
    let (out, _) = s.run(&tiny, &cancel);
    assert!(
        matches!(
            out,
            ExecOutcome::Resource {
                kind: ace_av_core::contract::ResourceKind::InstrSteps,
                ..
            }
        ),
        "got {out:?}"
    );
    let (r2_again, _) = s.run(&ExecLimits::default(), &cancel);
    assert_i32(&r2_again, 33, "snapshot unchanged after fuel exhaustion");
}

/// T-07 / V-07: warm restart re-admits code from the store (verified
/// ABC units, payload-name checked), edits keep working, and a store miss
/// falls back to a cold start — restore never trusts disk, and a miss
/// never masquerades as a warm update.
#[test]
fn restore_readmits_and_miss_falls_back() {
    let dir = std::env::temp_dir().join(format!("p828-store-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let cancel = CancelFlag::new(false);

    // Cold start + deposit.
    {
        let s = ace_av::session::warmup_persisted(SRC, "d_main", &[], &dir, "p828")
            .expect("cold persisted warm-up");
        let (r, _) = s.run(&ExecLimits::default(), &cancel);
        assert_i32(&r, 23, "cold run");
    }

    // Warm restart over the same store + source.
    {
        let mut s = ace_av::session::restore(SRC, "d_main", &[], &dir, "p828").expect("restore");
        assert_eq!(
            s.counters().get("restore_units_admitted"),
            2,
            "both functions' code re-admitted from disk"
        );
        assert_eq!(s.counters().get("restore_materialize"), 1);
        let (r, _) = s.run(&ExecLimits::default(), &cancel);
        assert_i32(&r, 23, "restored run");
        // Edits after restore stay on the incremental path.
        let star3 = SRC.find("x * 3").expect("target");
        let pos = (star3 + 4) as u64;
        let out = s.edit(pos, pos + 1, b"4".to_vec()).expect("edit");
        assert!(out.ok, "{:?}", out.non_fast_path_reason);
        let (r, _) = s.run(&ExecLimits::default(), &cancel);
        assert_i32(&r, 28, "post-restore edit");
        for (stage, count) in s.counters().stages_of("main") {
            assert_eq!(count, 0, "main stage `{stage}` after restore+edit");
        }
    }

    // Miss: a DIFFERENT source (content versions drifted) must fall back
    // to a cold start, still correct.
    {
        let other = SRC.replace("x * 3", "x * 9");
        let s = ace_av::session::restore(&other, "d_main", &[], &dir, "p828")
            .expect("miss falls back to cold");
        let (r, _) = s.run(&ExecLimits::default(), &cancel);
        assert_i32(&r, 53, "cold fallback run (helper(5)=5*9+7=52, +1)");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// T-07 / V-07: 512 real edits — every sampled revision correct, content
/// history bounded, and a snapshot pinned at revision 0 stays usable the
/// whole way (liveness), then is dropped (reclamation).
#[test]
fn case_512_revision_soak_bounded_history_pinned_old_snapshot() {
    let mut s = warmup(SRC, "d_main", &[]).expect("warm-up");
    let cancel = CancelFlag::new(false);
    let pinned = s.current();
    let star3 = SRC.find("x * 3").expect("target");
    let pos = (star3 + 4) as u64;
    let digit = |k: u32| char::from(b'0' + (k % 10) as u8);

    for k in 3u32..(3 + 512) {
        let out = s
            .edit(pos, pos + 1, digit(k).to_string().into_bytes())
            .expect("edit");
        assert!(out.ok, "revision {k}: {:?}", out.non_fast_path_reason);
        assert_eq!(s.revision(), (k - 2) as u64);
        if k % 64 == 0 {
            let want = 5 * (k % 10) as i32 + 7 + 1;
            let (r, _) = s.run(&ExecLimits::default(), &cancel);
            assert_i32(&r, want, "revision {k} result");
        }
    }
    assert_eq!(s.revision(), 512);
    // History stays bounded (2 functions × HISTORY_CAP).
    assert!(
        s.history_len() <= 2 * ace_av::session::HISTORY_CAP,
        "history {} unbounded",
        s.history_len()
    );
    // The pinned revision-0 snapshot stayed ALIVE the whole soak.
    let (r, _) = execute_program(&pinned, &ExecLimits::default(), &cancel);
    assert_i32(&r, 23, "pinned snapshot still usable");
    drop(pinned); // reclaimed; nothing to assert beyond drop semantics.
}

/// T-07 / V-02 (span axis): a trap's span maps through the accepted
/// source's line/col index, and an edit that shifts the trapping
/// expression moves the mapping with it.
#[test]
fn trap_span_maps_through_line_col() {
    let src_t = "fn helper(x int) int { let a = x + 1; return a / (x - x) }
fn main() int { return helper(6) }
";
    let mut s = warmup(src_t, "d_main", &[]).expect("warm-up");
    let cancel = CancelFlag::new(false);
    let (out, _) = s.run(&ExecLimits::default(), &cancel);
    let ExecOutcome::Trap {
        span: Some(sp1), ..
    } = out
    else {
        panic!("expected a trap with a span, got {out:?}")
    };
    let (l1, c1) = s.line_col(sp1.start as u64);
    assert_eq!((l1, c1), (1, c1), "trap lands on helper's line 1");

    // Shift the trapping expression right by inserting a statement before
    // it (`var b = 2; ` — 10 bytes): the trap column moves +10 on the same
    // line, proving spans ride the CURRENT accepted source.
    let at = src_t.find("return a").expect("target");
    let out_edit = s
        .edit(at as u64, at as u64, b"var b = 2; ".to_vec())
        .expect("edit");
    assert!(out_edit.ok, "{:?}", out_edit.non_fast_path_reason);
    let (out2, _) = s.run(&ExecLimits::default(), &cancel);
    let ExecOutcome::Trap {
        span: Some(sp2), ..
    } = out2
    else {
        panic!("expected trap after edit, got {out2:?}")
    };
    let (l2, c2) = s.line_col(sp2.start as u64);
    assert_eq!(
        (l2, c2),
        (1, c1 + 11),
        "trap span shifts with the edit ('var b = 2; ' is 11 bytes)"
    );
}

/// T-08 / V-09: gate negatives — each axis reddens on its own. A zero
/// threshold, a short series (missing samples), and a forced whole-module
/// recompute (simulated by a structural snapshot that counts unchanged
/// stage work) must each fail the gate.
#[test]
fn v09_gate_negatives_redden_independently() {
    use ace_av::formal_828::{validate_and_gate, FormalReport, FormalSample, StructuralSnapshot};
    // A minimal clean report.
    let clean = |unaffected: u64, samples: usize| -> FormalReport {
        FormalReport {
            samples: (1..=samples)
                .map(|i| FormalSample {
                    revision: i as u32,
                    edit_to_result_ms: 10.0,
                    expected: i as i32,
                    structural: StructuralSnapshot {
                        unaffected_stage_total: unaffected,
                        whole_module_total: 0,
                        install_set_len: 1,
                        reparsed_decls: 1,
                        reused_decls: 114,
                    },
                })
                .collect(),
            p50_ms: 10.0,
            p90_ms: 10.0,
            p95_ms: 10.0,
            p99_ms: 10.0,
            all_correct: true,
            peak_slot_bytes: 0,
        }
    };
    // Clean report passes.
    assert!(validate_and_gate(&clean(0, 30), 1000.0, 30).is_empty());
    // (a) zero threshold must fail by construction.
    assert!(!validate_and_gate(&clean(0, 30), 0.0, 30).is_empty());
    // (b) missing samples.
    let failures = validate_and_gate(&clean(0, 5), 1000.0, 30);
    assert!(failures.iter().any(|f| f.contains("insufficient samples")));
    // (c) forced whole-module recompute — the structural axis reddens even
    // though timing is green (a timing pass can never cover granularity).
    let failures = validate_and_gate(&clean(7, 30), 1000.0, 30);
    assert!(
        failures
            .iter()
            .any(|f| f.contains("unaffected stage total")),
        "{failures:?}"
    );
    // (d) timing axis alone: a slow-but-granular run still fails its P90
    // (the r9 ruling gates P90; P95 stays a reported-for-the-record axis).
    let mut slow = clean(0, 30);
    for s in slow.samples.iter_mut() {
        s.edit_to_result_ms = 1500.0;
    }
    slow.p50_ms = 1500.0;
    slow.p90_ms = 1500.0;
    slow.p95_ms = 1500.0;
    slow.p99_ms = 1500.0;
    let failures = validate_and_gate(&slow, 1000.0, 30);
    assert!(failures.iter().any(|f| f.contains("P90")));
}

#[test]
fn debug_changed_fn_calls_another() {
    let src = "fn leaf(v int) int { return v + 1 }
fn mid(x int) int { return leaf(x) * 2 }
fn main() int { return mid(5) }
";
    let mut s = warmup(src, "d_main", &[]).expect("warmup");
    let (r, _) = s.run(&ExecLimits::default(), &CancelFlag::new(false));
    assert_i32(&r, 12, "warm-up (leaf(5)+1)*2");
    let at = src.find("* 2").expect("site") + 2;
    let out = s
        .edit(at as u64, (at + 1) as u64, b"3".to_vec())
        .expect("edit");
    assert!(out.ok, "{:?}", out.non_fast_path_reason);
}

// ---------------------------------------------------------------------------
// PLAN-828 T-03: function-level HIR artifacts (A07 function units)
// ---------------------------------------------------------------------------

/// T-03 roundtrip: warm-up deposits every function's HIR artifact; one
/// edit deposits exactly ONE new row (only the changed unit produces new
/// products — unchanged content conflicts and is not counted); a warm
/// restart re-admits the ABC units AND adopts the function-HIR artifacts
/// after re-verification, and the restored body objects serve later
/// content hits by IDENTITY.
#[test]
fn a07_fn_hir_roundtrip_deposit_and_adopt() {
    use ace_av::session::{restore, warmup_persisted};
    let dir = std::env::temp_dir().join(format!("p828-t03-rr-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let cancel = CancelFlag::new(false);

    let mut s = warmup_persisted(SRC, "d_main", &[], &dir, "p828").expect("warmup-persisted");
    assert_eq!(
        s.counters().get("hir_artifacts_deposited"),
        2,
        "cold deposit writes every function's HIR artifact"
    );
    let (r0, _) = s.run(&ExecLimits::default(), &cancel);
    assert_i32(&r0, 23, "warm-up run");

    // One body edit: exactly ONE new HIR artifact (the changed unit).
    let star3 = SRC.find("x * 3").expect("target");
    let pos = (star3 + 4) as u64;
    let out = s.edit(pos, pos + 1, b"4".to_vec()).expect("edit");
    assert!(out.ok, "{:?}", out.non_fast_path_reason);
    assert_eq!(
        s.counters().get("hir_artifacts_deposited"),
        3,
        "only the changed unit produces a new HIR artifact"
    );
    let edited_src = s.src().to_string();

    // Warm restart over the EDITED source: units re-admitted AND both HIR
    // artifacts adopted after re-verification.
    let mut r = restore(&edited_src, "d_main", &[], &dir, "p828").expect("restore");
    assert_eq!(r.counters().get("restore_units_admitted"), 2);
    assert_eq!(r.counters().get("restore_hir_adopted"), 2);
    let (rr, _) = r.run(&ExecLimits::default(), &cancel);
    assert_i32(&rr, 28, "restored run");
    // Unchanged functions stayed untouched by the restore+run.
    for (stage, count) in r.counters().stages_of("main") {
        assert_eq!(count, 0, "main stage `{stage}` after restore");
    }

    // The ADOPTED body object serves a later content hit by identity:
    // helper 4 -> 5 (miss), then back to 4 (hit on the restored content)
    // reuses the very object the artifact decoded into.
    let (_, hir_restored, _) = r.resident("helper").expect("resident");
    let pos4 = edited_src.find("x * 4").expect("edited target") + 4;
    let out5 = r
        .edit(pos4 as u64, (pos4 + 1) as u64, b"5".to_vec())
        .expect("edit 4->5");
    assert!(out5.ok, "{:?}", out5.non_fast_path_reason);
    let src5 = r.src().to_string();
    let pos5 = src5.find("x * 5").expect("target") + 4;
    let out4 = r
        .edit(pos5 as u64, (pos5 + 1) as u64, b"4".to_vec())
        .expect("edit 5->4");
    assert!(out4.ok, "{:?}", out4.non_fast_path_reason);
    let (_, hir_now, _) = r.resident("helper").expect("resident");
    assert!(
        Arc::ptr_eq(&hir_restored, &hir_now),
        "the content hit reuses the artifact-adopted body object"
    );
    let (rfinal, _) = r.run(&ExecLimits::default(), &cancel);
    assert_i32(&rfinal, 28, "final run");
    let _ = std::fs::remove_dir_all(&dir);
}

/// T-03 trust boundary: a pre-existing garbage row under the exact HIR
/// key survives (the store has no rewrite; the session's own deposit
/// conflicts and is tolerated) — restore must treat it as an UNTRUSTED
/// description, fail the decode, skip the artifact and keep the cold
/// body. ABC admission is unaffected. Codec-level negatives included.
#[test]
fn a07_fn_hir_tampered_row_is_skipped_not_trusted() {
    use ace_av::cache::{
        canonical_hir_function, caps_fingerprint, open_store, put_enveloped, EnvelopeIdentity,
    };
    use ace_av::fn_hir_codec::{decode_fn_hir, fn_hir_key};
    use ace_av::function_queries::FnEnv;
    use ace_av::semrefs::content_version_of;
    use ace_av::session::{restore, warmup_persisted};
    use ace_core::source::{compile_scalar_source, SourceProfile};

    // Codec negatives first: untrusted bytes never decode.
    assert!(decode_fn_hir(b"").is_err(), "empty payload");
    assert!(decode_fn_hir(b"XXXX  ").is_err(), "bad magic");
    let mut trunc = Vec::new();
    trunc.extend_from_slice(b"FNH1");
    trunc.extend_from_slice(&1u32.to_le_bytes());
    assert!(decode_fn_hir(&trunc).is_err(), "truncated header");

    // Learn the exact HIR key for d_helper's cold content (owner is the
    // module id — deterministic for this source).
    let compiled = compile_scalar_source(SRC, "probe.at", &[]).expect("compile");
    let bundle = compiled.checked.bundle();
    let m = bundle.modules.first().expect("module");
    let owner = FnEnv::from_bundle(bundle).expect("fn env").owner;
    let def_idx = m.defs.iter().position(|d| d.id_text == "d_helper").unwrap();
    let body = match &m.defs[def_idx].kind {
        ace_core::hir::DefKind::Function { body, .. } => *body,
        _ => panic!("not a function"),
    };
    let cv = content_version_of(&canonical_hir_function(m, body));
    let caps_fp = caps_fingerprint(&[]);
    let key = fn_hir_key(&owner, "d_helper", &cv, &caps_fp);
    let expect = EnvelopeIdentity::fn_hir_bound(&owner, "d_helper", &cv);

    // A fresh store with GARBAGE pre-written under that exact key.
    let dir = std::env::temp_dir().join(format!("p828-t03-tamper-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    {
        let store = open_store(&dir).expect("open");
        put_enveloped(
            &store,
            &key,
            "A07",
            &expect,
            b"definitely-not-a-fnhir-payload",
            vec![],
            Some("p828"),
        )
        .expect("pre-write garbage");
    }

    // The session's own deposit for d_helper conflicts (tolerated): only
    // d_main's HIR artifact lands.
    let s = warmup_persisted(SRC, "d_main", &[], &dir, "p828").expect("warmup-persisted");
    assert_eq!(s.counters().get("hir_artifacts_deposited"), 1);
    drop(s);

    // Restore: the garbage row decodes to nothing — skipped, never
    // trusted; the cold body keeps serving; ABC admission is unaffected.
    let cancel = CancelFlag::new(false);
    let r = restore(SRC, "d_main", &[], &dir, "p828").expect("restore");
    assert_eq!(r.counters().get("restore_units_admitted"), 2);
    assert_eq!(
        r.counters().get("restore_hir_adopted"),
        1,
        "the tampered HIR row is skipped; main's clean row adopted"
    );
    let (out, _) = r.run(&ExecLimits::default(), &cancel);
    assert_i32(&out, 23, "cold body keeps serving");
    let _ = std::fs::remove_dir_all(&dir);

    // Distinguishing negative #2: a row that decodes CLEANLY, carries the
    // right body/content/spans/caps — and WRONG dependency facts (a forged
    // declaration-context fingerprint). Content alone must not adopt it.
    use ace_av::fn_hir_codec::{
        encode_fn_hir, fn_hir_span_digest, normalize_fn_hir_body, FnHirArtifact,
    };
    let dir = std::env::temp_dir().join(format!("p828-t03-ctx-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    // codec v2: the forged row's body must be normalized through the
    // producing (cold) tables so it DECODES cleanly — the only wrong
    // thing is the context fingerprint.
    let cold_defs: Vec<String> = m.defs.iter().map(|d| d.id_text.clone()).collect();
    let cold_types: Vec<String> = m.types.iter().map(|t| t.id_text.clone()).collect();
    let (norm_body, callee_table, type_table) =
        normalize_fn_hir_body(&cold_defs, &cold_types, &m.bodies[body.0 as usize])
            .expect("normalize");
    let forged = FnHirArtifact {
        def_id: "d_helper".to_string(),
        content_version: cv.clone(),
        span_digest: fn_hir_span_digest(&m.bodies[body.0 as usize]),
        context_fingerprint: "0000000000000000".to_string(), // forged context
        caps_fp: caps_fp.clone(),
        intrinsics: Vec::new(),
        callee_table,
        type_table,
        body: norm_body,
    };
    let forged_bytes = encode_fn_hir(&forged).expect("the forged row encodes cleanly");
    assert!(
        decode_fn_hir(&forged_bytes).is_ok(),
        "it is a VALID artifact payload — only the context is wrong"
    );
    {
        let store = open_store(&dir).expect("open");
        put_enveloped(
            &store,
            &key,
            "A07",
            &expect,
            &forged_bytes,
            vec![],
            Some("p828"),
        )
        .expect("pre-write forged row");
    }
    let s = warmup_persisted(SRC, "d_main", &[], &dir, "p828").expect("warmup-persisted");
    assert_eq!(s.counters().get("hir_artifacts_deposited"), 1);
    drop(s);
    let r = restore(SRC, "d_main", &[], &dir, "p828").expect("restore");
    assert_eq!(
        r.counters().get("restore_hir_adopted"),
        1,
        "a clean-decoding row with WRONG dependency facts is not adopted"
    );
    let (out, _) = r.run(&ExecLimits::default(), &cancel);
    assert_i32(&out, 23, "cold body keeps serving");
    let _ = std::fs::remove_dir_all(&dir);
    let _ = SourceProfile::Scalar;
}

// Formal campaigns run only through the explicit performance namespace.
mod performance {
    use super::*;

    /// T-08 / V-08: the formal dual gate on the frozen load — 30 distinct
    /// fn_037 revisions through the REAL resident session, edit→verified
    /// result P95 < 1000 ms AND every structural gate green AND every result
    /// matching the independent expected.
    #[test]
    fn v08_formal_and_structural_dual_gate() {
        let report = ace_av::formal_828::run_formal_series(30).expect("formal series");
        assert_eq!(report.samples.len(), 30);
        for s in &report.samples {
            assert_eq!(
                s.structural.unaffected_stage_total, 0,
                "revision {}: some unchanged function did stage work",
                s.revision
            );
            assert_eq!(s.structural.whole_module_total, 0);
            assert_eq!(s.structural.install_set_len, 1);
            assert_eq!(s.structural.reparsed_decls, 1, "exactly fn_037 re-parsed");
            assert!(s.structural.reused_decls >= 114, "the other decls graft");
        }
        let failures = ace_av::formal_828::validate_and_gate(&report, 1000.0, 30);
        assert!(
            failures.is_empty(),
            "dual gate failures: {failures:?} (p50={:.1} p95={:.1} p99={:.1}ms)",
            report.p50_ms,
            report.p95_ms,
            report.p99_ms
        );
    }

    /// T-07 / V-07 flagship: 512 REAL body edits on the frozen load — every
    /// revision's current result matches its independent expectation, the
    /// structural gate stays green on every single revision (unchanged
    /// functions do zero stage work; no whole-module axis moves), and the
    /// resident content history stays BOUNDED (HISTORY_CAP per function —
    /// the watermark converges instead of growing with the edit count).
    #[test]
    fn v07_512_revisions_bounded_history_stable_watermark() {
        use ace_av::formal_828::run_formal_series_retained;
        let (report, s) = run_formal_series_retained(512).expect("the 512-revision formal series");
        assert_eq!(report.samples.len(), 512);
        assert!(
            report.all_correct,
            "every revision verified against its own expectation"
        );
        for x in &report.samples {
            assert_eq!(
                x.structural.unaffected_stage_total, 0,
                "revision {}: unchanged functions must do zero stage work",
                x.revision
            );
            assert_eq!(
                x.structural.whole_module_total, 0,
                "revision {}: no whole-module axis may move",
                x.revision
            );
            assert_eq!(x.structural.reparsed_decls, 1, "revision {}", x.revision);
        }
        // Watermark: 115 functions × HISTORY_CAP(32) is the structural bound;
        // the changed function's own history is capped (≈114×1 + 32 ≪ 513 —
        // eviction really ran, the history did NOT grow with the edit count).
        assert!(
            s.history_len() <= 32 * 115,
            "resident history bounded: {}",
            s.history_len()
        );
        assert!(
            s.history_len() < 513,
            "the 512 edits did NOT accumulate 512+ history entries (eviction ran): {}",
            s.history_len()
        );
    }

    /// T-07 / V-07 COMPLETE storage axis (512 real edits + GC + restart on
    /// the frozen load): a PERSISTED session runs the frozen 512-edit series,
    /// every revision persisted changed-only and every value verified against
    /// its independent expectation. Periodic REAL GC reclaims each superseded
    /// revision's closure — the store's live-set watermark stays BOUNDED (it
    /// does not grow with the edit count; the per-function HIR soft root
    /// retires at supersession, and a restart seeds the tracker so the first
    /// post-restart supersession still retires). Two mid-series warm
    /// restarts re-verify and re-admit every unit from the store. The pinned
    /// revision-0 snapshot stays runnable for the whole series and dies on
    /// release; a user sentinel row survives every GC byte-identical.
    #[test]
    fn v07_full_storage_axis_512_edits_gc_restart_sentinel() {
        use ace_artifacts::identity::QueryKey;
        use ace_artifacts::store::{run_gc, GcConfig, PutRequest};
        use ace_av::cache::{open_store, ABC_LAYER};
        use ace_av::formal_828::diff_edit;
        use ace_av::session::{restore, warmup_persisted, HISTORY_CAP};

        const PROJECT: &str = "p828-t07-full-axis";
        let dir = std::env::temp_dir().join(format!("p828-t07-full512-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        // A user-owned row outside the session's key space, pinned by its
        // OWN root kind — it must survive every GC byte-identical.
        let sentinel_key = QueryKey::new(
            &ABC_LAYER,
            vec![
                ("owner".into(), "user-cache-sentinel".into()),
                ("kind".into(), "user-row".into()),
                ("fp".into(), "v1".into()),
            ],
        );
        let sentinel_payload = b"user-sentinel-payload-v1".to_vec();
        {
            let st = open_store(&dir).unwrap();
            let id = st
                .put_artifact(PutRequest {
                    layer: "A09",
                    key: sentinel_key.clone(),
                    kind: ace_artifacts::codec::ChunkKind::LayoutData,
                    payload: sentinel_payload.clone(),
                    edges: vec![],
                    project: None,
                })
                .unwrap();
            st.register_root(PROJECT, &id, "user-row").unwrap();
        }

        let base = load256::generate_revision(0);
        let mut s = warmup_persisted(&base.source, "d_main", &[], &dir, PROJECT).unwrap();
        let pinned = s.current();
        let pinned_weak = Arc::downgrade(&pinned);
        let cancel = CancelFlag::new(false);

        // The live-set reference point: the initial closure (per-unit ABC +
        // sidecar + HIR, the manifest, and the sentinel).
        let initial_live = {
            let st = open_store(&dir).unwrap();
            run_gc(&st, GcConfig { high: 1, low: 0 })
                .unwrap()
                .live_chunks
        };

        let mut prev_src = base.source.clone();
        let mut live_watermark = initial_live;
        let mut max_soft_roots = 0usize;
        let mut total_deleted = 0usize;
        let mut restarts = 0usize;
        let functions = s.function_names().len();
        for n in 1..=512u32 {
            let rev = load256::generate_revision(n);
            let (start, end, insert) = diff_edit(&prev_src, &rev.source).unwrap();
            let out = s.edit(start, end, insert).unwrap();
            assert!(out.ok, "revision {n}: {:?}", out.non_fast_path_reason);
            assert_eq!(out.install_set.len(), 1, "revision {n}");
            assert!(
                out.install_set[0].ends_with("fn_037"),
                "revision {n}: {:?}",
                out.install_set
            );
            let (exec, _) = s.run(&ExecLimits::default(), &cancel);
            assert_i32(&exec, rev.expected_main, "revision {n}");
            prev_src = rev.source;

            if n % 64 == 0 {
                let st = open_store(&dir).unwrap();
                let receipt = run_gc(&st, GcConfig { high: 1, low: 0 }).unwrap();
                assert!(receipt.ran, "revision {n}: forced GC ran");
                total_deleted += receipt.deleted_chunks;
                live_watermark = live_watermark.max(receipt.live_chunks);
                let soft = st.root_artifacts(PROJECT, "soft").unwrap().len();
                max_soft_roots = max_soft_roots.max(soft);
            }
            if n == 171 || n == 342 {
                // Mid-series warm restart: the manifest gate matches, every
                // unit re-verifies and re-admits from the store, and the
                // series continues on the restored session.
                let src_now = s.src().to_string();
                drop(s);
                let r = restore(&src_now, "d_main", &[], &dir, PROJECT).unwrap();
                assert_eq!(
                    r.counters().get("restore_manifest_matched"),
                    1,
                    "warm restart at {n}"
                );
                assert_eq!(
                    r.counters().get("restore_units_admitted") as usize,
                    functions,
                    "every unit warm-admitted at {n}"
                );
                s = r;
                restarts += 1;
            }
        }
        assert_eq!(restarts, 2, "two mid-series restarts happened");

        // Bounded store: real GC reclaimed the superseded closures and the
        // live watermark stayed at the initial closure's scale — never a
        // growth linear in the 512 edits.
        assert!(total_deleted > 0, "periodic GC actually reclaimed payloads");
        assert!(
            live_watermark <= initial_live + 8,
            "live watermark {live_watermark} vs initial closure {initial_live}"
        );
        // Bounded HIR soft roots: at most one live root per function (its
        // current content version) across all 512 supersessions.
        assert!(
            max_soft_roots <= functions,
            "HIR soft roots bounded by the function count: {max_soft_roots}"
        );
        // Bounded resident history (the in-memory axis of the same series).
        assert!(
            s.history_len() <= HISTORY_CAP * functions && s.history_len() < 513,
            "resident history {}",
            s.history_len()
        );

        // The pinned revision-0 snapshot stayed runnable for the whole
        // series (old code available while held) and dies on release.
        let (exec, _) = execute_program(&pinned, &ExecLimits::default(), &cancel);
        assert_i32(
            &exec,
            base.expected_main,
            "pinned revision-0 after the series",
        );
        drop(pinned);
        assert!(
            pinned_weak.upgrade().is_none(),
            "releasing the pin reclaims the old snapshot"
        );

        // The sentinel survived every GC byte-identical.
        {
            let st = open_store(&dir).unwrap();
            let loaded = st
                .get_artifact("A09", &sentinel_key)
                .unwrap()
                .expect("user sentinel alive after every GC");
            assert_eq!(loaded.payload, sentinel_payload);
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
