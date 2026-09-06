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
//   hidden or blink-off).
//
// T4 interaction protocol (004/005 migration):
// - mouse: pixel→cell → selection begin/extend/finish (multi-click window
//   500ms: 1=Simple 2=Semantic 3=Lines; Alt=Block), release publishes
//   on_select (payload: `terminal_selected_text(key)`);
// - wheel: core scroll offset ±3 lines/notch, publishes on_scroll
//   (payload: `terminal_scroll_offset(key)`), badge draws offset > 0;
// - right click: opens the Copy/Paste/SelectAll menu (drawn overlay), item
//   hit on left press publishes on_menu (payload `terminal_take_menu_item`);
// - IME: over-the-spot declaration kept for winit cursor-area anchoring,
//   preedit string self-drawn at the cursor cell (#11 workaround).

use iced::advanced::text::{LineHeight, Paragraph, Shaping, Span, Text, Wrapping};
use iced::advanced::text::Renderer as _;
use iced::advanced::widget::{tree, Tree};
use iced::advanced::{
    input_method, layout::Node, mouse, renderer, widget::Widget, Layout,
};
use iced::advanced::Renderer as _;
use iced::keyboard::{self, Modifiers};
use iced::{alignment, Background, Border, Color, Element, Font, Length, Point, Rectangle, Size, Theme, mouse::ScrollDelta};

use crate::ui::terminal::{
    TermCell, TermColor, TermCursorShape, TermSelectionType, TerminalCore,
};

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

type Para = <iced::Renderer as iced::advanced::text::Renderer>::Paragraph;

/// Cell metrics (fixed monospace approximation; auto-term measured these
/// from the font via GridMetrics — the component's T5 pipeline may refine).
pub const CELL_W: f32 = 8.0;
pub const CELL_H: f32 = 16.0;
pub const FONT_PX: f32 = 16.0;
const BORDER: f32 = 1.0;

const DEFAULT_FG: Color = Color::from_rgb8(0xe8, 0xe8, 0xe8);
const DEFAULT_BG: Color = Color::from_rgb8(0x06, 0x07, 0x09);

const MENU_ITEMS: [&str; 3] = ["Copy", "Paste", "Select All"];
const MENU_ITEM_W: f32 = 80.0;
const MULTI_CLICK_WINDOW: Duration = Duration::from_millis(500);
const WHEEL_LINES_PER_NOTCH: i32 = 3;

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

/// Terminal widget — draws the engine grid carried by `core` and maps
/// mouse/wheel events onto the selection/scroll/menu state machine.
pub struct Terminal<M> {
    pub core: &'static TerminalCore,
    pub key: String,
    pub scroll_offset: u16,
    pub preedit: Option<String>,
    pub on_select: Option<M>,
    pub on_menu: Option<M>,
    pub width: Length,
    pub height: Length,
}

/// Multi-click type ruling (005 T2 纯函数):Alt 恒为 Block;2=Sematic 词选、
/// 3=Lines 行选、1=Simple。
fn begin_selection_type(count: u8, mods: Modifiers) -> TermSelectionType {
    if mods.alt() {
        return TermSelectionType::Block;
    }
    match count {
        2 => TermSelectionType::Semantic,
        3 => TermSelectionType::Lines,
        _ => TermSelectionType::Simple,
    }
}

/// 菜单矩形(右键点为左上角;auto-term menu_rect 同款)。
fn menu_rect(at: (f32, f32)) -> Rectangle {
    Rectangle::new(
        Point::new(at.0, at.1),
        Size::new(MENU_ITEM_W, MENU_ITEMS.len() as f32 * CELL_H),
    )
}

/// 菜单命中(局部坐标 → 项下标)。
fn menu_item_at(at: (f32, f32), pos: Point) -> Option<usize> {
    let rect = menu_rect(at);
    if !rect.contains(pos) {
        return None;
    }
    Some(((pos.y - rect.y) / CELL_H) as usize)
}

