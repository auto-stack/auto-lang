//! Plan 481: SelectableText —— `iced::widget::text` 的可选/可复制变体。
//!
//! 架构（T3 spike 定案,见 plan §架构方案）:
//! - **绘制路径与 `text` 逐像素一致**:layout/draw 直接复用
//!   `iced::advanced::widget::text::{layout, draw}`(Plain paragraph 同参
//!   测量/绘制),高亮 quad 先画、文本后画(文本在上)。
//! - **命中测试走 `Paragraph::buffer()`**:`iced::advanced::graphics::text::
//!   Paragraph`(iced_graphics)公开 `.buffer() -> &cosmic_text::Buffer`,无需
//!   自持 Buffer——hit(point) 得 (逻辑行, 行内字节偏移),经 line_starts 表
//!   换算全局字节偏移,喂给 `super::selection` 状态机;选区矩形按
//!   `layout_runs()` 逐 run 钳制 + `index_x` glyph 步进累积
//!   (code_editor core/render.rs §选区 quads 同型先例)。
//! - 选区为 widget 本地状态(G2):不进 DesktopSession/WmState。
//!
//! v1 手势集(步骤 5 接线):拖选、双击选词、Ctrl+C 复制、Esc/单击清除。

use iced::advanced::graphics::text::Paragraph as GraphicsParagraph;
use iced::advanced::text::paragraph::{Paragraph as _, Plain};
use iced::advanced::widget::text::{self as text_widget, Format};
use iced::advanced::widget::tree::{self, Tag, Tree};
use iced::advanced::{Clipboard, Layout, Renderer as _, Shell, Widget};
use iced::event::Event;
use iced::keyboard::{self, key, Key};
use iced::mouse;
use iced::{
    Color, Element, Font, Length, Pixels, Point, Rectangle, Size, Theme,
};
use iced::alignment::{self};
use iced::advanced::text::{Alignment, LineHeight, Shaping, Wrapping};

use super::selection::Selection;

/// 双击判定窗口(与主流编辑器一致的 500ms)。
const DOUBLE_CLICK_WINDOW: std::time::Duration =
    std::time::Duration::from_millis(500);
/// 双击判定位移阈值(逻辑像素):超过视为重新按下。
const DOUBLE_CLICK_SLOP: f32 = 4.0;

/// 可选文本 widget。参数面与 `iced::widget::Text` 对齐(渲染面子集:
/// size/font/color/width/height/align/line_height/shaping/wrapping)。
#[derive(Debug, Clone)]
pub struct SelectableText {
    content: String,
    format: Format<Font>,
    color: Option<Color>,
}

