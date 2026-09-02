//! 非 Windows 平台（或未启用 `native-dock` feature）的 `win32` 同名 no-op 顶替。
//!
//! API 面与 `win32.rs` 一一对应：发现永远为空、几何/样式操作返回
//! `DockError::Api { op, code: 0 }`、谓词恒 false。宿主层（session/renderer）
//! 因此无需 cfg 包裹即可编译；dock 装配在非 Windows 平台不产生任何系统调用。

use crate::ui::native_dock::{NativeHwnd, Rect};

/// 与 `win32.rs` 同型错误枚举（no-op 平台不会产生 Elevated/StaleHwnd）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockError {
    /// 目标提权（仅 Windows 真实路径会出现）。
    Elevated,
    /// 句柄已失效（仅 Windows 真实路径会出现）。
    StaleHwnd,
    /// no-op 平台的统一失败形态。
    Api { op: &'static str, code: u32 },
}

pub fn find_top_level_by_pid(_pid: u32) -> Option<NativeHwnd> {
    None
}

pub fn list_top_level_windows() -> Vec<(NativeHwnd, u32, String)> {
    Vec::new()
}

pub fn pid_of(_target: NativeHwnd) -> Option<u32> {
    None
}

pub fn get_title(_target: NativeHwnd) -> Option<String> {
    None
}

pub fn set_bounds(_target: NativeHwnd, _rect: Rect) -> Result<(), DockError> {
    Err(DockError::Api { op: "noop", code: 0 })
}

pub fn get_bounds(_target: NativeHwnd) -> Option<Rect> {
    None
}

pub fn probe_bounds(_target: NativeHwnd, _requested: Rect) -> Option<Rect> {
    None
}

pub fn strip_chrome(_target: NativeHwnd) -> Result<u32, DockError> {
    Err(DockError::Api { op: "noop", code: 0 })
}

pub fn restore_chrome(_target: NativeHwnd, _saved: u32) -> Result<(), DockError> {
    Err(DockError::Api { op: "noop", code: 0 })
}

pub fn set_square_corners(_target: NativeHwnd) -> bool {
    false
}

pub fn sink_desktop_below(_desktop: NativeHwnd, _slot: NativeHwnd) -> Result<(), DockError> {
    Err(DockError::Api { op: "noop", code: 0 })
}

/// Plan 494：真洞 z 序翻转（no-op 平台同型失败）。
pub fn raise_desktop_above(_desktop: NativeHwnd, _slot: NativeHwnd) -> Result<(), DockError> {
    Err(DockError::Api { op: "noop", code: 0 })
}

/// Plan 494：Region 洞排除（no-op 平台同型失败；宿主层据此走回退路径）。
pub fn apply_hole_regions(
    _target: NativeHwnd,
    _win: Rect,
    _holes: &[Rect],
) -> Result<(), DockError> {
    Err(DockError::Api { op: "noop", code: 0 })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShowMode {
    Restore,
    Maximize,
    Minimize,
    Hide,
}

pub fn show_window(_target: NativeHwnd, _mode: ShowMode) -> Result<(), DockError> {
    Err(DockError::Api { op: "noop", code: 0 })
}

pub fn request_close(_target: NativeHwnd) -> Result<(), DockError> {
    Err(DockError::Api { op: "noop", code: 0 })
}

pub fn is_alive(_target: NativeHwnd) -> bool {
    false
}

pub fn is_minimized(_target: NativeHwnd) -> bool {
    false
}

pub fn is_maximized(_target: NativeHwnd) -> bool {
    false
}

/// 与 `win32.rs` 同名光标采样（no-op 平台无指针读数）。
pub fn cursor_pos() -> Option<(i32, i32)> {
    None
}

/// 与 `win32.rs` 同名前台聚焦（no-op 平台恒失败）。
pub fn focus_window(_target: NativeHwnd) -> bool {
    false
}

/// Plan 515 D1：no-op 平台恒无真图标（调用方回退 `app-window` 占位）。
pub fn window_icon_rgba(_target: NativeHwnd) -> Option<(Vec<u8>, u32, u32)> {
    None
}
