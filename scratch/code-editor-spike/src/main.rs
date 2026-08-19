//! Plan 413 Phase 0 spike:跨平台代码编辑器 widget 技术验证。
//!
//! 验证清单(Plan 413 §4 Phase 0):
//!   1. iced 0.14 `fill_raw` 渲染 cosmic-text Buffer(wgpu 主验;tiny-skia 见 README)
//!   2. 全局 `font_system()` 与 iced 文本管线共享,短锁策略无死锁
//!   3. cosmic-text 0.15 `ViEditor` + `SyntaxEditor`(syntect)rust 语法高亮
//!   4. IME:`Event::InputMethod` Preedit/Commit + `shell.request_input_method`
//!   5. 行号槽 CPU 光栅 + `FilterMethod::Nearest`
//!   6. two-face 与 cosmic-text 0.15 的 syntect 版本统一(`cargo tree -i syntect`)
//!
//! License: MIT.
//! Architecture inspired by cosmic-edit (GPL-3.0, System76); original
//! implementation — no code copied from cosmic-edit.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, Weak};
use std::time::{Duration, Instant};

use cosmic_text::{
    Action, Attrs, AttrsList, Buffer, BufferLine, Cursor, Edit, Family, FontSystem, LayoutLine,
    LayoutRun, LineEnding, Metrics, Motion, Selection, Shaping, SwashCache, SyntaxEditor,
    SyntaxSystem, ViEditor, Wrap,
};
use iced::advanced::graphics::text::{font_system, Raw};
use iced::advanced::widget::Widget;
use iced::advanced::graphics::text::Renderer as RawTextRenderer;
use iced::advanced::{
    clipboard::Kind as ClipboardKind, image as adv_image, input_method, layout::Node, mouse,
    renderer, text as adv_text, Clipboard, Layout, Renderer as _, Shell,
};
use iced::advanced::image::Renderer as ImageRenderer;
use iced::advanced::text::Renderer as TextRenderer;
use iced::event::Event;
use iced::keyboard::{key, Key, Modifiers};
use iced::mouse::ScrollDelta;
use iced::widget::{column, text};
use iced::{
    alignment, Background, Color, Element, Font, Length, Point, Rectangle, Size, Theme,
};

const FONT_SIZE: f32 = 15.0;
const LINE_HEIGHT: f32 = 20.0;
const GUTTER_PAD: f32 = 6.0;
const CURSOR_WIDTH: f32 = 2.0;
const MULTI_CLICK_MS: u64 = 400;
const SYNTAX_THEME: &str = "base16-eighties.dark";

const SAMPLE: &str = r#"// Plan 413 spike — 编辑这段文字验证:光标、选中、IME。
fn fibonacci(n: u32) -> u32 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

fn main() {
    let greeting = "你好,世界";
    for i in 0..10 {
        println!("{i}: {greeting} fib={}", fibonacci(i));
    }
}
"#;

#[derive(Debug, Clone)]
pub enum Message {
    /// Text or cursor changed — refresh the status line.
    Edited,
    /// F7: toggle vi mode (ViEditor passthrough).
    ToggleVi,
    /// F6: toggle soft wrap.
    ToggleWrap,
}

// ---------------------------------------------------------------------------
// Shared editor state (owned by the application, borrowed by the widget)
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct PreeditState {
    text: String,
}

struct EditorShared {
    editor: Mutex<ViEditor<'static, 'static>>,
    buffer: Weak<Buffer>,
    swash: Mutex<SwashCache>,
    /// Line-number layouts keyed by `(number, digit_width)`, laid out at
    /// font size 1.0 and scaled at raster time (keeps the cache tiny and
    /// resolution independent).
    gutter_layouts: Mutex<HashMap<(usize, usize), Vec<LayoutLine>>>,
    /// Rasterized gutter image (RGBA) — regenerated only when the editor
    /// `redraw()` flag is set or the gutter width changes.
    gutter_image: Mutex<Option<(adv_image::Handle, f32)>>,
    gutter_width: AtomicU32,
    focused: AtomicBool,
    dragging: AtomicBool,
    /// Fork-API downgrade: upstream iced has no `modified_key`, so we track
    /// modifiers ourselves from key/modifier events.
    modifiers: Mutex<Modifiers>,
    /// Fork-API downgrade: no `Style::scale_factor` in draw; track via
    /// `window::Event::Rescaled` (initial value limitation noted in README).
    scale_percent: AtomicU32,
    preedit: Mutex<Option<PreeditState>>,
    clicks: Mutex<Vec<Instant>>,
    vi_mode: AtomicBool,
    wrap: AtomicBool,
}

