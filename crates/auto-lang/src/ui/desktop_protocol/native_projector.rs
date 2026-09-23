// Plan 020 T-04 —— native queue 投影臂：`View<C::Msg>` → DrawList 投影器。
// PLAN-033 T-06：更名 RqProjector（NativeProjector → RqProjector——单投影
// 器统一，P020-D1 销账）；原"与解释态 AppProjector 并存"已收束——AppProjector
// 退役（T-04），VM 轨经物化 View 入本投影器（a2r/解释同律）。
//
// §5.1 定案（策略 B，运行期 View 投影）：a2r 编译 Component 的 `view()`
// 产物是**全物化 IR**——prop 已解析、handler 已是 `M` 值、条件/循环/插值
// 在构建期求值完毕，投影器即 `View` 的第四消费后端（iced/VTree 之外
// 的协议后端）。与解释态 [`super::client_runtime::AppProjector`] 并存
// （双轨 v1；解释态改写为 View 基的"双轨统一"记债不在本计划）：
//
// - 布局 = 块流 walker `layout_view_block`（镜像 client_runtime
//   `layout_block` 语义，节点面换 `View` 枚举——~150 行重复记债）；
// - 样式 = typed `Style`(StyleClass) → [`super::client_runtime::NodeStyle`]
//   适配器（盒模复用 `style::BoxLayout::from_style`，装饰/对齐/排版逐类
//   直填——不复刻字符串解析）；
// - 命中表 = `Vec<(WRect, C::Msg)>`（物化消息 clone 派发，`component.on`）；
// - 覆盖门 = [`super::coverage::scan_native_view`] ×
//   [`super::coverage::Coverage::native_queue_set`]——v1 覆盖集 = text/
//   button + col/row/container/list + 布局样式子集；payload 族显式
//   not-yet，启动门拒绝（AC-04，禁静默错绘）；渲染期动态分支遭遇未覆盖
//   变体 = 占位盒 + `uncovered_seen` 留痕（同 I3 纪律）；
// - tick = `Component::tick_interval_ms/tick_msg` 配方经
//   `FrameSource::poll_tick` 泵驱动（devtools 同源）。
//
// v1 边界（随注）：L3 StateSnapshot 注入 not-yet（T-03 像素臂同册——
// native 组件无字段写回路径）；右键 handler 不路由（解释态 queue 臂同
// 边界）；键盘/滚轮不路由（覆盖集无 input 族）。

use std::time::Instant;

use super::client_runtime::{
    dim_if, measure_text, NodeStyle, bg, button_bg, input_bg, input_border, image_placeholder,
    placeholder_fg, primary_fill, accent_fill, progress_track, text_fg, BUTTON_H, BUTTON_MIN_W,
    BUTTON_PAD, DISABLED_ALPHA, LINE_H_FACTOR, MARGIN, TEXT_SIZE,
};
// PLAN-674 T-01：codeeditor 投影臂/键入回传面（CODE_EDITORS 注册表 +
// core handle_input 全键面——ui-iced ⊇ code-editor，无 cfg 面）。
use crate::ui::code_editor as ce;
use super::coverage::{self, Coverage, Verdict};
use super::endpoint::FrameSource;
use super::message::{ControlMsg, DrawList, DrawOp, ImageFit, InputMsg, MouseButton, Rgba8, WRect};
use crate::ui::component::Component;
use crate::ui::style::{Color, Style, StyleClass};
use crate::ui::view::{
    PopoverAnchor, PopoverPlacement, ScrollCallback, ScrollMetrics, SelectCallback, TabsPosition,
    TabsSelectCallback, TabsVariant, View,
};

/// 输入框几何（client_runtime 私有常量的 native 同值镜像——视觉规格
/// 镜像解释态，参数面各自持有）。
const INPUT_H: f32 = 32.0;
const INPUT_PAD: f32 = 10.0;
/// 聚焦描边色（解释态 `resolve_color("blue-500")` 的常量形态）。
/// checkbox/radio 勾选盒标签与盒体的间距。
const CHECK_LABEL_GAP: f32 = 6.0;
/// slider 几何（轨道厚 / knob 边 / 命中带高——v1 常量档，样式类可覆高宽）。
const SLIDER_H: f32 = 20.0;
const SLIDER_TRACK_H: f32 = 4.0;
const SLIDER_KNOB: f32 = 12.0;
/// popover 面板几何（PLAN-029 T-04 D3）：内容可用宽（a2r 缺省 w-72 ≈
/// 288px 同刻度）/ 锚-面板间距 / 视口边距。
const POP_W: f32 = 288.0;
const POP_GAP: f32 = 6.0;
const POP_MARGIN: f32 = 8.0;
/// Modal scrim 半透明全屏层（RGB 0 + alpha 120）。
const POP_SCRIM: Rgba8 = Rgba8::new(0, 0, 0, 120);
/// 面板底/边（select 选项列同视觉族）——PLAN-679：Surface/Border 槽
/// （shadcn popover 面），随主题双盘。
fn pop_bg() -> Rgba8 {
    super::client_runtime::semantic_rgb(crate::ui::style::Color::Surface)
}
fn pop_border() -> Rgba8 {
    super::client_runtime::border_slot()
}

/// 分型命中表项（PLAN-025 T-01 D1/D3 定形态）：零参物化消息直入；payload
/// 族携派发材料（输入闭环身份/slider 几何/…——随覆盖爬坡扩臂）。
#[derive(Clone)]
enum HitEntry<M: Clone + std::fmt::Debug> {
    /// 零参物化消息（button click / checkbox·radio toggle——handler 在场 =
    /// handler 拥有状态变更，native 无字段写回路径，自动翻转不可达）。
    Msg { rect: WRect, msg: M },
    /// input/textarea：点击聚焦（编辑闭环走聚焦槽位，不经命中表派发）。
    /// `value` = 布局期视图值（聚焦时 buffer 初始化源，D2）；`slot` =
    /// 聚焦身份（Input/Textarea 合一计数，D1-A）；`on_change` = 键入
    /// 回写物化消息（None = 只显不编——登记省略）。
    /// PLAN-674 T-01：codeeditor 复用本槽位族——`editor` = CODE_EDITORS
    /// 注册表存储键（storage_key(key)；None = 平面 input/textarea）。
    /// 编辑器键入不走 input_buffer，走 core `handle_input` 全键面
    /// （光标/选区/undo/IME 引擎态在 core），INPUT_TEXT 通道携带
    /// 键入后全文（on_change 零参派发前代写）。编辑器恒登记命中
    /// （聚焦/光标定位即交互面——on_change 缺席不省略，与平面 input
    /// 的"登记省略"差分随注）。
    Input { rect: WRect, value: String, on_change: Option<M>, slot: usize, editor: Option<String> },
    /// slider：轨道点击 → 几何换算 f32（min..=max 线性 + step 取整）→
    /// 回调物化派发（T-01 附带定案：v1 点击定位，拖拽 not-yet；
    /// PLAN-661 T-02：on_change 转 `Option<SliderChangeHandler>`——None
    /// 不登记命中）。
    Slider {
        rect: WRect,
        min: f32,
        max: f32,
        step: Option<f32>,
        on_change: Option<crate::ui::view::SliderChangeHandler<M>>,
    },
    /// select 闭态盒：点击开（无消息派发——开合是投影器侧状态，D3）。
    SelectBox { rect: WRect, slot: usize },
    /// select 开态选项项（消息布局期已物化——`SelectCallback.call`）。
    SelectOption { rect: WRect, msg: M },
    /// scrollable 视口：滚轮派发（offset' = clamp(快照+delta)——T-01 D5；
    /// on_scroll 不在场不登记——I3 留痕面）。
    Scroll {
        rect: WRect,
        offset: (f32, f32),
        viewport: (f32, f32),
        content: (f32, f32),
        callback: ScrollCallback<M>,
    },
    /// PLAN-029 T-04（D3）：popover 开态全屏 catcher（rect = 视口——开态
    /// 任何落点即关；先于面板项登记 → rev 序面板项胜，锚/主块被吞）。
    /// Esc 同臂派发（key=27 且在场）。
    PopoverDismiss { rect: WRect, on_dismiss: Option<M> },
    /// PLAN-032 T-03（D2）：tabs 托盘项——点击 → TabsSelectCallback::
    /// call(index) 物化消息（VM 轨首参 = value 串在回调内包装；
    /// on_select 缺席不登记——受控语义，convert_tabs 契约）。
    TabSelect { rect: WRect, index: usize, on_select: TabsSelectCallback<M> },
}

impl<M: Clone + std::fmt::Debug> HitEntry<M> {
    /// PLAN-678 T-02：命中矩形统一可变访问（根平移通道唯一消费——
    /// 全变体携 rect 的枚举形状即本通道的契约面）。
    fn rect_mut(&mut self) -> &mut WRect {
        match self {
            HitEntry::Msg { rect, .. }
            | HitEntry::Input { rect, .. }
            | HitEntry::Slider { rect, .. }
            | HitEntry::SelectBox { rect, .. }
            | HitEntry::SelectOption { rect, .. }
            | HitEntry::Scroll { rect, .. }
            | HitEntry::PopoverDismiss { rect, .. }
            | HitEntry::TabSelect { rect, .. } => rect,
        }
    }
}

/// select 开态覆盖序记录（主块渲染后统一追加——D3：DrawList paint
/// order 天然置顶，无需 overlay 协议语义）。
struct SelectOverlay<M: Clone + std::fmt::Debug> {
    /// 闭态盒 rect（选项列贴盒底同宽）。
    rect: WRect,
    options: Vec<String>,
    selected_index: Option<usize>,
    on_select: Option<SelectCallback<M>>,
}

/// PLAN-029 T-04（D3）：popover 开态覆盖序记录（select 同序——主块渲染
/// 后追加面板 ops + 命中项；开合零投影器状态，open 随帧）。
struct PopoverOverlay<M: Clone + std::fmt::Debug> {
    /// 面板外框（几何已在登记期定：锚 + placement + 视口翻转）。
    rect: WRect,
    /// Modal = scrim 半透明全屏 Quad 先于面板 ops。
    modal: bool,
    on_dismiss: Option<M>,
    /// 面板子树渲染产物（走线期临时 ctx 收纳 + 原点平移完成）。
    ops: Vec<DrawOp>,
    hits: Vec<HitEntry<M>>,
}

/// 几何归一后的锚（Widget = 主流量 laid rect；Point = 视口坐标直用，
/// BottomStart 语义——a2r 缺省先例）。
enum PopoverAnchorSite {
    Widget(WRect),
    Point { x: f32, y: f32 },
}

impl<M: Clone + std::fmt::Debug> HitEntry<M> {
    fn rect(&self) -> &WRect {
        match self {
            HitEntry::Msg { rect, .. }
            | HitEntry::Input { rect, .. }
            | HitEntry::Slider { rect, .. }
            | HitEntry::SelectBox { rect, .. }
            | HitEntry::SelectOption { rect, .. }
            | HitEntry::Scroll { rect, .. }
            | HitEntry::PopoverDismiss { rect, .. }
            | HitEntry::TabSelect { rect, .. } => rect,
        }
    }

    /// 原点平移（popover 面板子树走线期 @0,0 渲染 → 平移到面板原点）。
    fn shifted(mut self, dx: f32, dy: f32) -> Self {
        fn move_rect(r: &mut WRect, dx: f32, dy: f32) {
            r.x += dx;
            r.y += dy;
        }
        match &mut self {
            HitEntry::Msg { rect, .. }
            | HitEntry::Input { rect, .. }
            | HitEntry::Slider { rect, .. }
            | HitEntry::SelectBox { rect, .. }
            | HitEntry::SelectOption { rect, .. }
            | HitEntry::Scroll { rect, .. }
            | HitEntry::PopoverDismiss { rect, .. }
            | HitEntry::TabSelect { rect, .. } => move_rect(rect, dx, dy),
        }
        self
    }
}

/// PLAN-674 T-01：协议键码（宿主 VK 形态——on_input 既有 8/27 同码制）
/// → 编辑器非打印键。打印字符走 CharTyped 字符道，未列键位（F 族/
/// 修饰裸键等）不映射（编辑器不消费）。
fn editor_key_of(vk: u32) -> Option<ce::EditorKey> {
    Some(match vk {
        13 => ce::EditorKey::Enter,
        8 => ce::EditorKey::Backspace,
        9 => ce::EditorKey::Tab,
        46 => ce::EditorKey::Delete,
        36 => ce::EditorKey::Home,
        35 => ce::EditorKey::End,
        33 => ce::EditorKey::PageUp,
        34 => ce::EditorKey::PageDown,
        37 => ce::EditorKey::Left,
        38 => ce::EditorKey::Up,
        39 => ce::EditorKey::Right,
        40 => ce::EditorKey::Down,
        _ => return None,
    })
}

/// DrawOp 原点平移（popover 面板子树走线产物同移——全部坐标字段绝对制）。
fn shift_draw_op(op: &DrawOp, dx: f32, dy: f32) -> DrawOp {
    match op {
        DrawOp::Quad { rect, color } => DrawOp::Quad {
            rect: WRect::new(rect.x + dx, rect.y + dy, rect.w, rect.h),
            color: *color,
        },
        DrawOp::QuadR { rect, color, radius } => DrawOp::QuadR {
            rect: WRect::new(rect.x + dx, rect.y + dy, rect.w, rect.h),
            color: *color,
            radius: *radius,
        },
        DrawOp::Text { x, y, size, line_height, color, text } => DrawOp::Text {
            x: x + dx,
            y: y + dy,
            size: *size,
            line_height: *line_height,
            color: *color,
            text: text.clone(),
        },
        DrawOp::TextStyled { x, y, size, line_height, color, weight, italic, text } => {
            DrawOp::TextStyled {
                x: x + dx,
                y: y + dy,
                size: *size,
                line_height: *line_height,
                color: *color,
                weight: *weight,
                italic: *italic,
                text: text.clone(),
            }
        }
        DrawOp::Scissor { rect } => DrawOp::Scissor {
            rect: WRect::new(rect.x + dx, rect.y + dy, rect.w, rect.h),
        },
        DrawOp::ScissorPop => DrawOp::ScissorPop,
        DrawOp::Image { rect, src, fit } => DrawOp::Image {
            rect: WRect::new(rect.x + dx, rect.y + dy, rect.w, rect.h),
            src: src.clone(),
            fit: *fit,
        },
    }
}

/// D3 定案几何：锚 + placement + 面板尺寸 → 面板原点（视口溢出对向翻转 +
/// 边距钳制；Modal 视口居中；Edge* 贴边 sheet；Pointer = 最近右键点）。
/// 纯函数——单测钉死全 14 枚举。
fn popover_panel_origin(
    site: &PopoverAnchorSite,
    placement: &PopoverPlacement,
    panel: (f32, f32),
    viewport: (f32, f32),
    last_right_click: (f32, f32),
) -> (f32, f32) {
    let (vw, vh) = viewport;
    let (pw, ph) = panel;
    let clamp_x = |x: f32| x.clamp(POP_MARGIN, (vw - pw - POP_MARGIN).max(POP_MARGIN));
    let clamp_y = |y: f32| y.clamp(POP_MARGIN, (vh - ph - POP_MARGIN).max(POP_MARGIN));
    let r = match site {
        PopoverAnchorSite::Widget(r) => *r,
        PopoverAnchorSite::Point { x, y } => WRect::new(*x, *y, 0.0, 0.0),
    };
    // 纵向对向翻转（Bottom 族下溢 → 上翻 / Top 族上溢 → 下翻）。
    let v_flip = |below: f32, above: f32, want_below: bool| {
        if want_below && below + ph > vh - POP_MARGIN {
            above
        } else if !want_below && above < POP_MARGIN {
            below
        } else if want_below {
            below
        } else {
            above
        }
    };
    // 横向对向翻转（Left 左溢 → 右翻 / Right 右溢 → 左翻）。
    let h_flip = |left: f32, right: f32, want_left: bool| {
        if want_left && left < POP_MARGIN {
            right
        } else if !want_left && right + pw > vw - POP_MARGIN {
            left
        } else if want_left {
            left
        } else {
            right
        }
    };
    // Point 锚 = 面板原点直用（view.rs 文档口径："面板左上角对齐该点"，
    /// 无 GAP/高度推导）；Widget 锚 = 边缘 + GAP。
    let (below, above) = match site {
        PopoverAnchorSite::Point { y, .. } => (*y, *y - ph),
        PopoverAnchorSite::Widget(_) => (r.y + r.h + POP_GAP, r.y - POP_GAP - ph),
    };
    let left = r.x - POP_GAP - pw;
    let right = r.x + r.w + POP_GAP;
    let cx = r.x + r.w / 2.0 - pw / 2.0;
    let cy = r.y + r.h / 2.0 - ph / 2.0;
    let (px, py) = match placement {
        PopoverPlacement::Modal => ((vw - pw) / 2.0, (vh - ph) / 2.0),
        PopoverPlacement::EdgeLeft => (POP_MARGIN, (vh - ph) / 2.0),
        PopoverPlacement::EdgeRight => ((vw - pw - POP_MARGIN).max(POP_MARGIN), (vh - ph) / 2.0),
        PopoverPlacement::EdgeTop => ((vw - pw) / 2.0, POP_MARGIN),
        PopoverPlacement::EdgeBottom => ((vw - pw) / 2.0, (vh - ph - POP_MARGIN).max(POP_MARGIN)),
        PopoverPlacement::Pointer => (clamp_x(last_right_click.0), clamp_y(last_right_click.1)),
        // Point 锚恒 BottomStart 语义（view.rs 文档口径"placement 固定按
        // BottomStart 处理"）——方向性枚举对坐标锚全部退化原点对齐。
        _ if matches!(site, PopoverAnchorSite::Point { .. }) => (clamp_x(r.x), clamp_y(below)),
        PopoverPlacement::BottomStart => (r.x, v_flip(below, above, true)),
        PopoverPlacement::BottomEnd => (r.x + r.w - pw, v_flip(below, above, true)),
        PopoverPlacement::Bottom => (cx, v_flip(below, above, true)),
        PopoverPlacement::TopStart => (r.x, v_flip(below, above, false)),
        PopoverPlacement::TopEnd => (r.x + r.w - pw, v_flip(below, above, false)),
        PopoverPlacement::Top => (cx, v_flip(below, above, false)),
        PopoverPlacement::Left => (h_flip(left, right, true), cy),
        PopoverPlacement::Right => (h_flip(left, right, false), cy),
    };
    (clamp_x(px), clamp_y(py))
}

/// 点是否在矩形内（命中判定——既有 position() 谓词的命名提取）。
fn rect_contains(r: &WRect, x: f32, y: f32) -> bool {
    x >= r.x && x < r.x + r.w && y >= r.y && y < r.y + r.h
}

/// View 枚举 → DrawList 投影器（实现 [`FrameSource`]，作
/// `AppEndpoint` 的会话——[`super::client_runtime::ClientPump`] 泛型泵
/// 驱动，native queue 臂全链）。
pub struct RqProjector<C: Component> {
    component: C,
    /// 最近一帧的分型命中表（渲染时刷新；左键按型派发）。
    hits: Vec<HitEntry<C::Msg>>,
    /// 聚焦 input 槽位（T-01 D1-A：Input/Textarea 槽序身份，帧后重定位；
    /// 点击聚焦，无显式失焦——解释态同边界）。
    focused_input: Option<usize>,
    /// 聚焦框编辑 buffer（T-01 D2：聚焦期显示/编辑面——聚焦时自视图值
    /// 初始化，键入/退格就地编辑后经 INPUT_TEXT 代写回写组件）。
    input_buffer: String,
    /// IME preedit 暂存（PLAN-026 T-05 D2-A 定案：Commit 前组合串——
    /// 聚焦框渲染尾拼显示；Commit 并入 buffer / Cancelled 消解）。
    ime_preedit: Option<String>,
    /// 无聚焦时 IME 输入丢弃计数（I3 留痕观测面——测试/e2e 断言口）。
    ime_dropped: usize,
    /// 开态 select 槽位（T-01 D3：投影器侧开合状态；None = 全闭）。
    select_open: Option<usize>,
    /// 最近一帧的右键命中表（`on_right_click` 物化消息；渲染时刷新）。
    right_hits: Vec<(WRect, C::Msg)>,
    /// 最近指针位（PointerMoved/Pressed 跟踪——wire Scroll 无坐标，滚轮
    /// 路由定位消费；T-01 D5 执行期附注）。
    pointer: (f32, f32),
    /// 最近右键落点（PLAN-029 T-04：PopoverPlacement::Pointer 面板原点）。
    last_right_click: (f32, f32),
    /// 渲染期遭遇的未覆盖 kind（动态分支防线——显式留痕面，测试/e2e 断言口）。
    uncovered_seen: Vec<String>,
    /// PLAN-034 T-05（D4：canvas=位图快照过线）：canvas 臂产出的待上传
    /// 位图（render_frame 填，drain_bitmap_uploads 排水）。
    pending_bitmaps: Vec<super::endpoint::BitmapUpload>,
    /// canvas 槽位场景签名缓存（Debug 表示——同签名跳过重上传：pen 级
    /// 高频重排的带宽抑制）。
    canvas_scene_sig: Vec<String>,
    rev: u64,
    width: f32,
    height: f32,
    /// 周期拍相位基准（首拍对齐 interval，`iced::time::every` 同相）。
    tick_last: Option<Instant>,
    /// PLAN-678 T-02：根缺省居中通道开关（构造期读 `AUTO_NO_AUTOCENTER`
    /// ——`1` = 旁路；热路径零 env 读）。
    autocenter: bool,
}

impl<C: Component> RqProjector<C> {
    pub fn new(component: C, width: f32, height: f32) -> Self {
        let autocenter = std::env::var("AUTO_NO_AUTOCENTER").map(|v| v != "1").unwrap_or(true);
        Self {
            component,
            hits: Vec::new(),
            focused_input: None,
            input_buffer: String::new(),
            ime_preedit: None,
            ime_dropped: 0,
            select_open: None,
            right_hits: Vec::new(),
            pointer: (0.0, 0.0),
            last_right_click: (0.0, 0.0),
            uncovered_seen: Vec::new(),
            pending_bitmaps: Vec::new(),
            canvas_scene_sig: Vec::new(),
            rev: 1,
            width,
            height,
            tick_last: None,
            autocenter,
        }
    }

    /// 测试缝（PLAN-678 T-02）：关根缺省居中——模块外几何钉测试
    /// （client_runtime 管道环 / stage3 快照 migration 钉预居中坐标）
    /// 用；居中行为面由 native_projector::tests::autocenter_* 承载。
    #[cfg(test)]
    pub(crate) fn proj_autocenter_off(&mut self) {
        self.autocenter = false;
    }

    /// 启动覆盖门（AC-04）：当前视图 vs [`Coverage::native_queue_set`]——
    /// not-yet = `Err(缺项清单)`（调用方拒绝退出留痕）。Auto 档不调此门
    /// （待澄清③：native auto 缺省 independent，queue 覆盖爬坡前安全缺省）。
    pub fn ensure_covered(&self) -> Result<(), String> {
        let view = self.component.view();
        let scan = coverage::scan_native_view(&view);
        match coverage::judge(&scan, &Coverage::native_queue_set()) {
            Verdict::Covered => Ok(()),
            Verdict::NotCovered(missing) => Err(format!(
                "native queue 臂视图未覆盖（拒绝渲染，禁静默错绘）: {}",
                missing.join(", ")
            )),
        }
    }

    /// 渲染期遭遇的未覆盖 kind（动态分支防线——门后变化的显式观测面）。
    pub fn uncovered_seen(&self) -> &[String] {
        &self.uncovered_seen
    }

    pub fn revision(&self) -> u64 {
        self.rev
    }

    /// PLAN-030 T-03：组件访问口（壳装配——投影 lowering 写状态/命令
    /// 读走消费；AppProjector 同形）。
    pub fn component(&self) -> &C {
        &self.component
    }

    pub fn component_mut(&mut self) -> &mut C {
        &mut self.component
    }

    /// 外部状态变化登记（投影 apply 后调用——revision 前进 = 泵对账产帧）。
    /// PLAN-036 T-07（D5）：child 自主聚焦首输入槽——宿主 `__focus_input`
    /// 重试环（iced focus Task + 5 次重试）的 child 等价：快照事件
    ///（ApplyFilter）后由装配层调用；无输入命中 = false。
    pub fn focus_first_input(&mut self) -> bool {
        for e in &self.hits {
            if let HitEntry::Input { slot, .. } = e {
                self.focused_input = Some(*slot);
                return true;
            }
        }
        false
    }

    pub fn bump_revision(&mut self) {
        self.rev += 1;
    }

    /// 命中区矩形快照（e2e 点击注入消费——宿主 pointer 路由的等价载荷）。
    pub fn hit_rects(&self) -> Vec<crate::ui::desktop_protocol::message::WRect> {
        self.hits.iter().map(|h| *h.rect()).collect()
    }

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn height(&self) -> f32 {
        self.height
    }
}

/// PLAN-033 T-05（D4=A）：VM 组件的字符串命中区表（remote 宿主孪生
/// 消费——`AppProjector::hit_regions` 退役后的等价断言口）。分型提取：
/// Msg 命中 → 物化消息的 event_name（button:<name>）；Input 命中 →
/// on_change 消息的 event_name 经 `input_state_map` 反查绑定字段
///（input:<field>）。Toggle/Select 等 kind 在孪生面无消费者（原表
/// `filter(kind != 0)` 只留 button/input），不镜像。
impl RqProjector<crate::ui::dynamic::DynamicComponent> {
    pub fn hit_regions(&self) -> Vec<(WRect, String)> {
        use crate::ui::interpreter::DynamicMessage;
        self.hits
            .iter()
            .filter_map(|e| match e {
                HitEntry::Msg { rect, msg, .. } => match msg {
                    DynamicMessage::Typed { event_name, .. } => {
                        Some((*rect, format!("button:{event_name}")))
                    }
                    DynamicMessage::String(name) => Some((*rect, format!("button:{name}"))),
                },
                HitEntry::Input { rect, on_change: Some(msg), .. } => match msg {
                    DynamicMessage::Typed { event_name, .. } => {
                        let field = self
                            .component
                            .input_state_map()
                            .get(event_name)
                            .cloned()
                            .unwrap_or_else(|| event_name.clone());
                        Some((*rect, format!("input:{field}")))
                    }
                    _ => None,
                },
                _ => None,
            })
            .collect()
    }
}

impl<C: Component> FrameSource for RqProjector<C> {
    fn revision(&self) -> u64 {
        self.rev
    }

    /// PLAN-034 T-05（D3）：位图排水 = 组件面（合成生产者）+ canvas 臂
    /// 产出（场景快照——render_frame 填）。
    fn drain_bitmap_uploads(&mut self) -> Vec<super::endpoint::BitmapUpload> {
        let mut ups = self.component.drain_bitmap_uploads();
        ups.extend(self.pending_bitmaps.drain(..));
        ups
    }

    fn render_frame(&mut self) -> DrawList {
        let view = self.component.view();
        let mut ctx = NativeCtx {
            ops: Vec::new(),
            hits: Vec::new(),
            uncovered: Vec::new(),
            input_slots: 0,
            focused_input: self.focused_input,
            input_buffer: self.input_buffer.clone(),
            ime_preedit: self.ime_preedit.clone(),
            select_slots: 0,
            select_open: self.select_open,
            overlays: Vec::new(),
            popover_overlays: Vec::new(),
            viewport: (self.width, self.height),
            last_right_click: self.last_right_click,
            right_hits: Vec::new(),
            pending_bitmaps: Vec::new(),
            canvas_slots: 0,
            canvas_scene_sig: std::mem::take(&mut self.canvas_scene_sig),
        };
        let root_style = NodeStyle::default();
        let _ = layout_view_block(
            &mut ctx,
            std::slice::from_ref(&view),
            MARGIN,
            MARGIN,
            (self.width - MARGIN * 2.0).max(0.0),
            Dir::Vertical,
            &root_style,
        );
        // PLAN-678 T-02：根缺省居中（放置平移——布局期测量零改动）。delta
        // 按**主块 ops 实测包围盒**对中（非 Laid.size：items-center 等内
        // 部居中的视觉 bbox 窄于报告尺寸，实测 bbox 使已居中内容 delta=0，
        // 两通道零叠加冲突）；overlays/select 弹层此后追加但坐标锚定内容
        // 空间，随同 delta 平移。
        let avail_w = (self.width - MARGIN * 2.0).max(0.0);
        let avail_h = (self.height - MARGIN * 2.0).max(0.0);
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        for op in &ctx.ops {
            match op {
                DrawOp::Quad { rect, .. } | DrawOp::QuadR { rect, .. } | DrawOp::Scissor { rect } | DrawOp::Image { rect, .. } => {
                    min_x = min_x.min(rect.x);
                    min_y = min_y.min(rect.y);
                    max_x = max_x.max(rect.x + rect.w);
                    max_y = max_y.max(rect.y + rect.h);
                }
                DrawOp::Text { x, y, size, line_height, text, .. }
                | DrawOp::TextStyled { x, y, size, line_height, text, .. } => {
                    min_x = min_x.min(*x);
                    min_y = min_y.min(*y);
                    max_x = max_x.max(*x + measure_text(text, *size));
                    max_y = max_y.max(*y + *line_height);
                }
                DrawOp::ScissorPop => {}
            }
        }
        let (dx, dy) = if self.autocenter && !ctx.ops.is_empty() {
            // 根显式视口/填充类（h-screen/w-full 族）= 满幅意图声明——该轴
            // 豁免（与 iced 轨显式类零改动同律；min-h-* 同 min_height 在场
            // 豁免纵轴）。
            let root_node = node_style_of_view(&view);
            let fill_w =
                matches!(root_node.box_layout.width, Some(crate::ui::style::SizeValue::Screen | crate::ui::style::SizeValue::Full));
            let fill_h = matches!(
                root_node.box_layout.height,
                Some(crate::ui::style::SizeValue::Screen | crate::ui::style::SizeValue::Full)
            ) || root_node.box_layout.min_height.is_some();
            let bbox_w = (max_x - min_x).max(0.0);
            let bbox_h = (max_y - min_y).max(0.0);
            let dx = if fill_w {
                0.0
            } else {
                MARGIN + (avail_w - bbox_w).max(0.0) / 2.0 - min_x
            };
            let dy = if fill_h {
                0.0
            } else {
                MARGIN + (avail_h - bbox_h).max(0.0) / 2.0 - min_y
            };
            (dx, dy)
        } else {
            (0.0, 0.0)
        };
        let autocenter_pending = dx != 0.0 || dy != 0.0;
        // （平移执行在后——select/popover overlays 追加完成后、命中表
        // 收册前统一施行，弹层坐标同 delta。）
        // 开态 select 覆盖序（T-01 D3）：主块渲染后追加选项列 ops +
        // 命中项——DrawList paint order 天然置顶（无 overlay 协议语义）。
        let overlays = std::mem::take(&mut ctx.overlays);
        for ov in overlays {
            let size = 14.0;
            let line_h = size * LINE_H_FACTOR;
            let mut oy = ov.rect.y + ov.rect.h;
            for (i, opt) in ov.options.iter().enumerate() {
                let or = WRect::new(ov.rect.x, oy, ov.rect.w, INPUT_H);
                ctx.push_quad(or, if ov.selected_index == Some(i) { accent_fill() } else { input_bg() });
                ctx.ops.push(DrawOp::Text {
                    x: or.x + INPUT_PAD,
                    y: or.y + (INPUT_H - line_h) / 2.0,
                    size,
                    line_height: line_h,
                    color: text_fg(),
                    text: opt.clone(),
                });
                if let Some(cb) = &ov.on_select {
                    let msg = cb.call(i, opt);
                    ctx.hits.push(HitEntry::SelectOption { rect: or, msg });
                }
                oy += INPUT_H;
            }
        }
        // PLAN-029 T-04（D3）：popover 开态覆盖序——scrim（Modal）→ 面板
        // 底/边 → 面板子树 ops；命中登记序 = catcher 先、面板项后（rev 序
        // 面板项胜、主块被吞——select 互斥语义同款，开合零投影器状态）。
        let popovers = std::mem::take(&mut ctx.popover_overlays);
        for po in popovers {
            if po.modal {
                ctx.push_quad(WRect::new(0.0, 0.0, self.width, self.height), POP_SCRIM);
            }
            ctx.push_quad(po.rect, pop_bg());
            ctx.push_border(po.rect, pop_border());
            ctx.ops.extend(po.ops);
            ctx.hits.push(HitEntry::PopoverDismiss {
                rect: WRect::new(0.0, 0.0, self.width, self.height),
                on_dismiss: po.on_dismiss,
            });
            ctx.hits.extend(po.hits);
        }
        // 聚焦重定位（T-01 D1-A）：槽位越界 = 视图结构变化 → 失焦 +
        // buffer 清空（不猜测对位——槽序身份在结构变化下不可靠，v1 边界）。
        if let Some(slot) = self.focused_input {
            if slot >= ctx.input_slots {
                self.focused_input = None;
                self.input_buffer.clear();
                self.ime_preedit = None;
            }
        }
        // select 开合重定位（同槽序纪律——D3）。
        if let Some(slot) = self.select_open {
            if slot >= ctx.select_slots {
                self.select_open = None;
            }
        }
        // PLAN-678 T-02：根平移执行（dx/dy 自主块 bbox，上文算得；overlays
        // 弹层已在此前追加完毕，同 delta 随行）。视口锚定 rect（全窗
        // scrim / PopoverDismiss catcher）不平移——平移会撕开覆盖。
        if autocenter_pending {
            let shift = |r: &mut WRect| {
                if r.w >= self.width && r.h >= self.height {
                    return;
                }
                r.x += dx;
                r.y += dy;
            };
            for op in &mut ctx.ops {
                match op {
                    DrawOp::Quad { rect, .. } | DrawOp::QuadR { rect, .. } | DrawOp::Scissor { rect } | DrawOp::Image { rect, .. } => {
                        shift(rect);
                    }
                    DrawOp::Text { x, y, .. } | DrawOp::TextStyled { x, y, .. } => {
                        *x += dx;
                        *y += dy;
                    }
                    DrawOp::ScissorPop => {}
                }
            }
            for hit in &mut ctx.hits {
                shift(hit.rect_mut());
            }
            for (r, _) in &mut ctx.right_hits {
                shift(r);
            }
        }
        self.hits = ctx.hits;
        self.right_hits = ctx.right_hits;
        self.uncovered_seen = ctx.uncovered;
        // PLAN-034：canvas 位图并入待传（extend——两渲染间隔水时双版本
        // 都过线，宿主后到者胜；同签名场景已被臂内去重）。
        self.pending_bitmaps.extend(ctx.pending_bitmaps);
        self.canvas_scene_sig = ctx.canvas_scene_sig;
        DrawList { clear: Some(bg()), ops: ctx.ops }
    }

