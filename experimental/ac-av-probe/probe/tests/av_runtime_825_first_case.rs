//! PLAN-825 T-02: real scalar HIR→MIR→ABC→LoadedProgram execution,
//! snapshot/resume, cache-hit and fresh-worker recovery. The source and the
//! independent result/trace oracle are frozen with the AutoDown first case.

use auto_av_core::av::{
    execute, execute_program, execute_program_reference, execute_program_steps, execute_reference,
    resume_program, CancelFlag, ExecLimits, SteppedOutcome,
};
use auto_av_core::contract::{ExecOutcome, Value, AV_PROFILE, TRACE_CAPABILITY};
use auto_av_core::mir::TraceEvent;
use auto_av_core::{abc_verify, decode_abc, encode_abc, FunctionCode, ProgramSnapshot};
use auto_av_probe::cache::{manifest_key, open_store, split_units};
use auto_av_probe::driver::Driver;
use std::sync::Arc;

const FIRST_CASE: &str =
    include_str!("../../../../crates/auto-lang/test/cases/av-abc/825/src/first-case.at");
const EXPECTED_TRACE: &[TraceEvent] = &[TraceEvent::MarkA, TraceEvent::MarkB, TraceEvent::MarkA];

fn admit_source() -> abc_verify::LoadedModule {
    let compiled = auto_ac_prototype::source::compile_scalar_source(
        FIRST_CASE,
        "first-case.at",
        &[TRACE_CAPABILITY.to_string()],
    )
    .expect("scalar source compiles");
    let mir = auto_mir_lower::lower_checked(&compiled.checked, "d_main", 825)
        .expect("lower real HIR");
    assert_eq!(mir.profile, AV_PROFILE);
    auto_av_core::verify_mir(&mir).expect("MIR verifies");
    let wire = encode_abc(&mir).expect("ABC encode").to_flat();
    let decoded = decode_abc(&wire).expect("ABC decode");
    abc_verify::verify_abc(decoded, &mir.owner).expect("ABC admission")
}

fn assert_42(out: ExecOutcome, events: &[TraceEvent]) {
    let ExecOutcome::Success { value, .. } = out else {
        panic!("first case must succeed: {out:?}")
    };
    assert_eq!(value, Value::I32(42));
    assert_eq!(events, EXPECTED_TRACE);
}

#[test]
fn av_runtime_825_first_case_direct_loaded_and_reference_match() {
    let loaded = admit_source();
    let cancel = CancelFlag::new(false);
    let (new_out, new_trace) = execute(&loaded, &ExecLimits::default(), &cancel);
    let (ref_out, ref_trace) = execute_reference(&loaded, &ExecLimits::default(), &cancel);
    assert_42(new_out, &new_trace.events);
    assert_42(ref_out, &ref_trace.events);
    assert!(new_trace.loaded_index_builds >= 2);
    assert!(new_trace.loaded_pc_lookups > 0);
    assert_eq!(new_trace.reference_pc_lookups, 0);
    assert_eq!(new_trace.loaded_param_map_queries, 1);
    assert_eq!(new_trace.reference_param_map_copies, 0);
    assert!(ref_trace.reference_pc_lookups > 0);
    assert_eq!(ref_trace.loaded_pc_lookups, 0);
    assert_eq!(ref_trace.reference_param_map_copies, 1);
}

#[test]
fn av_runtime_825_first_case_snapshot_and_resume_use_same_loaded_program() {
    let loaded = admit_source();
    let code = split_units(loaded.module())
        .expect("split admitted module")
        .iter()
        .map(|unit| {
            let wire = encode_abc(unit).expect("unit encode").to_flat();
            let admitted = auto_av_core::verify_abc_bytes(&wire, &unit.owner).expect("unit admit");
            Arc::new(FunctionCode::admit_unit(admitted, 1).expect("resident unit"))
        })
        .collect::<Vec<_>>();
    let snap = Arc::new(ProgramSnapshot::link(code, "d_main", 1).expect("link snapshot"));
    let cancel = CancelFlag::new(false);
    let (out, trace) = execute_program(&snap, &ExecLimits::default(), &cancel);
    assert_42(out, &trace.events);
    assert!(trace.loaded_index_builds >= 2);
    assert!(trace.loaded_pc_lookups > 0);
    assert_eq!(trace.reference_pc_lookups, 0);
    let (reference, reference_trace) =
        execute_program_reference(&snap, &ExecLimits::default(), &cancel);
    assert_42(reference, &reference_trace.events);

    let constrained = ExecLimits {
        max_slot_bytes: trace.loaded_index_bytes as usize - 1,
        ..ExecLimits::default()
    };
    let (out, constrained_trace) = execute_program(&snap, &constrained, &cancel);
    assert!(matches!(
        out,
        ExecOutcome::Resource {
            kind: auto_av_core::contract::ResourceKind::SlotBytes,
            ..
        }
    ));
    assert_eq!(constrained_trace.loaded_pc_lookups, 0);
    assert_eq!(constrained_trace.frames_remaining, 0);
    assert!(matches!(
        execute_program_steps(&snap, &constrained, &cancel, 1),
        SteppedOutcome::Finished(
            ExecOutcome::Resource {
                kind: auto_av_core::contract::ResourceKind::SlotBytes,
                ..
            },
            _
        )
    ));

    let mut stepped = execute_program_steps(&snap, &ExecLimits::default(), &cancel, 1);
    let mut final_trace = None;
    for _ in 0..32 {
        match stepped {
            SteppedOutcome::Finished(out, trace) => {
                assert_42(out, &trace.events);
                final_trace = Some(trace);
                break;
            }
            SteppedOutcome::Paused(paused) => {
                stepped = resume_program(paused, &cancel, Some(2));
            }
        }
    }
    let trace = final_trace.expect("resume finishes within the bounded control");
    assert!(trace.loaded_pc_lookups > 0);
    assert_eq!(trace.reference_pc_lookups, 0);
}

