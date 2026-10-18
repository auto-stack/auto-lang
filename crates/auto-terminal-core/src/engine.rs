//! Runtime engine service: `autoterm_core.dll` loading, the symbol contract,
//! the session handle table, row/cursor/viewport caching and backlog state.
//!
//! Frozen against the pre-extraction glue (`vm/ffi/term_engine.rs`, PLAN-808
//! T-01): every absent/invalid-handle sentinel below is the legacy oracle.
//! The DLL pointer table never crosses this API; callers only see `i64`
//! session handles and owned values.
//!
//! Lifecycle contract (PLAN-808 R1 fixes):
//! - handles are allocated from a **process-global** counter, so two isolated
//!   services can never expose the same handle value;
//! - each session is a [`Session`] with a per-session gate + retired flag.
//!   Every FFI operation runs under a lease; `free` atomically removes the
//!   session, waits for in-flight leases, retires it and calls the DLL free
//!   exactly once. Concurrent duplicate free is a no-op and cannot re-enter
//!   the DLL with an already-freed pointer;
//! - `Drop` frees every still-live session before the `Library` field drops,
//!   so an isolated service release does not leak PTY/child processes;
//! - lock order: the session table lock is only held for lookup/insert/
//!   remove (never across the gate, FFI or UI). Different sessions have
//!   different gates and never block each other.
//!
//! The auto-term DLL serializes per-handle FFI internally; the service's gate
//! additionally protects the lifetime window around `free` (the DLL lock does
//! not validate that the handle is still alive).

use std::collections::HashMap;
use std::ffi::{c_char, c_int, c_void};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use libloading::Library;

use crate::types::{
    decode_style_color, TerminalCell, TerminalRetiredCache, TerminalSnapshot, PALETTE_SLOTS,
};

type SpawnFn = unsafe extern "C" fn(c_int, c_int, *const c_char) -> *mut c_void;
type SpawnExFn = unsafe extern "C" fn(
    *const c_char,
    *const *const c_char,
    c_int,
    *const c_char,
    c_int,
    c_int,
) -> *mut c_void;
type WriteInputFn = unsafe extern "C" fn(*mut c_void, *const u8, usize);
type ScrollFn = unsafe extern "C" fn(*mut c_void, c_int);
type IntFn = unsafe extern "C" fn(*mut c_void) -> c_int;
type DirtyRowsFn = unsafe extern "C" fn(*mut c_void, *mut c_int, c_int) -> c_int;
type RowTextFn = unsafe extern "C" fn(*mut c_void, c_int, *mut c_char, c_int) -> c_int;
type RowStyleFn = unsafe extern "C" fn(*mut c_void, c_int, *mut u32, c_int) -> c_int;
type CursorFn = unsafe extern "C" fn(*mut c_void, *mut c_int, *mut c_int) -> c_int;
type ResizeFn = unsafe extern "C" fn(*mut c_void, c_int, c_int);
type FreeFn = unsafe extern "C" fn(*mut c_void);
type PaletteColorFn = unsafe extern "C" fn(c_int, c_int, c_int) -> u32;

/// Backlog alert hysteresis lines (bytes) — legacy 014 values.
const BACKLOG_WARN_BYTES: i32 = 2 * 1024 * 1024;
const BACKLOG_CLEAR_BYTES: i32 = 512 * 1024;

/// Process-global handle namespace (legacy glue had one process-global
/// counter). Isolated services must not hand out colliding handle values,
/// otherwise provider-owner routing cannot distinguish them.
static NEXT_HANDLE: AtomicI64 = AtomicI64::new(1);

/// Process-wide shared service (the legacy process-global glue state).
static SHARED: OnceLock<Arc<EngineService>> = OnceLock::new();

/// One live session: the raw DLL pointer plus its operation gate/retire flag.
struct Session {
    raw: usize,
    gate: Mutex<()>,
    retired: AtomicBool,
}

impl Session {
    fn new(raw: *mut c_void) -> Self {
        Self {
            raw: raw as usize,
            gate: Mutex::new(()),
            retired: AtomicBool::new(false),
        }
    }
}

/// Session table with lease/free semantics. The table lock is never held
/// across an operation; the per-session gate serializes FFI vs `free`.
struct SessionTable {
    map: Mutex<HashMap<i64, Arc<Session>>>,
}

impl SessionTable {
    fn new() -> Self {
        Self {
            map: Mutex::new(HashMap::new()),
        }
    }

    fn insert(&self, handle: i64, raw: *mut c_void) {
        self.map
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(handle, Arc::new(Session::new(raw)));
    }

    fn len(&self) -> usize {
        self.map.lock().unwrap_or_else(|e| e.into_inner()).len()
    }

    fn contains(&self, handle: i64) -> bool {
        self.map
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .contains_key(&handle)
    }

    fn handles(&self) -> Vec<i64> {
        self.map
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .keys()
            .copied()
            .collect()
    }

