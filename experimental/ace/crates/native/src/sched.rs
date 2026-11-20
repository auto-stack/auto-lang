//! PLAN-1067: the native task-scheduler lane.
//!
//! The lane consumes the SAME scheduler source surface PLAN-968 froze
//! (scheduler profile → `__task_<Name>` driver desugar → closures-family
//! lowering → verified MIR), splits the verified module into per-function
//! units, lowers every `SCHED_*` operation against the real
//! `mn.sched.runtime/1` helpers ([`crate::sched_runtime`]), links the
//! runtime + console shim + text runtime support and runs a REAL PE.
//!
//! It is an execution lane, not a simulation: Auto business code and the
//! task drivers execute as native machine functions on real OS threads; the
//! runtime never dispatches Auto semantics itself (it owns only the mailbox
//! / port / thread bookkeeping the frozen scheduler contract names).
//!
//! The exit contract: the startup wrapper calls the entry, then the orderly
//! drain, then `ExitProcess`. A non-zero drain residue is the named
//! `sched.leak` exit (72) — a leaked task/buffer/port can never look like a
//! successful run.

use crate::config::{RUN_BUDGET, START_SYMBOL, STREAM_CAP};
use crate::link_driver::{self, ObjectInput};
use crate::{emit, lower, symbols, MnError};
use ace_av_core::mir::{MirInstr, MirModule, MirType};
use ace_core::link::CaptureCaps;
use cranelift_codegen::ir::{AbiParam, InstBuilder, Signature};
use cranelift_codegen::isa::CallConv;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_module::{default_libcall_names, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};
use std::path::Path;

/// The named exit a non-zero drain residue reports (never a success code).
pub const SCHED_LEAK_EXIT_CODE: i32 = 72;

/// PLAN-1067 T-07: the ONLY Win32 entry points the scheduler lane may link.
/// Every one of them is a documented kernel32 contract the runtime's own
/// bookkeeping needs (thread per task chain, blocking mailbox waits, the
/// private heap the runtime audits, and the contained exit). Anything else —
/// in particular CRT/`Rust std` file or console IO, `VirtualAlloc` families
/// or the platform-io residue — is a named audit failure.
pub const SCHED_KERNEL32_IMPORTS: &[&str] = &[
    // console/1 shim (shared with the console lane)
    "GetStdHandle",
    "WriteFile",
    // task chains
    "CreateThread",
    "WaitForSingleObject",
    "CloseHandle",
    "TlsAlloc",
    "TlsSetValue",
    "TlsGetValue",
    // mailbox / port primitives
    "AcquireSRWLockExclusive",
    "ReleaseSRWLockExclusive",
    "SleepConditionVariableSRW",
    "WakeConditionVariable",
    "WakeAllConditionVariable",
    "Sleep",
    // the runtime's audited private heap
    "GetProcessHeap",
    "HeapAlloc",
    "HeapFree",
    // contained exit
    "ExitProcess",
];

/// PLAN-1067 T-07: audit a linked scheduler PE — exactly `kernel32.dll`, no
/// ordinals, and every import inside [`SCHED_KERNEL32_IMPORTS`]. Never masks a
/// forbidden symbol: the caller gets a named error.
pub fn audit_sched_pe_imports(
    pe_bytes: &[u8],
) -> Result<crate::platform_io::PeImportReport, MnError> {
    crate::platform_io::audit_pe_imports_with(pe_bytes, SCHED_KERNEL32_IMPORTS, &[], "scheduler")
}

/// Stage/code pair of the shared scheduler front door.
#[derive(Debug)]
pub struct SchedDiag {
    pub stage: String,
    pub code: String,
    pub message: String,
}

