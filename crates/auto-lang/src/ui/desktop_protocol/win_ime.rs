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
//! **T-04 实机勘定补全（2026-09-22）——wndproc 子类桥**：winit 的
//! `WM_IME_COMPOSITION` 处理臂门控在其内部 `ime_allowed` 旗标后
//!（event_loop.rs:1530；旗标仅 `Window::set_ime_allowed` 置位——iced
//! 只在其自身 widget 请求 `InputMethod::Enabled` 时经内部方法调用，
//! `&dyn Window` 面不可达）；旗标 false 时组合/提交事件被整段吞掉，
//! 且提交串走 `WM_IME_CHAR`（winit 无该 handler，恒丢）。OS 侧组合
//! UI 不受此旗标影响（ImmAssociateContextEx 即生效——实机候选窗/
//! 组合串可见、定位正确）。桥 = `SetWindowLongPtrW(GWLP_WNDPROC)`
//! 挂自有 proc：`WM_IME_COMPOSITION` 读 `GCS_RESULTSTR`（提交串）/
//! `GCS_COMPSTR`+`GCS_CURSORPOS`（组合串+光标字节位）→ 上行注册表 →
//! daemon 15ms Tick 泵 drain → `LiveInput::ImeCommit/ImePreedit` 上行
//! （winit 路径之外的原语面，非双投——winit 侧两条路都死）。Esc 取消
//! = OS 自理（组合态消失，App 无残留态——headless 截获环测试③④同
//! 语义）。
//!
//! 非 Windows 宿主 = 观测行 no-op（remote 桌面 v1 主战场 Windows；
//! Linux/Wayland IME 走 zwp text-input，v1 不展开——SD-01 边界注记）。

use super::message::ImeReq;

/// IME 上行事件（子类桥产出——daemon Tick 泵 drain 后转 LiveInput）。
#[derive(Debug, Clone, PartialEq)]
pub struct ImeUplink {
    /// 桥窗的 iced window Id（注册表随桥登记——client 路由键）。
    pub window: iced::window::Id,
    /// 提交串（GCS_RESULTSTR）。
    pub commit: Option<String>,
    /// 组合串 + 光标字节位（GCS_COMPSTR + GCS_CURSORPOS）。
    pub preedit: Option<(String, i32)>,
}

#[cfg(windows)]
mod bridge {
    use super::ImeUplink;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use std::sync::OnceLock;

    /// 桥注册表：hwnd → (前一 wndproc, iced 窗 Id, 上行队列)。事件循环
    /// 线程读写（子类挂载/卸载 + wndproc 本体 + daemon Tick drain 同
    /// 线程）——Mutex 防御 iced/winit 潜在的跨线程消息泵路径。
    pub struct Entry {
        pub prev: isize,
        pub window: iced::window::Id,
        pub uplinks: Vec<ImeUplink>,
    }

