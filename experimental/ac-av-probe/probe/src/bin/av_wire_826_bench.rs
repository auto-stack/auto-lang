//! PLAN-826 T-06 raw AV benchmark samples over BOTH wire generations. No
//! verdict is computed here: every row is emitted, including failures,
//! for the parent-side analyzer. Engine rows (av-reference/av-loaded)
//! execute COMPACT-admitted modules — this plan's new-wire path; size
//! rows account raw bytes for both generations of the same semantics and
//! the payload files are written for the analyzer's same-algorithm
//! compression pass.

use auto_av_core::av::{execute, execute_reference, CancelFlag, ExecLimits};
use auto_av_core::contract::{fnv1a64, ExecOutcome, Value, MIR_SCHEMA, TRACE_CAPABILITY};
use auto_av_core::mir::*;
use serde_json::json;
use std::io::Write;

const WARMUPS: usize = 10;
const SAMPLES: usize = 30;

fn owner() -> OwnerScope {
    OwnerScope {
        schema: MIR_SCHEMA.to_string(),
        module: "bench826".into(),
    }
}

fn entry_fn(
    m: &mut MirModule,
    name: &str,
    sig: TypeIdx,
    slots: Vec<MirSlot>,
    blocks: Vec<MirBlock>,
) {
    m.funcs.push(MirFunction {
        owner: owner(),
        name: name.into(),
        sig,
        slots,
        blocks,
        entry: 0,
        env: None,
        source_snapshot: 0,
        spans: vec![],
    });
}

fn i32_fn_type(m: &mut MirModule) -> TypeIdx {
    let ti = m.intern_type(MirType::I32);
    m.intern_type(MirType::Fn(FnSig {
        params: vec![],
        result: ti,
    }))
}

fn chain(blocks: usize) -> MirModule {
    let mut m = MirModule::new(owner());
    let ti = m.intern_type(MirType::I32);
    let sig = i32_fn_type(&mut m);
    let answer = m.intern_const(MirConst::I32(42));
    entry_fn(
        &mut m,
        "d_main",
        sig,
        vec![MirSlot {
            kind: SlotKind::Temp,
            ty: ti,
            name: None,
        }],
        (0..blocks)
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
    );
    m
}

