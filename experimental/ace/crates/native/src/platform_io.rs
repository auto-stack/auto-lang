//! PLAN-1039: Native Win64 COFF shim & PE dependency/symbol audit for
//! `platform.io/1` and `platform.mem/1`.
//!
//! Emits `mn-platform-shim` directly via Cranelift COFF (`x86_64-pc-windows-msvc`),
//! holding a `.bss` ledger (`mn_pio_state`) and importing ONLY `kernel32.dll`
//! Win32 entrypoints (`GetStdHandle`, `WriteFile`, `GetLastError`,
//! `GetSystemInfo`, `VirtualAlloc`, `VirtualFree`) with ZERO libc/ucrtbase/
//! `malloc`/`free`/`std::io`.

use crate::emit::{make_isa, WINCALL};
use crate::MnError;
use ace_av_core::platform_io::{
    PlatformOpKind, ERR_ALREADY_RELEASED, ERR_BORROWED, ERR_BUDGET, ERR_INVALID_BOUNDS,
    ERR_INVALID_HANDLE, ERR_INVALID_VALUE, ERR_OS_FAILED, ERR_WRONG_ORIGIN, MAX_OS_REGION_BYTES,
    MAX_PLATFORM_CALL_BYTES, MAX_PLATFORM_RUN_BYTES,
};
use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::ir::{
    types, AbiParam, InstBuilder, MemFlags, Signature, StackSlotData, StackSlotKind, Value,
};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_module::{default_libcall_names, DataDescription, DataId, FuncId, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};

pub(crate) const MAX_NATIVE_SLOTS: usize = 256;
pub(crate) const STATE_HEADER_BYTES: usize = 64;
pub(crate) const SLOT_BYTES: usize = 64;
pub(crate) const STATE_BYTES: usize = STATE_HEADER_BYTES + MAX_NATIVE_SLOTS * SLOT_BYTES;

// Header field byte offsets inside `mn_pio_state`
pub(crate) const H_NEXT_GEN: i32 = 0; // i32
pub(crate) const H_LAST_OS_ERROR: i32 = 4; // i32
pub(crate) const H_PAGE_SIZE: i32 = 8; // i32
pub(crate) const H_ALLOC_GRANULARITY: i32 = 12; // i32
pub(crate) const H_WRITTEN_TOTAL: i32 = 16; // i64
pub(crate) const H_LIVE_MAPPED: i32 = 24; // i64
pub(crate) const H_NUM_SLOTS: i32 = 32; // i32
pub(crate) const H_BYTES_WRITTEN_TMP: i32 = 36; // i32

// Per-slot field byte offsets (stride 64 bytes)
pub(crate) const S_KIND: i32 = 0; // i32 (0=empty, 1=StdStream, 2=OsMemRegion)
pub(crate) const S_RELEASED: i32 = 4; // i32
pub(crate) const S_GEN: i32 = 8; // i32
pub(crate) const S_BORROW: i32 = 12; // i32
pub(crate) const S_OS_PTR: i32 = 16; // i64
pub(crate) const S_LOGICAL_LEN: i32 = 24; // i64
pub(crate) const S_MAPPED_LEN: i32 = 32; // i64

pub(crate) const KIND_STREAM: i64 = 1;
pub(crate) const KIND_MEM: i64 = 2;

const MEM_COMMIT_RESERVE: i64 = 0x3000;
const PAGE_READWRITE: i64 = 0x04;
const MEM_RELEASE: i64 = 0x8000;

// SYSTEM_INFO x64 field offsets (matches av-core::platform_io::win32::SYSTEM_INFO)
const SYSINFO_BYTES: u32 = 48;
const SYSINFO_PAGE_SIZE_OFF: i32 = 4;
const SYSINFO_GRANULARITY_OFF: i32 = 40;

pub fn shim_symbol_for(kind: PlatformOpKind) -> &'static str {
    match kind {
        PlatformOpKind::IoStdout => "mn_pio_stdout",
        PlatformOpKind::IoStderr => "mn_pio_stderr",
        PlatformOpKind::IoWriteSome => "mn_pio_write_some",
        PlatformOpKind::IoWriteSomeSlice => "mn_pio_write_some_slice",
        PlatformOpKind::IoLastError => "mn_pio_last_error",
        PlatformOpKind::MemPageSize => "mn_pio_mem_page_size",
        PlatformOpKind::MemAllocGranularity => "mn_pio_mem_alloc_granularity",
        PlatformOpKind::MemAlloc => "mn_pio_mem_alloc",
        PlatformOpKind::MemFree => "mn_pio_mem_free",
        PlatformOpKind::MemLogicalLen => "mn_pio_mem_logical_len",
        PlatformOpKind::MemMappedLen => "mn_pio_mem_mapped_len",
        PlatformOpKind::MemBorrow => "mn_pio_mem_borrow",
        PlatformOpKind::MemUnborrow => "mn_pio_mem_unborrow",
        PlatformOpKind::MemWriteU8 => "mn_pio_mem_write_u8",
        PlatformOpKind::MemReadU8 => "mn_pio_mem_read_u8",
        // PLAN-1040 Linux probe seam shims (raw syscall marshalling).
        PlatformOpKind::ProbeOpenat => "mn_pio_probe_openat",
        PlatformOpKind::ProbeWriteFd => "mn_pio_probe_write_fd",
        PlatformOpKind::ProbeClose => "mn_pio_probe_close",
        // PLAN-1041 Windows text-console classification + Unicode write.
        PlatformOpKind::IoConsoleKind => "mn_pio_console_kind",
        PlatformOpKind::IoConsoleWrite16 => "mn_pio_console_write16",
    }
}

pub fn cranelift_shim_sig(kind: PlatformOpKind) -> Signature {
    cranelift_shim_sig_for(kind, WINCALL)
}

/// PLAN-1040: the shim signature under the target's call convention
/// (`WindowsFastcall` for COFF, `SystemV` for the Linux ELF shim).
pub fn cranelift_shim_sig_for(
    kind: PlatformOpKind,
    conv: cranelift_codegen::isa::CallConv,
) -> Signature {
    let params = match kind {
        PlatformOpKind::IoStdout
        | PlatformOpKind::IoStderr
        | PlatformOpKind::IoLastError
        | PlatformOpKind::MemPageSize
        | PlatformOpKind::MemAllocGranularity => vec![],
        PlatformOpKind::IoWriteSome => vec![
            AbiParam::new(types::I32),
            AbiParam::new(types::I64),
            AbiParam::new(types::I32),
        ],
        PlatformOpKind::IoWriteSomeSlice => vec![
            AbiParam::new(types::I32),
            AbiParam::new(types::I64),
            AbiParam::new(types::I32),
            AbiParam::new(types::I32),
            AbiParam::new(types::I32),
        ],
        PlatformOpKind::MemAlloc
        | PlatformOpKind::MemFree
        | PlatformOpKind::MemLogicalLen
        | PlatformOpKind::MemMappedLen
        | PlatformOpKind::MemBorrow
        | PlatformOpKind::MemUnborrow => vec![AbiParam::new(types::I32)],
        PlatformOpKind::IoConsoleKind => vec![AbiParam::new(types::I32)],
        PlatformOpKind::IoConsoleWrite16 => vec![
            AbiParam::new(types::I32),
            AbiParam::new(types::I32),
            AbiParam::new(types::I32),
            AbiParam::new(types::I32),
        ],
        PlatformOpKind::MemReadU8 => vec![AbiParam::new(types::I32), AbiParam::new(types::I32)],
        PlatformOpKind::MemWriteU8 => vec![
            AbiParam::new(types::I32),
            AbiParam::new(types::I32),
            AbiParam::new(types::I32),
        ],
        PlatformOpKind::ProbeOpenat => vec![
            AbiParam::new(types::I32),
            AbiParam::new(types::I64),
            AbiParam::new(types::I32),
            AbiParam::new(types::I32),
            AbiParam::new(types::I32),
        ],
        PlatformOpKind::ProbeWriteFd => vec![
            AbiParam::new(types::I32),
            AbiParam::new(types::I64),
            AbiParam::new(types::I32),
        ],
        PlatformOpKind::ProbeClose => vec![AbiParam::new(types::I32)],
    };
    Signature {
        call_conv: conv,
        params,
        returns: vec![AbiParam::new(types::I32)],
    }
}

fn sig(params: &[types::Type], ret: Option<types::Type>) -> Signature {
    Signature {
        call_conv: WINCALL,
        params: params.iter().copied().map(AbiParam::new).collect(),
        returns: ret.into_iter().map(AbiParam::new).collect(),
    }
}

struct Win32Ids {
    get_std_handle: FuncId,
    write_file: FuncId,
    get_last_error: FuncId,
    get_system_info: FuncId,
    virtual_alloc: FuncId,
    virtual_free: FuncId,
    // PLAN-1041: Windows text-console classification + Unicode write.
    get_console_mode: FuncId,
    write_console_w: FuncId,
}