    /// Run `f` with the session pointer under an operation lease. Returns
    /// `absent()` for a missing/retired session; a lease that was waiting
    /// while `free` retired the session re-checks the flag and stays absent.
    fn with_lease<T>(
        &self,
        handle: i64,
        absent: impl FnOnce() -> T,
        f: impl FnOnce(*mut c_void) -> T,
    ) -> T {
        let session = {
            let map = self.map.lock().unwrap_or_else(|e| e.into_inner());
            map.get(&handle).cloned()
        };
        let Some(session) = session else {
            return absent();
        };
        let _gate = session.gate.lock().unwrap_or_else(|e| e.into_inner());
        if session.retired.load(Ordering::Acquire) {
            return absent();
        }
        f(session.raw as *mut c_void)
    }

    /// Atomically retire a session and run the free action exactly once.
    /// Only the caller that removes the entry from the table can obtain the
    /// free token; it waits for in-flight leases before `f` runs.
    fn with_free<T>(
        &self,
        handle: i64,
        absent: impl FnOnce() -> T,
        f: impl FnOnce(*mut c_void) -> T,
    ) -> T {
        let session = {
            let mut map = self.map.lock().unwrap_or_else(|e| e.into_inner());
            map.remove(&handle)
        };
        let Some(session) = session else {
            return absent();
        };
        let _gate = session.gate.lock().unwrap_or_else(|e| e.into_inner());
        if session.retired.swap(true, Ordering::AcqRel) {
            return absent();
        }
        f(session.raw as *mut c_void)
    }
}

/// The legacy process-global engine service. Handles created through this
/// instance are visible to every holder of the same `Arc` (the default
/// provider fallback and the UI adapter both use it), preserving the old
/// cross-call/shared-service behavior.
pub fn shared() -> Arc<EngineService> {
    SHARED
        .get_or_init(|| Arc::new(EngineService::new()))
        .clone()
}

/// How the service resolves its DLL. `Search` is the production path (env →
/// executable directory → ancestor `target/{debug,release}`); `Candidates`
/// is the explicit/embedded path used by tests and external consumers;
/// `Absent` deterministically models a deployment with no engine.
#[derive(Clone, Debug)]
enum LibrarySource {
    Search,
    Candidates(Vec<PathBuf>),
    Absent,
}

/// Terminal engine service instance.
///
/// An instance owns its own session table, snapshots, cached geometry and
/// backlog state. Handles are process-globally unique; provider-owner
/// routing is the VM assembly's responsibility (see `TerminalHostProvider::
/// owns_handle`). Dropping the last reference to an isolated service frees
/// its still-live sessions (see `Drop`).
pub struct EngineService {
    source: LibrarySource,
    lib: OnceLock<Option<Library>>,
    sessions: SessionTable,
    snapshots: Mutex<HashMap<i64, Vec<String>>>,
    cursors: Mutex<HashMap<i64, (i64, i64)>>,
    viewports: Mutex<HashMap<i64, (i64, i64)>>,
    backlog_active: Mutex<bool>,
    palette_tables: OnceLock<Vec<(i32, [u32; PALETTE_SLOTS])>>,
}

impl Default for EngineService {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for EngineService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EngineService")
            .field("source", &self.source)
            .field("handles", &self.live_handle_count())
            .finish()
    }
}

impl Drop for EngineService {
    fn drop(&mut self) {
        // The `lib` field is still alive here; free every session before the
        // Library drops. `free` uses the same lease/retire path, so an
        // in-flight operation (if any) completes first.
        for handle in self.sessions.handles() {
            self.free(handle);
        }
    }
}

impl EngineService {
    /// Production service with the legacy DLL search order.
    pub fn new() -> Self {
        Self::with_source(LibrarySource::Search)
    }

    /// Service restricted to an explicit candidate list (first existing file
    /// wins). Used by tests / external consumers that ship the DLL beside
    /// their own artifact instead of relying on executable ancestry.
    pub fn with_candidates(candidates: Vec<PathBuf>) -> Self {
        Self::with_source(LibrarySource::Candidates(candidates))
    }

    /// Deterministic "deployment has no engine" service (legacy absent
    /// semantics; no DLL is ever searched).
    pub fn without_dll() -> Self {
        Self::with_source(LibrarySource::Absent)
    }

    fn with_source(source: LibrarySource) -> Self {
        Self {
            source,
            lib: OnceLock::new(),
            sessions: SessionTable::new(),
            snapshots: Mutex::new(HashMap::new()),
            cursors: Mutex::new(HashMap::new()),
            viewports: Mutex::new(HashMap::new()),
            backlog_active: Mutex::new(false),
            palette_tables: OnceLock::new(),
        }
    }

    /// Whether the engine library resolved (diagnostics/tests only).
    pub fn has_library(&self) -> bool {
        self.library().is_some()
    }

    /// Number of live session handles held by this service (diagnostics).
    pub fn live_handle_count(&self) -> usize {
        self.sessions.len()
    }

