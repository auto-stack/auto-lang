//! PLAN-1065: REAL machine-code bodies for the Windows-applicable sys973
//! platform operations (`platform.file` / `platform.file.path` /
//! `platform.env` / `platform.clock` / `platform.entropy` /
//! `platform.process`, 36 W ops) inside the existing `mn-platform-shim`
//! COFF — Win32 calls under the SAME `mn_pio_state` ledger, mirroring the
//! AV reference (`PlatformResourceLedger` + `Win32PlatformIo`) semantics:
//! handle encode/decode, origin/release checks, per-call/run budgets,
//! `last_os_error` / per-file `last64_hi` / global `clock_hi`, EOF-vs-
//! capacity, broken-pipe-as-EOF, V-03 process reclamation. Zero libc/CRT,
//! zero host fallback; the 14 raw-bytes ops that do not apply to the
//! Windows platform form stay named-unsupported stubs (Linux legs belong
//! to PLAN-1063).
//!
//! Block discipline: `branch2` seals BOTH blocks, so the ok block may
//! never gain another predecessor; joins use the `split_open` pattern
//! (seal only the first arm, seal the join after every jump).

use crate::platform_io::{
    declare_fn, define_fn, shim_symbol_for, sig, state_ptr, state_scratch_ptr, sys973_shim_sig,
    LocalIds, H_BYTES_WRITTEN_TMP, H_LAST_CLOCK_HI, H_LAST_OS_ERROR, H_READ_TOTAL, H_WRITTEN_TOTAL,
    KIND_DIR, KIND_FILE, KIND_MEM, KIND_PROC, KIND_STREAM, SLOT_BYTES, STATE_BYTES,
    STATE_HEADER_BYTES, SYS973_WINDOWS_N_KINDS, S_DIR_PENDING, S_DIR_PENDING_LEN, S_LAST_HI,
    S_LOGICAL_LEN, S_OS_PTR, S_PROC_ERR, S_PROC_JOB, S_PROC_OUT, S_RELEASED, S_STREAM_ID,
};
use crate::MnError;
use ace_av_core::platform_io::{
    PlatformOpKind, ERR_ALREADY_RELEASED, ERR_BUDGET, ERR_INVALID_BOUNDS, ERR_INVALID_HANDLE,
    ERR_INVALID_VALUE, ERR_OS_FAILED, ERR_TIMEOUT, ERR_WRONG_ORIGIN, MAX_PLATFORM_CALL_BYTES,
    MAX_PLATFORM_RUN_BYTES,
};
use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::ir::{
    types, Block, FuncRef, InstBuilder, MemFlags, StackSlotData, StackSlotKind, TrapCode, Value,
};
use cranelift_codegen::isa::CallConv;
use cranelift_frontend::FunctionBuilder;
use cranelift_module::{FuncId, Linkage, Module};

// ---- Win32 struct layouts (x64, mirrors av-core `win32` repr(C)) --------
const WFD_BYTES: u32 = 600; // WIN32_FIND_DATAW (real 592)
const WFD_CFILENAME: i32 = 44;
const BHFI_BYTES: u32 = 64; // BY_HANDLE_FILE_INFORMATION (real 56)
const BHFI_FTWRITE: i32 = 20;
const SI_BYTES: u32 = 112; // STARTUPINFOW (real 104)
const SI_CB: i32 = 0;
const SI_DWFLAGS: i32 = 60;
const SI_HSTDIN: i32 = 80;
const SI_HSTDOUT: i32 = 88;
const SI_HSTDERR: i32 = 96;
const PI_BYTES: u32 = 32; // PROCESS_INFORMATION (real 24)
const PI_HPROCESS: i32 = 0;
const PI_HTHREAD: i32 = 8;
const SA_BYTES: u32 = 32; // SECURITY_ATTRIBUTES (real 24)
const SA_NLENGTH: i32 = 0;
const SA_INHERIT: i32 = 16;
const JOBI_BYTES: u32 = 152; // JOBOBJECT_EXTENDED_LIMIT_INFORMATION (real 144)
const JOBI_LIMITFLAGS: i32 = 16;

/// Unit caps mirroring the AV ledger (`MAX_PROBE_PATH_BYTES` units; the
/// Windows env/cmdline representation limit is 32,767 chars).
const PATH_UNITS_MAX: i32 = 4096;
const PATH_BUF_BYTES: u32 = (PATH_UNITS_MAX as u32 + 8) * 2;
const ENV_UNITS_MAX: i32 = 32_768;
const CANON_FINAL_UNITS: i32 = 32_776; // AV final-path scratch bound (32768+8)

// Win32 flag constants (public semantics; see av-core `win32`).
const GENERIC_READ: i64 = 0x8000_0000;
const GENERIC_WRITE: i64 = 0x4000_0000;
const FILE_APPEND_DATA: i64 = 0x0000_0004;
const FILE_SHARE_ALL: i64 = 0x7;
const FILE_ATTRIBUTE_NORMAL: i64 = 0x80;
const FILE_READ_ATTRIBUTES: i64 = 0x0080;
const FILE_FLAG_BACKUP_SEMANTICS: i64 = 0x0200_0000;
const MOVEFILE_REPLACE_EXISTING: i64 = 0x1;
const INVALID_HANDLE_VALUE: i64 = -1;
const FILE_ATTRIBUTE_READONLY: i64 = 0x1;
const FILE_ATTRIBUTE_DIRECTORY: i64 = 0x10;
const STARTF_USESTDHANDLES: i64 = 0x100;
const CREATE_NO_WINDOW: i64 = 0x0800_0000;
const INFINITE_U32: i64 = 0xFFFF_FFFF;
const WAIT_OBJECT_0: i64 = 0;
const WAIT_TIMEOUT_CODE: i64 = 0x102;
const JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE: i64 = 0x2000;
const JOBOBJECT_EXTENDED_LIMIT_INFO_CLASS: i64 = 9;
const JOBI_REAL_BYTES: i64 = 144;
const BCRYPT_USE_SYSTEM_PREFERRED_RNG: i64 = 0x2;
const NTSTATUS_FAULT_MARKER: i64 = 0x0010_4000;
const FILETIME_UNIX_EPOCH_100NS: i64 = 116_444_736_000_000_000;
const ERROR_BROKEN_PIPE: i64 = 109;
const ERROR_NO_DATA: i64 = 232;
const ERROR_NO_MORE_FILES: i64 = 18;
const ERROR_MORE_DATA: i64 = 234;
const ERROR_INSUFFICIENT_BUFFER: i64 = 122;
const ERROR_INVALID_PARAMETER: i64 = 87;
const STD_INPUT_ID: i64 = -10;
const PIPE_BUF: i64 = 1 << 20;

type ObjModule = cranelift_object::ObjectModule;
type DataId = cranelift_module::DataId;

pub(crate) struct Sys973Ids {
    get_last_error: FuncId,
    set_last_error: FuncId,
    create_file_w: FuncId,
    write_file: FuncId,
    read_file: FuncId,
    close_handle: FuncId,
    set_file_pointer_ex: FuncId,
    get_file_size_ex: FuncId,
    get_file_information_by_handle: FuncId,
    flush_file_buffers: FuncId,
    get_file_attributes_w: FuncId,
    delete_file_w: FuncId,
    create_directory_w: FuncId,
    remove_directory_w: FuncId,
    move_file_ex_w: FuncId,
    get_final_path_name_by_handle_w: FuncId,
    find_first_file_w: FuncId,
    find_next_file_w: FuncId,
    find_close: FuncId,
    get_environment_variable_w: FuncId,
    set_environment_variable_w: FuncId,
    get_system_time_as_file_time: FuncId,
    query_performance_frequency: FuncId,
    query_performance_counter: FuncId,
    sleep_ex: FuncId,
    create_job_object_w: FuncId,
    set_information_job_object: FuncId,
    create_pipe: FuncId,
    set_handle_information: FuncId,
    create_process_w: FuncId,
    assign_process_to_job_object: FuncId,
    terminate_job_object: FuncId,
    wait_for_single_object: FuncId,
    get_exit_code_process: FuncId,
    peek_named_pipe: FuncId,
    get_current_directory_w: FuncId,
    set_current_directory_w: FuncId,
    get_command_line_w: FuncId,
    exit_process: FuncId,
    bcrypt_gen_random: FuncId,
}

fn declare_sys973_imports(om: &mut ObjModule) -> Result<Sys973Ids, MnError> {
    use types as t;
    let imp = |om: &mut ObjModule, name: &str, s| -> Result<FuncId, MnError> {
        declare_fn(om, name, Linkage::Import, &s)
    };
    Ok(Sys973Ids {
        get_last_error: imp(om, "GetLastError", sig(&[], Some(t::I32)))?,
        set_last_error: imp(om, "SetLastError", sig(&[t::I32], None))?,
        create_file_w: imp(
            om,
            "CreateFileW",
            sig(
                &[t::I64, t::I32, t::I32, t::I64, t::I32, t::I32, t::I64],
                Some(t::I64),
            ),
        )?,
        write_file: imp(
            om,
            "WriteFile",
            sig(&[t::I64, t::I64, t::I32, t::I64, t::I64], Some(t::I32)),
        )?,
        read_file: imp(
            om,
            "ReadFile",
            sig(&[t::I64, t::I64, t::I32, t::I64, t::I64], Some(t::I32)),
        )?,
        close_handle: imp(om, "CloseHandle", sig(&[t::I64], Some(t::I32)))?,
        set_file_pointer_ex: imp(
            om,
            "SetFilePointerEx",
            sig(&[t::I64, t::I64, t::I64, t::I32], Some(t::I32)),
        )?,
        get_file_size_ex: imp(om, "GetFileSizeEx", sig(&[t::I64, t::I64], Some(t::I32)))?,
        get_file_information_by_handle: imp(
            om,
            "GetFileInformationByHandle",
            sig(&[t::I64, t::I64], Some(t::I32)),
        )?,
        flush_file_buffers: imp(om, "FlushFileBuffers", sig(&[t::I64], Some(t::I32)))?,
        get_file_attributes_w: imp(om, "GetFileAttributesW", sig(&[t::I64], Some(t::I32)))?,
        delete_file_w: imp(om, "DeleteFileW", sig(&[t::I64], Some(t::I32)))?,
        create_directory_w: imp(om, "CreateDirectoryW", sig(&[t::I64, t::I64], Some(t::I32)))?,
        remove_directory_w: imp(om, "RemoveDirectoryW", sig(&[t::I64], Some(t::I32)))?,
        move_file_ex_w: imp(
            om,
            "MoveFileExW",
            sig(&[t::I64, t::I64, t::I32], Some(t::I32)),
        )?,
        get_final_path_name_by_handle_w: imp(
            om,
            "GetFinalPathNameByHandleW",
            sig(&[t::I64, t::I64, t::I32, t::I32], Some(t::I32)),
        )?,
        find_first_file_w: imp(om, "FindFirstFileW", sig(&[t::I64, t::I64], Some(t::I64)))?,
        find_next_file_w: imp(om, "FindNextFileW", sig(&[t::I64, t::I64], Some(t::I32)))?,
        find_close: imp(om, "FindClose", sig(&[t::I64], Some(t::I32)))?,
        get_environment_variable_w: imp(
            om,
            "GetEnvironmentVariableW",
            sig(&[t::I64, t::I64, t::I32], Some(t::I32)),
        )?,
        set_environment_variable_w: imp(
            om,
            "SetEnvironmentVariableW",
            sig(&[t::I64, t::I64], Some(t::I32)),
        )?,
        get_system_time_as_file_time: imp(om, "GetSystemTimeAsFileTime", sig(&[t::I64], None))?,
        query_performance_frequency: imp(
            om,
            "QueryPerformanceFrequency",
            sig(&[t::I64], Some(t::I32)),
        )?,
        query_performance_counter: imp(
            om,
            "QueryPerformanceCounter",
            sig(&[t::I64], Some(t::I32)),
        )?,
        sleep_ex: imp(om, "SleepEx", sig(&[t::I32, t::I32], Some(t::I32)))?,
        create_job_object_w: imp(om, "CreateJobObjectW", sig(&[t::I64, t::I64], Some(t::I64)))?,
        set_information_job_object: imp(
            om,
            "SetInformationJobObject",
            // (hJob, class, info, len)
            sig(&[t::I64, t::I32, t::I64, t::I32], Some(t::I32)),
        )?,
        create_pipe: imp(
            om,
            "CreatePipe",
            sig(&[t::I64, t::I64, t::I64, t::I32], Some(t::I32)),
        )?,
        set_handle_information: imp(
            om,
            "SetHandleInformation",
            sig(&[t::I64, t::I32, t::I32], Some(t::I32)),
        )?,
        create_process_w: imp(
            om,
            "CreateProcessW",
            sig(
                &[
                    t::I64,
                    t::I64,
                    t::I64,
                    t::I64,
                    t::I32,
                    t::I32,
                    t::I64,
                    t::I64,
                    t::I64,
                    t::I64,
                ],
                Some(t::I32),
            ),
        )?,
        assign_process_to_job_object: imp(
            om,
            "AssignProcessToJobObject",
            sig(&[t::I64, t::I64], Some(t::I32)),
        )?,
        terminate_job_object: imp(
            om,
            "TerminateJobObject",
            sig(&[t::I64, t::I32], Some(t::I32)),
        )?,
        wait_for_single_object: imp(
            om,
            "WaitForSingleObject",
            sig(&[t::I64, t::I32], Some(t::I32)),
        )?,
        get_exit_code_process: imp(
            om,
            "GetExitCodeProcess",
            sig(&[t::I64, t::I64], Some(t::I32)),
        )?,
        peek_named_pipe: imp(
            om,
            "PeekNamedPipe",
            sig(
                &[t::I64, t::I64, t::I32, t::I64, t::I64, t::I64],
                Some(t::I32),
            ),
        )?,
        get_current_directory_w: imp(
            om,
            "GetCurrentDirectoryW",
            sig(&[t::I32, t::I64], Some(t::I32)),
        )?,
        set_current_directory_w: imp(om, "SetCurrentDirectoryW", sig(&[t::I64], Some(t::I32)))?,
        get_command_line_w: imp(om, "GetCommandLineW", sig(&[], Some(t::I64)))?,
        exit_process: imp(om, "ExitProcess", sig(&[t::I32], None))?,
        bcrypt_gen_random: imp(
            om,
            "BCryptGenRandom",
            sig(&[t::I64, t::I64, t::I32, t::I32], Some(t::I32)),
        )?,
    })
}

// ---- small CLIF helpers --------------------------------------------------

fn i32v(fb: &mut FunctionBuilder<'_>, v: i32) -> Value {
    fb.ins().iconst(types::I32, v as i64)
}

fn i64v(fb: &mut FunctionBuilder<'_>, v: i64) -> Value {
    fb.ins().iconst(types::I64, v)
}

fn fref(om: &mut ObjModule, fb: &mut FunctionBuilder<'_>, id: FuncId) -> FuncRef {
    om.declare_func_in_func(id, fb.func)
}

/// (err, ok) pair on `cond`; BOTH sealed — the ok block must never gain
/// another predecessor.
fn branch2(fb: &mut FunctionBuilder<'_>, cond: Value) -> (Block, Block) {
    let err_blk = fb.create_block();
    let ok_blk = fb.create_block();
    fb.ins().brif(cond, err_blk, &[], ok_blk, &[]);
    fb.seal_block(err_blk);
    fb.seal_block(ok_blk);
    (err_blk, ok_blk)
}

/// (first, later) pair on `cond`; `first` is sealed (build it now, it must
/// terminate or jump elsewhere), `later` stays open until the caller emits
/// every remaining predecessor and seals it.
fn split_open(fb: &mut FunctionBuilder<'_>, cond: Value) -> (Block, Block) {
    let first = fb.create_block();
    let later = fb.create_block();
    fb.ins().brif(cond, first, &[], later, &[]);
    fb.seal_block(first);
    (first, later)
}

fn ret_code(fb: &mut FunctionBuilder<'_>, code: i32) {
    let _t0 = i32v(fb, code);

    fb.ins().return_(&[_t0]);
}

/// The AV `os_fail`: record GetLastError (fallback when 0) into the global
/// `last_os_error` slot and return ERR_OS_FAILED. Terminates the block.
fn os_fail(
    om: &mut ObjModule,
    fb: &mut FunctionBuilder<'_>,
    gle: FuncRef,
    state: DataId,
    flags: MemFlags,
    fallback: i32,
) {
    let base = state_ptr(om, state, fb);
    let gc = fb.ins().call(gle, &[]);
    let ecode = fb.inst_results(gc)[0];
    let ez = fb.ins().icmp_imm(IntCC::Equal, ecode, 0);
    let _t1 = i32v(fb, fallback);

    let rec_e = fb.ins().select(ez, _t1, ecode);
    fb.ins().store(flags, rec_e, base, H_LAST_OS_ERROR);
    ret_code(fb, ERR_OS_FAILED);
}

/// Full slot-lookup factor: calls `find_slot(h, kind)`; the negative-result
/// block returns the exact code sign-extended; returns the slot pointer
/// with the builder positioned in the ok block.
fn slot_lookup(fb: &mut FunctionBuilder<'_>, find: FuncRef, h: Value, kind: i64) -> Value {
    let k = i32v(fb, kind as i32);
    let c = fb.ins().call(find, &[h, k]);
    let slot = fb.inst_results(c)[0];
    let neg = fb.ins().icmp_imm(IntCC::SignedLessThan, slot, 0);
    let (bad, ok) = branch2(fb, neg);
    fb.switch_to_block(bad);
    let e = fb.ins().ireduce(types::I32, slot);
    fb.ins().return_(&[e]);
    fb.switch_to_block(ok);
    slot
}

