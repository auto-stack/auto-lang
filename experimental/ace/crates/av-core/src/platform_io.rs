//! PLAN-1039: Checked Platform I/O & OS Memory Region Primitives (`platform.io/1`, `platform.mem/1`).
//!
//! Single authoritative operation catalog, pre-OS ownership/generation/borrow/bounds
//! verification ledger, per-run capability re-authorization, and direct Windows x64 Win32
//! provider (`kernel32` `GetStdHandle`, `WriteFile`, `GetLastError`, `GetSystemInfo`,
//! `VirtualAlloc`, `VirtualFree`) with zero `std::io` / `libc` / CRT delegation on the OS path.

use crate::av::{CancelCheck, CancelFlag};
use crate::contract::Value;
use crate::host::{AdmissionError, HostError, ProviderScope};
use crate::mir::{MirHostImport, MirModule, MirType};
use std::collections::BTreeSet;

pub const CATEGORY_IO: &str = "platform.io/1";
pub const CATEGORY_MEM: &str = "platform.mem/1";
// PLAN-973 system-resource categories: files & paths, environment, clock,
// entropy and child processes — the same frozen op-table discipline as
// io/mem above (i32 in/out only; 64-bit values compose lo+hi in Auto).
pub const CATEGORY_FILE: &str = "platform.file/1";
pub const CATEGORY_ENV: &str = "platform.env/1";
pub const CATEGORY_CLOCK: &str = "platform.clock/1";
pub const CATEGORY_ENTROPY: &str = "platform.entropy/1";
pub const CATEGORY_PROC: &str = "platform.process/1";
/// PLAN-1040: the Linux direct-syscall internal probe seam (category
/// `platform.probe/1`). NOT a public file API — the full public File/path
/// contract stays PLAN-973's. These ops exist so the 1040 acceptance can
/// witness raw `openat`/`write`/`close` syscall marshalling (register
/// slots incl. the 4th argument, mode bits, negative errno
/// classification) from ordinary Auto sources on both AV (Linux backend)
/// and AC (ELF shim) without exposing arbitrary syscalls.
pub const CATEGORY_PROBE: &str = "platform.probe/1";

pub const CAP_STDOUT: &str = "platform.io.stdout";
pub const CAP_STDERR: &str = "platform.io.stderr";
pub const CAP_WRITE: &str = "platform.io.write";
pub const CAP_MEM: &str = "platform.mem";
pub const CAP_PROBE: &str = "platform.probe";
/// PLAN-974 (T-02): `platform.net/1` — the minimal TCP socket face.
pub const CATEGORY_NET: &str = "platform.net/1";
pub const CAP_NET_CONNECT: &str = "platform.net.connect";
pub const CAP_NET_WRITE: &str = "platform.net.write";
pub const CAP_NET_READ: &str = "platform.net.read";
pub const CAP_NET_CLOSE: &str = "platform.net.close";
/// PLAN-974 (T-02 inc3): the server face (listen + accept).
pub const CAP_NET_LISTEN: &str = "platform.net.listen";
/// PLAN-973 capabilities. One capability per resource family; stdin joins
/// the io category with its own grant so a session can take output-only.
pub const CAP_STDIN: &str = "platform.io.stdin";
pub const CAP_FILE: &str = "platform.file";
pub const CAP_PATH: &str = "platform.file.path";
pub const CAP_ENV: &str = "platform.env";
pub const CAP_CLOCK: &str = "platform.clock";
pub const CAP_ENTROPY: &str = "platform.entropy";
pub const CAP_PROC: &str = "platform.process";

pub const MAX_PLATFORM_IMPORTS: usize = 16;
pub const MAX_TOTAL_HOST_IMPORTS: usize = 24;
pub const MAX_PLATFORM_CALL_BYTES: usize = 65_536;
pub const MAX_PLATFORM_RUN_BYTES: usize = 1_048_576;
pub const MAX_OS_REGION_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_LEDGER_HANDLES: usize = 1024;
/// PLAN-974: concurrent TCP sockets per run (the ledger counts them).
pub const MAX_NET_SOCKETS: usize = 16;
/// PLAN-974: cumulative net bytes (read + written) per run.
pub const MAX_NET_RUN_BYTES: usize = 1_048_576;
/// PLAN-974: the raw backend marker for "no TCP backend here" (a named
/// control, never a silent zero).
pub const NET_TCP_UNSUPPORTED: u32 = 0x0010_4002;
/// PLAN-1040: bound of one probe `openat` path (excludes the NUL we
/// append ourselves; NAME_MAX-bound kernel paths stay far below this).
pub const MAX_PROBE_PATH_BYTES: usize = 4096;
/// PLAN-1040: marker a backend returns from the probe seam when the
/// current platform has no probe implementation (Windows Win32 backend);
/// distinct from any errno (<=~133) and Win32 error space on purpose.
pub const PROBE_UNSUPPORTED: u32 = 0x0010_4000;
/// PLAN-1040: the only `dirfd` the probe seam admits for `openat`
/// (Linux `AT_FDCWD`); arbitrary dirfds would smuggle kernel handles.
pub const PROBE_AT_FDCWD: i32 = -100;
/// PLAN-1041: marker a backend returns from `console_write16` when the
/// platform has no Windows text-console write (the Linux backend — the
/// common algorithm never selects the console path there because
/// `console_kind` is always 2). Distinct from errno (<=~133) and the
/// Win32 error space on purpose, like `PROBE_UNSUPPORTED`.
pub const CONSOLE_WRITE16_UNSUPPORTED: u32 = 0x0010_4001;
/// PLAN-1041: console classification results (see `IoConsoleKind`).
pub const CONSOLE_KIND_TEXT: i32 = 1;
pub const CONSOLE_KIND_BYTE_SINK: i32 = 2;

// Checked negative i32 error codes returned to guest code (deterministic; never trap on OS or preflight failure).
pub const ERR_UNAUTHORIZED: i32 = -1;
pub const ERR_INVALID_HANDLE: i32 = -2;
pub const ERR_WRONG_ORIGIN: i32 = -3;
pub const ERR_INVALID_BOUNDS: i32 = -4;
pub const ERR_BORROWED: i32 = -5;
pub const ERR_ALREADY_RELEASED: i32 = -6;
pub const ERR_INVALID_VALUE: i32 = -7;
pub const ERR_OS_FAILED: i32 = -8;
pub const ERR_BUDGET: i32 = -9;
/// PLAN-974 inc3: a nonblocking accept found no pending connection — a
/// RETRYABLE condition (the guest polls); never an OS failure.
pub const ERR_WOULDB_BLOCK: i32 = -10;
/// PLAN-973: the op is named-unsupported on THIS backend/target (e.g.
/// Linux `env_set` before the entry-captured environ arrives — deferred to
/// PLAN-1063 with the deferred-required/NOT_RUN handover). A named
/// rejection, never a silent fallback to another provider.
pub const ERR_UNSUPPORTED: i32 = -10;
/// PLAN-973: wait deadline expired (process still running; the ledger has
/// already killed the job tree per the V-03 contract before returning).
pub const ERR_TIMEOUT: i32 = -11;