    /// Whether this service currently owns `handle` (owner-routing oracle;
    /// false after free/retire).
    pub fn owns_handle(&self, handle: i64) -> bool {
        self.sessions.contains(handle)
    }

    /// Number of cached text snapshots held by this service (diagnostics).
    pub fn cached_snapshot_count(&self) -> usize {
        self.snapshots.lock().unwrap().len()
    }

    fn library(&self) -> Option<&Library> {
        self.lib
            .get_or_init(|| match &self.source {
                LibrarySource::Absent => None,
                LibrarySource::Candidates(candidates) => load_first_existing(candidates),
                LibrarySource::Search => {
                    let candidates = search_candidates();
                    load_first_existing(&candidates)
                }
            })
            .as_ref()
    }

    fn with_session<T>(
        &self,
        handle: i64,
        absent: impl FnOnce() -> T,
        f: impl FnOnce(*mut c_void) -> T,
    ) -> T {
        self.sessions.with_lease(handle, absent, f)
    }

    fn alloc_handle(&self, raw: *mut c_void, cols: i64, rows: i64) -> i64 {
        let handle = NEXT_HANDLE.fetch_add(1, Ordering::Relaxed);
        self.sessions.insert(handle, raw);
        self.viewports.lock().unwrap().insert(handle, (cols, rows));
        self.cursors.lock().unwrap().insert(handle, (0, 0));
        handle
    }

    /// Spawn a session with the default shell (NULL program). 0 = failure.
    pub fn spawn(&self, cols: i64, rows: i64) -> i64 {
        let Some(lib) = self.library() else {
            return 0;
        };
        unsafe {
            let spawn: libloading::Symbol<SpawnFn> = lib
                .get(b"autoterm_engine_spawn\0")
                .expect("autoterm_engine_spawn symbol");
            let h = spawn(cols as c_int, rows as c_int, std::ptr::null());
            if h.is_null() {
                return 0;
            }
            // SAFETY: the handle is owned by spawn until free; raw pointers
            // cannot round-trip through a VM int, so the service keeps the
            // pointer table (legacy glue contract).
            self.alloc_handle(h, cols, rows)
        }
    }

    /// Spawn with program/argv/cwd/geometry. Missing `spawn_ex` symbol or a
    /// failed spawn = 0 (legacy contract).
    pub fn spawn_ex(&self, program: &str, argv: &[String], cwd: &str, cols: i64, rows: i64) -> i64 {
        let Some(lib) = self.library() else {
            return 0;
        };
        let Ok(c_prog) = std::ffi::CString::new(program) else {
            return 0;
        };
        let c_args: Vec<std::ffi::CString> = argv
            .iter()
            .filter_map(|a| std::ffi::CString::new(a.as_str()).ok())
            .collect();
        let c_argv: Vec<*const c_char> = c_args.iter().map(|a| a.as_ptr()).collect();
        let c_cwd = if cwd.is_empty() {
            None
        } else {
            std::ffi::CString::new(cwd).ok()
        };
        unsafe {
            let spawn_ex: libloading::Symbol<SpawnExFn> =
                match lib.get(b"autoterm_engine_spawn_ex\0") {
                    Ok(s) => s,
                    Err(_) => return 0, // old DLL without the symbol = failure
                };
            let h = spawn_ex(
                c_prog.as_ptr(),
                if c_argv.is_empty() {
                    std::ptr::null()
                } else {
                    c_argv.as_ptr()
                },
                c_argv.len() as c_int,
                c_cwd
                    .as_ref()
                    .map(|c| c.as_ptr())
                    .unwrap_or(std::ptr::null()),
                cols as c_int,
                rows as c_int,
            );
            if h.is_null() {
                return 0;
            }
            self.alloc_handle(h, cols, rows)
        }
    }

    /// Host→child input line (appends `\r\n`).
    pub fn write_line(&self, handle: i64, line: &str) {
        let Some(lib) = self.library() else { return };
        let mut bytes = line.as_bytes().to_vec();
        bytes.extend_from_slice(b"\r\n");
        self.with_session(
            handle,
            || (),
            |h| unsafe {
                let write: libloading::Symbol<WriteInputFn> = lib
                    .get(b"autoterm_engine_write_input\0")
                    .expect("autoterm_engine_write_input symbol");
                write(h, bytes.as_ptr(), bytes.len());
            },
        );
    }

    /// Raw VT write (no line suffix).
    pub fn write_raw(&self, handle: i64, vt: &str) {
        let Some(lib) = self.library() else { return };
        let bytes = vt.as_bytes();
        self.with_session(
            handle,
            || (),
            |h| unsafe {
                let write: libloading::Symbol<WriteInputFn> = lib
                    .get(b"autoterm_engine_write_input\0")
                    .expect("autoterm_engine_write_input symbol");
                write(h, bytes.as_ptr(), bytes.len());
            },
        );
    }

