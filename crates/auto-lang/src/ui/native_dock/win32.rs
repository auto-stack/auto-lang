//! Win32 适配层（Plan 473）：发现 / 几何 / 样式 / 层级 / 显示态。
//! （WinEventHook 事件层见本文件 `events` 段。）
//!
//! `windows` crate 调用只允许出现在本文件与 `tools/native-fixture/`；
//! 其余模块一律经由 `native_dock`（mod.rs）的纯逻辑层与本层的薄封装。
//! 非 Windows / 未启用 `native-dock` feature 时，`win32_noop.rs` 提供同名
//! no-op API，宿主层无需 cfg 包裹。
//!
//! 版本注记：windows 0.58 的 `HWND` 为 `*mut c_void`（非 isize），
//! 与 [`NativeHwnd`] 的 isize 存储形态在本文件边界互转。

use crate::ui::native_dock::{NativeHwnd, Rect};
use windows::core::HRESULT;
use windows::Win32::Foundation::{ERROR_ACCESS_DENIED, BOOL, HWND, LPARAM, RECT, WPARAM};
use windows::Win32::Graphics::Dwm::{
    DwmGetWindowAttribute, DwmSetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS,
    DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_DONOTROUND,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowLongPtrW, GetWindowRect, GetWindowTextW, GetWindowThreadProcessId,
    IsIconic, IsWindow, IsWindowVisible, IsZoomed, PostMessageW, SetWindowLongPtrW, SetWindowPos,
    ShowWindow, GWL_STYLE, SW_HIDE, SW_MINIMIZE, SW_RESTORE, SWP_FRAMECHANGED, SWP_NOACTIVATE,
    SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, WM_CLOSE, WS_CAPTION, WS_THICKFRAME,
};

/// Win32 dock 操作错误（UIPI 拒绝单独分类，供 shell 层提示）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockError {
    /// 目标提权（ERROR_ACCESS_DENIED）——UIPI 防御路径（用例 C1）。
    Elevated,
    /// 句柄已失效（目标已销毁）。
    StaleHwnd,
    /// 其他 Win32 失败（op = 操作名，code = HRESULT 位型，仅诊断展示）。
    Api { op: &'static str, code: u32 },
}

/// WIN32_ERROR → HRESULT（FACILITY_WIN32 形态 0x8007xxxx）。
fn hresult_of_win32(code: u32) -> HRESULT {
    HRESULT(((code & 0xFFFF) | 0x8007_0000) as i32)
}

impl DockError {
    fn from_err(op: &'static str, err: windows::core::Error) -> Self {
        if err.code() == hresult_of_win32(ERROR_ACCESS_DENIED.0) {
            DockError::Elevated
        } else {
            DockError::Api {
                op,
                code: err.code().0 as u32,
            }
        }
    }
}

fn hwnd_of(h: NativeHwnd) -> HWND {
    HWND(h.0 as *mut core::ffi::c_void)
}

fn hwnd_value(h: HWND) -> isize {
    h.0 as isize
}

fn rect_of_win(r: RECT) -> Rect {
    Rect::new(r.left, r.top, r.right - r.left, r.bottom - r.top)
}

fn alive(h: NativeHwnd) -> bool {
    unsafe { IsWindow(hwnd_of(h)) }.as_bool()
}

// ---------------------------------------------------------------------------
// 发现（EnumWindows + PID）
// ---------------------------------------------------------------------------

struct EnumCtx<'a> {
    f: &'a mut dyn FnMut(HWND) -> bool,
}

unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let ctx = &mut *(lparam.0 as *mut EnumCtx<'_>);
    (ctx.f)(hwnd).into()
}

fn enum_top_level(mut f: impl FnMut(HWND) -> bool) -> windows::core::Result<()> {
    let mut ctx = EnumCtx { f: &mut f };
    unsafe { EnumWindows(Some(enum_proc), LPARAM(&mut ctx as *mut EnumCtx as isize)) }
}

fn window_pid(hwnd: HWND) -> u32 {
    let mut pid = 0u32;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
    pid
}