/// AV `op_file_hi` keeps the stored hi even for CLOSED files — a
/// released-tolerant file lookup: on ERR_ALREADY_RELEASED re-derive the
/// slot address from the handle (idx = low16 - 1).
fn slot_lookup_file_norel(
    om: &mut ObjModule,
    fb: &mut FunctionBuilder<'_>,
    find: FuncRef,
    h: Value,
    state: DataId,
) -> Value {
    let kf = i32v(fb, KIND_FILE as i32);
    let c = fb.ins().call(find, &[h, kf]);
    let slot = fb.inst_results(c)[0];
    let neg = fb.ins().icmp_imm(IntCC::SignedLessThan, slot, 0);
    let bad = fb.create_block();
    let ok = fb.create_block();
    fb.append_block_param(ok, types::I64);
    fb.ins().brif(neg, bad, &[], ok, &[slot.into()]);
    fb.seal_block(bad);

    fb.switch_to_block(bad);
    let is_rel = fb
        .ins()
        .icmp_imm(IntCC::Equal, slot, ERR_ALREADY_RELEASED as i64);
    let hard = fb.create_block();
    fb.ins().brif(is_rel, ok, &[slot.into()], hard, &[]);
    fb.seal_block(hard);
    fb.switch_to_block(hard);
    let e = fb.ins().ireduce(types::I32, slot);
    fb.ins().return_(&[e]);

    fb.seal_block(ok);
    fb.switch_to_block(ok);
    let got = fb.block_params(ok)[0];
    let is_ptr = fb.ins().icmp_imm(IntCC::SignedGreaterThan, got, 0);
    let derive = fb.create_block();
    let pass = fb.create_block();
    fb.append_block_param(pass, types::I64);
    fb.ins().brif(is_ptr, pass, &[got.into()], derive, &[]);
    fb.seal_block(derive);
    fb.seal_block(pass);
    fb.switch_to_block(derive);
    let mask16 = i32v(fb, 0xFFFF);
    let low16 = fb.ins().band(h, mask16);
    let _t2 = i32v(fb, 1);

    let idx = fb.ins().isub(low16, _t2);
    let base = state_ptr(om, state, fb);
    let idx64 = fb.ins().uextend(types::I64, idx);
    let off = fb.ins().imul_imm(idx64, SLOT_BYTES as i64);
    let _t4 = i64v(fb, STATE_HEADER_BYTES as i64);
    let _t3 = fb.ins().iadd(off, _t4);

    let addr = fb.ins().iadd(base, _t3);
    let done = fb.create_block();
    fb.append_block_param(done, types::I64);
    fb.ins().jump(done, &[addr.into()]);
    fb.switch_to_block(pass);
    fb.ins().jump(done, &[got.into()]);
    fb.seal_block(done);
    fb.switch_to_block(done);
    fb.block_params(done)[0]
}

/// `idx = (handle & 0xFFFF) - 1` — the raw slot index of an encoded handle.
fn handle_idx(fb: &mut FunctionBuilder<'_>, h: Value) -> Value {
    let mask16 = i32v(fb, 0xFFFF);
    let low16 = fb.ins().band(h, mask16);
    let _t5 = i32v(fb, 1);

    fb.ins().isub(low16, _t5)
}

/// Dir staging area pointer for slot `idx` (520 bytes per slot).
fn dir_stage_addr(
    om: &mut ObjModule,
    fb: &mut FunctionBuilder<'_>,
    state: DataId,
    idx: Value,
) -> Value {
    let base = state_ptr(om, state, fb);
    let idx64 = fb.ins().uextend(types::I64, idx);
    let off = fb
        .ins()
        .imul_imm(idx64, crate::platform_io::DIR_STAGE_UNIT_BYTES as i64);
    let _t7 = i64v(fb, STATE_BYTES as i64);
    let _t6 = fb.ins().iadd(off, _t7);

    fb.ins().iadd(base, _t6)
}

fn stack_buf(fb: &mut FunctionBuilder<'_>, bytes: u32) -> Value {
    let ss = fb.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, bytes, 3));
    fb.ins().stack_addr(types::I64, ss, 0)
}

fn zeroed_stack_buf(fb: &mut FunctionBuilder<'_>, bytes: u32) -> Value {
    let buffer = stack_buf(fb, bytes);
    let zero = i64v(fb, 0);
    for offset in (0..bytes).step_by(8) {
        fb.ins()
            .store(MemFlags::trusted(), zero, buffer, offset as i32);
    }
    buffer
}

/// Loop: does any of `count` UTF-16 units at `base + 2*off_units` equal 0?
/// Returns 1 (found) / 0 (clean) as i32.
fn units_have_nul(
    fb: &mut FunctionBuilder<'_>,
    flags: MemFlags,
    base: Value,
    off_units: Value,
    count: Value,
) -> Value {
    let off64 = fb.ins().uextend(types::I64, off_units);
    let _t8 = fb.ins().imul_imm(off64, 2);

    let at = fb.ins().iadd(base, _t8);

    let lp = fb.create_block();
    fb.append_block_param(lp, types::I32);
    let _t9 = i32v(fb, 0);

    fb.ins().jump(lp, &[_t9.into()]);

    let clean = fb.create_block();
    let body = fb.create_block();
    fb.switch_to_block(lp);
    let i = fb.block_params(lp)[0];
    let over = fb.ins().icmp(IntCC::SignedGreaterThanOrEqual, i, count);
    fb.ins().brif(over, clean, &[], body, &[]);
    fb.seal_block(clean);
    fb.seal_block(body);

    fb.switch_to_block(body);
    let i64i = fb.ins().uextend(types::I64, i);
    let _t10 = fb.ins().imul_imm(i64i, 2);

    let addr = fb.ins().iadd(at, _t10);
    let u = fb.ins().uload16(types::I32, flags, addr, 0);
    let found = fb.create_block();
    let is0 = fb.ins().icmp_imm(IntCC::Equal, u, 0);
    let cont = fb.create_block();
    fb.ins().brif(is0, found, &[], cont, &[]);
    fb.seal_block(found);
    fb.seal_block(cont);
    fb.switch_to_block(cont);
    let _t12 = i32v(fb, 1);
    let _t11 = fb.ins().iadd(i, _t12);

    fb.ins().jump(lp, &[_t11.into()]);
    fb.seal_block(lp);

    let join = fb.create_block();
    fb.append_block_param(join, types::I32);
    fb.switch_to_block(clean);
    let _t13 = i32v(fb, 0);

    fb.ins().jump(join, &[_t13.into()]);
    fb.switch_to_block(found);
    let _t14 = i32v(fb, 1);

    fb.ins().jump(join, &[_t14.into()]);
    fb.seal_block(join);
    fb.switch_to_block(join);
    fb.block_params(join)[0]
}

/// Byte copy loop: `len` bytes from `src` to `dst`.
fn copy_bytes(fb: &mut FunctionBuilder<'_>, flags: MemFlags, dst: Value, src: Value, len: Value) {
    let lp = fb.create_block();
    fb.append_block_param(lp, types::I64);
    let _t15 = i64v(fb, 0);

    fb.ins().jump(lp, &[_t15.into()]);
    let done = fb.create_block();
    let body = fb.create_block();
    fb.switch_to_block(lp);
    let i = fb.block_params(lp)[0];
    let over = fb.ins().icmp(IntCC::UnsignedGreaterThanOrEqual, i, len);
    fb.ins().brif(over, done, &[], body, &[]);
    fb.seal_block(done);
    fb.seal_block(body);
    fb.switch_to_block(body);
    let _t16 = fb.ins().iadd(src, i);

    let b32 = fb.ins().uload8(types::I32, flags, _t16, 0);
    let b = fb.ins().ireduce(types::I8, b32);
    let _t17 = fb.ins().iadd(dst, i);

    fb.ins().store(flags, b, _t17, 0);
    let _t19 = i64v(fb, 1);
    let _t18 = fb.ins().iadd(i, _t19);

    fb.ins().jump(lp, &[_t18.into()]);
    fb.seal_block(lp);
    fb.switch_to_block(done);
}

/// Count UTF-16 units until NUL at `src` (bounded by `max`); returns i32.
fn cstr_len_units(fb: &mut FunctionBuilder<'_>, flags: MemFlags, src: Value, max: i32) -> Value {
    let lp = fb.create_block();
    fb.append_block_param(lp, types::I32);
    let _t20 = i32v(fb, 0);

    fb.ins().jump(lp, &[_t20.into()]);
    let done = fb.create_block();
    fb.append_block_param(done, types::I32);
    let body = fb.create_block();
    fb.switch_to_block(lp);
    let i = fb.block_params(lp)[0];
    let at_max = fb
        .ins()
        .icmp_imm(IntCC::SignedGreaterThanOrEqual, i, max as i64);
    fb.ins().brif(at_max, done, &[i.into()], body, &[]);
    fb.seal_block(body);
    fb.switch_to_block(body);
    let i64i = fb.ins().uextend(types::I64, i);
    let _t22 = fb.ins().imul_imm(i64i, 2);
    let _t21 = fb.ins().iadd(src, _t22);

    let u = fb.ins().uload16(types::I32, flags, _t21, 0);
    let is0 = fb.ins().icmp_imm(IntCC::Equal, u, 0);
    let _t24 = i32v(fb, 1);
    let _t23 = fb.ins().iadd(i, _t24);

    fb.ins().brif(is0, done, &[i.into()], lp, &[_t23.into()]);
    fb.seal_block(lp);
    fb.seal_block(done);
    fb.switch_to_block(done);
    fb.block_params(done)[0]
}

/// The AV `file_target` for READ ops: OsFile OR the stdin std-stream.
/// Every lookup error collapses to ERR_INVALID_HANDLE (the AV read-op
/// squash). Returns the OS handle in the ok block.
fn file_read_target(
    fb: &mut FunctionBuilder<'_>,
    find: FuncRef,
    h: Value,
    flags: MemFlags,
) -> Value {
    let kf = i32v(fb, KIND_FILE as i32);
    let c = fb.ins().call(find, &[h, kf]);
    let slot = fb.inst_results(c)[0];
    let neg = fb.ins().icmp_imm(IntCC::SignedLessThan, slot, 0);
    let not_file = fb.create_block();
    let ok_file = fb.create_block();
    fb.ins().brif(neg, not_file, &[], ok_file, &[]);
    fb.seal_block(not_file);
    fb.seal_block(ok_file);

    fb.switch_to_block(not_file);
    // Only WRONG_ORIGIN retries as a stream; invalid-handle / released
    // keep their own codes — the op-level squash maps everything to
    // ERR_INVALID_HANDLE anyway.
    let wrong = fb
        .ins()
        .icmp_imm(IntCC::Equal, slot, ERR_WRONG_ORIGIN as i64);
    let give_up = fb.create_block();
    let try_stream = fb.create_block();
    fb.ins().brif(wrong, try_stream, &[], give_up, &[]);
    fb.seal_block(give_up);
    fb.seal_block(try_stream);
    fb.switch_to_block(give_up);
    ret_code(fb, ERR_INVALID_HANDLE);
    fb.switch_to_block(try_stream);
    let ks = i32v(fb, KIND_STREAM as i32);
    let c2 = fb.ins().call(find, &[h, ks]);
    let sslot = fb.inst_results(c2)[0];
    let sneg = fb.ins().icmp_imm(IntCC::SignedLessThan, sslot, 0);
    let sbad = fb.create_block();
    let sok = fb.create_block();
    fb.ins().brif(sneg, sbad, &[], sok, &[]);
    fb.seal_block(sbad);
    fb.seal_block(sok);
    fb.switch_to_block(sbad);
    ret_code(fb, ERR_INVALID_HANDLE);
    fb.switch_to_block(sok);
    let sid = fb.ins().load(types::I32, flags, sslot, S_STREAM_ID);
    let is_in = fb.ins().icmp_imm(IntCC::Equal, sid, STD_INPUT_ID);
    let no = fb.create_block();
    let yes = fb.create_block();
    fb.ins().brif(is_in, yes, &[], no, &[]);
    fb.seal_block(no);
    fb.seal_block(yes);
    fb.switch_to_block(no);
    ret_code(fb, ERR_INVALID_HANDLE);
    fb.switch_to_block(yes);
    let s_h = fb.ins().load(types::I64, flags, sslot, S_OS_PTR);

    let done = fb.create_block();
    fb.append_block_param(done, types::I64);
    fb.ins().jump(done, &[s_h.into()]);
    fb.switch_to_block(ok_file);
    let f_h = fb.ins().load(types::I64, flags, slot, S_OS_PTR);
    fb.ins().jump(done, &[f_h.into()]);
    fb.seal_block(done);
    fb.switch_to_block(done);
    fb.block_params(done)[0]
}

/// The AV `file_target` for WRITE ops: OsFile only; every lookup error
/// collapses to ERR_WRONG_ORIGIN. Returns the OS handle in the ok block.
fn file_write_target(
    fb: &mut FunctionBuilder<'_>,
    find: FuncRef,
    h: Value,
    flags: MemFlags,
) -> Value {
    let kf = i32v(fb, KIND_FILE as i32);
    let c = fb.ins().call(find, &[h, kf]);
    let slot = fb.inst_results(c)[0];
    let neg = fb.ins().icmp_imm(IntCC::SignedLessThan, slot, 0);
    let (bad, ok) = branch2(fb, neg);
    fb.switch_to_block(bad);
    ret_code(fb, ERR_WRONG_ORIGIN);
    fb.switch_to_block(ok);
    fb.ins().load(types::I64, flags, slot, S_OS_PTR)
}

/// Region lookup whose EVERY lookup error collapses to `code` (the AV
/// read/write/rand/pipe squash to ERR_WRONG_ORIGIN). Bounds errors keep
/// ERR_INVALID_BOUNDS. Returns (base, off64, len64) in the ok block.
fn region_lookup_squashed(
    fb: &mut FunctionBuilder<'_>,
    find: FuncRef,
    region: Value,
    off: Value,
    len: Value,
    code: i32,
    flags: MemFlags,
) -> (Value, Value, Value) {
    let k = i32v(fb, KIND_MEM as i32);
    let c = fb.ins().call(find, &[region, k]);
    let slot = fb.inst_results(c)[0];
    let neg = fb.ins().icmp_imm(IntCC::SignedLessThan, slot, 0);
    let (bad, ok) = branch2(fb, neg);
    fb.switch_to_block(bad);
    ret_code(fb, code);
    fb.switch_to_block(ok);
    let (off64, len64, _) = region_bounds_after_slot(fb, slot, off, len, flags);
    let base = fb.ins().load(types::I64, flags, slot, S_OS_PTR);
    (base, off64, len64)
}

/// Shared bounds ladder once a slot pointer exists: negative off/len and
/// off+len > logical each return ERR_INVALID_BOUNDS.
fn region_bounds_after_slot(
    fb: &mut FunctionBuilder<'_>,
    slot: Value,
    off: Value,
    len: Value,
    flags: MemFlags,
) -> (Value, Value, Value) {
    let neg_off = fb.ins().icmp_imm(IntCC::SignedLessThan, off, 0);
    let (b1, k1) = branch2(fb, neg_off);
    fb.switch_to_block(b1);
    ret_code(fb, ERR_INVALID_BOUNDS);
    fb.switch_to_block(k1);
    let neg_len = fb.ins().icmp_imm(IntCC::SignedLessThan, len, 0);
    let (b2, k2) = branch2(fb, neg_len);
    fb.switch_to_block(b2);
    ret_code(fb, ERR_INVALID_BOUNDS);
    fb.switch_to_block(k2);
    let off64 = fb.ins().uextend(types::I64, off);
    let len64 = fb.ins().uextend(types::I64, len);
    let end64 = fb.ins().iadd(off64, len64);
    let logical = fb.ins().load(types::I64, flags, slot, S_LOGICAL_LEN);
    let over = fb.ins().icmp(IntCC::SignedGreaterThan, end64, logical);
    let (b3, k3) = branch2(fb, over);
    fb.switch_to_block(b3);
    ret_code(fb, ERR_INVALID_BOUNDS);
    fb.switch_to_block(k3);
    (off64, len64, slot)
}

// ---- the sys973 body set -------------------------------------------------

struct BodyCtx<'a> {
    ids: &'a Sys973Ids,
    loc: &'a LocalIds,
    state: DataId,
    flags: MemFlags,
    units: FuncId,
    wru: FuncId,
}

type Src<'a> = (&'a Sys973Ids, &'a LocalIds, DataId, FuncId, FuncId);

fn emit_body<F>(
    om: &mut ObjModule,
    kind: PlatformOpKind,
    conv: CallConv,
    src: Src<'_>,
    build: F,
) -> Result<(), MnError>
where
    F: FnOnce(&mut ObjModule, &mut FunctionBuilder<'_>, Block, &BodyCtx<'_>),
{
    let s = sys973_shim_sig(kind, conv);
    let id = declare_fn(om, shim_symbol_for(kind), Linkage::Export, &s)?;
    let ctx = BodyCtx {
        ids: src.0,
        loc: src.1,
        state: src.2,
        flags: MemFlags::trusted(),
        units: src.3,
        wru: src.4,
    };
    define_fn(om, id, s, |om, fb, entry| build(om, fb, entry, &ctx))
}

/// `mn_pio_local_units_from_region(dst, cap, region, uoff, ulen) -> len`
/// — the AV `units_from_region(reject_nul)` + caller cap: validates the
/// region slot (raw codes), bounds, NUL-rejects, copies units to `dst`
/// and stores a NUL terminator. Returns the unit count or a negative code.
fn emit_local_units_from_region(om: &mut ObjModule, loc: &LocalIds) -> Result<FuncId, MnError> {
    use types as t;
    let s = sig(&[t::I64, t::I32, t::I32, t::I32, t::I32], Some(t::I32));
    let id = declare_fn(om, "mn_pio_local_units_from_region", Linkage::Local, &s)?;
    let flags = MemFlags::trusted();
    define_fn(om, id, s, |om, fb, entry| {
        let dst = fb.block_params(entry)[0];
        let cap = fb.block_params(entry)[1];
        let region = fb.block_params(entry)[2];
        let uoff = fb.block_params(entry)[3];
        let ulen = fb.block_params(entry)[4];
        let find = fref(om, fb, loc.find_slot);
        let slot = slot_lookup(fb, find, region, KIND_MEM);

        let over_cap = fb.ins().icmp(IntCC::SignedGreaterThan, ulen, cap);
        let (b0, k0) = branch2(fb, over_cap);
        fb.switch_to_block(b0);
        ret_code(fb, ERR_INVALID_BOUNDS);
        fb.switch_to_block(k0);

        let (off64, len64, _) = region_bounds_after_slot(fb, slot, uoff, ulen, flags);
        let base = fb.ins().load(types::I64, flags, slot, S_OS_PTR);
        let _t25 = fb.ins().imul_imm(off64, 2);

        let src_at = fb.ins().iadd(base, _t25);
        let len_b = fb.ins().imul_imm(len64, 2);
        let has_nul = units_have_nul(fb, flags, base, uoff, ulen);
        let nul = fb.ins().icmp_imm(IntCC::Equal, has_nul, 1);
        let (b5, k5) = branch2(fb, nul);
        fb.switch_to_block(b5);
        ret_code(fb, ERR_INVALID_VALUE);
        fb.switch_to_block(k5);

        copy_bytes(fb, flags, dst, src_at, len_b);
        let _t26 = i32v(fb, 0);
        let _t27 = fb.ins().iadd(dst, len_b);

        fb.ins().store(flags, _t26, _t27, 0);
        fb.ins().return_(&[ulen]);
    })?;
    Ok(id)
}

