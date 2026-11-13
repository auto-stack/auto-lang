//! PLAN-947: the console/1 lane on the MIR-native backend.
//!
//! The lane reuses the AV console front door verbatim (front-door admit →
//! `compile_host_console_source` → `lower_checked_console` → `verify_mir` →
//! the ABC identity admission), then performs the native half of console/1:
//! per-run authorization through the SHARED `av-core` host contract
//! (structural credentials never carry authority), a per-function unit
//! split whose units carry their console declarations, `Console`-mode
//! emission (guest-side exact formatting + the per-call budget, one shim
//! dispatch per call), a shim object that keeps the cumulative run budget
//! and performs one atomic OS write per call, a real multi-object link and
//! a contained real PE run. Nothing wraps the old VM.

use crate::config::{
    CONSOLE_REJECT_EXIT_CODE, CONSOLE_RUN_BYTES_SYMBOL, CONSOLE_WRITE_SYMBOL, MAX_COFF_BYTES,
    RUN_BUDGET, START_SYMBOL, STREAM_CAP,
};
use crate::link_driver::{self, ObjectInput};
use crate::{emit, symbols, MnError};
use ace_av_core::mir::{MirModule, MirType};
use ace_core::link::CaptureCaps;
use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::ir::{
    AbiParam, InstBuilder, MemFlags, Signature, StackSlotData, StackSlotKind,
};
use cranelift_codegen::isa::CallConv;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_module::{default_libcall_names, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};
use std::path::Path;