/// 按 pid 发现可见顶层窗口（z 序自上而下第一个命中）。
pub fn find_top_level_by_pid(pid: u32) -> Option<NativeHwnd> {
    let mut hit = None;
    let _ = enum_top_level(|hwnd| {
        if window_pid(hwnd) == pid && unsafe { IsWindowVisible(hwnd) }.as_bool() {
            hit = Some(hwnd);
            return false; // 停止枚举
        }
        true
    });
    hit.map(|h| NativeHwnd(hwnd_value(h)))
}

/// 枚举全部可见、有标题的顶层窗口（shell 选择列表用）：`(hwnd, pid, title)`。
pub fn list_top_level_windows() -> Vec<(NativeHwnd, u32, String)> {
    let mut out = Vec::new();
    let _ = enum_top_level(|hwnd| {
        if !unsafe { IsWindowVisible(hwnd) }.as_bool() {
            return true;
        }
        let title = title_of(hwnd);
        if title.is_empty() {
            return true;
        }
        out.push((NativeHwnd(hwnd_value(hwnd)), window_pid(hwnd), title));
        true
    });
    out
}

/// 读回窗口 pid（session 侧核对用）。
pub fn pid_of(target: NativeHwnd) -> Option<u32> {
    let hwnd = hwnd_of(target);
    if !alive(target) {
        return None;
    }
    Some(window_pid(hwnd))
}

// ---------------------------------------------------------------------------
// 标题
// ---------------------------------------------------------------------------

fn title_of(hwnd: HWND) -> String {
    let mut buf = [0u16; 512];
    let len = unsafe { GetWindowTextW(hwnd, &mut buf) };
    let len = len.max(0) as usize;
    String::from_utf16_lossy(&buf[..len])
}

/// 刷新标题缓存用：读回目标窗口当前标题。
pub fn get_title(target: NativeHwnd) -> Option<String> {
    if !alive(target) {
        return None;
    }
    Some(title_of(hwnd_of(target)))
}

// ---------------------------------------------------------------------------
// 几何（SetWindowPos / DwmGetWindowAttribute 写读回）
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// 几何（SetWindowPos / DwmGetWindowAttribute 写读回）
// ---------------------------------------------------------------------------

/// 把目标窗口摆到 `rect`（屏幕物理坐标；仅几何，不动 z 序、不抢激活）。
/// UIPI 拒绝映射为 [`DockError::Elevated`]。
///
/// 注（Win32 语义勘误，实测确立）：`SetWindowPos` 的 `hWndInsertAfter`
/// 是"被定位窗口正上方的参照窗口"——`SetWindowPos(slot, desktop)` 会把
/// slot 沉到桌面**下方**。故 z 序不变量（slot 紧贴桌面上方）由
/// [`sink_desktop_below`] 以 `SetWindowPos(desktop, slot)` 达成，
/// 本函数只管几何。
pub fn set_bounds(target: NativeHwnd, rect: Rect) -> Result<(), DockError> {
    if !alive(target) {
        return Err(DockError::StaleHwnd);
    }
    unsafe {
        SetWindowPos(
            hwnd_of(target),
            HWND::default(),
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            SWP_NOACTIVATE | SWP_NOZORDER,
        )
    }
    .map_err(|e| DockError::from_err("SetWindowPos", e))
}

/// 读回当前矩形：优先 DWM 扩展帧边界（可视内容，排除不可见 resize 边框），
/// DWM 不可用则退回 `GetWindowRect`。窗口已销毁 → None。
pub fn get_bounds(target: NativeHwnd) -> Option<Rect> {
    if !alive(target) {
        return None;
    }
    let hwnd = hwnd_of(target);
    unsafe {
        let mut r = RECT::default();
        let dwm_ok = DwmGetWindowAttribute(
            hwnd,
            DWMWA_EXTENDED_FRAME_BOUNDS,
            &mut r as *mut RECT as *mut core::ffi::c_void,
            core::mem::size_of::<RECT>() as u32,
        )
        .is_ok();
        if dwm_ok {
            return Some(rect_of_win(r));
        }
        let mut r = RECT::default();
        if GetWindowRect(hwnd, &mut r).is_ok() {
            return Some(rect_of_win(r));
        }
    }
    None
}

