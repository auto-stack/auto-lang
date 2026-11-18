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
/// PLAN-974 (T-02): the minimal TCP socket face — `platform.net/1`.
/// Full public net/http Auto API stays this package's later tasks; these
/// four primitives are the OS thin adapter the Auto protocol rides.
pub const CAP_NET_CONNECT: &str = "platform.net.connect";
pub const CAP_NET_WRITE: &str = "platform.net.write";
pub const CAP_NET_READ: &str = "platform.net.read";
pub const CAP_NET_CLOSE: &str = "platform.net.close";
/// PLAN-974 (T-02 inc3): the server face — listen + accept share one cap.
pub const CAP_NET_LISTEN: &str = "platform.net.listen";
// PLAN-973 system-resource capabilities (one per resource family; stdin
// joins io with its own grant so a session can take output-only).
pub const CAP_STDIN: &str = "platform.io.stdin";
pub const CAP_FILE: &str = "platform.file";
pub const CAP_PATH: &str = "platform.file.path";
pub const CAP_ENV: &str = "platform.env";
pub const CAP_CLOCK: &str = "platform.clock";
pub const CAP_ENTROPY: &str = "platform.entropy";
pub const CAP_PROC: &str = "platform.process";

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
const P_I32_NET: &[VT] = &[VT::I32];
const P_4I32: &[VT] = &[VT::I32, VT::I32, VT::I32, VT::I32];
const P_I32_STR: &[VT] = &[VT::I32, VT::Str];
const P_I32_STR_2I32: &[VT] = &[VT::I32, VT::Str, VT::I32, VT::I32];
const P_2I32: &[VT] = &[VT::I32, VT::I32];
const P_3I32: &[VT] = &[VT::I32, VT::I32, VT::I32];
const P_STR_I32: &[VT] = &[VT::Str, VT::I32];
const P_I32_STR_NET: &[VT] = &[VT::I32, VT::Str];
// PLAN-973 shapes.
const P_5I32: &[VT] = &[VT::I32, VT::I32, VT::I32, VT::I32, VT::I32];
const P_6I32: &[VT] = &[VT::I32, VT::I32, VT::I32, VT::I32, VT::I32, VT::I32];
const P_STR: &[VT] = &[VT::Str];
const P_2STR: &[VT] = &[VT::Str, VT::Str];
const P_STR_2I32: &[VT] = &[VT::Str, VT::I32, VT::I32];

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
    // PLAN-1041: classify a std stream — 1 = Windows text console, 2 =
    // byte sink (file/pipe/Linux stream); negative error on real
    // classification failures.
    PlatformSourceOp {
        callee: "io_console_kind",
        symbol: "auto.platform.io.console_kind.v1",
        params: P_I32,
        result: VT::I32,
        capability: CAP_WRITE,
    },
    // PLAN-1041: write UTF-16LE code units from an OS memory region to a
    // Windows text console (`WriteConsoleW`); params (stream handle,
    // region handle, unit offset, unit count); returns units written.
    PlatformSourceOp {
        callee: "io_console_write16",
        symbol: "auto.platform.io.console_write16.v1",
        params: P_4I32,
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
    PlatformSourceOp {
        callee: "net_tcp_connect",
        symbol: "auto.platform.net.tcp_connect.v1",
        params: P_STR_I32,
        result: VT::I32,
        capability: CAP_NET_CONNECT,
    },
    PlatformSourceOp {
        callee: "net_tcp_write_region",
        symbol: "auto.platform.net.tcp_write_region.v1",
        params: P_4I32,
        result: VT::I32,
        capability: CAP_NET_WRITE,
    },
    PlatformSourceOp {
        callee: "net_tcp_write_some",
        symbol: "auto.platform.net.tcp_write_some.v1",
        params: P_I32_STR_NET,
        result: VT::I32,
        capability: CAP_NET_WRITE,
    },
    PlatformSourceOp {
        callee: "net_tcp_read_into",
        symbol: "auto.platform.net.tcp_read_into.v1",
        params: P_4I32,
        result: VT::I32,
        capability: CAP_NET_READ,
    },
    PlatformSourceOp {
        callee: "net_tcp_listen",
        symbol: "auto.platform.net.tcp_listen.v1",
        params: P_I32_NET,
        result: VT::I32,
        capability: CAP_NET_LISTEN,
    },
    PlatformSourceOp {
        callee: "net_tcp_accept",
        symbol: "auto.platform.net.tcp_accept.v1",
        params: P_I32_NET,
        result: VT::I32,
        capability: CAP_NET_LISTEN,
    },
    PlatformSourceOp {
        callee: "net_tcp_local_port",
        symbol: "auto.platform.net.tcp_local_port.v1",
        params: P_I32_NET,
        result: VT::I32,
        capability: CAP_NET_LISTEN,
    },
    PlatformSourceOp {
        callee: "net_tcp_close",
        symbol: "auto.platform.net.tcp_close.v1",
        params: P_I32,
        result: VT::I32,
        capability: CAP_NET_CLOSE,
    },
    // ---- PLAN-973 system-resource reserved names --------------------
    PlatformSourceOp {
        callee: "io_stdin",
        symbol: "auto.platform.io.stdin.v1",
        params: P_EMPTY,
        result: VT::I32,
        capability: CAP_STDIN,
    },
    PlatformSourceOp {
        callee: "file_open_w16",
        symbol: "auto.platform.file.open_w16.v1",
        params: P_4I32,
        result: VT::I32,
        capability: CAP_FILE,
    },
    PlatformSourceOp {
        callee: "file_open_bytes",
        symbol: "auto.platform.file.open_bytes.v1",
        params: P_STR_I32,
        result: VT::I32,
        capability: CAP_FILE,
    },
    PlatformSourceOp {
        callee: "file_read_some",
        symbol: "auto.platform.file.read_some.v1",
        params: P_4I32,
        result: VT::I32,
        capability: CAP_FILE,
    },
    PlatformSourceOp {
        callee: "file_write_some",
        symbol: "auto.platform.file.write_some.v1",
        params: P_4I32,
        result: VT::I32,
        capability: CAP_FILE,
    },
    PlatformSourceOp {
        callee: "file_seek",
        symbol: "auto.platform.file.seek.v1",
        params: P_4I32,
        result: VT::I32,
        capability: CAP_FILE,
    },
    PlatformSourceOp {
        callee: "file_tell",
        symbol: "auto.platform.file.tell.v1",
        params: P_I32,
        result: VT::I32,
        capability: CAP_FILE,
    },
    PlatformSourceOp {
        callee: "file_hi",
        symbol: "auto.platform.file.hi.v1",
        params: P_I32,
        result: VT::I32,
        capability: CAP_FILE,
    },
    PlatformSourceOp {
        callee: "file_size",
        symbol: "auto.platform.file.size.v1",
        params: P_I32,
        result: VT::I32,
        capability: CAP_FILE,
    },
    PlatformSourceOp {
        callee: "file_mtime_ms",
        symbol: "auto.platform.file.mtime_ms.v1",
        params: P_I32,
        result: VT::I32,
        capability: CAP_FILE,
    },
    PlatformSourceOp {
        callee: "file_flush",
        symbol: "auto.platform.file.flush.v1",
        params: P_I32,
        result: VT::I32,
        capability: CAP_FILE,
    },
    PlatformSourceOp {
        callee: "file_close",
        symbol: "auto.platform.file.close.v1",
        params: P_I32,
        result: VT::I32,
        capability: CAP_FILE,
    },
    PlatformSourceOp {
        callee: "path_attrs_w16",
        symbol: "auto.platform.file.path_attrs_w16.v1",
        params: P_3I32,
        result: VT::I32,
        capability: CAP_PATH,
    },
    PlatformSourceOp {
        callee: "path_attrs_bytes",
        symbol: "auto.platform.file.path_attrs_bytes.v1",
        params: P_STR,
        result: VT::I32,
        capability: CAP_PATH,
    },
    PlatformSourceOp {
        callee: "file_delete_w16",
        symbol: "auto.platform.file.delete_w16.v1",
        params: P_3I32,
        result: VT::I32,
        capability: CAP_PATH,
    },
    PlatformSourceOp {
        callee: "file_delete_bytes",
        symbol: "auto.platform.file.delete_bytes.v1",
        params: P_STR,
        result: VT::I32,
        capability: CAP_PATH,
    },
    PlatformSourceOp {
        callee: "dir_create_w16",
        symbol: "auto.platform.file.dir_create_w16.v1",
        params: P_3I32,
        result: VT::I32,
        capability: CAP_PATH,
    },
    PlatformSourceOp {
        callee: "dir_create_bytes",
        symbol: "auto.platform.file.dir_create_bytes.v1",
        params: P_STR,
        result: VT::I32,
        capability: CAP_PATH,
    },
    PlatformSourceOp {
        callee: "dir_delete_w16",
        symbol: "auto.platform.file.dir_delete_w16.v1",
        params: P_3I32,
        result: VT::I32,
        capability: CAP_PATH,
    },
    PlatformSourceOp {
        callee: "dir_delete_bytes",
        symbol: "auto.platform.file.dir_delete_bytes.v1",
        params: P_STR,
        result: VT::I32,
        capability: CAP_PATH,
    },
    PlatformSourceOp {
        callee: "file_replace_w16",
        symbol: "auto.platform.file.replace_w16.v1",
        params: P_6I32,
        result: VT::I32,
        capability: CAP_PATH,
    },
    PlatformSourceOp {
        callee: "file_replace_bytes",
        symbol: "auto.platform.file.replace_bytes.v1",
        params: P_2STR,
        result: VT::I32,
        capability: CAP_PATH,
    },
    PlatformSourceOp {
        callee: "canonical_w16",
        symbol: "auto.platform.file.canonical_w16.v1",
        params: P_5I32,
        result: VT::I32,
        capability: CAP_PATH,
    },
    PlatformSourceOp {
        callee: "canonical_bytes",
        symbol: "auto.platform.file.canonical_bytes.v1",
        params: P_STR_2I32,
        result: VT::I32,
        capability: CAP_PATH,
    },
    PlatformSourceOp {
        callee: "dir_open_w16",
        symbol: "auto.platform.file.dir_open_w16.v1",
        params: P_3I32,
        result: VT::I32,
        capability: CAP_PATH,
    },
    PlatformSourceOp {
        callee: "dir_next_w16",
        symbol: "auto.platform.file.dir_next_w16.v1",
        params: P_3I32,
        result: VT::I32,
        capability: CAP_PATH,
    },
    PlatformSourceOp {
        callee: "dir_close",
        symbol: "auto.platform.file.dir_close.v1",
        params: P_I32,
        result: VT::I32,
        capability: CAP_PATH,
    },
    PlatformSourceOp {
        callee: "dir_open_bytes",
        symbol: "auto.platform.file.dir_open_bytes.v1",
        params: P_STR,
        result: VT::I32,
        capability: CAP_PATH,
    },
    PlatformSourceOp {
        callee: "dir_fill_bytes",
        symbol: "auto.platform.file.dir_fill_bytes.v1",
        params: P_3I32,
        result: VT::I32,
        capability: CAP_PATH,
    },
    PlatformSourceOp {
        callee: "env_get_w16",
        symbol: "auto.platform.env.get_w16.v1",
        params: P_5I32,
        result: VT::I32,
        capability: CAP_ENV,
    },
    PlatformSourceOp {
        callee: "env_set_w16",
        symbol: "auto.platform.env.set_w16.v1",
        params: P_6I32,
        result: VT::I32,
        capability: CAP_ENV,
    },
    PlatformSourceOp {
        callee: "env_remove_w16",
        symbol: "auto.platform.env.remove_w16.v1",
        params: P_3I32,
        result: VT::I32,
        capability: CAP_ENV,
    },
    PlatformSourceOp {
        callee: "env_get_bytes",
        symbol: "auto.platform.env.get_bytes.v1",
        params: P_3I32,
        result: VT::I32,
        capability: CAP_ENV,
    },
    PlatformSourceOp {
        callee: "clock_unix_ms",
        symbol: "auto.platform.clock.unix_ms.v1",
        params: P_EMPTY,
        result: VT::I32,
        capability: CAP_CLOCK,
    },
    PlatformSourceOp {
        callee: "clock_mono_ms",
        symbol: "auto.platform.clock.mono_ms.v1",
        params: P_EMPTY,
        result: VT::I32,
        capability: CAP_CLOCK,
    },
    PlatformSourceOp {
        callee: "clock_hi",
        symbol: "auto.platform.clock.hi.v1",
        params: P_EMPTY,
        result: VT::I32,
        capability: CAP_CLOCK,
    },
    PlatformSourceOp {
        callee: "sleep_ms",
        symbol: "auto.platform.clock.sleep_ms.v1",
        params: P_I32,
        result: VT::I32,
        capability: CAP_CLOCK,
    },
    PlatformSourceOp {
        callee: "rand_fill",
        symbol: "auto.platform.entropy.rand_fill.v1",
        params: P_3I32,
        result: VT::I32,
        capability: CAP_ENTROPY,
    },
    PlatformSourceOp {
        callee: "proc_spawn_w16",
        symbol: "auto.platform.process.spawn_w16.v1",
        params: P_4I32,
        result: VT::I32,
        capability: CAP_PROC,
    },
    PlatformSourceOp {
        callee: "proc_spawn_bytes",
        symbol: "auto.platform.process.spawn_bytes.v1",
        params: P_STR_I32,
        result: VT::I32,
        capability: CAP_PROC,
    },
    PlatformSourceOp {
        callee: "proc_pipe_read",
        symbol: "auto.platform.process.pipe_read.v1",
        params: P_6I32,
        result: VT::I32,
        capability: CAP_PROC,
    },
    PlatformSourceOp {
        callee: "proc_wait",
        symbol: "auto.platform.process.wait.v1",
        params: P_2I32,
        result: VT::I32,
        capability: CAP_PROC,
    },
    PlatformSourceOp {
        callee: "proc_release",
        symbol: "auto.platform.process.release.v1",
        params: P_I32,
        result: VT::I32,
        capability: CAP_PROC,
    },
    PlatformSourceOp {
        callee: "proc_exit",
        symbol: "auto.platform.process.exit.v1",
        params: P_I32,
        result: VT::I32,
        capability: CAP_PROC,
    },
    PlatformSourceOp {
        callee: "proc_cmdline_w16",
        symbol: "auto.platform.process.cmdline_w16.v1",
        params: P_2I32,
        result: VT::I32,
        capability: CAP_PROC,
    },
    /// PLAN-982 (Q-982-2 (a) step 1): the host-provided per-run argv line
    /// (narrow argv-read grant — never carries the spawn family).
    PlatformSourceOp {
        callee: "proc_run_argv_w16",
        symbol: "auto.platform.process.run_argv_w16.v1",
        params: P_2I32,
        result: VT::I32,
        capability: "platform.process.argv",
    },
    PlatformSourceOp {
        callee: "proc_cmdline_fill_bytes",
        symbol: "auto.platform.process.cmdline_fill_bytes.v1",
        params: P_3I32,
        result: VT::I32,
        capability: CAP_PROC,
    },
    PlatformSourceOp {
        callee: "proc_cwd_w16",
        symbol: "auto.platform.process.cwd_w16.v1",
        params: P_2I32,
        result: VT::I32,
        capability: CAP_PROC,
    },
    PlatformSourceOp {
        callee: "proc_cwd_bytes",
        symbol: "auto.platform.process.cwd_bytes.v1",
        params: P_2I32,
        result: VT::I32,
        capability: CAP_PROC,
    },
    PlatformSourceOp {
        callee: "proc_chdir_w16",
        symbol: "auto.platform.process.chdir_w16.v1",
        params: P_3I32,
        result: VT::I32,
        capability: CAP_PROC,
    },
    PlatformSourceOp {
        callee: "proc_chdir_bytes",
        symbol: "auto.platform.process.chdir_bytes.v1",
        params: P_STR,
        result: VT::I32,
        capability: CAP_PROC,
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
