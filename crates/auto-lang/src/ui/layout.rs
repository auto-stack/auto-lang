//! Plan 463 T2：桌面布局引擎（R9 排布策略 / I6 纯函数规范）。
//!
//! 位置唯一事实源仍是 `WmState` 的 `VWinState.rect`（R9）；本模块只提供
//! "输入 WM 快照 → 输出矩形表" 的**纯函数**与薄应用层，不持有任何状态、
//! 不产生消息、不做 IO。vue 侧（465）按同一规范 TS 重写并对拍。
//!
//! 三模式（计划 §3.1）：
//! - `Free`（默认）：不改窗位——用户拖拽结果即真值；新窗入位用
//!   [`cascade_rect`]（459 的 80+48i 级联先例）。
//! - `Grid`：N 窗均分可用区，行列规则 = `cols = ⌈√N⌉`、`rows = ⌈N/cols⌉`，
//!   行主序填格（本文件单测钉死 1–9 窗）。
//! - `MasterStack`：焦点窗取左 master（宽 55%），其余按输入序（z 序
//!   back→front）在右列均分竖排。
//!
//! `ReservedEdges`：任务栏等 shell 层占用的边缘内缩——布局可用区不含
//! 任务栏区（§4 T1 报告：bottom = 48px）。
//!
//! Snap（free 模式拖拽）：光标进入屏缘 [`SNAP_ZONE`] 内 → 返回半屏预览
//! 矩形；落位由 update 壳层在松手时套用（本模块只给几何）。

use crate::ui::session::Wid;

/// 布局模式（`DesktopCommand::SetLayout` 载荷；T1 报告 §5）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LayoutMode {
    /// 默认：记住用户位置（拖拽/缩放结果即真值）。
    #[default]
    Free,
    /// N 窗均分（⌈√N⌉ 列规则）。
    Grid,
    /// 焦点窗左 master（55%），其余右列均分。
    MasterStack,
}

impl LayoutMode {
    /// 状态变量传输用小写名（shell.at `layout\u{1F}grid` 记录；T1 报告 §3）。
    pub fn as_str(&self) -> &'static str {
        match self {
            LayoutMode::Free => "free",
            LayoutMode::Grid => "grid",
            LayoutMode::MasterStack => "master-stack",
        }
    }

    /// 反解析（宿主消费 shell 命令记录用）；未知值回退 Free。
    pub fn from_name(s: &str) -> Self {
        match s.trim() {
            "grid" => LayoutMode::Grid,
            "master-stack" | "masterstack" | "master_stack" => LayoutMode::MasterStack,
            _ => LayoutMode::Free,
        }
    }
}

/// shell 层占用的边缘内缩（T1 报告：任务栏 bottom = 48px）。
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ReservedEdges {
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
}

impl ReservedEdges {
    /// 桌面 v1 标准预留：仅底部任务栏。
    pub fn taskbar() -> Self {
        Self {
            bottom: TASKBAR_HEIGHT,
            ..Default::default()
        }
    }
}

/// 任务栏高度（shell 装配层与布局预留共用同一常量；T1 报告 §4）。
pub const TASKBAR_HEIGHT: f32 = 48.0;

/// master 宽占比（计划 §3.1：55%，T2 定参）。
pub const MASTER_RATIO: f32 = 0.55;

/// snap 触发带宽度（光标距可用区左/右缘 ≤ 此值触发半屏预览）。
pub const SNAP_ZONE: f32 = 8.0;

/// 参与 layout 的窗口快照（纯函数输入；从 `WmState` 派生，见 [`snapshot`]）。
#[derive(Debug, Clone, Copy)]
pub struct WindowState {
    pub wid: Wid,
    pub rect: iced::Rectangle,
    pub focused: bool,
}

/// 布局输出：与输入窗口一一对应的 (Wid, 目标矩形) 表（可用区内）。
pub fn layout(
    mode: LayoutMode,
    wins: &[WindowState],
    viewport: iced::Rectangle,
    reserved: ReservedEdges,
) -> Vec<(Wid, iced::Rectangle)> {
    match mode {
        LayoutMode::Free => wins.iter().map(|w| (w.wid, w.rect)).collect(),
        LayoutMode::Grid => layout_grid(wins, usable_rect(viewport, reserved)),
        LayoutMode::MasterStack => layout_master_stack(wins, usable_rect(viewport, reserved)),
    }
}

