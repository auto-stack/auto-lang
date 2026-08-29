//! Plan 473：原生窗口 dock（假洞 Phase 1）——`NativeSlot` 模型 + 状态机 + 策略。
//!
//! 本模块是纯逻辑层，零外部依赖，全平台可编译可单测（`cargo t native_dock`
//! 日常档）。Win32 调用只允许出现在 [`win32`] 子模块（`#[cfg(windows)]`；
//! 非 Windows 以同名 no-op 模块顶替，宿主层无需 cfg 包裹）。
//!
//! 状态机事件由宿主层（session / renderer）从 shell 命令面与 WinEventHook
//! 翻译而来；转移产生的 [`SlotAction`] 由宿主层驱动 win32 层执行。

// Win32 适配：cfg(windows) × native-dock feature（ui-iced 隐含启用）双门控——
// feature 门控使 Windows 上非 ui 构建（默认档）不引入 windows crate 编译开销；
// 其余情形（非 Windows / 未启用 feature）编译同名 no-op 模块，宿主层无需 cfg。
#[cfg(all(windows, feature = "native-dock"))]
pub mod win32;
#[cfg(not(all(windows, feature = "native-dock")))]
#[path = "win32_noop.rs"]
pub mod win32;

/// 槽位 ID：WM 注册表 `native_slots` 的键，单调递增分配。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NativeSlotId(pub u64);

/// 目标原生窗口句柄（HWND 的稳定存储形态；win32 层负责与 `HWND` 指针互转）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NativeHwnd(pub isize);

/// 屏幕物理像素矩形（Win32 `SetWindowPos` / `GetWindowRect` 坐标域）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { x, y, w, h }
    }

    pub fn right(&self) -> i32 {
        self.x + self.w
    }

    pub fn bottom(&self) -> i32 {
        self.y + self.h
    }

    pub fn size(&self) -> Size {
        Size { w: self.w, h: self.h }
    }
}

/// 物理像素尺寸。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Size {
    pub w: i32,
    pub h: i32,
}

impl Size {
    pub fn new(w: i32, h: i32) -> Self {
        Self { w, h }
    }
}

/// 桌面局部逻辑矩形（iced 坐标域，逻辑像素）。
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct LogicalRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// dock 拒绝原因（shell 层提示用）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RejectReason {
    /// UIPI：目标提权，几何写入遭 ACCESS_DENIED（用例 C1）。
    Elevated,
    /// 枚举 + 直传都未命中目标 HWND。
    HwndNotFound,
    /// min-size 装不下且槽位扩张到上限仍不够（用例 C3）。
    MinSizeUnfillable,
    /// 目标独占全屏（用例 C2）。
    ExclusiveFullscreen,
}

impl RejectReason {
    /// shell 层提示文案。
    pub fn message(&self) -> &'static str {
        match self {
            RejectReason::Elevated => "提权窗口无法收编",
            RejectReason::HwndNotFound => "未找到目标原生窗口",
            RejectReason::MinSizeUnfillable => "目标最小尺寸超出可分配空间",
            RejectReason::ExclusiveFullscreen => "独占全屏窗口无法收编",
        }
    }
}

/// 用户拖动判定阈值默认值（逻辑像素；比较前宿主层按 DPI 换算为物理像素）。
pub const USER_DRAG_THRESHOLD_PX: i32 = 32;

/// 钩子线程交付的窗口级事件（win32 层产出、宿主层消费；平台无关数据，
/// no-op 平台同型定义维持 API 面）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeSlotEvent {
    pub hwnd: NativeHwnd,
    pub kind: NativeSlotEventKind,
}

/// 计划 §2 事件清单：生命周期（Destroy）、几何（LocationChange/MoveSizeEnd）、
/// 显示态（MinimizeStart/End）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeSlotEventKind {
    MoveSizeEnd,
    MinimizeStart,
    MinimizeEnd,
    LocationChange,
    Destroy,
}

/// 槽位状态机状态（转移图见 Plan 473 §详细设计 1）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotState {
    /// 已定位目标、待启动 dock。
    Candidate,
    /// 几何/样式写入中。
    Docking,
    /// 已入桌面布局。
    Docked,
    /// 恢复 pre-dock bounds/样式中。
    Undocking,
    /// dock 被拒（驻留供 shell 读取原因，随后从注册表移除）。
    Rejected(RejectReason),
    /// 终态：已脱离注册表（undock 路径已恢复原状；回收路径无需恢复）。
    Restored,
}