/// One scheduler-lane run observation (never expected values).
#[derive(Debug)]
pub struct SchedNativeRun {
    pub exit_code: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

/// Compile through the shared scheduler front door (PLAN-968 profile), lower
/// through the closures-family adapter and verify the MIR.
pub fn sched_compile(src: &str, entry: &str) -> Result<MirModule, SchedDiag> {
    let diag = |stage: &str, code: &str, message: String| SchedDiag {
        stage: stage.into(),
        code: code.into(),
        message,
    };
    let compiled = ace_core::source::compile_scheduler_source(src, "mn.at", &[]).map_err(|ds| {
        let d = ds.first();
        diag(
            d.map(|x| format!("{:?}", x.stage).to_lowercase())
                .unwrap_or_default()
                .as_str(),
            d.map(|x| x.code.clone()).unwrap_or_default().as_str(),
            d.map(|x| x.message.clone()).unwrap_or_default(),
        )
    })?;
    let module =
        ace_lowering::lower_checked_closures(&compiled.checked, entry, 0).map_err(|e| {
            diag(
                "lower",
                "lower",
                match e {
                    ace_lowering::AdapterError::Unsupported { message, .. }
                    | ace_lowering::AdapterError::Invalid { message, .. }
                    | ace_lowering::AdapterError::Resource { message, .. } => message,
                },
            )
        })?;
    ace_av_core::verify_mir(&module).map_err(|e| diag("verify", "mir", format!("mir: {e:?}")))?;
    Ok(module)
}

/// The program-scoped callee table the SCHED_SPAWN/SCHED_GO indices resolve
/// through: one entry per MIR function index, holding the linked unit symbol
/// and the parameter kinds of its own signature.
pub fn sched_callees(module: &MirModule) -> Result<lower::SchedCtx, SchedDiag> {
    let mut callees = Vec::with_capacity(module.funcs.len());
    for f in &module.funcs {
        let symbol = symbols::function_symbol(&module.owner, &f.name);
        let mut params = Vec::new();
        if let Some(sig) = module.sig_of(f.sig) {
            for p in &sig.params {
                match module.types.get(*p as usize) {
                    Some(MirType::I32) | Some(MirType::Char) => {
                        params.push(lower::SchedArgKind::I32)
                    }
                    Some(MirType::Bool) => params.push(lower::SchedArgKind::Bool),
                    // PLAN-1067 T-03: owned text crosses the spawn thunk as
                    // an owned (ptr, len) pair materialized by the runtime.
                    Some(MirType::Str) | Some(MirType::StrView) => {
                        params.push(lower::SchedArgKind::Str)
                    }
                    other => {
                        return Err(SchedDiag {
                            stage: "lower".into(),
                            code: "native.sched-value".into(),
                            message: format!(
                                "scheduler callee `{}` parameter type {other:?} cannot cross the spawn thunk",
                                f.name
                            ),
                        })
                    }
                }
            }
        }
        // PLAN-1067 review3 T-1067-1: carry the callee own unit signature so
        // the spawn thunk declares exactly the same shape (see `SchedCallee`).
        let full = emit::cranelift_sig_of_for(
            module,
            f.sig,
            cranelift_codegen::isa::CallConv::WindowsFastcall,
        )
        .map_err(|e| SchedDiag {
            stage: "lower".into(),
            code: "native.sched-callee".into(),
            message: format!("scheduler callee `{}` signature: {e}", f.name),
        })?;
        let result_text = module
            .sig_of(f.sig)
            .and_then(|s| module.types.get(s.result as usize))
            .is_some_and(|t| matches!(t, MirType::Str | MirType::StrView));
        callees.push(lower::SchedCallee {
            symbol,
            params,
            sig: full,
            result_text,
        });
    }
    Ok(lower::SchedCtx {
        callees,
        report_spans: false,
    })
}

fn has_host_call(module: &MirModule) -> bool {
    module.funcs.iter().any(|f| {
        f.blocks.iter().any(|b| {
            b.instrs
                .iter()
                .any(|i| matches!(i, MirInstr::HostCall { .. }))
        })
    })
}

/// Emit the startup wrapper: `result = entry(); live = mn_sched_drain();
/// ExitProcess(if live != 0 { 72 } else { result })`. The drain is part of
/// the wrapper (never guest code), so every scheduler program exits through
/// the orderly close exactly once.
pub fn emit_sched_wrapper_coff(entry_symbol: &str) -> Result<Vec<u8>, MnError> {
    emit_wrapper(entry_symbol, false, false)
}

fn emit_wrapper(
    entry_symbol: &str,
    uses_platform: bool,
    uses_net: bool,
) -> Result<Vec<u8>, MnError> {
    let isa = emit::make_isa()?;
    let builder = ObjectBuilder::new(isa, "mn8-sched-start", default_libcall_names())
        .map_err(|e| MnError::Backend(format!("object builder: {e}")))?;
    let mut om = ObjectModule::new(builder);
    let conv = CallConv::WindowsFastcall;
    let entry_sig = Signature {
        call_conv: conv,
        params: vec![],
        returns: vec![AbiParam::new(cranelift_codegen::ir::types::I32)],
    };
    let entry_id = om
        .declare_function(entry_symbol, Linkage::Import, &entry_sig)
        .map_err(|e| MnError::Backend(format!("declare entry {entry_symbol}: {e}")))?;
    // PLAN-1067 T-03: the probe trace channel of the embedded owned-text
    // runtime is observation, not language output — switch it off before the
    // entry runs so this lane's stderr stays byte-identical to AV's.
    let trace_sig = Signature {
        call_conv: conv,
        params: vec![AbiParam::new(cranelift_codegen::ir::types::I32)],
        returns: vec![],
    };
    let trace_id = om
        .declare_function("ac_rt_trace_switch", Linkage::Import, &trace_sig)
        .map_err(|e| MnError::Backend(format!("declare ac_rt_trace_switch: {e}")))?;
    let drain_sig = Signature {
        call_conv: conv,
        params: vec![],
        returns: vec![AbiParam::new(cranelift_codegen::ir::types::I32)],
    };
    let drain_id = om
        .declare_function("mn_sched_drain", Linkage::Import, &drain_sig)
        .map_err(|e| MnError::Backend(format!("declare mn_sched_drain: {e}")))?;
    let net_drain = if uses_net {
        Some(
            om.declare_function("mn_sched_net_drain", Linkage::Import, &drain_sig)
                .map_err(|e| MnError::Backend(format!("declare net drain: {e}")))?,
        )
    } else {
        None
    };
    let platform_audit = if uses_platform {
        Some(
            om.declare_function(
                crate::platform_io::LIVE_RESOURCES,
                Linkage::Import,
                &drain_sig,
            )
            .map_err(|e| MnError::Backend(format!("declare platform audit: {e}")))?,
        )
    } else {
        None
    };
    let exit_sig = Signature {
        call_conv: conv,
        params: vec![AbiParam::new(cranelift_codegen::ir::types::I32)],
        returns: vec![],
    };
    let exit_id = om
        .declare_function("ExitProcess", Linkage::Import, &exit_sig)
        .map_err(|e| MnError::Backend(format!("declare ExitProcess: {e}")))?;
    let start_sig = Signature {
        call_conv: conv,
        params: vec![],
        returns: vec![],
    };
    let start_id = om
        .declare_function(START_SYMBOL, Linkage::Export, &start_sig)
        .map_err(|e| MnError::Backend(format!("declare {START_SYMBOL}: {e}")))?;
    let mut ctx = om.make_context();
    ctx.func.signature = start_sig;
    {
        let mut fbc = FunctionBuilderContext::new();
        let mut fb = FunctionBuilder::new(&mut ctx.func, &mut fbc);
        let blk = fb.create_block();
        fb.switch_to_block(blk);
        fb.seal_block(blk);
        let entry_ref = om.declare_func_in_func(entry_id, fb.func);
        let drain_ref = om.declare_func_in_func(drain_id, fb.func);
        let exit_ref = om.declare_func_in_func(exit_id, fb.func);
        let trace_ref = om.declare_func_in_func(trace_id, fb.func);
        let off = fb.ins().iconst(cranelift_codegen::ir::types::I32, 0);
        fb.ins().call(trace_ref, &[off]);
        let call = fb.ins().call(entry_ref, &[]);
        let result = fb.inst_results(call)[0];
        let drain = fb.ins().call(drain_ref, &[]);
        let mut live = fb.inst_results(drain)[0];
        if let Some(net_drain) = net_drain {
            let net = om.declare_func_in_func(net_drain, fb.func);
            let call = fb.ins().call(net, &[]);
            let net_live = fb.inst_results(call)[0];
            live = fb.ins().bor(live, net_live);
        }
        if let Some(platform_audit) = platform_audit {
            let audit = om.declare_func_in_func(platform_audit, fb.func);
            let call = fb.ins().call(audit, &[]);
            let platform_live = fb.inst_results(call)[0];
            live = fb.ins().bor(live, platform_live);
        }
        let zero = fb.ins().iconst(cranelift_codegen::ir::types::I32, 0);
        let leaked = fb.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::NotEqual,
            live,
            zero,
        );
        let leak_code = fb.ins().iconst(
            cranelift_codegen::ir::types::I32,
            SCHED_LEAK_EXIT_CODE as i64,
        );
        let exit_code = fb.ins().select(leaked, leak_code, result);
        fb.ins().call(exit_ref, &[exit_code]);
        fb.ins().return_(&[]);
        fb.finalize();
    }
    om.define_function(start_id, &mut ctx)
        .map_err(|e| MnError::Backend(format!("define {START_SYMBOL}: {e}")))?;
    om.finish()
        .emit()
        .map_err(|e| MnError::Backend(format!("emit scheduler wrapper: {e}")))
}

