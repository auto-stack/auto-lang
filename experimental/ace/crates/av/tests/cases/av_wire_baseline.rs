//! PLAN-826 T-11 / V-14: the 852 unified capability validation phase over
//! the compact wire. The frozen 852 source set (B01/B04/B09 scalar
//! positives, B13/B14 faults) runs through BOTH delivered wire policies —
//! legacy AVAB and compact AVAC — against the frozen independent oracle
//! table, with real admission, execution, reference cross-check, loaded
//! counters, wire-size accounting, and the I07 pause/new-version/resume
//! axis on compact-admitted snapshots.

use ace_av_core::av::{
    execute, execute_program, execute_program_steps, execute_reference, resume_program, CancelFlag,
    ExecLimits, SteppedOutcome,
};
use ace_av_core::contract::{ExecOutcome, TrapKind, Value, TRACE_CAPABILITY};
use ace_av_core::{ProgramSnapshot, COMPACT_ABC_MAGIC};

fn lower(src: &str, tag: u64) -> ace_av_core::mir::MirModule {
    let compiled = ace_core::source::compile_scalar_source(
        src,
        "ace-m1-826.at",
        &[TRACE_CAPABILITY.to_string()],
    )
    .expect("852 source compiles");
    let mir = ace_lowering::lower_checked(&compiled.checked, "d_main", tag).expect("lower 852 HIR");
    ace_av_core::verify_mir(&mir).expect("MIR verifies");
    mir
}

/// Legacy AVAB admission of a lowered module.
fn admit_legacy(mir: &ace_av_core::mir::MirModule) -> ace_av_core::LoadedModule {
    let wire = ace_av_core::encode_abc(mir)
        .expect("legacy encode")
        .to_flat();
    ace_av_core::verify_abc_bytes(&wire, &mir.owner).expect("legacy admission")
}

/// Compact AVAC admission of the same module.
fn admit_compact(mir: &ace_av_core::mir::MirModule) -> ace_av_core::LoadedModule {
    let wire = ace_av_core::encode_compact_abc(mir).expect("compact encode");
    assert!(wire.starts_with(&COMPACT_ABC_MAGIC));
    ace_av_core::verify_compact_abc_bytes(&wire, &mir.owner).expect("compact admission")
}

#[test]
fn av_wire_baseline_sources_match_independent_oracles_over_both_wires() {
    let mut compact_total = 0usize;
    let mut legacy_total = 0usize;
    for (id, src, value, trap) in [
        (
            "B01",
            include_str!("../../../../../../crates/auto-lang/test/cases/ace-m1/src/B01.at"),
            Some(14),
            None,
        ),
        (
            "B04",
            include_str!("../../../../../../crates/auto-lang/test/cases/ace-m1/src/B04.at"),
            Some(42),
            None,
        ),
        (
            "B09",
            include_str!("../../../../../../crates/auto-lang/test/cases/ace-m1/src/B09.at"),
            Some(120),
            None,
        ),
        (
            "B13",
            include_str!("../../../../../../crates/auto-lang/test/cases/ace-m1/src/B13.at"),
            None,
            Some(TrapKind::IntOverflow),
        ),
        (
            "B14",
            include_str!("../../../../../../crates/auto-lang/test/cases/ace-m1/src/B14.at"),
            None,
            Some(TrapKind::DivByZero),
        ),
    ] {
        let mir = lower(src, 826);
        // Wire-size accounting (raw bytes, both generations of the SAME
        // semantics): the compact wire is strictly smaller on every
        // baseline source — the density contract's observable face.
        let legacy_bytes = ace_av_core::encode_abc(&mir).expect("legacy encode");
        let compact_bytes = ace_av_core::encode_compact_abc(&mir).expect("compact encode");
        assert!(
            compact_bytes.len() < legacy_bytes.total_len,
            "{id}: compact {} must be smaller than legacy {}",
            compact_bytes.len(),
            legacy_bytes.total_len
        );
        compact_total += compact_bytes.len();
        legacy_total += legacy_bytes.total_len;

        let cancel = CancelFlag::new(false);
        for (policy, loaded) in [
            ("legacy", admit_legacy(&mir)),
            ("compact", admit_compact(&mir)),
        ] {
            let (new_out, new_trace) = execute(&loaded, &ExecLimits::default(), &cancel);
            let (ref_out, _) = execute_reference(&loaded, &ExecLimits::default(), &cancel);
            assert_eq!(new_out, ref_out, "{id}/{policy}: optimized vs reference");
            match (value, trap) {
                (Some(expected), None) => assert!(
                    matches!(new_out, ExecOutcome::Success { value: Value::I32(actual), .. } if actual == expected),
                    "{id}/{policy}: {new_out:?}"
                ),
                (None, Some(expected)) => assert!(
                    matches!(new_out, ExecOutcome::Trap { kind: actual, .. } if actual == expected),
                    "{id}/{policy}: {new_out:?}"
                ),
                _ => unreachable!(),
            }
            assert!(new_trace.loaded_pc_lookups > 0, "{id}/{policy}");
            assert_eq!(new_trace.reference_pc_lookups, 0, "{id}/{policy}");
        }
    }
    // Recorded for the validation binding: aggregate raw sizes over the
    // frozen five-source baseline (compact vs legacy).
    eprintln!(
        "AV826-BASELINE raw-bytes compact={compact_total} legacy={legacy_total} ratio={:.3}",
        compact_total as f64 / legacy_total as f64
    );
}

