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

use crate::ui::native_dock::{NativeHwnd, NativeSlotEvent, NativeSlotEventKind, Rect};
use std::cell::Cell;
use std::sync::mpsc;
use std::sync::RwLock;
use windows::core::HRESULT;
use windows::Win32::Foundation::{
    ERROR_ACCESS_DENIED, BOOL, HMODULE, HWND, LPARAM, POINT, RECT, WPARAM,
};
use windows::Win32::Graphics::Dwm::{
    DwmGetWindowAttribute, DwmSetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS,
    DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_DEFAULT, DWMWCP_DONOTROUND,
    DWM_WINDOW_CORNER_PREFERENCE,
};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK};
use windows::Win32::UI::HiDpi::{
    GetDpiForWindow, SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, EnumWindows, EVENT_OBJECT_DESTROY, EVENT_OBJECT_LOCATIONCHANGE,
    EVENT_SYSTEM_MINIMIZEEND, EVENT_SYSTEM_MINIMIZESTART, EVENT_SYSTEM_MOVESIZEEND,
    EVENT_SYSTEM_MOVESIZESTART, GetCursorPos, GetMessageW, GetWindowLongPtrW, GetWindowRect,
    GetWindowTextW, GetWindowThreadProcessId, IsIconic, IsWindow, IsWindowVisible, IsZoomed, MSG,
    PostMessageW, PostThreadMessageW, SetForegroundWindow, SetWindowLongPtrW, SetWindowPos,
    ShowWindow, SW_MAXIMIZE, TranslateMessage, GWL_STYLE, SW_HIDE, SW_MINIMIZE, SW_RESTORE,
    SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, WM_CLOSE, WM_QUIT,
    WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS, WS_CAPTION, WS_THICKFRAME,
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

/// 读回原始窗口矩形（`GetWindowRect` 域；含不可见边框）。pre-dock
/// bounds 捕获必须用本函数——恢复走 `set_bounds`（同为窗口矩形域），
/// 用 DWM 可视边界捕获会对带边框窗口产生 ~10px 级恢复偏差（T8 实测）。
pub fn get_bounds_window(target: NativeHwnd) -> Option<Rect> {
    if !alive(target) {
        return None;
    }
    let mut r = RECT::default();
    unsafe { GetWindowRect(hwnd_of(target), &mut r) }
        .ok()
        .map(|_| rect_of_win(r))
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
    set_corner_preference(target, DWMWCP_DONOTROUND)
}

/// undock 恢复系统默认圆角（与 [`set_square_corners`] 成对）。
pub fn restore_corner_preference(target: NativeHwnd) -> bool {
    set_corner_preference(target, DWMWCP_DEFAULT)
}

/// Win11 圆角偏好通用写入口（Win10 无此属性 → false，静默降级）。
pub fn set_corner_preference(target: NativeHwnd, pref: DWM_WINDOW_CORNER_PREFERENCE) -> bool {
    unsafe {
        DwmSetWindowAttribute(
            hwnd_of(target),
            DWMWA_WINDOW_CORNER_PREFERENCE,
            &pref as *const _ as *const core::ffi::c_void,
            core::mem::size_of_val(&pref) as u32,
        )
    }
    .is_ok()
}

/// 声明本进程 per-monitor v2 DPI 感知（测试/嵌入宿主用；生产 iced/winit
/// 进程自带声明，重复声明无害返回 false）。声明后 SetWindowPos /
/// GetWindowRect / DWM 读回统一物理像素坐标域（虚拟化关闭）。
pub fn set_process_dpi_aware_per_monitor_v2() -> bool {
    unsafe { SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) }.is_ok()
}

/// 读回目标窗口的 DPI 缩放比（`GetDpiForWindow / 96`；失败按 1.0）。
/// 宿主层局部逻辑坐标 → 屏幕物理坐标换算的缩放源（与 winit 一致）。
pub fn dpi_scale_of(target: NativeHwnd) -> f64 {
    if !alive(target) {
        return 1.0;
    }
    (unsafe { GetDpiForWindow(hwnd_of(target)) }) as f64 / 96.0
}

