//! PLAN-690 T-02 —— rqhost daemon 窗 IME 激活门控（Windows IMM 直写）。
//!
//! iced 0.14 无公开 `window::InputMethod` task（iced_winit 的
//! `request_input_method` 是内部方法，每帧 `update_mouse` 同款自刷）。
//! rqhost daemon 经 `iced::window::run`（iced_runtime/window.rs:463）在
//! 事件循环线程拿 `&dyn Window` → `RawWindowHandle::Win32` → HWND，按
//! winit 0.30.13 `ime.rs` 的同语义直写 IMM：
//! - enable = `ImmAssociateContextEx(hwnd, IACE_DEFAULT)` +
//!   `ImmSetCompositionWindow(CFS_POINT)` + `ImmSetCandidateWindow(CFS_EXCLUDE)`
//!   （组合窗/候选窗定位到 App 下发的焦点框矩形——物理坐标 = App 逻辑
//!   坐标 × daemon 窗 scale factor）；
//! - disable = `ImmAssociateContextEx(hwnd, IACE_CHILDREN)`（winit 同式）；
//! - purpose = Windows no-op（winit `set_ime_purpose` 空实现）。
//!
//! 非 Windows 宿主 = 观测行 no-op（remote 桌面 v1 主战场 Windows；
//! Linux/Wayland IME 走 zwp text-input，v1 不展开——SD-01 边界注记）。

use super::message::ImeReq;

/// IME enable：激活上下文 + 组合窗/候选窗定位（daemon 窗事件循环线程
/// 调用——`window::run` 闭包保证线程正确性，winit `execute_in_thread`
/// 同则）。
pub fn enable(window: &dyn iced_runtime::window::Window, req: &ImeReq, scale: f64, tag: &str) {
    eprintln!(
        "[rqhost] ime{tag} enable cursor=({:.0},{:.0} {:.0}x{:.0}) purpose={}",
        req.cursor.x, req.cursor.y, req.cursor.w, req.cursor.h, req.purpose
    );
    apply_cursor_area(window, Some(req), scale);
}

/// IME disable：失焦关上下文（App 侧 `InputMethod::Disabled` 下行）。
pub fn disable(window: &dyn iced_runtime::window::Window, tag: &str) {
    eprintln!("[rqhost] ime{tag} disable");
    apply_cursor_area(window, None, 1.0);
}

#[cfg(windows)]
fn hwnd_of(window: &dyn iced_runtime::window::Window) -> Option<windows::Win32::Foundation::HWND> {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    let handle = window.window_handle().ok()?;
    match handle.as_raw() {
        RawWindowHandle::Win32(h) => Some(windows::Win32::Foundation::HWND(
            h.hwnd.get() as *mut core::ffi::c_void,
        )),
        _ => None,
    }
}

#[cfg(windows)]
fn apply_cursor_area(
    window: &dyn iced_runtime::window::Window,
    req: Option<&ImeReq>,
    scale: f64,
) {
    use windows::Win32::Foundation::{POINT, RECT};
    use windows::Win32::UI::Input::Ime::{
        ImmAssociateContextEx, ImmGetContext, ImmReleaseContext, ImmSetCandidateWindow,
        ImmSetCompositionWindow, CANDIDATEFORM, COMPOSITIONFORM, CFS_EXCLUDE, CFS_POINT,
        IACE_CHILDREN, IACE_DEFAULT,
    };

    let Some(hwnd) = hwnd_of(window) else {
        eprintln!("[rqhost] ime 非 Win32 句柄——跳过");
        return;
    };
    unsafe {
        // 上下文开关（winit ImeContext::set_ime_allowed 同语义：启用 =
        // 关联缺省上下文，禁用 = 解除关联）。
        match req {
            Some(_) => {
                let _ = ImmAssociateContextEx(hwnd, None, IACE_DEFAULT);
            }
            None => {
                let _ = ImmAssociateContextEx(hwnd, None, IACE_CHILDREN);
                return;
            }
        }
        // 焦点框定位（winit set_ime_cursor_area 同式）：组合窗锚 =
        // 框底左（CFS_POINT），候选窗排除区 = 全框（CFS_EXCLUDE）。
        let Some(req) = req else { return };
        let himc = ImmGetContext(hwnd);
        if himc.is_invalid() {
            eprintln!("[rqhost] ime ImmGetContext 失败——定位跳过");
            return;
        }
        let (px, py) = (
            (req.cursor.x as f64 * scale).round() as i32,
            (req.cursor.y as f64 * scale).round() as i32,
        );
        let (pw, ph) = (
            (req.cursor.w as f64 * scale).round() as i32,
            (req.cursor.h as f64 * scale).round() as i32,
        );
        let rc = RECT { left: px, top: py, right: px + pw, bottom: py + ph };
        let composition = COMPOSITIONFORM {
            dwStyle: CFS_POINT,
            ptCurrentPos: POINT { x: px, y: py + ph },
            rcArea: rc,
        };
        let _ = ImmSetCompositionWindow(himc, &composition);
        let candidate = CANDIDATEFORM {
            dwIndex: 0,
            dwStyle: CFS_EXCLUDE,
            ptCurrentPos: POINT { x: px, y: py },
            rcArea: rc,
        };
        let _ = ImmSetCandidateWindow(himc, &candidate);
        ImmReleaseContext(hwnd, himc);
    }
}

#[cfg(not(windows))]
fn apply_cursor_area(
    _window: &dyn iced_runtime::window::Window,
    _req: Option<&ImeReq>,
    _scale: f64,
) {
    // 非 Windows daemon：观测行已由 enable/disable 打出，实际 IME 门控
    // v1 不展开（平台面归 SD-01 边界注记）。
}