impl<M> Terminal<M> {
    /// Get-or-create the terminal state for `key` (geometry diffed in).
    pub fn new(key: &str, cols: u16, rows: u16) -> Self {
        Self {
            core: crate::ui::terminal::terminal(key, cols, rows),
            key: key.to_owned(),
            scroll_offset: 0,
            preedit: None,
            on_select: None,
            on_menu: None,
            width: Length::Fixed(cols as f32 * CELL_W + 2.0 * BORDER),
            height: Length::Fixed(rows as f32 * CELL_H + 2.0 * BORDER),
        }
    }

    /// 像素坐标 → 视口格 (row, col);越界 clamp 到边缘格。
    fn pixel_to_cell(&self, pos: Point, bounds: Rectangle) -> (usize, usize) {
        let cols = self.core.cols.max(1) as usize;
        let rows = self.core.rows.max(1) as usize;
        let fx = (pos.x - bounds.x - BORDER) / CELL_W;
        let fy = (pos.y - bounds.y - BORDER) / CELL_H;
        let col = (fx.floor() as i32).clamp(0, cols as i32 - 1) as usize;
        let row = (fy.floor() as i32).clamp(0, rows as i32 - 1) as usize;
        (row, col)
    }

    /// 以终端光标格为锚向 runtime 声明 IME 策略(auto-term T8 同款:
    /// over-the-spot 不落屏,仍声明 Enabled 以保 winit 组合窗定位,
    /// preedit 串由本组件自绘)。
    fn request_ime(&self, shell: &mut iced::advanced::Shell<'_, M>, bounds: Rectangle) {
        let cursor = self.core.cursor();
        let rect = Rectangle::new(
            Point::new(
                bounds.x + BORDER + cursor.col as f32 * CELL_W,
                bounds.y + BORDER + cursor.row as f32 * CELL_H,
            ),
            Size::new(CELL_W, CELL_H),
        );
        shell.request_input_method(&input_method::InputMethod::<String>::Enabled {
            cursor: rect,
            purpose: input_method::Purpose::Terminal,
            preedit: None,
        });
    }
}