/// 发现本进程最大的可见顶层窗口（全屏壳拓扑下即桌面 OS 窗口；与
/// `vm/native.rs` 的 main_hwnd 同型启发式，进程级缓存一次——桌面为
/// 一次启动单实例拓扑）。
pub fn find_largest_own_window() -> Option<NativeHwnd> {
    static CACHE: std::sync::OnceLock<Option<NativeHwnd>> = std::sync::OnceLock::new();
    CACHE
        .get_or_init(|| {
            let me = std::process::id();
            let mut best: Option<(NativeHwnd, i64)> = None;
            let _ = enum_top_level(|hwnd| {
                if window_pid(hwnd) != me || !unsafe { IsWindowVisible(hwnd) }.as_bool() {
                    return true;
                }
                let mut r = RECT::default();
                if unsafe { GetWindowRect(hwnd, &mut r) }.is_ok() {
                    let area = (r.right - r.left) as i64 * (r.bottom - r.top) as i64;
                    if best.map_or(true, |(_, a)| area > a) {
                        best = Some((NativeHwnd(hwnd_value(hwnd)), area));
                    }
                }
                true
            });
            best.map(|(h, _)| h)
        })
        .clone()
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
// Plan 494：真洞（z 序翻转 + SetWindowRgn 洞排除）
// ---------------------------------------------------------------------------

/// 真洞 z 序不变量：把 `slot` 沉到桌面窗口正下方（⟺ 桌面紧贴 slot 正上方
/// ——473 假洞"slot 盖桌面"的翻转）。与 [`sink_desktop_below`] 参数对调、
/// 单步即达（`SetWindowPos(slot, desktop)`）；relayout 重申幂等。
/// 不动几何、不抢激活。
pub fn raise_desktop_above(desktop: NativeHwnd, slot: NativeHwnd) -> Result<(), DockError> {
    if !alive(desktop) {
        return Err(DockError::StaleHwnd);
    }
    if !alive(slot) {
        return Err(DockError::StaleHwnd);
    }
    unsafe {
        SetWindowPos(
            hwnd_of(slot),
            hwnd_of(desktop),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        )
    }
    .map_err(|e| DockError::from_err("SetWindowPos(raise desktop)", e))
}

/// 目标窗口客户区原点的屏幕物理坐标（`GetClientRect` + `ClientToScreen`
/// ——跨进程客户区坐标换算的断言基准，494 T3 E2E 用）。
pub fn client_origin(target: NativeHwnd) -> Option<(i32, i32)> {
    if !alive(target) {
        return None;
    }
    unsafe {
        let hwnd = hwnd_of(target);
        let mut rc = RECT::default();
        windows::Win32::UI::WindowsAndMessaging::GetClientRect(hwnd, &mut rc).ok()?;
        let mut pt = POINT { x: rc.left, y: rc.top };
        windows::Win32::Graphics::Gdi::ClientToScreen(hwnd, &mut pt)
            .as_bool()
            .then_some((pt.x, pt.y))
    }
}

/// 真洞 Region 排除：把 `holes`（屏幕物理矩形）从目标窗口的形状中扣除
/// （`SetWindowRgn` 窗口局部域，经 [`crate::ui::native_dock::window_local_holes`]
/// 裁剪换算）。洞区内窗口不存在——视觉透出 z 序下层 + 点击直达（OS 区域
/// 语义，无同线程限制）。`holes` 为空 → 复位全窗。
/// 成功后 Region 归系统所有（不 DeleteObject）；失败自清理。
/// 注：`SetWindowRgn(hwnd, None, false)` 不触发重绘（relayout 高频重申无闪）。
pub fn apply_hole_regions(
    target: NativeHwnd,
    win: Rect,
    holes: &[Rect],
) -> Result<(), DockError> {
    use windows::Win32::Graphics::Gdi::{
        CombineRgn, CreateRectRgn, DeleteObject, SetWindowRgn as GdiSetWindowRgn, HGDIOBJ,
        HRGN, RGN_DIFF,
    };
    if !alive(target) {
        return Err(DockError::StaleHwnd);
    }
    let hwnd = hwnd_of(target);
    unsafe {
        if holes.is_empty() {
            // None=0 复位为全窗口形状（undock/桌面退出清理路径）。
            let r = GdiSetWindowRgn(hwnd, HRGN(std::ptr::null_mut()), false);
            if r == -1 {
                return Err(DockError::Api {
                    op: "SetWindowRgn(reset)",
                    code: 0,
                });
            }
            return Ok(());
        }
        let full = CreateRectRgn(0, 0, win.w, win.h);
        if full.is_invalid() {
            return Err(DockError::Api {
                op: "CreateRectRgn(full)",
                code: 0,
            });
        }
        let mut combined = full;
        for local in crate::ui::native_dock::window_local_holes(win, holes) {
            let hole = CreateRectRgn(local.x, local.y, local.right(), local.bottom());
            if hole.is_invalid() {
                let _ = DeleteObject(HGDIOBJ(combined.0));
                return Err(DockError::Api {
                    op: "CreateRectRgn(hole)",
                    code: 0,
                });
            }
            let out = CreateRectRgn(0, 0, 0, 0);
            let code = CombineRgn(out, combined, hole, RGN_DIFF);
            // out = combined - hole；释放旧 combined 与 hole，保留 out。
            let _ = DeleteObject(HGDIOBJ(combined.0));
            let _ = DeleteObject(HGDIOBJ(hole.0));
            if code == windows::Win32::Graphics::Gdi::GDI_REGION_TYPE(0) {
                let _ = DeleteObject(HGDIOBJ(out.0));
                return Err(DockError::Api {
                    op: "CombineRgn(RGN_DIFF)",
                    code: 0,
                });
            }
            combined = out;
        }
        let r = GdiSetWindowRgn(hwnd, combined, false);
        if r == -1 {
            let _ = DeleteObject(HGDIOBJ(combined.0));
            return Err(DockError::Api {
                op: "SetWindowRgn(carve)",
                code: 0,
            });
        }
        // 成功：Region 所有权移交系统。
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 显示态（ShowWindow / WM_CLOSE）
// ---------------------------------------------------------------------------

/// 显示态切换（dock 前 restore 已最大化窗口；chrome 按钮驱动最小化/隐藏）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShowMode {
    Restore,
    Maximize,
    Minimize,
    Hide,
}

pub fn show_window(target: NativeHwnd, mode: ShowMode) -> Result<(), DockError> {
    if !alive(target) {
        return Err(DockError::StaleHwnd);
    }
    let cmd = match mode {
        ShowMode::Restore => SW_RESTORE,
        ShowMode::Maximize => SW_MAXIMIZE,
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

/// 前台聚焦（Plan 486 任务栏 focus_native）：`SetForegroundWindow`
/// best-effort——后台进程前台锁等限制下返回 false（调用方按"尽力"语义
/// 处理，不视为错误路径）。最小化还原由调用方先行 `SW_RESTORE`。
pub fn focus_window(target: NativeHwnd) -> bool {
    alive(target) && unsafe { SetForegroundWindow(hwnd_of(target)) }.as_bool()
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

/// C2：目标窗口矩形是否完整覆盖参照窗口所在屏域（独占全屏判据；
/// 参照取桌面 OS 窗口）。独占全屏（游戏/视频墙）收编会撕裂显示，
/// dock 侧据此拒绝（RejectReason::ExclusiveFullscreen）。
pub fn covers_rect(target: NativeHwnd, reference: NativeHwnd) -> bool {
    let (Some(tr), Some(rr)) = (get_bounds(target), get_bounds(reference)) else {
        return false;
    };
    tr.x <= rr.x && tr.y <= rr.y && tr.right() >= rr.right() && tr.bottom() >= rr.bottom()
}

// ---------------------------------------------------------------------------
// Plan 515 D1 —— HICON 真图标提取（native 槽位条目的真窗口图标）
// ---------------------------------------------------------------------------

/// native 窗口图标 → RGBA（top-down，straight alpha）。
///
/// 三级回退：`WM_GETICON(ICON_SMALL2)` → `WM_GETICON(ICON_SMALL)` →
/// `GetClassLongPtrW(GCLP_HICON)`；`GetIconInfo` 取彩色位图 → `GetDIBits`
/// 32bpp top-down → BGRA→RGBA。486 期 `app-window` 占位的清偿（473 两度
/// 延期项）。失败（无图标/无彩色位图/DC 失败）→ None（调用方回退占位）。
pub fn window_icon_rgba(target: NativeHwnd) -> Option<(Vec<u8>, u32, u32)> {
    use windows::Win32::Graphics::Gdi::{GetDC, ReleaseDC};
    use windows::Win32::UI::WindowsAndMessaging::{
        GetClassLongPtrW, GetIconInfo, SendMessageTimeoutW, GCLP_HICON, HICON, ICONINFO,
        ICON_SMALL, ICON_SMALL2, SMTO_ABORTIFHUNG, WM_GETICON,
    };
    unsafe {
        let hwnd = HWND(target.0 as *mut core::ffi::c_void);
        // SMTO_ABORTIFHUNG：无响应目标 100ms 内让行（挂死窗口不拖慢投影）。
        let query_icon = |msg: WPARAM| -> isize {
            let mut result = 0usize;
            let lr = SendMessageTimeoutW(
                hwnd,
                WM_GETICON,
                msg,
                LPARAM(0),
                SMTO_ABORTIFHUNG,
                100,
                Some(&mut result as *mut usize),
            );
            lr.0 as isize
        };
        // 候选序列：WM_GETICON 档可能返回哨兵小值（DefWindowProc 的
        // "类图标缺省"标记，非真柄）——首个 GetIconInfo 可解者胜，
        // GCLP_HICON 类图标兜底（scratch/部分老应用仅有类图标）。
        let mut candidates: [isize; 3] = [
            query_icon(WPARAM(ICON_SMALL2 as usize)),
            query_icon(WPARAM(ICON_SMALL as usize)),
            GetClassLongPtrW(hwnd, GCLP_HICON) as isize,
        ];
        candidates.sort_unstable_by(|a, b| b.cmp(a)); // 真柄（大值）优先
        // 屏幕 DC：GetDIBits 只借 DC 做颜色空间转换，不需要选中位图。
        let hdc = GetDC(None);
        let mut out = None;
        for c in candidates {
            if c <= 0 {
                continue;
            }
            if let Some(rgba) = dib_icon_to_rgba(HICON(c as *mut core::ffi::c_void), &hdc) {
                out = Some(rgba);
                break;
            }
        }
        let _ = ReleaseDC(None, hdc);
        out
    }
}

/// `GetIconInfo` → 彩色位图 → `GetDIBits` 32bpp top-down → BGRA→RGBA
/// （纯位图机械，独立于句柄来源；mask-only 单色图标不支持 → None）。
unsafe fn dib_icon_to_rgba(
    icon: windows::Win32::UI::WindowsAndMessaging::HICON,
    hdc: &windows::Win32::Graphics::Gdi::HDC,
) -> Option<(Vec<u8>, u32, u32)> {
    use windows::Win32::Graphics::Gdi::{
        DeleteObject, GetDIBits, GetObjectW, BITMAP, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        CopyImage, DestroyIcon, GetIconInfo, HICON, ICONINFO, IMAGE_FLAGS, IMAGE_ICON,
    };
    let mut info = ICONINFO::default();
    let mut gi = GetIconInfo(icon, &mut info);
    let mut copied: Option<HICON> = None;
    if gi.is_err() {
        // 共享图标 cookie（LoadIconW 标准图标的小值句柄——真窗口亦可能
        // 返回）GetIconInfo 不可解：CopyImage 克隆真句柄重试。
        let Ok(copy) = CopyImage(windows::Win32::Foundation::HANDLE(icon.0), IMAGE_ICON, 0, 0, IMAGE_FLAGS(0)) else {
            return None;
        };
        let copy_icon = HICON(copy.0);
        gi = GetIconInfo(copy_icon, &mut info);
        copied = Some(copy_icon);
    }
    if gi.is_err() {
        if let Some(c) = copied {
            unsafe { let _ = DestroyIcon(c); }
        }
        return None;
    }
    // 位图句柄所有权转移给本地守卫（成功/失败路径都 DeleteObject）。
    struct BmpGuard(windows::Win32::Graphics::Gdi::HBITMAP);
    impl Drop for BmpGuard {
        fn drop(&mut self) {
            unsafe { let _ = DeleteObject(self.0); }
        }
    }
    let color = BmpGuard(info.hbmColor);
    let _mask = BmpGuard(info.hbmMask);
    if color.0 .0.is_null() {
        return None; // mask-only 单色图标（罕见）v1 不支持
    }
    let mut bm = BITMAP::default();
    let got = GetObjectW(
        color.0,
        std::mem::size_of::<BITMAP>() as i32,
        Some(&mut bm as *mut BITMAP as *mut core::ffi::c_void),
    );
    if got == 0 {
        return None;
    }
    let (w, h) = (bm.bmWidth as u32, bm.bmHeight as u32);
    if w == 0 || h == 0 || w > 512 || h > 512 {
        return None; // 异常尺寸防御（图标域 16–256；>512 视为脏数据）
    }
    let mut bmi = BITMAPINFO::default();
    bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
    bmi.bmiHeader.biWidth = w as i32;
    bmi.bmiHeader.biHeight = -(h as i32); // 负高 = top-down（免手工翻行）
    bmi.bmiHeader.biPlanes = 1;
    bmi.bmiHeader.biBitCount = 32;
    bmi.bmiHeader.biCompression = 0; // BI_RGB
    let mut buf = vec![0u8; (w * h * 4) as usize];
    let lines = GetDIBits(
        *hdc,
        color.0,
        0,
        h,
        Some(buf.as_mut_ptr() as *mut core::ffi::c_void),
        &mut bmi,
        DIB_RGB_COLORS,
    );
    if lines != h as i32 {
        return None;
    }
    // BGRA → RGBA（就地字节交换）。
    for px in buf.chunks_exact_mut(4) {
        px.swap(0, 2);
    }
    Some((buf, w, h))
}

// ---------------------------------------------------------------------------
// events：WinEventHook 事件层（专用钩子线程 OUTOFCONTEXT → mpsc）
// ---------------------------------------------------------------------------

/// Win32 `EVENT_*` 常量 → 事件种类（纯函数；单测锁定映射表）。
pub fn map_win_event(event: u32) -> Option<NativeSlotEventKind> {
    match event {
        EVENT_SYSTEM_MOVESIZESTART => Some(NativeSlotEventKind::MoveSizeStart),
        EVENT_SYSTEM_MOVESIZEEND => Some(NativeSlotEventKind::MoveSizeEnd),
        EVENT_SYSTEM_MINIMIZESTART => Some(NativeSlotEventKind::MinimizeStart),
        EVENT_SYSTEM_MINIMIZEEND => Some(NativeSlotEventKind::MinimizeEnd),
        EVENT_OBJECT_LOCATIONCHANGE => Some(NativeSlotEventKind::LocationChange),
        EVENT_OBJECT_DESTROY => Some(NativeSlotEventKind::Destroy),
        _ => None,
    }
}

/// 读回当前指针屏幕物理坐标（DragWatch 光标采样；Plan 486）。
/// 返回 `(x, y)`；失败（罕见）→ None，调用方沿用上次采样。
pub fn cursor_pos() -> Option<(i32, i32)> {
    let mut pt = POINT::default();
    unsafe { GetCursorPos(&mut pt) }.ok().map(|_| (pt.x, pt.y))
}

struct HookShared {
    tx: mpsc::Sender<NativeSlotEvent>,
    thread_id: u32,
}

/// 钩子数据槽位：WinEventProc 无用户上下文参数，通道经全局槽位转交。
/// 回调侧只短暂拿读锁（绝不阻塞钩子线程的消息泵）；
/// 占用互斥（同进程仅一个钩子实例）由 [`HOOK_OCCUPIED`] 原子交换实现——
/// 不用持锁守卫跨生命周期（MutexGuard 非 Send，进不了订阅异步流状态）。
static HOOK_SHARED: RwLock<Option<HookShared>> = RwLock::new(None);
static HOOK_OCCUPIED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// 钩子线程句柄：Drop 时投递 WM_QUIT、清空槽位、释放占用并回收线程。
pub struct NativeSlotEventHook {
    thread: Option<std::thread::JoinHandle<()>>,
}

impl Drop for NativeSlotEventHook {
    fn drop(&mut self) {
        use std::sync::atomic::Ordering;
        let thread_id =
            HOOK_SHARED.read().ok().and_then(|g| g.as_ref().map(|s| s.thread_id));
        if let Some(tid) = thread_id {
            unsafe {
                let _ = PostThreadMessageW(tid, WM_QUIT, WPARAM(0), LPARAM(0));
            }
        }
        // 先清数据槽：解钩完成前的残留回调不再投递，再回收线程。
        if let Ok(mut g) = HOOK_SHARED.write() {
            *g = None;
        }
        HOOK_OCCUPIED.store(false, Ordering::SeqCst);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

/// 启动 WinEventHook 钩子线程：订阅 MoveSizeStart / MoveSizeEnd /
/// MinimizeStart / MinimizeEnd / LocationChange / Destroy 六事件（OUTOFCONTEXT；
/// `skip_own_process` 时叠加 SKIPOWNPROCESS——生产为 true，测试对本进程 scratch
/// 窗口收事件为 false）。返回 `(句柄, 事件接收端)`。同进程同时仅允许一个实例，
/// 重复启动报 `DockError::Api`。
pub fn spawn_event_hook(
    skip_own_process: bool,
) -> Result<(NativeSlotEventHook, mpsc::Receiver<NativeSlotEvent>), DockError> {
    use std::sync::atomic::Ordering;
    if HOOK_OCCUPIED.swap(true, Ordering::SeqCst) {
        return Err(DockError::Api {
            op: "hook already running",
            code: 0,
        });
    }
    let (tx, rx) = mpsc::channel();
    let (ready_tx, ready_rx) = mpsc::channel();
    let thread = std::thread::Builder::new()
        .name("native-dock-winevent".into())
        .spawn(move || unsafe {
            let thread_id = GetCurrentThreadId();
            let flags = WINEVENT_OUTOFCONTEXT
                | if skip_own_process {
                    WINEVENT_SKIPOWNPROCESS
                } else {
                    0
                };
            let mut hooks = Vec::new();
            for e in [
                EVENT_SYSTEM_MOVESIZESTART,
                EVENT_SYSTEM_MOVESIZEEND,
                EVENT_SYSTEM_MINIMIZESTART,
                EVENT_SYSTEM_MINIMIZEEND,
                EVENT_OBJECT_DESTROY,
                EVENT_OBJECT_LOCATIONCHANGE,
            ] {
                let h = SetWinEventHook(e, e, HMODULE::default(), Some(winevent_proc), 0, 0, flags);
                if !h.is_invalid() {
                    hooks.push(h);
                }
            }
            let _ = ready_tx.send((thread_id, hooks.len()));
            let mut msg = MSG::default();
            loop {
                let got = GetMessageW(&mut msg, HWND::default(), 0, 0);
                if !got.as_bool() || msg.message == WM_QUIT {
                    break;
                }
                let _ = TranslateMessage(&msg);
                let _ = DispatchMessageW(&msg);
            }
            for h in hooks {
                let _ = UnhookWinEvent(h);
            }
        })
        .map_err(|e| {
            HOOK_OCCUPIED.store(false, Ordering::SeqCst);
            DockError::Api {
                op: "spawn hook thread",
                code: e.raw_os_error().unwrap_or(0) as u32,
            }
        })?;
    let ready = ready_rx.recv().map_err(|_| {
        HOOK_OCCUPIED.store(false, Ordering::SeqCst);
        DockError::Api {
            op: "hook thread died",
            code: 0,
        }
    });
    let Ok((thread_id, _installed)) = ready else {
        let _ = thread.join();
        return Err(DockError::Api {
            op: "hook thread died",
            code: 0,
        });
    };
    match HOOK_SHARED.write() {
        Ok(mut g) => *g = Some(HookShared { tx, thread_id }),
        Err(_) => {
            HOOK_OCCUPIED.store(false, Ordering::SeqCst);
            return Err(DockError::Api {
                op: "hook slot poisoned",
                code: 0,
            });
        }
    }
    Ok((NativeSlotEventHook { thread: Some(thread) }, rx))
}

unsafe extern "system" fn winevent_proc(
    _hook: HWINEVENTHOOK,
    event: u32,
    hwnd: HWND,
    id_object: i32,
    _id_child: i32,
    _idevent_thread: u32,
    _dwmsevent_time: u32,
) {
    if hwnd.is_invalid() || hwnd.0.is_null() {
        return;
    }
    // 仅窗口级对象（OBJID_WINDOW = 0；子对象 LOCATIONCHANGE 不入队）
    if id_object != 0 {
        return;
    }
    let Some(kind) = map_win_event(event) else {
        return;
    };
    // 读锁 + try 语义：槽位空闲/上锁瞬间直接丢弃事件，绝不阻塞钩子线程
    if let Ok(guard) = HOOK_SHARED.read() {
        if let Some(state) = guard.as_ref() {
            let _ = state.tx.send(NativeSlotEvent {
                hwnd: NativeHwnd(hwnd.0 as isize),
                kind,
            });
        }
    }
}

// ---------------------------------------------------------------------------
// Plan 486 T4：合成拖拽（SendInput 真手势路径；仅 test-native-dock 构建）
// ---------------------------------------------------------------------------

/// 合成拖拽测试支持（`--features test-native-dock` 独占；生产二进制不编译）。
/// 手段定案（待澄清①执行期裁定）：SendInput 真实拖动标题栏——点击/移动/
/// 释放走真实输入管线，目标窗口进入真 move-size 模态循环，产出真实的
/// MOVESIZESTART/LOCATIONCHANGE/MOVESIZEEND WinEvent 序列。
#[cfg(feature = "test-native-dock")]
pub mod drag_sim {
    use super::{hwnd_of, NativeHwnd};
    use windows::Win32::Foundation::POINT;
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_LEFTDOWN,
        MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MOVE, MOUSE_EVENT_FLAGS, MOUSEINPUT,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        GetCursorPos, GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN,
    };

    fn send_mouse(dx: i32, dy: i32, flags: MOUSE_EVENT_FLAGS) -> bool {
        let input = INPUT {
            r#type: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx,
                    dy,
                    mouseData: 0,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        let sent = unsafe { SendInput(&[input], core::mem::size_of::<INPUT>() as i32) };
        sent == 1
    }

    fn send_button(flags: MOUSE_EVENT_FLAGS) -> bool {
        send_mouse(0, 0, flags)
    }

    /// 主屏坐标 → 绝对坐标归一（0..65535；单屏语义，与 B9 注记一致）。
    fn normalize(x: i32, y: i32) -> (i32, i32) {
        let sw = unsafe { GetSystemMetrics(SM_CXSCREEN) }.max(1);
        let sh = unsafe { GetSystemMetrics(SM_CYSCREEN) }.max(1);
        ((x.clamp(0, sw - 1) * 65535) / (sw - 1), (y.clamp(0, sh - 1) * 65535) / (sh - 1))
    }

    /// 绝对移动光标到主屏 `(x, y)`。
    fn move_to(x: i32, y: i32) -> bool {
        let (nx, ny) = normalize(x, y);
        send_mouse(nx, ny, MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE)
    }

    /// 读回当前光标（GetCursorPos 直通）。
    pub fn cursor() -> Option<(i32, i32)> {
        let mut pt = POINT::default();
        unsafe { GetCursorPos(&mut pt) }.ok().map(|_| (pt.x, pt.y))
    }

    /// 合成单击（move → down → up；实机冒烟驱动用，与拖拽同输入管线）。
    pub fn click_at(x: i32, y: i32) -> bool {
        use std::time::Duration;
        if !move_to(x, y) {
            return false;
        }
        std::thread::sleep(Duration::from_millis(50));
        send_button(MOUSEEVENTF_LEFTDOWN) && {
            std::thread::sleep(Duration::from_millis(50));
            send_button(MOUSEEVENTF_LEFTUP)
        }
    }

    /// 命中测试：屏幕点所在顶层窗口 hwnd 值（0 = 桌面）。拖拽前置校验用
    /// （标题栏被它窗遮挡时 SendInput 点击不会落在目标上）。
    pub fn window_from_point(x: i32, y: i32) -> isize {
        use windows::Win32::UI::WindowsAndMessaging::WindowFromPoint;
        let h = unsafe { WindowFromPoint(POINT { x, y }) };
        super::hwnd_value(h)
    }

    /// 提到 z 序顶（HWND_TOPMOST——越过 topmost 带的常驻窗如
    /// TextInputHost；测试支持语境，生产 z 序不变量不适用）。不动几何
    /// 不抢激活。
    pub fn raise_top(hwnd: NativeHwnd) -> bool {
        use windows::Win32::UI::WindowsAndMessaging::{SetWindowPos, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE};
        unsafe {
            SetWindowPos(
                hwnd_of(hwnd),
                HWND_TOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            )
        }
        .is_ok()
    }

    /// 撤销 [`raise_top`] 的 topmost 位（清理用）。
    pub fn unraise(hwnd: NativeHwnd) -> bool {
        use windows::Win32::UI::WindowsAndMessaging::{SetWindowPos, HWND_NOTOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE};
        unsafe {
            SetWindowPos(
                hwnd_of(hwnd),
                HWND_NOTOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            )
        }
        .is_ok()
    }

    /// 强制前台（AttachThreadInput 经典技巧）：后台进程的
    /// `SetForegroundWindow` 受前台锁限制，附着到当前前台窗口线程的输入
    /// 队列后可越过（同为用户完整性级别时）。488 OLE 拖拽 E2E 复用（pub）。
    pub fn force_foreground(hwnd: NativeHwnd) {
        use windows::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};
        use windows::Win32::UI::WindowsAndMessaging::{
            GetForegroundWindow, GetWindowThreadProcessId, SetForegroundWindow,
        };
        let target = hwnd_of(hwnd);
        let fg = unsafe { GetForegroundWindow() };
        if fg.0 == target.0 {
            return;
        }
        let mut fg_pid = 0u32;
        let fg_tid = unsafe { GetWindowThreadProcessId(fg, Some(&mut fg_pid)) };
        let my_tid = unsafe { GetCurrentThreadId() };
        let attached = fg_tid != 0 && fg_tid != my_tid && unsafe {
            AttachThreadInput(my_tid, fg_tid, true)
        }
        .as_bool();
        unsafe {
            let _ = SetForegroundWindow(target);
        }
        if attached {
            unsafe {
                let _ = AttachThreadInput(my_tid, fg_tid, false);
            }
        }
    }

    /// 系统命令拖拽（待澄清①"注入退路"的实装形态）：`WM_SYSCOMMAND
    /// (SC_MOVE|HTCAPTION)` 直达目标窗消息队列——**不依赖标题栏 z 序命中**
    ///（目标窗可被前台/提权窗覆盖，非提权进程无法置其上，caption 点击必然
    /// 落到遮挡窗）。前置强制前台（SC_MOVE 模态循环要求目标窗激活态），
    /// 命令使目标线程进入**真实** move-size 模态循环（真实
    /// MOVESIZESTART/LOCATIONCHANGE/MOVESIZEEND WinEvent 序列），随后
    /// SendInput 分步移动光标驱动窗口跟随，末尾左键点击提交落位。
    pub fn syscommand_drag_to(hwnd: NativeHwnd, to: (i32, i32), steps: usize) -> bool {
        use std::time::Duration;
        use windows::Win32::Foundation::{LPARAM, WPARAM};
        use windows::Win32::UI::WindowsAndMessaging::{
            PostMessageW, SC_MOVE, WM_SYSCOMMAND,
        };
        let _ = raise_top(hwnd);
        force_foreground(hwnd);
        std::thread::sleep(Duration::from_millis(80));
        // 光标先落在目标窗标题条中点（拖拽锚点语义对齐 caption 拖）。
        let Some(rect) = super::get_bounds_window(hwnd) else {
            return false;
        };
        // 抓点取 70% 宽处：兼顾三类 caption 形态——高 DPI 小窗（按钮占宽
        // 过半，正中命中按钮——4K@200% 实测教训）、Win11 Explorer 页签条
        // 占左半（1/3 宽落在页签上会触发撕页签）、常规窗（按钮在最右
        // ~15%）。y 取 caption 深处（200% 下 caption ≈46 物理px）。
        let grab = (rect.x + rect.w * 7 / 10, rect.y + 16);
        if !move_to(grab.0, grab.1) {
            return false;
        }
        std::thread::sleep(Duration::from_millis(50));
        // SC_MOVE | HTCAPTION（0xF010 | 0x2 = 0xF012：caption 拖拽形态，
        // 鼠标跟踪——SC_MOVE 裸值走键盘方向键形态）。
        let posted = unsafe {
            PostMessageW(
                hwnd_of(hwnd),
                WM_SYSCOMMAND,
                WPARAM((SC_MOVE | 0x2) as usize),
                LPARAM(0),
            )
        };
        if !posted.is_ok() {
            return false;
        }
        std::thread::sleep(Duration::from_millis(50));
        for i in 1..=steps.max(1) {
            let t = i as f32 / steps.max(1) as f32;
            let x = grab.0 as f32 + (to.0 as f32 - grab.0 as f32) * t;
            let y = grab.1 as f32 + (to.1 as f32 - grab.1 as f32) * t;
            if !move_to(x as i32, y as i32) {
                return false;
            }
            std::thread::sleep(Duration::from_millis(15));
        }
        std::thread::sleep(Duration::from_millis(30));
        // 提交：左键点击结束循环（消息发起的循环无按键态，点击即落定）。
        send_button(MOUSEEVENTF_LEFTDOWN) && send_button(MOUSEEVENTF_LEFTUP)
    }

    /// 合成标题栏拖拽：前置 [`raise_top`]（标题栏可命中）+ [`force_foreground`]
    ///（非激活窗的首击被激活语义吞掉、不达 caption——必须先行前台化）→
    /// 光标落标题条中点 → 按下 → 分步移到 `to` → 释放。`steps` 步间 15ms。
    /// 返回 SendInput 全链成功与否（环境故障时调用方走注入退路）。
    pub fn caption_drag_to(hwnd: NativeHwnd, to: (i32, i32), steps: usize) -> bool {
        use std::time::Duration;
        let _ = raise_top(hwnd);
        force_foreground(hwnd);
        std::thread::sleep(Duration::from_millis(80));
        let Some(rect) = super::get_bounds_window(hwnd) else {
            return false;
        };
        // 抓点取 70% 宽处：兼顾三类 caption 形态——高 DPI 小窗（按钮占宽
        // 过半，正中命中按钮——4K@200% 实测教训）、Win11 Explorer 页签条
        // 占左半（1/3 宽落在页签上会触发撕页签）、常规窗（按钮在最右
        // ~15%）。y 取 caption 深处（200% 下 caption ≈46 物理px）。
        let grab = (rect.x + rect.w * 7 / 10, rect.y + 16);
        if !move_to(grab.0, grab.1) {
            return false;
        }
        std::thread::sleep(Duration::from_millis(60));
        if window_from_point(grab.0, grab.1) != hwnd.0 {
            return false; // 标题栏被并发遮挡（z 序扰动）——交由调用方退路
        }
        // 激活结算：程序化前台（SetForegroundWindow）后的首击会被激活语义
        // 吞掉（不达 caption）——先点一次完成结算；间隔 > 双击窗（500ms
        // 默认 + 余量），避免与拖拽按下构成标题栏双击（Win11 标题栏双击
        // 默认动作会最小化/最大化窗口）。
        if !send_button(MOUSEEVENTF_LEFTDOWN) || !send_button(MOUSEEVENTF_LEFTUP) {
            return false;
        }
        std::thread::sleep(Duration::from_millis(700));
        if !move_to(grab.0, grab.1) {
            return false;
        }
        std::thread::sleep(Duration::from_millis(40));
        if !send_button(MOUSEEVENTF_LEFTDOWN) {
            return false;
        }
        for i in 1..=steps.max(1) {
            let t = i as f32 / steps.max(1) as f32;
            let x = grab.0 as f32 + (to.0 as f32 - grab.0 as f32) * t;
            let y = grab.1 as f32 + (to.1 as f32 - grab.1 as f32) * t;
            if !move_to(x as i32, y as i32) {
                return false;
            }
            std::thread::sleep(Duration::from_millis(15));
        }
        std::thread::sleep(Duration::from_millis(30));
        send_button(MOUSEEVENTF_LEFTUP)
    }

    // ── Plan 488 步骤 9：OLE 拖放合成原语（SendInput 真拖主路径，486
    //    待澄清③裁定沿用）。 ─────────────────────────────────────────

    /// 绝对移动光标到主屏 `(x, y)`（OLE 拖拽分步驱动用）。
    pub fn ole_move_to(x: i32, y: i32) -> bool {
        move_to(x, y)
    }

    /// 在当前光标处按下左键（OLE 拖出：先按后起 DoDragDrop 的时序用）。
    pub fn ole_press() -> bool {
        send_button(MOUSEEVENTF_LEFTDOWN)
    }

    /// 释放左键（拖拽提交）。
    pub fn ole_release() -> bool {
        send_button(MOUSEEVENTF_LEFTUP)
    }

    /// OLE 合成拖拽：`from`（源客户区点）按下 → 分步移到 `to` → 释放。
    /// **无激活结算点击**——caption_drag_to 的结算教训不适用于 OLE 拖源：
    /// 结算点击落在 --offer 客户区会自身触发一次 DoDragDrop，其退出时序
    /// 与后续真实拖拽互绞（实测卡死）。前置聚焦由调用方 `focus_window`
    /// 完成（程序化前台后客户区首击仍达 WM_LBUTTONDOWN——激活语义不
    /// 吞客户区消息）。
    pub fn ole_drag(from: (i32, i32), to: (i32, i32), steps: usize) -> bool {
        use std::time::Duration;
        if !move_to(from.0, from.1) {
            return false;
        }
        std::thread::sleep(Duration::from_millis(120));
        if !send_button(MOUSEEVENTF_LEFTDOWN) {
            return false;
        }
        // 起拖确认：DoDragDrop 进入模态循环需要首个移动消息（按下点静止
        // 不触发 DragEnter）。
        std::thread::sleep(Duration::from_millis(80));
        for i in 1..=steps.max(1) {
            let t = i as f32 / steps.max(1) as f32;
            let x = from.0 as f32 + (to.0 as f32 - from.0 as f32) * t;
            let y = from.1 as f32 + (to.1 as f32 - from.1 as f32) * t;
            if !move_to(x as i32, y as i32) {
                return false;
            }
            std::thread::sleep(Duration::from_millis(15));
        }
        std::thread::sleep(Duration::from_millis(60));
        send_button(MOUSEEVENTF_LEFTUP)
    }
}

// ---------------------------------------------------------------------------
// Plan 494 T3：E2E 测试支持（feature `test-native-dock` 独占；生产二进制
// 不编译）——进程内 scratch 顶层窗（DefWindowProc + 消息泵），供
// tests/native_dock_e2e.rs 充当"桌面替身"窗（Region 洞/z 序断言载体）。
// ---------------------------------------------------------------------------

#[cfg(all(windows, feature = "test-native-dock"))]
pub mod test_support {
    use super::{hwnd_of, hwnd_value, NativeHwnd, Rect};
    use std::sync::OnceLock;
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::UI::HiDpi::{
        SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, LoadIconW,
        PeekMessageW, RegisterClassW, SetWindowPos, TranslateMessage, HWND_TOPMOST,
        IDI_APPLICATION, WINDOW_EX_STYLE, MSG,
        PM_REMOVE, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW, WNDCLASSW,
        WS_OVERLAPPEDWINDOW, WS_VISIBLE,
    };

    fn class_name() -> &'static [u16] {
        static NAME: OnceLock<Vec<u16>> = OnceLock::new();
        NAME.get_or_init(|| "auto_lang_e2e_scratch ".encode_utf16().collect())
    }

    unsafe extern "system" fn wndproc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
    }

    /// 进程内 scratch 顶层窗（返回 NativeHwnd；Drop 即 DestroyWindow）。
    /// 与生产宿主对齐声明 per-monitor v2（坐标域一致）。
    pub struct Scratch(pub NativeHwnd);

    impl Drop for Scratch {
        fn drop(&mut self) {
            unsafe {
                let _ = DestroyWindow(hwnd_of(self.0));
            }
        }
    }

    pub fn spawn(title: &str, rect: Rect) -> Scratch {
        static DONE: OnceLock<()> = OnceLock::new();
        DONE.get_or_init(|| unsafe {
            let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
            let hmodule = GetModuleHandleW(None).expect("GetModuleHandleW");
            let wc = WNDCLASSW {
                lpfnWndProc: Some(wndproc),
                hInstance: HINSTANCE(hmodule.0),
                lpszClassName: PCWSTR(class_name().as_ptr()),
                ..Default::default()
            };
            let atom = RegisterClassW(&wc);
            assert_ne!(
                atom, 0,
                "RegisterClassW failed: {:?}",
                windows::Win32::Foundation::GetLastError()
            );
        });
        let mut title_w: Vec<u16> = title.encode_utf16().collect();
        title_w.push(0);
        let hmodule = unsafe { GetModuleHandleW(None) }.expect("GetModuleHandleW");
        let hwnd = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                PCWSTR(class_name().as_ptr()),
                PCWSTR(title_w.as_ptr()),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                rect.x,
                rect.y,
                rect.w,
                rect.h,
                HWND::default(),
                windows::Win32::UI::WindowsAndMessaging::HMENU::default(),
                hmodule,
                None,
            )
        }
        .expect("CreateWindowExW failed");
        Scratch(NativeHwnd(hwnd_value(hwnd)))
    }

    /// 提 TOPMOST 带（E2E 后台进程窗不自动置顶——终端遮挡候选摆位）。
    pub fn raise_topmost(h: NativeHwnd) {
        unsafe {
            let _ = SetWindowPos(
                hwnd_of(h),
                HWND_TOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW,
            );
        }
    }

    /// 泵该窗消息队列至多 `ms` 毫秒（WM_NCHITTEST 同步送达需泵）。
    pub fn pump_for(h: NativeHwnd, ms: u64) {
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(ms);
        while std::time::Instant::now() < deadline {
            let mut msg = MSG::default();
            unsafe {
                while PeekMessageW(&mut msg, hwnd_of(h), 0, 0, PM_REMOVE).as_bool() {
                    let _ = TranslateMessage(&msg);
                    let _ = DispatchMessageW(&msg);
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }
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
        PeekMessageW, RegisterClassW, TranslateMessage, CW_USEDEFAULT, GW_HWNDNEXT, GW_HWNDPREV,
        MSG,
        PM_REMOVE, WINDOW_EX_STYLE, WNDCLASSW, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
    };

    /// 本进程 scratch 窗口（DefWindowProc；Drop 时 DestroyWindow）。
    pub(super) struct Scratch(pub(super) HWND);

    impl Drop for Scratch {
        fn drop(&mut self) {
            unsafe {
                let _ = DestroyWindow(self.0);
            }
        }
    }

    fn class_name() -> &'static [u16] {
        static NAME: OnceLock<Vec<u16>> = OnceLock::new();
        NAME.get_or_init(|| "auto_lang_native_dock_scratch ".encode_utf16().collect())
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
                // Plan 515 D1：挂私有真图标（.ico 合成文件 + LoadImageW
                // ——与生产应用同链；32bpp BGRA blue-500 实色像素断言用。
                // CreateBitmap DDB 位图经 GetDIBits 不保真内容/alpha，故
                // 走真 .ico 装载）。
                hIcon: load_blue_test_icon(),
                ..Default::default()
            };
            let atom = RegisterClassW(&wc);
            assert_ne!(
                atom, 0,
                "RegisterClassW failed: {:?}",
                windows::Win32::Foundation::GetLastError()
            );
        });
    }

    pub(super) fn scratch(title: &str) -> Scratch {
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

    pub(super) fn pump_one(hwnd: HWND) -> bool {
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

    /// 自 `hwnd` 向下走 z 链，取第一个**可见**窗口（IME 伴随窗
    /// （Default IME / MSCTFIME UI）不可见且楔入相邻位——直接 prev 断言
    /// 在带输入法的环境恒假，实测教训）。
    fn first_visible_below(hwnd: HWND) -> Option<HWND> {
        let mut cur = hwnd;
        for _ in 0..64 {
            let next = unsafe { GetWindow(cur, GW_HWNDNEXT) }.ok()?;
            if next.is_invalid() || next.0.is_null() {
                return None;
            }
            if unsafe { IsWindowVisible(next) }.as_bool() {
                return Some(next);
            }
            cur = next;
        }
        None
    }

    /// Plan 494 T5：真洞 z 序不变量——raise_desktop_above(desktop, slot) 后
    /// slot 紧贴桌面正下方（prev(slot) == desktop）。
    #[test]
    fn z_order_true_hole_desktop_above_native() {
        let desktop = scratch("hole-desktop");
        let target = scratch("hole-native");
        let desktop_h = NativeHwnd(hwnd_value(desktop.0));
        let target_h = NativeHwnd(hwnd_value(target.0));
        let mut ok = false;
        for _ in 0..20 {
            raise_desktop_above(desktop_h, target_h).expect("raise desktop above slot");
            if first_visible_below(desktop.0) == Some(target.0) {
                ok = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(
            ok,
            "raise_desktop_above 后 desktop 应紧贴 slot 正上方（真洞不变量）"
        );
    }

    /// Plan 494 T5：Region 洞排除——apply_hole_regions 后洞区命中测试直达
    /// z 序下层窗（WindowFromPoint），洞外仍命中本窗；空表复位全窗。
    /// 摆位：候选点轮询直到 pair 不被终端等前台窗遮挡（spike② 同型教训）。
    #[test]
    fn hole_region_carves_input_pass_through() {
        let bottom = scratch("hole-region-bottom");
        let top = scratch("hole-region-top");
        let bottom_h = NativeHwnd(hwnd_value(bottom.0));
        let top_h = NativeHwnd(hwnd_value(top.0));
        let hit = |x: i32, y: i32| unsafe {
            hwnd_value(windows::Win32::UI::WindowsAndMessaging::WindowFromPoint(POINT { x, y }))
        };
        // top 完整覆盖 bottom 并压其上（真洞 z 序）；候选摆位到无遮挡点。
        let mut brect = Rect::new(400, 300, 640, 440);
        let mut settled = false;
        for cand in [(400, 300), (900, 200), (1500, 500), (200, 900), (1400, 1000)] {
            brect = Rect::new(cand.0, cand.1, 640, 440);
            set_bounds(bottom_h, brect).expect("place bottom");
            set_bounds(top_h, brect).expect("cover bottom");
            // 后台进程的新窗不自动置顶——pair 提 TOPMOST 带（终端/IDE 覆盖
            // 候选点时 WindowFromPoint 全部落空，spike② 同型教训）。
            unsafe {
                use windows::Win32::UI::WindowsAndMessaging::{
                    HWND_TOPMOST, SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW,
                };
                let _ = SetWindowPos(
                    hwnd_of(top_h),
                    HWND_TOPMOST,
                    0,
                    0,
                    0,
                    0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW,
                );
            }
            raise_desktop_above(top_h, bottom_h).expect("top above bottom");
            std::thread::sleep(std::time::Duration::from_millis(250));
            // 就位自检：非洞点命中 top（若被终端遮挡则换点）。
            if hit(brect.x + 20, brect.y + 20) == hwnd_value(top.0) {
                settled = true;
                break;
            }
        }
        assert!(settled, "未找到无遮挡摆位（终端覆盖全部候选点？）");
        // 洞 = pair 中心 120x120（屏幕物理坐标；函数内部换算窗口局部域）。
        let hole_screen = Rect::new(
            brect.x + brect.w / 2 - 60,
            brect.y + brect.h / 2 - 60,
            120,
            120,
        );
        apply_hole_regions(top_h, brect, &[hole_screen]).expect("carve hole");
        std::thread::sleep(std::time::Duration::from_millis(200));
        // 洞心：命中直达 bottom（区域语义——窗口在洞内不存在）。
        let hole_pt = (hole_screen.x + 60, hole_screen.y + 60);
        assert_eq!(
            hit(hole_pt.0, hole_pt.1),
            hwnd_value(bottom.0),
            "洞心命中应直达下层窗（Region 排除）"
        );
        // 洞外（top 左上角内、洞矩形外）：仍命中 top。
        assert_eq!(
            hit(brect.x + 20, brect.y + 20),
            hwnd_value(top.0),
            "洞外命中应仍是 top"
        );
        // 复位：空表 → 全窗形状恢复（洞心回到 top）。
        apply_hole_regions(top_h, brect, &[]).expect("reset region");
        std::thread::sleep(std::time::Duration::from_millis(200));
        assert_eq!(
            hit(hole_pt.0, hole_pt.1),
            hwnd_value(top.0),
            "Region 复位后洞心应回到 top"
        );
    }

    #[test]
    fn covers_rect_detects_fullscreen_coverage_c2() {
        let desktop = scratch("covers-ref");
        let target = scratch("covers-target");
        let d = NativeHwnd(hwnd_value(desktop.0));
        let t = NativeHwnd(hwnd_value(target.0));
        // 桌面替身摆到屏内一域；目标先收在该域内部 → 不算覆盖。
        let drect = Rect::new(400, 300, 500, 400);
        set_bounds(d, drect).expect("set desktop rect");
        set_bounds(t, Rect::new(450, 350, 200, 200)).expect("set target inside");
        assert!(!covers_rect(t, d), "域内窗口不应判独占全屏");
        // 目标扩张到完整覆盖参照域 → 判独占全屏（C2）。
        set_bounds(t, Rect::new(395, 295, 520, 420)).expect("set target covering");
        assert!(covers_rect(t, d), "完整覆盖参照域应判独占全屏");
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
        // find 结果与两次枚举间的系统窗口增删存在竞态，不做跨调用集合核对，
        // 改为读回 pid 自洽核对
        let found = find_top_level_by_pid(std::process::id()).expect("存在同进程可见顶层窗口");
        assert_eq!(
            pid_of(found),
            Some(std::process::id()),
            "find 结果 pid 读回核对"
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

// ---------------------------------------------------------------------------
// T3：WinEventHook 事件层测试——映射表纯单测 + 本进程 scratch 窗口真实收事件
// ---------------------------------------------------------------------------

/// Plan 515 D1：合成 32×32 32bpp blue-500 实心 .ico（ICONDIR + 双倍高
/// BITMAPINFOHEADER + bottom-up BGRA XOR + AND 掩码），落临时文件后
/// `LoadImageW(LR_LOADFROMFILE)` 装载为真 HICON。
fn load_blue_test_icon() -> windows::Win32::UI::WindowsAndMessaging::HICON {
    use std::io::Write;
    use windows::Win32::UI::WindowsAndMessaging::{
        LoadImageW, IMAGE_ICON, LR_DEFAULTSIZE, LR_LOADFROMFILE,
    };
    let (w, h) = (32u32, 32u32);
    let xor = (w * h * 4) as u32;
    let mask = w * 4; // 1bpp 行对齐 = w/8 → 4B/行
    let mut ico: Vec<u8> = Vec::with_capacity(22 + 40 + xor as usize + mask as usize);
    ico.extend_from_slice(&[0, 0]); // reserved
    ico.extend_from_slice(&[1, 0]); // type = icon
    ico.extend_from_slice(&[1, 0]); // count
    ico.extend_from_slice(&[32, 32, 0, 0]); // w h colors rsv
    ico.extend_from_slice(&[1, 0]); // planes
    ico.extend_from_slice(&[32, 0]); // bpp
    ico.extend_from_slice(&(40 + xor + mask).to_le_bytes()); // bytes in rsrc
    ico.extend_from_slice(&22u32.to_le_bytes()); // offset
    ico.extend_from_slice(&40u32.to_le_bytes()); // biSize
    ico.extend_from_slice(&32i32.to_le_bytes()); // biWidth
    ico.extend_from_slice(&64i32.to_le_bytes()); // biHeight ×2（XOR+AND）
    ico.extend_from_slice(&[1, 0]); // planes
    ico.extend_from_slice(&[32, 0]); // bitcount
    ico.extend_from_slice(&0u32.to_le_bytes()); // BI_RGB
    ico.extend_from_slice(&(xor + mask).to_le_bytes());
    ico.extend_from_slice(&[0u8; 16]); // 分辨率/色数零
    for _ in 0..w * h {
        ico.extend_from_slice(&[0xF6, 0x82, 0x3B, 0xFF]); // BGRA blue-500
    }
    ico.extend_from_slice(&vec![0u8; mask as usize]); // AND 全 0 = 不透明
    // 唯一文件名（并行测试各写各的——共享路径会互锁 create）。
    static SEQ: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let n = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "auto_scratch_test_icon_{}_{}.ico",
        std::process::id(),
        n
    ));
    let mut f = std::fs::File::create(&path).expect("create ico");
    f.write_all(&ico).expect("write ico");
    drop(f);
    let mut wide: Vec<u16> = path.as_os_str().to_string_lossy().encode_utf16().collect();
    wide.push(0);
    unsafe {
        LoadImageW(
            None,
            windows::core::PCWSTR(wide.as_ptr()),
            IMAGE_ICON,
            32,
            32,
            LR_LOADFROMFILE | LR_DEFAULTSIZE,
        )
        .map(|h| windows::Win32::UI::WindowsAndMessaging::HICON(h.0))
        .unwrap_or_default()
    }
}

#[cfg(all(test, windows, feature = "test-native-dock"))]
mod native_dock_events {
    use super::*;
    use super::native_dock_geometry::{pump_one, scratch};
    use std::time::Duration;

    /// 事件测试间互斥：全局钩子槽位同进程仅一个实例，并行调度下自旋等位。
    fn spawn_with_retry() -> (NativeSlotEventHook, mpsc::Receiver<NativeSlotEvent>) {
        for _ in 0..100 {
            match spawn_event_hook(false) {
                Ok(pair) => return pair,
                Err(_) => std::thread::sleep(Duration::from_millis(100)),
            }
        }
        panic!("钩子槽位 10s 未释放（并行事件测试死锁？）");
    }

    /// 从全局事件流中等待指定 hwnd 的目标事件（跳过系统噪声），3s 超时。
    fn wait_for(
        rx: &mpsc::Receiver<NativeSlotEvent>,
        hwnd: NativeHwnd,
        kind: NativeSlotEventKind,
    ) -> bool {
        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        while std::time::Instant::now() < deadline {
            match rx.recv_timeout(Duration::from_millis(500)) {
                Ok(evt) if evt.hwnd == hwnd && evt.kind == kind => return true,
                Ok(_) => continue, // 其他窗口/种类噪声
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
                Err(mpsc::RecvTimeoutError::Disconnected) => return false,
            }
        }
        false
    }

    #[test]
    fn map_win_event_covers_matrix() {
        assert_eq!(
            map_win_event(EVENT_SYSTEM_MOVESIZESTART),
            Some(NativeSlotEventKind::MoveSizeStart)
        );
        assert_eq!(
            map_win_event(EVENT_SYSTEM_MOVESIZEEND),
            Some(NativeSlotEventKind::MoveSizeEnd)
        );
        assert_eq!(
            map_win_event(EVENT_SYSTEM_MINIMIZESTART),
            Some(NativeSlotEventKind::MinimizeStart)
        );
        assert_eq!(
            map_win_event(EVENT_SYSTEM_MINIMIZEEND),
            Some(NativeSlotEventKind::MinimizeEnd)
        );
        assert_eq!(
            map_win_event(EVENT_OBJECT_LOCATIONCHANGE),
            Some(NativeSlotEventKind::LocationChange)
        );
        assert_eq!(
            map_win_event(EVENT_OBJECT_DESTROY),
            Some(NativeSlotEventKind::Destroy)
        );
        assert_eq!(map_win_event(0x1234), None);
    }

    #[test]
    fn cursor_pos_samples_physical_screen_point() {
        // 无 UIPI 门槛的系统级读数；只要返回 Some 即证明坐标系可用
        let pt = cursor_pos();
        assert!(pt.is_some(), "GetCursorPos 不应失败");
    }

    #[test]
    fn hook_receives_location_change() {
        let (_hook, rx) = spawn_with_retry();
        let s = scratch("evt-move");
        let h = NativeHwnd(hwnd_value(s.0));
        // 两步几何写，保证至少一次位置变化事件
        set_bounds(h, Rect::new(80, 80, 320, 240)).expect("set_bounds 1");
        set_bounds(h, Rect::new(90, 100, 320, 240)).expect("set_bounds 2");
        assert!(
            wait_for(&rx, h, NativeSlotEventKind::LocationChange),
            "3s 内未收到 scratch 窗口的 LOCATIONCHANGE"
        );
    }

    #[test]
    fn hook_receives_minimize_lifecycle() {
        let (_hook, rx) = spawn_with_retry();
        let s = scratch("evt-min");
        let h = NativeHwnd(hwnd_value(s.0));
        show_window(h, ShowMode::Minimize).expect("minimize");
        assert!(
            wait_for(&rx, h, NativeSlotEventKind::MinimizeStart),
            "3s 内未收到 MINIMIZESTART"
        );
        show_window(h, ShowMode::Restore).expect("restore");
        assert!(
            wait_for(&rx, h, NativeSlotEventKind::MinimizeEnd),
            "3s 内未收到 MINIMIZEEND"
        );
    }

    #[test]
    fn hook_receives_destroy() {
        let (_hook, rx) = spawn_with_retry();
        let s = scratch("evt-close");
        let h = NativeHwnd(hwnd_value(s.0));
        request_close(h).expect("request_close");
        // WM_CLOSE 需宿主线程泵消息才会走 DestroyWindow
        for _ in 0..100 {
            if !is_alive(h) {
                break;
            }
            pump_one(s.0);
        }
        assert!(
            wait_for(&rx, h, NativeSlotEventKind::Destroy),
            "3s 内未收到 DESTROY"
        );
        // s 在此 Drop：对已死窗口 DestroyWindow 是 no-op 错误，安全忽略
    }

    /// Plan 486 T4：进程内 scratch 窗上的合成 caption 拖拽——真实
    /// move-size 循环被 SendInput 序列驱动（跨进程 E2E 的进程内复现）。
    /// 窗口取大尺寸（高 DPI 下小窗 caption 按钮占宽过半，1/3 宽抓点也
    /// 会命中按钮——4K@200% 实测教训）。
    #[test]
    fn drag_sim_captions_drags_scratch_window() {
        let (_hook, rx) = spawn_with_retry();
        let s = scratch("evt-drag");
        let h = NativeHwnd(hwnd_value(s.0));
        set_bounds(h, Rect::new(80, 80, 1400, 900)).expect("place");
        let before = get_bounds_window(h).expect("before");
        // 拖拽序列在后台线程注入；窗口线程（本线程）持续泵消息——
        // caption 模态 move 循环在本线程的 DispatchMessage 内运行。
        let hd = h;
        let drag = std::thread::spawn(move || {
            drag_sim::caption_drag_to(hd, (1600, 1200), 20)
        });
        while !drag.is_finished() {
            pump_one(s.0);
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert!(drag.join().expect("drag thread"), "SendInput 链成功");
        let after = get_bounds_window(h).expect("after");
        let moved = (after.x - before.x).abs() > 60 || (after.y - before.y).abs() > 60;
        assert!(moved, "scratch 窗应被真实拖动（before {before:?} after {after:?}）");
        assert!(
            wait_for(&rx, h, NativeSlotEventKind::MoveSizeStart),
            "3s 内未收到 MOVESIZESTART（scratch 窗）"
        );
    }

    /// Plan 515 D1：HICON 提取真窗口演练——scratch 窗带系统类图标
    ///（CreateWindowW 类图标链），提取应出 32bpp RGBA 且尺寸在图标域。
    #[test]
    fn window_icon_rgba_extracts_scratch_window_icon() {
        let s = scratch("icon-extract");
        // 本测试段的 Scratch 包 HWND（0.58 指针形态）→ NativeHwnd isize 存储。
        let (rgba, w, h) = window_icon_rgba(NativeHwnd(s.0 .0 as isize))
            .expect("scratch 窗应带类图标（GCLP_HICON 回退档）");
        assert_eq!(rgba.len(), (w * h * 4) as usize, "32bpp 尺寸自洽");
        assert!(w > 0 && h > 0 && w <= 512 && h <= 512, "图标域: {w}x{h}");
        // 像素内容证据：blue-500（RGB 0x3B82F6）特征——GDI DDB 位图
        // 不保真 alpha（真 .ico 才有），故验色不验 alpha。
        assert!(
            rgba.chunks_exact(4).any(|px| px[0] == 0x3B && px[1] == 0x82 && px[2] == 0xF6),
            "提取像素含 blue-500 特征（内容非空壳）"
        );
    }
}

/// PLAN-526 T39：本进程主顶层窗是否处于最小化（IsIconic）。
/// 用途：MCP autoui_screenshot 前置护栏——最小化窗物理尺寸为 0，
/// window::screenshot 触发 wgpu create_texture "Dimension X is zero"
/// panic（wgpu-27 backend/wgpu_core.rs:1588，exit 101 实录）。
pub fn main_window_minimized(pid: u32) -> bool {
    mod raw {
        #[link(name = "user32")]
        extern "system" {
            pub fn EnumWindows(cb: extern "system" fn(isize, isize) -> i32, lparam: isize) -> i32;
            pub fn IsIconic(hwnd: isize) -> i32;
            pub fn GetWindowThreadProcessId(hwnd: isize, pid: *mut u32) -> u32;
        }
    }
    thread_local! {
        static MIN: Cell<Option<bool>> = const { Cell::new(None) };
    }
    extern "system" fn enum_cb(hwnd: isize, _lparam: isize) -> i32 {
        let mut w = 0u32;
        unsafe { raw::GetWindowThreadProcessId(hwnd, &mut w) };
        if w == std::process::id() {
            let iconic = unsafe { raw::IsIconic(hwnd) } != 0;
            MIN.with(|m| m.set(Some(iconic)));
            return 0;
        }
        1
    }
    MIN.with(|m| m.set(None));
    unsafe { raw::EnumWindows(enum_cb, 0) };
    MIN.with(|m| m.get().unwrap_or(false))
}