/// `mn_pio_local_region_write_units(region, src, n) -> 0` — write `n`
/// UTF-16 units from `src` into the region at unit offset 0 (the AV
/// `write_units_to_region` at uoff 0): raw region codes, bounds -4.
fn emit_local_region_write_units(om: &mut ObjModule, loc: &LocalIds) -> Result<FuncId, MnError> {
    use types as t;
    let s = sig(&[t::I32, t::I64, t::I32], Some(t::I32));
    let id = declare_fn(om, "mn_pio_local_region_write_units", Linkage::Local, &s)?;
    let flags = MemFlags::trusted();
    define_fn(om, id, s, |om, fb, entry| {
        let region = fb.block_params(entry)[0];
        let src = fb.block_params(entry)[1];
        let n = fb.block_params(entry)[2];
        let find = fref(om, fb, loc.find_slot);
        let slot = slot_lookup(fb, find, region, KIND_MEM);
        let neg_n = fb.ins().icmp_imm(IntCC::SignedLessThan, n, 0);
        let (b0, k0) = branch2(fb, neg_n);
        fb.switch_to_block(b0);
        ret_code(fb, ERR_INVALID_BOUNDS);
        fb.switch_to_block(k0);
        let n64 = fb.ins().uextend(types::I64, n);
        let total = fb.ins().imul_imm(n64, 2);
        let logical = fb.ins().load(types::I64, flags, slot, S_LOGICAL_LEN);
        let over = fb.ins().icmp(IntCC::SignedGreaterThan, total, logical);
        let (b1, k1) = branch2(fb, over);
        fb.switch_to_block(b1);
        ret_code(fb, ERR_INVALID_BOUNDS);
        fb.switch_to_block(k1);
        let base = fb.ins().load(types::I64, flags, slot, S_OS_PTR);
        copy_bytes(fb, flags, base, src, total);
        let _t28 = i32v(fb, 0);

        fb.ins().return_(&[_t28]);
    })?;
    Ok(id)
}

/// Entry point: emit every Windows-applicable sys973 real body (36; the
/// 14 bytes-form N kinds stay named-unsupported stubs at the caller).
pub(crate) fn emit_sys973_win_bodies(
    om: &mut ObjModule,
    state: DataId,
    loc: &LocalIds,
    conv: CallConv,
) -> Result<(), MnError> {
    debug_assert_eq!(SYS973_WINDOWS_N_KINDS.len(), 14);
    let ids = declare_sys973_imports(om)?;
    let units = emit_local_units_from_region(om, loc)?;
    let wru = emit_local_region_write_units(om, loc)?;
    let src = (&ids, loc, state, units, wru);

    // ---- F-IO -----------------------------------------------------------
    emit_file_open_w16(om, src, conv)?;
    emit_file_read_some(om, src, conv)?;
    emit_file_write_some(om, src, conv)?;
    emit_file_seek(om, src, conv)?;
    emit_file_tell(om, src, conv)?;
    emit_file_hi(om, src, conv)?;
    emit_file_size(om, src, conv)?;
    emit_file_mtime_ms(om, src, conv)?;
    emit_file_flush(om, src, conv)?;
    emit_file_close(om, src, conv)?;

    // ---- F-DIRPATH ------------------------------------------------------
    emit_path_w16(om, src, conv, PathOp::Attrs)?;
    emit_path_w16(om, src, conv, PathOp::Delete)?;
    emit_path_w16(om, src, conv, PathOp::DirCreate)?;
    emit_path_w16(om, src, conv, PathOp::DirDelete)?;
    emit_file_replace_w16(om, src, conv)?;
    emit_canonical_w16(om, src, conv)?;

    // ---- F-DIR ----------------------------------------------------------
    emit_dir_open_w16(om, src, conv)?;
    emit_dir_next_w16(om, src, conv)?;
    emit_dir_close(om, src, conv)?;

    // ---- F-ENV ----------------------------------------------------------
    emit_env_get_w16(om, src, conv)?;
    emit_env_set_w16(om, src, conv)?;
    emit_env_remove_w16(om, src, conv)?;

    // ---- F-TIME / F-ENTROPY ---------------------------------------------
    emit_clock_unix_ms(om, src, conv)?;
    emit_clock_mono_ms(om, src, conv)?;
    emit_clock_hi(om, src, conv)?;
    emit_sleep_ms(om, src, conv)?;
    emit_rand_fill(om, src, conv)?;

    // ---- F-PROC ---------------------------------------------------------
    emit_proc_spawn_w16(om, src, conv)?;
    emit_proc_pipe_read(om, src, conv)?;
    emit_proc_wait(om, src, conv)?;
    emit_proc_release(om, src, conv)?;
    emit_proc_exit(om, src, conv)?;
    emit_proc_cmdline_w16(om, src, conv)?;
    emit_proc_cwd_w16(om, src, conv)?;
    emit_proc_chdir_w16(om, src, conv)?;
    Ok(())
}

// ---- F-IO bodies ---------------------------------------------------------

fn emit_file_open_w16(om: &mut ObjModule, src: Src<'_>, conv: CallConv) -> Result<(), MnError> {
    emit_body(
        om,
        PlatformOpKind::FileOpenW16,
        conv,
        src,
        |om, fb, entry, ctx| {
            let flags = ctx.flags;
            let region = fb.block_params(entry)[0];
            let uoff = fb.block_params(entry)[1];
            let ulen = fb.block_params(entry)[2];
            let open_flags = fb.block_params(entry)[3];
            let gle = fref(om, fb, ctx.ids.get_last_error);

            // units -> stack path buffer (raw error codes, AV file_open).
            let dst = stack_buf(fb, PATH_BUF_BYTES);
            let units = fref(om, fb, ctx.units);
            let cap = i32v(fb, PATH_UNITS_MAX);
            let uc = fb.ins().call(units, &[dst, cap, region, uoff, ulen]);
            let n = fb.inst_results(uc)[0];
            let neg_n = fb.ins().icmp_imm(IntCC::SignedLessThan, n, 0);
            let (bad, ok_n) = branch2(fb, neg_n);
            fb.switch_to_block(bad);
            fb.ins().return_(&[n]);
            fb.switch_to_block(ok_n);
            let empty = fb.ins().icmp_imm(IntCC::Equal, n, 0);
            let (b0, k0) = branch2(fb, empty);
            fb.switch_to_block(b0);
            ret_code(fb, ERR_INVALID_BOUNDS);
            fb.switch_to_block(k0);

            // flags -> access/disposition (AV open_flags_to_native).
            let _t29 = i32v(fb, 1);
            let read = fb.ins().band(open_flags, _t29);
            let _t30 = i32v(fb, 2);

            let write = fb.ins().band(open_flags, _t30);
            let _t31 = i32v(fb, 4);

            let create = fb.ins().band(open_flags, _t31);
            let _t32 = i32v(fb, 8);

            let trunc = fb.ins().band(open_flags, _t32);
            let _t33 = i32v(fb, 16);

            let excl = fb.ins().band(open_flags, _t33);
            let _t34 = i32v(fb, 32);

            let append = fb.ins().band(open_flags, _t34);
            let nz =
                |fb: &mut FunctionBuilder<'_>, v: Value| fb.ins().icmp_imm(IntCC::NotEqual, v, 0);
            let r_ok = nz(fb, read);
            let w_ok = nz(fb, write);
            let a_ok = nz(fb, append);
            let c_ok = nz(fb, create);
            let t_ok = nz(fb, trunc);
            let e_ok = nz(fb, excl);
            let _t35 = fb.ins().bor(r_ok, w_ok);

            let any_access = fb.ins().bor(_t35, a_ok);
            let no_acc = fb.ins().icmp_imm(IntCC::Equal, any_access, 0);
            let (bacc, kacc) = branch2(fb, no_acc);
            fb.switch_to_block(bacc);
            os_fail(
                om,
                fb,
                gle,
                ctx.state,
                flags,
                ERROR_INVALID_PARAMETER as i32,
            );
            fb.switch_to_block(kacc);

            let z = i32v(fb, 0);
            let _t36 = i32v(fb, GENERIC_READ as i32);

            let access_r = fb.ins().select(r_ok, _t36, z);
            let w_or_a = fb.ins().bor(w_ok, a_ok);
            let _t37 = i32v(fb, (GENERIC_READ | GENERIC_WRITE) as i32);

            let access_rw = fb.ins().select(w_or_a, _t37, access_r);
            // APPEND: (access & GENERIC_READ) | FILE_APPEND_DATA.
            let _t38 = i32v(fb, GENERIC_READ as i32);
            let a_masked = fb.ins().band(access_rw, _t38);
            let _t39 = i32v(fb, FILE_APPEND_DATA as i32);

            let a_final = fb.ins().bor(a_masked, _t39);
            let access = fb.ins().select(a_ok, a_final, access_rw);

            let dispo3 = i32v(fb, 3); // OPEN_EXISTING
            let _t40 = i32v(fb, 5);
            let dispo5 = fb.ins().select(t_ok, _t40, dispo3);
            let _t41 = i32v(fb, 4);

            let dispo4 = fb.ins().select(c_ok, _t41, dispo5);
            let _t42 = fb.ins().band(c_ok, t_ok);
            let _t43 = i32v(fb, 2);

            let dispo2 = fb.ins().select(_t42, _t43, dispo4);
            let _t44 = fb.ins().band(c_ok, e_ok);
            let _t45 = i32v(fb, 1);

            let disposition = fb.ins().select(_t44, _t45, dispo2);

            let cf = fref(om, fb, ctx.ids.create_file_w);
            let _t46 = i32v(fb, FILE_SHARE_ALL as i32);
            let _t47 = i64v(fb, 0);
            let _t48 = i32v(fb, FILE_ATTRIBUTE_NORMAL as i32);
            let _t49 = i64v(fb, 0);

            let c = fb
                .ins()
                .call(cf, &[dst, access, _t46, _t47, disposition, _t48, _t49]);
            let h = fb.inst_results(c)[0];
            let bad_h = fb.ins().icmp_imm(IntCC::Equal, h, INVALID_HANDLE_VALUE);
            let (bh, kh) = branch2(fb, bad_h);
            fb.switch_to_block(bh);
            os_fail(om, fb, gle, ctx.state, flags, 5);
            fb.switch_to_block(kh);
            let alloc = fref(om, fb, ctx.loc.alloc_slot);
            let _t50 = i32v(fb, KIND_FILE as i32);
            let _t51 = i64v(fb, 0);
            let _t52 = i64v(fb, 0);

            let ac = fb.ins().call(alloc, &[_t50, h, _t51, _t52, z]);
            let hd = fb.inst_results(ac)[0];
            let no_slot = fb.ins().icmp_imm(IntCC::SignedLessThan, hd, 0);
            let (rollback, committed) = branch2(fb, no_slot);
            fb.switch_to_block(rollback);
            let close = fref(om, fb, ctx.ids.close_handle);
            fb.ins().call(close, &[h]);
            fb.ins().return_(&[hd]);
            fb.switch_to_block(committed);
            let base = state_ptr(om, ctx.state, fb);
            fb.ins().store(flags, z, base, H_LAST_OS_ERROR);
            fb.ins().return_(&[hd]);
        },
    )
}

/// Read-run budget + EOF-as-broken-pipe ladder shared by file/pipe reads.
fn emit_read_call(
    om: &mut ObjModule,
    fb: &mut FunctionBuilder<'_>,
    ctx: &BodyCtx<'_>,
    gle: FuncRef,
    os_h: Value,
    buf: Value,
    len: Value,
) -> Value {
    let flags = ctx.flags;
    let st = state_ptr(om, ctx.state, fb);
    let _t53 = i64v(fb, H_BYTES_WRITTEN_TMP as i64);

    let outp = fb.ins().iadd(st, _t53);
    let _t54 = fref(om, fb, ctx.ids.read_file);
    let _t55 = i64v(fb, 0);

    let c = fb.ins().call(_t54, &[os_h, buf, len, outp, _t55]);
    let ok = fb.inst_results(c)[0];
    let failed = fb.ins().icmp_imm(IntCC::Equal, ok, 0);
    let (bf, kf) = branch2(fb, failed);
    fb.switch_to_block(bf);
    let gc = fb.ins().call(gle, &[]);
    let ecode = fb.inst_results(gc)[0];
    let _t56 = fb
        .ins()
        .icmp_imm(IntCC::Equal, ecode, ERROR_BROKEN_PIPE as i64);
    let _t57 = fb.ins().icmp_imm(IntCC::Equal, ecode, ERROR_NO_DATA as i64);

    let is_pipe_eof = fb.ins().bor(_t56, _t57);
    let (bpe, bose) = branch2(fb, is_pipe_eof);
    fb.switch_to_block(bpe);
    let _t58 = i32v(fb, 0);

    fb.ins().return_(&[_t58]);
    fb.switch_to_block(bose);
    let ez = fb.ins().icmp_imm(IntCC::Equal, ecode, 0);
    let _t59 = i32v(fb, 6);

    let rec = fb.ins().select(ez, _t59, ecode);
    fb.ins().store(flags, rec, st, H_LAST_OS_ERROR);
    ret_code(fb, ERR_OS_FAILED);
    fb.switch_to_block(kf);
    let n = fb.ins().load(types::I32, flags, st, H_BYTES_WRITTEN_TMP);
    let gt = fb.ins().icmp(IntCC::UnsignedGreaterThan, n, len);
    let clamped = fb.ins().select(gt, len, n);
    let rt2 = fb.ins().load(types::I64, flags, st, H_READ_TOTAL);
    let _t61 = fb.ins().uextend(types::I64, clamped);
    let _t60 = fb.ins().iadd(rt2, _t61);

    fb.ins().store(flags, _t60, st, H_READ_TOTAL);
    let _t62 = i32v(fb, 0);

    fb.ins().store(flags, _t62, st, H_LAST_OS_ERROR);
    clamped
}

fn emit_file_read_some(om: &mut ObjModule, src: Src<'_>, conv: CallConv) -> Result<(), MnError> {
    emit_body(
        om,
        PlatformOpKind::FileReadSome,
        conv,
        src,
        |om, fb, entry, ctx| {
            let flags = ctx.flags;
            let h = fb.block_params(entry)[0];
            let region = fb.block_params(entry)[1];
            let off = fb.block_params(entry)[2];
            let len = fb.block_params(entry)[3];
            let gle = fref(om, fb, ctx.ids.get_last_error);
            let find = fref(om, fb, ctx.loc.find_slot);

            let over_call = fb.ins().icmp_imm(
                IntCC::SignedGreaterThan,
                len,
                MAX_PLATFORM_CALL_BYTES as i64,
            );
            let (bb, kb) = branch2(fb, over_call);
            fb.switch_to_block(bb);
            ret_code(fb, ERR_BUDGET);
            fb.switch_to_block(kb);

            let os_h = file_read_target(fb, find, h, flags);
            let (base, off64, len64) =
                region_lookup_squashed(fb, find, region, off, len, ERR_WRONG_ORIGIN, flags);
            let empty = fb.ins().icmp_imm(IntCC::Equal, len, 0);
            let (b0, k0) = branch2(fb, empty);
            fb.switch_to_block(b0);
            let _t63 = i32v(fb, 0);

            fb.ins().return_(&[_t63]);
            fb.switch_to_block(k0);

            let st = state_ptr(om, ctx.state, fb);
            let rt = fb.ins().load(types::I64, flags, st, H_READ_TOTAL);
            let _t64 = fb.ins().iadd(rt, len64);

            let over_run = fb.ins().icmp_imm(
                IntCC::SignedGreaterThan,
                _t64,
                MAX_PLATFORM_RUN_BYTES as i64,
            );
            let (b1, k1) = branch2(fb, over_run);
            fb.switch_to_block(b1);
            ret_code(fb, ERR_BUDGET);
            fb.switch_to_block(k1);

            let buf = fb.ins().iadd(base, off64);
            let n = emit_read_call(om, fb, ctx, gle, os_h, buf, len);
            fb.ins().return_(&[n]);
        },
    )
}

fn emit_file_write_some(om: &mut ObjModule, src: Src<'_>, conv: CallConv) -> Result<(), MnError> {
    emit_body(
        om,
        PlatformOpKind::FileWriteSome,
        conv,
        src,
        |om, fb, entry, ctx| {
            let flags = ctx.flags;
            let h = fb.block_params(entry)[0];
            let region = fb.block_params(entry)[1];
            let off = fb.block_params(entry)[2];
            let len = fb.block_params(entry)[3];
            let gle = fref(om, fb, ctx.ids.get_last_error);
            let find = fref(om, fb, ctx.loc.find_slot);

            let over_call = fb.ins().icmp_imm(
                IntCC::SignedGreaterThan,
                len,
                MAX_PLATFORM_CALL_BYTES as i64,
            );
            let (bb, kb) = branch2(fb, over_call);
            fb.switch_to_block(bb);
            ret_code(fb, ERR_BUDGET);
            fb.switch_to_block(kb);

            let os_h = file_write_target(fb, find, h, flags);
            let (base, off64, len64) =
                region_lookup_squashed(fb, find, region, off, len, ERR_WRONG_ORIGIN, flags);
            let empty = fb.ins().icmp_imm(IntCC::Equal, len, 0);
            let (b0, k0) = branch2(fb, empty);
            fb.switch_to_block(b0);
            let _t65 = i32v(fb, 0);

            fb.ins().return_(&[_t65]);
            fb.switch_to_block(k0);

            let st = state_ptr(om, ctx.state, fb);
            let wt = fb.ins().load(types::I64, flags, st, H_WRITTEN_TOTAL);
            let _t66 = fb.ins().iadd(wt, len64);

            let over_run = fb.ins().icmp_imm(
                IntCC::SignedGreaterThan,
                _t66,
                MAX_PLATFORM_RUN_BYTES as i64,
            );
            let (b1, k1) = branch2(fb, over_run);
            fb.switch_to_block(b1);
            ret_code(fb, ERR_BUDGET);
            fb.switch_to_block(k1);

            let buf = fb.ins().iadd(base, off64);
            let _t67 = i64v(fb, H_BYTES_WRITTEN_TMP as i64);

            let outp = fb.ins().iadd(st, _t67);
            let _t68 = fref(om, fb, ctx.ids.write_file);
            let _t69 = i64v(fb, 0);

            let c = fb.ins().call(_t68, &[os_h, buf, len, outp, _t69]);
            let ok = fb.inst_results(c)[0];
            let failed = fb.ins().icmp_imm(IntCC::Equal, ok, 0);
            let (bf, kf) = branch2(fb, failed);
            fb.switch_to_block(bf);
            os_fail(om, fb, gle, ctx.state, flags, 5);
            fb.switch_to_block(kf);
            let n = fb.ins().load(types::I32, flags, st, H_BYTES_WRITTEN_TMP);
            let gt = fb.ins().icmp(IntCC::UnsignedGreaterThan, n, len);
            let clamped = fb.ins().select(gt, len, n);
            let wt2 = fb.ins().load(types::I64, flags, st, H_WRITTEN_TOTAL);
            let _t71 = fb.ins().uextend(types::I64, clamped);
            let _t70 = fb.ins().iadd(wt2, _t71);

            fb.ins().store(flags, _t70, st, H_WRITTEN_TOTAL);
            let _t72 = i32v(fb, 0);

            fb.ins().store(flags, _t72, st, H_LAST_OS_ERROR);
            fb.ins().return_(&[clamped]);
        },
    )
}

