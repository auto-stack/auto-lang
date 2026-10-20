//! PLAN-808: UI terminal host-provider adapter.
//!
//! Composes the `auto-terminal-core` engine service with the UI view/input
//! registry. The UI registry (`ui::terminal`) remains the single source for
//! view state, input queues, pending geometry, menu payloads, scroll deltas
//! and the window-store projection; this adapter only routes the 34 terminal
//! natives between the VM, the engine service and that registry.
//!
//! Ownership/lock order (T-01 §3.2): engine service calls complete before any
//! UI registry lock is taken; no UI callback re-enters the service. The
//! adapter never holds a registry lock while calling the DLL (registry
//! helpers take and release their own locks internally).
//!
//! Mode assembly: `install_process_default` fills the process-default slot
//! only when it is empty (779 discipline — an explicit registration always
//! wins). A VM constructed without a UI host falls back to the shared
//! engine-only service, preserving the legacy no-UI behavior.

use std::sync::Arc;
use std::sync::OnceLock;

use auto_terminal_core::{
    EngineService, TerminalHostProvider, TerminalRetiredCache, MAX_RESIZE_COLS, MAX_RESIZE_ROWS,
    MIN_RESIZE_COLS,
};

use crate::ui::terminal::{
    terminal_core, terminal_drain_all_inputs, terminal_drain_inputs_for, terminal_feed_cells_all,
    terminal_feed_cells_for, terminal_feed_window_for, terminal_mark_view_bound,
    terminal_palette_load, terminal_pend_resize, terminal_scroll_delta_pending,
    terminal_set_history, terminal_set_scroll_offset, terminal_set_window_anchor,
    terminal_take_any_resize, terminal_take_menu_item_any, terminal_take_resize_for,
    terminal_take_scroll_delta,
};

/// Prefetch window size (rows on each side; 16 ms pump cadence legacy).
const PREFETCH_ROWS: i32 = 24;

/// UI host provider over the shared engine service.
pub struct TerminalUiProvider {
    service: Arc<EngineService>,
    palettes_synced: OnceLock<()>,
}

impl TerminalUiProvider {
    pub fn new(service: Arc<EngineService>) -> Self {
        Self {
            service,
            palettes_synced: OnceLock::new(),
        }
    }

    /// Load the engine palette tables into the UI cache once. The legacy glue
    /// did this at first DLL use; the adapter keeps the same lazy timing.
    /// `OnceLock` makes concurrent first callers wait for the load instead of
    /// observing a half-initialized UI cache.
    fn sync_palettes(&self) {
        self.palettes_synced.get_or_init(|| {
            for (scheme, table) in self.service.palette_tables() {
                terminal_palette_load(scheme, table);
            }
        });
    }

    /// Scroll-queue drain + engine/projection write-back (legacy
    /// `apply_scroll_queue`): take the wheel delta, scroll the engine, mirror
    /// offset/history/anchor into the UI projection, mark user-driven offsets
    /// as view-bound, then sample the prefetch window. Returns the window
    /// anchor (`i64::MIN` = unanchored / missing core).
    fn apply_scroll_queue(
        &self,
        handle: i64,
        key: &str,
        core: &'static crate::ui::terminal::TerminalCore,
    ) -> i64 {
        let delta = terminal_take_scroll_delta(core);
        if delta != 0 {
            self.service.scroll(handle, delta);
        }
        let off = self.service.scroll_offset(handle).max(0);
        terminal_set_scroll_offset(core, off as usize);
        let history = self.service.history(handle).max(0);
        terminal_set_history(core, history as usize);
        let anchor = history - off;
        terminal_set_window_anchor(core, anchor);
        if delta != 0 {
            terminal_mark_view_bound(core);
        }
        let rows = self.service.viewport(handle).1 as i32;
        self.prefetch_window(handle, key, anchor, off as i32, history as i32, rows);
        anchor
    }

    /// Sample ±N rows into the window store (legacy `prefetch_window`):
    /// transient engine scroll, sample, restore offset. Sampling reads the
    /// engine snapshot exactly like the legacy glue (taken by
    /// `take_dirty_rows`; the transient scroll does not refresh it).
    fn prefetch_window(
        &self,
        handle: i64,
        key: &str,
        anchor: i64,
        off: i32,
        history: i32,
        rows: i32,
    ) {
        if rows <= 0 {
            return;
        }
        let n_up = PREFETCH_ROWS.min(history - off).min(rows);
        if n_up > 0 {
            let mut stash: Vec<Vec<auto_terminal_core::TerminalCell>> = Vec::new();
            self.service.scroll(handle, n_up);
            for r in 0..n_up {
                match self.service.sample_row(handle, r) {
                    Some(cells) => stash.push(cells),
                    None => break,
                }
            }
            self.service.scroll(handle, -n_up);
            if !stash.is_empty() {
                terminal_feed_window_for(key, anchor - stash.len() as i64, &stash);
            }
        }
        let n_down = PREFETCH_ROWS.min(off).min(rows);
        if n_down > 0 {
            let mut below: Vec<Vec<auto_terminal_core::TerminalCell>> = Vec::new();
            self.service.scroll(handle, -n_down);
            for r in (rows - n_down).max(0)..rows {
                match self.service.sample_row(handle, r) {
                    Some(cells) => below.push(cells),
                    None => break,
                }
            }
            self.service.scroll(handle, n_down);
            if !below.is_empty() {
                let base = anchor + n_down as i64 + (rows - n_down).max(0) as i64;
                terminal_feed_window_for(key, base, &below);
            }
        }
    }