struct LocalIds {
    state_sysinfo: FuncId,
    find_slot: FuncId,
    alloc_slot: FuncId,
    write_slice: FuncId,
}

pub(crate) fn state_ptr(
    om: &mut ObjectModule,
    state: DataId,
    fb: &mut FunctionBuilder<'_>,
) -> Value {
    let gv = om.declare_data_in_func(state, fb.func);
    fb.ins().symbol_value(types::I64, gv)
}

pub(crate) fn ret_err(fb: &mut FunctionBuilder<'_>, code: i32) {
    let v = fb.ins().iconst(types::I32, code as i64);
    fb.ins().return_(&[v]);
}

pub(crate) fn define_fn<F>(
    om: &mut ObjectModule,
    id: FuncId,
    f_sig: Signature,
    build: F,
) -> Result<(), MnError>
where
    F: FnOnce(&mut ObjectModule, &mut FunctionBuilder<'_>, cranelift_codegen::ir::Block),
{
    let mut ctx = om.make_context();
    ctx.func.signature = f_sig;
    {
        let mut fb_ctx = FunctionBuilderContext::new();
        let mut fb = FunctionBuilder::new(&mut ctx.func, &mut fb_ctx);
        let entry = fb.create_block();
        fb.append_block_params_for_function_params(entry);
        fb.switch_to_block(entry);
        fb.seal_block(entry);
        build(om, &mut fb, entry);
        fb.finalize();
    }
    om.define_function(id, &mut ctx)
        .map_err(|e| MnError::Backend(format!("define platform shim fn: {e}")))
}

pub(crate) fn declare_fn(
    om: &mut ObjectModule,
    name: &str,
    linkage: Linkage,
    f_sig: &Signature,
) -> Result<FuncId, MnError> {
    om.declare_function(name, linkage, f_sig)
        .map_err(|e| MnError::Backend(format!("declare {name}: {e}")))
}

/// Emit the standalone checked Win64 platform shim COFF (`mn-platform-shim`).
/// Zero C runtime, zero `msvcrt`/`ucrtbase`, zero `malloc`/`free`, zero `std::io`.
pub fn emit_platform_shim_coff() -> Result<Vec<u8>, MnError> {
    let isa = make_isa()?;
    let builder = ObjectBuilder::new(isa, "mn-platform-shim", default_libcall_names())
        .map_err(|e| MnError::Backend(format!("platform shim builder: {e}")))?;
    let mut om = ObjectModule::new(builder);

    let state = om
        .declare_data("mn_pio_state", Linkage::Local, true, false)
        .map_err(|e| MnError::Backend(format!("declare mn_pio_state: {e}")))?;
    let mut dd = DataDescription::new();
    dd.define_zeroinit(STATE_BYTES);
    om.define_data(state, &dd)
        .map_err(|e| MnError::Backend(format!("define mn_pio_state: {e}")))?;

    let win = Win32Ids {
        get_std_handle: declare_fn(
            &mut om,
            "GetStdHandle",
            Linkage::Import,
            &sig(&[types::I32], Some(types::I64)),
        )?,
        write_file: declare_fn(
            &mut om,
            "WriteFile",
            Linkage::Import,
            &sig(
                &[types::I64, types::I64, types::I32, types::I64, types::I64],
                Some(types::I32),
            ),
        )?,
        get_last_error: declare_fn(
            &mut om,
            "GetLastError",
            Linkage::Import,
            &sig(&[], Some(types::I32)),
        )?,
        get_system_info: declare_fn(
            &mut om,
            "GetSystemInfo",
            Linkage::Import,
            &sig(&[types::I64], None),
        )?,
        virtual_alloc: declare_fn(
            &mut om,
            "VirtualAlloc",
            Linkage::Import,
            &sig(
                &[types::I64, types::I64, types::I32, types::I32],
                Some(types::I64),
            ),
        )?,
        virtual_free: declare_fn(
            &mut om,
            "VirtualFree",
            Linkage::Import,
            &sig(&[types::I64, types::I64, types::I32], Some(types::I32)),
        )?,
        get_console_mode: declare_fn(
            &mut om,
            "GetConsoleMode",
            Linkage::Import,
            &sig(&[types::I64, types::I64], Some(types::I32)),
        )?,
        write_console_w: declare_fn(
            &mut om,
            "WriteConsoleW",
            Linkage::Import,
            &sig(
                &[types::I64, types::I64, types::I32, types::I64, types::I64],
                Some(types::I32),
            ),
        )?,
    };

    let state_sysinfo_sig = sig(&[], Some(types::I64));
    let find_slot_sig = sig(&[types::I32, types::I32], Some(types::I64));
    let alloc_slot_sig = sig(
        &[types::I32, types::I64, types::I64, types::I64],
        Some(types::I32),
    );
    let write_slice_sig = sig(
        &[types::I32, types::I64, types::I32, types::I32, types::I32],
        Some(types::I32),
    );

    let loc = LocalIds {
        state_sysinfo: declare_fn(
            &mut om,
            "mn_pio_local_state_sysinfo",
            Linkage::Local,
            &state_sysinfo_sig,
        )?,
        find_slot: declare_fn(
            &mut om,
            "mn_pio_local_find_slot",
            Linkage::Local,
            &find_slot_sig,
        )?,
        alloc_slot: declare_fn(
            &mut om,
            "mn_pio_local_alloc_slot",
            Linkage::Local,
            &alloc_slot_sig,
        )?,
        write_slice: declare_fn(
            &mut om,
            "mn_pio_local_write_slice",
            Linkage::Local,
            &write_slice_sig,
        )?,
    };

    let flags = MemFlags::trusted();

    // 1. `mn_pio_local_state_sysinfo() -> *mut State`
    define_fn(
        &mut om,
        loc.state_sysinfo,
        state_sysinfo_sig.clone(),
        |om, fb, _| {
            let base = state_ptr(om, state, fb);
            let ps = fb.ins().load(types::I32, flags, base, H_PAGE_SIZE);
            let zero = fb.ins().iconst(types::I32, 0);
            let need = fb.ins().icmp(IntCC::Equal, ps, zero);
            let init_blk = fb.create_block();
            let done_blk = fb.create_block();
            fb.ins().brif(need, init_blk, &[], done_blk, &[]);
            fb.seal_block(init_blk);
            fb.switch_to_block(init_blk);
            {
                let ss = fb.create_sized_stack_slot(StackSlotData::new(
                    StackSlotKind::ExplicitSlot,
                    SYSINFO_BYTES,
                    3,
                ));
                let sptr = fb.ins().stack_addr(types::I64, ss, 0);
                let gsi = om.declare_func_in_func(win.get_system_info, fb.func);
                fb.ins().call(gsi, &[sptr]);
                let ps_val = fb
                    .ins()
                    .load(types::I32, flags, sptr, SYSINFO_PAGE_SIZE_OFF);
                let ag_val = fb
                    .ins()
                    .load(types::I32, flags, sptr, SYSINFO_GRANULARITY_OFF);
                fb.ins().store(flags, ps_val, base, H_PAGE_SIZE);
                fb.ins().store(flags, ag_val, base, H_ALLOC_GRANULARITY);
                fb.ins().jump(done_blk, &[]);
            }
            fb.seal_block(done_blk);
            fb.switch_to_block(done_blk);
            fb.ins().return_(&[base]);
        },
    )?;

    // 2. `mn_pio_local_find_slot(handle: i32, expected_kind: i32) -> i64`
    //    Returns slot pointer (> 0) or negative ERR_* sign-extended to i64.
    define_fn(
        &mut om,
        loc.find_slot,
        find_slot_sig.clone(),
        |om, fb, entry| crate::platform_io::emit_find_slot_body(om, fb, entry, state, flags),
    )?;

    // 3. `mn_pio_local_alloc_slot(kind: i32, os_ptr: i64, logical: i64, mapped: i64) -> i32`
    define_fn(
        &mut om,
        loc.alloc_slot,
        alloc_slot_sig.clone(),
        |om, fb, entry| crate::platform_io::emit_alloc_slot_body(om, fb, entry, state, flags),
    )?;

    // 4. `mn_pio_local_write_slice(h, ptr, slen, off, req_len) -> i32`
    define_fn(
        &mut om,
        loc.write_slice,
        write_slice_sig.clone(),
        |om, fb, entry| {
            let h = fb.block_params(entry)[0];
            let ptr = fb.block_params(entry)[1];
            let slen = fb.block_params(entry)[2];
            let off = fb.block_params(entry)[3];
            let req_len = fb.block_params(entry)[4];

            let find = om.declare_func_in_func(loc.find_slot, fb.func);
            let exp_kind = fb.ins().iconst(types::I32, KIND_STREAM);
            let c1 = fb.ins().call(find, &[h, exp_kind]);
            let slot = fb.inst_results(c1)[0];
            let z64 = fb.ins().iconst(types::I64, 0);
            let neg = fb.ins().icmp(IntCC::SignedLessThan, slot, z64);
            let bad_slot = fb.create_block();
            let chk_bounds = fb.create_block();
            fb.ins().brif(neg, bad_slot, &[], chk_bounds, &[]);
            fb.seal_block(bad_slot);
            fb.seal_block(chk_bounds);

            fb.switch_to_block(bad_slot);
            let s32 = fb.ins().ireduce(types::I32, slot);
            fb.ins().return_(&[s32]);

            fb.switch_to_block(chk_bounds);
            let z32 = fb.ins().iconst(types::I32, 0);
            let b1 = fb.ins().icmp(IntCC::SignedLessThan, off, z32);
            let b2 = fb.ins().icmp(IntCC::SignedLessThan, req_len, z32);
            let b3 = fb.ins().icmp(IntCC::SignedLessThan, slen, z32);
            let off64 = fb.ins().sextend(types::I64, off);
            let req64 = fb.ins().sextend(types::I64, req_len);
            let slen64 = fb.ins().sextend(types::I64, slen);
            let end64 = fb.ins().iadd(off64, req64);
            let b4 = fb.ins().icmp(IntCC::SignedGreaterThan, end64, slen64);
            let b12 = fb.ins().bor(b1, b2);
            let b34 = fb.ins().bor(b3, b4);
            let oob = fb.ins().bor(b12, b34);
            let bad_b = fb.create_block();
            let chk_empty = fb.create_block();
            fb.ins().brif(oob, bad_b, &[], chk_empty, &[]);
            fb.seal_block(bad_b);
            fb.seal_block(chk_empty);

            fb.switch_to_block(bad_b);
            ret_err(fb, ERR_INVALID_BOUNDS);

            fb.switch_to_block(chk_empty);
            let is_empty = fb.ins().icmp(IntCC::Equal, req_len, z32);
            let ret_zero = fb.create_block();
            let chk_budget = fb.create_block();
            fb.ins().brif(is_empty, ret_zero, &[], chk_budget, &[]);
            fb.seal_block(ret_zero);
            fb.seal_block(chk_budget);

            fb.switch_to_block(ret_zero);
            fb.ins().return_(&[z32]);

            fb.switch_to_block(chk_budget);
            let base = state_ptr(om, state, fb);
            let max_call = fb.ins().iconst(types::I64, MAX_PLATFORM_CALL_BYTES as i64);
            let over_call = fb.ins().icmp(IntCC::SignedGreaterThan, req64, max_call);
            let chunk64 = fb.ins().select(over_call, max_call, req64);
            let chunk32 = fb.ins().ireduce(types::I32, chunk64);
            let wt = fb.ins().load(types::I64, flags, base, H_WRITTEN_TOTAL);
            let next_wt = fb.ins().iadd(wt, chunk64);
            let max_run = fb.ins().iconst(types::I64, MAX_PLATFORM_RUN_BYTES as i64);
            let over_run = fb.ins().icmp(IntCC::SignedGreaterThan, next_wt, max_run);
            let bad_budget = fb.create_block();
            let do_os = fb.create_block();
            fb.ins().brif(over_run, bad_budget, &[], do_os, &[]);
            fb.seal_block(bad_budget);
            fb.seal_block(do_os);

            fb.switch_to_block(bad_budget);
            ret_err(fb, ERR_BUDGET);

            fb.switch_to_block(do_os);
            fb.ins().store(flags, z32, base, H_BYTES_WRITTEN_TMP);
            let os_h = fb.ins().load(types::I64, flags, slot, S_OS_PTR);
            let buf_ptr = fb.ins().iadd(ptr, off64);
            let tmp_off = fb.ins().iconst(types::I64, H_BYTES_WRITTEN_TMP as i64);
            let out_ptr = fb.ins().iadd(base, tmp_off);
            let wf = om.declare_func_in_func(win.write_file, fb.func);
            let wc = fb.ins().call(wf, &[os_h, buf_ptr, chunk32, out_ptr, z64]);
            let ok = fb.inst_results(wc)[0];
            let failed = fb.ins().icmp(IntCC::Equal, ok, z32);
            let err_os = fb.create_block();
            let ok_os = fb.create_block();
            fb.ins().brif(failed, err_os, &[], ok_os, &[]);
            fb.seal_block(err_os);
            fb.seal_block(ok_os);

            fb.switch_to_block(err_os);
            let gle = om.declare_func_in_func(win.get_last_error, fb.func);
            let gc = fb.ins().call(gle, &[]);
            let ecode = fb.inst_results(gc)[0];
            let ez = fb.ins().icmp(IntCC::Equal, ecode, z32);
            let fb_err = fb.ins().iconst(types::I32, 109);
            let rec_e = fb.ins().select(ez, fb_err, ecode);
            fb.ins().store(flags, rec_e, base, H_LAST_OS_ERROR);
            ret_err(fb, ERR_OS_FAILED);

            fb.switch_to_block(ok_os);
            let nw = fb.ins().load(types::I32, flags, base, H_BYTES_WRITTEN_TMP);
            let gt_chunk = fb.ins().icmp(IntCC::UnsignedGreaterThan, nw, chunk32);
            let nw_clamped = fb.ins().select(gt_chunk, chunk32, nw);
            let nw64 = fb.ins().uextend(types::I64, nw_clamped);
            let wt2 = fb.ins().iadd(wt, nw64);
            fb.ins().store(flags, wt2, base, H_WRITTEN_TOTAL);
            fb.ins().store(flags, z32, base, H_LAST_OS_ERROR);
            fb.ins().return_(&[nw_clamped]);
        },
    )?;

    // Helper closure for stdout / stderr
    let mut emit_std_query = |kind: PlatformOpKind, std_id: i64| -> Result<(), MnError> {
        let name = shim_symbol_for(kind);
        let s = cranelift_shim_sig(kind);
        let id = declare_fn(&mut om, name, Linkage::Export, &s)?;
        define_fn(&mut om, id, s, |om, fb, _| {
            let base = state_ptr(om, state, fb);
            let sid = fb.ins().iconst(types::I32, std_id);
            let gsh = om.declare_func_in_func(win.get_std_handle, fb.func);
            let c = fb.ins().call(gsh, &[sid]);
            let raw = fb.inst_results(c)[0];
            let z64 = fb.ins().iconst(types::I64, 0);
            let inv = fb.ins().iconst(types::I64, -1);
            let is0 = fb.ins().icmp(IntCC::Equal, raw, z64);
            let ism1 = fb.ins().icmp(IntCC::Equal, raw, inv);
            let bad = fb.ins().bor(is0, ism1);
            let err_blk = fb.create_block();
            let ok_blk = fb.create_block();
            fb.ins().brif(bad, err_blk, &[], ok_blk, &[]);
            fb.seal_block(err_blk);
            fb.seal_block(ok_blk);

            fb.switch_to_block(err_blk);
            let gle = om.declare_func_in_func(win.get_last_error, fb.func);
            let gc = fb.ins().call(gle, &[]);
            let ec = fb.inst_results(gc)[0];
            let z32 = fb.ins().iconst(types::I32, 0);
            let ez = fb.ins().icmp(IntCC::Equal, ec, z32);
            let def_e = fb.ins().iconst(types::I32, 6);
            let rec_e = fb.ins().select(ez, def_e, ec);
            fb.ins().store(flags, rec_e, base, H_LAST_OS_ERROR);
            ret_err(fb, ERR_OS_FAILED);

            fb.switch_to_block(ok_blk);
            let z32 = fb.ins().iconst(types::I32, 0);
            fb.ins().store(flags, z32, base, H_LAST_OS_ERROR);
            let alloc = om.declare_func_in_func(loc.alloc_slot, fb.func);
            let k = fb.ins().iconst(types::I32, KIND_STREAM);
            let ac = fb.ins().call(alloc, &[k, raw, z64, z64]);
            let h = fb.inst_results(ac)[0];
            fb.ins().return_(&[h]);
        })
    };

    emit_std_query(PlatformOpKind::IoStdout, -11)?;
    emit_std_query(PlatformOpKind::IoStderr, -12)?;

    // IoWriteSome(h, ptr, len) -> write_slice(h, ptr, len, 0, len)
    {
        let s = cranelift_shim_sig(PlatformOpKind::IoWriteSome);
        let id = declare_fn(
            &mut om,
            shim_symbol_for(PlatformOpKind::IoWriteSome),
            Linkage::Export,
            &s,
        )?;
        define_fn(&mut om, id, s, |om, fb, entry| {
            let h = fb.block_params(entry)[0];
            let ptr = fb.block_params(entry)[1];
            let len = fb.block_params(entry)[2];
            let z32 = fb.ins().iconst(types::I32, 0);
            let ws = om.declare_func_in_func(loc.write_slice, fb.func);
            let c = fb.ins().call(ws, &[h, ptr, len, z32, len]);
            let r = fb.inst_results(c)[0];
            fb.ins().return_(&[r]);
        })?;
    }

    // IoWriteSomeSlice(h, ptr, slen, off, req_len)
    {
        let s = cranelift_shim_sig(PlatformOpKind::IoWriteSomeSlice);
        let id = declare_fn(
            &mut om,
            shim_symbol_for(PlatformOpKind::IoWriteSomeSlice),
            Linkage::Export,
            &s,
        )?;
        define_fn(&mut om, id, s, |om, fb, entry| {
            let h = fb.block_params(entry)[0];
            let ptr = fb.block_params(entry)[1];
            let slen = fb.block_params(entry)[2];
            let off = fb.block_params(entry)[3];
            let rlen = fb.block_params(entry)[4];
            let ws = om.declare_func_in_func(loc.write_slice, fb.func);
            let c = fb.ins().call(ws, &[h, ptr, slen, off, rlen]);
            let r = fb.inst_results(c)[0];
            fb.ins().return_(&[r]);
        })?;
    }

    // IoLastError()
    {
        let s = cranelift_shim_sig(PlatformOpKind::IoLastError);
        let id = declare_fn(
            &mut om,
            shim_symbol_for(PlatformOpKind::IoLastError),
            Linkage::Export,
            &s,
        )?;
        define_fn(&mut om, id, s, |om, fb, _| {
            let base = state_ptr(om, state, fb);
            let v = fb.ins().load(types::I32, flags, base, H_LAST_OS_ERROR);
            fb.ins().return_(&[v]);
        })?;
    }

    // IoConsoleKind(h) — PLAN-1041: GetConsoleMode classification.
    // ok!=0 -> 1 (text console); fail w/ GetLastError==5 -> record + -8;
    // any other failure -> 2 (byte sink; not recorded as an error).
    {
        let s = cranelift_shim_sig(PlatformOpKind::IoConsoleKind);
        let id = declare_fn(
            &mut om,
            shim_symbol_for(PlatformOpKind::IoConsoleKind),
            Linkage::Export,
            &s,
        )?;
        define_fn(&mut om, id, s, |om, fb, entry| {
            let h = fb.block_params(entry)[0];
            let z64 = fb.ins().iconst(types::I64, 0);
            let find = om.declare_func_in_func(loc.find_slot, fb.func);
            let exp_kind = fb.ins().iconst(types::I32, KIND_STREAM);
            let c = fb.ins().call(find, &[h, exp_kind]);
            let slot = fb.inst_results(c)[0];
            let neg = fb.ins().icmp(IntCC::SignedLessThan, slot, z64);
            let bad = fb.create_block();
            let ok_slot = fb.create_block();
            fb.ins().brif(neg, bad, &[], ok_slot, &[]);
            fb.seal_block(bad);
            fb.seal_block(ok_slot);

            fb.switch_to_block(bad);
            let s32 = fb.ins().ireduce(types::I32, slot);
            fb.ins().return_(&[s32]);

            fb.switch_to_block(ok_slot);
            let os_h = fb.ins().load(types::I64, flags, slot, S_OS_PTR);
            let st = state_ptr(om, state, fb);
            let tmp_off = fb.ins().iconst(types::I64, H_BYTES_WRITTEN_TMP as i64);
            let mode_ptr = fb.ins().iadd(st, tmp_off);
            let gcm = om.declare_func_in_func(win.get_console_mode, fb.func);
            let call = fb.ins().call(gcm, &[os_h, mode_ptr]);
            let ok = fb.inst_results(call)[0];
            let z32 = fb.ins().iconst(types::I32, 0);
            let is_ok = fb.ins().icmp(IntCC::NotEqual, ok, z32);
            let console = fb.create_block();
            let failed = fb.create_block();
            fb.ins().brif(is_ok, console, &[], failed, &[]);
            fb.seal_block(console);
            fb.seal_block(failed);

            fb.switch_to_block(console);
            let kind_text = fb.ins().iconst(types::I32, 1);
            fb.ins().return_(&[kind_text]);

            fb.switch_to_block(failed);
            let gle = om.declare_func_in_func(win.get_last_error, fb.func);
            let call = fb.ins().call(gle, &[]);
            let err = fb.inst_results(call)[0];
            let five = fb.ins().iconst(types::I32, 5);
            let denied = fb.ins().icmp(IntCC::Equal, err, five);
            let hard = fb.create_block();
            let sink = fb.create_block();
            fb.ins().brif(denied, hard, &[], sink, &[]);
            fb.seal_block(hard);
            fb.seal_block(sink);

            fb.switch_to_block(hard);
            fb.ins().store(flags, err, st, H_LAST_OS_ERROR);
            ret_err(fb, ERR_OS_FAILED);

            fb.switch_to_block(sink);
            let kind_sink = fb.ins().iconst(types::I32, 2);
            fb.ins().return_(&[kind_sink]);
        })?;
    }

    // IoConsoleWrite16(stream_h, region_h, unit_off, unit_len) — PLAN-1041:
    // WriteConsoleW over UTF-16LE units staged in an OS memory region.
    {
        let s = cranelift_shim_sig(PlatformOpKind::IoConsoleWrite16);
        let id = declare_fn(
            &mut om,
            shim_symbol_for(PlatformOpKind::IoConsoleWrite16),
            Linkage::Export,
            &s,
        )?;
        define_fn(&mut om, id, s, |om, fb, entry| {
            let sh = fb.block_params(entry)[0];
            let rh = fb.block_params(entry)[1];
            let uoff = fb.block_params(entry)[2];
            let ulen = fb.block_params(entry)[3];
            let z32 = fb.ins().iconst(types::I32, 0);
            let z64 = fb.ins().iconst(types::I64, 0);

            // negative offset/length -> ERR_INVALID_BOUNDS
            let o_neg = fb.ins().icmp(IntCC::SignedLessThan, uoff, z32);
            let l_neg = fb.ins().icmp(IntCC::SignedLessThan, ulen, z32);
            let any_neg = fb.ins().bor(o_neg, l_neg);
            let bad_bounds = fb.create_block();
            let cont1 = fb.create_block();
            fb.ins().brif(any_neg, bad_bounds, &[], cont1, &[]);
            fb.seal_block(bad_bounds);
            fb.seal_block(cont1);
            fb.switch_to_block(bad_bounds);
            ret_err(fb, ERR_INVALID_BOUNDS);
            fb.switch_to_block(cont1);

            // stream slot (StdStream)
            let find = om.declare_func_in_func(loc.find_slot, fb.func);
            let kind_stream = fb.ins().iconst(types::I32, KIND_STREAM);
            let cs = fb.ins().call(find, &[sh, kind_stream]);
            let sslot = fb.inst_results(cs)[0];
            let s_neg = fb.ins().icmp(IntCC::SignedLessThan, sslot, z64);
            let bad_slot = fb.create_block();
            let cont2 = fb.create_block();
            fb.ins().brif(s_neg, bad_slot, &[], cont2, &[]);
            fb.seal_block(bad_slot);
            fb.seal_block(cont2);
            fb.switch_to_block(bad_slot);
            let e1 = fb.ins().ireduce(types::I32, sslot);
            fb.ins().return_(&[e1]);
            fb.switch_to_block(cont2);

            // region slot (OsMemRegion)
            let kind_mem = fb.ins().iconst(types::I32, KIND_MEM);
            let cr = fb.ins().call(find, &[rh, kind_mem]);
            let rslot = fb.inst_results(cr)[0];
            let r_neg = fb.ins().icmp(IntCC::SignedLessThan, rslot, z64);
            let bad_rslot = fb.create_block();
            let cont3 = fb.create_block();
            fb.ins().brif(r_neg, bad_rslot, &[], cont3, &[]);
            fb.seal_block(bad_rslot);
            fb.seal_block(cont3);
            fb.switch_to_block(bad_rslot);
            let e2 = fb.ins().ireduce(types::I32, rslot);
            fb.ins().return_(&[e2]);
            fb.switch_to_block(cont3);

            let st = state_ptr(om, state, fb);

            // released region -> ERR_ALREADY_RELEASED
            let rel = fb.ins().load(types::I32, flags, rslot, S_RELEASED);
            let is_rel = fb.ins().icmp(IntCC::NotEqual, rel, z32);
            let was_rel = fb.create_block();
            let cont4 = fb.create_block();
            fb.ins().brif(is_rel, was_rel, &[], cont4, &[]);
            fb.seal_block(was_rel);
            fb.seal_block(cont4);
            fb.switch_to_block(was_rel);
            ret_err(fb, ERR_ALREADY_RELEASED);
            fb.switch_to_block(cont4);

            // bounds: 2*(uoff+ulen) <= logical_len (unsigned)
            let off64 = fb.ins().uextend(types::I64, uoff);
            let len64 = fb.ins().uextend(types::I64, ulen);
            let two = fb.ins().iconst(types::I64, 2);
            let boff = fb.ins().imul(off64, two);
            let blen = fb.ins().imul(len64, two);
            let bsum = fb.ins().iadd(boff, blen);
            let logical = fb.ins().load(types::I64, flags, rslot, S_LOGICAL_LEN);
            let fits = fb.ins().icmp(IntCC::UnsignedLessThanOrEqual, bsum, logical);
            let bad_b = fb.create_block();
            let cont5 = fb.create_block();
            fb.ins().brif(fits, cont5, &[], bad_b, &[]);
            fb.seal_block(bad_b);
            fb.seal_block(cont5);
            fb.switch_to_block(bad_b);
            ret_err(fb, ERR_INVALID_BOUNDS);
            fb.switch_to_block(cont5);

            // empty -> 0
            let is_empty = fb.ins().icmp(IntCC::Equal, ulen, z32);
            let empty = fb.create_block();
            let cont6 = fb.create_block();
            fb.ins().brif(is_empty, empty, &[], cont6, &[]);
            fb.seal_block(empty);
            fb.seal_block(cont6);
            fb.switch_to_block(empty);
            fb.ins().return_(&[z32]);
            fb.switch_to_block(cont6);

            // per-call budget (bytes = 2*ulen)
            let cap32 = fb.ins().iconst(types::I32, MAX_PLATFORM_CALL_BYTES as i64);
            let blen32 = fb.ins().ireduce(types::I32, blen);
            let over_call = fb.ins().icmp(IntCC::SignedGreaterThan, blen32, cap32);
            let bad_budget = fb.create_block();
            let cont7 = fb.create_block();
            fb.ins().brif(over_call, bad_budget, &[], cont7, &[]);
            fb.seal_block(bad_budget);
            fb.seal_block(cont7);
            fb.switch_to_block(bad_budget);
            ret_err(fb, ERR_BUDGET);
            fb.switch_to_block(cont7);

            // cumulative budget: written_total + blen <= MAX_RUN
            let wt = fb.ins().load(types::I64, flags, st, H_WRITTEN_TOTAL);
            let run_cap = fb.ins().iconst(types::I64, MAX_PLATFORM_RUN_BYTES as i64);
            let nwt = fb.ins().iadd(wt, blen);
            let over_run = fb.ins().icmp(IntCC::SignedGreaterThan, nwt, run_cap);
            let bad_run = fb.create_block();
            let cont8 = fb.create_block();
            fb.ins().brif(over_run, bad_run, &[], cont8, &[]);
            fb.seal_block(bad_run);
            fb.seal_block(cont8);
            fb.switch_to_block(bad_run);
            ret_err(fb, ERR_BUDGET);
            fb.switch_to_block(cont8);

            // units ptr = region base + 2*uoff; one WriteConsoleW call
            let base = fb.ins().load(types::I64, flags, rslot, S_OS_PTR);
            let uptr = fb.ins().iadd(base, boff);
            let os_h = fb.ins().load(types::I64, flags, sslot, S_OS_PTR);
            let tmp_off2 = fb.ins().iconst(types::I64, H_BYTES_WRITTEN_TMP as i64);
            let wptr = fb.ins().iadd(st, tmp_off2);
            let wcw = om.declare_func_in_func(win.write_console_w, fb.func);
            let call = fb.ins().call(wcw, &[os_h, uptr, ulen, wptr, z64]);
            let ok = fb.inst_results(call)[0];
            let okz = fb.ins().icmp(IntCC::Equal, ok, z32);
            let failed = fb.create_block();
            let done = fb.create_block();
            fb.ins().brif(okz, failed, &[], done, &[]);
            fb.seal_block(failed);
            fb.seal_block(done);

            fb.switch_to_block(failed);
            let gle = om.declare_func_in_func(win.get_last_error, fb.func);
            let call = fb.ins().call(gle, &[]);
            let err = fb.inst_results(call)[0];
            fb.ins().store(flags, err, st, H_LAST_OS_ERROR);
            ret_err(fb, ERR_OS_FAILED);

            fb.switch_to_block(done);
            let written = fb.ins().load(types::I32, flags, st, H_BYTES_WRITTEN_TMP);
            let capped = fb.ins().umin(written, ulen);
            let capped64 = fb.ins().uextend(types::I64, capped);
            let two2 = fb.ins().iconst(types::I64, 2);
            let bytes = fb.ins().imul(capped64, two2);
            let total = fb.ins().load(types::I64, flags, st, H_WRITTEN_TOTAL);
            let ntotal = fb.ins().iadd(total, bytes);
            fb.ins().store(flags, ntotal, st, H_WRITTEN_TOTAL);
            fb.ins().store(flags, z32, st, H_LAST_OS_ERROR);
            fb.ins().return_(&[capped]);
        })?;
    }

    // MemPageSize() & MemAllocGranularity()
    for (kind, field) in [
        (PlatformOpKind::MemPageSize, H_PAGE_SIZE),
        (PlatformOpKind::MemAllocGranularity, H_ALLOC_GRANULARITY),
    ] {
        let s = cranelift_shim_sig(kind);
        let id = declare_fn(&mut om, shim_symbol_for(kind), Linkage::Export, &s)?;
        define_fn(&mut om, id, s, |om, fb, _| {
            let si = om.declare_func_in_func(loc.state_sysinfo, fb.func);
            let c = fb.ins().call(si, &[]);
            let base = fb.inst_results(c)[0];
            let v = fb.ins().load(types::I32, flags, base, field);
            fb.ins().return_(&[v]);
        })?;
    }

    // MemAlloc(len: i32) -> handle or ERR_*
    {
        let s = cranelift_shim_sig(PlatformOpKind::MemAlloc);
        let id = declare_fn(
            &mut om,
            shim_symbol_for(PlatformOpKind::MemAlloc),
            Linkage::Export,
            &s,
        )?;
        define_fn(&mut om, id, s, |om, fb, entry| {
            let len = fb.block_params(entry)[0];
            let z32 = fb.ins().iconst(types::I32, 0);
            let le0 = fb.ins().icmp(IntCC::SignedLessThanOrEqual, len, z32);
            let bad_len = fb.create_block();
            let chk_ps = fb.create_block();
            fb.ins().brif(le0, bad_len, &[], chk_ps, &[]);
            fb.seal_block(bad_len);
            fb.seal_block(chk_ps);

            fb.switch_to_block(bad_len);
            ret_err(fb, ERR_INVALID_BOUNDS);

            fb.switch_to_block(chk_ps);
            let si = om.declare_func_in_func(loc.state_sysinfo, fb.func);
            let sc = fb.ins().call(si, &[]);
            let base = fb.inst_results(sc)[0];
            let ps32 = fb.ins().load(types::I32, flags, base, H_PAGE_SIZE);
            let ps_bad = fb.ins().icmp(IntCC::SignedLessThanOrEqual, ps32, z32);
            let bad_ps = fb.create_block();
            let chk_b = fb.create_block();
            fb.ins().brif(ps_bad, bad_ps, &[], chk_b, &[]);
            fb.seal_block(bad_ps);
            fb.seal_block(chk_b);

            fb.switch_to_block(bad_ps);
            ret_err(fb, ERR_OS_FAILED);

            fb.switch_to_block(chk_b);
            let logical = fb.ins().sextend(types::I64, len);
            let ps = fb.ins().sextend(types::I64, ps32);
            let one64 = fb.ins().iconst(types::I64, 1);
            let ps_m1 = fb.ins().isub(ps, one64);
            let sum = fb.ins().iadd(logical, ps_m1);
            let mask = fb.ins().bnot(ps_m1);
            let aligned = fb.ins().band(sum, mask);

            let live = fb.ins().load(types::I64, flags, base, H_LIVE_MAPPED);
            let next_live = fb.ins().iadd(live, aligned);
            let max_live = fb.ins().iconst(types::I64, MAX_OS_REGION_BYTES as i64);
            let over_mem = fb.ins().icmp(IntCC::SignedGreaterThan, next_live, max_live);
            let num = fb.ins().load(types::I32, flags, base, H_NUM_SLOTS);
            let cap = fb.ins().iconst(types::I32, MAX_NATIVE_SLOTS as i64);
            let over_slots = fb.ins().icmp(IntCC::SignedGreaterThanOrEqual, num, cap);
            let over_b = fb.ins().bor(over_mem, over_slots);
            let bad_bud = fb.create_block();
            let do_va = fb.create_block();
            fb.ins().brif(over_b, bad_bud, &[], do_va, &[]);
            fb.seal_block(bad_bud);
            fb.seal_block(do_va);

            fb.switch_to_block(bad_bud);
            ret_err(fb, ERR_BUDGET);

            fb.switch_to_block(do_va);
            let z64 = fb.ins().iconst(types::I64, 0);
            let ftype = fb.ins().iconst(types::I32, MEM_COMMIT_RESERVE);
            let fprot = fb.ins().iconst(types::I32, PAGE_READWRITE);
            let va = om.declare_func_in_func(win.virtual_alloc, fb.func);
            let vc = fb.ins().call(va, &[z64, aligned, ftype, fprot]);
            let ptr = fb.inst_results(vc)[0];
            let null_p = fb.ins().icmp(IntCC::Equal, ptr, z64);
            let err_os = fb.create_block();
            let ok_os = fb.create_block();
            fb.ins().brif(null_p, err_os, &[], ok_os, &[]);
            fb.seal_block(err_os);
            fb.seal_block(ok_os);

            fb.switch_to_block(err_os);
            let gle = om.declare_func_in_func(win.get_last_error, fb.func);
            let gc = fb.ins().call(gle, &[]);
            let ec = fb.inst_results(gc)[0];
            let ez = fb.ins().icmp(IntCC::Equal, ec, z32);
            let def_e = fb.ins().iconst(types::I32, 8);
            let rec_e = fb.ins().select(ez, def_e, ec);
            fb.ins().store(flags, rec_e, base, H_LAST_OS_ERROR);
            ret_err(fb, ERR_OS_FAILED);

            fb.switch_to_block(ok_os);
            let alloc = om.declare_func_in_func(loc.alloc_slot, fb.func);
            let k = fb.ins().iconst(types::I32, KIND_MEM);
            let ac = fb.ins().call(alloc, &[k, ptr, logical, aligned]);
            let h = fb.inst_results(ac)[0];
            fb.ins().store(flags, next_live, base, H_LIVE_MAPPED);
            fb.ins().store(flags, z32, base, H_LAST_OS_ERROR);
            fb.ins().return_(&[h]);
        })?;
    }

    // MemFree(handle: i32) -> 0 or ERR_*
    {
        let s = cranelift_shim_sig(PlatformOpKind::MemFree);
        let id = declare_fn(
            &mut om,
            shim_symbol_for(PlatformOpKind::MemFree),
            Linkage::Export,
            &s,
        )?;
        define_fn(&mut om, id, s, |om, fb, entry| {
            let h = fb.block_params(entry)[0];
            let find = om.declare_func_in_func(loc.find_slot, fb.func);
            let k = fb.ins().iconst(types::I32, KIND_MEM);
            let fc = fb.ins().call(find, &[h, k]);
            let slot = fb.inst_results(fc)[0];
            let z64 = fb.ins().iconst(types::I64, 0);
            let neg = fb.ins().icmp(IntCC::SignedLessThan, slot, z64);
            let bad_slot = fb.create_block();
            let chk_bor = fb.create_block();
            fb.ins().brif(neg, bad_slot, &[], chk_bor, &[]);
            fb.seal_block(bad_slot);
            fb.seal_block(chk_bor);

            fb.switch_to_block(bad_slot);
            let s32 = fb.ins().ireduce(types::I32, slot);
            fb.ins().return_(&[s32]);

            fb.switch_to_block(chk_bor);
            let z32 = fb.ins().iconst(types::I32, 0);
            let bc = fb.ins().load(types::I32, flags, slot, S_BORROW);
            let borrowed = fb.ins().icmp(IntCC::SignedGreaterThan, bc, z32);
            let bad_bor = fb.create_block();
            let do_vf = fb.create_block();
            fb.ins().brif(borrowed, bad_bor, &[], do_vf, &[]);
            fb.seal_block(bad_bor);
            fb.seal_block(do_vf);

            fb.switch_to_block(bad_bor);
            ret_err(fb, ERR_BORROWED);

            fb.switch_to_block(do_vf);
            let ptr = fb.ins().load(types::I64, flags, slot, S_OS_PTR);
            let mapped = fb.ins().load(types::I64, flags, slot, S_MAPPED_LEN);
            let rel_ty = fb.ins().iconst(types::I32, MEM_RELEASE);
            let vf = om.declare_func_in_func(win.virtual_free, fb.func);
            let vc = fb.ins().call(vf, &[ptr, z64, rel_ty]);
            let ok = fb.inst_results(vc)[0];
            let failed = fb.ins().icmp(IntCC::Equal, ok, z32);
            let err_os = fb.create_block();
            let ok_os = fb.create_block();
            fb.ins().brif(failed, err_os, &[], ok_os, &[]);
            fb.seal_block(err_os);
            fb.seal_block(ok_os);

            fb.switch_to_block(err_os);
            let base = state_ptr(om, state, fb);
            let gle = om.declare_func_in_func(win.get_last_error, fb.func);
            let gc = fb.ins().call(gle, &[]);
            let ec = fb.inst_results(gc)[0];
            let ez = fb.ins().icmp(IntCC::Equal, ec, z32);
            let def_e = fb.ins().iconst(types::I32, 87);
            let rec_e = fb.ins().select(ez, def_e, ec);
            fb.ins().store(flags, rec_e, base, H_LAST_OS_ERROR);
            ret_err(fb, ERR_OS_FAILED);

            fb.switch_to_block(ok_os);
            let base = state_ptr(om, state, fb);
            let one32 = fb.ins().iconst(types::I32, 1);
            fb.ins().store(flags, one32, slot, S_RELEASED);
            let live = fb.ins().load(types::I64, flags, base, H_LIVE_MAPPED);
            let next_live = fb.ins().isub(live, mapped);
            fb.ins().store(flags, next_live, base, H_LIVE_MAPPED);
            fb.ins().store(flags, z32, base, H_LAST_OS_ERROR);
            fb.ins().return_(&[z32]);
        })?;
    }

    // MemLogicalLen / MemMappedLen
    for (kind, field) in [
        (PlatformOpKind::MemLogicalLen, S_LOGICAL_LEN),
        (PlatformOpKind::MemMappedLen, S_MAPPED_LEN),
    ] {
        let s = cranelift_shim_sig(kind);
        let id = declare_fn(&mut om, shim_symbol_for(kind), Linkage::Export, &s)?;
        define_fn(&mut om, id, s, |om, fb, entry| {
            let h = fb.block_params(entry)[0];
            let find = om.declare_func_in_func(loc.find_slot, fb.func);
            let k = fb.ins().iconst(types::I32, KIND_MEM);
            let fc = fb.ins().call(find, &[h, k]);
            let slot = fb.inst_results(fc)[0];
            let z64 = fb.ins().iconst(types::I64, 0);
            let neg = fb.ins().icmp(IntCC::SignedLessThan, slot, z64);
            let bad = fb.create_block();
            let ok = fb.create_block();
            fb.ins().brif(neg, bad, &[], ok, &[]);
            fb.seal_block(bad);
            fb.seal_block(ok);

            fb.switch_to_block(bad);
            let e = fb.ins().ireduce(types::I32, slot);
            fb.ins().return_(&[e]);

            fb.switch_to_block(ok);
            let len64 = fb.ins().load(types::I64, flags, slot, field);
            let len32 = fb.ins().ireduce(types::I32, len64);
            fb.ins().return_(&[len32]);
        })?;
    }

    // MemBorrow / MemUnborrow
    for (kind, acquire) in [
        (PlatformOpKind::MemBorrow, true),
        (PlatformOpKind::MemUnborrow, false),
    ] {
        let s = cranelift_shim_sig(kind);
        let id = declare_fn(&mut om, shim_symbol_for(kind), Linkage::Export, &s)?;
        define_fn(&mut om, id, s, |om, fb, entry| {
            let h = fb.block_params(entry)[0];
            let find = om.declare_func_in_func(loc.find_slot, fb.func);
            let k = fb.ins().iconst(types::I32, KIND_MEM);
            let fc = fb.ins().call(find, &[h, k]);
            let slot = fb.inst_results(fc)[0];
            let z64 = fb.ins().iconst(types::I64, 0);
            let neg = fb.ins().icmp(IntCC::SignedLessThan, slot, z64);
            let bad = fb.create_block();
            let ok = fb.create_block();
            fb.ins().brif(neg, bad, &[], ok, &[]);
            fb.seal_block(bad);
            fb.seal_block(ok);

            fb.switch_to_block(bad);
            let e = fb.ins().ireduce(types::I32, slot);
            fb.ins().return_(&[e]);

            fb.switch_to_block(ok);
            let bc = fb.ins().load(types::I32, flags, slot, S_BORROW);
            let one32 = fb.ins().iconst(types::I32, 1);
            if acquire {
                let next = fb.ins().iadd(bc, one32);
                fb.ins().store(flags, next, slot, S_BORROW);
                fb.ins().return_(&[next]);
            } else {
                let z32 = fb.ins().iconst(types::I32, 0);
                let empty = fb.ins().icmp(IntCC::SignedLessThanOrEqual, bc, z32);
                let uflow = fb.create_block();
                let dec = fb.create_block();
                fb.ins().brif(empty, uflow, &[], dec, &[]);
                fb.seal_block(uflow);
                fb.seal_block(dec);

                fb.switch_to_block(uflow);
                ret_err(fb, ERR_INVALID_BOUNDS);

                fb.switch_to_block(dec);
                let prev = fb.ins().isub(bc, one32);
                fb.ins().store(flags, prev, slot, S_BORROW);
                fb.ins().return_(&[prev]);
            }
        })?;
    }

    // MemWriteU8(handle, offset, byte) -> 0 or ERR_*
    {
        let s = cranelift_shim_sig(PlatformOpKind::MemWriteU8);
        let id = declare_fn(
            &mut om,
            shim_symbol_for(PlatformOpKind::MemWriteU8),
            Linkage::Export,
            &s,
        )?;
        define_fn(&mut om, id, s, |om, fb, entry| {
            let h = fb.block_params(entry)[0];
            let off = fb.block_params(entry)[1];
            let byte = fb.block_params(entry)[2];
            let z32 = fb.ins().iconst(types::I32, 0);
            let max_b = fb.ins().iconst(types::I32, 255);
            let lo = fb.ins().icmp(IntCC::SignedLessThan, byte, z32);
            let hi = fb.ins().icmp(IntCC::SignedGreaterThan, byte, max_b);
            let bad_val = fb.ins().bor(lo, hi);
            let err_val = fb.create_block();
            let chk_slot = fb.create_block();
            fb.ins().brif(bad_val, err_val, &[], chk_slot, &[]);
            fb.seal_block(err_val);
            fb.seal_block(chk_slot);

            fb.switch_to_block(err_val);
            ret_err(fb, ERR_INVALID_VALUE);

            fb.switch_to_block(chk_slot);
            let find = om.declare_func_in_func(loc.find_slot, fb.func);
            let k = fb.ins().iconst(types::I32, KIND_MEM);
            let fc = fb.ins().call(find, &[h, k]);
            let slot = fb.inst_results(fc)[0];
            let z64 = fb.ins().iconst(types::I64, 0);
            let neg = fb.ins().icmp(IntCC::SignedLessThan, slot, z64);
            let bad_slot = fb.create_block();
            let chk_oob = fb.create_block();
            fb.ins().brif(neg, bad_slot, &[], chk_oob, &[]);
            fb.seal_block(bad_slot);
            fb.seal_block(chk_oob);

            fb.switch_to_block(bad_slot);
            let e = fb.ins().ireduce(types::I32, slot);
            fb.ins().return_(&[e]);

            fb.switch_to_block(chk_oob);
            let logical = fb.ins().load(types::I64, flags, slot, S_LOGICAL_LEN);
            let off64 = fb.ins().sextend(types::I64, off);
            let oob1 = fb.ins().icmp(IntCC::SignedLessThan, off, z32);
            let oob2 = fb
                .ins()
                .icmp(IntCC::SignedGreaterThanOrEqual, off64, logical);
            let oob = fb.ins().bor(oob1, oob2);
            let bad_oob = fb.create_block();
            let do_wr = fb.create_block();
            fb.ins().brif(oob, bad_oob, &[], do_wr, &[]);
            fb.seal_block(bad_oob);
            fb.seal_block(do_wr);

            fb.switch_to_block(bad_oob);
            ret_err(fb, ERR_INVALID_BOUNDS);

            fb.switch_to_block(do_wr);
            let base_ptr = fb.ins().load(types::I64, flags, slot, S_OS_PTR);
            let addr = fb.ins().iadd(base_ptr, off64);
            let b8 = fb.ins().ireduce(types::I8, byte);
            fb.ins().store(flags, b8, addr, 0);
            fb.ins().return_(&[z32]);
        })?;
    }

    // MemReadU8(handle, offset) -> 0..=255 or ERR_*
    {
        let s = cranelift_shim_sig(PlatformOpKind::MemReadU8);
        let id = declare_fn(
            &mut om,
            shim_symbol_for(PlatformOpKind::MemReadU8),
            Linkage::Export,
            &s,
        )?;
        define_fn(&mut om, id, s, |om, fb, entry| {
            let h = fb.block_params(entry)[0];
            let off = fb.block_params(entry)[1];
            let z32 = fb.ins().iconst(types::I32, 0);
            let find = om.declare_func_in_func(loc.find_slot, fb.func);
            let k = fb.ins().iconst(types::I32, KIND_MEM);
            let fc = fb.ins().call(find, &[h, k]);
            let slot = fb.inst_results(fc)[0];
            let z64 = fb.ins().iconst(types::I64, 0);
            let neg = fb.ins().icmp(IntCC::SignedLessThan, slot, z64);
            let bad_slot = fb.create_block();
            let chk_oob = fb.create_block();
            fb.ins().brif(neg, bad_slot, &[], chk_oob, &[]);
            fb.seal_block(bad_slot);
            fb.seal_block(chk_oob);

            fb.switch_to_block(bad_slot);
            let e = fb.ins().ireduce(types::I32, slot);
            fb.ins().return_(&[e]);

            fb.switch_to_block(chk_oob);
            let logical = fb.ins().load(types::I64, flags, slot, S_LOGICAL_LEN);
            let off64 = fb.ins().sextend(types::I64, off);
            let oob1 = fb.ins().icmp(IntCC::SignedLessThan, off, z32);
            let oob2 = fb
                .ins()
                .icmp(IntCC::SignedGreaterThanOrEqual, off64, logical);
            let oob = fb.ins().bor(oob1, oob2);
            let bad_oob = fb.create_block();
            let do_rd = fb.create_block();
            fb.ins().brif(oob, bad_oob, &[], do_rd, &[]);
            fb.seal_block(bad_oob);
            fb.seal_block(do_rd);

            fb.switch_to_block(bad_oob);
            ret_err(fb, ERR_INVALID_BOUNDS);

            fb.switch_to_block(do_rd);
            let base_ptr = fb.ins().load(types::I64, flags, slot, S_OS_PTR);
            let addr = fb.ins().iadd(base_ptr, off64);
            let val = fb.ins().uload8(types::I32, flags, addr, 0);
            fb.ins().return_(&[val]);
        })?;
    }

    let product = om.finish();
    let bytes = product
        .emit()
        .map_err(|e| MnError::Backend(format!("emit platform shim coff: {e}")))?;
    Ok(bytes)
}

/// PLAN-1040: the shared slot-lookup body (identical ledger semantics on
/// the Win64 COFF and Linux ELF shims). Returns the slot pointer (> 0) or
/// the negative ERR_* sign-extended to i64.
pub(crate) fn emit_find_slot_body(
    om: &mut ObjectModule,
    fb: &mut FunctionBuilder<'_>,
    entry: cranelift_codegen::ir::Block,
    state: cranelift_module::DataId,
    flags: MemFlags,
) {
    let handle = fb.block_params(entry)[0];
    let expected_kind = fb.block_params(entry)[1];
    let base = state_ptr(om, state, fb);
    let bad_h = fb.ins().iconst(types::I64, ERR_INVALID_HANDLE as i64);
    let bad_o = fb.ins().iconst(types::I64, ERR_WRONG_ORIGIN as i64);
    let bad_r = fb.ins().iconst(types::I64, ERR_ALREADY_RELEASED as i64);

    let zero32 = fb.ins().iconst(types::I32, 0);
    let le_zero = fb.ins().icmp(IntCC::SignedLessThanOrEqual, handle, zero32);
    let err_h = fb.create_block();
    let chk_idx = fb.create_block();
    fb.ins().brif(le_zero, err_h, &[], chk_idx, &[]);
    fb.seal_block(chk_idx);

    fb.switch_to_block(chk_idx);
    let mask16 = fb.ins().iconst(types::I32, 0xFFFF);
    let low16 = fb.ins().band(handle, mask16);
    let one32 = fb.ins().iconst(types::I32, 1);
    let idx = fb.ins().isub(low16, one32);
    let shifted = fb.ins().ushr_imm(handle, 16);
    let mask15 = fb.ins().iconst(types::I32, 0x7FFF);
    let gen = fb.ins().band(shifted, mask15);

    let num = fb.ins().load(types::I32, flags, base, H_NUM_SLOTS);
    let neg_idx = fb.ins().icmp(IntCC::SignedLessThan, idx, zero32);
    let oob_idx = fb.ins().icmp(IntCC::SignedGreaterThanOrEqual, idx, num);
    let bad_idx = fb.ins().bor(neg_idx, oob_idx);
    let chk_slot = fb.create_block();
    fb.ins().brif(bad_idx, err_h, &[], chk_slot, &[]);
    fb.seal_block(chk_slot);

    fb.switch_to_block(chk_slot);
    let idx64 = fb.ins().sextend(types::I64, idx);
    let off64 = fb.ins().ishl_imm(idx64, 6); // * 64
    let hdr = fb.ins().iconst(types::I64, STATE_HEADER_BYTES as i64);
    let slot_off = fb.ins().iadd(off64, hdr);
    let slot = fb.ins().iadd(base, slot_off);

    let skind = fb.ins().load(types::I32, flags, slot, S_KIND);
    let sgen = fb.ins().load(types::I32, flags, slot, S_GEN);
    let empty = fb.ins().icmp(IntCC::Equal, skind, zero32);
    let gen_mismatch = fb.ins().icmp(IntCC::NotEqual, sgen, gen);
    let bad_meta = fb.ins().bor(empty, gen_mismatch);
    let chk_origin = fb.create_block();
    fb.ins().brif(bad_meta, err_h, &[], chk_origin, &[]);
    fb.seal_block(err_h);
    fb.seal_block(chk_origin);

    fb.switch_to_block(err_h);
    fb.ins().return_(&[bad_h]);

    fb.switch_to_block(chk_origin);
    let kind_mismatch = fb.ins().icmp(IntCC::NotEqual, skind, expected_kind);
    let err_o = fb.create_block();
    let chk_rel = fb.create_block();
    fb.ins().brif(kind_mismatch, err_o, &[], chk_rel, &[]);
    fb.seal_block(err_o);
    fb.seal_block(chk_rel);

    fb.switch_to_block(err_o);
    fb.ins().return_(&[bad_o]);

    fb.switch_to_block(chk_rel);
    let srel = fb.ins().load(types::I32, flags, slot, S_RELEASED);
    let is_rel = fb.ins().icmp(IntCC::NotEqual, srel, zero32);
    let err_r = fb.create_block();
    let ok_blk = fb.create_block();
    fb.ins().brif(is_rel, err_r, &[], ok_blk, &[]);
    fb.seal_block(err_r);
    fb.seal_block(ok_blk);

    fb.switch_to_block(err_r);
    fb.ins().return_(&[bad_r]);

    fb.switch_to_block(ok_blk);
    fb.ins().return_(&[slot]);
}

/// PLAN-1040: the shared slot-allocation body (identical ledger semantics
/// on the Win64 COFF and Linux ELF shims).
pub(crate) fn emit_alloc_slot_body(
    om: &mut ObjectModule,
    fb: &mut FunctionBuilder<'_>,
    entry: cranelift_codegen::ir::Block,
    state: cranelift_module::DataId,
    flags: MemFlags,
) {
    let kind = fb.block_params(entry)[0];
    let os_ptr = fb.block_params(entry)[1];
    let logical = fb.block_params(entry)[2];
    let mapped = fb.block_params(entry)[3];
    let base = state_ptr(om, state, fb);
    let num = fb.ins().load(types::I32, flags, base, H_NUM_SLOTS);
    let cap = fb.ins().iconst(types::I32, MAX_NATIVE_SLOTS as i64);
    let full = fb.ins().icmp(IntCC::SignedGreaterThanOrEqual, num, cap);
    let err_blk = fb.create_block();
    let ok_blk = fb.create_block();
    fb.ins().brif(full, err_blk, &[], ok_blk, &[]);
    fb.seal_block(err_blk);
    fb.seal_block(ok_blk);

    fb.switch_to_block(err_blk);
    ret_err(fb, ERR_BUDGET);

    fb.switch_to_block(ok_blk);
    let zero32 = fb.ins().iconst(types::I32, 0);
    let one32 = fb.ins().iconst(types::I32, 1);
    let g0 = fb.ins().load(types::I32, flags, base, H_NEXT_GEN);
    let is_z = fb.ins().icmp(IntCC::Equal, g0, zero32);
    let g = fb.ins().select(is_z, one32, g0);
    let g1 = fb.ins().iadd(g, one32);
    let m15 = fb.ins().iconst(types::I32, 0x7FFF);
    let ng0 = fb.ins().band(g1, m15);
    let ng_z = fb.ins().icmp(IntCC::Equal, ng0, zero32);
    let ng = fb.ins().select(ng_z, one32, ng0);
    fb.ins().store(flags, ng, base, H_NEXT_GEN);

    let num64 = fb.ins().sextend(types::I64, num);
    let off64 = fb.ins().ishl_imm(num64, 6);
    let hdr = fb.ins().iconst(types::I64, STATE_HEADER_BYTES as i64);
    let slot_off = fb.ins().iadd(off64, hdr);
    let slot = fb.ins().iadd(base, slot_off);
    fb.ins().store(flags, kind, slot, S_KIND);
    fb.ins().store(flags, zero32, slot, S_RELEASED);
    fb.ins().store(flags, g, slot, S_GEN);
    fb.ins().store(flags, zero32, slot, S_BORROW);
    fb.ins().store(flags, os_ptr, slot, S_OS_PTR);
    fb.ins().store(flags, logical, slot, S_LOGICAL_LEN);
    fb.ins().store(flags, mapped, slot, S_MAPPED_LEN);

    let next_num = fb.ins().iadd(num, one32);
    fb.ins().store(flags, next_num, base, H_NUM_SLOTS);
    let hi = fb.ins().ishl_imm(g, 16);
    let h = fb.ins().bor(hi, next_num);
    fb.ins().return_(&[h]);
}

/// Allowed PE import symbols for the checked Win64 platform I/O lane (zero libc/CRT).
pub const ALLOWED_KERNEL32_IMPORTS: &[&str] = &[
    "ExitProcess",
    "GetStdHandle",
    "WriteFile",
    "GetLastError",
    "GetSystemInfo",
    "VirtualAlloc",
    "VirtualFree",
    // PLAN-1041: Windows text-console classification + Unicode write.
    "GetConsoleMode",
    "WriteConsoleW",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeImportReport {
    pub dlls: Vec<String>,
    pub symbols: Vec<(String, String)>,
}

/// Inspect a linked Win64 PE executable and reject ANY non-kernel32 DLL
/// (such as `msvcrt.dll`, `ucrtbase.dll`, `vcruntime*.dll`, `api-ms-win-crt-*`)
/// or unapproved / CRT-delegated import symbol (`malloc`, `free`, `puts`, `printf`, etc.).
pub fn audit_pe_imports(pe_bytes: &[u8]) -> Result<PeImportReport, MnError> {
    use cranelift_object::object::pe::{ImageNtHeaders64, IMAGE_DIRECTORY_ENTRY_IMPORT};
    use cranelift_object::object::read::pe::{ImageThunkData, PeFile64};
    use cranelift_object::object::LittleEndian as LE;

    let pe =
        PeFile64::parse(pe_bytes).map_err(|e| MnError::Infra(format!("pe.audit.parse: {e}")))?;
    let data_dirs = pe.data_directories();
    let import_dir = data_dirs
        .get(IMAGE_DIRECTORY_ENTRY_IMPORT)
        .ok_or_else(|| MnError::Infra("pe.audit.no-import-table".into()))?;
    if import_dir.virtual_address.get(LE) == 0 {
        return Err(MnError::Infra("pe.audit.empty-import-table".into()));
    }
    let table = pe
        .import_table()
        .map_err(|e| MnError::Infra(format!("pe.audit.import-table: {e}")))?
        .ok_or_else(|| MnError::Infra("pe.audit.missing-import-table".into()))?;

    let mut dlls = Vec::new();
    let mut symbols = Vec::new();
    let mut descs = table
        .descriptors()
        .map_err(|e| MnError::Infra(format!("pe.audit.descriptors: {e}")))?;
    while let Some(desc) = descs
        .next()
        .map_err(|e| MnError::Infra(format!("pe.audit.descriptor: {e}")))?
    {
        let raw_dll = table
            .name(desc.name.get(LE))
            .map_err(|e| MnError::Infra(format!("pe.audit.dll-name: {e}")))?;
        let dll = String::from_utf8_lossy(raw_dll).into_owned();
        if !dll.eq_ignore_ascii_case("kernel32.dll") {
            return Err(MnError::Unsupported(
                "pe.audit.forbidden-dll",
                format!("unexpected DLL `{dll}` in native platform-io executable (expected kernel32.dll only)"),
            ));
        }
        dlls.push(dll.clone());

        let ilt = desc.original_first_thunk.get(LE);
        let thunk_rva = if ilt != 0 {
            ilt
        } else {
            desc.first_thunk.get(LE)
        };
        let mut thunks = table
            .thunks(thunk_rva)
            .map_err(|e| MnError::Infra(format!("pe.audit.thunks: {e}")))?;
        while let Some(thunk) = thunks
            .next::<ImageNtHeaders64>()
            .map_err(|e| MnError::Infra(format!("pe.audit.thunk: {e}")))?
        {
            if thunk.is_ordinal() {
                return Err(MnError::Unsupported(
                    "pe.audit.ordinal-import",
                    format!("ordinal import from `{dll}` is forbidden in platform-io audit"),
                ));
            }
            let (_, name_bytes) = table
                .hint_name(thunk.address())
                .map_err(|e| MnError::Infra(format!("pe.audit.hint-name: {e}")))?;
            let sym = String::from_utf8_lossy(name_bytes).into_owned();
            if !ALLOWED_KERNEL32_IMPORTS.contains(&sym.as_str()) {
                return Err(MnError::Unsupported(
                    "pe.audit.forbidden-symbol",
                    format!("unexpected Win32 import `{sym}` from `{dll}`"),
                ));
            }
            symbols.push((dll.clone(), sym));
        }
    }
    Ok(PeImportReport { dlls, symbols })
}