impl SelectableText {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            format: Format::default(),
            color: None,
        }
    }

    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.format.size = Some(size.into());
        self
    }

    pub fn font(mut self, font: impl Into<Font>) -> Self {
        self.format.font = Some(font.into());
        self
    }

    pub fn color(mut self, color: impl Into<Color>) -> Self {
        self.color = Some(color.into());
        self
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.format.width = width.into();
        self
    }

    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.format.height = height.into();
        self
    }

    pub fn align_x(mut self, align: impl Into<Alignment>) -> Self {
        self.format.align_x = align.into();
        self
    }

    pub fn align_y(mut self, align: alignment::Vertical) -> Self {
        self.format.align_y = align;
        self
    }

    pub fn line_height(mut self, line_height: impl Into<LineHeight>) -> Self {
        self.format.line_height = line_height.into();
        self
    }

    pub fn shaping(mut self, shaping: Shaping) -> Self {
        self.format.shaping = shaping;
        self
    }

    pub fn wrapping(mut self, wrapping: Wrapping) -> Self {
        self.format.wrapping = wrapping;
        self
    }

    // -------------------------------------------------------------------
    // 手势集(v1):拖选 / 双击选词 / 单击清除 / Ctrl+C 复制 / Esc 清除。
    // 独立于 iced 事件流的纯处理函数(单测直接驱动),update 只做坐标
    // 换算与捕获状态上报。
    // -------------------------------------------------------------------

    /// 段落局部坐标命中 → 全局字节偏移。
    fn hit_at(state: &State, para_local: Point) -> usize {
        hit_global(state.paragraph.raw().buffer(), &state.line_starts, para_local)
    }

    /// 鼠标手势。`local` = 光标相对 widget 原点坐标(无可用时 None);
    /// `anchor` = 段落绘制锚点(local - anchor = 段落局部坐标);
    /// `bounds` = widget 界。返回是否捕获。
    fn handle_mouse(
        &self,
        state: &mut State,
        event: &mouse::Event,
        local: Option<Point>,
        anchor: Point,
        bounds: Rectangle,
    ) -> bool {
        match event {
            mouse::Event::ButtonPressed(mouse::Button::Left) => {
                let Some(p) = local else { return false };
                if !bounds.contains(p) {
                    return false;
                }
                let para = Point::new(p.x - anchor.x, p.y - anchor.y);
                let g = Self::hit_at(state, para).min(self.content.len());
                // 双击判定:窗口内 + 近位点。
                let now = std::time::Instant::now();
                let is_double = state
                    .last_click
                    .map(|(t, lp)| {
                        now.duration_since(t) <= DOUBLE_CLICK_WINDOW
                            && (lp.x - p.x).abs() < DOUBLE_CLICK_SLOP
                            && (lp.y - p.y).abs() < DOUBLE_CLICK_SLOP
                    })
                    .unwrap_or(false);
                if is_double {
                    state.selection.select_word(&self.content, g);
                } else {
                    // 按下即锚定(单击不拖 → anchor==head 自然清空)。
                    state.selection.anchor = g;
                    state.selection.head = g;
                }
                state.dragging = true;
                state.last_click = Some((now, p));
                true
            }
            // CursorMoved 的事件坐标即新位置(iced 的 cursor.position() 在
            // 事件分发时仍为移动前的旧值——单步跳变时选区会恒空)。
            mouse::Event::CursorMoved { position } => {
                if !state.dragging {
                    return false;
                }
                let p = Point::new(
                    position.x - bounds.x,
                    position.y - bounds.y,
                );
                let para = Point::new(p.x - anchor.x, p.y - anchor.y);
                let g = Self::hit_at(state, para).min(self.content.len());
                state.selection.extend_to(g);
                true
            }
            mouse::Event::ButtonReleased(mouse::Button::Left) => {
                if state.dragging {
                    state.dragging = false;
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// 键盘手势:Ctrl+C 复制(捕获);Esc 清选区(不捕获,不夺全局流)。
    fn handle_keyboard(
        &self,
        state: &mut State,
        event: &keyboard::Event,
        clipboard: &mut dyn Clipboard,
    ) -> bool {
        match event {
            keyboard::Event::KeyPressed {
                key: Key::Character(c),
                modifiers,
                ..
            } if c.as_str().eq_ignore_ascii_case("c")
                && modifiers.control()
                && !state.selection.is_empty() =>
            {
                clipboard.write(
                    iced::advanced::clipboard::Kind::Standard,
                    state.selection.selected_text(&self.content).to_owned(),
                );
                true
            }
            keyboard::Event::KeyPressed {
                key: Key::Named(key::Named::Escape),
                ..
            } if !state.selection.is_empty() => {
                state.selection.clear();
                false
            }
            _ => false,
        }
    }
}

/// Widget 本地状态(进 iced widget Tree,G2:不进桌面会话)。
#[derive(Debug)]
pub struct State {
    /// 绘制 + 命中共用的 Plain paragraph(与 `text` widget 同型同参)。
    pub paragraph: Plain<GraphicsParagraph>,
    /// 选区状态机(全局字节偏移)。
    pub selection: Selection,
    /// 逻辑行首的字节偏移表(cosmic 按 \n/\r\n/\r 切行、行文本不含结尾符)。
    pub line_starts: Vec<usize>,
    /// 上次 shaping 的内容(layout 时内容变化则重算 line_starts)。
    pub last_content: String,
    /// 拖选中(ButtonPressed 置位、ButtonReleased 复位)。
    pub dragging: bool,
    /// 上次单击的(时刻, widget 局部坐标)——双击判定。
    pub last_click: Option<(std::time::Instant, Point)>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            paragraph: Plain::default(),
            selection: Selection::new(),
            line_starts: vec![0],
            last_content: String::new(),
            dragging: false,
            last_click: None,
        }
    }
}

impl<'a, Message> Widget<Message, Theme, iced::Renderer> for SelectableText
where
    Message: 'a + Clone,
{
    fn tag(&self) -> Tag {
        Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.format.width,
            height: self.format.height,
        }
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        let state = tree.state.downcast_mut::<State>();
        if state.last_content != self.content {
            state.line_starts = line_starts(&self.content);
            state.last_content = self.content.clone();
        }
        text_widget::layout(
            &mut state.paragraph,
            renderer,
            limits,
            &self.content,
            self.format,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        _theme: &Theme,
        defaults: &iced::advanced::renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();
        let paragraph = state.paragraph.raw();
        let anchor = bounds.anchor(
            paragraph.min_bounds(),
            paragraph.align_x(),
            paragraph.align_y(),
        );

        // 1) 选区高亮(先画,文本覆盖其上)。
        if !state.selection.is_empty() {
            let color = selection_color();
            for rect in selection_rects(
                paragraph.buffer(),
                &state.selection,
                &state.line_starts,
                anchor,
            ) {
                renderer.fill_quad(
                    iced::advanced::renderer::Quad {
                        bounds: rect,
                        ..iced::advanced::renderer::Quad::default()
                    },
                    iced::Background::Color(color),
                );
            }
        }

        // 2) 文本本体 —— 与 `text` widget 完全同参同路径。
        text_widget::draw(
            renderer,
            defaults,
            bounds,
            paragraph,
            text_widget::Style { color: self.color },
            viewport,
        );
    }

    /// 步骤 5:手势集接线。鼠标事件在 widget 界内捕获(拖选/双击/单击
    /// 清除);Ctrl+C 有选区时捕获并写剪贴板;Esc 清选区但不捕获(不夺
    /// 全局 Esc 流,弹层/对话框语义不受影响)。
    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let state = tree.state.downcast_mut::<State>();

        let anchor = {
            let paragraph = state.paragraph.raw();
            bounds.anchor(
                paragraph.min_bounds(),
                paragraph.align_x(),
                paragraph.align_y(),
            )
        };

        let captured = match event {
            Event::Mouse(mouse_event) => {
                let local = cursor
                    .position()
                    .map(|p| Point::new(p.x - bounds.x, p.y - bounds.y));
                self.handle_mouse(state, mouse_event, local, anchor, bounds)
            }
            Event::Keyboard(kb_event) => {
                self.handle_keyboard(state, kb_event, clipboard)
            }
            _ => false,
        };
        if captured {
            shell.capture_event();
        }
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        if cursor.is_over(layout.bounds()) {
            mouse::Interaction::Text
        } else {
            mouse::Interaction::default()
        }
    }
}

