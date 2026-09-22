// PLAN-683（方案 2：远程 Renderer）—— headless iced 宿主。
//
// **架构**（T-00 spike 定案，替代原计划的"自研 RecordRenderer trait 实现"）：
// `iced::Renderer` 在本仓 feature 集下 = `iced_renderer::fallback::Renderer
// <iced_wgpu::Renderer, iced_tiny_skia::Renderer>` 公开枚举——
// `Renderer::Secondary(iced_tiny_skia::Renderer::new(..))` 纯 CPU 构造
//（零 wgpu/零窗口），与 `view.into_iced()` 产出的组件树（实例化于
// `iced::Renderer`）**类型同源**，`UserInterface::draw` 可直接驱动。
// tiny_skia 后端是延迟栅格化设计：draw 期原语只被记录进
// `layers()`（quads/text/images + 各自 clip 层），本模块把这些记录降格为
// 协议 DrawList 过线——iced 组件树的布局/命中/聚焦/IME 全原生，渲染语义
// 与独立轨同出一份 iced 实现（"函数调用变 RPC"的字面达成）。
//
// 与设计文档 rq-remote-renderer §4 的偏差登记：原设想的 RecordRenderer
// = 手写 `iced_core::Renderer` + 文本子 trait 全表面；实勘发现该路线须把
// 仓内 3.5 万行 `into_iced()` 适配层按 Renderer 泛型化（Element 树在构造
// 期钉死具体 Renderer 类型），成本不可行。tiny_skia Secondary 臂是同一
// 截获语义的现成载体（其 Layer 即"记录原语的 display list"），且官方
// CPU 后端长期维护。详见 T-00 决策档。

use crate::ui::component::Component;
use crate::ui::desktop_protocol::message::{DrawList, DrawOp, Rgba8, WRect};
use crate::ui::iced::renderer::IntoIcedElement;

use iced::advanced::graphics::text::Text as GfxText;
use iced::advanced::graphics::text::font_system;
use iced::advanced::renderer as advanced_renderer;
use iced::mouse;
use iced::theme::Base;
use iced::Element;
use iced_runtime::user_interface::{Cache, UserInterface};
use iced_tiny_skia::Layer;

/// 字体装载（进程内一次）：独立轨 application 链同款 Inter 三字重
///（renderer.rs Plan 411 P1-C），中文等族外字形走系统回退（cosmic-text）。
/// daemon 侧重整行盒所需字体经 FontBlob 握手通道下发（413 §7.2）。
pub(crate) fn ensure_headless_fonts() {
    use std::sync::Once;
    static FONTS: Once = Once::new();
    FONTS.call_once(|| {
        let mut system = font_system().write().expect("Write font system");
        system.load_font(crate::ui::iced::renderer::INTER_FONT_REGULAR.into());
        system.load_font(crate::ui::iced::renderer::INTER_FONT_MEDIUM.into());
        system.load_font(crate::ui::iced::renderer::INTER_FONT_SEMIBOLD.into());
    });
}

/// headless 会话表面：组件 + 视口 + iced widget 树缓存。
/// 帧循环（T-02 扩展 revision 门控）：`render_frame` = view() →
/// UserInterface::build（复用 Cache 状态树 diff）→ draw（tiny_skia 记录）
/// → 降格 DrawList。
pub struct HeadlessSurface<C: Component> {
    pub component: C,
    width: f32,
    height: f32,
    cursor: mouse::Cursor,
    cache: Cache,
    /// 上帧观测行（未覆盖原语/降格近似——wire v1 词汇面之外的内容）。
    observations: Vec<String>,
}