/// 状态机事件（宿主层从命令面 / WinEventHook 翻译而来）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotEvent {
    /// 受理 dock，开始写入几何（Candidate → Docking）。
    DockRequested,
    /// 几何写读回成功（Docking → Docked）。
    DockConfirmed,
    /// 写入失败（Docking → Rejected）。
    DockFailed(RejectReason),
    /// 用户把窗口拖离槽位超阈值（Docked → Undocking）。
    UserDraggedAway,
    /// 目标窗口销毁（Docked → 回收槽位并 relayout）。
    TargetClosed,
    /// shell 命令 undock（Docked → Undocking）。
    UndockRequested,
    /// 桌面退出批量恢复（Docked → Undocking）。
    DesktopExiting,
    /// pre-dock 状态恢复完成（Undocking → Restored）。
    RestoreCompleted,
}

/// 转移后宿主层要执行的动作（win32 调用由此驱动）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotAction {
    /// 纯状态转移，无外部动作。
    Idle,
    /// 把窗口摆到槽位矩形；写读回成功后宿主发 [`SlotEvent::DockConfirmed`]。
    SyncGeometry(Rect),
    /// 恢复 pre-dock bounds（样式还原由 win32 层凭 `pre_dock_style` 处理）
    /// 后从注册表移除槽位。
    RestoreAndRemove { bounds: Rect },
    /// 目标已销毁：直接移除槽位并 relayout（无恢复动作）。
    Recycle,
}

/// 原生窗口槽位：与 VirtualWindow 同级参与 WM 布局的不透明单元。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeSlot {
    pub id: NativeSlotId,
    pub hwnd: NativeHwnd,
    pub pid: u32,
    /// 标题缓存（`GetWindowTextW` 快照，宿主层定期刷新）。
    pub title_cache: String,
    /// dock 前窗口矩形（undock / 桌面退出时恢复目标）。
    pub pre_dock_bounds: Rect,
    /// dock 前样式位（GWL_STYLE 快照；win32 层 dock 时填写、undock 时还原）。
    pub pre_dock_style: u32,
    /// 当前槽位矩形（屏幕物理坐标；clamp 扩张后同步更新）。
    pub slot_rect: Rect,
    pub state: SlotState,
    /// 写读回探测到的最小尺寸估计（不可信窗口防御）。
    pub min_size_est: Option<Size>,
}

impl NativeSlot {
    #[allow(clippy::too_many_arguments)]
    pub fn new_candidate(
        id: NativeSlotId,
        hwnd: NativeHwnd,
        pid: u32,
        title: impl Into<String>,
        pre_dock_bounds: Rect,
        slot_rect: Rect,
    ) -> Self {
        Self {
            id,
            hwnd,
            pid,
            title_cache: title.into(),
            pre_dock_bounds,
            pre_dock_style: 0,
            slot_rect,
            state: SlotState::Candidate,
            min_size_est: None,
        }
    }

    /// 状态机转移：返回宿主层要执行的动作；非法转移防御性忽略（返回 Idle）。
    pub fn handle(&mut self, event: SlotEvent) -> SlotAction {
        match self.state {
            SlotState::Candidate if event == SlotEvent::DockRequested => {
                self.state = SlotState::Docking;
                SlotAction::SyncGeometry(self.slot_rect)
            }
            SlotState::Docking => match event {
                SlotEvent::DockConfirmed => {
                    self.state = SlotState::Docked;
                    SlotAction::Idle
                }
                SlotEvent::DockFailed(reason) => {
                    self.state = SlotState::Rejected(reason);
                    SlotAction::Idle
                }
                _ => SlotAction::Idle,
            },
            SlotState::Docked => match event {
                SlotEvent::UserDraggedAway
                | SlotEvent::UndockRequested
                | SlotEvent::DesktopExiting => {
                    self.state = SlotState::Undocking;
                    SlotAction::RestoreAndRemove {
                        bounds: self.pre_dock_bounds,
                    }
                }
                SlotEvent::TargetClosed => {
                    self.state = SlotState::Restored;
                    SlotAction::Recycle
                }
                _ => SlotAction::Idle,
            },
            SlotState::Undocking if event == SlotEvent::RestoreCompleted => {
                self.state = SlotState::Restored;
                SlotAction::Idle
            }
            _ => SlotAction::Idle,
        }
    }

    /// 是否已到终态（宿主层据此从注册表移除）。
    pub fn is_terminal(&self) -> bool {
        matches!(self.state, SlotState::Restored | SlotState::Rejected(_))
    }
}

/// C3 策略：把目标尺寸装进槽位。槽位小于 `min` 时向 `expand_limit`（桌面
/// 可用区上限）内向下/向右扩张；仍装不下 → 拒绝 dock。
/// 返回 `(窗口摆放矩形, 扩张后的槽位矩形)`——docked 窗口填满槽位，二者一致。
pub fn clamp_to_slot(
    target: Size,
    slot: Rect,
    min: Size,
    expand_limit: Rect,
) -> Result<(Rect, Rect), RejectReason> {
    let need = Size::new(target.w.max(min.w), target.h.max(min.h));
    let mut fitted = slot;
    if fitted.w < need.w {
        fitted.w = need.w.min((expand_limit.right() - fitted.x).max(0));
    }
    if fitted.h < need.h {
        fitted.h = need.h.min((expand_limit.bottom() - fitted.y).max(0));
    }
    if fitted.w < need.w || fitted.h < need.h {
        return Err(RejectReason::MinSizeUnfillable);
    }
    Ok((fitted, fitted))
}