/// Shared hi/lo record for a 64-bit result on THIS file handle; returns lo.
fn store_file_hi_lo(
    om: &mut ObjModule,
    fb: &mut FunctionBuilder<'_>,
    ctx: &BodyCtx<'_>,
    find: FuncRef,
    h: Value,
    v64: Value,
) -> Value {
    let slot = slot_lookup_file_norel(om, fb, find, h, ctx.state);
    let _t74 = fb.ins().ushr_imm(v64, 32);
    let _t73 = fb.ins().ireduce(types::I32, _t74);

    fb.ins().store(ctx.flags, _t73, slot, S_LAST_HI);
    let st = state_ptr(om, ctx.state, fb);
    let _t75 = i32v(fb, 0);

    fb.ins().store(ctx.flags, _t75, st, H_LAST_OS_ERROR);
    fb.ins().ireduce(types::I32, v64)
}

fn emit_file_seek(om: &mut ObjModule, src: Src<'_>, conv: CallConv) -> Result<(), MnError> {
    emit_body(
        om,
        PlatformOpKind::FileSeek,
        conv,
        src,
        |om, fb, entry, ctx| {
            let flags = ctx.flags;
            let h = fb.block_params(entry)[0];
            let origin = fb.block_params(entry)[1];
            let lo = fb.block_params(entry)[2];
            let hi = fb.block_params(entry)[3];
            let gle = fref(om, fb, ctx.ids.get_last_error);
            let find = fref(om, fb, ctx.loc.find_slot);

            let o_lo = fb.ins().icmp_imm(IntCC::SignedLessThan, origin, 0);
            let o_hi = fb.ins().icmp_imm(IntCC::SignedGreaterThan, origin, 2);
            let bad_o = fb.ins().bor(o_lo, o_hi);
            let (b0, k0) = branch2(fb, bad_o);
            fb.switch_to_block(b0);
            ret_code(fb, ERR_INVALID_VALUE);
            fb.switch_to_block(k0);

            let os_h = file_write_target(fb, find, h, flags);
            let _t78 = fb.ins().sextend(types::I64, hi);
            let _t76 = fb.ins().ishl_imm(_t78, 32);
            let _t77 = fb.ins().uextend(types::I64, lo);

            let offset = fb.ins().bor(_t76, _t77);
            let out = stack_buf(fb, 16);
            let _t79 = fref(om, fb, ctx.ids.set_file_pointer_ex);

            let c = fb.ins().call(_t79, &[os_h, offset, out, origin]);
            let ok = fb.inst_results(c)[0];
            let failed = fb.ins().icmp_imm(IntCC::Equal, ok, 0);
            let (bf, kf) = branch2(fb, failed);
            fb.switch_to_block(bf);
            os_fail(om, fb, gle, ctx.state, flags, 6);
            fb.switch_to_block(kf);
            let pos = fb.ins().load(types::I64, flags, out, 0);
            let lo32 = store_file_hi_lo(om, fb, ctx, find, h, pos);
            fb.ins().return_(&[lo32]);
        },
    )
}

fn emit_file_tell(om: &mut ObjModule, src: Src<'_>, conv: CallConv) -> Result<(), MnError> {
    emit_body(
        om,
        PlatformOpKind::FileTell,
        conv,
        src,
        |om, fb, entry, ctx| {
            let flags = ctx.flags;
            let h = fb.block_params(entry)[0];
            let gle = fref(om, fb, ctx.ids.get_last_error);
            let find = fref(om, fb, ctx.loc.find_slot);
            let os_h = file_write_target(fb, find, h, flags);
            let out = stack_buf(fb, 16);
            let _t80 = fref(om, fb, ctx.ids.set_file_pointer_ex);
            let _t81 = i64v(fb, 0);
            let _t82 = i32v(fb, 1);

            let c = fb.ins().call(_t80, &[os_h, _t81, out, _t82]);
            let ok = fb.inst_results(c)[0];
            let failed = fb.ins().icmp_imm(IntCC::Equal, ok, 0);
            let (bf, kf) = branch2(fb, failed);
            fb.switch_to_block(bf);
            os_fail(om, fb, gle, ctx.state, flags, 6);
            fb.switch_to_block(kf);
            let pos = fb.ins().load(types::I64, flags, out, 0);
            let lo32 = store_file_hi_lo(om, fb, ctx, find, h, pos);
            fb.ins().return_(&[lo32]);
        },
    )
}

fn emit_file_hi(om: &mut ObjModule, src: Src<'_>, conv: CallConv) -> Result<(), MnError> {
    emit_body(
        om,
        PlatformOpKind::FileHi,
        conv,
        src,
        |om, fb, entry, ctx| {
            let h = fb.block_params(entry)[0];
            let find = fref(om, fb, ctx.loc.find_slot);
            let slot = slot_lookup_file_norel(om, fb, find, h, ctx.state);
            let v = fb.ins().load(types::I32, ctx.flags, slot, S_LAST_HI);
            fb.ins().return_(&[v]);
        },
    )
}

fn emit_file_size(om: &mut ObjModule, src: Src<'_>, conv: CallConv) -> Result<(), MnError> {
    emit_body(
        om,
        PlatformOpKind::FileSize,
        conv,
        src,
        |om, fb, entry, ctx| {
            let flags = ctx.flags;
            let h = fb.block_params(entry)[0];
            let gle = fref(om, fb, ctx.ids.get_last_error);
            let find = fref(om, fb, ctx.loc.find_slot);
            let os_h = file_write_target(fb, find, h, flags);
            let out = stack_buf(fb, 16);
            let _t83 = fref(om, fb, ctx.ids.get_file_size_ex);

            let c = fb.ins().call(_t83, &[os_h, out]);
            let ok = fb.inst_results(c)[0];
            let failed = fb.ins().icmp_imm(IntCC::Equal, ok, 0);
            let (bf, kf) = branch2(fb, failed);
            fb.switch_to_block(bf);
            os_fail(om, fb, gle, ctx.state, flags, 6);
            fb.switch_to_block(kf);
            let v = fb.ins().load(types::I64, flags, out, 0);
            let lo32 = store_file_hi_lo(om, fb, ctx, find, h, v);
            fb.ins().return_(&[lo32]);
        },
    )
}

fn emit_file_mtime_ms(om: &mut ObjModule, src: Src<'_>, conv: CallConv) -> Result<(), MnError> {
    emit_body(
        om,
        PlatformOpKind::FileMtimeMs,
        conv,
        src,
        |om, fb, entry, ctx| {
            let flags = ctx.flags;
            let h = fb.block_params(entry)[0];
            let gle = fref(om, fb, ctx.ids.get_last_error);
            let find = fref(om, fb, ctx.loc.find_slot);
            let os_h = file_write_target(fb, find, h, flags);
            let out = stack_buf(fb, BHFI_BYTES);
            let _t84 = fref(om, fb, ctx.ids.get_file_information_by_handle);

            let c = fb.ins().call(_t84, &[os_h, out]);
            let ok = fb.inst_results(c)[0];
            let failed = fb.ins().icmp_imm(IntCC::Equal, ok, 0);
            let (bf, kf) = branch2(fb, failed);
            fb.switch_to_block(bf);
            os_fail(om, fb, gle, ctx.state, flags, 6);
            fb.switch_to_block(kf);
            // ftLastWriteTime (dwLow@+0, dwHigh@+4) -> unix ms.
            let _t85 = fb.ins().uload32(flags, out, BHFI_FTWRITE);
            let low = _t85;
            let _t86 = fb.ins().uload32(flags, out, BHFI_FTWRITE + 4);
            let high = _t86;
            let _t87 = fb.ins().ishl_imm(high, 32);

            let raw = fb.ins().bor(_t87, low);
            let _t88 = i64v(fb, FILETIME_UNIX_EPOCH_100NS);

            let shifted = fb.ins().isub(raw, _t88);
            let _t89 = i64v(fb, 10_000);

            let ms = fb.ins().sdiv(shifted, _t89);
            let lo32 = store_file_hi_lo(om, fb, ctx, find, h, ms);
            fb.ins().return_(&[lo32]);
        },
    )
}

fn emit_file_flush(om: &mut ObjModule, src: Src<'_>, conv: CallConv) -> Result<(), MnError> {
    emit_file_flush_close(om, src, conv, false)
}

fn emit_file_close(om: &mut ObjModule, src: Src<'_>, conv: CallConv) -> Result<(), MnError> {
    emit_file_flush_close(om, src, conv, true)
}

fn emit_file_flush_close(
    om: &mut ObjModule,
    src: Src<'_>,
    conv: CallConv,
    close: bool,
) -> Result<(), MnError> {
    let kind = if close {
        PlatformOpKind::FileClose
    } else {
        PlatformOpKind::FileFlush
    };
    emit_body(om, kind, conv, src, |om, fb, entry, ctx| {
        let flags = ctx.flags;
        let h = fb.block_params(entry)[0];
        let gle = fref(om, fb, ctx.ids.get_last_error);
        let find = fref(om, fb, ctx.loc.find_slot);
        let os_h = file_write_target(fb, find, h, flags);
        let imp = if close {
            fref(om, fb, ctx.ids.close_handle)
        } else {
            fref(om, fb, ctx.ids.flush_file_buffers)
        };
        let c = fb.ins().call(imp, &[os_h]);
        let ok = fb.inst_results(c)[0];
        let failed = fb.ins().icmp_imm(IntCC::Equal, ok, 0);
        let (bf, kf) = branch2(fb, failed);
        fb.switch_to_block(bf);
        os_fail(om, fb, gle, ctx.state, flags, 6);
        fb.switch_to_block(kf);
        if close {
            let slot = slot_lookup_file_norel(om, fb, find, h, ctx.state);
            let _t90 = i32v(fb, 1);

            fb.ins().store(flags, _t90, slot, S_RELEASED);
        }
        let st = state_ptr(om, ctx.state, fb);
        let _t91 = i32v(fb, 0);

        fb.ins().store(flags, _t91, st, H_LAST_OS_ERROR);
        let _t92 = i32v(fb, 0);

        fb.ins().return_(&[_t92]);
    })
}

// ---- F-DIRPATH bodies ----------------------------------------------------

#[derive(Clone, Copy)]
enum PathOp {
    Attrs,
    Delete,
    DirCreate,
    DirDelete,
}

fn emit_path_w16(
    om: &mut ObjModule,
    src: Src<'_>,
    conv: CallConv,
    op: PathOp,
) -> Result<(), MnError> {
    let kind = match op {
        PathOp::Attrs => PlatformOpKind::PathAttrsW16,
        PathOp::Delete => PlatformOpKind::FileDeleteW16,
        PathOp::DirCreate => PlatformOpKind::DirCreateW16,
        PathOp::DirDelete => PlatformOpKind::DirDeleteW16,
    };
    emit_body(om, kind, conv, src, |om, fb, entry, ctx| {
        let flags = ctx.flags;
        let region = fb.block_params(entry)[0];
        let uoff = fb.block_params(entry)[1];
        let ulen = fb.block_params(entry)[2];
        let gle = fref(om, fb, ctx.ids.get_last_error);
        // AV path ops squash every units error to ERR_INVALID_BOUNDS.
        let dst = stack_buf(fb, PATH_BUF_BYTES);
        let units = fref(om, fb, ctx.units);
        let cap = i32v(fb, PATH_UNITS_MAX);
        let uc = fb.ins().call(units, &[dst, cap, region, uoff, ulen]);
        let n = fb.inst_results(uc)[0];
        let bad = fb.ins().icmp_imm(IntCC::SignedLessThanOrEqual, n, 0);
        let (b0, k0) = branch2(fb, bad);
        fb.switch_to_block(b0);
        ret_code(fb, ERR_INVALID_BOUNDS);
        fb.switch_to_block(k0);

        let st = state_ptr(om, ctx.state, fb);
        match op {
            PathOp::Attrs => {
                let _t93 = fref(om, fb, ctx.ids.get_file_attributes_w);

                let c = fb.ins().call(_t93, &[dst]);
                let attrs = fb.inst_results(c)[0];
                // INVALID_FILE_ATTRIBUTES == 0xFFFFFFFF == -1 as i32.
                let bad = fb.ins().icmp_imm(IntCC::Equal, attrs, -1);
                let (ba, ka) = branch2(fb, bad);
                fb.switch_to_block(ba);
                os_fail(om, fb, gle, ctx.state, flags, 5);
                fb.switch_to_block(ka);
                let _t94 = i32v(fb, FILE_ATTRIBUTE_DIRECTORY as i32);

                let is_dir = fb.ins().band(attrs, _t94);
                let _t95 = i32v(fb, FILE_ATTRIBUTE_READONLY as i32);

                let is_ro = fb.ins().band(attrs, _t95);
                let _t96 = fb.ins().icmp_imm(IntCC::NotEqual, is_dir, 0);
                let _t97 = i32v(fb, 1);
                let _t98 = i32v(fb, 0);

                let bit_d = fb.ins().select(_t96, _t97, _t98);
                let _t99 = fb.ins().icmp_imm(IntCC::NotEqual, is_ro, 0);
                let _t100 = i32v(fb, 2);
                let _t101 = i32v(fb, 0);

                let bit_r = fb.ins().select(_t99, _t100, _t101);
                let _t102 = i32v(fb, 0);

                fb.ins().store(flags, _t102, st, H_LAST_OS_ERROR);
                let _t103 = fb.ins().bor(bit_d, bit_r);

                fb.ins().return_(&[_t103]);
            }
            PathOp::Delete | PathOp::DirCreate | PathOp::DirDelete => {
                let imp = match op {
                    PathOp::Delete => fref(om, fb, ctx.ids.delete_file_w),
                    PathOp::DirCreate => fref(om, fb, ctx.ids.create_directory_w),
                    PathOp::DirDelete => fref(om, fb, ctx.ids.remove_directory_w),
                    PathOp::Attrs => unreachable!(),
                };
                let args: &[Value] = match op {
                    PathOp::DirCreate => &[dst, i64v(fb, 0)],
                    _ => &[dst],
                };
                let c = fb.ins().call(imp, args);
                let ok = fb.inst_results(c)[0];
                let failed = fb.ins().icmp_imm(IntCC::Equal, ok, 0);
                let (bf, kf) = branch2(fb, failed);
                fb.switch_to_block(bf);
                os_fail(om, fb, gle, ctx.state, flags, 5);
                fb.switch_to_block(kf);
                let _t104 = i32v(fb, 0);

                fb.ins().store(flags, _t104, st, H_LAST_OS_ERROR);
                let _t105 = i32v(fb, 0);

                fb.ins().return_(&[_t105]);
            }
        }
    })
}

fn emit_file_replace_w16(om: &mut ObjModule, src: Src<'_>, conv: CallConv) -> Result<(), MnError> {
    emit_body(
        om,
        PlatformOpKind::FileReplaceW16,
        conv,
        src,
        |om, fb, entry, ctx| {
            let flags = ctx.flags;
            let s_region = fb.block_params(entry)[0];
            let s_off = fb.block_params(entry)[1];
            let s_len = fb.block_params(entry)[2];
            let d_region = fb.block_params(entry)[3];
            let d_off = fb.block_params(entry)[4];
            let d_len = fb.block_params(entry)[5];
            let gle = fref(om, fb, ctx.ids.get_last_error);
            let sbuf = stack_buf(fb, PATH_BUF_BYTES);
            let dbuf = stack_buf(fb, PATH_BUF_BYTES);
            let units = fref(om, fb, ctx.units);
            let cap = i32v(fb, PATH_UNITS_MAX);

            let uc = fb.ins().call(units, &[sbuf, cap, s_region, s_off, s_len]);
            let sn = fb.inst_results(uc)[0];
            let sbad = fb.ins().icmp_imm(IntCC::SignedLessThanOrEqual, sn, 0);
            let (b1, k1) = branch2(fb, sbad);
            fb.switch_to_block(b1);
            ret_code(fb, ERR_INVALID_BOUNDS);
            fb.switch_to_block(k1);
            let uc2 = fb.ins().call(units, &[dbuf, cap, d_region, d_off, d_len]);
            let dn = fb.inst_results(uc2)[0];
            let dbad = fb.ins().icmp_imm(IntCC::SignedLessThanOrEqual, dn, 0);
            let (b2, k2) = branch2(fb, dbad);
            fb.switch_to_block(b2);
            ret_code(fb, ERR_INVALID_BOUNDS);
            fb.switch_to_block(k2);
            let _t106 = fref(om, fb, ctx.ids.move_file_ex_w);
            let _t107 = i32v(fb, MOVEFILE_REPLACE_EXISTING as i32);

            let c = fb.ins().call(_t106, &[sbuf, dbuf, _t107]);
            let ok = fb.inst_results(c)[0];
            let failed = fb.ins().icmp_imm(IntCC::Equal, ok, 0);
            let (bf, kf) = branch2(fb, failed);
            fb.switch_to_block(bf);
            os_fail(om, fb, gle, ctx.state, flags, 5);
            fb.switch_to_block(kf);
            let st = state_ptr(om, ctx.state, fb);
            let _t108 = i32v(fb, 0);

            fb.ins().store(flags, _t108, st, H_LAST_OS_ERROR);
            let _t109 = i32v(fb, 0);

            fb.ins().return_(&[_t109]);
        },
    )
}