impl<C: Component> HeadlessSurface<C>
where
    C::Msg: Clone + std::fmt::Debug + 'static,
{
    pub fn new(component: C, width: f32, height: f32) -> Self {
        ensure_headless_fonts();
        Self {
            component,
            width,
            height,
            cursor: mouse::Cursor::Unavailable,
            cache: Cache::default(),
            observations: Vec::new(),
        }
    }

    /// 产一帧（v1 wire 降格——T-01 升 DisplayList v2）。
    pub fn render_frame(&mut self) -> DrawList {
        let mut view = self.component.view();
        crate::ui::iced::renderer::rewrite_viewport_units(&mut view);
        let element: Element<'_, C::Msg> = view.into_iced();

        // tiny_skia Secondary 臂：纯 CPU 记录器（iced::Renderer 的合法值）。
        let mut renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            crate::ui::iced::renderer::INTER_FONT,
            iced::Pixels(16.0),
        ));

        let cache = std::mem::take(&mut self.cache);
        let mut ui = UserInterface::build(
            element,
            iced::Size::new(self.width, self.height),
            cache,
            &mut renderer,
        );

        // RedrawRequested 先行（iced_test snapshot 同款驱动）——部分 widget
        // 首帧状态推进依赖一次 redraw 事件。
        let mut messages = Vec::new();
        let _ = ui.update(
            &[iced::Event::Window(iced::window::Event::RedrawRequested(
                iced::time::Instant::now(),
            ))],
            self.cursor,
            &mut renderer,
            &mut iced::advanced::clipboard::Null,
            &mut messages,
        );

        // 主题单源（PLAN-679）：与独立轨 run 链同款 shadcn 调色板。
        let theme = crate::ui::iced::renderer::shadcn_theme(
            crate::ui::style::iced_adapter::dark_mode(),
        );
        let style = advanced_renderer::Style {
            text_color: theme.base().text_color,
        };
        ui.draw(&mut renderer, &theme, &style, self.cursor);
        self.cache = ui.into_cache();
        for msg in messages {
            self.component.on(msg);
        }

        // Secondary 臂拆取 tiny_skia 记录层（fallback 枚举不转发 layers()）。
        let layers: &[Layer] = match &mut renderer {
            iced::Renderer::Secondary(tiny) => tiny.layers(),
            iced::Renderer::Primary(_) => unreachable!("headless 宿主恒构造 Secondary 臂"),
        };
        self.observations.clear();
        lower_layers(layers, &mut self.observations)
    }

    /// 远程事件注入（T-02 事件回路径主入口；T-00 首帧 spike 仅渲染）。
    pub fn update(&mut self, events: &[iced::Event]) -> Vec<C::Msg> {
        let mut view = self.component.view();
        crate::ui::iced::renderer::rewrite_viewport_units(&mut view);
        let element: Element<'_, C::Msg> = view.into_iced();

        let mut renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            crate::ui::iced::renderer::INTER_FONT,
            iced::Pixels(16.0),
        ));

        let cache = std::mem::take(&mut self.cache);
        let mut ui = UserInterface::build(
            element,
            iced::Size::new(self.width, self.height),
            cache,
            &mut renderer,
        );
        let mut messages = Vec::new();
        if !events.is_empty() {
            let _ = ui.update(
                events,
                self.cursor,
                &mut renderer,
                &mut iced::advanced::clipboard::Null,
                &mut messages,
            );
        }
        self.cache = ui.into_cache();
        messages
    }

    /// 指针位同步（daemon 上行 PointerMoved 后调用——hover 态渲染依赖）。
    pub fn point_at(&mut self, position: Option<iced::Point>) {
        self.cursor = match position {
            Some(p) => mouse::Cursor::Available(p),
            None => mouse::Cursor::Unavailable,
        };
    }

    /// 上帧降格观测（未入 wire 的原语——T-01 逐项消解的台账）。
    pub fn observations(&self) -> &[String] {
        &self.observations
    }
}

// ---------------------------------------------------------------------------
// Layer 记录 → DrawList v1 降格
// ---------------------------------------------------------------------------

/// tiny_skia 记录层 → v1 DrawList。层序 = tiny_skia 渲染序（base 层先画，
/// 后续 clip 层叠上）；层 0 不发 Scissor（全域），其余层以自身 bounds 裁剪
///（与 tiny_skia 光栅期"仅按层自身 bounds 裁剪"同口径）。
fn lower_layers(layers: &[Layer], observations: &mut Vec<String>) -> DrawList {
    let mut ops = Vec::new();
    for (index, layer) in layers.iter().enumerate() {
        let clipped = index > 0;
        if clipped {
            ops.push(DrawOp::Scissor { rect: to_wrect(layer.bounds) });
        }
        for (quad, background) in &layer.quads {
            lower_quad(quad, background, &mut ops, observations);
        }
        for item in &layer.text {
            for text in item.as_slice() {
                lower_text(text, item.transformation(), &mut ops, observations);
            }
        }
        if !layer.images.is_empty() {
            observations.push(format!(
                "image x{}（v1 词汇面外——T-01 v2 Image 原语承接）",
                layer.images.len()
            ));
        }
        if !layer.primitives.is_empty() {
            observations.push(format!(
                "mesh primitive x{}（canvas/vector 面——T-01 勘定：Path 序列化或位图降级）",
                layer.primitives.len()
            ));
        }
        if clipped {
            ops.push(DrawOp::ScissorPop);
        }
    }
    DrawList { clear: Some(super::client_runtime::bg()), ops }
}