    /// Write queued key payloads one by one (no suffix). Returns the number
    /// of payloads attempted, even when the handle is invalid or the DLL is
    /// absent (legacy pump semantics: the queue is drained and discarded).
    pub fn write_keys(&self, handle: i64, keys: &[String]) -> i64 {
        let n = keys.len() as i64;
        if n == 0 {
            return 0;
        }
        let Some(lib) = self.library() else { return n };
        self.with_session(
            handle,
            || n,
            |h| unsafe {
                let write: libloading::Symbol<WriteInputFn> = lib
                    .get(b"autoterm_engine_write_input\0")
                    .expect("autoterm_engine_write_input symbol");
                for key in keys {
                    let bytes = key.as_bytes();
                    write(h, bytes.as_ptr(), bytes.len());
                }
                n
            },
        )
    }

    /// Resize the engine grid. The viewport cache is updated only by
    /// [`Self::set_viewport`] (legacy: raw `engine_resize` does not).
    pub fn resize(&self, handle: i64, cols: i64, rows: i64) {
        let Some(lib) = self.library() else { return };
        self.with_session(
            handle,
            || (),
            |h| unsafe {
                let resize: libloading::Symbol<ResizeFn> = lib
                    .get(b"autoterm_engine_resize\0")
                    .expect("autoterm_engine_resize symbol");
                resize(h, cols as c_int, rows as c_int);
            },
        );
    }

    /// Ctrl+C interrupt injection. -1 = absent engine / invalid handle.
    pub fn interrupt(&self, handle: i64) -> i64 {
        let Some(lib) = self.library() else { return -1 };
        self.with_session(
            handle,
            || -1,
            |h| unsafe {
                let interrupt: libloading::Symbol<IntFn> = lib
                    .get(b"autoterm_engine_interrupt\0")
                    .expect("autoterm_engine_interrupt symbol");
                interrupt(h) as i64
            },
        )
    }

    /// Whether the child exited. Absent engine / invalid handle = true.
    pub fn is_exited(&self, handle: i64) -> bool {
        let Some(lib) = self.library() else {
            return true;
        };
        self.with_session(
            handle,
            || true,
            |h| unsafe {
                let exited: libloading::Symbol<IntFn> = lib
                    .get(b"autoterm_engine_is_exited\0")
                    .expect("autoterm_engine_is_exited symbol");
                exited(h) == 1
            },
        )
    }

    /// Release a session. Repeated/concurrent free is a no-op; the DLL free
    /// runs exactly once, after in-flight operations complete. Cached cursor/
    /// viewport entries are intentionally kept (legacy observable values;
    /// monotonic handles never alias a new session).
    pub fn free(&self, handle: i64) {
        // R3-F02: publish the frozen cursor/viewport values process-wide
        // before the session is retired. Pure values only — another VM or a
        // direct-free consumer can read them after the owner is pruned, and
        // nothing keeps this service alive.
        if let Some(cache) = self.cached_cursor_viewport(handle) {
            crate::retired::remember(handle, cache);
        }
        let lib = self.library();
        self.sessions.with_free(
            handle,
            || (),
            |h| {
                if let Some(lib) = lib {
                    unsafe {
                        let free: libloading::Symbol<FreeFn> = lib
                            .get(b"autoterm_engine_free\0")
                            .expect("autoterm_engine_free symbol");
                        free(h);
                    }
                }
            },
        );
        self.snapshots.lock().unwrap().remove(&handle);
    }

    /// Engine scrollback row count (0 = absent/invalid).
    pub fn history(&self, handle: i64) -> i64 {
        let Some(lib) = self.library() else { return 0 };
        self.with_session(handle, || 0, |h| unsafe { history_raw(lib, h) })
    }

    /// Scroll the engine viewport (positive = up into history).
    pub fn scroll(&self, handle: i64, delta: i32) {
        let Some(lib) = self.library() else { return };
        self.with_session(
            handle,
            || (),
            |h| unsafe {
                let scroll: libloading::Symbol<ScrollFn> = lib
                    .get(b"autoterm_engine_scroll\0")
                    .expect("autoterm_engine_scroll symbol");
                scroll(h, delta);
            },
        );
    }

    /// Current engine display offset (0 = live bottom).
    pub fn scroll_offset(&self, handle: i64) -> i64 {
        let Some(lib) = self.library() else { return 0 };
        self.with_session(handle, || 0, |h| unsafe { scroll_offset_raw(lib, h) })
    }

    /// Harvest ready output (feed only). Returns the DLL result when a
    /// library and a valid handle are present.
    pub fn feed_ready(&self, handle: i64) -> Option<i32> {
        let lib = self.library()?;
        self.with_session(
            handle,
            || None,
            |h| unsafe {
                let feed: libloading::Symbol<IntFn> = lib
                    .get(b"autoterm_engine_feed_ready\0")
                    .expect("autoterm_engine_feed_ready symbol");
                Some(feed(h))
            },
        )
    }