/// One console-lane run observation (never expected values).
#[derive(Debug)]
pub struct ConsoleNativeRun {
    pub exit_code: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

/// Stage/code pair of the shared front door (the same taxonomy the AV
/// console recipe reports).
#[derive(Debug)]
pub struct ConsoleDiag {
    pub stage: String,
    pub code: String,
    pub message: String,
}

/// Compile through the shared console front door and admit the ABC
/// identity exactly like the AV lane, returning the verified module MIR.
pub fn console_compile(src: &str, entry: &str) -> Result<MirModule, ConsoleDiag> {
    let program = ace_av::console_host::compile(src, entry).map_err(|d| ConsoleDiag {
        stage: d.stage,
        code: d.code,
        message: d.message,
    })?;
    let loaded = program.load().map_err(|d| ConsoleDiag {
        stage: d.stage,
        code: d.code,
        message: d.message,
    })?;
    Ok(loaded.module().clone())
}

/// Per-run authorization + validation through the shared console/1 host
/// contract. An unauthorized run is rejected BEFORE any emission or
/// execution — dispatch count zero, no partial output (AC-02).
pub fn console_authorize(mir: &MirModule, grants: &[String]) -> Result<(), ConsoleDiag> {
    let reject = |code: &str, message: String| ConsoleDiag {
        stage: "admission".into(),
        code: code.into(),
        message,
    };
    ace_av_core::host::validate(mir).map_err(|e| reject(e.code, e.message))?;
    ace_av_core::host::authorize(mir, grants).map_err(|e| reject(e.code, e.message))?;
    Ok(())
}

const T_I32: cranelift_codegen::ir::types::Type = cranelift_codegen::ir::types::I32;
const T_I64: cranelift_codegen::ir::types::Type = cranelift_codegen::ir::types::I64;

/// Emit the console shim object: defines [`CONSOLE_WRITE_SYMBOL`] and the
/// cumulative run-byte counter. The shim performs one kernel32 `WriteFile`
/// per call (atomic per call) and enforces the console/1 budgets
/// defensively (the guest already enforced the per-call budget before
/// dispatch). Status: 0 written; 1 per-call budget; 2 cumulative budget;
/// 3 OS write failure.
pub fn emit_console_shim_coff() -> Result<Vec<u8>, MnError> {
    let isa = emit::make_isa()?;
    let builder = ObjectBuilder::new(isa, "mn-console-shim", default_libcall_names())
        .map_err(|e| MnError::Backend(format!("object builder: {e}")))?;
    let mut om = ObjectModule::new(builder);
    let wincall = CallConv::WindowsFastcall;
    // HANDLE GetStdHandle(DWORD id)
    let gsh_sig = Signature {
        call_conv: wincall,
        params: vec![AbiParam::new(T_I32)],
        returns: vec![AbiParam::new(T_I64)],
    };
    let gsh = om
        .declare_function("GetStdHandle", Linkage::Import, &gsh_sig)
        .map_err(|e| MnError::Backend(format!("declare GetStdHandle: {e}")))?;
    // BOOL WriteFile(HANDLE, LPCVOID, DWORD, LPDWORD, LPOVERLAPPED)
    let wf_sig = Signature {
        call_conv: wincall,
        params: vec![
            AbiParam::new(T_I64),
            AbiParam::new(T_I64),
            AbiParam::new(T_I32),
            AbiParam::new(T_I64),
            AbiParam::new(T_I64),
        ],
        returns: vec![AbiParam::new(T_I32)],
    };
    let wf = om
        .declare_function("WriteFile", Linkage::Import, &wf_sig)
        .map_err(|e| MnError::Backend(format!("declare WriteFile: {e}")))?;
    let counter = om
        .declare_data(CONSOLE_RUN_BYTES_SYMBOL, Linkage::Export, true, false)
        .map_err(|e| MnError::Backend(format!("declare counter: {e}")))?;
    // i32 mn_console_write(i32 stream, i64 ptr, i64 len)
    let write_sig = Signature {
        call_conv: wincall,
        params: vec![
            AbiParam::new(T_I32),
            AbiParam::new(T_I64),
            AbiParam::new(T_I64),
        ],
        returns: vec![AbiParam::new(T_I32)],
    };
    let write_id = om
        .declare_function(CONSOLE_WRITE_SYMBOL, Linkage::Export, &write_sig)
        .map_err(|e| MnError::Backend(format!("declare {CONSOLE_WRITE_SYMBOL}: {e}")))?;
    let mut ctx = om.make_context();
    ctx.func.signature = write_sig;
    {
        let mut fbc = FunctionBuilderContext::new();
        let mut fb = FunctionBuilder::new(&mut ctx.func, &mut fbc);
        let blk = fb.create_block();
        fb.switch_to_block(blk);
        fb.seal_block(blk);
        fb.append_block_params_for_function_params(blk);
        let stream = fb.block_params(blk)[0];
        let ptr = fb.block_params(blk)[1];
        let len = fb.block_params(blk)[2];
        let ret = |fb: &mut FunctionBuilder, code: i64| {
            let v = fb.ins().iconst(T_I32, code);
            fb.ins().return_(&[v]);
        };
        // Defensive per-call budget re-check (the guest guards first).
        let call_cap = fb
            .ins()
            .iconst(T_I64, ace_av_core::host::MAX_CALL_BYTES as i64);
        let over = fb.ins().icmp(IntCC::UnsignedGreaterThan, len, call_cap);
        let bad_call = fb.create_block();
        let cont1 = fb.create_block();
        fb.ins().brif(over, bad_call, &[], cont1, &[]);
        fb.seal_block(bad_call);
        fb.seal_block(cont1);
        fb.switch_to_block(bad_call);
        ret(&mut fb, 1);
        fb.switch_to_block(cont1);
        // Cumulative run budget.
        let counter_in_func = om.declare_data_in_func(counter, fb.func);
        let counter_addr = fb.ins().symbol_value(T_I64, counter_in_func);
        let run = fb.ins().load(T_I64, MemFlags::trusted(), counter_addr, 0);
        let next = fb.ins().iadd(run, len);
        let run_cap = fb
            .ins()
            .iconst(T_I64, ace_av_core::host::MAX_RUN_BYTES as i64);
        let over_run = fb.ins().icmp(IntCC::UnsignedGreaterThan, next, run_cap);
        let bad_run = fb.create_block();
        let cont2 = fb.create_block();
        fb.ins().brif(over_run, bad_run, &[], cont2, &[]);
        fb.seal_block(bad_run);
        fb.seal_block(cont2);
        fb.switch_to_block(bad_run);
        ret(&mut fb, 2);
        fb.switch_to_block(cont2);
        // stream == 0 → STD_OUTPUT_HANDLE (-11) else STD_ERROR_HANDLE (-12).
        let zero = fb.ins().iconst(T_I32, 0);
        let is_out = fb.ins().icmp(IntCC::Equal, stream, zero);
        let id_out = fb.ins().iconst(T_I32, -11);
        let id_err = fb.ins().iconst(T_I32, -12);
        let id = fb.ins().select(is_out, id_out, id_err);
        let gsh_ref = om.declare_func_in_func(gsh, fb.func);
        let handle_call = fb.ins().call(gsh_ref, &[id]);
        let handle = fb.inst_results(handle_call)[0];
        let written =
            fb.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, 4, 2));
        let written_addr = fb.ins().stack_addr(T_I64, written, 0);
        let len32 = fb.ins().ireduce(T_I32, len);
        let null = fb.ins().iconst(T_I64, 0);
        let wf_ref = om.declare_func_in_func(wf, fb.func);
        let call = fb
            .ins()
            .call(wf_ref, &[handle, ptr, len32, written_addr, null]);
        let ok = fb.inst_results(call)[0];
        let one = fb.ins().iconst(T_I32, 1);
        let ok_bit = fb.ins().icmp(IntCC::Equal, ok, one);
        let io_fail = fb.create_block();
        let cont3 = fb.create_block();
        fb.ins().brif(ok_bit, cont3, &[], io_fail, &[]);
        fb.seal_block(io_fail);
        fb.seal_block(cont3);
        fb.switch_to_block(io_fail);
        ret(&mut fb, 3);
        fb.switch_to_block(cont3);
        // Commit the counter only after a successful whole-line write.
        fb.ins().store(MemFlags::trusted(), next, counter_addr, 0);
        ret(&mut fb, 0);
        fb.finalize();
    }
    om.define_function(write_id, &mut ctx)
        .map_err(|e| MnError::Backend(format!("define {CONSOLE_WRITE_SYMBOL}: {e}")))?;
    let mut counter_init = cranelift_module::DataDescription::new();
    counter_init.define_zeroinit(8);
    om.define_data(counter, &counter_init)
        .map_err(|e| MnError::Backend(format!("define counter: {e}")))?;
    let product = om.finish();
    let bytes = product
        .emit()
        .map_err(|e| MnError::Backend(format!("emit shim object: {e}")))?;
    if bytes.len() > MAX_COFF_BYTES {
        return Err(MnError::Resource(
            "coff-size",
            format!(
                "console shim object {} bytes above the frozen cap",
                bytes.len()
            ),
        ));
    }
    Ok(bytes)
}

