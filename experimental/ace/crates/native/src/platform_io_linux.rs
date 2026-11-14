//! PLAN-1040: the Linux x86-64 ELF platform shim (`mn-platform-shim-elf`)
//! and the linked-ELF dependency audit.
//!
//! Mirrors the PLAN-1039 Win64 shim's ledger semantics slot for slot
//! (same `mn_pio_state` layout, same checked negative `ERR_*` codes,
//! same generation-encoded handles) but performs the OS boundary through
//! the raw `mn_syscall*` stubs of the linux support object — `write`
//! (1), `close` (3), `mmap` (9), `munmap` (11), `openat` (257) — with
//! ZERO libc/CRT: no imports but the stubs, page size from the
//! `_start`-filled `mn_pagesz` auxv slot. The probe seam
//! (`mn_pio_probe_*`) returns RAW syscall results (positive fd/count,
//! negative errno) — classification is the guest's oracle.

use crate::config::{LINUX_PAGESZ_SYMBOL, MAX_COFF_BYTES};
use crate::emit::make_isa_for;
use crate::mn_target::MnOsTarget;
use crate::platform_io::{
    cranelift_shim_sig_for, declare_fn, define_fn, ret_err, shim_symbol_for, H_LAST_OS_ERROR,
    H_LIVE_MAPPED, H_NUM_SLOTS, H_WRITTEN_TOTAL, KIND_MEM, KIND_STREAM, MAX_NATIVE_SLOTS,
    STATE_BYTES, S_BORROW, S_LOGICAL_LEN, S_MAPPED_LEN, S_OS_PTR, S_RELEASED,
};
use crate::MnError;
use ace_av_core::platform_io::{
    linux_sysno, PlatformOpKind, ERR_BORROWED, ERR_BUDGET, ERR_INVALID_BOUNDS, ERR_INVALID_VALUE,
    ERR_OS_FAILED, MAX_OS_REGION_BYTES, MAX_PLATFORM_CALL_BYTES, MAX_PLATFORM_RUN_BYTES,
    MAX_PROBE_PATH_BYTES, PROBE_AT_FDCWD,
};
use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::ir::{types, AbiParam, InstBuilder, MemFlags, Signature};
use cranelift_frontend::FunctionBuilder;
use cranelift_module::{default_libcall_names, DataDescription, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};

const CONV: cranelift_codegen::isa::CallConv = cranelift_codegen::isa::CallConv::SystemV;

// Linux x86-64 UAPI constants of this shim.
const PROT_READ_WRITE: i64 = 0x3;
const MAP_PRIVATE_ANONYMOUS: i64 = 0x22;
const MAP_FD_NONE: i64 = -1;
const O_CLOEXEC: i64 = 0o2000000;

fn sig(params: &[types::Type], ret: Option<types::Type>) -> Signature {
    Signature {
        call_conv: CONV,
        params: params.iter().copied().map(AbiParam::new).collect(),
        returns: ret.into_iter().map(AbiParam::new).collect(),
    }
}

#[derive(Clone, Copy)]
struct SyscallIds {
    s1: cranelift_module::FuncId,
    s2: cranelift_module::FuncId,
    s3: cranelift_module::FuncId,
    s4: cranelift_module::FuncId,
    s6: cranelift_module::FuncId,
}

impl SyscallIds {
    fn declare(om: &mut ObjectModule) -> Result<Self, MnError> {
        // Each stub takes (nr, a1..aN) — one leading nr parameter more
        // than its syscall-arg count.
        let two = &[types::I64; 2]; // syscall1
        let three = &[types::I64; 3]; // syscall2
        let four = &[types::I64; 4]; // syscall3
        let five = &[types::I64; 5]; // syscall4
        let seven = &[types::I64; 7]; // syscall6
        Ok(Self {
            s1: declare_fn(
                om,
                "mn_syscall1",
                Linkage::Import,
                &sig(two, Some(types::I64)),
            )?,
            s2: declare_fn(
                om,
                "mn_syscall2",
                Linkage::Import,
                &sig(three, Some(types::I64)),
            )?,
            s3: declare_fn(
                om,
                "mn_syscall3",
                Linkage::Import,
                &sig(four, Some(types::I64)),
            )?,
            s4: declare_fn(
                om,
                "mn_syscall4",
                Linkage::Import,
                &sig(five, Some(types::I64)),
            )?,
            s6: declare_fn(
                om,
                "mn_syscall6",
                Linkage::Import,
                &sig(seven, Some(types::I64)),
            )?,
        })
    }
}

#[derive(Clone, Copy)]
struct LocalIds {
    find_slot: cranelift_module::FuncId,
    alloc_slot: cranelift_module::FuncId,
    write_slice: cranelift_module::FuncId,
}