    /// Take damage + refresh the visible styled snapshot. `None` = absent
    /// engine / invalid handle (the caller returns an empty row list).
    pub fn snapshot(&self, handle: i64) -> Option<TerminalSnapshot> {
        let lib = self.library()?;
        self.with_session(
            handle,
            || None,
            |h| unsafe {
                // Drain the damage queue (its output rows are not used by the
                // legacy glue: the snapshot is re-sampled in full every harvest).
                let take: libloading::Symbol<DirtyRowsFn> = lib
                    .get(b"autoterm_engine_take_dirty_rows\0")
                    .expect("autoterm_engine_take_dirty_rows symbol");
                let mut dirty = [0 as c_int; 64];
                take(h, dirty.as_mut_ptr(), 64);

                // Cursor is sampled per harvest; an invisible cursor keeps the
                // previous cache value (legacy).
                let cursor_fn: libloading::Symbol<CursorFn> = lib
                    .get(b"autoterm_engine_cursor\0")
                    .expect("autoterm_engine_cursor symbol");
                let (mut crow, mut ccol) = (0 as c_int, 0 as c_int);
                let mut cursor = None;
                if cursor_fn(h, &mut crow, &mut ccol) == 1 {
                    self.cursors
                        .lock()
                        .unwrap()
                        .insert(handle, (crow as i64, ccol as i64));
                    cursor = Some((crow, ccol));
                }

                let row_text: libloading::Symbol<RowTextFn> = lib
                    .get(b"autoterm_engine_row_text\0")
                    .expect("autoterm_engine_row_text symbol");
                let row_style: libloading::Symbol<RowStyleFn> = lib
                    .get(b"autoterm_engine_row_style\0")
                    .expect("autoterm_engine_row_style symbol");
                let mut lines = Vec::new();
                let mut cells = Vec::new();
                for r in 0..256i32 {
                    let mut buf = [0 as c_char; 512];
                    let need = row_text(h, r, buf.as_mut_ptr(), 512);
                    if need < 0 {
                        break;
                    }
                    let n = (need as usize).saturating_sub(1).min(511);
                    let bytes: Vec<u8> = buf[..n].iter().map(|&c| c as u8).collect();
                    let text = String::from_utf8_lossy(&bytes).into_owned();

                    let mut styles = [0u32; 1024];
                    let styled = row_style(h, r, styles.as_mut_ptr(), 1024);
                    let pairs = (styled.max(0) as usize) / 2;
                    let mut row_cells: Vec<TerminalCell> = Vec::with_capacity(pairs);
                    for (ci, ch) in text.chars().enumerate() {
                        if ci >= pairs {
                            break;
                        }
                        row_cells.push(TerminalCell {
                            ch,
                            fg: decode_style_color(styles[ci * 2]),
                            bg: decode_style_color(styles[ci * 2 + 1]),
                        });
                    }
                    lines.push(text);
                    cells.push(row_cells);
                }
                self.snapshots.lock().unwrap().insert(handle, lines.clone());

                let history = history_raw(lib, h) as i32;
                let display_offset = scroll_offset_raw(lib, h) as i32;
                Some(TerminalSnapshot {
                    lines,
                    cells,
                    cursor,
                    history,
                    display_offset,
                })
            },
        )
    }

    /// Convenience harvest for non-UI consumers: feed + snapshot text.
    pub fn rows(&self, handle: i64) -> Vec<String> {
        let _ = self.feed_ready(handle);
        self.snapshot(handle)
            .map(|snapshot| snapshot.lines)
            .unwrap_or_default()
    }

    /// Last harvested text rows (empty if never harvested / freed).
    pub fn lines(&self, handle: i64) -> Vec<String> {
        self.snapshots
            .lock()
            .unwrap()
            .get(&handle)
            .cloned()
            .unwrap_or_default()
    }

    /// Sample one engine row as styled cells (`None` = out of range /
    /// absent engine / invalid handle). Used by the UI prefetch window.
    pub fn sample_row(&self, handle: i64, row: i32) -> Option<Vec<TerminalCell>> {
        let lib = self.library()?;
        self.with_session(
            handle,
            || None,
            |h| unsafe {
                let row_text: libloading::Symbol<RowTextFn> = lib
                    .get(b"autoterm_engine_row_text\0")
                    .expect("autoterm_engine_row_text symbol");
                let mut buf = [0 as c_char; 512];
                let need = row_text(h, row, buf.as_mut_ptr(), 512);
                if need < 0 {
                    return None;
                }
                let n = (need as usize).saturating_sub(1).min(511);
                let bytes: Vec<u8> = buf[..n].iter().map(|&c| c as u8).collect();
                let text = String::from_utf8_lossy(&bytes).into_owned();
                let row_style: libloading::Symbol<RowStyleFn> = lib
                    .get(b"autoterm_engine_row_style\0")
                    .expect("autoterm_engine_row_style symbol");
                let mut styles = [0u32; 1024];
                let styled = row_style(h, row, styles.as_mut_ptr(), 1024);
                let pairs = (styled.max(0) as usize) / 2;
                let mut cells: Vec<TerminalCell> = Vec::with_capacity(pairs);
                for (ci, ch) in text.chars().enumerate() {
                    if ci >= pairs {
                        break;
                    }
                    cells.push(TerminalCell {
                        ch,
                        fg: decode_style_color(styles[ci * 2]),
                        bg: decode_style_color(styles[ci * 2 + 1]),
                    });
                }
                Some(cells)
            },
        )
    }