/// Build the scheduler PE in `workdir`: per-function scheduler units + the
/// `mn.sched.runtime/1` static library (+ the console shim when the program
/// prints, + the text runtime when it carries text operations) + the
/// drain-bearing startup wrapper.
pub fn sched_native_build(
    module: &MirModule,
    entry: &str,
    workdir: &Path,
) -> Result<link_driver::PeLink, MnError> {
    sched_native_build_inner(module, entry, workdir, None)
}

/// Link a real scheduler PE with a source-bound runtime prepared by the
/// caller. The object/link phase never invokes Cargo to create the runtime.
pub fn sched_native_build_prepared(
    module: &MirModule,
    entry: &str,
    workdir: &Path,
    runtime: &Path,
    runtime_hash: &str,
) -> Result<link_driver::PeLink, MnError> {
    sched_native_build_inner(module, entry, workdir, Some((runtime, runtime_hash)))
}

fn sched_native_build_inner(
    module: &MirModule,
    entry: &str,
    workdir: &Path,
    prepared: Option<(&Path, &str)>,
) -> Result<link_driver::PeLink, MnError> {
    if let Some((runtime, hash)) = prepared {
        if crate::sched_runtime::file_hash(runtime)? != hash {
            return Err(MnError::Admit(
                "runtime-content",
                "prepared scheduler runtime has changed".into(),
            ));
        }
    }
    let program = emit_sched_program_objects(module, entry)?;
    // The scheduler staticlib includes the owned-text support symbols.
    let library = match prepared {
        Some((runtime, _)) => runtime.to_path_buf(),
        None => crate::sched_runtime::prepare()?.library,
    };
    let objects: Vec<_> = program
        .units
        .into_iter()
        .map(|unit| ObjectInput {
            name: unit.symbol,
            bytes: unit.bytes,
        })
        .collect();
    link_driver::link_only(
        &objects,
        &program.wrapper,
        START_SYMBOL,
        workdir,
        &[library],
        if program.uses_platform {
            "sched-platform/1"
        } else {
            "plan1067-sched-lane"
        },
    )
}

