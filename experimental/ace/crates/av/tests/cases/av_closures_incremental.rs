//! PLAN-823 T-09 V-13 tests: the closures update consumer over the D828
//! unit/snapshot seam (r8 env contract — the closure BODY is a unit with
//! a declared env; the CREATOR unit binds it through an import carrying
//! the same declaration; link checks the equivalence).
//! * sub-case 2 — a body-only revision rebuilds EXACTLY the submitted
//!   unit; the carried creator's code object and production/IO stay at
//!   zero; semantics + cleanup stay correct;
//! * sub-case 3 — an env-layout drift on either side is a PRECISE link
//!   refusal (never a silent mismatched run);
//! * sub-case 4 — re-submitting an identical module (content unchanged,
//!   fresh object) reads degraded-unchanged: the gate is NON-ZERO.
//! Sub-case 1 (the D828 source-edit baseline, 273575B/115 funcs/30
//! revisions) is 828's own delivered corpus — 822 replayed it under the
//! same session machinery; this file covers the closure-capability axes.

use ace_av::closures_fixture::closures_owner_scope;
use ace_av::closures_session::ClosuresUpdateSession;
use ace_av_core::mir::{
    CalleeRef, CaptureKind, CaptureOperand, FnSig, MirBlock, MirConst, MirFunction, MirImport,
    MirInstr, MirModule, MirSlot, MirType, SlotKind, Terminator,
};
use ace_av_core::mir_verify::verify_mir;
use std::sync::Arc;

fn slot(ty: u32, name: &str) -> MirSlot {
    MirSlot {
        kind: SlotKind::Local,
        ty,
        name: Some(name.to_string()),
    }
}
fn param(ty: u32, name: &str) -> MirSlot {
    MirSlot {
        kind: SlotKind::Param,
        ty,
        name: Some(name.to_string()),
    }
}