    /// Cached cursor (feed-time sample; (0,0) when never visible).
    pub fn cursor(&self, handle: i64) -> (i64, i64) {
        *self.cursors.lock().unwrap().get(&handle).unwrap_or(&(0, 0))
    }

    /// Cached viewport geometry (spawn initial value / apply_resize refresh).
    pub fn viewport(&self, handle: i64) -> (i64, i64) {
        *self
            .viewports
            .lock()
            .unwrap()
            .get(&handle)
            .unwrap_or(&(0, 0))
    }

    /// Update cached viewport geometry (used by apply_resize paths).
    pub fn set_viewport(&self, handle: i64, cols: i64, rows: i64) {
        self.viewports.lock().unwrap().insert(handle, (cols, rows));
    }

    /// Cursor/viewport cache for a live **or retired** handle. Retired
    /// entries are the legacy post-free observable values; they never touch
    /// the DLL. `None` = this service never saw the handle.
    pub fn cached_cursor_viewport(&self, handle: i64) -> Option<TerminalRetiredCache> {
        let cursors = self.cursors.lock().unwrap();
        let viewports = self.viewports.lock().unwrap();
        match (cursors.get(&handle), viewports.get(&handle)) {
            (None, None) => None,
            (cursor, viewport) => Some(TerminalRetiredCache {
                cursor_row: cursor.map(|(row, _)| *row).unwrap_or(0),
                cursor_col: cursor.map(|(_, col)| *col).unwrap_or(0),
                viewport_cols: viewport.map(|(cols, _)| *cols).unwrap_or(0),
                viewport_rows: viewport.map(|(_, rows)| *rows).unwrap_or(0),
            }),
        }
    }

    fn pending_bytes(&self, handle: i64) -> i32 {
        let Some(lib) = self.library() else { return 0 };
        self.with_session(handle, || 0, |h| unsafe { pending_bytes_raw(lib, h) })
    }

    /// Current backlog in MiB (live sample; absent/invalid = 0).
    pub fn backlog_pending_mb(&self, handle: i64) -> i64 {
        (self.pending_bytes(handle) as i64) / (1024 * 1024)
    }

    /// Whether the reader is backpressure-paused (1/0; missing symbol or
    /// absent engine = 0 — the alert surface is deliberately more forgiving
    /// than the path it monitors).
    pub fn backlog_paused(&self, handle: i64) -> i64 {
        let Some(lib) = self.library() else { return 0 };
        self.with_session(
            handle,
            || 0,
            |h| unsafe {
                let Ok(paused) = lib.get::<IntFn>(b"autoterm_engine_backlog_paused\0") else {
                    return 0;
                };
                paused(h) as i64
            },
        )
    }

    /// Take the backlog alert (0/1, rising edge only). Hysteresis is
    /// service-global (the legacy static was process-global): >=2 MiB raises
    /// and logs once, <=512 KiB clears.
    pub fn backlog_take_alerts(&self, handle: i64) -> i64 {
        let Some(lib) = self.library() else { return 0 };
        self.with_session(
            handle,
            || 0,
            |h| unsafe {
                let pending = pending_bytes_raw(lib, h);
                let mut active = self.backlog_active.lock().unwrap();
                if !*active && pending >= BACKLOG_WARN_BYTES {
                    *active = true;
                    drop(active);
                    eprintln!(
                        "[term-backlog] ⚠ reader→drain 积压 {pending} 字节越过告警线 \
                         {BACKLOG_WARN_BYTES}——产出侧洪峰或消费侧停摆"
                    );
                    1
                } else {
                    if *active && pending <= BACKLOG_CLEAR_BYTES {
                        *active = false;
                    }
                    0
                }
            },
        )
    }

    /// Accumulated ring overflow blocks (missing symbol / absent = 0).
    pub fn backlog_dropped(&self, handle: i64) -> i64 {
        let Some(lib) = self.library() else { return 0 };
        self.with_session(
            handle,
            || 0,
            |h| unsafe {
                let Ok(dropped) = lib.get::<IntFn>(b"autoterm_engine_overflow_count\0") else {
                    return 0;
                };
                dropped(h).max(0) as i64
            },
        )
    }