/// quad 降格：Background::Color → Quad/QuadR（圆角取四角 max——v1 单半径
/// 词汇，四角一致的常态零损）。border 以 inset 单像素描边近似（shadcn
/// 卡面观感主体）；gradient 取停点均值近似；shadow 省略——后三者均为
/// v1 词汇面外，T-01 v2 原生承接（观测行留痕）。
fn lower_quad(
    quad: &advanced_renderer::Quad,
    background: &iced::Background,
    ops: &mut Vec<DrawOp>,
    observations: &mut Vec<String>,
) {
    let rect = to_wrect(quad.bounds);
    let radius = quad
        .border
        .radius
        .top_left
        .max(quad.border.radius.top_right)
        .max(quad.border.radius.bottom_right)
        .max(quad.border.radius.bottom_left);
    match background {
        iced::Background::Color(color) => {
            let rgba = to_rgba8(*color);
            if radius > 0.5 {
                ops.push(DrawOp::QuadR { rect, color: rgba, radius });
            } else {
                ops.push(DrawOp::Quad { rect, color: rgba });
            }
        }
        iced::Background::Gradient(gradient) => {
            observations.push(format!(
                "gradient 降格均值近似（{rect:?}——T-01 v2 原生 stop 记录）"
            ));
            ops.push(DrawOp::QuadR { rect, color: gradient_average(gradient), radius: 0.0 });
        }
    }
    if quad.border.width > 0.0 {
        let border = to_rgba8(quad.border.color);
        let w = quad.border.width;
        let WRect { x, y, w: rw, h: rh } = rect;
        ops.push(DrawOp::Quad { rect: WRect { x, y, w: rw, h: w }, color: border });
        ops.push(DrawOp::Quad { rect: WRect { x, y: y + rh - w, w: rw, h: w }, color: border });
        ops.push(DrawOp::Quad { rect: WRect { x, y, w, h: rh }, color: border });
        ops.push(DrawOp::Quad { rect: WRect { x: x + rw - w, y, w, h: rh }, color: border });
    }
    if quad.shadow.color.a > 0.0
        && (quad.shadow.blur_radius > 0.0
            || quad.shadow.offset.x != 0.0
            || quad.shadow.offset.y != 0.0)
    {
        // v1 无阴影原语——T-01 v2 承接（004 卡面立体感主源）。
        observations.push(format!("shadow 降格省略（{rect:?}——T-01 v2）"));
    }
}

/// 文本记录降格：Paragraph/Editor（cosmic 整形 Buffer）→ 逐 layout run 的
/// TextStyled op（daemon 侧重整形同字体栈）；Cached → 同型直落。
/// 行盒定位 = run 顶（canvas fill_text 顶锚语义同构）；weight 取首字形
///（fontdb 直读）；italic 在 glyph 面缺失——v1 近似省略（观测行）。
fn lower_text(
    text: &GfxText,
    transformation: iced::Transformation,
    ops: &mut Vec<DrawOp>,
    observations: &mut Vec<String>,
) {
    let translation = transformation.translation();
    match text {
        GfxText::Paragraph { paragraph, position, color, .. } => {
            let Some(paragraph) = paragraph.upgrade() else { return };
            lower_buffer_runs(
                paragraph.buffer(),
                *position,
                *color,
                translation,
                ops,
            );
        }
        GfxText::Editor { editor, position, color, .. } => {
            let Some(editor) = editor.upgrade() else { return };
            lower_buffer_runs(editor.buffer(), *position, *color, translation, ops);
        }
        GfxText::Cached {
            content,
            bounds,
            color,
            size,
            line_height,
            font,
            align_x,
            ..
        } => {
            let width = text_width_approx(content, size.0);
            let x = match align_x {
                iced::advanced::text::Alignment::Center => bounds.x - width / 2.0,
                iced::advanced::text::Alignment::Right => bounds.x - width,
                _ => bounds.x,
            };
            ops.push(DrawOp::TextStyled {
                x: x + translation.x,
                y: bounds.y + translation.y,
                size: size.0,
                line_height: line_height.0,
                color: to_rgba8(*color),
                weight: iced_weight_u16(font.weight),
                italic: matches!(font.style, iced::font::Style::Italic | iced::font::Style::Oblique),
                text: content.clone(),
            });
        }
        GfxText::Raw { .. } => {
            observations.push(
                "raw text buffer 降格省略（编辑器组合态——T-02 IME 面）".to_string(),
            );
        }
    }
}