fn emit_canonical_w16(om: &mut ObjModule, src: Src<'_>, conv: CallConv) -> Result<(), MnError> {
    emit_body(
        om,
        PlatformOpKind::CanonicalW16,
        conv,
        src,
        |om, fb, entry, ctx| {
            let flags = ctx.flags;
            let region = fb.block_params(entry)[0];
            let uoff = fb.block_params(entry)[1];
            let ulen = fb.block_params(entry)[2];
            let out_region = fb.block_params(entry)[3];
            let out_cap = fb.block_params(entry)[4];
            let gle = fref(om, fb, ctx.ids.get_last_error);
            let st = state_ptr(om, ctx.state, fb);

            let neg_cap = fb.ins().icmp_imm(IntCC::SignedLessThan, out_cap, 0);
            let (b0, k0) = branch2(fb, neg_cap);
            fb.switch_to_block(b0);
            ret_code(fb, ERR_INVALID_BOUNDS);
            fb.switch_to_block(k0);

            // scratch_cap = clamp(out_cap, 1, 32768)
            let one = i32v(fb, 1);
            let c32768 = i32v(fb, 32_768);
            let _t110 = fb.ins().icmp_imm(IntCC::SignedLessThan, out_cap, 1);

            let cap1 = fb.ins().select(_t110, one, out_cap);
            let _t111 = fb.ins().icmp(IntCC::SignedGreaterThan, cap1, c32768);

            let scratch_cap = fb.ins().select(_t111, c32768, cap1);

            let dst = stack_buf(fb, PATH_BUF_BYTES);
            let units = fref(om, fb, ctx.units);
            let cap = i32v(fb, PATH_UNITS_MAX);
            let uc = fb.ins().call(units, &[dst, cap, region, uoff, ulen]);
            let n = fb.inst_results(uc)[0];
            let bad = fb.ins().icmp_imm(IntCC::SignedLessThanOrEqual, n, 0);
            let (b1, k1) = branch2(fb, bad);
            fb.switch_to_block(b1);
            ret_code(fb, ERR_INVALID_BOUNDS);
            fb.switch_to_block(k1);

            // Resolve exclusively via CreateFileW + GetFinalPathNameByHandleW so
            // missing/unopenable targets fail with ERR_OS_FAILED (aligned with
            // AV / A2R / A2Py D-04; no GetFullPathNameW logical fallback).
            let cf = fref(om, fb, ctx.ids.create_file_w);
            let z64c = i64v(fb, 0);
            let acc_attrs = i32v(fb, FILE_READ_ATTRIBUTES as i32);
            let acc_share = i32v(fb, FILE_SHARE_ALL as i32);
            let disp_exist = i32v(fb, 3); // OPEN_EXISTING
            let flag_backup = i32v(fb, FILE_FLAG_BACKUP_SEMANTICS as i32);
            let open_c = fb.ins().call(
                cf,
                &[
                    dst,
                    acc_attrs,
                    acc_share,
                    z64c,
                    disp_exist,
                    flag_backup,
                    z64c,
                ],
            );
            let h = fb.inst_results(open_c)[0];
            let not_open = fb.ins().icmp_imm(IntCC::Equal, h, INVALID_HANDLE_VALUE);
            let (bad_open, try_final) = branch2(fb, not_open);
            fb.switch_to_block(bad_open);
            os_fail(
                om,
                fb,
                gle,
                ctx.state,
                flags,
                ERROR_INVALID_PARAMETER as i32,
            );

            fb.switch_to_block(try_final);
            let final_buf = state_scratch_ptr(om, fb, ctx.state, 2);
            let gfpn = fref(om, fb, ctx.ids.get_final_path_name_by_handle_w);
            let _t117 = i32v(fb, CANON_FINAL_UNITS);
            let _t118 = i32v(fb, 0);

            let fc = fb.ins().call(gfpn, &[h, final_buf, _t117, _t118]);
            let nf = fb.inst_results(fc)[0];
            let _t120 = fb.ins().icmp_imm(IntCC::SignedLessThanOrEqual, nf, 0);
            let _t121 = fb.ins().icmp_imm(
                IntCC::SignedGreaterThanOrEqual,
                nf,
                CANON_FINAL_UNITS as i64,
            );

            let no_final = fb.ins().bor(_t120, _t121);
            let (bad_final, have_final) = branch2(fb, no_final);
            fb.switch_to_block(bad_final);
            let gc = fb.ins().call(gle, &[]);
            let ecode = fb.inst_results(gc)[0];
            let ch_bad = fref(om, fb, ctx.ids.close_handle);
            let _ = fb.ins().call(ch_bad, &[h]);
            let ez = fb.ins().icmp_imm(IntCC::Equal, ecode, 0);
            let fb_err = i32v(fb, ERROR_INVALID_PARAMETER as i32);
            let rec_e = fb.ins().select(ez, fb_err, ecode);
            fb.ins().store(flags, rec_e, st, H_LAST_OS_ERROR);
            ret_code(fb, ERR_OS_FAILED);

            fb.switch_to_block(have_final);
            let ch_ok = fref(om, fb, ctx.ids.close_handle);
            let _ = fb.ins().call(ch_ok, &[h]);
            // resolved but over the scratch cap -> os_fail(122)
            let over = fb.ins().icmp(IntCC::SignedGreaterThan, nf, scratch_cap);
            let (of, fits) = branch2(fb, over);
            fb.switch_to_block(of);
            let ins_err = i32v(fb, ERROR_INSUFFICIENT_BUFFER as i32);
            fb.ins().store(flags, ins_err, st, H_LAST_OS_ERROR);
            ret_code(fb, ERR_OS_FAILED);

            fb.switch_to_block(fits);
            let wru = fref(om, fb, ctx.wru);
            let wc = fb.ins().call(wru, &[out_region, final_buf, nf]);
            let wres = fb.inst_results(wc)[0];
            let wbad = fb.ins().icmp_imm(IntCC::SignedLessThan, wres, 0);
            let (bw2, kw2) = branch2(fb, wbad);
            fb.switch_to_block(bw2);
            fb.ins().return_(&[wres]);
            fb.switch_to_block(kw2);
            let _t122 = i32v(fb, 0);

            fb.ins().store(flags, _t122, st, H_LAST_OS_ERROR);
            fb.ins().return_(&[nf]);
        },
    )
}

// ---- F-DIR bodies --------------------------------------------------------

fn emit_dir_open_w16(om: &mut ObjModule, src: Src<'_>, conv: CallConv) -> Result<(), MnError> {
    emit_body(
        om,
        PlatformOpKind::DirOpenW16,
        conv,
        src,
        |om, fb, entry, ctx| {
            let flags = ctx.flags;
            let region = fb.block_params(entry)[0];
            let uoff = fb.block_params(entry)[1];
            let ulen = fb.block_params(entry)[2];
            let gle = fref(om, fb, ctx.ids.get_last_error);
            let st = state_ptr(om, ctx.state, fb);

            // units + "\*" + NUL — the `<dir>\*` enumeration pattern.
            let dst = stack_buf(fb, PATH_BUF_BYTES + 8);
            let units = fref(om, fb, ctx.units);
            let cap = i32v(fb, PATH_UNITS_MAX - 2);
            let uc = fb.ins().call(units, &[dst, cap, region, uoff, ulen]);
            let n = fb.inst_results(uc)[0];
            let bad = fb.ins().icmp_imm(IntCC::SignedLessThanOrEqual, n, 0);
            let (b0, k0) = branch2(fb, bad);
            fb.switch_to_block(b0);
            ret_code(fb, ERR_INVALID_BOUNDS);
            fb.switch_to_block(k0);
            // overwrite the units NUL with '\\', then '*', then NUL.
            let n64 = fb.ins().uextend(types::I64, n);
            let at = fb.ins().imul_imm(n64, 2);
            let p1 = fb.ins().iadd(dst, at);
            let _t124 = i32v(fb, 0x5C);

            fb.ins().store(flags, _t124, p1, 0);
            let _t125 = i64v(fb, 2);

            let p2 = fb.ins().iadd(p1, _t125);
            let _t126 = i32v(fb, 0x2A);

            fb.ins().store(flags, _t126, p2, 0);
            let _t127 = i64v(fb, 2);

            let p3 = fb.ins().iadd(p2, _t127);
            let _t128 = i32v(fb, 0);

            fb.ins().store(flags, _t128, p3, 0);

            let data = stack_buf(fb, WFD_BYTES);
            let _t129 = fref(om, fb, ctx.ids.find_first_file_w);

            let c = fb.ins().call(_t129, &[dst, data]);
            let h = fb.inst_results(c)[0];
            let bad_h = fb.ins().icmp_imm(IntCC::Equal, h, INVALID_HANDLE_VALUE);
            let (bh, kh) = branch2(fb, bad_h);
            fb.switch_to_block(bh);
            os_fail(om, fb, gle, ctx.state, flags, 5);
            fb.switch_to_block(kh);

            let alloc = fref(om, fb, ctx.loc.alloc_slot);
            let _t130 = i32v(fb, KIND_DIR as i32);
            let _t131 = i64v(fb, 0);
            let _t132 = i64v(fb, 0);
            let _t133 = i32v(fb, 0);

            let ac = fb.ins().call(alloc, &[_t130, h, _t131, _t132, _t133]);
            let hd = fb.inst_results(ac)[0];
            // Stage the first entry (cFileName) into this slot's staging area.
            let idx = handle_idx(fb, hd);
            let stage = dir_stage_addr(om, fb, ctx.state, idx);
            let _t134 = i64v(fb, WFD_CFILENAME as i64);

            let cfile = fb.ins().iadd(data, _t134);
            let name_len = cstr_len_units(fb, flags, cfile, 260);
            let _t135 = fb.ins().uextend(types::I64, name_len);

            let name_bytes = fb.ins().imul_imm(_t135, 2);
            copy_bytes(fb, flags, stage, cfile, name_bytes);
            let slot = {
                let base = state_ptr(om, ctx.state, fb);
                let idx64 = fb.ins().uextend(types::I64, idx);
                let off = fb.ins().imul_imm(idx64, SLOT_BYTES as i64);
                let _t137 = i64v(fb, STATE_HEADER_BYTES as i64);
                let _t136 = fb.ins().iadd(off, _t137);

                fb.ins().iadd(base, _t136)
            };
            let _t138 = i32v(fb, 1);

            fb.ins().store(flags, _t138, slot, S_DIR_PENDING);
            fb.ins().store(flags, name_len, slot, S_DIR_PENDING_LEN);
            let _t139 = i32v(fb, 0);

            fb.ins().store(flags, _t139, st, H_LAST_OS_ERROR);
            fb.ins().return_(&[hd]);
        },
    )
}

fn emit_dir_next_w16(om: &mut ObjModule, src: Src<'_>, conv: CallConv) -> Result<(), MnError> {
    emit_body(
        om,
        PlatformOpKind::DirNextW16,
        conv,
        src,
        |om, fb, entry, ctx| {
            let flags = ctx.flags;
            let h = fb.block_params(entry)[0];
            let out_region = fb.block_params(entry)[1];
            let out_cap = fb.block_params(entry)[2];
            let gle = fref(om, fb, ctx.ids.get_last_error);
            let find = fref(om, fb, ctx.loc.find_slot);
            let st = state_ptr(om, ctx.state, fb);

            let neg_cap = fb.ins().icmp_imm(IntCC::SignedLessThan, out_cap, 0);
            let (b0, k0) = branch2(fb, neg_cap);
            fb.switch_to_block(b0);
            ret_code(fb, ERR_INVALID_BOUNDS);
            fb.switch_to_block(k0);

            let slot = slot_lookup(fb, find, h, KIND_DIR);
            let pending = fb.ins().load(types::I32, flags, slot, S_DIR_PENDING);
            let is_pending = fb.ins().icmp_imm(IntCC::NotEqual, pending, 0);
            let (drain, advance) = branch2(fb, is_pending);

            // name source + length join.
            let join = fb.create_block();
            fb.append_block_param(join, types::I64); // name ptr
            fb.append_block_param(join, types::I32); // name len

            fb.switch_to_block(drain);
            let _t140 = i32v(fb, 0);

            fb.ins().store(flags, _t140, slot, S_DIR_PENDING);
            let idx = handle_idx(fb, h);
            let stage = dir_stage_addr(om, fb, ctx.state, idx);
            let plen = fb.ins().load(types::I32, flags, slot, S_DIR_PENDING_LEN);
            fb.ins().jump(join, &[stage.into(), plen.into()]);

            fb.switch_to_block(advance);
            let data = stack_buf(fb, WFD_BYTES);
            let _t141 = fref(om, fb, ctx.ids.find_next_file_w);
            let _t142 = fb.ins().load(types::I64, flags, slot, S_OS_PTR);

            let c = fb.ins().call(_t141, &[_t142, data]);
            let ok = fb.inst_results(c)[0];
            let failed = fb.ins().icmp_imm(IntCC::Equal, ok, 0);
            let (bf, kf) = branch2(fb, failed);
            fb.switch_to_block(bf);
            let gc = fb.ins().call(gle, &[]);
            let ecode = fb.inst_results(gc)[0];
            let is_end = fb
                .ins()
                .icmp_imm(IntCC::Equal, ecode, ERROR_NO_MORE_FILES as i64);
            let (end, hard) = branch2(fb, is_end);
            fb.switch_to_block(end);
            let _t143 = i32v(fb, 0);

            fb.ins().return_(&[_t143]);
            fb.switch_to_block(hard);
            os_fail(om, fb, gle, ctx.state, flags, 6);

            fb.switch_to_block(kf);
            let _t144 = i64v(fb, WFD_CFILENAME as i64);

            let cfile = fb.ins().iadd(data, _t144);
            let nlen = cstr_len_units(fb, flags, cfile, 260);
            fb.ins().jump(join, &[cfile.into(), nlen.into()]);

            fb.seal_block(join);
            fb.switch_to_block(join);
            let name_ptr = fb.block_params(join)[0];
            let name_len = fb.block_params(join)[1];

            let over = fb.ins().icmp(IntCC::SignedGreaterThan, name_len, out_cap);
            let (bo, ko) = branch2(fb, over);
            fb.switch_to_block(bo);
            ret_code(fb, ERR_INVALID_BOUNDS);
            fb.switch_to_block(ko);
            let wru = fref(om, fb, ctx.wru);
            let wc = fb.ins().call(wru, &[out_region, name_ptr, name_len]);
            let wres = fb.inst_results(wc)[0];
            let wbad = fb.ins().icmp_imm(IntCC::SignedLessThan, wres, 0);
            let (bw, kw2) = branch2(fb, wbad);
            fb.switch_to_block(bw);
            fb.ins().return_(&[wres]);
            fb.switch_to_block(kw2);
            let _t145 = i32v(fb, 0);

            fb.ins().store(flags, _t145, st, H_LAST_OS_ERROR);
            fb.ins().return_(&[name_len]);
        },
    )
}

fn emit_dir_close(om: &mut ObjModule, src: Src<'_>, conv: CallConv) -> Result<(), MnError> {
    emit_body(
        om,
        PlatformOpKind::DirClose,
        conv,
        src,
        |om, fb, entry, ctx| {
            let flags = ctx.flags;
            let h = fb.block_params(entry)[0];
            let gle = fref(om, fb, ctx.ids.get_last_error);
            let find = fref(om, fb, ctx.loc.find_slot);
            let slot = slot_lookup(fb, find, h, KIND_DIR);
            let os_h = fb.ins().load(types::I64, flags, slot, S_OS_PTR);
            let _t146 = fref(om, fb, ctx.ids.find_close);

            let c = fb.ins().call(_t146, &[os_h]);
            let ok = fb.inst_results(c)[0];
            let failed = fb.ins().icmp_imm(IntCC::Equal, ok, 0);
            let (bf, kf) = branch2(fb, failed);
            fb.switch_to_block(bf);
            os_fail(om, fb, gle, ctx.state, flags, 6);
            fb.switch_to_block(kf);
            let _t147 = i32v(fb, 1);

            fb.ins().store(flags, _t147, slot, S_RELEASED);
            let st = state_ptr(om, ctx.state, fb);
            let _t148 = i32v(fb, 0);

            fb.ins().store(flags, _t148, st, H_LAST_OS_ERROR);
            let _t149 = i32v(fb, 0);

            fb.ins().return_(&[_t149]);
        },
    )
}

// ---- F-ENV bodies --------------------------------------------------------

fn emit_env_get_w16(om: &mut ObjModule, src: Src<'_>, conv: CallConv) -> Result<(), MnError> {
    emit_body(
        om,
        PlatformOpKind::EnvGetW16,
        conv,
        src,
        |om, fb, entry, ctx| {
            let flags = ctx.flags;
            let n_region = fb.block_params(entry)[0];
            let n_off = fb.block_params(entry)[1];
            let n_len = fb.block_params(entry)[2];
            let out_region = fb.block_params(entry)[3];
            let out_cap = fb.block_params(entry)[4];
            let gle = fref(om, fb, ctx.ids.get_last_error);
            let st = state_ptr(om, ctx.state, fb);

            let neg_cap = fb.ins().icmp_imm(IntCC::SignedLessThan, out_cap, 0);
            let (b0, k0) = branch2(fb, neg_cap);
            fb.switch_to_block(b0);
            ret_code(fb, ERR_INVALID_BOUNDS);
            fb.switch_to_block(k0);

            let nbuf = state_scratch_ptr(om, fb, ctx.state, 0);
            let units = fref(om, fb, ctx.units);
            let _t150 = i32v(fb, ENV_UNITS_MAX - 1);

            let uc = fb.ins().call(units, &[nbuf, _t150, n_region, n_off, n_len]);
            let n = fb.inst_results(uc)[0];
            // AV env ops squash units errors (incl. empty name) to -4.
            let bad = fb.ins().icmp_imm(IntCC::SignedLessThanOrEqual, n, 0);
            let (b1, k1) = branch2(fb, bad);
            fb.switch_to_block(b1);
            ret_code(fb, ERR_INVALID_BOUNDS);
            fb.switch_to_block(k1);

            // scratch_cap = clamp(out_cap, 1, 32768)
            let one = i32v(fb, 1);
            let c32768 = i32v(fb, 32_768);
            let _t151 = fb.ins().icmp_imm(IntCC::SignedLessThan, out_cap, 1);

            let cap1 = fb.ins().select(_t151, one, out_cap);
            let _t152 = fb.ins().icmp(IntCC::SignedGreaterThan, cap1, c32768);

            let scratch_cap = fb.ins().select(_t152, c32768, cap1);

            let vbuf = state_scratch_ptr(om, fb, ctx.state, 1);
            let _t153 = i32v(fb, 1);

            let cap_arg = fb.ins().iadd(scratch_cap, _t153);
            // Clear GetLastError before GetEnvironmentVariableW so ret==0 & gle==0
            // means a present empty variable `""` (returns 0), whereas unset sets
            // ERROR_ENVVAR_NOT_FOUND (203) -> ERR_OS_FAILED (-8), matching AV D-03.
            let sle = fref(om, fb, ctx.ids.set_last_error);
            let zero32 = i32v(fb, 0);
            let _ = fb.ins().call(sle, &[zero32]);
            let _t154 = fref(om, fb, ctx.ids.get_environment_variable_w);

            let c = fb.ins().call(_t154, &[nbuf, vbuf, cap_arg]);
            let ret = fb.inst_results(c)[0];
            let is0 = fb.ins().icmp_imm(IntCC::Equal, ret, 0);
            let (zero_ret, have) = branch2(fb, is0);
            fb.switch_to_block(zero_ret);
            let gc = fb.ins().call(gle, &[]);
            let ecode = fb.inst_results(gc)[0];
            let is_empty = fb.ins().icmp_imm(IntCC::Equal, ecode, 0);
            let (present_empty, missing_or_err) = branch2(fb, is_empty);
            fb.switch_to_block(present_empty);
            let _t155 = i32v(fb, 0);
            fb.ins().store(flags, _t155, st, H_LAST_OS_ERROR);
            let _t156 = i32v(fb, 0);
            fb.ins().return_(&[_t156]);
            fb.switch_to_block(missing_or_err);
            fb.ins().store(flags, ecode, st, H_LAST_OS_ERROR);
            ret_code(fb, ERR_OS_FAILED);

            fb.switch_to_block(have);
            let over = fb.ins().icmp(IntCC::SignedGreaterThan, ret, scratch_cap);
            let (of, fits) = branch2(fb, over);
            fb.switch_to_block(of);
            os_fail(om, fb, gle, ctx.state, flags, ERROR_MORE_DATA as i32);
            fb.switch_to_block(fits);
            let wru = fref(om, fb, ctx.wru);
            let wc = fb.ins().call(wru, &[out_region, vbuf, ret]);
            let wres = fb.inst_results(wc)[0];
            let wbad = fb.ins().icmp_imm(IntCC::SignedLessThan, wres, 0);
            let (bw, kw) = branch2(fb, wbad);
            fb.switch_to_block(bw);
            fb.ins().return_(&[wres]);
            fb.switch_to_block(kw);
            let _t157 = i32v(fb, 0);

            fb.ins().store(flags, _t157, st, H_LAST_OS_ERROR);
            fb.ins().return_(&[ret]);
        },
    )
}