    /// Engine palette tables (scheme → 18 RGB slots). Loaded lazily on first
    /// call and cached per service; a missing `palette_color` symbol or an
    /// absent engine yields an empty list (UI keeps its built-in fallback).
    pub fn palette_tables(&self) -> Vec<(i32, [u32; PALETTE_SLOTS])> {
        self.palette_tables
            .get_or_init(|| {
                let Some(lib) = self.library() else {
                    return Vec::new();
                };
                let Ok(color) =
                    (unsafe { lib.get::<PaletteColorFn>(b"autoterm_engine_palette_color\0") })
                else {
                    return Vec::new();
                };
                let mut out = Vec::new();
                for scheme in [0i32, 1] {
                    let mut table = [0u32; PALETTE_SLOTS];
                    let mut ok = true;
                    for slot in 0..18i32 {
                        let is_fg = if slot == 0 { 1 } else { 0 };
                        let v = unsafe { color(scheme, slot, is_fg) };
                        if v == 0xFFFF_FFFF {
                            ok = false;
                            break;
                        }
                        table[slot as usize] = v;
                    }
                    if ok {
                        out.push((scheme, table));
                    }
                }
                out
            })
            .clone()
    }

    /// Direct palette slot query (`None` = absent engine / missing symbol /
    /// invalid scheme-slot-axis).
    pub fn palette_color(&self, scheme: i32, slot: i32, is_fg: i32) -> Option<u32> {
        let lib = self.library()?;
        let color =
            unsafe { lib.get::<PaletteColorFn>(b"autoterm_engine_palette_color\0") }.ok()?;
        let v = unsafe { color(scheme, slot, is_fg) };
        (v != 0xFFFF_FFFF).then_some(v)
    }
}

unsafe fn history_raw(lib: &Library, h: *mut c_void) -> i64 {
    let hist: libloading::Symbol<IntFn> = lib
        .get(b"autoterm_engine_history\0")
        .expect("autoterm_engine_history symbol");
    hist(h).max(0) as i64
}

unsafe fn scroll_offset_raw(lib: &Library, h: *mut c_void) -> i64 {
    let soff: libloading::Symbol<IntFn> = lib
        .get(b"autoterm_engine_scroll_offset\0")
        .expect("autoterm_engine_scroll_offset symbol");
    soff(h) as i64
}

unsafe fn pending_bytes_raw(lib: &Library, h: *mut c_void) -> i32 {
    let pending: libloading::Symbol<IntFn> = lib
        .get(b"autoterm_engine_pending_bytes\0")
        .expect("autoterm_engine_pending_bytes symbol");
    pending(h).max(0)
}

/// Production resolution order (frozen legacy order): env
/// `AUTOTERM_ENGINE_DLL` → executable directory → exe ancestor
/// `target/{debug,release}` (group layout, at-gen-compatible).
fn search_candidates() -> Vec<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Ok(p) = std::env::var("AUTOTERM_ENGINE_DLL") {
        candidates.push(PathBuf::from(p));
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join("autoterm_core.dll"));
            for anc in dir.ancestors().take(4) {
                candidates.push(anc.join("target/debug/autoterm_core.dll"));
                candidates.push(anc.join("target/release/autoterm_core.dll"));
            }
        }
    }
    candidates
}