impl<M: Clone + std::fmt::Debug + 'static> Widget<M, Theme, iced::Renderer> for Terminal<M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<TerminalState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(TerminalState::default())
    }

    fn size(&self) -> Size<Length> {
        Size { width: self.width, height: self.height }
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

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &iced::Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &iced::Renderer,
        _clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, M>,
        _viewport: &Rectangle,
    ) {
        // 任意事件到达即刷新 IME 声明(幂等;auto-term T8 同款)。
        self.request_ime(shell, layout.bounds());

        let state = tree.state.downcast_mut::<TerminalState>();
        let bounds = layout.bounds();
        let core = self.core;

        match event {
            iced::Event::Keyboard(keyboard::Event::ModifiersChanged(mods)) => {
                state.mods = *mods;
            }
            iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let Some(pos) = cursor.position_over(bounds) else { return };
                // 菜单开着时左键归菜单:命中项→动作,未命中→关闭;一律吞。
                if let Some(at) = state.menu_open {
                    if let Some(idx) = menu_item_at(at, Point::new(pos.x - bounds.x, pos.y - bounds.y)) {
                        crate::ui::terminal::terminal_set_menu_item(core, idx as u8);
                        if let Some(msg) = self.on_menu.clone() {
                            shell.publish(msg);
                        }
                    }
                    state.menu_open = None;
                    return;
                }
                // 多击判定(500ms 窗口 + 同格)。
                let now = Instant::now();
                let cell = self.pixel_to_cell(pos, bounds);
                let same_cell = state.last_cell == Some(cell);
                let count = if now.duration_since(state.last_click_at.unwrap_or(now - Duration::from_secs(10)))
                    <= MULTI_CLICK_WINDOW
                    && same_cell
                {
                    (state.last_count % 3) + 1
                } else {
                    1
                };
                state.last_click_at = Some(now);
                state.last_count = count;
                state.last_cell = Some(cell);
                let kind = begin_selection_type(count, state.mods);
                crate::ui::terminal::terminal_selection_begin(core, kind, cell.0, cell.1);
                if kind == TermSelectionType::Semantic {
                    crate::ui::terminal::terminal_selection_expand_word(core, cell.0, cell.1);
                }
                state.dragging = true;
            }
            iced::Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if state.dragging {
                    if let Some(pos) = cursor.position_over(bounds) {
                        let cell = self.pixel_to_cell(pos, bounds);
                        crate::ui::terminal::terminal_selection_extend(core, cell.0, cell.1);
                        if let Some(msg) = self.on_select.clone() {
                            shell.publish(msg);
                        }
                    }
                }
                // 菜单悬停项随光标刷新(draw 反色)。
                if let Some(at) = state.menu_open {
                    state.hover_item = cursor
                        .position()
                        .and_then(|pos| menu_item_at(at, Point::new(pos.x - bounds.x, pos.y - bounds.y)));
                }
            }
            iced::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if state.dragging {
                    state.dragging = false;
                    crate::ui::terminal::terminal_selection_finish(core);
                    if let Some(msg) = self.on_select.clone() {
                        shell.publish(msg);
                    }
                }
            }
            iced::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Right)) => {
                // 右键释放开菜单(auto-term 同款;位置为组件局部坐标)。
                if let Some(pos) = cursor.position_over(bounds) {
                    state.menu_open = Some((pos.x - bounds.x, pos.y - bounds.y));
                }
            }
            iced::Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let lines: i32 = match delta {
                    ScrollDelta::Lines { y, .. } => *y as i32,
                    ScrollDelta::Pixels { y, .. } => (*y / CELL_H) as i32,
                };
                if lines != 0 {
                    crate::ui::terminal::terminal_scroll(
                        core,
                        -lines * WHEEL_LINES_PER_NOTCH,
                    );
                    if let Some(msg) = self.on_select.clone() {
                        shell.publish(msg);
                    }
                }
            }
            _ => {}
        }
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
        let state = tree.state.downcast_ref::<TerminalState>();

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

        // 选中高亮层:overlay quad 每帧 emit(004 T3;不进 digest 缓存,
        // 损伤门控不感知 selection 变化)。区间为喂入缓冲行;滚动偏移仅
        // 由 app 重喂内容体现,此处不做二次映射。
        if let Some(sel) = crate::ui::terminal::terminal_selection(self.core) {
            let highlight = Color::from_rgba(0.35, 0.48, 0.66, 0.45);
            for row in sel.start.0..=sel.end.0 {
                let (col_begin, col_last) = if sel.kind == TermSelectionType::Block {
                    (sel.start.1.min(sel.end.1), sel.start.1.max(sel.end.1))
                } else if row == sel.start.0 && row == sel.end.0 {
                    (sel.start.1, sel.end.1)
                } else if row == sel.start.0 {
                    (sel.start.1, self.core.cols as usize - 1)
                } else if row == sel.end.0 {
                    (0, sel.end.1)
                } else {
                    (0, self.core.cols as usize - 1)
                };
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle::new(
                            Point::new(
                                bounds.x + BORDER + col_begin as f32 * CELL_W,
                                bounds.y + BORDER + row as f32 * CELL_H,
                            ),
                            Size::new(
                                (col_last - col_begin + 1) as f32 * CELL_W,
                                CELL_H,
                            ),
                        ),
                        ..renderer::Quad::default()
                    },
                    Background::Color(highlight),
                );
            }
        }

        // 保留式文本层:每行 Paragraph 缓存 + digest 门控重建。
        let mut caches = row_caches().lock().unwrap();
        let cache = caches.entry(self.key.clone()).or_default();
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

        // preedit 自绘覆盖层(#11 绕行):光标格锚定,下划线标记组合串。
        if let Some(preedit) = self.preedit.as_deref().filter(|p| !p.is_empty()) {
            let cursor = self.core.cursor();
            let y = bounds.y + BORDER + cursor.row as f32 * CELL_H;
            let x = bounds.x + BORDER + cursor.col as f32 * CELL_W;
            let w = preedit.chars().count() as f32 * CELL_W;
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle::new(Point::new(x, y), Size::new(w, CELL_H)),
                    ..renderer::Quad::default()
                },
                Background::Color(Color::from_rgba(0.2, 0.3, 0.45, 0.9)),
            );
            let para = plain_para(preedit, w);
            renderer.fill_paragraph(&para, Point::new(x, y), DEFAULT_FG, bounds);
        }

        // 滚动偏移 badge(offset > 0 时右上角指示;auto-term 同款)。
        let offset = self.scroll_offset;
        if offset > 0 {
            let badge = format!("↑{offset}");
            let badge_w = badge.chars().count() as f32 * CELL_W + CELL_W;
            let bg_bounds = Rectangle::new(
                Point::new(bounds.x + bounds.width - badge_w - CELL_W, bounds.y),
                Size::new(badge_w, CELL_H),
            );
            renderer.fill_quad(
                renderer::Quad { bounds: bg_bounds, ..renderer::Quad::default() },
                Background::Color(DEFAULT_BG),
            );
            let para = plain_para(&badge, badge_w);
            renderer.fill_paragraph(&para, bg_bounds.position(), DEFAULT_FG, bounds);
        }

        // 菜单层:右键打开的 Copy/Paste/Select All 浮层,悬停项反色。
        if let Some(at) = state.menu_open {
            let rect = menu_rect(at);
            renderer.fill_quad(
                renderer::Quad {
                    bounds: rect,
                    border: Border {
                        color: Color::from_rgb(0.35, 0.38, 0.42),
                        width: 1.0.into(),
                        radius: 2.0.into(),
                    },
                    ..renderer::Quad::default()
                },
                Background::Color(Color::from_rgb(0.12, 0.13, 0.16)),
            );
            for (i, label) in MENU_ITEMS.iter().enumerate() {
                let item_rect = Rectangle::new(
                    Point::new(rect.x, rect.y + i as f32 * CELL_H),
                    Size::new(MENU_ITEM_W, CELL_H),
                );
                let hover = state.hover_item == Some(i);
                if hover {
                    renderer.fill_quad(
                        renderer::Quad { bounds: item_rect, ..renderer::Quad::default() },
                        Background::Color(Color::from_rgb(0.25, 0.35, 0.55)),
                    );
                }
                let para = plain_para(label, MENU_ITEM_W);
                renderer.fill_paragraph(
                    &para,
                    item_rect.position(),
                    DEFAULT_FG,
                    bounds,
                );
            }
        }
    }
}

/// Widget 交互状态(tree state;多击/拖选/菜单)。
#[derive(Default)]
pub struct TerminalState {
    /// Last drawn content generation (damage seed).
    pub generation: u64,
    mods: Modifiers,
    dragging: bool,
    last_click_at: Option<Instant>,
    last_count: u8,
    last_cell: Option<(usize, usize)>,
    /// 菜单锚点(组件局部坐标;None=关闭)。
    menu_open: Option<(f32, f32)>,
    hover_item: Option<usize>,
}

impl<'a, M: Clone + std::fmt::Debug + 'static> From<Terminal<M>> for Element<'a, M> {
    fn from(widget: Terminal<M>) -> Self {
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

/// 纯文本段落(badge / preedit / 菜单标签)。
fn plain_para(text: &str, width: f32) -> Para {
    Para::with_text(Text {
        content: text,
        bounds: Size::new(width, CELL_H),
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