/// 写读回探测：请求设为 `requested` 并读回实际矩形（不可信窗口的
/// min-size 探测手段；配合 [`crate::ui::native_dock::observe_min_size_estimate`]
/// 缓存估计值）。
pub fn probe_bounds(target: NativeHwnd, requested: Rect) -> Option<Rect> {
    set_bounds(target, requested).ok()?;
    get_bounds(target)
}

// ---------------------------------------------------------------------------
// 样式（GWL_STYLE 剥离/还原 + SWP_FRAMECHANGED）
// ---------------------------------------------------------------------------

/// 剥离原生标题栏与可调边框（`WS_CAPTION | WS_THICKFRAME`），
/// 返回 pre-dock 样式位（宿主存入 `NativeSlot::pre_dock_style`）。
pub fn strip_chrome(target: NativeHwnd) -> Result<u32, DockError> {
    if !alive(target) {
        return Err(DockError::StaleHwnd);
    }
    let hwnd = hwnd_of(target);
    let old = (unsafe { GetWindowLongPtrW(hwnd, GWL_STYLE) }) as u32;
    let stripped = old & !(WS_CAPTION.0 | WS_THICKFRAME.0);
    unsafe { SetWindowLongPtrW(hwnd, GWL_STYLE, stripped as isize) };
    apply_frame_change(target)?;
    Ok(old)
}

/// 还原 pre-dock 样式（undock 路径；`saved` 来自 [`strip_chrome`] 返回值）。
pub fn restore_chrome(target: NativeHwnd, saved: u32) -> Result<(), DockError> {
    if !alive(target) {
        return Err(DockError::StaleHwnd);
    }
    let hwnd = hwnd_of(target);
    unsafe { SetWindowLongPtrW(hwnd, GWL_STYLE, saved as isize) };
    apply_frame_change(target)
}

fn apply_frame_change(target: NativeHwnd) -> Result<(), DockError> {
    unsafe {
        SetWindowPos(
            hwnd_of(target),
            HWND::default(),
            0,
            0,
            0,
            0,
            SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
        )
    }
    .map_err(|e| DockError::from_err("SetWindowPos(FRAMECHANGED)", e))
}

/// Win11 直角偏好（`DWMWCP_DONOTROUND`）；Win10 无此属性 → false
/// （静默降级：直角诉求退化为保留系统圆角，不致命）。
pub fn set_square_corners(target: NativeHwnd) -> bool {
    unsafe {
        DwmSetWindowAttribute(
            hwnd_of(target),
            DWMWA_WINDOW_CORNER_PREFERENCE,
            &DWMWCP_DONOTROUND as *const _ as *const core::ffi::c_void,
            core::mem::size_of_val(&DWMWCP_DONOTROUND) as u32,
        )
    }
    .is_ok()
}

// ---------------------------------------------------------------------------
// 层级（z 序不变量：docked 窗口紧贴桌面正上方）
// ---------------------------------------------------------------------------

/// 重申 z 序不变量：把桌面窗口沉到 `slot` 正下方（⟺ slot 紧贴桌面正上方）。
/// 每次 relayout 后宿主层按 WM 布局顺序对每个 slot 重申；
/// 多 slot 的层间次序由调用顺序决定（自下而上处理，最后处理者最顶）。
/// 不动几何、不抢激活。
pub fn sink_desktop_below(desktop: NativeHwnd, slot: NativeHwnd) -> Result<(), DockError> {
    if !alive(desktop) {
        return Err(DockError::StaleHwnd);
    }
    if !alive(slot) {
        return Err(DockError::StaleHwnd);
    }
    unsafe {
        SetWindowPos(
            hwnd_of(desktop),
            hwnd_of(slot),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        )
    }
    .map_err(|e| DockError::from_err("SetWindowPos(sink desktop)", e))
}