/// C4 策略：MOVESSIZE 结束读回的窗口矩形是否判为用户拖走
/// （原点偏离槽位 > 阈值即拖走；轻微偏离按布局抖动忽略）。
/// `threshold_px` 为物理像素（默认逻辑 32px × DPI 缩放）。
pub fn detect_user_drag(cur: Rect, slot: Rect, threshold_px: i32) -> bool {
    (cur.x - slot.x).abs() > threshold_px || (cur.y - slot.y).abs() > threshold_px
}

/// 写读回探测：请求缩到 `requested` 而读回的 `actual` 更大（窗口拒绝缩小）
/// 时，把实际值缓存为 min-size 估计；读回与请求一致则返回 None（无新信息）。
pub fn observe_min_size_estimate(requested: Size, actual: Size) -> Option<Size> {
    (actual.w > requested.w || actual.h > requested.h).then_some(actual)
}

/// 局部逻辑坐标 → 屏幕物理坐标映射器。
/// `origin` = 桌面 OS 窗口客户区左上角的屏幕物理坐标；
/// `scale` = winit per-monitor DPI 缩放比。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CoordMapper {
    pub origin_x: f64,
    pub origin_y: f64,
    pub scale: f64,
}

impl CoordMapper {
    pub fn local_to_screen(&self, r: LogicalRect) -> Rect {
        Rect {
            x: (self.origin_x + r.x as f64 * self.scale).round() as i32,
            y: (self.origin_y + r.y as f64 * self.scale).round() as i32,
            w: (r.w as f64 * self.scale).round() as i32,
            h: (r.h as f64 * self.scale).round() as i32,
        }
    }