/// cosmic Buffer 逐行降格。行 y = run 顶（line_top），x = 行首字形 x
///（居中对齐/缩进行非零）。
fn lower_buffer_runs(
    buffer: &cosmic_text::Buffer,
    position: iced::Point,
    color: iced::Color,
    translation: iced::Vector,
    ops: &mut Vec<DrawOp>,
) {
    let rgba = to_rgba8(color);
    for run in buffer.layout_runs() {
        let Some(first) = run.glyphs.first() else { continue };
        let line_h = if run.line_height > 0.0 {
            run.line_height
        } else {
            first.font_size * 1.3
        };
        ops.push(DrawOp::TextStyled {
            x: position.x + translation.x + first.x,
            y: position.y + translation.y + run.line_top,
            size: first.font_size,
            line_height: line_h,
            color: rgba,
            weight: first.font_weight.0,
            italic: false,
            text: run.text.to_string(),
        });
    }
}

/// iced 字重枚举 → CSS 刻度 u16（v1 TextStyled 词汇）。
fn iced_weight_u16(weight: iced::font::Weight) -> u16 {
    match weight {
        iced::font::Weight::Thin => 100,
        iced::font::Weight::ExtraLight => 200,
        iced::font::Weight::Light => 300,
        iced::font::Weight::Normal => 400,
        iced::font::Weight::Medium => 500,
        iced::font::Weight::Semibold => 600,
        iced::font::Weight::Bold => 700,
        iced::font::Weight::ExtraBold => 800,
        iced::font::Weight::Black => 900,
    }
}

/// 渐变停点均值（v1 近似——T-01 二选一勘定后替换）。
fn gradient_average(gradient: &iced::gradient::Gradient) -> Rgba8 {
    let stops = match gradient {
        iced::gradient::Gradient::Linear(linear) => &linear.stops,
    };
    let mut count = 0usize;
    let (mut r, mut g, mut b, mut a) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
    for stop in stops.iter().flatten() {
        r += stop.color.r;
        g += stop.color.g;
        b += stop.color.b;
        a += stop.color.a;
        count += 1;
    }
    if count == 0 {
        return Rgba8::new(0, 0, 0, 255);
    }
    let n = count as f32;
    let to_u8 = |v: f32| (v * 255.0).round().clamp(0.0, 255.0) as u8;
    Rgba8::new(to_u8(r / n), to_u8(g / n), to_u8(b / n), to_u8(a / n))
}

/// 宽度近似（Cached 文本对齐定位用——测量启发式，与 v1 投影器
/// measure_text 同刻度：0.6 × size × 字符数）。
fn text_width_approx(content: &str, size: f32) -> f32 {
    content.chars().count() as f32 * size * 0.6
}

fn to_wrect(rect: iced::Rectangle) -> WRect {
    WRect { x: rect.x, y: rect.y, w: rect.width, h: rect.height }
}

fn to_rgba8(color: iced::Color) -> Rgba8 {
    let to_u8 = |v: f32| (v * 255.0).round().clamp(0.0, 255.0) as u8;
    Rgba8::new(to_u8(color.r), to_u8(color.g), to_u8(color.b), to_u8(color.a))
}

// ================================ 测试 ================================

#[cfg(test)]
mod tests {
    use super::*;

    fn example_source(dir: &str) -> Option<String> {
        let base = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/ui/");
        let path = format!("{base}{dir}/src/front/app.at");
        std::fs::read_to_string(&path).ok()
    }

