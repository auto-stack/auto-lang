//! PLAN-825 T-06 raw AV benchmark samples. No verdict is computed here:
//! every row is emitted, including failures, for the parent-side analyzer.

use auto_av_core::av::{
    execute, execute_program, execute_program_reference, execute_reference, CancelFlag, ExecLimits,
};
use auto_av_core::contract::{fnv1a64, ExecOutcome, Value, MIR_SCHEMA, TRACE_CAPABILITY};
use auto_av_core::mir::*;
use auto_av_core::{FunctionCode, ProgramSnapshot};
use auto_av_probe::cache::split_units;
use serde_json::json;
use std::io::Write;
use std::sync::Arc;

const WARMUPS: usize = 10;
const SAMPLES: usize = 30;

fn owner() -> OwnerScope {
    OwnerScope {
        schema: MIR_SCHEMA.to_string(),
        module: "bench825".into(),
    }
}

fn chain(blocks: usize) -> MirModule {
    let mut m = MirModule::new(owner());
    let ti = m.intern_type(MirType::I32);
    let sig = m.intern_type(MirType::Fn(FnSig {
        params: vec![],
        result: ti,
    }));
    let answer = m.intern_const(MirConst::I32(42));
    m.funcs.push(MirFunction {
        owner: owner(),
        name: "d_main".into(),
        sig,
        slots: vec![MirSlot {
            kind: SlotKind::Temp,
            ty: ti,
            name: None,
        }],
        blocks: (0..blocks)
            .map(|block| MirBlock {
                instrs: vec![if block == 0 {
                    MirInstr::Const {
                        dst: 0,
                        value: answer,
                    }
                } else {
                    MirInstr::Copy { dst: 0, src: 0 }
                }],
                term: if block + 1 == blocks {
                    Terminator::Return { value: 0 }
                } else {
                    Terminator::Jump {
                        target: block as u32 + 1,
                    }
                },
            })
            .collect(),
        entry: 0,
        source_snapshot: 0,
        spans: vec![],
        env: None,
    });
    m
}