#[test]
fn av_runtime_825_first_case_cache_hit_and_new_worker_restore() {
    let root = std::env::temp_dir().join(format!("av825-first-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let caps = vec![TRACE_CAPABILITY.to_string()];
    let mut driver = Driver::with_store(&root, "av825-first").expect("store driver");
    let first = driver.request(FIRST_CASE, "d_main", &caps);
    assert_42(first.av.clone().expect("AV outcome"), &first.trace);
    assert!(first.execution.loaded_index_builds >= 2);
    assert!(first.execution.loaded_pc_lookups > 0);
    assert_eq!(first.execution.reference_pc_lookups, 0);
    let second = driver.request(FIRST_CASE, "d_main", &caps);
    assert_42(second.av.clone().expect("AV outcome"), &second.trace);
    assert!(*second.counters.get("cache_hit_abc").unwrap_or(&0) >= 2);
    assert!(*second.counters.get("av_execute").unwrap_or(&0) > 0);
    assert!(second.execution.loaded_pc_lookups > 0);
    assert_eq!(second.execution.reference_pc_lookups, 0);

    let manifest = second.manifest.as_ref().expect("persisted manifest");
    let store = open_store(&root).expect("reopen store");
    let key = manifest_key(&manifest.owner, &manifest.entry, &manifest.fingerprint());
    let artifact = store
        .get_artifact("A09", &key)
        .expect("manifest lookup")
        .expect("manifest artifact");
    drop(store);
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_av_probe_worker"))
        .args([
            "restore-run",
            "--store",
            root.to_str().unwrap(),
            "--owner",
            &manifest.owner,
            "--entry",
            &manifest.entry,
            "--manifest-artifact",
            &artifact.artifact_id,
            "--snapshot",
            &format!("{:016x}", second.snapshot),
            "--caps",
            TRACE_CAPABILITY,
        ])
        .output()
        .expect("fresh worker restore");
    assert!(
        output.status.success(),
        "worker failed: {}\nstdout={}\nstderr={}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let restored: auto_av_probe::worker_restore::RestoreOutcome =
        serde_json::from_slice(&output.stdout).expect("worker JSON");
    assert!(restored.executed_ok, "{:?}", restored.error);
    assert_eq!(restored.value, Some(42));
    assert_eq!(restored.units_verified, 2);
    assert_eq!(restored.trace, ["a", "b", "a"]);
    assert!(restored.loaded_index_builds >= 2);
    assert!(restored.loaded_pc_lookups > 0);
    assert_eq!(restored.reference_pc_lookups, 0);
    assert_eq!(restored.loaded_param_map_queries, 1);
    assert_eq!(restored.reference_param_map_copies, 0);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn av_runtime_825_first_case_unit_never_dispatches() {
    let loaded = admit_source();
    let unit = split_units(loaded.module()).expect("split").remove(0);
    let wire = encode_abc(&unit).expect("unit encode").to_flat();
    let admitted = auto_av_core::verify_abc_bytes(&wire, &unit.owner).expect("unit admission");
    assert!(!admitted.executable());
    let (out, trace) = execute(&admitted, &ExecLimits::default(), &CancelFlag::new(false));
    assert!(matches!(out, ExecOutcome::Infra(_)));
    assert_eq!(trace.loaded_index_builds, 0);
    assert_eq!(trace.loaded_pc_lookups, 0);
    assert_eq!(trace.reference_pc_lookups, 0);
    assert!(trace.events.is_empty());
}