/// The console entry wrapper: `ac_start` calls the entry function and
/// forwards its i32 result to ExitProcess (a Unit entry exits 0). The
/// wrapper mirrors `emit_wrapper_coff` but admits the Unit result channel
/// the console profile allows.
fn emit_console_wrapper_coff(entry_symbol: &str, unit_result: bool) -> Result<Vec<u8>, MnError> {
    let isa = emit::make_isa()?;
    let builder = ObjectBuilder::new(isa, "mn-console-start", default_libcall_names())
        .map_err(|e| MnError::Backend(format!("object builder: {e}")))?;
    let mut om = ObjectModule::new(builder);
    let wincall = CallConv::WindowsFastcall;
    let entry_sig = Signature {
        call_conv: wincall,
        params: vec![],
        returns: if unit_result {
            vec![AbiParam::new(T_I32)]
        } else {
            vec![]
        },
    };
    let entry_id = om
        .declare_function(entry_symbol, Linkage::Import, &entry_sig)
        .map_err(|e| MnError::Backend(format!("declare entry {entry_symbol}: {e}")))?;
    let exit_sig = Signature {
        call_conv: wincall,
        params: vec![AbiParam::new(T_I32)],
        returns: vec![],
    };
    let exit_id = om
        .declare_function("ExitProcess", Linkage::Import, &exit_sig)
        .map_err(|e| MnError::Backend(format!("declare ExitProcess: {e}")))?;
    let start_sig = Signature {
        call_conv: wincall,
        params: vec![],
        returns: vec![],
    };
    let start_id = om
        .declare_function(START_SYMBOL, Linkage::Export, &start_sig)
        .map_err(|e| MnError::Backend(format!("declare {}: {e}", START_SYMBOL)))?;
    let mut ctx = om.make_context();
    ctx.func.signature = start_sig;
    {
        let mut fbc = FunctionBuilderContext::new();
        let mut fb = FunctionBuilder::new(&mut ctx.func, &mut fbc);
        let blk = fb.create_block();
        fb.switch_to_block(blk);
        fb.seal_block(blk);
        let entry_ref = om.declare_func_in_func(entry_id, fb.func);
        let exit_ref = om.declare_func_in_func(exit_id, fb.func);
        if unit_result {
            let call = fb.ins().call(entry_ref, &[]);
            let result = fb.inst_results(call)[0];
            fb.ins().call(exit_ref, &[result]);
        } else {
            fb.ins().call(entry_ref, &[]);
            let zero = fb.ins().iconst(T_I32, 0);
            fb.ins().call(exit_ref, &[zero]);
        }
        fb.ins().return_(&[]);
        fb.finalize();
    }
    om.define_function(start_id, &mut ctx)
        .map_err(|e| MnError::Backend(format!("define {}: {e}", START_SYMBOL)))?;
    let product = om.finish();
    product
        .emit()
        .map_err(|e| MnError::Backend(format!("emit console wrapper: {e}")))
}