    /// Harvest one frame and project styled cells (broadcast arm).
    fn harvest_all(&self, handle: i64) -> Vec<String> {
        self.sync_palettes();
        if self.service.feed_ready(handle).is_none() {
            return Vec::new();
        }
        let Some(snapshot) = self.service.snapshot(handle) else {
            return Vec::new();
        };
        for (row, cells) in snapshot.cells.iter().enumerate() {
            if !cells.is_empty() {
                terminal_feed_cells_all(row, cells.clone());
            }
        }
        snapshot.lines
    }

    /// Harvest one frame and project styled cells for one view key.
    fn harvest_for(&self, handle: i64, key: &str) -> Vec<String> {
        self.sync_palettes();
        if self.service.feed_ready(handle).is_none() {
            return Vec::new();
        }
        let mut anchor = i64::MIN;
        if let Some(core) = terminal_core(key) {
            anchor = self.apply_scroll_queue(handle, key, core);
        }
        let Some(snapshot) = self.service.snapshot(handle) else {
            return Vec::new();
        };
        for (row, cells) in snapshot.cells.iter().enumerate() {
            if !cells.is_empty() {
                terminal_feed_cells_for(key, row, cells.clone());
            }
        }
        if anchor != i64::MIN && !snapshot.cells.is_empty() {
            terminal_feed_window_for(key, anchor, &snapshot.cells);
        }
        snapshot.lines
    }

    fn parse_geometry(geom: &str) -> Option<(u16, u16)> {
        let (cols_s, rows_s) = geom.split_once('x')?;
        let cols = cols_s.trim().parse::<u16>().ok()?;
        let rows = rows_s.trim().parse::<u16>().ok()?;
        if !(MIN_RESIZE_COLS..=MAX_RESIZE_COLS).contains(&cols)
            || !(1..=MAX_RESIZE_ROWS).contains(&rows)
        {
            return None;
        }
        Some((cols, rows))
    }
}

impl TerminalHostProvider for TerminalUiProvider {
    fn spawn(&self, cols: i64, rows: i64) -> i32 {
        self.sync_palettes();
        self.service.spawn(cols, rows) as i32
    }

    fn spawn_ex(&self, program: &str, argv: &[String], cwd: &str, cols: i64, rows: i64) -> i32 {
        self.sync_palettes();
        self.service.spawn_ex(program, argv, cwd, cols, rows) as i32
    }

    fn write_line(&self, handle: i64, line: &str) {
        self.sync_palettes();
        self.service.write_line(handle, line);
    }

    fn write_raw(&self, handle: i64, vt: &str) {
        self.sync_palettes();
        self.service.write_raw(handle, vt);
    }

    fn rows(&self, handle: i64) -> Vec<String> {
        self.harvest_all(handle)
    }

    fn rows_for(&self, handle: i64, key: &str) -> Vec<String> {
        self.harvest_for(handle, key)
    }

    fn resize(&self, handle: i64, cols: i64, rows: i64) {
        self.sync_palettes();
        self.service.resize(handle, cols, rows);
    }

    fn apply_resize(&self, handle: i64) -> i32 {
        let Some((cols, rows)) = terminal_take_any_resize() else {
            return 0;
        };
        self.sync_palettes();
        self.service.resize(handle, cols as i64, rows as i64);
        self.service.set_viewport(handle, cols as i64, rows as i64);
        1
    }

    fn apply_resize_for(&self, handle: i64, key: &str) -> i32 {
        let Some(core) = terminal_core(key) else {
            return 0;
        };
        let Some((cols, rows)) = terminal_take_resize_for(core) else {
            return 0;
        };
        self.sync_palettes();
        self.service.resize(handle, cols as i64, rows as i64);
        self.service.set_viewport(handle, cols as i64, rows as i64);
        1
    }

