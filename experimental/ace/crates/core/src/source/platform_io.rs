//! PLAN-1039 typed source platform I/O & OS memory symbols (`platform.io/1`, `platform.mem/1`).
//! No provider or host addresses appear in source or HIR.

use super::resolve::VT;

pub const CAP_STDOUT: &str = "platform.io.stdout";
pub const CAP_STDERR: &str = "platform.io.stderr";
pub const CAP_WRITE: &str = "platform.io.write";
pub const CAP_MEM: &str = "platform.mem";
/// PLAN-1040: the Linux direct-syscall internal probe seam — NOT a public
/// file API (full public File/path stays PLAN-973). Sources reach it only
/// through these internal reserved names.
pub const CAP_PROBE: &str = "platform.probe";

#[derive(Clone, Copy, Debug)]
pub struct PlatformSourceOp {
    pub callee: &'static str,
    pub symbol: &'static str,
    pub params: &'static [VT],
    pub result: VT,
    pub capability: &'static str,
}

const P_EMPTY: &[VT] = &[];
const P_I32: &[VT] = &[VT::I32];
const P_I32_STR: &[VT] = &[VT::I32, VT::Str];
const P_I32_STR_2I32: &[VT] = &[VT::I32, VT::Str, VT::I32, VT::I32];
const P_2I32: &[VT] = &[VT::I32, VT::I32];
const P_3I32: &[VT] = &[VT::I32, VT::I32, VT::I32];

pub const PLATFORM_SOURCE_OPS: &[PlatformSourceOp] = &[
    PlatformSourceOp {
        callee: "io_stdout",
        symbol: "auto.platform.io.stdout.v1",
        params: P_EMPTY,
        result: VT::I32,
        capability: CAP_STDOUT,
    },
    PlatformSourceOp {
        callee: "io_stderr",
        symbol: "auto.platform.io.stderr.v1",
        params: P_EMPTY,
        result: VT::I32,
        capability: CAP_STDERR,
    },
    PlatformSourceOp {
        callee: "io_write_some",
        symbol: "auto.platform.io.write_some.v1",
        params: P_I32_STR,
        result: VT::I32,
        capability: CAP_WRITE,
    },
    PlatformSourceOp {
        callee: "io_write_some_slice",
        symbol: "auto.platform.io.write_some_slice.v1",
        params: P_I32_STR_2I32,
        result: VT::I32,
        capability: CAP_WRITE,
    },
    PlatformSourceOp {
        callee: "io_last_error",
        symbol: "auto.platform.io.last_error.v1",
        params: P_EMPTY,
        result: VT::I32,
        capability: CAP_WRITE,
    },
    PlatformSourceOp {
        callee: "mem_page_size",
        symbol: "auto.platform.mem.page_size.v1",
        params: P_EMPTY,
        result: VT::I32,
        capability: CAP_MEM,
    },
    PlatformSourceOp {
        callee: "mem_alloc_granularity",
        symbol: "auto.platform.mem.alloc_granularity.v1",
        params: P_EMPTY,
        result: VT::I32,
        capability: CAP_MEM,
    },
    PlatformSourceOp {
        callee: "mem_alloc",
        symbol: "auto.platform.mem.alloc.v1",
        params: P_I32,
        result: VT::I32,
        capability: CAP_MEM,
    },
    PlatformSourceOp {
        callee: "mem_free",
        symbol: "auto.platform.mem.free.v1",
        params: P_I32,
        result: VT::I32,
        capability: CAP_MEM,
    },
    PlatformSourceOp {
        callee: "mem_logical_len",
        symbol: "auto.platform.mem.logical_len.v1",
        params: P_I32,
        result: VT::I32,
        capability: CAP_MEM,
    },
    PlatformSourceOp {
        callee: "mem_mapped_len",
        symbol: "auto.platform.mem.mapped_len.v1",
        params: P_I32,
        result: VT::I32,
        capability: CAP_MEM,
    },
    PlatformSourceOp {
        callee: "mem_borrow",
        symbol: "auto.platform.mem.borrow.v1",
        params: P_I32,
        result: VT::I32,
        capability: CAP_MEM,
    },
    PlatformSourceOp {
        callee: "mem_unborrow",
        symbol: "auto.platform.mem.unborrow.v1",
        params: P_I32,
        result: VT::I32,
        capability: CAP_MEM,
    },
    PlatformSourceOp {
        callee: "mem_write_u8",
        symbol: "auto.platform.mem.write_u8.v1",
        params: P_3I32,
        result: VT::I32,
        capability: CAP_MEM,
    },
    PlatformSourceOp {
        callee: "mem_read_u8",
        symbol: "auto.platform.mem.read_u8.v1",
        params: P_2I32,
        result: VT::I32,
        capability: CAP_MEM,
    },
    // ---- PLAN-1040 Linux direct-syscall internal probe seam -------------
    PlatformSourceOp {
        callee: "probe_openat",
        symbol: "auto.platform.probe.openat.v1",
        params: P_I32_STR_2I32,
        result: VT::I32,
        capability: CAP_PROBE,
    },
    PlatformSourceOp {
        callee: "probe_write_fd",
        symbol: "auto.platform.probe.write_fd.v1",
        params: P_I32_STR,
        result: VT::I32,
        capability: CAP_PROBE,
    },
    PlatformSourceOp {
        callee: "probe_close",
        symbol: "auto.platform.probe.close.v1",
        params: P_I32,
        result: VT::I32,
        capability: CAP_PROBE,
    },
];

pub fn lookup_callee(callee: &str) -> Option<&'static PlatformSourceOp> {
    PLATFORM_SOURCE_OPS.iter().find(|op| op.callee == callee)
}

pub fn signature(symbol: &str) -> Option<(&'static [VT], VT, &'static str)> {
    PLATFORM_SOURCE_OPS
        .iter()
        .find(|op| op.symbol == symbol)
        .map(|op| (op.params, op.result, op.capability))
}

pub fn is_reserved_name(name: &str) -> bool {
    lookup_callee(name).is_some()
}