// ---------------------------------------------------------------------------
// 显示态（ShowWindow / WM_CLOSE）
// ---------------------------------------------------------------------------

/// 显示态切换（dock 前 restore 已最大化窗口；chrome 按钮驱动最小化/隐藏）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShowMode {
    Restore,
    Minimize,
    Hide,
}

pub fn show_window(target: NativeHwnd, mode: ShowMode) -> Result<(), DockError> {
    if !alive(target) {
        return Err(DockError::StaleHwnd);
    }
    let cmd = match mode {
        ShowMode::Restore => SW_RESTORE,
        ShowMode::Minimize => SW_MINIMIZE,
        ShowMode::Hide => SW_HIDE,
    };
    let _ = unsafe { ShowWindow(hwnd_of(target), cmd) };
    Ok(())
}

/// 请求关闭：`PostMessageW(WM_CLOSE)`——给目标 app 正常关闭机会
/// （弹确认框/保存提示），不直接 `DestroyWindow`。
pub fn request_close(target: NativeHwnd) -> Result<(), DockError> {
    if !alive(target) {
        return Err(DockError::StaleHwnd);
    }
    unsafe { PostMessageW(hwnd_of(target), WM_CLOSE, WPARAM(0), LPARAM(0)) }
        .map_err(|e| DockError::from_err("PostMessageW(WM_CLOSE)", e))
}

/// 目标窗口是否仍然存活（IsWindow；WinEvent DESTROY 的兜底核对）。
pub fn is_alive(target: NativeHwnd) -> bool {
    alive(target)
}

/// 是否最小化（IsIconic）。
pub fn is_minimized(target: NativeHwnd) -> bool {
    alive(target) && unsafe { IsIconic(hwnd_of(target)) }.as_bool()
}

/// 是否最大化（IsZoomed；C5：已最大化窗口 dock 前先 SW_RESTORE）。
pub fn is_maximized(target: NativeHwnd) -> bool {
    alive(target) && unsafe { IsZoomed(hwnd_of(target)) }.as_bool()
}

// ---------------------------------------------------------------------------
// T2：Win32 几何集成测试——全部针对本进程 scratch 窗口，无第三方依赖
// ---------------------------------------------------------------------------

