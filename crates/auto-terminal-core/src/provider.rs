//! `TerminalHostProvider` — the light terminal capability port (PLAN-808
//! SD-01). One method per terminal native (34), owned value domain only:
//! no UI registry references, raw DLL pointers, VM stack values or closures.
//!
//! The VM side only pops arguments, calls the port and pushes results. The
//! production composition is:
//! - instance provider (per VM) if set;
//! - process default provider (UI host installs an adapter; explicit
//!   registration wins over the compatibility install);
//! - [`default_provider`] = the process-shared [`EngineService`], so an
//!   engine remains available without any UI host (legacy no-UI behavior).

use std::sync::Arc;

use crate::engine::{shared, EngineService};
use crate::types::TerminalRetiredCache;

/// Terminal capability port (34 methods ↔ 34 `auto.term.*` natives).
///
/// Implementations must be `Send + Sync` and must not call back into the
/// caller while holding implementation locks. UI implementations compose the
/// core service with the UI view/input registry; non-UI implementations (the
/// default) provide engine-only semantics.
pub trait TerminalHostProvider: Send + Sync {
    /// 2943: spawn with the default shell. 0 = failure / absent engine.
    fn spawn(&self, cols: i64, rows: i64) -> i32;
    /// 2983: spawn with program/argv/cwd. 0 = failure / missing symbol.
    fn spawn_ex(&self, program: &str, argv: &[String], cwd: &str, cols: i64, rows: i64) -> i32;
    /// 2944: line input (implementation appends the line suffix).
    fn write_line(&self, handle: i64, line: &str);
    /// 3000: raw VT input (no suffix).
    fn write_raw(&self, handle: i64, vt: &str);
    /// 2945: harvest + viewport text rows (broadcast view sideband for UI).
    fn rows(&self, handle: i64) -> Vec<String>;
    /// 2984: harvest + viewport text rows for one view key.
    fn rows_for(&self, handle: i64, key: &str) -> Vec<String>;
    /// 2946: engine grid resize (does not refresh the viewport cache).
    fn resize(&self, handle: i64, cols: i64, rows: i64);
    /// 2976: consume any pending view geometry and resize. 0 = none.
    fn apply_resize(&self, handle: i64) -> i32;
    /// 2986: consume the pending geometry for one view key. 0 = none.
    fn apply_resize_for(&self, handle: i64, key: &str) -> i32;
    /// 2957: drain every queued keystroke into this session. Returns the
    /// number of payloads (drained even when the handle is invalid).
    fn pump_input(&self, handle: i64) -> i32;
    /// 2985: drain one view key's queued keystrokes.
    fn pump_for(&self, handle: i64, key: &str) -> i32;
    /// 3001: pending scroll delta for a view key (peek, do not drain).
    fn scroll_pending_for(&self, handle: i64, key: &str) -> i32;
    /// 3002: parse + queue a `"colsxrows"` geometry request. 1 = queued.
    fn pend_resize_geom(&self, key: &str, geom: &str) -> i32;
    /// 2987: take one menu action payload; -1 = none.
    fn menu_take(&self) -> i32;
    /// 2947: interrupt; -1 = absent engine / invalid handle.
    fn interrupt(&self, handle: i64) -> i32;
    /// 2948: child exited; absent engine / invalid handle = true.
    fn is_exited(&self, handle: i64) -> bool;
    /// 2949: release the session (repeated free = no-op).
    fn free(&self, handle: i64);
    /// 2958: cached cursor row.
    fn cursor_row(&self, handle: i64) -> i32;
    /// 2959: cached cursor column.
    fn cursor_col(&self, handle: i64) -> i32;
    /// 2977: cached viewport columns.
    fn viewport_cols(&self, handle: i64) -> i32;
    /// 2978: cached viewport rows.
    fn viewport_rows(&self, handle: i64) -> i32;
    /// 2979: backlog MiB.
    fn backlog_pending_mb(&self, handle: i64) -> i32;
    /// 2980: reader paused (1/0).
    fn backlog_paused(&self, handle: i64) -> i32;
    /// 2981: backlog alert rising edge (0/1).
    fn backlog_take_alerts(&self, handle: i64) -> i32;
    /// 2982: accumulated overflow blocks.
    fn backlog_dropped(&self, handle: i64) -> i32;
    /// 3003: engine scrollback rows.
    fn history(&self, handle: i64) -> i32;
    /// 2998: host window width in logical px.
    fn window_width(&self) -> i32;
    /// 2999: host window height in logical px.
    fn window_height(&self) -> i32;
    /// 2988: configured profile names (intentionally empty).
    fn config_profiles(&self) -> Vec<String>;
    /// 2989: default profile (intentionally empty).
    fn config_default_profile(&self) -> String;
    /// 2990: full profile projection (intentionally empty).
    fn config_profiles_full(&self) -> Vec<String>;
    /// 2991: profile spawn program (intentionally empty).
    fn config_spawn_program(&self, profile: &str) -> String;
    /// 2992: profile spawn argv (intentionally empty).
    fn config_spawn_argv(&self, profile: &str) -> Vec<String>;
    /// 2993: profile spawn cwd (intentionally empty).
    fn config_spawn_cwd(&self, _profile: &str) -> String;