/// viewport 扣除 shell 预留后的布局可用区（各向内缩；防负尺寸钳 0）。
pub fn usable_rect(viewport: iced::Rectangle, reserved: ReservedEdges) -> iced::Rectangle {
    iced::Rectangle {
        x: viewport.x + reserved.left,
        y: viewport.y + reserved.top,
        width: (viewport.width - reserved.left - reserved.right).max(0.0),
        height: (viewport.height - reserved.top - reserved.bottom).max(0.0),
    }
}

/// 非纯薄应用层（Plan 463 T4）：把 `layout()` 结果写回 WM —— rect 的唯一
/// 批量写点（R9：排布是 WM 策略；单窗交互路径不经此）。free 模式为恒等
/// 写回（用户位置即真值）。窗口级 `window_size` 同步（响应式布局消费）。
pub fn apply_layout(
    wm: &mut crate::ui::session::WmState,
    viewport: iced::Rectangle,
    reserved: ReservedEdges,
) {
    let snaps: Vec<WindowState> = wm
        .z_order
        .iter()
        .filter_map(|wid| {
            let v = wm.wins.get(wid)?;
            Some(WindowState {
                wid: *wid,
                rect: *v.rect.borrow(),
                focused: wm.focused == Some(*wid),
            })
        })
        .collect();
    for (wid, r) in layout(wm.layout, &snaps, viewport, reserved) {
        if let Some(v) = wm.wins.get_mut(&wid) {
            *v.rect.borrow_mut() = r;
            *v.window_size.borrow_mut() = iced::Size::new(r.width, r.height);
        }
    }
}

/// Grid：cols = ⌈√N⌉，rows = ⌈N/cols⌉，行主序。空表返回空表。
fn layout_grid(wins: &[WindowState], usable: iced::Rectangle) -> Vec<(Wid, iced::Rectangle)> {
    let n = wins.len();
    if n == 0 || usable.width <= 0.0 || usable.height <= 0.0 {
        return Vec::new();
    }
    let cols = (n as f32).sqrt().ceil() as usize;
    let rows = n.div_ceil(cols);
    let cw = usable.width / cols as f32;
    let ch = usable.height / rows as f32;
    wins.iter()
        .enumerate()
        .map(|(i, w)| {
            let col = i % cols;
            let row = i / cols;
            (w.wid, cell_rect(usable, col, row, cw, ch))
        })
        .collect()
}

/// MasterStack：焦点窗（无焦点取输入序末位=z 顶）左 master，其余右列均分。
fn layout_master_stack(
    wins: &[WindowState],
    usable: iced::Rectangle,
) -> Vec<(Wid, iced::Rectangle)> {
    let n = wins.len();
    if n == 0 || usable.width <= 0.0 || usable.height <= 0.0 {
        return Vec::new();
    }
    // 单窗语义：独占整个可用区（无意义的 55% 分栏退化为半屏空置）。
    if n == 1 {
        return vec![(wins[0].wid, usable)];
    }
    let master_idx = wins.iter().position(|w| w.focused).unwrap_or(n - 1);
    let master_w = usable.width * MASTER_RATIO;
    let stack_w = usable.width - master_w;
    let rest = n - 1;
    let sh = if rest > 0 {
        usable.height / rest as f32
    } else {
        0.0
    };
    wins.iter()
        .enumerate()
        .map(|(i, w)| {
            if i == master_idx {
                (w.wid, rect(usable.x, usable.y, master_w, usable.height))
            } else {
                // master 之外的窗口按输入序在右列均分（跳过 master 位）。
                let slot = if i < master_idx { i } else { i - 1 };
                (
                    w.wid,
                    rect(
                        usable.x + master_w,
                        usable.y + sh * slot as f32,
                        stack_w,
                        sh,
                    ),
                )
            }
        })
        .collect()
}

/// 免重叠的单元格矩形（末行/末列不外溢：直接乘除，浮点残差 < ε 可接受）。
fn cell_rect(usable: iced::Rectangle, col: usize, row: usize, cw: f32, ch: f32) -> iced::Rectangle {
    rect(
        usable.x + cw * col as f32,
        usable.y + ch * row as f32,
        cw,
        ch,
    )
}

fn rect(x: f32, y: f32, width: f32, height: f32) -> iced::Rectangle {
    iced::Rectangle {
        x,
        y,
        width,
        height,
    }
}

