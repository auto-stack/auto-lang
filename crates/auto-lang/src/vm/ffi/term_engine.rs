//! PLAN-808: terminal native shims (`auto.term.*`, IDs 2943..2949,
//! 2957..2959, 2976..2982, 2983..2987, 2988..2993, 2998..3003).
//!
//! This module is **codec only**: pop arguments in the legacy order, call the
//! terminal capability port, push the result. The engine service, DLL symbol
//! contract, handle table and sampling live in `auto-terminal-core`; the UI
//! view/input/palette/window projection lives in `ui::terminal_provider`.
//!
//! Provider resolution (frozen T-01 assembly): instance provider → process
//! default provider → `auto_terminal_core::default_provider()` (the
//! process-shared engine service). The last step preserves the pre-extraction
//! "engine works without any UI host" behavior; a missing provider is NOT an
//! error state for this family.
//!
//! No Cargo edge to auto-term: the DLL is loaded at runtime by the core
//! service (search order: `AUTOTERM_ENGINE_DLL` → executable directory →
//! ancestor `target/{debug,release}`).

use std::sync::Arc;

use auto_terminal_core::TerminalHostProvider;

use crate::vm::engine::{AutoVM, VMError};
use crate::vm::ffi::convert::VMConvertible;
use crate::vm::task::AutoTask;

fn terminal_provider(vm: &AutoVM) -> Arc<dyn TerminalHostProvider> {
    vm.native_interface
        .terminal_provider()
        .unwrap_or_else(auto_terminal_core::default_provider)
}

// ── spawn / input / harvest ──────────────────────────────────────────────

/// 2943 `spawn(cols int, rows int) -> int`.
pub fn shim_term_spawn(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let rows = crate::vm::native::pop_arg_i32(task) as i64;
    let cols = crate::vm::native::pop_arg_i32(task) as i64;
    let handle = terminal_provider(vm).spawn(cols, rows);
    task.ram.push_nv(auto_val::encode_i32(handle));
    Ok(())
}

/// 2983 `spawn_ex(program str, argv []str, cwd str, cols int, rows int) -> int`.
pub fn shim_term_spawn_ex(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let rows = crate::vm::native::pop_arg_i32(task) as i64;
    let cols = crate::vm::native::pop_arg_i32(task) as i64;
    let cwd: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let argv: Vec<String> = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let program: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let handle = terminal_provider(vm).spawn_ex(&program, &argv, &cwd, cols, rows);
    task.ram.push_nv(auto_val::encode_i32(handle));
    Ok(())
}

/// 2944 `write_line(handle int, line str)`.
pub fn shim_term_write_line(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let line: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let handle = crate::vm::native::pop_arg_i32(task) as i64;
    terminal_provider(vm).write_line(handle, &line);
    Ok(())
}

/// 3000 `write_raw(handle int, vt str)`.
pub fn shim_term_write_raw(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let vt: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let handle = crate::vm::native::pop_arg_i32(task) as i64;
    terminal_provider(vm).write_raw(handle, &vt);
    Ok(())
}

/// 2945 `rows(handle int) -> []str`.
pub fn shim_term_rows(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let handle = crate::vm::native::pop_arg_i32(task) as i64;
    terminal_provider(vm)
        .rows(handle)
        .push_to_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))
}

/// 2984 `rows_for(handle int, key str) -> []str`.
pub fn shim_term_rows_for(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let key: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let handle = crate::vm::native::pop_arg_i32(task) as i64;
    terminal_provider(vm)
        .rows_for(handle, &key)
        .push_to_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))
}

/// 2946 `resize(handle int, cols int, rows int)`.
pub fn shim_term_resize(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let rows = crate::vm::native::pop_arg_i32(task) as i64;
    let cols = crate::vm::native::pop_arg_i32(task) as i64;
    let handle = crate::vm::native::pop_arg_i32(task) as i64;
    terminal_provider(vm).resize(handle, cols, rows);
    Ok(())
}

// ── lifecycle ────────────────────────────────────────────────────────────

/// 2947 `interrupt(handle int) -> int`.
pub fn shim_term_interrupt(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let handle = crate::vm::native::pop_arg_i32(task) as i64;
    let result = terminal_provider(vm).interrupt(handle);
    task.ram.push_nv(auto_val::encode_i32(result));
    Ok(())
}