fn emit_env_set_w16(om: &mut ObjModule, src: Src<'_>, conv: CallConv) -> Result<(), MnError> {
    emit_body(
        om,
        PlatformOpKind::EnvSetW16,
        conv,
        src,
        |om, fb, entry, ctx| {
            let flags = ctx.flags;
            let n_region = fb.block_params(entry)[0];
            let n_off = fb.block_params(entry)[1];
            let n_len = fb.block_params(entry)[2];
            let v_region = fb.block_params(entry)[3];
            let v_off = fb.block_params(entry)[4];
            let v_len = fb.block_params(entry)[5];
            let gle = fref(om, fb, ctx.ids.get_last_error);

            let nbuf = state_scratch_ptr(om, fb, ctx.state, 0);
            let vbuf = state_scratch_ptr(om, fb, ctx.state, 1);
            let units = fref(om, fb, ctx.units);
            let _t158 = i32v(fb, ENV_UNITS_MAX - 1);

            let uc = fb.ins().call(units, &[nbuf, _t158, n_region, n_off, n_len]);
            let n = fb.inst_results(uc)[0];
            let nbad = fb.ins().icmp_imm(IntCC::SignedLessThan, n, 0);
            let (b1, k1) = branch2(fb, nbad);
            fb.switch_to_block(b1);
            ret_code(fb, ERR_INVALID_BOUNDS);
            fb.switch_to_block(k1);
            let nempty = fb.ins().icmp_imm(IntCC::Equal, n, 0);
            let (b2, k2) = branch2(fb, nempty);
            fb.switch_to_block(b2);
            ret_code(fb, ERR_INVALID_BOUNDS);
            fb.switch_to_block(k2);
            // value may be empty (empty value removes); cap at the OS limit.
            let _t159 = i32v(fb, ENV_UNITS_MAX - 1);
            let uc2 = fb.ins().call(units, &[vbuf, _t159, v_region, v_off, v_len]);
            let v = fb.inst_results(uc2)[0];
            let vbad = fb.ins().icmp_imm(IntCC::SignedLessThan, v, 0);
            let (b3, k3) = branch2(fb, vbad);
            fb.switch_to_block(b3);
            ret_code(fb, ERR_INVALID_BOUNDS);
            fb.switch_to_block(k3);
            let _t160 = fref(om, fb, ctx.ids.set_environment_variable_w);

            let c = fb.ins().call(_t160, &[nbuf, vbuf]);
            let ok = fb.inst_results(c)[0];
            let failed = fb.ins().icmp_imm(IntCC::Equal, ok, 0);
            let (bf, kf) = branch2(fb, failed);
            fb.switch_to_block(bf);
            os_fail(om, fb, gle, ctx.state, flags, 5);
            fb.switch_to_block(kf);
            let st = state_ptr(om, ctx.state, fb);
            let _t161 = i32v(fb, 0);

            fb.ins().store(flags, _t161, st, H_LAST_OS_ERROR);
            let _t162 = i32v(fb, 0);

            fb.ins().return_(&[_t162]);
        },
    )
}

fn emit_env_remove_w16(om: &mut ObjModule, src: Src<'_>, conv: CallConv) -> Result<(), MnError> {
    emit_body(
        om,
        PlatformOpKind::EnvRemoveW16,
        conv,
        src,
        |om, fb, entry, ctx| {
            let flags = ctx.flags;
            let n_region = fb.block_params(entry)[0];
            let n_off = fb.block_params(entry)[1];
            let n_len = fb.block_params(entry)[2];
            let gle = fref(om, fb, ctx.ids.get_last_error);
            let nbuf = state_scratch_ptr(om, fb, ctx.state, 0);
            let units = fref(om, fb, ctx.units);
            let _t163 = i32v(fb, ENV_UNITS_MAX - 1);

            let uc = fb.ins().call(units, &[nbuf, _t163, n_region, n_off, n_len]);
            let n = fb.inst_results(uc)[0];
            let bad = fb.ins().icmp_imm(IntCC::SignedLessThanOrEqual, n, 0);
            let (b1, k1) = branch2(fb, bad);
            fb.switch_to_block(b1);
            ret_code(fb, ERR_INVALID_BOUNDS);
            fb.switch_to_block(k1);
            let _t164 = fref(om, fb, ctx.ids.set_environment_variable_w);
            let _t165 = i64v(fb, 0);

            let c = fb.ins().call(_t164, &[nbuf, _t165]);
            let ok = fb.inst_results(c)[0];
            let failed = fb.ins().icmp_imm(IntCC::Equal, ok, 0);
            let (bf, kf) = branch2(fb, failed);
            fb.switch_to_block(bf);
            os_fail(om, fb, gle, ctx.state, flags, 5);
            fb.switch_to_block(kf);
            let st = state_ptr(om, ctx.state, fb);
            let _t166 = i32v(fb, 0);

            fb.ins().store(flags, _t166, st, H_LAST_OS_ERROR);
            let _t167 = i32v(fb, 0);

            fb.ins().return_(&[_t167]);
        },
    )
}

// ---- F-TIME / F-ENTROPY bodies -------------------------------------------

fn emit_clock_unix_ms(om: &mut ObjModule, src: Src<'_>, conv: CallConv) -> Result<(), MnError> {
    emit_body(
        om,
        PlatformOpKind::ClockUnixMs,
        conv,
        src,
        |om, fb, _entry, ctx| {
            let flags = ctx.flags;
            let ft = stack_buf(fb, 16);
            let _t168 = fref(om, fb, ctx.ids.get_system_time_as_file_time);

            fb.ins().call(_t168, &[ft]);
            let _t169 = fb.ins().uload32(flags, ft, 0);

            let low = _t169;
            let _t170 = fb.ins().uload32(flags, ft, 4);
            let high = _t170;
            let _t171 = fb.ins().ishl_imm(high, 32);

            let raw = fb.ins().bor(_t171, low);
            let _t172 = i64v(fb, FILETIME_UNIX_EPOCH_100NS);

            let shifted = fb.ins().isub(raw, _t172);
            let _t173 = i64v(fb, 10_000);

            let ms = fb.ins().sdiv(shifted, _t173);
            let st = state_ptr(om, ctx.state, fb);
            let _t175 = fb.ins().ushr_imm(ms, 32);
            let _t174 = fb.ins().ireduce(types::I32, _t175);

            fb.ins().store(flags, _t174, st, H_LAST_CLOCK_HI);
            let _t176 = fb.ins().ireduce(types::I32, ms);

            fb.ins().return_(&[_t176]);
        },
    )
}

fn emit_clock_mono_ms(om: &mut ObjModule, src: Src<'_>, conv: CallConv) -> Result<(), MnError> {
    emit_body(
        om,
        PlatformOpKind::ClockMonoMs,
        conv,
        src,
        |om, fb, _entry, ctx| {
            let flags = ctx.flags;
            let fbuf = stack_buf(fb, 16);
            let cbuf = stack_buf(fb, 16);
            let _t177 = fref(om, fb, ctx.ids.query_performance_frequency);

            let c1 = fb.ins().call(_t177, &[fbuf]);
            let ok1 = fb.inst_results(c1)[0];
            let _t178 = fref(om, fb, ctx.ids.query_performance_counter);

            let c2 = fb.ins().call(_t178, &[cbuf]);
            let ok2 = fb.inst_results(c2)[0];
            let freq = fb.ins().load(types::I64, flags, fbuf, 0);
            let counter = fb.ins().load(types::I64, flags, cbuf, 0);
            let _t181 = fb.ins().icmp_imm(IntCC::Equal, ok1, 0);
            let _t182 = fb.ins().icmp_imm(IntCC::Equal, ok2, 0);
            let _t179 = fb.ins().bor(_t181, _t182);
            let _t180 = fb.ins().icmp_imm(IntCC::SignedLessThanOrEqual, freq, 0);

            let bad = fb.ins().bor(_t179, _t180);
            let gle = fref(om, fb, ctx.ids.get_last_error);
            let (bb, kb) = branch2(fb, bad);
            fb.switch_to_block(bb);
            os_fail(
                om,
                fb,
                gle,
                ctx.state,
                flags,
                ERROR_INVALID_PARAMETER as i32,
            );
            fb.switch_to_block(kb);
            // counter.saturating_mul(1000): QPC counters are non-negative — a
            // nonzero high half of the 128-bit product means saturation.
            let _t183 = i64v(fb, 1000);
            let prod_hi = fb.ins().umulhi(counter, _t183);
            let _t184 = i64v(fb, 1000);

            let prod_lo = fb.ins().imul(counter, _t184);
            let saturated = fb.ins().icmp_imm(IntCC::NotEqual, prod_hi, 0);
            let _t185 = i64v(fb, i64::MAX);
            let _t186 = fb.ins().sdiv(prod_lo, freq);

            let ms = fb.ins().select(saturated, _t185, _t186);
            let st = state_ptr(om, ctx.state, fb);
            let _t188 = fb.ins().ushr_imm(ms, 32);
            let _t187 = fb.ins().ireduce(types::I32, _t188);

            fb.ins().store(flags, _t187, st, H_LAST_CLOCK_HI);
            let _t189 = fb.ins().ireduce(types::I32, ms);

            fb.ins().return_(&[_t189]);
        },
    )
}

fn emit_clock_hi(om: &mut ObjModule, src: Src<'_>, conv: CallConv) -> Result<(), MnError> {
    emit_body(
        om,
        PlatformOpKind::ClockHi,
        conv,
        src,
        |om, fb, _entry, ctx| {
            let st = state_ptr(om, ctx.state, fb);
            let v = fb.ins().load(types::I32, ctx.flags, st, H_LAST_CLOCK_HI);
            fb.ins().return_(&[v]);
        },
    )
}

fn emit_sleep_ms(om: &mut ObjModule, src: Src<'_>, conv: CallConv) -> Result<(), MnError> {
    emit_body(
        om,
        PlatformOpKind::SleepMs,
        conv,
        src,
        |om, fb, entry, ctx| {
            let ms = fb.block_params(entry)[0];
            let neg = fb.ins().icmp_imm(IntCC::SignedLessThan, ms, 0);
            let (b0, k0) = branch2(fb, neg);
            fb.switch_to_block(b0);
            ret_code(fb, ERR_INVALID_VALUE);
            fb.switch_to_block(k0);
            let sleep = fref(om, fb, ctx.ids.sleep_ex);
            let lp = fb.create_block();
            fb.append_block_param(lp, types::I32); // remaining
            fb.ins().jump(lp, &[ms.into()]);
            let done = fb.create_block();
            let body = fb.create_block();
            fb.switch_to_block(lp);
            let remaining = fb.block_params(lp)[0];
            let fin = fb.ins().icmp_imm(IntCC::Equal, remaining, 0);
            fb.ins().brif(fin, done, &[], body, &[]);
            fb.seal_block(done);
            fb.seal_block(body);
            fb.switch_to_block(body);
            let fifty = i32v(fb, 50);
            let lt50 = fb.ins().icmp(IntCC::SignedLessThan, remaining, fifty);
            let slice = fb.ins().select(lt50, remaining, fifty);
            let _t190 = i32v(fb, 0);

            fb.ins().call(sleep, &[slice, _t190]);
            let _t191 = fb.ins().isub(remaining, slice);

            fb.ins().jump(lp, &[_t191.into()]);
            fb.seal_block(lp);
            fb.switch_to_block(done);
            let _t192 = i32v(fb, 0);

            fb.ins().return_(&[_t192]);
        },
    )
}

fn emit_rand_fill(om: &mut ObjModule, src: Src<'_>, conv: CallConv) -> Result<(), MnError> {
    emit_body(
        om,
        PlatformOpKind::RandFill,
        conv,
        src,
        |om, fb, entry, ctx| {
            let flags = ctx.flags;
            let region = fb.block_params(entry)[0];
            let off = fb.block_params(entry)[1];
            let len = fb.block_params(entry)[2];
            let find = fref(om, fb, ctx.loc.find_slot);

            let empty = fb.ins().icmp_imm(IntCC::Equal, len, 0);
            let (b0, k0) = branch2(fb, empty);
            fb.switch_to_block(b0);
            let _t193 = i32v(fb, 0);

            fb.ins().return_(&[_t193]);
            fb.switch_to_block(k0);
            let over_call = fb.ins().icmp_imm(
                IntCC::SignedGreaterThan,
                len,
                MAX_PLATFORM_CALL_BYTES as i64,
            );
            let (bb, kb) = branch2(fb, over_call);
            fb.switch_to_block(bb);
            ret_code(fb, ERR_BUDGET);
            fb.switch_to_block(kb);

            let (base, off64, len64) =
                region_lookup_squashed(fb, find, region, off, len, ERR_WRONG_ORIGIN, flags);
            let st = state_ptr(om, ctx.state, fb);
            let rt = fb.ins().load(types::I64, flags, st, H_READ_TOTAL);
            let _t194 = fb.ins().iadd(rt, len64);

            let over_run = fb.ins().icmp_imm(
                IntCC::SignedGreaterThan,
                _t194,
                MAX_PLATFORM_RUN_BYTES as i64,
            );
            let (b1, k1) = branch2(fb, over_run);
            fb.switch_to_block(b1);
            ret_code(fb, ERR_BUDGET);
            fb.switch_to_block(k1);

            let dst = fb.ins().iadd(base, off64);
            let _t195 = fref(om, fb, ctx.ids.bcrypt_gen_random);
            let _t196 = i64v(fb, 0);
            let _t197 = i32v(fb, BCRYPT_USE_SYSTEM_PREFERRED_RNG as i32);

            let c = fb.ins().call(_t195, &[_t196, dst, len, _t197]);
            let status = fb.inst_results(c)[0];
            let bad = fb.ins().icmp_imm(IntCC::NotEqual, status, 0);
            let (bs, ks) = branch2(fb, bad);
            fb.switch_to_block(bs);
            let _t198 = fb.ins().uextend(types::I64, status);
            let _t199 = i64v(fb, NTSTATUS_FAULT_MARKER);

            let marked = fb.ins().bor(_t198, _t199);
            let _t200 = fb.ins().ireduce(types::I32, marked);

            fb.ins().store(flags, _t200, st, H_LAST_OS_ERROR);
            ret_code(fb, ERR_OS_FAILED);
            fb.switch_to_block(ks);
            let rt2 = fb.ins().load(types::I64, flags, st, H_READ_TOTAL);
            let _t201 = fb.ins().iadd(rt2, len64);

            fb.ins().store(flags, _t201, st, H_READ_TOTAL);
            let _t202 = i32v(fb, 0);

            fb.ins().store(flags, _t202, st, H_LAST_OS_ERROR);
            let _t203 = i32v(fb, 0);

            fb.ins().return_(&[_t203]);
        },
    )
}

// ---- F-PROC bodies -------------------------------------------------------