    fn on_input(&mut self, input: &InputMsg) {
        match input {
            // 最近指针位跟踪（滚轮定位消费——wire Scroll 无坐标，T-01 D5
            // 执行期附注）。
            InputMsg::PointerMoved { x, y, .. } => self.pointer = (*x, *y),
            InputMsg::PointerPressed { x, y, button: MouseButton::Left, .. } => {
                self.pointer = (*x, *y);
                self.pointer_down_left(*x, *y);
            }
            // 右键命中派发（`on_right_click` 物化消息——倒序置顶优先）。
            InputMsg::PointerPressed { x, y, button: MouseButton::Right, .. } => {
                self.pointer = (*x, *y);
                // PLAN-029 T-04：Pointer placement 面板原点跟踪。
                self.last_right_click = (*x, *y);
                let hit = self
                    .right_hits
                    .iter()
                    .rev()
                    .find(|(r, _)| rect_contains(r, *x, *y))
                    .cloned();
                if let Some((_, msg)) = hit {
                    self.component.on(msg);
                    self.rev += 1;
                }
            }
            InputMsg::CharTyped { ch, .. } => self.char_typed(*ch),
            // PLAN-674 T-01：聚焦编辑器的非打印键（Enter/箭标/Home/End/
            // PageUp·Down/Delete/Tab/Backspace）→ core handle_input 键面
            //（修饰位随行；Esc 仍走下方 select/popover 关闭臂——编辑器
            // Esc 语义 not-yet 随注）。
            InputMsg::KeyPressed { key, modifiers, .. }
                if self.focused_editor_key().is_some() && editor_key_of(*key).is_some() =>
            {
                let sk = self.focused_editor_key().expect("guard 已验");
                let input = ce::EditorInput::KeyPressed {
                    key: editor_key_of(*key).expect("guard 已验"),
                    text: None,
                    modifiers: super::editor_frame::wire_mods(*modifiers),
                };
                self.feed_editor_input(&sk, input);
            }
            // IME 闭环（PLAN-026 T-05，D2 定案）：Commit = 聚焦 buffer
            // 追加 → INPUT_TEXT 代写 → on_change 派发；Cancelled =
            // preedit 消解；Preedit = 暂存（渲染尾拼）。消费先例 =
            // editor_frame.rs:195-199（wire v1.0 在册变体）。PLAN-674
            // T-01：聚焦编辑器优先走 core IME 面（preedit 经
            // EditorDrawList 自渲，不经 input 尾拼道）。
            InputMsg::ImeCommit { text, .. } => self.ime_commit(text),
            InputMsg::ImeCancelled { .. } => {
                if let Some(sk) = self.focused_editor_key() {
                    self.feed_editor_input(&sk, ce::EditorInput::ImeClosed);
                }
                self.ime_preedit = None;
            }
            InputMsg::ImePreedit { text, .. } => {
                if let Some(sk) = self.focused_editor_key() {
                    self.feed_editor_input(&sk, ce::EditorInput::ImePreedit(text.clone()));
                } else if self.focused_input.is_some() {
                    self.ime_preedit = Some(text.clone());
                } else {
                    self.ime_dropped += 1;
                }
            }
            InputMsg::KeyPressed { key, .. } if *key == 8 => self.backspace(),
            // Esc（VK_ESCAPE = 27）关开态 select（T-01 D3）+ preedit
            // 消解（ImeCancelled 同义宿主路径）。
            InputMsg::KeyPressed { key, .. } if *key == 27 => {
                if self.select_open.take().is_some() {
                    self.rev += 1;
                } else if let Some(msg) = self.hits.iter().rev().find_map(|e| match e {
                    HitEntry::PopoverDismiss { on_dismiss: Some(m), .. } => Some(m.clone()),
                    _ => None,
                }) {
                    // PLAN-029 T-04：Esc 关开态 popover（最顶者——rev 序
                    // 末位 = 最后登记的开态面板）。
                    self.component.on(msg);
                    self.rev += 1;
                }
                self.ime_preedit = None;
            }
            // 滚轮派发（T-01 D5）：指针位包含 → 内层胜（倒序；嵌套
            // scrollable 外层先登记）→ 唯一 Scrollable 兜底（wire Scroll
            // 无坐标）；offset' = clamp(快照+delta) 后组装 ScrollMetrics
            // （镜像 iced absolute_offset = 滚动后偏移语义）。
            InputMsg::Scroll { dx, dy, .. } => self.wheel(*dx, *dy),
            _ => {}
        }
    }

    fn on_control(&mut self, control: &ControlMsg) {
        match control {
            ControlMsg::Resize { width, height, .. } => {
                self.width = *width;
                self.height = *height;
            }
            // PLAN-033 T-04：L3 v2a 快照注入（AppProjector 退役语义平移）
            // ——经 Component::apply_state_snapshot（VM 组件实现；a2r
            // typed 缺省 false 维持 not-yet 留痕）；revision 续接快照值。
            ControlMsg::StateSnapshot { payload, .. } => {
                if let Ok((rev, fields)) =
                    crate::ui::desktop_protocol::client_runtime::decode_state_snapshot(payload)
                {
                    if self.component.apply_state_snapshot(rev, &fields) {
                        self.rev = rev;
                    }
                }
            }
            _ => {}
        }
    }

    /// PLAN-033 T-03③（D3）：`__desktop_cmd` 读走转发（经
    /// [`Component::drain_desktop_commands`]——VM 组件 c4 语义实现，a2r
    /// 缺省空）。泵在读走点消费后上行 `ControlMsg::DesktopBus`。
    fn drain_desktop_commands(&mut self) -> Vec<String> {
        self.component.drain_desktop_commands()
    }

    /// 周期拍（`FrameSource::poll_tick`）：
    /// - PLAN-033 T-03②（D2=B）：组件侧多 timer 泵优先——VM 轨
    ///   （`DynamicComponent`）的 timesources 逐条到期派发；返回 true =
    ///   revision 前进（泵侧对账产帧）。a2r 编译组件缺省 false 零开销。
    /// - 既有单通道 tick：interval 到期 → `component.on(tick_msg)` +
    ///   revision 前进。
    fn poll_tick(&mut self) {
        if self.component.fire_due_timers() {
            self.rev += 1;
        }
        let Some(interval) = self.component.tick_interval_ms() else {
            return;
        };
        let now = Instant::now();
        match self.tick_last {
            None => self.tick_last = Some(now),
            Some(last) => {
                if now.duration_since(last)
                    >= std::time::Duration::from_millis(u64::from(interval))
                {
                    self.tick_last = Some(now);
                    if let Some(msg) = self.component.tick_msg() {
                        self.component.on(msg);
                        self.rev += 1;
                    }
                }
            }
        }
    }
}

impl<C: Component> RqProjector<C> {
    /// 左键按下：分型派发（倒序 = 绘制序置顶优先）。
    fn pointer_down_left(&mut self, x: f32, y: f32) {
        // select 开态命中互斥（T-01 D3）：仅选项项 + 关闭区——命中选项
        // → 物化消息派发；未命中（外点）→ 仅关闭（吞掉不下穿主块）。
        if self.select_open.is_some() {
            let hit = self.hits.iter().rev().find_map(|e| match e {
                HitEntry::SelectOption { rect, msg } if rect_contains(rect, x, y) => Some(msg.clone()),
                _ => None,
            });
            self.select_open = None;
            self.rev += 1; // 关闭也是状态变化（帧回闭态）
            if let Some(msg) = hit {
                self.component.on(msg);
            }
            return;
        }
        let hit = self
            .hits
            .iter()
            .rev()
            .find(|e| rect_contains(e.rect(), x, y))
            .cloned();
        match hit {
            Some(HitEntry::Msg { msg, .. }) => {
                self.component.on(msg);
                self.rev += 1;
            }
            // 聚焦（T-01 D1/D2）：记槽位 + buffer 自视图值初始化。聚焦
            // 改变帧面（焦点描边）→ rev 前进（解释态不推版——native 帧
            // 面全由 rev 驱动，差异随注）。
            // PLAN-674 T-01：编辑器槽位（editor = 存储键）——旧编辑器
            // FocusLost → FocusGained + 光标定位（局部坐标 = 命中点 -
            // rect 原点；core handle_mouse_press 引擎态光标/选区）。
            Some(HitEntry::Input { rect, value, slot, editor, .. }) => {
                if let Some(sk) = editor {
                    let prev = self.focused_editor_key();
                    if prev.as_deref() != Some(sk.as_str()) {
                        if let Some(old) = prev {
                            self.feed_editor_input(&old, ce::EditorInput::FocusLost);
                        }
                        self.feed_editor_input(sk.as_str(), ce::EditorInput::FocusGained);
                    }
                    let (lx, ly) = (x - rect.x, y - rect.y);
                    self.feed_editor_input(
                        sk.as_str(),
                        ce::EditorInput::MousePressed { button: ce::EditorButton::Left, x: lx, y: ly },
                    );
                }
                self.focused_input = Some(slot);
                self.input_buffer = value;
                self.ime_preedit = None;
                self.rev += 1;
            }
            // 轨道点击 → f32 = min + clamp((x-x0)/w)×range（step 取整）→
            // 回调物化派发（零 thread-local——载荷自足；登记时已滤 None）。
            Some(HitEntry::Slider { rect, min, max, step, on_change }) => {
                let t = if rect.w > 0.0 {
                    ((x - rect.x) / rect.w).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                let raw = min + t * (max - min);
                let v = match step {
                    Some(st) if st > 0.0 => min + ((raw - min) / st).round() * st,
                    _ => raw,
                };
                if let Some(handler) = on_change {
                    self.component.on(handler.call(v.clamp(min, max)));
                }
                self.rev += 1;
            }
            // select 闭态盒点击 → 开（无消息——投影器侧状态，D3）。
            Some(HitEntry::SelectBox { slot, .. }) => {
                self.select_open = Some(slot);
                self.rev += 1;
            }
            // 开态选项项只在互斥臂可达（闭态派发不落此处）；滚轮只在
            // wheel 臂可达（左键不派发）。
            Some(HitEntry::SelectOption { .. }) | Some(HitEntry::Scroll { .. }) => {}
            // PLAN-029 T-04（D3）：popover 开态 catcher 命中 = 外点/scrim
            // → on_dismiss 派发 + 关闭（吞——不落穿主块，select 互斥同
            // 语义；面板项在 catcher 之前登记 = rev 序先胜不落此处）。
            Some(HitEntry::PopoverDismiss { on_dismiss, .. }) => {
                if let Some(msg) = on_dismiss {
                    self.component.on(msg);
                }
                self.rev += 1;
            }
            // PLAN-032 T-03（D2）：tabs 托盘项 → index 物化派发（回调内
            // 包装 value 串载荷——受控切换由 app 状态经 view() 重入驱动）。
            Some(HitEntry::TabSelect { index, on_select, .. }) => {
                self.component.on(on_select.call(index));
                self.rev += 1;
            }
            None => {}
        }
    }

    /// 聚焦槽位的 on_change 物化消息（无槽位/无 handler → None）。
    fn focused_on_change(&self) -> Option<C::Msg> {
        let slot = self.focused_input?;
        self.hits.iter().find_map(|e| match e {
            HitEntry::Input { on_change, slot: s, .. } if *s == slot => on_change.clone(),
            _ => None,
        })
    }

    /// PLAN-674 T-01：聚焦槽位的编辑器存储键（平面 input/textarea =
    /// None）。命中表为最近一帧快照——编辑器随视图消失后此处返回 None，
    /// 键入自然 no-op（槽位失配）。
    fn focused_editor_key(&self) -> Option<String> {
        let slot = self.focused_input?;
        self.hits.iter().find_map(|e| match e {
            HitEntry::Input { editor: Some(sk), slot: s, .. } if *s == slot => Some(sk.clone()),
            _ => None,
        })
    }

    /// PLAN-674 T-01：编辑器 core 输入馈送——`handle_input` 全键面
    /// （光标/选区/undo/IME 引擎态在 core，NullClipboard = 剪贴板族
    /// not-yet 随注）→ text_changed 时 INPUT_TEXT 全文代写 +
    /// on_change 零参派发。INPUT_TEXT 即 inproc 侧
    /// `IcedMessage.input_value: Some(text)` 的 RQ 通道等价（泛型
    /// Component::on 无参数面——解释态 on() 单参注入臂与注册表读面
    /// `code_editor_text(key)`（同进程）皆可消费）；cursor/redraw 变化
    /// 亦推 rev（当前行高亮/caret 帧面）。
    fn feed_editor_input(&mut self, sk: &str, input: ce::EditorInput) {
        let out = ce::code_editor_with(sk, |core| {
            ce::with_font_system(|fs| core.handle_input(fs, input, &mut ce::NullClipboard))
        });
        let Some(out) = out else { return };
        if out.text_changed {
            let text = ce::code_editor_text(sk).unwrap_or_default();
            if let Some(msg) = self.focused_on_change() {
                crate::ui::iced::store_input_text(&text);
                self.component.on(msg);
            }
        }
        if out.text_changed || out.cursor_changed || out.request_redraw {
            self.rev += 1;
        }
    }

    /// 键入回写（T-01 D2 定案 A）：编辑 buffer → INPUT_TEXT thread-local
    /// 代写（与 a2r 生成 on() 的 `last_input_text()` 读面同线程接驳——
    /// ClientPump 单线程泵）→ on_change 派发 → rev 前进。
    fn dispatch_input_edit(&mut self, msg: C::Msg) {
        crate::ui::iced::store_input_text(&self.input_buffer);
        self.component.on(msg);
        self.rev += 1;
    }

    /// CharTyped：聚焦框 buffer 追加 → 回写（控制字符不过——Enter/
    /// on_submit not-yet，T-01 D2 随注）。PLAN-674 T-01：聚焦编辑器
    /// 优先走 core 键入（光标处插入——Engine 层完整编辑语义）。
    fn char_typed(&mut self, ch: char) {
        if ch.is_control() {
            return;
        }
        if let Some(sk) = self.focused_editor_key() {
            self.feed_editor_input(
                &sk,
                ce::EditorInput::KeyPressed {
                    key: ce::EditorKey::Char(ch),
                    text: Some(ch.to_string()),
                    modifiers: ce::EditorModifiers::none(),
                },
            );
            return;
        }
        if self.focused_input.is_none() {
            return;
        }
        let Some(msg) = self.focused_on_change() else { return };
        self.input_buffer.push(ch);
        self.dispatch_input_edit(msg);
    }

    /// VK_BACK：聚焦框 buffer 回退一格 → 同通道回写（退格到空仍派发——
    /// 清空语义合法；解释态同口径 client_runtime.rs:450-457）。
    fn backspace(&mut self) {
        if self.focused_input.is_none() {
            return;
        }
        let Some(msg) = self.focused_on_change() else { return };
        self.input_buffer.pop();
        self.dispatch_input_edit(msg);
    }

    /// ImeCommit（PLAN-026 T-05）：组合串并入聚焦 buffer → 同 CharTyped
    /// 通道回写（INPUT_TEXT 代写 + on_change 派发 + rev 前进）。无聚焦 /
    /// 无 handler = 丢弃 + ime_dropped 留痕（I3）。preedit 暂存随并入
    /// 消解（组合终态 = Commit）。PLAN-674 T-01：聚焦编辑器走 core
    /// ImeCommit（insert_string + delta 同流——人/agent 写无差别）。
    fn ime_commit(&mut self, text: &str) {
        if let Some(sk) = self.focused_editor_key() {
            self.feed_editor_input(&sk, ce::EditorInput::ImeCommit(text.to_string()));
            return;
        }
        if self.focused_input.is_none() {
            self.ime_dropped += 1;
            return;
        }
        let Some(msg) = self.focused_on_change() else {
            self.ime_dropped += 1;
            return;
        };
        self.ime_preedit = None;
        self.input_buffer.push_str(text);
        self.dispatch_input_edit(msg);
    }

    /// 无聚焦 IME 丢弃计数（观测面——与 uncovered_seen 同级的显式留痕）。
    pub fn ime_dropped(&self) -> usize {
        self.ime_dropped
    }

    /// 滚轮路由（T-01 D5）：内层命中胜 → 唯一 Scrollable 兜底；不命中
    /// 且非唯一 = 静默不路由（多 Scrollable 且指针缺席 → 目标歧义，I3）。
    /// PLAN-674 T-01：指针位编辑器优先——core 内部滚动（cosmic-text
    /// buffer scroll + 归一钳制，render 视口虚拟化消费；无需聚焦——
    /// 悬停滚动即编辑器惯例），无编辑器命中走既有 Scrollable 路由。
    fn wheel(&mut self, dx: f32, dy: f32) {
        let editor_hit = self.hits.iter().rev().find_map(|e| match e {
            HitEntry::Input { rect, editor: Some(sk), .. }
                if rect_contains(rect, self.pointer.0, self.pointer.1) =>
            {
                Some(sk.clone())
            }
            _ => None,
        });
        if let Some(sk) = editor_hit {
            self.feed_editor_input(&sk, ce::EditorInput::WheelScrolled { dx, dy, shift: false });
            return;
        }
        let entry = {
            let containing = self.hits.iter().enumerate().rev().find_map(|(i, e)| match e {
                HitEntry::Scroll { .. } if rect_contains(e.rect(), self.pointer.0, self.pointer.1) => Some(i),
                _ => None,
            });
            let idx = containing.or_else(|| {
                let scrolls: Vec<usize> = self
                    .hits
                    .iter()
                    .enumerate()
                    .filter_map(|(i, e)| if matches!(e, HitEntry::Scroll { .. }) { Some(i) } else { None })
                    .collect();
                (scrolls.len() == 1).then_some(scrolls[0])
            });
            idx.and_then(|i| self.hits.get(i).cloned())
        };
        let Some(HitEntry::Scroll { offset, viewport, content, callback, .. }) = entry else {
            return;
        };
        let max_y = (content.1 - viewport.1).max(0.0);
        let max_x = (content.0 - viewport.0).max(0.0);
        let msg = callback.call(ScrollMetrics {
            offset_x: (offset.0 + dx).clamp(0.0, max_x),
            offset_y: (offset.1 + dy).clamp(0.0, max_y),
            viewport_w: viewport.0,
            viewport_h: viewport.1,
            content_w: content.0,
            content_h: content.1,
        });
        self.component.on(msg);
        self.rev += 1;
    }
}

// ---------------------------------------------------------------------------
// 块流布局 walker（镜像 client_runtime::layout_block 语义，节点面换 View）
// ---------------------------------------------------------------------------

/// 子级堆叠方向（client_runtime::Dir 同型）。
#[derive(Clone, Copy, PartialEq)]
enum Dir {
    Vertical,
    Horizontal,
}

/// 布局产物：一个节点子树投影后的外框尺寸（client_runtime::LaidBlock 同型）。
struct Laid {
    size: (f32, f32),
}

/// 投影上下文（一次 render_frame 的累积状态；无组件借用——View 全物化，
/// 状态读在 view() 构建期已完成）。
struct NativeCtx<M: Clone + std::fmt::Debug> {
    ops: Vec<DrawOp>,
    hits: Vec<HitEntry<M>>,
    uncovered: Vec<String>,
    /// Input/Textarea 槽位计数（登记序 = 确定性树序——D1-A 聚焦身份）。
    input_slots: usize,
    /// 聚焦槽位（projector 状态的布局期只读快照——臂内判焦点描边）。
    focused_input: Option<usize>,
    /// 聚焦框编辑 buffer 快照（臂内显示消费——聚焦框显示 buffer 而非
    /// 视图值，D2）。
    input_buffer: String,
    /// IME preedit 暂存快照（PLAN-026 T-05 D2-A：聚焦框渲染尾拼消费）。
    ime_preedit: Option<String>,
    /// select 槽位计数（开合身份，D3）。
    select_slots: usize,
    /// 开态 select 槽位快照（臂内判开态渲染 + 命中互斥登记）。
    select_open: Option<usize>,
    /// 开态 select 覆盖序记录（render_frame 主块后统一追加）。
    overlays: Vec<SelectOverlay<M>>,
    /// PLAN-029 T-04：开态 popover 覆盖序记录（select 同序追加）。
    popover_overlays: Vec<PopoverOverlay<M>>,
    /// 视口尺寸快照（popover 面板几何推导）。
    viewport: (f32, f32),
    /// 最近右键落点快照（Pointer placement 原点）。
    last_right_click: (f32, f32),
    /// 右键命中表（Button/Row/Column/Container `on_right_click`）。
    right_hits: Vec<(WRect, M)>,
    /// PLAN-034 T-05：canvas 臂待上传位图（render 结束并入 projector）。
    pending_bitmaps: Vec<super::endpoint::BitmapUpload>,
    /// canvas 槽位计数（帧内确定性树序——位图 id 身份）。
    canvas_slots: usize,
    /// canvas 槽位场景签名（render 起点自 projector 摘取，帧后归还）。
    canvas_scene_sig: Vec<String>,
}

impl<M: Clone + std::fmt::Debug> NativeCtx<M> {
    fn push_quad(&mut self, rect: WRect, color: Rgba8) {
        self.ops.push(DrawOp::Quad { rect, color });
    }

    /// PLAN-679 Phase 2：带圆角发射——radius Some 且 >0.5 → QuadR
    /// （rounded-full 哨兵 9999 在此解析 min(w,h)/2），否则普通 Quad。
    fn push_quad_rounded(&mut self, rect: WRect, color: Rgba8, radius: Option<f32>) {
        match radius {
            Some(r) if r > 0.5 => {
                let r = r.min(rect.w.min(rect.h) / 2.0);
                self.ops.push(DrawOp::QuadR { rect, color, radius: r });
            }
            _ => self.push_quad(rect, color),
        }
    }

    /// PLAN-679 Phase 2：线性渐变 N 条带近似（wire 无渐变 op；24 条带
    /// 在 480 宽下每条 20px——观感连续度对 004 头带足够，真渐变 op
    /// 另立 wire 提案）。
    fn push_gradient(&mut self, rect: WRect, vertical: bool, from: Rgba8, to: Rgba8) {
        const STEPS: usize = 24;
        let n = STEPS as f32;
        for i in 0..STEPS {
            let t = i as f32 / n;
            let frac = 1.0 / n;
            let mid = t + frac / 2.0;
            let c = Rgba8::new(
                (from.r as f32 + (to.r as f32 - from.r as f32) * mid) as u8,
                (from.g as f32 + (to.g as f32 - from.g as f32) * mid) as u8,
                (from.b as f32 + (to.b as f32 - from.b as f32) * mid) as u8,
                255,
            );
            let sub = if vertical {
                WRect::new(rect.x, rect.y + rect.h * t, rect.w, rect.h * frac + 0.5)
            } else {
                WRect::new(rect.x + rect.w * t, rect.y, rect.w * frac + 0.5, rect.h)
            };
            self.push_quad(sub, c);
        }
    }