impl EditorShared {
    fn new() -> Arc<Self> {
        let mut fs = font_system().write().unwrap();
        let attrs = Attrs::new().family(Family::Monospace);

        let mut buffer = Buffer::new(fs.raw(), Metrics::new(FONT_SIZE, LINE_HEIGHT));
        buffer.set_text(fs.raw(), SAMPLE, &attrs, Shaping::Advanced, None);
        buffer.set_wrap(fs.raw(), Wrap::None);

        let arc = Arc::new(buffer);
        let weak = Arc::downgrade(&arc);

        // two-face provides the full syntax set + themes; the syntect
        // instance is shared with cosmic-text (single 5.x copy — spike
        // verification item 6).
        let syntax_system: &'static SyntaxSystem = Box::leak(Box::new({
            let mut system = SyntaxSystem::new();
            system.syntax_set = two_face::syntax::extra_no_newlines();
            let theme = two_face::theme::extra()
                .get(two_face::theme::EmbeddedThemeName::Base16EightiesDark)
                .clone();
            system.theme_set.themes.insert(SYNTAX_THEME.to_owned(), theme);
            system
        }));

        let mut syntax_editor =
            SyntaxEditor::new(arc, syntax_system, SYNTAX_THEME).expect("syntax theme missing");
        syntax_editor.syntax_by_extension("rs");
        let mut vi = ViEditor::new(syntax_editor);
        vi.set_passthrough(true);
        drop(fs);

        Arc::new(Self {
            editor: Mutex::new(vi),
            buffer: weak,
            swash: Mutex::new(SwashCache::new()),
            gutter_layouts: Mutex::new(HashMap::new()),
            gutter_image: Mutex::new(None),
            gutter_width: AtomicU32::new(0),
            focused: AtomicBool::new(false),
            dragging: AtomicBool::new(false),
            modifiers: Mutex::new(Modifiers::default()),
            scale_percent: AtomicU32::new(100),
            preedit: Mutex::new(None),
            clicks: Mutex::new(Vec::new()),
            vi_mode: AtomicBool::new(false),
            wrap: AtomicBool::new(false),
        })
    }

    fn status(&self) -> String {
        let editor = self.editor.lock().unwrap();
        let cursor = editor.cursor();
        let sel_len = editor
            .selection_bounds()
            .map(|(start, end)| {
                editor.with_buffer(|b| {
                    let mut bytes = end.index.saturating_sub(start.index);
                    for line in start.line..end.line {
                        bytes += b.lines.get(line).map(|l| l.text().len()).unwrap_or(0);
                    }
                    bytes
                })
            })
            .unwrap_or(0);
        let (chars, lines) = editor.with_buffer(|b| {
            (
                b.lines.iter().map(|l| l.text().chars().count()).sum::<usize>(),
                b.lines.len(),
            )
        });
        let mode = if self.vi_mode.load(Ordering::Relaxed) {
            "vi"
        } else {
            "normal"
        };
        let wrap = if self.wrap.load(Ordering::Relaxed) { "on" } else { "off" };
        let scale = self.scale_percent.load(Ordering::Relaxed);
        format!(
            "{mode} | wrap {wrap} | scale {scale}% | sel {sel_len}B | {chars} chars / {lines} lines | F6 wrap, F7 vi — Ln {}, Col {}",
            cursor.line + 1,
            cursor.index,
        )
    }
}

// ---------------------------------------------------------------------------
// The widget
// ---------------------------------------------------------------------------

struct SpikeEditor<'a> {
    shared: &'a EditorShared,
}

impl<'a> SpikeEditor<'a> {
    fn new(shared: &'a EditorShared) -> Self {
        Self { shared }
    }

    /// Buffer-local coordinates (relative to the text area origin) for a
    /// window-space point.
    fn to_buffer_pos(&self, bounds: Rectangle, pos: Point) -> (i32, i32) {
        let gutter = self.shared.gutter_width.load(Ordering::Relaxed) as f32;
        (
            (pos.x - bounds.x - gutter).max(0.0) as i32,
            (pos.y - bounds.y).max(0.0) as i32,
        )
    }

    fn run_action(&self, action: Action) {
        let s = self.shared;
        let mut editor = s.editor.lock().unwrap();
        let mut fs = font_system().write().unwrap();
        editor.action(fs.raw(), action);
    }

    /// Cursor motion with Ctrl (word/buffer jumps) and Shift (extend
    /// selection) handling. Always consumed.
    fn motion_action(&self, m: Motion, modifiers: &Modifiers) -> bool {
        let m = match (m, modifiers.control()) {
            (Motion::Left, true) => Motion::LeftWord,
            (Motion::Right, true) => Motion::RightWord,
            (Motion::Home, true) => Motion::BufferStart,
            (Motion::End, true) => Motion::BufferEnd,
            (m, _) => m,
        };
        let s = self.shared;
        let mut editor = s.editor.lock().unwrap();
        if modifiers.shift() {
            if matches!(editor.selection(), Selection::None) {
                let caret = editor.cursor();
                editor.set_selection(Selection::Normal(caret));
            }
        } else if !matches!(editor.selection(), Selection::None) {
            editor.set_selection(Selection::None);
        }
        let mut fs = font_system().write().unwrap();
        editor.action(fs.raw(), Action::Motion(m));
        true
    }