    pub fn registry() -> &'static Mutex<HashMap<isize, Entry>> {
        static REG: OnceLock<Mutex<HashMap<isize, Entry>>> = OnceLock::new();
        REG.get_or_init(|| Mutex::new(HashMap::new()))
    }

    /// 组合串读取（GCS_* → UTF-16 → String）。
    fn composition_string(
        himc: windows::Win32::UI::Input::Ime::HIMC,
        gcs: windows::Win32::UI::Input::Ime::IME_COMPOSITION_STRING,
    ) -> Option<String> {
        use windows::Win32::UI::Input::Ime::ImmGetCompositionStringW;
        let len = unsafe { ImmGetCompositionStringW(himc, gcs, None, 0) };
        if len < 0 {
            return None;
        }
        let bytes = len as usize;
        if bytes == 0 {
            return Some(String::new());
        }
        let mut buf = vec![0u16; bytes / 2 + 1];
        let got = unsafe {
            ImmGetCompositionStringW(
                himc,
                gcs,
                Some(buf.as_mut_ptr().cast()),
                bytes as u32,
            )
        };
        if got < 0 {
            return None;
        }
        let units = got as usize / 2;
        let mut chars = Vec::with_capacity(units);
        chars.extend_from_slice(&buf[..units]);
        String::from_utf16(&chars).ok()
    }

    type WndProc = unsafe extern "system" fn(
        windows::Win32::Foundation::HWND,
        u32,
        windows::Win32::Foundation::WPARAM,
        windows::Win32::Foundation::LPARAM,
    ) -> windows::Win32::Foundation::LRESULT;

    /// 桥 wndproc：WM_IME_* 截获（组合/提交上行）后原链转发。
    unsafe extern "system" fn ime_wndproc(
        hwnd: windows::Win32::Foundation::HWND,
        msg: u32,
        wparam: windows::Win32::Foundation::WPARAM,
        lparam: windows::Win32::Foundation::LPARAM,
    ) -> windows::Win32::Foundation::LRESULT {
        use windows::Win32::UI::Input::Ime::{
            ImmGetCompositionStringW, ImmGetContext, ImmReleaseContext, GCS_COMPSTR,
            GCS_CURSORPOS, GCS_RESULTSTR,
        };
        use windows::Win32::UI::WindowsAndMessaging::{
            CallWindowProcW, WM_IME_COMPOSITION,
        };

        let hwnd_addr = hwnd.0 as isize;
        if msg == WM_IME_COMPOSITION {
            let himc = unsafe { ImmGetContext(hwnd) };
            if !himc.is_invalid() {
                let window = registry().lock().unwrap().get(&hwnd_addr).map(|e| e.window);
                if let Some(window) = window {
                    let mut uplink = ImeUplink {
                        window,
                        commit: None,
                        preedit: None,
                    };
                if lparam.0 & GCS_RESULTSTR.0 as isize != 0 {
                    uplink.commit = composition_string(himc, GCS_RESULTSTR);
                }
                if lparam.0 & GCS_COMPSTR.0 as isize != 0 {
                    if let Some(text) = composition_string(himc, GCS_COMPSTR) {
                        let pos = unsafe {
                            ImmGetCompositionStringW(himc, GCS_CURSORPOS, None, 0)
                        };
                        uplink.preedit = Some((text, pos));
                    }
                }
                    if lparam.0 == 0 {
                        // winit 同式：lparam==0 → 清组合态（App 侧 preedit 复位）。
                        uplink.preedit = Some((String::new(), 0));
                    }
                    if let Some(entry) = registry().lock().unwrap().get_mut(&hwnd_addr) {
                        entry.uplinks.push(uplink);
                    }
                }
                unsafe { ImmReleaseContext(hwnd, himc) };
            }
        }
        let prev = registry().lock().unwrap().get(&hwnd_addr).map(|e| e.prev);
        match prev {
            Some(prev) => {
                let prev: WndProc = std::mem::transmute(prev);
                unsafe { CallWindowProcW(Some(prev), hwnd, msg, wparam, lparam) }
            }
            None => unsafe {
                windows::Win32::UI::WindowsAndMessaging::DefWindowProcW(hwnd, msg, wparam, lparam)
            },
        }
    }

    /// 挂桥（enable 路径调用，事件循环线程）。已挂 = 幂等（窗 Id 刷新）。
    pub fn attach(hwnd: isize, window: iced::window::Id) {
        use windows::Win32::UI::WindowsAndMessaging::{SetWindowLongPtrW, GWLP_WNDPROC};
        let mut reg = registry().lock().unwrap();
        if let Some(entry) = reg.get_mut(&hwnd) {
            entry.window = window;
            return;
        }
        let prev = unsafe { SetWindowLongPtrW(
            windows::Win32::Foundation::HWND(hwnd as *mut _),
            GWLP_WNDPROC,
            ime_wndproc as usize as isize,
        ) };
        reg.insert(hwnd, Entry { prev, window, uplinks: Vec::new() });
    }

    /// 摘桥（disable/窗回收路径调用）。恢复原 wndproc。
    pub fn detach(hwnd: isize) {
        use windows::Win32::UI::WindowsAndMessaging::{SetWindowLongPtrW, GWLP_WNDPROC};
        if let Some(entry) = registry().lock().unwrap().remove(&hwnd) {
            unsafe {
                SetWindowLongPtrW(
                    windows::Win32::Foundation::HWND(hwnd as *mut _),
                    GWLP_WNDPROC,
                    entry.prev,
                );
            }
        }
    }

    /// 上行排水（daemon Tick 泵调用，事件循环线程）。
    pub fn drain(hwnd: isize) -> Vec<ImeUplink> {
        registry()
            .lock()
            .unwrap()
            .get_mut(&hwnd)
            .map(|e| std::mem::take(&mut e.uplinks))
            .unwrap_or_default()
    }
}

