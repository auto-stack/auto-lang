// Plan 020 T-04 —— native queue 投影臂：`View<C::Msg>` → DrawList 投影器。
//
// §5.1 定案（策略 B，运行期 View 投影）：a2r 编译 Component 的 `view()`
// 产物是**全物化 IR**——prop 已解析、handler 已是 `M` 值、条件/循环/插值
// 在构建期求值完毕，投影器即 `View` 的第四消费后端（iced/GPUI/VTree 之外
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
    dim_if, measure_text, NodeStyle, BG, BUTTON_BG, BUTTON_H, BUTTON_MIN_W, BUTTON_PAD,
    DISABLED_ALPHA, INPUT_BG, INPUT_BORDER, LABEL_FG, LINE_H_FACTOR, MARGIN, PLACEHOLDER_FG,
    TEXT_FG, TEXT_SIZE,
};
use super::coverage::{self, Coverage, Verdict};
use super::endpoint::FrameSource;
use super::message::{ControlMsg, DrawList, DrawOp, InputMsg, MouseButton, Rgba8, WRect};
use crate::ui::component::Component;
use crate::ui::style::{Color, Style, StyleClass};
use crate::ui::view::{ScrollCallback, ScrollMetrics, SelectCallback, View};

/// 输入框几何（client_runtime 私有常量的 native 同值镜像——视觉规格
/// 镜像解释态，参数面各自持有）。
const INPUT_H: f32 = 32.0;
const INPUT_PAD: f32 = 10.0;
/// 聚焦描边色（解释态 `resolve_color("blue-500")` 的常量形态）。
const FOCUS_BORDER: Rgba8 = Rgba8::new(59, 130, 246, 255);
/// checkbox/radio 勾选盒标签与盒体的间距。
const CHECK_LABEL_GAP: f32 = 6.0;
/// slider 几何（轨道厚 / knob 边 / 命中带高——v1 常量档，样式类可覆高宽）。
const SLIDER_H: f32 = 20.0;
const SLIDER_TRACK_H: f32 = 4.0;
const SLIDER_KNOB: f32 = 12.0;

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
    Input { rect: WRect, value: String, on_change: Option<M>, slot: usize },
    /// slider：轨道点击 → 几何换算 f32（min..=max 线性 + step 取整）→
    /// fn 指针物化派发（T-01 附带定案：v1 点击定位，拖拽 not-yet）。
    Slider {
        rect: WRect,
        min: f32,
        max: f32,
        step: Option<f32>,
        on_change: fn(f32) -> M,
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

impl<M: Clone + std::fmt::Debug> HitEntry<M> {
    fn rect(&self) -> &WRect {
        match self {
            HitEntry::Msg { rect, .. }
            | HitEntry::Input { rect, .. }
            | HitEntry::Slider { rect, .. }
            | HitEntry::SelectBox { rect, .. }
            | HitEntry::SelectOption { rect, .. }
            | HitEntry::Scroll { rect, .. } => rect,
        }
    }
}

/// 点是否在矩形内（命中判定——既有 position() 谓词的命名提取）。
fn rect_contains(r: &WRect, x: f32, y: f32) -> bool {
    x >= r.x && x < r.x + r.w && y >= r.y && y < r.y + r.h
}

/// View 枚举 → DrawList 投影器（实现 [`FrameSource`]，作
/// `AppEndpoint` 的会话——[`super::client_runtime::ClientPump`] 泛型泵
/// 驱动，native queue 臂全链）。
pub struct NativeProjector<C: Component> {
    component: C,
    /// 最近一帧的分型命中表（渲染时刷新；左键按型派发）。
    hits: Vec<HitEntry<C::Msg>>,
    /// 聚焦 input 槽位（T-01 D1-A：Input/Textarea 槽序身份，帧后重定位；
    /// 点击聚焦，无显式失焦——解释态同边界）。
    focused_input: Option<usize>,
    /// 聚焦框编辑 buffer（T-01 D2：聚焦期显示/编辑面——聚焦时自视图值
    /// 初始化，键入/退格就地编辑后经 INPUT_TEXT 代写回写组件）。
    input_buffer: String,
    /// 开态 select 槽位（T-01 D3：投影器侧开合状态；None = 全闭）。
    select_open: Option<usize>,
    /// 最近一帧的右键命中表（`on_right_click` 物化消息；渲染时刷新）。
    right_hits: Vec<(WRect, C::Msg)>,
    /// 最近指针位（PointerMoved/Pressed 跟踪——wire Scroll 无坐标，滚轮
    /// 路由定位消费；T-01 D5 执行期附注）。
    pointer: (f32, f32),
    /// 渲染期遭遇的未覆盖 kind（动态分支防线——显式留痕面，测试/e2e 断言口）。
    uncovered_seen: Vec<String>,
    rev: u64,
    width: f32,
    height: f32,
    /// 周期拍相位基准（首拍对齐 interval，`iced::time::every` 同相）。
    tick_last: Option<Instant>,
}

impl<C: Component> NativeProjector<C> {
    pub fn new(component: C, width: f32, height: f32) -> Self {
        Self {
            component,
            hits: Vec::new(),
            focused_input: None,
            input_buffer: String::new(),
            select_open: None,
            right_hits: Vec::new(),
            pointer: (0.0, 0.0),
            uncovered_seen: Vec::new(),
            rev: 1,
            width,
            height,
            tick_last: None,
        }
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

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn height(&self) -> f32 {
        self.height
    }
}

impl<C: Component> FrameSource for NativeProjector<C> {
    fn revision(&self) -> u64 {
        self.rev
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
            select_slots: 0,
            select_open: self.select_open,
            overlays: Vec::new(),
            right_hits: Vec::new(),
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
        // 开态 select 覆盖序（T-01 D3）：主块渲染后追加选项列 ops +
        // 命中项——DrawList paint order 天然置顶（无 overlay 协议语义）。
        let overlays = std::mem::take(&mut ctx.overlays);
        for ov in overlays {
            let size = 14.0;
            let line_h = size * LINE_H_FACTOR;
            let mut oy = ov.rect.y + ov.rect.h;
            for (i, opt) in ov.options.iter().enumerate() {
                let or = WRect::new(ov.rect.x, oy, ov.rect.w, INPUT_H);
                ctx.push_quad(or, if ov.selected_index == Some(i) { BUTTON_BG } else { INPUT_BG });
                ctx.ops.push(DrawOp::Text {
                    x: or.x + INPUT_PAD,
                    y: or.y + (INPUT_H - line_h) / 2.0,
                    size,
                    line_height: line_h,
                    color: TEXT_FG,
                    text: opt.clone(),
                });
                if let Some(cb) = &ov.on_select {
                    let msg = cb.call(i, opt);
                    ctx.hits.push(HitEntry::SelectOption { rect: or, msg });
                }
                oy += INPUT_H;
            }
        }
        // 聚焦重定位（T-01 D1-A）：槽位越界 = 视图结构变化 → 失焦 +
        // buffer 清空（不猜测对位——槽序身份在结构变化下不可靠，v1 边界）。
        if let Some(slot) = self.focused_input {
            if slot >= ctx.input_slots {
                self.focused_input = None;
                self.input_buffer.clear();
            }
        }
        // select 开合重定位（同槽序纪律——D3）。
        if let Some(slot) = self.select_open {
            if slot >= ctx.select_slots {
                self.select_open = None;
            }
        }
        self.hits = ctx.hits;
        self.right_hits = ctx.right_hits;
        self.uncovered_seen = ctx.uncovered;
        DrawList { clear: Some(BG), ops: ctx.ops }
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
            InputMsg::KeyPressed { key, .. } if *key == 8 => self.backspace(),
            // Esc（VK_ESCAPE = 27）关开态 select（T-01 D3）。
            InputMsg::KeyPressed { key, .. } if *key == 27 => {
                if self.select_open.take().is_some() {
                    self.rev += 1;
                }
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
            // L3 StateSnapshot 注入：native not-yet（T-03 像素臂同册——
            // typed 组件无字段写回路径；融合态迁移另立）。
            _ => {}
        }
    }

    /// 周期拍（`FrameSource::poll_tick`）：interval 到期 →
    /// `component.on(tick_msg)` + revision 前进（泵侧对账产帧）。
    fn poll_tick(&mut self) {
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

impl<C: Component> NativeProjector<C> {
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
            Some(HitEntry::Input { value, slot, .. }) => {
                self.focused_input = Some(slot);
                self.input_buffer = value;
                self.rev += 1;
            }
            // 轨道点击 → f32 = min + clamp((x-x0)/w)×range（step 取整）→
            // fn 指针物化派发（零 thread-local——载荷自足）。
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
                self.component.on(on_change(v.clamp(min, max)));
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

    /// 键入回写（T-01 D2 定案 A）：编辑 buffer → INPUT_TEXT thread-local
    /// 代写（与 a2r 生成 on() 的 `last_input_text()` 读面同线程接驳——
    /// ClientPump 单线程泵）→ on_change 派发 → rev 前进。
    fn dispatch_input_edit(&mut self, msg: C::Msg) {
        crate::ui::iced::store_input_text(&self.input_buffer);
        self.component.on(msg);
        self.rev += 1;
    }

    /// CharTyped：聚焦框 buffer 追加 → 回写（控制字符不过——Enter/
    /// on_submit not-yet，T-01 D2 随注）。
    fn char_typed(&mut self, ch: char) {
        if ch.is_control() || self.focused_input.is_none() {
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

    /// 滚轮路由（T-01 D5）：内层命中胜 → 唯一 Scrollable 兜底；不命中
    /// 且非唯一 = 静默不路由（多 Scrollable 且指针缺席 → 目标歧义，I3）。
    fn wheel(&mut self, dx: f32, dy: f32) {
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
    /// select 槽位计数（开合身份，D3）。
    select_slots: usize,
    /// 开态 select 槽位快照（臂内判开态渲染 + 命中互斥登记）。
    select_open: Option<usize>,
    /// 开态 select 覆盖序记录（render_frame 主块后统一追加）。
    overlays: Vec<SelectOverlay<M>>,
    /// 右键命中表（Button/Row/Column/Container `on_right_click`）。
    right_hits: Vec<(WRect, M)>,
}

impl<M: Clone + std::fmt::Debug> NativeCtx<M> {
    fn push_quad(&mut self, rect: WRect, color: Rgba8) {
        self.ops.push(DrawOp::Quad { rect, color });
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
    let ops_mark = ctx.ops.len();
    let hits_mark = ctx.hits.len();
    let mut cursor = 0.0f32;
    let mut cross_max = 0.0f32;
    let mut first = true;
    for view in views {
        if !first {
            cursor += gap;
        }
        first = false;
        let style = node_style_of_view(view);
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
                cursor += my + laid.size.1 + style.box_layout.margin_bottom.unwrap_or(0.0);
                cross_max = cross_max.max(laid.size.0);
            }
            Dir::Horizontal => {
                let laid = layout_view_node(ctx, view, x + cursor, y + my, w);
                cursor += laid.size.0;
                cross_max = cross_max.max(my + laid.size.1);
            }
        }
    }
    if dir == Dir::Horizontal && parent.center_children && !views.is_empty() {
        // 主轴居中：撤首轮产物后以居中起点重排（client_runtime 同款两遍法）。
        let used = cursor;
        ctx.ops.truncate(ops_mark);
        ctx.hits.truncate(hits_mark);
        let slack = (w - used).max(0.0) / 2.0;
        let mut cursor = slack;
        let mut cross_max = 0.0f32;
        let mut first = true;
        for view in views {
            if !first {
                cursor += gap;
            }
            first = false;
            let style = node_style_of_view(view);
            let my = style.margin_y();
            let laid = layout_view_node(ctx, view, x + cursor, y + my, w);
            cursor += laid.size.0;
            cross_max = cross_max.max(my + laid.size.1);
        }
        return Laid { size: (w.max(cursor - slack), cross_max) };
    }
    // 垂直块高度 = cursor（主轴累计）——515 G1 修正同源。
    let size = match dir {
        Dir::Vertical => (cross_max.max(0.0), cursor.max(0.0)),
        Dir::Horizontal => (cursor.max(0.0), cross_max.max(0.0)),
    };
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
            let w = measure_text(content, size);
            let tx = if style.text_center {
                x + (avail_w - w).max(0.0) / 2.0
            } else {
                x
            };
            if style.font_bold {
                ctx.ops.push(DrawOp::TextStyled {
                    x: tx,
                    y,
                    size,
                    line_height: line_h,
                    color: style.fg.unwrap_or(TEXT_FG),
                    weight: 700,
                    italic: false,
                    text: content.clone(),
                });
            } else {
                ctx.ops.push(DrawOp::Text {
                    x: tx,
                    y,
                    size,
                    line_height: line_h,
                    color: style.fg.unwrap_or(TEXT_FG),
                    text: content.clone(),
                });
            }
            Laid { size: (w, line_h) }
        }
        View::Button { label, onclick, on_right_click, content, disabled, .. } => {
            let size = style.font_size.unwrap_or(14.0);
            let label_w = measure_text(label, size);
            let w = style
                .fixed_w()
                .unwrap_or((label_w + BUTTON_PAD * 2.0).max(BUTTON_MIN_W))
                .min(avail_w.max(0.0));
            let h = style.fixed_h().unwrap_or(BUTTON_H);
            let bg = dim_if(*disabled, style.bg.unwrap_or(BUTTON_BG));
            ctx.push_quad(WRect::new(x, y, w, h), bg);
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
                    color: dim_if(*disabled, style.fg.unwrap_or(LABEL_FG)),
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
            ctx.push_quad(WRect::new(x, cy - SLIDER_TRACK_H / 2.0, w, SLIDER_TRACK_H), INPUT_BORDER);
            if vx > x {
                ctx.push_quad(WRect::new(x, cy - SLIDER_TRACK_H / 2.0, vx - x, SLIDER_TRACK_H), BUTTON_BG);
            }
            ctx.push_quad(
                WRect::new(vx - SLIDER_KNOB / 2.0, cy - SLIDER_KNOB / 2.0, SLIDER_KNOB, SLIDER_KNOB),
                LABEL_FG,
            );
            ctx.hits.push(HitEntry::Slider {
                rect: WRect::new(x, y, w, h),
                min: *min,
                max: *max,
                step: *step,
                on_change: *on_change,
            });
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
            ctx.push_quad(WRect::new(x, y, w, h), style.bg.unwrap_or(INPUT_BG));
            ctx.push_border(WRect::new(x, y, w, h), style.border.unwrap_or(INPUT_BORDER));
            let size = style.font_size.unwrap_or(14.0);
            let line_h = size * LINE_H_FACTOR;
            if let Some(label) = selected_index.and_then(|i| options.get(i)) {
                ctx.ops.push(DrawOp::Text {
                    x: x + INPUT_PAD,
                    y: y + (h - line_h) / 2.0,
                    size,
                    line_height: line_h,
                    color: style.fg.unwrap_or(TEXT_FG),
                    text: label.clone(),
                });
            }
            ctx.ops.push(DrawOp::Text {
                x: x + w - 14.0,
                y: y + (h - line_h) / 2.0,
                size,
                line_height: line_h,
                color: PLACEHOLDER_FG,
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
        View::Container { child, on_right_click, .. } => {
            layout_view_container(ctx, child, on_right_click.clone(), &style, x, y, avail_w)
        }
        // —— 覆盖门后动态分支防线：占位盒 + 留痕（I3：非静默错绘）。
        other => {
            let kind = coverage::native_kind_of(other);
            ctx.uncovered.push(kind.to_string());
            let w = avail_w.min(160.0).max(40.0);
            let h = 24.0;
            ctx.push_quad(WRect::new(x, y, w, h), INPUT_BG);
            ctx.ops.push(DrawOp::Text {
                x: x + 6.0,
                y: y + 5.0,
                size: 12.0,
                line_height: 16.0,
                color: PLACEHOLDER_FG,
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
    let laid = layout_view_block(ctx, children, x, y, avail_w, dir, &group_style);
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
    style: &NodeStyle,
    x: f32,
    y: f32,
    avail_w: f32,
) -> Laid {
    let container_right_click = right_click;
    let pad = (style.pad_left(), style.pad_top(), style.pad_right(), style.pad_bottom());
    let mut inner_w = avail_w;
    if let Some(fw) = style.fixed_w() {
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
    let outer_w = match style.fixed_w() {
        Some(fw) => fw,
        None => (laid.size.0 + pad.0 + pad.2).max(0.0),
    };
    let outer_h = match style.fixed_h() {
        Some(fh) => fh,
        None => laid.size.1 + pad.1 + pad.3,
    };
    if style.bg.is_some() || style.border.is_some() {
        ctx.ops.truncate(ops_mark);
        ctx.hits.truncate(hits_mark);
        if let Some(bg) = style.bg {
            ctx.push_quad(WRect::new(x, y, outer_w, outer_h), bg);
        }
        let _ = layout_view_block(
            ctx,
            std::slice::from_ref(child),
            x + pad.0,
            y + pad.1,
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
    ctx.push_quad(WRect::new(x, y, w, h), style.bg.unwrap_or(INPUT_BG));
    let slot = ctx.input_slots;
    let focused = ctx.focused_input == Some(slot);
    let border = if focused { FOCUS_BORDER } else { INPUT_BORDER };
    ctx.push_border(WRect::new(x, y, w, h), style.border.unwrap_or(border));
    ctx.input_slots += 1;
    // 显示面（D2）：聚焦框显 buffer（编辑面——解析失败时组件状态不变，
    // 用户意图仍可见），非聚焦框显视图值；空显 placeholder。
    let (text, color) = {
        let shown = if focused { &ctx.input_buffer } else { value };
        if shown.is_empty() {
            (placeholder.to_string(), PLACEHOLDER_FG)
        } else {
            (shown.to_string(), style.fg.unwrap_or(TEXT_FG))
        }
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
        ctx.ops.push(DrawOp::Text {
            x: x + INPUT_PAD,
            y: y + (h - line_h) / 2.0,
            size,
            line_height: line_h,
            color,
            text,
        });
    }
    if let Some(msg) = on_change {
        ctx.hits.push(HitEntry::Input {
            rect: WRect::new(x, y, w, h),
            value: value.to_string(),
            on_change: Some(msg.clone()),
            slot,
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
    ctx.push_quad(WRect::new(x, y, w, h), style.bg.unwrap_or(INPUT_BG));
    ctx.push_border(WRect::new(x, y, w, h), style.border.unwrap_or(INPUT_BORDER));
    if checked {
        let inset = (w.min(h) * inset_factor).clamp(1.5, inset_clamp);
        ctx.push_quad(
            WRect::new(x + inset, y + inset, w - inset * 2.0, h - inset * 2.0),
            style.fg.unwrap_or(BUTTON_BG),
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
            color: style.fg.unwrap_or(TEXT_FG),
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
        | View::Slider { style, .. } => style.as_ref(),
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
            s.border = s.border.or(Some(super::client_runtime::INPUT_BORDER));
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
        StyleClass::ItemsCenter | StyleClass::JustifyCenter | StyleClass::MarginXAuto => {
            s.center_children = true;
        }
        StyleClass::TextCenter => {
            s.center_children = true;
            s.text_center = true;
        }
        // rounded 档：命令帧无圆角 op——直角化（解释态保真边界同款）。
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

    fn counter() -> NativeProjector<Counter> {
        NativeProjector::new(Counter { count: 0 }, 480.0, 320.0)
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
        assert_eq!(frame.clear, Some(BG));
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

        let mut p = NativeProjector::new(D, 480.0, 320.0);
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
                View::slider(0.0..=1.0, 0.5, |_| WMsg::Nop).build()
            }
        }

        let p = NativeProjector::new(WithSlider, 480.0, 320.0);
        p.ensure_covered().expect("slider 入覆盖集（020 拒面反转）");

        // 新拒样本：grid（PLAN-025 非目标——kind 未入册）→ 拒绝 + 缺项。
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

        let p = NativeProjector::new(WithGrid, 480.0, 320.0);
        let err = p.ensure_covered().unwrap_err();
        assert!(err.contains("grid"), "缺项清单随行: {err}");
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
                // shadow 已降级放行（T-06——解释态同款保真边界）；样本换
                // underline（装饰未实现面——token 无支持前缀 → not-yet）。
                View::text_styled("x", "underline")
            }
        }

        let p = NativeProjector::new(Shadowed, 480.0, 320.0);
        let err = p.ensure_covered().unwrap_err();
        assert!(err.contains("style:underline"), "native 无 underline 渲染: {err}");
    }

    #[test]
    fn dynamic_branch_uncovered_placeholder_tracked() {
        // 门后动态分支：状态切换遭遇未覆盖变体 → 占位盒 + uncovered_seen。
        // （样本 = grid——PLAN-025 非目标 kind；原 slider 样本随 T-03
        // 覆盖扩容转正，拒面换 grid 与 gate 反转测试同册。）
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
                // 门时刻 grid 不可见 → Covered；Toggle 后动态出现。
                let mut col = View::col().child(
                    View::button("t").on_click(|_| BMsg::Toggle).build(),
                );
                if self.show_grid {
                    col = col.child(View::Grid {
                        cols: 2,
                        gap: 4,
                        cells: vec![View::text("a"), View::text("b")],
                        style: None,
                    });
                }
                col.build()
            }
        }

        let mut p = NativeProjector::new(Branchy { show_grid: false }, 480.0, 320.0);
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
        assert_eq!(p.uncovered_seen(), ["grid"], "动态分支遭遇留痕");
        assert!(
            texts_of(&frame).iter().any(|t| t.starts_with("not-rendered: grid")),
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

        let mut p = NativeProjector::new(Ticky { ticks: 0 }, 480.0, 320.0);
        let before = p.revision();
        p.poll_tick(); // 首拍只对相位（不派发）。
        assert_eq!(p.revision(), before);
        std::thread::sleep(std::time::Duration::from_millis(8));
        p.poll_tick();
        assert_eq!(p.revision(), before + 1, "interval 到期派发 tick_msg");
    }

    // —— PLAN-025 T-02 form 族单测（golden + 聚焦编辑闭环 + toggle）——

    const FOC: Rgba8 = Rgba8::new(59, 130, 246, 255);

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

    fn click<C: Component>(p: &mut NativeProjector<C>, x: f32, y: f32) {
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
        let mut p = NativeProjector::new(OneInput, 480.0, 320.0);
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
        let frame = NativeProjector::new(Empty, 480.0, 320.0).render_frame();
        assert_eq!(texts_of(&frame), vec!["your name"]);
        let ph_color = frame.ops.iter().find_map(|op| match op {
            DrawOp::Text { color, text, .. } if text == "your name" => Some(*color),
            _ => None,
        });
        assert_eq!(ph_color, Some(PLACEHOLDER_FG), "placeholder 灰");
        // 聚焦 → 描边变蓝（FOCUS_BORDER 顶边 quad）。
        let mut p = NativeProjector::new(OneInput, 480.0, 320.0);
        let _ = p.render_frame(); // 首帧建命中表（点击寻址前提）。
        click(&mut p, 100.0, 26.0);
        let frame = p.render_frame();
        assert!(
            quads_of(&frame)
                .iter()
                .any(|r| r.x == 10.0 && r.y == 10.0 && r.w == 320.0 && r.h == 1.0),
            "顶边 1px 边框在册"
        );
        let has_focus_border = frame.ops.iter().any(|op| match op {
            DrawOp::Quad { color: c, .. } => *c == FOC,
            _ => false,
        });
        assert!(has_focus_border, "聚焦描边 blue-500");
    }

    #[test]
    fn focus_edit_closure_converter() {
        let mut p = NativeProjector::new(
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
                .child(View::slider(0.0..=100.0, self.vol, SMsg::Vol).build())
                .child(View::text(format!("vol: {}", self.vol)))
                .build()
        }
    }

    #[test]
    fn slider_geometry_golden() {
        let mut p = NativeProjector::new(SliderBox { vol: 25.0 }, 480.0, 320.0);
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
        let mut p = NativeProjector::new(SliderBox { vol: 0.0 }, 480.0, 320.0);
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
                View::slider(0.0..=100.0, self.seen, SMsg::Vol).step(30.0).build()
            }
        }
        let mut sp = NativeProjector::new(Stepper { seen: 0.0 }, 480.0, 320.0);
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
        let mut p = NativeProjector::new(
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
        assert_eq!(hl, Some(BUTTON_BG), "当前项高亮");
    }

    #[test]
    fn select_option_dispatch_and_close() {
        let mut p = NativeProjector::new(
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
        let mut p = NativeProjector::new(
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
        let mut p = NativeProjector::new(
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
        let mut p = NativeProjector::new(Scroller { offset_y: 0.0 }, 480.0, 320.0);
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
        let mut p = NativeProjector::new(RClick { lefts: 0, rights: 0 }, 480.0, 320.0);
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
                    .child(View::slider(0.0..=1.0, 0.5, |_| MMsg::Nop).build())
                    .child(View::Select {
                        options: vec!["o".into()],
                        selected_index: Some(0),
                        on_select: None,
                        style: None,
                    })
                    .child(View::scrollable(View::text("s")).height(16).build())
                    .child(View::row().child(View::text("r1")).build())
                    .child(View::container(View::text("c")).build())
                    .child(View::list(vec![View::text("l1")]).build())
                    // 透传壳（layouts: empty / anchorslot）。
                    .child(View::spacer())
                    .child(View::AnchorSlot { index: 0, child: Box::new(View::text("a")) })
                    .build()
            }
        }

        let p = NativeProjector::new(Matrix, 480.0, 320.0);
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
        let mut p = NativeProjector::new(Matrix, 480.0, 320.0);
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
        let mut p = NativeProjector::new(
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
        let mut p = NativeProjector::new(Note, 480.0, 320.0);
        p.ensure_covered().expect("textarea 入覆盖集");
        let frame = p.render_frame();
        assert_eq!(texts_of(&frame), vec!["a", "b"], "按 '\\n' 分行");
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
        let mut p = NativeProjector::new(Toggles { on: false, picked: false }, 480.0, 320.0);
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
        let projector = NativeProjector::new(Counter { count: 0 }, 480.0, 320.0);
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
            client: &mut ClientPump<NativeProjector<Counter>>,
        ) -> Option<(ClientExit, NativeProjector<Counter>)> {
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
        let projector = NativeProjector::new(
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
            client: &mut ClientPump<NativeProjector<Converter>>,
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
}