impl<'a, Message> From<SelectableText> for Element<'a, Message, Theme, iced::Renderer>
where
    Message: 'a + Clone,
{
    fn from(widget: SelectableText) -> Self {
        Element::new(widget)
    }
}

// ---------------------------------------------------------------------------
// 纯逻辑:全局字节偏移 ↔ (逻辑行, 行内偏移) ↔ 选区矩形
// ---------------------------------------------------------------------------

/// 逻辑行首字节偏移表。与 cosmic `Buffer::set_text` 的 LineIter 切分一致:
/// 按 `\n` / `\r\n` / `\r` 分行,行文本不含结尾符;结尾符本身归属前行。
pub fn line_starts(text: &str) -> Vec<usize> {
    let mut starts = vec![0usize];
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\n' => {
                starts.push(i + 1);
                i += 1;
            }
            b'\r' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
                    starts.push(i + 2);
                    i += 2;
                } else {
                    starts.push(i + 1);
                    i += 1;
                }
            }
            _ => i += 1,
        }
    }
    starts
}

/// (逻辑行, 行内字节偏移) → 全局字节偏移。
pub fn global_offset(line_starts: &[usize], line: usize, index: usize) -> usize {
    line_starts
        .get(line)
        .map(|&s| s + index)
        .unwrap_or_else(|| line_starts.last().copied().unwrap_or(0))
}