#[test]
fn av_wire_baseline_resume_old_call_after_new_version() {
    // I07 over the compact wire: an OLD compact-admitted call pauses
    // mid-run; a NEW version (body edit) links, fully runs and returns
    // its new value; the paused OLD call resumes and finishes with the
    // OLD value — the old snapshot stays internally consistent.
    use std::sync::Arc;
    const OLD: &str = "fn main() int {\n    let a = 1\n    let b = 2\n    return a + b\n}\n";
    const NEW: &str = "fn main() int {\n    let a = 10\n    let b = 20\n    return a + b\n}\n";
    let old_mir = lower(OLD, 11);
    let new_mir = lower(NEW, 12);

    let build_snapshot =
        |mir: &ace_av_core::mir::MirModule, revision: u64| -> Arc<ProgramSnapshot> {
            // Split the ORIGINAL lowered module (its function names are the
            // def ids; the compact wire strips names, so units derive from
            // the in-memory module and re-bind before admission) and link a
            // runnable snapshot at the requested revision (the real 828 link
            // path).
            let units = ace_av::cache::split_units(mir).expect("split");
            let codec = ace_av::wire::WireCodec::CompactAbc;
            let codes = units
                .iter()
                .map(|unit| {
                    let mut spanless = unit.clone();
                    spanless.funcs[0].spans.clear();
                    let wire = ace_av_core::encode_compact_abc(&spanless).expect("unit encode");
                    let mut decoded = ace_av_core::decode_compact_abc(&wire).expect("unit decode");
                    // The compact wire carries no function names — the unit's
                    // def id re-binds before admission (envelope rule).
                    codec.rebind_unit_name(&mut decoded.module, &spanless.funcs[0].name);
                    let admitted = ace_av_core::verify_compact_abc(
                        ace_av_core::DecodedCompactAbc {
                            version: decoded.version,
                            profile: decoded.profile,
                            module: decoded.module,
                        },
                        &unit.owner,
                    )
                    .expect("unit admit");
                    Arc::new(
                        ace_av_core::FunctionCode::admit_unit(admitted, revision)
                            .expect("resident unit"),
                    )
                })
                .collect::<Vec<_>>();
            Arc::new(ProgramSnapshot::link(codes, "d_main", revision).expect("link snapshot"))
        };

    let old_snap = build_snapshot(&old_mir, 1);
    let new_snap = build_snapshot(&new_mir, 2);
    let cancel = CancelFlag::new(false);

    // Old call pauses after its first step.
    let stepped = execute_program_steps(&old_snap, &ExecLimits::default(), &cancel, 1);
    let paused = match stepped {
        SteppedOutcome::Paused(p) => p,
        SteppedOutcome::Finished(out, _) => {
            panic!("old call unexpectedly finished in one step: {out:?}")
        }
    };
    // New version fully runs while the old call is paused.
    let (new_out, _) = execute_program(&new_snap, &ExecLimits::default(), &cancel);
    assert!(
        matches!(
            new_out,
            ExecOutcome::Success {
                value: Value::I32(30),
                ..
            }
        ),
        "{new_out:?}"
    );
    // Resuming the old call finishes with the OLD value on the OLD link.
    let resumed = resume_program(paused, &cancel, None);
    match resumed {
        SteppedOutcome::Finished(out, trace) => {
            assert!(
                matches!(
                    out,
                    ExecOutcome::Success {
                        value: Value::I32(3),
                        ..
                    }
                ),
                "{out:?}"
            );
            assert!(trace.loaded_pc_lookups > 0);
        }
        _ => panic!("resume must complete the old call"),
    }
    // And the old snapshot still runs standalone.
    let (old_out, _) = execute_program(&old_snap, &ExecLimits::default(), &cancel);
    assert!(matches!(
        old_out,
        ExecOutcome::Success {
            value: Value::I32(3),
            ..
        }
    ));
}
