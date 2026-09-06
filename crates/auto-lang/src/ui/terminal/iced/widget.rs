// PLAN-009 P1 ② iced adapter — the `Terminal` widget.
//
// The only iced dependency point of the terminal component (hard layering:
// mod.rs never imports iced).
//
// T3 draw protocol (migrated from auto-term widget.rs):
// - background quad per frame;
// - non-default-bg runs as per-frame quads (no shaping cost);
// - one cached shaped paragraph per row, rebuilt only when the row digest
//   (chars + fg + bg) changes — `fill_paragraph` reuses undamaged rows;
// - cursor as an inversion block / beam / underline quad (skipped while
//   hidden or blink-off);
// - scroll/selection/IME/menu arrive with T4.

use iced::advanced::text::{LineHeight, Paragraph, Shaping, Span, Text, Wrapping};
use iced::advanced::text::Renderer as _;
use iced::advanced::widget::{tree, Tree};
use iced::advanced::{layout::Node, mouse, renderer, widget::Widget, Layout};
use iced::advanced::Renderer as _;
use iced::{alignment, Background, Border, Color, Element, Font, Length, Point, Rectangle, Size, Theme};

use crate::ui::terminal::{TermCell, TermColor, TermCursorShape, TerminalCore};

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

type Para = <iced::Renderer as iced::advanced::text::Renderer>::Paragraph;

/// Cell metrics (fixed monospace approximation; auto-term measured these
/// from the font via GridMetrics — the component's T5 pipeline may refine).
pub const CELL_W: f32 = 8.0;
pub const CELL_H: f32 = 16.0;
pub const FONT_PX: f32 = 16.0;
const BORDER: f32 = 1.0;

const DEFAULT_FG: Color = Color::from_rgb8(0xe8, 0xe8, 0xe8);
const DEFAULT_BG: Color = Color::from_rgb8(0x06, 0x07, 0x09);

/// 一行的保留缓存:shaping 产物 + 内容 digest(auto-term RowEntry)。
struct RowEntry {
    para: Para,
    digest: u64,
}

/// Per-terminal row cache, keyed by the terminal key (auto-term kept one
/// global Vec — multi-terminal needs the key dimension; draw receives
/// `&Tree`, so mutable caches live here rather than in widget state).
static ROW_CACHES: OnceLock<Mutex<HashMap<String, Vec<Option<RowEntry>>>>> = OnceLock::new();

fn row_caches() -> &'static Mutex<HashMap<String, Vec<Option<RowEntry>>>> {
    ROW_CACHES.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Terminal widget — draws the engine grid carried by `core`.
pub struct Terminal {
    core: &'static TerminalCore,
    width: Length,
    height: Length,
}

impl Terminal {
    /// Get-or-create the terminal state for `key` (geometry diffed in).
    pub fn new(key: &str, cols: u16, rows: u16) -> Self {
        Self {
            core: crate::ui::terminal::terminal(key, cols, rows),
            width: Length::Fixed(cols as f32 * CELL_W + 2.0 * BORDER),
            height: Length::Fixed(rows as f32 * CELL_H + 2.0 * BORDER),
        }
    }

    /// Override the suggested size (defaults to the grid's cell metrics).
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }
}

impl<Message> Widget<Message, Theme, iced::Renderer> for Terminal
where
    Message: Clone,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<TerminalState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(TerminalState::default())
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &iced::Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> Node {
        let limits = limits.width(self.width).height(self.height);
        Node::new(limits.resolve(self.width, self.height, Size::default()))
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        _theme: &Theme,
        _defaults: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let _ = tree;

        renderer.fill_quad(
            renderer::Quad { bounds, ..renderer::Quad::default() },
            Background::Color(DEFAULT_BG),
        );
        // Placeholder frame: an empty grid still shows its border + geometry
        // (T2 占位矩形语义,空网格时即整体外观)。
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: Border {
                    color: Color::from_rgb(0.25, 0.28, 0.32),
                    width: BORDER.into(),
                    radius: 0.0.into(),
                },
                ..renderer::Quad::default()
            },
            Background::Color(Color::TRANSPARENT),
        );

        let (cells, digests) = self.core.snapshot();
        if cells.is_empty() {
            return;
        }

        // 背景层:非默认 bg 的 run(每帧 emit;无形状成本)。
        for (y, line) in cells.iter().enumerate() {
            let line_y = bounds.y + y as f32 * CELL_H;
            if line_y > bounds.y + bounds.height {
                break;
            }
            let mut idx = 0usize;
            while idx < line.len() {
                if line[idx].bg == TermColor::Default {
                    idx += 1;
                    continue;
                }
                let bg = line[idx].bg;
                let start = idx;
                while idx < line.len() && same_color(line[idx].bg, bg) {
                    idx += 1;
                }
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle::new(
                            Point::new(bounds.x + start as f32 * CELL_W, line_y),
                            Size::new((idx - start) as f32 * CELL_W, CELL_H),
                        ),
                        ..renderer::Quad::default()
                    },
                    Background::Color(to_iced_color(bg, false)),
                );
            }
        }

        // 保留式文本层:每行 Paragraph 缓存 + digest 门控重建。
        let mut caches = row_caches().lock().unwrap();
        let cache = caches.entry(self.core.key.clone()).or_default();
        if cache.len() != cells.len() {
            cache.clear();
            cache.resize_with(cells.len(), || None);
        }
        for (y, line) in cells.iter().enumerate() {
            let line_y = bounds.y + y as f32 * CELL_H;
            if line_y > bounds.y + bounds.height {
                break;
            }
            let digest = digests[y];
            let stale = cache[y].as_ref().is_none_or(|e| e.digest != digest);
            if stale {
                let para = build_row_paragraph(line);
                cache[y] = Some(RowEntry { para, digest });
            }
            if let Some(entry) = cache[y].as_ref() {
                renderer.fill_paragraph(
                    &entry.para,
                    Point::new(bounds.x + BORDER, line_y),
                    DEFAULT_FG,
                    bounds,
                );
            }
        }
        drop(caches);

        // 光标层:块/竖线/下划线(Hidden 或闪烁熄灭相不画)。
        let cursor = self.core.cursor();
        if cursor.shape != TermCursorShape::Hidden && cursor.on {
            let row = cursor.row as f32 * CELL_H;
            let col = cursor.col as f32 * CELL_W;
            let (rect, color) = match cursor.shape {
                TermCursorShape::Block => (
                    Rectangle::new(
                        Point::new(bounds.x + BORDER + col, bounds.y + BORDER + row),
                        Size::new(CELL_W, CELL_H),
                    ),
                    Color::from_rgba(0.91, 0.91, 0.91, 0.85),
                ),
                TermCursorShape::Beam => (
                    Rectangle::new(
                        Point::new(bounds.x + BORDER + col, bounds.y + BORDER + row),
                        Size::new(2.0, CELL_H),
                    ),
                    DEFAULT_FG,
                ),
                TermCursorShape::Underline => (
                    Rectangle::new(
                        Point::new(
                            bounds.x + BORDER + col,
                            bounds.y + BORDER + row + CELL_H - 2.0,
                        ),
                        Size::new(CELL_W, 2.0),
                    ),
                    DEFAULT_FG,
                ),
                TermCursorShape::Hidden => unreachable!("filtered above"),
            };
            renderer.fill_quad(
                renderer::Quad { bounds: rect, ..renderer::Quad::default() },
                Background::Color(color),
            );
        }
    }
}