    /// 1px 边框（四边细条——client_runtime::push_border 同型）。
    fn push_border(&mut self, rect: WRect, color: Rgba8) {
        let t = 1.0;
        self.push_quad(WRect::new(rect.x, rect.y, rect.w, t), color);
        self.push_quad(WRect::new(rect.x, rect.y + rect.h - t, rect.w, t), color);
        self.push_quad(WRect::new(rect.x, rect.y, t, rect.h), color);
        self.push_quad(WRect::new(rect.x + rect.w - t, rect.y, t, rect.h), color);
    }
}

/// 走线期临时 ctx（子树 @0,0 渲染后平移——Popover 臂同款快照字段透传，
/// 槽位计数接续）。
fn detached_ctx<M: Clone + std::fmt::Debug>(ctx: &NativeCtx<M>) -> NativeCtx<M> {
    NativeCtx {
        ops: Vec::new(),
        hits: Vec::new(),
        uncovered: Vec::new(),
        pending_bitmaps: Vec::new(),
        canvas_slots: ctx.canvas_slots,
        canvas_scene_sig: Vec::new(),
        input_slots: ctx.input_slots,
        focused_input: ctx.focused_input,
        input_buffer: ctx.input_buffer.clone(),
        ime_preedit: ctx.ime_preedit.clone(),
        select_slots: ctx.select_slots,
        select_open: ctx.select_open,
        overlays: Vec::new(),
        popover_overlays: Vec::new(),
        viewport: ctx.viewport,
        last_right_click: ctx.last_right_click,
        right_hits: Vec::new(),
    }
}

/// PLAN-032 T-05（D1 分层）：absolute 延迟放置——脱离流子级在父块尺寸
/// 已知后按 offsets 锚定父内容盒（left/top 优先——CSS 过约束 left 胜；
/// right/bottom 按父盒尺寸反算；无 offset 取流起点近似）@0,0 走线渲染
/// → shift 平移，ops/hits 追加主序之后（覆盖序置顶——Popover 平移臂
/// 先例 :1370-1383）。同层按 z 稳定排序（文档序同 z 保持）；in-flow z
/// 与完整栈序 out of scope（I3 随注）。父盒尺寸 = 块流内容尺寸
///（absolute 子级不贡献父高——CSS 同语义）。
fn place_absolute_children<M: Clone + std::fmt::Debug>(
    ctx: &mut NativeCtx<M>,
    children: &[&View<M>],
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) {
    let mut ordered: Vec<&View<M>> = children.to_vec();
    ordered.sort_by_key(|v| node_style_of_view(v).z_index.unwrap_or(0));
    for view in ordered {
        let st = node_style_of_view(view);
        let mut tmp = detached_ctx(ctx);
        let laid = layout_view_node(&mut tmp, view, 0.0, 0.0, w);
        let ax = match (st.offset_left, st.offset_right) {
            (Some(l), _) => x + l,
            (None, Some(r)) => x + w - laid.size.0 - r,
            (None, None) => x,
        };
        let ay = match (st.offset_top, st.offset_bottom) {
            (Some(t), _) => y + t,
            (None, Some(b)) => y + h - laid.size.1 - b,
            (None, None) => y,
        };
        ctx.ops.extend(tmp.ops.iter().map(|op| shift_draw_op(op, ax, ay)));
        ctx.hits.extend(tmp.hits.into_iter().map(|hh| hh.shifted(ax, ay)));
        ctx.right_hits.extend(tmp.right_hits.into_iter().map(|(mut r, m)| {
            r.x += ax;
            r.y += ay;
            (r, m)
        }));
        ctx.uncovered.extend(tmp.uncovered);
        ctx.input_slots = tmp.input_slots;
        ctx.select_slots = tmp.select_slots;
        ctx.popover_overlays.extend(tmp.popover_overlays);
        // PLAN-034：popover 面板内 canvas——槽位接续 + 位图并回（tmp 的
        // 签名缓存独立 → 面板内容每帧重上传，正确性优先的取舍）。
        ctx.canvas_slots = tmp.canvas_slots;
        ctx.pending_bitmaps.extend(tmp.pending_bitmaps);
    }
}

/// PLAN-032 T-04（D4）：Grid walker 单源（View::Grid 变体臂与样式版
/// grid 分岔共用——026 T-04 两遍网格原实现提取；等宽列 row-major 行序、
/// 行高 = 行内最大、bg 两遍法置子级之下，语义零变化）。
fn layout_grid_cells<M: Clone + std::fmt::Debug>(
    ctx: &mut NativeCtx<M>,
    cells: &[View<M>],
    cols: usize,
    gap: f32,
    style: &NodeStyle,
    x: f32,
    y: f32,
    avail_w: f32,
) -> Laid {
    let cols = cols.max(1);
    let pad = (style.pad_left(), style.pad_top(), style.pad_right(), style.pad_bottom());
    let inner_w = (avail_w - pad.0 - pad.2).max(0.0);
    let cell_w = if cells.is_empty() {
        0.0
    } else {
        ((inner_w - gap * (cols.saturating_sub(1)) as f32) / cols as f32).max(0.0)
    };
    let place = |ctx: &mut NativeCtx<M>| -> (f32, f32) {
        let mut row_y = 0.0f32;
        for (ri, row) in cells.chunks(cols).enumerate() {
            if ri > 0 {
                row_y += gap;
            }
            let mut row_h = 0.0f32;
            for (ci, cell) in row.iter().enumerate() {
                let cell_x = ci as f32 * (cell_w + gap);
                let laid = layout_view_node(ctx, cell, x + pad.0 + cell_x, y + pad.1 + row_y, cell_w);
                row_h = row_h.max(laid.size.1);
            }
            row_y += row_h;
        }
        (cell_w * cols as f32 + gap * (cols.saturating_sub(1)) as f32, row_y)
    };
    let ops_mark = ctx.ops.len();
    let hits_mark = ctx.hits.len();
    let (content_w, content_h) = place(ctx);
    let outer_w = match style.fixed_w() {
        Some(fw) => fw,
        None => content_w + pad.0 + pad.2,
    };
    let outer_h = match style.fixed_h() {
        Some(fh) => fh,
        None => content_h + pad.1 + pad.3,
    };
    if style.bg.is_some() {
        ctx.ops.truncate(ops_mark);
        ctx.hits.truncate(hits_mark);
        ctx.push_quad(WRect::new(x, y, outer_w, outer_h), style.bg.unwrap());
        place(ctx);
    }
    if let Some(border) = style.border {
        ctx.push_border(WRect::new(x, y, outer_w, outer_h), border);
    }
    Laid { size: (outer_w, outer_h) }
}

/// 块流布局：把一列视图排进 `(x, y, w)` 内容盒，返回内容尺寸。
/// 语义镜像 `client_runtime::layout_block`（纵向依序下排 / row 横排 /
/// `center_children` 主轴居中两遍法）——子级已物化，无 flatten/条件求值。
fn layout_view_block<M: Clone + std::fmt::Debug>(
    ctx: &mut NativeCtx<M>,
    views: &[View<M>],
    x: f32,
    y: f32,
    w: f32,
    dir: Dir,
    parent: &NodeStyle,
) -> Laid {
    let gap = parent.gap();
    // PLAN-032 T-04（D4）：样式版 grid 分岔单一 choke——容器/堆叠族
    //（group/container/scrollable 等全路径经此）style 带 Grid/GridCols/
    // GridRows 类时整体改走网格布局（复用 Grid walker，024 真源 =
    // `col (style: "grid grid-cols-2 gap-2")`）。
    if parent.grid_cols.is_some() || parent.grid_rows.is_some() {
        let cols = parent
            .grid_cols
            .map(|c| c.max(1))
            .unwrap_or_else(|| {
                parent
                    .grid_rows
                    .map(|r| (views.len() + r - 1) / r.max(1))
                    .expect("choke 条件保证 grid_cols/grid_rows 至少其一")
            });
        return layout_grid_cells(ctx, views, cols, gap, parent, x, y, w);
    }
    let ops_mark = ctx.ops.len();
    let hits_mark = ctx.hits.len();
    // PLAN-032 T-05（D1）：absolute 子级收集延后（不入流不占位——尺寸
    // 已知后在块返回前锚定放置）。
    let mut absolute_children: Vec<&View<M>> = Vec::new();
    let mut cursor = 0.0f32;
    let mut cross_max = 0.0f32;
    let mut first = true;
    // 交叉轴居中（列臂 items-center）首轮自然宽录制——仅 parent 声明
    // 居中时收集（热路径零余账；Horizontal 主轴臂不需要逐子宽）。
    let mut child_widths: Vec<f32> = Vec::new();
    // P679-D1②：Horizontal 行（natural_w, flex 因子）逐可见子录制——
    // flex 份额分配的消费源。
    let mut h_meta: Vec<(f32, f32)> = Vec::new();
    for view in views {
        // PLAN-032 T-02（D3）：hidden 子级整段跳过——不占主轴 cursor 也
        // 不参与 gap 序（CSS display:none：兄弟间只留一个 gap，非每
        // 缺席位一个）。节点级 choke 保留兜底（单子引用/网格 cell 等
        // 直调 layout_view_node 的路径）。T-05：absolute 同跳过并收集
        ///（延迟放置见块尾）。
        let style = node_style_of_view(view);
        if style.hidden {
            continue;
        }
        if style.absolute {
            absolute_children.push(view);
            continue;
        }
        if !first {
            cursor += gap;
        }
        first = false;
        let my = style.margin_y();
        match dir {
            Dir::Vertical => {
                let inner_w = style.fixed_w().unwrap_or(w);
                let child_x = if style.center_children {
                    x + (w - inner_w).max(0.0) / 2.0
                } else {
                    x
                };
                let laid = layout_view_node(ctx, view, child_x, y + cursor + my, inner_w);
                if parent.center_children {
                    child_widths.push(laid.size.0);
                }
                cursor += my + laid.size.1 + style.box_layout.margin_bottom.unwrap_or(0.0);
                cross_max = cross_max.max(laid.size.0);
            }
            Dir::Horizontal => {
                let laid = layout_view_node(ctx, view, x + cursor, y + my, w);
                h_meta.push((laid.size.0, style.flex.unwrap_or(0.0)));
                cursor += laid.size.0;
                cross_max = cross_max.max(my + laid.size.1);
            }
        }
    }
    // P679-D1②：Horizontal 行 flex 份额分配——flex 子级（flex-1/flex-auto）
    // 按因子均分「行宽 − gap 总额 − 非伸缩子级自然宽」，非伸缩子级维持
    // 首轮自然宽（iced Row 行为对齐：003 双 flex-1 字段列 200+200 而非
    // 各自全宽横向溢出）。弃置首轮重排会二次累加槽位计数/位图/覆盖层
    // 登记——全套快照回滚后以同一槽序重注册（聚焦身份不变）。Vertical
    // 列 flex 暂不消费（登记边界）。
    let mut flex_applied = false;
    if dir == Dir::Horizontal && h_meta.iter().any(|(_, f)| *f > 0.0) {
        let gaps_total = gap * h_meta.len().saturating_sub(1) as f32;
        let fixed_total: f32 = h_meta.iter().filter(|(_, f)| *f == 0.0).map(|(n, _)| *n).sum();
        let flex_total: f32 = h_meta.iter().filter(|m| m.1 > 0.0).map(|m| m.1).sum();
        let share = ((w - gaps_total - fixed_total) / flex_total.max(1.0)).max(0.0);
        let snap = (
            ctx.input_slots,
            ctx.select_slots,
            ctx.canvas_slots,
            ctx.pending_bitmaps.len(),
            ctx.overlays.len(),
            ctx.popover_overlays.len(),
            ctx.right_hits.len(),
            ctx.uncovered.len(),
            ctx.canvas_scene_sig.clone(),
        );
        ctx.ops.truncate(ops_mark);
        ctx.hits.truncate(hits_mark);
        ctx.right_hits.truncate(snap.6);
        ctx.overlays.truncate(snap.4);
        ctx.popover_overlays.truncate(snap.5);
        ctx.pending_bitmaps.truncate(snap.3);
        ctx.uncovered.truncate(snap.7);
        ctx.canvas_scene_sig = snap.8;
        ctx.input_slots = snap.0;
        ctx.select_slots = snap.1;
        ctx.canvas_slots = snap.2;
        let mut cursor2 = 0.0f32;
        let mut cross2 = 0.0f32;
        let mut first2 = true;
        let mut mi = 0usize;
        for view in views {
            let style = node_style_of_view(view);
            if style.hidden || style.absolute {
                continue;
            }
            if !first2 {
                cursor2 += gap;
            }
            first2 = false;
            let my = style.margin_y();
            let (natural, flex) = h_meta[mi];
            mi += 1;
            let inner = if flex > 0.0 { share * flex } else { natural };
            let laid = layout_view_node(ctx, view, x + cursor2, y + my, inner);
            cursor2 += laid.size.0;
            cross2 = cross2.max(my + laid.size.1);
        }
        cursor = cursor2.max(0.0);
        cross_max = cross2;
        flex_applied = true;
        if std::env::var("AUTO_FIT_TRACE").as_deref() == Ok("1") {
            eprintln!("[rq-flex] w={w} gaps={gaps_total} fixed={fixed_total} share={share} meta={h_meta:?}");
        }
    }
    if dir == Dir::Horizontal && parent.center_children && !views.is_empty() && !flex_applied {
        // 主轴居中：撤首轮产物后以居中起点重排（client_runtime 同款两遍法）。
        let used = cursor;
        ctx.ops.truncate(ops_mark);
        ctx.hits.truncate(hits_mark);
        let slack = (w - used).max(0.0) / 2.0;
        let mut cursor = slack;
        let mut cross_max = 0.0f32;
        let mut first = true;
        for view in views {
            let style = node_style_of_view(view);
            if style.hidden || style.absolute {
                continue;
            }
            if !first {
                cursor += gap;
            }
            first = false;
            let my = style.margin_y();
            let laid = layout_view_node(ctx, view, x + cursor, y + my, w);
            cursor += laid.size.0;
            cross_max = cross_max.max(my + laid.size.1);
        }
        let size = (w.max(cursor - slack), cross_max);
        place_absolute_children(ctx, &absolute_children, x, y, w, size.1);
        return Laid { size };
    }
    if dir == Dir::Vertical && parent.center_children && !child_widths.is_empty() {
        // 交叉轴居中（items-center 列臂）：首轮自然宽已录 → 撤产物后以
        // (w - natural_w)/2 起点重排（Horizontal 主轴两遍法同构；块流
        // 无 Fill——自然宽 = 首轮 laid.size.0，重排零测量漂移；固定宽
        // 子级的自身居中臂与本通道偏移恒等，双录不冲突）。
        ctx.ops.truncate(ops_mark);
        ctx.hits.truncate(hits_mark);
        let mut cursor = 0.0f32;
        let mut cross_max = 0.0f32;
        let mut first = true;
        let mut wi = 0usize;
        for view in views {
            let style = node_style_of_view(view);
            if style.hidden || style.absolute {
                continue;
            }
            if !first {
                cursor += gap;
            }
            first = false;
            let my = style.margin_y();
            let inner_w = style.fixed_w().unwrap_or(w);
            let natural_w = child_widths[wi];
            wi += 1;
            let child_x = x + (w - natural_w).max(0.0) / 2.0;
            let laid = layout_view_node(ctx, view, child_x, y + cursor + my, inner_w);
            cursor += my + laid.size.1 + style.box_layout.margin_bottom.unwrap_or(0.0);
            cross_max = cross_max.max(laid.size.0);
        }
        let size = (cross_max.max(0.0), cursor.max(0.0));
        place_absolute_children(ctx, &absolute_children, x, y, w, size.1);
        return Laid { size };
    }
    // 垂直块高度 = cursor（主轴累计）——515 G1 修正同源。
    let size = match dir {
        Dir::Vertical => (cross_max.max(0.0), cursor.max(0.0)),
        Dir::Horizontal => (cursor.max(0.0), cross_max.max(0.0)),
    };
    place_absolute_children(ctx, &absolute_children, x, y, w, size.1);
    Laid { size }
}

/// 单节点布局：容器（bg/padding/子级）或叶子 widget。返回外框尺寸。
fn layout_view_node<M: Clone + std::fmt::Debug>(
    ctx: &mut NativeCtx<M>,
    view: &View<M>,
    x: f32,
    y: f32,
    avail_w: f32,
) -> Laid {
    let style = node_style_of_view(view);
    // PLAN-032 T-02（D3）：hidden 单一 choke——display:none 子树整体
    // 不渲染不占位（零 ops 零尺寸；兄弟节点如同不存在——flex-1/居中
    // 两遍法聚合连锁零贡献自然成立）。
    if style.hidden {
        return Laid { size: (0.0, 0.0) };
    }
    match view {
        View::Empty => Laid { size: (0.0, 0.0) },
        // 块锚定槽：VM 轨专用透传壳（native 生成物不产——防御透传）。
        View::AnchorSlot { child, .. } => layout_view_node(ctx, child, x, y, avail_w),
        View::Text { content, .. } => {
            if content.is_empty() {
                return Laid { size: (0.0, 0.0) };
            }
            let size = style.font_size.unwrap_or(TEXT_SIZE);
            let line_h = size * LINE_H_FACTOR;
            // PLAN-036 T-02：truncate 真渲——超 avail_w 时逐字收缩 + `…`
            // 尾接（CSS truncate 语义的单行省略号近似；018 债清偿）。
            let mut text = content.clone();
            let mut w = measure_text(&text, size);
            if style.truncate && avail_w > 0.0 {
                const ELLIPSIS: &str = "…";
                let ell_w = measure_text(ELLIPSIS, size);
                if w > avail_w {
                    while w + ell_w > avail_w && !text.is_empty() {
                        text.pop();
                        w = measure_text(&text, size);
                    }
                    text.push_str(ELLIPSIS);
                    w = measure_text(&text, size);
                }
            }
            // PLAN-032 T-02（D5）：text_center 之外，center_children
            //（SelfCenter/mx-auto 族）同作水平居中——收缩文本（自然宽）
            // 的居中在臂内消费（块流 center_children 通道只覆盖固定宽子级）。
            // P679-D1③：文本换行（wire 无富排——投影端按 avail_w 预折行，
            // 逐行发射同款 op；词优先 + CJK 字符兜底）。truncate 档维持
            // 单行省略号语义不折行。
            let mut lines: Vec<(String, f32)> = Vec::new();
            if !style.truncate && avail_w > 0.0 && w > avail_w {
                let mut cur = String::new();
                let mut cur_w = 0.0f32;
                let mut max_w = 0.0f32;
                for word in text.split(' ') {
                    if word.is_empty() {
                        continue;
                    }
                    let pw = measure_text(word, size);
                    let sep = if cur.is_empty() { 0.0 } else { measure_text(" ", size) };
                    if cur_w + sep + pw <= avail_w {
                        if !cur.is_empty() {
                            cur.push(' ');
                            cur_w += sep;
                        }
                        cur.push_str(word);
                        cur_w += pw;
                    } else if pw > avail_w {
                        if !cur.is_empty() {
                            max_w = max_w.max(cur_w);
                            lines.push((core::mem::take(&mut cur), cur_w));
                            cur_w = 0.0;
                        }
                        for ch in word.chars() {
                            let cw = measure_text(&ch.to_string(), size);
                            if cur_w + cw > avail_w && !cur.is_empty() {
                                max_w = max_w.max(cur_w);
                                lines.push((core::mem::take(&mut cur), cur_w));
                                cur_w = 0.0;
                            }
                            cur.push(ch);
                            cur_w += cw;
                        }
                    } else {
                        max_w = max_w.max(cur_w);
                        lines.push((core::mem::take(&mut cur), cur_w));
                        cur.push_str(word);
                        cur_w = pw;
                    }
                }
                max_w = max_w.max(cur_w);
                lines.push((cur, cur_w));
                w = max_w;
            } else {
                lines.push((text.clone(), w));
            }
            let line_n = lines.len().max(1) as f32;
            let tx = x;
            // bg 徽章面（bg-secondary rounded-full px-3 py-1 形态——004
            // role 徽章）：文本自带 bg 时先垫底盒（圆角随 style；宽 =
            // 自然宽 + px-3 等效 24）。
            if let Some(bg) = style.bg {
                let box_w = (w + 24.0).min(if avail_w > 0.0 { avail_w } else { w + 24.0 });
                ctx.push_quad_rounded(
                    WRect::new(tx, y - 4.0, box_w, line_n * line_h + 8.0),
                    bg,
                    style.radius,
                );
            }
            for (li, (line, lw)) in lines.iter().enumerate() {
                let ltx = if style.text_center || style.center_children {
                    tx + (avail_w - lw).max(0.0) / 2.0
                } else {
                    tx
                };
                let ly = y + li as f32 * line_h;
                if style.font_bold {
                    ctx.ops.push(DrawOp::TextStyled {
                        x: ltx,
                        y: ly,
                        size,
                        line_height: line_h,
                        color: style.fg.unwrap_or(text_fg()),
                        weight: 700,
                        italic: false,
                        text: line.clone(),
                    });
                } else {
                    ctx.ops.push(DrawOp::Text {
                        x: ltx,
                        y: ly,
                        size,
                        line_height: line_h,
                        color: style.fg.unwrap_or(text_fg()),
                        text: line.clone(),
                    });
                }
            }
            Laid { size: (w, line_n * line_h) }
        }
        View::Button { label, onclick, on_right_click, content, disabled, .. } => {
            let size = style.font_size.unwrap_or(14.0);
            let label_w = measure_text(label, size);
            let w = style
                .fixed_w()
                .unwrap_or((label_w + BUTTON_PAD * 2.0).max(BUTTON_MIN_W))
                .min(avail_w.max(0.0));
            let h = style.fixed_h().unwrap_or(BUTTON_H);
            let bg = dim_if(*disabled, style.bg.unwrap_or(button_bg()));
            ctx.push_quad_rounded(WRect::new(x, y, w, h), bg, style.radius);
            if let Some(border) = style.border {
                ctx.push_border(WRect::new(x, y, w, h), border);
            }
            if let Some(content) = content {
                // link 转换形态：内容子树在按钮盒内顶流排布（v1 边界随注——
                // 不做盒内垂直居中）。
                let _ = layout_view_block(
                    ctx,
                    std::slice::from_ref(content.as_ref()),
                    x,
                    y,
                    w,
                    Dir::Vertical,
                    &style,
                );
            } else {
                let line_h = size * LINE_H_FACTOR;
                ctx.ops.push(DrawOp::Text {
                    x: x + (w - label_w) / 2.0,
                    y: y + (h - line_h) / 2.0,
                    size,
                    line_height: line_h,
                    color: dim_if(*disabled, style.fg.unwrap_or(text_fg())),
                    text: label.clone(),
                });
            }
            if !disabled {
                ctx.hits.push(HitEntry::Msg {
                    rect: WRect::new(x, y, w, h),
                    msg: onclick.clone(),
                });
            }
            // 右键命中（T-05）：on_right_click 在场且未禁用才登记。
            if !disabled {
                if let Some(rc) = on_right_click {
                    ctx.right_hits.push((WRect::new(x, y, w, h), rc.clone()));
                }
            }
            Laid { size: (w, h) }
        }
        // —— PLAN-025 T-02 form 族臂（视觉镜像解释态 layout_input/
        // layout_textarea/checkbox/radio；命中/聚焦/编辑闭环 = 分型命中表
        // + 聚焦槽位，T-01 D1/D2 定案）。
        View::Input { placeholder, value, on_change, width, .. } => {
            layout_view_input(
                ctx,
                placeholder,
                value,
                on_change.as_ref(),
                false,
                style.fixed_w().or_else(|| width.map(f32::from)),
                None,
                &style,
                x,
                y,
                avail_w,
            )
        }
        View::Textarea { placeholder, value, on_change, height, .. } => {
            // 多行框复用 input 命中/编辑闭环（槽位合一计数——D1）；行数
            // = height px 折行数（rows 语义的解释态缺省 4 行档对齐）。
            let size = style.font_size.unwrap_or(14.0);
            let line_h = size * LINE_H_FACTOR;
            let rows_h = |px: f32| INPUT_PAD * 2.0 + line_h * (px / line_h).max(1.0);
            let h = style
                .fixed_h()
                .or_else(|| height.map(|v| rows_h(f32::from(v))))
                .unwrap_or_else(|| rows_h(line_h * 4.0));
            layout_view_input(
                ctx,
                placeholder,
                value,
                on_change.as_ref(),
                true,
                style.fixed_w(),
                Some(h),
                &style,
                x,
                y,
                avail_w,
            )
        }
        // checkbox/radio：勾选图形 + 标签文本；命中 = handler 在场才登记
        // （native 无字段写回路径——自动翻转不可达，T-01 D2）。
        View::Checkbox { is_checked, label, on_toggle, .. } => {
            layout_view_toggle(ctx, *is_checked, label, on_toggle.as_ref(), &style, x, y, avail_w, false)
        }
        View::Radio { label, is_selected, on_select, .. } => {
            layout_view_toggle(ctx, *is_selected, label, on_select.as_ref(), &style, x, y, avail_w, true)
        }
        // PLAN-025 T-03 slider 臂：track 底 + fill + knob（值比例几何）+
        // 轨道命中（点击 → f32 → fn 指针物化派发；step 取整在派发侧）。
        View::Slider { min, max, value, on_change, step, .. } => {
            let w = style.fixed_w().unwrap_or(avail_w.min(320.0)).min(avail_w.max(0.0));
            let h = style.fixed_h().unwrap_or(SLIDER_H);
            let cy = y + h / 2.0;
            let t = if *max > *min {
                ((*value - *min) / (*max - *min)).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let vx = x + w * t;
            ctx.push_quad(WRect::new(x, cy - SLIDER_TRACK_H / 2.0, w, SLIDER_TRACK_H), progress_track());
            if vx > x {
                ctx.push_quad(WRect::new(x, cy - SLIDER_TRACK_H / 2.0, vx - x, SLIDER_TRACK_H), primary_fill());
            }
            ctx.push_quad(
                WRect::new(vx - SLIDER_KNOB / 2.0, cy - SLIDER_KNOB / 2.0, SLIDER_KNOB, SLIDER_KNOB),
                text_fg(),
            );
            // None 不登记（无动作面——HitEntry::Slider 文档同口径）。
            if on_change.is_some() {
                ctx.hits.push(HitEntry::Slider {
                    rect: WRect::new(x, y, w, h),
                    min: *min,
                    max: *max,
                    step: *step,
                    on_change: on_change.clone(),
                });
            }
            Laid { size: (w, h) }
        }
        // PLAN-025 T-04 select 臂（D3）：闭态 = 值盒 + ▾ + 点击开；开态
        // = 值盒照常 + 覆盖序选项列（render_frame 尾追加）+ 命中互斥。
        View::Select { options, selected_index, on_select, .. } => {
            let w = style.fixed_w().unwrap_or(avail_w.min(320.0)).min(avail_w.max(0.0));
            let h = style.fixed_h().unwrap_or(INPUT_H);
            let slot = ctx.select_slots;
            ctx.select_slots += 1;
            let open = ctx.select_open == Some(slot);
            ctx.push_quad_rounded(WRect::new(x, y, w, h), style.bg.unwrap_or(input_bg()), style.radius);
            ctx.push_border(WRect::new(x, y, w, h), style.border.unwrap_or(input_border()));
            let size = style.font_size.unwrap_or(14.0);
            let line_h = size * LINE_H_FACTOR;
            if let Some(label) = selected_index.and_then(|i| options.get(i)) {
                ctx.ops.push(DrawOp::Text {
                    x: x + INPUT_PAD,
                    y: y + (h - line_h) / 2.0,
                    size,
                    line_height: line_h,
                    color: style.fg.unwrap_or(text_fg()),
                    text: label.clone(),
                });
            }
            ctx.ops.push(DrawOp::Text {
                x: x + w - 14.0,
                y: y + (h - line_h) / 2.0,
                size,
                line_height: line_h,
                color: placeholder_fg(),
                text: '\u{25be}'.to_string(),
            });
            if !open {
                ctx.hits.push(HitEntry::SelectBox { rect: WRect::new(x, y, w, h), slot });
            } else {
                ctx.overlays.push(SelectOverlay {
                    rect: WRect::new(x, y, w, h),
                    options: options.clone(),
                    selected_index: *selected_index,
                    on_select: on_select.clone(),
                });
            }
            Laid { size: (w, h) }
        }
        // PLAN-026 T-03 display 族臂（I4：保真口径 = 解释态 queue 臂同级
        // 占位——client_runtime::layout_image / layout_progress
        // 镜像；ImageSurface 仍落 catch-all 占位盒，D5 整 kind
        // not-yet 在册）。
        View::Image { src, .. } => {
            // PLAN-028 真图升级（v1.9，与解释态 layout_image 同刻度）：
            // src 在场 → `DrawOp::Image`（tag 6；rect 推导零变化——026
            // §1.8 占位保真注释核销，占位转宿主侧未解析兜底语义）。
            // icon 经 codegen 降级到本臂（src = "lucide:{name}"，宿主
            // 字形解析 not-yet → 未解析降级占位，行为与旧占位口径连续）。
            let w = style.fixed_w().unwrap_or(avail_w.min(96.0)).min(avail_w.max(0.0));
            let h = style.fixed_h().unwrap_or(w);
            if src.is_empty() {
                ctx.push_quad(WRect::new(x, y, w, h), style.bg.unwrap_or(image_placeholder()));
            } else {
                ctx.ops.push(DrawOp::Image {
                    rect: WRect::new(x, y, w, h),
                    src: src.clone(),
                    fit: ImageFit::Stretch,
                });
            }
            Laid { size: (w, h) }
        }
        // PLAN-034 T-05（D4 裁定：canvas=位图快照过线）——场景栅格化
        // （tiny_skia，inproc CanvasPainter 的进程外孪生：映射规约共享、
        // 代码独立）→ 待上传位图 + `bitmap://{pid}-canvas-{slot}` 引用；
        // 同签名场景跳过重上传（pen 级高频重排的带宽抑制）。on_hit =
        // 节点命中物化（049 图元三表同律）；pen 三件套 not-yet（坐标
        // 回传路径归 M7-c terminal 撞面批裁定）；labels not-yet（软栅格
        // 无文本面——043 样板 strokes 主路径无撞）。
        View::Canvas { scene, logical_extent, clear, on_hit, style: _, .. } => {
            // style 用 fn 顶 NodeStyle（解构位是 Option<Style> 原始声明）。
            let logical_extent = *logical_extent;
            let (dw, dh) = logical_extent.unwrap_or((avail_w.max(1.0), 240.0));
            let w = style.fixed_w().unwrap_or(dw).max(1.0);
            let h = style.fixed_h().unwrap_or(dh).max(1.0);
            let slot = ctx.canvas_slots;
            ctx.canvas_slots += 1;
            let local_id = super::endpoint::bitmap_local_id(&format!("canvas-{slot}"));
            let sig = format!("{scene:?}{clear:?}");
            let changed = ctx.canvas_scene_sig.get(slot) != Some(&sig);
            if changed {
                if let Some(rgba) = rasterize_canvas_scene(
                    scene,
                    clear.as_deref(),
                    logical_extent,
                    w.ceil() as u32,
                    h.ceil() as u32,
                ) {
                    ctx.pending_bitmaps.push(super::endpoint::BitmapUpload {
                        id: local_id.clone(),
                        w: w.ceil() as u32,
                        h: h.ceil() as u32,
                        stride: w.ceil() as u32 * 4,
                        rgba,
                    });
                }
                if ctx.canvas_scene_sig.len() <= slot {
                    ctx.canvas_scene_sig.resize(slot + 1, String::new());
                }
                ctx.canvas_scene_sig[slot] = sig;
            }
            ctx.ops.push(DrawOp::Image {
                rect: WRect::new(x, y, w, h),
                src: format!("bitmap://{local_id}"),
                fit: ImageFit::Stretch,
            });
            if let Some(on_hit) = on_hit {
                let (sx, sy) = match logical_extent {
                    Some((ew, eh)) if ew > 0.0 && eh > 0.0 => (w / ew, h / eh),
                    _ => (1.0, 1.0),
                };
                for node in &scene.nodes {
                    let (nx, ny) = (x + node.x * sx, y + node.y * sy);
                    let rect = if node.shape == "rect" {
                        let (rw, rh) = (
                            node.w.unwrap_or(40.0) * sx,
                            node.h.unwrap_or(40.0) * sy,
                        );
                        WRect::new(nx - rw / 2.0, ny - rh / 2.0, rw, rh)
                    } else {
                        let r = node.r.unwrap_or(16.0) * sx;
                        WRect::new(nx - r, ny - r, r * 2.0, r * 2.0)
                    };
                    ctx.hits.push(HitEntry::Msg { rect, msg: on_hit.call(node.id.clone()) });
                }
            }
            Laid { size: (w, h) }
        }
        View::ProgressBar { progress, .. } => {
            // 轨道 + 填充条比例几何；on_seek 点击定位 not-yet（解释态
            // queue 臂同边界——I3 留痕）。
            let frac = progress.clamp(0.0, 1.0);
            let w = style.fixed_w().unwrap_or(avail_w).min(avail_w.max(0.0));
            let h = style.fixed_h().unwrap_or(8.0);
            ctx.push_quad(WRect::new(x, y, w, h), progress_track());
            if frac > 0.0 {
                ctx.push_quad(WRect::new(x, y, w * frac, h), style.bg.unwrap_or(primary_fill()));
            }
            Laid { size: (w, h) }
        }
        // PLAN-026 T-04 grid walker（镜像 client_runtime::layout_grid
        // :831——cols 等宽格 × row-major 行序，行高 = 行内最大，bg 底色
        // 两遍法置子级之下）。
        View::Grid { cols, gap, cells, .. } => {
            // gap：Grid.gap 字段（a2r codegen .spacing() 通道）优先，
            // style gap- 类回退档（解释态 layout_grid 同序）。
            // PLAN-032 T-04（D4）：walker 体提取为 layout_grid_cells
            // 单源，样式版 grid 分岔共用（语义零变化）。
            let gap = if *gap > 0 { f32::from(*gap) } else { style.gap() };
            layout_grid_cells(ctx, cells, *cols, gap, &style, x, y, avail_w)
        }
        // PLAN-025 T-05 scrollable 臂：溢出裁剪（Scissor push/pop——镜像
        // client_runtime::layout_scroll :953-1020）+ 滚轮命中（on_scroll
        // 在场才登记——I3；先登记后走子级：嵌套时内层倒序胜，D5）。滚动
        // 偏移 = app 状态经 on_scroll 重入 view()（投影器只裁剪不缓存）。
        View::Scrollable { child, width, height, offset, on_scroll, .. } => {
            let outer_w = style
                .fixed_w()
                .or_else(|| width.map(f32::from))
                .unwrap_or(avail_w.max(0.0))
                .min(avail_w.max(0.0));
            let viewport_h = style.fixed_h().or_else(|| height.map(f32::from));
            let off = offset.unwrap_or((0.0, 0.0));
            let Some(vh) = viewport_h else {
                // 无固定高：自然高容器语义（零裁剪对）。
                let laid = layout_view_block(
                    ctx,
                    std::slice::from_ref(child.as_ref()),
                    x,
                    y,
                    outer_w,
                    Dir::Vertical,
                    &style,
                );
                return Laid { size: (outer_w, laid.size.1) };
            };
            // Scroll 命中先登记（on_scroll 在场才登记——I3 滚轮不路由
            // 留痕；content_h 两遍法后补——见下）。
            let scroll_idx = ctx.hits.len();
            if let Some(cb) = on_scroll {
                ctx.hits.push(HitEntry::Scroll {
                    rect: WRect::new(x, y, outer_w, vh),
                    offset: off,
                    viewport: (outer_w, vh),
                    content: (outer_w, 0.0),
                    callback: cb.clone(),
                });
            }
            let ops_mark = ctx.ops.len();
            let hits_mark = ctx.hits.len();
            let laid = layout_view_block(
                ctx,
                std::slice::from_ref(child.as_ref()),
                x,
                y,
                outer_w,
                Dir::Vertical,
                &style,
            );
            let content_h = laid.size.1;
            if content_h > vh + 0.01 {
                // 溢出：撤首轮 → Scissor push → 子级 → pop；视口外命中
                // 区不登记（几何判交——嵌套组合正确，解释态同款）。
                ctx.ops.truncate(ops_mark);
                ctx.ops.push(DrawOp::Scissor { rect: WRect::new(x, y, outer_w, vh) });
                let _ = layout_view_block(
                    ctx,
                    std::slice::from_ref(child.as_ref()),
                    x,
                    y,
                    outer_w,
                    Dir::Vertical,
                    &style,
                );
                ctx.ops.push(DrawOp::ScissorPop);
                let clip = WRect::new(x, y, outer_w, vh);
                let child_hits: Vec<HitEntry<M>> = ctx.hits.drain(hits_mark..).collect();
                for h in child_hits {
                    let r = h.rect();
                    let intersects =
                        r.x < clip.x + clip.w && r.x + r.w > clip.x && r.y < clip.y + clip.h && r.y + r.h > clip.y;
                    if intersects {
                        ctx.hits.push(h);
                    }
                }
            }
            if let Some(HitEntry::Scroll { content, .. }) = ctx.hits.get_mut(scroll_idx) {
                content.1 = content_h;
            }
            // 视口占位 = viewport_h（溢出不影响兄弟节点位置——解释态同款）。
            Laid { size: (outer_w, vh) }
        }
        View::Row { .. } => {
            let dir = Dir::Horizontal;
            layout_view_group(ctx, view, &style, x, y, avail_w, dir)
        }
        View::Column { .. } | View::List { .. } => {
            let dir = Dir::Vertical;
            layout_view_group(ctx, view, &style, x, y, avail_w, dir)
        }
        View::Container { child, on_right_click, center_x, center_y, width, height, .. } => {
            layout_view_container(
                ctx,
                child,
                on_right_click.clone(),
                *center_x,
                *center_y,
                *width,
                *height,
                &style,
                x,
                y,
                avail_w,
            )
        }
        // PLAN-029 T-04（D3）：popover 臂——锚子树主流量渲染；开态 = 面板
        // 子树临时 ctx @0,0 走线（槽位计数接续——树序身份稳定）→ 量尺
        // 寸 → 几何定原点 → 平移入覆盖序记录（主块后追加，paint order
        // 置顶）；闭态零面板 ops/hits（open 随帧，投影器零开合状态机）。
        View::Popover { anchor, content, placement, open, on_dismiss } => {
            let site = match anchor {
                PopoverAnchor::Widget(child) => {
                    let laid = layout_view_node(ctx, child, x, y, avail_w);
                    PopoverAnchorSite::Widget(WRect::new(x, y, laid.size.0, laid.size.1))
                }
                PopoverAnchor::Point { x: px, y: py } => {
                    PopoverAnchorSite::Point { x: *px, y: *py }
                }
            };
            if !*open {
                return match site {
                    PopoverAnchorSite::Widget(r) => Laid { size: (r.w, r.h) },
                    PopoverAnchorSite::Point { .. } => Laid { size: (0.0, 0.0) },
                };
            }
            // 面板子树走线（快照字段透传——content 内 input/select 焦点/
            // 开合渲染同册；Edge* 面板几何覆盖 = 贴边 sheet 尺寸）。
            let mut tmp = NativeCtx {
                ops: Vec::new(),
                hits: Vec::new(),
                uncovered: Vec::new(),
                pending_bitmaps: Vec::new(),
                canvas_slots: ctx.canvas_slots,
                canvas_scene_sig: Vec::new(),
                input_slots: ctx.input_slots,
                focused_input: ctx.focused_input,
                input_buffer: ctx.input_buffer.clone(),
                ime_preedit: ctx.ime_preedit.clone(),
                select_slots: ctx.select_slots,
                select_open: ctx.select_open,
                overlays: Vec::new(),
                popover_overlays: Vec::new(),
                viewport: ctx.viewport,
                last_right_click: ctx.last_right_click,
                right_hits: Vec::new(),
            };
            let laid = layout_view_node(&mut tmp, content, 0.0, 0.0, POP_W);
            // 开态 select 选项列同走 tmp（互斥面板内嵌时的追加点在 tmp 侧
            // 未发生——select overlay 只在 render_frame 顶层追加；嵌 select
            // 面板 v1 以闭态渲染，随注）。
            let panel = match placement {
                PopoverPlacement::EdgeLeft | PopoverPlacement::EdgeRight => {
                    (laid.size.0.max(200.0).min(ctx.viewport.0 / 2.0), ctx.viewport.1 - POP_MARGIN * 2.0)
                }
                PopoverPlacement::EdgeTop | PopoverPlacement::EdgeBottom => {
                    (ctx.viewport.0 - POP_MARGIN * 2.0, laid.size.1.min(ctx.viewport.1 / 2.0))
                }
                _ => (laid.size.0.max(40.0), laid.size.1.max(INPUT_H)),
            };
            let (px, py) =
                popover_panel_origin(&site, placement, panel, ctx.viewport, ctx.last_right_click);
            let rect = WRect::new(px, py, panel.0, panel.1);
            // 平移：面板子树 ops/hits + 右键表 + 嵌套 popover 记录（嵌套
            // 几何按 tmp 空间推导后整体平移——翻转边界以真视口近似，v1）。
            let ops = tmp.ops.iter().map(|op| shift_draw_op(op, px, py)).collect();
            let hits = tmp.hits.into_iter().map(|h| h.shifted(px, py)).collect();
            ctx.right_hits.extend(tmp.right_hits.into_iter().map(|(mut r, m)| {
                r.x += px;
                r.y += py;
                (r, m)
            }));
            for mut nested in std::mem::take(&mut tmp.popover_overlays) {
                nested.rect.x += px;
                nested.rect.y += py;
                nested.ops = nested.ops.iter().map(|op| shift_draw_op(op, px, py)).collect();
                nested.hits = nested.hits.into_iter().map(|h| h.shifted(px, py)).collect();
                ctx.popover_overlays.push(nested);
            }
            // 槽位计数回接（tmp 接续起点 = ctx 当前值——直接采纳终值）。
            ctx.input_slots = tmp.input_slots;
            ctx.select_slots = tmp.select_slots;
            ctx.uncovered.extend(tmp.uncovered);
            // PLAN-034：同上——面板 canvas 位图并回。
            ctx.canvas_slots = tmp.canvas_slots;
            ctx.pending_bitmaps.extend(tmp.pending_bitmaps);
            ctx.popover_overlays.push(PopoverOverlay {
                rect,
                modal: *placement == PopoverPlacement::Modal,
                on_dismiss: on_dismiss.clone(),
                ops,
                hits,
            });
            match site {
                PopoverAnchorSite::Widget(r) => Laid { size: (r.w, r.h) },
                PopoverAnchorSite::Point { .. } => Laid { size: (0.0, 0.0) },
            }
        }
        // PLAN-029 T-05（D4）：thumbnail/preview 桥接臂——虚拟引用语法
        // `thumbnail://{wid}!{fallback}` / `workspace://{ws}!{fallback}`
        //（028 宿主解析直用；miss → 宿主转 `lucide:{fallback}` 占位图标
        // 真渲——I3 降级升级，语法入册 §1.10）。几何同 Image 臂。
        View::WindowThumbnail { wid, fallback_icon, .. } => {
            let w = style.fixed_w().unwrap_or(avail_w.min(192.0)).min(avail_w.max(0.0));
            let h = style.fixed_h().unwrap_or(w * 112.0 / 192.0);
            let fallback = if fallback_icon.is_empty() { "app-window" } else { fallback_icon };
            ctx.ops.push(DrawOp::Image {
                rect: WRect::new(x, y, w, h),
                src: format!("thumbnail://{wid}!{fallback}"),
                fit: ImageFit::Stretch,
            });
            Laid { size: (w, h) }
        }
        View::WorkspacePreview { ws, fallback_icon, .. } => {
            let w = style.fixed_w().unwrap_or(avail_w.min(176.0)).min(avail_w.max(0.0));
            let h = style.fixed_h().unwrap_or(w * 64.0 / 176.0);
            let fallback = if fallback_icon.is_empty() { "app-window" } else { fallback_icon };
            ctx.ops.push(DrawOp::Image {
                rect: WRect::new(x, y, w, h),
                src: format!("workspace://{ws}!{fallback}"),
                fit: ImageFit::Stretch,
            });
            Laid { size: (w, h) }
        }
        // PLAN-029 T-06：MouseArea 透传 + 命中项——area 命中先 push、
        // content 子树后 push（rev 序 content 项优先，空白落 area——iced
        // mouse_area 冒泡语义的命中序等价）；contextmenu → 右键表。
        // on_enter/on_exit/on_move/on_release/on_double_click = hover/时序
        // 语义 not-yet（投影器无 hover 态，I3 随注——shell ×13 全
        // click/contextmenu 族不受影响）。
        View::MouseArea { content, on_click, on_context_menu, logical_extent, .. } => {
            let (ew, eh) = logical_extent.unwrap_or((avail_w.min(120.0), 24.0));
            let rect = WRect::new(x, y, ew, eh);
            // area 命中先 push（rev 序 content 项优先胜——空白落 area）。
            if let Some(msg) = on_click {
                ctx.hits.push(HitEntry::Msg { rect, msg: msg.clone() });
            }
            if let Some(msg) = on_context_menu {
                ctx.right_hits.push((rect, msg.clone()));
            }
            let laid = layout_view_node(ctx, content, x, y, avail_w);
            let (ew, eh) = logical_extent.unwrap_or(laid.size);
            Laid { size: (ew, eh) }
        }
        // PLAN-032 T-03（D2）：tabs kind 臂——标签托盘（等宽按钮态 Quad/
        // Text，选中态 bg/fg 差分）+ 内容区子树（contents[selected]）+
        // on_select 逐项命中（TabSelect.call(index) 物化——VM 轨首参 =
        // value 串在回调内包装，convert_tabs 契约；缺席不登记 = 受控
        // 点击不切换）。variant：default = 按钮托盘；enclosed = 连通
        // 形态（托盘底 + 选中下划线，PLAN-641 视觉子集）。position：
        // Top/Bottom = 托盘上/下；Left/Right 渲染降级 Top（not-yet 随注
        //——扫描面不含 position 载荷，判定无感）。
        View::Tabs { labels, contents, selected, position, on_select, variant, .. } => {
            let n = labels.len().max(1);
            let sel = (*selected).min(n - 1);
            let font = style.font_size.unwrap_or(14.0);
            let line_h = font * LINE_H_FACTOR;
            // 托盘项高：一行文本 + 上下各 4px 呼吸。
            let tab_h = line_h + 8.0;
            let tab_w = (avail_w.max(0.0) / n as f32).max(0.0);
            let draw_tray = |ctx: &mut NativeCtx<M>, tray_y: f32| {
                for (i, label) in labels.iter().enumerate() {
                    let tx = x + tab_w * i as f32;
                    let rect = WRect::new(tx, tray_y, tab_w, tab_h);
                    let active = i == sel;
                    match variant {
                        TabsVariant::Default => {
                            ctx.push_quad(
                                rect,
                                if active { style.bg.unwrap_or(primary_fill()) } else { input_bg() },
                            );
                            if let Some(border) = style.border {
                                ctx.push_border(rect, border);
                            }
                        }
                        TabsVariant::Enclosed => {
                            ctx.push_quad(rect, input_bg());
                            if active {
                                ctx.push_quad(
                                    WRect::new(tx, tray_y + tab_h - 2.0, tab_w, 2.0),
                                    style.bg.unwrap_or(primary_fill()),
                                );
                            }
                        }
                    }
                    let lw = measure_text(label, font);
                    ctx.ops.push(DrawOp::Text {
                        x: tx + (tab_w - lw).max(0.0) / 2.0,
                        y: tray_y + 4.0,
                        size: font,
                        line_height: line_h,
                        color: if active {
                            style.fg.unwrap_or(text_fg())
                        } else {
                            placeholder_fg()
                        },
                        text: label.clone(),
                    });
                    if let Some(cb) = on_select {
                        ctx.hits.push(HitEntry::TabSelect { rect, index: i, on_select: cb.clone() });
                    }
                }
            };
            let tray_first = !matches!(position, TabsPosition::Bottom);
            let (tray_y, content_y) = if tray_first { (y, y + tab_h) } else { (y + tab_h, y) };
            // 内容区：空内容 = 透明占位（enclosed 连通边框 v1 以托盘下缘
            // 线承载，内容边框 not-yet 随注——视觉子集，I3）。ops 序随
            // position 自然阅读序：Top 托盘先、Bottom 内容先（几何不重叠，
            // paint order 无牵连）。
            let content_h = if tray_first {
                draw_tray(ctx, tray_y);
                contents
                    .get(sel)
                    .map(|child| layout_view_node(ctx, child, x, content_y, avail_w))
                    .map(|l| l.size.1)
                    .unwrap_or(0.0)
            } else {
                let h = contents
                    .get(sel)
                    .map(|child| layout_view_node(ctx, child, x, content_y, avail_w))
                    .map(|l| l.size.1)
                    .unwrap_or(0.0);
                draw_tray(ctx, tray_y);
                h
            };
            let outer_w = style.fixed_w().unwrap_or(avail_w.max(0.0));
            Laid { size: (outer_w, tab_h + content_h) }
        }
        // PLAN-674 T-01（§10-1 裁定 A：结构 DrawOps）：codeeditor 投影臂。
        // 状态面 = CODE_EDITORS 注册表 get-or-create（`code_editor(sk,
        // &config)`——inproc iced widget 同源同键，renderer.rs VM 路径
        // 同款）；外部值经 `code_editor_set_text` 差分推入（last_external
        // 差分——视图值回推不覆写用户进行中编辑）。渲染面 =
        // `core::render::render`（视口虚拟化：text_runs 按可见 run ×
        // syntax span 发射——op 流有界）→ `lower_editor_frame`（Plan
        // 386 降层：gutter/当前行/选区/搜索/caret/preedit/滚动条全 op
        // 面）→ 平移 (x, y) 入流。命中 = Input 槽位族复用（editor =
        // 存储键——点击聚焦 + 光标定位，键入/IME/箭标走 core
        // handle_input，见 on_input 编辑器分派臂）；on_context_menu →
        // 右键表（MouseArea 同律）。折叠 gutter 点击/搜索面板跳转等
        // 宿主面板族 not-yet（I3 随注——消费方 L1/L2 结构+存活+满帧
        // 不依赖）。
        // 复审 R-1（§10-4 定边界）：`on_cursor` 显式降级弃置（I3 留痕）
        //——最小键入面之外（消费方声明位如 auto-edit oncursor:
        // .CursorMoved；inproc 轨经 widget on_cursor 派发，RQ 轨 v1
        // 不回传 caret 移动事件），接线随消费方需要另立（P674-D4）。
        View::CodeEditor {
            key,
            value,
            lang,
            line_numbers,
            wrap,
            vi,
            highlight_current_line,
            readonly,
            tab_width,
            font_size,
            on_change,
            on_cursor: _,   // R-1：显式降级弃置（见上注/P674-D4），非静默
            on_context_menu,
            search,
            style: _,
        } => {
            // 无 iced 宿主（VM/RQ 进程）的字体系统源安装（幂等——已装
            // 零影响；CODE_EDITORS 注册表触达前置）。
            ce::ensure_font_system_call();
            let config = ce::CodeEditorConfig {
                lang: lang.clone(),
                line_numbers: *line_numbers,
                wrap: *wrap,
                vi: *vi,
                highlight_current_line: *highlight_current_line,
                readonly: *readonly,
                tab_width: *tab_width as u16,
                font_size: *font_size,
            };
            let sk = ce::storage_key(key);
            let core = ce::code_editor(&sk, &config);
            ce::code_editor_set_text(&sk, value);
            if !search.is_empty() {
                core.set_search(search);
            }
            let w = style.fixed_w().unwrap_or(avail_w.max(0.0)).max(1.0);
            let h = style.fixed_h().unwrap_or(240.0).max(1.0);
            let rect = WRect::new(x, y, w, h);
            let list =
                ce::with_font_system(|fs| crate::ui::code_editor::core::render::render(core, fs, w, h, None));
            let frame = super::editor_frame::lower_editor_frame(&list);
            for op in &frame.ops {
                ctx.ops.push(shift_draw_op(op, x, y));
            }
            // 编辑器恒登记 Input 命中（聚焦/光标定位即交互面——on_change
            // 缺席不省略；平面 input 的"登记省略"差分见 HitEntry::Input
            // 随注）。
            let slot = ctx.input_slots;
            ctx.input_slots += 1;
            ctx.hits.push(HitEntry::Input {
                rect,
                value: value.clone(),
                on_change: on_change.clone(),
                slot,
                editor: Some(sk),
            });
            if let Some(msg) = on_context_menu {
                ctx.right_hits.push((rect, msg.clone()));
            }
            Laid { size: (w, h) }
        }
        // —— 覆盖门后动态分支防线：占位盒 + 留痕（I3：非静默错绘）。
        other => {
            let kind = coverage::native_kind_of(other);
            ctx.uncovered.push(kind.to_string());
            let w = avail_w.min(160.0).max(40.0);
            let h = 24.0;
            ctx.push_quad(WRect::new(x, y, w, h), input_bg());
            ctx.ops.push(DrawOp::Text {
                x: x + 6.0,
                y: y + 5.0,
                size: 12.0,
                line_height: 16.0,
                color: placeholder_fg(),
                text: format!("not-rendered: {kind}"),
            });
            Laid { size: (w, h) }
        }
    }
}

/// 线性堆叠族（Row/Column/List）：legacy spacing/padding 兜底（style 未
/// 声明 gap/padding 时消费 builder legacy 字段——iced 兼容面同源）。
fn layout_view_group<M: Clone + std::fmt::Debug>(
    ctx: &mut NativeCtx<M>,
    view: &View<M>,
    style: &NodeStyle,
    x: f32,
    y: f32,
    avail_w: f32,
    dir: Dir,
) -> Laid {
    let (children, legacy_spacing, legacy_padding) = match view {
        View::Row { children, spacing, padding, .. }
        | View::Column { children, spacing, padding, .. } => (children, *spacing, *padding),
        View::List { items, spacing, .. } => (items, *spacing, 0u16),
        _ => unreachable!("layout_view_group 只接堆叠族"),
    };
    // legacy 兜底仅补 style 未声明的槽（style 优先——View 变体注释口径）。
    let mut group_style = style.clone();
    if group_style.box_layout.gap.is_none() && legacy_spacing > 0 {
        group_style.box_layout.gap = Some(f32::from(legacy_spacing));
    }
    let no_pad = group_style.box_layout.padding_top.is_none()
        && group_style.box_layout.padding_left.is_none();
    if no_pad && legacy_padding > 0 {
        let p = f32::from(legacy_padding);
        group_style.box_layout.padding_top = Some(p);
        group_style.box_layout.padding_bottom = Some(p);
        group_style.box_layout.padding_left = Some(p);
        group_style.box_layout.padding_right = Some(p);
    }
    // PLAN-679 Phase 2ï¼max_width é³å¶ï¼max-w-md å¡çæ¶çªï¼ã
    let avail_w = group_style
        .box_layout
        .max_width
        .map_or(avail_w, |mw| avail_w.min(mw));
    // PLAN-679 Phase 2ï¼groupï¼Row/Column/Listï¼è¡¥é½ container åæ¬¾
    // è§è§é¢ï¼padding æ¶è´¹ + bg/æ¸å/åè§/borderï¼ï¼îï¼æ­¤å group èªèº«ä¸ç» bgï¼
    // bg-card å¡çå¨ RQ è½¨ä¸å¯è§ï¼ãæ¨¡å¼å containerï¼éå°ºå¯¸ â æ¤äº§ç© â
    // bg/æ¸å â éæå­çº§ â borderã
    let pad = (
        group_style.pad_left(),
        group_style.pad_top(),
        group_style.pad_right(),
        group_style.pad_bottom(),
    );
    let inner_w = (avail_w - pad.0 - pad.2).max(0.0);
    let has_visual = group_style.bg.is_some()
        || group_style.border.is_some()
        || group_style.radius.is_some()
        || group_style.grad_dir.is_some();
    let ops_mark = ctx.ops.len();
    let hits_mark = ctx.hits.len();
    let laid = layout_view_block(
        ctx,
        children,
        x + pad.0,
        y + pad.1,
        inner_w,
        dir,
        &group_style,
    );
    let mut laid = laid;
    if has_visual {
        let outer_w = laid.size.0 + pad.0 + pad.2;
        let outer_h = laid.size.1 + pad.1 + pad.3;
        let outer = WRect::new(x, y, outer_w, outer_h);
        ctx.ops.truncate(ops_mark);
        ctx.hits.truncate(hits_mark);
        if let (Some(vertical), Some(from), Some(to)) =
            (group_style.grad_dir, group_style.grad_from, group_style.grad_to)
        {
            ctx.push_gradient(outer, vertical, from, to);
        } else if let Some(bg) = group_style.bg {
            ctx.push_quad_rounded(outer, bg, group_style.radius);
        }
        let relaid = layout_view_block(
            ctx,
            children,
            x + pad.0,
            y + pad.1,
            inner_w,
            dir,
            &group_style,
        );
        laid = relaid;
        if let Some(border) = group_style.border {
            ctx.push_border(outer, border);
        }
        laid.size.0 = outer_w;
        laid.size.1 = outer_h;
    }
    // å³é®å½ä¸­ï¼T-05ï¼ï¼å¸å±ä»¶æ´æ¡ç»è®°ï¼Row/Column `on_right_click`ï¼ã
    // 右键命中（T-05）：布局件整框登记（Row/Column `on_right_click`）。
    let rc = match view {
        View::Row { on_right_click, .. } | View::Column { on_right_click, .. } => {
            on_right_click.clone()
        }
        _ => None,
    };
    if let Some(rc) = rc {
        ctx.right_hits.push((WRect::new(x, y, laid.size.0, laid.size.1), rc));
    }
    laid
}

/// 容器（View::Container）：bg/padding 包装 + 子级块流。z 序镜像
/// `client_runtime::layout_container`（底色先于子级——量尺寸 → 撤产物 →
/// bg → 重排子级 → border）。
fn layout_view_container<M: Clone + std::fmt::Debug>(
    ctx: &mut NativeCtx<M>,
    child: &View<M>,
    right_click: Option<M>,
    center_x: bool,
    center_y: bool,
    legacy_width: Option<u16>,
    legacy_height: Option<u16>,
    style: &NodeStyle,
    x: f32,
    y: f32,
    avail_w: f32,
) -> Laid {
    let container_right_click = right_click;
    let pad = (style.pad_left(), style.pad_top(), style.pad_right(), style.pad_bottom());
    // PLAN-026 T-04：legacy width/height 兜底（View::container/.center()
    // builder 字段——iced 消费面同源，typed style 优先）。
    let mut inner_w = avail_w;
    if let Some(fw) = style.fixed_w().or_else(|| legacy_width.map(f32::from)) {
        inner_w = (fw - pad.0 - pad.2).max(0.0);
    }
    if let Some(max_w) = style.box_layout.max_width {
        inner_w = inner_w.min((max_w - pad.0 - pad.2).max(0.0));
    }
    let ops_mark = ctx.ops.len();
    let hits_mark = ctx.hits.len();
    let laid = layout_view_block(
        ctx,
        std::slice::from_ref(child),
        x + pad.0,
        y + pad.1,
        inner_w.max(0.0),
        Dir::Vertical,
        style,
    );
    // PLAN-026 T-07：w-full（Width(Full)）= 块级满宽（iced 消费面同语义
    // ——divider/spacer 降级形态的宽度承载；无尺寸声明仍收内容宽）。
    let fills_w = matches!(
        style.box_layout.width,
        Some(crate::ui::style::SizeValue::Full)
    );
    let outer_w = match style.fixed_w().or_else(|| legacy_width.map(f32::from)) {
        Some(fw) => fw,
        None if fills_w => avail_w.max(0.0),
        None => (laid.size.0 + pad.0 + pad.2).max(0.0),
    };
    let outer_h = match style.fixed_h().or_else(|| legacy_height.map(f32::from)) {
        Some(fh) => fh,
        None => laid.size.1 + pad.1 + pad.3,
    };
    // PLAN-026 T-04 center：View::center/Container center_x/center_y 臂
    // （解释态 center 经 items-center/mx-auto 类两遍法同档；center_y 需
    // fixed_h 外框——自然高容器居中无位移）。
    let child_x = if center_x {
        x + pad.0 + ((inner_w.max(0.0) - laid.size.0).max(0.0)) / 2.0
    } else {
        x + pad.0
    };
    let child_y = if center_y && (style.fixed_h().is_some() || legacy_height.is_some()) {
        let inner_h = (outer_h - pad.1 - pad.3).max(0.0);
        y + pad.1 + ((inner_h - laid.size.1).max(0.0)) / 2.0
    } else {
        y + pad.1
    };
    if style.bg.is_some() || style.border.is_some() || child_x != x + pad.0 || child_y != y + pad.1
    {
        ctx.ops.truncate(ops_mark);
        ctx.hits.truncate(hits_mark);
        // PLAN-679 Phase 2：渐变优先（bg-gradient+from+to 三类齐备）→
        // 条带近似；否则普通/圆角 quad。
        if let (Some(vertical), Some(from), Some(to)) =
            (style.grad_dir, style.grad_from, style.grad_to)
        {
            ctx.push_gradient(WRect::new(x, y, outer_w, outer_h), vertical, from, to);
        } else if let Some(bg) = style.bg {
            ctx.push_quad_rounded(WRect::new(x, y, outer_w, outer_h), bg, style.radius);
        }
        let _ = layout_view_block(
            ctx,
            std::slice::from_ref(child),
            child_x,
            child_y,
            inner_w.max(0.0),
            Dir::Vertical,
            style,
        );
        if let Some(border) = style.border {
            ctx.push_border(WRect::new(x, y, outer_w, outer_h), border);
        }
    }
    // 右键命中（T-05）：容器整框登记（`on_right_click`——div 形态消费面）。
    if let Some(rc) = container_right_click {
        ctx.right_hits.push((WRect::new(x, y, outer_w, outer_h), rc));
    }
    Laid { size: (outer_w, outer_h) }
}

// ---------------------------------------------------------------------------
// PLAN-025 T-02 form 族臂（input/textarea/checkbox/radio——视觉镜像解释态）
// ---------------------------------------------------------------------------

/// input/textarea 布局臂：INPUT_BG 盒 + 1px 边框 + 焦点蓝描边（聚焦槽位
/// = 本框）+ placeholder/值文本 + Input 命中登记（镜像解释态
/// layout_input :1047-1126；多行 = 按 '\n' 分行自上而下、无自动换行——
/// 解释态 layout_textarea 同边界）。
#[allow(clippy::too_many_arguments)]
fn layout_view_input<M: Clone + std::fmt::Debug>(
    ctx: &mut NativeCtx<M>,
    placeholder: &str,
    value: &str,
    on_change: Option<&M>,
    multiline: bool,
    fixed_w: Option<f32>,
    h_override: Option<f32>,
    style: &NodeStyle,
    x: f32,
    y: f32,
    avail_w: f32,
) -> Laid {
    let w = fixed_w.unwrap_or_else(|| {
        if multiline {
            avail_w.max(0.0)
        } else {
            avail_w.min(320.0)
        }
    })
    .min(avail_w.max(0.0));
    let h = h_override.unwrap_or(INPUT_H);
    ctx.push_quad(WRect::new(x, y, w, h), style.bg.unwrap_or(input_bg()));
    let slot = ctx.input_slots;
    let focused = ctx.focused_input == Some(slot);
    let border = if focused { primary_fill() } else { input_border() };
    ctx.push_border(WRect::new(x, y, w, h), style.border.unwrap_or(border));
    ctx.input_slots += 1;
    // 显示面（D2）：聚焦框显 buffer（编辑面——解析失败时组件状态不变，
    // 用户意图仍可见），非聚焦框显视图值；空显 placeholder。
    // PLAN-026 T-05（D2-A）：IME preedit 尾拼——聚焦框 buffer 后接组合串
    // （单行 = 独立 Text op 差分色显示；多行 = 并入末行同色，边界随注）。
    let shown = if focused { ctx.input_buffer.clone() } else { value.to_string() };
    let preedit_tail =
        if focused { ctx.ime_preedit.clone().unwrap_or_default() } else { String::new() };
    let (text, color) = if shown.is_empty() && preedit_tail.is_empty() {
        (placeholder.to_string(), placeholder_fg())
    } else if multiline && !preedit_tail.is_empty() {
        (format!("{shown}{preedit_tail}"), style.fg.unwrap_or(text_fg()))
    } else {
        (shown.clone(), style.fg.unwrap_or(text_fg()))
    };
    let size = style.font_size.unwrap_or(14.0);
    let line_h = size * LINE_H_FACTOR;
    if !text.is_empty() && multiline {
        // 多行：按 '\n' 分行自上而下排（无自动换行——宽度溢出裁剪边界
        // 归宿主；保真边界随注，解释态 layout_textarea 同款）。
        let rows = (((h - INPUT_PAD * 2.0) / line_h).floor() as usize).max(1);
        for (i, line) in text.split('\n').take(rows).enumerate() {
            ctx.ops.push(DrawOp::Text {
                x: x + INPUT_PAD,
                y: y + INPUT_PAD + line_h * i as f32,
                size,
                line_height: line_h,
                color,
                text: line.to_string(),
            });
        }
    } else if !text.is_empty() {
        // preedit 尾拼 op（差分色——真下划线无 DrawOp 通道，PLACEHOLDER_FG
        // 近似 + 随注；文本排布 = buffer 尾 x 累进）。
        let tail = if focused { preedit_tail.clone() } else { String::new() };
        let shown_w = measure_text(&text, size);
        ctx.ops.push(DrawOp::Text {
            x: x + INPUT_PAD,
            y: y + (h - line_h) / 2.0,
            size,
            line_height: line_h,
            color,
            text,
        });
        if focused && !tail.is_empty() {
            ctx.ops.push(DrawOp::Text {
                x: x + INPUT_PAD + shown_w,
                y: y + (h - line_h) / 2.0,
                size,
                line_height: line_h,
                color: placeholder_fg(),
                text: tail,
            });
        }
    }
    if let Some(msg) = on_change {
        ctx.hits.push(HitEntry::Input {
            rect: WRect::new(x, y, w, h),
            value: value.to_string(),
            on_change: Some(msg.clone()),
            slot,
            editor: None,
        });
    }
    Laid { size: (w, h) }
}

/// checkbox/radio 布局臂：勾选盒（checkbox 18×18 / radio 16×16，圆形
/// 直角化保真边界——解释态同款内芯 inset）+ 标签文本；命中 = handler
/// 在场才登记（native 组件无字段写回路径，"handler 在场 = handler 拥有
/// 状态变更"——解释态 register_toggle 语义的物化消息形）。
#[allow(clippy::too_many_arguments)]
fn layout_view_toggle<M: Clone + std::fmt::Debug>(
    ctx: &mut NativeCtx<M>,
    checked: bool,
    label: &str,
    handler: Option<&M>,
    style: &NodeStyle,
    x: f32,
    y: f32,
    avail_w: f32,
    radio: bool,
) -> Laid {
    let (box_w, inset_factor, inset_clamp) = if radio {
        (16.0f32, 0.28f32, 5.0f32)
    } else {
        (18.0f32, 0.22f32, 6.0f32)
    };
    let w = style.fixed_w().unwrap_or(box_w).min(avail_w.max(0.0));
    let h = style.fixed_h().unwrap_or(w);
    ctx.push_quad(WRect::new(x, y, w, h), style.bg.unwrap_or(input_bg()));
    ctx.push_border(WRect::new(x, y, w, h), style.border.unwrap_or(input_border()));
    if checked {
        let inset = (w.min(h) * inset_factor).clamp(1.5, inset_clamp);
        ctx.push_quad(
            WRect::new(x + inset, y + inset, w - inset * 2.0, h - inset * 2.0),
            style.fg.unwrap_or(primary_fill()),
        );
    }
    let mut outer_w = w;
    let mut outer_h = h;
    if !label.is_empty() {
        let size = style.font_size.unwrap_or(14.0);
        let line_h = size * LINE_H_FACTOR;
        let label_w = measure_text(label, size);
        ctx.ops.push(DrawOp::Text {
            x: x + w + CHECK_LABEL_GAP,
            y: y + (h - line_h) / 2.0,
            size,
            line_height: line_h,
            color: style.fg.unwrap_or(text_fg()),
            text: label.to_string(),
        });
        outer_w = w + CHECK_LABEL_GAP + label_w;
        outer_h = h.max(line_h);
    }
    if let Some(msg) = handler {
        ctx.hits.push(HitEntry::Msg {
            rect: WRect::new(x, y, outer_w, outer_h),
            msg: msg.clone(),
        });
    }
    Laid { size: (outer_w, outer_h) }
}

// ---------------------------------------------------------------------------
// typed 样式适配器（StyleClass → NodeStyle；盒模复用 layout_extract）
// ---------------------------------------------------------------------------

/// View 变体 → NodeStyle（自带 `style` + legacy spacing/padding 兜底面由
/// `layout_view_group` 消费——此处仅 typed 类直填）。
fn node_style_of_view<M: Clone + std::fmt::Debug>(view: &View<M>) -> NodeStyle {
    let style = match view {
        View::Text { style, .. }
        | View::Button { style, .. }
        | View::Row { style, .. }
        | View::Column { style, .. }
        | View::List { style, .. }
        | View::Container { style, .. }
        | View::Input { style, .. }
        | View::Textarea { style, .. }
        | View::Checkbox { style, .. }
        | View::Radio { style, .. }
        | View::Select { style, .. }
        | View::Slider { style, .. }
        // PLAN-026 T-03：display 族（image/progress 占位臂消费样式
        // 尺寸/bg——scan_native_node 变体样式收集面同册）。
        | View::Image { style, .. }
        | View::ProgressBar { style, .. } => style.as_ref(),
        // PLAN-032 T-02：hidden/样式 grid choke 收集面扩容——Grid/
        // Scrollable/MouseArea 自带 style（Tabs 随 T-03 kind 臂入列）。
        | View::Grid { style, .. }
        | View::Scrollable { style, .. }
        | View::MouseArea { style, .. } => style.as_ref(),
        // PLAN-032 T-03：Tabs 托盘视觉消费（bg/fg/border/font——variant
        // 差分见 View::Tabs 臂）。
        | View::Tabs { style, .. } => style.as_ref(),
        // PLAN-674 T-01：codeeditor 尺寸面（w-/h- 定尺寸——臂内
        // fixed_w/fixed_h 消费；缺省 avail_w × 240）。
        | View::CodeEditor { style, .. } => style.as_ref(),
        _ => None,
    };
    node_style_of(style)
}

/// typed `Style` → NodeStyle：盒模走
/// `style::BoxLayout::from_style`（共享 parser/提取器——零复刻）；装饰/
/// 对齐/排版逐类直填。渲染未实现面（shadow/渐变 to 端/flex 族/字族）按
/// 解释态同款边界静默降级（覆盖门已把**不可降级面**挡在启动前——入册
/// 差异见 `Coverage::native_queue_set` 随注）。
fn node_style_of(style: Option<&Style>) -> NodeStyle {
    let mut s = NodeStyle::default();
    let Some(style) = style else { return s };
    s.box_layout = crate::ui::style::BoxLayout::from_style(style);
    for class in &style.classes {
        apply_style_class(class, &mut s);
    }
    s
}

fn apply_style_class(class: &StyleClass, s: &mut NodeStyle) {
    match class {
        StyleClass::BackgroundColor(c) => s.bg = resolve_typed_color(c).or(s.bg),
        // 渐变：取 from 端色平铺（解释态保真边界同款）。
        StyleClass::GradientFrom(c) => s.bg = s.bg.or(resolve_typed_color(c)),
        StyleClass::TextColor(c) => s.fg = resolve_typed_color(c).or(s.fg),
        StyleClass::Border | StyleClass::Border0 | StyleClass::BorderWidth(_) => {
            s.border = s.border.or(Some(input_border()));
        }
        StyleClass::BorderColor(c) => s.border = resolve_typed_color(c).or(s.border),
        StyleClass::TextXs => s.font_size = Some(12.0),
        StyleClass::TextSm => s.font_size = Some(14.0),
        StyleClass::TextBase => s.font_size = Some(16.0),
        StyleClass::TextLg => s.font_size = Some(18.0),
        StyleClass::TextXl => s.font_size = Some(20.0),
        StyleClass::Text2Xl => s.font_size = Some(24.0),
        StyleClass::Text3Xl => s.font_size = Some(30.0),
        StyleClass::Text4Xl => s.font_size = Some(36.0),
        StyleClass::Text5Xl => s.font_size = Some(48.0),
        StyleClass::FontBold | StyleClass::FontMedium => s.font_bold = true,
        // P679-D1②：flex 伸缩因子入 NodeStyle（Horizontal 行份额分配）。
        StyleClass::Flex1 | StyleClass::FlexAuto => s.flex = Some(1.0),
        StyleClass::FlexInitial | StyleClass::FlexNone => {}
        // PLAN-679 Phase 2：圆角档 + 渐变族入 NodeStyle。
        StyleClass::Rounded => s.radius = Some(4.0),
        StyleClass::RoundedSm => s.radius = Some(2.0),
        StyleClass::RoundedMd => s.radius = Some(6.0),
        StyleClass::RoundedLg => s.radius = Some(8.0),
        StyleClass::RoundedXl => s.radius = Some(12.0),
        StyleClass::Rounded2Xl => s.radius = Some(16.0),
        StyleClass::Rounded3Xl => s.radius = Some(24.0),
        StyleClass::RoundedFull => s.radius = Some(9999.0),
        StyleClass::RoundedNone => s.radius = None,
        StyleClass::BgGradient(d) => {
            s.grad_dir = Some(matches!(d, crate::ui::style::GradientDir::ToB
                | crate::ui::style::GradientDir::ToT
                | crate::ui::style::GradientDir::ToTR
                | crate::ui::style::GradientDir::ToTL));
        }
        StyleClass::GradientFrom(c) => s.grad_from = resolve_typed_color(c),
        StyleClass::GradientTo(c) => s.grad_to = resolve_typed_color(c),
        StyleClass::ItemsCenter | StyleClass::JustifyCenter | StyleClass::MarginXAuto => {
            s.center_children = true;
        }
        StyleClass::TextCenter => {
            s.center_children = true;
            s.text_center = true;
        }
        // rounded 档：命令帧无圆角 op——直角化（解释态保真边界同款）。
        // PLAN-032 T-02（D3）：hidden = display:none（布局单一 choke 跳过
        // 子树）；display 族类清位——"hidden md:flex" 经 parser 剥响应式
        // 前缀（class.rs sm/md/lg/xl/2xl）成 [Hidden, Flex]，类序后者胜
        // = CSS 桌面档覆盖语义（Tailwind 生成序 + 桌面目标假设；018
        // app.at:24 侧栏/md:hidden 移动头两形态皆此）。
        StyleClass::Hidden => s.hidden = true,
        StyleClass::Flex
        | StyleClass::FlexRow
        | StyleClass::FlexCol
        | StyleClass::FlexColReverse
        | StyleClass::Block
        | StyleClass::Inline
        | StyleClass::InlineBlock
        | StyleClass::InlineFlex => s.hidden = false,
        // PLAN-032 T-02（D5）：self-center = 交叉轴自对齐——块流单列 =
        // 父宽居中（layout_view_block 子级 center_children 通道既有，
        // 真渲非降级；012 步进值/单位标签）。
        StyleClass::SelfCenter => s.center_children = true,
        // PLAN-032 T-04（D4）：样式版 grid（grid-cols/grid-rows）——记档
        // 入 NodeStyle，layout_view_block 入口分岔复用 Grid walker。
        // GridCols(n) 优先；GridRows(m) 主驱时列数 = ceil(cells/m)（布局
        // 期定）；裸 Grid 不记档——无模板的单列网格与纵向堆叠视觉等价
        ///（token "style-grid" 判定面不受影响）。
        StyleClass::GridCols(n) => s.grid_cols = Some(usize::from(*n)),
        StyleClass::GridRows(n) => s.grid_rows = Some(usize::from(*n)),
        // PLAN-032 T-05（D1 分层）：absolute 真渲标记 + offset/z 记档
        ///（layout_view_block 延迟放置消费）；fixed/sticky 降级放行
        ///（in-flow no-op——opacity/overflow 先例，真渲债 §10-④）。
        StyleClass::Absolute => s.absolute = true,
        StyleClass::Fixed | StyleClass::Sticky => s.position_degraded = true,
        StyleClass::TopOffset(px) => s.offset_top = Some(*px),
        StyleClass::LeftOffset(px) => s.offset_left = Some(*px),
        // PLAN-036 T-02：truncate 真渲标记（Text 发射臂 measure 收缩 +
        // `…` 尾接——018 真渲债清偿；覆盖 prefixes ⑫ 同批）。
        StyleClass::Truncate => s.truncate = true,
        StyleClass::RightOffset(px) => s.offset_right = Some(*px),
        StyleClass::BottomOffset(px) => s.offset_bottom = Some(*px),
        StyleClass::ZIndex(z) => s.z_index = Some(*z),
        // PLAN-032 T-07（D5 族）：inset-N 四向偏移归一（未设槽填充——
        // 显式 offset 类优先；与 absolute 组合 = 全覆盖锚定）。
        StyleClass::Inset(px) => {
            if s.offset_top.is_none() {
                s.offset_top = Some(*px);
            }
            if s.offset_left.is_none() {
                s.offset_left = Some(*px);
            }
            if s.offset_right.is_none() {
                s.offset_right = Some(*px);
            }
            if s.offset_bottom.is_none() {
                s.offset_bottom = Some(*px);
            }
        }
        // line-clamp：DrawOp 无行数裁剪通道——判定放行渲染 no-op
        ///（underline 先例；真渲债 KNOWN-DEBT 随注）。
        StyleClass::LineClamp(_) | StyleClass::LineClampNone => {}
        _ => {}
    }
}

/// typed Color → Rgba8（语义 token 走 theme 双盘解析，余者直转——
/// 解释态 `resolve_color` 的 typed 直达形）。
fn resolve_typed_color(c: &Color) -> Option<Rgba8> {
    if let Some((r, g, b)) = crate::ui::style::theme::resolve_semantic_rgb(c) {
        return Some(Rgba8::new(r, g, b, 255));
    }
    let (r, g, b) = c.to_rgb8();
    Some(Rgba8::new(r, g, b, 255))
}

// ---------------------------------------------------------------------------
// ---------------------------------------------------------------------------
// PLAN-034 T-05：CanvasScene → RGBA（canvas=位图快照过线的栅格化孪生）
// ---------------------------------------------------------------------------

/// CSS 色串 → tiny-skia Color（`#rrggbb` 六位；兜底黑——canvas_css_color
/// 同规约的软栅格版）。
fn css_color_tiny(s: &str) -> tiny_skia::Color {
    let hex = s.strip_prefix('#').unwrap_or(s);
    if hex.len() == 6 {
        if let Ok(v) = u32::from_str_radix(hex, 16) {
            return tiny_skia::Color::from_rgba8(
                ((v >> 16) & 0xff) as u8,
                ((v >> 8) & 0xff) as u8,
                (v & 0xff) as u8,
                255,
            );
        }
    }
    tiny_skia::Color::BLACK
}

fn solid_paint(color: tiny_skia::Color) -> tiny_skia::Paint<'static> {
    tiny_skia::Paint {
        shader: tiny_skia::Shader::SolidColor(color),
        anti_alias: true,
        ..Default::default()
    }
}

/// CanvasScene → RGBA（tiny_skia 栅格化——inproc CanvasPainter 的进程外
/// 孪生：clear 铺底 + strokes（折线/单点圆，round 帽角，eraser = clear
/// 色）+ edges（线段）+ nodes（circle/rect）；labels not-yet（软栅格无
/// 文本面）。返回 None = Pixmap 分配失败（调用方跳过上传，占位兜底）。
fn rasterize_canvas_scene(
    scene: &crate::ui::view::CanvasScene,
    clear: Option<&str>,
    extent: Option<(f32, f32)>,
    w: u32,
    h: u32,
) -> Option<Vec<u8>> {
    let mut pm = tiny_skia::Pixmap::new(w.max(1), h.max(1))?;
    let (pw, ph) = (w.max(1) as f32, h.max(1) as f32);
    pm.fill(css_color_tiny(clear.unwrap_or("#ffffff")));
    let (sx, sy) = match extent {
        Some((ew, eh)) if ew > 0.0 && eh > 0.0 => (pw / ew, ph / eh),
        _ => (1.0, 1.0),
    };
    let eraser_color = clear.unwrap_or("#ffffff");
    let identity = tiny_skia::Transform::identity();
    for stroke in &scene.strokes {
        if stroke.points.is_empty() {
            continue;
        }
        let color = css_color_tiny(if stroke.eraser { eraser_color } else { &stroke.color });
        if stroke.points.len() == 1 {
            let (px, py) = stroke.points[0];
            let path = tiny_skia::PathBuilder::from_circle(
                px * sx,
                py * sy,
                (stroke.width / 2.0).max(0.5),
            )?;
            pm.fill_path(&path, &solid_paint(color), tiny_skia::FillRule::Winding, identity, None);
            continue;
        }
        let mut pb = tiny_skia::PathBuilder::new();
        pb.move_to(stroke.points[0].0 * sx, stroke.points[0].1 * sy);
        for (px, py) in &stroke.points[1..] {
            pb.line_to(px * sx, py * sy);
        }
        if let Some(path) = pb.finish() {
            pm.stroke_path(
                &path,
                &solid_paint(color),
                &tiny_skia::Stroke {
                    width: stroke.width.max(0.5),
                    line_cap: tiny_skia::LineCap::Round,
                    line_join: tiny_skia::LineJoin::Round,
                    ..Default::default()
                },
                identity,
                None,
            );
        }
    }
    for edge in &scene.edges {
        let mut pb = tiny_skia::PathBuilder::new();
        pb.move_to(edge.x1 * sx, edge.y1 * sy);
        pb.line_to(edge.x2 * sx, edge.y2 * sy);
        if let Some(path) = pb.finish() {
            pm.stroke_path(
                &path,
                &solid_paint(css_color_tiny(&edge.color)),
                &tiny_skia::Stroke {
                    width: edge.width.max(0.5),
                    line_cap: tiny_skia::LineCap::Round,
                    ..Default::default()
                },
                identity,
                None,
            );
        }
    }
    for node in &scene.nodes {
        let (cx, cy) = (node.x * sx, node.y * sy);
        let color = css_color_tiny(&node.color);
        let path = if node.shape == "rect" {
            let (rw, rh) = (node.w.unwrap_or(40.0) * sx, node.h.unwrap_or(40.0) * sy);
            let rect = tiny_skia::Rect::from_xywh(cx - rw / 2.0, cy - rh / 2.0, rw, rh)?;
            tiny_skia::PathBuilder::from_rect(rect)
        } else {
            tiny_skia::PathBuilder::from_circle(cx, cy, node.r.unwrap_or(16.0) * sx)?
        };
        pm.fill_path(&path, &solid_paint(color), tiny_skia::FillRule::Winding, identity, None);
    }
    // labels：软栅格无文本面——not-yet（043 样板 strokes 主路径无撞；
    // 049 label 视觉缺席为已知边界，文本栅格归后续字体臂）。
    let _ = &scene.labels;
    Some(pm.take())
}

// 测试：native 投影 golden + 命中派发 + 覆盖门 + tick + 管道全循环
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::desktop_protocol::client_runtime::ClientConfig;
    use crate::ui::desktop_protocol::message::DrawOp;
    use crate::ui::desktop_protocol::transport;