    /// Keyboard mapping. Returns `true` when the key was consumed.
    fn handle_key(
        &self,
        key: &Key,
        modifiers: &Modifiers,
        text: Option<&str>,
        clipboard: &mut dyn Clipboard,
    ) -> bool {
        let s = self.shared;

        // --- function keys (app-level toggles, handled before editor) ---
        if let Key::Named(key::Named::F6) = key {
            let on = !s.wrap.load(Ordering::Relaxed);
            s.wrap.store(on, Ordering::Relaxed);
            let mut editor = s.editor.lock().unwrap();
            let mut fs = font_system().write().unwrap();
            let wrap = if on { Wrap::Word } else { Wrap::None };
            editor.with_buffer_mut(|b| b.set_wrap(fs.raw(), wrap));
            return true;
        }
        if let Key::Named(key::Named::F7) = key {
            let on = !s.vi_mode.load(Ordering::Relaxed);
            s.vi_mode.store(on, Ordering::Relaxed);
            s.editor.lock().unwrap().set_passthrough(!on);
            return true;
        }

        // --- clipboard / history shortcuts ---
        if modifiers.control() {
            let c = match key {
                Key::Character(c) => c.as_str(),
                _ => "",
            };
            return match c {
                "c" | "C" => {
                    if let Some(selection) = s.editor.lock().unwrap().copy_selection() {
                        clipboard.write(ClipboardKind::Standard, selection);
                    }
                    true
                }
                "x" | "X" => {
                    let mut editor = s.editor.lock().unwrap();
                    if let Some(selection) = editor.copy_selection() {
                        clipboard.write(ClipboardKind::Standard, selection);
                        let mut fs = font_system().write().unwrap();
                        editor.action(fs.raw(), Action::Backspace);
                    }
                    true
                }
                "v" | "V" => {
                    if let Some(contents) = clipboard.read(ClipboardKind::Standard) {
                        s.editor.lock().unwrap().insert_string(&contents, None);
                    }
                    true
                }
                "z" | "Z" => {
                    let mut editor = s.editor.lock().unwrap();
                    if modifiers.shift() {
                        editor.redo();
                    } else {
                        editor.undo();
                    }
                    true
                }
                "y" | "Y" => {
                    s.editor.lock().unwrap().redo();
                    true
                }
                "a" | "A" => {
                    let mut editor = s.editor.lock().unwrap();
                    let end = editor.with_buffer(|b| {
                        Cursor::new(
                            b.lines.len() - 1,
                            b.lines.last().map(|l| l.text().len()).unwrap_or(0),
                        )
                    });
                    editor.set_cursor(Cursor::new(0, 0));
                    editor.set_selection(Selection::Normal(end));
                    true
                }
                _ => false,
            };
        }

        // --- plain keys: map to cosmic-text actions ---
        let action = match key {
            Key::Named(key::Named::ArrowLeft) => {
                return self.motion_action(Motion::Left, modifiers)
            }
            Key::Named(key::Named::ArrowRight) => {
                return self.motion_action(Motion::Right, modifiers)
            }
            Key::Named(key::Named::ArrowUp) => return self.motion_action(Motion::Up, modifiers),
            Key::Named(key::Named::ArrowDown) => {
                return self.motion_action(Motion::Down, modifiers)
            }
            Key::Named(key::Named::Home) => return self.motion_action(Motion::Home, modifiers),
            Key::Named(key::Named::End) => return self.motion_action(Motion::End, modifiers),
            Key::Named(key::Named::PageUp) => {
                return self.motion_action(Motion::PageUp, modifiers)
            }
            Key::Named(key::Named::PageDown) => {
                return self.motion_action(Motion::PageDown, modifiers)
            }
            Key::Named(key::Named::Enter) => Action::Enter,
            Key::Named(key::Named::Backspace) => Action::Backspace,
            Key::Named(key::Named::Delete) => Action::Delete,
            Key::Named(key::Named::Escape) => Action::Escape,
            Key::Named(key::Named::Tab) => {
                if modifiers.shift() {
                    Action::Unindent
                } else {
                    Action::Indent
                }
            }
            _ => {
                // Character input (dead keys, layouts): iced hands us the
                // composed `text` only when no modifiers other than shift
                // are involved.
                if let Some(text) = text.filter(|t| !t.is_empty()) {
                    if text.chars().any(|c| c.is_control()) {
                        return false;
                    }
                    let mut editor = s.editor.lock().unwrap();
                    let mut fs = font_system().write().unwrap();
                    for c in text.chars() {
                        editor.action(fs.raw(), Action::Insert(c));
                    }
                    return true;
                }
                return false;
            }
        };

        self.run_action(action);
        true
    }

