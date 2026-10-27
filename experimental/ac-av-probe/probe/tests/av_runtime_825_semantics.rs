//! PLAN-825 T-03/T-04 production semantics and resource oracles. Every case
//! executes the real admitted module through LoadedProgram; the checked
//! reference entry is an independent comparison, never the expected-value
//! source for the hand-frozen result.

use auto_av_core::av::{execute, execute_entry, execute_reference, CancelFlag, ExecLimits};
use auto_av_core::contract::{
    ExecOutcome, ResourceKind, SpanInfo, TrapKind, Value, MIR_SCHEMA, SLOT_CELL_BYTES,
    TRACE_CAPABILITY,
};
use auto_av_core::mir::*;
use auto_av_core::{abc_verify, decode_abc, encode_abc};

fn owner() -> OwnerScope {
    OwnerScope {
        schema: MIR_SCHEMA.to_string(),
        module: "runtime825".into(),
    }
}

fn admit(m: &MirModule) -> abc_verify::LoadedModule {
    auto_av_core::verify_mir(m).expect("real MIR admission");
    let wire = encode_abc(m).expect("ABC encode").to_flat();
    let decoded = decode_abc(&wire).expect("ABC decode");
    abc_verify::verify_abc(decoded, &m.owner).expect("ABC admission")
}

fn admit_source(src: &str) -> abc_verify::LoadedModule {
    let compiled = auto_ac_prototype::source::compile_scalar_source(
        src,
        "runtime825.at",
        &[TRACE_CAPABILITY.to_string()],
    )
    .expect("source compiles");
    let mir = auto_av_probe::hir_adapter::lower_checked(&compiled.checked, "d_main", 825)
        .expect("lower real HIR");
    admit(&mir)
}