    /// 计数器 native 组件（a2r 生成物最小同构——T-03 像素臂示例同源）。
    #[derive(Debug)]
    struct Counter {
        count: i64,
    }

    #[derive(Debug, Clone, PartialEq)]
    enum CounterMsg {
        Inc,
        Tick,
    }

    impl Component for Counter {
        type Msg = CounterMsg;

        fn on(&mut self, msg: Self::Msg) {
            match msg {
                CounterMsg::Inc => self.count += 1,
                CounterMsg::Tick => self.count += 10,
            }
        }

        fn view(&self) -> View<Self::Msg> {
            View::col()
                .spacing(8)
                .child(
                    View::text_styled(format!("count: {}", self.count), "text-lg text-slate-200"),
                )
                .child(
                    View::button("+").on_click(|_| CounterMsg::Inc).build(),
                )
                .build()
        }
    }

    fn counter() -> RqProjector<Counter> {
        proj_center_off(Counter { count: 0 }, 480.0, 320.0)
    }

    /// 几何钉测试装配器：关根缺省居中（PLAN-678 T-02）。金样/几何测试
    /// 钉的是组件形状坐标非居中语义——居中行为面由 autocenter_* 四测试
    /// 承载（它们用缺省 `RqProjector::new`，通道常开）。
    fn proj_center_off<C: Component>(component: C, width: f32, height: f32) -> RqProjector<C> {
        let mut p = RqProjector::new(component, width, height);
        p.autocenter = false;
        p
    }