#[derive(Default)]
pub struct TerminalState {
    /// Last drawn content generation (damage seed; interaction state lands
    /// with T4).
    pub generation: u64,
}

impl<'a, Message> From<Terminal> for Element<'a, Message>
where
    Message: 'a + Clone,
{
    fn from(widget: Terminal) -> Self {
        Self::new(widget)
    }
}

/// run 聚合:同前景色的连续 cell 合并为一个 span,前景色烘焙进
/// paragraph(auto-term build_row_paragraph 同款)。
fn build_row_paragraph(line: &[TermCell]) -> Para {
    let mut text = String::with_capacity(line.len());
    let mut runs: Vec<(usize, usize, TermColor)> = Vec::new();
    let mut idx = 0usize;
    while idx < line.len() {
        let fg = line[idx].fg;
        let start = idx;
        while idx < line.len() && line[idx].fg == fg {
            idx += 1;
        }
        let begin = text.len();
        for cell in &line[start..idx] {
            text.push(cell.ch);
        }
        runs.push((begin, text.len(), fg));
    }
    let spans: Vec<Span<'_, ()>> = runs
        .iter()
        .map(|(begin, end, fg)| Span {
            text: std::borrow::Cow::Borrowed(&text[*begin..*end]),
            color: Some(to_iced_color(*fg, true)),
            ..Default::default()
        })
        .collect();

    Para::with_spans(Text {
        content: spans.as_slice(),
        // 宽度加一格余量,避免最末字符因舍入被折行
        bounds: Size::new(line.len() as f32 * CELL_W + CELL_W, CELL_H),
        size: FONT_PX.into(),
        line_height: LineHeight::Absolute(CELL_H.into()),
        font: Font::MONOSPACE,
        align_x: alignment::Horizontal::Left.into(),
        align_y: alignment::Vertical::Top,
        shaping: Shaping::Basic,
        wrapping: Wrapping::None,
    })
}

/// Run-merge color comparison (fg/bg direct equality on the scalar palette).
fn same_color(a: TermColor, b: TermColor) -> bool {
    a == b
}

/// 本地标量色板 → iced Color(xterm 256 全映射;Named 语义已并 Indexed 0-15)。
fn to_iced_color(c: TermColor, is_fg: bool) -> Color {
    match c {
        TermColor::Default => {
            if is_fg {
                DEFAULT_FG
            } else {
                DEFAULT_BG
            }
        }
        TermColor::Rgb(r, g, b) => Color::from_rgb8(r, g, b),
        TermColor::Indexed(i) => {
            let [r, g, b] = xterm256(i);
            Color::from_rgb8(r, g, b)
        }
    }
}

/// xterm 256 palette → RGB (0-15 = base16, 16-231 = 6×6×6 cube,
/// 232-255 = grayscale ramp).
fn xterm256(i: u8) -> [u8; 3] {
    const BASE16: [[u8; 3]; 16] = [
        [0x00, 0x00, 0x00], [0x80, 0x00, 0x00], [0x00, 0x80, 0x00], [0x80, 0x80, 0x00],
        [0x00, 0x00, 0x80], [0x80, 0x00, 0x80], [0x00, 0x80, 0x80], [0xc0, 0xc0, 0xc0],
        [0x80, 0x80, 0x80], [0xff, 0x00, 0x00], [0x00, 0xff, 0x00], [0xff, 0xff, 0x00],
        [0x00, 0x00, 0xff], [0xff, 0x00, 0xff], [0x00, 0xff, 0xff], [0xff, 0xff, 0xff],
    ];
    if i < 16 {
        return BASE16[i as usize];
    }
    if i < 232 {
        let v = i - 16;
        let levels = [0u8, 95, 135, 175, 215, 255];
        return [
            levels[(v / 36) as usize],
            levels[((v % 36) / 6) as usize],
            levels[(v % 6) as usize],
        ];
    }
    let gray = 8 + 10 * (i - 232);
    [gray, gray, gray]
}