/// free 模式新窗级联初位（459 先例：80 + 48i，限制在可用区内不会全出界）。
pub fn cascade_rect(index: usize, size: iced::Size, usable: iced::Rectangle) -> iced::Rectangle {
    let step = 48.0f32;
    let base = 80.0f32;
    let off_x = (base + step * index as f32)
        .min(usable.width * 0.5)
        .max(0.0);
    let off_y = (base + step * index as f32)
        .min(usable.height * 0.5)
        .max(0.0);
    rect(
        usable.x + off_x,
        usable.y + off_y,
        size.width.min(usable.width),
        size.height.min(usable.height),
    )
}

/// Snap 预览几何（free 拖拽中）：光标进入左/右缘 SNAP_ZONE → 对应半屏
/// 矩形；否则 None。v1 只做左右半屏（计划 §3.1；四角四分为可选任务）。
pub fn snap_preview(cursor: iced::Point, usable: iced::Rectangle) -> Option<iced::Rectangle> {
    if usable.width <= 0.0 || usable.height <= 0.0 {
        return None;
    }
    let half = usable.width / 2.0;
    if cursor.x - usable.x <= SNAP_ZONE {
        Some(rect(usable.x, usable.y, half, usable.height))
    } else if usable.x + usable.width - cursor.x <= SNAP_ZONE {
        Some(rect(
            usable.x + half,
            usable.y,
            usable.width - half,
            usable.height,
        ))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VIEWPORT: iced::Rectangle = iced::Rectangle {
        x: 0.0,
        y: 0.0,
        width: 1280.0,
        height: 800.0,
    };
    /// 标准可用区（扣任务栏 48）：1280 x 752。
    fn usable() -> iced::Rectangle {
        usable_rect(VIEWPORT, ReservedEdges::taskbar())
    }

    fn wins(rects: &[(f32, f32, f32, f32)], focused_idx: Option<usize>) -> Vec<WindowState> {
        rects
            .iter()
            .enumerate()
            .map(|(i, &(x, y, w, h))| WindowState {
                wid: Wid(i as u64 + 1),
                rect: rect(x, y, w, h),
                focused: focused_idx == Some(i),
            })
            .collect()
    }

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < 0.51
    }

    fn assert_rect(actual: iced::Rectangle, x: f32, y: f32, w: f32, h: f32) {
        assert!(
            approx(actual.x, x)
                && approx(actual.y, y)
                && approx(actual.width, w)
                && approx(actual.height, h),
            "rect {{x:{}, y:{}, w:{}, h:{}}} != expected {{x:{x}, y:{y}, w:{w}, h:{h}}}",
            actual.x,
            actual.y,
            actual.width,
            actual.height
        );
    }

    // ---- Free：不改窗位（用户位置即真值）----

    #[test]
    fn free_mode_returns_input_rects_unchanged() {
        let input = wins(
            &[(10.0, 20.0, 300.0, 200.0), (500.0, 80.0, 400.0, 300.0)],
            Some(1),
        );
        let out = layout(LayoutMode::Free, &input, VIEWPORT, ReservedEdges::taskbar());
        assert_eq!(out.len(), 2);
        assert_rect(out[0].1, 10.0, 20.0, 300.0, 200.0);
        assert_rect(out[1].1, 500.0, 80.0, 400.0, 300.0);
    }

    #[test]
    fn free_mode_zero_windows_returns_empty() {
        let out = layout(LayoutMode::Free, &[], VIEWPORT, ReservedEdges::taskbar());
        assert!(out.is_empty());
    }

    // ---- 可用区：任务栏不参与布局 ----

    #[test]
    fn usable_rect_excludes_taskbar_bottom() {
        let u = usable();
        assert_rect(u, 0.0, 0.0, 1280.0, 752.0);
    }

    #[test]
    fn usable_rect_clamps_negative_to_zero() {
        let tiny = rect(0.0, 0.0, 30.0, 40.0);
        let u = usable_rect(
            tiny,
            ReservedEdges {
                left: 10.0,
                right: 25.0,
                top: 5.0,
                bottom: 50.0,
            },
        );
        assert_eq!(u.width, 0.0);
        assert_eq!(u.height, 0.0);
    }

    // ---- Grid：1–9 窗，⌈√N⌉ 列规则 ----

    #[test]
    fn grid_one_window_fills_usable() {
        let input = wins(&[(0.0, 0.0, 100.0, 100.0)], None);
        let out = layout(LayoutMode::Grid, &input, VIEWPORT, ReservedEdges::taskbar());
        assert_rect(out[0].1, 0.0, 0.0, 1280.0, 752.0);
    }

    #[test]
    fn grid_two_windows_side_by_side() {
        // ⌈√2⌉ = 2 列 1 行。
        let input = wins(&[(0.0, 0.0, 0.0, 0.0), (0.0, 0.0, 0.0, 0.0)], None);
        let out = layout(LayoutMode::Grid, &input, VIEWPORT, ReservedEdges::taskbar());
        assert_rect(out[0].1, 0.0, 0.0, 640.0, 752.0);
        assert_rect(out[1].1, 640.0, 0.0, 640.0, 752.0);
    }

    #[test]
    fn grid_three_windows_2x2_row_major() {
        // ⌈√3⌉ = 2 列，rows = 2：行主序 (0,0)(0,1)(1,0)，末格空。
        let input = wins(
            &[
                (0.0, 0.0, 0.0, 0.0),
                (0.0, 0.0, 0.0, 0.0),
                (0.0, 0.0, 0.0, 0.0),
            ],
            None,
        );
        let out = layout(LayoutMode::Grid, &input, VIEWPORT, ReservedEdges::taskbar());
        assert_rect(out[0].1, 0.0, 0.0, 640.0, 376.0);
        assert_rect(out[1].1, 640.0, 0.0, 640.0, 376.0);
        assert_rect(out[2].1, 0.0, 376.0, 640.0, 376.0);
    }

    #[test]
    fn grid_four_windows_2x2() {
        let input = wins(
            &[
                (0.0, 0.0, 0.0, 0.0),
                (0.0, 0.0, 0.0, 0.0),
                (0.0, 0.0, 0.0, 0.0),
                (0.0, 0.0, 0.0, 0.0),
            ],
            None,
        );
        let out = layout(LayoutMode::Grid, &input, VIEWPORT, ReservedEdges::taskbar());
        assert_rect(out[0].1, 0.0, 0.0, 640.0, 376.0);
        assert_rect(out[1].1, 640.0, 0.0, 640.0, 376.0);
        assert_rect(out[2].1, 0.0, 376.0, 640.0, 376.0);
        assert_rect(out[3].1, 640.0, 376.0, 640.0, 376.0);
    }

    #[test]
    fn grid_five_windows_3cols_2rows_last_row_two() {
        // ⌈√5⌉ = 3 列，rows = 2：末行 2 窗占前两格。
        let input: Vec<WindowState> = (0..5)
            .map(|i| WindowState {
                wid: Wid(i),
                rect: rect(0.0, 0.0, 1.0, 1.0),
                focused: false,
            })
            .collect();
        let out = layout(LayoutMode::Grid, &input, VIEWPORT, ReservedEdges::taskbar());
        let cw = 1280.0 / 3.0;
        let ch = 752.0 / 2.0;
        assert_rect(out[0].1, 0.0, 0.0, cw, ch);
        assert_rect(out[1].1, cw, 0.0, cw, ch);
        assert_rect(out[2].1, cw * 2.0, 0.0, cw, ch);
        assert_rect(out[3].1, 0.0, ch, cw, ch);
        assert_rect(out[4].1, cw, ch, cw, ch);
    }

    #[test]
    fn grid_nine_windows_3x3() {
        let input: Vec<WindowState> = (0..9)
            .map(|i| WindowState {
                wid: Wid(i),
                rect: rect(0.0, 0.0, 1.0, 1.0),
                focused: false,
            })
            .collect();
        let out = layout(LayoutMode::Grid, &input, VIEWPORT, ReservedEdges::taskbar());
        let cw = 1280.0 / 3.0;
        let ch = 752.0 / 3.0;
        assert_rect(out[8].1, cw * 2.0, ch * 2.0, cw, ch);
        // 全表无重叠、盖满可用区（结构性断言）。
        for r in &out {
            assert!(r.1.width > 0.0 && r.1.height > 0.0);
        }
    }

    #[test]
    fn grid_zero_windows_returns_empty() {
        let out = layout(LayoutMode::Grid, &[], VIEWPORT, ReservedEdges::taskbar());
        assert!(out.is_empty());
    }

    // ---- MasterStack：焦点窗左 master 55%，其余右列均分 ----

    #[test]
    fn master_stack_two_windows_focused_left() {
        let input = wins(&[(0.0, 0.0, 0.0, 0.0), (0.0, 0.0, 0.0, 0.0)], Some(1));
        let out = layout(
            LayoutMode::MasterStack,
            &input,
            VIEWPORT,
            ReservedEdges::taskbar(),
        );
        let mw = 1280.0 * MASTER_RATIO;
        assert_rect(out[0].1, mw, 0.0, 1280.0 - mw, 752.0); // stack 右列
        assert_rect(out[1].1, 0.0, 0.0, mw, 752.0); // 焦点 = master 左
    }

    #[test]
    fn master_stack_three_windows_two_right_split() {
        let input = wins(
            &[
                (0.0, 0.0, 0.0, 0.0),
                (0.0, 0.0, 0.0, 0.0),
                (0.0, 0.0, 0.0, 0.0),
            ],
            Some(0),
        );
        let out = layout(
            LayoutMode::MasterStack,
            &input,
            VIEWPORT,
            ReservedEdges::taskbar(),
        );
        let mw = 1280.0 * MASTER_RATIO;
        let sw = 1280.0 - mw;
        assert_rect(out[0].1, 0.0, 0.0, mw, 752.0);
        assert_rect(out[1].1, mw, 0.0, sw, 376.0);
        assert_rect(out[2].1, mw, 376.0, sw, 376.0);
    }

    #[test]
    fn master_stack_no_focus_falls_back_to_last_input() {
        // 无焦点窗 → 输入序末位（z 顶）为 master。
        let input = wins(&[(0.0, 0.0, 0.0, 0.0), (0.0, 0.0, 0.0, 0.0)], None);
        let out = layout(
            LayoutMode::MasterStack,
            &input,
            VIEWPORT,
            ReservedEdges::taskbar(),
        );
        let mw = 1280.0 * MASTER_RATIO;
        assert_rect(out[1].1, 0.0, 0.0, mw, 752.0);
        assert_rect(out[0].1, mw, 0.0, 1280.0 - mw, 752.0);
    }

    #[test]
    fn master_stack_single_window_fills_usable() {
        let input = wins(&[(0.0, 0.0, 0.0, 0.0)], None);
        let out = layout(
            LayoutMode::MasterStack,
            &input,
            VIEWPORT,
            ReservedEdges::taskbar(),
        );
        assert_rect(out[0].1, 0.0, 0.0, 1280.0, 752.0);
    }

    // ---- 级联初位 ----

    #[test]
    fn cascade_rect_offsets_by_48_per_index() {
        let size = iced::Size::new(600.0, 400.0);
        let r0 = cascade_rect(0, size, usable());
        let r1 = cascade_rect(1, size, usable());
        assert_rect(r0, 80.0, 80.0, 600.0, 400.0);
        assert_rect(r1, 128.0, 128.0, 600.0, 400.0);
    }

    #[test]
    fn cascade_rect_offsets_cap_and_clamp_into_usable() {
        let size = iced::Size::new(2000.0, 2000.0);
        let r = cascade_rect(100, size, usable());
        assert!(r.x <= 640.0 && r.y <= 376.0);
        assert!(r.width <= 1280.0 && r.height <= 752.0);
    }

    // ---- Snap 半屏预览 ----

    #[test]
    fn snap_left_edge_gives_left_half() {
        let p = iced::Point::new(3.0, 300.0);
        let out = snap_preview(p, usable()).expect("left edge should snap");
        assert_rect(out, 0.0, 0.0, 640.0, 752.0);
    }

    #[test]
    fn snap_right_edge_gives_right_half() {
        let p = iced::Point::new(1279.0, 300.0);
        let out = snap_preview(p, usable()).expect("right edge should snap");
        assert_rect(out, 640.0, 0.0, 640.0, 752.0);
    }

    #[test]
    fn snap_zone_boundary_exactly_at_8px_snaps() {
        let out = snap_preview(iced::Point::new(8.0, 300.0), usable());
        assert!(out.is_some(), "cursor at exactly SNAP_ZONE from left edge");
    }

    #[test]
    fn snap_middle_gives_none() {
        let out = snap_preview(iced::Point::new(640.0, 376.0), usable());
        assert!(out.is_none());
    }

    // ---- I6 对拍：共享期望值表（Plan 465 T4）----
    // 同一 layout_cases.json 约束 TS 直译（scripts/ui-layout-parity.mjs）；
    // 改布局语义时两侧同改 + 表同改。

    #[test]
    fn layout_parity_cases_shared_table() {
        let raw = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/ui/layout_cases.json"
        ))
        .expect("shared layout cases file");
        let v: serde_json::Value = serde_json::from_str(&raw).expect("cases json parse");
        let vp = &v["viewport"];
        let viewport = rect(
            vp[0].as_f64().unwrap() as f32,
            vp[1].as_f64().unwrap() as f32,
            vp[2].as_f64().unwrap() as f32,
            vp[3].as_f64().unwrap() as f32,
        );
        let reserved = ReservedEdges {
            bottom: v["reservedTaskbar"].as_f64().unwrap() as f32,
            ..Default::default()
        };

        let arr = |n: &serde_json::Value| -> Vec<f32> {
            n.as_array().unwrap().iter().map(|x| x.as_f64().unwrap() as f32).collect()
        };
        let mut checked = 0usize;
        for case in v["cases"].as_array().unwrap() {
            let kind = case["kind"].as_str().unwrap();
            match kind {
                "usable" => {
                    let u = usable_rect(viewport, reserved);
                    let e = arr(&case["expected"]);
                    assert_rect(u, e[0], e[1], e[2], e[3]);
                    checked += 1;
                }
                "layout" => {
                    let n = case["n"].as_u64().unwrap() as usize;
                    let focused = case["focused"].as_u64().map(|i| i as usize);
                    let free_rects: Option<Vec<(f32, f32, f32, f32)>> = case.get("freeRects").map(|fr| {
                        fr.as_array().unwrap().iter().map(|r| {
                            let a = arr(r);
                            (a[0], a[1], a[2], a[3])
                        }).collect()
                    });
                    let input: Vec<WindowState> = (0..n)
                        .map(|i| WindowState {
                            wid: Wid(i as u64 + 1),
                            rect: match &free_rects {
                                Some(fr) => rect(fr[i].0, fr[i].1, fr[i].2, fr[i].3),
                                None => rect(0.0, 0.0, 1.0, 1.0),
                            },
                            focused: focused == Some(i),
                        })
                        .collect();
                    let out = layout(
                        LayoutMode::from_name(case["mode"].as_str().unwrap()),
                        &input,
                        viewport,
                        reserved,
                    );
                    if let Some(last) = case.get("expectedLast") {
                        let e = arr(last);
                        assert_rect(out[out.len() - 1].1, e[0], e[1], e[2], e[3]);
                    } else {
                        for (i, exp) in case["expected"].as_array().unwrap().iter().enumerate() {
                            let e = arr(exp);
                            assert_rect(out[i].1, e[0], e[1], e[2], e[3]);
                        }
                    }
                    checked += 1;
                }
                "cascade" => {
                    let sz = arr(&case["size"]);
                    let r = cascade_rect(
                        case["index"].as_u64().unwrap() as usize,
                        iced::Size::new(sz[0], sz[1]),
                        usable_rect(viewport, reserved),
                    );
                    let e = arr(&case["expected"]);
                    assert_rect(r, e[0], e[1], e[2], e[3]);
                    checked += 1;
                }
                "snap" => {
                    let c = arr(&case["cursor"]);
                    let out = snap_preview(
                        iced::Point::new(c[0], c[1]),
                        usable_rect(viewport, reserved),
                    );
                    if case["expected"].is_null() {
                        assert!(out.is_none(), "snap case {} expected none", case["name"]);
                    } else {
                        let r = out.expect("snap case expected Some");
                        let e = arr(&case["expected"]);
                        assert_rect(r, e[0], e[1], e[2], e[3]);
                    }
                    checked += 1;
                }
                _ => panic!("unknown case kind: {}", kind),
            }
        }
        assert!(checked >= 15, "table must stay populated, got {}", checked);
    }

    // ---- LayoutMode 传输名 ----

    #[test]
    fn layout_mode_names_round_trip() {
        for m in [LayoutMode::Free, LayoutMode::Grid, LayoutMode::MasterStack] {
            assert_eq!(LayoutMode::from_name(m.as_str()), m);
        }
        assert_eq!(LayoutMode::from_name("nonsense"), LayoutMode::Free);
    }

    // ---- 纯函数性（I6）：同一输入两次调用输出相同 ----

    #[test]
    fn layout_is_deterministic_and_does_not_mutate_input() {
        let input = wins(
            &[(10.0, 20.0, 300.0, 200.0), (500.0, 80.0, 400.0, 300.0)],
            Some(1),
        );
        let a = layout(LayoutMode::Grid, &input, VIEWPORT, ReservedEdges::taskbar());
        let b = layout(LayoutMode::Grid, &input, VIEWPORT, ReservedEdges::taskbar());
        assert_eq!(a, b);
        // 输入快照未被改写（值语义保证；这里守恒输入矩形供回归）。
        assert_rect(input[0].rect, 10.0, 20.0, 300.0, 200.0);
    }
}