/// 2948 `is_exited(handle int) -> bool`.
pub fn shim_term_is_exited(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let handle = crate::vm::native::pop_arg_i32(task) as i64;
    let exited = terminal_provider(vm).is_exited(handle);
    task.ram.push_nv(auto_val::encode_bool(exited));
    Ok(())
}

/// 2949 `free(handle int)`.
pub fn shim_term_free(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let handle = crate::vm::native::pop_arg_i32(task) as i64;
    terminal_provider(vm).free(handle);
    Ok(())
}

// ── input pump / geometry ────────────────────────────────────────────────

/// 2957 `pump_input(handle int) -> int`.
pub fn shim_term_pump_input(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let handle = crate::vm::native::pop_arg_i32(task) as i64;
    let count = terminal_provider(vm).pump_input(handle);
    task.ram.push_nv(auto_val::encode_i32(count));
    Ok(())
}

/// 2985 `pump_for(handle int, key str) -> int`.
pub fn shim_term_pump_for(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let key: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let handle = crate::vm::native::pop_arg_i32(task) as i64;
    let count = terminal_provider(vm).pump_for(handle, &key);
    task.ram.push_nv(auto_val::encode_i32(count));
    Ok(())
}

/// 2976 `apply_resize(handle int) -> int`.
pub fn shim_term_apply_resize(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let handle = crate::vm::native::pop_arg_i32(task) as i64;
    let applied = terminal_provider(vm).apply_resize(handle);
    task.ram.push_nv(auto_val::encode_i32(applied));
    Ok(())
}

/// 2986 `apply_resize_for(handle int, key str) -> int`.
pub fn shim_term_apply_resize_for(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let key: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let handle = crate::vm::native::pop_arg_i32(task) as i64;
    let applied = terminal_provider(vm).apply_resize_for(handle, &key);
    task.ram.push_nv(auto_val::encode_i32(applied));
    Ok(())
}

/// 3002 `pend_resize_geom(key str, geom str) -> int`.
pub fn shim_term_engine_pend_resize_geom(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let geom: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let key: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let queued = terminal_provider(vm).pend_resize_geom(&key, &geom);
    task.ram.push_nv(auto_val::encode_i32(queued));
    Ok(())
}

/// 3001 `scroll_pending_for(handle int, key str) -> int`.
pub fn shim_term_scroll_pending_for(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let key: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let handle = crate::vm::native::pop_arg_i32(task) as i64;
    let pending = terminal_provider(vm).scroll_pending_for(handle, &key);
    task.ram.push_nv(auto_val::encode_i32(pending));
    Ok(())
}

/// 2987 `menu_take() -> int`.
pub fn shim_term_menu_take(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let item = terminal_provider(vm).menu_take();
    task.ram.push_nv(auto_val::encode_i32(item));
    Ok(())
}

// ── cached geometry / query surfaces ─────────────────────────────────────

/// 2958 `cursor_row(handle int) -> int`.
pub fn shim_term_cursor_row(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let handle = crate::vm::native::pop_arg_i32(task) as i64;
    let row = terminal_provider(vm).cursor_row(handle);
    task.ram.push_nv(auto_val::encode_i32(row));
    Ok(())
}

/// 2959 `cursor_col(handle int) -> int`.
pub fn shim_term_cursor_col(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let handle = crate::vm::native::pop_arg_i32(task) as i64;
    let col = terminal_provider(vm).cursor_col(handle);
    task.ram.push_nv(auto_val::encode_i32(col));
    Ok(())
}

/// 2977 `viewport_cols(handle int) -> int`.
pub fn shim_term_viewport_cols(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let handle = crate::vm::native::pop_arg_i32(task) as i64;
    let cols = terminal_provider(vm).viewport_cols(handle);
    task.ram.push_nv(auto_val::encode_i32(cols));
    Ok(())
}

/// 2978 `viewport_rows(handle int) -> int`.
pub fn shim_term_viewport_rows(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let handle = crate::vm::native::pop_arg_i32(task) as i64;
    let rows = terminal_provider(vm).viewport_rows(handle);
    task.ram.push_nv(auto_val::encode_i32(rows));
    Ok(())
}