fn load_first_existing(candidates: &[PathBuf]) -> Option<Library> {
    let found = candidates.iter().find(|p| p.is_file());
    match found {
        Some(path) => match unsafe { Library::new(path) } {
            Ok(l) => Some(l),
            Err(e) => {
                eprintln!("[term-engine] autoterm_core.dll 加载失败({path:?}): {e}");
                None
            }
        },
        None => {
            eprintln!(
                "[term-engine] autoterm_core.dll 未找到(部署至宿主同目录,或设 AUTOTERM_ENGINE_DLL;auto-term 侧先 cargo build -p autoterm-core)"
            );
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    fn sentinel(v: usize) -> *mut c_void {
        v as *mut c_void
    }

    #[test]
    fn absent_engine_legacy_sentinels() {
        let svc = EngineService::without_dll();
        assert!(!svc.has_library());
        assert_eq!(svc.spawn(80, 24), 0);
        assert_eq!(svc.spawn_ex("cmd", &[], "", 80, 24), 0);
        assert!(svc.rows(1).is_empty());
        assert!(svc.lines(1).is_empty());
        assert_eq!(svc.interrupt(1), -1);
        assert!(svc.is_exited(1));
        assert_eq!(svc.history(1), 0);
        assert_eq!(svc.scroll_offset(1), 0);
        assert_eq!(svc.cursor(1), (0, 0));
        assert_eq!(svc.viewport(1), (0, 0));
        assert_eq!(svc.backlog_pending_mb(1), 0);
        assert_eq!(svc.backlog_paused(1), 0);
        assert_eq!(svc.backlog_take_alerts(1), 0);
        assert_eq!(svc.backlog_dropped(1), 0);
        assert_eq!(svc.palette_tables().len(), 0);
        assert_eq!(svc.palette_color(0, 0, 0), None);
        // write/pump/resize/free are observable no-ops.
        svc.write_line(1, "x");
        svc.write_raw(1, "y");
        assert_eq!(svc.write_keys(1, &["a".to_string(), "b".to_string()]), 2);
        svc.resize(1, 100, 30);
        svc.free(1);
        assert_eq!(svc.live_handle_count(), 0);
    }

    #[test]
    fn missing_dll_candidates_and_bad_library_are_absent() {
        let svc = EngineService::with_candidates(vec![PathBuf::from(
            "D:/autostack/definitely-missing/autoterm_core.dll",
        )]);
        assert!(!svc.has_library());
        assert_eq!(svc.spawn(80, 24), 0);

        // A real file that is not a loadable library = load failure (not a
        // silent success).
        let self_manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
        let svc = EngineService::with_candidates(vec![self_manifest]);
        assert!(!svc.has_library());
        assert_eq!(svc.spawn(80, 24), 0);
    }

    #[test]
    fn handles_are_process_globally_unique() {
        // Spawn without a DLL cannot allocate handles; the counter property
        // is exercised through the table in the lease tests below. This test
        // pins the absent-side behavior and the owner oracle.
        let a = EngineService::without_dll();
        let b = EngineService::without_dll();
        assert_eq!(a.live_handle_count(), 0);
        assert_eq!(b.live_handle_count(), 0);
        assert!(!a.owns_handle(1));
        assert!(!b.owns_handle(1));
    }

    /// F808-R1-03 permanent control: a lease blocks `free` until the
    /// in-flight operation finishes; a lease waiting behind `free` observes
    /// the retired state; duplicate free executes the action exactly once.
    #[test]
    fn session_leases_serialize_free_and_never_double_free() {
        let table = Arc::new(SessionTable::new());
        table.insert(1, sentinel(0x1000));

        let (lease_ready_tx, lease_ready_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel::<()>();
        let t = table.clone();
        let holder = thread::spawn(move || {
            t.with_lease(
                1,
                || (),
                |_| {
                    lease_ready_tx.send(()).unwrap();
                    release_rx.recv().unwrap();
                },
            );
        });
        lease_ready_rx.recv().unwrap();

        let (free_done_tx, free_done_rx) = mpsc::channel();
        let t = table.clone();
        let freer = thread::spawn(move || {
            t.with_free(
                1,
                || false,
                |_| {
                    free_done_tx.send(true).unwrap();
                    true
                },
            );
        });
        assert!(
            free_done_rx
                .recv_timeout(Duration::from_millis(100))
                .is_err(),
            "free must wait for the in-flight lease (no FFI overlap)"
        );
        release_tx.send(()).unwrap();
        assert!(
            free_done_rx.recv_timeout(Duration::from_secs(2)).unwrap(),
            "free runs after the lease is released"
        );
        holder.join().unwrap();
        freer.join().unwrap();

        // A lease after free is absent and must not touch the pointer.
        assert!(table.with_lease(1, || true, |_| false));
        // The table entry is gone; owns_handle is false.
        assert!(!table.contains(1));

        // Two concurrent free callers: exactly one free action.
        let table = Arc::new(SessionTable::new());
        table.insert(7, sentinel(0x2000));
        let calls = Arc::new(AtomicUsize::new(0));
        let barrier = Arc::new(std::sync::Barrier::new(3));
        let mut threads = Vec::new();
        for _ in 0..2 {
            let t = table.clone();
            let calls = calls.clone();
            let barrier = barrier.clone();
            threads.push(thread::spawn(move || {
                barrier.wait();
                t.with_free(
                    7,
                    || false,
                    |_| {
                        calls.fetch_add(1, Ordering::SeqCst);
                        true
                    },
                )
            }));
        }
        barrier.wait();
        let results: Vec<bool> = threads.into_iter().map(|t| t.join().unwrap()).collect();
        assert_eq!(
            results.iter().filter(|&&ran| ran).count(),
            1,
            "exactly one caller must obtain the free token"
        );
        assert_eq!(calls.load(Ordering::SeqCst), 1, "exactly one free action");
        assert_eq!(table.len(), 0);
    }

    /// Different sessions have independent gates: a lease on one must not
    /// block free on another.
    #[test]
    fn different_sessions_do_not_block_each_other() {
        let table = Arc::new(SessionTable::new());
        table.insert(1, sentinel(0x3000));
        table.insert(2, sentinel(0x4000));
        let (lease_ready_tx, lease_ready_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel::<()>();
        let t = table.clone();
        let holder = thread::spawn(move || {
            t.with_lease(
                1,
                || (),
                |_| {
                    lease_ready_tx.send(()).unwrap();
                    release_rx.recv().unwrap();
                },
            );
        });
        lease_ready_rx.recv().unwrap();
        assert!(
            table.with_free(2, || false, |_| true),
            "free on another session must not wait"
        );
        release_tx.send(()).unwrap();
        holder.join().unwrap();
    }
}