fn long_block() -> MirModule {
    let mut m = MirModule::new(owner());
    let ti = m.intern_type(MirType::I32);
    let sig = m.intern_type(MirType::Fn(FnSig {
        params: vec![],
        result: ti,
    }));
    let zero = m.intern_const(MirConst::I32(0));
    let one = m.intern_const(MirConst::I32(1));
    let mut instrs = vec![
        MirInstr::Const {
            dst: 0,
            value: zero,
        },
        MirInstr::Const { dst: 1, value: one },
    ];
    instrs.extend((0..1024).map(|_| MirInstr::Bin {
        op: BinOp::AddI32,
        dst: 0,
        lhs: 0,
        rhs: 1,
    }));
    m.funcs.push(MirFunction {
        owner: owner(),
        name: "d_main".into(),
        sig,
        slots: vec![
            MirSlot {
                kind: SlotKind::Temp,
                ty: ti,
                name: None,
            },
            MirSlot {
                kind: SlotKind::Temp,
                ty: ti,
                name: None,
            },
        ],
        blocks: vec![MirBlock {
            instrs,
            term: Terminator::Return { value: 0 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: vec![],
        env: None,
    });
    m
}

fn source(src: &str) -> MirModule {
    let compiled = auto_ac_prototype::source::compile_scalar_source(
        src,
        "bench825.at",
        &[TRACE_CAPABILITY.to_string()],
    )
    .expect("benchmark source compiles");
    auto_av_probe::hir_adapter::lower_checked(&compiled.checked, "d_main", 825)
        .expect("benchmark HIR lowers")
}

fn workloads() -> Vec<(String, i32, MirModule)> {
    let mut out = vec![
        (
            "arithmetic".to_string(),
            500500,
            source("fn main() int {\n    var i = 1\n    var sum = 0\n    for i <= 1000 {\n        sum = sum + i\n        i = i + 1\n    }\n    return sum\n}\n"),
        ),
        ("long1024".to_string(), 1024, long_block()),
        (
            "recursive32".to_string(),
            528,
            source("fn sum(n int) int {\n    if n <= 1 {\n        return n\n    }\n    return n + sum(n - 1)\n}\nfn main() int { return sum(32) }\n"),
        ),
    ];
    for blocks in [1, 8, 64, 256, 1024] {
        out.push((format!("short-b{blocks}"), 42, chain(blocks)));
    }
    out
}

fn snapshot(mir: &MirModule) -> Arc<ProgramSnapshot> {
    auto_av_core::verify_mir(mir).expect("MIR verifies");
    let code = split_units(mir)
        .expect("split units")
        .iter()
        .map(|unit| {
            let wire = auto_av_core::encode_abc(unit)
                .expect("unit encode")
                .to_flat();
            let admitted = auto_av_core::verify_abc_bytes(&wire, &unit.owner).expect("unit admit");
            Arc::new(FunctionCode::admit_unit(admitted, 1).expect("resident code"))
        })
        .collect();
    Arc::new(ProgramSnapshot::link(code, "d_main", 1).expect("snapshot link"))
}

fn admit(mir: &MirModule) -> auto_av_core::abc_verify::LoadedModule {
    auto_av_core::verify_mir(mir).expect("MIR verifies");
    let wire = auto_av_core::encode_abc(mir).expect("ABC encode").to_flat();
    let decoded = auto_av_core::decode_abc(&wire).expect("ABC decode");
    auto_av_core::abc_verify::verify_abc(decoded, &mir.owner).expect("ABC admission")
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let out = args
        .windows(2)
        .find(|pair| pair[0] == "--out")
        .map(|pair| pair[1].clone())
        .unwrap_or_else(|| "-".into());
    if args.iter().any(|arg| arg == "--driver-load") {
        let load = auto_av_probe::load256::generate_revision(0);
        let mut driver = auto_av_probe::driver::Driver::ephemeral();
        let result = driver.request(&load.source, "d_main", &[]);
        println!(
            "wall_ms={} expected={} value={:?}",
            result.wall_ms,
            load.expected_main,
            result.value_i32()
        );
        for span in result.spans {
            println!("{}={}ms", span.name, span.dur.as_secs_f64() * 1000.0);
        }
        return;
    }
    let binary = std::env::current_exe().expect("benchmark executable identity");
    let binary_fp = fnv1a64(&std::fs::read(&binary).expect("read benchmark executable"));
    let mut writer: Box<dyn Write> = if out == "-" {
        Box::new(std::io::stdout())
    } else {
        Box::new(std::fs::File::create(out).expect("create raw output"))
    };
    let workloads = if args.iter().any(|arg| arg == "--full-load") {
        let load = auto_av_probe::load256::generate_revision(0);
        vec![(
            "load256".to_string(),
            load.expected_main,
            source(&load.source),
        )]
    } else {
        workloads()
    };
    for (name, expected, mir) in workloads {
        let input_fp = fnv1a64(format!("{mir:?}").as_bytes());
        for engine in ["av-reference", "av-loaded"] {
            let direct = args.iter().any(|arg| arg == "--direct");
            let sampler = auto_av_probe::mempeek::PeakSampler::start(5);
            let started = std::time::Instant::now();
            let loaded_module = direct.then(|| admit(&mir));
            let snapshot = (!direct).then(|| snapshot(&mir));
            let load_ms = started.elapsed().as_secs_f64() * 1000.0;
            writeln!(writer, "{}", json!({
                "engine": engine, "workload": name, "phase": "load", "sample": 0,
                "ms": load_ms, "expected": expected, "value": expected,
                "input_fnv64": format!("{input_fp:016x}"),
                "binary_fnv64": format!("{binary_fp:016x}"),
                "loaded_index_builds": snapshot.as_ref().map(|s| s.loaded_program().builds()).unwrap_or(0),
                "loaded_index_reuses": snapshot.as_ref().map(|s| s.loaded_program().reuses()).unwrap_or(0),
                "loaded_index_bytes": snapshot.as_ref().map(|s| s.loaded_program().usage().0).unwrap_or(0),
            }))
            .expect("write load row");
            let cancel = CancelFlag::new(false);
            let limits = ExecLimits::default();
            let mut run = |phase: &str, sample: usize| {
                let started = std::time::Instant::now();
                let (outcome, trace) = match (direct, engine) {
                    (true, "av-loaded") => {
                        execute(loaded_module.as_ref().unwrap(), &limits, &cancel)
                    }
                    (true, _) => {
                        execute_reference(loaded_module.as_ref().unwrap(), &limits, &cancel)
                    }
                    (false, "av-loaded") => {
                        execute_program(snapshot.as_ref().unwrap(), &limits, &cancel)
                    }
                    (false, _) => {
                        execute_program_reference(snapshot.as_ref().unwrap(), &limits, &cancel)
                    }
                };
                let ms = started.elapsed().as_secs_f64() * 1000.0;
                let value = match outcome {
                    ExecOutcome::Success {
                        value: Value::I32(value),
                        ..
                    } => value,
                    other => panic!("{engine}/{name} failed: {other:?}"),
                };
                assert_eq!(value, expected, "{engine}/{name} independent result");
                writeln!(
                    writer,
                    "{}",
                    json!({
                        "engine": engine, "workload": name, "phase": phase, "sample": sample,
                        "ms": ms, "expected": expected, "value": value,
                        "input_fnv64": format!("{input_fp:016x}"),
                        "binary_fnv64": format!("{binary_fp:016x}"),
                        "loaded_index_builds": trace.loaded_index_builds,
                        "loaded_index_reuses": trace.loaded_index_reuses,
                        "loaded_index_bytes": trace.loaded_index_bytes,
                        "loaded_pc_lookups": trace.loaded_pc_lookups,
                        "reference_pc_lookups": trace.reference_pc_lookups,
                        "reference_param_map_copies": trace.reference_param_map_copies,
                        "root_frame_commits": trace.root_frame_commits,
                        "call_frame_commits": trace.call_frame_commits,
                        "frames_remaining": trace.frames_remaining,
                    })
                )
                .expect("write execution row");
            };
            run("first", 0);
            for sample in 1..=WARMUPS {
                run("warmup", sample);
            }
            for sample in 1..=SAMPLES {
                run("hot", sample);
            }
            writeln!(
                writer,
                "{}",
                json!({
                    "engine": engine, "workload": name, "phase": "process-peak", "sample": 0,
                    "ms": 0.0, "expected": expected, "value": expected,
                    "input_fnv64": format!("{input_fp:016x}"),
                    "binary_fnv64": format!("{binary_fp:016x}"),
                    "private_bytes_peak": sampler.stop(),
                })
            )
            .expect("write process peak row");
        }
    }
}