#[cfg(all(test, windows, feature = "test-native-dock"))]
mod native_dock_geometry {
    use super::*;
    use crate::ui::native_dock::Size;
    use std::sync::OnceLock;
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::{HINSTANCE, LRESULT};
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::UI::HiDpi::{
        SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
    };
    use windows::Win32::UI::WindowsAndMessaging::HMENU;
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetWindow,
        PeekMessageW, RegisterClassW, TranslateMessage, CW_USEDEFAULT, GW_HWNDPREV, MSG,
        PM_REMOVE, WINDOW_EX_STYLE, WNDCLASSW, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
    };

    /// 本进程 scratch 窗口（DefWindowProc；Drop 时 DestroyWindow）。
    struct Scratch(HWND);

    impl Drop for Scratch {
        fn drop(&mut self) {
            unsafe {
                let _ = DestroyWindow(self.0);
            }
        }
    }

    fn class_name() -> &'static [u16] {
        static NAME: OnceLock<Vec<u16>> = OnceLock::new();
        NAME.get_or_init(|| "auto_lang_native_dock_scratch\0".encode_utf16().collect())
    }

    /// 0.58 的 `DefWindowProcW` 是 Rust-abi 安全包装，不能直接作 WNDPROC；
    /// 经 extern "system" 蹦床转接。
    unsafe extern "system" fn scratch_wndproc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
    }

    fn ensure_class() {
        static DONE: OnceLock<()> = OnceLock::new();
        DONE.get_or_init(|| unsafe {
            // 与生产 iced/winit 进程（per-monitor aware）对齐：关闭 DPI 虚拟化，
            // 使 SetWindowPos 写入与 DWM/GetWindowRect 读回同一物理像素坐标域。
            let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
            let hmodule = GetModuleHandleW(None).expect("GetModuleHandleW");
            let wc = WNDCLASSW {
                lpfnWndProc: Some(scratch_wndproc),
                hInstance: HINSTANCE(hmodule.0),
                lpszClassName: PCWSTR(class_name().as_ptr()),
                ..Default::default()
            };
            assert_ne!(RegisterClassW(&wc), 0, "RegisterClassW failed");
        });
    }

    fn scratch(title: &str) -> Scratch {
        ensure_class();
        let mut title_w: Vec<u16> = title.encode_utf16().collect();
        title_w.push(0);
        let hmodule = unsafe { GetModuleHandleW(None) }.expect("GetModuleHandleW");
        let hwnd = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                PCWSTR(class_name().as_ptr()),
                PCWSTR(title_w.as_ptr()),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                480,
                320,
                HWND::default(),
                HMENU::default(),
                hmodule,
                None,
            )
        }
        .expect("CreateWindowExW failed");
        Scratch(hwnd)
    }

    fn pump_one(hwnd: HWND) -> bool {
        unsafe {
            let mut msg = MSG::default();
            if PeekMessageW(&mut msg, hwnd, 0, 0, PM_REMOVE).as_bool() {
                let _ = TranslateMessage(&msg);
                let _ = DispatchMessageW(&msg);
                true
            } else {
                false
            }
        }
    }

    fn style_of(hwnd: HWND) -> u32 {
        (unsafe { GetWindowLongPtrW(hwnd, GWL_STYLE) }) as u32
    }

    fn close_enough(got: Rect, want: Rect, tol: i32) -> bool {
        (got.x - want.x).abs() <= tol
            && (got.y - want.y).abs() <= tol
            && (got.w - want.w).abs() <= tol
            && (got.h - want.h).abs() <= tol
    }

    #[test]
    fn set_bounds_write_read_back_matches() {
        let s = scratch("geom");
        let h = NativeHwnd(hwnd_value(s.0));
        // 先剥离边框：DWM 扩展帧边界与窗口矩形重合，写读回可精确对账
        strip_chrome(h).expect("strip");
        let rect = Rect::new(120, 90, 640, 480);
        set_bounds(h, rect).expect("set_bounds");
        let got = get_bounds(h).expect("read back");
        assert!(close_enough(got, rect, 2), "read back {got:?} vs {rect:?}");
    }

    #[test]
    fn min_size_probe_reports_clamp() {
        // 带 caption 的 scratch：系统默认 min-track 尺寸生效（无边框窗口无下限）
        let s = scratch("minsize");
        let h = NativeHwnd(hwnd_value(s.0));
        // 请求极小尺寸 → min-track 生效 → 读回大于请求 → 产生估计
        let tiny =
            probe_bounds(h, Rect::new(50, 50, 40, 30)).expect("probe tiny");
        assert!(tiny.w > 40 || tiny.h > 30, "expected clamp, got {tiny:?}");
        assert_eq!(
            crate::ui::native_dock::observe_min_size_estimate(Size::new(40, 30), tiny.size()),
            Some(tiny.size())
        );
        // 正常尺寸：读回（DWM 可视边界，比窗口矩形小 ~10px 边框内缩）不大于请求
        // → 无新估计
        let ok = probe_bounds(h, Rect::new(50, 50, 500, 400))
            .expect("probe ok");
        assert_eq!(
            crate::ui::native_dock::observe_min_size_estimate(Size::new(500, 400), ok.size()),
            None
        );
    }

    #[test]
    fn strip_and_restore_chrome_roundtrip() {
        let s = scratch("chrome");
        let h = NativeHwnd(hwnd_value(s.0));
        let before = style_of(s.0);
        assert_ne!(before & WS_CAPTION.0, 0, "scratch 应带标题栏");
        let saved = strip_chrome(h).expect("strip");
        assert_eq!(saved, before);
        let after = style_of(s.0);
        assert_eq!(
            after & (WS_CAPTION.0 | WS_THICKFRAME.0),
            0,
            "剥离后不应残留 caption/thickframe"
        );
        restore_chrome(h, saved).expect("restore");
        assert_eq!(style_of(s.0), before);
    }

    #[test]
    fn z_order_stays_above_desktop_stand_in() {
        let desktop = scratch("desktop-stand-in");
        let target = scratch("target");
        let target_h = NativeHwnd(hwnd_value(target.0));
        let desktop_h = NativeHwnd(hwnd_value(desktop.0));
        // 并行测试共享同一桌面 z 序：校验前重申不变量，容忍瞬时扰动。
        // 不变量 = slot 紧贴桌面正上方（prev(desktop) == slot）；
        // 达成方式 = sink_desktop_below(desktop, slot)。
        let mut ok = false;
        for _ in 0..20 {
            sink_desktop_below(desktop_h, target_h).expect("sink desktop below slot");
            if unsafe { GetWindow(desktop.0, GW_HWNDPREV) }.expect("GetWindow") == target.0 {
                ok = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(
            ok,
            "sink_desktop_below 后 slot 应紧贴桌面正上方（重试 20 次仍被扰动）"
        );
    }

    #[test]
    fn square_corners_smoke() {
        let s = scratch("corners");
        // Win11 上应为 true；Win10 无此属性静默降级 false，不致命
        let _ = set_square_corners(NativeHwnd(hwnd_value(s.0)));
    }

    #[test]
    fn show_minimize_restore_roundtrip() {
        let s = scratch("show");
        let h = NativeHwnd(hwnd_value(s.0));
        show_window(h, ShowMode::Minimize).expect("minimize");
        assert!(is_minimized(h));
        show_window(h, ShowMode::Restore).expect("restore");
        assert!(!is_minimized(h));
        assert!(!is_maximized(h));
    }

    #[test]
    fn request_close_destroys_scratch() {
        let s = scratch("close-me");
        request_close(NativeHwnd(hwnd_value(s.0))).expect("request_close");
        for _ in 0..100 {
            if !is_alive(NativeHwnd(hwnd_value(s.0))) {
                break;
            }
            pump_one(s.0);
        }
        assert!(
            !is_alive(NativeHwnd(hwnd_value(s.0))),
            "WM_CLOSE 应让 DefWindowProc 销毁窗口"
        );
    }

    #[test]
    fn find_by_pid_finds_own_scratch() {
        let s = scratch("find-me");
        let _ = set_bounds(NativeHwnd(hwnd_value(s.0)), Rect::new(60, 60, 320, 240));
        // 并行测试共享进程：同 pid 可能同时存在多个测试窗口，
        // 故断言"集合命中"而非"最顶命中"（后者属调度巧合）。
        let same_pid: Vec<(NativeHwnd, u32, String)> = list_top_level_windows()
            .into_iter()
            .filter(|(_, pid, _)| *pid == std::process::id())
            .collect();
        assert!(
            same_pid
                .iter()
                .any(|(h, _, t)| h.0 == hwnd_value(s.0) && t == "find-me"),
            "可见+有标题+pid 过滤应命中本进程 scratch 窗口"
        );
        let found = find_top_level_by_pid(std::process::id()).expect("存在同进程可见顶层窗口");
        assert!(
            same_pid.iter().any(|(h, ..)| h.0 == found.0),
            "find 结果必须属于本进程可见顶层窗口集合"
        );
        assert_eq!(pid_of(NativeHwnd(hwnd_value(s.0))), Some(std::process::id()));
    }

    #[test]
    fn stale_hwnd_maps_to_stale_error() {
        let h = NativeHwnd(0x0000_DEAD_BEEF as isize);
        assert_eq!(
            set_bounds(h, Rect::new(0, 0, 10, 10)),
            Err(DockError::StaleHwnd)
        );
        assert!(!is_alive(h));
        assert_eq!(get_bounds(h), None);
    }

    #[test]
    fn list_top_level_includes_scratch_title() {
        let s = scratch("native-dock-listing-probe");
        let listed = list_top_level_windows();
        assert!(listed
            .iter()
            .any(|(h, _, t)| h.0 == hwnd_value(s.0) && t == "native-dock-listing-probe"));
    }
}