    /// 列臂 items-center 探针（无事件面，() 消息即足）。
    #[derive(Debug)]
    struct CenterProbe;

    impl Component for CenterProbe {
        type Msg = ();

        fn on(&mut self, _msg: ()) {}

        fn view(&self) -> View<Self::Msg> {
            View::col()
                .style("items-center")
                .child(View::text_styled("abcd", "text-base"))
                .build()
        }
    }

    /// 交叉轴居中（列臂 items-center）：parent.center_children 在
    /// Vertical 块的消费通道——自然宽文本以 (w-自然宽)/2 起点放置。
    /// 修复前该标志仅 Horizontal 臂消费（:Horizontal 两遍法），列内
    /// 恒左贴 x=MARGIN（001-helloworld 实机分歧面）。
    #[test]
    fn vertical_items_center_centers_natural_width_children() {
        let mut p = RqProjector::new(CenterProbe, 480.0, 320.0);
        let frame = p.render_frame();
        let xs: Vec<f32> = frame
            .ops
            .iter()
            .filter_map(|op| match op {
                DrawOp::Text { x, .. } | DrawOp::TextStyled { x, .. } => Some(*x),
                _ => None,
            })
            .collect();
        assert_eq!(xs.len(), 1, "单文本 op 形态: {xs:?}");
        let w = 480.0 - MARGIN * 2.0;
        let natural = measure_text("abcd", TEXT_SIZE);
        let want = MARGIN + (w - natural).max(0.0) / 2.0;
        assert!(
            (xs[0] - want).abs() < 1.0,
            "items-center 列的文本应居中于 {want}，实得 {}（修复前恒左贴 {MARGIN}）",
            xs[0]
        );
    }

    /// PLAN-678 T-02 探针：无任何居中声明的小内容（左贴起点）。
    #[derive(Debug)]
    struct PlainProbe;

    impl Component for PlainProbe {
        type Msg = ();

        fn on(&mut self, _msg: ()) {}

        fn view(&self) -> View<Self::Msg> {
            View::col().child(View::text_styled("abcd", "text-base")).build()
        }
    }

    /// 左上角贴齐的小内容 → ops 实测 bbox 对中视口（单文本：x/y 双轴
    /// 各走 (avail - bbox)/2 + MARGIN）。
    #[test]
    fn autocenter_centers_small_content() {
        let mut p = RqProjector::new(PlainProbe, 480.0, 320.0);
        let frame = p.render_frame();
        let ops: Vec<(f32, f32, f32, f32)> = frame
            .ops
            .iter()
            .filter_map(|op| match op {
                DrawOp::Text { x, y, size, line_height, text, .. }
                | DrawOp::TextStyled { x, y, size, line_height, text, .. } => {
                    Some((*x, *y, *line_height, measure_text(text, *size)))
                }
                _ => None,
            })
            .collect();
        assert_eq!(ops.len(), 1, "单文本 op 形态: {ops:?}");
        let (x, y, line_h, natural) = ops[0];
        let avail_w = 480.0 - MARGIN * 2.0;
        let avail_h = 320.0 - MARGIN * 2.0;
        let want_x = MARGIN + (avail_w - natural) / 2.0;
        let want_y = MARGIN + (avail_h - line_h) / 2.0;
        assert!((x - want_x).abs() < 1.0, "文本 x 应居中于 {want_x}，实得 {x}");
        assert!((y - want_y).abs() < 1.0, "文本 y 应居中于 {want_y}，实得 {y}");
    }

    /// 根显式视口类（h-screen/w-screen 族）= 满幅意图 → 双轴豁免，
    /// 坐标维持左贴（与 iced 轨显式类零改动同律）。
    #[test]
    fn autocenter_exempts_explicit_viewport_classes() {
        #[derive(Debug)]
        struct ScreenProbe;
        impl Component for ScreenProbe {
            type Msg = ();
            fn on(&mut self, _msg: ()) {}
            fn view(&self) -> View<Self::Msg> {
                View::col()
                    .style("w-screen h-screen")
                    .child(View::text_styled("abcd", "text-base"))
                    .build()
            }
        }
        let mut p = RqProjector::new(ScreenProbe, 480.0, 320.0);
        let frame = p.render_frame();
        let xs: Vec<f32> = frame
            .ops
            .iter()
            .filter_map(|op| match op {
                DrawOp::Text { x, .. } | DrawOp::TextStyled { x, .. } => Some(*x),
                _ => None,
            })
            .collect();
        assert_eq!(xs.len(), 1);
        assert!((xs[0] - MARGIN).abs() < 1.0, "视口类根应豁免平移（x={}), 期望 {MARGIN}", xs[0]);
    }

    /// AUTO_NO_AUTOCENTER=1 → 通道旁路（构造期读取——nextest 逐测试
    /// 进程隔离，env 置位零串染）。
    #[test]
    fn autocenter_env_gate_bypasses_translation() {
        std::env::set_var("AUTO_NO_AUTOCENTER", "1");
        let mut p = RqProjector::new(PlainProbe, 480.0, 320.0);
        let frame = p.render_frame();
        let xs: Vec<f32> = frame
            .ops
            .iter()
            .filter_map(|op| match op {
                DrawOp::Text { x, .. } | DrawOp::TextStyled { x, .. } => Some(*x),
                _ => None,
            })
            .collect();
        assert_eq!(xs.len(), 1);
        assert!((xs[0] - MARGIN).abs() < 1.0, "env 门应旁路平移（x={}), 期望 {MARGIN}", xs[0]);
    }