/// 全局字节偏移 → (逻辑行, 行内字节偏移)。
pub fn line_index(line_starts: &[usize], global: usize) -> (usize, usize) {
    match line_starts.binary_search(&global) {
        Ok(line) => (line, 0),
        Err(next) => {
            let line = next.saturating_sub(1);
            (line, global - line_starts[line])
        }
    }
}

/// cosmic 光标命中 → 全局字节偏移(无命中/空文本 → 0)。
pub fn hit_global(
    buffer: &cosmic_text::Buffer,
    line_starts: &[usize],
    local: Point,
) -> usize {
    buffer
        .hit(local.x, local.y)
        .map(|c| global_offset(line_starts, c.line, c.index))
        .unwrap_or(0)
}

/// 字节索引 → run 内 x 偏移(glyph 步进累积,簇内插值;
/// code_editor core/render.rs `index_x` 同型)。
fn index_x(run: &cosmic_text::LayoutRun, index: usize) -> Option<f32> {
    let mut prev_end = 0.0f32;
    for glyph in run.glyphs {
        if index < glyph.start {
            return Some(prev_end);
        }
        if index <= glyph.end {
            let cluster = &run.text[glyph.start..glyph.end];
            let total = cluster.chars().count().max(1) as f32;
            let before =
                run.text[glyph.start..index.min(glyph.end)].chars().count() as f32;
            return Some(glyph.x + (glyph.w / total) * before);
        }
        prev_end = glyph.x + glyph.w;
    }
    Some(prev_end)
}

/// 选区矩形(widget 坐标;anchor = 段落绘制锚点)。逐 layout run 钳制
/// 起止(跨行/软换行正确;code_editor 选区 quads 同型)。
pub fn selection_rects(
    buffer: &cosmic_text::Buffer,
    sel: &Selection,
    starts: &[usize],
    anchor: Point,
) -> Vec<Rectangle> {
    let mut out = Vec::new();
    if sel.is_empty() {
        return out;
    }
    let (g_lo, g_hi) = {
        let r = sel.range();
        (r.start, r.end)
    };
    let (lo_line, lo_idx) = line_index(starts, g_lo);
    let (hi_line, hi_idx) = line_index(starts, g_hi);
    for run in buffer.layout_runs() {
        if run.line_i < lo_line || run.line_i > hi_line {
            continue;
        }
        let lo = if run.line_i == lo_line { lo_idx } else { 0 };
        let hi = if run.line_i == hi_line {
            hi_idx
        } else {
            run.text.len()
        };
        if hi <= lo {
            continue;
        }
        if let (Some(x0), Some(x1)) = (index_x(&run, lo), index_x(&run, hi)) {
            out.push(Rectangle::new(
                Point::new(anchor.x + x0.min(x1), anchor.y + run.line_top),
                Size::new((x1 - x0).abs().max(2.0), run.line_height),
            ));
        }
    }
    out
}

