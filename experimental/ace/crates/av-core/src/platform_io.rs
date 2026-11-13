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

pub const CAP_STDOUT: &str = "platform.io.stdout";
pub const CAP_STDERR: &str = "platform.io.stderr";
pub const CAP_WRITE: &str = "platform.io.write";
pub const CAP_MEM: &str = "platform.mem";

pub const MAX_PLATFORM_IMPORTS: usize = 16;
pub const MAX_TOTAL_HOST_IMPORTS: usize = 24;
pub const MAX_PLATFORM_CALL_BYTES: usize = 65_536;
pub const MAX_PLATFORM_RUN_BYTES: usize = 1_048_576;
pub const MAX_OS_REGION_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_LEDGER_HANDLES: usize = 1024;

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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StreamKind {
    Stdout,
    Stderr,
}

impl StreamKind {
    pub fn capability(self) -> &'static str {
        match self {
            StreamKind::Stdout => CAP_STDOUT,
            StreamKind::Stderr => CAP_STDERR,
        }
    }

    pub fn win32_std_id(self) -> i32 {
        match self {
            StreamKind::Stdout => -11, // STD_OUTPUT_HANDLE
            StreamKind::Stderr => -12, // STD_ERROR_HANDLE
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

const P_EMPTY: &[MirType] = &[];
const P_I32: &[MirType] = &[MirType::I32];
const P_I32_STR: &[MirType] = &[MirType::I32, MirType::StrView];
const P_I32_STR_2I32: &[MirType] = &[MirType::I32, MirType::StrView, MirType::I32, MirType::I32];
const P_2I32: &[MirType] = &[MirType::I32, MirType::I32];
const P_3I32: &[MirType] = &[MirType::I32, MirType::I32, MirType::I32];

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
    fn virtual_free(&mut self, base: *mut u8) -> Result<(), u32>;
}

#[cfg(windows)]
mod win32 {
    pub const MEM_COMMIT: u32 = 0x1000;
    pub const MEM_RESERVE: u32 = 0x2000;
    pub const MEM_RELEASE: u32 = 0x8000;
    pub const PAGE_READWRITE: u32 = 0x04;
    pub const ERROR_INVALID_HANDLE: u32 = 6;

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
        pub fn GetSystemInfo(lpSystemInfo: *mut SYSTEM_INFO);
        pub fn VirtualAlloc(
            lpAddress: *mut u8,
            dwSize: usize,
            flAllocationType: u32,
            flProtect: u32,
        ) -> *mut u8;
        pub fn VirtualFree(lpAddress: *mut u8, dwSize: usize, dwFreeType: u32) -> i32;
    }

    pub fn query_system_info() -> (u32, u32) {
        let mut info = std::mem::MaybeUninit::<SYSTEM_INFO>::zeroed();
        unsafe {
            GetSystemInfo(info.as_mut_ptr());
            let info = info.assume_init();
            (info.dwPageSize, info.dwAllocationGranularity)
        }
    }
}

/// Real Win32 backend calling `kernel32` directly (zero `std::io::stdout()`, zero `libc`).
#[derive(Debug, Default)]
pub struct Win32PlatformIo {
    pub dispatches: DispatchCounters,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct DispatchCounters {
    pub std_handle: u64,
    pub write_some: u64,
    pub system_info: u64,
    pub virtual_alloc: u64,
    pub virtual_free: u64,
}

impl PlatformIoBackend for Win32PlatformIo {
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
    fn virtual_free(&mut self, base: *mut u8) -> Result<(), u32> {
        self.dispatches.virtual_free += 1;
        let ok = unsafe { win32::VirtualFree(base, 0, win32::MEM_RELEASE) };
        if ok == 0 {
            let err = unsafe { win32::GetLastError() };
            return Err(if err == 0 { 87 } else { err });
        }
        Ok(())
    }

    #[cfg(not(windows))]
    fn virtual_free(&mut self, _base: *mut u8) -> Result<(), u32> {
        self.dispatches.virtual_free += 1;
        Err(50)
    }
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
}

impl PlatformIoBackend for CapturePlatformIo {
    fn std_handle(&mut self, stream: StreamKind) -> Result<usize, u32> {
        if self.use_synthetic_std_handle {
            self.win32.dispatches.std_handle += 1;
            return Ok(match stream {
                StreamKind::Stdout => 0x1001,
                StreamKind::Stderr => 0x1002,
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

    fn virtual_free(&mut self, base: *mut u8) -> Result<(), u32> {
        if let Some(err) = self.inject_free_error {
            self.win32.dispatches.virtual_free += 1;
            return Err(err);
        }
        self.win32.virtual_free(base)
    }
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
        }
    }

    pub fn grants(&self) -> &BTreeSet<String> {
        &self.grants
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
        };
        Ok(code)
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
            LedgerPayload::OsMemRegion { .. } => return ERR_WRONG_ORIGIN,
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
            LedgerPayload::StdStream { .. } => return ERR_WRONG_ORIGIN,
            LedgerPayload::OsMemRegion { released: true, .. } => return ERR_ALREADY_RELEASED,
            LedgerPayload::OsMemRegion { borrow_count, .. } if *borrow_count > 0 => {
                return ERR_BORROWED
            }
            LedgerPayload::OsMemRegion {
                base, mapped_len, ..
            } => (*base, *mapped_len),
        };
        match backend.virtual_free(base) {
            Ok(()) => {
                if let LedgerPayload::OsMemRegion { released, .. } = &mut self.entries[idx].payload
                {
                    *released = true;
                }
                self.live_mapped_bytes = self.live_mapped_bytes.saturating_sub(mapped_len);
                self.last_os_error = 0;
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
            LedgerPayload::StdStream { .. } => ERR_WRONG_ORIGIN,
            LedgerPayload::OsMemRegion { released: true, .. } => ERR_ALREADY_RELEASED,
            LedgerPayload::OsMemRegion {
                logical_len,
                mapped_len,
                ..
            } => {
                let v = if mapped { *mapped_len } else { *logical_len };
                i32::try_from(v).unwrap_or(ERR_INVALID_BOUNDS)
            }
        }
    }

    fn op_mem_borrow(&mut self, handle: i32, acquire: bool) -> i32 {
        let Ok((idx, _)) = self.decode_handle(handle) else {
            return ERR_INVALID_HANDLE;
        };
        match &mut self.entries[idx].payload {
            LedgerPayload::StdStream { .. } => ERR_WRONG_ORIGIN,
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
            LedgerPayload::StdStream { .. } => ERR_WRONG_ORIGIN,
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
        }
    }

    fn op_mem_read_u8(&self, handle: i32, offset: i32) -> i32 {
        let Ok((idx, _)) = self.decode_handle(handle) else {
            return ERR_INVALID_HANDLE;
        };
        match &self.entries[idx].payload {
            LedgerPayload::StdStream { .. } => ERR_WRONG_ORIGIN,
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
        }
    }

    pub fn cleanup(&mut self, backend: &mut dyn PlatformIoBackend) {
        for entry in &mut self.entries {
            if let LedgerPayload::OsMemRegion {
                base,
                mapped_len,
                released,
                ..
            } = &mut entry.payload
            {
                if !*released {
                    let _ = backend.virtual_free(*base);
                    *released = true;
                    self.live_mapped_bytes = self.live_mapped_bytes.saturating_sub(*mapped_len);
                }
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