fn state_ptr(
    om: &mut ObjectModule,
    state: cranelift_module::DataId,
    fb: &mut FunctionBuilder<'_>,
) -> cranelift_codegen::ir::Value {
    let gv = om.declare_data_in_func(state, fb.func);
    fb.ins().symbol_value(types::I64, gv)
}

/// Load the `_start`-filled `mn_pagesz` (0 when auxv had no AT_PAGESZ —
/// callers fail closed on that).
fn load_pagesz(
    om: &mut ObjectModule,
    pagesz: cranelift_module::DataId,
    fb: &mut FunctionBuilder<'_>,
    flags: MemFlags,
) -> cranelift_codegen::ir::Value {
    let gv = om.declare_data_in_func(pagesz, fb.func);
    let addr = fb.ins().symbol_value(types::I64, gv);
    fb.ins().load(types::I32, flags, addr, 0)
}

/// `ret` in `-4095..=-1` → the raw errno range of x86-64 syscalls.
fn in_errno_range(
    fb: &mut FunctionBuilder<'_>,
    ret: cranelift_codegen::ir::Value,
) -> cranelift_codegen::ir::Value {
    let lo = fb.ins().iconst(types::I64, -4095);
    let hi = fb.ins().iconst(types::I64, -1);
    let ge_lo = fb.ins().icmp(IntCC::SignedGreaterThanOrEqual, ret, lo);
    let le_hi = fb.ins().icmp(IntCC::SignedLessThanOrEqual, ret, hi);
    fb.ins().band(ge_lo, le_hi)
}