    fn ime_request(&self, bounds: Rectangle) -> input_method::InputMethod<&'static str> {
        let s = self.shared;
        if !s.focused.load(Ordering::Relaxed) {
            return input_method::InputMethod::Disabled;
        }

        let editor = s.editor.lock().unwrap();
        let caret = editor.cursor_position().unwrap_or((0, 0));
        let gutter = s.gutter_width.load(Ordering::Relaxed) as f32;
        let position = Point::new(
            bounds.x + gutter + caret.0 as f32,
            bounds.y + caret.1 as f32,
        );

        input_method::InputMethod::Enabled {
            cursor: Rectangle::new(position, Size::new(1.0, LINE_HEIGHT)),
            purpose: input_method::Purpose::Normal,
            preedit: None,
        }
    }
}

impl Widget<Message, Theme, iced::Renderer> for SpikeEditor<'_> {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }

    fn layout(
        &mut self,
        _tree: &mut iced::advanced::widget::Tree,
        _renderer: &iced::Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> Node {
        Node::new(limits.max())
    }

    fn update(
        &mut self,
        _tree: &mut iced::advanced::widget::Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let s = self.shared;
        let bounds = layout.bounds();
        let over = cursor.position().is_some_and(|p| bounds.contains(p));

        match event {
            // ----- mouse -----
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) if over => {
                s.focused.store(true, Ordering::Relaxed);
                let (x, y) = self.to_buffer_pos(bounds, cursor.position().unwrap());

                let action = {
                    let mut clicks = s.clicks.lock().unwrap();
                    let now = Instant::now();
                    clicks.retain(|t| {
                        now.duration_since(*t) < Duration::from_millis(MULTI_CLICK_MS)
                    });
                    let count = clicks.len() + 1;
                    clicks.push(now);
                    match count.min(3) {
                        1 => Action::Click { x, y },
                        2 => Action::DoubleClick { x, y },
                        _ => Action::TripleClick { x, y },
                    }
                };

                s.dragging.store(true, Ordering::Relaxed);
                self.run_action(action);
                shell.request_redraw();
                shell.publish(Message::Edited);
                shell.capture_event();
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                s.dragging.store(false, Ordering::Relaxed);
            }
            Event::Mouse(mouse::Event::CursorMoved { .. })
                if s.dragging.load(Ordering::Relaxed) =>
            {
                if let Some(pos) = cursor.position() {
                    let (x, y) = self.to_buffer_pos(bounds, pos);
                    self.run_action(Action::Drag { x, y });
                    shell.request_redraw();
                    shell.publish(Message::Edited);
                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::WheelScrolled { delta }) if over => {
                let pixels = match delta {
                    ScrollDelta::Lines { y, .. } => y * LINE_HEIGHT,
                    ScrollDelta::Pixels { y, .. } => *y,
                };
                self.run_action(Action::Scroll { pixels });
                shell.request_redraw();
                shell.capture_event();
            }

            // ----- keyboard -----
            Event::Keyboard(iced::keyboard::Event::KeyPressed {
                key,
                modifiers,
                text,
                ..
            }) => {
                *s.modifiers.lock().unwrap() = *modifiers;
                if !s.focused.load(Ordering::Relaxed) {
                    return;
                }
                let handled = self.handle_key(key, modifiers, text.as_deref(), clipboard);
                if handled {
                    shell.request_redraw();
                    shell.publish(Message::Edited);
                    shell.capture_event();
                }
            }
            Event::Keyboard(iced::keyboard::Event::KeyReleased { modifiers, .. })
            | Event::Keyboard(iced::keyboard::Event::ModifiersChanged(modifiers)) => {
                *s.modifiers.lock().unwrap() = *modifiers;
            }

            // ----- IME -----
            Event::InputMethod(input_method::Event::Preedit(preedit, _)) => {
                *s.preedit.lock().unwrap() = if preedit.is_empty() {
                    None
                } else {
                    Some(PreeditState {
                        text: preedit.clone(),
                    })
                };
                shell.request_redraw();
                shell.capture_event();
            }
            Event::InputMethod(input_method::Event::Commit(content)) => {
                *s.preedit.lock().unwrap() = None;
                s.editor.lock().unwrap().insert_string(content, None);
                shell.request_redraw();
                shell.publish(Message::Edited);
                shell.capture_event();
            }
            Event::InputMethod(input_method::Event::Opened)
            | Event::InputMethod(input_method::Event::Closed) => {
                shell.request_redraw();
            }

            // ----- window -----
            Event::Window(iced::window::Event::RedrawRequested(_)) => {
                shell.request_input_method(&self.ime_request(bounds));
            }
            Event::Window(iced::window::Event::Rescaled(factor)) => {
                s.scale_percent
                    .store((*factor * 100.0).round() as u32, Ordering::Relaxed);
            }
            _ => {}
        }
    }

    fn draw(
        &self,
        _tree: &iced::advanced::widget::Tree,
        renderer: &mut iced::Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let s = self.shared;
        let bounds = layout.bounds();
        if bounds.width <= 1.0 || bounds.height <= 1.0 {
            return;
        }

        // Background panel.
        let mut editor = s.editor.lock().unwrap();
        let bg = to_iced(editor.background_color());
        let fg = to_iced(editor.foreground_color());
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                ..renderer::Quad::default()
            },
            Background::Color(bg),
        );

        // Shape text with the global font system (short write lock; the
        // RwLock is only ever taken on the UI thread, never re-entered).
        let mut fs = font_system().write().unwrap();
        let fs = fs.raw();

        // --- gutter width ---
        let line_count = editor.with_buffer(|b| b.lines.len()).max(1);
        let digits = digits_of(line_count);
        let mut gutter_width = 0.0f32;
        {
            let mut layouts = s.gutter_layouts.lock().unwrap();
            if let Some(line) = gutter_layout(&mut layouts, fs, 1, digits).first() {
                gutter_width = line.w * FONT_SIZE;
            }
        }
        let gutter_total = (gutter_width + GUTTER_PAD * 2.0).ceil();
        s.gutter_width.store(gutter_total as u32, Ordering::Relaxed);

        // --- size + shape ---
        editor.with_buffer_mut(|b| {
            b.set_metrics_and_size(
                fs,
                Metrics::new(FONT_SIZE, LINE_HEIGHT),
                Some((bounds.width - gutter_total).max(0.0)),
                Some(bounds.height),
            )
        });
        editor.shape_as_needed(fs, true);

        let text_origin = Point::new(bounds.x + gutter_total, bounds.y);
        let text_bounds = Rectangle::new(
            text_origin,
            Size::new((bounds.width - gutter_total).max(0.0), bounds.height),
        );

        // --- gutter raster (only when the buffer asked for a redraw) ---
        let needs_redraw = editor.redraw();
        {
            let mut image = s.gutter_image.lock().unwrap();
            let stale = match image.as_ref() {
                Some((_, w)) => (*w - gutter_total).abs() > 0.5,
                None => true,
            };
            if stale || needs_redraw {
                *image = rasterize_gutter(s, fs, &editor, digits, gutter_total);
            }
            if let Some((handle, _)) = image.clone() {
                let rect =
                    Rectangle::new(bounds.position(), Size::new(gutter_total, bounds.height));
                renderer.draw_image(
                    adv_image::Image {
                        filter_method: adv_image::FilterMethod::Nearest,
                        ..adv_image::Image::new(handle)
                    },
                    rect,
                    bounds,
                );
            }
        }

        // --- current line highlight (only without selection) ---
        if matches!(editor.selection(), Selection::None) {
            let cursor_line = editor.cursor().line;
            editor.with_buffer(|b| {
                for run in b.layout_runs() {
                    if run.line_i == cursor_line {
                        let rect = Rectangle::new(
                            Point::new(text_origin.x, text_origin.y + run.line_top),
                            Size::new(text_bounds.width, run.line_height),
                        );
                        renderer.fill_quad(
                            renderer::Quad {
                                bounds: rect,
                                ..renderer::Quad::default()
                            },
                            Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.04)),
                        );
                        break;
                    }
                }
            });
        }

        // --- selection quads ---
        if let Some((start, end)) = editor.selection_bounds() {
            editor.with_buffer(|b| {
                for run in b.layout_runs() {
                    if run.line_i < start.line || run.line_i > end.line {
                        continue;
                    }
                    let lo = if run.line_i == start.line { start.index } else { 0 };
                    let hi = if run.line_i == end.line {
                        end.index
                    } else {
                        run.text.len()
                    };
                    if hi <= lo {
                        continue;
                    }
                    if let (Some(x0), Some(x1)) = (index_x(&run, lo), index_x(&run, hi)) {
                        let rect = Rectangle::new(
                            Point::new(text_origin.x + x0.min(x1), text_origin.y + run.line_top),
                            Size::new((x1 - x0).abs().max(2.0), run.line_height),
                        );
                        renderer.fill_quad(
                            renderer::Quad {
                                bounds: rect,
                                ..renderer::Quad::default()
                            },
                            Background::Color(Color::from_rgba(0.35, 0.55, 0.85, 0.35)),
                        );
                    }
                }
            });
        }

        // --- text body: the whole point of the spike ---
        renderer.fill_raw(Raw {
            buffer: s.buffer.clone(),
            position: text_origin,
            color: fg,
            clip_bounds: text_bounds,
        });

        // --- caret ---
        if let Some((cx, cy)) = editor.cursor_position() {
            let rect = Rectangle::new(
                Point::new(text_origin.x + cx as f32, text_origin.y + cy as f32),
                Size::new(CURSOR_WIDTH, FONT_SIZE * 1.15),
            );
            renderer.fill_quad(
                renderer::Quad {
                    bounds: rect,
                    ..renderer::Quad::default()
                },
                Background::Color(to_iced(editor.cursor_color())),
            );

            // --- on-the-spot preedit overlay ---
            let preedit = s.preedit.lock().unwrap().clone();
            if let Some(preedit) = preedit {
                if !preedit.text.is_empty() {
                    let pos = Point::new(text_origin.x + cx as f32, text_origin.y + cy as f32);
                    renderer.fill_text(
                        adv_text::Text {
                            content: preedit.text,
                            bounds: Size::new(text_bounds.width.max(1.0), LINE_HEIGHT),
                            size: FONT_SIZE.into(),
                            line_height: adv_text::LineHeight::Absolute(LINE_HEIGHT.into()),
                            font: Font::MONOSPACE,
                            align_x: adv_text::Alignment::Left,
                            align_y: alignment::Vertical::Top,
                            wrapping: adv_text::Wrapping::Word,
                            shaping: adv_text::Shaping::Advanced,
                        },
                        pos,
                        Color::WHITE,
                        text_bounds,
                    );
                    let underline = Rectangle::new(
                        Point::new(pos.x, pos.y + FONT_SIZE * 1.15 - 2.0),
                        Size::new(text_bounds.width.min(240.0), 1.5),
                    );
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: underline,
                            ..renderer::Quad::default()
                        },
                        Background::Color(Color::WHITE),
                    );
                }
            }
        }

        if needs_redraw {
            editor.set_redraw(false);
        }
    }

    fn mouse_interaction(
        &self,
        _tree: &iced::advanced::widget::Tree,
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

// ---------------------------------------------------------------------------
// Gutter rasterization (CPU) and index→x mapping
// ---------------------------------------------------------------------------

fn digits_of(mut n: usize) -> usize {
    let mut digits = 1;
    while n >= 10 {
        n /= 10;
        digits += 1;
    }
    digits
}

/// Get (or lazily lay out at font size 1.0) the line-number glyph layout.
fn gutter_layout<'a>(
    cache: &'a mut HashMap<(usize, usize), Vec<LayoutLine>>,
    font_system: &mut FontSystem,
    number: usize,
    digits: usize,
) -> &'a Vec<LayoutLine> {
    cache.entry((number, digits)).or_insert_with(|| {
        let attrs = Attrs::new().family(Family::Monospace);
        let text = format!("{number:>digits$}");
        let mut line = BufferLine::new(
            text,
            LineEnding::None,
            AttrsList::new(&attrs),
            Shaping::Advanced,
        );
        line.layout(font_system, 1.0, None, Wrap::None, None, 8).to_vec()
    })
}