fn long_block() -> MirModule {
    let mut m = MirModule::new(owner());
    let ti = m.intern_type(MirType::I32);
    let sig = i32_fn_type(&mut m);
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
    entry_fn(
        &mut m,
        "d_main",
        sig,
        vec![
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
        vec![MirBlock {
            instrs,
            term: Terminator::Return { value: 0 },
        }],
    );
    m
}

fn source(src: &str) -> MirModule {
    let compiled = auto_ac_prototype::source::compile_scalar_source(
        src,
        "bench826.at",
        &[TRACE_CAPABILITY.to_string()],
    )
    .expect("benchmark source compiles");
    auto_mir_lower::lower_checked(&compiled.checked, "d_main", 826)
        .expect("benchmark HIR lowers")
}

/// Wire-density group: 64 one-block functions of two instructions each,
/// main calls a chain of them (many small functions).
fn many_small_functions() -> MirModule {
    let mut m = MirModule::new(owner());
    let ti = m.intern_type(MirType::I32);
    let sig = i32_fn_type(&mut m);
    for i in 0..64u32 {
        let c = m.intern_const(MirConst::I32(1));
        entry_fn(
            &mut m,
            &format!("d_f{i}"),
            sig,
            vec![MirSlot {
                kind: SlotKind::Temp,
                ty: ti,
                name: None,
            }],
            vec![MirBlock {
                instrs: vec![MirInstr::Const { dst: 0, value: c }],
                term: Terminator::Return { value: 0 },
            }],
        );
    }
    // main = f0 + f1 + ... (63 calls in one block)
    let c0 = m.intern_const(MirConst::I32(0));
    let mut instrs = vec![MirInstr::Const { dst: 0, value: c0 }];
    for i in 0..64u32 {
        instrs.push(MirInstr::Call {
            callee: CalleeRef::Func(i),
            dst: 1,
            args: vec![],
        });
        instrs.push(MirInstr::Bin {
            op: BinOp::AddI32,
            dst: 0,
            lhs: 0,
            rhs: 1,
        });
    }
    entry_fn(
        &mut m,
        "d_main",
        sig,
        vec![
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
        vec![MirBlock {
            instrs,
            term: Terminator::Return { value: 0 },
        }],
    );
    m.entry = 64;
    m
}

/// Wire-density group: 256 short constants referenced once each (large
/// constant indices cross the ULEB boundary).
fn short_consts() -> MirModule {
    let mut m = MirModule::new(owner());
    let ti = m.intern_type(MirType::I32);
    let sig = i32_fn_type(&mut m);
    let mut consts = Vec::new();
    for i in 0..256u32 {
        // Distinct values: intern_const dedupes equal payloads, and the
        // group needs a real 256-entry table so the referenced indices
        // cross the ULEB one-byte boundary.
        consts.push(m.intern_const(MirConst::I32(i as i32)));
    }
    let mut instrs = vec![MirInstr::Const {
        dst: 0,
        value: consts[0],
    }];
    for &c in &consts {
        instrs.push(MirInstr::Const { dst: 1, value: c });
        instrs.push(MirInstr::Bin {
            op: BinOp::AddI32,
            dst: 0,
            lhs: 0,
            rhs: 1,
        });
    }
    entry_fn(
        &mut m,
        "d_main",
        sig,
        vec![
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
        vec![MirBlock {
            instrs,
            term: Terminator::Return { value: 0 },
        }],
    );
    m
}

/// Wire-density group: legal indices well past the one-byte ULEB range
/// (300 slots).
fn large_indices() -> MirModule {
    let mut m = MirModule::new(owner());
    let ti = m.intern_type(MirType::I32);
    let sig = i32_fn_type(&mut m);
    let zero = m.intern_const(MirConst::I32(0));
    let one = m.intern_const(MirConst::I32(1));
    let mut instrs = vec![
        MirInstr::Const {
            dst: 200,
            value: zero,
        },
        MirInstr::Const {
            dst: 299,
            value: one,
        },
    ];
    for _ in 0..64 {
        instrs.push(MirInstr::Bin {
            op: BinOp::AddI32,
            dst: 200,
            lhs: 200,
            rhs: 299,
        });
    }
    entry_fn(
        &mut m,
        "d_main",
        sig,
        vec![
            MirSlot {
                kind: SlotKind::Temp,
                ty: ti,
                name: None,
            };
            300
        ],
        vec![MirBlock {
            instrs,
            term: Terminator::Return { value: 200 },
        }],
    );
    m
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
    out.push((
        "many-small-functions".to_string(),
        64,
        many_small_functions(),
    ));
    let short_const_sum: i32 = (0..256).sum();
    out.push(("short-consts".to_string(), short_const_sum, short_consts()));
    out.push(("large-indices".to_string(), 64, large_indices()));
    // Debug presence pair: the SAME lowered source with and without its
    // span tables (the optional SOURCE section's cost face).
    let mut with_debug = source(
        "fn side(x int) int { return x + 1 }\nfn main() int { return side(20) + side(1) }\n",
    );
    with_debug.funcs[0].source_snapshot = 1;
    with_debug.funcs[1].source_snapshot = 1;
    for f in with_debug.funcs.iter_mut() {
        f.spans = (0..f.flat_instr_count())
            .map(|i| {
                (
                    i,
                    auto_av_core::contract::SpanInfo {
                        func: 0,
                        start: i * 2,
                        end: i * 2 + 1,
                    },
                )
            })
            .collect();
    }
    let mut without_debug = with_debug.clone();
    for f in without_debug.funcs.iter_mut() {
        f.spans.clear();
        f.source_snapshot = 0;
    }
    out.push(("debug-present".to_string(), 23, with_debug));
    out.push(("debug-stripped".to_string(), 23, without_debug));
    out
}

fn compact_admit(mir: &MirModule) -> auto_av_core::LoadedModule {
    auto_av_core::verify_mir(mir).expect("MIR verifies");
    let wire = auto_av_core::encode_compact_abc(mir).expect("compact encode");
    auto_av_core::verify_compact_abc_bytes(&wire, &mir.owner).expect("compact admission")
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let out = args
        .windows(2)
        .find(|pair| pair[0] == "--out")
        .map(|pair| pair[1].clone())
        .unwrap_or_else(|| "-".into());
    let payload_dir = args
        .windows(2)
        .find(|pair| pair[0] == "--payload-dir")
        .map(|pair| pair[1].clone());
    let binary = std::env::current_exe().expect("benchmark executable identity");
    let binary_fp = fnv1a64(&std::fs::read(&binary).expect("read benchmark executable"));
    let mut writer: Box<dyn Write> = if out == "-" {
        Box::new(std::io::stdout())
    } else {
        Box::new(std::fs::File::create(out).expect("create raw output"))
    };
    if let Some(dir) = payload_dir.as_ref() {
        std::fs::create_dir_all(dir).expect("create payload dir");
    }
    for (name, expected, mir) in workloads() {
        let input_fp = fnv1a64(format!("{mir:?}").as_bytes());
        // ---- size accounting over BOTH wire generations ----------------
        for (wire_name, wire_bytes) in [
            ("legacy", {
                let bytes = auto_av_core::encode_abc(&mir).expect("legacy encode");
                bytes.to_flat()
            }),
            (
                "compact",
                auto_av_core::encode_compact_abc(&mir).expect("compact encode"),
            ),
        ] {
            if let Some(dir) = &payload_dir {
                let path = std::path::Path::new(dir).join(format!(
                    "{}.{}.bin",
                    name.replace(['/', '\\'], "_"),
                    wire_name
                ));
                std::fs::write(&path, &wire_bytes).expect("write payload for compression");
            }
            writeln!(
                writer,
                "{}",
                json!({
                    "engine": "wire-size", "workload": name, "phase": "size", "sample": 0,
                    "wire": wire_name, "raw_bytes": wire_bytes.len(),
                    "expected": expected, "value": expected,
                    "input_fnv64": format!("{input_fp:016x}"),
                    "binary_fnv64": format!("{binary_fp:016x}"),
                })
            )
            .expect("write size row");
        }
        // ---- engine rows over the COMPACT wire --------------------------
        for engine in ["av-reference", "av-loaded"] {
            let sampler = auto_av_probe::mempeek::PeakSampler::start(5);
            let started = std::time::Instant::now();
            let loaded_module = compact_admit(&mir);
            let load_ms = started.elapsed().as_secs_f64() * 1000.0;
            writeln!(
                writer,
                "{}",
                json!({
                    "engine": engine, "workload": name, "phase": "load", "sample": 0,
                    "ms": load_ms, "expected": expected, "value": expected,
                    "input_fnv64": format!("{input_fp:016x}"),
                    "binary_fnv64": format!("{binary_fp:016x}"),
                    "wire": "compact",
                })
            )
            .expect("write load row");
            let cancel = CancelFlag::new(false);
            let limits = ExecLimits::default();
            let mut run = |phase: &str, sample: usize| {
                let started = std::time::Instant::now();
                let (outcome, trace) = if engine == "av-loaded" {
                    execute(&loaded_module, &limits, &cancel)
                } else {
                    execute_reference(&loaded_module, &limits, &cancel)
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
                        "wire": "compact",
                        "loaded_index_builds": trace.loaded_index_builds,
                        "loaded_index_reuses": trace.loaded_index_reuses,
                        "loaded_index_bytes": trace.loaded_index_bytes,
                        "loaded_pc_lookups": trace.loaded_pc_lookups,
                        "reference_pc_lookups": trace.reference_pc_lookups,
                        "reference_param_map_copies": trace.reference_param_map_copies,
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
                    "wire": "compact",
                    "private_bytes_peak": sampler.stop(),
                })
            )
            .expect("write process peak row");
        }
    }
}