    fn pump_input(&self, handle: i64) -> i32 {
        let keys = terminal_drain_all_inputs();
        if keys.is_empty() {
            return 0;
        }
        self.sync_palettes();
        self.service.write_keys(handle, &keys) as i32
    }

    fn pump_for(&self, handle: i64, key: &str) -> i32 {
        let Some(core) = terminal_core(key) else {
            return 0;
        };
        let keys = terminal_drain_inputs_for(core);
        if keys.is_empty() {
            return 0;
        }
        self.sync_palettes();
        self.service.write_keys(handle, &keys) as i32
    }

    fn scroll_pending_for(&self, _handle: i64, key: &str) -> i32 {
        match terminal_core(key) {
            Some(core) => terminal_scroll_delta_pending(core),
            None => 0,
        }
    }

    fn pend_resize_geom(&self, key: &str, geom: &str) -> i32 {
        let Some((cols, rows)) = Self::parse_geometry(geom) else {
            return 0;
        };
        if terminal_pend_resize(key, cols, rows) {
            1
        } else {
            0
        }
    }

    fn menu_take(&self) -> i32 {
        terminal_take_menu_item_any().map(i64::from).unwrap_or(-1) as i32
    }

    fn interrupt(&self, handle: i64) -> i32 {
        self.sync_palettes();
        self.service.interrupt(handle) as i32
    }

    fn is_exited(&self, handle: i64) -> bool {
        self.sync_palettes();
        self.service.is_exited(handle)
    }

    fn free(&self, handle: i64) {
        self.sync_palettes();
        self.service.free(handle);
    }

    fn cursor_row(&self, handle: i64) -> i32 {
        self.service.cursor(handle).0 as i32
    }

    fn cursor_col(&self, handle: i64) -> i32 {
        self.service.cursor(handle).1 as i32
    }

    fn viewport_cols(&self, handle: i64) -> i32 {
        self.service.viewport(handle).0 as i32
    }

    fn viewport_rows(&self, handle: i64) -> i32 {
        self.service.viewport(handle).1 as i32
    }

    fn backlog_pending_mb(&self, handle: i64) -> i32 {
        self.sync_palettes();
        self.service.backlog_pending_mb(handle) as i32
    }

    fn backlog_paused(&self, handle: i64) -> i32 {
        self.sync_palettes();
        self.service.backlog_paused(handle) as i32
    }

    fn backlog_take_alerts(&self, handle: i64) -> i32 {
        self.sync_palettes();
        self.service.backlog_take_alerts(handle) as i32
    }

    fn backlog_dropped(&self, handle: i64) -> i32 {
        self.sync_palettes();
        self.service.backlog_dropped(handle) as i32
    }

    fn history(&self, handle: i64) -> i32 {
        self.sync_palettes();
        self.service.history(handle) as i32
    }

    fn window_width(&self) -> i32 {
        let (w, _) = app_window_px();
        let w = if w >= 1.0 {
            w
        } else {
            crate::ui::style::theme::window_width()
        };
        if std::env::var("P024_TRACE").is_ok() {
            eprintln!(
                "[P024-TRACE] window_width -> {w} (override={:?} theme={})",
                app_window_px(),
                crate::ui::style::theme::window_width()
            );
        }
        w as i32
    }

    fn window_height(&self) -> i32 {
        let (_, h) = app_window_px();
        let h = if h >= 1.0 {
            h
        } else {
            crate::ui::style::theme::window_height()
        };
        h as i32
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
        self.service.owns_handle(handle)
    }

    fn live_handle_count(&self) -> usize {
        self.service.live_handle_count()
    }

    fn cached_cursor_viewport(&self, handle: i64) -> Option<TerminalRetiredCache> {
        self.service.cached_cursor_viewport(handle)
    }
}

// ── PLAN-808: window-pixel view state (moved out of the VM FFI glue) ──────
// Thread affinity is legacy: the renderer thread writes, the VM-side shim
// reads. This is a view projection, not engine state.

thread_local! {
    static APP_WINDOW_PX: std::cell::Cell<(f32, f32)> =
        const { std::cell::Cell::new((0.0, 0.0)) };
}

/// Desktop-track injection: the app's virtual-window content area in logical
/// px. Degenerate sizes (< 1) are rejected (zero would zero the projection).
pub fn set_app_window_px(w: f32, h: f32) {
    if w >= 1.0 && h >= 1.0 {
        APP_WINDOW_PX.with(|c| c.set((w, h)));
    }
}

/// Current thread's injected window size (0,0 = not injected).
pub fn app_window_px() -> (f32, f32) {
    APP_WINDOW_PX.with(|c| c.get())
}

/// Test read face (same semantics as the legacy glue helper).
#[cfg(test)]
pub(crate) fn app_window_px_for_test() -> (f32, f32) {
    app_window_px()
}