/// types: 0 i32, 1 Str, 2 StrView, 3 Fn(i32)->i32, 4 Closure(3).
/// The body unit: `(i32)->i32`, env [Str] — returns the env text length
/// * bonus_factor (the revision knob).
fn body_unit(bonus_factor: i32) -> MirModule {
    let owner = closures_owner_scope();
    let mut m = MirModule::closures_unit(owner.clone());
    m.types = vec![
        MirType::I32,
        MirType::Str,
        MirType::StrView,
        MirType::Fn(FnSig {
            params: vec![0],
            result: 0,
        }),
        MirType::Closure(FnSig {
            params: vec![0],
            result: 0,
        }),
    ];
    let c_f = m.intern_const(MirConst::I32(bonus_factor));
    // (the revision knob uses f²; no ten needed)
    let mut f = MirFunction {
        owner,
        name: "body".to_string(),
        sig: 3,
        slots: vec![
            param(0, "bonus"),
            slot(1, "env_text"),
            slot(2, "view"),
            slot(0, "slen"),
            slot(0, "f"),
            slot(0, "mul"),
        ],
        blocks: vec![MirBlock {
            instrs: vec![
                MirInstr::EnvGet { dst: 1, field: 0 },
                MirInstr::View { dst: 2, src: 1 },
                MirInstr::StrLen { dst: 3, view: 2 },
                MirInstr::Const {
                    dst: 4,
                    value: c_f as u32,
                },
                MirInstr::Bin {
                    op: ace_av_core::mir::BinOp::MulI32,
                    dst: 5,
                    lhs: 4,
                    rhs: 4,
                },
                MirInstr::Bin {
                    op: ace_av_core::mir::BinOp::AddI32,
                    dst: 5,
                    lhs: 3,
                    rhs: 5,
                },
            ],
            term: Terminator::Return { value: 5 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: Vec::new(),
        env: None,
    };
    f.env = Some(vec![1]); // [Str]
    m.funcs = vec![f];
    m.entry = 0;
    m
}

/// The creator unit: `()->i32` — imports `fn:body` (env [Str]), mints an
/// owned Str, CREATE_CLOSURE via the import, calls, drops.
fn main_unit() -> MirModule {
    let owner = closures_owner_scope();
    let mut m = MirModule::closures_unit(owner.clone());
    m.types = vec![
        MirType::I32,
        MirType::Str,
        MirType::StrView,
        MirType::Fn(FnSig {
            params: vec![0],
            result: 0,
        }),
        MirType::Closure(FnSig {
            params: vec![0],
            result: 0,
        }),
        MirType::Fn(FnSig {
            params: vec![],
            result: 0,
        }),
    ];
    let c_text = m.intern_const(MirConst::Str(Box::from(*b"auto")));
    let c_five = m.intern_const(MirConst::I32(5));
    let imp = m.intern_import(MirImport {
        symbol: "fn:body".to_string(),
        effect: ace_av_core::mir::Effect::Pure,
        sig: 3,
        env: Some(vec![1]), // the SAME declaration the body unit carries
    });
    m.funcs = vec![MirFunction {
        owner,
        name: "main".to_string(),
        sig: 5,
        slots: vec![
            slot(1, "text"),
            slot(4, "cl"),
            slot(0, "arg"),
            slot(0, "ret"),
        ],
        blocks: vec![MirBlock {
            instrs: vec![
                MirInstr::Const {
                    dst: 0,
                    value: c_text as u32,
                },
                MirInstr::Const {
                    dst: 2,
                    value: c_five as u32,
                },
                MirInstr::CreateClosure {
                    dst: 1,
                    callee: CalleeRef::Import(imp),
                    captures: vec![CaptureOperand {
                        kind: CaptureKind::Clone,
                        src: 0,
                    }],
                },
                MirInstr::CallClosure {
                    dst: 3,
                    closure: 1,
                    args: vec![2],
                },
                MirInstr::Drop { slot: 1 },
            ],
            term: Terminator::Return { value: 3 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: Vec::new(),
        env: None,
    }];
    m.entry = 0;
    m
}

#[test]
fn av_closures_incremental_body_edit_identity() {
    // V-13 sub-case 2: install both units, then revise ONLY the body.
    // oracle: slen("auto")=4; body ret = 4 + f²  (f=1 → 5; f=2 → 8; f=3 → 13)
    let mut s = ClosuresUpdateSession::new(None, "main");
    let out0 = s
        .apply_revision(&[("main", main_unit()), ("body", body_unit(1))])
        .expect("install");
    assert_eq!(out0.value, Some(5), "4 + 1²");
    assert_eq!(out0.live_after_release, 0);
    assert_eq!(out0.allocs, out0.frees);
    let main_obj = s.unit("main").expect("installed").clone();

    for (f, want) in [(2, 8), (3, 13)] {
        let out = s
            .apply_revision(&[("body", body_unit(f))])
            .unwrap_or_else(|e| panic!("body revision f={f}: {e}"));
        assert_eq!(out.value, Some(want), "f={f} semantics");
        assert_eq!(out.live_after_release, 0, "f={f}: ledger drains");
        // Production/IO of the UNCHANGED creator unit: zero on every stage.
        assert_eq!(out.production.verified, 1, "f={f}: only body verified");
        assert_eq!(out.production.encoded, 1);
        assert_eq!(out.production.admitted, 1);
        assert_eq!(out.gate.rebuilt_units, 1);
        assert_eq!(out.gate.unchanged_degraded, 0);
        assert!(out.gate.pass());
        assert!(
            Arc::ptr_eq(s.unit("main").unwrap(), &main_obj),
            "f={f}: the creator's code object is carried, not rebuilt"
        );
    }
}

#[test]
fn av_closures_incremental_env_contract_drift_refused() {
    // V-13 sub-case 3: a legal creator whose import declares a DIFFERENT
    // env layout links with a precise refusal — the real interface check;
    // the session keeps its prior state and a correct body still updates.
    let mut s = ClosuresUpdateSession::new(None, "main");
    s.apply_revision(&[("main", main_unit()), ("body", body_unit(1))])
        .expect("install");

    let mut bad = main_unit();
    bad.imports[0].env = Some(vec![0]); // [i32] drift — Str was declared
    let err = s
        .apply_revision(&[("main", bad)])
        .expect_err("an env-layout drift must refuse the link");
    assert!(err.contains("env-contract"), "{err}");

    // The session still works after the refusal.
    let out = s
        .apply_revision(&[("body", body_unit(2))])
        .expect("recovers");
    assert_eq!(out.value, Some(8));
}

#[test]
fn av_closures_incremental_forced_degradation_gate_nonzero() {
    // V-13 sub-case 4: re-submitting the IDENTICAL body module (content
    // fingerprint unchanged, fresh object) reads degraded-unchanged — the
    // production gate FAILS with a non-zero count.
    let mut s = ClosuresUpdateSession::new(None, "main");
    s.apply_revision(&[("main", main_unit()), ("body", body_unit(1))])
        .expect("install");
    let out = s
        .apply_revision(&[("body", body_unit(1))]) // identical content
        .expect("the identical revision still executes");
    assert_eq!(out.value, Some(5), "semantics unchanged");
    assert!(
        out.gate.unchanged_degraded >= 1,
        "the forced-whole shape is detected"
    );
    assert!(!out.gate.pass(), "the gate is NON-ZERO");
}

#[test]
fn av_closures_incremental_units_verify_standalone() {
    // The r8 contract's core claim: BOTH units pass the REAL verify gate
    // alone (the body's ENV_GET types against its OWN declaration; the
    // creator's site matches the import's declaration).
    verify_mir(&body_unit(1)).expect("body unit verifies standalone");
    verify_mir(&main_unit()).expect("creator unit verifies standalone");
}

// The user-approved P90 gate is exactly ten measurements. At least nine
// must be finite and at or below 2000ms; all ten still need correct results.
fn p90_gate_10(times: &[f64]) -> bool {
    times.len() == 10
        && times.iter().all(|v| v.is_finite() && *v >= 0.0)
        && times.iter().filter(|v| **v <= 2000.0).count() >= 9
}

#[test]
fn av_closures_incremental_threshold_controls() {
    assert!(p90_gate_10(&[2000.0; 10]));
    let mut one = [1.0; 10];
    one[9] = 2000.1;
    assert!(p90_gate_10(&one), "one outlier is permitted");
    one[8] = 2000.1;
    assert!(!p90_gate_10(&one), "two outliers must fail");
    assert!(!p90_gate_10(&[1.0; 9]), "missing sample must fail");
    one[8] = f64::NAN;
    assert!(!p90_gate_10(&one), "invalid duration must fail");
}

#[test]
fn av_closures_incremental_warm_edit_p90() {
    let mut session = ClosuresUpdateSession::new(None, "main");
    session
        .apply_revision(&[("main", main_unit()), ("body", body_unit(1))])
        .expect("warm-up");
    let mut times = Vec::new();
    for factor in 2..=11 {
        let start = std::time::Instant::now();
        let out = session
            .apply_revision(&[("body", body_unit(factor))])
            .expect("body edit");
        times.push(start.elapsed().as_secs_f64() * 1000.0);
        assert_eq!(out.value, Some(4 + factor * factor));
        assert_eq!(out.live_after_release, 0);
        assert!(out.gate.pass());
    }
    let mut sorted = times.clone();
    sorted.sort_by(f64::total_cmp);
    println!(
        "closures warm-edit: samples={times:?}, P90={:.3}ms, limit=2000ms",
        sorted[8]
    );
    assert!(
        p90_gate_10(&times),
        "at least 9/10 samples must be <= 2000ms: {times:?}"
    );
}

// R1-F05: the D828 source-edit baseline campaign replayed on THIS tree's
// frozen shared core (823 changed MIR/wire/verifier/link; the baseline
// gates are the shared machinery's own).

// R1-F02: a REAL new-process worker restores the persisted closures CODE
// artifact from the same store (fresh PID, full re-verify, executes 29).
#[test]
fn av_closures_recipe_new_process_worker_restore() {
    let dir = std::env::temp_dir().join(format!("av823-worker-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let r = ace_av::closures_fixture::request_closures(Some(&dir), "worker-probe", "first-case")
        .expect("persisted");
    let worker = env!("CARGO_BIN_EXE_av-worker");
    let out = std::process::Command::new(worker)
        .arg("closures-restore-run")
        .arg("--store")
        .arg(&dir)
        .arg("--owner")
        .arg(&r.owner)
        .arg("--entry")
        .arg(&r.entry)
        .arg("--snapshot")
        .arg(format!("{:016x}", r.snapshot))
        .arg("--types-fp")
        .arg(&r.types_fp)
        .output()
        .expect("spawn worker");
    assert!(
        out.status.success(),
        "new-process restore exits 0; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(stdout.contains("Success"), "got {stdout}");
    assert!(stdout.contains("I32(29)"), "got {stdout}");
    let _ = std::fs::remove_dir_all(&dir);
}

// R1-F03: the closures worker's REAL private-bytes peak (parent-sampled,
// fresh process) stays under the 798 B-line 128MiB envelope; the managed
// partitions hold their own dimensions.
#[test]
fn av_closures_worker_memory_hard_gate_parent_sampled() {
    use std::process::{Command, Stdio};
    let dir = std::env::temp_dir().join(format!("av823-mem-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let worker = env!("CARGO_BIN_EXE_av-worker");
    // Same worker held idle while the parent samples its private bytes.
    let mut idle = Command::new(worker)
        .args(["data-mem-envelope", "--rounds", "0", "--hold-ms", "400"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("baseline spawn");
    let idle_sampler =
        ace_av::mempeek::ChildPeakSampler::start(idle.id(), 2).expect("sample idle worker");
    assert!(idle.wait().expect("idle exits").success());
    let (idle_peak, idle_samples) = idle_sampler.stop();
    assert!(
        idle_samples > 0 && idle_peak > 0,
        "real idle samples required"
    );
    let child = Command::new(worker)
        .args([
            "closures-request",
            "--store",
            dir.join("s1").to_str().unwrap(),
            "--fixture",
            "first-case",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn closures worker");
    let sampler = ace_av::mempeek::ChildPeakSampler::start(child.id(), 5)
        .expect("the parent samples the child");
    let out = child.wait_with_output().expect("child exits");
    let (peak, samples) = sampler.stop();
    assert!(out.status.success(), "worker ok");
    assert!(samples > 0, "sampler polled");
    let payload: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).expect("one JSON line");
    let hot = payload["peak_hot_bytes"].as_u64().expect("hot");
    let staging = payload["peak_staging_bytes"].as_u64().expect("staging");
    let exec = payload["peak_exec_slot_bytes"].as_u64().expect("exec");
    // Every partition has its own bound; B-line is the measured delta.
    let evidence = [peak.saturating_sub(idle_peak), hot, staging, exec];
    let limits = [
        128 * 1024 * 1024,
        8 * 1024 * 1024,
        16 * 1024 * 1024,
        8 * 1024 * 1024,
    ];
    let within = |values: [u64; 4]| values.iter().zip(limits).all(|(v, limit)| *v <= limit);
    println!("idle={idle_peak} samples={idle_samples}; load={peak} samples={samples}; delta/hot/staging/exec={evidence:?}");
    assert!(
        within(evidence),
        "each dimension must stay within its own budget"
    );
    assert!(
        hot > 0 && staging > 0 && exec > 0,
        "partition evidence required"
    );
    for i in 0..limits.len() {
        let mut over = evidence;
        over[i] = limits[i] + 1;
        assert!(
            !within(over),
            "budget line {i} must reject an over-limit sample"
        );
    }
    let _ = std::fs::remove_dir_all(&dir);
}

// I07: the supported AV closure chain holds its old function snapshot
// while a new revision is installed.  Native online loading is separate.
#[test]
fn av_closures_incremental_paused_call_pins_old_body() {
    use ace_av_core::av::{
        execute_program_steps, resume_program, CancelFlag, ExecLimits, SteppedOutcome,
    };
    use ace_av_core::contract::{ExecOutcome, Value};

    let mut session = ClosuresUpdateSession::new(None, "main");
    session
        .apply_revision(&[("main", main_unit()), ("body", body_unit(1))])
        .expect("initial install");
    let old_body = Arc::downgrade(session.unit("body").unwrap());
    let old_snapshot = Arc::new(session.snapshot().expect("old snapshot"));
    let cancel = CancelFlag::new(false);
    let mut stepped = execute_program_steps(&old_snapshot, &ExecLimits::default(), &cancel, 1);
    let paused = loop {
        let SteppedOutcome::Paused(run) = stepped else {
            panic!("finished before entering closure body")
        };
        if matches!(run.active_closure_frame(), Some(("body", _))) {
            break run;
        }
        assert!(run.step_count() < 1000, "closure body was never entered");
        stepped = resume_program(run, &cancel, Some(1));
    };
    assert!(
        paused.arena().live() > 0,
        "published captured value must be live"
    );
    drop(old_snapshot);
    let new = session
        .apply_revision(&[("body", body_unit(2))])
        .expect("new revision");
    assert_eq!(new.value, Some(8), "new callers observe the new body");
    assert_eq!(new.live_after_release, 0);
    assert!(old_body.upgrade().is_some(), "paused chain pins old code");
    session.close().expect("close");
    session.close().expect("close is idempotent");
    assert!(
        session.snapshot().is_err(),
        "closed session rejects snapshots"
    );
    assert!(
        session.apply_revision(&[("body", body_unit(1))]).is_err(),
        "closed session rejects edits"
    );
    assert!(
        old_body.upgrade().is_some(),
        "paused call keeps code alive after close"
    );
    let (out, trace) = match resume_program(paused, &cancel, None) {
        SteppedOutcome::Finished(out, trace) => (out, trace),
        SteppedOutcome::Paused(_) => panic!("old call must finish"),
    };
    assert!(
        matches!(
            out,
            ExecOutcome::Success {
                value: Value::I32(5),
                ..
            }
        ),
        "old call observes old body: {out:?}"
    );
    assert_eq!(
        trace.arena.expect("ledger").live(),
        0,
        "old environment is released"
    );
    assert!(
        old_body.upgrade().is_none(),
        "old body is released after session and last pin"
    );
}

// I05: a real historical content revisit must hit its admitted code object.
#[test]
fn av_closures_incremental_history_revisit_hits() {
    let mut session = ClosuresUpdateSession::new(None, "main");
    let initial = session
        .apply_revision(&[("main", main_unit()), ("body", body_unit(1))])
        .expect("initial");
    assert_eq!(initial.value, Some(5));
    let historical = session.unit("body").unwrap().clone();
    assert_eq!(
        session
            .apply_revision(&[("body", body_unit(2))])
            .expect("edit")
            .value,
        Some(8)
    );
    let revisit = session
        .apply_revision(&[("body", body_unit(1))])
        .expect("revisit");
    assert_eq!(revisit.value, Some(5));
    assert_eq!(revisit.live_after_release, 0);
    assert!(
        Arc::ptr_eq(&historical, session.unit("body").unwrap()),
        "revisit must reuse the historical code object"
    );
    assert_eq!(
        revisit.production.admitted, 0,
        "content hit must not re-admit"
    );
    assert!(revisit.gate.pass());
}

#[test]
fn av_closures_incremental_history_bound_and_failed_publish() {
    let mut session = ClosuresUpdateSession::new(None, "main");
    session
        .apply_revision(&[("main", main_unit()), ("body", body_unit(1))])
        .expect("initial");
    let oldest = Arc::downgrade(session.unit("body").unwrap());
    for factor in 2..=35 {
        let out = session
            .apply_revision(&[("body", body_unit(factor))])
            .expect("distinct revision");
        assert_eq!(out.value, Some(4 + factor * factor));
        assert!(out.gate.pass());
    }
    assert!(
        oldest.upgrade().is_none(),
        "per-unit history evicts the oldest code"
    );
    let retained = session.unit("body").unwrap().clone();
    let mut bad = main_unit();
    bad.imports[0].env = Some(vec![0]);
    assert!(
        session.apply_revision(&[("main", bad)]).is_err(),
        "invalid link must fail"
    );
    assert!(
        Arc::ptr_eq(&retained, session.unit("body").unwrap()),
        "failed publication preserves current code"
    );
    let out = session
        .apply_revision(&[("body", body_unit(34))])
        .expect("history after failed publication");
    assert_eq!(out.value, Some(4 + 34 * 34));
    assert_eq!(
        out.production.admitted, 0,
        "history survives failed publication"
    );
    assert!(out.gate.pass());
}

#[test]
fn av_closures_incremental_store_history_revisit() {
    let dir = std::env::temp_dir().join(format!("av823-history-store-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("store directory");
    let mut session = ClosuresUpdateSession::new(Some(&dir), "main");
    let initial = session
        .apply_revision(&[("main", main_unit()), ("body", body_unit(1))])
        .expect("initial store install");
    assert_eq!(initial.value, Some(5));
    let first = session.unit("body").unwrap().clone();
    let changed = session
        .apply_revision(&[("body", body_unit(2))])
        .expect("changed body store install");
    assert_eq!(changed.value, Some(8));
    let revisit = session
        .apply_revision(&[("body", body_unit(1))])
        .expect("historical store revisit");
    assert_eq!(revisit.value, Some(5));
    assert!(Arc::ptr_eq(&first, session.unit("body").unwrap()));
    assert_eq!(revisit.production.admitted, 0);
    assert_eq!(revisit.production.store_writes, 0);
    drop(session);
    let _ = std::fs::remove_dir_all(&dir);
}

// History shares the hot budget with current code: sizeable admitted units
// force eviction before the per-function 32-entry bound is reached.
#[test]
fn av_closures_incremental_history_hot_budget_evicts() {
    fn large_body(factor: i32) -> MirModule {
        let mut body = body_unit(factor);
        body.intern_const(MirConst::Str(vec![b'x'; 300_000].into_boxed_slice()));
        body
    }
    let mut session = ClosuresUpdateSession::new(None, "main");
    session
        .apply_revision(&[("main", main_unit()), ("body", large_body(1))])
        .expect("initial");
    let first = Arc::downgrade(session.unit("body").unwrap());
    let mut recent = None;
    for factor in 2..=18 {
        let out = session
            .apply_revision(&[("body", large_body(factor))])
            .expect("bounded edit");
        assert_eq!(out.value, Some(4 + factor * factor));
        if factor == 17 {
            recent = Some(Arc::downgrade(session.unit("body").unwrap()));
        }
    }
    assert!(
        first.upgrade().is_none(),
        "hot budget evicts the old large unit"
    );
    let expected = recent
        .unwrap()
        .upgrade()
        .expect("recent entry stays cached");
    let out = session
        .apply_revision(&[("body", large_body(17))])
        .expect("recent revisit");
    assert_eq!(out.value, Some(293));
    assert_eq!(out.production.admitted, 0);
    assert!(Arc::ptr_eq(&expected, session.unit("body").unwrap()));
}

#[test]
fn av_closures_incremental_store_failed_publish_does_not_root_candidate() {
    let dir = std::env::temp_dir().join(format!("av823-failed-store-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let mut session = ClosuresUpdateSession::new(Some(&dir), "main");
    session
        .apply_revision(&[("main", main_unit()), ("body", body_unit(1))])
        .expect("initial");
    let project = session.store_project().to_owned();
    let store = ace_artifacts::store::ManagedStore::open(&dir).expect("store");
    let mut before = store.root_artifacts(&project, "soft").expect("roots");
    before.sort();
    let count_before = ace_artifacts::store::gc::artifact_count(&store).expect("artifact count");
    let mut bad = body_unit(2);
    bad.funcs[0].env = Some(vec![1, 0]); // standalone legal; incompatible with creator import
    verify_mir(&bad).expect("candidate must reach the link step");
    let error = session
        .apply_revision(&[("body", bad)])
        .expect_err("incompatible candidate must fail link");
    assert!(error.contains("env-contract"), "must fail at link: {error}");
    let mut after = store
        .root_artifacts(&project, "soft")
        .expect("roots after refusal");
    after.sort();
    assert_eq!(
        after, before,
        "a failed link cannot publish a candidate root"
    );
    let receipt =
        ace_artifacts::store::run_gc(&store, ace_artifacts::store::GcConfig { high: 1, low: 0 })
            .expect("GC");
    assert!(receipt.ran);
    assert_eq!(
        ace_artifacts::store::gc::artifact_count(&store).unwrap(),
        count_before,
        "unrooted candidate is reclaimable"
    );
    for id in &before {
        assert!(
            store.get_artifact_by_id(id).unwrap().is_some(),
            "prior roots survive GC"
        );
    }

    drop(store);
    drop(session);
    let _ = std::fs::remove_dir_all(&dir);
}

// I10: real persisted edits and periodic GC; the session owns at most
// 32 historical bodies, the current body, and the unchanged creator.

#[test]
fn av_closures_incremental_shared_store_session_roots_are_independent() {
    use ace_artifacts::store::{run_gc, GcConfig, ManagedStore};
    let dir = std::env::temp_dir().join(format!("av823-two-sessions-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let mut first = ClosuresUpdateSession::new(Some(&dir), "main");
    let mut second = ClosuresUpdateSession::new(Some(&dir), "main");
    for session in [&mut first, &mut second] {
        assert_eq!(
            session
                .apply_revision(&[("main", main_unit()), ("body", body_unit(1))])
                .unwrap()
                .value,
            Some(5)
        );
    }
    let project_a = first.store_project().to_owned();
    let project_b = second.store_project().to_owned();
    assert_ne!(project_a, project_b);
    let store = ManagedStore::open(&dir).unwrap();
    let roots_b = store.root_artifacts(&project_b, "soft").unwrap();
    assert_eq!(
        roots_b.len(),
        2,
        "idempotent writes still acquire the second session's roots"
    );
    first.close().unwrap();
    assert!(store.root_artifacts(&project_a, "soft").unwrap().is_empty());
    assert!(run_gc(&store, GcConfig { high: 1, low: 0 }).unwrap().ran);
    for id in &roots_b {
        assert!(
            store.get_artifact_by_id(id).unwrap().is_some(),
            "second session survives first close and GC"
        );
    }
    assert_eq!(
        second
            .apply_revision(&[("body", body_unit(2))])
            .unwrap()
            .value,
        Some(8)
    );
    second.close().unwrap();
    drop(store);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn av_closures_incremental_dropped_session_releases_roots() {
    let dir = std::env::temp_dir().join(format!("av823-drop-session-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let store = ace_artifacts::store::ManagedStore::open(&dir).unwrap();
    let project;
    {
        let mut session = ClosuresUpdateSession::new(Some(&dir), "main");
        project = session.store_project().to_owned();
        assert_eq!(
            session
                .apply_revision(&[("main", main_unit()), ("body", body_unit(1))])
                .unwrap()
                .value,
            Some(5)
        );
        assert_eq!(store.root_artifacts(&project, "soft").unwrap().len(), 2);
    }
    assert!(
        store.root_artifacts(&project, "soft").unwrap().is_empty(),
        "ordinary drop releases session roots"
    );
    drop(store);
    let _ = std::fs::remove_dir_all(&dir);
}

// Explicit formal benchmark or sustained scale/stress contract.
mod performance {
    use super::*;

    /// I10: 512 real body revisions exercise the session and cleanup gates.
    /// I11 is measured separately with the user-approved 10-sample P90 gate.
    #[test]
    fn av_closures_incremental_churn_512() {
        let mut s = ClosuresUpdateSession::new(None, "main");
        s.apply_revision(&[("main", main_unit()), ("body", body_unit(1))])
            .expect("install");
        for i in 0..512 {
            let f = (i % 7) as i32 + 2; // bodies f=2..8 — real content churn vs current
            let out = s
                .apply_revision(&[("body", body_unit(f))])
                .unwrap_or_else(|e| panic!("revision {i}: {e}"));
            assert_eq!(
                out.gate.unchanged_degraded, 0,
                "rev {i}: honest revisions only"
            );
            assert_eq!(out.live_after_release, 0, "rev {i}: ledger drains");
        }
        // I10: bounded churn — 512 revisions, all gates green, session usable.
        let out = s
            .apply_revision(&[("body", body_unit(1))])
            .expect("post-churn");
        assert_eq!(
            out.gate.rebuilt_units, 0,
            "post-churn returns to retained historical content"
        );
        assert_eq!(out.value, Some(5));
        assert!(out.gate.pass());
    }

    #[test]
    fn av_closures_incremental_store_512_revisions_gc_bounded() {
        use ace_artifacts::store::{run_gc, GcConfig, ManagedStore};
        let dir = std::env::temp_dir().join(format!("av823-store-gc-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let sentinel = dir.join("user-sentinel.txt");
        std::fs::write(&sentinel, b"do not remove").unwrap();
        let separate =
            ace_av::closures_fixture::request_closures(Some(&dir), "user-project", "first-case")
                .expect("user artifact");
        let user_id = separate.artifact.expect("user root");
        let mut session = ClosuresUpdateSession::new(Some(&dir), "main");
        assert_eq!(
            session
                .apply_revision(&[("main", main_unit()), ("body", body_unit(1))])
                .unwrap()
                .value,
            Some(5)
        );
        let project = session.store_project().to_owned();
        let store = ManagedStore::open(&dir).unwrap();
        let mut gc_watermarks = Vec::new();
        for factor in 2..=513 {
            let result = session
                .apply_revision(&[("body", body_unit(factor))])
                .expect("edit");
            assert_eq!(result.value, Some(4 + factor * factor));
            assert_eq!(result.live_after_release, 0);
            assert!(result.gate.pass());
            let roots = store.root_artifacts(&project, "soft").unwrap();
            assert!(
                roots.len() <= 34,
                "current+history roots must stay bounded: {}",
                roots.len()
            );
            if factor % 64 == 0 {
                let receipt = run_gc(&store, GcConfig { high: 1, low: 0 }).expect("GC");
                assert!(receipt.ran);
                gc_watermarks.push(receipt.total_after);
                let space = store.space();
                let lock_files = std::fs::read_dir(dir.join("locks")).unwrap().count();
                println!("GC factor={factor} lock_files={lock_files}");
                #[cfg(windows)]
                assert!(
                    lock_files <= 128,
                    "retired lock names must stay bounded: {lock_files}"
                );
                let (chunks, deleted, transactions): (i64, i64, i64) = {
                    let db = store.db();
                    (
                        db.query_row("SELECT COUNT(*) FROM chunks", [], |r| r.get(0))
                            .unwrap(),
                        db.query_row(
                            "SELECT COUNT(*) FROM chunks WHERE state='deleted'",
                            [],
                            |r| r.get(0),
                        )
                        .unwrap(),
                        db.query_row("SELECT COUNT(*) FROM txns", [], |r| r.get(0))
                            .unwrap(),
                    )
                };
                println!("GC metadata chunks={chunks} deleted={deleted} txns={transactions}");
                assert_eq!(deleted, 0, "completed deleted metadata must be pruned");
                assert_eq!(transactions, 0, "completed journal entries must be pruned");
                assert_eq!(
                    chunks as usize, space.blocks_count as usize,
                    "only live chunk rows remain"
                );
                println!("GC factor={factor} total={} blocks={} count={} db={} staging={} trash={} locks={} logs={}", space.total, space.blocks, space.blocks_count, space.db, space.staging, space.trash, space.locks, space.logs);
                assert!(
                    store.get_artifact_by_id(&user_id).unwrap().is_some(),
                    "other project root survives"
                );
                assert_eq!(std::fs::read(&sentinel).unwrap(), b"do not remove");
                for id in &roots {
                    assert!(
                        store.get_artifact_by_id(id).unwrap().is_some(),
                        "session root survives GC"
                    );
                }
            }
        }
        assert!(
            session.apply_revision(&[("body", body_unit(513))]).is_ok(),
            "session remains usable"
        );
        assert_eq!(
            gc_watermarks.len(),
            8,
            "periodic GC checkpoints must execute"
        );
        let middle = gc_watermarks[3];
        let end = gc_watermarks[7];
        println!("store GC settled bytes: {gc_watermarks:?}");
        assert!(
            end <= middle + 64 * 1024,
            "store watermark grew without bound: mid={middle}, end={end}"
        );
        session.close().expect("close removes session roots");
        session.close().expect("idempotent close");
        assert!(store.root_artifacts(&project, "soft").unwrap().is_empty());
        assert!(
            store.get_artifact_by_id(&user_id).unwrap().is_some(),
            "other project remains intact"
        );
        drop(store);
        drop(session);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn av_closures_incremental_baseline_campaign_replay() {
        use ace_av::formal_campaign::{run_formal_campaign, CampaignConfig};
        // The shared 828 judge retains its historical P95 contract. Its raw
        // parent-measured samples and correctness/structure gates are reused;
        // this Plan independently applies the user-approved P90 criterion.
        let cfg = CampaignConfig {
            revisions: 10,
            gate_ms: f64::MAX,
        };
        let worker = std::path::PathBuf::from(env!("CARGO_BIN_EXE_av-worker"));
        let report = run_formal_campaign(&worker, &cfg).expect("the baseline campaign completes");
        let evidence = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/plan823-source-p90");
        report
            .write_evidence(&evidence)
            .expect("preserve raw judge samples before assertions");
        assert!(report.failures.is_empty(), "{:?}", report.failures);
        assert!(report.all_correct, "every baseline revision matched");
        assert!(report.structural_ok, "the structural gate stays green");
        assert!(
            report.gate_pass,
            "shared judge must complete and accept the non-latency gates"
        );
        let times: Vec<f64> = report
            .samples
            .iter()
            .map(|s| s.edit_to_verified_ms)
            .collect();
        let mut sorted = times.clone();
        sorted.sort_by(f64::total_cmp);
        println!(
            "source formal: samples={times:?}, P90={:.3}ms, limit=2000ms",
            sorted.get(8).copied().unwrap_or(f64::NAN)
        );
        assert!(
            p90_gate_10(&times),
            "at least 9/10 samples must be <= 2000ms: {times:?}"
        );
    }
}