/// PLAN-973 uniform open-flags bit set (mapped to OS flags per backend;
/// read/write/create/truncate/exclusive/append are separate switches per
/// the platform-IO design §7).
pub const OPEN_READ: i32 = 1;
pub const OPEN_WRITE: i32 = 2;
pub const OPEN_CREATE: i32 = 4;
pub const OPEN_TRUNCATE: i32 = 8;
pub const OPEN_EXCLUSIVE: i32 = 16;
pub const OPEN_APPEND: i32 = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParamDir {
    In,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LenUnit {
    None,
    Utf8Bytes,
    ByteCount,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HandleOrigin {
    None,
    StdStream,
    OsMemRegion,
    NetSocket,
    /// PLAN-973: ledger file handle (`file_*` ops).
    OsFile,
    /// PLAN-973: ledger directory-enumeration handle (`dir_*` ops).
    OsDir,
    /// PLAN-973: ledger child-process handle (`proc_*` ops).
    OsProcess,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StreamKind {
    Stdout,
    Stderr,
    /// PLAN-973: the standard input stream — readable via `file_read_some`
    /// (the legacy `IO.read_line` seam); never writable.
    Stdin,
}

impl StreamKind {
    pub fn capability(self) -> &'static str {
        match self {
            StreamKind::Stdout => CAP_STDOUT,
            StreamKind::Stderr => CAP_STDERR,
            StreamKind::Stdin => CAP_STDIN,
        }
    }

    pub fn win32_std_id(self) -> i32 {
        match self {
            StreamKind::Stdout => -11, // STD_OUTPUT_HANDLE
            StreamKind::Stderr => -12, // STD_ERROR_HANDLE
            StreamKind::Stdin => -10,  // STD_INPUT_HANDLE
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlatformOpKind {
    IoStdout,
    IoStderr,
    IoWriteSome,
    IoWriteSomeSlice,
    IoLastError,
    MemPageSize,
    MemAllocGranularity,
    MemAlloc,
    MemFree,
    MemLogicalLen,
    MemMappedLen,
    MemBorrow,
    MemUnborrow,
    MemWriteU8,
    MemReadU8,
    /// PLAN-1040 probe seam: raw `openat(dirfd, path, flags, mode)` —
    /// returns a raw positive fd or the raw negative Linux errno.
    ProbeOpenat,
    /// PLAN-1040 probe seam: raw `write(fd, bytes)` — returns the written
    /// count or the raw negative Linux errno.
    ProbeWriteFd,
    /// PLAN-1040 probe seam: raw `close(fd)` — returns 0 or the raw
    /// negative Linux errno.
    ProbeClose,
    /// PLAN-1041: classify a std stream handle — 1 = Windows text console,
    /// 2 = byte sink (file/pipe/Linux stream). Negative error codes on real
    /// classification failures (permission); invalid-handle surfaces at the
    /// std-handle query step (1039 behavior).
    IoConsoleKind,
    /// PLAN-1041: write UTF-16LE code units from an OS memory region to a
    /// Windows text console via `WriteConsoleW`. Params: (stream handle,
    /// region handle, unit offset, unit count). Returns units written
    /// (progress) or a negative error code.
    IoConsoleWrite16,
    /// PLAN-974: connect a TCP socket (host, port) — returns a ledger
    /// socket handle or a negative error code.
    NetTcpConnect,
    /// PLAN-974: write UTF-8 bytes to a socket — returns bytes written.
    NetTcpWriteSome,
    /// PLAN-974: read into an OS memory region — returns bytes read
    /// (0 = orderly EOF) or a negative error code.
    NetTcpReadInto,
    /// PLAN-974: close a socket — 0 or a negative error code.
    NetTcpClose,
    /// PLAN-974 inc3: bind a loopback listener (port 0 = ephemeral); the
    /// actual port is observed via `net_tcp_local_port`.
    NetTcpListen,
    /// PLAN-974 inc3: accept one connection on a listener — returns a NEW
    /// socket handle (a fresh ledger entry).
    NetTcpAccept,
    /// PLAN-974 T-08: write RAW BYTES from an OS memory region (non-UTF-8
    /// frames; the write_some Str face stays UTF-8-only).
    NetTcpWriteRegion,
    /// PLAN-974 inc3: the listener's actual bound port (ephemeral recall).
    NetTcpLocalPort,
    // ---- PLAN-973 system-resource operations --------------------------------
    /// stdin ledger handle (`GetStdHandle` / fd 0).
    IoStdin,
    /// Windows file open: UTF-16LE path units staged in an OS region.
    /// Params (region, unit off, unit count, OPEN_* flags) → file handle.
    FileOpenW16,
    /// Linux file open: raw path bytes (NUL rejected pre-OS). Params
    /// (path, OPEN_* flags) → file handle.
    FileOpenBytes,
    /// Read some bytes into an OS region. Params (handle, region, byte
    /// off, byte len) → byte count (0 = EOF at a nonempty read) or error.
    /// Accepts file handles and the stdin stream handle.
    FileReadSome,
    /// Write some bytes from an OS region. Params (handle, region, byte
    /// off, byte len) → byte count written (real progress) or error.
    FileWriteSome,
    /// Seek. Params (handle, origin 0=set/1=cur/2=end, pos lo, pos hi as
    /// signed 64-bit offset) → new position lo; high half via `file_hi`.
    FileSeek,
    /// Current position lo; high half via `file_hi`.
    FileTell,
    /// High 32 bits of the last 64-bit result on this handle
    /// (seek/tell/size/mtime).
    FileHi,
    /// File size lo; high half via `file_hi`.
    FileSize,
    /// Last-write time as unix milliseconds lo; high half via `file_hi`.
    FileMtimeMs,
    /// Flush OS buffers to durable storage (distinct from Auto buffering).
    FileFlush,
    /// Close a file handle; owner-visible close errors surface here.
    FileClose,
    /// Windows path attributes (GetFileAttributesW): bit0 = directory,
    /// bit1 = read-only; negative error when the path cannot be queried.
    PathAttrsW16,
    /// Linux path attributes via openat(O_PATH)+fstat: same bit layout.
    PathAttrsBytes,
    /// Windows delete file (DeleteFileW).
    FileDeleteW16,
    /// Linux delete file (unlinkat, flags 0).
    FileDeleteBytes,
    /// Windows create directory (CreateDirectoryW).
    DirCreateW16,
    /// Linux create directory (mkdirat, mode 0o777 & umask).
    DirCreateBytes,
    /// Windows delete directory (RemoveDirectoryW — must be empty).
    DirDeleteW16,
    /// Linux delete directory (unlinkat with AT_REMOVEDIR — must be empty).
    DirDeleteBytes,
    /// Windows atomic replace (MoveFileExW, REPLACE_EXISTING) — dest is
    /// replaced in one OS step; a failure keeps the old file untouched.
    FileReplaceW16,
    /// Linux rename (renameat, atomic on the same filesystem).
    FileReplaceBytes,
    /// Windows canonical path: resolve to `\\?\`-prefixed final path
    /// (GetFullPathNameW + GetFinalPathNameByHandleW when openable).
    /// Params (region, unit off, unit count, out region, out unit cap) →
    /// written unit count.
    CanonicalW16,
    /// Linux canonical path via the O_PATH + readlink(/proc/self/fd)
    /// component walk (no libc realpath). Same param/return shape over
    /// raw bytes.
    CanonicalBytes,
    /// Windows directory enumeration open (FindFirstFileW pattern
    /// `<dir>\*`); the first entry is staged in the ledger and drained by
    /// the first `dir_next_w16`.
    DirOpenW16,
    /// Windows next entry name into a region (unit cap) → unit count,
    /// 0 = end of stream.
    DirNextW16,
    /// Close a directory handle (FindClose / close).
    DirClose,
    /// Linux directory open (openat O_RDONLY|O_DIRECTORY) → dir handle.
    DirOpenBytes,
    /// Linux batched directory read (getdents64) into a region → byte
    /// count filled; the shared Auto layer parses linux_dirent64 records.
    DirFillBytes,
    /// Windows env get: (name region, unit off, unit count, out region,
    /// out unit cap) → value unit count (0 = unset).
    EnvGetW16,
    /// Windows env set (SetEnvironmentVariableW): empty value removes.
    EnvSetW16,
    /// Windows env remove.
    EnvRemoveW16,
    /// Linux env get via /proc/self/environ (raw byte snapshot parse in
    /// the shared Auto layer).
    EnvGetBytes,
    /// Unix wall clock in epoch milliseconds (lo; hi via `clock_hi`).
    ClockUnixMs,
    /// Monotonic clock in milliseconds (lo; hi via `clock_hi`) — deadlines
    /// and sleeps only, never wall time.
    ClockMonoMs,
    /// High 32 bits of the last clock result.
    ClockHi,
    /// Sleep the guest thread for whole milliseconds (interruptible by
    /// cancellation on the AV host).
    SleepMs,
    /// Fill an OS region with system CSPRNG bytes (BCryptGenRandom /
    /// getrandom). Params (region, off, len).
    RandFill,
    /// Windows spawn (CreateProcessW + job kill-on-close + optional
    /// capture pipes). Params (cmdline region, unit off, unit count,
    /// mode bit0 = capture) → process handle.
    ProcSpawnW16,
    /// Read captured child output. Params (proc handle, which 1=stdout
    /// 2=stderr, region, off, cap, timeout ms) → byte count; 0 = that
    /// pipe reached EOF; `-11` timeout with no data (V-03: the child is
    /// NOT killed by a pipe-read timeout — `proc_wait` owns the kill).
    ProcPipeRead,
    /// Wait for the child. Params (proc handle, timeout ms <0 = forever).
    /// → exit code; `-11` timeout AFTER killing the job tree (children
    /// included) per the V-03 reclamation contract.
    ProcWait,
    /// Release the process handle (closes pipes; job close kills any
    /// still-running tree).
    ProcRelease,
    /// Terminate the running program with the given exit code. On AV this
    /// returns a controlled `ExecOutcome::Exit` (never kills the host);
    /// on AC it lowers to `ExitProcess`/`exit_group`.
    ProcExit,
    /// Windows copy the full command line into a region (GetCommandLineW)
    /// → unit count; the shared Auto layer parses quoting.
    ProcCmdlineW16,
    /// Linux fill /proc/self/cmdline into a region → byte count
    /// (NUL-separated; the Auto layer parses).
    ProcCmdlineFillBytes,
    /// Windows current directory into a region → unit count.
    ProcCwdW16,
    /// Linux getcwd into a region → byte count.
    ProcCwdBytes,
    /// Windows change directory (SetCurrentDirectoryW).
    ProcChdirW16,
    /// Linux change directory (chdir).
    ProcChdirBytes,
    /// Linux spawn: NUL-separated argv blob (`"cmd\0arg1\0arg2\0"`) +
    /// mode bit0 = capture. Deferred-required on Linux (1063) — the
    /// Windows surface uses `proc_spawn_w16`.
    ProcSpawnBytes,
}

#[derive(Clone, Copy, Debug)]
pub struct PlatformOpSpec {
    pub kind: PlatformOpKind,
    pub callee: &'static str,
    pub symbol: &'static str,
    pub category: &'static str,
    pub capability: &'static str,
    pub params: &'static [MirType],
    pub param_dirs: &'static [ParamDir],
    pub len_unit: LenUnit,
    pub origin: HandleOrigin,
    pub max_call_bytes: u32,
}

const D_0: &[ParamDir] = &[];
const D_1: &[ParamDir] = &[ParamDir::In];
const D_2: &[ParamDir] = &[ParamDir::In, ParamDir::In];
const D_3: &[ParamDir] = &[ParamDir::In, ParamDir::In, ParamDir::In];
const D_4: &[ParamDir] = &[ParamDir::In, ParamDir::In, ParamDir::In, ParamDir::In];
const D_5: &[ParamDir] = &[ParamDir::In; 5];
const D_6: &[ParamDir] = &[ParamDir::In; 6];

const P_EMPTY: &[MirType] = &[];
const P_I32: &[MirType] = &[MirType::I32];
const P_I32_STR: &[MirType] = &[MirType::I32, MirType::StrView];
const P_I32_STR_2I32: &[MirType] = &[MirType::I32, MirType::StrView, MirType::I32, MirType::I32];
const P_2I32: &[MirType] = &[MirType::I32, MirType::I32];
const P_3I32: &[MirType] = &[MirType::I32, MirType::I32, MirType::I32];
const P_4I32: &[MirType] = &[MirType::I32, MirType::I32, MirType::I32, MirType::I32];
const P_STR_I32: &[MirType] = &[MirType::StrView, MirType::I32];
const P_5I32: &[MirType] = &[
    MirType::I32,
    MirType::I32,
    MirType::I32,
    MirType::I32,
    MirType::I32,
];
const P_6I32: &[MirType] = &[
    MirType::I32,
    MirType::I32,
    MirType::I32,
    MirType::I32,
    MirType::I32,
    MirType::I32,
];
// PLAN-973 path-bearing shapes (raw byte paths on Linux).
const P_STR: &[MirType] = &[MirType::StrView];
const P_2STR: &[MirType] = &[MirType::StrView, MirType::StrView];
const P_STR_2I32: &[MirType] = &[MirType::StrView, MirType::I32, MirType::I32];

pub const PLATFORM_OPS: &[PlatformOpSpec] = &[
    PlatformOpSpec {
        kind: PlatformOpKind::IoStdout,
        callee: "io_stdout",
        symbol: "auto.platform.io.stdout.v1",
        category: CATEGORY_IO,
        capability: CAP_STDOUT,
        params: P_EMPTY,
        param_dirs: D_0,
        len_unit: LenUnit::None,
        origin: HandleOrigin::StdStream,
        max_call_bytes: 0,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::IoStderr,
        callee: "io_stderr",
        symbol: "auto.platform.io.stderr.v1",
        category: CATEGORY_IO,
        capability: CAP_STDERR,
        params: P_EMPTY,
        param_dirs: D_0,
        len_unit: LenUnit::None,
        origin: HandleOrigin::StdStream,
        max_call_bytes: 0,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::IoWriteSome,
        callee: "io_write_some",
        symbol: "auto.platform.io.write_some.v1",
        category: CATEGORY_IO,
        capability: CAP_WRITE,
        params: P_I32_STR,
        param_dirs: D_2,
        len_unit: LenUnit::Utf8Bytes,
        origin: HandleOrigin::StdStream,
        max_call_bytes: MAX_PLATFORM_CALL_BYTES as u32,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::IoWriteSomeSlice,
        callee: "io_write_some_slice",
        symbol: "auto.platform.io.write_some_slice.v1",
        category: CATEGORY_IO,
        capability: CAP_WRITE,
        params: P_I32_STR_2I32,
        param_dirs: D_4,
        len_unit: LenUnit::Utf8Bytes,
        origin: HandleOrigin::StdStream,
        max_call_bytes: MAX_PLATFORM_CALL_BYTES as u32,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::IoLastError,
        callee: "io_last_error",
        symbol: "auto.platform.io.last_error.v1",
        category: CATEGORY_IO,
        capability: CAP_WRITE,
        params: P_EMPTY,
        param_dirs: D_0,
        len_unit: LenUnit::None,
        origin: HandleOrigin::None,
        max_call_bytes: 0,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::IoConsoleKind,
        callee: "io_console_kind",
        symbol: "auto.platform.io.console_kind.v1",
        category: CATEGORY_IO,
        capability: CAP_WRITE,
        params: P_I32,
        param_dirs: D_1,
        len_unit: LenUnit::None,
        origin: HandleOrigin::StdStream,
        max_call_bytes: 0,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::IoConsoleWrite16,
        callee: "io_console_write16",
        symbol: "auto.platform.io.console_write16.v1",
        category: CATEGORY_IO,
        capability: CAP_WRITE,
        params: P_4I32,
        param_dirs: D_4,
        len_unit: LenUnit::None,
        origin: HandleOrigin::StdStream,
        max_call_bytes: MAX_PLATFORM_CALL_BYTES as u32,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::MemPageSize,
        callee: "mem_page_size",
        symbol: "auto.platform.mem.page_size.v1",
        category: CATEGORY_MEM,
        capability: CAP_MEM,
        params: P_EMPTY,
        param_dirs: D_0,
        len_unit: LenUnit::ByteCount,
        origin: HandleOrigin::None,
        max_call_bytes: 0,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::MemAllocGranularity,
        callee: "mem_alloc_granularity",
        symbol: "auto.platform.mem.alloc_granularity.v1",
        category: CATEGORY_MEM,
        capability: CAP_MEM,
        params: P_EMPTY,
        param_dirs: D_0,
        len_unit: LenUnit::ByteCount,
        origin: HandleOrigin::None,
        max_call_bytes: 0,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::MemAlloc,
        callee: "mem_alloc",
        symbol: "auto.platform.mem.alloc.v1",
        category: CATEGORY_MEM,
        capability: CAP_MEM,
        params: P_I32,
        param_dirs: D_1,
        len_unit: LenUnit::ByteCount,
        origin: HandleOrigin::OsMemRegion,
        max_call_bytes: MAX_OS_REGION_BYTES as u32,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::MemFree,
        callee: "mem_free",
        symbol: "auto.platform.mem.free.v1",
        category: CATEGORY_MEM,
        capability: CAP_MEM,
        params: P_I32,
        param_dirs: D_1,
        len_unit: LenUnit::None,
        origin: HandleOrigin::OsMemRegion,
        max_call_bytes: 0,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::MemLogicalLen,
        callee: "mem_logical_len",
        symbol: "auto.platform.mem.logical_len.v1",
        category: CATEGORY_MEM,
        capability: CAP_MEM,
        params: P_I32,
        param_dirs: D_1,
        len_unit: LenUnit::ByteCount,
        origin: HandleOrigin::OsMemRegion,
        max_call_bytes: 0,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::MemMappedLen,
        callee: "mem_mapped_len",
        symbol: "auto.platform.mem.mapped_len.v1",
        category: CATEGORY_MEM,
        capability: CAP_MEM,
        params: P_I32,
        param_dirs: D_1,
        len_unit: LenUnit::ByteCount,
        origin: HandleOrigin::OsMemRegion,
        max_call_bytes: 0,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::MemBorrow,
        callee: "mem_borrow",
        symbol: "auto.platform.mem.borrow.v1",
        category: CATEGORY_MEM,
        capability: CAP_MEM,
        params: P_I32,
        param_dirs: D_1,
        len_unit: LenUnit::None,
        origin: HandleOrigin::OsMemRegion,
        max_call_bytes: 0,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::MemUnborrow,
        callee: "mem_unborrow",
        symbol: "auto.platform.mem.unborrow.v1",
        category: CATEGORY_MEM,
        capability: CAP_MEM,
        params: P_I32,
        param_dirs: D_1,
        len_unit: LenUnit::None,
        origin: HandleOrigin::OsMemRegion,
        max_call_bytes: 0,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::MemWriteU8,
        callee: "mem_write_u8",
        symbol: "auto.platform.mem.write_u8.v1",
        category: CATEGORY_MEM,
        capability: CAP_MEM,
        params: P_3I32,
        param_dirs: D_3,
        len_unit: LenUnit::ByteCount,
        origin: HandleOrigin::OsMemRegion,
        max_call_bytes: 0,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::MemReadU8,
        callee: "mem_read_u8",
        symbol: "auto.platform.mem.read_u8.v1",
        category: CATEGORY_MEM,
        capability: CAP_MEM,
        params: P_2I32,
        param_dirs: D_2,
        len_unit: LenUnit::ByteCount,
        origin: HandleOrigin::OsMemRegion,
        max_call_bytes: 0,
    },
    // ---- PLAN-1040 Linux direct-syscall internal probe seam -------------
    PlatformOpSpec {
        kind: PlatformOpKind::ProbeOpenat,
        callee: "probe_openat",
        symbol: "auto.platform.probe.openat.v1",
        category: CATEGORY_PROBE,
        capability: CAP_PROBE,
        params: P_I32_STR_2I32,
        param_dirs: D_4,
        len_unit: LenUnit::ByteCount,
        origin: HandleOrigin::None,
        max_call_bytes: MAX_PROBE_PATH_BYTES as u32,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::ProbeWriteFd,
        callee: "probe_write_fd",
        symbol: "auto.platform.probe.write_fd.v1",
        category: CATEGORY_PROBE,
        capability: CAP_PROBE,
        params: P_I32_STR,
        param_dirs: D_2,
        len_unit: LenUnit::ByteCount,
        origin: HandleOrigin::None,
        max_call_bytes: MAX_PLATFORM_CALL_BYTES as u32,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::ProbeClose,
        callee: "probe_close",
        symbol: "auto.platform.probe.close.v1",
        category: CATEGORY_PROBE,
        capability: CAP_PROBE,
        params: P_I32,
        param_dirs: D_1,
        len_unit: LenUnit::None,
        origin: HandleOrigin::None,
        max_call_bytes: 0,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::NetTcpConnect,
        callee: "net_tcp_connect",
        symbol: "auto.platform.net.tcp_connect.v1",
        category: CATEGORY_NET,
        capability: CAP_NET_CONNECT,
        params: P_STR_I32,
        param_dirs: D_2,
        len_unit: LenUnit::Utf8Bytes,
        origin: HandleOrigin::None,
        max_call_bytes: 256,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::NetTcpWriteRegion,
        callee: "net_tcp_write_region",
        symbol: "auto.platform.net.tcp_write_region.v1",
        category: CATEGORY_NET,
        capability: CAP_NET_WRITE,
        params: P_4I32,
        param_dirs: D_4,
        len_unit: LenUnit::None,
        origin: HandleOrigin::NetSocket,
        max_call_bytes: 0,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::NetTcpWriteSome,
        callee: "net_tcp_write_some",
        symbol: "auto.platform.net.tcp_write_some.v1",
        category: CATEGORY_NET,
        capability: CAP_NET_WRITE,
        params: P_I32_STR,
        param_dirs: D_2,
        len_unit: LenUnit::Utf8Bytes,
        origin: HandleOrigin::NetSocket,
        max_call_bytes: MAX_PLATFORM_CALL_BYTES as u32,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::NetTcpReadInto,
        callee: "net_tcp_read_into",
        symbol: "auto.platform.net.tcp_read_into.v1",
        category: CATEGORY_NET,
        capability: CAP_NET_READ,
        params: P_4I32,
        param_dirs: D_4,
        len_unit: LenUnit::None,
        origin: HandleOrigin::NetSocket,
        max_call_bytes: 0,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::NetTcpClose,
        callee: "net_tcp_close",
        symbol: "auto.platform.net.tcp_close.v1",
        category: CATEGORY_NET,
        capability: CAP_NET_CLOSE,
        params: P_I32,
        param_dirs: D_1,
        len_unit: LenUnit::None,
        origin: HandleOrigin::NetSocket,
        max_call_bytes: 0,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::NetTcpListen,
        callee: "net_tcp_listen",
        symbol: "auto.platform.net.tcp_listen.v1",
        category: CATEGORY_NET,
        capability: CAP_NET_LISTEN,
        params: P_I32,
        param_dirs: D_1,
        len_unit: LenUnit::None,
        origin: HandleOrigin::None,
        max_call_bytes: 0,
    },
    // ---- PLAN-973 system-resource operations -------------------------------
    PlatformOpSpec {
        kind: PlatformOpKind::IoStdin,
        callee: "io_stdin",
        symbol: "auto.platform.io.stdin.v1",
        category: CATEGORY_IO,
        capability: CAP_STDIN,
        params: P_EMPTY,
        param_dirs: D_0,
        len_unit: LenUnit::None,
        origin: HandleOrigin::StdStream,
        max_call_bytes: 0,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::FileOpenW16,
        callee: "file_open_w16",
        symbol: "auto.platform.file.open_w16.v1",
        category: CATEGORY_FILE,
        capability: CAP_FILE,
        params: P_4I32,
        param_dirs: D_4,
        len_unit: LenUnit::None,
        origin: HandleOrigin::OsFile,
        max_call_bytes: 0,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::FileOpenBytes,
        callee: "file_open_bytes",
        symbol: "auto.platform.file.open_bytes.v1",
        category: CATEGORY_FILE,
        capability: CAP_FILE,
        params: P_STR_I32,
        param_dirs: D_2,
        len_unit: LenUnit::ByteCount,
        origin: HandleOrigin::OsFile,
        max_call_bytes: MAX_PROBE_PATH_BYTES as u32,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::FileReadSome,
        callee: "file_read_some",
        symbol: "auto.platform.file.read_some.v1",
        category: CATEGORY_FILE,
        capability: CAP_FILE,
        params: P_4I32,
        param_dirs: D_4,
        len_unit: LenUnit::ByteCount,
        origin: HandleOrigin::OsFile,
        max_call_bytes: MAX_PLATFORM_CALL_BYTES as u32,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::FileWriteSome,
        callee: "file_write_some",
        symbol: "auto.platform.file.write_some.v1",
        category: CATEGORY_FILE,
        capability: CAP_FILE,
        params: P_4I32,
        param_dirs: D_4,
        len_unit: LenUnit::ByteCount,
        origin: HandleOrigin::OsFile,
        max_call_bytes: MAX_PLATFORM_CALL_BYTES as u32,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::FileSeek,
        callee: "file_seek",
        symbol: "auto.platform.file.seek.v1",
        category: CATEGORY_FILE,
        capability: CAP_FILE,
        params: P_4I32,
        param_dirs: D_4,
        len_unit: LenUnit::None,
        origin: HandleOrigin::OsFile,
        max_call_bytes: 0,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::FileTell,
        callee: "file_tell",
        symbol: "auto.platform.file.tell.v1",
        category: CATEGORY_FILE,
        capability: CAP_FILE,
        params: P_I32,
        param_dirs: D_1,
        len_unit: LenUnit::None,
        origin: HandleOrigin::OsFile,
        max_call_bytes: 0,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::FileHi,
        callee: "file_hi",
        symbol: "auto.platform.file.hi.v1",
        category: CATEGORY_FILE,
        capability: CAP_FILE,
        params: P_I32,
        param_dirs: D_1,
        len_unit: LenUnit::None,
        origin: HandleOrigin::OsFile,
        max_call_bytes: 0,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::FileSize,
        callee: "file_size",
        symbol: "auto.platform.file.size.v1",
        category: CATEGORY_FILE,
        capability: CAP_FILE,
        params: P_I32,
        param_dirs: D_1,
        len_unit: LenUnit::ByteCount,
        origin: HandleOrigin::OsFile,
        max_call_bytes: 0,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::FileMtimeMs,
        callee: "file_mtime_ms",
        symbol: "auto.platform.file.mtime_ms.v1",
        category: CATEGORY_FILE,
        capability: CAP_FILE,
        params: P_I32,
        param_dirs: D_1,
        len_unit: LenUnit::None,
        origin: HandleOrigin::OsFile,
        max_call_bytes: 0,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::FileFlush,
        callee: "file_flush",
        symbol: "auto.platform.file.flush.v1",
        category: CATEGORY_FILE,
        capability: CAP_FILE,
        params: P_I32,
        param_dirs: D_1,
        len_unit: LenUnit::None,
        origin: HandleOrigin::OsFile,
        max_call_bytes: 0,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::FileClose,
        callee: "file_close",
        symbol: "auto.platform.file.close.v1",
        category: CATEGORY_FILE,
        capability: CAP_FILE,
        params: P_I32,
        param_dirs: D_1,
        len_unit: LenUnit::None,
        origin: HandleOrigin::OsFile,
        max_call_bytes: 0,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::PathAttrsW16,
        callee: "path_attrs_w16",
        symbol: "auto.platform.file.path_attrs_w16.v1",
        category: CATEGORY_FILE,
        capability: CAP_PATH,
        params: P_3I32,
        param_dirs: D_3,
        len_unit: LenUnit::None,
        origin: HandleOrigin::None,
        max_call_bytes: 0,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::PathAttrsBytes,
        callee: "path_attrs_bytes",
        symbol: "auto.platform.file.path_attrs_bytes.v1",
        category: CATEGORY_FILE,
        capability: CAP_PATH,
        params: P_STR,
        param_dirs: D_1,
        len_unit: LenUnit::ByteCount,
        origin: HandleOrigin::None,
        max_call_bytes: MAX_PROBE_PATH_BYTES as u32,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::FileDeleteW16,
        callee: "file_delete_w16",
        symbol: "auto.platform.file.delete_w16.v1",
        category: CATEGORY_FILE,
        capability: CAP_PATH,
        params: P_3I32,
        param_dirs: D_3,
        len_unit: LenUnit::None,
        origin: HandleOrigin::None,
        max_call_bytes: 0,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::FileDeleteBytes,
        callee: "file_delete_bytes",
        symbol: "auto.platform.file.delete_bytes.v1",
        category: CATEGORY_FILE,
        capability: CAP_PATH,
        params: P_STR,
        param_dirs: D_1,
        len_unit: LenUnit::ByteCount,
        origin: HandleOrigin::None,
        max_call_bytes: MAX_PROBE_PATH_BYTES as u32,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::DirCreateW16,
        callee: "dir_create_w16",
        symbol: "auto.platform.file.dir_create_w16.v1",
        category: CATEGORY_FILE,
        capability: CAP_PATH,
        params: P_3I32,
        param_dirs: D_3,
        len_unit: LenUnit::None,
        origin: HandleOrigin::None,
        max_call_bytes: 0,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::DirCreateBytes,
        callee: "dir_create_bytes",
        symbol: "auto.platform.file.dir_create_bytes.v1",
        category: CATEGORY_FILE,
        capability: CAP_PATH,
        params: P_STR,
        param_dirs: D_1,
        len_unit: LenUnit::ByteCount,
        origin: HandleOrigin::None,
        max_call_bytes: MAX_PROBE_PATH_BYTES as u32,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::DirDeleteW16,
        callee: "dir_delete_w16",
        symbol: "auto.platform.file.dir_delete_w16.v1",
        category: CATEGORY_FILE,
        capability: CAP_PATH,
        params: P_3I32,
        param_dirs: D_3,
        len_unit: LenUnit::None,
        origin: HandleOrigin::None,
        max_call_bytes: 0,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::DirDeleteBytes,
        callee: "dir_delete_bytes",
        symbol: "auto.platform.file.dir_delete_bytes.v1",
        category: CATEGORY_FILE,
        capability: CAP_PATH,
        params: P_STR,
        param_dirs: D_1,
        len_unit: LenUnit::ByteCount,
        origin: HandleOrigin::None,
        max_call_bytes: MAX_PROBE_PATH_BYTES as u32,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::FileReplaceW16,
        callee: "file_replace_w16",
        symbol: "auto.platform.file.replace_w16.v1",
        category: CATEGORY_FILE,
        capability: CAP_PATH,
        params: P_6I32,
        param_dirs: D_6,
        len_unit: LenUnit::None,
        origin: HandleOrigin::None,
        max_call_bytes: 0,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::FileReplaceBytes,
        callee: "file_replace_bytes",
        symbol: "auto.platform.file.replace_bytes.v1",
        category: CATEGORY_FILE,
        capability: CAP_PATH,
        params: P_2STR,
        param_dirs: D_2,
        len_unit: LenUnit::ByteCount,
        origin: HandleOrigin::None,
        max_call_bytes: 2 * MAX_PROBE_PATH_BYTES as u32,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::CanonicalW16,
        callee: "canonical_w16",
        symbol: "auto.platform.file.canonical_w16.v1",
        category: CATEGORY_FILE,
        capability: CAP_PATH,
        params: P_5I32,
        param_dirs: D_5,
        len_unit: LenUnit::None,
        origin: HandleOrigin::None,
        max_call_bytes: 0,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::CanonicalBytes,
        callee: "canonical_bytes",
        symbol: "auto.platform.file.canonical_bytes.v1",
        category: CATEGORY_FILE,
        capability: CAP_PATH,
        params: P_STR_2I32,
        param_dirs: D_3,
        len_unit: LenUnit::ByteCount,
        origin: HandleOrigin::None,
        max_call_bytes: MAX_PROBE_PATH_BYTES as u32,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::DirOpenW16,
        callee: "dir_open_w16",
        symbol: "auto.platform.file.dir_open_w16.v1",
        category: CATEGORY_FILE,
        capability: CAP_PATH,
        params: P_3I32,
        param_dirs: D_3,
        len_unit: LenUnit::None,
        origin: HandleOrigin::OsDir,
        max_call_bytes: 0,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::DirNextW16,
        callee: "dir_next_w16",
        symbol: "auto.platform.file.dir_next_w16.v1",
        category: CATEGORY_FILE,
        capability: CAP_PATH,
        params: P_3I32,
        param_dirs: D_3,
        len_unit: LenUnit::None,
        origin: HandleOrigin::OsDir,
        max_call_bytes: 0,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::DirClose,
        callee: "dir_close",
        symbol: "auto.platform.file.dir_close.v1",
        category: CATEGORY_FILE,
        capability: CAP_PATH,
        params: P_I32,
        param_dirs: D_1,
        len_unit: LenUnit::None,
        origin: HandleOrigin::OsDir,
        max_call_bytes: 0,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::DirOpenBytes,
        callee: "dir_open_bytes",
        symbol: "auto.platform.file.dir_open_bytes.v1",
        category: CATEGORY_FILE,
        capability: CAP_PATH,
        params: P_STR,
        param_dirs: D_1,
        len_unit: LenUnit::ByteCount,
        origin: HandleOrigin::OsDir,
        max_call_bytes: MAX_PROBE_PATH_BYTES as u32,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::DirFillBytes,
        callee: "dir_fill_bytes",
        symbol: "auto.platform.file.dir_fill_bytes.v1",
        category: CATEGORY_FILE,
        capability: CAP_PATH,
        params: P_3I32,
        param_dirs: D_3,
        len_unit: LenUnit::ByteCount,
        origin: HandleOrigin::OsDir,
        max_call_bytes: MAX_PLATFORM_CALL_BYTES as u32,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::EnvGetW16,
        callee: "env_get_w16",
        symbol: "auto.platform.env.get_w16.v1",
        category: CATEGORY_ENV,
        capability: CAP_ENV,
        params: P_5I32,
        param_dirs: D_5,
        len_unit: LenUnit::None,
        origin: HandleOrigin::None,
        max_call_bytes: 0,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::EnvSetW16,
        callee: "env_set_w16",
        symbol: "auto.platform.env.set_w16.v1",
        category: CATEGORY_ENV,
        capability: CAP_ENV,
        params: P_6I32,
        param_dirs: D_6,
        len_unit: LenUnit::None,
        origin: HandleOrigin::None,
        max_call_bytes: 0,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::EnvRemoveW16,
        callee: "env_remove_w16",
        symbol: "auto.platform.env.remove_w16.v1",
        category: CATEGORY_ENV,
        capability: CAP_ENV,
        params: P_3I32,
        param_dirs: D_3,
        len_unit: LenUnit::None,
        origin: HandleOrigin::None,
        max_call_bytes: 0,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::EnvGetBytes,
        callee: "env_get_bytes",
        symbol: "auto.platform.env.get_bytes.v1",
        category: CATEGORY_ENV,
        capability: CAP_ENV,
        params: P_3I32,
        param_dirs: D_3,
        len_unit: LenUnit::ByteCount,
        origin: HandleOrigin::None,
        max_call_bytes: MAX_PLATFORM_CALL_BYTES as u32,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::ClockUnixMs,
        callee: "clock_unix_ms",
        symbol: "auto.platform.clock.unix_ms.v1",
        category: CATEGORY_CLOCK,
        capability: CAP_CLOCK,
        params: P_EMPTY,
        param_dirs: D_0,
        len_unit: LenUnit::None,
        origin: HandleOrigin::None,
        max_call_bytes: 0,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::ClockMonoMs,
        callee: "clock_mono_ms",
        symbol: "auto.platform.clock.mono_ms.v1",
        category: CATEGORY_CLOCK,
        capability: CAP_CLOCK,
        params: P_EMPTY,
        param_dirs: D_0,
        len_unit: LenUnit::None,
        origin: HandleOrigin::None,
        max_call_bytes: 0,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::ClockHi,
        callee: "clock_hi",
        symbol: "auto.platform.clock.hi.v1",
        category: CATEGORY_CLOCK,
        capability: CAP_CLOCK,
        params: P_EMPTY,
        param_dirs: D_0,
        len_unit: LenUnit::None,
        origin: HandleOrigin::None,
        max_call_bytes: 0,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::SleepMs,
        callee: "sleep_ms",
        symbol: "auto.platform.clock.sleep_ms.v1",
        category: CATEGORY_CLOCK,
        capability: CAP_CLOCK,
        params: P_I32,
        param_dirs: D_1,
        len_unit: LenUnit::None,
        origin: HandleOrigin::None,
        max_call_bytes: 0,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::NetTcpAccept,
        callee: "net_tcp_accept",
        symbol: "auto.platform.net.tcp_accept.v1",
        category: CATEGORY_NET,
        capability: CAP_NET_LISTEN,
        params: P_I32,
        param_dirs: D_1,
        len_unit: LenUnit::None,
        origin: HandleOrigin::NetSocket,
        max_call_bytes: 0,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::NetTcpLocalPort,
        callee: "net_tcp_local_port",
        symbol: "auto.platform.net.tcp_local_port.v1",
        category: CATEGORY_NET,
        capability: CAP_NET_LISTEN,
        params: P_I32,
        param_dirs: D_1,
        len_unit: LenUnit::None,
        origin: HandleOrigin::NetSocket,
        max_call_bytes: 0,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::RandFill,
        callee: "rand_fill",
        symbol: "auto.platform.entropy.rand_fill.v1",
        category: CATEGORY_ENTROPY,
        capability: CAP_ENTROPY,
        params: P_3I32,
        param_dirs: D_3,
        len_unit: LenUnit::ByteCount,
        origin: HandleOrigin::OsMemRegion,
        max_call_bytes: MAX_PLATFORM_CALL_BYTES as u32,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::ProcSpawnW16,
        callee: "proc_spawn_w16",
        symbol: "auto.platform.process.spawn_w16.v1",
        category: CATEGORY_PROC,
        capability: CAP_PROC,
        params: P_4I32,
        param_dirs: D_4,
        len_unit: LenUnit::None,
        origin: HandleOrigin::OsProcess,
        max_call_bytes: 0,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::ProcPipeRead,
        callee: "proc_pipe_read",
        symbol: "auto.platform.process.pipe_read.v1",
        category: CATEGORY_PROC,
        capability: CAP_PROC,
        params: P_6I32,
        param_dirs: D_6,
        len_unit: LenUnit::ByteCount,
        origin: HandleOrigin::OsProcess,
        max_call_bytes: MAX_PLATFORM_CALL_BYTES as u32,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::ProcWait,
        callee: "proc_wait",
        symbol: "auto.platform.process.wait.v1",
        category: CATEGORY_PROC,
        capability: CAP_PROC,
        params: P_2I32,
        param_dirs: D_2,
        len_unit: LenUnit::None,
        origin: HandleOrigin::OsProcess,
        max_call_bytes: 0,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::ProcRelease,
        callee: "proc_release",
        symbol: "auto.platform.process.release.v1",
        category: CATEGORY_PROC,
        capability: CAP_PROC,
        params: P_I32,
        param_dirs: D_1,
        len_unit: LenUnit::None,
        origin: HandleOrigin::OsProcess,
        max_call_bytes: 0,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::ProcExit,
        callee: "proc_exit",
        symbol: "auto.platform.process.exit.v1",
        category: CATEGORY_PROC,
        capability: CAP_PROC,
        params: P_I32,
        param_dirs: D_1,
        len_unit: LenUnit::None,
        origin: HandleOrigin::None,
        max_call_bytes: 0,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::ProcCmdlineW16,
        callee: "proc_cmdline_w16",
        symbol: "auto.platform.process.cmdline_w16.v1",
        category: CATEGORY_PROC,
        capability: CAP_PROC,
        params: P_2I32,
        param_dirs: D_2,
        len_unit: LenUnit::None,
        origin: HandleOrigin::None,
        max_call_bytes: 0,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::ProcCmdlineFillBytes,
        callee: "proc_cmdline_fill_bytes",
        symbol: "auto.platform.process.cmdline_fill_bytes.v1",
        category: CATEGORY_PROC,
        capability: CAP_PROC,
        params: P_3I32,
        param_dirs: D_3,
        len_unit: LenUnit::ByteCount,
        origin: HandleOrigin::None,
        max_call_bytes: MAX_PLATFORM_CALL_BYTES as u32,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::ProcCwdW16,
        callee: "proc_cwd_w16",
        symbol: "auto.platform.process.cwd_w16.v1",
        category: CATEGORY_PROC,
        capability: CAP_PROC,
        params: P_2I32,
        param_dirs: D_2,
        len_unit: LenUnit::None,
        origin: HandleOrigin::None,
        max_call_bytes: 0,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::ProcCwdBytes,
        callee: "proc_cwd_bytes",
        symbol: "auto.platform.process.cwd_bytes.v1",
        category: CATEGORY_PROC,
        capability: CAP_PROC,
        params: P_2I32,
        param_dirs: D_2,
        len_unit: LenUnit::ByteCount,
        origin: HandleOrigin::None,
        max_call_bytes: MAX_PROBE_PATH_BYTES as u32,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::ProcChdirW16,
        callee: "proc_chdir_w16",
        symbol: "auto.platform.process.chdir_w16.v1",
        category: CATEGORY_PROC,
        capability: CAP_PROC,
        params: P_3I32,
        param_dirs: D_3,
        len_unit: LenUnit::None,
        origin: HandleOrigin::None,
        max_call_bytes: 0,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::ProcChdirBytes,
        callee: "proc_chdir_bytes",
        symbol: "auto.platform.process.chdir_bytes.v1",
        category: CATEGORY_PROC,
        capability: CAP_PROC,
        params: P_STR,
        param_dirs: D_1,
        len_unit: LenUnit::ByteCount,
        origin: HandleOrigin::None,
        max_call_bytes: MAX_PROBE_PATH_BYTES as u32,
    },
    PlatformOpSpec {
        kind: PlatformOpKind::ProcSpawnBytes,
        callee: "proc_spawn_bytes",
        symbol: "auto.platform.process.spawn_bytes.v1",
        category: CATEGORY_PROC,
        capability: CAP_PROC,
        params: P_STR_I32,
        param_dirs: D_2,
        len_unit: LenUnit::ByteCount,
        origin: HandleOrigin::OsProcess,
        max_call_bytes: MAX_PLATFORM_CALL_BYTES as u32,
    },
];

pub fn lookup_callee(callee: &str) -> Option<&'static PlatformOpSpec> {
    PLATFORM_OPS.iter().find(|op| op.callee == callee)
}

pub fn lookup_symbol(symbol: &str) -> Option<&'static PlatformOpSpec> {
    PLATFORM_OPS.iter().find(|op| op.symbol == symbol)
}

pub fn is_platform_symbol(symbol: &str) -> bool {
    symbol.starts_with("auto.platform.")
}

pub fn declaration(symbol: &str, sig: u32) -> Result<MirHostImport, AdmissionError> {
    let spec = lookup_symbol(symbol).ok_or_else(|| AdmissionError {
        code: "host.unknown-symbol",
        message: "unknown platform operation symbol".into(),
    })?;
    Ok(MirHostImport {
        category: spec.category.into(),
        symbol: spec.symbol.into(),
        sig,
        capability: spec.capability.into(),
        max_call_bytes: spec.max_call_bytes,
    })
}

pub fn validate_import(
    module: &MirModule,
    h: &MirHostImport,
    spec: &PlatformOpSpec,
) -> Result<(), AdmissionError> {
    let sig = module.sig_of(h.sig).ok_or_else(|| AdmissionError {
        code: "host.signature",
        message: "platform operation signature is not a function type".into(),
    })?;
    let params_ok = sig.params.len() == spec.params.len()
        && sig
            .params
            .iter()
            .zip(spec.params.iter())
            .all(|(p, expected)| module.types.get(*p as usize) == Some(expected));
    let ret_ok = module.types.get(sig.result as usize) == Some(&MirType::I32);
    if h.category != spec.category
        || h.capability != spec.capability
        || h.max_call_bytes != spec.max_call_bytes
        || !params_ok
        || !ret_ok
    {
        return Err(AdmissionError {
            code: "host.signature",
            message: "platform operation category/symbol/signature/capability/budget mismatch"
                .into(),
        });
    }
    if !module.capabilities.iter().any(|c| c == &h.capability) {
        return Err(AdmissionError {
            code: "host.unauthorized",
            message: "platform capability requirement is missing".into(),
        });
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Unified Host Resource Ledger + Win32 Provider
// ---------------------------------------------------------------------------

pub trait PlatformIoBackend {
    fn std_handle(&mut self, stream: StreamKind) -> Result<usize, u32>;
    fn write_some(
        &mut self,
        stream: StreamKind,
        os_handle: usize,
        bytes: &[u8],
        cancel: &CancelFlag,
    ) -> Result<usize, u32>;
    fn system_info(&mut self) -> (u32, u32);
    fn virtual_alloc(&mut self, commit_bytes: usize) -> Result<*mut u8, u32>;
    /// PLAN-1040: `mapped_len` is the exact mapping length the ledger
    /// recorded at alloc time. The Win32 implementation ignores it
    /// (`VirtualFree(base, 0, MEM_RELEASE)` releases the whole region by
    /// base); the Linux implementation needs it — `munmap(base, len)`
    /// takes no "whole region" form.
    fn virtual_free(&mut self, base: *mut u8, mapped_len: usize) -> Result<(), u32>;
    /// PLAN-1040 probe seam. `path` is the raw guest byte slice WITHOUT a
    /// NUL terminator; the backend appends one on the OS boundary. Returns
    /// the raw positive fd or the raw NEGATIVE Linux errno value. The
    /// default reports `PROBE_UNSUPPORTED` (no probe on this platform).
    fn probe_openat(
        &mut self,
        _dirfd: i32,
        _path: &[u8],
        _flags: i32,
        _mode: i32,
    ) -> Result<i32, u32> {
        Err(PROBE_UNSUPPORTED)
    }
    fn probe_write_fd(&mut self, _fd: i32, _bytes: &[u8]) -> Result<i32, u32> {
        Err(PROBE_UNSUPPORTED)
    }
    fn probe_close(&mut self, _fd: i32) -> Result<i32, u32> {
        Err(PROBE_UNSUPPORTED)
    }
    /// PLAN-1041: classify a std-stream OS handle. `Ok(1)` = Windows text
    /// console, `Ok(2)` = byte sink (file/pipe/Linux stream). `Err` carries
    /// the raw OS error for REAL classification failures (e.g. access
    /// denied); "not a console" is `Ok(2)`, never an error. The default
    /// reports byte-sink (platforms without a text-console API).
    fn console_kind(&mut self, _os_handle: usize) -> Result<i32, u32> {
        Ok(CONSOLE_KIND_BYTE_SINK)
    }
    /// PLAN-1041: write UTF-16LE code units to a text console via
    /// `WriteConsoleW`. Returns units written (real OS progress; may be
    /// fewer than requested — the caller keeps unit-level state). The
    /// default reports the unsupported marker (no console write here).
    fn console_write16(
        &mut self,
        _stream: StreamKind,
        _os_handle: usize,
        _units: &[u16],
        _cancel: &CancelFlag,
    ) -> Result<usize, u32> {
        Err(CONSOLE_WRITE16_UNSUPPORTED)
    }
    /// PLAN-974 (T-02): the TCP backend face. `host` is the raw guest
    /// byte slice (no NUL). Returns a backend-owned socket id (stored in
    /// the ledger entry, mirroring `os_handle`) or the raw OS error.
    fn net_connect(&mut self, _host: &[u8], _port: i32) -> Result<u64, u32> {
        Err(NET_TCP_UNSUPPORTED)
    }
    /// Write bytes to a backend socket; returns real OS progress.
    fn net_write_some(
        &mut self,
        _socket: u64,
        _bytes: &[u8],
        _cancel: &CancelFlag,
    ) -> Result<usize, u32> {
        Err(NET_TCP_UNSUPPORTED)
    }
    /// Read into `buf`; `Ok(0)` = orderly EOF. Returns bytes read.
    fn net_read_some(&mut self, _socket: u64, _buf: &mut [u8]) -> Result<usize, u32> {
        Err(NET_TCP_UNSUPPORTED)
    }
    /// Close a backend socket; the entry release is ledger-owned.
    fn net_close(&mut self, _socket: u64) -> Result<(), u32> {
        Err(NET_TCP_UNSUPPORTED)
    }
    /// PLAN-974 inc3: bind a loopback listener (port 0 = ephemeral);
    /// returns a backend-owned listener id.
    fn net_listen(&mut self, _port: i32) -> Result<u64, u32> {
        Err(NET_TCP_UNSUPPORTED)
    }
    /// PLAN-974 inc3: accept one connection on a listener; returns a
    /// backend socket id (the ledger opens a FRESH entry for it).
    fn net_accept(&mut self, _listener: u64) -> Result<u64, u32> {
        Err(NET_TCP_UNSUPPORTED)
    }
    /// PLAN-974 inc3: the listener's actual bound port.
    fn net_local_port(&mut self, _listener: u64) -> Result<i32, u32> {
        Err(NET_TCP_UNSUPPORTED)
    }

    // ---- PLAN-973 system-resource backend surface --------------------------
    // Unit/layout conventions (frozen in the plan T-01 record):
    // - `*_w16` ops: paths/values staged as UTF-16LE code units (the
    //   shared Auto layer converted them; the backend appends the NUL
    //   terminator on the OS boundary only).
    // - `*_bytes` ops: raw guest path bytes WITHOUT a terminator.
    // - open flags: the uniform `OPEN_*` bit set (read/write/create/
    //   truncate/exclusive/append are separate switches).
    // - attrs bits: 1 = directory, 2 = read-only.
    // Defaults report `PROBE_UNSUPPORTED` so each backend implements
    // exactly the surface it genuinely serves (no silent cross-fallback).

    /// Windows file open (`CreateFileW`) → raw OS handle.
    fn file_open_w16(&mut self, _units: &[u16], _flags: i32) -> Result<usize, u32> {
        Err(PROBE_UNSUPPORTED)
    }
    /// Linux file open (`openat`, O_CLOEXEC always) → raw fd.
    fn file_open_bytes(&mut self, _path: &[u8], _flags: i32) -> Result<usize, u32> {
        Err(PROBE_UNSUPPORTED)
    }
    /// Read some bytes into `buf` (`ReadFile`/`read`). `Ok(0)` on a
    /// nonempty request is EOF; a zero-capacity request never implies EOF.
    fn file_read_some(
        &mut self,
        _os_handle: usize,
        _buf: &mut [u8],
        _cancel: &CancelFlag,
    ) -> Result<usize, u32> {
        Err(PROBE_UNSUPPORTED)
    }
    /// Write some bytes (`WriteFile`/`write`) → real byte progress.
    fn file_write_some(
        &mut self,
        _os_handle: usize,
        _bytes: &[u8],
        _cancel: &CancelFlag,
    ) -> Result<usize, u32> {
        Err(PROBE_UNSUPPORTED)
    }
    /// Seek (`SetFilePointerEx`/`lseek`); origin 0=set 1=cur 2=end.
    fn file_seek(&mut self, _os_handle: usize, _origin: i32, _offset: i64) -> Result<i64, u32> {
        Err(PROBE_UNSUPPORTED)
    }
    /// Current file position.
    fn file_tell(&mut self, _os_handle: usize) -> Result<i64, u32> {
        Err(PROBE_UNSUPPORTED)
    }
    /// File size in bytes.
    fn file_size(&mut self, _os_handle: usize) -> Result<u64, u32> {
        Err(PROBE_UNSUPPORTED)
    }
    /// Last-write time as unix milliseconds.
    fn file_mtime_ms(&mut self, _os_handle: usize) -> Result<i64, u32> {
        Err(PROBE_UNSUPPORTED)
    }
    /// Flush OS buffers to durable storage.
    fn file_flush(&mut self, _os_handle: usize) -> Result<(), u32> {
        Err(PROBE_UNSUPPORTED)
    }
    /// Close a raw file handle; close errors surface here.
    fn file_close(&mut self, _os_handle: usize) -> Result<(), u32> {
        Err(PROBE_UNSUPPORTED)
    }
    /// Windows path attribute bits (1=dir, 2=readonly).
    fn path_attrs_w16(&mut self, _units: &[u16]) -> Result<i32, u32> {
        Err(PROBE_UNSUPPORTED)
    }
    /// Linux path attribute bits.
    fn path_attrs_bytes(&mut self, _path: &[u8]) -> Result<i32, u32> {
        Err(PROBE_UNSUPPORTED)
    }
    fn file_delete_w16(&mut self, _units: &[u16]) -> Result<(), u32> {
        Err(PROBE_UNSUPPORTED)
    }
    fn file_delete_bytes(&mut self, _path: &[u8]) -> Result<(), u32> {
        Err(PROBE_UNSUPPORTED)
    }
    fn dir_create_w16(&mut self, _units: &[u16]) -> Result<(), u32> {
        Err(PROBE_UNSUPPORTED)
    }
    fn dir_create_bytes(&mut self, _path: &[u8]) -> Result<(), u32> {
        Err(PROBE_UNSUPPORTED)
    }
    /// Delete an EMPTY directory (RemoveDirectoryW / unlinkat AT_REMOVEDIR).
    fn dir_delete_w16(&mut self, _units: &[u16]) -> Result<(), u32> {
        Err(PROBE_UNSUPPORTED)
    }
    fn dir_delete_bytes(&mut self, _path: &[u8]) -> Result<(), u32> {
        Err(PROBE_UNSUPPORTED)
    }
    /// Atomic replace (`MoveFileExW`/`renameat`): dest replaced in one OS
    /// step; a failure leaves the old file untouched.
    fn file_replace_w16(&mut self, _src: &[u16], _dst: &[u16]) -> Result<(), u32> {
        Err(PROBE_UNSUPPORTED)
    }
    fn file_replace_bytes(&mut self, _src: &[u8], _dst: &[u8]) -> Result<(), u32> {
        Err(PROBE_UNSUPPORTED)
    }
    /// Canonical path into `out` → written unit count. Windows resolves
    /// the `\\?\`-prefixed final path; Linux does the O_PATH+readlink
    /// component walk.
    fn canonical_w16(&mut self, _units: &[u16], _out: &mut [u16]) -> Result<usize, u32> {
        Err(PROBE_UNSUPPORTED)
    }
    fn canonical_bytes(&mut self, _path: &[u8], _out: &mut [u8]) -> Result<usize, u32> {
        Err(PROBE_UNSUPPORTED)
    }
    /// Windows directory open (`FindFirstFileW` on `<dir>\*`) → handle
    /// plus the first entry name (drained by the first `dir_next_w16`).
    fn dir_open_w16(&mut self, _units: &[u16]) -> Result<(usize, Option<Vec<u16>>), u32> {
        Err(PROBE_UNSUPPORTED)
    }
    /// Next entry name into `out` → unit count; `Ok(0)` = end.
    fn dir_next_w16(&mut self, _os_handle: usize, _out: &mut [u16]) -> Result<usize, u32> {
        Err(PROBE_UNSUPPORTED)
    }
    fn dir_close(&mut self, _os_handle: usize) -> Result<(), u32> {
        Err(PROBE_UNSUPPORTED)
    }
    /// Linux directory open (O_RDONLY|O_DIRECTORY) → fd.
    fn dir_open_bytes(&mut self, _path: &[u8]) -> Result<usize, u32> {
        Err(PROBE_UNSUPPORTED)
    }
    /// Linux batched `getdents64` into `buf` → byte count filled.
    fn dir_fill_bytes(&mut self, _os_handle: usize, _buf: &mut [u8]) -> Result<usize, u32> {
        Err(PROBE_UNSUPPORTED)
    }
    /// Windows env get → value unit count; `Ok(0)` = unset.
    fn env_get_w16(&mut self, _name: &[u16], _out: &mut [u16]) -> Result<usize, u32> {
        Err(PROBE_UNSUPPORTED)
    }
    /// Windows env set (empty value removes).
    fn env_set_w16(&mut self, _name: &[u16], _value: &[u16]) -> Result<(), u32> {
        Err(PROBE_UNSUPPORTED)
    }
    fn env_remove_w16(&mut self, _name: &[u16]) -> Result<(), u32> {
        Err(PROBE_UNSUPPORTED)
    }
    /// Linux env snapshot via /proc/self/environ → byte count filled.
    fn env_get_bytes(&mut self, _out: &mut [u8]) -> Result<usize, u32> {
        Err(PROBE_UNSUPPORTED)
    }
    /// Wall clock (epoch ms).
    fn clock_unix_ms(&mut self) -> Result<i64, u32> {
        Err(PROBE_UNSUPPORTED)
    }
    /// Monotonic clock (ms) — deadlines/sleeps only.
    fn clock_mono_ms(&mut self) -> Result<i64, u32> {
        Err(PROBE_UNSUPPORTED)
    }
    /// Sleep whole ms, cancellation-checked between bounded slices.
    fn sleep_ms(&mut self, _ms: u32, _cancel: &CancelFlag) -> Result<(), u32> {
        Err(PROBE_UNSUPPORTED)
    }
    /// CSPRNG fill (`BCryptGenRandom`/`getrandom`).
    fn rand_fill(&mut self, _buf: &mut [u8]) -> Result<(), u32> {
        Err(PROBE_UNSUPPORTED)
    }
    /// Windows spawn (`CreateProcessW` + job kill-on-close; capture pipes
    /// when requested — the parent's write ends are closed here so EOF is
    /// observable). Returns the raw process/job/pipe-read handles.
    fn proc_spawn_w16(&mut self, _cmdline: &[u16], _capture: bool) -> Result<ProcSpawn, u32> {
        Err(PROBE_UNSUPPORTED)
    }
    /// Linux spawn (NUL-separated argv blob + mode bit0 = capture).
    fn proc_spawn_bytes(&mut self, _argv_blob: &[u8], _capture: bool) -> Result<ProcSpawn, u32> {
        Err(PROBE_UNSUPPORTED)
    }
    /// Bytes currently available on a capture pipe (`PeekNamedPipe`).
    fn pipe_avail(&mut self, _os_handle: usize) -> Result<usize, u32> {
        Err(PROBE_UNSUPPORTED)
    }
    /// Wait for the child process handle: `Ok(true)` = exited within the
    /// timeout, `Ok(false)` = timeout (child NOT killed — the caller owns
    /// the V-03 kill contract). `timeout_ms < 0` = wait forever.
    fn wait_child(&mut self, _os_handle: usize, _timeout_ms: i32) -> Result<bool, u32> {
        Err(PROBE_UNSUPPORTED)
    }
    /// Child exit code after a signaled wait.
    fn child_exit_code(&mut self, _os_handle: usize) -> Result<i32, u32> {
        Err(PROBE_UNSUPPORTED)
    }
    /// Kill the whole job tree (`TerminateJobObject`) — grandchildren
    /// included (V-03/V-08 reclamation contract).
    fn kill_tree(&mut self, _job_handle: usize) -> Result<(), u32> {
        Err(PROBE_UNSUPPORTED)
    }
    /// Close a raw non-file handle (job / pipe ends).
    fn close_raw(&mut self, _os_handle: usize) -> Result<(), u32> {
        Err(PROBE_UNSUPPORTED)
    }
    /// Windows full command line → unit count (shared Auto parses quotes).
    fn proc_cmdline_w16(&mut self, _out: &mut [u16]) -> Result<usize, u32> {
        Err(PROBE_UNSUPPORTED)
    }
    /// Linux /proc/self/cmdline fill → byte count.
    fn proc_cmdline_fill_bytes(&mut self, _out: &mut [u8]) -> Result<usize, u32> {
        Err(PROBE_UNSUPPORTED)
    }
    fn proc_cwd_w16(&mut self, _out: &mut [u16]) -> Result<usize, u32> {
        Err(PROBE_UNSUPPORTED)
    }
    fn proc_cwd_bytes(&mut self, _out: &mut [u8]) -> Result<usize, u32> {
        Err(PROBE_UNSUPPORTED)
    }
    fn proc_chdir_w16(&mut self, _units: &[u16]) -> Result<(), u32> {
        Err(PROBE_UNSUPPORTED)
    }
    fn proc_chdir_bytes(&mut self, _path: &[u8]) -> Result<(), u32> {
        Err(PROBE_UNSUPPORTED)
    }
}

/// PLAN-973: raw OS handle bundle returned by `proc_spawn_w16` (the
/// ledger takes ownership; `write` ends are already closed in the parent
/// so pipe EOF tracks child exit).
#[derive(Clone, Copy, Debug, Default)]
pub struct ProcSpawn {
    pub process: usize,
    pub job: usize,
    pub out_read: usize,
    pub err_read: usize,
}

#[cfg(windows)]
mod win32 {
    pub const MEM_COMMIT: u32 = 0x1000;
    pub const MEM_RESERVE: u32 = 0x2000;
    pub const MEM_RELEASE: u32 = 0x8000;
    pub const PAGE_READWRITE: u32 = 0x04;
    pub const ERROR_INVALID_HANDLE: u32 = 6;
    pub const ERROR_ACCESS_DENIED: u32 = 5;
    // PLAN-973 flag constants (public Win32 semantics; no CRT).
    pub const GENERIC_READ: u32 = 0x8000_0000;
    pub const GENERIC_WRITE: u32 = 0x4000_0000;
    pub const FILE_SHARE_READ: u32 = 0x1;
    pub const FILE_SHARE_WRITE: u32 = 0x2;
    pub const FILE_SHARE_DELETE: u32 = 0x4;
    pub const CREATE_NEW: u32 = 1;
    pub const CREATE_ALWAYS: u32 = 2;
    pub const OPEN_EXISTING: u32 = 3;
    pub const OPEN_ALWAYS: u32 = 4;
    pub const TRUNCATE_EXISTING: u32 = 5;
    pub const INVALID_HANDLE_VALUE: isize = -1;
    pub const INVALID_FILE_ATTRIBUTES: u32 = 0xFFFF_FFFF;
    pub const FILE_ATTRIBUTE_READONLY: u32 = 0x1;
    pub const FILE_ATTRIBUTE_DIRECTORY: u32 = 0x10;
    pub const FILE_READ_ATTRIBUTES: u32 = 0x0080;
    pub const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
    pub const MOVEFILE_REPLACE_EXISTING: u32 = 0x1;
    pub const STARTF_USESTDHANDLES: u32 = 0x100;
    pub const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    pub const INFINITE_U32: u32 = 0xFFFF_FFFF;
    pub const WAIT_OBJECT_0: u32 = 0;
    pub const WAIT_TIMEOUT: u32 = 0x102;
    pub const JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE: u32 = 0x2000;
    #[allow(non_upper_case_globals)]
    pub const JobObjectExtendedLimitInformation: i32 = 9;
    /// `BCryptGenRandom` with a NULL algorithm handle +
    /// `BCRYPT_USE_SYSTEM_PREFERRED_RNG` (0x2) — the documented public
    /// system CSPRNG entry; no handle management, no user entropy.
    pub const BCRYPT_USE_SYSTEM_PREFERRED_RNG: u32 = 0x2;
    /// unix epoch shift: FILETIME (100ns since 1601-01-01) → unix ms.
    pub const FILETIME_UNIX_EPOCH_100NS: i64 = 116_444_736_000_000_000;

    #[repr(C)]
    #[allow(non_snake_case)]
    pub struct SYSTEM_INFO {
        pub wProcessorArchitecture: u16,
        pub wReserved: u16,
        pub dwPageSize: u32,
        pub lpMinimumApplicationAddress: *mut u8,
        pub lpMaximumApplicationAddress: *mut u8,
        pub dwActiveProcessorMask: usize,
        pub dwNumberOfProcessors: u32,
        pub dwProcessorType: u32,
        pub dwAllocationGranularity: u32,
        pub wProcessorLevel: u16,
        pub wProcessorRevision: u16,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    #[allow(non_snake_case)]
    pub struct FILETIME {
        pub dwLowDateTime: u32,
        pub dwHighDateTime: u32,
    }

    impl FILETIME {
        pub fn to_unix_ms(self) -> i64 {
            let raw = ((self.dwHighDateTime as i64) << 32) | self.dwLowDateTime as i64;
            (raw - FILETIME_UNIX_EPOCH_100NS) / 10_000
        }
    }

    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    #[allow(non_snake_case)]
    pub struct BY_HANDLE_FILE_INFORMATION {
        pub dwFileAttributes: u32,
        pub ftCreationTime: FILETIME,
        pub ftLastAccessTime: FILETIME,
        pub ftLastWriteTime: FILETIME,
        pub dwVolumeSerialNumber: u32,
        pub nFileSizeHigh: u32,
        pub nFileSizeLow: u32,
        pub nNumberOfLinks: u32,
        pub nFileIndexHigh: u32,
        pub nFileIndexLow: u32,
    }

    #[repr(C)]
    #[allow(non_snake_case)]
    pub struct WIN32_FIND_DATAW {
        pub dwFileAttributes: u32,
        pub ftCreationTime: FILETIME,
        pub ftLastAccessTime: FILETIME,
        pub ftLastWriteTime: FILETIME,
        pub nFileSizeHigh: u32,
        pub nFileSizeLow: u32,
        pub dwReserved0: u32,
        pub dwReserved1: u32,
        pub cFileName: [u16; 260],
        pub cAlternateFileName: [u16; 14],
    }

    #[repr(C)]
    #[allow(non_snake_case)]
    pub struct SECURITY_ATTRIBUTES {
        pub nLength: u32,
        pub lpSecurityDescriptor: *mut u8,
        pub bInheritHandle: i32,
    }

    #[repr(C)]
    #[allow(non_snake_case)]
    pub struct STARTUPINFOW {
        pub cb: u32,
        pub lpReserved: *mut u16,
        pub lpDesktop: *mut u16,
        pub lpTitle: *mut u16,
        pub dwX: u32,
        pub dwY: u32,
        pub dwXSize: u32,
        pub dwYSize: u32,
        pub dwXCountChars: u32,
        pub dwYCountChars: u32,
        pub dwFillAttribute: u32,
        pub dwFlags: u32,
        pub wShowWindow: u16,
        pub cbReserved2: u16,
        pub lpReserved2: *mut u8,
        pub hStdInput: isize,
        pub hStdOutput: isize,
        pub hStdError: isize,
    }

    #[repr(C)]
    #[allow(non_snake_case)]
    pub struct PROCESS_INFORMATION {
        pub hProcess: isize,
        pub hThread: isize,
        pub dwProcessId: u32,
        pub dwThreadId: u32,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    #[allow(non_snake_case)]
    pub struct IO_COUNTERS {
        pub ReadOperationCount: u64,
        pub WriteOperationCount: u64,
        pub OtherOperationCount: u64,
        pub ReadTransferCount: u64,
        pub WriteTransferCount: u64,
        pub OtherTransferCount: u64,
    }

    #[repr(C)]
    #[allow(non_snake_case)]
    pub struct JOBOBJECT_BASIC_LIMIT_INFORMATION {
        pub PerProcessUserTimeLimit: i64,
        pub PerJobUserTimeLimit: i64,
        pub LimitFlags: u32,
        pub MinimumWorkingSetSize: usize,
        pub MaximumWorkingSetSize: usize,
        pub ActiveProcessLimit: u32,
        pub Affinity: usize,
        pub PriorityClass: u32,
        pub SchedulingClass: u32,
    }

    #[repr(C)]
    #[allow(non_snake_case)]
    pub struct JOBOBJECT_EXTENDED_LIMIT_INFORMATION {
        pub BasicLimitInformation: JOBOBJECT_BASIC_LIMIT_INFORMATION,
        pub IoInfo: IO_COUNTERS,
        pub ProcessMemoryLimit: usize,
        pub JobMemoryLimit: usize,
        pub PeakProcessMemoryUsed: usize,
        pub PeakJobMemoryUsed: usize,
    }

    extern "system" {
        pub fn GetStdHandle(nStdHandle: i32) -> isize;
        pub fn WriteFile(
            hFile: isize,
            lpBuffer: *const u8,
            nNumberOfBytesToWrite: u32,
            lpNumberOfBytesWritten: *mut u32,
            lpOverlapped: *mut u8,
        ) -> i32;
        pub fn GetLastError() -> u32;
        pub fn SetLastError(dwErrCode: u32);
        pub fn GetSystemInfo(lpSystemInfo: *mut SYSTEM_INFO);
        pub fn VirtualAlloc(
            lpAddress: *mut u8,
            dwSize: usize,
            flAllocationType: u32,
            flProtect: u32,
        ) -> *mut u8;
        pub fn VirtualFree(lpAddress: *mut u8, dwSize: usize, dwFreeType: u32) -> i32;
        // PLAN-1041: Windows text-console classification + Unicode write.
        pub fn GetConsoleMode(hConsoleHandle: isize, lpMode: *mut u32) -> i32;
        pub fn WriteConsoleW(
            hConsoleOutput: isize,
            lpBuffer: *const u16,
            nNumberOfCharsToWrite: u32,
            lpNumberOfCharsWritten: *mut u32,
            lpReserved: *mut u8,
        ) -> i32;
        // PLAN-973: file & path surface (public kernel32 entry points).
        pub fn CreateFileW(
            lpFileName: *const u16,
            dwDesiredAccess: u32,
            dwShareMode: u32,
            lpSecurityAttributes: *mut SECURITY_ATTRIBUTES,
            dwCreationDisposition: u32,
            dwFlagsAndAttributes: u32,
            hTemplateFile: isize,
        ) -> isize;
        pub fn ReadFile(
            hFile: isize,
            lpBuffer: *mut u8,
            nNumberOfBytesToRead: u32,
            lpNumberOfBytesRead: *mut u32,
            lpOverlapped: *mut u8,
        ) -> i32;
        pub fn CloseHandle(hObject: isize) -> i32;
        pub fn SetFilePointerEx(
            hFile: isize,
            liDistanceToMove: i64,
            lpNewFilePointer: *mut i64,
            dwMoveMethod: u32,
        ) -> i32;
        pub fn GetFileSizeEx(hFile: isize, lpFileSize: *mut i64) -> i32;
        pub fn GetFileInformationByHandle(
            hFile: isize,
            lpFileInformation: *mut BY_HANDLE_FILE_INFORMATION,
        ) -> i32;
        pub fn FlushFileBuffers(hFile: isize) -> i32;
        pub fn GetFileAttributesW(lpFileName: *const u16) -> u32;
        pub fn DeleteFileW(lpFileName: *const u16) -> i32;
        pub fn CreateDirectoryW(
            lpPathName: *const u16,
            lpSecurityAttributes: *mut SECURITY_ATTRIBUTES,
        ) -> i32;
        pub fn RemoveDirectoryW(lpPathName: *const u16) -> i32;
        pub fn MoveFileExW(
            lpExistingFileName: *const u16,
            lpNewFileName: *const u16,
            dwFlags: u32,
        ) -> i32;
        pub fn GetFinalPathNameByHandleW(
            hFile: isize,
            lpszFilePath: *mut u16,
            cchFilePath: u32,
            dwFlags: u32,
        ) -> u32;
        pub fn FindFirstFileW(
            lpFileName: *const u16,
            lpFindFileData: *mut WIN32_FIND_DATAW,
        ) -> isize;
        pub fn FindNextFileW(hFindFile: isize, lpFindFileData: *mut WIN32_FIND_DATAW) -> i32;
        pub fn FindClose(hFindFile: isize) -> i32;
        // PLAN-973: environment, clock, sleep.
        pub fn GetEnvironmentVariableW(lpName: *const u16, lpBuffer: *mut u16, nSize: u32) -> u32;
        pub fn SetEnvironmentVariableW(lpName: *const u16, lpValue: *const u16) -> i32;
        pub fn GetSystemTimeAsFileTime(lpSystemTimeAsFileTime: *mut FILETIME);
        pub fn QueryPerformanceCounter(lpPerformanceCount: *mut i64) -> i32;
        pub fn QueryPerformanceFrequency(lpFrequency: *mut i64) -> i32;
        pub fn SleepEx(dwMilliseconds: u32, bAlertable: i32) -> u32;
        // PLAN-973: process surface.
        pub fn CreateProcessW(
            lpApplicationName: *const u16,
            lpCommandLine: *mut u16,
            lpProcessAttributes: *mut SECURITY_ATTRIBUTES,
            lpThreadAttributes: *mut SECURITY_ATTRIBUTES,
            bInheritHandles: i32,
            dwCreationFlags: u32,
            lpEnvironment: *mut u8,
            lpCurrentDirectory: *const u16,
            lpStartupInfo: *mut STARTUPINFOW,
            lpProcessInformation: *mut PROCESS_INFORMATION,
        ) -> i32;
        pub fn CreatePipe(
            hReadPipe: *mut isize,
            hWritePipe: *mut isize,
            lpPipeAttributes: *mut SECURITY_ATTRIBUTES,
            nSize: u32,
        ) -> i32;
        pub fn CreateJobObjectW(
            lpJobAttributes: *mut SECURITY_ATTRIBUTES,
            lpName: *const u16,
        ) -> isize;
        pub fn SetInformationJobObject(
            hJob: isize,
            JobObjectInformationClass: i32,
            lpJobObjectInformation: *mut u8,
            cbJobObjectInformationLength: u32,
        ) -> i32;
        pub fn AssignProcessToJobObject(hJob: isize, hProcess: isize) -> i32;
        pub fn TerminateJobObject(hJob: isize, uExitCode: u32) -> i32;
        pub fn WaitForSingleObject(hHandle: isize, dwMilliseconds: u32) -> u32;
        pub fn GetExitCodeProcess(hProcess: isize, lpExitCode: *mut u32) -> i32;
        pub fn PeekNamedPipe(
            hNamedPipe: isize,
            lpBuffer: *mut u8,
            nBufferSize: u32,
            lpBytesRead: *mut u32,
            lpTotalBytesAvail: *mut u32,
            lpBytesLeftThisMessage: *mut u32,
        ) -> i32;
        pub fn GetCurrentDirectoryW(nBufferLength: u32, lpBuffer: *mut u16) -> u32;
        pub fn SetCurrentDirectoryW(lpPathName: *const u16) -> i32;
        pub fn GetCommandLineW() -> *mut u16;
        pub fn SetHandleInformation(hObject: isize, dwMask: u32, dwFlags: u32) -> i32;
    }

    // bcrypt.dll — the documented public system CSPRNG entry. Linked via
    // its import library (NOT default-linked like kernel32).
    #[link(name = "bcrypt")]
    extern "system" {
        pub fn BCryptGenRandom(
            hAlgorithm: isize,
            pbBuffer: *mut u8,
            cbBuffer: u32,
            dwFlags: u32,
        ) -> i32;
    }

    pub fn query_system_info() -> (u32, u32) {
        let mut info = std::mem::MaybeUninit::<SYSTEM_INFO>::zeroed();
        unsafe {
            GetSystemInfo(info.as_mut_ptr());
            let info = info.assume_init();
            (info.dwPageSize, info.dwAllocationGranularity)
        }
    }

    /// Map the uniform `OPEN_*` bit set to (access, disposition).
    pub fn open_flags_to_native(flags: i32) -> Result<(u32, u32), u32> {
        let read = flags & 1 != 0;
        let write = flags & 2 != 0;
        let create = flags & 4 != 0;
        let truncate = flags & 8 != 0;
        let exclusive = flags & 16 != 0;
        let append = flags & 32 != 0;
        if !read && !write && !append {
            return Err(87); // ERROR_INVALID_PARAMETER
        }
        let mut access = 0u32;
        if read {
            access |= GENERIC_READ;
        }
        if write || append {
            access |= GENERIC_WRITE;
        }
        let disposition = if create && exclusive {
            CREATE_NEW
        } else if create && truncate {
            CREATE_ALWAYS
        } else if create {
            OPEN_ALWAYS
        } else if truncate {
            TRUNCATE_EXISTING
        } else {
            OPEN_EXISTING
        };
        Ok((access, disposition))
    }
}

/// Real Win32 backend calling `kernel32` directly (zero `std::io::stdout()`, zero `libc`).
#[derive(Debug, Default)]
pub struct Win32PlatformIo {
    pub dispatches: DispatchCounters,
    /// PLAN-974: backend-owned TCP sockets (slot id = index + 1; the
    /// ledger entry stores that id, mirroring `os_handle`). Taking the
    /// slot closes the socket by drop.
    pub net_sockets: std::cell::RefCell<Vec<Option<std::net::TcpStream>>>,
    /// PLAN-974 inc3: backend-owned listeners (id = index + 1).
    pub net_listeners: std::cell::RefCell<Vec<Option<std::net::TcpListener>>>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct DispatchCounters {
    pub std_handle: u64,
    pub write_some: u64,
    pub system_info: u64,
    pub virtual_alloc: u64,
    pub virtual_free: u64,
    /// PLAN-1041 console ops.
    pub console_kind: u64,
    pub console_write16: u64,
    /// PLAN-974 net ops.
    pub net_connect: u64,
    pub net_write: u64,
    pub net_read: u64,
    pub net_close: u64,
    /// PLAN-973 system-resource ops (one shared counter — the per-op
    /// breakdown lives in the op table, not N extra fields).
    pub sys: u64,
}

impl PlatformIoBackend for Win32PlatformIo {
    #[cfg(windows)]
    fn net_connect(&mut self, host: &[u8], port: i32) -> Result<u64, u32> {
        self.dispatches.net_connect += 1;
        use std::net::ToSocketAddrs as _;
        let host = std::str::from_utf8(host).map_err(|_| 87u32)?;
        let addr = std::format!("{host}:{port}");
        let target = addr
            .to_socket_addrs()
            .map_err(io_err_code)?
            .next()
            .ok_or(87u32)?;
        let stream =
            std::net::TcpStream::connect_timeout(&target, std::time::Duration::from_secs(5))
                .map_err(io_err_code)?;
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .map_err(io_err_code)?;
        stream
            .set_write_timeout(Some(std::time::Duration::from_secs(5)))
            .map_err(io_err_code)?;
        let mut slots = self.net_sockets.borrow_mut();
        let idx = match slots.iter().position(|s| s.is_none()) {
            Some(free) => {
                slots[free] = Some(stream);
                free
            }
            None => {
                slots.push(Some(stream));
                slots.len() - 1
            }
        };
        Ok(idx as u64 + 1)
    }

    #[cfg(windows)]
    fn net_write_some(
        &mut self,
        socket: u64,
        bytes: &[u8],
        cancel: &CancelFlag,
    ) -> Result<usize, u32> {
        self.dispatches.net_write += 1;
        use std::io::Write as _;
        if CancelCheck::poll(cancel) {
            return Err(995); // ERROR_OPERATION_ABORTED
        }
        let mut slots = self.net_sockets.borrow_mut();
        let Some(stream) = slots.get_mut(socket as usize - 1).and_then(|s| s.as_mut()) else {
            return Err(6); // ERROR_INVALID_HANDLE
        };
        let mut written = 0usize;
        while written < bytes.len() {
            if CancelCheck::poll(cancel) {
                return Err(995);
            }
            match stream.write(&bytes[written..]) {
                Ok(0) => return Err(232), // ERROR_NO_DATA
                Ok(n) => written += n,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    return if written > 0 {
                        Ok(written)
                    } else {
                        Err(io_err_code(e))
                    }
                }
                Err(e) => {
                    return if written > 0 {
                        Ok(written)
                    } else {
                        Err(io_err_code(e))
                    }
                }
            }
        }
        Ok(written)
    }

    #[cfg(windows)]
    fn net_read_some(&mut self, socket: u64, buf: &mut [u8]) -> Result<usize, u32> {
        self.dispatches.net_read += 1;
        use std::io::Read as _;
        let mut slots = self.net_sockets.borrow_mut();
        let Some(stream) = slots.get_mut(socket as usize - 1).and_then(|s| s.as_mut()) else {
            return Err(6); // ERROR_INVALID_HANDLE
        };
        // paced bounded wait (accept's discipline): peer data normally
        // arrives within microseconds; a quiet window returns the
        // RETRYABLE -10 instead of a hard timeout, and the guest loop
        // re-calls — closing the accept/read interleaving race of inc3
        let _ = stream.set_nonblocking(true);
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(500);
        loop {
            match stream.read(buf) {
                Ok(n) => {
                    let _ = stream.set_nonblocking(false);
                    return Ok(n);
                }
                Err(e)
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut =>
                {
                    if std::time::Instant::now() >= deadline {
                        return Err(u32::MAX);
                    }
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
                Err(e) => {
                    let _ = stream.set_nonblocking(false);
                    return Err(io_err_code(e));
                }
            }
        }
    }

    #[cfg(windows)]
    fn net_close(&mut self, socket: u64) -> Result<(), u32> {
        self.dispatches.net_close += 1;
        let mut slots = self.net_sockets.borrow_mut();
        let idx = socket as usize - 1;
        if idx >= slots.len() || slots[idx].is_none() {
            return Err(6); // ERROR_INVALID_HANDLE
        }
        slots[idx] = None; // drop closes the socket
        Ok(())
    }

    #[cfg(windows)]
    fn net_listen(&mut self, port: i32) -> Result<u64, u32> {
        let listener =
            std::net::TcpListener::bind(("127.0.0.1", port as u16)).map_err(io_err_code)?;
        let mut slots = self.net_listeners.borrow_mut();
        let idx = match slots.iter().position(|s| s.is_none()) {
            Some(free) => {
                slots[free] = Some(listener);
                free
            }
            None => {
                slots.push(Some(listener));
                slots.len() - 1
            }
        };
        Ok(idx as u64 + 1)
    }

    #[cfg(windows)]
    fn net_accept(&mut self, listener: u64) -> Result<u64, u32> {
        let slots = self.net_listeners.borrow();
        let Some(l) = slots.get(listener as usize - 1).and_then(|s| s.as_ref()) else {
            return Err(6); // ERROR_INVALID_HANDLE
        };
        // paced bounded wait (5ms sleeps up to 50ms per call): a pending
        // connection returns promptly; none is a RETRYABLE condition and
        // the guest loop re-calls — no wall-budget hogging, no step burn
        l.set_nonblocking(true).map_err(io_err_code)?;
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(50);
        let (stream, _peer) = loop {
            match l.accept() {
                Ok(pair) => break pair,
                Err(e)
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut =>
                {
                    if std::time::Instant::now() >= deadline {
                        return Err(u32::MAX);
                    }
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
                Err(e) => return Err(io_err_code(e)),
            }
        };
        drop(slots);
        // the accepted socket MUST return to blocking mode before use —
        // a silently-swallowed failure here leaves reads WouldBlocking
        // instantly (the 29/32 partial-delivery puzzle of inc3)
        stream.set_nonblocking(false).map_err(io_err_code)?;
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .map_err(io_err_code)?;
        stream
            .set_write_timeout(Some(std::time::Duration::from_secs(5)))
            .map_err(io_err_code)?;
        let mut socks = self.net_sockets.borrow_mut();
        let idx = match socks.iter().position(|s| s.is_none()) {
            Some(free) => {
                socks[free] = Some(stream);
                free
            }
            None => {
                socks.push(Some(stream));
                socks.len() - 1
            }
        };
        Ok(idx as u64 + 1)
    }

    #[cfg(windows)]
    fn net_local_port(&mut self, listener: u64) -> Result<i32, u32> {
        let slots = self.net_listeners.borrow();
        let Some(l) = slots.get(listener as usize - 1).and_then(|s| s.as_ref()) else {
            return Err(6); // ERROR_INVALID_HANDLE
        };
        let addr = l.local_addr().map_err(io_err_code)?;
        Ok(addr.port() as i32)
    }

    #[cfg(windows)]
    fn std_handle(&mut self, stream: StreamKind) -> Result<usize, u32> {
        self.dispatches.std_handle += 1;
        let h = unsafe { win32::GetStdHandle(stream.win32_std_id()) };
        if h == 0 || h == -1 {
            let err = unsafe { win32::GetLastError() };
            return Err(if err == 0 {
                win32::ERROR_INVALID_HANDLE
            } else {
                err
            });
        }
        Ok(h as usize)
    }

    #[cfg(not(windows))]
    fn std_handle(&mut self, _stream: StreamKind) -> Result<usize, u32> {
        self.dispatches.std_handle += 1;
        Err(50)
    }

    #[cfg(windows)]
    fn write_some(
        &mut self,
        _stream: StreamKind,
        os_handle: usize,
        bytes: &[u8],
        cancel: &CancelFlag,
    ) -> Result<usize, u32> {
        if CancelCheck::poll(cancel) {
            return Err(995); // ERROR_OPERATION_ABORTED
        }
        self.dispatches.write_some += 1;
        if bytes.is_empty() {
            return Ok(0);
        }
        let chunk = u32::try_from(bytes.len()).unwrap_or(u32::MAX);
        let mut written: u32 = 0;
        let ok = unsafe {
            win32::WriteFile(
                os_handle as isize,
                bytes.as_ptr(),
                chunk,
                &mut written,
                core::ptr::null_mut(),
            )
        };
        if ok == 0 {
            let err = unsafe { win32::GetLastError() };
            return Err(if err == 0 { 5 } else { err });
        }
        Ok(written as usize)
    }

    #[cfg(not(windows))]
    fn write_some(
        &mut self,
        _stream: StreamKind,
        _os_handle: usize,
        _bytes: &[u8],
        _cancel: &CancelFlag,
    ) -> Result<usize, u32> {
        self.dispatches.write_some += 1;
        Err(50)
    }

    #[cfg(windows)]
    fn system_info(&mut self) -> (u32, u32) {
        self.dispatches.system_info += 1;
        win32::query_system_info()
    }

    #[cfg(not(windows))]
    fn system_info(&mut self) -> (u32, u32) {
        self.dispatches.system_info += 1;
        (4096, 65536)
    }

    #[cfg(windows)]
    fn virtual_alloc(&mut self, commit_bytes: usize) -> Result<*mut u8, u32> {
        self.dispatches.virtual_alloc += 1;
        let ptr = unsafe {
            win32::VirtualAlloc(
                core::ptr::null_mut(),
                commit_bytes,
                win32::MEM_RESERVE | win32::MEM_COMMIT,
                win32::PAGE_READWRITE,
            )
        };
        if ptr.is_null() {
            let err = unsafe { win32::GetLastError() };
            return Err(if err == 0 { 8 } else { err }); // ERROR_NOT_ENOUGH_MEMORY
        }
        Ok(ptr)
    }

    #[cfg(not(windows))]
    fn virtual_alloc(&mut self, _commit_bytes: usize) -> Result<*mut u8, u32> {
        self.dispatches.virtual_alloc += 1;
        Err(50)
    }

    #[cfg(windows)]
    fn virtual_free(&mut self, base: *mut u8, _mapped_len: usize) -> Result<(), u32> {
        self.dispatches.virtual_free += 1;
        let ok = unsafe { win32::VirtualFree(base, 0, win32::MEM_RELEASE) };
        if ok == 0 {
            let err = unsafe { win32::GetLastError() };
            return Err(if err == 0 { 87 } else { err });
        }
        Ok(())
    }

    #[cfg(not(windows))]
    fn virtual_free(&mut self, _base: *mut u8, _mapped_len: usize) -> Result<(), u32> {
        self.dispatches.virtual_free += 1;
        Err(50)
    }

    #[cfg(windows)]
    fn console_kind(&mut self, os_handle: usize) -> Result<i32, u32> {
        self.dispatches.console_kind += 1;
        let mut mode: u32 = 0;
        let ok = unsafe { win32::GetConsoleMode(os_handle as isize, &mut mode) };
        if ok != 0 {
            return Ok(CONSOLE_KIND_TEXT);
        }
        // PLAN-1041 freeze: a real permission failure must surface, not be
        // swallowed into "redirect"; every other failure means the handle is
        // not a console (file/pipe/char device) — a byte sink.
        let err = unsafe { win32::GetLastError() };
        if err == win32::ERROR_ACCESS_DENIED {
            return Err(err);
        }
        Ok(CONSOLE_KIND_BYTE_SINK)
    }

    #[cfg(not(windows))]
    fn console_kind(&mut self, _os_handle: usize) -> Result<i32, u32> {
        self.dispatches.console_kind += 1;
        Ok(CONSOLE_KIND_BYTE_SINK)
    }

    #[cfg(windows)]
    fn console_write16(
        &mut self,
        _stream: StreamKind,
        os_handle: usize,
        units: &[u16],
        cancel: &CancelFlag,
    ) -> Result<usize, u32> {
        if CancelCheck::poll(cancel) {
            return Err(995); // ERROR_OPERATION_ABORTED
        }
        self.dispatches.console_write16 += 1;
        if units.is_empty() {
            return Ok(0);
        }
        let mut written: u32 = 0;
        let ok = unsafe {
            win32::WriteConsoleW(
                os_handle as isize,
                units.as_ptr(),
                u32::try_from(units.len()).unwrap_or(u32::MAX),
                &mut written,
                core::ptr::null_mut(),
            )
        };
        if ok == 0 {
            let err = unsafe { win32::GetLastError() };
            return Err(if err == 0 { 6 } else { err });
        }
        Ok(written as usize)
    }

    #[cfg(not(windows))]
    fn console_write16(
        &mut self,
        _stream: StreamKind,
        _os_handle: usize,
        _units: &[u16],
        _cancel: &CancelFlag,
    ) -> Result<usize, u32> {
        self.dispatches.console_write16 += 1;
        Err(CONSOLE_WRITE16_UNSUPPORTED)
    }

    // ---- PLAN-973: Win32 file/path/env/clock/entropy/process surface -------

    #[cfg(windows)]
    fn file_open_w16(&mut self, units: &[u16], flags: i32) -> Result<usize, u32> {
        self.dispatches.sys += 1;
        let (mut access, disposition) = win32::open_flags_to_native(flags)?;
        // True OS append semantics (design §7): whenever APPEND is
        // requested the write access is FILE_APPEND_DATA — the OS moves
        // each write to EOF; a userspace seek-end+write is not allowed to
        // impersonate it. (GENERIC_WRITE would write at the shared pointer.)
        if flags & OPEN_APPEND != 0 {
            access = (access & win32::GENERIC_READ) | 0x0000_0004;
        }
        let mut path = units.to_vec();
        path.push(0);
        let h = unsafe {
            win32::CreateFileW(
                path.as_ptr(),
                access,
                win32::FILE_SHARE_READ | win32::FILE_SHARE_WRITE | win32::FILE_SHARE_DELETE,
                core::ptr::null_mut(),
                disposition,
                0x80, // FILE_ATTRIBUTE_NORMAL
                0,
            )
        };
        if h == win32::INVALID_HANDLE_VALUE {
            let err = unsafe { win32::GetLastError() };
            return Err(if err == 0 { 5 } else { err });
        }
        Ok(h as usize)
    }

    #[cfg(windows)]
    fn file_read_some(
        &mut self,
        os_handle: usize,
        buf: &mut [u8],
        cancel: &CancelFlag,
    ) -> Result<usize, u32> {
        if CancelCheck::poll(cancel) {
            return Err(995); // ERROR_OPERATION_ABORTED
        }
        self.dispatches.sys += 1;
        if buf.is_empty() {
            return Ok(0);
        }
        let mut read: u32 = 0;
        let chunk = u32::try_from(buf.len()).unwrap_or(u32::MAX);
        let ok = unsafe {
            win32::ReadFile(
                os_handle as isize,
                buf.as_mut_ptr(),
                chunk,
                &mut read,
                core::ptr::null_mut(),
            )
        };
        if ok == 0 {
            // A broken pipe on a child's stdout is EOF, not a guest error
            // (the legacy 64KB-capture contract treats it as end of data).
            let err = unsafe { win32::GetLastError() };
            if err == 109 || err == 232 {
                return Ok(0); // ERROR_BROKEN_PIPE / ERROR_NO_DATA
            }
            return Err(if err == 0 { 6 } else { err });
        }
        Ok(read as usize)
    }

    #[cfg(windows)]
    fn file_write_some(
        &mut self,
        os_handle: usize,
        bytes: &[u8],
        cancel: &CancelFlag,
    ) -> Result<usize, u32> {
        if CancelCheck::poll(cancel) {
            return Err(995);
        }
        self.dispatches.sys += 1;
        if bytes.is_empty() {
            return Ok(0);
        }
        let mut written: u32 = 0;
        let chunk = u32::try_from(bytes.len()).unwrap_or(u32::MAX);
        let ok = unsafe {
            win32::WriteFile(
                os_handle as isize,
                bytes.as_ptr(),
                chunk,
                &mut written,
                core::ptr::null_mut(),
            )
        };
        if ok == 0 {
            let err = unsafe { win32::GetLastError() };
            return Err(if err == 0 { 5 } else { err });
        }
        Ok(written as usize)
    }

    #[cfg(windows)]
    fn file_seek(&mut self, os_handle: usize, origin: i32, offset: i64) -> Result<i64, u32> {
        self.dispatches.sys += 1;
        let method = match origin {
            0 => 0, // FILE_BEGIN
            1 => 1, // FILE_CURRENT
            2 => 2, // FILE_END
            _ => return Err(87),
        };
        let mut pos: i64 = 0;
        let ok = unsafe { win32::SetFilePointerEx(os_handle as isize, offset, &mut pos, method) };
        if ok == 0 {
            let err = unsafe { win32::GetLastError() };
            return Err(if err == 0 { 6 } else { err });
        }
        Ok(pos)
    }

    #[cfg(windows)]
    fn file_tell(&mut self, os_handle: usize) -> Result<i64, u32> {
        self.dispatches.sys += 1;
        let mut pos: i64 = 0;
        let ok = unsafe {
            win32::SetFilePointerEx(os_handle as isize, 0, &mut pos, 1) // FILE_CURRENT
        };
        if ok == 0 {
            let err = unsafe { win32::GetLastError() };
            return Err(if err == 0 { 6 } else { err });
        }
        Ok(pos)
    }

    #[cfg(windows)]
    fn file_size(&mut self, os_handle: usize) -> Result<u64, u32> {
        self.dispatches.sys += 1;
        let mut size: i64 = 0;
        let ok = unsafe { win32::GetFileSizeEx(os_handle as isize, &mut size) };
        if ok == 0 {
            let err = unsafe { win32::GetLastError() };
            return Err(if err == 0 { 6 } else { err });
        }
        Ok(size as u64)
    }

    #[cfg(windows)]
    fn file_mtime_ms(&mut self, os_handle: usize) -> Result<i64, u32> {
        self.dispatches.sys += 1;
        let mut info = win32::BY_HANDLE_FILE_INFORMATION::default();
        let ok = unsafe { win32::GetFileInformationByHandle(os_handle as isize, &mut info) };
        if ok == 0 {
            let err = unsafe { win32::GetLastError() };
            return Err(if err == 0 { 6 } else { err });
        }
        Ok(info.ftLastWriteTime.to_unix_ms())
    }

    #[cfg(windows)]
    fn file_flush(&mut self, os_handle: usize) -> Result<(), u32> {
        self.dispatches.sys += 1;
        let ok = unsafe { win32::FlushFileBuffers(os_handle as isize) };
        if ok == 0 {
            let err = unsafe { win32::GetLastError() };
            return Err(if err == 0 { 6 } else { err });
        }
        Ok(())
    }

    #[cfg(windows)]
    fn file_close(&mut self, os_handle: usize) -> Result<(), u32> {
        self.dispatches.sys += 1;
        let ok = unsafe { win32::CloseHandle(os_handle as isize) };
        if ok == 0 {
            let err = unsafe { win32::GetLastError() };
            return Err(if err == 0 { 6 } else { err });
        }
        Ok(())
    }

    #[cfg(windows)]
    fn path_attrs_w16(&mut self, units: &[u16]) -> Result<i32, u32> {
        self.dispatches.sys += 1;
        let mut path = units.to_vec();
        path.push(0);
        let attrs = unsafe { win32::GetFileAttributesW(path.as_ptr()) };
        if attrs == win32::INVALID_FILE_ATTRIBUTES {
            let err = unsafe { win32::GetLastError() };
            return Err(if err == 0 { 5 } else { err });
        }
        let mut bits = 0;
        if attrs & win32::FILE_ATTRIBUTE_DIRECTORY != 0 {
            bits |= 1;
        }
        if attrs & win32::FILE_ATTRIBUTE_READONLY != 0 {
            bits |= 2;
        }
        Ok(bits)
    }

    #[cfg(windows)]
    fn file_delete_w16(&mut self, units: &[u16]) -> Result<(), u32> {
        self.dispatches.sys += 1;
        let mut path = units.to_vec();
        path.push(0);
        let ok = unsafe { win32::DeleteFileW(path.as_ptr()) };
        if ok == 0 {
            let err = unsafe { win32::GetLastError() };
            return Err(if err == 0 { 5 } else { err });
        }
        Ok(())
    }

    #[cfg(windows)]
    fn dir_create_w16(&mut self, units: &[u16]) -> Result<(), u32> {
        self.dispatches.sys += 1;
        let mut path = units.to_vec();
        path.push(0);
        let ok = unsafe { win32::CreateDirectoryW(path.as_ptr(), core::ptr::null_mut()) };
        if ok == 0 {
            let err = unsafe { win32::GetLastError() };
            return Err(if err == 0 { 5 } else { err });
        }
        Ok(())
    }

    #[cfg(windows)]
    fn dir_delete_w16(&mut self, units: &[u16]) -> Result<(), u32> {
        self.dispatches.sys += 1;
        let mut path = units.to_vec();
        path.push(0);
        let ok = unsafe { win32::RemoveDirectoryW(path.as_ptr()) };
        if ok == 0 {
            let err = unsafe { win32::GetLastError() };
            return Err(if err == 0 { 5 } else { err });
        }
        Ok(())
    }

    #[cfg(windows)]
    fn file_replace_w16(&mut self, src: &[u16], dst: &[u16]) -> Result<(), u32> {
        self.dispatches.sys += 1;
        let mut s = src.to_vec();
        s.push(0);
        let mut d = dst.to_vec();
        d.push(0);
        let ok =
            unsafe { win32::MoveFileExW(s.as_ptr(), d.as_ptr(), win32::MOVEFILE_REPLACE_EXISTING) };
        if ok == 0 {
            let err = unsafe { win32::GetLastError() };
            return Err(if err == 0 { 5 } else { err });
        }
        Ok(())
    }

    #[cfg(windows)]
    fn canonical_w16(&mut self, units: &[u16], out: &mut [u16]) -> Result<usize, u32> {
        self.dispatches.sys += 1;
        let mut path = units.to_vec();
        path.push(0);
        // Require the target to exist and resolve via its open handle
        // (matching `std::fs::canonicalize` — never fall back to a logical
        // unverified full path on missing targets).
        let h = unsafe {
            win32::CreateFileW(
                path.as_ptr(),
                win32::FILE_READ_ATTRIBUTES,
                win32::FILE_SHARE_READ | win32::FILE_SHARE_WRITE | win32::FILE_SHARE_DELETE,
                core::ptr::null_mut(),
                win32::OPEN_EXISTING,
                win32::FILE_FLAG_BACKUP_SEMANTICS, // opens directories too
                0,
            )
        };
        if h == win32::INVALID_HANDLE_VALUE {
            let err = unsafe { win32::GetLastError() };
            return Err(if err == 0 { 2 } else { err });
        }
        let mut final_buf = vec![0u16; out.len() + 1];
        unsafe { win32::SetLastError(0) };
        let n = unsafe {
            win32::GetFinalPathNameByHandleW(
                h,
                final_buf.as_mut_ptr(),
                final_buf.len() as u32,
                0,
            )
        };
        let err = unsafe { win32::GetLastError() };
        unsafe { win32::CloseHandle(h) };
        if n == 0 {
            return Err(if err == 0 { 5 } else { err });
        }
        if (n as usize) >= final_buf.len() || (n as usize) > out.len() {
            return Err(122); // ERROR_INSUFFICIENT_BUFFER
        }
        out[..n as usize].copy_from_slice(&final_buf[..n as usize]);
        Ok(n as usize)
    }

    #[cfg(windows)]
    fn dir_open_w16(&mut self, units: &[u16]) -> Result<(usize, Option<Vec<u16>>), u32> {
        self.dispatches.sys += 1;
        let mut pattern = units.to_vec();
        // `<dir>\*` enumeration; a trailing separator is tolerated by the
        // Win32 path normalizer, so appending blindly is safe.
        pattern.extend_from_slice(&[0x5C, 0x2A, 0]); // '\\', '*', NUL
        let mut data = unsafe { std::mem::zeroed::<win32::WIN32_FIND_DATAW>() };
        let h = unsafe { win32::FindFirstFileW(pattern.as_ptr(), &mut data) };
        if h == win32::INVALID_HANDLE_VALUE {
            let err = unsafe { win32::GetLastError() };
            return Err(if err == 0 { 5 } else { err });
        }
        let first = cstr_units(&data.cFileName);
        Ok((h as usize, Some(first)))
    }

    #[cfg(windows)]
    fn dir_next_w16(&mut self, os_handle: usize, out: &mut [u16]) -> Result<usize, u32> {
        self.dispatches.sys += 1;
        loop {
            let mut data = unsafe { std::mem::zeroed::<win32::WIN32_FIND_DATAW>() };
            let ok = unsafe { win32::FindNextFileW(os_handle as isize, &mut data) };
            if ok == 0 {
                let err = unsafe { win32::GetLastError() };
                if err == 18 {
                    return Ok(0); // ERROR_NO_MORE_FILES — clean end
                }
                return Err(if err == 0 { 6 } else { err });
            }
            let name = cstr_units(&data.cFileName);
            if name.len() > out.len() {
                return Err(122); // ERROR_INSUFFICIENT_BUFFER
            }
            out[..name.len()].copy_from_slice(&name);
            return Ok(name.len());
        }
    }

    #[cfg(windows)]
    fn dir_close(&mut self, os_handle: usize) -> Result<(), u32> {
        self.dispatches.sys += 1;
        let ok = unsafe { win32::FindClose(os_handle as isize) };
        if ok == 0 {
            let err = unsafe { win32::GetLastError() };
            return Err(if err == 0 { 6 } else { err });
        }
        Ok(())
    }

    #[cfg(windows)]
    fn env_get_w16(&mut self, name: &[u16], out: &mut [u16]) -> Result<usize, u32> {
        self.dispatches.sys += 1;
        let mut n = name.to_vec();
        n.push(0);
        if out.is_empty() {
            return Err(122);
        }
        let cap = u32::try_from(out.len() - 1).unwrap_or(u32::MAX);
        unsafe { win32::SetLastError(0) };
        let ret = unsafe { win32::GetEnvironmentVariableW(n.as_ptr(), out.as_mut_ptr(), cap + 1) };
        if ret == 0 {
            let err = unsafe { win32::GetLastError() };
            if err == 0 {
                return Ok(0); // present with empty value ""
            }
            return Err(err); // e.g. 203 (ERROR_ENVVAR_NOT_FOUND)
        }
        if ret > cap {
            return Err(234); // ERROR_MORE_DATA
        }
        Ok(ret as usize)
    }

    #[cfg(windows)]
    fn env_set_w16(&mut self, name: &[u16], value: &[u16]) -> Result<(), u32> {
        self.dispatches.sys += 1;
        let mut n = name.to_vec();
        n.push(0);
        let mut v = value.to_vec();
        v.push(0);
        let ok = unsafe { win32::SetEnvironmentVariableW(n.as_ptr(), v.as_ptr()) };
        if ok == 0 {
            let err = unsafe { win32::GetLastError() };
            return Err(if err == 0 { 5 } else { err });
        }
        Ok(())
    }

    #[cfg(windows)]
    fn env_remove_w16(&mut self, name: &[u16]) -> Result<(), u32> {
        self.dispatches.sys += 1;
        let mut n = name.to_vec();
        n.push(0);
        let ok = unsafe { win32::SetEnvironmentVariableW(n.as_ptr(), core::ptr::null()) };
        if ok == 0 {
            let err = unsafe { win32::GetLastError() };
            return Err(if err == 0 { 5 } else { err });
        }
        Ok(())
    }

    #[cfg(windows)]
    fn clock_unix_ms(&mut self) -> Result<i64, u32> {
        self.dispatches.sys += 1;
        let mut ft = win32::FILETIME::default();
        unsafe { win32::GetSystemTimeAsFileTime(&mut ft) };
        Ok(ft.to_unix_ms())
    }

    #[cfg(windows)]
    fn clock_mono_ms(&mut self) -> Result<i64, u32> {
        self.dispatches.sys += 1;
        let mut counter: i64 = 0;
        let mut freq: i64 = 0;
        let ok1 = unsafe { win32::QueryPerformanceFrequency(&mut freq) };
        let ok2 = unsafe { win32::QueryPerformanceCounter(&mut counter) };
        if ok1 == 0 || ok2 == 0 || freq <= 0 {
            return Err(87);
        }
        Ok(counter.saturating_mul(1000) / freq)
    }

    #[cfg(windows)]
    fn sleep_ms(&mut self, ms: u32, cancel: &CancelFlag) -> Result<(), u32> {
        self.dispatches.sys += 1;
        const SLICE_MS: u32 = 50;
        let mut remaining = ms;
        loop {
            if CancelCheck::poll(cancel) {
                return Err(995);
            }
            if remaining == 0 {
                return Ok(());
            }
            let slice = remaining.min(SLICE_MS);
            unsafe { win32::SleepEx(slice, 0) };
            remaining -= slice;
        }
    }

    #[cfg(windows)]
    fn rand_fill(&mut self, buf: &mut [u8]) -> Result<(), u32> {
        self.dispatches.sys += 1;
        if buf.is_empty() {
            return Ok(());
        }
        let status = unsafe {
            win32::BCryptGenRandom(
                0,
                buf.as_mut_ptr(),
                u32::try_from(buf.len()).unwrap_or(u32::MAX),
                win32::BCRYPT_USE_SYSTEM_PREFERRED_RNG,
            )
        };
        if status != 0 {
            return Err((status as u32) | 0x0010_4000); // NTSTATUS fault marker
        }
        Ok(())
    }

    #[cfg(windows)]
    fn proc_spawn_w16(&mut self, cmdline: &[u16], capture: bool) -> Result<ProcSpawn, u32> {
        self.dispatches.sys += 1;
        if cmdline.is_empty() {
            return Err(87);
        }
        unsafe {
            // kill-on-close job: covers the whole descendant tree (V-03).
            let job = win32::CreateJobObjectW(core::ptr::null_mut(), core::ptr::null());
            if job == 0 {
                let err = win32::GetLastError();
                return Err(if err == 0 { 8 } else { err });
            }
            let mut info = std::mem::zeroed::<win32::JOBOBJECT_EXTENDED_LIMIT_INFORMATION>();
            info.BasicLimitInformation.LimitFlags = win32::JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            let ok = win32::SetInformationJobObject(
                job,
                win32::JobObjectExtendedLimitInformation,
                &mut info as *mut _ as *mut u8,
                std::mem::size_of::<win32::JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            );
            if ok == 0 {
                let err = win32::GetLastError();
                win32::CloseHandle(job);
                return Err(err);
            }

            let mut out_read: isize = 0;
            let mut out_write: isize = 0;
            let mut err_read: isize = 0;
            let mut err_write: isize = 0;
            if capture {
                let mut sa = win32::SECURITY_ATTRIBUTES {
                    nLength: std::mem::size_of::<win32::SECURITY_ATTRIBUTES>() as u32,
                    lpSecurityDescriptor: core::ptr::null_mut(),
                    bInheritHandle: 1,
                };
                const PIPE_BUF: u32 = 1 << 20; // 1 MiB kernel buffer
                if win32::CreatePipe(&mut out_read, &mut out_write, &mut sa, PIPE_BUF) == 0 {
                    return Err(win32::GetLastError());
                }
                if win32::CreatePipe(&mut err_read, &mut err_write, &mut sa, PIPE_BUF) == 0 {
                    let e = win32::GetLastError();
                    win32::CloseHandle(out_read);
                    win32::CloseHandle(out_write);
                    return Err(e);
                }
                // only the write ends may cross into the child
                win32::SetHandleInformation(out_read, 0x1, 0);
                win32::SetHandleInformation(err_read, 0x1, 0);
            }

            let mut cmd = cmdline.to_vec();
            cmd.push(0);
            let mut si = std::mem::zeroed::<win32::STARTUPINFOW>();
            si.cb = std::mem::size_of::<win32::STARTUPINFOW>() as u32;
            let mut pi = std::mem::zeroed::<win32::PROCESS_INFORMATION>();
            if capture {
                si.dwFlags = win32::STARTF_USESTDHANDLES;
                si.hStdOutput = out_write;
                si.hStdError = err_write;
                si.hStdInput = -1; // no stdin for captured children
            }
            let ok = win32::CreateProcessW(
                core::ptr::null(),
                cmd.as_mut_ptr(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                if capture { 1 } else { 0 },
                win32::CREATE_NO_WINDOW,
                core::ptr::null_mut(),
                core::ptr::null(),
                &mut si,
                &mut pi,
            );
            if ok == 0 {
                let e = win32::GetLastError();
                if capture {
                    win32::CloseHandle(out_read);
                    win32::CloseHandle(out_write);
                    win32::CloseHandle(err_read);
                    win32::CloseHandle(err_write);
                }
                win32::CloseHandle(job);
                return Err(e);
            }
            win32::CloseHandle(pi.hThread);
            win32::AssignProcessToJobObject(job, pi.hProcess);
            if capture {
                // parent closes the write ends: EOF tracks child exit
                win32::CloseHandle(out_write);
                win32::CloseHandle(err_write);
            }
            Ok(ProcSpawn {
                process: pi.hProcess as usize,
                job: job as usize,
                out_read: out_read as usize,
                err_read: err_read as usize,
            })
        }
    }

    #[cfg(windows)]
    fn pipe_avail(&mut self, os_handle: usize) -> Result<usize, u32> {
        self.dispatches.sys += 1;
        let mut avail: u32 = 0;
        let ok = unsafe {
            win32::PeekNamedPipe(
                os_handle as isize,
                core::ptr::null_mut(),
                0,
                core::ptr::null_mut(),
                &mut avail,
                core::ptr::null_mut(),
            )
        };
        if ok == 0 {
            let err = unsafe { win32::GetLastError() };
            // broken pipe = no more data ever (child exited + handles gone)
            if err == 109 || err == 232 {
                return Ok(0);
            }
            return Err(if err == 0 { 6 } else { err });
        }
        Ok(avail as usize)
    }

    #[cfg(windows)]
    fn wait_child(&mut self, os_handle: usize, timeout_ms: i32) -> Result<bool, u32> {
        self.dispatches.sys += 1;
        let wait = if timeout_ms < 0 {
            win32::INFINITE_U32
        } else {
            timeout_ms as u32
        };
        let r = unsafe { win32::WaitForSingleObject(os_handle as isize, wait) };
        match r {
            win32::WAIT_OBJECT_0 => Ok(true),
            win32::WAIT_TIMEOUT => Ok(false),
            _ => Err(unsafe { win32::GetLastError() }),
        }
    }

    #[cfg(windows)]
    fn child_exit_code(&mut self, os_handle: usize) -> Result<i32, u32> {
        self.dispatches.sys += 1;
        let mut code: u32 = 0;
        let ok = unsafe { win32::GetExitCodeProcess(os_handle as isize, &mut code) };
        if ok == 0 {
            let err = unsafe { win32::GetLastError() };
            return Err(if err == 0 { 6 } else { err });
        }
        Ok(code as i32)
    }

    #[cfg(windows)]
    fn kill_tree(&mut self, job_handle: usize) -> Result<(), u32> {
        self.dispatches.sys += 1;
        let ok = unsafe { win32::TerminateJobObject(job_handle as isize, 1) };
        if ok == 0 {
            let err = unsafe { win32::GetLastError() };
            return Err(if err == 0 { 6 } else { err });
        }
        Ok(())
    }

    #[cfg(windows)]
    fn close_raw(&mut self, os_handle: usize) -> Result<(), u32> {
        self.dispatches.sys += 1;
        let ok = unsafe { win32::CloseHandle(os_handle as isize) };
        if ok == 0 {
            let err = unsafe { win32::GetLastError() };
            return Err(if err == 0 { 6 } else { err });
        }
        Ok(())
    }

    #[cfg(windows)]
    fn proc_cmdline_w16(&mut self, out: &mut [u16]) -> Result<usize, u32> {
        self.dispatches.sys += 1;
        let p = unsafe { win32::GetCommandLineW() };
        if p.is_null() {
            return Err(8);
        }
        let mut len = 0usize;
        while unsafe { core::ptr::read(p.add(len)) } != 0 {
            len += 1;
        }
        if len > out.len() {
            return Err(122);
        }
        for i in 0..len {
            out[i] = unsafe { core::ptr::read(p.add(i)) };
        }
        Ok(len)
    }

    #[cfg(windows)]
    fn proc_cwd_w16(&mut self, out: &mut [u16]) -> Result<usize, u32> {
        self.dispatches.sys += 1;
        if out.is_empty() {
            return Err(122);
        }
        let cap = u32::try_from(out.len() - 1).unwrap_or(u32::MAX);
        let ret = unsafe { win32::GetCurrentDirectoryW(cap + 1, out.as_mut_ptr()) };
        if ret == 0 {
            let err = unsafe { win32::GetLastError() };
            return Err(if err == 0 { 6 } else { err });
        }
        if ret > cap {
            return Err(234);
        }
        Ok(ret as usize)
    }

    #[cfg(windows)]
    fn proc_chdir_w16(&mut self, units: &[u16]) -> Result<(), u32> {
        self.dispatches.sys += 1;
        let mut path = units.to_vec();
        path.push(0);
        let ok = unsafe { win32::SetCurrentDirectoryW(path.as_ptr()) };
        if ok == 0 {
            let err = unsafe { win32::GetLastError() };
            return Err(if err == 0 { 5 } else { err });
        }
        Ok(())
    }
}

/// Take a NUL-terminated UTF-16 fixed array to its unit vector (no NUL).
#[cfg(windows)]
fn cstr_units(buf: &[u16]) -> Vec<u16> {
    let mut end = buf.len();
    for (i, &u) in buf.iter().enumerate() {
        if u == 0 {
            end = i;
            break;
        }
    }
    buf[..end].to_vec()
}

/// Observable test provider: delegates real Win32 OS handles/system_info/VirtualAlloc/VirtualFree
/// to `Win32PlatformIo`, captures `write_some` stream output cleanly, and supports programmable
/// partial-write and OS error injection for PIO-02 & V-MEM controls.
#[derive(Debug, Default)]
pub struct CapturePlatformIo {
    pub win32: Win32PlatformIo,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub max_write_chunk: Option<usize>,
    pub inject_write_error: Option<u32>,
    pub inject_alloc_error: Option<u32>,
    pub inject_free_error: Option<u32>,
    pub use_synthetic_std_handle: bool,
    /// PLAN-1041: force `console_kind == 1` so the console text path runs
    /// under capture without a real Windows console; console writes land in
    /// the UTF-16 unit buffers below (independent oracle for the Auto-side
    /// UTF-8→UTF-16 conversion loop).
    pub force_console_kind: bool,
    pub console16_stdout: Vec<u16>,
    pub console16_stderr: Vec<u16>,
    pub max_console16_units: Option<usize>,
    pub inject_console16_error: Option<u32>,
    /// PLAN-973 file-op oracles/faults: injected OS error on open/read/
    /// write/replace, a partial-read cap, fixed clock values (V-05
    /// injectable oracle) and a deterministic entropy pattern.
    pub inject_file_error: Option<u32>,
    pub max_file_read_chunk: Option<usize>,
    pub clock_unix_oracle: Option<i64>,
    pub clock_mono_oracle: Option<i64>,
    pub rand_oracle: Option<Vec<u8>>,
}

impl PlatformIoBackend for CapturePlatformIo {
    fn std_handle(&mut self, stream: StreamKind) -> Result<usize, u32> {
        if self.use_synthetic_std_handle {
            self.win32.dispatches.std_handle += 1;
            return Ok(match stream {
                StreamKind::Stdout => 0x1001,
                StreamKind::Stderr => 0x1002,
                StreamKind::Stdin => 0x1003,
            });
        }
        match self.win32.std_handle(stream) {
            Ok(h) => Ok(h),
            Err(err) => {
                // In headless GUI/runner tests where parent stdio might be detached, allow synthetic
                // capture handle only if explicitly opted in; otherwise propagate real Win32 error!
                Err(err)
            }
        }
    }

    fn write_some(
        &mut self,
        stream: StreamKind,
        _os_handle: usize,
        bytes: &[u8],
        cancel: &CancelFlag,
    ) -> Result<usize, u32> {
        if CancelCheck::poll(cancel) {
            return Err(995);
        }
        self.win32.dispatches.write_some += 1;
        if let Some(err) = self.inject_write_error {
            return Err(err);
        }
        let n = self
            .max_write_chunk
            .map_or(bytes.len(), |cap| bytes.len().min(cap));
        let slice = &bytes[..n];
        let dest = match stream {
            StreamKind::Stdout => &mut self.stdout,
            StreamKind::Stderr => &mut self.stderr,
            // stdin is never written; a synthetic sink keeps capture clean
            StreamKind::Stdin => return Err(6),
        };
        dest.try_reserve_exact(slice.len()).map_err(|_| 8u32)?;
        dest.extend_from_slice(slice);
        Ok(n)
    }

    fn system_info(&mut self) -> (u32, u32) {
        self.win32.system_info()
    }

    fn virtual_alloc(&mut self, commit_bytes: usize) -> Result<*mut u8, u32> {
        if let Some(err) = self.inject_alloc_error {
            self.win32.dispatches.virtual_alloc += 1;
            return Err(err);
        }
        self.win32.virtual_alloc(commit_bytes)
    }

    fn virtual_free(&mut self, base: *mut u8, mapped_len: usize) -> Result<(), u32> {
        if let Some(err) = self.inject_free_error {
            self.win32.dispatches.virtual_free += 1;
            return Err(err);
        }
        self.win32.virtual_free(base, mapped_len)
    }

    // PLAN-1041: capture console ops — deterministic oracle + fault
    // injection for the console text path (OUT-02/OUT-03 controls).
    fn console_kind(&mut self, _os_handle: usize) -> Result<i32, u32> {
        self.win32.dispatches.console_kind += 1;
        if self.force_console_kind {
            return Ok(CONSOLE_KIND_TEXT);
        }
        self.win32.console_kind(_os_handle)
    }

    fn console_write16(
        &mut self,
        stream: StreamKind,
        _os_handle: usize,
        units: &[u16],
        cancel: &CancelFlag,
    ) -> Result<usize, u32> {
        if CancelCheck::poll(cancel) {
            return Err(995);
        }
        self.win32.dispatches.console_write16 += 1;
        if let Some(err) = self.inject_console16_error {
            return Err(err);
        }
        let n = self
            .max_console16_units
            .map_or(units.len(), |cap| units.len().min(cap));
        let dest = match stream {
            StreamKind::Stdout => &mut self.console16_stdout,
            StreamKind::Stderr => &mut self.console16_stderr,
            // stdin is never a console target
            StreamKind::Stdin => return Err(6),
        };
        dest.try_reserve_exact(n).map_err(|_| 8u32)?;
        dest.extend_from_slice(&units[..n]);
        Ok(n)
    }

    // ---- PLAN-973: real-OS delegation with capture-side oracles/faults ---

    fn file_open_w16(&mut self, units: &[u16], flags: i32) -> Result<usize, u32> {
        if let Some(err) = self.inject_file_error {
            self.win32.dispatches.sys += 1;
            return Err(err);
        }
        self.win32.file_open_w16(units, flags)
    }

    fn file_open_bytes(&mut self, path: &[u8], flags: i32) -> Result<usize, u32> {
        self.win32.file_open_bytes(path, flags)
    }

    fn file_read_some(
        &mut self,
        os_handle: usize,
        buf: &mut [u8],
        cancel: &CancelFlag,
    ) -> Result<usize, u32> {
        if let Some(err) = self.inject_file_error {
            self.win32.dispatches.sys += 1;
            return Err(err);
        }
        let cap = self
            .max_file_read_chunk
            .map_or(buf.len(), |c| buf.len().min(c));
        self.win32
            .file_read_some(os_handle, &mut buf[..cap], cancel)
    }

    fn file_write_some(
        &mut self,
        os_handle: usize,
        bytes: &[u8],
        cancel: &CancelFlag,
    ) -> Result<usize, u32> {
        if let Some(err) = self.inject_file_error {
            self.win32.dispatches.sys += 1;
            return Err(err);
        }
        self.win32.file_write_some(os_handle, bytes, cancel)
    }

    fn file_seek(&mut self, os_handle: usize, origin: i32, offset: i64) -> Result<i64, u32> {
        self.win32.file_seek(os_handle, origin, offset)
    }

    fn file_tell(&mut self, os_handle: usize) -> Result<i64, u32> {
        self.win32.file_tell(os_handle)
    }

    fn file_size(&mut self, os_handle: usize) -> Result<u64, u32> {
        self.win32.file_size(os_handle)
    }

    fn file_mtime_ms(&mut self, os_handle: usize) -> Result<i64, u32> {
        self.win32.file_mtime_ms(os_handle)
    }

    fn file_flush(&mut self, os_handle: usize) -> Result<(), u32> {
        self.win32.file_flush(os_handle)
    }

    fn file_close(&mut self, os_handle: usize) -> Result<(), u32> {
        self.win32.file_close(os_handle)
    }

    fn path_attrs_w16(&mut self, units: &[u16]) -> Result<i32, u32> {
        self.win32.path_attrs_w16(units)
    }

    fn path_attrs_bytes(&mut self, path: &[u8]) -> Result<i32, u32> {
        self.win32.path_attrs_bytes(path)
    }

    fn file_delete_w16(&mut self, units: &[u16]) -> Result<(), u32> {
        if let Some(err) = self.inject_file_error {
            self.win32.dispatches.sys += 1;
            return Err(err);
        }
        self.win32.file_delete_w16(units)
    }

    fn file_delete_bytes(&mut self, path: &[u8]) -> Result<(), u32> {
        self.win32.file_delete_bytes(path)
    }

    fn dir_create_w16(&mut self, units: &[u16]) -> Result<(), u32> {
        self.win32.dir_create_w16(units)
    }

    fn dir_create_bytes(&mut self, path: &[u8]) -> Result<(), u32> {
        self.win32.dir_create_bytes(path)
    }

    fn dir_delete_w16(&mut self, units: &[u16]) -> Result<(), u32> {
        self.win32.dir_delete_w16(units)
    }

    fn dir_delete_bytes(&mut self, path: &[u8]) -> Result<(), u32> {
        self.win32.dir_delete_bytes(path)
    }

    fn file_replace_w16(&mut self, src: &[u16], dst: &[u16]) -> Result<(), u32> {
        if let Some(err) = self.inject_file_error {
            self.win32.dispatches.sys += 1;
            return Err(err);
        }
        self.win32.file_replace_w16(src, dst)
    }

    fn file_replace_bytes(&mut self, src: &[u8], dst: &[u8]) -> Result<(), u32> {
        self.win32.file_replace_bytes(src, dst)
    }

    fn canonical_w16(&mut self, units: &[u16], out: &mut [u16]) -> Result<usize, u32> {
        self.win32.canonical_w16(units, out)
    }

    fn canonical_bytes(&mut self, path: &[u8], out: &mut [u8]) -> Result<usize, u32> {
        self.win32.canonical_bytes(path, out)
    }

    fn dir_open_w16(&mut self, units: &[u16]) -> Result<(usize, Option<Vec<u16>>), u32> {
        self.win32.dir_open_w16(units)
    }

    fn dir_next_w16(&mut self, os_handle: usize, out: &mut [u16]) -> Result<usize, u32> {
        self.win32.dir_next_w16(os_handle, out)
    }

    fn dir_close(&mut self, os_handle: usize) -> Result<(), u32> {
        self.win32.dir_close(os_handle)
    }

    fn dir_open_bytes(&mut self, path: &[u8]) -> Result<usize, u32> {
        self.win32.dir_open_bytes(path)
    }

    fn dir_fill_bytes(&mut self, os_handle: usize, buf: &mut [u8]) -> Result<usize, u32> {
        self.win32.dir_fill_bytes(os_handle, buf)
    }

    fn env_get_w16(&mut self, name: &[u16], out: &mut [u16]) -> Result<usize, u32> {
        self.win32.env_get_w16(name, out)
    }

    fn env_set_w16(&mut self, name: &[u16], value: &[u16]) -> Result<(), u32> {
        self.win32.env_set_w16(name, value)
    }

    fn env_remove_w16(&mut self, name: &[u16]) -> Result<(), u32> {
        self.win32.env_remove_w16(name)
    }

    fn env_get_bytes(&mut self, out: &mut [u8]) -> Result<usize, u32> {
        self.win32.env_get_bytes(out)
    }

    fn clock_unix_ms(&mut self) -> Result<i64, u32> {
        self.win32.dispatches.sys += 1;
        if let Some(v) = self.clock_unix_oracle {
            return Ok(v);
        }
        self.win32.clock_unix_ms()
    }

    fn clock_mono_ms(&mut self) -> Result<i64, u32> {
        self.win32.dispatches.sys += 1;
        if let Some(v) = self.clock_mono_oracle {
            return Ok(v);
        }
        self.win32.clock_mono_ms()
    }

    fn sleep_ms(&mut self, ms: u32, cancel: &CancelFlag) -> Result<(), u32> {
        self.win32.sleep_ms(ms, cancel)
    }

    fn rand_fill(&mut self, buf: &mut [u8]) -> Result<(), u32> {
        self.win32.dispatches.sys += 1;
        if let Some(pattern) = &self.rand_oracle {
            for (i, b) in buf.iter_mut().enumerate() {
                *b = pattern[i % pattern.len()];
            }
            return Ok(());
        }
        self.win32.rand_fill(buf)
    }

    fn proc_spawn_w16(&mut self, cmdline: &[u16], capture: bool) -> Result<ProcSpawn, u32> {
        self.win32.proc_spawn_w16(cmdline, capture)
    }

    fn pipe_avail(&mut self, os_handle: usize) -> Result<usize, u32> {
        self.win32.pipe_avail(os_handle)
    }

    fn wait_child(&mut self, os_handle: usize, timeout_ms: i32) -> Result<bool, u32> {
        self.win32.wait_child(os_handle, timeout_ms)
    }

    fn child_exit_code(&mut self, os_handle: usize) -> Result<i32, u32> {
        self.win32.child_exit_code(os_handle)
    }

    fn kill_tree(&mut self, job_handle: usize) -> Result<(), u32> {
        self.win32.kill_tree(job_handle)
    }

    fn close_raw(&mut self, os_handle: usize) -> Result<(), u32> {
        self.win32.close_raw(os_handle)
    }

    fn proc_cmdline_w16(&mut self, out: &mut [u16]) -> Result<usize, u32> {
        self.win32.proc_cmdline_w16(out)
    }

    fn proc_cmdline_fill_bytes(&mut self, out: &mut [u8]) -> Result<usize, u32> {
        self.win32.proc_cmdline_fill_bytes(out)
    }

    fn proc_cwd_w16(&mut self, out: &mut [u16]) -> Result<usize, u32> {
        self.win32.proc_cwd_w16(out)
    }

    fn proc_cwd_bytes(&mut self, out: &mut [u8]) -> Result<usize, u32> {
        self.win32.proc_cwd_bytes(out)
    }

    fn proc_chdir_w16(&mut self, units: &[u16]) -> Result<(), u32> {
        self.win32.proc_chdir_w16(units)
    }

    fn proc_chdir_bytes(&mut self, path: &[u8]) -> Result<(), u32> {
        self.win32.proc_chdir_bytes(path)
    }
}

// ---------------------------------------------------------------------------
// PLAN-1040: Linux x86-64 direct-kernel backend (raw syscalls, zero libc /
// zero std::io / zero errno TLS)
// ---------------------------------------------------------------------------

/// x86-64 Linux syscall numbers used by the platform backend and the
/// probe seam (target UAPI; arm64 needs its own table — out of scope).
pub mod linux_sysno {
    pub const READ: isize = 0;
    pub const WRITE: isize = 1;
    pub const CLOSE: isize = 3;
    pub const FSTAT: isize = 5;
    pub const POLL: isize = 7;
    pub const LSEEK: isize = 8;
    pub const MMAP: isize = 9;
    pub const MUNMAP: isize = 11;
    pub const IOCTL: isize = 16;
    pub const PIPE: isize = 22;
    pub const DUP2: isize = 33;
    pub const NANOSLEEP: isize = 35;
    pub const CLONE: isize = 56;
    pub const EXECVE: isize = 59;
    pub const EXIT: isize = 60;
    pub const WAIT4: isize = 61;
    pub const KILL: isize = 62;
    pub const GETCWD: isize = 79;
    pub const CHDIR: isize = 80;
    pub const FSYNC: isize = 10;
    pub const CLOCK_GETTIME: isize = 228;
    pub const GETDENTS64: isize = 217;
    pub const MKDIRAT: isize = 258;
    pub const NEWFSTATAT: isize = 262;
    pub const UNLINKAT: isize = 263;
    pub const RENAMEAT: isize = 264;
    pub const READLINKAT: isize = 267;
    pub const OPENAT: isize = 257;
    pub const GETRANDOM: isize = 318;
    pub const SETPGID: isize = 154;
    pub const EXIT_GROUP: isize = 231;
}

/// x86-64 O_ flag constants used by the file path (target UAPI).
pub mod linux_oflags {
    pub const O_RDONLY: i32 = 0;
    pub const O_WRONLY: i32 = 1;
    pub const O_RDWR: i32 = 2;
    pub const O_CREAT: i32 = 0o100;
    pub const O_EXCL: i32 = 0o200;
    pub const O_TRUNC: i32 = 0o1000;
    pub const O_APPEND: i32 = 0o2000;
    pub const O_DIRECTORY: i32 = 0o200000;
    pub const O_PATH: i32 = 0o10000000;
    pub const O_CLOEXEC: i32 = 0o2000000;
}

/// x86-64 poll(2) constants.
pub const LINUX_POLLIN: i16 = 0x1;
pub const LINUX_POLLHUP: i16 = 0x10;
pub const LINUX_POLLERR: i16 = 0x8;
/// FIONREAD (termios ioctl).
pub const LINUX_FIONREAD: isize = 0x541B;
/// Default file creation mode (matches the legacy Rust surface).
pub const LINUX_DEFAULT_MODE: i32 = 0o666;
/// siginfo_t field offsets on x86-64 (si_pid at 12, si_status at 16).
pub const LINUX_SI_PID_OFF: usize = 12;
pub const LINUX_SI_STATUS_OFF: usize = 16;

/// `PROT_READ | PROT_WRITE` for the anonymous private region source.
pub const LINUX_PROT_RW: isize = 0x3;
/// `MAP_PRIVATE | MAP_ANONYMOUS` (anonymous private RW mapping, fd=-1,
/// offset=0, never MAP_FIXED).
pub const LINUX_MAP_PRIVATE_ANON: isize = 0x22;
/// x86-64 raw-syscall error interval is `-4095..=-1` — only those return
/// values are errno, anything else (incl. high pointers) is a success
/// value carried bit-exact.
pub fn linux_is_errno(ret: isize) -> bool {
    (-4095..=-1).contains(&ret)
}

#[cfg(target_os = "linux")]
mod linux_asm {
    /// Raw syscall wrappers (x86-64 kernel ABI: nr in rax, args
    /// rdi/rsi/rdx/r10/r8/r9; rcx/r11 clobbered). Stable `core::arch::asm!`
    /// — no libc `syscall()`, no errno TLS, ever.
    #[inline]
    pub unsafe fn syscall1(nr: isize, a1: isize) -> isize {
        let ret: isize;
        core::arch::asm!(
            "syscall",
            inlateout("rax") nr => ret,
            in("rdi") a1,
            lateout("rcx") _, lateout("r11") _,
            options(nostack)
        );
        ret
    }

    #[inline]
    pub unsafe fn syscall2(nr: isize, a1: isize, a2: isize) -> isize {
        let ret: isize;
        core::arch::asm!(
            "syscall",
            inlateout("rax") nr => ret,
            in("rdi") a1, in("rsi") a2,
            lateout("rcx") _, lateout("r11") _,
            options(nostack)
        );
        ret
    }

    #[inline]
    pub unsafe fn syscall3(nr: isize, a1: isize, a2: isize, a3: isize) -> isize {
        let ret: isize;
        core::arch::asm!(
            "syscall",
            inlateout("rax") nr => ret,
            in("rdi") a1, in("rsi") a2, in("rdx") a3,
            lateout("rcx") _, lateout("r11") _,
            options(nostack)
        );
        ret
    }

    #[inline]
    pub unsafe fn syscall4(nr: isize, a1: isize, a2: isize, a3: isize, a4: isize) -> isize {
        let ret: isize;
        core::arch::asm!(
            "syscall",
            inlateout("rax") nr => ret,
            in("rdi") a1, in("rsi") a2, in("rdx") a3, in("r10") a4,
            lateout("rcx") _, lateout("r11") _,
            options(nostack)
        );
        ret
    }

    /// PLAN-973: the 5-arg form (clone/execve-with-env/llseek families).
    /// Reserved for the 1063 Linux process shim; allowed dead until then.
    #[allow(dead_code)]
    #[inline]
    pub unsafe fn syscall5(
        nr: isize,
        a1: isize,
        a2: isize,
        a3: isize,
        a4: isize,
        a5: isize,
    ) -> isize {
        let ret: isize;
        core::arch::asm!(
            "syscall",
            inlateout("rax") nr => ret,
            in("rdi") a1, in("rsi") a2, in("rdx") a3, in("r10") a4, in("r8") a5,
            lateout("rcx") _, lateout("r11") _,
            options(nostack)
        );
        ret
    }

    #[inline]
    pub unsafe fn syscall6(
        nr: isize,
        a1: isize,
        a2: isize,
        a3: isize,
        a4: isize,
        a5: isize,
        a6: isize,
    ) -> isize {
        let ret: isize;
        core::arch::asm!(
            "syscall",
            inlateout("rax") nr => ret,
            in("rdi") a1, in("rsi") a2, in("rdx") a3,
            in("r10") a4, in("r8") a5, in("r9") a6,
            lateout("rcx") _, lateout("r11") _,
            options(nostack)
        );
        ret
    }
}

/// PLAN-1040: one raw `write` syscall (the runner's console host uses the
/// same primitive surface as the platform backend — no libc stdio). Only
/// compiled on Linux; returns the raw kernel value (count or -errno).
#[cfg(target_os = "linux")]
pub fn linux_raw_write(fd: i32, buf: usize, len: usize) -> isize {
    unsafe { linux_asm::syscall3(linux_sysno::WRITE, fd as isize, buf as isize, len as isize) }
}

/// Discover the page size from THIS process's auxv (`AT_PAGESZ`, key 6)
/// using only raw syscalls: openat("/proc/self/auxv") + bounded read loop
/// + close. No `std::fs`, no libc `sysconf`, no cross-arch hardcode. A
/// process whose /proc is missing gets `None` — the ledger then rejects
/// mem ops fail-closed instead of guessing.
#[cfg(target_os = "linux")]
fn linux_auxv_page_size() -> Option<u32> {
    use std::sync::OnceLock;
    static PAGE: OnceLock<Option<u32>> = OnceLock::new();
    *PAGE.get_or_init(|| {
        const AT_PAGESZ: isize = 6;
        const AUXV_PAIRS: usize = 64; // auxv is a few dozen pairs; bound the loop
        const AT_NULL: isize = 0;
        let path = b"/proc/self/auxv\0";
        let fd = unsafe {
            linux_asm::syscall4(
                linux_sysno::OPENAT,
                PROBE_AT_FDCWD as isize,
                path.as_ptr() as isize,
                0, // O_RDONLY
                0,
            )
        };
        if linux_is_errno(fd) {
            return None;
        }
        let mut buf = [0u8; 16 * AUXV_PAIRS];
        let mut filled = 0usize;
        while filled < buf.len() {
            let n = unsafe {
                linux_asm::syscall3(
                    linux_sysno::READ,
                    fd,
                    buf.as_mut_ptr().add(filled) as isize,
                    (buf.len() - filled) as isize,
                )
            };
            if linux_is_errno(n) {
                unsafe { linux_asm::syscall1(linux_sysno::CLOSE, fd) };
                return None;
            }
            if n == 0 {
                break;
            }
            filled += n as usize;
        }
        unsafe { linux_asm::syscall1(linux_sysno::CLOSE, fd) };
        for pair in buf[..filled].chunks_exact(16) {
            let key = isize::from_ne_bytes(pair[0..8].try_into().ok()?);
            let val = isize::from_ne_bytes(pair[8..16].try_into().ok()?);
            if key == AT_NULL {
                break;
            }
            if key == AT_PAGESZ {
                return u32::try_from(val)
                    .ok()
                    .filter(|p| *p > 0 && p.is_power_of_two());
            }
        }
        None
    })
}

/// PLAN-1040: the Linux leg of `PlatformIoBackend` — direct kernel entry
/// points only (write/mmap/munmap/openat/close via raw syscall), page
/// size from this process's auxv. `std_handle` returns the fixed Linux
/// std fds (1/2) — no query call exists or is needed.
#[derive(Debug, Default)]
pub struct LinuxPlatformIo {
    pub dispatches: DispatchCounters,
    /// PLAN-973: wait4 reaps the child, so the raw wait status is cached
    /// here for the follow-up `child_exit_code` query.
    pub last_wait_status: i32,
}

impl LinuxPlatformIo {
    /// True when this process can actually serve the Linux backend (raw
    /// syscalls + auxv page size). Hosts use it to fail closed.
    #[cfg(target_os = "linux")]
    pub fn available() -> bool {
        linux_auxv_page_size().is_some()
    }

    #[cfg(not(target_os = "linux"))]
    pub fn available() -> bool {
        false
    }
}

impl PlatformIoBackend for LinuxPlatformIo {
    #[cfg(target_os = "linux")]
    fn std_handle(&mut self, stream: StreamKind) -> Result<usize, u32> {
        self.dispatches.std_handle += 1;
        Ok(match stream {
            StreamKind::Stdout => 1,
            StreamKind::Stderr => 2,
        })
    }

    #[cfg(not(target_os = "linux"))]
    fn std_handle(&mut self, _stream: StreamKind) -> Result<usize, u32> {
        self.dispatches.std_handle += 1;
        Err(PROBE_UNSUPPORTED)
    }

    #[cfg(target_os = "linux")]
    fn write_some(
        &mut self,
        _stream: StreamKind,
        os_handle: usize,
        bytes: &[u8],
        cancel: &CancelFlag,
    ) -> Result<usize, u32> {
        if CancelCheck::poll(cancel) {
            return Err(125); // ECANCELED
        }
        self.dispatches.write_some += 1;
        if bytes.is_empty() {
            return Ok(0);
        }
        // Bounded EINTR retry: the raw write made no progress when it
        // returns -EINTR, so restarting the same call is lossless; every
        // retry re-checks cancellation so the loop cannot spin unbounded.
        loop {
            if CancelCheck::poll(cancel) {
                return Err(125); // ECANCELED
            }
            let ret = unsafe {
                linux_asm::syscall3(
                    linux_sysno::WRITE,
                    os_handle as isize,
                    bytes.as_ptr() as isize,
                    bytes.len() as isize,
                )
            };
            if ret >= 0 {
                return Ok(ret as usize);
            }
            if linux_is_errno(ret) {
                let errno = (-ret) as u32;
                if errno == 4 {
                    continue; // EINTR with zero progress — retry
                }
                return Err(errno);
            }
            return Err(linux_non_errno_fault(ret));
        }
    }

    #[cfg(not(target_os = "linux"))]
    fn write_some(
        &mut self,
        _stream: StreamKind,
        _os_handle: usize,
        _bytes: &[u8],
        _cancel: &CancelFlag,
    ) -> Result<usize, u32> {
        self.dispatches.write_some += 1;
        Err(PROBE_UNSUPPORTED)
    }

    #[cfg(target_os = "linux")]
    fn system_info(&mut self) -> (u32, u32) {
        self.dispatches.system_info += 1;
        match linux_auxv_page_size() {
            Some(page) => (page, page), // mmap granularity == page size on Linux
            None => (0, 0),
        }
    }

    #[cfg(not(target_os = "linux"))]
    fn system_info(&mut self) -> (u32, u32) {
        self.dispatches.system_info += 1;
        (0, 0)
    }

    #[cfg(target_os = "linux")]
    fn virtual_alloc(&mut self, commit_bytes: usize) -> Result<*mut u8, u32> {
        self.dispatches.virtual_alloc += 1;
        let ret = unsafe {
            linux_asm::syscall6(
                linux_sysno::MMAP,
                0, // addr: kernel chooses (never MAP_FIXED)
                commit_bytes as isize,
                LINUX_PROT_RW,
                LINUX_MAP_PRIVATE_ANON,
                -1,
                0,
            )
        };
        if linux_is_errno(ret) {
            return Err((-ret) as u32);
        }
        Ok(ret as usize as *mut u8)
    }

    #[cfg(not(target_os = "linux"))]
    fn virtual_alloc(&mut self, _commit_bytes: usize) -> Result<*mut u8, u32> {
        self.dispatches.virtual_alloc += 1;
        Err(PROBE_UNSUPPORTED)
    }

    #[cfg(target_os = "linux")]
    fn virtual_free(&mut self, base: *mut u8, mapped_len: usize) -> Result<(), u32> {
        self.dispatches.virtual_free += 1;
        let ret =
            unsafe { linux_asm::syscall2(linux_sysno::MUNMAP, base as isize, mapped_len as isize) };
        if ret == 0 {
            return Ok(());
        }
        if linux_is_errno(ret) {
            return Err((-ret) as u32);
        }
        Err(linux_non_errno_fault(ret))
    }

    #[cfg(not(target_os = "linux"))]
    fn virtual_free(&mut self, _base: *mut u8, _mapped_len: usize) -> Result<(), u32> {
        self.dispatches.virtual_free += 1;
        Err(PROBE_UNSUPPORTED)
    }

    #[cfg(target_os = "linux")]
    fn probe_openat(&mut self, dirfd: i32, path: &[u8], flags: i32, mode: i32) -> Result<i32, u32> {
        if path.len() + 1 > MAX_PROBE_PATH_BYTES + 1 {
            return Err(36); // ENAMETOOLONG
        }
        // NUL-terminate on the OS boundary only (fixed kernel-side buffer).
        let mut buf = [0u8; MAX_PROBE_PATH_BYTES + 1];
        buf[..path.len()].copy_from_slice(path);
        let ret = unsafe {
            linux_asm::syscall4(
                linux_sysno::OPENAT,
                dirfd as isize,
                buf.as_ptr() as isize,
                (flags as isize) | 0o2000000, // O_CLOEXEC
                mode as isize,
            )
        };
        // Raw syscall result crosses in Ok(): positive fd, or the raw
        // NEGATIVE errno (the probe seam's contract — classification is
        // the guest's oracle, not a host ERR_* code).
        Ok(ret as i32)
    }

    #[cfg(not(target_os = "linux"))]
    fn probe_openat(
        &mut self,
        _dirfd: i32,
        _path: &[u8],
        _flags: i32,
        _mode: i32,
    ) -> Result<i32, u32> {
        Err(PROBE_UNSUPPORTED)
    }

    #[cfg(target_os = "linux")]
    fn probe_write_fd(&mut self, fd: i32, bytes: &[u8]) -> Result<i32, u32> {
        if bytes.is_empty() {
            return Ok(0);
        }
        let ret = unsafe {
            linux_asm::syscall3(
                linux_sysno::WRITE,
                fd as isize,
                bytes.as_ptr() as isize,
                bytes.len() as isize,
            )
        };
        Ok(ret as i32)
    }

    #[cfg(not(target_os = "linux"))]
    fn probe_write_fd(&mut self, _fd: i32, _bytes: &[u8]) -> Result<i32, u32> {
        Err(PROBE_UNSUPPORTED)
    }

    #[cfg(target_os = "linux")]
    fn probe_close(&mut self, fd: i32) -> Result<i32, u32> {
        let ret = unsafe { linux_asm::syscall1(linux_sysno::CLOSE, fd as isize) };
        Ok(ret as i32)
    }

    #[cfg(not(target_os = "linux"))]
    fn probe_close(&mut self, _fd: i32) -> Result<i32, u32> {
        Err(PROBE_UNSUPPORTED)
    }

    // PLAN-1041: Linux std streams are always byte sinks (the kernel does
    // not interpret UTF-8 for the terminal); there is no Windows
    // text-console write here — the common algorithm never selects it.
    fn console_kind(&mut self, _os_handle: usize) -> Result<i32, u32> {
        self.dispatches.console_kind += 1;
        Ok(CONSOLE_KIND_BYTE_SINK)
    }

    fn console_write16(
        &mut self,
        _stream: StreamKind,
        _os_handle: usize,
        _units: &[u16],
        _cancel: &CancelFlag,
    ) -> Result<usize, u32> {
        self.dispatches.console_write16 += 1;
        Err(CONSOLE_WRITE16_UNSUPPORTED)
    }

    // ---- PLAN-973: Linux raw-syscall system-resource surface ---------------
    // Compiled only on Linux targets; per the 2026-11-15 Linux ruling these
    // arms are CODE-ONLY from Windows development (deferred-required /
    // NOT_RUN, owner = PLAN-1063) — they never silently stand in for the
    // Win32 arms on a Windows host.

    #[cfg(target_os = "linux")]
    fn map_open_flags(flags: i32) -> i32 {
        use linux_oflags as o;
        let mut native = if flags & OPEN_WRITE != 0 || flags & OPEN_APPEND != 0 {
            if flags & OPEN_READ != 0 {
                o::O_RDWR
            } else {
                o::O_WRONLY
            }
        } else {
            o::O_RDONLY
        };
        if flags & OPEN_CREATE != 0 {
            native |= o::O_CREAT;
        }
        if flags & OPEN_TRUNCATE != 0 {
            native |= o::O_TRUNC;
        }
        if flags & OPEN_EXCLUSIVE != 0 {
            native |= o::O_EXCL;
        }
        if flags & OPEN_APPEND != 0 {
            native |= o::O_APPEND;
        }
        native | o::O_CLOEXEC
    }

    #[cfg(target_os = "linux")]
    fn path_z(path: &[u8]) -> Result<[u8; MAX_PROBE_PATH_BYTES + 1], u32> {
        if path.len() + 1 > MAX_PROBE_PATH_BYTES + 1 {
            return Err(36); // ENAMETOOLONG
        }
        let mut buf = [0u8; MAX_PROBE_PATH_BYTES + 1];
        buf[..path.len()].copy_from_slice(path);
        Ok(buf)
    }

    #[cfg(target_os = "linux")]
    fn file_open_bytes(&mut self, path: &[u8], flags: i32) -> Result<usize, u32> {
        self.dispatches.sys += 1;
        let buf = Self::path_z(path)?;
        let ret = unsafe {
            linux_asm::syscall4(
                linux_sysno::OPENAT,
                PROBE_AT_FDCWD as isize,
                buf.as_ptr() as isize,
                Self::map_open_flags(flags) as isize,
                if flags & OPEN_CREATE != 0 {
                    LINUX_DEFAULT_MODE as isize
                } else {
                    0
                },
            )
        };
        if linux_is_errno(ret) {
            return Err((-ret) as u32);
        }
        Ok(ret as usize)
    }

    #[cfg(target_os = "linux")]
    fn file_read_some(
        &mut self,
        os_handle: usize,
        buf: &mut [u8],
        cancel: &CancelFlag,
    ) -> Result<usize, u32> {
        if CancelCheck::poll(cancel) {
            return Err(125); // ECANCELED
        }
        self.dispatches.sys += 1;
        if buf.is_empty() {
            return Ok(0);
        }
        loop {
            let ret = unsafe {
                linux_asm::syscall3(
                    linux_sysno::READ,
                    os_handle as isize,
                    buf.as_mut_ptr() as isize,
                    buf.len() as isize,
                )
            };
            if ret >= 0 {
                return Ok(ret as usize);
            }
            if linux_is_errno(ret) {
                let errno = (-ret) as u32;
                if errno == 4 {
                    continue; // EINTR with zero progress
                }
                return Err(errno);
            }
            return Err(linux_non_errno_fault(ret));
        }
    }

    #[cfg(target_os = "linux")]
    fn file_write_some(
        &mut self,
        os_handle: usize,
        bytes: &[u8],
        cancel: &CancelFlag,
    ) -> Result<usize, u32> {
        if CancelCheck::poll(cancel) {
            return Err(125);
        }
        self.dispatches.sys += 1;
        if bytes.is_empty() {
            return Ok(0);
        }
        loop {
            let ret = unsafe {
                linux_asm::syscall3(
                    linux_sysno::WRITE,
                    os_handle as isize,
                    bytes.as_ptr() as isize,
                    bytes.len() as isize,
                )
            };
            if ret >= 0 {
                return Ok(ret as usize);
            }
            if linux_is_errno(ret) {
                let errno = (-ret) as u32;
                if errno == 4 {
                    continue; // EINTR with zero progress
                }
                return Err(errno);
            }
            return Err(linux_non_errno_fault(ret));
        }
    }

    #[cfg(target_os = "linux")]
    fn file_seek(&mut self, os_handle: usize, origin: i32, offset: i64) -> Result<i64, u32> {
        self.dispatches.sys += 1;
        let whence = match origin {
            0 => 0, // SEEK_SET
            1 => 1, // SEEK_CUR
            2 => 2, // SEEK_END
            _ => return Err(22),
        };
        let ret = unsafe {
            linux_asm::syscall4(
                linux_sysno::LSEEK,
                os_handle as isize,
                offset as isize,
                whence as isize,
                0,
            )
        };
        if linux_is_errno(ret) {
            return Err((-ret) as u32);
        }
        Ok(ret)
    }

    #[cfg(target_os = "linux")]
    fn file_tell(&mut self, os_handle: usize) -> Result<i64, u32> {
        self.dispatches.sys += 1;
        let ret = unsafe { linux_asm::syscall4(linux_sysno::LSEEK, os_handle as isize, 0, 1, 0) };
        if linux_is_errno(ret) {
            return Err((-ret) as u32);
        }
        Ok(ret)
    }

    /// Read the x86-64 `struct stat` fields via `fstat` into a stack
    /// buffer; returns (size, mtime_ms) — the fixed offsets are the
    /// glibc/kernel stat64 layout (size at 48, mtim at 96).
    #[cfg(target_os = "linux")]
    fn fstat_fields(os_handle: usize) -> Result<(u64, i64), u32> {
        let mut st = [0u8; 144];
        let ret = unsafe {
            linux_asm::syscall2(
                linux_sysno::FSTAT,
                os_handle as isize,
                st.as_mut_ptr() as isize,
            )
        };
        if linux_is_errno(ret) {
            return Err((-ret) as u32);
        }
        if ret != 0 {
            return Err(linux_non_errno_fault(ret));
        }
        let size = u64::from_le_bytes(st[48..56].try_into().unwrap());
        let sec = i64::from_le_bytes(st[96..104].try_into().unwrap());
        let nsec = i64::from_le_bytes(st[104..112].try_into().unwrap());
        Ok((size, sec.saturating_mul(1000) + nsec / 1_000_000))
    }

    #[cfg(target_os = "linux")]
    fn file_size(&mut self, os_handle: usize) -> Result<u64, u32> {
        self.dispatches.sys += 1;
        Self::fstat_fields(os_handle).map(|(s, _)| s)
    }

    #[cfg(target_os = "linux")]
    fn file_mtime_ms(&mut self, os_handle: usize) -> Result<i64, u32> {
        self.dispatches.sys += 1;
        Self::fstat_fields(os_handle).map(|(_, m)| m)
    }

    #[cfg(target_os = "linux")]
    fn file_flush(&mut self, os_handle: usize) -> Result<(), u32> {
        self.dispatches.sys += 1;
        let ret = unsafe { linux_asm::syscall1(linux_sysno::FSYNC, os_handle as isize) };
        if linux_is_errno(ret) {
            return Err((-ret) as u32);
        }
        if ret != 0 {
            return Err(linux_non_errno_fault(ret));
        }
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn file_close(&mut self, os_handle: usize) -> Result<(), u32> {
        self.dispatches.sys += 1;
        let ret = unsafe { linux_asm::syscall1(linux_sysno::CLOSE, os_handle as isize) };
        if linux_is_errno(ret) {
            // A failed close never retries (the fd may already be gone).
            return Err((-ret) as u32);
        }
        Ok(())
    }

    /// Attribute bits via O_PATH open + fstat (no path content is touched).
    #[cfg(target_os = "linux")]
    fn attrs_by_open(&mut self, path: &[u8]) -> Result<i32, u32> {
        let buf = Self::path_z(path)?;
        let fd = unsafe {
            linux_asm::syscall4(
                linux_sysno::OPENAT,
                PROBE_AT_FDCWD as isize,
                buf.as_ptr() as isize,
                (linux_oflags::O_PATH | linux_oflags::O_CLOEXEC) as isize,
                0,
            )
        };
        if linux_is_errno(fd) {
            return Err((-fd) as u32);
        }
        let mut st = [0u8; 144];
        let ret = unsafe { linux_asm::syscall2(linux_sysno::FSTAT, fd, st.as_mut_ptr() as isize) };
        unsafe { linux_asm::syscall1(linux_sysno::CLOSE, fd) };
        if linux_is_errno(ret) || ret != 0 {
            return Err(5);
        }
        let mode = u32::from_le_bytes(st[24..28].try_into().unwrap());
        let mut bits = 0;
        if mode & 0o170000 == 0o040000 {
            bits |= 1; // S_IFDIR
        }
        if mode & 0o200 == 0 {
            bits |= 2; // owner-write bit clear = read-only
        }
        Ok(bits)
    }

    #[cfg(target_os = "linux")]
    fn path_attrs_bytes(&mut self, path: &[u8]) -> Result<i32, u32> {
        self.dispatches.sys += 1;
        self.attrs_by_open(path)
    }

    #[cfg(target_os = "linux")]
    fn file_delete_bytes(&mut self, path: &[u8]) -> Result<(), u32> {
        self.dispatches.sys += 1;
        let buf = Self::path_z(path)?;
        let ret = unsafe {
            linux_asm::syscall4(
                linux_sysno::UNLINKAT,
                PROBE_AT_FDCWD as isize,
                buf.as_ptr() as isize,
                0,
            )
        };
        if linux_is_errno(ret) {
            return Err((-ret) as u32);
        }
        if ret != 0 {
            return Err(linux_non_errno_fault(ret));
        }
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn dir_create_bytes(&mut self, path: &[u8]) -> Result<(), u32> {
        self.dispatches.sys += 1;
        let buf = Self::path_z(path)?;
        let ret = unsafe {
            linux_asm::syscall4(
                linux_sysno::MKDIRAT,
                PROBE_AT_FDCWD as isize,
                buf.as_ptr() as isize,
                0o777,
            )
        };
        if linux_is_errno(ret) {
            return Err((-ret) as u32);
        }
        if ret != 0 {
            return Err(linux_non_errno_fault(ret));
        }
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn dir_delete_bytes(&mut self, path: &[u8]) -> Result<(), u32> {
        self.dispatches.sys += 1;
        let buf = Self::path_z(path)?;
        // AT_REMOVEDIR = 0x200
        let ret = unsafe {
            linux_asm::syscall4(
                linux_sysno::UNLINKAT,
                PROBE_AT_FDCWD as isize,
                buf.as_ptr() as isize,
                0x200,
            )
        };
        if linux_is_errno(ret) {
            return Err((-ret) as u32);
        }
        if ret != 0 {
            return Err(linux_non_errno_fault(ret));
        }
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn file_replace_bytes(&mut self, src: &[u8], dst: &[u8]) -> Result<(), u32> {
        self.dispatches.sys += 1;
        let s = Self::path_z(src)?;
        let d = Self::path_z(dst)?;
        let ret = unsafe {
            linux_asm::syscall4(
                linux_sysno::RENAMEAT,
                PROBE_AT_FDCWD as isize,
                s.as_ptr() as isize,
                PROBE_AT_FDCWD as isize,
                d.as_ptr() as isize,
            )
        };
        if linux_is_errno(ret) {
            return Err((-ret) as u32);
        }
        if ret != 0 {
            return Err(linux_non_errno_fault(ret));
        }
        Ok(())
    }

    /// Linux canonicalize: NAMED-DEFERRED (PLAN-1063). The no-libc
    /// realpath is a component walk with O_PATH opens + readlinkat on
    /// /proc/self/fd — its absolute-symlink restart and .. interactions
    /// need on-target compile+validation (the arm is cfg(linux), so a
    /// Windows dev host cannot even compile-check it). v1 canonicalizes
    /// syntactically: normalize separators and . / .. against the cwd
    /// WITHOUT symlink resolution, and says so via the plan's handover
    /// table; 1063 delivers the resolved walk and its fixtures.
    #[cfg(target_os = "linux")]
    fn canonical_bytes(&mut self, path: &[u8], out: &mut [u8]) -> Result<usize, u32> {
        self.dispatches.sys += 1;
        if path.is_empty() || path.contains(&0) {
            return Err(22);
        }
        let mut resolved: Vec<u8> = Vec::with_capacity(path.len() + 64);
        if path[0] == b'/' {
            resolved.push(b'/');
        } else {
            let mut cwd = vec![0u8; MAX_PROBE_PATH_BYTES];
            let n = unsafe {
                linux_asm::syscall2(
                    linux_sysno::GETCWD,
                    cwd.as_mut_ptr() as isize,
                    cwd.len() as isize,
                )
            };
            if linux_is_errno(n) || n <= 0 {
                return Err(2);
            }
            cwd.truncate(n as usize);
            resolved = cwd;
        }
        let mut parts: Vec<&[u8]> = Vec::new();
        for seg in path.split(|b| *b == b'/') {
            if seg.is_empty() || seg == b"." {
                continue;
            }
            if seg == b".." {
                parts.pop();
                continue;
            }
            parts.push(seg);
        }
        for seg in parts {
            if resolved.last() != Some(&b'/') {
                resolved.push(b'/');
            }
            resolved.extend_from_slice(seg);
        }
        if resolved.len() > out.len() {
            return Err(36);
        }
        out[..resolved.len()].copy_from_slice(&resolved);
        Ok(resolved.len())
    }

    #[cfg(target_os = "linux")]
    fn dir_open_bytes(&mut self, path: &[u8]) -> Result<usize, u32> {
        self.dispatches.sys += 1;
        let buf = Self::path_z(path)?;
        let ret = unsafe {
            linux_asm::syscall4(
                linux_sysno::OPENAT,
                PROBE_AT_FDCWD as isize,
                buf.as_ptr() as isize,
                (linux_oflags::O_RDONLY | linux_oflags::O_DIRECTORY | linux_oflags::O_CLOEXEC)
                    as isize,
                0,
            )
        };
        if linux_is_errno(ret) {
            return Err((-ret) as u32);
        }
        Ok(ret as usize)
    }

    #[cfg(target_os = "linux")]
    fn dir_fill_bytes(&mut self, os_handle: usize, buf: &mut [u8]) -> Result<usize, u32> {
        self.dispatches.sys += 1;
        loop {
            let ret = unsafe {
                linux_asm::syscall3(
                    linux_sysno::GETDENTS64,
                    os_handle as isize,
                    buf.as_mut_ptr() as isize,
                    buf.len() as isize,
                )
            };
            if ret >= 0 {
                return Ok(ret as usize);
            }
            if linux_is_errno(ret) {
                let errno = (-ret) as u32;
                if errno == 4 {
                    continue;
                }
                return Err(errno);
            }
            return Err(linux_non_errno_fault(ret));
        }
    }

    #[cfg(target_os = "linux")]
    fn dir_close(&mut self, os_handle: usize) -> Result<(), u32> {
        self.file_close(os_handle)
    }

    /// /proc/self/environ snapshot (NUL-separated `KEY=VALUE` bytes).
    #[cfg(target_os = "linux")]
    fn env_get_bytes(&mut self, out: &mut [u8]) -> Result<usize, u32> {
        self.dispatches.sys += 1;
        let path = b"/proc/self/environ\0";
        let fd = unsafe {
            linux_asm::syscall4(
                linux_sysno::OPENAT,
                PROBE_AT_FDCWD as isize,
                path.as_ptr() as isize,
                linux_oflags::O_RDONLY as isize,
                0,
            )
        };
        if linux_is_errno(fd) {
            return Err((-fd) as u32);
        }
        let mut filled = 0usize;
        while filled < out.len() {
            let n = unsafe {
                linux_asm::syscall3(
                    linux_sysno::READ,
                    fd,
                    out.as_mut_ptr().add(filled) as isize,
                    (out.len() - filled) as isize,
                )
            };
            if linux_is_errno(n) {
                unsafe { linux_asm::syscall1(linux_sysno::CLOSE, fd) };
                return Err((-n) as u32);
            }
            if n == 0 {
                break;
            }
            filled += n as usize;
        }
        unsafe { linux_asm::syscall1(linux_sysno::CLOSE, fd) };
        Ok(filled)
    }

    #[cfg(target_os = "linux")]
    fn clock_ms(clk_id: isize) -> Result<i64, u32> {
        // clock_gettime(clk, &timespec{tv_sec i64, tv_nsec i64})
        let mut ts = [0i64; 2];
        let ret = unsafe {
            linux_asm::syscall2(linux_sysno::CLOCK_GETTIME, clk_id, ts.as_mut_ptr() as isize)
        };
        if linux_is_errno(ret) || ret != 0 {
            return Err(22);
        }
        Ok(ts[0].saturating_mul(1000) + ts[1] / 1_000_000)
    }

    #[cfg(target_os = "linux")]
    fn clock_unix_ms(&mut self) -> Result<i64, u32> {
        self.dispatches.sys += 1;
        Self::clock_ms(0) // CLOCK_REALTIME
    }

    #[cfg(target_os = "linux")]
    fn clock_mono_ms(&mut self) -> Result<i64, u32> {
        self.dispatches.sys += 1;
        Self::clock_ms(1) // CLOCK_MONOTONIC
    }

    #[cfg(target_os = "linux")]
    fn sleep_ms(&mut self, ms: u32, cancel: &CancelFlag) -> Result<(), u32> {
        self.dispatches.sys += 1;
        const SLICE_MS: u32 = 50;
        let mut remaining = ms;
        loop {
            if CancelCheck::poll(cancel) {
                return Err(125);
            }
            if remaining == 0 {
                return Ok(());
            }
            let slice = remaining.min(SLICE_MS);
            // nanosleep(&{sec, nsec}, NULL) — EINTR keeps partial progress
            let mut ts = [(slice / 1000) as i64, ((slice % 1000) as i64) * 1_000_000];
            unsafe { linux_asm::syscall2(linux_sysno::NANOSLEEP, ts.as_mut_ptr() as isize, 0) };
            remaining -= slice;
        }
    }

    #[cfg(target_os = "linux")]
    fn rand_fill(&mut self, buf: &mut [u8]) -> Result<(), u32> {
        self.dispatches.sys += 1;
        if buf.is_empty() {
            return Ok(());
        }
        let mut filled = 0usize;
        while filled < buf.len() {
            let n = unsafe {
                linux_asm::syscall4(
                    linux_sysno::GETRANDOM,
                    buf.as_mut_ptr().add(filled) as isize,
                    (buf.len() - filled) as isize,
                    0,
                    0,
                )
            };
            if linux_is_errno(n) {
                let errno = (-n) as u32;
                if errno == 4 {
                    continue; // EINTR
                }
                return Err(errno);
            }
            filled += n as usize;
        }
        Ok(())
    }

    /// Linux spawn: NAMED-DEFERRED (PLAN-1063). The no-libc path is
    /// pipe(2) + clone(CLONE_VM|CLONE_VFORK|CLONE_NEWPGROUP) on a private
    /// stack with a raw-syscall child shim (close write ends → dup2 →
    /// execve), kill_tree = kill(-pgid, SIGKILL). The child-side shim and
    /// the vfork resume discipline can only be validated on a Linux
    /// target (the arm is cfg(linux); a Windows host never compiles it) —
    /// shipping unvalidated child-machine-sequence code would be worse
    /// than the named gap, so v1 reports unsupported and 1063 delivers
    /// the sequence with real fixtures.
    #[cfg(target_os = "linux")]
    fn proc_spawn_bytes(&mut self, _argv_blob: &[u8], _capture: bool) -> Result<ProcSpawn, u32> {
        self.dispatches.sys += 1;
        Err(PROBE_UNSUPPORTED)
    }

    #[cfg(target_os = "linux")]
    fn pipe_avail(&mut self, os_handle: usize) -> Result<usize, u32> {
        self.dispatches.sys += 1;
        let mut avail: i32 = 0;
        let ret = unsafe {
            linux_asm::syscall3(
                linux_sysno::IOCTL,
                os_handle as isize,
                LINUX_FIONREAD,
                &mut avail as *mut i32 as isize,
            )
        };
        if linux_is_errno(ret) || ret != 0 {
            return Err(5);
        }
        Ok(avail.max(0) as usize)
    }

    #[cfg(target_os = "linux")]
    fn wait_child(&mut self, os_handle: usize, timeout_ms: i32) -> Result<bool, u32> {
        self.dispatches.sys += 1;
        // wait4(pid, &status, WNOHANG, NULL) — WNOWAIT is not exposed by
        // wait4; reap only when exiting: keep zombie for exit-code query
        // by reading the status in the same call and caching it in the
        // caller (the ledger re-queries via child_exit_code; v1 re-waits
        // WNOHANG after reaping is impossible — so reap here and let the
        // ledger carry the code; child_exit_code returns the last status).
        let mut status: i32 = 0;
        let deadline = timeout_ms;
        let mut waited = 0i32;
        loop {
            let ret = unsafe {
                linux_asm::syscall4(
                    linux_sysno::WAIT4,
                    os_handle as isize,
                    &mut status as *mut i32 as isize,
                    1, // WNOHANG
                    0,
                )
            };
            if linux_is_errno(ret) {
                return Err((-ret) as u32);
            }
            if ret == os_handle as isize {
                self.last_wait_status = status;
                return Ok(true);
            }
            if deadline >= 0 && waited >= deadline {
                return Ok(false);
            }
            let mut ts = [0i64, 20_000_000i64]; // 20 ms
            unsafe { linux_asm::syscall2(linux_sysno::NANOSLEEP, ts.as_mut_ptr() as isize, 0) };
            waited += 20;
        }
    }

    #[cfg(target_os = "linux")]
    fn child_exit_code(&mut self, os_handle: usize) -> Result<i32, u32> {
        self.dispatches.sys += 1;
        let status = self.last_wait_status;
        if status & 0x7F == 0 {
            return Ok((status >> 8) & 0xFF);
        }
        Ok(-(status & 0x7F)) // signal death: negative signal number
    }

    #[cfg(target_os = "linux")]
    fn kill_tree(&mut self, job_handle: usize) -> Result<(), u32> {
        self.dispatches.sys += 1;
        // kill(-pgid, SIGKILL) — the whole group (grandchildren included
        // unless they detached via setsid).
        let ret = unsafe { linux_asm::syscall2(linux_sysno::KILL, -(job_handle as isize), 9) };
        if linux_is_errno(ret) && (-ret) as u32 != 3 {
            // ESRCH (3) = already gone — success for our purpose
            return Err((-ret) as u32);
        }
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn close_raw(&mut self, os_handle: usize) -> Result<(), u32> {
        self.file_close(os_handle)
    }

    #[cfg(target_os = "linux")]
    fn proc_cmdline_fill_bytes(&mut self, out: &mut [u8]) -> Result<usize, u32> {
        self.dispatches.sys += 1;
        let path = b"/proc/self/cmdline\0";
        let fd = unsafe {
            linux_asm::syscall4(
                linux_sysno::OPENAT,
                PROBE_AT_FDCWD as isize,
                path.as_ptr() as isize,
                linux_oflags::O_RDONLY as isize,
                0,
            )
        };
        if linux_is_errno(fd) {
            return Err((-fd) as u32);
        }
        let mut filled = 0usize;
        while filled < out.len() {
            let n = unsafe {
                linux_asm::syscall3(
                    linux_sysno::READ,
                    fd,
                    out.as_mut_ptr().add(filled) as isize,
                    (out.len() - filled) as isize,
                )
            };
            if linux_is_errno(n) {
                unsafe { linux_asm::syscall1(linux_sysno::CLOSE, fd) };
                return Err((-n) as u32);
            }
            if n == 0 {
                break;
            }
            filled += n as usize;
        }
        unsafe { linux_asm::syscall1(linux_sysno::CLOSE, fd) };
        Ok(filled)
    }

    #[cfg(target_os = "linux")]
    fn proc_cwd_bytes(&mut self, out: &mut [u8]) -> Result<usize, u32> {
        self.dispatches.sys += 1;
        let ret = unsafe {
            linux_asm::syscall2(
                linux_sysno::GETCWD,
                out.as_mut_ptr() as isize,
                out.len() as isize,
            )
        };
        if linux_is_errno(ret) {
            return Err((-ret) as u32);
        }
        if ret == 0 {
            return Err(5);
        }
        Ok(ret as usize)
    }

    #[cfg(target_os = "linux")]
    fn proc_chdir_bytes(&mut self, path: &[u8]) -> Result<(), u32> {
        self.dispatches.sys += 1;
        let buf = Self::path_z(path)?;
        let ret = unsafe { linux_asm::syscall1(linux_sysno::CHDIR, buf.as_ptr() as isize) };
        if linux_is_errno(ret) {
            return Err((-ret) as u32);
        }
        if ret != 0 {
            return Err(linux_non_errno_fault(ret));
        }
        Ok(())
    }
}

/// A raw syscall returned a negative value outside the errno interval —
/// cannot happen on x86-64 Linux; report it as a distinct host fault
/// instead of misclassifying it. (Only reachable from the linux cfg arms.)
#[cfg(target_os = "linux")]
fn linux_non_errno_fault(ret: isize) -> u32 {
    let bits = ret as u64;
    u32::try_from(bits & 0xFFFF).unwrap_or(0xFFFF) | 0x0010_4000
}

// PLAN-973 ledger op discriminants (private dispatch helpers).
#[derive(Clone, Copy)]
enum FileQuery {
    Tell,
    Size,
    MtimeMs,
}

#[derive(Clone, Copy)]
enum PathUnitOp {
    Attrs,
    Delete,
    DirCreate,
    DirDelete,
}

#[derive(Clone, Copy)]
enum PathBytesOp {
    Attrs,
    Delete,
    DirCreate,
    DirDelete,
}

#[derive(Clone, Copy)]
enum OutUnitOp {
    Cmdline,
    Cwd,
}

#[derive(Clone, Copy)]
enum OutBytesOp {
    Cmdline,
    Cwd,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OwnerKind {
    GuestAv,
}

#[derive(Debug)]
enum LedgerPayload {
    StdStream {
        stream: StreamKind,
        os_handle: usize,
    },
    OsMemRegion {
        base: *mut u8,
        logical_len: usize,
        mapped_len: usize,
        borrow_count: u32,
        released: bool,
    },
    /// PLAN-974: the socket lives in the backend; the ledger tracks the
    /// backend id + release state (the single ownership account).
    NetSocket { socket: u64, released: bool },
    /// PLAN-974 inc3: backend-owned listener id.
    NetListener { listener: u64, released: bool },
    /// PLAN-973: an open OS file handle. `last64_hi` keeps the high 32
    /// bits of the last 64-bit result (seek/tell/size/mtime) for the
    /// `file_hi` paired query.
    OsFile {
        os_handle: usize,
        last64_hi: u32,
        closed: bool,
    },
    /// PLAN-973: a directory-enumeration handle. `pending` drains the
    /// `FindFirstFileW` first entry before `FindNextFileW` runs.
    OsDir {
        os_handle: usize,
        pending: Option<Vec<u16>>,
        closed: bool,
    },
    /// PLAN-973: a spawned child with its kill-on-close job tree and the
    /// parent-side capture-pipe read ends.
    OsProcess { spawn: ProcSpawn, released: bool },
}

#[derive(Debug)]
struct LedgerEntry {
    generation: u16,
    owner: OwnerKind,
    session_id: u64,
    payload: LedgerPayload,
}

/// Unified Host Resource Ledger owning BOTH stdio handles and OS virtual memory regions
/// for a single execution session (no parallel ownership ledger).
#[derive(Debug)]
pub struct PlatformResourceLedger {
    session_id: u64,
    next_gen: u16,
    entries: Vec<LedgerEntry>,
    grants: BTreeSet<String>,
    pub last_os_error: u32,
    pub written_total_bytes: usize,
    pub live_mapped_bytes: usize,
    /// PLAN-1040 probe seam: cumulative probe bytes per run (defensive
    /// mirror of the console run budget; probes are internal, not public).
    pub probe_written_total: usize,
    /// PLAN-974: cumulative net bytes (read + written) this run.
    pub net_bytes_total: usize,
    /// PLAN-973: cumulative bytes read into the guest this run (files,
    /// pipes, directory batches, entropy) under the same per-run budget
    /// as writes.
    pub read_total_bytes: usize,
    /// PLAN-973: high 32 bits of the last clock result (`clock_hi`).
    pub last_clock_hi: u32,
}

impl PlatformResourceLedger {
    pub fn new(session_id: u64, grants: &[String]) -> Self {
        Self {
            session_id,
            next_gen: 1,
            entries: Vec::new(),
            grants: grants.iter().cloned().collect(),
            last_os_error: 0,
            written_total_bytes: 0,
            live_mapped_bytes: 0,
            probe_written_total: 0,
            net_bytes_total: 0,
            read_total_bytes: 0,
            last_clock_hi: 0,
        }
    }

    pub fn grants(&self) -> &BTreeSet<String> {
        &self.grants
    }

    /// PLAN-974: live (unreleased) sockets in this session.
    pub fn active_sockets(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| {
                matches!(
                    e.payload,
                    LedgerPayload::NetSocket {
                        released: false,
                        ..
                    }
                )
            })
            .count()
    }

    pub fn active_regions(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| {
                matches!(
                    e.payload,
                    LedgerPayload::OsMemRegion {
                        released: false,
                        ..
                    }
                )
            })
            .count()
    }

    fn bump_gen(&mut self) -> u16 {
        let g = if self.next_gen == 0 { 1 } else { self.next_gen };
        self.next_gen = self.next_gen.wrapping_add(1).max(1);
        g
    }

    fn encode_handle(idx: usize, gen: u16) -> Result<i32, i32> {
        if idx >= MAX_LEDGER_HANDLES {
            return Err(ERR_BUDGET);
        }
        // Positive i32: bits 0..15 = (idx + 1), bits 16..30 = gen & 0x7FFF.
        let encoded = (((gen as u32) & 0x7FFF) << 16) | ((idx as u32) + 1);
        Ok(encoded as i32)
    }

    fn decode_handle(&self, handle: i32) -> Result<(usize, u16), i32> {
        if handle <= 0 {
            return Err(ERR_INVALID_HANDLE);
        }
        let raw = handle as u32;
        let idx = ((raw & 0xFFFF) as usize)
            .checked_sub(1)
            .ok_or(ERR_INVALID_HANDLE)?;
        let gen = ((raw >> 16) & 0x7FFF) as u16;
        let entry = self.entries.get(idx).ok_or(ERR_INVALID_HANDLE)?;
        if entry.session_id != self.session_id
            || (entry.generation & 0x7FFF) != gen
            || entry.owner != OwnerKind::GuestAv
        {
            return Err(ERR_INVALID_HANDLE);
        }
        Ok((idx, gen))
    }

    pub fn dispatch(
        &mut self,
        backend: &mut dyn PlatformIoBackend,
        spec: &PlatformOpSpec,
        args: &[Value],
        cancel: &CancelFlag,
    ) -> Result<i32, HostError> {
        let _scope = ProviderScope::enter()
            .ok_or_else(|| HostError::Io("host.reentry: reentrant platform host call".into()))?;
        if !self.grants.contains(spec.capability) {
            return Ok(ERR_UNAUTHORIZED);
        }
        if args.len() != spec.params.len() {
            return Err(HostError::Io("platform arity mismatch".into()));
        }
        let code = match spec.kind {
            PlatformOpKind::IoStdout => self.op_std_stream(backend, StreamKind::Stdout),
            PlatformOpKind::IoStderr => self.op_std_stream(backend, StreamKind::Stderr),
            PlatformOpKind::IoWriteSome => {
                let handle = as_i32(&args[0])?;
                let bytes = as_str_bytes(&args[1])?;
                self.op_write_some(backend, handle, bytes, 0, bytes.len(), false, cancel)
            }
            PlatformOpKind::IoWriteSomeSlice => {
                let handle = as_i32(&args[0])?;
                let bytes = as_str_bytes(&args[1])?;
                let offset = as_i32(&args[2])?;
                let len = as_i32(&args[3])?;
                if offset < 0 || len < 0 {
                    ERR_INVALID_BOUNDS
                } else {
                    self.op_write_some(
                        backend,
                        handle,
                        bytes,
                        offset as usize,
                        len as usize,
                        true,
                        cancel,
                    )
                }
            }
            PlatformOpKind::IoLastError => i32::try_from(self.last_os_error).unwrap_or(i32::MAX),
            PlatformOpKind::MemPageSize => {
                let (page, _) = backend.system_info();
                i32::try_from(page).unwrap_or(ERR_OS_FAILED)
            }
            PlatformOpKind::MemAllocGranularity => {
                let (_, gran) = backend.system_info();
                i32::try_from(gran).unwrap_or(ERR_OS_FAILED)
            }
            PlatformOpKind::MemAlloc => self.op_mem_alloc(backend, as_i32(&args[0])?),
            PlatformOpKind::MemFree => self.op_mem_free(backend, as_i32(&args[0])?),
            PlatformOpKind::MemLogicalLen => self.op_mem_len(as_i32(&args[0])?, false),
            PlatformOpKind::MemMappedLen => self.op_mem_len(as_i32(&args[0])?, true),
            PlatformOpKind::MemBorrow => self.op_mem_borrow(as_i32(&args[0])?, true),
            PlatformOpKind::MemUnborrow => self.op_mem_borrow(as_i32(&args[0])?, false),
            PlatformOpKind::MemWriteU8 => {
                self.op_mem_write_u8(as_i32(&args[0])?, as_i32(&args[1])?, as_i32(&args[2])?)
            }
            PlatformOpKind::MemReadU8 => self.op_mem_read_u8(as_i32(&args[0])?, as_i32(&args[1])?),
            PlatformOpKind::ProbeOpenat => self.op_probe_openat(
                backend,
                as_i32(&args[0])?,
                as_str_bytes(&args[1])?,
                as_i32(&args[2])?,
                as_i32(&args[3])?,
            ),
            PlatformOpKind::ProbeWriteFd => {
                self.op_probe_write_fd(backend, as_i32(&args[0])?, as_str_bytes(&args[1])?)
            }
            PlatformOpKind::ProbeClose => self.op_probe_close(backend, as_i32(&args[0])?),
            PlatformOpKind::IoConsoleKind => self.op_console_kind(backend, as_i32(&args[0])?),
            PlatformOpKind::IoConsoleWrite16 => self.op_console_write16(
                backend,
                as_i32(&args[0])?,
                as_i32(&args[1])?,
                as_i32(&args[2])?,
                as_i32(&args[3])?,
                cancel,
            ),
            PlatformOpKind::NetTcpConnect => {
                let host = as_str_bytes(&args[0])?;
                self.op_net_connect(backend, host, as_i32(&args[1])?)
            }
            PlatformOpKind::NetTcpWriteSome => {
                let handle = as_i32(&args[0])?;
                let bytes = as_str_bytes(&args[1])?;
                self.op_net_write_some(backend, handle, bytes, cancel)
            }
            PlatformOpKind::NetTcpReadInto => self.op_net_read_into(
                backend,
                as_i32(&args[0])?,
                as_i32(&args[1])?,
                as_i32(&args[2])?,
                as_i32(&args[3])?,
            ),
            // ---- PLAN-973 system-resource dispatch ------------------------
            PlatformOpKind::IoStdin => self.op_std_stream(backend, StreamKind::Stdin),
            PlatformOpKind::FileOpenW16 => self.op_file_open_w16(
                backend,
                as_i32(&args[0])?,
                as_i32(&args[1])?,
                as_i32(&args[2])?,
                as_i32(&args[3])?,
            ),
            PlatformOpKind::NetTcpClose => self.op_net_close(backend, as_i32(&args[0])?),
            PlatformOpKind::NetTcpWriteRegion => self.op_net_write_region(
                backend,
                as_i32(&args[0])?,
                as_i32(&args[1])?,
                as_i32(&args[2])?,
                as_i32(&args[3])?,
            ),
            PlatformOpKind::FileOpenBytes => {
                self.op_file_open_bytes(backend, as_str_bytes(&args[0])?, as_i32(&args[1])?)
            }
            PlatformOpKind::FileReadSome => self.op_file_read_some(
                backend,
                as_i32(&args[0])?,
                as_i32(&args[1])?,
                as_i32(&args[2])?,
                as_i32(&args[3])?,
                cancel,
            ),
            PlatformOpKind::FileWriteSome => self.op_file_write_some(
                backend,
                as_i32(&args[0])?,
                as_i32(&args[1])?,
                as_i32(&args[2])?,
                as_i32(&args[3])?,
                cancel,
            ),
            PlatformOpKind::FileSeek => self.op_file_seek(
                backend,
                as_i32(&args[0])?,
                as_i32(&args[1])?,
                as_i32(&args[2])?,
                as_i32(&args[3])?,
            ),
            PlatformOpKind::NetTcpListen => self.op_net_listen(backend, as_i32(&args[0])?),
            PlatformOpKind::NetTcpAccept => self.op_net_accept(backend, as_i32(&args[0])?),
            PlatformOpKind::NetTcpLocalPort => self.op_net_local_port(backend, as_i32(&args[0])?),
            PlatformOpKind::FileTell => {
                self.op_file_query(backend, as_i32(&args[0])?, FileQuery::Tell)
            }
            PlatformOpKind::FileHi => self.op_file_hi(as_i32(&args[0])?),
            PlatformOpKind::FileSize => {
                self.op_file_query(backend, as_i32(&args[0])?, FileQuery::Size)
            }
            PlatformOpKind::FileMtimeMs => {
                self.op_file_query(backend, as_i32(&args[0])?, FileQuery::MtimeMs)
            }
            PlatformOpKind::FileFlush => {
                self.op_file_flush_close(backend, as_i32(&args[0])?, false)
            }
            PlatformOpKind::FileClose => self.op_file_flush_close(backend, as_i32(&args[0])?, true),
            PlatformOpKind::PathAttrsW16 => self.op_path_units(
                backend,
                as_i32(&args[0])?,
                as_i32(&args[1])?,
                as_i32(&args[2])?,
                PathUnitOp::Attrs,
            ),
            PlatformOpKind::PathAttrsBytes => {
                self.op_path_bytes(backend, as_str_bytes(&args[0])?, PathBytesOp::Attrs)
            }
            PlatformOpKind::FileDeleteW16 => self.op_path_units(
                backend,
                as_i32(&args[0])?,
                as_i32(&args[1])?,
                as_i32(&args[2])?,
                PathUnitOp::Delete,
            ),
            PlatformOpKind::FileDeleteBytes => {
                self.op_path_bytes(backend, as_str_bytes(&args[0])?, PathBytesOp::Delete)
            }
            PlatformOpKind::DirCreateW16 => self.op_path_units(
                backend,
                as_i32(&args[0])?,
                as_i32(&args[1])?,
                as_i32(&args[2])?,
                PathUnitOp::DirCreate,
            ),
            PlatformOpKind::DirCreateBytes => {
                self.op_path_bytes(backend, as_str_bytes(&args[0])?, PathBytesOp::DirCreate)
            }
            PlatformOpKind::DirDeleteW16 => self.op_path_units(
                backend,
                as_i32(&args[0])?,
                as_i32(&args[1])?,
                as_i32(&args[2])?,
                PathUnitOp::DirDelete,
            ),
            PlatformOpKind::DirDeleteBytes => {
                self.op_path_bytes(backend, as_str_bytes(&args[0])?, PathBytesOp::DirDelete)
            }
            PlatformOpKind::FileReplaceW16 => self.op_file_replace_w16(
                backend,
                as_i32(&args[0])?,
                as_i32(&args[1])?,
                as_i32(&args[2])?,
                as_i32(&args[3])?,
                as_i32(&args[4])?,
                as_i32(&args[5])?,
            ),
            PlatformOpKind::FileReplaceBytes => self.op_file_replace_bytes(
                backend,
                as_str_bytes(&args[0])?,
                as_str_bytes(&args[1])?,
            ),
            PlatformOpKind::CanonicalW16 => self.op_canonical_w16(
                backend,
                as_i32(&args[0])?,
                as_i32(&args[1])?,
                as_i32(&args[2])?,
                as_i32(&args[3])?,
                as_i32(&args[4])?,
            ),
            PlatformOpKind::CanonicalBytes => self.op_canonical_bytes(
                backend,
                as_str_bytes(&args[0])?,
                as_i32(&args[1])?,
                as_i32(&args[2])?,
            ),
            PlatformOpKind::DirOpenW16 => self.op_dir_open_w16(
                backend,
                as_i32(&args[0])?,
                as_i32(&args[1])?,
                as_i32(&args[2])?,
            ),
            PlatformOpKind::DirNextW16 => self.op_dir_next_w16(
                backend,
                as_i32(&args[0])?,
                as_i32(&args[1])?,
                as_i32(&args[2])?,
            ),
            PlatformOpKind::DirClose => self.op_dir_close(backend, as_i32(&args[0])?),
            PlatformOpKind::DirOpenBytes => {
                self.op_dir_open_bytes(backend, as_str_bytes(&args[0])?)
            }
            PlatformOpKind::DirFillBytes => self.op_dir_fill_bytes(
                backend,
                as_i32(&args[0])?,
                as_i32(&args[1])?,
                as_i32(&args[2])?,
            ),
            PlatformOpKind::EnvGetW16 => self.op_env_get_w16(
                backend,
                as_i32(&args[0])?,
                as_i32(&args[1])?,
                as_i32(&args[2])?,
                as_i32(&args[3])?,
                as_i32(&args[4])?,
            ),
            PlatformOpKind::EnvSetW16 => self.op_env_set_w16(
                backend,
                as_i32(&args[0])?,
                as_i32(&args[1])?,
                as_i32(&args[2])?,
                as_i32(&args[3])?,
                as_i32(&args[4])?,
                as_i32(&args[5])?,
            ),
            PlatformOpKind::EnvRemoveW16 => self.op_env_remove_w16(
                backend,
                as_i32(&args[0])?,
                as_i32(&args[1])?,
                as_i32(&args[2])?,
            ),
            PlatformOpKind::EnvGetBytes => self.op_env_get_bytes(
                backend,
                as_i32(&args[0])?,
                as_i32(&args[1])?,
                as_i32(&args[2])?,
            ),
            PlatformOpKind::ClockUnixMs => self.op_clock(backend, false),
            PlatformOpKind::ClockMonoMs => self.op_clock(backend, true),
            PlatformOpKind::ClockHi => self.last_clock_hi as i32,
            PlatformOpKind::SleepMs => self.op_sleep_ms(backend, as_i32(&args[0])?, cancel),
            PlatformOpKind::RandFill => self.op_rand_fill(
                backend,
                as_i32(&args[0])?,
                as_i32(&args[1])?,
                as_i32(&args[2])?,
            ),
            PlatformOpKind::ProcSpawnW16 => self.op_proc_spawn_w16(
                backend,
                as_i32(&args[0])?,
                as_i32(&args[1])?,
                as_i32(&args[2])?,
                as_i32(&args[3])?,
            ),
            PlatformOpKind::ProcSpawnBytes => {
                self.op_proc_spawn_bytes(backend, as_str_bytes(&args[0])?, as_i32(&args[1])?)
            }
            PlatformOpKind::ProcPipeRead => self.op_proc_pipe_read(
                backend,
                as_i32(&args[0])?,
                as_i32(&args[1])?,
                as_i32(&args[2])?,
                as_i32(&args[3])?,
                as_i32(&args[4])?,
                as_i32(&args[5])?,
                cancel,
            ),
            PlatformOpKind::ProcWait => {
                self.op_proc_wait(backend, as_i32(&args[0])?, as_i32(&args[1])?)
            }
            PlatformOpKind::ProcRelease => self.op_proc_release(backend, as_i32(&args[0])?),
            PlatformOpKind::ProcExit => as_i32(&args[0])?, // av.rs turns this into ExecOutcome::Exit
            PlatformOpKind::ProcCmdlineW16 => self.op_out_units(
                backend,
                as_i32(&args[0])?,
                as_i32(&args[1])?,
                OutUnitOp::Cmdline,
            ),
            PlatformOpKind::ProcCmdlineFillBytes => self.op_out_bytes_fill(
                backend,
                as_i32(&args[0])?,
                as_i32(&args[1])?,
                OutBytesOp::Cmdline,
            ),
            PlatformOpKind::ProcCwdW16 => self.op_out_units(
                backend,
                as_i32(&args[0])?,
                as_i32(&args[1])?,
                OutUnitOp::Cwd,
            ),
            PlatformOpKind::ProcCwdBytes => self.op_out_bytes_fill(
                backend,
                as_i32(&args[0])?,
                as_i32(&args[1])?,
                OutBytesOp::Cwd,
            ),
            PlatformOpKind::ProcChdirW16 => self.op_chdir_w16(
                backend,
                as_i32(&args[0])?,
                as_i32(&args[1])?,
                as_i32(&args[2])?,
            ),
            PlatformOpKind::ProcChdirBytes => self.op_chdir_bytes(backend, as_str_bytes(&args[0])?),
        };
        Ok(code)
    }

    /// PLAN-974 inc3: bind a loopback listener. The listener lives in the
    /// backend; `net_tcp_local_port` recalls the actual (ephemeral) port.
    fn op_net_listen(&mut self, backend: &mut dyn PlatformIoBackend, port: i32) -> i32 {
        if port < 0 || port > 65535 {
            return ERR_INVALID_VALUE;
        }
        match backend.net_listen(port) {
            Ok(listener) => {
                let gen = self.bump_gen();
                self.entries.push(LedgerEntry {
                    generation: gen,
                    owner: OwnerKind::GuestAv,
                    session_id: self.session_id,
                    payload: LedgerPayload::NetListener {
                        listener,
                        released: false,
                    },
                });
                let idx = self.entries.len() - 1;
                Self::encode_handle(idx, gen).unwrap_or(ERR_BUDGET)
            }
            Err(err) => {
                self.last_os_error = err;
                ERR_OS_FAILED
            }
        }
    }

    /// PLAN-974 inc3: listener-origin decode (accept/local_port share it).
    fn net_listener_of(&self, handle: i32) -> Result<u64, i32> {
        let (idx, _) = self.decode_handle(handle)?;
        match &self.entries[idx].payload {
            LedgerPayload::NetListener {
                listener,
                released: false,
            } => Ok(*listener),
            LedgerPayload::NetListener { released: true, .. } => Err(ERR_ALREADY_RELEASED),
            _ => Err(ERR_WRONG_ORIGIN),
        }
    }

    /// PLAN-974 inc3: accept ONE connection — the accepted socket is a
    /// FRESH ledger entry (the caller owns and closes it separately).
    fn op_net_accept(&mut self, backend: &mut dyn PlatformIoBackend, handle: i32) -> i32 {
        if self.active_sockets() >= MAX_NET_SOCKETS {
            return ERR_BUDGET;
        }
        let Ok(listener) = self.net_listener_of(handle) else {
            return self.net_listener_of(handle).unwrap_err();
        };
        match backend.net_accept(listener) {
            Ok(socket) => {
                let gen = self.bump_gen();
                self.entries.push(LedgerEntry {
                    generation: gen,
                    owner: OwnerKind::GuestAv,
                    session_id: self.session_id,
                    payload: LedgerPayload::NetSocket {
                        socket,
                        released: false,
                    },
                });
                let idx = self.entries.len() - 1;
                Self::encode_handle(idx, gen).unwrap_or(ERR_BUDGET)
            }
            Err(u32::MAX) => ERR_WOULDB_BLOCK,
            Err(err) => {
                self.last_os_error = err;
                ERR_OS_FAILED
            }
        }
    }

    /// PLAN-974 inc3: the listener's actual bound port (ephemeral recall).
    fn op_net_local_port(&mut self, backend: &mut dyn PlatformIoBackend, handle: i32) -> i32 {
        let Ok(listener) = self.net_listener_of(handle) else {
            return self.net_listener_of(handle).unwrap_err();
        };
        match backend.net_local_port(listener) {
            Ok(port) => port,
            Err(err) => {
                self.last_os_error = err;
                ERR_OS_FAILED
            }
        }
    }

    /// PLAN-974: connect (host, port). The socket itself lives in the
    /// backend; the ledger entry is the single ownership account (the cap
    /// was checked in dispatch; the concurrent-socket budget is here).
    fn op_net_connect(
        &mut self,
        backend: &mut dyn PlatformIoBackend,
        host: &[u8],
        port: i32,
    ) -> i32 {
        if port < 0 || port > 65535 {
            return ERR_INVALID_VALUE;
        }
        if host.is_empty() || host.len() > 256 || host.contains(&0) {
            return ERR_INVALID_VALUE;
        }
        if self.active_sockets() >= MAX_NET_SOCKETS {
            return ERR_BUDGET;
        }
        match backend.net_connect(host, port) {
            Ok(socket) => {
                let gen = self.bump_gen();
                self.entries.push(LedgerEntry {
                    generation: gen,
                    owner: OwnerKind::GuestAv,
                    session_id: self.session_id,
                    payload: LedgerPayload::NetSocket {
                        socket,
                        released: false,
                    },
                });
                let idx = self.entries.len() - 1;
                Self::encode_handle(idx, gen).unwrap_or(ERR_BUDGET)
            }
            Err(err) => {
                self.last_os_error = err;
                ERR_OS_FAILED
            }
        }
    }

    /// PLAN-974: socket-origin decode shared by write/read/close. Returns
    /// the backend socket id or the exact negative code (invalid handle /
    /// already released / wrong origin).
    fn net_socket_of(&self, handle: i32) -> Result<u64, i32> {
        let (idx, _) = self.decode_handle(handle)?;
        match &self.entries[idx].payload {
            LedgerPayload::NetSocket {
                socket,
                released: false,
            } => Ok(*socket),
            LedgerPayload::NetSocket { released: true, .. } => Err(ERR_ALREADY_RELEASED),
            _ => Err(ERR_WRONG_ORIGIN),
        }
    }

    fn op_net_write_some(
        &mut self,
        backend: &mut dyn PlatformIoBackend,
        handle: i32,
        bytes: &[u8],
        cancel: &CancelFlag,
    ) -> i32 {
        if bytes.len() > MAX_PLATFORM_CALL_BYTES {
            return ERR_BUDGET;
        }
        let Ok(socket) = self.net_socket_of(handle) else {
            return self.net_socket_of(handle).unwrap_err();
        };
        if self.net_bytes_total.saturating_add(bytes.len()) > MAX_NET_RUN_BYTES {
            return ERR_BUDGET;
        }
        match backend.net_write_some(socket, bytes, cancel) {
            Ok(written) => {
                self.net_bytes_total += written;
                i32::try_from(written).unwrap_or(i32::MAX)
            }
            Err(err) => {
                self.last_os_error = err;
                ERR_OS_FAILED
            }
        }
    }

    /// PLAN-974: read into an OS memory region (mixed-origin params, the
    /// console_write16 discipline): both handles ledger-validated, byte
    /// bounds checked against the region's logical length, the net run
    /// budget enforced. Returns bytes read (0 = orderly EOF).
    fn op_net_read_into(
        &mut self,
        backend: &mut dyn PlatformIoBackend,
        socket_handle: i32,
        region_handle: i32,
        byte_off: i32,
        byte_len: i32,
    ) -> i32 {
        if byte_off < 0 || byte_len < 0 {
            return ERR_INVALID_BOUNDS;
        }
        let Ok(socket) = self.net_socket_of(socket_handle) else {
            return self.net_socket_of(socket_handle).unwrap_err();
        };
        let Ok((ridx, _)) = self.decode_handle(region_handle) else {
            return ERR_INVALID_HANDLE;
        };
        let (base, logical_len) = match &self.entries[ridx].payload {
            LedgerPayload::OsMemRegion {
                base,
                logical_len,
                released: false,
                ..
            } => (*base, *logical_len),
            LedgerPayload::OsMemRegion { released: true, .. } => return ERR_ALREADY_RELEASED,
            _ => return ERR_WRONG_ORIGIN,
        };
        let (off, len) = (byte_off as usize, byte_len as usize);
        let end = match off.checked_add(len) {
            Some(e) if e <= logical_len => e,
            _ => return ERR_INVALID_BOUNDS,
        };
        if self.net_bytes_total.saturating_add(len) > MAX_NET_RUN_BYTES {
            return ERR_BUDGET;
        }
        // SAFETY: the region stays mapped and exclusively owned by this
        // session for the call; bounds were checked against logical_len.
        let buf = unsafe { std::slice::from_raw_parts_mut(base.add(off), end - off) };
        match backend.net_read_some(socket, buf) {
            Ok(read) => {
                self.net_bytes_total += read;
                i32::try_from(read).unwrap_or(i32::MAX)
            }
            Err(u32::MAX) => ERR_WOULDB_BLOCK,
            Err(err) => {
                self.last_os_error = err;
                ERR_OS_FAILED
            }
        }
    }

    /// PLAN-974 T-08: write raw region bytes (the read_into mirror):
    /// mixed-origin validation + byte bounds + net budget, then the
    /// backend write with the paced cancel discipline.
    fn op_net_write_region(
        &mut self,
        backend: &mut dyn PlatformIoBackend,
        socket_handle: i32,
        region_handle: i32,
        byte_off: i32,
        byte_len: i32,
    ) -> i32 {
        if byte_off < 0 || byte_len < 0 {
            return ERR_INVALID_BOUNDS;
        }
        let Ok(socket) = self.net_socket_of(socket_handle) else {
            return self.net_socket_of(socket_handle).unwrap_err();
        };
        let Ok((ridx, _)) = self.decode_handle(region_handle) else {
            return ERR_INVALID_HANDLE;
        };
        let (base, logical_len) = match &self.entries[ridx].payload {
            LedgerPayload::OsMemRegion {
                base,
                logical_len,
                released: false,
                ..
            } => (*base, *logical_len),
            LedgerPayload::OsMemRegion { released: true, .. } => return ERR_ALREADY_RELEASED,
            _ => return ERR_WRONG_ORIGIN,
        };
        let (off, len) = (byte_off as usize, byte_len as usize);
        let end = match off.checked_add(len) {
            Some(e) if e <= logical_len => e,
            _ => return ERR_INVALID_BOUNDS,
        };
        if self.net_bytes_total.saturating_add(len) > MAX_NET_RUN_BYTES {
            return ERR_BUDGET;
        }
        // SAFETY: region stays mapped and session-owned for the call.
        let buf = unsafe { std::slice::from_raw_parts(base.add(off), end - off) };
        match backend.net_write_some(socket, buf, &CancelFlag::new(false)) {
            Ok(written) => {
                self.net_bytes_total += written;
                i32::try_from(written).unwrap_or(i32::MAX)
            }
            Err(err) => {
                self.last_os_error = err;
                ERR_OS_FAILED
            }
        }
    }

    fn op_net_close(&mut self, backend: &mut dyn PlatformIoBackend, handle: i32) -> i32 {
        let Ok((idx, _)) = self.decode_handle(handle) else {
            return ERR_INVALID_HANDLE;
        };
        let socket = match &self.entries[idx].payload {
            LedgerPayload::NetSocket {
                socket,
                released: false,
            } => *socket,
            LedgerPayload::NetSocket { released: true, .. } => return ERR_ALREADY_RELEASED,
            _ => return ERR_WRONG_ORIGIN,
        };
        match backend.net_close(socket) {
            Ok(()) => {
                if let LedgerPayload::NetSocket { released, .. } = &mut self.entries[idx].payload {
                    *released = true;
                }
                0
            }
            Err(err) => {
                self.last_os_error = err;
                ERR_OS_FAILED
            }
        }
    }

    /// PLAN-1041: classify a ledger std-stream handle. The stream handle
    /// must exist and carry the stream capability; the OS classification
    /// comes from the backend (Win32 `GetConsoleMode`; Linux is always a
    /// byte sink). Permission failures surface as `ERR_OS_FAILED` with the
    /// raw error recorded for `io_last_error`.
    fn op_console_kind(&mut self, backend: &mut dyn PlatformIoBackend, handle: i32) -> i32 {
        let Ok((idx, _)) = self.decode_handle(handle) else {
            return ERR_INVALID_HANDLE;
        };
        let (stream, os_handle) = match &self.entries[idx].payload {
            LedgerPayload::StdStream { stream, os_handle } => (*stream, *os_handle),
            LedgerPayload::OsMemRegion { .. }
            | LedgerPayload::NetSocket { .. }
            | LedgerPayload::NetListener { .. } => return ERR_WRONG_ORIGIN,
            _ => return ERR_WRONG_ORIGIN,
        };
        if !self.grants.contains(stream.capability()) {
            return ERR_UNAUTHORIZED;
        }
        match backend.console_kind(os_handle) {
            // PLAN-1041: a successful classification does not clear the
            // last io error (the freeze: out_last_error reports the last
            // FAILING primitive's raw code).
            Ok(kind) => kind,
            Err(err) => {
                self.last_os_error = err;
                ERR_OS_FAILED
            }
        }
    }

    /// PLAN-1041: write UTF-16LE units from a ledger OS memory region to a
    /// Windows text console. Mixed-origin params (stream handle + region
    /// handle) are both ledger-validated; the byte budget counts 2×units
    /// (the units cross the OS boundary as bytes); progress is UTF-16
    /// units written so the caller keeps unit-level partial state.
    fn op_console_write16(
        &mut self,
        backend: &mut dyn PlatformIoBackend,
        stream_handle: i32,
        region_handle: i32,
        unit_off: i32,
        unit_len: i32,
        cancel: &CancelFlag,
    ) -> i32 {
        if unit_off < 0 || unit_len < 0 {
            return ERR_INVALID_BOUNDS;
        }
        let Ok((sidx, _)) = self.decode_handle(stream_handle) else {
            return ERR_INVALID_HANDLE;
        };
        let (stream, os_handle) = match &self.entries[sidx].payload {
            LedgerPayload::StdStream { stream, os_handle } => (*stream, *os_handle),
            LedgerPayload::OsMemRegion { .. }
            | LedgerPayload::NetSocket { .. }
            | LedgerPayload::NetListener { .. } => return ERR_WRONG_ORIGIN,
            _ => return ERR_WRONG_ORIGIN,
        };
        if !self.grants.contains(stream.capability()) {
            return ERR_UNAUTHORIZED;
        }
        let Ok((ridx, _)) = self.decode_handle(region_handle) else {
            return ERR_INVALID_HANDLE;
        };
        let (base, logical_len) = match &self.entries[ridx].payload {
            LedgerPayload::OsMemRegion {
                base,
                logical_len,
                released: false,
                ..
            } => (*base, *logical_len),
            LedgerPayload::OsMemRegion { released: true, .. } => return ERR_ALREADY_RELEASED,
            LedgerPayload::StdStream { .. }
            | LedgerPayload::NetSocket { .. }
            | LedgerPayload::NetListener { .. } => return ERR_WRONG_ORIGIN,
            _ => return ERR_WRONG_ORIGIN,
        };
        let (unit_off, unit_len) = (unit_off as usize, unit_len as usize);
        let byte_off = match unit_off.checked_mul(2) {
            Some(v) => v,
            None => return ERR_INVALID_BOUNDS,
        };
        let byte_len = match unit_len.checked_mul(2) {
            Some(v) => v,
            None => return ERR_INVALID_BOUNDS,
        };
        let byte_end = match byte_off.checked_add(byte_len) {
            Some(e) if e <= logical_len => e,
            _ => return ERR_INVALID_BOUNDS,
        };
        if byte_len == 0 {
            return 0;
        }
        if byte_len > MAX_PLATFORM_CALL_BYTES {
            return ERR_BUDGET;
        }
        let Some(next_total) = self.written_total_bytes.checked_add(byte_len) else {
            return ERR_BUDGET;
        };
        if next_total > MAX_PLATFORM_RUN_BYTES {
            return ERR_BUDGET;
        }
        // Region memory is page-aligned and 2-byte offsets keep alignment,
        // but read unit-by-unit to stay safe under any future layout.
        let mut units = Vec::with_capacity(unit_len);
        for i in (byte_off..byte_end).step_by(2) {
            let lo = unsafe { core::ptr::read(base.add(i)) };
            let hi = unsafe { core::ptr::read(base.add(i + 1)) };
            units.push(u16::from_le_bytes([lo, hi]));
        }
        match backend.console_write16(stream, os_handle, &units, cancel) {
            Ok(n) => {
                let n = n.min(unit_len);
                self.written_total_bytes += n * 2;
                self.last_os_error = 0;
                n as i32
            }
            Err(CONSOLE_WRITE16_UNSUPPORTED) => ERR_INVALID_VALUE,
            Err(err) => {
                self.last_os_error = err;
                ERR_OS_FAILED
            }
        }
    }

    /// PLAN-1040: probe ops bypass the handle ledger (their fds are raw
    /// kernel numbers the guest owns explicitly) but keep the pre-OS
    /// checks: dirfd must be `AT_FDCWD`, the path bounded, the per-run
    /// probe byte budget enforced, and std fds never closed.
    fn op_probe_openat(
        &mut self,
        backend: &mut dyn PlatformIoBackend,
        dirfd: i32,
        path: &[u8],
        flags: i32,
        mode: i32,
    ) -> i32 {
        if dirfd != PROBE_AT_FDCWD {
            return ERR_INVALID_VALUE;
        }
        if path.is_empty() || path.len() > MAX_PROBE_PATH_BYTES || path.contains(&0) {
            return ERR_INVALID_BOUNDS;
        }
        if flags == 0 || mode < 0 {
            return ERR_INVALID_VALUE;
        }
        match backend.probe_openat(dirfd, path, flags, mode) {
            Ok(raw) => raw,
            Err(err) => {
                self.last_os_error = err;
                ERR_OS_FAILED
            }
        }
    }

    fn op_probe_write_fd(
        &mut self,
        backend: &mut dyn PlatformIoBackend,
        fd: i32,
        bytes: &[u8],
    ) -> i32 {
        if fd <= 0 {
            return ERR_INVALID_VALUE;
        }
        if bytes.len() > MAX_PLATFORM_CALL_BYTES {
            return ERR_BUDGET;
        }
        let Some(next) = self.probe_written_total.checked_add(bytes.len()) else {
            return ERR_BUDGET;
        };
        if next > MAX_PLATFORM_RUN_BYTES {
            return ERR_BUDGET;
        }
        match backend.probe_write_fd(fd, bytes) {
            Ok(raw) => {
                self.probe_written_total = next;
                raw
            }
            Err(err) => {
                self.last_os_error = err;
                ERR_OS_FAILED
            }
        }
    }

    fn op_probe_close(&mut self, backend: &mut dyn PlatformIoBackend, fd: i32) -> i32 {
        if fd <= 2 {
            // never hand the guest its own std streams to close
            return ERR_INVALID_VALUE;
        }
        match backend.probe_close(fd) {
            Ok(raw) => raw,
            Err(err) => {
                self.last_os_error = err;
                ERR_OS_FAILED
            }
        }
    }

    fn op_std_stream(&mut self, backend: &mut dyn PlatformIoBackend, stream: StreamKind) -> i32 {
        if self.entries.len() >= MAX_LEDGER_HANDLES {
            return ERR_BUDGET;
        }
        match backend.std_handle(stream) {
            Ok(os_handle) => {
                let gen = self.bump_gen();
                let idx = self.entries.len();
                self.entries.push(LedgerEntry {
                    generation: gen,
                    owner: OwnerKind::GuestAv,
                    session_id: self.session_id,
                    payload: LedgerPayload::StdStream { stream, os_handle },
                });
                Self::encode_handle(idx, gen).unwrap_or(ERR_BUDGET)
            }
            Err(err) => {
                self.last_os_error = err;
                ERR_OS_FAILED
            }
        }
    }

    fn op_write_some(
        &mut self,
        backend: &mut dyn PlatformIoBackend,
        handle: i32,
        bytes: &[u8],
        offset: usize,
        len: usize,
        slice_mode: bool,
        cancel: &CancelFlag,
    ) -> i32 {
        let Ok((idx, _)) = self.decode_handle(handle) else {
            return ERR_INVALID_HANDLE;
        };
        let (stream, os_handle) = match &self.entries[idx].payload {
            LedgerPayload::StdStream { stream, os_handle } => (*stream, *os_handle),
            LedgerPayload::OsMemRegion { .. }
            | LedgerPayload::NetSocket { .. }
            | LedgerPayload::NetListener { .. } => return ERR_WRONG_ORIGIN,
            _ => return ERR_WRONG_ORIGIN,
        };
        if !self.grants.contains(stream.capability()) {
            return ERR_UNAUTHORIZED;
        }
        let end = match offset.checked_add(len) {
            Some(e) if e <= bytes.len() => e,
            _ => return ERR_INVALID_BOUNDS,
        };
        let target = &bytes[offset..end];
        if target.len() > MAX_PLATFORM_CALL_BYTES {
            return ERR_BUDGET;
        }
        let Some(next_total) = self.written_total_bytes.checked_add(target.len()) else {
            return ERR_BUDGET;
        };
        if next_total > MAX_PLATFORM_RUN_BYTES {
            return ERR_BUDGET;
        }
        if slice_mode && offset > bytes.len() {
            return ERR_INVALID_BOUNDS;
        }
        match backend.write_some(stream, os_handle, target, cancel) {
            Ok(n) => {
                let n = n.min(target.len());
                self.written_total_bytes += n;
                self.last_os_error = 0;
                n as i32
            }
            Err(err) => {
                self.last_os_error = err;
                ERR_OS_FAILED
            }
        }
    }

    fn op_mem_alloc(&mut self, backend: &mut dyn PlatformIoBackend, len: i32) -> i32 {
        if len <= 0 {
            return ERR_INVALID_BOUNDS;
        }
        if self.entries.len() >= MAX_LEDGER_HANDLES {
            return ERR_BUDGET;
        }
        let logical_len = len as usize;
        let (page_size, _) = backend.system_info();
        if page_size == 0 || !page_size.is_power_of_two() {
            return ERR_OS_FAILED;
        }
        let ps = page_size as usize;
        let Some(aligned) = logical_len.checked_add(ps - 1).map(|x| x & !(ps - 1)) else {
            return ERR_INVALID_BOUNDS;
        };
        let Some(next_live) = self.live_mapped_bytes.checked_add(aligned) else {
            return ERR_BUDGET;
        };
        if next_live > MAX_OS_REGION_BYTES {
            return ERR_BUDGET;
        }
        match backend.virtual_alloc(aligned) {
            Ok(base) => {
                self.live_mapped_bytes = next_live;
                self.last_os_error = 0;
                let gen = self.bump_gen();
                let idx = self.entries.len();
                self.entries.push(LedgerEntry {
                    generation: gen,
                    owner: OwnerKind::GuestAv,
                    session_id: self.session_id,
                    payload: LedgerPayload::OsMemRegion {
                        base,
                        logical_len,
                        mapped_len: aligned,
                        borrow_count: 0,
                        released: false,
                    },
                });
                Self::encode_handle(idx, gen).unwrap_or(ERR_BUDGET)
            }
            Err(err) => {
                self.last_os_error = err;
                ERR_OS_FAILED
            }
        }
    }

    fn op_mem_free(&mut self, backend: &mut dyn PlatformIoBackend, handle: i32) -> i32 {
        let Ok((idx, _)) = self.decode_handle(handle) else {
            return ERR_INVALID_HANDLE;
        };
        let (base, mapped_len) = match &self.entries[idx].payload {
            LedgerPayload::StdStream { .. }
            | LedgerPayload::NetSocket { .. }
            | LedgerPayload::NetListener { .. } => return ERR_WRONG_ORIGIN,

            LedgerPayload::OsMemRegion { released: true, .. } => return ERR_ALREADY_RELEASED,
            LedgerPayload::OsMemRegion { borrow_count, .. } if *borrow_count > 0 => {
                return ERR_BORROWED
            }
            LedgerPayload::OsMemRegion {
                base, mapped_len, ..
            } => (*base, *mapped_len),
            _ => return ERR_WRONG_ORIGIN,
        };
        match backend.virtual_free(base, mapped_len) {
            Ok(()) => {
                if let LedgerPayload::OsMemRegion { released, .. } = &mut self.entries[idx].payload
                {
                    *released = true;
                }
                self.live_mapped_bytes = self.live_mapped_bytes.saturating_sub(mapped_len);
                // PLAN-1041: a successful release is not an OS write/query
                // — keep the last io error observable (an error-path free
                // after a failed write must not erase out_last_error()).
                0
            }
            Err(err) => {
                // OS release failed: preserve original owner, generation, and released=false!
                self.last_os_error = err;
                ERR_OS_FAILED
            }
        }
    }

    fn op_mem_len(&self, handle: i32, mapped: bool) -> i32 {
        let Ok((idx, _)) = self.decode_handle(handle) else {
            return ERR_INVALID_HANDLE;
        };
        match &self.entries[idx].payload {
            LedgerPayload::StdStream { .. }
            | LedgerPayload::NetSocket { .. }
            | LedgerPayload::NetListener { .. } => ERR_WRONG_ORIGIN,

            LedgerPayload::OsMemRegion { released: true, .. } => ERR_ALREADY_RELEASED,
            LedgerPayload::OsMemRegion {
                logical_len,
                mapped_len,
                ..
            } => {
                let v = if mapped { *mapped_len } else { *logical_len };
                i32::try_from(v).unwrap_or(ERR_INVALID_BOUNDS)
            }
            _ => ERR_WRONG_ORIGIN,
        }
    }

    fn op_mem_borrow(&mut self, handle: i32, acquire: bool) -> i32 {
        let Ok((idx, _)) = self.decode_handle(handle) else {
            return ERR_INVALID_HANDLE;
        };
        match &mut self.entries[idx].payload {
            LedgerPayload::StdStream { .. }
            | LedgerPayload::NetSocket { .. }
            | LedgerPayload::NetListener { .. } => ERR_WRONG_ORIGIN,

            LedgerPayload::OsMemRegion { released: true, .. } => ERR_ALREADY_RELEASED,
            LedgerPayload::OsMemRegion { borrow_count, .. } => {
                if acquire {
                    let Some(next) = borrow_count.checked_add(1) else {
                        return ERR_BUDGET;
                    };
                    *borrow_count = next;
                    *borrow_count as i32
                } else {
                    let Some(prev) = borrow_count.checked_sub(1) else {
                        return ERR_INVALID_BOUNDS;
                    };
                    *borrow_count = prev;
                    *borrow_count as i32
                }
            }
            _ => ERR_WRONG_ORIGIN,
        }
    }

    fn op_mem_write_u8(&mut self, handle: i32, offset: i32, byte: i32) -> i32 {
        if !(0..=255).contains(&byte) {
            return ERR_INVALID_VALUE;
        }
        let Ok((idx, _)) = self.decode_handle(handle) else {
            return ERR_INVALID_HANDLE;
        };
        match &self.entries[idx].payload {
            LedgerPayload::StdStream { .. }
            | LedgerPayload::NetSocket { .. }
            | LedgerPayload::NetListener { .. } => ERR_WRONG_ORIGIN,

            LedgerPayload::OsMemRegion { released: true, .. } => ERR_ALREADY_RELEASED,
            LedgerPayload::OsMemRegion {
                base, logical_len, ..
            } => {
                if offset < 0 || (offset as usize) >= *logical_len {
                    return ERR_INVALID_BOUNDS;
                }
                unsafe {
                    core::ptr::write(base.add(offset as usize), byte as u8);
                }
                0
            }
            _ => ERR_WRONG_ORIGIN,
        }
    }

    fn op_mem_read_u8(&self, handle: i32, offset: i32) -> i32 {
        let Ok((idx, _)) = self.decode_handle(handle) else {
            return ERR_INVALID_HANDLE;
        };
        match &self.entries[idx].payload {
            LedgerPayload::StdStream { .. }
            | LedgerPayload::NetSocket { .. }
            | LedgerPayload::NetListener { .. } => ERR_WRONG_ORIGIN,

            LedgerPayload::OsMemRegion { released: true, .. } => ERR_ALREADY_RELEASED,
            LedgerPayload::OsMemRegion {
                base, logical_len, ..
            } => {
                if offset < 0 || (offset as usize) >= *logical_len {
                    return ERR_INVALID_BOUNDS;
                }
                let val = unsafe { core::ptr::read(base.add(offset as usize)) };
                val as i32
            }
            _ => ERR_WRONG_ORIGIN,
        }
    }

    // -----------------------------------------------------------------------
    // PLAN-973 system-resource ledger ops
    // -----------------------------------------------------------------------

    /// Map a backend error to the guest sentinel: named-unsupported stays
    /// distinct from a real OS failure (V-DETAIL-02 state separation).
    fn os_fail(&mut self, err: u32) -> i32 {
        if err == PROBE_UNSUPPORTED {
            return ERR_UNSUPPORTED;
        }
        self.last_os_error = err;
        ERR_OS_FAILED
    }

    fn insert_entry(&mut self, payload: LedgerPayload) -> i32 {
        if self.entries.len() >= MAX_LEDGER_HANDLES {
            return ERR_BUDGET;
        }
        let gen = self.bump_gen();
        let idx = self.entries.len();
        self.entries.push(LedgerEntry {
            generation: gen,
            owner: OwnerKind::GuestAv,
            session_id: self.session_id,
            payload,
        });
        Self::encode_handle(idx, gen).unwrap_or(ERR_BUDGET)
    }

    /// Borrow the (base, logical_len) of a live OS memory region.
    fn region_base(&self, handle: i32) -> Result<(*mut u8, usize), i32> {
        let Ok((idx, _)) = self.decode_handle(handle) else {
            return Err(ERR_INVALID_HANDLE);
        };
        match &self.entries[idx].payload {
            LedgerPayload::OsMemRegion {
                base,
                logical_len,
                released: false,
                ..
            } => Ok((*base, *logical_len)),
            LedgerPayload::OsMemRegion { released: true, .. } => Err(ERR_ALREADY_RELEASED),
            _ => Err(ERR_WRONG_ORIGIN),
        }
    }

    /// Read UTF-16LE units out of a region (paths/values staged by the
    /// shared Auto layer). `reject_nul` enforces the no-NUL path rule
    /// BEFORE any OS call (zero side effects on rejection — V-02).
    fn units_from_region(
        &self,
        handle: i32,
        uoff: i32,
        ulen: i32,
        reject_nul: bool,
    ) -> Result<Vec<u16>, i32> {
        if uoff < 0 || ulen < 0 {
            return Err(ERR_INVALID_BOUNDS);
        }
        let (base, logical) = self.region_base(handle)?;
        let byte_off = (uoff as usize) * 2;
        let byte_len = (ulen as usize) * 2;
        let byte_end = match byte_off.checked_add(byte_len) {
            Some(e) if e <= logical => e,
            _ => return Err(ERR_INVALID_BOUNDS),
        };
        if byte_len > MAX_PLATFORM_CALL_BYTES {
            return Err(ERR_BUDGET);
        }
        let mut units = Vec::with_capacity(ulen as usize);
        for i in (byte_off..byte_end).step_by(2) {
            let lo = unsafe { core::ptr::read(base.add(i)) };
            let hi = unsafe { core::ptr::read(base.add(i + 1)) };
            let u = u16::from_le_bytes([lo, hi]);
            if reject_nul && u == 0 {
                return Err(ERR_INVALID_VALUE);
            }
            units.push(u);
        }
        Ok(units)
    }

    fn write_units_to_region(&mut self, handle: i32, uoff: i32, units: &[u16]) -> Result<(), i32> {
        if uoff < 0 {
            return Err(ERR_INVALID_BOUNDS);
        }
        let (base, logical) = self.region_base(handle)?;
        let byte_off = (uoff as usize) * 2;
        let byte_len = units.len() * 2;
        let byte_end = match byte_off.checked_add(byte_len) {
            Some(e) if e <= logical => e,
            _ => return Err(ERR_INVALID_BOUNDS),
        };
        for (k, i) in (byte_off..byte_end).step_by(2).enumerate() {
            let [lo, hi] = units[k].to_le_bytes();
            unsafe {
                core::ptr::write(base.add(i), lo);
                core::ptr::write(base.add(i + 1), hi);
            }
        }
        Ok(())
    }

    fn write_bytes_to_region(&mut self, handle: i32, off: i32, bytes: &[u8]) -> Result<(), i32> {
        if off < 0 {
            return Err(ERR_INVALID_BOUNDS);
        }
        let (base, logical) = self.region_base(handle)?;
        let end = match (off as usize).checked_add(bytes.len()) {
            Some(e) if e <= logical => e,
            _ => return Err(ERR_INVALID_BOUNDS),
        };
        for (k, i) in (off as usize..end).enumerate() {
            unsafe { core::ptr::write(base.add(i), bytes[k]) };
        }
        Ok(())
    }

    fn op_file_open_w16(
        &mut self,
        backend: &mut dyn PlatformIoBackend,
        region: i32,
        uoff: i32,
        ulen: i32,
        flags: i32,
    ) -> i32 {
        let units = match self.units_from_region(region, uoff, ulen, true) {
            Ok(u) => u,
            Err(code) => return code,
        };
        if units.is_empty() || units.len() > MAX_PROBE_PATH_BYTES {
            return ERR_INVALID_BOUNDS;
        }
        match backend.file_open_w16(&units, flags) {
            Ok(os_handle) => {
                self.last_os_error = 0;
                self.insert_entry(LedgerPayload::OsFile {
                    os_handle,
                    last64_hi: 0,
                    closed: false,
                })
            }
            Err(err) => self.os_fail(err),
        }
    }

    fn op_file_open_bytes(
        &mut self,
        backend: &mut dyn PlatformIoBackend,
        path: &[u8],
        flags: i32,
    ) -> i32 {
        if path.is_empty() || path.len() > MAX_PROBE_PATH_BYTES || path.contains(&0) {
            return ERR_INVALID_BOUNDS;
        }
        if flags & (OPEN_READ | OPEN_WRITE | OPEN_APPEND) == 0 {
            return ERR_INVALID_VALUE;
        }
        match backend.file_open_bytes(path, flags) {
            Ok(os_handle) => {
                self.last_os_error = 0;
                self.insert_entry(LedgerPayload::OsFile {
                    os_handle,
                    last64_hi: 0,
                    closed: false,
                })
            }
            Err(err) => self.os_fail(err),
        }
    }

    /// Decode a file-op target: file handles plus (read ops only) stdin.
    fn file_target(&self, handle: i32, for_read: bool) -> Result<(usize, Option<StreamKind>), i32> {
        let Ok((idx, _)) = self.decode_handle(handle) else {
            return Err(ERR_INVALID_HANDLE);
        };
        match &self.entries[idx].payload {
            LedgerPayload::OsFile {
                os_handle,
                closed: false,
                ..
            } => Ok((*os_handle, None)),
            LedgerPayload::OsFile { closed: true, .. } => Err(ERR_ALREADY_RELEASED),
            LedgerPayload::StdStream { stream, os_handle } if for_read => {
                if *stream == StreamKind::Stdin {
                    Ok((*os_handle, Some(*stream)))
                } else {
                    Err(ERR_WRONG_ORIGIN)
                }
            }
            _ => Err(ERR_WRONG_ORIGIN),
        }
    }

    fn op_file_read_some(
        &mut self,
        backend: &mut dyn PlatformIoBackend,
        handle: i32,
        region: i32,
        off: i32,
        len: i32,
        cancel: &CancelFlag,
    ) -> i32 {
        if off < 0 || len < 0 {
            return ERR_INVALID_BOUNDS;
        }
        let len_us = len as usize;
        if len_us > MAX_PLATFORM_CALL_BYTES {
            return ERR_BUDGET;
        }
        let Ok(target) = self.file_target(handle, true) else {
            return ERR_INVALID_HANDLE;
        };
        let (os_handle, stream) = target;
        if let Some(s) = stream {
            if !self.grants.contains(s.capability()) {
                return ERR_UNAUTHORIZED;
            }
        }
        let Ok((base, logical)) = self.region_base(region) else {
            return ERR_WRONG_ORIGIN;
        };
        match (off as usize).checked_add(len_us) {
            Some(e) if e <= logical => {}
            _ => return ERR_INVALID_BOUNDS,
        }
        if len_us == 0 {
            return 0; // a zero-capacity read never implies EOF
        }
        let Some(next_total) = self.read_total_bytes.checked_add(len_us) else {
            return ERR_BUDGET;
        };
        if next_total > MAX_PLATFORM_RUN_BYTES {
            return ERR_BUDGET;
        }
        let slice = unsafe { core::slice::from_raw_parts_mut(base.add(off as usize), len_us) };
        match backend.file_read_some(os_handle, slice, cancel) {
            Ok(n) => {
                let n = n.min(len_us);
                self.read_total_bytes += n;
                self.last_os_error = 0;
                n as i32
            }
            Err(err) => self.os_fail(err),
        }
    }

    fn op_file_write_some(
        &mut self,
        backend: &mut dyn PlatformIoBackend,
        handle: i32,
        region: i32,
        off: i32,
        len: i32,
        cancel: &CancelFlag,
    ) -> i32 {
        if off < 0 || len < 0 {
            return ERR_INVALID_BOUNDS;
        }
        let len_us = len as usize;
        if len_us > MAX_PLATFORM_CALL_BYTES {
            return ERR_BUDGET;
        }
        let Ok(target) = self.file_target(handle, false) else {
            return ERR_WRONG_ORIGIN;
        };
        let (os_handle, _) = target;
        let Ok((base, logical)) = self.region_base(region) else {
            return ERR_WRONG_ORIGIN;
        };
        match (off as usize).checked_add(len_us) {
            Some(e) if e <= logical => {}
            _ => return ERR_INVALID_BOUNDS,
        }
        if len_us == 0 {
            return 0;
        }
        let Some(next_total) = self.written_total_bytes.checked_add(len_us) else {
            return ERR_BUDGET;
        };
        if next_total > MAX_PLATFORM_RUN_BYTES {
            return ERR_BUDGET;
        }
        let slice = unsafe { core::slice::from_raw_parts(base.add(off as usize), len_us) };
        match backend.file_write_some(os_handle, slice, cancel) {
            Ok(n) => {
                let n = n.min(len_us);
                self.written_total_bytes += n;
                self.last_os_error = 0;
                n as i32
            }
            Err(err) => self.os_fail(err),
        }
    }

    fn op_file_seek(
        &mut self,
        backend: &mut dyn PlatformIoBackend,
        handle: i32,
        origin: i32,
        lo: i32,
        hi: i32,
    ) -> i32 {
        if !(0..=2).contains(&origin) {
            return ERR_INVALID_VALUE;
        }
        let Ok(target) = self.file_target(handle, false) else {
            return ERR_WRONG_ORIGIN;
        };
        let (os_handle, _) = target;
        let offset = ((hi as i64) << 32) | (lo as u32 as u64) as i64;
        match backend.file_seek(os_handle, origin, offset) {
            Ok(pos) => {
                if let Ok((idx, _)) = self.decode_handle(handle) {
                    if let LedgerPayload::OsFile { last64_hi, .. } = &mut self.entries[idx].payload
                    {
                        *last64_hi = (pos as u64 >> 32) as u32;
                    }
                }
                self.last_os_error = 0;
                pos as u32 as i32
            }
            Err(err) => self.os_fail(err),
        }
    }

    fn op_file_hi(&self, handle: i32) -> i32 {
        let Ok((idx, _)) = self.decode_handle(handle) else {
            return ERR_INVALID_HANDLE;
        };
        match &self.entries[idx].payload {
            LedgerPayload::OsFile { last64_hi, .. } => *last64_hi as i32,
            _ => ERR_WRONG_ORIGIN,
        }
    }

    fn op_file_query(
        &mut self,
        backend: &mut dyn PlatformIoBackend,
        handle: i32,
        query: FileQuery,
    ) -> i32 {
        let Ok(target) = self.file_target(handle, false) else {
            return ERR_WRONG_ORIGIN;
        };
        let (os_handle, _) = target;
        let result = match query {
            FileQuery::Tell => backend.file_tell(os_handle).map(|v| v as u64),
            FileQuery::Size => backend.file_size(os_handle),
            FileQuery::MtimeMs => backend.file_mtime_ms(os_handle).map(|v| v as u64),
        };
        match result {
            Ok(v) => {
                if let Ok((idx, _)) = self.decode_handle(handle) {
                    if let LedgerPayload::OsFile { last64_hi, .. } = &mut self.entries[idx].payload
                    {
                        *last64_hi = (v >> 32) as u32;
                    }
                }
                self.last_os_error = 0;
                v as u32 as i32
            }
            Err(err) => self.os_fail(err),
        }
    }

    fn op_file_flush_close(
        &mut self,
        backend: &mut dyn PlatformIoBackend,
        handle: i32,
        close: bool,
    ) -> i32 {
        let Ok(target) = self.file_target(handle, false) else {
            return ERR_WRONG_ORIGIN;
        };
        let (os_handle, _) = target;
        let result = if close {
            backend.file_close(os_handle)
        } else {
            backend.file_flush(os_handle)
        };
        match result {
            Ok(()) => {
                if close {
                    if let Ok((idx, _)) = self.decode_handle(handle) {
                        if let LedgerPayload::OsFile { closed, .. } = &mut self.entries[idx].payload
                        {
                            *closed = true;
                        }
                    }
                }
                self.last_os_error = 0;
                0
            }
            Err(err) => self.os_fail(err),
        }
    }

    fn check_path_units(&self, units: &[u16]) -> bool {
        !units.is_empty() && units.len() <= MAX_PROBE_PATH_BYTES
    }

    fn check_path_bytes(&self, path: &[u8]) -> bool {
        !path.is_empty() && path.len() <= MAX_PROBE_PATH_BYTES && !path.contains(&0)
    }

    fn op_path_units(
        &mut self,
        backend: &mut dyn PlatformIoBackend,
        region: i32,
        uoff: i32,
        ulen: i32,
        op: PathUnitOp,
    ) -> i32 {
        let Ok(units) = self.units_from_region(region, uoff, ulen, true) else {
            return ERR_INVALID_BOUNDS;
        };
        if !self.check_path_units(&units) {
            return ERR_INVALID_BOUNDS;
        }
        let result = match op {
            PathUnitOp::Attrs => backend.path_attrs_w16(&units).map(|v| v as u64),
            PathUnitOp::Delete => backend.file_delete_w16(&units).map(|_| 0),
            PathUnitOp::DirCreate => backend.dir_create_w16(&units).map(|_| 0),
            PathUnitOp::DirDelete => backend.dir_delete_w16(&units).map(|_| 0),
        };
        match result {
            Ok(v) => {
                self.last_os_error = 0;
                v as i32
            }
            Err(err) => self.os_fail(err),
        }
    }

    fn op_path_bytes(
        &mut self,
        backend: &mut dyn PlatformIoBackend,
        path: &[u8],
        op: PathBytesOp,
    ) -> i32 {
        if !self.check_path_bytes(path) {
            return ERR_INVALID_BOUNDS;
        }
        let result = match op {
            PathBytesOp::Attrs => backend.path_attrs_bytes(path).map(|v| v as u64),
            PathBytesOp::Delete => backend.file_delete_bytes(path).map(|_| 0),
            PathBytesOp::DirCreate => backend.dir_create_bytes(path).map(|_| 0),
            PathBytesOp::DirDelete => backend.dir_delete_bytes(path).map(|_| 0),
        };
        match result {
            Ok(v) => {
                self.last_os_error = 0;
                v as i32
            }
            Err(err) => self.os_fail(err),
        }
    }

    fn op_file_replace_w16(
        &mut self,
        backend: &mut dyn PlatformIoBackend,
        s_region: i32,
        s_off: i32,
        s_len: i32,
        d_region: i32,
        d_off: i32,
        d_len: i32,
    ) -> i32 {
        let Ok(src) = self.units_from_region(s_region, s_off, s_len, true) else {
            return ERR_INVALID_BOUNDS;
        };
        let Ok(dst) = self.units_from_region(d_region, d_off, d_len, true) else {
            return ERR_INVALID_BOUNDS;
        };
        if !self.check_path_units(&src) || !self.check_path_units(&dst) {
            return ERR_INVALID_BOUNDS;
        }
        match backend.file_replace_w16(&src, &dst) {
            Ok(()) => {
                self.last_os_error = 0;
                0
            }
            Err(err) => self.os_fail(err),
        }
    }

    fn op_file_replace_bytes(
        &mut self,
        backend: &mut dyn PlatformIoBackend,
        src: &[u8],
        dst: &[u8],
    ) -> i32 {
        if !self.check_path_bytes(src) || !self.check_path_bytes(dst) {
            return ERR_INVALID_BOUNDS;
        }
        match backend.file_replace_bytes(src, dst) {
            Ok(()) => {
                self.last_os_error = 0;
                0
            }
            Err(err) => self.os_fail(err),
        }
    }

    fn op_canonical_w16(
        &mut self,
        backend: &mut dyn PlatformIoBackend,
        region: i32,
        uoff: i32,
        ulen: i32,
        out_region: i32,
        out_cap: i32,
    ) -> i32 {
        if out_cap < 0 {
            return ERR_INVALID_BOUNDS;
        }
        let Ok(units) = self.units_from_region(region, uoff, ulen, true) else {
            return ERR_INVALID_BOUNDS;
        };
        if !self.check_path_units(&units) {
            return ERR_INVALID_BOUNDS;
        }
        let mut scratch = vec![0u16; (out_cap as usize).min(MAX_PLATFORM_CALL_BYTES / 2)];
        match backend.canonical_w16(&units, &mut scratch) {
            Ok(n) => match self.write_units_to_region(out_region, 0, &scratch[..n]) {
                Ok(()) => {
                    self.last_os_error = 0;
                    n as i32
                }
                Err(code) => code,
            },
            Err(err) => self.os_fail(err),
        }
    }

    fn op_canonical_bytes(
        &mut self,
        backend: &mut dyn PlatformIoBackend,
        path: &[u8],
        out_region: i32,
        out_cap: i32,
    ) -> i32 {
        if out_cap < 0 {
            return ERR_INVALID_BOUNDS;
        }
        if !self.check_path_bytes(path) {
            return ERR_INVALID_BOUNDS;
        }
        let mut scratch = vec![0u8; (out_cap as usize).min(MAX_PLATFORM_CALL_BYTES)];
        match backend.canonical_bytes(path, &mut scratch) {
            Ok(n) => match self.write_bytes_to_region(out_region, 0, &scratch[..n]) {
                Ok(()) => {
                    self.last_os_error = 0;
                    n as i32
                }
                Err(code) => code,
            },
            Err(err) => self.os_fail(err),
        }
    }

    fn op_dir_open_w16(
        &mut self,
        backend: &mut dyn PlatformIoBackend,
        region: i32,
        uoff: i32,
        ulen: i32,
    ) -> i32 {
        let Ok(units) = self.units_from_region(region, uoff, ulen, true) else {
            return ERR_INVALID_BOUNDS;
        };
        if !self.check_path_units(&units) {
            return ERR_INVALID_BOUNDS;
        }
        match backend.dir_open_w16(&units) {
            Ok((os_handle, pending)) => {
                self.last_os_error = 0;
                self.insert_entry(LedgerPayload::OsDir {
                    os_handle,
                    pending,
                    closed: false,
                })
            }
            Err(err) => self.os_fail(err),
        }
    }

    fn op_dir_next_w16(
        &mut self,
        backend: &mut dyn PlatformIoBackend,
        handle: i32,
        out_region: i32,
        out_cap: i32,
    ) -> i32 {
        if out_cap < 0 {
            return ERR_INVALID_BOUNDS;
        }
        let Ok((idx, _)) = self.decode_handle(handle) else {
            return ERR_INVALID_HANDLE;
        };
        let (os_handle, pending) = match &mut self.entries[idx].payload {
            LedgerPayload::OsDir {
                os_handle,
                pending,
                closed: false,
            } => (*os_handle, pending.take()),
            LedgerPayload::OsDir { closed: true, .. } => return ERR_ALREADY_RELEASED,
            _ => return ERR_WRONG_ORIGIN,
        };
        let name = if let Some(first) = pending {
            first
        } else {
            let mut scratch = vec![0u16; (out_cap as usize).clamp(1, 640)];
            match backend.dir_next_w16(os_handle, &mut scratch) {
                Ok(0) => return 0,
                Ok(n) => scratch.truncate(n),
                Err(err) => return self.os_fail(err),
            }
            scratch
        };
        if name.len() > out_cap as usize {
            return ERR_INVALID_BOUNDS;
        }
        match self.write_units_to_region(out_region, 0, &name) {
            Ok(()) => {
                self.last_os_error = 0;
                name.len() as i32
            }
            Err(code) => code,
        }
    }

    fn op_dir_close(&mut self, backend: &mut dyn PlatformIoBackend, handle: i32) -> i32 {
        let Ok((idx, _)) = self.decode_handle(handle) else {
            return ERR_INVALID_HANDLE;
        };
        let os_handle = match &mut self.entries[idx].payload {
            LedgerPayload::OsDir {
                os_handle,
                closed: false,
                ..
            } => *os_handle,
            LedgerPayload::OsDir { closed: true, .. } => return ERR_ALREADY_RELEASED,
            _ => return ERR_WRONG_ORIGIN,
        };
        match backend.dir_close(os_handle) {
            Ok(()) => {
                if let LedgerPayload::OsDir { closed, .. } = &mut self.entries[idx].payload {
                    *closed = true;
                }
                self.last_os_error = 0;
                0
            }
            Err(err) => self.os_fail(err),
        }
    }

    fn op_dir_open_bytes(&mut self, backend: &mut dyn PlatformIoBackend, path: &[u8]) -> i32 {
        if !self.check_path_bytes(path) {
            return ERR_INVALID_BOUNDS;
        }
        match backend.dir_open_bytes(path) {
            Ok(os_handle) => {
                self.last_os_error = 0;
                self.insert_entry(LedgerPayload::OsDir {
                    os_handle,
                    pending: None,
                    closed: false,
                })
            }
            Err(err) => self.os_fail(err),
        }
    }

    fn op_dir_fill_bytes(
        &mut self,
        backend: &mut dyn PlatformIoBackend,
        handle: i32,
        region: i32,
        cap: i32,
    ) -> i32 {
        if cap < 0 {
            return ERR_INVALID_BOUNDS;
        }
        let cap_us = (cap as usize).min(MAX_PLATFORM_CALL_BYTES);
        let Ok((idx, _)) = self.decode_handle(handle) else {
            return ERR_INVALID_HANDLE;
        };
        let os_handle = match &self.entries[idx].payload {
            LedgerPayload::OsDir {
                os_handle,
                closed: false,
                ..
            } => *os_handle,
            LedgerPayload::OsDir { closed: true, .. } => return ERR_ALREADY_RELEASED,
            _ => return ERR_WRONG_ORIGIN,
        };
        let Ok((base, logical)) = self.region_base(region) else {
            return ERR_WRONG_ORIGIN;
        };
        if cap_us == 0 || cap_us > logical {
            return ERR_INVALID_BOUNDS;
        }
        let Some(next_total) = self.read_total_bytes.checked_add(cap_us) else {
            return ERR_BUDGET;
        };
        if next_total > MAX_PLATFORM_RUN_BYTES {
            return ERR_BUDGET;
        }
        let slice = unsafe { core::slice::from_raw_parts_mut(base, cap_us) };
        match backend.dir_fill_bytes(os_handle, slice) {
            Ok(n) => {
                let n = n.min(cap_us);
                self.read_total_bytes += n;
                self.last_os_error = 0;
                n as i32
            }
            Err(err) => self.os_fail(err),
        }
    }

    fn op_env_get_w16(
        &mut self,
        backend: &mut dyn PlatformIoBackend,
        name_region: i32,
        n_off: i32,
        n_len: i32,
        out_region: i32,
        out_cap: i32,
    ) -> i32 {
        if out_cap < 0 {
            return ERR_INVALID_BOUNDS;
        }
        let Ok(name) = self.units_from_region(name_region, n_off, n_len, true) else {
            return ERR_INVALID_BOUNDS;
        };
        if name.is_empty() {
            return ERR_INVALID_BOUNDS;
        }
        let mut scratch = vec![0u16; (out_cap as usize).clamp(1, 32_768)];
        match backend.env_get_w16(&name, &mut scratch) {
            Ok(0) => {
                self.last_os_error = 0;
                0
            }
            Ok(n) => match self.write_units_to_region(out_region, 0, &scratch[..n]) {
                Ok(()) => {
                    self.last_os_error = 0;
                    n as i32
                }
                Err(code) => code,
            },
            Err(err) => self.os_fail(err),
        }
    }

    fn op_env_set_w16(
        &mut self,
        backend: &mut dyn PlatformIoBackend,
        n_region: i32,
        n_off: i32,
        n_len: i32,
        v_region: i32,
        v_off: i32,
        v_len: i32,
    ) -> i32 {
        let Ok(name) = self.units_from_region(n_region, n_off, n_len, true) else {
            return ERR_INVALID_BOUNDS;
        };
        if name.is_empty() {
            return ERR_INVALID_BOUNDS;
        }
        let Ok(value) = self.units_from_region(v_region, v_off, v_len, true) else {
            return ERR_INVALID_BOUNDS;
        };
        match backend.env_set_w16(&name, &value) {
            Ok(()) => {
                self.last_os_error = 0;
                0
            }
            Err(err) => self.os_fail(err),
        }
    }

    fn op_env_remove_w16(
        &mut self,
        backend: &mut dyn PlatformIoBackend,
        n_region: i32,
        n_off: i32,
        n_len: i32,
    ) -> i32 {
        let Ok(name) = self.units_from_region(n_region, n_off, n_len, true) else {
            return ERR_INVALID_BOUNDS;
        };
        if name.is_empty() {
            return ERR_INVALID_BOUNDS;
        }
        match backend.env_remove_w16(&name) {
            Ok(()) => {
                self.last_os_error = 0;
                0
            }
            Err(err) => self.os_fail(err),
        }
    }

    fn op_env_get_bytes(
        &mut self,
        backend: &mut dyn PlatformIoBackend,
        out_region: i32,
        out_off: i32,
        out_cap: i32,
    ) -> i32 {
        if out_off < 0 || out_cap < 0 {
            return ERR_INVALID_BOUNDS;
        }
        let cap_us = (out_cap as usize).min(MAX_PLATFORM_CALL_BYTES);
        let Ok((base, logical)) = self.region_base(out_region) else {
            return ERR_WRONG_ORIGIN;
        };
        match (out_off as usize).checked_add(cap_us) {
            Some(e) if e <= logical => {}
            _ => return ERR_INVALID_BOUNDS,
        }
        if cap_us == 0 {
            return ERR_INVALID_BOUNDS;
        }
        let Some(next_total) = self.read_total_bytes.checked_add(cap_us) else {
            return ERR_BUDGET;
        };
        if next_total > MAX_PLATFORM_RUN_BYTES {
            return ERR_BUDGET;
        }
        let slice = unsafe { core::slice::from_raw_parts_mut(base.add(out_off as usize), cap_us) };
        match backend.env_get_bytes(slice) {
            Ok(n) => {
                let n = n.min(cap_us);
                self.read_total_bytes += n;
                self.last_os_error = 0;
                n as i32
            }
            Err(err) => self.os_fail(err),
        }
    }

    fn op_clock(&mut self, backend: &mut dyn PlatformIoBackend, mono: bool) -> i32 {
        let result = if mono {
            backend.clock_mono_ms()
        } else {
            backend.clock_unix_ms()
        };
        match result {
            Ok(v) => {
                self.last_clock_hi = (v as u64 >> 32) as u32;
                v as u32 as i32
            }
            Err(err) => self.os_fail(err),
        }
    }

    fn op_sleep_ms(
        &mut self,
        backend: &mut dyn PlatformIoBackend,
        ms: i32,
        cancel: &CancelFlag,
    ) -> i32 {
        if ms < 0 {
            return ERR_INVALID_VALUE;
        }
        match backend.sleep_ms(ms as u32, cancel) {
            Ok(()) => 0,
            Err(err) => self.os_fail(err),
        }
    }

    fn op_rand_fill(
        &mut self,
        backend: &mut dyn PlatformIoBackend,
        region: i32,
        off: i32,
        len: i32,
    ) -> i32 {
        if off < 0 || len < 0 {
            return ERR_INVALID_BOUNDS;
        }
        let len_us = len as usize;
        if len_us == 0 {
            return 0;
        }
        if len_us > MAX_PLATFORM_CALL_BYTES {
            return ERR_BUDGET;
        }
        let Ok((base, logical)) = self.region_base(region) else {
            return ERR_WRONG_ORIGIN;
        };
        match (off as usize).checked_add(len_us) {
            Some(e) if e <= logical => {}
            _ => return ERR_INVALID_BOUNDS,
        }
        let Some(next_total) = self.read_total_bytes.checked_add(len_us) else {
            return ERR_BUDGET;
        };
        if next_total > MAX_PLATFORM_RUN_BYTES {
            return ERR_BUDGET;
        }
        let slice = unsafe { core::slice::from_raw_parts_mut(base.add(off as usize), len_us) };
        match backend.rand_fill(slice) {
            Ok(()) => {
                self.read_total_bytes += len_us;
                self.last_os_error = 0;
                0
            }
            Err(err) => self.os_fail(err),
        }
    }

    fn op_proc_spawn_w16(
        &mut self,
        backend: &mut dyn PlatformIoBackend,
        cmd_region: i32,
        uoff: i32,
        ulen: i32,
        mode: i32,
    ) -> i32 {
        let Ok(units) = self.units_from_region(cmd_region, uoff, ulen, true) else {
            return ERR_INVALID_BOUNDS;
        };
        if units.is_empty() {
            return ERR_INVALID_BOUNDS;
        }
        match backend.proc_spawn_w16(&units, mode & 1 != 0) {
            Ok(spawn) => {
                self.last_os_error = 0;
                self.insert_entry(LedgerPayload::OsProcess {
                    spawn,
                    released: false,
                })
            }
            Err(err) => self.os_fail(err),
        }
    }

    fn op_proc_spawn_bytes(
        &mut self,
        backend: &mut dyn PlatformIoBackend,
        argv_blob: &[u8],
        mode: i32,
    ) -> i32 {
        if argv_blob.is_empty() || argv_blob.len() > MAX_PLATFORM_CALL_BYTES {
            return ERR_INVALID_BOUNDS;
        }
        match backend.proc_spawn_bytes(argv_blob, mode & 1 != 0) {
            Ok(spawn) => {
                self.last_os_error = 0;
                self.insert_entry(LedgerPayload::OsProcess {
                    spawn,
                    released: false,
                })
            }
            Err(err) => self.os_fail(err),
        }
    }

    fn proc_target(&self, handle: i32) -> Result<(usize, usize, usize, usize), i32> {
        let Ok((idx, _)) = self.decode_handle(handle) else {
            return Err(ERR_INVALID_HANDLE);
        };
        match &self.entries[idx].payload {
            LedgerPayload::OsProcess {
                spawn,
                released: false,
            } => Ok((spawn.process, spawn.job, spawn.out_read, spawn.err_read)),
            LedgerPayload::OsProcess { released: true, .. } => Err(ERR_ALREADY_RELEASED),
            _ => Err(ERR_WRONG_ORIGIN),
        }
    }

    /// Read captured child output with a bounded-wait loop (PeekNamedPipe +
    /// WaitForSingleObject slices — never busy polling). `0` = EOF on that
    /// pipe; `-11` = deadline with no data (the child is NOT killed here —
    /// `proc_wait` owns the kill contract). `timeout_ms < 0` waits forever.
    fn op_proc_pipe_read(
        &mut self,
        backend: &mut dyn PlatformIoBackend,
        handle: i32,
        which: i32,
        region: i32,
        off: i32,
        cap: i32,
        timeout_ms: i32,
        cancel: &CancelFlag,
    ) -> i32 {
        if !(1..=2).contains(&which) || off < 0 || cap <= 0 {
            return ERR_INVALID_VALUE;
        }
        let cap_us = (cap as usize).min(MAX_PLATFORM_CALL_BYTES);
        if cap_us == 0 {
            return ERR_INVALID_BOUNDS;
        }
        let Ok((process, _job, out_read, err_read)) = self.proc_target(handle) else {
            return ERR_WRONG_ORIGIN;
        };
        let pipe = if which == 1 { out_read } else { err_read };
        if pipe == 0 {
            return ERR_INVALID_VALUE; // spawned without capture
        }
        let Ok((base, logical)) = self.region_base(region) else {
            return ERR_WRONG_ORIGIN;
        };
        match (off as usize).checked_add(cap_us) {
            Some(e) if e <= logical => {}
            _ => return ERR_INVALID_BOUNDS,
        }
        const SLICE_MS: i64 = 50;
        let deadline: Option<i64> = if timeout_ms < 0 {
            None
        } else {
            Some(timeout_ms as i64)
        };
        let mut waited: i64 = 0;
        loop {
            if CancelCheck::poll(cancel) {
                return self.os_fail(995);
            }
            let avail = match backend.pipe_avail(pipe) {
                Ok(n) => n,
                Err(err) => return self.os_fail(err),
            };
            if avail > 0 {
                let want = cap_us.min(avail);
                let Some(next_total) = self.read_total_bytes.checked_add(want) else {
                    return ERR_BUDGET;
                };
                if next_total > MAX_PLATFORM_RUN_BYTES {
                    return ERR_BUDGET;
                }
                let slice =
                    unsafe { core::slice::from_raw_parts_mut(base.add(off as usize), want) };
                return match backend.file_read_some(pipe, slice, cancel) {
                    Ok(n) => {
                        let n = n.min(want);
                        self.read_total_bytes += n;
                        self.last_os_error = 0;
                        n as i32
                    }
                    Err(err) => self.os_fail(err),
                };
            }
            if let Some(dl) = deadline {
                if waited >= dl {
                    return ERR_TIMEOUT;
                }
            }
            let slice = SLICE_MS.min(deadline.map_or(SLICE_MS, |dl| (dl - waited).max(1)));
            match backend.wait_child(process, slice as i32) {
                Ok(true) => {
                    // child exited: buffered bytes may still drain
                    let avail = match backend.pipe_avail(pipe) {
                        Ok(n) => n,
                        Err(err) => return self.os_fail(err),
                    };
                    if avail == 0 {
                        return 0; // EOF
                    }
                }
                Ok(false) => {
                    waited += slice;
                }
                Err(err) => return self.os_fail(err),
            }
        }
    }

    /// Wait for the child; on timeout kill the whole job tree
    /// (grandchildren included) and settle before reporting `-11`.
    fn op_proc_wait(
        &mut self,
        backend: &mut dyn PlatformIoBackend,
        handle: i32,
        timeout_ms: i32,
    ) -> i32 {
        let Ok((process, job, _out, _err)) = self.proc_target(handle) else {
            return ERR_WRONG_ORIGIN;
        };
        match backend.wait_child(process, timeout_ms) {
            Ok(true) => match backend.child_exit_code(process) {
                Ok(code) => {
                    self.last_os_error = 0;
                    code
                }
                Err(err) => self.os_fail(err),
            },
            Ok(false) => {
                let _ = backend.kill_tree(job);
                let _ = backend.wait_child(process, -1);
                ERR_TIMEOUT
            }
            Err(err) => self.os_fail(err),
        }
    }

    fn op_proc_release(&mut self, backend: &mut dyn PlatformIoBackend, handle: i32) -> i32 {
        let Ok((idx, _)) = self.decode_handle(handle) else {
            return ERR_INVALID_HANDLE;
        };
        let spawn = match &mut self.entries[idx].payload {
            LedgerPayload::OsProcess {
                spawn,
                released: false,
            } => *spawn,
            LedgerPayload::OsProcess { released: true, .. } => return ERR_ALREADY_RELEASED,
            _ => return ERR_WRONG_ORIGIN,
        };
        if let Ok(false) = backend.wait_child(spawn.process, 0) {
            let _ = backend.kill_tree(spawn.job);
            let _ = backend.wait_child(spawn.process, -1);
        }
        if spawn.out_read != 0 {
            let _ = backend.close_raw(spawn.out_read);
        }
        if spawn.err_read != 0 {
            let _ = backend.close_raw(spawn.err_read);
        }
        let _ = backend.close_raw(spawn.job);
        match backend.close_raw(spawn.process) {
            Ok(()) => {
                if let LedgerPayload::OsProcess { released, .. } = &mut self.entries[idx].payload {
                    *released = true;
                }
                self.last_os_error = 0;
                0
            }
            Err(err) => self.os_fail(err),
        }
    }

    fn op_out_units(
        &mut self,
        backend: &mut dyn PlatformIoBackend,
        out_region: i32,
        out_cap: i32,
        op: OutUnitOp,
    ) -> i32 {
        if out_cap < 0 {
            return ERR_INVALID_BOUNDS;
        }
        let mut scratch = vec![0u16; (out_cap as usize).clamp(1, 32_768)];
        let result = match op {
            OutUnitOp::Cmdline => backend.proc_cmdline_w16(&mut scratch).map(|v| v as u64),
            OutUnitOp::Cwd => backend.proc_cwd_w16(&mut scratch).map(|v| v as u64),
        };
        match result {
            Ok(n) => {
                let n = n as usize;
                if n > scratch.len() {
                    return ERR_INVALID_BOUNDS;
                }
                match self.write_units_to_region(out_region, 0, &scratch[..n]) {
                    Ok(()) => {
                        self.last_os_error = 0;
                        n as i32
                    }
                    Err(code) => code,
                }
            }
            Err(err) => self.os_fail(err),
        }
    }

    fn op_out_bytes_fill(
        &mut self,
        backend: &mut dyn PlatformIoBackend,
        out_region: i32,
        out_cap: i32,
        op: OutBytesOp,
    ) -> i32 {
        if out_cap < 0 {
            return ERR_INVALID_BOUNDS;
        }
        let cap_us = (out_cap as usize).min(MAX_PLATFORM_CALL_BYTES);
        if cap_us == 0 {
            return ERR_INVALID_BOUNDS;
        }
        let mut scratch = vec![0u8; cap_us];
        let result = match op {
            OutBytesOp::Cmdline => backend
                .proc_cmdline_fill_bytes(&mut scratch)
                .map(|v| v as u64),
            OutBytesOp::Cwd => backend.proc_cwd_bytes(&mut scratch).map(|v| v as u64),
        };
        match result {
            Ok(n) => {
                let n = (n as usize).min(cap_us);
                match self.write_bytes_to_region(out_region, 0, &scratch[..n]) {
                    Ok(()) => {
                        self.last_os_error = 0;
                        n as i32
                    }
                    Err(code) => code,
                }
            }
            Err(err) => self.os_fail(err),
        }
    }

    fn op_chdir_w16(
        &mut self,
        backend: &mut dyn PlatformIoBackend,
        region: i32,
        uoff: i32,
        ulen: i32,
    ) -> i32 {
        let Ok(units) = self.units_from_region(region, uoff, ulen, true) else {
            return ERR_INVALID_BOUNDS;
        };
        if !self.check_path_units(&units) {
            return ERR_INVALID_BOUNDS;
        }
        match backend.proc_chdir_w16(&units) {
            Ok(()) => {
                self.last_os_error = 0;
                0
            }
            Err(err) => self.os_fail(err),
        }
    }

    fn op_chdir_bytes(&mut self, backend: &mut dyn PlatformIoBackend, path: &[u8]) -> i32 {
        if !self.check_path_bytes(path) {
            return ERR_INVALID_BOUNDS;
        }
        match backend.proc_chdir_bytes(path) {
            Ok(()) => {
                self.last_os_error = 0;
                0
            }
            Err(err) => self.os_fail(err),
        }
    }

    pub fn cleanup(&mut self, backend: &mut dyn PlatformIoBackend) {
        for entry in &mut self.entries {
            match &mut entry.payload {
                LedgerPayload::OsMemRegion {
                    base,
                    mapped_len,
                    released,
                    ..
                } => {
                    if !*released {
                        let _ = backend.virtual_free(*base, *mapped_len);
                        *released = true;
                        self.live_mapped_bytes = self.live_mapped_bytes.saturating_sub(*mapped_len);
                    }
                }
                // PLAN-973: owner-terminated handles are closed exactly
                // once here (never double-closed; close errors swallowed
                // at teardown — they were observable on the explicit path).
                LedgerPayload::OsFile {
                    os_handle, closed, ..
                } => {
                    if !*closed {
                        let _ = backend.file_close(*os_handle);
                        *closed = true;
                    }
                }
                LedgerPayload::OsDir {
                    os_handle, closed, ..
                } => {
                    if !*closed {
                        let _ = backend.dir_close(*os_handle);
                        *closed = true;
                    }
                }
                LedgerPayload::OsProcess { spawn, released } => {
                    if !*released {
                        if let Ok(false) = backend.wait_child(spawn.process, 0) {
                            let _ = backend.kill_tree(spawn.job);
                            let _ = backend.wait_child(spawn.process, -1);
                        }
                        if spawn.out_read != 0 {
                            let _ = backend.close_raw(spawn.out_read);
                        }
                        if spawn.err_read != 0 {
                            let _ = backend.close_raw(spawn.err_read);
                        }
                        let _ = backend.close_raw(spawn.job);
                        let _ = backend.close_raw(spawn.process);
                        *released = true;
                    }
                }
                LedgerPayload::StdStream { .. } => {}
                // PLAN-974 net payloads: release stays explicit
                // (`net_tcp_close`); the master teardown does not
                // force-close sockets/listeners — no-op arms keep that
                // behavior while the 973 exhaustive cleanup stands.
                LedgerPayload::NetSocket { .. } | LedgerPayload::NetListener { .. } => {}
            }
        }
    }
}

impl Drop for PlatformResourceLedger {
    fn drop(&mut self) {
        let mut fallback = Win32PlatformIo::default();
        self.cleanup(&mut fallback);
    }
}

fn as_i32(v: &Value) -> Result<i32, HostError> {
    match v {
        Value::I32(n) => Ok(*n),
        other => Err(HostError::Io(format!(
            "platform op expected i32, got {}",
            other.type_name()
        ))),
    }
}

/// PLAN-974: map a std io error to the raw OS error code (1 = generic).
fn io_err_code(e: std::io::Error) -> u32 {
    e.raw_os_error().map(|c| c as u32).unwrap_or(1)
}

fn as_str_bytes<'a>(v: &'a Value) -> Result<&'a [u8], HostError> {
    match v {
        Value::StrView(sv) => {
            let raw = match &sv.src {
                crate::contract::StrViewSrc::Share(h) => h.bytes(),
                crate::contract::StrViewSrc::Static { buf } => buf.as_ref(),
            };
            crate::owned_text::view_bytes(raw, sv)
                .map_err(|t| HostError::Io(format!("invalid str view: {t:?}")))
        }
        Value::Str(h) => Ok(h.bytes()),
        other => Err(HostError::Io(format!(
            "platform op expected str, got {}",
            other.type_name()
        ))),
    }
}