/// 3003 `history(handle int) -> int`.
pub fn shim_term_engine_history(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let handle = crate::vm::native::pop_arg_i32(task) as i64;
    let rows = terminal_provider(vm).history(handle);
    task.ram.push_nv(auto_val::encode_i32(rows));
    Ok(())
}

/// 2979 `backlog_pending_mb(handle int) -> int`.
pub fn shim_term_backlog_pending_mb(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let handle = crate::vm::native::pop_arg_i32(task) as i64;
    let mb = terminal_provider(vm).backlog_pending_mb(handle);
    task.ram.push_nv(auto_val::encode_i32(mb));
    Ok(())
}

/// 2980 `backlog_paused(handle int) -> int`.
pub fn shim_term_backlog_paused(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let handle = crate::vm::native::pop_arg_i32(task) as i64;
    let paused = terminal_provider(vm).backlog_paused(handle);
    task.ram.push_nv(auto_val::encode_i32(paused));
    Ok(())
}

/// 2981 `backlog_take_alerts(handle int) -> int`.
pub fn shim_term_backlog_take_alerts(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let handle = crate::vm::native::pop_arg_i32(task) as i64;
    let alerts = terminal_provider(vm).backlog_take_alerts(handle);
    task.ram.push_nv(auto_val::encode_i32(alerts));
    Ok(())
}

/// 2982 `backlog_dropped(handle int) -> int`.
pub fn shim_term_backlog_dropped(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let handle = crate::vm::native::pop_arg_i32(task) as i64;
    let dropped = terminal_provider(vm).backlog_dropped(handle);
    task.ram.push_nv(auto_val::encode_i32(dropped));
    Ok(())
}

/// 2998 `window_width() -> int`.
pub fn shim_term_window_width(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let width = terminal_provider(vm).window_width();
    task.ram.push_nv(auto_val::encode_i32(width));
    Ok(())
}

/// 2999 `window_height() -> int`.
pub fn shim_term_window_height(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let height = terminal_provider(vm).window_height();
    task.ram.push_nv(auto_val::encode_i32(height));
    Ok(())
}

// ── configuration stubs (intentionally empty; no autoterm-config edge) ───

/// 2988 `config_profiles() -> []str`.
pub fn shim_term_config_profiles(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let profiles = terminal_provider(vm).config_profiles();
    profiles
        .push_to_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))
}

/// 2989 `config_default_profile() -> str`.
pub fn shim_term_config_default_profile(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    terminal_provider(vm)
        .config_default_profile()
        .push_to_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))
}

/// 2990 `config_profiles_full() -> []str`.
pub fn shim_term_config_profiles_full(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let profiles = terminal_provider(vm).config_profiles_full();
    profiles
        .push_to_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))
}

/// 2991 `config_spawn_program(profile str) -> str`.
pub fn shim_term_config_spawn_program(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let profile: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    terminal_provider(vm)
        .config_spawn_program(&profile)
        .push_to_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))
}

/// 2992 `config_spawn_argv(profile str) -> []str`.
pub fn shim_term_config_spawn_argv(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let profile: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let argv = terminal_provider(vm).config_spawn_argv(&profile);
    argv.push_to_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))
}

/// 2993 `config_spawn_cwd(profile str) -> str`.
pub fn shim_term_config_spawn_cwd(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let profile: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    terminal_provider(vm)
        .config_spawn_cwd(&profile)
        .push_to_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))
}

// ── window-pixel compatibility facade (view state owner = UI adapter) ────

/// PLAN-024 T-01 desktop-track injection. Compat facade forwarding to the
/// named UI view adapter; the state is a thread-local view projection, not
/// engine state. No-UI builds have no window and keep the legacy "function
/// absent" surface.
#[cfg(feature = "ui")]
pub fn set_app_window_px(w: f32, h: f32) {
    crate::ui::terminal_provider::set_app_window_px(w, h);
}

/// Test read face (injection semantics assertions).
#[cfg(all(test, feature = "ui"))]
pub(crate) fn app_window_px_for_test() -> (f32, f32) {
    crate::ui::terminal_provider::app_window_px_for_test()
}