/// 选区高亮色:主题 accent @ 0.25(对齐 code_editor/text_editor 选区色板)。
fn selection_color() -> Color {
    let (dark, accent) = crate::ui::code_editor::theme::theme_source();
    let theme = if dark {
        crate::ui::code_editor::theme::CodeEditorTheme::dark(&accent)
    } else {
        crate::ui::code_editor::theme::CodeEditorTheme::light(&accent)
    };
    Color::from_rgba(
        theme.selection.r,
        theme.selection.g,
        theme.selection.b,
        theme.selection.a,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Text 无 Default impl —— 显式默认字段(与 iced Format::default 同参)。
    fn spike_text_default() -> iced::advanced::Text<&'static str, Font> {
        iced::advanced::Text {
            content: "",
            bounds: Size::new(f32::MAX, f32::MAX),
            size: Pixels(16.0),
            line_height: LineHeight::default(),
            font: Font::default(),
            align_x: Alignment::default(),
            align_y: alignment::Vertical::Top,
            shaping: Shaping::default(),
            wrapping: Wrapping::default(),
        }
    }

    #[test]
    fn t01_line_starts_unix() {
        assert_eq!(line_starts(""), vec![0]);
        assert_eq!(line_starts("one"), vec![0]);
        assert_eq!(line_starts("a\nb"), vec![0, 2]);
        assert_eq!(line_starts("a\nbb\nccc"), vec![0, 2, 5]);
        // 结尾换行 → 尾部空行
        assert_eq!(line_starts("a\n"), vec![0, 2]);
    }

    #[test]
    fn t02_line_starts_crlf_cr() {
        // \r\n 计 2 字节,归属前行
        assert_eq!(line_starts("a\r\nb"), vec![0, 3]);
        assert_eq!(line_starts("a\rb"), vec![0, 2]);
    }

    #[test]
    fn t03_global_roundtrip() {
        let text = "aa\nbbbb\nc";
        let starts = line_starts(text);
        assert_eq!(starts, vec![0, 3, 8]);
        assert_eq!(global_offset(&starts, 0, 1), 1);
        assert_eq!(global_offset(&starts, 1, 2), 5);
        assert_eq!(global_offset(&starts, 2, 0), 8);
        for g in 0..=text.len() {
            let (l, i) = line_index(&starts, g);
            assert_eq!(global_offset(&starts, l, i), g, "roundtrip at {g}");
        }
    }

    #[test]
    fn t04_state_default_and_widget_build() {
        let state = State::default();
        assert!(state.selection.is_empty());
        assert_eq!(state.line_starts, vec![0]);
        let w = SelectableText::new("hello").size(14).width(Length::Shrink);
        assert_eq!(w.content, "hello");
        assert_eq!(w.format.size, Some(Pixels(14.0)));
    }

    /// T3 spike 核心:真实 shaping(headless,全局 font system)下
    /// `Paragraph::buffer()` 的 hit → 全局偏移 → word_range → 选区矩形链路。
    #[test]
    fn t05_spike_hit_test_and_rects_chain() {
        use iced::advanced::text::paragraph::Paragraph as _;
        use iced::advanced::text::Text;

        let content = "hello world";
        let paragraph = GraphicsParagraph::with_text(Text {
            content,
            bounds: Size::new(400.0, 100.0),
            size: Pixels(16.0),
            ..spike_text_default()
        });
        let buffer = paragraph.buffer();
        let starts = line_starts(content);

        // hit 链路:命中中段某点 → 全局字节偏移落在文本内。
        let cursor = buffer.hit(40.0, 8.0).expect("hit mid text");
        let g = global_offset(&starts, cursor.line, cursor.index);
        assert!(g <= content.len(), "global offset in range: {g}");

        // 由命中点做词选,得 ["hello"|"world"] 之一的字节区间。
        let mut sel = Selection::new();
        sel.select_word(content, g.min(content.len()));
        let chosen = sel.selected_text(content);
        assert!(
            chosen == "hello" || chosen == "world",
            "word-select picks a word, got {chosen:?}"
        );

        // 矩形链路:非空选区在 layout_runs 上产生 ≥1 个矩形。
        let rects = selection_rects(buffer, &sel, &starts, Point::new(10.0, 5.0));
        assert!(!rects.is_empty(), "selection rect chain yields rects");
        let r = rects[0];
        assert!(r.width >= 2.0, "rect width sane");
        assert!(r.x >= 10.0 && r.y >= 5.0, "rect anchored: {r:?}");
    }

    // ------------------------------------------------------------------
    // T2 交互测试(text_selection 前缀):手势集驱动,真实 shaping 段落。
    // ------------------------------------------------------------------

    /// 测试剪贴板:捕获写入内容。
    #[derive(Default)]
    struct TestClipboard {
        written: Option<String>,
    }
    impl iced::advanced::Clipboard for TestClipboard {
        fn read(&self, _kind: iced::advanced::clipboard::Kind) -> Option<String> {
            None
        }
        fn write(&mut self, _kind: iced::advanced::clipboard::Kind, contents: String) {
            self.written = Some(contents);
        }
    }

    /// KeyPressed 事件构造(补齐 0.14 的 modified_key/physical/location/repeat)。
    fn kp(key: Key, modifiers: keyboard::Modifiers) -> keyboard::Event {
        keyboard::Event::KeyPressed {
            modified_key: key.clone(),
            key,
            physical_key: keyboard::key::Physical::Unidentified(keyboard::key::NativeCode::Unidentified),
            location: keyboard::Location::Standard,
            modifiers,
            text: None,
            repeat: false,
        }
    }

    /// 构造带真实 shaping 段落的 State(锚点取原点,与 bounds==段落盒一致)。
    fn shaped_state(content: &str) -> State {
        use iced::advanced::text::Text;
        let plain = Plain::new(Text {
            content: content.to_string(),
            bounds: Size::new(400.0, 200.0),
            size: Pixels(16.0),
            ..spike_text_default_str()
        });
        State {
            line_starts: line_starts(content),
            last_content: content.to_string(),
            ..State {
                paragraph: plain,
                ..State::default()
            }
        }
    }

    fn spike_text_default_str() -> iced::advanced::Text<String, Font> {
        iced::advanced::Text {
            content: String::new(),
            bounds: Size::new(f32::MAX, f32::MAX),
            size: Pixels(16.0),
            line_height: LineHeight::default(),
            font: Font::default(),
            align_x: Alignment::default(),
            align_y: alignment::Vertical::Top,
            shaping: Shaping::default(),
            wrapping: Wrapping::default(),
        }
    }

    const W: &str = "hello world";

    fn press(widget: &SelectableText, state: &mut State, x: f32, y: f32) -> bool {
        widget.handle_mouse(
            state,
            &mouse::Event::ButtonPressed(mouse::Button::Left),
            Some(Point::new(x, y)),
            Point::ORIGIN,
            Rectangle::new(Point::ORIGIN, Size::new(400.0, 100.0)),
        )
    }

    fn r#move(widget: &SelectableText, state: &mut State, x: f32, y: f32) -> bool {
        widget.handle_mouse(
            state,
            &mouse::Event::CursorMoved { position: Point::new(x, y) },
            Some(Point::new(x, y)),
            Point::ORIGIN,
            Rectangle::new(Point::ORIGIN, Size::new(400.0, 100.0)),
        )
    }

    fn release(widget: &SelectableText, state: &mut State) -> bool {
        widget.handle_mouse(
            state,
            &mouse::Event::ButtonReleased(mouse::Button::Left),
            Some(Point::ORIGIN),
            Point::ORIGIN,
            Rectangle::new(Point::ORIGIN, Size::new(400.0, 100.0)),
        )
    }

    #[test]
    fn text_selection_drag_selects_range() {
        // 按下 "hello" 首部 → 拖到 world 中段 → 释放:选中非空、以词边界为界。
        let w = SelectableText::new(W);
        let mut state = shaped_state(W);
        assert!(press(&w, &mut state, 2.0, 8.0), "press captured");
        assert!(r#move(&w, &mut state, 48.0, 8.0), "drag captured");
        assert!(release(&w, &mut state), "release captured");
        assert!(!state.dragging);
        let sel = state.selection.selected_text(W);
        assert!(!sel.is_empty(), "drag selects text");
        assert!(W.contains(sel), "selection is a substring: {sel:?}");
        assert!(sel.starts_with('h'), "starts at press point: {sel:?}");
    }

    #[test]
    fn text_selection_drag_backward() {
        // 右→左拖:归一区间,选中文本相同语义。
        let w = SelectableText::new(W);
        let mut state = shaped_state(W);
        press(&w, &mut state, 48.0, 8.0);
        r#move(&w, &mut state, 2.0, 8.0);
        release(&w, &mut state);
        let sel = state.selection.selected_text(W);
        assert!(sel.starts_with('h'), "backward drag still from left: {sel:?}");
    }

    #[test]
    fn text_selection_double_click_selects_word() {
        // 两次快速近位按下 → 词选(整词,无半个词)。
        let w = SelectableText::new(W);
        let mut state = shaped_state(W);
        press(&w, &mut state, 40.0, 8.0);
        release(&w, &mut state);
        assert!(press(&w, &mut state, 41.0, 8.0), "second press captured");
        let sel = state.selection.selected_text(W);
        assert!(
            sel == "hello" || sel == "world",
            "double-click picks a whole word, got {sel:?}"
        );
    }

    #[test]
    fn text_selection_single_click_after_drag_clears() {
        // 已有选区后,异位点单击(超 4px 位移,非双击)→ 清空。
        let w = SelectableText::new(W);
        let mut state = shaped_state(W);
        press(&w, &mut state, 40.0, 8.0);
        release(&w, &mut state);
        press(&w, &mut state, 41.0, 8.0); // 双击词选
        assert!(!state.selection.is_empty());
        press(&w, &mut state, 3.0, 8.0); // 异位单击
        release(&w, &mut state);
        assert!(state.selection.is_empty(), "distant click clears");
    }

    #[test]
    fn text_selection_esc_clears_non_capturing() {
        let w = SelectableText::new(W);
        let mut state = shaped_state(W);
        press(&w, &mut state, 40.0, 8.0);
        press(&w, &mut state, 41.0, 8.0); // 词选
        assert!(!state.selection.is_empty());
        let mut cb = TestClipboard::default();
        let captured = w.handle_keyboard(
            &mut state,
            &kp(Key::Named(key::Named::Escape), Default::default()),
            &mut cb,
        );
        assert!(!captured, "Esc must NOT capture (global flows keep it)");
        assert!(state.selection.is_empty(), "Esc clears selection");
    }

    #[test]
    fn text_selection_ctrl_c_writes_clipboard() {
        let w = SelectableText::new(W);
        let mut state = shaped_state(W);
        press(&w, &mut state, 40.0, 8.0);
        press(&w, &mut state, 41.0, 8.0); // 词选
        let word = state.selection.selected_text(W).to_string();
        assert!(!word.is_empty());

        let mut cb = TestClipboard::default();
        let captured = w.handle_keyboard(
            &mut state,
            &kp(Key::Character("c".into()), keyboard::Modifiers::CTRL),
            &mut cb,
        );
        assert!(captured, "Ctrl+C captures when selection non-empty");
        assert_eq!(cb.written.as_deref(), Some(word.as_str()));

        // 无选区时 Ctrl+C 不捕获不写(不抢编辑器快捷键)。
        let mut cb2 = TestClipboard::default();
        state.selection.clear();
        let captured2 = w.handle_keyboard(
            &mut state,
            &kp(Key::Character("c".into()), keyboard::Modifiers::CTRL),
            &mut cb2,
        );
        assert!(!captured2 && cb2.written.is_none());
    }

    /// 多行文本的 hit → (行,偏移) → 全局换算正确性(shaping 真实行切分)。
    #[test]
    fn t06_spike_multiline_global_mapping() {
        use iced::advanced::text::paragraph::Paragraph as _;
        use iced::advanced::text::Text;

        let content = "one\ntwo words\nthree";
        let paragraph = GraphicsParagraph::with_text(Text {
            content,
            bounds: Size::new(400.0, 200.0),
            size: Pixels(16.0),
            ..spike_text_default()
        });
        let starts = line_starts(content);
        assert_eq!(starts, vec![0, 4, 14]);

        let buffer = paragraph.buffer();
        // 命中第 2 行(行高 16×1.2≈19.2,第 2 行 y 中心 ≈ 28)中部。
        let cursor = buffer.hit(10.0, 28.0).expect("hit line 2");
        assert_eq!(cursor.line, 1, "hit lands on logical line 2");
        let g = global_offset(&starts, cursor.line, cursor.index);
        assert!((4..14).contains(&g), "global maps into line 2 span: {g}");
        let (l, i) = line_index(&starts, g);
        assert_eq!((l, global_offset(&starts, l, i)), (1, g));

        // 跨行选区矩形:one 整行 + two words 整行 + three 前缀。
        let sel = Selection { anchor: 0, head: 17 }; // "one\ntwo words\nth"
        let rects = selection_rects(buffer, &sel, &starts, Point::new(0.0, 0.0));
        assert!(rects.len() >= 3, "one rect per touched line, got {}", rects.len());
    }

    /// T2 管线冒烟(iced_test simulator,feature `iced-layout-tests`):
    /// 真实 iced 事件流 press→move→release 全程无 panic 且事件被捕获
    /// (shell.capture_event → Status::Captured)。
    #[cfg(feature = "iced-layout-tests")]
    #[test]
    fn text_selection_simulator_pipeline_smoke() {
        use iced_test::simulator;

        let ui: iced::Element<'static, (), iced::Theme, iced::Renderer> =
            SelectableText::new("hello world").size(16).into();
        let mut sim = simulator(ui);
        sim.point_at(iced::Point::new(30.0, 8.0));

        let statuses = sim.simulate([
            iced::Event::Mouse(iced::mouse::Event::ButtonPressed(
                iced::mouse::Button::Left,
            )),
            iced::Event::Mouse(iced::mouse::Event::CursorMoved {
                position: iced::Point::new(60.0, 8.0),
            }),
            iced::Event::Mouse(iced::mouse::Event::ButtonReleased(
                iced::mouse::Button::Left,
            )),
        ]);
        assert!(
            statuses.iter().any(|s| matches!(s, iced::event::Status::Captured)),
            "drag gestures must be captured by the real pipeline: {statuses:?}"
        );
    }

    /// T3 管线链路(iced-layout-tests 档):真实 iced 事件流中 Ctrl+C 键盘
    /// 事件同样送达 widget update 并被捕获(无选区静默/有选区捕获——
    /// 运行时与单测同机制,实机剩余变量仅窗口焦点)。
    #[cfg(feature = "iced-layout-tests")]
    #[test]
    fn text_selection_simulator_ctrl_c_captured() {
        use iced::keyboard::{self, key, Key};
        use iced_test::simulator;

        let ui: iced::Element<'static, (), iced::Theme, iced::Renderer> =
            SelectableText::new("hello world").size(16).into();
        let mut sim = simulator(ui);
        sim.point_at(iced::Point::new(40.0, 8.0));

        // 无选区:Ctrl+C 必须不捕获(不抢编辑器快捷键)。
        let idle = sim.simulate([iced::Event::Keyboard(
            keyboard::Event::KeyPressed {
                key: Key::Character("c".into()),
                modified_key: Key::Character("c".into()),
                physical_key: key::Physical::Unidentified(
                    key::NativeCode::Unidentified,
                ),
                location: keyboard::Location::Standard,
                modifiers: keyboard::Modifiers::CTRL,
                text: None,
                repeat: false,
            },
        )]);
        assert!(
            idle.iter().all(|s| matches!(s, iced::event::Status::Ignored)),
            "idle Ctrl+C must pass through: {idle:?}"
        );

        // 拖选后有选区:Ctrl+C 必须捕获(进剪贴板路径)。
        sim.simulate([
            iced::Event::Mouse(iced::mouse::Event::ButtonPressed(
                iced::mouse::Button::Left,
            )),
            iced::Event::Mouse(iced::mouse::Event::CursorMoved {
                position: iced::Point::new(60.0, 8.0),
            }),
            iced::Event::Mouse(iced::mouse::Event::ButtonReleased(
                iced::mouse::Button::Left,
            )),
        ]);
        let with_sel = sim.simulate([iced::Event::Keyboard(
            keyboard::Event::KeyPressed {
                key: Key::Character("c".into()),
                modified_key: Key::Character("c".into()),
                physical_key: key::Physical::Unidentified(
                    key::NativeCode::Unidentified,
                ),
                location: keyboard::Location::Standard,
                modifiers: keyboard::Modifiers::CTRL,
                text: None,
                repeat: false,
            },
        )]);
        assert!(
            with_sel
                .iter()
                .any(|s| matches!(s, iced::event::Status::Captured)),
            "Ctrl+C with selection must be captured: {with_sel:?}"
        );
    }
}