fn one_slot_module() -> MirModule {
    let mut m = MirModule::new(owner());
    let ti = m.intern_type(MirType::I32);
    let sig = m.intern_type(MirType::Fn(FnSig {
        params: vec![],
        result: ti,
    }));
    let c7 = m.intern_const(MirConst::I32(7));
    m.funcs.push(MirFunction {
        owner: owner(),
        name: "main".into(),
        sig,
        slots: vec![MirSlot {
            kind: SlotKind::Local,
            ty: ti,
            name: Some("r".into()),
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
    m
}

fn chain_module(blocks: usize, width: usize) -> MirModule {
    let mut m = MirModule::new(owner());
    let ti = m.intern_type(MirType::I32);
    let sig = m.intern_type(MirType::Fn(FnSig {
        params: vec![],
        result: ti,
    }));
    let zero = m.intern_const(MirConst::I32(0));
    let body = (0..blocks)
        .map(|block| MirBlock {
            instrs: (0..width)
                .map(|ip| {
                    if block == 0 && ip == 0 {
                        MirInstr::Const {
                            dst: 0,
                            value: zero,
                        }
                    } else {
                        MirInstr::Copy { dst: 0, src: 0 }
                    }
                })
                .collect(),
            term: if block + 1 == blocks {
                Terminator::Return { value: 0 }
            } else {
                Terminator::Jump {
                    target: block as u32 + 1,
                }
            },
        })
        .collect();
    m.funcs.push(MirFunction {
        owner: owner(),
        name: "main".into(),
        sig,
        slots: vec![MirSlot {
            kind: SlotKind::Temp,
            ty: ti,
            name: None,
        }],
        blocks: body,
        entry: 0,
        source_snapshot: 0,
        spans: vec![],
        env: None,
    });
    m
}

fn run(mir: &MirModule) -> (ExecOutcome, auto_av_core::av::ExecTrace) {
    execute(&admit(mir), &ExecLimits::default(), &CancelFlag::new(false))
}

#[test]
fn av_runtime_825_pc_short_long_and_terminal_source() {
    for (blocks, width) in [(1, 1), (8, 1), (64, 1), (256, 1), (1, 256)] {
        let (out, trace) = run(&chain_module(blocks, width));
        assert!(matches!(out, ExecOutcome::Success { .. }), "{out:?}");
        assert_eq!(trace.loaded_index_builds, 1);
        assert_eq!(trace.loaded_pc_lookups, (blocks * width) as u64);
        assert_eq!(trace.reference_pc_lookups, 0);
        assert_eq!(trace.frames_remaining, 0);
    }

    let src = "fn main() int {\n    let x = 4\n    if x > 0 {\n        return 1 / 0\n    }\n    return 0\n}\n";
    let loaded = admit_source(src);
    let cancel = CancelFlag::new(false);
    let (new_out, new_trace) = execute(&loaded, &ExecLimits::default(), &cancel);
    let (ref_out, _) = execute_reference(&loaded, &ExecLimits::default(), &cancel);
    let expected = src.find("1 / 0").expect("independent source position") as u32;
    for out in [new_out, ref_out] {
        let ExecOutcome::Trap {
            kind: TrapKind::DivByZero,
            span: Some(SpanInfo { start, .. }),
        } = out
        else {
            panic!("expected spanned div-zero: {out:?}")
        };
        assert_eq!(start, expected);
    }
    assert!(new_trace.loaded_pc_lookups > 0);
    assert_eq!(new_trace.reference_pc_lookups, 0);
    assert_eq!(new_trace.frames_remaining, 0);
}

#[test]
fn av_runtime_825_calls_params_and_recursion() {
    for (src, expected) in [
        ("fn main() int { return 42 }\n", 42),
        (
            "fn calc(a int, b int, c int) int { return a * 100 + b * 10 + c }\nfn main() int { return calc(2, 3, 4) }\n",
            234,
        ),
        (
            "fn sum(n int) int {\n    if n <= 1 {\n        return n\n    }\n    return n + sum(n - 1)\n}\nfn main() int { return sum(4) }\n",
            10,
        ),
    ] {
        let loaded = admit_source(src);
        let cancel = CancelFlag::new(false);
        let (new_out, new_trace) = execute(&loaded, &ExecLimits::default(), &cancel);
        let (ref_out, ref_trace) = execute_reference(&loaded, &ExecLimits::default(), &cancel);
        assert_eq!(new_out, ref_out);
        assert_eq!(new_out, ExecOutcome::Success {
            value: Value::I32(expected),
            steps: match new_out {
                ExecOutcome::Success { steps, .. } => steps,
                _ => unreachable!(),
            },
        });
        assert_eq!(new_trace.events, ref_trace.events);
        assert!(new_trace.loaded_pc_lookups > 0);
        assert_eq!(new_trace.reference_pc_lookups, 0);
        assert_eq!(new_trace.frames_remaining, 0);
    }
}

#[test]
fn av_runtime_825_entry_signature_and_unit_zero_dispatch() {
    let mut m = MirModule::unit(owner());
    let ti = m.intern_type(MirType::I32);
    let sig = m.intern_type(MirType::Fn(FnSig {
        params: vec![ti],
        result: ti,
    }));
    m.funcs.push(MirFunction {
        owner: owner(),
        name: "main".into(),
        sig,
        slots: vec![MirSlot {
            kind: SlotKind::Param,
            ty: ti,
            name: None,
        }],
        blocks: vec![MirBlock {
            instrs: vec![],
            term: Terminator::Return { value: 0 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: vec![],
        env: None,
    });
    let wire = encode_abc(&m).expect("unit encode").to_flat();
    let loaded = auto_av_core::verify_abc_bytes(&wire, &m.owner).expect("unit admission");
    assert!(!loaded.executable());
    let (out, trace) = execute_entry(
        &loaded,
        &ExecLimits::default(),
        &CancelFlag::new(false),
        &[Value::I32(9)],
    );
    assert!(matches!(out, ExecOutcome::Infra(_)));
    assert_eq!(trace.loaded_pc_lookups, 0);
    assert_eq!(trace.reference_pc_lookups, 0);
    assert_eq!(trace.root_frame_commits, 0);
    assert_eq!(trace.loaded_param_map_queries, 0);
    assert_eq!(trace.frames_remaining, 0);

    let mut complete = m.clone();
    complete.is_unit = false;
    let err = auto_av_core::verify_mir(&complete).expect_err("complete entry must be zero-param");
    assert!(err.code().contains("entry-sig"));
}

#[test]
fn av_runtime_825_numeric_traps_and_short_circuit_match_reference() {
    for (src, kind) in [
        (
            "fn main() int { return 2147483647 + 1 }\n",
            TrapKind::IntOverflow,
        ),
        ("fn main() int { return 1 / 0 }\n", TrapKind::DivByZero),
        ("fn main() int { return 1 % 0 }\n", TrapKind::DivByZero),
        (
            "fn main() int { return -2147483648 / -1 }\n",
            TrapKind::DivOverflow,
        ),
        (
            "fn main() int { return -2147483648 % -1 }\n",
            TrapKind::RemOverflow,
        ),
    ] {
        let loaded = admit_source(src);
        let cancel = CancelFlag::new(false);
        let (new_out, new_trace) = execute(&loaded, &ExecLimits::default(), &cancel);
        let (ref_out, _) = execute_reference(&loaded, &ExecLimits::default(), &cancel);
        assert_eq!(new_out, ref_out);
        assert!(matches!(new_out, ExecOutcome::Trap { kind: got, .. } if got == kind));
        assert_eq!(new_trace.frames_remaining, 0);
    }
    let src = include_str!(
        "../../../../crates/auto-lang/test/cases/ac-av/av/core/shortcircuit_latent_trap.at"
    );
    let loaded = admit_source(src);
    let cancel = CancelFlag::new(false);
    let (new_out, _) = execute(&loaded, &ExecLimits::default(), &cancel);
    let (ref_out, _) = execute_reference(&loaded, &ExecLimits::default(), &cancel);
    assert_eq!(new_out, ref_out);
    assert!(matches!(new_out, ExecOutcome::Success { .. }));
}

#[test]
fn av_runtime_825_fuel_boundary_matches_reference() {
    let loaded = admit_source(
        "fn sum(n int) int {\n    if n <= 1 {\n        return n\n    }\n    return n + sum(n - 1)\n}\nfn main() int { return sum(4) }\n",
    );
    let cancel = CancelFlag::new(false);
    let (success, _) = execute(&loaded, &ExecLimits::default(), &cancel);
    let ExecOutcome::Success { steps, .. } = success else {
        panic!("baseline must succeed")
    };
    for (limit, accepted) in [(steps - 1, false), (steps, true)] {
        let limits = ExecLimits {
            max_steps: limit,
            ..ExecLimits::default()
        };
        let (new_out, trace) = execute(&loaded, &limits, &cancel);
        let (ref_out, _) = execute_reference(&loaded, &limits, &cancel);
        assert_eq!(new_out, ref_out);
        assert_eq!(matches!(new_out, ExecOutcome::Success { .. }), accepted);
        assert_eq!(trace.frames_remaining, 0);
    }
}

#[test]
fn av_runtime_825_root_budget_both_sides() {
    let loaded = admit(&one_slot_module());
    let cancel = CancelFlag::new(false);
    for (frames, accepted) in [(0, false), (1, true)] {
        let (out, trace) = execute(
            &loaded,
            &ExecLimits {
                max_frames: frames,
                ..ExecLimits::default()
            },
            &cancel,
        );
        assert_eq!(matches!(out, ExecOutcome::Success { .. }), accepted);
        assert_eq!(trace.root_frame_commits, accepted as u64);
        assert_eq!(trace.frames_remaining, 0);
    }
    let (_, baseline) = execute(&loaded, &ExecLimits::default(), &cancel);
    let index_bytes = baseline.loaded_index_bytes as usize;
    assert!(index_bytes > 0);
    for (bytes, accepted) in [
        (index_bytes - 1, false),
        (index_bytes, false),
        (index_bytes + SLOT_CELL_BYTES - 1, false),
        (index_bytes + SLOT_CELL_BYTES, true),
    ] {
        let (out, trace) = execute(
            &loaded,
            &ExecLimits {
                max_slot_bytes: bytes,
                ..ExecLimits::default()
            },
            &cancel,
        );
        assert_eq!(matches!(out, ExecOutcome::Success { .. }), accepted);
        if !accepted {
            assert!(matches!(
                out,
                ExecOutcome::Resource {
                    kind: ResourceKind::SlotBytes,
                    ..
                }
            ));
        }
        assert_eq!(trace.root_frame_commits, accepted as u64);
        assert_eq!(trace.frames_remaining, 0);
    }
}

#[test]
fn av_runtime_825_call_budget_both_sides_and_cleanup() {
    let loaded = admit_source("fn id(x int) int { return x }\nfn main() int { return id(7) }\n");
    let cancel = CancelFlag::new(false);
    for (frames, accepted) in [(1, false), (2, true)] {
        let (out, trace) = execute(
            &loaded,
            &ExecLimits {
                max_frames: frames,
                ..ExecLimits::default()
            },
            &cancel,
        );
        assert_eq!(matches!(out, ExecOutcome::Success { .. }), accepted);
        assert_eq!(trace.call_frame_commits, accepted as u64);
        assert_eq!(trace.frames_remaining, 0);
        if !accepted {
            assert!(matches!(
                out,
                ExecOutcome::Resource {
                    kind: ResourceKind::Frames,
                    ..
                }
            ));
        }
    }
    let (_, baseline) = execute(&loaded, &ExecLimits::default(), &cancel);
    let frame_bytes = baseline.peak_slot_bytes as usize;
    let index_bytes = baseline.loaded_index_bytes as usize;
    assert!(index_bytes > 0);
    for (bytes, accepted) in [
        (index_bytes + frame_bytes - 1, false),
        (index_bytes + frame_bytes, true),
    ] {
        let (out, trace) = execute(
            &loaded,
            &ExecLimits {
                max_slot_bytes: bytes,
                ..ExecLimits::default()
            },
            &cancel,
        );
        assert_eq!(matches!(out, ExecOutcome::Success { .. }), accepted);
        assert_eq!(trace.call_frame_commits, accepted as u64);
        assert_eq!(trace.frames_remaining, 0);
    }
}

#[test]
fn av_runtime_825_cancel_exhausted_then_next_request_clean() {
    let loaded = admit(&chain_module(1, 2048));
    let cancelled = CancelFlag::new(true);
    let (out, trace) = execute(&loaded, &ExecLimits::default(), &cancelled);
    assert_eq!(out, ExecOutcome::Cancelled);
    assert_eq!(trace.frames_remaining, 0);
    assert_eq!(trace.call_frame_commits, 0);

    let (out, trace) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    assert!(matches!(out, ExecOutcome::Success { .. }));
    assert_eq!(trace.frames_remaining, 0);
    assert_eq!(trace.root_frame_commits, 1);

    let exhausted = ExecLimits {
        max_steps: 1,
        ..ExecLimits::default()
    };
    let (out, trace) = execute(&loaded, &exhausted, &CancelFlag::new(false));
    assert!(matches!(
        out,
        ExecOutcome::Resource {
            kind: ResourceKind::InstrSteps,
            ..
        }
    ));
    assert_eq!(trace.frames_remaining, 0);
    let (out, trace) = run(&one_slot_module());
    assert!(matches!(
        out,
        ExecOutcome::Success {
            value: Value::I32(7),
            ..
        }
    ));
    assert_eq!(trace.frames_remaining, 0);
}