/// IME 上行排水（daemon Tick 泵——返回各窗事件批）。
#[cfg(windows)]
pub fn drain_uplinks() -> Vec<ImeUplink> {
    let hwnds: Vec<isize> = bridge::registry().lock().unwrap().keys().copied().collect();
    let mut out = Vec::new();
    for hwnd in hwnds {
        out.extend(bridge::drain(hwnd));
    }
    out
}

#[cfg(not(windows))]
pub fn drain_uplinks() -> Vec<ImeUplink> {
    Vec::new()
}

/// IME enable：激活上下文 + 组合窗/候选窗定位 + 子类桥挂载（daemon 窗
/// 事件循环线程调用——`window::run` 闭包保证线程正确性，winit
/// `execute_in_thread` 同则）。`window` = 桥上行的 client 路由键。
pub fn enable(
    window_handle: &dyn iced_runtime::window::Window,
    req: &ImeReq,
    scale: f64,
    tag: &str,
    window: iced::window::Id,
) {
    eprintln!(
        "[rqhost] ime{tag} enable cursor=({:.0},{:.0} {:.0}x{:.0}) purpose={}",
        req.cursor.x, req.cursor.y, req.cursor.w, req.cursor.h, req.purpose
    );
    apply_cursor_area(window_handle, Some(req), scale, Some(window));
}

/// IME disable：失焦关上下文 + 摘桥（App 侧 `InputMethod::Disabled` 下行）。
pub fn disable(window_handle: &dyn iced_runtime::window::Window, tag: &str) {
    eprintln!("[rqhost] ime{tag} disable");
    apply_cursor_area(window_handle, None, 1.0, None);
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
    window_handle: &dyn iced_runtime::window::Window,
    req: Option<&ImeReq>,
    scale: f64,
    window: Option<iced::window::Id>,
) {
    use windows::Win32::Foundation::{POINT, RECT};
    use windows::Win32::UI::Input::Ime::{
        ImmAssociateContextEx, ImmGetContext, ImmReleaseContext, ImmSetCandidateWindow,
        ImmSetCompositionWindow, CANDIDATEFORM, COMPOSITIONFORM, CFS_EXCLUDE, CFS_POINT,
        IACE_CHILDREN, IACE_DEFAULT,
    };

    let Some(hwnd) = hwnd_of(window_handle) else {
        eprintln!("[rqhost] ime 非 Win32 句柄——跳过");
        return;
    };
    let hwnd_addr = hwnd.0 as isize;
    unsafe {
        // 上下文开关（winit ImeContext::set_ime_allowed 同语义：启用 =
        // 关联缺省上下文，禁用 = 解除关联）。
        match (req, window) {
            (Some(_), Some(window)) => {
                let _ = ImmAssociateContextEx(hwnd, None, IACE_DEFAULT);
                bridge::attach(hwnd_addr, window);
            }
            _ => {
                let _ = ImmAssociateContextEx(hwnd, None, IACE_CHILDREN);
                bridge::detach(hwnd_addr);
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
    _window_handle: &dyn iced_runtime::window::Window,
    _req: Option<&ImeReq>,
    _scale: f64,
    _window: Option<iced::window::Id>,
) {
    // 非 Windows daemon：观测行已由 enable/disable 打出，实际 IME 门控
    // v1 不展开（平台面归 SD-01 边界注记）。
}