pub struct SchedProgramObjects {
    pub entry_symbol: String,
    pub units: Vec<emit::UnitObject>,
    pub wrapper: Vec<u8>,
    pub uses_platform: bool,
}

/// This boundary accepts only the provided MIR closure. Runtime libraries
/// and the linker are selected by the caller after object production.
pub fn emit_sched_program_objects(
    module: &MirModule,
    entry: &str,
) -> Result<SchedProgramObjects, MnError> {
    emit_objects(module, entry, false)
}

pub fn emit_sched_service_objects(
    module: &MirModule,
    entry: &str,
) -> Result<SchedProgramObjects, MnError> {
    emit_objects(module, entry, true)
}

fn emit_objects(
    module: &MirModule,
    entry: &str,
    report_spans: bool,
) -> Result<SchedProgramObjects, MnError> {
    ace_av_core::verify_mir(module)
        .map_err(|error| MnError::Unsupported("sched.mir", format!("{error:?}")))?;
    let entry_unit = module
        .funcs
        .get(module.entry as usize)
        .ok_or_else(|| MnError::Unsupported("sched.entry", "module entry missing".to_string()))?;
    if entry_unit.name != entry {
        return Err(MnError::Unsupported(
            "sched.entry",
            format!("entry unit `{}` is not `{entry}`", entry_unit.name),
        ));
    }
    if !module.sig_of(entry_unit.sig).is_some_and(|sig| {
        sig.params.is_empty()
            && matches!(
                module.types.get(sig.result as usize),
                Some(MirType::I32 | MirType::Bool | MirType::Char)
            )
    }) {
        return Err(MnError::Unsupported(
            "sched.entry-abi",
            "scheduler startup requires a zero-argument i32-family entry".into(),
        ));
    }
    let entry_symbol = symbols::function_symbol(&module.owner, entry);
    let mut callees = sched_callees(module)
        .map_err(|d| MnError::Unsupported("sched.value", format!("{}: {}", d.code, d.message)))?;
    callees.report_spans = report_spans;
    let units = ace_av::cache::split_units(module)
        .map_err(|e| MnError::Backend(format!("scheduler split: {e}")))?;
    for unit in &units {
        emit::admit_sched_unit(unit)?;
    }
    let mut objects = Vec::with_capacity(units.len() + 2);
    for unit in &units {
        let object = emit::emit_sched_unit_coff(unit, &callees)?;
        objects.push(object);
    }
    if has_host_call(module) {
        objects.push(emit::UnitObject {
            symbol: "mn-console-shim".into(),
            imports: Vec::new(),
            bytes: crate::console::emit_console_shim_coff()?,
        });
    }
    let platform_calls = module
        .funcs
        .iter()
        .flat_map(|function| function.blocks.iter().flat_map(|block| &block.instrs))
        .filter_map(|instruction| match instruction {
            MirInstr::PlatformCall { host, .. } => module.host_imports.get(*host as usize),
            _ => None,
        });
    let uses_platform = platform_calls.clone().next().is_some();
    let uses_net = platform_calls
        .into_iter()
        .any(|decl| decl.category == ace_av_core::platform_io::CATEGORY_NET);
    if uses_platform {
        objects.push(emit::UnitObject {
            symbol: "mn-platform-shim".into(),
            imports: Vec::new(),
            bytes: crate::platform_io::emit_platform_shim_with_net(uses_net)?,
        });
    }
    let wrapper = emit_wrapper(&entry_symbol, uses_platform, uses_net)?;
    Ok(SchedProgramObjects {
        entry_symbol,
        units: objects,
        wrapper,
        uses_platform,
    })
}

/// Compile → build → run ONE scheduler program on the native lane. The run
/// is a contained real PE execution with byte-exact stream capture; the
/// observed values are returned, never PASS/FAIL.
pub fn sched_native_run(src: &str, entry: &str, workdir: &Path) -> Result<SchedNativeRun, MnError> {
    let module = sched_compile(src, entry).map_err(|d| {
        MnError::Unsupported("sched.frontend", format!("{}: {}", d.code, d.message))
    })?;
    let linked = sched_native_build(&module, entry, workdir)?;
    let run = link_driver::run_pe(
        &linked.exe_path,
        RUN_BUDGET,
        CaptureCaps {
            stdout_max: STREAM_CAP,
            stderr_max: STREAM_CAP,
        },
    )?;
    Ok(SchedNativeRun {
        exit_code: run.exit_code,
        stdout: run.stdout,
        stderr: run.stderr,
    })
}
