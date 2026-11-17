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
    let module = ace_lowering::lower_checked_closures(&compiled.checked, entry, 0).map_err(|e| {
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
    ace_av_core::verify_mir(&module)
        .map_err(|e| diag("verify", "mir", format!("mir: {e:?}")))?;
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
                    Some(MirType::Str) | Some(MirType::StrView) => {
                        return Err(SchedDiag {
                            stage: "lower".into(),
                            code: "native.sched-value".into(),
                            message: format!(
                                "owned text spawn initializer on `{}` is not in the scheduler lane's first cut (PLAN-1067 T-03)",
                                f.name
                            ),
                        })
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
        callees.push(lower::SchedCallee { symbol, params });
    }
    Ok(lower::SchedCtx { callees })
}

fn has_text_ops(module: &MirModule) -> bool {
    module.funcs.iter().any(|f| {
        f.blocks.iter().any(|b| {
            b.instrs.iter().any(|i| {
                matches!(
                    i,
                    MirInstr::StrCharAt { .. }
                        | MirInstr::StrSlice { .. }
                        | MirInstr::StrConcat { .. }
                        | MirInstr::StrToOwned { .. }
                        | MirInstr::StrFromChar { .. }
                )
            })
        })
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
    let drain_sig = Signature {
        call_conv: conv,
        params: vec![],
        returns: vec![AbiParam::new(cranelift_codegen::ir::types::I32)],
    };
    let drain_id = om
        .declare_function("mn_sched_drain", Linkage::Import, &drain_sig)
        .map_err(|e| MnError::Backend(format!("declare mn_sched_drain: {e}")))?;
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
        let call = fb.ins().call(entry_ref, &[]);
        let result = fb.inst_results(call)[0];
        let drain = fb.ins().call(drain_ref, &[]);
        let live = fb.inst_results(drain)[0];
        let zero = fb.ins().iconst(cranelift_codegen::ir::types::I32, 0);
        let leaked = fb.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::NotEqual,
            live,
            zero,
        );
        let leak_code = fb
            .ins()
            .iconst(cranelift_codegen::ir::types::I32, SCHED_LEAK_EXIT_CODE as i64);
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
    let entry_symbol = symbols::function_symbol(&module.owner, entry);
    let callees = sched_callees(module).map_err(|d| {
        MnError::Unsupported(
            "sched.value",
            format!("{}: {}", d.code, d.message),
        )
    })?;
    let units = ace_av::cache::split_units(module)
        .map_err(|e| MnError::Backend(format!("scheduler split: {e}")))?;
    let mut objects = Vec::with_capacity(units.len() + 2);
    for (i, unit) in units.iter().enumerate() {
        let object = emit::emit_sched_unit_coff(unit, &callees)?;
        objects.push(ObjectInput {
            name: format!("mn-sched-unit-{i}"),
            bytes: object.bytes,
        });
    }
    if has_host_call(module) {
        objects.push(ObjectInput {
            name: "mn-console-shim".into(),
            bytes: crate::console::emit_console_shim_coff()?,
        });
    }
    let wrapper = emit_sched_wrapper_coff(&entry_symbol)?;
    let _ = has_text_ops(module);
    // ONE Rust support staticlib: the scheduler runtime embeds the owned-text
    // runtime as a dependency (its text/alloc ABI + the compiler-generated
    // mem* intrinsics). Two separate Rust staticlibs collide on the panic /
    // unwind runtime symbols.
    let support_libs = vec![crate::sched_runtime::prepare()?.library];
    link_driver::link_only(
        &objects,
        &wrapper,
        START_SYMBOL,
        workdir,
        &support_libs,
        "plan1067-sched-lane",
    )
}

/// Compile → build → run ONE scheduler program on the native lane. The run
/// is a contained real PE execution with byte-exact stream capture; the
/// observed values are returned, never PASS/FAIL.
pub fn sched_native_run(
    src: &str,
    entry: &str,
    workdir: &Path,
) -> Result<SchedNativeRun, MnError> {
    let module = sched_compile(src, entry)
        .map_err(|d| MnError::Unsupported("sched.frontend", format!("{}: {}", d.code, d.message)))?;
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