fn emit_proc_spawn_w16(om: &mut ObjModule, src: Src<'_>, conv: CallConv) -> Result<(), MnError> {
    emit_body(
        om,
        PlatformOpKind::ProcSpawnW16,
        conv,
        src,
        |om, fb, entry, ctx| {
            let flags = ctx.flags;
            let cmd_region = fb.block_params(entry)[0];
            let uoff = fb.block_params(entry)[1];
            let ulen = fb.block_params(entry)[2];
            let mode = fb.block_params(entry)[3];
            let gle = fref(om, fb, ctx.ids.get_last_error);
            let z64 = i64v(fb, 0);

            // cmdline units (empty -> -4, AV squash).
            let cmdbuf = state_scratch_ptr(om, fb, ctx.state, 3);
            let units = fref(om, fb, ctx.units);
            let _t204 = i32v(fb, ENV_UNITS_MAX - 1);

            let uc = fb
                .ins()
                .call(units, &[cmdbuf, _t204, cmd_region, uoff, ulen]);
            let n = fb.inst_results(uc)[0];
            let bad = fb.ins().icmp_imm(IntCC::SignedLessThanOrEqual, n, 0);
            let (b0, k0) = branch2(fb, bad);
            fb.switch_to_block(b0);
            ret_code(fb, ERR_INVALID_BOUNDS);
            fb.switch_to_block(k0);

            // kill-on-close job (V-03).
            let _t205 = fref(om, fb, ctx.ids.create_job_object_w);
            let job_call = fb.ins().call(_t205, &[z64, z64]);
            let job = fb.inst_results(job_call)[0];
            let no_job = fb.ins().icmp_imm(IntCC::Equal, job, 0);
            let (bj, kj) = branch2(fb, no_job);
            fb.switch_to_block(bj);
            os_fail(om, fb, gle, ctx.state, flags, 8);
            fb.switch_to_block(kj);

            let jobi = zeroed_stack_buf(fb, JOBI_BYTES);
            let _t206 = i32v(fb, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE as i32);

            fb.ins().store(flags, _t206, jobi, JOBI_LIMITFLAGS);
            let _t207 = fref(om, fb, ctx.ids.set_information_job_object);
            let _t208 = i32v(fb, JOBOBJECT_EXTENDED_LIMIT_INFO_CLASS as i32);
            let _t209 = i32v(fb, JOBI_REAL_BYTES as i32);

            let sic = fb.ins().call(_t207, &[job, _t208, jobi, _t209]);
            let si_ok = fb.inst_results(sic)[0];
            let si_bad = fb.ins().icmp_imm(IntCC::Equal, si_ok, 0);
            let (bs, ks) = branch2(fb, si_bad);
            fb.switch_to_block(bs);
            let _t210 = fref(om, fb, ctx.ids.close_handle);

            let _ = fb.ins().call(_t210, &[job]);
            os_fail(om, fb, gle, ctx.state, flags, 6);
            fb.switch_to_block(ks);

            // capture pipes (mode bit0); pipe scratch slots live in the entry
            // block so both arms can read them (no-capture reads are unused).
            let out_r_ptr = stack_buf(fb, 16);
            let out_w_ptr = stack_buf(fb, 16);
            let err_r_ptr = stack_buf(fb, 16);
            let err_w_ptr = stack_buf(fb, 16);
            fb.ins().store(flags, z64, out_r_ptr, 0);
            fb.ins().store(flags, z64, out_w_ptr, 0);
            fb.ins().store(flags, z64, err_r_ptr, 0);
            fb.ins().store(flags, z64, err_w_ptr, 0);
            let _t211 = i32v(fb, 1);

            let capture = fb.ins().band(mode, _t211);
            let use_std = fb.ins().icmp_imm(IntCC::NotEqual, capture, 0);
            let (do_cap, no_cap) = split_open(fb, use_std);
            fb.switch_to_block(do_cap);
            let sa = zeroed_stack_buf(fb, SA_BYTES);
            let _t212 = i32v(fb, 24);

            fb.ins().store(flags, _t212, sa, SA_NLENGTH);
            fb.ins().store(flags, z64, sa, 8);
            let _t213 = i32v(fb, 1);

            fb.ins().store(flags, _t213, sa, SA_INHERIT);
            let _t214 = fref(om, fb, ctx.ids.create_pipe);
            let _t215 = i32v(fb, PIPE_BUF as i32);

            let pc1 = fb.ins().call(_t214, &[out_r_ptr, out_w_ptr, sa, _t215]);
            let p1_ok = fb.inst_results(pc1)[0];
            let p1_bad = fb.ins().icmp_imm(IntCC::Equal, p1_ok, 0);
            let (bp1, kp1) = branch2(fb, p1_bad);
            fb.switch_to_block(bp1);
            let _t216 = fref(om, fb, ctx.ids.close_handle);

            let _ = fb.ins().call(_t216, &[job]);
            os_fail(om, fb, gle, ctx.state, flags, 6);
            fb.switch_to_block(kp1);
            let _t217 = fref(om, fb, ctx.ids.create_pipe);
            let _t218 = i32v(fb, PIPE_BUF as i32);

            let pc2 = fb.ins().call(_t217, &[err_r_ptr, err_w_ptr, sa, _t218]);
            let p2_ok = fb.inst_results(pc2)[0];
            let p2_bad = fb.ins().icmp_imm(IntCC::Equal, p2_ok, 0);
            let (bp2, kp2) = branch2(fb, p2_bad);
            fb.switch_to_block(bp2);
            let o_r1 = fb.ins().load(types::I64, flags, out_r_ptr, 0);
            let o_w1 = fb.ins().load(types::I64, flags, out_w_ptr, 0);
            let _t219 = fref(om, fb, ctx.ids.close_handle);

            let _ = fb.ins().call(_t219, &[o_r1]);
            let _t220 = fref(om, fb, ctx.ids.close_handle);

            let _ = fb.ins().call(_t220, &[o_w1]);
            let _t221 = fref(om, fb, ctx.ids.close_handle);

            let _ = fb.ins().call(_t221, &[job]);
            os_fail(om, fb, gle, ctx.state, flags, 6);
            fb.switch_to_block(kp2);
            // read ends are NOT inheritable (mask 0x1, clear).
            let o_r2 = fb.ins().load(types::I64, flags, out_r_ptr, 0);
            let e_r2 = fb.ins().load(types::I64, flags, err_r_ptr, 0);
            let _t222 = fref(om, fb, ctx.ids.set_handle_information);
            let _t223 = i32v(fb, 1);
            let _t224 = i32v(fb, 0);

            let _ = fb.ins().call(_t222, &[o_r2, _t223, _t224]);
            let _t225 = fref(om, fb, ctx.ids.set_handle_information);
            let _t226 = i32v(fb, 1);
            let _t227 = i32v(fb, 0);

            let _ = fb.ins().call(_t225, &[e_r2, _t226, _t227]);
            fb.ins().jump(no_cap, &[]);
            fb.seal_block(no_cap);

            fb.switch_to_block(no_cap);
            let out_r = fb.ins().load(types::I64, flags, out_r_ptr, 0);
            let out_w = fb.ins().load(types::I64, flags, out_w_ptr, 0);
            let err_r = fb.ins().load(types::I64, flags, err_r_ptr, 0);
            let err_w = fb.ins().load(types::I64, flags, err_w_ptr, 0);

            // STARTUPINFOW.
            let si = zeroed_stack_buf(fb, SI_BYTES);
            let _t228 = i32v(fb, 104);

            fb.ins().store(flags, _t228, si, SI_CB);
            let set_std = fb.create_block();
            let after_std = fb.create_block();
            fb.ins().brif(use_std, set_std, &[], after_std, &[]);
            fb.seal_block(set_std);
            fb.switch_to_block(set_std);
            let _t229 = i32v(fb, STARTF_USESTDHANDLES as i32);

            fb.ins().store(flags, _t229, si, SI_DWFLAGS);
            let _t230 = i64v(fb, -1);

            fb.ins().store(flags, _t230, si, SI_HSTDIN);
            fb.ins().store(flags, out_w, si, SI_HSTDOUT);
            fb.ins().store(flags, err_w, si, SI_HSTDERR);
            fb.ins().jump(after_std, &[]);
            fb.seal_block(after_std);
            fb.switch_to_block(after_std);
            let _t231 = i32v(fb, 1);
            let _t232 = i32v(fb, 0);

            let inherit = fb.ins().select(use_std, _t231, _t232);
            let pi = stack_buf(fb, PI_BYTES);
            let _t233 = fref(om, fb, ctx.ids.create_process_w);
            let _t234 = i32v(fb, CREATE_NO_WINDOW as i32);

            let cpc = fb.ins().call(
                _t233,
                &[z64, cmdbuf, z64, z64, inherit, _t234, z64, z64, si, pi],
            );
            let cp_ok = fb.inst_results(cpc)[0];
            let cp_bad = fb.ins().icmp_imm(IntCC::Equal, cp_ok, 0);
            let (bcp, kcp) = branch2(fb, cp_bad);
            fb.switch_to_block(bcp);
            let cleanup = fb.create_block();
            let skip_cleanup = fb.create_block();
            fb.ins().brif(use_std, cleanup, &[], skip_cleanup, &[]);
            fb.seal_block(cleanup);
            fb.switch_to_block(cleanup);
            let _t235 = fref(om, fb, ctx.ids.close_handle);

            let _ = fb.ins().call(_t235, &[out_r]);
            let _t236 = fref(om, fb, ctx.ids.close_handle);

            let _ = fb.ins().call(_t236, &[out_w]);
            let _t237 = fref(om, fb, ctx.ids.close_handle);

            let _ = fb.ins().call(_t237, &[err_r]);
            let _t238 = fref(om, fb, ctx.ids.close_handle);

            let _ = fb.ins().call(_t238, &[err_w]);
            fb.ins().jump(skip_cleanup, &[]);
            fb.seal_block(skip_cleanup);
            fb.switch_to_block(skip_cleanup);
            let _t239 = fref(om, fb, ctx.ids.close_handle);

            let _ = fb.ins().call(_t239, &[job]);
            os_fail(om, fb, gle, ctx.state, flags, 5);
            fb.switch_to_block(kcp);

            let hproc = fb.ins().load(types::I64, flags, pi, PI_HPROCESS);
            let hthread = fb.ins().load(types::I64, flags, pi, PI_HTHREAD);
            let _t240 = fref(om, fb, ctx.ids.close_handle);

            let _ = fb.ins().call(_t240, &[hthread]);
            let _t241 = fref(om, fb, ctx.ids.assign_process_to_job_object);

            let _ = fb.ins().call(_t241, &[job, hproc]);
            // parent closes the write ends (EOF tracks child exit).
            let close_writes = fb.create_block();
            let after_writes = fb.create_block();
            fb.ins().brif(use_std, close_writes, &[], after_writes, &[]);
            fb.seal_block(close_writes);
            fb.switch_to_block(close_writes);
            let _t242 = fref(om, fb, ctx.ids.close_handle);

            let _ = fb.ins().call(_t242, &[out_w]);
            let _t243 = fref(om, fb, ctx.ids.close_handle);

            let _ = fb.ins().call(_t243, &[err_w]);
            fb.ins().jump(after_writes, &[]);
            fb.seal_block(after_writes);
            fb.switch_to_block(after_writes);

            let st = state_ptr(om, ctx.state, fb);
            let alloc = fref(om, fb, ctx.loc.alloc_slot);
            let _t244 = i32v(fb, KIND_PROC as i32);
            let _t245 = i32v(fb, 0);

            let ac = fb.ins().call(alloc, &[_t244, hproc, z64, z64, _t245]);
            let hd = fb.inst_results(ac)[0];
            let idx2 = handle_idx(fb, hd);
            let slot = {
                let base = state_ptr(om, ctx.state, fb);
                let idx64 = fb.ins().uextend(types::I64, idx2);
                let off = fb.ins().imul_imm(idx64, SLOT_BYTES as i64);
                let _t247 = i64v(fb, STATE_HEADER_BYTES as i64);
                let _t246 = fb.ins().iadd(off, _t247);

                fb.ins().iadd(base, _t246)
            };
            fb.ins().store(flags, job, slot, S_PROC_JOB);
            let or_sel = fb.ins().select(use_std, out_r, z64);
            let er_sel = fb.ins().select(use_std, err_r, z64);
            fb.ins().store(flags, or_sel, slot, S_PROC_OUT);
            fb.ins().store(flags, er_sel, slot, S_PROC_ERR);
            let _t248 = i32v(fb, 0);

            fb.ins().store(flags, _t248, st, H_LAST_OS_ERROR);
            fb.ins().return_(&[hd]);
        },
    )
}

fn emit_proc_pipe_read(om: &mut ObjModule, src: Src<'_>, conv: CallConv) -> Result<(), MnError> {
    emit_body(
        om,
        PlatformOpKind::ProcPipeRead,
        conv,
        src,
        |om, fb, entry, ctx| {
            let flags = ctx.flags;
            let h = fb.block_params(entry)[0];
            let which = fb.block_params(entry)[1];
            let region = fb.block_params(entry)[2];
            let off = fb.block_params(entry)[3];
            let cap = fb.block_params(entry)[4];
            let timeout = fb.block_params(entry)[5];
            let gle = fref(om, fb, ctx.ids.get_last_error);
            let find = fref(om, fb, ctx.loc.find_slot);
            let st = state_ptr(om, ctx.state, fb);

            // which 1..=2; off >= 0; cap > 0 (AV: ERR_INVALID_VALUE).
            let w_lo = fb.ins().icmp_imm(IntCC::SignedLessThan, which, 1);
            let w_hi = fb.ins().icmp_imm(IntCC::SignedGreaterThan, which, 2);
            let off_neg = fb.ins().icmp_imm(IntCC::SignedLessThan, off, 0);
            let cap_neg = fb.ins().icmp_imm(IntCC::SignedLessThanOrEqual, cap, 0);
            let _t250 = fb.ins().bor(w_lo, w_hi);
            let _t249 = fb.ins().bor(_t250, off_neg);

            let bad = fb.ins().bor(_t249, cap_neg);
            let (b0, k0) = branch2(fb, bad);
            fb.switch_to_block(b0);
            ret_code(fb, ERR_INVALID_VALUE);
            fb.switch_to_block(k0);
            // cap_us = min(cap, 65536)
            let maxc = i32v(fb, MAX_PLATFORM_CALL_BYTES as i32);
            let over_max = fb.ins().icmp(IntCC::SignedGreaterThan, cap, maxc);
            let cap_us = fb.ins().select(over_max, maxc, cap);

            // proc target (AV squash: ERR_WRONG_ORIGIN).
            let kp_slot = {
                let kpr = i32v(fb, KIND_PROC as i32);
                let pc0 = fb.ins().call(find, &[h, kpr]);
                let ps0 = fb.inst_results(pc0)[0];
                let ps0neg = fb.ins().icmp_imm(IntCC::SignedLessThan, ps0, 0);
                let (pb, pok) = branch2(fb, ps0neg);
                fb.switch_to_block(pb);
                ret_code(fb, ERR_WRONG_ORIGIN);
                fb.switch_to_block(pok);
                ps0
            };
            let slot = kp_slot;
            let process = fb.ins().load(types::I64, flags, slot, S_OS_PTR);
            let pipe_w1 = fb.ins().load(types::I64, flags, slot, S_PROC_OUT);
            let pipe_w2 = fb.ins().load(types::I64, flags, slot, S_PROC_ERR);
            let sel2 = fb.ins().icmp_imm(IntCC::Equal, which, 1);
            let pipe = fb.ins().select(sel2, pipe_w1, pipe_w2);
            let no_pipe = fb.ins().icmp_imm(IntCC::Equal, pipe, 0);
            let (bnp, knp) = branch2(fb, no_pipe);
            fb.switch_to_block(bnp);
            ret_code(fb, ERR_INVALID_VALUE);
            fb.switch_to_block(knp);

            // region (AV squash: ERR_WRONG_ORIGIN).
            let (base, off64, _) =
                region_lookup_squashed(fb, find, region, off, cap_us, ERR_WRONG_ORIGIN, flags);

            // paced bounded-wait loop: waited (i64) is the loop param.
            let _t251 = i64v(fb, 0);
            let _t252 = i64v(fb, H_BYTES_WRITTEN_TMP as i64);
            let avail_ptr = fb.ins().iadd(st, _t252);
            let lp = fb.create_block();
            fb.append_block_param(lp, types::I64);
            fb.ins().jump(lp, &[_t251.into()]);
            fb.switch_to_block(lp);
            let waited = fb.block_params(lp)[0];
            let _t253 = i32v(fb, 0);

            fb.ins().store(flags, _t253, avail_ptr, 0);
            let _t254 = fref(om, fb, ctx.ids.peek_named_pipe);
            let _t255 = i64v(fb, 0);
            let _t256 = i32v(fb, 0);
            let _t257 = i64v(fb, 0);
            let _t258 = i64v(fb, 0);

            let pk = fb
                .ins()
                .call(_t254, &[pipe, _t255, _t256, _t257, avail_ptr, _t258]);
            let pk_ok = fb.inst_results(pk)[0];
            let pk_bad = fb.ins().icmp_imm(IntCC::Equal, pk_ok, 0);
            let bp = fb.create_block();
            let kp = fb.create_block();
            fb.ins().brif(pk_bad, bp, &[], kp, &[]);
            fb.seal_block(bp);
            fb.switch_to_block(bp);
            let gc = fb.ins().call(gle, &[]);
            let ecode = fb.inst_results(gc)[0];
            let _t259 = fb
                .ins()
                .icmp_imm(IntCC::Equal, ecode, ERROR_BROKEN_PIPE as i64);
            let _t260 = fb.ins().icmp_imm(IntCC::Equal, ecode, ERROR_NO_DATA as i64);

            let is_eof = fb.ins().bor(_t259, _t260);
            let (be, he) = branch2(fb, is_eof);
            fb.switch_to_block(be);
            let _t261 = i32v(fb, 0);

            fb.ins().store(flags, _t261, avail_ptr, 0);
            fb.ins().jump(kp, &[]);
            fb.seal_block(kp);
            fb.switch_to_block(he);
            os_fail(om, fb, gle, ctx.state, flags, 6);

            fb.switch_to_block(kp);
            let avail = fb.ins().load(types::I32, flags, avail_ptr, 0);
            let has_data = fb.ins().icmp_imm(IntCC::SignedGreaterThan, avail, 0);
            let (data, nodata) = branch2(fb, has_data);

            fb.switch_to_block(data);
            let _t262 = fb.ins().icmp(IntCC::SignedLessThan, cap_us, avail);

            let want32 = fb.ins().select(_t262, cap_us, avail);
            let want64 = fb.ins().uextend(types::I64, want32);
            let rt = fb.ins().load(types::I64, flags, st, H_READ_TOTAL);
            let _t263 = fb.ins().iadd(rt, want64);

            let over_run = fb.ins().icmp_imm(
                IntCC::SignedGreaterThan,
                _t263,
                MAX_PLATFORM_RUN_BYTES as i64,
            );
            let (br, kr) = branch2(fb, over_run);
            fb.switch_to_block(br);
            ret_code(fb, ERR_BUDGET);
            fb.switch_to_block(kr);
            let buf = fb.ins().iadd(base, off64);
            let n = emit_read_call(om, fb, ctx, gle, pipe, buf, want32);
            fb.ins().return_(&[n]);

            fb.switch_to_block(nodata);
            let has_dl = fb
                .ins()
                .icmp_imm(IntCC::SignedGreaterThanOrEqual, timeout, 0);
            let dl = fb.create_block();
            let no_dl = fb.create_block();
            fb.ins().brif(has_dl, dl, &[], no_dl, &[]);
            fb.seal_block(dl);
            fb.switch_to_block(dl);
            let _t264 = fb.ins().sextend(types::I64, timeout);

            let over_dl = fb
                .ins()
                .icmp(IntCC::SignedGreaterThanOrEqual, waited, _t264);
            let (bdl, kdl) = branch2(fb, over_dl);
            fb.switch_to_block(bdl);
            ret_code(fb, ERR_TIMEOUT);
            fb.switch_to_block(kdl);
            fb.ins().jump(no_dl, &[]);
            fb.seal_block(no_dl);
            fb.switch_to_block(no_dl);
            // slice = bounded timeout ? max(1, dl-waited) capped at 50 : 50.
            let fifty64 = i64v(fb, 50);
            let _t265 = fb.ins().sextend(types::I64, timeout);

            let dl_rem = fb.ins().isub(_t265, waited);
            let rem_ge1 = fb.ins().icmp_imm(IntCC::SignedLessThan, dl_rem, 1);
            let _t266 = i64v(fb, 1);

            let dl_rem1 = fb.ins().select(rem_ge1, _t266, dl_rem);
            let _t267 = fb.ins().icmp(IntCC::SignedLessThan, dl_rem1, fifty64);

            let dl_slice = fb.ins().select(_t267, dl_rem1, fifty64);
            let slice = fb.ins().select(has_dl, dl_slice, fifty64);
            let _t268 = fref(om, fb, ctx.ids.wait_for_single_object);
            let _t269 = fb.ins().ireduce(types::I32, slice);

            let wc = fb.ins().call(_t268, &[process, _t269]);
            let wr = fb.inst_results(wc)[0];
            let exited = fb.ins().icmp_imm(IntCC::Equal, wr, WAIT_OBJECT_0 as i64);
            let (was_exit, was_to) = branch2(fb, exited);
            fb.switch_to_block(was_exit);
            // child exited: buffered bytes may still drain; empty -> EOF.
            let _t270 = i32v(fb, 0);
            fb.ins().store(flags, _t270, avail_ptr, 0);
            let _t271 = fref(om, fb, ctx.ids.peek_named_pipe);
            let _t272 = i64v(fb, 0);
            let _t273 = i32v(fb, 0);
            let _t274 = i64v(fb, 0);
            let _t275 = i64v(fb, 0);

            let pk2 = fb
                .ins()
                .call(_t271, &[pipe, _t272, _t273, _t274, avail_ptr, _t275]);
            let pk2_ok = fb.inst_results(pk2)[0];
            let pk2_bad = fb.ins().icmp_imm(IntCC::Equal, pk2_ok, 0);
            let bp2 = fb.create_block();
            let kp2 = fb.create_block();
            fb.ins().brif(pk2_bad, bp2, &[], kp2, &[]);
            fb.seal_block(bp2);
            fb.switch_to_block(bp2);
            let gc3 = fb.ins().call(gle, &[]);
            let e3 = fb.inst_results(gc3)[0];
            let _t276 = fb
                .ins()
                .icmp_imm(IntCC::Equal, e3, ERROR_BROKEN_PIPE as i64);
            let _t277 = fb.ins().icmp_imm(IntCC::Equal, e3, ERROR_NO_DATA as i64);

            let eof3 = fb.ins().bor(_t276, _t277);
            let (be3, he3) = branch2(fb, eof3);
            fb.switch_to_block(be3);
            let _t278 = i32v(fb, 0);

            fb.ins().store(flags, _t278, avail_ptr, 0);
            fb.ins().jump(kp2, &[]);
            fb.seal_block(kp2);
            fb.switch_to_block(he3);
            os_fail(om, fb, gle, ctx.state, flags, 6);
            fb.switch_to_block(kp2);
            let avail2 = fb.ins().load(types::I32, flags, avail_ptr, 0);
            let none = fb.ins().icmp_imm(IntCC::Equal, avail2, 0);
            let (eof_blk, more_blk) = branch2(fb, none);
            fb.switch_to_block(eof_blk);
            let _t279 = i32v(fb, 0);

            fb.ins().return_(&[_t279]);
            fb.switch_to_block(more_blk);
            fb.ins().jump(lp, &[waited.into()]);

            fb.switch_to_block(was_to);
            let _t280 = fb.ins().iadd(waited, slice);

            fb.ins().jump(lp, &[_t280.into()]);
            // all lp predecessors exist now (entry, more_blk, was_to).
            fb.seal_block(lp);
        },
    )
}