/// Rasterize the visible line numbers into an RGBA image.
///
/// Numbers are laid out once at font size 1.0 (cached), then scaled via
/// `physical((offset, line_y), FONT_SIZE)` — rasterization runs at the real
/// size so `FilterMethod::Nearest` stays crisp.
fn rasterize_gutter(
    s: &EditorShared,
    font_system: &mut FontSystem,
    editor: &ViEditor<'static, 'static>,
    digits: usize,
    gutter_total: f32,
) -> Option<(adv_image::Handle, f32)> {
    let width = gutter_total.ceil() as u32;
    let theme = editor.theme();
    let gutter_bg = theme
        .settings
        .gutter
        .map(|c| cosmic_text::Color::rgba(c.r, c.g, c.b, c.a))
        .unwrap_or_else(|| editor.background_color());
    let gutter_fg = theme
        .settings
        .gutter_foreground
        .map(|c| cosmic_text::Color::rgba(c.r, c.g, c.b, c.a))
        .unwrap_or_else(|| editor.foreground_color());

    let height = editor.with_buffer(|b| b.size().1.unwrap_or(0.0)).ceil() as u32;
    if width == 0 || height == 0 {
        return None;
    }

    let mut rgba = vec![0u8; (width as usize) * (height as usize) * 4];
    for px in rgba.chunks_exact_mut(4) {
        px[0] = gutter_bg.r();
        px[1] = gutter_bg.g();
        px[2] = gutter_bg.b();
        px[3] = gutter_bg.a();
    }

    let mut swash = s.swash.lock().unwrap();
    let mut layouts = s.gutter_layouts.lock().unwrap();

    editor.with_buffer(|buffer| {
        let mut last_number = 0;
        for run in buffer.layout_runs() {
            let number = run.line_i + 1;
            if number == last_number {
                continue;
            }
            last_number = number;

            let Some(layout) = gutter_layout(&mut layouts, font_system, number, digits).first()
            else {
                continue;
            };

            // Scale the 1.0-size layout to the real font size.
            let max_ascent = layout.max_ascent * FONT_SIZE;
            let max_descent = layout.max_descent * FONT_SIZE;
            let glyph_height = max_ascent + max_descent;
            let centering = (LINE_HEIGHT - glyph_height) / 2.0;
            let line_y = run.line_top + centering + max_ascent;

            for glyph in &layout.glyphs {
                let physical = glyph.physical((GUTTER_PAD, line_y), FONT_SIZE);
                swash.with_pixels(font_system, physical.cache_key, gutter_fg, |x, y, color| {
                    blend_pixel(&mut rgba, width, height, physical.x + x, physical.y + y, color);
                });
            }
        }
    });

    Some((adv_image::Handle::from_rgba(width, height, rgba), gutter_total))
}

