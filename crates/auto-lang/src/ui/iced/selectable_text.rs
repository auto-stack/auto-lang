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
use iced::advanced::{Layout, Renderer as _, Widget};
use iced::mouse;
use iced::{
    Color, Element, Font, Length, Pixels, Point, Rectangle, Size, Theme,
};
use iced::alignment::{self};
use iced::advanced::text::{Alignment, LineHeight, Shaping, Wrapping};

use super::selection::Selection;

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
}

impl Default for State {
    fn default() -> Self {
        Self {
            paragraph: Plain::default(),
            selection: Selection::new(),
            line_starts: vec![0],
            last_content: String::new(),
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
}