    // ── lifecycle helpers (not natives; no VM stack shape) ────────────────
    //
    // The VM keeps a per-VM history of terminal providers and asks each one
    // whether it owns a handle before routing reads/writes/free. This is what
    // keeps an old handle bound to its original service across
    // `set_terminal_provider` replacement (F808-R1-01). Implementations that
    // cannot report ownership keep the default `false` (the current provider
    // is used), which is correct for stateless fakes.
    //
    /// Whether this provider currently owns `handle` (a live session).
    fn owns_handle(&self, _handle: i64) -> bool {
        false
    }

    /// Number of live sessions owned by this provider. The VM uses it to
    /// prune replaced owners with no remaining sessions (so a replaced
    /// isolated service can Drop and free its sessions, F808-R1-04).
    fn live_handle_count(&self) -> usize {
        0
    }

    /// Cached cursor/viewport for a live **or retired** handle. The cache
    /// shims fall back to this after free so the legacy post-free values
    /// survive provider replacement/owner pruning (R2-F02). `None` = this
    /// provider never saw the handle.
    fn cached_cursor_viewport(&self, _handle: i64) -> Option<TerminalRetiredCache> {
        None
    }
}

impl TerminalHostProvider for EngineService {
    fn spawn(&self, cols: i64, rows: i64) -> i32 {
        EngineService::spawn(self, cols, rows) as i32
    }

    fn spawn_ex(&self, program: &str, argv: &[String], cwd: &str, cols: i64, rows: i64) -> i32 {
        EngineService::spawn_ex(self, program, argv, cwd, cols, rows) as i32
    }

    fn write_line(&self, handle: i64, line: &str) {
        EngineService::write_line(self, handle, line);
    }

    fn write_raw(&self, handle: i64, vt: &str) {
        EngineService::write_raw(self, handle, vt);
    }

    fn rows(&self, handle: i64) -> Vec<String> {
        EngineService::rows(self, handle)
    }

    fn rows_for(&self, handle: i64, _key: &str) -> Vec<String> {
        // Non-UI provider: there is no view sideband to direct.
        EngineService::rows(self, handle)
    }

    fn resize(&self, handle: i64, cols: i64, rows: i64) {
        EngineService::resize(self, handle, cols, rows);
    }

    fn apply_resize(&self, _handle: i64) -> i32 {
        0
    }

    fn apply_resize_for(&self, _handle: i64, _key: &str) -> i32 {
        0
    }

    fn pump_input(&self, _handle: i64) -> i32 {
        0
    }

    fn pump_for(&self, _handle: i64, _key: &str) -> i32 {
        0
    }

    fn scroll_pending_for(&self, _handle: i64, _key: &str) -> i32 {
        0
    }

    fn pend_resize_geom(&self, _key: &str, _geom: &str) -> i32 {
        0
    }

    fn menu_take(&self) -> i32 {
        -1
    }

    fn interrupt(&self, handle: i64) -> i32 {
        EngineService::interrupt(self, handle) as i32
    }

    fn is_exited(&self, handle: i64) -> bool {
        EngineService::is_exited(self, handle)
    }

    fn free(&self, handle: i64) {
        EngineService::free(self, handle);
    }

    fn cursor_row(&self, handle: i64) -> i32 {
        EngineService::cursor(self, handle).0 as i32
    }

    fn cursor_col(&self, handle: i64) -> i32 {
        EngineService::cursor(self, handle).1 as i32
    }

    fn viewport_cols(&self, handle: i64) -> i32 {
        EngineService::viewport(self, handle).0 as i32
    }

    fn viewport_rows(&self, handle: i64) -> i32 {
        EngineService::viewport(self, handle).1 as i32
    }

    fn backlog_pending_mb(&self, handle: i64) -> i32 {
        EngineService::backlog_pending_mb(self, handle) as i32
    }

    fn backlog_paused(&self, handle: i64) -> i32 {
        EngineService::backlog_paused(self, handle) as i32
    }

    fn backlog_take_alerts(&self, handle: i64) -> i32 {
        EngineService::backlog_take_alerts(self, handle) as i32
    }

    fn backlog_dropped(&self, handle: i64) -> i32 {
        EngineService::backlog_dropped(self, handle) as i32
    }

    fn history(&self, handle: i64) -> i32 {
        EngineService::history(self, handle) as i32
    }

    fn window_width(&self) -> i32 {
        1024
    }

    fn window_height(&self) -> i32 {
        768
    }

    fn config_profiles(&self) -> Vec<String> {
        Vec::new()
    }

    fn config_default_profile(&self) -> String {
        String::new()
    }

    fn config_profiles_full(&self) -> Vec<String> {
        Vec::new()
    }

    fn config_spawn_program(&self, _profile: &str) -> String {
        String::new()
    }

    fn config_spawn_argv(&self, _profile: &str) -> Vec<String> {
        Vec::new()
    }

    fn config_spawn_cwd(&self, _profile: &str) -> String {
        String::new()
    }

    fn owns_handle(&self, handle: i64) -> bool {
        EngineService::owns_handle(self, handle)
    }

    fn live_handle_count(&self) -> usize {
        EngineService::live_handle_count(self)
    }

    fn cached_cursor_viewport(&self, handle: i64) -> Option<TerminalRetiredCache> {
        EngineService::cached_cursor_viewport(self, handle)
    }
}

/// The engine-only default provider (process-shared service). Used when the
/// VM has neither an instance nor a process-default provider, preserving the
/// legacy "engine works without a UI host" behavior.
pub fn default_provider() -> Arc<dyn TerminalHostProvider> {
    let service: Arc<dyn TerminalHostProvider> = shared();
    service
}