fn emit_proc_wait(om: &mut ObjModule, src: Src<'_>, conv: CallConv) -> Result<(), MnError> {
    emit_body(
        om,
        PlatformOpKind::ProcWait,
        conv,
        src,
        |om, fb, entry, ctx| {
            let flags = ctx.flags;
            let h = fb.block_params(entry)[0];
            let timeout = fb.block_params(entry)[1];
            let gle = fref(om, fb, ctx.ids.get_last_error);
            let find = fref(om, fb, ctx.loc.find_slot);
            let slot = {
                let kwr = i32v(fb, KIND_PROC as i32);
                let wc0 = fb.ins().call(find, &[h, kwr]);
                let ws0 = fb.inst_results(wc0)[0];
                let ws0neg = fb.ins().icmp_imm(IntCC::SignedLessThan, ws0, 0);
                let (wb, wok) = branch2(fb, ws0neg);
                fb.switch_to_block(wb);
                ret_code(fb, ERR_WRONG_ORIGIN);
                fb.switch_to_block(wok);
                ws0
            };
            let process = fb.ins().load(types::I64, flags, slot, S_OS_PTR);
            let job = fb.ins().load(types::I64, flags, slot, S_PROC_JOB);

            let neg_to = fb.ins().icmp_imm(IntCC::SignedLessThan, timeout, 0);
            let _t281 = i32v(fb, INFINITE_U32 as i32);

            let wait_ms = fb.ins().select(neg_to, _t281, timeout);
            let _t282 = fref(om, fb, ctx.ids.wait_for_single_object);

            let wc = fb.ins().call(_t282, &[process, wait_ms]);
            let wr = fb.inst_results(wc)[0];
            let exited = fb.ins().icmp_imm(IntCC::Equal, wr, WAIT_OBJECT_0 as i64);
            let timedout = fb
                .ins()
                .icmp_imm(IntCC::Equal, wr, WAIT_TIMEOUT_CODE as i64);
            let ok_path = fb.ins().bor(exited, timedout);
            let not_ok = fb.ins().icmp_imm(IntCC::Equal, ok_path, 0);
            let (hard, okk) = branch2(fb, not_ok);
            fb.switch_to_block(hard);
            os_fail(om, fb, gle, ctx.state, flags, 6);
            fb.switch_to_block(okk);
            // branch2's first block is the COND-TRUE arm: timedout -> kill path.
            let not_timedout = fb.ins().icmp_imm(IntCC::Equal, timedout, 0);
            let (exit_blk, to_blk) = branch2(fb, not_timedout);
            fb.switch_to_block(exit_blk);
            let st = state_ptr(om, ctx.state, fb);
            let _t283 = i64v(fb, H_BYTES_WRITTEN_TMP as i64);

            let code_ptr = fb.ins().iadd(st, _t283);
            let _t284 = i32v(fb, 0);

            fb.ins().store(flags, _t284, code_ptr, 0);
            let _t285 = fref(om, fb, ctx.ids.get_exit_code_process);

            let gc = fb.ins().call(_t285, &[process, code_ptr]);
            let g_ok = fb.inst_results(gc)[0];
            let g_bad = fb.ins().icmp_imm(IntCC::Equal, g_ok, 0);
            let (bg, kg) = branch2(fb, g_bad);
            fb.switch_to_block(bg);
            os_fail(om, fb, gle, ctx.state, flags, 6);
            fb.switch_to_block(kg);
            let code = fb.ins().load(types::I32, flags, code_ptr, 0);
            let _t286 = i32v(fb, 0);

            fb.ins().store(flags, _t286, st, H_LAST_OS_ERROR);
            fb.ins().return_(&[code]);

            fb.switch_to_block(to_blk);
            // timeout AFTER killing the job tree and settling (V-03).
            let _t287 = fref(om, fb, ctx.ids.terminate_job_object);
            let _t288 = i32v(fb, 1);
            let _ = fb.ins().call(_t287, &[job, _t288]);
            let _t289 = fref(om, fb, ctx.ids.wait_for_single_object);
            let _t290 = i32v(fb, INFINITE_U32 as i32);

            let _ = fb.ins().call(_t289, &[process, _t290]);
            ret_code(fb, ERR_TIMEOUT);
        },
    )
}

fn emit_proc_release(om: &mut ObjModule, src: Src<'_>, conv: CallConv) -> Result<(), MnError> {
    emit_body(
        om,
        PlatformOpKind::ProcRelease,
        conv,
        src,
        |om, fb, entry, ctx| {
            let flags = ctx.flags;
            let h = fb.block_params(entry)[0];
            let gle = fref(om, fb, ctx.ids.get_last_error);
            let find = fref(om, fb, ctx.loc.find_slot);
            // AV op_proc_release keeps the EXACT lookup codes (decode -2,
            // released -6, wrong origin -3) — no squash here.
            let slot = slot_lookup(fb, find, h, KIND_PROC);
            let process = fb.ins().load(types::I64, flags, slot, S_OS_PTR);
            let job = fb.ins().load(types::I64, flags, slot, S_PROC_JOB);
            let out_r = fb.ins().load(types::I64, flags, slot, S_PROC_OUT);
            let err_r = fb.ins().load(types::I64, flags, slot, S_PROC_ERR);
            let ch = fref(om, fb, ctx.ids.close_handle);
            // best-effort closes: pipe read ends (if any) and the job.
            for v in [out_r, err_r, job] {
                let is0 = fb.ins().icmp_imm(IntCC::Equal, v, 0);
                let do_close = fb.create_block();
                let skip = fb.create_block();
                fb.ins().brif(is0, skip, &[], do_close, &[]);
                fb.seal_block(do_close);
                fb.switch_to_block(do_close);
                let _ = fb.ins().call(ch, &[v]);
                fb.ins().jump(skip, &[]);
                fb.seal_block(skip);
                fb.switch_to_block(skip);
            }
            let c = fb.ins().call(ch, &[process]);
            let ok = fb.inst_results(c)[0];
            let failed = fb.ins().icmp_imm(IntCC::Equal, ok, 0);
            let (bf, kf) = branch2(fb, failed);
            fb.switch_to_block(bf);
            os_fail(om, fb, gle, ctx.state, flags, 6);
            fb.switch_to_block(kf);
            let _t291 = i32v(fb, 1);

            fb.ins().store(flags, _t291, slot, S_RELEASED);
            let st = state_ptr(om, ctx.state, fb);
            let _t292 = i32v(fb, 0);

            fb.ins().store(flags, _t292, st, H_LAST_OS_ERROR);
            let _t293 = i32v(fb, 0);

            fb.ins().return_(&[_t293]);
        },
    )
}

fn emit_proc_exit(om: &mut ObjModule, src: Src<'_>, conv: CallConv) -> Result<(), MnError> {
    emit_body(
        om,
        PlatformOpKind::ProcExit,
        conv,
        src,
        |om, fb, entry, ctx| {
            let code = fb.block_params(entry)[0];
            let _t294 = fref(om, fb, ctx.ids.exit_process);

            fb.ins().call(_t294, &[code]);
            // ExitProcess never returns; satisfy the CLIF verifier.
            fb.ins()
                .trap(TrapCode::user(1).expect("valid user trap code"));
        },
    )
}

fn emit_proc_cmdline_w16(om: &mut ObjModule, src: Src<'_>, conv: CallConv) -> Result<(), MnError> {
    emit_body(
        om,
        PlatformOpKind::ProcCmdlineW16,
        conv,
        src,
        |om, fb, entry, ctx| {
            let flags = ctx.flags;
            let out_region = fb.block_params(entry)[0];
            let out_cap = fb.block_params(entry)[1];
            let gle = fref(om, fb, ctx.ids.get_last_error);
            let st = state_ptr(om, ctx.state, fb);

            let neg_cap = fb.ins().icmp_imm(IntCC::SignedLessThan, out_cap, 0);
            let (b0, k0) = branch2(fb, neg_cap);
            fb.switch_to_block(b0);
            ret_code(fb, ERR_INVALID_BOUNDS);
            fb.switch_to_block(k0);
            let _t295 = fref(om, fb, ctx.ids.get_command_line_w);

            let cmdline_call = fb.ins().call(_t295, &[]);
            let p = fb.inst_results(cmdline_call)[0];
            let null_p = fb.ins().icmp_imm(IntCC::Equal, p, 0);
            let (bn, kn) = branch2(fb, null_p);
            fb.switch_to_block(bn);
            os_fail(om, fb, gle, ctx.state, flags, 8);
            fb.switch_to_block(kn);
            let len = cstr_len_units(fb, flags, p, 32_768);

            // scratch_cap = clamp(out_cap, 1, 32768)
            let one = i32v(fb, 1);
            let c32768 = i32v(fb, 32_768);
            let _t296 = fb.ins().icmp_imm(IntCC::SignedLessThan, out_cap, 1);

            let cap1 = fb.ins().select(_t296, one, out_cap);
            let _t297 = fb.ins().icmp(IntCC::SignedGreaterThan, cap1, c32768);

            let scratch_cap = fb.ins().select(_t297, c32768, cap1);
            let over = fb.ins().icmp(IntCC::SignedGreaterThan, len, scratch_cap);
            let (bo, ko) = branch2(fb, over);
            fb.switch_to_block(bo);
            os_fail(
                om,
                fb,
                gle,
                ctx.state,
                flags,
                ERROR_INSUFFICIENT_BUFFER as i32,
            );
            fb.switch_to_block(ko);
            let wru = fref(om, fb, ctx.wru);
            let wc = fb.ins().call(wru, &[out_region, p, len]);
            let wres = fb.inst_results(wc)[0];
            let wbad = fb.ins().icmp_imm(IntCC::SignedLessThan, wres, 0);
            let (bw, kw) = branch2(fb, wbad);
            fb.switch_to_block(bw);
            fb.ins().return_(&[wres]);
            fb.switch_to_block(kw);
            let _t298 = i32v(fb, 0);

            fb.ins().store(flags, _t298, st, H_LAST_OS_ERROR);
            fb.ins().return_(&[len]);
        },
    )
}

fn emit_proc_cwd_w16(om: &mut ObjModule, src: Src<'_>, conv: CallConv) -> Result<(), MnError> {
    emit_body(
        om,
        PlatformOpKind::ProcCwdW16,
        conv,
        src,
        |om, fb, entry, ctx| {
            let flags = ctx.flags;
            let out_region = fb.block_params(entry)[0];
            let out_cap = fb.block_params(entry)[1];
            let gle = fref(om, fb, ctx.ids.get_last_error);
            let st = state_ptr(om, ctx.state, fb);

            let neg_cap = fb.ins().icmp_imm(IntCC::SignedLessThan, out_cap, 0);
            let (b0, k0) = branch2(fb, neg_cap);
            fb.switch_to_block(b0);
            ret_code(fb, ERR_INVALID_BOUNDS);
            fb.switch_to_block(k0);

            // scratch_cap = clamp(out_cap, 1, 32768); GetCurrentDirectoryW(cap+1, buf)
            let one = i32v(fb, 1);
            let c32768 = i32v(fb, 32_768);
            let _t299 = fb.ins().icmp_imm(IntCC::SignedLessThan, out_cap, 1);

            let cap1 = fb.ins().select(_t299, one, out_cap);
            let _t300 = fb.ins().icmp(IntCC::SignedGreaterThan, cap1, c32768);

            let scratch_cap = fb.ins().select(_t300, c32768, cap1);
            let buf = state_scratch_ptr(om, fb, ctx.state, 1);
            let _t301 = i32v(fb, 1);

            let cap_arg = fb.ins().iadd(scratch_cap, _t301);
            let _t302 = fref(om, fb, ctx.ids.get_current_directory_w);

            let c = fb.ins().call(_t302, &[cap_arg, buf]);
            let ret = fb.inst_results(c)[0];
            let is0 = fb.ins().icmp_imm(IntCC::Equal, ret, 0);
            let (b1, k1) = branch2(fb, is0);
            fb.switch_to_block(b1);
            os_fail(om, fb, gle, ctx.state, flags, 6);
            fb.switch_to_block(k1);
            let over = fb.ins().icmp(IntCC::SignedGreaterThan, ret, scratch_cap);
            let (bo, ko) = branch2(fb, over);
            fb.switch_to_block(bo);
            os_fail(om, fb, gle, ctx.state, flags, ERROR_MORE_DATA as i32);
            fb.switch_to_block(ko);
            let wru = fref(om, fb, ctx.wru);
            let wc = fb.ins().call(wru, &[out_region, buf, ret]);
            let wres = fb.inst_results(wc)[0];
            let wbad = fb.ins().icmp_imm(IntCC::SignedLessThan, wres, 0);
            let (bw, kw) = branch2(fb, wbad);
            fb.switch_to_block(bw);
            fb.ins().return_(&[wres]);
            fb.switch_to_block(kw);
            let _t303 = i32v(fb, 0);

            fb.ins().store(flags, _t303, st, H_LAST_OS_ERROR);
            fb.ins().return_(&[ret]);
        },
    )
}

fn emit_proc_chdir_w16(om: &mut ObjModule, src: Src<'_>, conv: CallConv) -> Result<(), MnError> {
    emit_body(
        om,
        PlatformOpKind::ProcChdirW16,
        conv,
        src,
        |om, fb, entry, ctx| {
            let flags = ctx.flags;
            let region = fb.block_params(entry)[0];
            let uoff = fb.block_params(entry)[1];
            let ulen = fb.block_params(entry)[2];
            let gle = fref(om, fb, ctx.ids.get_last_error);
            let dst = stack_buf(fb, PATH_BUF_BYTES);
            let units = fref(om, fb, ctx.units);
            let _t304 = i32v(fb, PATH_UNITS_MAX);

            let uc = fb.ins().call(units, &[dst, _t304, region, uoff, ulen]);
            let n = fb.inst_results(uc)[0];
            let bad = fb.ins().icmp_imm(IntCC::SignedLessThanOrEqual, n, 0);
            let (b0, k0) = branch2(fb, bad);
            fb.switch_to_block(b0);
            ret_code(fb, ERR_INVALID_BOUNDS);
            fb.switch_to_block(k0);
            let _t305 = fref(om, fb, ctx.ids.set_current_directory_w);

            let c = fb.ins().call(_t305, &[dst]);
            let ok = fb.inst_results(c)[0];
            let failed = fb.ins().icmp_imm(IntCC::Equal, ok, 0);
            let (bf, kf) = branch2(fb, failed);
            fb.switch_to_block(bf);
            os_fail(om, fb, gle, ctx.state, flags, 5);
            fb.switch_to_block(kf);
            let st = state_ptr(om, ctx.state, fb);
            let _t306 = i32v(fb, 0);

            fb.ins().store(flags, _t306, st, H_LAST_OS_ERROR);
            let _t307 = i32v(fb, 0);

            fb.ins().return_(&[_t307]);
        },
    )
}