    /// PLAN-683 T-00 —— headless 首帧 spike：001 组件树经 tiny_skia 记录层
    /// 截获 → DrawList v1 → codec round-trip → paint_ops 回放（`()` canvas
    /// 后端，debug 断言档）。验收：非空帧 + 背景铺底 + 标题文本在档。
    #[test]
    fn p683_t00_helloworld_headless_first_frame() {
        let Some(src) = example_source("001-helloworld") else {
            eprintln!("[p683] skip: 001-helloworld 载体缺席");
            return;
        };
        let component =
            crate::build_dynamic_component(&src, None).unwrap_or_else(|e| panic!("{e}"));
        let mut surface = HeadlessSurface::new(component, 480.0, 320.0);
        let list = surface.render_frame();

        assert!(!list.ops.is_empty(), "空帧——headless draw 未产原语");
        assert!(list.clear.is_some(), "缺省 clear（主题底）缺席");

        // 标题文本：TextStyled 含 "Hello, World!"（run 级提取）。
        let has_title = list.ops.iter().any(|op| match op {
            DrawOp::Text { text, .. } | DrawOp::TextStyled { text, .. } => {
                text.contains("Hello, World!")
            }
            _ => false,
        });
        assert!(has_title, "标题文本未入帧: {:?}", list.ops);

        // round-trip：encode → decode 恒等（v1 codec 兼容性）。
        let mut encoded = Vec::new();
        list.encode(&mut encoded);
        let mut reader = crate::ui::desktop_protocol::codec::Reader::new(&encoded);
        let decoded = DrawList::decode(&mut reader).expect("decode");
        assert_eq!(list, decoded, "round-trip 漂移");

        // paint_ops 回放：`()` canvas 后端（debug 断言档 geometry::Renderer）
        // 驱动 daemon 同一降格路径——op 词汇面兼容性（越界值/路径构造炸点）。
        replay_with_paint_ops(&list);
    }

    /// 003-converter：交互组件树（button/input）首帧——组件覆盖 =
    /// iced 全集（coverage 门禁在 remote 模式失效的设计前验证）。
    #[test]
    fn p683_t00_converter_headless_first_frame() {
        let Some(src) = example_source("003-converter") else {
            eprintln!("[p683] skip: 003-converter 载体缺席");
            return;
        };
        let component =
            crate::build_dynamic_component(&src, None).unwrap_or_else(|e| panic!("{e}"));
        let mut surface = HeadlessSurface::new(component, 480.0, 360.0);
        let list = surface.render_frame();
        assert!(!list.ops.is_empty(), "003 空帧");
        let has_text = list.ops.iter().any(|op| matches!(
            op,
            DrawOp::Text { .. } | DrawOp::TextStyled { .. }
        ));
        assert!(has_text, "003 无文本原语（标签/数值面缺席）");
        let mut encoded = Vec::new();
        list.encode(&mut encoded);
        let mut reader = crate::ui::desktop_protocol::codec::Reader::new(&encoded);
        let decoded = DrawList::decode(&mut reader).expect("decode");
        assert_eq!(list, decoded, "003 round-trip 漂移");
        replay_with_paint_ops(&list);
    }

    /// daemon paint_ops 回放（`()` 后端）——与 broker_surface 的实机重放
    /// 同一降格函数。debug 断言档外该后端不存在，测试跳过（release 测档
    /// 由 T-03 实机窗承接）。
    fn replay_with_paint_ops(list: &DrawList) {
        if cfg!(debug_assertions) {
            let bounds =
                iced::Rectangle::new(iced::Point::ORIGIN, iced::Size::new(640.0, 480.0));
            let mut frame = iced::widget::canvas::Frame::with_bounds(&(), bounds);
            frame.fill_rectangle(
                iced::Point::ORIGIN,
                bounds.size(),
                iced::Color::from_rgba8(
                    list.clear.map(|c| c.r).unwrap_or(0),
                    list.clear.map(|c| c.g).unwrap_or(0),
                    list.clear.map(|c| c.b).unwrap_or(0),
                    1.0,
                ),
            );
            crate::ui::iced::broker_surface::paint_ops(&mut frame, &list.ops);
        }
    }
}
