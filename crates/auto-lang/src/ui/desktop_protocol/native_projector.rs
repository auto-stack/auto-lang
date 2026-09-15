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
    DISABLED_ALPHA, INPUT_BG, LABEL_FG, LINE_H_FACTOR, MARGIN, PLACEHOLDER_FG, TEXT_FG, TEXT_SIZE,
};
use super::coverage::{self, Coverage, Verdict};
use super::endpoint::FrameSource;
use super::message::{ControlMsg, DrawList, DrawOp, InputMsg, MouseButton, Rgba8, WRect};
use crate::ui::component::Component;
use crate::ui::style::{Color, Style, StyleClass};
use crate::ui::view::View;

/// View 枚举 → DrawList 投影器（实现 [`FrameSource`]，作
/// `AppEndpoint` 的会话——[`super::client_runtime::ClientPump`] 泛型泵
/// 驱动，native queue 臂全链）。
pub struct NativeProjector<C: Component> {
    component: C,
    /// 最近一帧的命中区 `(rect, 物化消息)`（渲染时刷新）。
    hits: Vec<(WRect, C::Msg)>,
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
        self.hits = ctx.hits;
        self.uncovered_seen = ctx.uncovered;
        DrawList { clear: Some(BG), ops: ctx.ops }
    }

    fn on_input(&mut self, input: &InputMsg) {
        match input {
            InputMsg::PointerPressed { x, y, button: MouseButton::Left, .. } => {
                let hit = self.hits.iter().position(|(r, _)| {
                    *x >= r.x && *x < r.x + r.w && *y >= r.y && *y < r.y + r.h
                });
                if let Some(i) = hit {
                    let msg = self.hits[i].1.clone();
                    self.component.on(msg);
                    self.rev += 1;
                }
            }
            // 键盘/滚轮/右键 v1 不路由（覆盖集无 input 族；右键为解释态
            // queue 臂同边界）。
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
    hits: Vec<(WRect, M)>,
    uncovered: Vec<String>,
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
        View::Button { label, onclick, content, disabled, .. } => {
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
                ctx.hits.push((WRect::new(x, y, w, h), onclick.clone()));
            }
            Laid { size: (w, h) }
        }
        View::Row { .. } => {
            let dir = Dir::Horizontal;
            layout_view_group(ctx, view, &style, x, y, avail_w, dir)
        }
        View::Column { .. } | View::List { .. } => {
            let dir = Dir::Vertical;
            layout_view_group(ctx, view, &style, x, y, avail_w, dir)
        }
        View::Container { child, .. } => {
            layout_view_container(ctx, child, &style, x, y, avail_w)
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
    layout_view_block(ctx, children, x, y, avail_w, dir, &group_style)
}

/// 容器（View::Container）：bg/padding 包装 + 子级块流。z 序镜像
/// `client_runtime::layout_container`（底色先于子级——量尺寸 → 撤产物 →
/// bg → 重排子级 → border）。
fn layout_view_container<M: Clone + std::fmt::Debug>(
    ctx: &mut NativeCtx<M>,
    child: &View<M>,
    style: &NodeStyle,
    x: f32,
    y: f32,
    avail_w: f32,
) -> Laid {
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
        | View::Container { style, .. } => style.as_ref(),
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
    fn coverage_gate_refuses_payload_family() {
        // counter 级视图：Covered。
        let ok = counter();
        assert!(ok.ensure_covered().is_ok());

        // payload 族（Slider）：not-yet → 拒绝 + 缺项清单。
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
        let err = p.ensure_covered().unwrap_err();
        assert!(err.contains("slider"), "缺项清单随行: {err}");
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
                View::text_styled("x", "shadow-lg")
            }
        }

        let p = NativeProjector::new(Shadowed, 480.0, 320.0);
        let err = p.ensure_covered().unwrap_err();
        assert!(err.contains("style:shadow"), "native v1 无 shadow 渲染: {err}");
    }

    #[test]
    fn dynamic_branch_uncovered_placeholder_tracked() {
        // 门后动态分支：状态切换遭遇未覆盖变体 → 占位盒 + uncovered_seen。
        #[derive(Debug)]
        struct Branchy {
            show_slider: bool,
        }

        #[derive(Debug, Clone, PartialEq)]
        enum BMsg {
            Toggle,
        }

        impl Component for Branchy {
            type Msg = BMsg;
            fn on(&mut self, msg: Self::Msg) {
                if msg == BMsg::Toggle {
                    self.show_slider = !self.show_slider;
                }
            }
            fn view(&self) -> View<Self::Msg> {
                // 门时刻 slider 不可见 → Covered；Toggle 后动态出现。
                let mut col = View::col().child(
                    View::button("t").on_click(|_| BMsg::Toggle).build(),
                );
                if self.show_slider {
                    col = col.child(View::slider(0.0..=1.0, 0.5, |_| BMsg::Toggle).build());
                }
                col.build()
            }
        }

        let mut p = NativeProjector::new(Branchy { show_slider: false }, 480.0, 320.0);
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
        assert_eq!(p.uncovered_seen(), ["slider"], "动态分支遭遇留痕");
        assert!(
            texts_of(&frame).iter().any(|t| t.starts_with("not-rendered: slider")),
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
}
