//! PLAN-825 T-11 / V-14: exact 852 source IDs through both real AV paths.
//! Expected values and traps come from the frozen source/oracle table, not
//! from the reference interpreter.

use auto_av_core::av::{execute, execute_reference, CancelFlag, ExecLimits};
use auto_av_core::contract::{ExecOutcome, TrapKind, Value, TRACE_CAPABILITY};

fn admit(src: &str) -> auto_av_core::abc_verify::LoadedModule {
    let compiled = auto_ac_prototype::source::compile_scalar_source(
        src,
        "ace-m1-825.at",
        &[TRACE_CAPABILITY.to_string()],
    )
    .expect("852 source compiles");
    let mir = auto_av_probe::hir_adapter::lower_checked(&compiled.checked, "d_main", 825)
        .expect("lower 852 HIR");
    auto_av_core::verify_mir(&mir).expect("MIR verifies");
    let wire = auto_av_core::encode_abc(&mir)
        .expect("ABC encode")
        .to_flat();
    let decoded = auto_av_core::decode_abc(&wire).expect("ABC decode");
    auto_av_core::abc_verify::verify_abc(decoded, &mir.owner).expect("ABC admission")
}

#[test]
fn av_runtime_825_baseline_sources_match_independent_oracles() {
    for (id, src, value, trap) in [
        (
            "B01",
            include_str!("../../../../crates/auto-lang/test/cases/ace-m1/src/B01.at"),
            Some(14),
            None,
        ),
        (
            "B04",
            include_str!("../../../../crates/auto-lang/test/cases/ace-m1/src/B04.at"),
            Some(42),
            None,
        ),
        (
            "B09",
            include_str!("../../../../crates/auto-lang/test/cases/ace-m1/src/B09.at"),
            Some(120),
            None,
        ),
        (
            "B13",
            include_str!("../../../../crates/auto-lang/test/cases/ace-m1/src/B13.at"),
            None,
            Some(TrapKind::IntOverflow),
        ),
        (
            "B14",
            include_str!("../../../../crates/auto-lang/test/cases/ace-m1/src/B14.at"),
            None,
            Some(TrapKind::DivByZero),
        ),
    ] {
        let loaded = admit(src);
        let cancel = CancelFlag::new(false);
        let (new_out, new_trace) = execute(&loaded, &ExecLimits::default(), &cancel);
        let (ref_out, ref_trace) = execute_reference(&loaded, &ExecLimits::default(), &cancel);
        assert_eq!(new_out, ref_out, "{id}: optimized/reference outcome");
        match (value, trap) {
            (Some(expected), None) => assert!(
                matches!(new_out, ExecOutcome::Success { value: Value::I32(actual), .. } if actual == expected),
                "{id}: {new_out:?}"
            ),
            (None, Some(expected)) => assert!(
                matches!(new_out, ExecOutcome::Trap { kind: actual, .. } if actual == expected),
                "{id}: {new_out:?}"
            ),
            _ => unreachable!(),
        }
        assert!(new_trace.loaded_index_builds > 0, "{id}");
        assert!(new_trace.loaded_index_bytes > 0, "{id}");
        assert!(new_trace.loaded_pc_lookups > 0, "{id}");
        assert_eq!(new_trace.reference_pc_lookups, 0, "{id}");
        assert_eq!(new_trace.loaded_param_map_queries, 1, "{id}");
        assert_eq!(new_trace.reference_param_map_copies, 0, "{id}");
        assert!(ref_trace.reference_pc_lookups > 0, "{id}");
        assert_eq!(ref_trace.loaded_pc_lookups, 0, "{id}");
        assert_eq!(ref_trace.reference_param_map_copies, 1, "{id}");
        assert_eq!(new_trace.frames_remaining, 0, "{id}");
        if id == "B09" {
            assert!(new_trace.loaded_index_builds >= 2);
            assert!(new_trace.call_frame_commits >= 5);
        }
    }
}
