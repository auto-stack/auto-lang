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
use crate::ui::desktop_protocol::message::{
    BorderSpec, ControlMsg, DisplayList, DisplayOp, DrawList, DrawOp, Fill, ImeReq, Rgba8,
    ShadowSpec, WRect,
};
use crate::ui::iced::renderer::IntoIcedElement;

use iced::advanced::graphics::text::Text as GfxText;
use iced::advanced::graphics::text::font_system;
use iced::advanced::input_method::{InputMethod, Purpose};
use iced::advanced::renderer as advanced_renderer;
use iced::mouse;
use iced::theme::Base;
use iced::Element;
use iced_runtime::user_interface::{Cache, State as UiState, UserInterface};
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
    /// 上帧观测行（未覆盖原语/降格近似——T-01 逐项消解台账，v2 后剩余项）。
    observations: Vec<String>,
    /// v2 帧待上传位图（Rgba 句柄 → bitmap:// 通道；v1 投影器
    /// drain_bitmap_uploads 同型缝）。
    pending_bitmaps: Vec<super::endpoint::BitmapUpload>,
    /// PLAN-690 T-02：上次下行的 IME 请求（去重基线——iced 每帧
    /// RedrawRequested 重发 `InputMethod::Enabled`，变化才下行）。
    last_ime: Option<ImeReq>,
    /// PLAN-690 T-05：上次下行的光标形状（0 default / 1 pointer / 2 text）。
    last_cursor_kind: u8,
    /// PLAN-690：待下行控制批（ImeRequest/SetCursor——drain_window_controls
    /// 消费，泵在产帧后上行）。
    pending_controls: Vec<ControlMsg>,
    /// PLAN-690 T-03：最近一帧 text_input 上报的组合串（Enabled.preedit
    /// 内容——上行 Preedit 事件 → App iced 态 → 原样上报的环测断言口；
    /// wire ImeRequest v1 不携带组合串本体）。
    preedit_reported: Option<String>,
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
            pending_bitmaps: Vec::new(),
            last_ime: None,
            last_cursor_kind: 0,
            pending_controls: Vec::new(),
            preedit_reported: None,
        }
    }

    /// 产一帧（v1 wire 降格——legacy/parity 探针面；产线帧走
    /// [`Self::render_frame_v2`]）。
    pub fn render_frame(&mut self) -> DrawList {
        let layers = self.drive_and_draw();
        self.observations.clear();
        lower_layers(&layers, &mut self.observations)
    }

    /// 产一帧 v2（PLAN-683 SD-01 remote 模式产线帧）：tiny_skia 记录层 →
    /// DisplayList v2 原生原语（渐变 stop/四角半径/border/shadow/
    /// transform/位图通道）。
    pub fn render_frame_v2(&mut self) -> DisplayList {
        let layers = self.drive_and_draw();
        self.observations.clear();
        self.pending_bitmaps.clear();
        let mut bitmap_seq = 0usize;
        lower_layers_v2(&layers, &mut self.observations, &mut self.pending_bitmaps, &mut bitmap_seq)
    }

    /// 驱动一轮 iced 帧循环：view → UI::build（Cache 复用）→ RedrawRequested
    /// → draw（tiny_skia 记录）→ 消息回灌组件。返回记录层快照（Clone）。
    fn drive_and_draw(&mut self) -> Vec<Layer> {
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
        let (state, _) = ui.update(
            &[iced::Event::Window(iced::window::Event::RedrawRequested(
                iced::time::Instant::now(),
            ))],
            self.cursor,
            &mut renderer,
            &mut iced::advanced::clipboard::Null,
            &mut messages,
        );
        // PLAN-690 T-01：State::Updated.input_method / mouse_interaction
        // 截获（IME 激活门控 + 光标形状的下行源头）。
        self.capture_ui_state(state);

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
        match &mut renderer {
            iced::Renderer::Secondary(tiny) => tiny.layers().to_vec(),
            iced::Renderer::Primary(_) => unreachable!("headless 宿主恒构造 Secondary 臂"),
        }
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
            // 截获面不在事件路径：iced_winit 只在 RedrawRequested 处理臂
            // 消费 input_method/mouse_interaction（lib.rs:940-970）——事件
            // 轮的 shell 态是瞬态 Disabled（text_input 仅在 Redraw 臂请求
            // IME），照收会误发 Disable 下行。drive_and_draw 独占截获。
        }
        self.cache = ui.into_cache();
        // 事件消息回灌（drive_and_draw 同律——组件状态变更即刻生效，
        // 下一帧 view() 反映；INPUT_TEXT 通道同线程纪律：闭包写入 →
        // on() 读面在同一 update 调用内闭合）。返回值保留原消息集
        //（HeadlessFrameSource 的 revision 门控按非空计前进）。
        for msg in &messages {
            self.component.on(msg.clone());
        }
        messages
    }

    /// 指针位同步（daemon 上行 PointerMoved 后调用——hover 态渲染依赖）。
    pub fn point_at(&mut self, position: Option<iced::Point>) {
        self.cursor = match position {
            Some(p) => mouse::Cursor::Available(p),
            None => mouse::Cursor::Unavailable,
        };
    }

    /// PLAN-690 T-01/T-05：`UserInterface::update` 返回态截获——
    /// `State::Updated { input_method, mouse_interaction }` 是 iced 0.14
    /// 公开的 IME 请求/光标形状出口（user_interface.rs:615-640；headless
    /// 旧代码以 `let _ =` 丢弃）。变化才入下行批（iced 每帧重发请求）。
    fn capture_ui_state(&mut self, state: UiState) {
        let UiState::Updated { input_method, mouse_interaction, .. } = state else {
            return;
        };
        let ime = match input_method {
            InputMethod::Disabled => {
                self.preedit_reported = None;
                None
            }
            InputMethod::Enabled { cursor, purpose, preedit } => {
                // 空组合串归一 None（winit Windows commit 序 = Preedit("")
                // 先行清态——text_input 会存空串，上报面按"无组合态"口径）。
                self.preedit_reported =
                    preedit.filter(|p| !p.content.is_empty()).map(|p| p.content);
                Some(ImeReq {
                    cursor: WRect { x: cursor.x, y: cursor.y, w: cursor.width, h: cursor.height },
                    purpose: purpose_u8(purpose),
                })
            }
        };
        if ime != self.last_ime {
            self.last_ime = ime.clone();
            // wid = 0 哨兵（连接级寻址——headless 装配期不知真实 wid，
            // rqhost 客户端单 wid 连接按 client 记录路由，壳投影族同型）。
            let wid = 0;
            self.pending_controls.push(ControlMsg::ImeRequest { wid, enabled: ime });
        }
        let kind = cursor_kind_u8(mouse_interaction);
        if kind != self.last_cursor_kind {
            self.last_cursor_kind = kind;
            let wid = 0;
            self.pending_controls.push(ControlMsg::SetCursor { wid, kind });
        }
    }

    /// PLAN-690：App 派生窗口控制下行批读走（ImeRequest/SetCursor——
    /// 泵在产帧后消费并上行 `ProtocolMsg::Control`）。
    pub fn drain_window_controls(&mut self) -> Vec<ControlMsg> {
        std::mem::take(&mut self.pending_controls)
    }

    /// PLAN-690 T-03：最近一帧 text_input 上报的组合串（环测断言口，
    /// 见 `preedit_reported` 字段注）。
    pub fn preedit_reported(&self) -> Option<&str> {
        self.preedit_reported.as_deref()
    }

    /// 上帧降格观测（未入 wire 的原语——T-01 逐项消解的台账）。
    pub fn observations(&self) -> &[String] {
        &self.observations
    }

    /// v2 帧位图排水（Rgba/Bytes 句柄 → bitmap:// 上传；泵在
    /// `drain_bitmap_uploads` 缝消费——v1 投影器同型）。
    pub fn drain_bitmap_uploads(&mut self) -> Vec<super::endpoint::BitmapUpload> {
        std::mem::take(&mut self.pending_bitmaps)
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
            buffer_runs(paragraph.buffer(), *position, *color, translation, &mut |x, y, size, lh, rgba, weight, text| {
                ops.push(DrawOp::TextStyled {
                    x, y, size, line_height: lh, color: rgba, weight, italic: false, text,
                });
            });
        }
        GfxText::Editor { editor, position, color, .. } => {
            let Some(editor) = editor.upgrade() else { return };
            buffer_runs(editor.buffer(), *position, *color, translation, &mut |x, y, size, lh, rgba, weight, text| {
                ops.push(DrawOp::TextStyled {
                    x, y, size, line_height: lh, color: rgba, weight, italic: false, text,
                });
            });
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

/// cosmic Buffer 逐行提取（v1/v2 降格共用——run 顶锚 + 行首字形 x +
/// 首字形字号/字重；emit 回调落各自的 op 词汇）。
fn buffer_runs(
    buffer: &cosmic_text::Buffer,
    position: iced::Point,
    color: iced::Color,
    translation: iced::Vector,
    mut emit: impl FnMut(f32, f32, f32, f32, Rgba8, u16, String),
) {
    let rgba = to_rgba8(color);
    for run in buffer.layout_runs() {
        let Some(first) = run.glyphs.first() else { continue };
        let line_h = if run.line_height > 0.0 {
            run.line_height
        } else {
            first.font_size * 1.3
        };
        emit(
            position.x + translation.x + first.x,
            position.y + translation.y + run.line_top,
            first.font_size,
            line_h,
            rgba,
            first.font_weight.0,
            run.text.to_string(),
        );
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

// ---------------------------------------------------------------------------
// 事件回路径（T-02）：wire InputMsg → iced::Event 注入
// ---------------------------------------------------------------------------

/// wire 修饰位（bit0 shift / bit1 ctrl / bit2 alt / bit3 logo——
/// session.rs `wire_mods` 同表反向）→ iced Modifiers。
fn wire_mods_to_iced(bits: u8) -> iced::keyboard::Modifiers {
    use iced::keyboard::Modifiers;
    let mut m = Modifiers::empty();
    if bits & 0b0001 != 0 {
        m |= Modifiers::SHIFT;
    }
    if bits & 0b0010 != 0 {
        m |= Modifiers::CTRL;
    }
    if bits & 0b0100 != 0 {
        m |= Modifiers::ALT;
    }
    if bits & 0b1000 != 0 {
        m |= Modifiers::LOGO;
    }
    m
}

/// Windows VK → iced Named/Code（宿主 LiveInput 语义：Named 键映射；字母
/// 数字走 Character 由 CharTyped 承担——此处只收控制面键）。
fn vk_to_named(vk: u32) -> Option<iced::keyboard::key::Named> {
    use iced::keyboard::key::Named;
    Some(match vk {
        8 => Named::Backspace,
        9 => Named::Tab,
        13 => Named::Enter,
        27 => Named::Escape,
        32 => Named::Space,
        33 => Named::PageUp,
        34 => Named::PageDown,
        35 => Named::End,
        36 => Named::Home,
        37 => Named::ArrowLeft,
        38 => Named::ArrowUp,
        39 => Named::ArrowRight,
        40 => Named::ArrowDown,
        45 => Named::Insert,
        46 => Named::Delete,
        _ => return None,
    })
}

/// Windows VK → iced Physical Code（字母/数字——物理位映射）。
fn vk_to_code(vk: u32) -> Option<iced::keyboard::key::Code> {
    use iced::keyboard::key::Code;
    Some(match vk {
        0x30..=0x39 => Code::Digit0,
        0x41 => Code::KeyA,
        0x42 => Code::KeyB,
        0x43 => Code::KeyC,
        0x44 => Code::KeyD,
        0x45 => Code::KeyE,
        0x46 => Code::KeyF,
        0x47 => Code::KeyG,
        0x48 => Code::KeyH,
        0x49 => Code::KeyI,
        0x4A => Code::KeyJ,
        0x4B => Code::KeyK,
        0x4C => Code::KeyL,
        0x4D => Code::KeyM,
        0x4E => Code::KeyN,
        0x4F => Code::KeyO,
        0x50 => Code::KeyP,
        0x51 => Code::KeyQ,
        0x52 => Code::KeyR,
        0x53 => Code::KeyS,
        0x54 => Code::KeyT,
        0x55 => Code::KeyU,
        0x56 => Code::KeyV,
        0x57 => Code::KeyW,
        0x58 => Code::KeyX,
        0x59 => Code::KeyY,
        0x5A => Code::KeyZ,
        _ => return None,
    })
}

/// wire InputMsg → iced::Event（远程事件回路径——命中/聚焦/IME 全由
/// iced widget 树原生处理）。IME 三态映射：Preedit/Commit →
/// `input_method::Event`（text_input 组合态原生消费）；Cancelled →
/// Closed（组合态清理）。Windows 中文 IME 实机完备性 = T-02 风险面
///（待澄清③），残缺则混合方案兜底登记。
pub(crate) fn input_to_iced_events(input: &crate::ui::desktop_protocol::message::InputMsg) -> Vec<iced::Event> {
    use crate::ui::desktop_protocol::message::InputMsg;
    use iced::keyboard::{Event as KeyEvent, Key};
    use iced::mouse;
    match input {
        InputMsg::PointerMoved { x, y, .. } => vec![iced::Event::Mouse(
            mouse::Event::CursorMoved { position: iced::Point::new(*x, *y) },
        )],
        InputMsg::PointerPressed { button, x, y, modifiers, .. } => {
            vec![
                iced::Event::Mouse(mouse::Event::CursorMoved { position: iced::Point::new(*x, *y) }),
                iced::Event::Mouse(mouse::Event::ButtonPressed(wire_button(*button))),
                // 修饰键态同步（iced 内部跟踪依赖事件流——Ctrl+C 等组合
                // 键的正确派发需要按下时修饰在场）。
                iced::Event::Keyboard(KeyEvent::ModifiersChanged(wire_mods_to_iced(*modifiers))),
            ]
        }
        InputMsg::PointerReleased { button, x, y, modifiers, .. } => {
            vec![
                iced::Event::Mouse(mouse::Event::CursorMoved { position: iced::Point::new(*x, *y) }),
                iced::Event::Mouse(mouse::Event::ButtonReleased(wire_button(*button))),
                iced::Event::Keyboard(KeyEvent::ModifiersChanged(wire_mods_to_iced(*modifiers))),
            ]
        }
        InputMsg::KeyPressed { key, modifiers, .. } => {
            let mods = wire_mods_to_iced(*modifiers);
            let Some(named) = vk_to_named(*key) else {
                return Vec::new();
            };
            vec![iced::Event::Keyboard(KeyEvent::KeyPressed {
                key: Key::Named(named),
                modified_key: Key::Named(named),
                physical_key: vk_to_code(*key)
                    .map(iced::keyboard::key::Physical::Code)
                    .unwrap_or(iced::keyboard::key::Physical::Unidentified(
                        iced::keyboard::key::NativeCode::Unidentified,
                    )),
                location: iced::keyboard::Location::Standard,
                modifiers: mods,
                text: None,
                repeat: false,
            })]
        }
        InputMsg::KeyReleased { key, modifiers, .. } => {
            let mods = wire_mods_to_iced(*modifiers);
            let Some(named) = vk_to_named(*key) else {
                return Vec::new();
            };
            vec![iced::Event::Keyboard(KeyEvent::KeyReleased {
                key: Key::Named(named),
                modified_key: Key::Named(named),
                physical_key: vk_to_code(*key)
                    .map(iced::keyboard::key::Physical::Code)
                    .unwrap_or(iced::keyboard::key::Physical::Unidentified(
                        iced::keyboard::key::NativeCode::Unidentified,
                    )),
                location: iced::keyboard::Location::Standard,
                modifiers: mods,
            })]
        }
        InputMsg::CharTyped { ch, .. } => {
            let text: String = ch.to_string();
            let key = Key::Character(iced_widget::core::SmolStr::new(&text));
            vec![iced::Event::Keyboard(KeyEvent::KeyPressed {
                key: key.clone(),
                modified_key: key,
                physical_key: iced::keyboard::key::Physical::Unidentified(
                    iced::keyboard::key::NativeCode::Unidentified,
                ),
                location: iced::keyboard::Location::Standard,
                modifiers: iced::keyboard::Modifiers::empty(),
                text: Some(iced_widget::core::SmolStr::new(&text)),
                repeat: false,
            })]
        }
        InputMsg::Scroll { dx, dy, .. } => vec![iced::Event::Mouse(
            mouse::Event::WheelScrolled {
                delta: iced::mouse::ScrollDelta::Pixels { x: *dx, y: *dy },
            },
        )],
        InputMsg::ImePreedit { text, selection, .. } => vec![iced::Event::InputMethod(
            iced::advanced::input_method::Event::Preedit(
                text.clone(),
                selection.map(|(s, e)| s as usize..e as usize),
            ),
        )],
        InputMsg::ImeCommit { text, .. } => vec![iced::Event::InputMethod(
            iced::advanced::input_method::Event::Commit(text.clone()),
        )],
        InputMsg::ImeCancelled { .. } => vec![iced::Event::InputMethod(
            iced::advanced::input_method::Event::Closed,
        )],
    }
}

/// iced IME purpose → wire u8（0 Normal / 1 Secure / 2 Terminal——
/// `ControlMsg::ImeRequest` 载荷语义，透传留协议面）。
fn purpose_u8(purpose: Purpose) -> u8 {
    match purpose {
        Purpose::Normal => 0,
        Purpose::Secure => 1,
        Purpose::Terminal => 2,
    }
}

/// iced 鼠标 Interaction → wire 光标形状 u8（PLAN-690 v1 两级：pointer/
/// text 显式，其余 default——iced ~27 变体不逐个过线，SD-02 契约注记）。
fn cursor_kind_u8(interaction: mouse::Interaction) -> u8 {
    match interaction {
        mouse::Interaction::Pointer => 1,
        mouse::Interaction::Text => 2,
        _ => 0,
    }
}

/// wire u8 → iced 光标图标（daemon view mouse_area 消费侧同表反向；
/// Idle 在 iced_winit conversion 映射 CursorIcon::Default——0 档即箭头）。
pub(crate) fn cursor_kind_icon(kind: u8) -> mouse::Interaction {
    match kind {
        1 => mouse::Interaction::Pointer,
        2 => mouse::Interaction::Text,
        _ => mouse::Interaction::Idle,
    }
}

fn wire_button(button: crate::ui::desktop_protocol::message::MouseButton) -> iced::mouse::Button {
    match button {
        crate::ui::desktop_protocol::message::MouseButton::Left => iced::mouse::Button::Left,
        crate::ui::desktop_protocol::message::MouseButton::Right => iced::mouse::Button::Right,
        crate::ui::desktop_protocol::message::MouseButton::Middle => iced::mouse::Button::Middle,
    }
}

// ---------------------------------------------------------------------------
// HeadlessFrameSource（T-02）：FrameSource 接入——ClientPump 直驱
// ---------------------------------------------------------------------------

/// remote 模式帧源（PLAN-683）：HeadlessSurface 的 FrameSource 适配——
/// 事件回路径（InputMsg → iced::Event → widget 树 → 消息 → 组件）+
/// revision 门控（消息产出/tick 到期 → 前进 → 泵侧对账产帧）+ v2 产线帧。
pub struct HeadlessFrameSource<C: Component> {
    surface: HeadlessSurface<C>,
    rev: u64,
    tick_last: Option<std::time::Instant>,
    observations: Vec<String>,
}

impl<C: Component> HeadlessFrameSource<C>
where
    C::Msg: Clone + std::fmt::Debug + 'static,
{
    pub fn new(component: C, width: f32, height: f32) -> Self {
        Self {
            surface: HeadlessSurface::new(component, width, height),
            rev: 0,
            tick_last: None,
            observations: Vec::new(),
        }
    }

    /// 组件只读访问（调用侧装载/观测）。
    pub fn component(&self) -> &C {
        &self.surface.component
    }
}

impl<C: Component> super::endpoint::FrameSource for HeadlessFrameSource<C>
where
    C::Msg: Clone + std::fmt::Debug + 'static,
{
    fn revision(&self) -> u64 {
        self.rev
    }

    fn render_frame(&mut self) -> DrawList {
        // v1 面（legacy/parity）：泵 v2 位开着时本方法不为主产线。
        let list = self.surface.render_frame();
        self.observations = self.surface.observations().to_vec();
        list
    }

    fn render_frame_v2(&mut self) -> DisplayList {
        let list = self.surface.render_frame_v2();
        self.observations = self.surface.observations().to_vec();
        list
    }

    fn on_input(&mut self, input: &crate::ui::desktop_protocol::message::InputMsg) {
        // 光标位同步（hover 态渲染依赖——iced Cursor::Available）。
        // PLAN-694：press/release 同步（携带 x,y——任何位置性输入都蕴含
        // 光标在位；桌面会话注入面无 PointerMoved 生产（hover 不上线），
        // 协议级点击（p694 环测）据此可聚焦 iced text_input——物理鼠标
        // 场景 move 流恒先到，行为不变）。
        match input {
            crate::ui::desktop_protocol::message::InputMsg::PointerMoved { x, y, .. }
            | crate::ui::desktop_protocol::message::InputMsg::PointerPressed { x, y, .. }
            | crate::ui::desktop_protocol::message::InputMsg::PointerReleased { x, y, .. } => {
                self.surface.point_at(Some(iced::Point::new(*x, *y)));
            }
            _ => {}
        }
        let events = input_to_iced_events(input);
        if events.is_empty() {
            return;
        }
        let messages = self.surface.update(&events);
        // 消息产出 = 组件状态可能变化 → revision 前进（泵对账产帧）。
        if !messages.is_empty() {
            self.rev += 1;
        }
    }

    fn on_control(&mut self, _control: &crate::ui::desktop_protocol::message::ControlMsg) {}

    fn poll_tick(&mut self) {
        // 双 tick 源（RqProjector 同构）：timesources 到期派发 + 单通道
        // interval tick。
        if self.surface.component.fire_due_timers() {
            self.rev += 1;
        }
        let Some(interval) = self.surface.component.tick_interval_ms() else {
            return;
        };
        let now = std::time::Instant::now();
        match self.tick_last {
            None => self.tick_last = Some(now),
            Some(last) => {
                if now.duration_since(last) >= std::time::Duration::from_millis(u64::from(interval))
                {
                    self.tick_last = Some(now);
                    if let Some(msg) = self.surface.component.tick_msg() {
                        self.surface.component.on(msg);
                        self.rev += 1;
                    }
                }
            }
        }
    }

    fn drain_desktop_commands(&mut self) -> Vec<String> {
        self.surface.component.drain_desktop_commands()
    }

    fn drain_bitmap_uploads(&mut self) -> Vec<super::endpoint::BitmapUpload> {
        self.surface.drain_bitmap_uploads()
    }

    fn drain_window_controls(&mut self) -> Vec<crate::ui::desktop_protocol::message::ControlMsg> {
        self.surface.drain_window_controls()
    }
}

// ---------------------------------------------------------------------------
// Layer 记录 → DisplayList v2（native 原语——remote 模式产线降格）
// ---------------------------------------------------------------------------

/// v2 降格（PLAN-683 SD-01）：quad 全参原生（渐变 stop/四角半径/border/
/// shadow）；文本 run 承接非恒等变换（Transform push/pop 包络——scale+
/// translate 近似，旋转不可达见观测）；图像句柄 → src 词汇或 bitmap://
/// 上传通道。mesh（canvas/vector）= v2 词汇面外（G2 词汇表未列），位图
/// 降级承接为登记债（试点三例无 canvas 面）。
fn lower_layers_v2(
    layers: &[Layer],
    observations: &mut Vec<String>,
    bitmaps: &mut Vec<super::endpoint::BitmapUpload>,
    bitmap_seq: &mut usize,
) -> DisplayList {
    let mut ops = Vec::new();
    for (index, layer) in layers.iter().enumerate() {
        let clipped = index > 0;
        if clipped {
            ops.push(DisplayOp::Scissor { rect: to_wrect(layer.bounds) });
        }
        for (quad, background) in &layer.quads {
            lower_quad_v2(quad, background, &mut ops);
        }
        for item in &layer.text {
            lower_text_v2(item.as_slice(), item.transformation(), &mut ops, observations);
        }
        for image in &layer.images {
            lower_image_v2(image, &mut ops, observations, bitmaps, bitmap_seq);
        }
        if !layer.primitives.is_empty() {
            observations.push(format!(
                "mesh primitive x{}（canvas/vector 面——G2 词汇外，位图降级=登记债 P683-D1）",
                layer.primitives.len()
            ));
        }
        if clipped {
            ops.push(DisplayOp::ScissorPop);
        }
    }
    DisplayList { clear: Some(super::client_runtime::bg()), ops }
}

/// v2 quad：原生全参——Fill（色/渐变 angle+stops）、四角半径、border、
/// shadow 直落 wire（v1 近似面全部消除）。
fn lower_quad_v2(
    quad: &advanced_renderer::Quad,
    background: &iced::Background,
    ops: &mut Vec<DisplayOp>,
) {
    let rect = to_wrect(quad.bounds);
    let radius = [
        quad.border.radius.top_left,
        quad.border.radius.top_right,
        quad.border.radius.bottom_right,
        quad.border.radius.bottom_left,
    ];
    let fill = match background {
        iced::Background::Color(color) => Fill::Color(to_rgba8(*color)),
        iced::Background::Gradient(gradient) => match gradient {
            iced::gradient::Gradient::Linear(linear) => Fill::LinearGradient {
                angle: linear.angle.0,
                stops: linear
                    .stops
                    .iter()
                    .flatten()
                    .map(|stop| (stop.offset, to_rgba8(stop.color)))
                    .collect(),
            },
        },
    };
    let border = (quad.border.width > 0.0).then(|| BorderSpec {
        color: to_rgba8(quad.border.color),
        width: quad.border.width,
    });
    let shadow = (quad.shadow.color.a > 0.0
        && (quad.shadow.blur_radius > 0.0
            || quad.shadow.offset.x != 0.0
            || quad.shadow.offset.y != 0.0))
        .then(|| ShadowSpec {
            color: to_rgba8(quad.shadow.color),
            offset: (quad.shadow.offset.x, quad.shadow.offset.y),
            blur: quad.shadow.blur_radius,
        });
    ops.push(DisplayOp::Quad { rect, fill, radius, border, shadow });
}

/// v2 文本：项级非恒等变换 → Transform 包络（scale+translate 近似——
/// iced Transformation 未公开矩阵元，旋转/剪切不可达，观测留痕）；run
/// 几何在变换局部坐标直落。
fn lower_text_v2(
    texts: &[GfxText],
    transformation: iced::Transformation,
    ops: &mut Vec<DisplayOp>,
    observations: &mut Vec<String>,
) {
    let wrapped = transformation != iced::Transformation::IDENTITY;
    if wrapped {
        let translation = transformation.translation();
        let scale = transformation.scale_factor();
        ops.push(DisplayOp::Transform {
            matrix: [scale, 0.0, 0.0, scale, translation.x, translation.y],
        });
        if (scale - 1.0).abs() > f32::EPSILON {
            observations.push(
                "transform scale 近似（旋转/剪切矩阵元 iced 未公开——P683-D2）".to_string(),
            );
        }
    }
    for text in texts {
        match text {
            GfxText::Paragraph { paragraph, position, color, .. } => {
                if let Some(paragraph) = paragraph.upgrade() {
                    buffer_runs(
                        paragraph.buffer(),
                        *position,
                        *color,
                        iced::Vector::default(),
                        &mut |x, y, size, lh, rgba, weight, text| {
                            ops.push(DisplayOp::TextStyled {
                                x,
                                y,
                                size,
                                line_height: lh,
                                color: rgba,
                                weight,
                                italic: false,
                                text,
                            });
                        },
                    );
                }
            }
            GfxText::Editor { editor, position, color, .. } => {
                if let Some(editor) = editor.upgrade() {
                    buffer_runs(
                        editor.buffer(),
                        *position,
                        *color,
                        iced::Vector::default(),
                        &mut |x, y, size, lh, rgba, weight, text| {
                            ops.push(DisplayOp::TextStyled {
                                x,
                                y,
                                size,
                                line_height: lh,
                                color: rgba,
                                weight,
                                italic: false,
                                text,
                            });
                        },
                    );
                }
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
                ops.push(DisplayOp::TextStyled {
                    x,
                    y: bounds.y,
                    size: size.0,
                    line_height: line_height.0,
                    color: to_rgba8(*color),
                    weight: iced_weight_u16(font.weight),
                    italic: matches!(
                        font.style,
                        iced::font::Style::Italic | iced::font::Style::Oblique
                    ),
                    text: content.clone(),
                });
            }
            GfxText::Raw { .. } => {
                observations.push(
                    "raw text buffer 省略（编辑器组合态——T-02 IME 面承接）".to_string(),
                );
            }
        }
    }
    if wrapped {
        ops.push(DisplayOp::TransformPop);
    }
}

/// v2 图像：句柄 → src 词汇（Path 直通）/ bitmap:// 上传（Rgba 直取；
/// Bytes 解码）。呈现参数（rotation/opacity/filter）非缺省 → 观测行
///（v2 Image op 保持 v1 形态——呈现参数追加留后续 tag）。
fn lower_image_v2(
    image: &iced::advanced::graphics::image::Image,
    ops: &mut Vec<DisplayOp>,
    observations: &mut Vec<String>,
    bitmaps: &mut Vec<super::endpoint::BitmapUpload>,
    bitmap_seq: &mut usize,
) {
    use iced::advanced::image::Handle;

    let (handle, bounds, rotation, opacity) = match image {
        iced::advanced::graphics::image::Image::Raster { image, bounds, .. } => (
            &image.handle,
            *bounds,
            image.rotation,
            image.opacity,
        ),
        iced::advanced::graphics::image::Image::Vector { bounds, .. } => {
            observations.push(format!(
                "svg 图像降格省略（{bounds:?}——svg 通道=登记债 P683-D3）"
            ));
            return;
        }
    };
    let rect = to_wrect(bounds);
    let src = match handle {
        Handle::Path(_, path) => path.to_string_lossy().into_owned(),
        Handle::Rgba { width, height, pixels, .. } => {
            *bitmap_seq += 1;
            let local = format!("p683-img-{bitmap_seq}");
            bitmaps.push(super::endpoint::BitmapUpload {
                id: super::endpoint::bitmap_local_id(&local),
                w: *width,
                h: *height,
                stride: width * 4,
                rgba: pixels.to_vec(),
            });
            super::endpoint::bitmap_src(&local)
        }
        Handle::Bytes(_, bytes) => match decode_image_rgba(&bytes[..]) {
            Some((w, h, rgba)) => {
                *bitmap_seq += 1;
                let local = format!("p683-img-{bitmap_seq}");
                bitmaps.push(super::endpoint::BitmapUpload {
                    id: super::endpoint::bitmap_local_id(&local),
                    w,
                    h,
                    stride: w * 4,
                    rgba,
                });
                super::endpoint::bitmap_src(&local)
            }
            None => {
                observations.push("Bytes 图像解码失败（观测省略）".to_string());
                return;
            }
        },
    };
    if rotation.0 != 0.0 || (opacity - 1.0).abs() > f32::EPSILON {
        observations.push(format!(
            "image 呈现参数（rotation/opacity）非缺省观测省略（{src}）"
        ));
    }
    ops.push(DisplayOp::Image {
        rect,
        src,
        fit: crate::ui::desktop_protocol::message::ImageFit::Stretch,
    });
}

/// 编码图像字节 → RGBA（Bytes 句柄解码——repo image 管线同源）。
fn decode_image_rgba(bytes: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    let img = image::load_from_memory(bytes).ok()?;
    let rgba = img.to_rgba8();
    let (w, h) = (rgba.width(), rgba.height());
    Some((w, h, rgba.into_raw()))
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

    /// PLAN-683 T-01（AC-04）—— v2 产线帧：headless 降格产 DisplayList v2
    /// 全原语（001/003 载体）→ codec round-trip 恒等；v1 降格面与 v2 直搬
    /// 面（from_v1）语义等价（quad/text 同几何同色）。
    #[test]
    fn p683_t01_v2_first_frame_round_trip() {
        for dir in ["001-helloworld", "003-converter"] {
            let Some(src) = example_source(dir) else {
                eprintln!("[p683] skip: {dir} 载体缺席");
                return;
            };
            let component =
                crate::build_dynamic_component(&src, None).unwrap_or_else(|e| panic!("{dir}: {e}"));
            let mut surface = HeadlessSurface::new(component, 480.0, 360.0);

            let v1 = surface.render_frame();
            let lifted = DisplayList::from_v1(&v1);
            let v2 = surface.render_frame_v2();

            // v2 非空 + 文本在档。
            assert!(!v2.ops.is_empty(), "{dir} v2 空帧");
            assert!(
                v2.ops.iter().any(|op| matches!(op, DisplayOp::TextStyled { .. })),
                "{dir} v2 无文本"
            );

            // round-trip 恒等。
            let mut buf = Vec::new();
            v2.encode(&mut buf);
            assert_eq!(buf[0], 2, "{dir} v2 载荷 tag");
            let mut reader = crate::ui::desktop_protocol::codec::Reader::new(&buf);
            assert_eq!(DisplayList::decode(&mut reader).unwrap(), v2, "{dir} v2 round-trip 漂移");

            // v1 降格 vs v2 原生：文本 run 集合等价（同组件同驱动的
            // 两条降格路径文本面一致——run 提取单源）。
            let texts_of = |list: &DisplayList| -> Vec<(f32, f32, String)> {
                list.ops
                    .iter()
                    .filter_map(|op| match op {
                        DisplayOp::TextStyled { x, y, text, .. } => Some((*x, *y, text.clone())),
                        _ => None,
                    })
                    .collect()
            };
            assert_eq!(texts_of(&lifted), texts_of(&v2), "{dir} 文本面 v1/v2 不一致");

            // 消息信封 round-trip（FrameReadyV2）。
            let msg = crate::ui::desktop_protocol::message::ProtocolMsg::Frame(
                crate::ui::desktop_protocol::message::FrameMsg::FrameReadyV2 {
                    wid: 1,
                    frame_id: 2,
                    slot: 0,
                    damage: None,
                    revision: 1,
                    payload: v2.clone(),
                },
            );
            let bytes = msg.encode();
            assert_eq!(
                crate::ui::desktop_protocol::message::ProtocolMsg::decode(&bytes).unwrap(),
                msg,
                "{dir} FrameReadyV2 信封 round-trip 漂移"
            );
        }
    }

    /// PLAN-690 T-02/T-03/T-05 —— IME 截获环（headless 单元级）：
    /// 点击聚焦 → `ImeRequest(Enabled{cursor})` + hover 光标 `SetCursor(2)`
    /// 下行批；Preedit（含字节选区）注入 → text_input 收态 → 下一帧经
    /// `Enabled.preedit` 原样上报（`preedit_reported` 断言口）→ Commit
    /// 入值出帧 → Esc 取消清组合态。
    #[test]
    fn p690_ime_capture_and_preedit_round_trip() {
        use crate::ui::desktop_protocol::message::{InputMsg, MouseButton};

        let Some(src) = example_source("003-converter") else {
            eprintln!("[p690] skip: 003-converter 载体缺席");
            return;
        };
        let component =
            crate::build_dynamic_component(&src, None).unwrap_or_else(|e| panic!("{e}"));
        let mut surface = HeadlessSurface::new(component, 480.0, 360.0);

        // 首帧 + celsius 输入定位（p683 typing 环同锚：值文本 `0`）。
        let first = surface.render_frame();
        let (fx, fy) = first
            .ops
            .iter()
            .find_map(|op| match op {
                DrawOp::TextStyled { x, y, text, .. } if text == "0" => Some((*x, *y)),
                _ => None,
            })
            .expect("celsius 值文本 `0` 缺席");
        let (cx, cy) = (fx + 4.0, fy + 8.0);

        // hover 同点（光标形状断言前置——text_input hover = Text）。
        surface.point_at(Some(iced::Point::new(cx, cy)));
        // 点击聚焦（Pressed + Released 完整对——iced text_input 点击聚焦）。
        let press = input_to_iced_events(&InputMsg::PointerPressed {
            wid: 0,
            button: MouseButton::Left,
            x: cx,
            y: cy,
            modifiers: 0,
        });
        let release = input_to_iced_events(&InputMsg::PointerReleased {
            wid: 0,
            button: MouseButton::Left,
            x: cx,
            y: cy,
            modifiers: 0,
        });
        surface.update(&press);
        surface.update(&release);

        // 下一帧：RedrawRequested 臂 → text_input 请求 IME（聚焦态）——
        // 截获出 ImeRequest(Enabled) + SetCursor(Text)。
        let _ = surface.render_frame();
        let controls = surface.drain_window_controls();
        assert!(
            controls.iter().any(|c| matches!(
                c,
                ControlMsg::ImeRequest { enabled: Some(req), .. }
                    if req.cursor.w > 0.0 && req.cursor.h > 0.0
            )),
            "聚焦后 IME enable 下行缺席: {controls:?}"
        );
        assert!(
            controls.iter().any(|c| matches!(c, ControlMsg::SetCursor { kind: 2, .. })),
            "text_input hover 光标（Text）下行缺席: {controls:?}"
        );

        // Preedit 注入（组合串 + 字节选区）→ 下一帧 text_input 经
        // Enabled.preedit 原样上报。
        let preedit = input_to_iced_events(&InputMsg::ImePreedit {
            wid: 0,
            text: "nihao".into(),
            selection: Some((2, 4)),
        });
        surface.update(&preedit);
        let _ = surface.render_frame();
        assert_eq!(
            surface.preedit_reported(),
            Some("nihao"),
            "组合串未达 text_input 态（Enabled.preedit 上报缺席）"
        );
        let re_drain = surface.drain_window_controls();
        assert!(
            re_drain.is_empty(),
            "组合态帧不重发 IME 请求（去重基线失效）: {re_drain:?}"
        );

        // Commit 入值：winit Windows commit 序 = `Preedit("")` 清态先行 +
        // `Commit(text)`（event_loop.rs:1546-1556）——镜像真实事件序。
        let clear = input_to_iced_events(&InputMsg::ImePreedit {
            wid: 0,
            text: String::new(),
            selection: None,
        });
        surface.update(&clear);
        let commit = input_to_iced_events(&InputMsg::ImeCommit { wid: 0, text: "你好".into() });
        surface.update(&commit);
        let committed = surface.render_frame();
        assert!(
            committed.ops.iter().any(|op| matches!(
                op,
                DrawOp::TextStyled { text, .. } | DrawOp::Text { text, .. } if text.contains("你好")
            )),
            "Commit 后帧文本不含上屏串"
        );
        assert_eq!(
            surface.preedit_reported(),
            None,
            "Commit 后组合态未清（preedit 上报残留）"
        );

        // Esc 取消（ImeCancelled → Closed 清组合态——G1 取消腿）。
        let cancelled = input_to_iced_events(&InputMsg::ImeCancelled { wid: 0 });
        surface.update(&cancelled);
        let _ = surface.render_frame();
        assert_eq!(surface.preedit_reported(), None, "取消后组合态残留");
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

    /// PLAN-683 T-03——v2 原语 canvas 重放：001/003 实帧（含渐变/圆角/
    /// border/shadow 全参 quad）经 `()` 后端完整走 paint_ops_v2 降格
    /// 路径（词汇面兼容/路径构造炸点/嵌套 scissor-transform 扫描）。
    #[test]
    fn p683_t03_v2_paint_replay() {
        for dir in ["001-helloworld", "003-converter"] {
            let Some(src) = example_source(dir) else {
                eprintln!("[p683] skip: {dir} 载体缺席");
                return;
            };
            let component =
                crate::build_dynamic_component(&src, None).unwrap_or_else(|e| panic!("{dir}: {e}"));
            let mut surface = HeadlessSurface::new(component, 480.0, 360.0);
            let v2 = surface.render_frame_v2();
            assert!(!v2.ops.is_empty(), "{dir} v2 空帧");
            if cfg!(debug_assertions) {
                let bounds =
                    iced::Rectangle::new(iced::Point::ORIGIN, iced::Size::new(640.0, 480.0));
                let mut frame = iced::widget::canvas::Frame::with_bounds(&(), bounds);
                if let Some(clear) = v2.clear {
                    frame.fill_rectangle(
                        iced::Point::ORIGIN,
                        bounds.size(),
                        iced::Color::from_rgba8(clear.r, clear.g, clear.b, 1.0),
                    );
                }
                crate::ui::iced::broker_surface::paint_ops_v2(&mut frame, &v2.ops);
            }
        }
    }
}