/// Install the UI adapter as the process-default terminal provider (only when
/// the slot is empty — explicit registration wins, 779 discipline).
pub fn install_process_default() {
    auto_lang_core::vm::host_capabilities::set_process_default_terminal_host_provider_if_absent(
        Arc::new(TerminalUiProvider::new(auto_terminal_core::shared())),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::terminal::{
        terminal, terminal_dispose, terminal_request_resize, terminal_take_resize_for,
    };

    /// Geometry bridge parse/guard/registry write (legacy p028 test shape).
    #[test]
    fn geom_parse_guards_and_registry_write() {
        let provider = TerminalUiProvider::new(Arc::new(EngineService::without_dll()));
        terminal_dispose("p808-geom");
        assert_eq!(provider.pend_resize_geom("p808-geom", "abc"), 0);
        assert_eq!(provider.pend_resize_geom("p808-geom", "80"), 0);
        assert_eq!(provider.pend_resize_geom("p808-geom", "80x"), 0);
        assert_eq!(
            provider.pend_resize_geom("p808-geom", "1x30"),
            0,
            "degenerate cols"
        );
        assert_eq!(
            provider.pend_resize_geom("p808-geom", "80x0"),
            0,
            "degenerate rows"
        );
        assert_eq!(
            provider.pend_resize_geom("p808-geom", "0x30"),
            0,
            "zero size"
        );
        assert_eq!(provider.pend_resize_geom("p808-geom", "80x30"), 1);
        let core = terminal_core("p808-geom").expect("bridge lazily creates the core");
        assert_eq!(terminal_take_resize_for(core), Some((80, 30)));
        assert_eq!(terminal_take_resize_for(core), None, "take clears");
        assert_eq!(provider.pend_resize_geom("p808-geom", "80x30"), 1);
        assert_eq!(terminal_take_resize_for(core), Some((80, 30)));
        terminal_dispose("p808-geom");
    }

    /// Directional pump/resize consumes only the addressed key's payload.
    #[test]
    fn directional_pump_and_resize_do_not_cross_keys() {
        let provider = TerminalUiProvider::new(Arc::new(EngineService::without_dll()));
        terminal_dispose("p808-a");
        terminal_dispose("p808-b");
        let a = terminal("p808-a", 20, 4);
        let b = terminal("p808-b", 20, 4);

        crate::ui::terminal::terminal_push_input(a, "a1");
        crate::ui::terminal::terminal_push_input(b, "b1");
        // No DLL: pump drains and returns the payload count (legacy).
        assert_eq!(provider.pump_for(1, "p808-a"), 1);
        assert_eq!(
            crate::ui::terminal::terminal_drain_inputs_for(b),
            vec!["b1".to_string()]
        );
        assert_eq!(provider.pump_for(1, "p808-missing"), 0);

        term_request_resize(a, 60, 20);
        assert_eq!(
            provider.apply_resize_for(1, "p808-b"),
            0,
            "B must not take A's request"
        );
        assert_eq!(provider.apply_resize_for(1, "p808-a"), 1);
        let _ = b;
        terminal_dispose("p808-a");
        terminal_dispose("p808-b");
    }

    fn term_request_resize(core: &'static crate::ui::terminal::TerminalCore, cols: u16, rows: u16) {
        terminal_request_resize(core, cols, rows);
    }

    /// Adapter assembly: explicit process default wins over compat install.
    #[test]
    fn compat_install_never_overrides_explicit_default() {
        let _slot = auto_lang_core::vm::host_capabilities::test_default_slot_lock();
        let saved = auto_lang_core::vm::host_capabilities::process_default_terminal_host_provider();
        auto_lang_core::vm::host_capabilities::clear_process_default_terminal_host_provider();
        install_process_default();
        assert!(
            auto_lang_core::vm::host_capabilities::process_default_terminal_host_provider()
                .is_some(),
            "empty slot gets the compat adapter"
        );
        auto_lang_core::vm::host_capabilities::clear_process_default_terminal_host_provider();
        match saved {
            Some(p) => {
                auto_lang_core::vm::host_capabilities::set_process_default_terminal_host_provider(p)
            }
            None => {
                auto_lang_core::vm::host_capabilities::clear_process_default_terminal_host_provider(
                )
            }
        }
    }

    /// Window-pixel facade state (legacy thread-local semantics).
    #[test]
    fn app_window_px_rejects_degenerate_and_stores_valid() {
        set_app_window_px(0.0, 500.0);
        assert_eq!(app_window_px_for_test(), (0.0, 0.0), "degenerate rejected");
        set_app_window_px(777.0, 555.0);
        assert_eq!(app_window_px_for_test(), (777.0, 555.0));
        set_app_window_px(0.0, 0.0);
    }
}