    /// 命中表与 ops 同 delta（按钮探针：hit rect 中心 = 视口中心 ±1）。
    #[test]
    fn autocenter_translates_hits_with_ops() {
        #[derive(Debug)]
        struct ButtonProbe;
        impl Component for ButtonProbe {
            type Msg = ();
            fn on(&mut self, _msg: ()) {}
            fn view(&self) -> View<Self::Msg> {
                View::col()
                    .child(View::button("OK").on_click(|_| ()).build())
                    .build()
            }
        }
        let mut p = RqProjector::new(ButtonProbe, 480.0, 320.0);
        let frame = p.render_frame();
        let quads: Vec<&WRect> = frame
            .ops
            .iter()
            .filter_map(|op| match op {
                DrawOp::Quad { rect, .. } => Some(rect),
                _ => None,
            })
            .collect();
        assert!(!quads.is_empty(), "按钮 quad 形态");
        assert!(!p.hits.is_empty(), "按钮命中登记（on_click 在场）");
        // ops 与 hits 的联合包围盒中心 = 视口中心（双轴）。
        let (mut min_x, mut min_y, mut max_x, mut max_y) =
            (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
        for r in quads.iter().copied() {
            min_x = min_x.min(r.x);
            min_y = min_y.min(r.y);
            max_x = max_x.max(r.x + r.w);
            max_y = max_y.max(r.y + r.h);
        }
        for hit in p.hits.iter_mut() {
            let r = hit.rect_mut();
            min_x = min_x.min(r.x);
            min_y = min_y.min(r.y);
            max_x = max_x.max(r.x + r.w);
            max_y = max_y.max(r.y + r.h);
        }
        assert!(
            ((min_x + max_x) / 2.0 - 240.0).abs() < 1.5
                && ((min_y + max_y) / 2.0 - 160.0).abs() < 1.5,
            "内容 bbox 中心应=视口中心 (240,160)，实得 ({:.1},{:.1})",
            (min_x + max_x) / 2.0,
            (min_y + max_y) / 2.0
        );
    }

    fn texts_of(frame: &DrawList) -> Vec<&str> {
        frame
            .ops
            .iter()
            .filter_map(|op| match op {
                DrawOp::Text { text, .. } | DrawOp::TextStyled { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn golden_counter_frame_shape() {
        let mut p = counter();
        let frame = p.render_frame();
        assert_eq!(frame.clear, Some(bg()));
        // col(8) { text, button } → [Text(count: 0), Quad(按钮底), Text(+)]。
        assert_eq!(texts_of(&frame), vec!["count: 0", "+"]);
        assert!(
            matches!(frame.ops[1], DrawOp::Quad { .. }),
            "按钮底盒 op: {:?}",
            frame.ops[1]
        );
        // 文本色来自 text-slate-200（typed 链路生效——非缺省 TEXT_FG）。
        let DrawOp::Text { color, size, .. } = frame.ops[0] else {
            panic!("首 op 应为 Text");
        };
        assert_eq!(color, Rgba8::new(226, 232, 240, 255), "text-slate-200 语义色");
        assert_eq!(size, 18.0, "text-lg 字号档");
    }

    #[test]
    fn hit_dispatch_increments_and_reframes() {
        let mut p = counter();
        let frame = p.render_frame();
        // 命中区 = 按钮底盒（Quad op 的 rect）。
        let DrawOp::Quad { rect, .. } = frame.ops[1] else {
            panic!();
        };
        assert_eq!(p.revision(), 1);
        let mid = (rect.x + rect.w / 2.0, rect.y + rect.h / 2.0);
        p.on_input(&InputMsg::PointerPressed {
            wid: 1,
            button: MouseButton::Left,
            x: mid.0,
            y: mid.1,
            modifiers: 0,
        });
        assert_eq!(p.revision(), 2, "命中派发推版本");
        let frame = p.render_frame();
        assert_eq!(texts_of(&frame), vec!["count: 1", "+"], "状态变化入帧");
        // 盒外点击不派发。
        p.on_input(&InputMsg::PointerPressed {
            wid: 1,
            button: MouseButton::Left,
            x: 400.0,
            y: 300.0,
            modifiers: 0,
        });
        assert_eq!(p.revision(), 2);
    }

    #[test]
    fn disabled_button_no_hit_region() {
        #[derive(Debug)]
        struct D;

        #[derive(Debug, Clone)]
        enum DMsg {
            Go,
        }

        impl Component for D {
            type Msg = DMsg;
            fn on(&mut self, _msg: Self::Msg) {
                unreachable!("禁用按钮不派发");
            }
            fn view(&self) -> View<Self::Msg> {
                // 禁用态直接构造变体（builder 无 disabled 槽——a2r 生成器
                // 同样直填字段）。
                View::Button {
                    label: "x".into(),
                    onclick: DMsg::Go,
                    style: None,
                    on_right_click: None,
                    content: None,
                    disabled: true,
                }
            }
        }

        let mut p = proj_center_off(D, 480.0, 320.0);
        let frame = p.render_frame();
        // 禁用观感：底盒仍在但压暗（alpha ≤ DISABLED_ALPHA——命令差分同款）。
        let DrawOp::Quad { color, .. } = frame.ops[0] else {
            panic!("首 op 应为压暗底盒");
        };
        assert!(color.a <= DISABLED_ALPHA, "禁用压暗: {color:?}");
        let mid = (70.0, 28.0);
        let before = p.revision();
        p.on_input(&InputMsg::PointerPressed {
            wid: 1,
            button: MouseButton::Left,
            x: mid.0,
            y: mid.1,
            modifiers: 0,
        });
        assert_eq!(p.revision(), before, "禁用态无命中区");
    }

    #[test]
    fn coverage_gate_refuses_uncovered_family() {
        // counter 级视图：Covered。
        let ok = counter();
        assert!(ok.ensure_covered().is_ok());

        // PLAN-025 T-03 语义反转：slider 已入 native 覆盖集 → Covered
        // （020 原样本由拒转收；防漏面 = 覆盖单测 + native 臂测试）。
        #[derive(Debug)]
        struct WithSlider;

        #[derive(Debug, Clone)]
        enum WMsg {
            Nop,
        }

        impl Component for WithSlider {
            type Msg = WMsg;
            fn on(&mut self, _msg: Self::Msg) {}
            fn view(&self) -> View<Self::Msg> {
                View::slider(0.0..=1.0, 0.5).on_change(|_| WMsg::Nop).build()
            }
        }

        let p = proj_center_off(WithSlider, 480.0, 320.0);
        p.ensure_covered().expect("slider 入覆盖集（020 拒面反转）");

        // PLAN-026 T-04 语义反转：grid 已入 native 覆盖集 → Covered
        // （025 拒面样本转正；防漏面 = grid_layout_and_hit_golden）。
        #[derive(Debug)]
        struct WithGrid;

        impl Component for WithGrid {
            type Msg = WMsg;
            fn on(&mut self, _msg: Self::Msg) {}
            fn view(&self) -> View<Self::Msg> {
                View::Grid {
                    cols: 2,
                    gap: 4,
                    cells: vec![View::text("a"), View::text("b")],
                    style: None,
                }
            }
        }

        let p = proj_center_off(WithGrid, 480.0, 320.0);
        p.ensure_covered().expect("grid 入覆盖集（025 拒面反转）");

        // 新拒样本：imagesurface（PLAN-026 §5.1 D5 定案——渲染占位顺带
        // 但 kind 整体 not-yet：交互回调无采集面，登记即静默放行，I3）
        // → 拒绝 + 缺项。
        #[derive(Debug)]
        struct WithImageSurface;

        impl Component for WithImageSurface {
            type Msg = WMsg;
            fn on(&mut self, _msg: Self::Msg) {}
            fn view(&self) -> View<Self::Msg> {
                View::image_surface("x.png")
            }
        }

        let p = proj_center_off(WithImageSurface, 480.0, 320.0);
        let err = p.ensure_covered().unwrap_err();
        assert!(err.contains("imagesurface"), "缺项清单随行: {err}");
    }

    #[test]
    fn coverage_gate_lists_unsupported_style() {
        #[derive(Debug)]
        struct Shadowed;

        #[derive(Debug, Clone)]
        enum SMsg {}

        impl Component for Shadowed {
            type Msg = SMsg;
            fn on(&mut self, _msg: Self::Msg) {}
            fn view(&self) -> View<Self::Msg> {
                // shadow/underline 均降级放行（025 T-06 / 026 T-06——
                // 解释态同款保真边界）；opacity 亦放行（029 T-08 前缀⑦
                // ——alpha 合成无通道的显式降级）。PLAN-032 T-02 起
                // hidden 转真渲放行（D3 display:none）——防漏钉样本换
                // truncate（§1.8 在册稳定缺项）；PLAN-036 T-02（⑫）
                // truncate 亦真渲放行（Text 发射臂 `…`）——样本再换
                // rotate-1（视觉变换 not-yet，定位族在册）。
                View::text_styled("x", "rotate-1")
            }
        }

        let p = proj_center_off(Shadowed, 480.0, 320.0);
        let err = p.ensure_covered().unwrap_err();
        assert!(err.contains("style:rotate-1"), "native 无 rotate 渲染: {err}");
    }

    /// PLAN-032 T-02（D3）：hidden = display:none——子树整体不渲染不占位
    ///（零 ops 零行高痕迹，兄弟节点如同其不存在）。
    #[test]
    fn hidden_display_none_golden() {
        #[derive(Debug)]
        struct HiddenBox;

        #[derive(Debug, Clone)]
        enum HMsg {}

        impl Component for HiddenBox {
            type Msg = HMsg;
            fn on(&mut self, _msg: Self::Msg) {}
            fn view(&self) -> View<Self::Msg> {
                View::col()
                    .child(View::text_styled("A", "text-sm"))
                    .child(View::text_styled("B", "text-sm hidden"))
                    .child(View::text_styled("C", "text-sm"))
                    .build()
            }
        }

        let mut p = proj_center_off(HiddenBox, 480.0, 320.0);
        p.ensure_covered().expect("hidden 放行（prefixes ⑧）");
        let frame = p.render_frame();
        // B 整体缺席；且布局与"B 从未存在"逐坐标等价（对照组件同帧金样
        //——gap 序/行高零痕迹；display:none 非留白）。
        let lines: Vec<(f32, &str)> = frame
            .ops
            .iter()
            .filter_map(|op| match op {
                DrawOp::Text { y, text, .. } => Some((*y, text.as_str())),
                _ => None,
            })
            .collect();
        assert_eq!(lines.iter().map(|(_, t)| *t).collect::<Vec<_>>(), vec!["A", "C"]);
        let yc = lines[1].0;

        #[derive(Debug)]
        struct ControlBox;
        impl Component for ControlBox {
            type Msg = HMsg;
            fn on(&mut self, _msg: Self::Msg) {}
            fn view(&self) -> View<Self::Msg> {
                View::col()
                    .child(View::text_styled("A", "text-sm"))
                    .child(View::text_styled("C", "text-sm"))
                    .build()
            }
        }
        let mut q = proj_center_off(ControlBox, 480.0, 320.0);
        let control = q.render_frame();
        let yc_control = control
            .ops
            .iter()
            .filter_map(|op| match op {
                DrawOp::Text { y, text, .. } if text == "C" => Some(*y),
                _ => None,
            })
            .next()
            .expect("对照帧含 C");
        assert!(
            (yc - yc_control).abs() < 0.01,
            "C 落位与 B 不存在的对照等价: hidden={yc} control={yc_control}"
        );
    }

    /// PLAN-032 T-02（D3）：响应式覆盖——"hidden md:flex" 经 parser 剥
    /// 响应式前缀成 [Hidden, Flex]，display 族在场清位 → 桌面档可见
    ///（018 侧栏形态；对照裸 md:hidden = 隐藏）。
    #[test]
    fn hidden_responsive_override_visible() {
        #[derive(Debug)]
        struct OverrideBox;

        #[derive(Debug, Clone)]
        enum OMsg {}

        impl Component for OverrideBox {
            type Msg = OMsg;
            fn on(&mut self, _msg: Self::Msg) {}
            fn view(&self) -> View<Self::Msg> {
                View::col()
                    .child(
                        View::row()
                            .style("hidden md:flex items-center gap-2")
                            .child(View::text_styled("S", "text-sm"))
                            .build(),
                    )
                    .child(View::text_styled("M", "text-sm md:hidden"))
                    .build()
            }
        }

        let mut p = proj_center_off(OverrideBox, 480.0, 320.0);
        p.ensure_covered().expect("hidden/响应式覆盖两形态均放行");
        let frame = p.render_frame();
        let texts = texts_of(&frame);
        assert!(texts.contains(&"S"), "hidden md:flex = 桌面可见: {texts:?}");
        assert!(!texts.contains(&"M"), "md:hidden = 桌面隐藏: {texts:?}");
    }

    /// PLAN-032 T-05（D1 分层）：absolute 真渲——脱离流不占位（不贡献
    /// 父高）、offsets 锚定父内容盒（top/left 直加；bottom/right 按父
    /// 盒尺寸反算——018 bookshelf 封面徽条 "absolute bottom-0 left-0
    /// right-0" 形态）、覆盖序追加主序之后、同层 z 稳定排序。
    #[test]
    fn absolute_overlay_golden() {
        #[derive(Debug)]
        struct AbsBox;

        #[derive(Debug, Clone)]
        enum AMsg {
            Noop,
        }

        impl Component for AbsBox {
            type Msg = AMsg;
            fn on(&mut self, _msg: Self::Msg) {}
            fn view(&self) -> View<Self::Msg> {
                // 定高流内件（button h-32=128px——叶子臂消费 fixed_h；堆
                // 叠族自身 fixed 高为既有丢弃边界，锚定按内容盒自洽）+
                // absolute 徽条（bottom 反算）+ absolute 顶标（top/left
                // 直加，z-20 后绘置顶）。
                View::col()
                    .style("relative w-full")
                    .child(View::button("cover").style("h-32 w-full").on_click(|_| AMsg::Noop).build())
                    .child(View::text_styled("badge", "absolute bottom-0 left-0 right-0 text-xs"))
                    .child(View::text_styled("tag", "absolute top-1 left-2 z-20 text-xs"))
                    .child(View::text_styled("under", "absolute top-1 left-2 z-10 text-xs"))
                    .build()
            }
        }

        let mut p = proj_center_off(AbsBox, 480.0, 320.0);
        p.ensure_covered().expect("定位族放行（prefixes ⑪）");
        let frame = p.render_frame();
        let ops: Vec<(f32, f32, &str)> = frame
            .ops
            .iter()
            .filter_map(|op| match op {
                DrawOp::Text { x, y, text, .. } => Some((*x, *y, text.as_str())),
                _ => None,
            })
            .collect();
        let find = |t: &str| {
            let e = ops.iter().find(|(_, _, n)| *n == t).expect(t);
            (e.0, e.1)
        };
        // 流内 button h-32=128 撑父内容盒：badge bottom-0 → y = 10 + 128
        // - 行高(12×1.35=16.2) = 121.8；right-0 反算被 left-0 过约束覆盖
        ///（left 胜——CSS ltr 同语义），x = 父左 10。
        let (bx, by) = find("badge");
        assert!((by - (10.0 + 128.0 - 16.2)).abs() < 0.01, "badge bottom 反算: {by}");
        assert!((bx - 10.0).abs() < 0.01, "badge left-0（right 过约束让位）: {bx}");
        // tag/under：top-1(4px)/left-2(8px) 直加；z-10 先绘、z-20 后绘
        ///（ops 序 = 覆盖序，稳定排序文档序同 z）。
        let (tx, ty) = find("tag");
        let (ux, uy) = find("under");
        assert!((tx - 18.0).abs() < 0.01 && (ty - 14.0).abs() < 0.01, "tag top/left 直加: {tx},{ty}");
        assert!((ux - 18.0).abs() < 0.01 && (uy - 14.0).abs() < 0.01, "under 同锚");
        assert!(
            ops.iter().position(|(_, _, n)| *n == "under") < ops.iter().position(|(_, _, n)| *n == "tag"),
            "z-20 tag 后绘置顶（z-10 under 先绘）"
        );
        // 覆盖序：absolutes 追加主序之后（cover 按钮文本在前）。
        assert!(
            ops.iter().position(|(_, _, n)| *n == "cover")
                < ops.iter().position(|(_, _, n)| *n == "badge"),
            "absolute 追加主序之后"
        );
        // absolutes 零流内占位：badge 之前仅 cover 一项文本（tag/under 均
        /// 延后），父内容高 = 纯流内件（button 128）。
        assert_eq!(ops.iter().take_while(|(_, _, n)| *n != "badge").count(), 1, "absolutes 零流内占位");
    }

    /// PLAN-032 T-05（D1 分层）：fixed/sticky 降级放行——in-flow no-op
    /// 渲染（坐标与无类对照逐项等价；真渲债 §10-④ 另立——保真边界
    /// 显式钉：降级不是错绘成隐藏，而是原位渲染）。
    #[test]
    fn fixed_sticky_degraded_inflow_golden() {
        #[derive(Debug)]
        struct DegradedBox;

        #[derive(Debug, Clone)]
        enum DMsg {}

        impl Component for DegradedBox {
            type Msg = DMsg;
            fn on(&mut self, _msg: Self::Msg) {}
            fn view(&self) -> View<Self::Msg> {
                View::col()
                    .child(View::text_styled("nav", "sticky top-0 z-30 text-sm"))
                    .child(View::text_styled("side", "fixed inset-y-0 z-40 text-sm"))
                    .child(View::text_styled("body", "text-sm"))
                    .build()
            }
        }

        #[derive(Debug)]
        struct ControlBox;

        impl Component for ControlBox {
            type Msg = DMsg;
            fn on(&mut self, _msg: Self::Msg) {}
            fn view(&self) -> View<Self::Msg> {
                View::col()
                    .child(View::text_styled("nav", "text-sm"))
                    .child(View::text_styled("side", "text-sm"))
                    .child(View::text_styled("body", "text-sm"))
                    .build()
            }
        }

        let mut p = proj_center_off(DegradedBox, 480.0, 320.0);
        p.ensure_covered().expect("fixed/sticky 降级放行（prefixes ⑪）");
        let frame = p.render_frame();
        let mut q = proj_center_off(ControlBox, 480.0, 320.0);
        let control = q.render_frame();
        fn coords(f: &DrawList) -> Vec<(f32, f32, &str)> {
            f.ops
                .iter()
                .filter_map(|op| match op {
                    DrawOp::Text { x, y, text, .. } => Some((*x, *y, text.as_str())),
                    _ => None,
                })
                .collect()
        }
        assert_eq!(coords(&frame), coords(&control), "fixed/sticky = in-flow 原位渲染（降级随注非错绘）");
    }

    /// PLAN-032 T-04（D4）：样式版 grid 分岔——`col (style: "grid
    /// grid-cols-2 gap-2")` 与 View::Grid 变体臂同构（等宽列 row-major
    /// × 行高=行内最大）；GridRows(m) → 列数 = ceil(cells/m)。
    #[test]
    fn style_grid_golden_isomorphic_to_variant() {
        #[derive(Debug)]
        struct StyleGrid;

        #[derive(Debug, Clone)]
        enum GMsg {}

        impl Component for StyleGrid {
            type Msg = GMsg;
            fn on(&mut self, _msg: Self::Msg) {}
            fn view(&self) -> View<Self::Msg> {
                View::col()
                    .style("grid grid-cols-2 gap-2")
                    .child(View::text_styled("a", "text-sm"))
                    .child(View::text_styled("b", "text-sm"))
                    .child(View::text_styled("c", "text-sm"))
                    .child(View::text_styled("d", "text-sm"))
                    .build()
            }
        }

        #[derive(Debug)]
        struct VariantGrid;

        impl Component for VariantGrid {
            type Msg = GMsg;
            fn on(&mut self, _msg: Self::Msg) {}
            fn view(&self) -> View<Self::Msg> {
                // gap-2 = Tailwind 刻度（SizeValue::Fixed(2) → 8px）——
                // 变体 spacing 字段为裸 px，同构对照取 8。
                View::grid()
                    .cols(2)
                    .spacing(8)
                    .child(View::text_styled("a", "text-sm"))
                    .child(View::text_styled("b", "text-sm"))
                    .child(View::text_styled("c", "text-sm"))
                    .child(View::text_styled("d", "text-sm"))
                    .build()
            }
        }

        #[derive(Debug)]
        struct RowsGrid;

        impl Component for RowsGrid {
            type Msg = GMsg;
            fn on(&mut self, _msg: Self::Msg) {}
            fn view(&self) -> View<Self::Msg> {
                View::col()
                    .style("grid grid-rows-2 gap-2")
                    .child(View::text_styled("a", "text-sm"))
                    .child(View::text_styled("b", "text-sm"))
                    .child(View::text_styled("c", "text-sm"))
                    .child(View::text_styled("d", "text-sm"))
                    .build()
            }
        }

        let mut p = proj_center_off(StyleGrid, 480.0, 320.0);
        p.ensure_covered().expect("style-grid 放行（prefixes ⑩）");
        let style_frame = p.render_frame();
        let mut q = proj_center_off(VariantGrid, 480.0, 320.0);
        let variant_frame = q.render_frame();
        // 同构断言：文本坐标逐项相等（024 真源 = 左面板 2×2 按钮组）。
        let coords = |f: &DrawList| -> Vec<(f32, f32, String)> {
            f.ops
                .iter()
                .filter_map(|op| match op {
                    DrawOp::Text { x, y, text, .. } => Some((*x, *y, text.clone())),
                    _ => None,
                })
                .collect()
        };
        assert_eq!(coords(&style_frame), coords(&variant_frame), "样式 grid 与变体 Grid 同构");
        // 2 列几何：a/b 同行（y 相等），c/d 同行且 y 前进一行高。
        let cs = coords(&style_frame);
        let (a, b, c) = (cs[0].clone(), cs[1].clone(), cs[2].clone());
        assert_eq!(a.1, b.1, "a/b 同行");
        assert!(c.1 > a.1, "c 次行");
        // GridRows(2) × 4 cells → 2 列（与 cols-2 同布局）。
        let mut r = proj_center_off(RowsGrid, 480.0, 320.0);
        let rows_frame = r.render_frame();
        assert_eq!(coords(&rows_frame), coords(&style_frame), "grid-rows-2 × 4 = 2 列同构");
    }

    /// PLAN-032 T-03（D2）：tabs kind golden——default/enclosed 两变体 ×
    /// 选中态切换 on_select 闭环（点击托盘项 → TabSelect 物化 → 受控
    /// 状态重入 view() → 内容区切换帧）。
    #[test]
    fn tabs_tray_golden_and_select_loopback() {
        #[derive(Debug)]
        struct TabsBox {
            which: usize,
            sel: usize,
            seen: Vec<String>,
        }

        #[derive(Debug, Clone, PartialEq)]
        enum TbMsg {
            Pick(usize, String),
        }

        impl Component for TabsBox {
            type Msg = TbMsg;
            fn on(&mut self, msg: Self::Msg) {
                if let TbMsg::Pick(i, v) = msg {
                    self.sel = i;
                    self.seen.push(v);
                }
            }
            fn view(&self) -> View<Self::Msg> {
                let labels = vec!["Alpha".to_string(), "Beta".to_string()];
                let contents = vec![
                    View::text_styled("panel-a", "text-sm"),
                    View::text_styled("panel-b", "text-sm"),
                ];
                View::Tabs {
                    labels,
                    contents,
                    selected: self.sel,
                    position: TabsPosition::Top,
                    on_select: Some(TabsSelectCallback::new(|idx| {
                        // select 臂双载荷闭包先例（idx + value 串）。
                        let val = if idx == 0 { "a" } else { "b" };
                        TbMsg::Pick(idx, val.to_string())
                    })),
                    style: None,
                    variant: if self.which == 0 { TabsVariant::Default } else { TabsVariant::Enclosed },
                }
            }
        }

        // default：托盘两等宽按钮 + 选中面板；切换闭环。
        let mut p = proj_center_off(TabsBox { which: 0, sel: 0, seen: vec![] }, 480.0, 320.0);
        p.ensure_covered().expect("tabs 入覆盖集");
        let frame = p.render_frame();
        assert_eq!(
            texts_of(&frame),
            vec!["Alpha", "Beta", "panel-a"],
            "default：托盘 + 选中内容（panel-b 不渲染）"
        );
        let w = (480.0 - 20.0) / 2.0;
        assert!(
            quads_of(&frame).iter().any(|r| *r == WRect::new(10.0, 10.0, w, 26.9)),
            "等宽托盘项: {:?}",
            quads_of(&frame)
        );
        click(&mut p, 10.0 + w + 10.0, 18.0); // 第二项（Beta）
        assert_eq!(p.component().seen, vec!["b".to_string()], "on_select value 串载荷");
        let frame = p.render_frame();
        assert_eq!(
            texts_of(&frame),
            vec!["Alpha", "Beta", "panel-b"],
            "受控切换：内容区换 panel-b"
        );

        // enclosed：托盘底 + 选中下划线（2px 底缘条）。
        let mut q = proj_center_off(TabsBox { which: 1, sel: 1, seen: vec![] }, 480.0, 320.0);
        let frame = q.render_frame();
        assert_eq!(texts_of(&frame), vec!["Alpha", "Beta", "panel-b"], "enclosed：内容同律");
        assert!(
            quads_of(&frame)
                .iter()
                .any(|r| *r == WRect::new(10.0 + w, 10.0 + 26.9 - 2.0, w, 2.0)),
            "enclosed 选中下划线: {:?}",
            quads_of(&frame)
        );
    }
    /// PLAN-032 T-02（D5）：self-center 交叉轴自对齐——块流单列 = 父宽
    /// 居中（center_children 通道真渲；012 步进值/单位标签映射清偿）。
    #[test]
    fn self_center_aligns_golden() {
        #[derive(Debug)]
        struct SelfCenterBox;

        #[derive(Debug, Clone)]
        enum SCMsg {}

        impl Component for SelfCenterBox {
            type Msg = SCMsg;
            fn on(&mut self, _msg: Self::Msg) {}
            fn view(&self) -> View<Self::Msg> {
                View::col()
                    .child(View::text_styled("L", "text-sm self-center"))
                    .child(View::text_styled("R", "text-sm"))
                    .build()
            }
        }

        let mut p = proj_center_off(SelfCenterBox, 480.0, 320.0);
        p.ensure_covered().expect("self- 放行（prefixes ⑨）");
        let frame = p.render_frame();
        let xs: Vec<(f32, &str)> = frame
            .ops
            .iter()
            .filter_map(|op| match op {
                DrawOp::Text { x, text, .. } => Some((*x, text.as_str())),
                _ => None,
            })
            .collect();
        let (l, r) = (
            xs.iter().find(|(_, t)| *t == "L").unwrap(),
            xs.iter().find(|(_, t)| *t == "R").unwrap(),
        );
        let lw = measure_text("L", 14.0);
        assert!(
            (l.0 - (480.0 - lw) / 2.0).abs() < 0.01,
            "L 父宽居中: x={} 期望={}（measure={lw}）",
            l.0,
            (480.0 - lw) / 2.0
        );
        assert!(r.0 == 10.0, "R 缺省左对齐: {}", r.0);
    }

    #[test]
    fn dynamic_branch_uncovered_placeholder_tracked() {
        // 门后动态分支：状态切换遭遇未覆盖变体 → 占位盒 + uncovered_seen。
        // （样本 = imagesurface——PLAN-026 D5 整 kind not-yet；原 grid
        // 样本随 T-04 覆盖扩容转正，拒面换 imagesurface 同册。）
        #[derive(Debug)]
        struct Branchy {
            show_grid: bool,
        }

        #[derive(Debug, Clone, PartialEq)]
        enum BMsg {
            Toggle,
        }

        impl Component for Branchy {
            type Msg = BMsg;
            fn on(&mut self, msg: Self::Msg) {
                if msg == BMsg::Toggle {
                    self.show_grid = !self.show_grid;
                }
            }
            fn view(&self) -> View<Self::Msg> {
                // 门时刻 imagesurface 不可见 → Covered；Toggle 后动态出现。
                let mut col = View::col().child(
                    View::button("t").on_click(|_| BMsg::Toggle).build(),
                );
                if self.show_grid {
                    col = col.child(View::image_surface("x.png"));
                }
                col.build()
            }
        }

        let mut p = proj_center_off(Branchy { show_grid: false }, 480.0, 320.0);
        assert!(p.ensure_covered().is_ok(), "门时刻无未覆盖变体");
        let _ = p.render_frame();
        assert!(p.uncovered_seen().is_empty());

        // 命中按钮（col 首子 → 顶部按钮区）切状态 → 下一帧出现占位。
        p.on_input(&InputMsg::PointerPressed {
            wid: 1,
            button: MouseButton::Left,
            x: 70.0,
            y: 28.0,
            modifiers: 0,
        });
        let frame = p.render_frame();
        assert_eq!(p.uncovered_seen(), ["imagesurface"], "动态分支遭遇留痕");
        assert!(
            texts_of(&frame).iter().any(|t| t.starts_with("not-rendered: imagesurface")),
            "占位盒显式标记: {:?}",
            texts_of(&frame)
        );
    }

    #[test]
    fn poll_tick_fires_on_interval() {
        #[derive(Debug)]
        struct Ticky {
            ticks: i64,
        }

        impl Component for Ticky {
            type Msg = CounterMsg;
            fn on(&mut self, msg: Self::Msg) {
                if msg == CounterMsg::Tick {
                    self.ticks += 1;
                }
            }
            fn tick_interval_ms(&self) -> Option<u32> {
                Some(1)
            }
            fn tick_msg(&self) -> Option<Self::Msg> {
                Some(CounterMsg::Tick)
            }
            fn view(&self) -> View<Self::Msg> {
                View::text("t")
            }
        }

        let mut p = proj_center_off(Ticky { ticks: 0 }, 480.0, 320.0);
        let before = p.revision();
        p.poll_tick(); // 首拍只对相位（不派发）。
        assert_eq!(p.revision(), before);
        std::thread::sleep(std::time::Duration::from_millis(8));
        p.poll_tick();
        assert_eq!(p.revision(), before + 1, "interval 到期派发 tick_msg");
    }

    // —— PLAN-025 T-02 form 族单测（golden + 聚焦编辑闭环 + toggle）——

    /// 双 input 换算组件（a2r 生成物最小同构——on() 读 INPUT_TEXT 写绑定
    /// 字段 + 内联换算；003-converter 同构）。
    #[derive(Debug)]
    struct Converter {
        celsius: f64,
        fahrenheit: f64,
    }

    #[derive(Debug, Clone, PartialEq)]
    enum CvMsg {
        SetC,
        SetF,
    }

    impl Component for Converter {
        type Msg = CvMsg;
        fn on(&mut self, msg: Self::Msg) {
            let text = crate::ui::iced::last_input_text();
            match msg {
                CvMsg::SetC => {
                    self.celsius = text.parse::<f64>().unwrap_or(self.celsius);
                    self.fahrenheit = (self.celsius * 9.0 / 5.0 + 32.0) * 100.0 / 100.0;
                }
                CvMsg::SetF => {
                    self.fahrenheit = text.parse::<f64>().unwrap_or(self.fahrenheit);
                    self.celsius = (self.fahrenheit - 32.0) * 5.0 / 9.0 * 100.0 / 100.0;
                }
            }
        }
        fn view(&self) -> View<Self::Msg> {
            View::col()
                .child(
                    View::input("c")
                        .value(format!("{}", self.celsius))
                        .on_change(CvMsg::SetC)
                        .build(),
                )
                .child(
                    View::input("f")
                        .value(format!("{}", self.fahrenheit))
                        .on_change(CvMsg::SetF)
                        .build(),
                )
                .build()
        }
    }

    /// PLAN-026 T-05：IME 闭环投影器侧（D2 定案：Commit 并入 buffer →
    /// INPUT_TEXT 代写 → on_change 派发 → 帧变；Preedit 尾拼显示（差分
    /// 色 op）；Cancelled 消解；无聚焦丢弃 + ime_dropped 留痕）。
    #[test]
    fn ime_commit_preedit_cancelled_loop() {
        let mut p = proj_center_off(Converter { celsius: 0.0, fahrenheit: 32.0 }, 480.0, 320.0);
        p.ensure_covered().expect("converter Covered");
        let _ = p.render_frame(); // 命中表首帧（click 消费上一帧 hits）
        // 聚焦 celsius（首 input 槽位——003 金样同位坐标）。
        click(&mut p, 100.0, 26.0);
        // ① ImePreedit：暂存 → 聚焦框尾拼 op（差分色）。
        p.on_input(&InputMsg::ImePreedit { wid: 1, text: "中文".into(), selection: None });
        let frame = p.render_frame();
        let texts = texts_of(&frame);
        assert!(
            texts.iter().any(|t| t.contains("中文")),
            "preedit 尾拼进帧: {texts:?}"
        );
        // preedit 独立 op（差分色 = PLACEHOLDER_FG 近似下划线）。
        let preedit_op = frame.ops.iter().any(|op| {
            matches!(op, DrawOp::Text { color, text, .. }
                if *color == placeholder_fg() && text.contains("中文"))
        });
        assert!(preedit_op, "preedit 尾拼差分色 op 在场");
        // ② ImeCancelled：组合取消 → preedit 消解（帧面回退）。
        p.on_input(&InputMsg::ImeCancelled { wid: 1 });
        let frame = p.render_frame();
        assert!(
            !texts_of(&frame).iter().any(|t| t.contains("中文")),
            "Cancelled 消解 preedit"
        );
        // ③ ImeCommit：并入 buffer → on_change 派发（值 5 → 帧联动）。
        p.on_input(&InputMsg::ImeCommit { wid: 1, text: "5".into() });
        p.on_input(&InputMsg::ImeCommit { wid: 1, text: "中文".into() });
        assert_eq!(p.ime_dropped(), 0, "聚焦在册不丢弃");
        let frame = p.render_frame();
        let texts = texts_of(&frame);
        assert!(
            texts.iter().any(|t| t.contains("5中文")),
            "Commit 并入 buffer 显示面: {texts:?}"
        );
        assert!(
            texts.iter().any(|t| t.contains("41")),
            "on_change 派发联动帧（celsius=5中文 parse 前缀 5 → f=41）: {texts:?}"
        );
        // ④ 无聚焦 Commit = 丢弃留痕。
        p.on_input(&InputMsg::KeyPressed { wid: 1, key: 27, modifiers: 0 }); // Esc 不失焦——改走结构变化失焦：省略，直接测无聚焦路径
        let mut p2 = proj_center_off(Converter { celsius: 0.0, fahrenheit: 32.0 }, 480.0, 320.0);
        p2.on_input(&InputMsg::ImeCommit { wid: 1, text: "x".into() });
        p2.on_input(&InputMsg::ImePreedit { wid: 1, text: "y".into(), selection: None });
        assert_eq!(p2.ime_dropped(), 2, "无聚焦丢弃留痕");
    }

    fn quads_of(frame: &DrawList) -> Vec<WRect> {
        frame
            .ops
            .iter()
            .filter_map(|op| match op {
                DrawOp::Quad { rect, .. } => Some(*rect),
                _ => None,
            })
            .collect()
    }

    fn click<C: Component>(p: &mut RqProjector<C>, x: f32, y: f32) {
        p.on_input(&InputMsg::PointerPressed {
            wid: 1,
            button: MouseButton::Left,
            x,
            y,
            modifiers: 0,
        });
    }

    #[test]
    fn input_golden_placeholder_value_focus_border() {
        #[derive(Debug)]
        struct OneInput;
        #[derive(Debug, Clone)]
        enum OMsg {
            Nop,
        }
        impl Component for OneInput {
            type Msg = OMsg;
            fn on(&mut self, _m: Self::Msg) {}
            fn view(&self) -> View<Self::Msg> {
                // on_change 在场 = 可聚焦可编辑（登记纪律与解释态同：
                // 无绑定不登记——None 时点击不聚焦）。
                View::input("your name")
                    .value("Zhang".to_string())
                    .on_change(OMsg::Nop)
                    .build()
            }
        }
        let mut p = proj_center_off(OneInput, 480.0, 320.0);
        p.ensure_covered().expect("input 级入覆盖集");
        let frame = p.render_frame();
        // 盒 (10,10,320,32) + 1px 边框 + 值文本（未聚焦 = 视图值）。
        assert_eq!(quads_of(&frame)[0], WRect::new(10.0, 10.0, 320.0, 32.0));
        assert_eq!(texts_of(&frame), vec!["Zhang"], "值文本（placeholder 隐藏）");
        // 空值 → placeholder（PLACEHOLDER_FG 色）。
        #[derive(Debug)]
        struct Empty;
        impl Component for Empty {
            type Msg = OMsg;
            fn on(&mut self, _m: Self::Msg) {}
            fn view(&self) -> View<Self::Msg> {
                View::input("your name").build()
            }
        }
        let frame = proj_center_off(Empty, 480.0, 320.0).render_frame();
        assert_eq!(texts_of(&frame), vec!["your name"]);
        let ph_color = frame.ops.iter().find_map(|op| match op {
            DrawOp::Text { color, text, .. } if text == "your name" => Some(*color),
            _ => None,
        });
        assert_eq!(ph_color, Some(placeholder_fg()), "placeholder 灰");
        // 聚焦 → 描边变蓝（FOCUS_BORDER 顶边 quad）。
        let mut p = proj_center_off(OneInput, 480.0, 320.0);
        let _ = p.render_frame(); // 首帧建命中表（点击寻址前提）。
        click(&mut p, 100.0, 26.0);
        let frame = p.render_frame();
        assert!(
            quads_of(&frame)
                .iter()
                .any(|r| r.x == 10.0 && r.y == 10.0 && r.w == 320.0 && r.h == 1.0),
            "顶边 1px 边框在册"
        );
        // PLAN-679：聚焦描边 = Primary 槽（accent 驱动，原硬编码 blue-500）。
        let foc = primary_fill();
        let has_focus_border = frame.ops.iter().any(|op| match op {
            DrawOp::Quad { color: c, .. } => *c == foc,
            _ => false,
        });
        assert!(has_focus_border, "聚焦描边 blue-500");
    }

    #[test]
    fn focus_edit_closure_converter() {
        let mut p = proj_center_off(
            Converter { celsius: 0.0, fahrenheit: 32.0 },
            480.0,
            320.0,
        );
        p.ensure_covered().expect("form 级入覆盖集");
        let frame = p.render_frame();
        // 双 input 框（槽 0 = celsius y=10，槽 1 = fahrenheit y=50——gap 8）。
        assert!(quads_of(&frame).iter().any(|r| *r == WRect::new(10.0, 10.0, 320.0, 32.0)));
        assert!(quads_of(&frame).iter().any(|r| *r == WRect::new(10.0, 50.0, 320.0, 32.0)));

        // 点击聚焦槽 0 → 键入 "100" → 换算联动（fahrenheit = 212）。
        click(&mut p, 100.0, 26.0);
        assert_eq!(p.focused_input, Some(0), "点击聚焦槽位（D1-A）");
        for ch in "100".chars() {
            p.on_input(&InputMsg::CharTyped { wid: 1, ch });
        }
        let frame = p.render_frame();
        assert!(
            texts_of(&frame).iter().any(|t| t.starts_with("212")),
            "换算联动帧变: {:?}",
            texts_of(&frame)
        );
        // 聚焦框显示 buffer（视图值 "0" + 键入 "100" = "0100"——iced
        // 内部 buffer 同款语义），非聚焦框显示换算值。
        assert_eq!(texts_of(&frame)[0], "0100", "聚焦框显 buffer（D2）");

        // 切聚焦槽 1（buffer 自视图值初始化）→ 退格 "212"→"21" → 联动。
        click(&mut p, 100.0, 66.0);
        assert_eq!(p.focused_input, Some(1));
        assert_eq!(p.input_buffer, "212", "聚焦时 buffer 自视图值初始化");
        p.on_input(&InputMsg::KeyPressed { wid: 1, key: 8, modifiers: 0 });
        let frame = p.render_frame();
        let f = 21.0_f64;
        let c = (f - 32.0) * 5.0 / 9.0;
        assert_eq!(texts_of(&frame)[0], format!("{c}"), "celsius 联动重算");
        assert_eq!(texts_of(&frame)[1], "21", "聚焦框显退格后 buffer");

        // 无聚焦键入不派发（解释态同口径——无路由目标静默）。
        let before = p.revision();
        p.focused_input = None;
        p.on_input(&InputMsg::CharTyped { wid: 1, ch: 'x' });
        assert_eq!(p.revision(), before, "无聚焦不派发");
    }

    // —— PLAN-025 T-03 slider 单测 ——

    #[derive(Debug)]
    struct SliderBox {
        vol: f32,
    }

    #[derive(Debug, Clone, PartialEq)]
    enum SMsg {
        Vol(f32),
    }

    impl Component for SliderBox {
        type Msg = SMsg;
        fn on(&mut self, msg: Self::Msg) {
            if let SMsg::Vol(v) = msg {
                self.vol = v;
            }
        }
        fn view(&self) -> View<Self::Msg> {
            View::col()
                .child(View::slider(0.0..=100.0, self.vol).on_change(SMsg::Vol).build())
                .child(View::text(format!("vol: {}", self.vol)))
                .build()
        }
    }

    #[test]
    fn slider_geometry_golden() {
        let mut p = proj_center_off(SliderBox { vol: 25.0 }, 480.0, 320.0);
        p.ensure_covered().expect("slider 入覆盖集");
        let frame = p.render_frame();
        // track (10, 18, 320, 4) 底；fill (10,18,80,4)（25%）；knob
        // (84,14,12,12)。布局：h=20 → cy=20；vx = 10 + 320×0.25 = 90。
        let qs = quads_of(&frame);
        assert!(qs.iter().any(|r| *r == WRect::new(10.0, 18.0, 320.0, 4.0)), "track: {qs:?}");
        assert!(qs.iter().any(|r| *r == WRect::new(10.0, 18.0, 80.0, 4.0)), "fill 25%");
        assert!(qs.iter().any(|r| *r == WRect::new(84.0, 14.0, 12.0, 12.0)), "knob: {qs:?}");
        // 值文本（第二子）。
        assert!(texts_of(&frame).iter().any(|t| t.starts_with("vol: 25")),);
    }

    #[test]
    fn slider_track_click_dispatch() {
        let mut p = proj_center_off(SliderBox { vol: 0.0 }, 480.0, 320.0);
        let _ = p.render_frame();
        // 轨道 50% 处点击（(170, 20)）→ vol = 50 → 帧文本联动。
        click(&mut p, 170.0, 20.0);
        let frame = p.render_frame();
        assert!(
            texts_of(&frame).iter().any(|t| *t == "vol: 50"),
            "轨道点击 f32 派发: {:?}",
            texts_of(&frame)
        );
        // 近右缘点击（rect 内 x=329）→ t=0.996875 → v=99.6875（f32 精确
        // 表示，文本可比；越界点击 rect 外不命中——命中判定语义不变）。
        click(&mut p, 329.0, 20.0);
        let frame = p.render_frame();
        assert!(
            texts_of(&frame).iter().any(|t| *t == "vol: 99.6875"),
            "右缘线性换算: {:?}",
            texts_of(&frame)
        );
        // step 取整：0..=100 step 30 → 25% 点击（raw 25）→ 30（fill
        // 96px = 320×0.30 独立证词）。
        #[derive(Debug)]
        struct Stepper {
            seen: f32,
        }
        impl Component for Stepper {
            type Msg = SMsg;
            fn on(&mut self, msg: Self::Msg) {
                if let SMsg::Vol(v) = msg {
                    self.seen = v;
                }
            }
            fn view(&self) -> View<Self::Msg> {
                View::slider(0.0..=100.0, self.seen).on_change(SMsg::Vol).step(30.0).build()
            }
        }
        let mut sp = proj_center_off(Stepper { seen: 0.0 }, 480.0, 320.0);
        let _ = sp.render_frame();
        click(&mut sp, 90.0, 20.0); // 25% → raw 25 → step 30
        let frame = sp.render_frame();
        assert!(
            quads_of(&frame)
                .iter()
                .any(|r| *r == WRect::new(10.0, 18.0, 96.0, 4.0)),
            "step 30 取整后 fill 30%: {:?}",
            quads_of(&frame)
        );
    }

    // —— PLAN-025 T-04 select 单测 ——

    #[derive(Debug)]
    struct SelectBox {
        pick: String,
        open_seen: bool,
    }

    #[derive(Debug, Clone, PartialEq)]
    enum SbMsg {
        Pick(usize, String),
    }

    impl Component for SelectBox {
        type Msg = SbMsg;
        fn on(&mut self, msg: Self::Msg) {
            if let SbMsg::Pick(_, s) = msg {
                self.pick = s;
                self.open_seen = false;
            }
        }
        fn view(&self) -> View<Self::Msg> {
            let options = vec!["Small".to_string(), "Medium".to_string(), "Large".to_string()];
            let selected_index = options.iter().position(|o| o == &self.pick);
            View::col()
                .child(View::Select {
                    options,
                    selected_index,
                    on_select: Some(SelectCallback::new(|i, s| SbMsg::Pick(i, s.to_string()))),
                    style: None,
                })
                .child(View::text(format!("pick: {}", self.pick)))
                .build()
        }
    }

    #[test]
    fn select_closed_golden_and_open() {
        let mut p = proj_center_off(
            SelectBox { pick: "Small".into(), open_seen: false },
            480.0,
            320.0,
        );
        p.ensure_covered().expect("select 入覆盖集");
        let frame = p.render_frame();
        // 闭态：值盒 + 当前值 + ▾；无选项列。
        assert_eq!(texts_of(&frame), vec!["Small", "▾", "pick: Small"]);
        assert!(quads_of(&frame).iter().any(|r| *r == WRect::new(10.0, 10.0, 320.0, 32.0)));

        // 点击盒 → 开（覆盖序选项列在主块后追加）。
        click(&mut p, 100.0, 26.0);
        let frame = p.render_frame();
        let texts = texts_of(&frame);
        assert_eq!(&texts[..5], &["Small", "▾", "pick: Small", "Small", "Medium"], "选项列置顶: {texts:?}");
        assert!(texts.contains(&"Large"));
        // 高亮当前项（选项 0 rect (10,42,320,32) quad = BUTTON_BG；其余
        // 选项 INPUT_BG）。
        let hl = frame.ops.iter().find_map(|op| match op {
            DrawOp::Quad { rect, color } if *rect == WRect::new(10.0, 42.0, 320.0, 32.0) => Some(*color),
            _ => None,
        });
        assert_eq!(hl, Some(accent_fill()), "当前项高亮");
    }

    #[test]
    fn select_option_dispatch_and_close() {
        let mut p = proj_center_off(
            SelectBox { pick: "Small".into(), open_seen: false },
            480.0,
            320.0,
        );
        let _ = p.render_frame();
        click(&mut p, 100.0, 26.0); // 开
        let _ = p.render_frame(); // 开态帧刷新命中表（泵语义：rev 前进即产帧）。
        // 命中选项 1（Medium）——rect (10, 74, 320, 32) 中心（选项列
        // 自盒底 42 起每项 32px）。
        click(&mut p, 170.0, 90.0);
        let frame = p.render_frame();
        assert!(
            texts_of(&frame).iter().any(|t| *t == "Medium"),
            "SelectCallback 物化派发 → 帧值变: {:?}",
            texts_of(&frame)
        );
        assert_eq!(p.select_open, None, "命中后关闭");
        // 闭态帧无选项列。
        assert_eq!(texts_of(&frame), vec!["Medium", "▾", "pick: Medium"]);
    }

    #[test]
    fn select_outside_click_closes_only() {
        let mut p = proj_center_off(
            SelectBox { pick: "Small".into(), open_seen: false },
            480.0,
            320.0,
        );
        let _ = p.render_frame();
        click(&mut p, 100.0, 26.0); // 开
        let _ = p.render_frame();
        let before_pick = "Small";
        // 外点（值文本子区域——主块非选项区）→ 仅关闭，不下穿派发。
        click(&mut p, 400.0, 200.0);
        let frame = p.render_frame();
        assert_eq!(p.select_open, None, "外点关闭");
        assert!(
            texts_of(&frame).iter().any(|t| t.starts_with(&format!("pick: {before_pick}"))),
            "外点不派发: {:?}",
            texts_of(&frame)
        );
    }

    #[test]
    fn select_esc_closes() {
        let mut p = proj_center_off(
            SelectBox { pick: "Small".into(), open_seen: false },
            480.0,
            320.0,
        );
        let _ = p.render_frame();
        click(&mut p, 100.0, 26.0); // 开
        assert_eq!(p.select_open, Some(0));
        p.on_input(&InputMsg::KeyPressed { wid: 1, key: 27, modifiers: 0 });
        assert_eq!(p.select_open, None, "Esc 关闭");
        let frame = p.render_frame();
        assert_eq!(texts_of(&frame), vec!["Small", "▾", "pick: Small"], "回闭态");
    }

    // —— PLAN-025 T-05 右键/滚轮/Scissor 单测 ——

    #[derive(Debug)]
    struct Scroller {
        offset_y: f32,
    }

    #[derive(Debug, Clone)]
    enum ScMsg {
        Scrolled(f32),
    }

    impl Component for Scroller {
        type Msg = ScMsg;
        fn on(&mut self, msg: Self::Msg) {
            if let ScMsg::Scrolled(oy) = msg {
                self.offset_y = oy;
            }
        }
        fn view(&self) -> View<Self::Msg> {
            // 视口 h40；内容 3 行（≈72.9px）→ 溢出 → Scissor 对。
            View::col()
                .child(
                    View::scrollable(
                        View::col()
                            .child(View::text("t1"))
                            .child(View::text("t2"))
                            .child(View::text("t3"))
                            .build(),
                    )
                    .height(40)
                    .offset((0.0, self.offset_y))
                    .on_scroll(|m: crate::ui::view::ScrollMetrics| ScMsg::Scrolled(m.offset_y))
                    .build(),
                )
                .child(View::text(format!("oy: {}", self.offset_y)))
                .build()
        }
    }

    #[test]
    fn scrollable_scissor_frame_and_wheel() {
        let mut p = proj_center_off(Scroller { offset_y: 0.0 }, 480.0, 320.0);
        p.ensure_covered().expect("scroll 入覆盖集（layouts + scroll）");
        let frame = p.render_frame();
        // 溢出（内容 72.9 > 视口 40）→ Scissor push/pop 对在册。
        let scissors = frame
            .ops
            .iter()
            .filter(|op| matches!(op, DrawOp::Scissor { .. } | DrawOp::ScissorPop))
            .count();
        assert_eq!(scissors, 2, "Scissor push+pop 对: {:?}", frame.ops);

        // 滚轮（唯一 Scrollable 兜底——指针位缺席也能定位）dy=+15 →
        // offset' = clamp(0+15, 0..=32.9) = 15 → on_scroll 派发 → app 状态。
        p.on_input(&InputMsg::Scroll { wid: 1, dx: 0.0, dy: 15.0 });
        let frame = p.render_frame();
        assert!(
            texts_of(&frame).iter().any(|t| *t == "oy: 15"),
            "滚轮派发 offset' 前进: {:?}",
            texts_of(&frame)
        );

        // 越界钳制：dy=+1000 → offset' = content_h - viewport_h
        // （内容 3×21.6 + 2×gap8 = 80.8 → max_oy ≈ 40.8）。
        p.on_input(&InputMsg::Scroll { wid: 1, dx: 0.0, dy: 1000.0 });
        let frame = p.render_frame();
        assert!(
            texts_of(&frame).iter().any(|t| t.starts_with("oy: 40.8")),
            "滚轮末端钳制: {:?}",
            texts_of(&frame)
        );
    }

    #[test]
    fn right_click_dispatch() {
        #[derive(Debug)]
        struct RClick {
            lefts: u32,
            rights: u32,
        }
        #[derive(Debug, Clone)]
        enum RMsg {
            Left,
            Right,
        }
        impl Component for RClick {
            type Msg = RMsg;
            fn on(&mut self, msg: Self::Msg) {
                match msg {
                    RMsg::Left => self.lefts += 1,
                    RMsg::Right => self.rights += 1,
                }
            }
            fn view(&self) -> View<Self::Msg> {
                View::col()
                    .child(
                        View::Button {
                            label: "ctx".into(),
                            onclick: RMsg::Left,
                            style: None,
                            on_right_click: Some(RMsg::Right),
                            content: None,
                            disabled: false,
                        },
                    )
                    .child(View::text(format!("l{} r{}", self.lefts, self.rights)))
                    .build()
            }
        }
        let mut p = proj_center_off(RClick { lefts: 0, rights: 0 }, 480.0, 320.0);
        let _ = p.render_frame();
        // 按钮盒 (10,10,120,36) 中心右键 → Right 派发（帧文本 l0 r1）。
        p.on_input(&InputMsg::PointerPressed {
            wid: 1,
            button: MouseButton::Right,
            x: 70.0,
            y: 28.0,
            modifiers: 0,
        });
        let frame = p.render_frame();
        assert!(
            texts_of(&frame).iter().any(|t| *t == "l0 r1"),
            "右键派发 Right: {:?}",
            texts_of(&frame)
        );
        // 左键同区 → Left 派发（l1 r1）。
        p.on_input(&InputMsg::PointerPressed {
            wid: 1,
            button: MouseButton::Left,
            x: 70.0,
            y: 28.0,
            modifiers: 0,
        });
        let frame = p.render_frame();
        assert!(
            texts_of(&frame).iter().any(|t| *t == "l1 r1"),
            "左键派发 Left: {:?}",
            texts_of(&frame)
        );
    }

    // —— PLAN-025 T-06 防漏钉（双向）+ native queue 金样 ——

    /// 防漏钉（双向，parity_matrix_covers_target_set 的 native 同型）：
    /// ① 覆盖表 → 投影器臂：native_queue_set 每个 kind/layout 在矩阵
    /// 夹具中在场，且各夹具 ensure_covered 通过 + 渲染零 uncovered
    /// （= 投影器**确有**对应臂——gate 过但臂缺会当场炸）；② 投影器臂
    /// → 覆盖表：矩阵中每种被渲染的变体 kind 均在 native_queue_set
    ///（防"臂已写、表未扩"漂移）。
    #[test]
    fn native_coverage_matrix_pinned_to_projector() {
        use std::collections::BTreeSet;

        #[derive(Debug)]
        struct Matrix;
        #[derive(Debug, Clone)]
        enum MMsg {
            Nop,
        }
        impl Component for Matrix {
            type Msg = MMsg;
            fn on(&mut self, _m: Self::Msg) {}
            fn view(&self) -> View<Self::Msg> {
                View::col()
                    .child(View::text("t"))
                    .child(View::button("b").on_click(|_| MMsg::Nop).build())
                    .child(View::input("i").build())
                    .child(View::textarea("ta").build())
                    .child(View::checkbox(true, "cb"))
                    .child(View::radio(false, "r"))
                    .child(View::slider(0.0..=1.0, 0.5).on_change(|_| MMsg::Nop).build())
                    .child(View::Select {
                        options: vec!["o".into()],
                        selected_index: Some(0),
                        on_select: None,
                        style: None,
                    })
                    .child(View::scrollable(View::text("s")).height(16).build())
                    // PLAN-026 T-03 display 族夹具（image/progress 臂在场
                    // ——覆盖表扩容 kinds+image/progress 的防漏钉夹具）。
                    .child(View::image_styled("img", "w-16 h-16"))
                    .child(View::progress_bar_styled(0.5, "w-20 h-2"))
                    // PLAN-029 shell queue 面四 kind 夹具（T-04/T-05/T-06
                    // ——覆盖表扩容的防漏钉夹具；popover 闭态零面板 ops）。
                    .child(View::Popover {
                        anchor: crate::ui::view::PopoverAnchor::Widget(Box::new(
                            View::button("pm").on_click(|_| MMsg::Nop).build(),
                        )),
                        content: Box::new(View::text("pc")),
                        placement: PopoverPlacement::BottomStart,
                        open: false,
                        on_dismiss: None,
                    })
                    .child(View::MouseArea {
                        content: Box::new(View::text("ma")),
                        on_enter: None,
                        on_exit: None,
                        on_double_click: None,
                        on_click: Some(MMsg::Nop),
                        on_context_menu: Some(MMsg::Nop),
                        on_release: None,
                        on_move: None,
                        logical_extent: Some((40.0, 16.0)),
                        style: None,
                    })
                    .child(View::WindowThumbnail {
                        wid: "42".into(),
                        fallback_icon: "app-window".into(),
                        style: None,
                    })
                    .child(View::WorkspacePreview {
                        ws: "0".into(),
                        fallback_icon: "app-window".into(),
                        style: None,
                    })
                    // PLAN-032 T-03 tabs kind 夹具（防漏钉矩阵双向更新——
                    // 托盘/内容区/on_select 命中全链）。
                    .child(View::Tabs {
                        labels: vec!["tb".into()],
                        contents: vec![View::text("tc")],
                        selected: 0,
                        position: TabsPosition::Top,
                        on_select: Some(TabsSelectCallback::new(|_| MMsg::Nop)),
                        style: None,
                        variant: TabsVariant::Default,
                    })
                    // PLAN-034 T-05 canvas kind 夹具（D4 裁定：位图快照
                    // 过线——防漏钉矩阵双向更新）。
                    .child(View::Canvas {
                        scene: crate::ui::view::CanvasScene::default(),
                        logical_extent: Some((8.0, 4.0)),
                        clear: Some("#ffffff".into()),
                        on_pen_start: None,
                        on_pen_move: None,
                        on_pen_end: None,
                        on_hit: None,
                        style: None,
                    })
                    // PLAN-674 T-01 codeeditor kind 夹具（§10-1 裁定 A：
                    // 结构 DrawOps——防漏钉矩阵双向更新；builder 形态
                    // 产 View::CodeEditor 变体）。
                    .child(
                        View::<MMsg>::code_editor("matrix-674")
                            .value("let x = 1;")
                            .lang("rust")
                            .line_numbers(true)
                            .build(),
                    )
                    .child(View::grid().cols(2).spacing(8).child(View::text("g1")).child(View::text("g2")).build())
                    .child(View::row().child(View::text("r1")).build())
                    .child(View::container(View::text("c")).build())
                    .child(View::list(vec![View::text("l1")]).build())
                    // 透传壳（layouts: empty / anchorslot）。
                    .child(View::spacer())
                    .child(View::AnchorSlot { index: 0, child: Box::new(View::text("a")) })
                    .build()
            }
        }

        let p = proj_center_off(Matrix, 480.0, 320.0);
        let scan = coverage::scan_native_view(&p.component.view());
        let set = Coverage::native_queue_set();

        // ① kinds ∪ layouts ⊆ 夹具扫描标签并集。
        for kind in &set.kinds {
            assert!(scan.tags.contains(kind), "矩阵缺 {kind} 夹具");
        }
        for layout in &set.layouts {
            assert!(scan.tags.contains(layout), "矩阵缺 {layout} 夹具");
        }
        // 夹具 Covered（扫描面过 gate）。
        assert!(p.ensure_covered().is_ok(), "矩阵视图应 Covered");

        // ② 投影器臂在场：渲染零 uncovered_seen（占位臂未触发——
        // 每个登记 kind 都有真臂）。反向钉：native_kind_of 全变体
        // 映射逐一入表 or 显式 not-yet（无第三态——表外 kind 渲染即
        // 占位留痕，由 dynamic_branch 测试钉住）。
        let mut p = proj_center_off(Matrix, 480.0, 320.0);
        let _ = p.render_frame();
        assert!(
            p.uncovered_seen().is_empty(),
            "覆盖 kind 渲染不得落占位臂: {:?}",
            p.uncovered_seen()
        );
        // 防表内幽灵 kind：set 中每个 kind 必须被 native_kind_of 产出
        // 过（矩阵扫描标签并集 == 表面，防手滑多登记）。
        let produced: BTreeSet<String> = scan.tags.iter().cloned().collect();
        for kind in &set.kinds {
            assert!(produced.contains(kind), "表内 kind 无投影臂夹具: {kind}");
        }
        for layout in &set.layouts {
            assert!(produced.contains(layout), "表内 layout 无投影臂夹具: {layout}");
        }
    }

    /// PLAN-674 T-01：codeeditor 投影/键入回传闭环（§10-1 裁定 A：结构
    /// DrawOps + core handle_input 全键面）——渲染面 = EditorDrawList 降层
    /// op（文本 run 入帧）；键入 = 注册表文本增长 + on_change 派发 +
    /// INPUT_TEXT 全文代写；退格/回车同律（key 8/13 编辑器键面）。
    #[test]
    fn codeeditor_rq_typing_roundtrip() {
        use crate::ui::code_editor as ce;
        #[derive(Debug)]
        struct Ed {
            changed: u32,
        }
        #[derive(Debug, Clone)]
        enum EMsg {
            Changed,
        }
        impl Component for Ed {
            type Msg = EMsg;
            fn on(&mut self, _m: Self::Msg) {
                self.changed += 1;
            }
            fn view(&self) -> View<Self::Msg> {
                View::code_editor("rq-674-typing")
                    .value("let x = 1;")
                    .lang("rust")
                    .line_numbers(true)
                    .on_change(EMsg::Changed)
                    .build()
            }
        }
        let key = ce::storage_key("rq-674-typing");
        let _ = ce::code_editor_dispose(&key);
        let mut p = RqProjector::new(Ed { changed: 0 }, 480.0, 320.0);
        p.ensure_covered().expect("codeeditor 入覆盖集");
        let frame = p.render_frame();
        assert!(
            texts_of(&frame).iter().any(|t| t.contains("let") || t.contains("x")),
            "codeeditor 文本 run 入帧: {:?}",
            texts_of(&frame)
        );
        let before = ce::code_editor_text(&key).expect("注册表键在场");
        assert_eq!(before, "let x = 1;", "外部值差分推入");
        // 聚焦（点击编辑器矩形内）+ 键入 '0'。
        click(&mut p, 240.0, 100.0);
        p.on_input(&InputMsg::CharTyped { wid: 1, ch: '0' });
        let after = ce::code_editor_text(&key).expect("注册表键在场");
        assert_eq!(after.len(), before.len() + 1, "键入 1 字符入注册表: {after:?}");
        assert_eq!(p.component().changed, 1, "on_change 派发 1 次");
        assert_eq!(crate::ui::iced::last_input_text(), after, "INPUT_TEXT 全文代写");
        // 退格回原长 + 同通道派发。
        p.on_input(&InputMsg::KeyPressed { wid: 1, key: 8, modifiers: 0 });
        assert_eq!(ce::code_editor_text(&key).unwrap().len(), before.len());
        assert_eq!(p.component().changed, 2, "退格同通道派发");
        // 回车 = 换行插入（key 13 编辑器键面——平面 input 的 Enter
        // not-yet 不受影响，编辑器键位表先行）。
        p.on_input(&InputMsg::KeyPressed { wid: 1, key: 13, modifiers: 0 });
        assert_eq!(
            ce::code_editor_text(&key).unwrap().matches('\n').count(),
            1,
            "回车换行入缓冲"
        );
        assert_eq!(p.component().changed, 3, "回车文本变化同通道派发");
        // IME 提交 = 组合串并入（同流）。
        p.on_input(&InputMsg::ImeCommit { wid: 1, text: "字".to_string() });
        assert!(ce::code_editor_text(&key).unwrap().ends_with('字'), "IME 提交并入");
        // 复帧（rev 前进——换行后两行仍入帧）。
        let frame2 = p.render_frame();
        assert!(!frame2.ops.is_empty());
        let _ = ce::code_editor_dispose(&key);
    }

    /// native queue 金样（003-converter 形态——双 input + 换算文本，
    /// a2r 生成 View 的手建同构；drawlist_to_text 全精度锁，
    /// AUTO_WRITE_GOLDEN=1 重写）。三臂对拍：本金样 = queue 臂；
    /// 独立窗/pixels 臂 = stage3 e2e（p025_native_input_arm）+ 020
    /// 像素臂在册。
    #[test]
    fn native_queue_golden_003_shape() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("test/parity/native");
        std::fs::create_dir_all(&dir).expect("mkdir parity/native");
        let mut p = proj_center_off(
            Converter { celsius: 0.0, fahrenheit: 32.0 },
            480.0,
            320.0,
        );
        let mut out = String::new();
        out.push_str("---- frame 1 ----\n");
        out.push_str(&crate::ui::desktop_protocol::client_runtime::tests::drawlist_to_text(&p.render_frame()));
        // 输入交互差分：聚焦 celsius → 键入 "0100"（视图值 0 + 100）→
        // 换算联动复帧。
        click(&mut p, 100.0, 26.0);
        for ch in "100".chars() {
            p.on_input(&InputMsg::CharTyped { wid: 1, ch });
        }
        out.push_str("---- after input ----\n");
        out.push_str(&crate::ui::desktop_protocol::client_runtime::tests::drawlist_to_text(&p.render_frame()));

        let exp_path = dir.join("003-converter.expected.txt");
        if std::env::var("AUTO_WRITE_GOLDEN").is_ok() || !exp_path.is_file() {
            std::fs::write(&exp_path, &out).expect("write golden");
        }
        let expected = std::fs::read_to_string(&exp_path)
            .unwrap_or_else(|e| panic!("read golden: {e}"));
        if out != expected {
            let _ = std::fs::write(dir.join("003-converter.wrong.txt"), &out);
            panic!(
                "003 native queue 金样不匹配（见 test/parity/native/003-converter.wrong.txt）"
            );
        }
    }

    #[test]
    fn textarea_multiline_golden() {
        #[derive(Debug)]
        struct Note;
        #[derive(Debug, Clone)]
        enum NMsg {}
        impl Component for Note {
            type Msg = NMsg;
            fn on(&mut self, _m: Self::Msg) {}
            fn view(&self) -> View<Self::Msg> {
                View::textarea("note").value("a\nb".to_string()).build()
            }
        }
        let mut p = proj_center_off(Note, 480.0, 320.0);
        p.ensure_covered().expect("textarea 入覆盖集");
        let frame = p.render_frame();
        assert_eq!(texts_of(&frame), vec!["a", "b"], "按 '\\n' 分行");
    }

    /// PLAN-026 T-03：display 族 golden。PLAN-028 真图升级归因——image
    /// 臂占位 Quad → `DrawOp::Image`（src 在场即发，rect 推导零变化；
    /// 占位保真口径转宿主侧未解析兜底），progress 几何维持 quad。I4——
    /// 解释态 layout_image 同刻度（client_runtime.rs）。
    #[test]
    fn display_family_placeholder_golden() {
        #[derive(Debug)]
        struct Disp;
        #[derive(Debug, Clone)]
        enum DMsg {}
        impl Component for Disp {
            type Msg = DMsg;
            fn on(&mut self, _m: Self::Msg) {}
            fn view(&self) -> View<Self::Msg> {
                View::col()
                    .child(View::image_styled("x", "w-16 h-16"))
                    .child(View::image("bare")) // 无样式：缺省 96 上限方形
                    .child(View::progress_bar_styled(0.5, "w-20 h-2"))
                    .child(View::progress_bar_styled(0.25, "bg-blue-500"))
                    .build()
            }
        }
        let quads = |p: &mut RqProjector<Disp>| -> Vec<(f32, f32, f32, f32, Rgba8)> {
            p.render_frame()
                .ops
                .iter()
                .filter_map(|op| match op {
                    DrawOp::Quad { rect, color } => {
                        Some((rect.x, rect.y, rect.w, rect.h, *color))
                    }
                    _ => None,
                })
                .collect()
        };
        // PLAN-028：image op 定位器（真图升级断言面）。
        let images = |p: &mut RqProjector<Disp>| -> Vec<(f32, f32, f32, f32, String)> {
            p.render_frame()
                .ops
                .iter()
                .filter_map(|op| match op {
                    DrawOp::Image { rect, src, .. } => {
                        Some((rect.x, rect.y, rect.w, rect.h, src.clone()))
                    }
                    _ => None,
                })
                .collect()
        };
        let mut p = proj_center_off(Disp, 480.0, 320.0);
        p.ensure_covered().expect("display 族入覆盖集");
        let ims = images(&mut p);
        // styled image：w-16 h-16（Tailwind 刻度 16×4=64px）→ 64×64 Image op。
        assert!(
            ims.iter()
                .any(|&(x, _, w, h, ref s)| x == 10.0 && w == 64.0 && h == 64.0 && s == "x"),
            "styled image 64×64 Image op (x=MARGIN): {ims:?}"
        );
        // 无样式 image：w = min(avail, 96) = 96，h = w（方形缺省）。
        assert!(
            ims.iter()
                .any(|&(x, _, w, h, ref s)| x == 10.0 && w == 96.0 && h == 96.0 && s == "bare"),
            "bare image 缺省 96 方形 Image op: {ims:?}"
        );
        let qs = quads(&mut p);
        // progress 0.5（w-20=80, h-2=8）：track 全长 + fill = w*frac 同位。
        let track = qs
            .iter()
            .find(|&&(x, _, w, h, _)| x == 10.0 && w == 80.0 && h == 8.0)
            .copied()
            .expect("progress track 80×8 (w-20/h-2 刻度)");
        assert!(
            qs.iter().any(|&(x, y, w, h, _)| x == track.0
                && y == track.1
                && w == 40.0
                && h == 8.0),
            "progress 0.5 → fill 40×8 同位: {qs:?}"
        );
        // progress 0.25 无尺寸类：w = avail = 480；fill = 120 同位。
        let track2 = qs
            .iter()
            .find(|&&(x, _, w, h, _)| x == 10.0 && w == 460.0 && h == 8.0)
            .copied()
            .expect("progress 缺省宽 460×8");
        let fill2 = qs
            .iter()
            .find(|&&(x, y, w, h, _)| x == track2.0
                && y == track2.1
                && w == 115.0
                && h == 8.0)
            .copied()
            .expect("progress 0.25 → fill 115×8 同位");
        assert_ne!(
            fill2.4, track2.4,
            "style.bg 覆盖 fill 色（bg-blue-500 ≠ 轨道底）: {qs:?}"
        );
    }

    /// PLAN-026 T-04：grid walker golden + 命中派发（镜像
    /// client_runtime::layout_grid :831——cols 等宽格 row-major，格宽 =
    /// (内容宽 - gap×(cols-1))/cols；格内按钮命中派发正确）。
    #[test]
    fn grid_layout_and_hit_golden() {
        #[derive(Debug)]
        struct GridApp {
            hits: u32,
        }
        #[derive(Debug, Clone)]
        enum GMsg {
            Hit,
        }
        impl Component for GridApp {
            type Msg = GMsg;
            fn on(&mut self, m: Self::Msg) {
                if matches!(m, GMsg::Hit) {
                    self.hits += 1;
                }
            }
            fn view(&self) -> View<Self::Msg> {
                // 2 列 × gap 8；内容宽 460-0 pad → 格宽 (460-8)/2 = 226。
                View::grid()
                    .cols(2)
                    .spacing(8)
                    .child(View::text("a"))
                    .child(View::button("b1").on_click(|_| GMsg::Hit).build())
                    .child(View::button("b2").on_click(|_| GMsg::Hit).build())
                    .child(View::text("d"))
                    .build()
            }
        }
        let mut p = proj_center_off(GridApp { hits: 0 }, 480.0, 320.0);
        p.ensure_covered().expect("grid 入覆盖集");
        let frame = p.render_frame();
        let quads: Vec<(f32, f32, f32, f32)> = frame
            .ops
            .iter()
            .filter_map(|op| match op {
                DrawOp::Quad { rect, .. } => Some((rect.x, rect.y, rect.w, rect.h)),
                _ => None,
            })
            .collect();
        // 第 1 行第 1 格（文本 a 无 quad）；第 1 行第 2 格按钮 b1：格 x =
        // MARGIN + 1×(226+8) = 244；第 2 行第 1 格 b2：y 下移一行。
        // 按钮宽 = 内容驱动（BUTTON_MIN_W 档），格位 = 等宽格起点。
        let b1 = quads.iter().find(|&&(x, y, _, _)| x == 244.0 && y == 10.0)
            .expect("b1 格位 (244,10)");
        let b2 = quads.iter().find(|&&(x, y, _, _)| x == 10.0 && y > b1.1)
            .expect("b2 次行首格");
        let _ = (b1, b2);
        // 命中：b1 按钮中心点击派发 Hit。
        click(&mut p, 244.0 + 100.0, 10.0 + 12.0);
        let frame2 = p.render_frame();
        let _ = frame2;
        assert_eq!(p.component.hits, 1, "grid 格内按钮命中派发");
        // b2 同样派发。
        click(&mut p, 10.0 + 100.0, b2.1 + 12.0);
        assert_eq!(p.component.hits, 2, "次行格内按钮命中派发");
    }

    /// PLAN-026 T-04：center 臂（View::center → Container center_x/
    /// center_y——fixed_w 容器内子级水平居中；fixed_h 下垂直居中）。
    #[test]
    fn center_container_golden() {
        #[derive(Debug)]
        struct CApp;
        #[derive(Debug, Clone)]
        enum CMsg2 {
            Nop,
        }
        impl Component for CApp {
            type Msg = CMsg2;
            fn on(&mut self, _m: Self::Msg) {}
            fn view(&self) -> View<Self::Msg> {
                View::center(View::button("mid").on_click(|_| CMsg2::Nop).build())
                    .width(200)
                    .height(100)
                    .build()
            }
        }
        let mut p = proj_center_off(CApp, 480.0, 320.0);
        let frame = p.render_frame();
        let quads: Vec<(f32, f32, f32, f32)> = frame
            .ops
            .iter()
            .filter_map(|op| match op {
                DrawOp::Quad { rect, .. } => Some((rect.x, rect.y, rect.w, rect.h)),
                _ => None,
            })
            .collect();
        // 容器 (10,10) 200×100（legacy width/height 臂——iced 消费面
        // 同源）；按钮 (120×36 内容驱动) 居中 → (10+(200-120)/2,
        // 10+(100-36)/2) = (50,42)。
        let btn = quads
            .iter()
            .find(|&&(x, y, _, _)| x == 50.0 && y == 42.0)
            .expect("center 子级水平+垂直居中（50,42）");
        let _ = btn;
    }

    #[test]
    fn toggle_golden_and_dispatch() {
        #[derive(Debug)]
        struct Toggles {
            on: bool,
            picked: bool,
        }
        #[derive(Debug, Clone)]
        enum TMsg {
            Flip,
            Pick,
        }
        impl Component for Toggles {
            type Msg = TMsg;
            fn on(&mut self, msg: Self::Msg) {
                match msg {
                    TMsg::Flip => self.on = !self.on,
                    TMsg::Pick => self.picked = true,
                }
            }
            fn view(&self) -> View<Self::Msg> {
                View::col()
                    .child(View::Checkbox {
                        is_checked: self.on,
                        label: "opt".into(),
                        on_toggle: Some(TMsg::Flip),
                        style: None,
                    })
                    .child(View::radio(self.picked, "pick").on_select(TMsg::Pick))
                    .child(View::checkbox(false, "no handler")) // 无 handler 不登记
                    .build()
            }
        }
        let mut p = proj_center_off(Toggles { on: false, picked: false }, 480.0, 320.0);
        p.ensure_covered().expect("toggle 族入覆盖集");
        let frame = p.render_frame();
        assert_eq!(texts_of(&frame), vec!["opt", "pick", "no handler"], "标签随盒渲染");
        // checkbox 命中（盒 + 标签整行）：盒 18×18 @ (10,10)，中心 (19,19)。
        click(&mut p, 19.0, 19.0);
        let frame = p.render_frame();
        // 勾选内芯 quad：inset = 18*0.22=3.96 → (13.96,13.96,10.08,10.08)。
        assert!(
            quads_of(&frame)
                .iter()
                .any(|r| (r.x - 13.96).abs() < 0.01 && (r.w - 10.08).abs() < 0.01),
            "选中内芯: {:?}",
            quads_of(&frame)
        );
        // radio 命中（第二行：checkbox 行高 = max(18, 标签行高 18.9)=18.9，
        // radio y = 10 + 18.9 + gap 8 = 36.9，盒中心 y ≈ 44.9）。
        let radio_y = 10.0 + 18.0_f32.max(14.0 * 1.35) + 8.0 + 8.0;
        click(&mut p, 18.0, radio_y);
        let frame = p.render_frame();
        assert!(
            quads_of(&frame)
                .iter()
                .any(|r| (r.w - 16.0).abs() < 0.01 && (r.h - 16.0).abs() < 0.01),
            "radio 盒 16×16 在册"
        );
        // 无 handler checkbox（第三行）：点击不派发（revision 不动）。
        let before = p.revision();
        let third_y = radio_y + 18.0_f32.max(14.0 * 1.35) + 8.0 + 9.0;
        click(&mut p, 19.0, third_y);
        assert_eq!(p.revision(), before, "无 handler 不登记不派发");
    }

    /// 全循环（真实命名管道，同线程协同泵；client_runtime 解释态全循环
    /// 同机件换 native 会话）：握手 → shm 产帧（native View 投影）→ 协议
    /// 点击 → count 递增 → L2Detach 出口。
    #[test]
    fn native_client_full_cycle_over_pipe() {
        use crate::ui::desktop_protocol::client_runtime::{ClientExit, ClientPump, ReconnectPolicy};
        use crate::ui::desktop_protocol::host::ProtocolHost;
        use crate::ui::session::DesktopSession;

        // 宿主侧解析器材料（协议级机件——native child 不消费宿主组件，
        // 但 ResolveAndAttach 仍走名字解析，需合法 .at）。
        const HOST_SRC: &str = r#"widget native-counter { view { text "x" } }"#;

        let pipe = format!("autodesk-native-rt-{}", std::process::id());
        let listener = transport::listen(&pipe).expect("listen");
        let config = ClientConfig {
            app_name: "native-counter".into(),
            title: "native-counter".into(),
            width: 480.0,
            height: 320.0,
        };

        // child 泵（native queue 臂——单线程，无 Send 需求）。
        let app_end = transport::connect(&pipe, 2000).expect("connect");
        let projector = proj_center_off(Counter { count: 0 }, 480.0, 320.0);
        projector.ensure_covered().expect("counter 级入覆盖集");
        let reconnect = ReconnectPolicy { pipe: pipe.clone(), budget_ms: 30_000, interval_ms: 50 };
        let mut client =
            ClientPump::new(app_end, projector, config, Some(reconnect));
        let mut server_end = listener.wait_connect().expect("server connect");

        // 桌面侧：真实 462 会话 + ProtocolHost 泵（合成走既有通道）。
        let mut session = DesktopSession::__test_session();
        session.open_desktop(iced::window::Id::unique());
        let mut ph = ProtocolHost::new(&mut session, move |name: &str| {
            if name == "native-counter" {
                crate::build_dynamic_component(HOST_SRC, None).map_err(|e| format!("{e}"))
            } else {
                Err(format!("unknown app {name}"))
            }
        });

        fn pump(server_end: &mut Box<dyn transport::Transport + Send>, ph: &mut ProtocolHost<'_>) {
            while let Some(loaded) = server_end.try_recv() {
                let msg = loaded.expect("解码");
                ph.handle(&msg).expect("host 状态机");
                for reply in std::mem::take(&mut ph.to_app) {
                    let _ = server_end.send(&reply);
                }
            }
        }

        fn drive(
            server_end: &mut Box<dyn transport::Transport + Send>,
            ph: &mut ProtocolHost<'_>,
            client: &mut ClientPump<RqProjector<Counter>>,
        ) -> Option<(ClientExit, RqProjector<Counter>)> {
            pump(server_end, ph);
            client.step()
        }

        // 泵到 Active（Hello → Welcome/BufferAlloc → Ready；native 首帧）。
        let mut wid = None;
        for _ in 0..1000 {
            if let Some((exit, _)) = drive(&mut server_end, &mut ph, &mut client) {
                panic!("Active 前意外出口 {exit:?}");
            }
            if !ph.session.apps.is_empty() {
                wid = ph.active().1;
                if ph.composed(wid.expect("wid").0).is_some() {
                    break;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let wid = wid.expect("child 已孵化");
        let composed = ph.composed(wid.0).expect("Active 首帧已合成");
        assert!(
            composed.ops.iter().any(|op| matches!(op, DrawOp::Text { text, .. } if text == "count: 0")),
            "native View 投影帧已入宿主: {composed:?}"
        );

        // 协议点击（按钮区内）→ native 命中派发 → 帧递增。
        // 布局：col 顶部 text(10,10,h≈24.3) + gap 8 → 按钮 y≈42.3 x=10
        // w=120 h=36 → 点 (70, 60)。
        let injected = ph.pointer_down(70.0, 60.0, MouseButton::Left).expect("窗内命中");
        server_end.send(&injected).unwrap();
        let mut count_seen = false;
        for _ in 0..1000 {
            if let Some((exit, _)) = drive(&mut server_end, &mut ph, &mut client) {
                panic!("点击阶段意外出口 {exit:?}");
            }
            if let Some(list) = ph.composed(wid.0) {
                if list.ops.iter().any(|op| matches!(op, DrawOp::Text { text, .. } if text == "count: 1")) {
                    count_seen = true;
                    break;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(count_seen, "协议点击经 native 命中表派发产帧递增");

        // L2Detach → child 出口 L2Detached；projector 状态交还（revision 连续）。
        let detach = ph.endpoint.l2_detach().expect("Active 才可 l2_detach");
        server_end.send(&detach).unwrap();
        let (exit, projector) = loop {
            if let Some(done) = drive(&mut server_end, &mut ph, &mut client) {
                break done;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        };
        assert_eq!(exit, ClientExit::L2Detached);
        assert_eq!(projector.revision(), 2, "点击一次 revision 连续");
    }

    /// PLAN-025 T-02 集成（真管道全循环，form 级）：双 input 换算 App
    /// —— 协议点击聚焦 input 0 → 协议级 CharTyped 注入（P020-D4 GUI
    /// 自动化债未清前的承载口径，⑤）→ INPUT_TEXT → on_change → 换算
    /// 联动帧断言（AC-02 的协议级证据）。
    #[test]
    fn native_form_full_cycle_over_pipe() {
        use crate::ui::desktop_protocol::client_runtime::{ClientConfig, ClientPump};
        use crate::ui::desktop_protocol::host::ProtocolHost;
        use crate::ui::desktop_protocol::message::ProtocolMsg;
        use crate::ui::session::DesktopSession;

        const HOST_SRC: &str = r#"widget native-form { view { text "x" } }"#;

        let pipe = format!("autodesk-native-form-{}", std::process::id());
        let listener = transport::listen(&pipe).expect("listen");
        let config = ClientConfig {
            app_name: "native-form".into(),
            title: "native-form".into(),
            width: 480.0,
            height: 320.0,
        };

        let app_end = transport::connect(&pipe, 2000).expect("connect");
        let projector = proj_center_off(
            Converter { celsius: 0.0, fahrenheit: 32.0 },
            480.0,
            320.0,
        );
        let mut client = ClientPump::new(app_end, projector, config, None);
        let mut server_end = listener.wait_connect().expect("server connect");

        let mut session = DesktopSession::__test_session();
        session.open_desktop(iced::window::Id::unique());
        let mut ph = ProtocolHost::new(&mut session, move |name: &str| {
            if name == "native-form" {
                crate::build_dynamic_component(HOST_SRC, None).map_err(|e| format!("{e}"))
            } else {
                Err(format!("unknown app {name}"))
            }
        });

        fn pump(server_end: &mut Box<dyn transport::Transport + Send>, ph: &mut ProtocolHost<'_>) {
            while let Some(loaded) = server_end.try_recv() {
                let msg = loaded.expect("解码");
                ph.handle(&msg).expect("host 状态机");
                for reply in std::mem::take(&mut ph.to_app) {
                    let _ = server_end.send(&reply);
                }
            }
        }

        fn drive(
            server_end: &mut Box<dyn transport::Transport + Send>,
            ph: &mut ProtocolHost<'_>,
            client: &mut ClientPump<RqProjector<Converter>>,
        ) {
            pump(server_end, ph);
            if let Some((exit, _)) = client.step() {
                panic!("form 全循环意外出口 {exit:?}");
            }
        }

        // 泵到 Active + 首帧。
        let mut wid = None;
        for _ in 0..1000 {
            drive(&mut server_end, &mut ph, &mut client);
            if !ph.session.apps.is_empty() {
                wid = ph.active().1;
                if ph.composed(wid.expect("wid").0).is_some() {
                    break;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let wid = wid.expect("child 已孵化");
        let wwid = wid.0;
        let frame_has = |ph: &ProtocolHost<'_>, needle: &str| {
            ph.composed(wwid).is_some_and(|list| {
                list.ops
                    .iter()
                    .any(|op| matches!(op, DrawOp::Text { text, .. } if text.starts_with(needle)))
            })
        };
        assert!(frame_has(&ph, "32"), "首帧 fahrenheit=32 在册");

        // 协议点击 input 0（rect (10,10,320,32) 中心）→ 聚焦（聚焦
        // 推版一帧——泵数轮消化）。
        let injected = ph.pointer_down(170.0, 26.0, MouseButton::Left).expect("窗内命中");
        server_end.send(&injected).unwrap();
        for _ in 0..10 {
            drive(&mut server_end, &mut ph, &mut client);
            std::thread::sleep(std::time::Duration::from_millis(5));
        }

        // 协议级 CharTyped 注入 "5" → buffer "05" → celsius=5 →
        // fahrenheit=41 联动帧。
        let typed = ProtocolMsg::Input(InputMsg::CharTyped { wid: wid.0, ch: '5' });
        server_end.send(&typed).unwrap();
        let mut seen = false;
        for _ in 0..1000 {
            drive(&mut server_end, &mut ph, &mut client);
            if frame_has(&ph, "41") {
                seen = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(seen, "CharTyped 经 INPUT_TEXT → on_change → 换算联动帧: {:?}", ph.composed(wwid));
    }

    // —— PLAN-029 T-04 popover 臂（D3 全 14 placement + 覆盖序/命中/Esc）——

    #[derive(Debug, Clone, PartialEq)]
    enum PopMsg {
        Go,
        Dismiss,
    }

    /// popover 载体：锚（Widget=按钮 / Point=坐标）+ 面板（文本 + 按钮）；
    /// on_dismiss = Dismiss（on 内自关——应用态驱动，投影器零开合状态）。
    #[derive(Debug)]
    struct PopHost {
        open: bool,
        point_anchor: bool,
        placement: PopoverPlacement,
        last: Option<PopMsg>,
    }

    impl PopHost {
        fn widget(open: bool, placement: PopoverPlacement) -> Self {
            Self { open, point_anchor: false, placement, last: None }
        }
    }

    impl Component for PopHost {
        type Msg = PopMsg;
        fn on(&mut self, msg: Self::Msg) {
            if msg == PopMsg::Dismiss {
                self.open = false;
            }
            self.last = Some(msg);
        }
        fn view(&self) -> View<Self::Msg> {
            let anchor = if self.point_anchor {
                PopoverAnchor::Point { x: 40.0, y: 60.0 }
            } else {
                PopoverAnchor::Widget(Box::new(
                    View::button("menu").on_click(|_| PopMsg::Go).build(),
                ))
            };
            View::col()
                .child(View::Popover {
                    anchor,
                    content: Box::new(
                        View::col()
                            .child(View::text("panel-item"))
                            .child(View::button("go").on_click(|_| PopMsg::Go).build())
                            .build(),
                    ),
                    placement: self.placement,
                    open: self.open,
                    on_dismiss: Some(PopMsg::Dismiss),
                })
                .build()
        }
    }

    /// D3 几何纯函数：全 14 枚举 + 对向翻转（file 级常量代入）。
    #[test]
    fn popover_geometry_all_placements() {
        use PopoverPlacement as PP;
        let site = PopoverAnchorSite::Widget(WRect::new(100.0, 50.0, 80.0, 24.0));
        let panel = (288.0, 96.0);
        let vp = (480.0, 320.0);
        // Bottom 族：y = 锚底 + GAP；Start 左对齐 / 居中 clamp。
        assert_eq!(popover_panel_origin(&site, &PP::BottomStart, panel, vp, (0.0, 0.0)), (100.0, 80.0));
        assert_eq!(popover_panel_origin(&site, &PP::Bottom, panel, vp, (0.0, 0.0)), (8.0, 80.0));
        assert_eq!(popover_panel_origin(&site, &PP::BottomEnd, panel, vp, (0.0, 0.0)), (8.0, 80.0));
        // Top 族上溢 → 对向翻转到下方。
        assert_eq!(popover_panel_origin(&site, &PP::TopStart, panel, vp, (0.0, 0.0)), (100.0, 80.0));
        assert_eq!(popover_panel_origin(&site, &PP::Top, panel, vp, (0.0, 0.0)), (8.0, 80.0));
        // Left 左溢 → 右翻（右位 186 → clamp 184）。
        assert_eq!(
            popover_panel_origin(&site, &PP::Left, panel, vp, (0.0, 0.0)),
            (184.0, 14.0)
        );
        // Right 微溢（186+288=474 > 472）→ 翻左 -194 → 钳 8（翻转-钳制
        // 确定性口径——窄视口下双侧不贴合，贴边落定）。
        assert_eq!(
            popover_panel_origin(&site, &PP::Right, panel, vp, (0.0, 0.0)),
            (8.0, 14.0)
        );
        // Modal 居中 / Edge* 贴边。
        assert_eq!(popover_panel_origin(&site, &PP::Modal, panel, vp, (0.0, 0.0)), (96.0, 112.0));
        assert_eq!(popover_panel_origin(&site, &PP::EdgeLeft, panel, vp, (0.0, 0.0)), (8.0, 112.0));
        assert_eq!(popover_panel_origin(&site, &PP::EdgeRight, panel, vp, (0.0, 0.0)), (184.0, 112.0));
        assert_eq!(popover_panel_origin(&site, &PP::EdgeTop, panel, vp, (0.0, 0.0)), (96.0, 8.0));
        assert_eq!(popover_panel_origin(&site, &PP::EdgeBottom, panel, vp, (0.0, 0.0)), (96.0, 216.0));
        // Pointer = 最近右键点。
        assert_eq!(
            popover_panel_origin(&site, &PP::Pointer, panel, vp, (30.0, 40.0)),
            (30.0, 40.0)
        );
        // Point 锚恒 BottomStart 语义（原点对齐）。
        let pt = PopoverAnchorSite::Point { x: 40.0, y: 60.0 };
        assert_eq!(popover_panel_origin(&pt, &PP::BottomStart, panel, vp, (0.0, 0.0)), (40.0, 60.0));
        // Top 对 Point 锚同样回落 BottomStart 语义（缺省先例）。
        assert_eq!(popover_panel_origin(&pt, &PP::Top, panel, vp, (0.0, 0.0)), (40.0, 60.0));
    }

    /// 开态覆盖序渲染（面板 ops 主块后追加 = paint order 置顶）+ 命中
    /// 互斥（面板项 > catcher > 主块）+ 外点/Esc → on_dismiss + 闭态零
    /// 面板 ops（open 随帧）。
    #[test]
    fn popover_open_closed_render_and_hit_semantics() {
        let mut p = proj_center_off(PopHost::widget(true, PopoverPlacement::BottomStart), 480.0, 320.0);
        p.ensure_covered().expect("popover 载体 Covered");
        let frame = p.render_frame();
        // 开态：面板底 Quad（POP_BG）+ 面板文本在场。
        let panel_quads: Vec<WRect> = frame
            .ops
            .iter()
            .filter_map(|op| match op {
                DrawOp::Quad { rect, color } if *color == pop_bg() => Some(*rect),
                _ => None,
            })
            .collect();
        assert_eq!(panel_quads.len(), 1, "单面板底: {panel_quads:?}");
        assert!(texts_of(&frame).iter().any(|t| *t == "panel-item"), "面板子树渲染: {:?}", texts_of(&frame));
        // 命中登记序：catcher 在场且面板项（Msg）在其后（rev 序面板项胜）。
        let dismiss_idx = p
            .hits
            .iter()
            .position(|e| matches!(e, HitEntry::PopoverDismiss { .. }))
            .expect("catcher 在场");
        let go_idx = p
            .hits
            .iter()
            .rposition(|e| matches!(e, HitEntry::Msg { msg, .. } if matches!(msg, PopMsg::Go)))
            .expect("面板项在场");
        // 面板内 go 按钮 + 锚 menu 按钮都是 Go——取 catcher 之后者（面板项）。
        assert!(go_idx > dismiss_idx, "面板项在 catcher 后（rev 序胜）");
        // 外点 → on_dismiss 派发 + 吞（锚按钮不派发）。
        p.on_input(&InputMsg::PointerPressed {
            wid: 1,
            button: MouseButton::Left,
            x: 470.0,
            y: 310.0,
            modifiers: 0,
        });
        assert_eq!(p.component.last, Some(PopMsg::Dismiss), "外点 → on_dismiss");
        // 应用态自关 → 闭帧行零面板。
        let closed = p.render_frame();
        assert!(
            !closed.ops.iter().any(|op| matches!(op, DrawOp::Quad { color, .. } if *color == pop_bg())),
            "闭态零面板 ops（open 随帧）"
        );
        assert!(!p.hits.iter().any(|e| matches!(e, HitEntry::PopoverDismiss { .. })));

        // Esc → on_dismiss（重开态）。
        p.component.open = true;
        let _ = p.render_frame();
        p.on_input(&InputMsg::KeyPressed { wid: 1, key: 27, modifiers: 0 });
        assert_eq!(p.component.last, Some(PopMsg::Dismiss), "Esc → on_dismiss");

        // 面板项命中 → 项消息派发（Go——非 Dismiss）。
        p.component.open = true;
        p.component.last = None;
        let _ = p.render_frame();
        let go_rect = p
            .hits
            .iter()
            .rev()
            .find_map(|e| match e {
                HitEntry::Msg { rect, msg } if matches!(msg, PopMsg::Go) => Some(*rect),
                _ => None,
            })
            .expect("面板 go 项");
        p.on_input(&InputMsg::PointerPressed {
            wid: 1,
            button: MouseButton::Left,
            x: go_rect.x + 2.0,
            y: go_rect.y + 2.0,
            modifiers: 0,
        });
        assert_eq!(p.component.last, Some(PopMsg::Go), "面板项命中派发项消息");
    }

    /// Modal scrim：全屏半透明 Quad 先于面板 ops。
    #[test]
    fn popover_modal_scrim_order() {
        let mut p = proj_center_off(
            PopHost::widget(true, PopoverPlacement::Modal),
            480.0,
            320.0,
        );
        let frame = p.render_frame();
        let scrim_idx = frame
            .ops
            .iter()
            .position(|op| matches!(op, DrawOp::Quad { rect, color } if *color == POP_SCRIM && rect.w == 480.0 && rect.h == 320.0))
            .expect("scrim 全屏 Quad");
        let panel_idx = frame
            .ops
            .iter()
            .position(|op| matches!(op, DrawOp::Quad { color, .. } if *color == pop_bg()))
            .expect("面板底");
        assert!(scrim_idx < panel_idx, "scrim 先于面板（paint order）");
    }

    // —— PLAN-029 T-05/T-06 桥接与命中臂 ——

    #[derive(Debug, Clone, PartialEq)]
    enum BridgeMsg {
        Click,
        Ctx,
        Inner,
    }

    /// 桥接 + MouseArea 载体：thumbnail/preview 两 op + area（click/
    /// contextmenu）包裹按钮（content 项优先命中验证）。
    #[derive(Debug)]
    struct BridgeHost {
        last: Option<BridgeMsg>,
    }

    impl Component for BridgeHost {
        type Msg = BridgeMsg;
        fn on(&mut self, msg: Self::Msg) {
            self.last = Some(msg);
        }
        fn view(&self) -> View<Self::Msg> {
            View::col()
                .child(View::WindowThumbnail {
                    wid: "42842".into(),
                    fallback_icon: w_fallback(),
                    style: None,
                })
                .child(View::WorkspacePreview {
                    ws: "1".into(),
                    fallback_icon: "app-window".into(),
                    style: None,
                })
                .child(View::MouseArea {
                    content: Box::new(
                        View::button("inner")
                            .on_click(|_| BridgeMsg::Inner)
                            .build(),
                    ),
                    on_enter: None,
                    on_exit: None,
                    on_double_click: None,
                    on_click: Some(BridgeMsg::Click),
                    on_context_menu: Some(BridgeMsg::Ctx),
                    on_release: None,
                    on_move: None,
                    logical_extent: Some((200.0, 60.0)),
                    style: None,
                })
                .build()
        }
    }

    fn w_fallback() -> String {
        "panel-top".into()
    }

    /// T-05：两虚拟引用 src 语法（thumbnail://{wid}!{fallback} /
    /// workspace://{ws}!{fallback}）——宿主解析侧（SWR/合成）028/T-07
    /// 在册，本测钉投影器桥接语法。
    #[test]
    fn thumbnail_preview_bridge_src_grammar() {
        let mut p = proj_center_off(BridgeHost { last: None }, 480.0, 320.0);
        p.ensure_covered().expect("桥接载体 Covered");
        let frame = p.render_frame();
        let srcs: Vec<&str> = frame
            .ops
            .iter()
            .filter_map(|op| match op {
                DrawOp::Image { src, .. } => Some(src.as_str()),
                _ => None,
            })
            .collect();
        assert!(
            srcs.iter().any(|s| *s == "thumbnail://42842!panel-top"),
            "thumbnail 桥接语法: {srcs:?}"
        );
        assert!(
            srcs.iter().any(|s| *s == "workspace://1!app-window"),
            "workspace 桥接语法: {srcs:?}"
        );
    }

    /// T-06：MouseArea 命中序——content 项（按钮）rev 序优先于 area 命中；
    /// 空白落 area（click）；右键 → contextmenu。
    #[test]
    fn mousearea_hit_priority_and_context_menu() {
        let mut p = proj_center_off(BridgeHost { last: None }, 480.0, 320.0);
        let frame = p.render_frame();
        // area 命中盒（200×60 逻辑_extent）。
        let area_rect = frame
            .ops
            .iter()
            .find_map(|op| match op {
                DrawOp::Quad { rect, .. } if rect.w == 200.0 && rect.h == 60.0 => Some(*rect),
                _ => None,
            });
        let _ = area_rect;
        // 按钮盒（inner）与 area 盒皆从 hits 取（同文件测试可及）。
        let (btn, area) = {
            let mut btn = None;
            let mut area = None;
            for e in &p.hits {
                if let HitEntry::Msg { rect, msg } = e {
                    if matches!(msg, BridgeMsg::Inner) {
                        btn = Some(*rect);
                    }
                    if matches!(msg, BridgeMsg::Click) {
                        area = Some(*rect);
                    }
                }
            }
            (btn.expect("按钮命中"), area.expect("area 命中"))
        };
        // 命中登记序：area 先、按钮后（rev 序按钮胜）。
        let area_idx = p
            .hits
            .iter()
            .position(|e| matches!(e, HitEntry::Msg { msg, .. } if matches!(msg, BridgeMsg::Click)))
            .expect("area idx");
        let btn_idx = p
            .hits
            .iter()
            .position(|e| matches!(e, HitEntry::Msg { msg, .. } if matches!(msg, BridgeMsg::Inner)))
            .expect("btn idx");
        assert!(btn_idx > area_idx, "content 项后登记（rev 序优先）");
        // 点按钮 → Inner（非 area Click）。
        p.on_input(&InputMsg::PointerPressed {
            wid: 1,
            button: MouseButton::Left,
            x: btn.x + 2.0,
            y: btn.y + 2.0,
            modifiers: 0,
        });
        assert_eq!(p.component.last, Some(BridgeMsg::Inner), "按钮优先命中");
        // 点 area 空白 → Click。
        p.on_input(&InputMsg::PointerPressed {
            wid: 1,
            button: MouseButton::Left,
            x: area.x + area.w - 2.0,
            y: area.y + area.h - 2.0,
            modifiers: 0,
        });
        assert_eq!(p.component.last, Some(BridgeMsg::Click), "空白落 area");
        // 右键 → Ctx。
        p.on_input(&InputMsg::PointerPressed {
            wid: 1,
            button: MouseButton::Right,
            x: area.x + 4.0,
            y: area.y + 4.0,
            modifiers: 0,
        });
        assert_eq!(p.component.last, Some(BridgeMsg::Ctx), "右键 contextmenu");
    }

    // ------------------------------------------------------------------
    // PLAN-033 T-03：VM 三补（D1 回写 / D2 timer / D3 命令读走）
    // ------------------------------------------------------------------

    /// PLAN-033 T-03①（AC-02）：003-converter 真源——零参内联闭包 oninput
    /// 的 input_state_map 回写闭环。native 臂键入路径：聚焦 → CharTyped →
    /// INPUT_TEXT 代写 → on()（D1=A：绑定字段类型保值回写[零参闭包读它]
    /// + 单参注入）。键入 "1" 后：击中 Celsius 框 → fahrenheit = 1×9/5+32
    /// = 33.8；击中 Fahrenheit 框 → celsius = (1-32)×5/9 ≈ -13.89（两向
    /// 皆证回写——零参闭包读的就是回写后的绑定字段）。
    #[test]
    fn p033_vm_input_writeback_zero_param() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/ui/003-converter/src/front/app.at"
        );
        let Ok(src) = std::fs::read_to_string(path) else {
            eprintln!("[p033] skip: 003-converter 载体缺席");
            return;
        };
        let comp = crate::build_dynamic_component(&src, None).expect("build 003");
        let mut p = proj_center_off(comp, 480.0, 320.0);
        p.ensure_covered().expect("003 native 覆盖门");
        p.render_frame();
        // 逐命中矩形试探：点中心 → 键入 "1" → 检查双向换算落点。
        let rects = p.hit_rects();
        assert!(!rects.is_empty(), "003 有命中区");
        for r in &rects {
            p.on_input(&InputMsg::PointerPressed {
                wid: 1,
                button: MouseButton::Left,
                x: r.x + r.w / 2.0,
                y: r.y + r.h / 2.0,
                modifiers: 0,
            });
            p.on_input(&InputMsg::CharTyped { wid: 1, ch: '1' });
            let f = num_state(&p, "fahrenheit");
            let c = num_state(&p, "celsius");
            if f == Some(33.8) {
                assert!(
                    (c.unwrap_or(0.0) - 1.0).abs() < 0.01,
                    "celsius 绑定字段同步 = 1（回写实证）: {c:?}"
                );
                return;
            }
            if (c.unwrap_or(0.0) + 13.888888).abs() < 0.01 {
                assert!(
                    (f.unwrap_or(0.0) - 1.0).abs() < 0.01,
                    "fahrenheit 绑定字段同步 = 1（回写实证）: {f:?}"
                );
                return;
            }
        }
        panic!("003 双 input 均未落换算（回写缺失）");
    }

    /// PLAN-033 T-03①（AC-02）：单参 oninput 形态——注入臂（PLAN-013 W2）
    /// 与回写臂叠加（on_with_input_for 语义位对位）：t 收全量文本 + 绑定
    /// 字段类型保值同步。
    #[test]
    fn p033_vm_input_writeback_single_param() {
        let src = "widget P {\n    model {\n        var q double = 0\n        var out str = \"\"\n    }\n    view { input (value: .q) { oninput: .SetQ } }\n    on { .SetQ(t) -> { .out = t } }\n}\n";
        let comp = crate::build_dynamic_component(src, None).expect("build");
        let mut p = proj_center_off(comp, 480.0, 320.0);
        p.render_frame();
        let r = p.hit_rects()[0];
        p.on_input(&InputMsg::PointerPressed {
            wid: 1,
            button: MouseButton::Left,
            x: r.x + 4.0,
            y: r.y + 4.0,
            modifiers: 0,
        });
        for ch in "21".chars() {
            p.on_input(&InputMsg::CharTyped { wid: 1, ch });
        }
        assert_eq!(num_state(&p, "q"), Some(21.0), "绑定字段 double 保值回写");
        let out = p.component().read_state("out").ok().map(|v| match v {
            auto_val::Value::Str(s) => s.as_str().to_string(),
            other => format!("{other:?}"),
        });
        // buffer 自视图值初始化（double 0 → "0"）再追加键入——native 臂
        // 编辑语义（a2r 同构）；注入 t = 全量 buffer 文本。
        assert_eq!(out, Some("021".to_string()), "单参注入 t = 全量 buffer 文本");
    }

    /// PLAN-033 T-03②（AC-02）：VM timer 经泵侧周期拍派发（D2=B）——首拍
    /// 对齐 interval 不立即拍；到期拍 → handler 执行 + revision 前进。
    #[test]
    fn p033_vm_timer_poll_tick() {
        let src = "widget T {\n    msg { Beat }\n    model { var beat int = 0 }\n    timer { Beat (every_ms: 60) }\n    view { text `beat: ${.beat}` }\n    on { .Beat -> { .beat += 1 } }\n}\n";
        let comp = crate::build_dynamic_component(src, None).expect("build");
        let mut p = proj_center_off(comp, 480.0, 320.0);
        let rev0 = p.revision();
        p.poll_tick();
        assert_eq!(int_state(&p, "beat"), Some(0), "首拍对齐 interval（不立即拍）");
        std::thread::sleep(std::time::Duration::from_millis(80));
        p.poll_tick();
        assert_eq!(int_state(&p, "beat"), Some(1), "到期拍派发 handler");
        assert!(p.revision() > rev0, "派发驱动 revision（泵对账产帧）");
    }

    /// PLAN-033 T-03③（AC-02）：`__desktop_cmd` 读走（c4 语义——read +
    /// 清空 + 分行；D3 组件面）。
    #[test]
    fn p033_desktop_cmd_drain() {
        let src = "widget D {\n    model {\n        var n int = 0\n        var __desktop_cmd str = \"\"\n    }\n    view { text \"d\" }\n}\n";
        let comp = crate::build_dynamic_component(src, None).expect("build");
        let mut p = proj_center_off(comp, 480.0, 320.0);
        assert!(p.drain_desktop_commands().is_empty(), "空态幂等");
        p.component_mut()
            .write_state("__desktop_cmd", auto_val::Value::str("launch\u{1f}counter\nnotify\u{1f}hi"))
            .expect("write");
        assert_eq!(
            p.drain_desktop_commands(),
            vec![
                "launch\u{1f}counter".to_string(),
                "notify\u{1f}hi".to_string()
            ],
            "两命令分行读走"
        );
        assert!(p.drain_desktop_commands().is_empty(), "读走即清空（幂等）");
    }

    fn num_state(
        p: &RqProjector<crate::ui::dynamic::DynamicComponent>,
        field: &str,
    ) -> Option<f64> {
        p.component().read_state(field).ok().and_then(|v| match v {
            auto_val::Value::Double(d) => Some(d),
            auto_val::Value::Float(f) => Some(f as f64),
            auto_val::Value::Int(i) => Some(i as f64),
            auto_val::Value::Str(s) => s.parse::<f64>().ok(),
            _ => None,
        })
    }

    fn int_state(
        p: &RqProjector<crate::ui::dynamic::DynamicComponent>,
        field: &str,
    ) -> Option<i64> {
        p.component().read_state(field).ok().and_then(|v| match v {
            auto_val::Value::Int(i) => Some(i as i64),
            auto_val::Value::Str(s) => s.parse::<i64>().ok(),
            _ => None,
        })
    }

    /// PLAN-034 T-05（D4：canvas=位图快照过线）：canvas 臂投影——覆盖门
    /// 放行（"canvas" 入 native_queue_set）+ render_frame 产
    /// `Image{src: bitmap://…}` op + drain 产出位图上传（栅格化孪生真
    /// 像素：clear 铺底 + 笔画 ink）+ 同签名二渲染零重传。
    #[test]
    fn p034_canvas_snapshot_arm_projects_and_uploads() {
        use crate::ui::view::{CanvasScene, CanvasStroke, View as V};

        #[derive(Debug)]
        struct CanvasApp {
            scene: CanvasScene,
        }

        impl crate::ui::component::Component for CanvasApp {
            type Msg = u32;
            fn on(&mut self, _msg: Self::Msg) {}
            fn view(&self) -> V<Self::Msg> {
                V::Canvas {
                    scene: self.scene.clone(),
                    logical_extent: Some((8.0, 4.0)),
                    clear: Some("#ffffff".into()),
                    on_pen_start: None,
                    on_pen_move: None,
                    on_pen_end: None,
                    on_hit: None,
                    style: None,
                }
            }
        }

        let scene = CanvasScene {
            strokes: vec![CanvasStroke {
                points: vec![(1.0, 2.0), (7.0, 2.0)],
                color: "#111827".into(),
                width: 2.0,
                eraser: false,
            }],
            ..Default::default()
        };
        let mut proj = proj_center_off(CanvasApp { scene }, 64.0, 32.0);
        // 覆盖门：canvas 经位图快照臂入册（此前拒绝）。
        proj.ensure_covered().expect("canvas 覆盖门放行");

        let list = proj.render_frame();
        let src = list
            .ops
            .iter()
            .find_map(|op| match op {
                DrawOp::Image { src, .. } => Some(src.clone()),
                _ => None,
            })
            .expect("canvas → Image op");
        assert!(src.starts_with("bitmap://"), "bitmap:// 词汇：{src}");
        assert!(src.contains("canvas-0"), "槽位 id：{src}");

        let uploads = FrameSource::drain_bitmap_uploads(&mut proj);
        assert_eq!(uploads.len(), 1, "首渲染产出一次上传");
        let up = &uploads[0];
        assert_eq!((up.w, up.h, up.stride as usize), (8, 4, 8 * 4), "幅面 = 逻辑 extent");
        assert_eq!(up.rgba.len(), 8 * 4 * 4, "RGBA 长度");
        assert!(up.rgba.chunks(4).any(|px| px[0] > 200 && px[1] > 200), "clear 白底在场");
        assert!(
            up.rgba.chunks(4).any(|px| px[0] < 100 && px[1] < 100),
            "笔画 ink 在场（栅格化孪生真像素）"
        );
        // 同签名二渲染：零重传（带宽抑制）。
        let list2 = proj.render_frame();
        let _ = list2;
        assert!(
            FrameSource::drain_bitmap_uploads(&mut proj).is_empty(),
            "同签名场景零重传"
        );
    }
}