/// Alpha-blend one pixel of a `cosmic_text::Color` onto the RGBA canvas.
fn blend_pixel(
    rgba: &mut [u8],
    width: u32,
    height: u32,
    x: i32,
    y: i32,
    color: cosmic_text::Color,
) {
    if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 {
        return;
    }
    let idx = (y as usize * width as usize + x as usize) * 4;
    let sa = color.a() as f32 / 255.0;
    let da = rgba[idx + 3] as f32 / 255.0;
    let out_a = sa + da * (1.0 - sa);
    if out_a <= 0.0 {
        return;
    }
    for (c, sc) in [(0, color.r()), (1, color.g()), (2, color.b())] {
        let dst = rgba[idx + c] as f32 / 255.0;
        let src = sc as f32 / 255.0;
        rgba[idx + c] = (((src * sa + dst * da * (1.0 - sa)) / out_a) * 255.0).round() as u8;
    }
    rgba[idx + 3] = (out_a * 255.0).round() as u8;
}

/// Byte index → x offset within a laid-out run (walks glyph clusters).
fn index_x(run: &LayoutRun, index: usize) -> Option<f32> {
    let mut prev_end = 0.0f32;
    for glyph in run.glyphs {
        if index < glyph.start {
            return Some(prev_end);
        }
        if index <= glyph.end {
            let cluster = &run.text[glyph.start..glyph.end];
            let total = cluster.chars().count().max(1) as f32;
            let before = run.text[glyph.start..index.min(glyph.end)].chars().count() as f32;
            return Some(glyph.x + (glyph.w / total) * before);
        }
        prev_end = glyph.x + glyph.w;
    }
    Some(prev_end)
}