    pub fn screen_to_local(&self, r: Rect) -> LogicalRect {
        LogicalRect {
            x: ((r.x as f64 - self.origin_x) / self.scale) as f32,
            y: ((r.y as f64 - self.origin_y) / self.scale) as f32,
            w: (r.w as f64 / self.scale) as f32,
            h: (r.h as f64 / self.scale) as f32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slot() -> NativeSlot {
        NativeSlot::new_candidate(
            NativeSlotId(1),
            NativeHwnd(0x1234),
            4242,
            "fixture",
            Rect::new(100, 100, 800, 600),
            Rect::new(1200, 100, 640, 480),
        )
    }

    // ---- 状态机转移 ----

    #[test]
    fn dock_happy_path() {
        let mut s = slot();
        assert_eq!(
            s.handle(SlotEvent::DockRequested),
            SlotAction::SyncGeometry(Rect::new(1200, 100, 640, 480))
        );
        assert_eq!(s.state, SlotState::Docking);
        assert_eq!(s.handle(SlotEvent::DockConfirmed), SlotAction::Idle);
        assert_eq!(s.state, SlotState::Docked);
        assert!(!s.is_terminal());
    }

    #[test]
    fn dock_rejected_carries_reason() {
        let mut s = slot();
        s.handle(SlotEvent::DockRequested);
        s.handle(SlotEvent::DockFailed(RejectReason::Elevated));
        assert_eq!(s.state, SlotState::Rejected(RejectReason::Elevated));
        assert!(s.is_terminal());
        assert_eq!(RejectReason::Elevated.message(), "提权窗口无法收编");
    }

    #[test]
    fn undock_restores_pre_dock_bounds() {
        let mut s = slot();
        s.handle(SlotEvent::DockRequested);
        s.handle(SlotEvent::DockConfirmed);
        assert_eq!(
            s.handle(SlotEvent::UndockRequested),
            SlotAction::RestoreAndRemove {
                bounds: Rect::new(100, 100, 800, 600)
            }
        );
        assert_eq!(s.state, SlotState::Undocking);
        s.handle(SlotEvent::RestoreCompleted);
        assert_eq!(s.state, SlotState::Restored);
        assert!(s.is_terminal());
    }

    #[test]
    fn user_drag_triggers_undock() {
        let mut s = slot();
        s.handle(SlotEvent::DockRequested);
        s.handle(SlotEvent::DockConfirmed);
        assert!(matches!(
            s.handle(SlotEvent::UserDraggedAway),
            SlotAction::RestoreAndRemove { .. }
        ));
        assert_eq!(s.state, SlotState::Undocking);
    }

    #[test]
    fn desktop_exit_triggers_undock() {
        let mut s = slot();
        s.handle(SlotEvent::DockRequested);
        s.handle(SlotEvent::DockConfirmed);
        assert!(matches!(
            s.handle(SlotEvent::DesktopExiting),
            SlotAction::RestoreAndRemove { .. }
        ));
    }

    #[test]
    fn target_closed_recycles_without_restore() {
        let mut s = slot();
        s.handle(SlotEvent::DockRequested);
        s.handle(SlotEvent::DockConfirmed);
        assert_eq!(s.handle(SlotEvent::TargetClosed), SlotAction::Recycle);
        assert_eq!(s.state, SlotState::Restored);
    }

    #[test]
    fn illegal_transitions_are_ignored() {
        let mut s = slot();
        assert_eq!(s.handle(SlotEvent::DockConfirmed), SlotAction::Idle);
        assert_eq!(s.state, SlotState::Candidate);
        s.handle(SlotEvent::DockRequested);
        assert_eq!(s.handle(SlotEvent::UserDraggedAway), SlotAction::Idle);
        assert_eq!(s.state, SlotState::Docking);
    }

    // ---- C3 clamp 策略 ----

    #[test]
    fn clamp_fits_when_slot_large_enough() {
        let slot = Rect::new(0, 0, 800, 600);
        let (win, fitted) =
            clamp_to_slot(Size::new(400, 300), slot, Size::new(200, 150), Rect::new(0, 0, 1920, 1080))
                .unwrap();
        assert_eq!(win, slot);
        assert_eq!(fitted, slot);
    }

    #[test]
    fn clamp_expands_slot_to_min_size() {
        let slot = Rect::new(0, 0, 300, 200);
        let (win, fitted) =
            clamp_to_slot(Size::new(300, 200), slot, Size::new(500, 400), Rect::new(0, 0, 1920, 1080))
                .unwrap();
        assert_eq!(win, Rect::new(0, 0, 500, 400));
        assert_eq!(fitted, win);
    }

    #[test]
    fn clamp_expansion_capped_by_limit_then_rejects() {
        // 右下角贴近 1920x1080 上限的槽位：扩张空间不足 → 拒绝
        let slot = Rect::new(1500, 900, 300, 150);
        let r = clamp_to_slot(
            Size::new(300, 150),
            slot,
            Size::new(500, 400),
            Rect::new(0, 0, 1920, 1080),
        );
        assert_eq!(r, Err(RejectReason::MinSizeUnfillable));
    }

    // ---- C4 拖动阈值 ----

    #[test]
    fn drag_beyond_threshold_detected() {
        let slot = Rect::new(100, 100, 640, 480);
        assert!(detect_user_drag(Rect::new(133, 100, 640, 480), slot, 32));
        assert!(detect_user_drag(Rect::new(100, 133, 640, 480), slot, 32));
    }

    #[test]
    fn jitter_within_threshold_ignored() {
        let slot = Rect::new(100, 100, 640, 480);
        // 恰在阈值上不算拖走（判定为严格大于）
        assert!(!detect_user_drag(Rect::new(132, 100, 640, 480), slot, 32));
        assert!(!detect_user_drag(Rect::new(95, 105, 640, 480), slot, 32));
    }

    // ---- 坐标映射（注入虚拟桌面 rect）----

    #[test]
    fn local_to_screen_with_dpi_scale() {
        let m = CoordMapper {
            origin_x: 1920.0,
            origin_y: 0.0,
            scale: 1.5,
        };
        let r = m.local_to_screen(LogicalRect {
            x: 10.0,
            y: 20.0,
            w: 100.0,
            h: 50.0,
        });
        assert_eq!(r, Rect::new(1935, 30, 150, 75));
        let back = m.screen_to_local(r);
        assert!((back.x - 10.0).abs() < 1e-4);
        assert!((back.y - 20.0).abs() < 1e-4);
        assert!((back.w - 100.0).abs() < 1e-4);
        assert!((back.h - 50.0).abs() < 1e-4);
    }

    // ---- min-size 估计 ----

    #[test]
    fn min_size_estimate_cached_only_when_clamped() {
        // 请求 400x300 读回 400x300：窗口服从，无新信息
        assert_eq!(
            observe_min_size_estimate(Size::new(400, 300), Size::new(400, 300)),
            None
        );
        // 请求 400x300 读回 360x300：读回更小（窗口比请求还小，非拒绝缩小），无新信息
        assert_eq!(
            observe_min_size_estimate(Size::new(400, 300), Size::new(360, 300)),
            None
        );
        // 请求缩到 40x30 读回 136x39：窗口拒绝缩到请求值 → 实际值即 min-size 估计
        assert_eq!(
            observe_min_size_estimate(Size::new(40, 30), Size::new(136, 39)),
            Some(Size::new(136, 39))
        );
    }
}