/// Emit the standalone Linux ELF platform shim. Zero libc/CRT, zero
/// imports besides the raw syscall stubs and the `mn_pagesz` slot.
pub fn emit_platform_shim_elf() -> Result<Vec<u8>, MnError> {
    let isa = make_isa_for(MnOsTarget::LinuxGnu)?;
    let builder = ObjectBuilder::new(isa, "mn-platform-shim-elf", default_libcall_names())
        .map_err(|e| MnError::Backend(format!("elf platform shim builder: {e}")))?;
    let mut om = ObjectModule::new(builder);

    let state = om
        .declare_data("mn_pio_state", Linkage::Local, true, false)
        .map_err(|e| MnError::Backend(format!("declare mn_pio_state: {e}")))?;
    let mut dd = DataDescription::new();
    dd.define_zeroinit(STATE_BYTES);
    om.define_data(state, &dd)
        .map_err(|e| MnError::Backend(format!("define mn_pio_state: {e}")))?;

    let pathbuf = om
        .declare_data("mn_pio_pathbuf", Linkage::Local, true, false)
        .map_err(|e| MnError::Backend(format!("declare mn_pio_pathbuf: {e}")))?;
    let mut pdd = DataDescription::new();
    pdd.define_zeroinit(MAX_PROBE_PATH_BYTES + 1);
    om.define_data(pathbuf, &pdd)
        .map_err(|e| MnError::Backend(format!("define mn_pio_pathbuf: {e}")))?;

    let pagesz = om
        .declare_data(LINUX_PAGESZ_SYMBOL, Linkage::Import, false, false)
        .map_err(|e| MnError::Backend(format!("declare {LINUX_PAGESZ_SYMBOL}: {e}")))?;

    let sys = SyscallIds::declare(&mut om)?;

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
    let _ = flags; // silence in case of path changes; used per-fn below

    // 1. find_slot(handle, expected_kind) -> slot ptr | negative ERR_* (i64)
    define_fn(
        &mut om,
        loc.find_slot,
        find_slot_sig.clone(),
        |om, fb, entry| crate::platform_io::emit_find_slot_body(om, fb, entry, state, flags),
    )?;

    // 2. alloc_slot(kind, os_ptr, logical, mapped) -> encoded handle
    define_fn(
        &mut om,
        loc.alloc_slot,
        alloc_slot_sig.clone(),
        |om, fb, entry| crate::platform_io::emit_alloc_slot_body(om, fb, entry, state, flags),
    )?;

    // 3. write_slice(h, ptr, slen, off, req_len) -> written | ERR_*  — the
    //    OS boundary is one raw `write` (partial writes cross as progress).
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
            let os_fd = fb.ins().load(types::I64, flags, slot, S_OS_PTR);
            let buf_ptr = fb.ins().iadd(ptr, off64);
            let nr = fb.ins().iconst(types::I64, linux_sysno::WRITE as i64);
            let wc = om.declare_func_in_func(sys.s3, fb.func);
            let call = fb.ins().call(wc, &[nr, os_fd, buf_ptr, chunk64]);
            let ret = fb.inst_results(call)[0];
            let failed = in_errno_range(fb, ret);
            let err_os = fb.create_block();
            let ok_os = fb.create_block();
            fb.ins().brif(failed, err_os, &[], ok_os, &[]);
            fb.seal_block(err_os);
            fb.seal_block(ok_os);

            fb.switch_to_block(err_os);
            let neg_err = fb.ins().ineg(ret);
            let e32 = fb.ins().ireduce(types::I32, neg_err);
            fb.ins().store(flags, e32, base, H_LAST_OS_ERROR);
            ret_err(fb, ERR_OS_FAILED);

            fb.switch_to_block(ok_os);
            // written = min(ret, chunk) — clamp an oversized kernel count.
            let gt_chunk = fb.ins().icmp(IntCC::SignedGreaterThan, ret, chunk64);
            let w64 = fb.ins().select(gt_chunk, chunk64, ret);
            let w32 = fb.ins().ireduce(types::I32, w64);
            let wu64 = fb.ins().uextend(types::I64, w32);
            let wt2 = fb.ins().iadd(wt, wu64);
            fb.ins().store(flags, wt2, base, H_WRITTEN_TOTAL);
            fb.ins().store(flags, z32, base, H_LAST_OS_ERROR);
            fb.ins().return_(&[w32]);
        },
    )?;

    // ---- exported operations ---------------------------------------------
    // stdout/stderr: Linux std fds are fixed (1/2); validity surfaces at
    // the write boundary as a controlled errno (no query call exists).
    for (kind, fd) in [
        (PlatformOpKind::IoStdout, 1i64),
        (PlatformOpKind::IoStderr, 2i64),
    ] {
        let s = cranelift_shim_sig_for(kind, CONV);
        let id = declare_fn(&mut om, shim_symbol_for(kind), Linkage::Export, &s)?;
        define_fn(&mut om, id, s, |om, fb, _| {
            let base = state_ptr(om, state, fb);
            let z32 = fb.ins().iconst(types::I32, 0);
            fb.ins().store(flags, z32, base, H_LAST_OS_ERROR);
            let alloc = om.declare_func_in_func(loc.alloc_slot, fb.func);
            let k = fb.ins().iconst(types::I32, KIND_STREAM);
            let os_fd = fb.ins().iconst(types::I64, fd);
            let z64 = fb.ins().iconst(types::I64, 0);
            let ac = fb.ins().call(alloc, &[k, os_fd, z64, z64]);
            let h = fb.inst_results(ac)[0];
            fb.ins().return_(&[h]);
        })?;
    }

    // IoWriteSome(h, ptr, len) → write_slice(h, ptr, len, 0, len)
    {
        let s = cranelift_shim_sig_for(PlatformOpKind::IoWriteSome, CONV);
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

    // IoWriteSomeSlice(h, ptr, slen, off, rlen)
    {
        let s = cranelift_shim_sig_for(PlatformOpKind::IoWriteSomeSlice, CONV);
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
        let s = cranelift_shim_sig_for(PlatformOpKind::IoLastError, CONV);
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

    // IoConsoleKind(h) — PLAN-1041: Linux std streams are always byte
    // sinks; the common algorithm never selects a console path here.
    {
        let s = cranelift_shim_sig_for(PlatformOpKind::IoConsoleKind, CONV);
        let id = declare_fn(
            &mut om,
            shim_symbol_for(PlatformOpKind::IoConsoleKind),
            Linkage::Export,
            &s,
        )?;
        define_fn(&mut om, id, s, |_, fb, _| {
            let sink = fb.ins().iconst(types::I32, 2);
            fb.ins().return_(&[sink]);
        })?;
    }

    // IoConsoleWrite16(...) — PLAN-1041: no Windows text-console write on
    // Linux; a call is a controlled value error, never a silent fallback.
    {
        let s = cranelift_shim_sig_for(PlatformOpKind::IoConsoleWrite16, CONV);
        let id = declare_fn(
            &mut om,
            shim_symbol_for(PlatformOpKind::IoConsoleWrite16),
            Linkage::Export,
            &s,
        )?;
        define_fn(&mut om, id, s, |_, fb, _| {
            ret_err(fb, ERR_INVALID_VALUE);
        })?;
    }

    // MemPageSize() / MemAllocGranularity() — from the auxv-filled slot;
    // Linux mmap granularity is the page size itself.
    for (kind, _) in [
        (PlatformOpKind::MemPageSize, 0i32),
        (PlatformOpKind::MemAllocGranularity, 0),
    ] {
        let s = cranelift_shim_sig_for(kind, CONV);
        let id = declare_fn(&mut om, shim_symbol_for(kind), Linkage::Export, &s)?;
        define_fn(&mut om, id, s, |om, fb, _| {
            let v = load_pagesz(om, pagesz, fb, flags);
            fb.ins().return_(&[v]);
        })?;
    }

    // MemAlloc(len) — anonymous private RW mmap via the raw 6-arg ABI.
    {
        let s = cranelift_shim_sig_for(PlatformOpKind::MemAlloc, CONV);
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
            let base = state_ptr(om, state, fb);
            let ps32 = load_pagesz(om, pagesz, fb, flags);
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
            let do_map = fb.create_block();
            fb.ins().brif(over_b, bad_bud, &[], do_map, &[]);
            fb.seal_block(bad_bud);
            fb.seal_block(do_map);

            fb.switch_to_block(bad_bud);
            ret_err(fb, ERR_BUDGET);

            fb.switch_to_block(do_map);
            let z64 = fb.ins().iconst(types::I64, 0);
            let nr = fb.ins().iconst(types::I64, linux_sysno::MMAP as i64);
            let prot = fb.ins().iconst(types::I64, PROT_READ_WRITE);
            let mflags = fb.ins().iconst(types::I64, MAP_PRIVATE_ANONYMOUS);
            let fdm1 = fb.ins().iconst(types::I64, MAP_FD_NONE);
            let mc = om.declare_func_in_func(sys.s6, fb.func);
            let call = fb
                .ins()
                .call(mc, &[nr, z64, aligned, prot, mflags, fdm1, z64]);
            let ptr = fb.inst_results(call)[0];
            let failed = in_errno_range(fb, ptr);
            let err_os = fb.create_block();
            let ok_os = fb.create_block();
            fb.ins().brif(failed, err_os, &[], ok_os, &[]);
            fb.seal_block(err_os);
            fb.seal_block(ok_os);

            fb.switch_to_block(err_os);
            let neg_err = fb.ins().ineg(ptr);
            let e32 = fb.ins().ireduce(types::I32, neg_err);
            fb.ins().store(flags, e32, base, H_LAST_OS_ERROR);
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

    // MemFree(handle) — munmap(base, exact mapped length).
    {
        let s = cranelift_shim_sig_for(PlatformOpKind::MemFree, CONV);
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
            let do_unmap = fb.create_block();
            fb.ins().brif(borrowed, bad_bor, &[], do_unmap, &[]);
            fb.seal_block(bad_bor);
            fb.seal_block(do_unmap);

            fb.switch_to_block(bad_bor);
            ret_err(fb, ERR_BORROWED);

            fb.switch_to_block(do_unmap);
            let base_ptr = fb.ins().load(types::I64, flags, slot, S_OS_PTR);
            let mapped = fb.ins().load(types::I64, flags, slot, S_MAPPED_LEN);
            let nr = fb.ins().iconst(types::I64, linux_sysno::MUNMAP as i64);
            let uc = om.declare_func_in_func(sys.s2, fb.func);
            let call = fb.ins().call(uc, &[nr, base_ptr, mapped]);
            let ret = fb.inst_results(call)[0];
            let failed = in_errno_range(fb, ret);
            let err_os = fb.create_block();
            let ok_os = fb.create_block();
            fb.ins().brif(failed, err_os, &[], ok_os, &[]);
            fb.seal_block(err_os);
            fb.seal_block(ok_os);

            fb.switch_to_block(err_os);
            let st_base = state_ptr(om, state, fb);
            let neg_err = fb.ins().ineg(ret);
            let e32 = fb.ins().ireduce(types::I32, neg_err);
            fb.ins().store(flags, e32, st_base, H_LAST_OS_ERROR);
            ret_err(fb, ERR_OS_FAILED);

            fb.switch_to_block(ok_os);
            let st_base = state_ptr(om, state, fb);
            let one32 = fb.ins().iconst(types::I32, 1);
            fb.ins().store(flags, one32, slot, S_RELEASED);
            let live = fb.ins().load(types::I64, flags, st_base, H_LIVE_MAPPED);
            let next_live = fb.ins().isub(live, mapped);
            fb.ins().store(flags, next_live, st_base, H_LIVE_MAPPED);
            fb.ins().store(flags, z32, st_base, H_LAST_OS_ERROR);
            fb.ins().return_(&[z32]);
        })?;
    }

    // MemLogicalLen / MemMappedLen
    for (kind, field) in [
        (PlatformOpKind::MemLogicalLen, S_LOGICAL_LEN),
        (PlatformOpKind::MemMappedLen, S_MAPPED_LEN),
    ] {
        let s = cranelift_shim_sig_for(kind, CONV);
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
        let s = cranelift_shim_sig_for(kind, CONV);
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

    // MemWriteU8 / MemReadU8 — same as the Win64 shim (raw guest memory).
    {
        let s = cranelift_shim_sig_for(PlatformOpKind::MemWriteU8, CONV);
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

    {
        let s = cranelift_shim_sig_for(PlatformOpKind::MemReadU8, CONV);
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

    // ---- PLAN-1040 probe seam (raw syscall marshalling) -------------------
    // probe_openat(dirfd, path_ptr, path_len, flags, mode) -> RAW result
    // (positive fd, or the raw negative errno). NUL-terminates on the OS
    // boundary into the fixed `mn_pio_pathbuf`; rejects embedded NULs.
    {
        let s = cranelift_shim_sig_for(PlatformOpKind::ProbeOpenat, CONV);
        let id = declare_fn(
            &mut om,
            shim_symbol_for(PlatformOpKind::ProbeOpenat),
            Linkage::Export,
            &s,
        )?;
        define_fn(&mut om, id, s, |om, fb, entry| {
            let dirfd = fb.block_params(entry)[0];
            let path_ptr = fb.block_params(entry)[1];
            let path_len = fb.block_params(entry)[2];
            let oflags = fb.block_params(entry)[3];
            let mode = fb.block_params(entry)[4];

            let fdcwd = fb.ins().iconst(types::I32, PROBE_AT_FDCWD as i64);
            let dir_ok = fb.ins().icmp(IntCC::Equal, dirfd, fdcwd);
            let bad_dir = fb.create_block();
            let chk_flags = fb.create_block();
            fb.ins().brif(dir_ok, chk_flags, &[], bad_dir, &[]);
            fb.seal_block(bad_dir);
            fb.seal_block(chk_flags);

            fb.switch_to_block(bad_dir);
            ret_err(fb, ERR_INVALID_VALUE);

            // Mirrors of the AV dispatch pre-checks (AV/AC parity, F2):
            // flags==0 and negative mode are invalid before any syscall.
            fb.switch_to_block(chk_flags);
            let zero32 = fb.ins().iconst(types::I32, 0);
            let flags_zero = fb.ins().icmp(IntCC::Equal, oflags, zero32);
            let mode_neg = fb.ins().icmp(IntCC::SignedLessThan, mode, zero32);
            let bad_fm = fb.ins().bor(flags_zero, mode_neg);
            let bad_fm_blk = fb.create_block();
            let chk_len = fb.create_block();
            fb.ins().brif(bad_fm, bad_fm_blk, &[], chk_len, &[]);
            fb.seal_block(bad_fm_blk);
            fb.seal_block(chk_len);

            fb.switch_to_block(bad_fm_blk);
            ret_err(fb, ERR_INVALID_VALUE);

            fb.switch_to_block(chk_len);
            let z32 = fb.ins().iconst(types::I32, 0);
            let len_lo = fb.ins().icmp(IntCC::SignedLessThanOrEqual, path_len, z32);
            let max_len = fb.ins().iconst(types::I32, MAX_PROBE_PATH_BYTES as i64);
            let len_hi = fb.ins().icmp(IntCC::SignedGreaterThan, path_len, max_len);
            let bad_len = fb.ins().bor(len_lo, len_hi);
            let bad_len_blk = fb.create_block();
            let loop_head = fb.create_block();
            let after_copy = fb.create_block();
            let reject_nul = fb.create_block();
            let body = fb.create_block();
            let store_blk = fb.create_block();
            // Loop index rides a frontend variable (the SSA builder joins
            // the backedge phis) — no block params needed.
            let ivar = fb.declare_var(types::I32);
            fb.def_var(ivar, z32);
            fb.ins().brif(bad_len, bad_len_blk, &[], loop_head, &[]);
            fb.seal_block(bad_len_blk);

            // loop_head: done? -> after_copy | body
            fb.switch_to_block(loop_head);
            let i = fb.use_var(ivar);
            let done = fb.ins().icmp(IntCC::SignedGreaterThanOrEqual, i, path_len);
            fb.ins().brif(done, after_copy, &[], body, &[]);
            fb.seal_block(body);
            fb.seal_block(after_copy);

            // body: b = path[i]; NUL -> reject; else store_blk
            fb.switch_to_block(body);
            let bi = fb.use_var(ivar);
            let i64v = fb.ins().sextend(types::I64, bi);
            let src = fb.ins().iadd(path_ptr, i64v);
            let b = fb.ins().uload8(types::I32, flags, src, 0);
            let z8 = fb.ins().iconst(types::I32, 0);
            let is_nul = fb.ins().icmp(IntCC::Equal, b, z8);
            fb.ins().brif(is_nul, reject_nul, &[], store_blk, &[]);

            // store_blk: pathbuf[i] = b; loop(i + 1)
            fb.switch_to_block(store_blk);
            let si = fb.use_var(ivar);
            let gv = om.declare_data_in_func(pathbuf, fb.func);
            let pbuf = fb.ins().symbol_value(types::I64, gv);
            let si64 = fb.ins().sextend(types::I64, si);
            let dst = fb.ins().iadd(pbuf, si64);
            let b8 = fb.ins().ireduce(types::I8, b);
            fb.ins().store(flags, b8, dst, 0);
            let one32 = fb.ins().iconst(types::I32, 1);
            let next_i = fb.ins().iadd(si, one32);
            fb.def_var(ivar, next_i);
            fb.ins().jump(loop_head, &[]);
            fb.seal_block(store_blk); // preds: body brif only
            fb.seal_block(loop_head); // preds: chk_len brif + store_blk jump
            fb.seal_block(reject_nul);

            // The out-of-range exit (its own block, so the loop stays a
            // two-predecessor shape).
            fb.switch_to_block(bad_len_blk);
            ret_err(fb, ERR_INVALID_BOUNDS);

            fb.switch_to_block(reject_nul);
            ret_err(fb, ERR_INVALID_BOUNDS);

            // after_copy: pathbuf[len] = 0; syscall4(openat, ...)
            fb.switch_to_block(after_copy);
            let len64 = fb.ins().sextend(types::I64, path_len);
            let gv2 = om.declare_data_in_func(pathbuf, fb.func);
            let pbuf2 = fb.ins().symbol_value(types::I64, gv2);
            let dst_end = fb.ins().iadd(pbuf2, len64);
            let nul8 = fb.ins().iconst(types::I8, 0);
            fb.ins().store(flags, nul8, dst_end, 0);

            let nr = fb.ins().iconst(types::I64, linux_sysno::OPENAT as i64);
            let dir64 = fb.ins().sextend(types::I64, dirfd);
            let bufaddr = fb.ins().symbol_value(types::I64, gv2);
            let clo = fb.ins().iconst(types::I64, O_CLOEXEC);
            let flags64 = fb.ins().sextend(types::I64, oflags);
            let flags_clo = fb.ins().bor(flags64, clo);
            let mode64 = fb.ins().sextend(types::I64, mode);
            let sc = om.declare_func_in_func(sys.s4, fb.func);
            let call = fb.ins().call(sc, &[nr, dir64, bufaddr, flags_clo, mode64]);
            let ret = fb.inst_results(call)[0];
            let raw = fb.ins().ireduce(types::I32, ret);
            fb.ins().return_(&[raw]);
        })?;
    }

    // probe_write_fd(fd, ptr, len) -> RAW result (count, or negative errno)
    {
        let s = cranelift_shim_sig_for(PlatformOpKind::ProbeWriteFd, CONV);
        let id = declare_fn(
            &mut om,
            shim_symbol_for(PlatformOpKind::ProbeWriteFd),
            Linkage::Export,
            &s,
        )?;
        define_fn(&mut om, id, s, |om, fb, entry| {
            let fd = fb.block_params(entry)[0];
            let ptr = fb.block_params(entry)[1];
            let len = fb.block_params(entry)[2];
            let z32 = fb.ins().iconst(types::I32, 0);
            // Defensive mirrors of the ledger pre-checks.
            let fd_bad = fb.ins().icmp(IntCC::SignedLessThanOrEqual, fd, z32);
            let bad_fd = fb.create_block();
            let chk_len = fb.create_block();
            fb.ins().brif(fd_bad, bad_fd, &[], chk_len, &[]);
            fb.seal_block(bad_fd);
            fb.seal_block(chk_len);

            fb.switch_to_block(bad_fd);
            ret_err(fb, ERR_INVALID_VALUE);

            fb.switch_to_block(chk_len);
            let len_bad_lo = fb.ins().icmp(IntCC::SignedLessThan, len, z32);
            let max_len = fb.ins().iconst(types::I32, MAX_PLATFORM_CALL_BYTES as i64);
            let len_bad_hi = fb.ins().icmp(IntCC::SignedGreaterThan, len, max_len);
            let len_bad = fb.ins().bor(len_bad_lo, len_bad_hi);
            let bad_len = fb.create_block();
            let chk_empty = fb.create_block();
            fb.ins().brif(len_bad, bad_len, &[], chk_empty, &[]);
            fb.seal_block(bad_len);
            fb.seal_block(chk_empty);

            fb.switch_to_block(bad_len);
            ret_err(fb, ERR_BUDGET);

            fb.switch_to_block(chk_empty);
            let is_empty = fb.ins().icmp(IntCC::Equal, len, z32);
            let ret_zero = fb.create_block();
            let do_wr = fb.create_block();
            fb.ins().brif(is_empty, ret_zero, &[], do_wr, &[]);
            fb.seal_block(ret_zero);
            fb.seal_block(do_wr);

            fb.switch_to_block(ret_zero);
            fb.ins().return_(&[z32]);

            fb.switch_to_block(do_wr);
            let nr = fb.ins().iconst(types::I64, linux_sysno::WRITE as i64);
            let fd64 = fb.ins().sextend(types::I64, fd);
            let len64 = fb.ins().sextend(types::I64, len);
            let sc = om.declare_func_in_func(sys.s3, fb.func);
            let call = fb.ins().call(sc, &[nr, fd64, ptr, len64]);
            let ret = fb.inst_results(call)[0];
            let raw = fb.ins().ireduce(types::I32, ret);
            fb.ins().return_(&[raw]);
        })?;
    }

    // probe_close(fd) -> RAW result (0, or negative errno); std fds never.
    {
        let s = cranelift_shim_sig_for(PlatformOpKind::ProbeClose, CONV);
        let id = declare_fn(
            &mut om,
            shim_symbol_for(PlatformOpKind::ProbeClose),
            Linkage::Export,
            &s,
        )?;
        define_fn(&mut om, id, s, |om, fb, entry| {
            let fd = fb.block_params(entry)[0];
            let two = fb.ins().iconst(types::I32, 2);
            let fd_bad = fb.ins().icmp(IntCC::SignedLessThanOrEqual, fd, two);
            let bad_fd = fb.create_block();
            let do_close = fb.create_block();
            fb.ins().brif(fd_bad, bad_fd, &[], do_close, &[]);
            fb.seal_block(bad_fd);
            fb.seal_block(do_close);

            fb.switch_to_block(bad_fd);
            ret_err(fb, ERR_INVALID_VALUE);

            fb.switch_to_block(do_close);
            let nr = fb.ins().iconst(types::I64, linux_sysno::CLOSE as i64);
            let fd64 = fb.ins().sextend(types::I64, fd);
            let sc = om.declare_func_in_func(sys.s1, fb.func);
            let call = fb.ins().call(sc, &[nr, fd64]);
            let ret = fb.inst_results(call)[0];
            let raw = fb.ins().ireduce(types::I32, ret);
            fb.ins().return_(&[raw]);
        })?;
    }

    let product = om.finish();
    let bytes = product
        .emit()
        .map_err(|e| MnError::Backend(format!("emit platform shim elf: {e}")))?;
    if bytes.len() > MAX_COFF_BYTES {
        return Err(MnError::Resource(
            "coff-size",
            format!(
                "elf platform shim object {} bytes above the frozen cap",
                bytes.len()
            ),
        ));
    }
    Ok(bytes)
}

// ---- linked-ELF dependency audit (ELF-04) -----------------------------

/// The dependency facts of one audited linked ELF executable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElfDependencyReport {
    /// ELF64/x86-64/ET_EXEC actually observed.
    pub elf64_x86_64_exec: bool,
    /// The PT_INTERP interpreter path, if any (forbidden).
    pub interpreter: Option<String>,
    /// True when a PT_DYNAMIC segment exists (forbidden for our products).
    pub has_dynamic: bool,
    /// Undefined non-local symbols remaining in the symtab (forbidden).
    pub undefined_symbols: Vec<String>,
}

/// Inspect a linked Linux ELF executable and reject ANY interpreter
/// (`PT_INTERP` — dynamic loader), ANY dynamic segment (`PT_DYNAMIC` —
/// shared-object machinery; our products are fully static), and ANY
/// undefined non-local symbol (an unresolved external — e.g. a smuggled
/// CRT `memcpy`/`malloc`); real execution in the target environment is
/// the second, independent witness (ELF-04). Hand-parses the ELF header
/// (no loader involvement — the audit must not depend on the thing it
/// audits).
pub fn audit_elf_dependencies(elf_bytes: &[u8]) -> Result<ElfDependencyReport, MnError> {
    const PT_DYNAMIC: u32 = 2;
    const PT_INTERP: u32 = 3;
    const SHT_SYMTAB: u32 = 2;
    const STB_LOCAL: u8 = 0;
    fn err(w: &'static str) -> MnError {
        MnError::Infra(format!("elf.audit: {w}"))
    }

    if elf_bytes.len() < 64 {
        return Err(err("shorter than an ELF64 header"));
    }
    if !(elf_bytes[0] == 0x7F
        && elf_bytes[1] == b'E'
        && elf_bytes[2] == b'L'
        && elf_bytes[3] == b'F')
    {
        return Err(err("ELF magic missing"));
    }
    let u16_at = |o: usize| u16::from_le_bytes([elf_bytes[o], elf_bytes[o + 1]]);
    let u32_at = |o: usize| {
        u32::from_le_bytes([
            elf_bytes[o],
            elf_bytes[o + 1],
            elf_bytes[o + 2],
            elf_bytes[o + 3],
        ])
    };
    let u64_at = |o: usize| {
        u64::from_le_bytes([
            elf_bytes[o],
            elf_bytes[o + 1],
            elf_bytes[o + 2],
            elf_bytes[o + 3],
            elf_bytes[o + 4],
            elf_bytes[o + 5],
            elf_bytes[o + 6],
            elf_bytes[o + 7],
        ])
    };
    if elf_bytes[4] != 2 || elf_bytes[5] != 1 {
        return Err(err("not ELF64 little-endian"));
    }
    let e_type = u16_at(16);
    let e_machine = u16_at(18);
    let elf64_x86_64_exec = e_type == 2 && e_machine == 62;

    // ---- program headers: PT_INTERP / PT_DYNAMIC -------------------------
    let phoff = u64_at(32);
    let phentsize = u16_at(54) as usize;
    let phnum = u16_at(56) as usize;
    let mut interpreter = None;
    let mut has_dynamic = false;
    if phnum > 0 {
        if phentsize < 56 {
            return Err(err("program header entry size too small"));
        }
        let table_end = (phoff as usize)
            .checked_add(
                phnum
                    .checked_mul(phentsize)
                    .ok_or_else(|| err("phdr overflow"))?,
            )
            .ok_or_else(|| err("phdr overflow"))?;
        if table_end > elf_bytes.len() {
            return Err(err("program header table overruns the file"));
        }
        for i in 0..phnum {
            let ph = phoff as usize + i * phentsize;
            let p_type = u32_at(ph);
            let p_offset = u64_at(ph + 8) as usize;
            let p_filesz = u64_at(ph + 32) as usize;
            if p_type == PT_INTERP {
                let raw = elf_bytes
                    .get(p_offset..p_offset.saturating_add(p_filesz))
                    .ok_or_else(|| err("interp window overruns the file"))?;
                let mut t_end = raw.len();
                while t_end > 0 && raw[t_end - 1] == 0 {
                    t_end -= 1;
                }
                let text = String::from_utf8_lossy(&raw[..t_end]).into_owned();
                interpreter = Some(text);
            }
            if p_type == PT_DYNAMIC {
                has_dynamic = true;
            }
        }
    }

    // ---- symtab: undefined non-local symbols -----------------------------
    let shoff = u64_at(40);
    let shentsize = u16_at(58) as usize;
    let shnum = u16_at(60) as usize;
    let mut undefined_symbols = Vec::new();
    if shnum > 0 && shentsize >= 64 {
        let sh_end = (shoff as usize)
            .checked_add(
                shnum
                    .checked_mul(shentsize)
                    .ok_or_else(|| err("shdr overflow"))?,
            )
            .ok_or_else(|| err("shdr overflow"))?;
        if sh_end > elf_bytes.len() {
            return Err(err("section header table overruns the file"));
        }
        for i in 0..shnum {
            let sh = shoff as usize + i * shentsize;
            if u32_at(sh + 4) != SHT_SYMTAB {
                continue;
            }
            let sym_off = u64_at(sh + 24) as usize;
            let sym_size = u64_at(sh + 32) as usize;
            let sym_link = u32_at(sh + 40) as usize; // strtab section index
            let entsize = u64_at(sh + 56) as usize;
            if entsize < 24 || sym_size % entsize != 0 {
                return Err(err("malformed symtab geometry"));
            }
            // The associated string table section.
            let strtab = if sym_link < shnum as usize {
                let st = shoff as usize + sym_link * shentsize;
                let st_off = u64_at(st + 24) as usize;
                let st_size = u64_at(st + 32) as usize;
                elf_bytes
                    .get(st_off..st_off.saturating_add(st_size))
                    .ok_or_else(|| err("strtab window overruns the file"))?
            } else {
                &[][..]
            };
            for off in (0..sym_size).step_by(entsize) {
                let sym = sym_off + off;
                if sym + 24 > elf_bytes.len() {
                    return Err(err("symbol window overruns the file"));
                }
                let st_name = u32_at(sym) as usize;
                let st_info = elf_bytes[sym + 4];
                let st_shndx = u16_at(sym + 6);
                let binding = st_info >> 4;
                if st_shndx == 0 && binding != STB_LOCAL {
                    // SHN_UNDEF + non-local = an unresolved external.
                    if let Some(tail) = strtab.get(st_name..) {
                        if let Some(end) = tail.iter().position(|b| *b == 0) {
                            let name = String::from_utf8_lossy(&tail[..end]).into_owned();
                            if !name.is_empty() {
                                undefined_symbols.push(name);
                            }
                        }
                    }
                }
            }
        }
    }

    if let Some(interp) = &interpreter {
        return Err(MnError::Unsupported(
            "elf.audit.interpreter",
            format!(
                "PT_INTERP `{interp}` present — the Linux native executable must be interpreter-free"
            ),
        ));
    }
    if has_dynamic {
        return Err(MnError::Unsupported(
            "elf.audit.dynamic",
            "PT_DYNAMIC present — shared-object machinery in a product that must be fully static"
                .to_string(),
        ));
    }
    if !undefined_symbols.is_empty() {
        return Err(MnError::Unsupported(
            "elf.audit.undefined",
            format!("undefined symbols remain: {undefined_symbols:?} — fully static link required"),
        ));
    }
    Ok(ElfDependencyReport {
        elf64_x86_64_exec,
        interpreter,
        has_dynamic,
        undefined_symbols,
    })
}