fn to_iced(c: cosmic_text::Color) -> Color {
    Color::from_rgba8(c.r(), c.g(), c.b(), c.a() as f32 / 255.0)
}

// ---------------------------------------------------------------------------
// Application shell
// ---------------------------------------------------------------------------

struct Spike {
    shared: Arc<EditorShared>,
    status: String,
}

impl Spike {
    fn update(&mut self, message: Message) {
        match message {
            Message::Edited | Message::ToggleVi | Message::ToggleWrap => {
                // Toggles happen inside the widget; refresh status only.
                self.status = self.shared.status();
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        column![
            text(self.status.clone())
                .font(Font::MONOSPACE)
                .size(12)
                .color(Color::from_rgb(0.45, 0.45, 0.5)),
            Element::new(SpikeEditor::new(&self.shared)),
        ]
        .spacing(6)
        .padding(10)
        .into()
    }
}

impl Default for Spike {
    fn default() -> Self {
        let shared = EditorShared::new();
        let status = shared.status();
        Self { shared, status }
    }
}

fn main() -> iced::Result {
    iced::application(Spike::default, Spike::update, Spike::view)
        .title("Plan 413 — code editor spike")
        .window_size(Size::new(960.0, 680.0))
        .default_font(Font::MONOSPACE)
        .run()
}

// ---------------------------------------------------------------------------
// Headless pipeline tests (iced_test): drive the real widget update path.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use iced::keyboard::Modifiers as IModifiers;
    use iced_test::simulator::{self as sim, click, typewrite};

    /// A KeyPressed event with explicit modifiers (iced_test::press_key
    /// hardcodes empty modifiers).
    fn key_with_modifiers(key: Key, modifiers: IModifiers) -> Event {
        Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key: key.clone(),
            modified_key: key,
            physical_key: iced::keyboard::key::Physical::Unidentified(
                iced::keyboard::key::NativeCode::Unidentified,
            ),
            location: iced::keyboard::Location::Standard,
            modifiers,
            repeat: false,
            text: None,
        })
    }

    fn enter_event() -> Event {
        key_with_modifiers(Key::Named(key::Named::Enter), IModifiers::default())
    }

    fn home_event() -> Event {
        key_with_modifiers(Key::Named(key::Named::Home), IModifiers::default())
    }

    fn editor_text(shared: &EditorShared) -> String {
        shared.editor.lock().unwrap().with_buffer(|b| {
            b.lines
                .iter()
                .map(|l| l.text())
                .collect::<Vec<_>>()
                .join("\n")
        })
    }

    /// Rebuild the app view around the shared state for the simulator.
    /// Safety: the returned element borrows `shared` via a raw pointer
    /// lifetime extension; every test drops the simulator before `shared`.
    fn view_for(shared: &Arc<EditorShared>) -> Element<'static, Message> {
        iced::widget::column![
            text(String::new()).font(Font::MONOSPACE).size(12),
            Element::new(SpikeEditor::new(unsafe {
                &*(shared.as_ref() as *const EditorShared)
            })),
        ]
        .spacing(6)
        .padding(10)
        .into()
    }

    /// Build a simulator over the app view and click into the first line of
    /// the editor to focus it.
    fn focused(shared: &Arc<EditorShared>) -> sim::Simulator<'_, Message> {
        let mut ui = sim::simulator(view_for(shared));
        ui.point_at(Point::new(400.0, 45.0));
        let _ = ui.simulate(click());
        assert!(
            shared.focused.load(Ordering::Relaxed),
            "click should focus the editor"
        );
        ui
    }

    #[test]
    fn typewrite_inserts_text_at_cursor() {
        let shared = EditorShared::new();
        let mut ui = focused(&shared);
        let _ = ui.simulate([home_event()]);
        let _ = ui.simulate(typewrite("let x = 1;"));
        let text = editor_text(&shared);
        assert!(
            text.starts_with("let x = 1;"),
            "expected insertion at cursor, got: {text:?}"
        );
    }

    #[test]
    fn enter_splits_line_and_undo_restores() {
        let shared = EditorShared::new();
        let mut ui = focused(&shared);

        // Move to column 0, then Enter -> blank first line.
        let _ = ui.simulate([home_event()]);
        let _ = ui.simulate([enter_event()]);
        let _ = ui.simulate(typewrite("C"));
        {
            let editor = shared.editor.lock().unwrap();
            let first = editor.with_buffer(|b| b.lines[0].text().to_string());
            let second = editor.with_buffer(|b| b.lines[1].text().to_string());
            assert_eq!(first, "");
            assert!(
                second.starts_with("C// Plan 413"),
                "second line: {second:?}"
            );
        }

        // Ctrl+Z undoes the 'C'.
        let _ = ui.simulate([key_with_modifiers(
            Key::Character("z".into()),
            IModifiers::CTRL,
        )]);
        {
            let editor = shared.editor.lock().unwrap();
            let second = editor.with_buffer(|b| b.lines[1].text().to_string());
            assert!(
                second.starts_with("// Plan 413"),
                "after undo: {second:?}"
            );
        }
    }

    #[test]
    fn shift_arrow_extends_selection() {
        let shared = EditorShared::new();
        let mut ui = focused(&shared);

        // Home, then Shift+Right three times.
        let _ = ui.simulate([home_event()]);
        for _ in 0..3 {
            let _ = ui.simulate([key_with_modifiers(
                Key::Named(key::Named::ArrowRight),
                IModifiers::SHIFT,
            )]);
        }
        let editor = shared.editor.lock().unwrap();
        let (start, end) = editor
            .selection_bounds()
            .expect("shift+right should select");
        assert_eq!(start.index, 0);
        assert_eq!(end.index, 3);
    }

    #[test]
    fn ime_commit_inserts_text() {
        let shared = EditorShared::new();
        let mut ui = focused(&shared);
        let _ = ui.simulate([home_event()]);

        let commit = "你好".to_owned();
        let _ = ui.simulate([Event::InputMethod(input_method::Event::Commit(commit))]);
        let text = editor_text(&shared);
        assert!(
            text.starts_with("你好"),
            "IME commit should insert, got: {text:?}"
        );
    }

    #[test]
    fn click_moves_cursor_to_clicked_line() {
        let shared = EditorShared::new();
        let mut ui = sim::simulator(view_for(&shared));
        // Click deep into the editor area (~line 10).
        ui.point_at(Point::new(300.0, 300.0));
        let _ = ui.simulate(click());
        let editor = shared.editor.lock().unwrap();
        assert!(
            editor.cursor().line >= 5,
            "cursor line: {}",
            editor.cursor().line
        );
    }
}