/// The console/1 native lane: authorize per run → split → emit → link →
/// one contained real PE run. Exit codes: the entry result (success), 70
/// (the frozen mn trap), [`CONSOLE_REJECT_EXIT_CODE`] (console resource
/// rejection). Output capture is byte-exact.
pub fn console_native_run(
    mir: &MirModule,
    entry: &str,
    workdir: &Path,
) -> Result<ConsoleNativeRun, MnError> {
    // Entry identity: the module entry names the exported unit symbol.
    let entry_unit = mir
        .funcs
        .get(mir.entry as usize)
        .ok_or_else(|| MnError::Unsupported("console.entry", "module entry missing".to_string()))?;
    if entry_unit.name != entry {
        return Err(MnError::Unsupported(
            "console.entry",
            format!("entry unit `{}` is not `{entry}`", entry_unit.name),
        ));
    }
    let entry_symbol = symbols::function_symbol(&mir.owner, entry);
    let unit_result = match mir.sig_of(entry_unit.sig) {
        Some(sig) => matches!(
            mir.types.get(sig.result as usize),
            Some(MirType::I32) | Some(MirType::Bool) | Some(MirType::Char)
        ),
        None => false,
    };
    let units = ace_av::cache::split_units(mir)
        .map_err(|e| MnError::Backend(format!("console split: {e}")))?;
    let mut objects = Vec::with_capacity(units.len() + 2);
    for (i, unit) in units.iter().enumerate() {
        let object = emit::emit_console_unit_coff(unit)?;
        objects.push(ObjectInput {
            name: format!("mn-console-unit-{i}"),
            bytes: object.bytes,
        });
    }
    objects.push(ObjectInput {
        name: "mn-console-shim".into(),
        bytes: emit_console_shim_coff()?,
    });
    let wrapper = emit_console_wrapper_coff(&entry_symbol, unit_result)?;
    let linked = link_driver::link_only(
        &objects,
        &wrapper,
        START_SYMBOL,
        workdir,
        &[],
        "plan947-console-lane",
    )?;
    let run = link_driver::run_pe(
        &linked.exe_path,
        RUN_BUDGET,
        CaptureCaps {
            stdout_max: STREAM_CAP,
            stderr_max: STREAM_CAP,
        },
    )?;
    Ok(ConsoleNativeRun {
        exit_code: run.exit_code,
        stdout: run.stdout,
        stderr: run.stderr,
    })
}

/// The console/1 native resource-reject exit (recipe reporting helper).
pub fn console_reject_exit() -> i32 {
    CONSOLE_REJECT_EXIT_CODE
}
