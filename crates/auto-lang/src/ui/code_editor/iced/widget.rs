// Plan 413 §3.1 ③ iced adapter — the `CodeEditor` widget.
//
// The only place the editor touches iced: `update` maps iced events onto
// the backend-neutral `EditorInput`, `draw` lowers the `EditorDrawList` to
// `fill_quad` / `fill_raw` / image calls. Everything below the draw list
// (state machine, shaping, geometry) lives in the core layer.
//
// License: MIT. Architecture inspired by cosmic-edit (GPL-3.0, System76);
// original implementation.

use std::cell::RefCell;

use iced::advanced::graphics::text::{Raw, Renderer as RawTextRenderer};
use iced::advanced::image::Renderer as ImageRenderer;
use iced::advanced::text::Renderer as TextRenderer;
use iced::advanced::widget::Tree;
use iced::advanced::{
    image as adv_image, input_method, layout::Node, mouse, renderer, text as adv_text,
    widget::Widget, Clipboard, Layout, Renderer as _, Shell,
};
use iced::event::Event;
use iced::keyboard::{self, key, Key, Modifiers};
use iced::mouse::ScrollDelta;
use iced::{Background, Color, Element, Font, Length, Point, Rectangle, Size, Theme};

use crate::ui::code_editor::core::{
    code_editor, CodeEditorConfig, CodeEditorCore, CoreOutput, EditorButton, EditorClipboard,
    EditorInput, EditorKey, EditorModifiers,
};
use crate::ui::code_editor::draw;
use crate::ui::code_editor::iced::gutter::GutterCache;
use crate::ui::code_editor::theme;

/// Install the iced global font system as the core layer's font system
/// source (same instance as iced's own text pipeline — single cosmic-text
/// stack). Idempotent.
fn install_font_system_source() {
    crate::ui::code_editor::core::set_font_system_call(|with| {
        let mut guard = iced::advanced::graphics::text::font_system().write().unwrap();
        with(guard.raw());
    });
}

/// Adapter: iced clipboard → backend-neutral clipboard.
struct IcedClipboard<'a> {
    inner: &'a mut dyn Clipboard,
}

impl EditorClipboard for IcedClipboard<'_> {
    fn read(&mut self) -> Option<String> {
        self.inner.read(iced::advanced::clipboard::Kind::Standard)
    }
    fn write(&mut self, text: &str) {
        self.inner
            .write(iced::advanced::clipboard::Kind::Standard, text.to_owned());
    }
}

/// Editable code editor widget (syntax highlighting, line numbers, soft
/// wrap, vi/undo, IME).
///
/// State lives in the global registry under `key`; the widget itself is
/// rebuilt freely every frame (TEXTAREA_CONTENTS pattern). The message
/// callbacks read payloads through `code_editor_text(key)` /
/// `code_editor_cursor(key)`.
pub struct CodeEditor<'a, M> {
    core: &'static CodeEditorCore,
    on_change: Option<Box<dyn Fn() -> M + 'a>>,
    on_cursor: Option<Box<dyn Fn() -> M + 'a>>,
    on_context_menu: Option<Box<dyn Fn(Option<(f32, f32)>) -> M + 'a>>,
    width: Length,
    height: Length,
}

impl<'a, M: Clone> CodeEditor<'a, M> {
    /// Get-or-create the editor state for `key` (config is diffed in).
    pub fn new(key: &str, config: &CodeEditorConfig) -> Self {
        install_font_system_source();
        Self {
            core: code_editor(key, config),
            on_change: None,
            on_cursor: None,
            on_context_menu: None,
            width: Length::Fill,
            height: Length::Fill,
        }
    }

    /// Fires whenever the text changes (payload via `code_editor_text`).
    pub fn on_change(mut self, f: impl Fn() -> M + 'a) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }

    /// Fires whenever the caret or selection moves (payload via
    /// `code_editor_cursor`).
    pub fn on_cursor(mut self, f: impl Fn() -> M + 'a) -> Self {
        self.on_cursor = Some(Box::new(f));
        self
    }

    /// Fires on right-click (`Some(x, y)` widget-local) and on other
    /// buttons (`None`, used to close an open menu).
    pub fn on_context_menu(mut self, f: impl Fn(Option<(f32, f32)>) -> M + 'a) -> Self {
        self.on_context_menu = Some(Box::new(f));
        self
    }

    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    /// The editor state (for programmatic access from app code).
    pub fn core(&self) -> &'static CodeEditorCore {
        self.core
    }

    fn publish(&self, out: &CoreOutput, shell: &mut Shell<'_, M>) {
        if out.text_changed {
            if let Some(f) = &self.on_change {
                shell.publish(f());
            }
        }
        if out.cursor_changed {
            if let Some(f) = &self.on_cursor {
                shell.publish(f());
            }
        }
        if let Some(pos) = out.context_menu {
            if let Some(f) = &self.on_context_menu {
                shell.publish(f(Some(pos)));
            }
        }
    }

    /// Convert an iced key into the backend-neutral form.
    fn map_key(key: &Key) -> EditorKey {
        match key {
            Key::Named(key::Named::ArrowLeft) => EditorKey::Left,
            Key::Named(key::Named::ArrowRight) => EditorKey::Right,
            Key::Named(key::Named::ArrowUp) => EditorKey::Up,
            Key::Named(key::Named::ArrowDown) => EditorKey::Down,
            Key::Named(key::Named::Home) => EditorKey::Home,
            Key::Named(key::Named::End) => EditorKey::End,
            Key::Named(key::Named::PageUp) => EditorKey::PageUp,
            Key::Named(key::Named::PageDown) => EditorKey::PageDown,
            Key::Named(key::Named::Enter) => EditorKey::Enter,
            Key::Named(key::Named::Backspace) => EditorKey::Backspace,
            Key::Named(key::Named::Delete) => EditorKey::Delete,
            Key::Named(key::Named::Escape) => EditorKey::Escape,
            Key::Named(key::Named::Tab) => EditorKey::Tab,
            Key::Character(c) => c
                .chars()
                .next()
                .map(EditorKey::Char)
                .unwrap_or_else(|| EditorKey::Other(c.to_string())),
            other => EditorKey::Other(format!("{other:?}")),
        }
    }

    fn map_modifiers(m: &Modifiers) -> EditorModifiers {
        EditorModifiers {
            shift: m.shift(),
            control: m.control(),
            alt: m.alt(),
            logo: m.logo(),
        }
    }

    fn ime_request(&self, bounds: Rectangle) -> input_method::InputMethod<&'static str> {
        if !self.core.is_focused() {
            return input_method::InputMethod::Disabled;
        }
        let caret = self.core.caret_rect().unwrap_or(draw::Rect::new(0.0, 0.0, 1.0, 20.0));
        input_method::InputMethod::Enabled {
            cursor: Rectangle::new(
                Point::new(bounds.x + caret.x, bounds.y + caret.y),
                Size::new(caret.w.max(1.0), caret.h),
            ),
            purpose: input_method::Purpose::Normal,
            preedit: None,
        }
    }
}

/// Widget-tree state: the gutter raster cache. Only touched on the UI
/// thread (draw), behind a RefCell because draw receives `&Tree`.
struct WidgetState {
    gutter: RefCell<GutterCache>,
}

impl<M: Clone> Widget<M, Theme, iced::Renderer> for CodeEditor<'_, M> {
    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        iced::advanced::widget::tree::Tag::of::<WidgetState>()
    }

    fn state(&self) -> iced::advanced::widget::tree::State {
        iced::advanced::widget::tree::State::new(WidgetState {
            gutter: RefCell::new(GutterCache::default()),
        })
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &iced::Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> Node {
        Node::new(limits.max())
    }

    fn update(
        &mut self,
        _tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, M>,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        // Keep the syntect highlight theme in sync with the semantic theme
        // source (no-op when unchanged).
        let (dark, accent) = theme::theme_source();
        self.core.sync_syntax_theme(dark, &accent);

        let local = |p: Point| (p.x - bounds.x, p.y - bounds.y);
        let input = match event {
            Event::Mouse(mouse::Event::ButtonPressed(button)) => {
                let button = match button {
                    mouse::Button::Left => EditorButton::Left,
                    mouse::Button::Right => EditorButton::Right,
                    mouse::Button::Middle => EditorButton::Middle,
                    _ => EditorButton::Other,
                };
                cursor.position().map(|p| {
                    let (x, y) = local(p);
                    EditorInput::MousePressed { button, x, y }
                })
            }
            Event::Mouse(mouse::Event::ButtonReleased(button)) => {
                let button = match button {
                    mouse::Button::Left => EditorButton::Left,
                    mouse::Button::Right => EditorButton::Right,
                    mouse::Button::Middle => EditorButton::Middle,
                    _ => EditorButton::Other,
                };
                Some(EditorInput::MouseReleased { button })
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                let (x, y) = local(*position);
                Some(EditorInput::MouseMoved { x, y })
            }
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                if !cursor.is_over(bounds) {
                    return;
                }
                let (dx, dy) = match delta {
                    ScrollDelta::Lines { x, y } => (*x, *y),
                    ScrollDelta::Pixels { x, y } => (x / 8.0, y / 8.0),
                };
                // Wheel events carry no modifiers; the core tracks the
                // latest window modifiers.
                Some(EditorInput::WheelScrolled { dx, dy, shift: false })
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key, modifiers, text, ..
            }) => Some(EditorInput::KeyPressed {
                key: Self::map_key(key),
                text: text.as_ref().map(|t| t.to_string()),
                modifiers: Self::map_modifiers(modifiers),
            }),
            Event::Keyboard(keyboard::Event::KeyReleased { .. }) => {
                Some(EditorInput::KeyReleased)
            }
            Event::Keyboard(keyboard::Event::ModifiersChanged(m)) => {
                Some(EditorInput::ModifiersChanged(Self::map_modifiers(m)))
            }
            Event::InputMethod(ime) => match ime {
                input_method::Event::Opened => Some(EditorInput::ImeOpened),
                input_method::Event::Preedit(p, _) => Some(EditorInput::ImePreedit(p.clone())),
                input_method::Event::Commit(c) => Some(EditorInput::ImeCommit(c.clone())),
                input_method::Event::Closed => Some(EditorInput::ImeClosed),
            },
            Event::Window(iced::window::Event::Focused) => Some(EditorInput::FocusGained),
            Event::Window(iced::window::Event::Unfocused) => Some(EditorInput::FocusLost),
            Event::Window(iced::window::Event::Rescaled(f)) => Some(EditorInput::Rescaled(*f)),
            Event::Window(iced::window::Event::RedrawRequested(_)) => {
                // Refresh the IME input area every frame (cheap when the
                // rect is unchanged).
                shell.request_input_method(&self.ime_request(bounds));
                return;
            }
            _ => None,
        };

        let Some(input) = input else { return };

        let out = crate::ui::code_editor::core::with_font_system(|fs| {
            let mut clipboard_adapter = IcedClipboard { inner: clipboard };
            self.core.handle_input(fs, input, &mut clipboard_adapter)
        });

        if out.request_redraw {
            shell.request_redraw();
        }
        if out.captured {
            shell.capture_event();
        }
        self.publish(&out, shell);
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let state = tree.state.downcast_ref::<WidgetState>();

        // Theme sync also guards draw-only paths (e.g. the very first frame
        // before any event reached update).
        let (dark, accent) = theme::theme_source();
        self.core.sync_syntax_theme(dark, &accent);

        // Render (shape + geometry) and the gutter raster share one font
        // system guard — single short critical section.
        let list = crate::ui::code_editor::core::with_font_system(|fs| {
            let list = crate::ui::code_editor::core::render::render(
                self.core,
                fs,
                bounds.width,
                bounds.height,
                None,
            );
            if let Some(section) = &list.gutter {
                let mut gutter = state.gutter.borrow_mut();
                if let Some((handle, _)) = gutter.image(section, fs, list.revision) {
                    renderer.draw_image(
                        adv_image::Image {
                            filter_method: adv_image::FilterMethod::Nearest,
                            ..adv_image::Image::new(handle)
                        },
                        to_rect(bounds, section.bounds),
                        bounds,
                    );
                }
            }
            list
        });

        let to_color = |c: theme::Rgba| {
            Color::from_rgba(
                c.r.clamp(0.0, 1.0),
                c.g.clamp(0.0, 1.0),
                c.b.clamp(0.0, 1.0),
                c.a.clamp(0.0, 1.0),
            )
        };

        if let Some((rect, color)) = list.background {
            fill_quad(renderer, to_rect(bounds, rect), to_color(color));
        }
        if let Some((rect, color)) = list.current_line {
            fill_quad(renderer, to_rect(bounds, rect), to_color(color));
        }
        for (rect, color) in &list.selection {
            fill_quad(renderer, to_rect(bounds, *rect), to_color(*color));
        }

        if let Some(text) = &list.text {
            renderer.fill_raw(Raw {
                buffer: text.buffer.clone(),
                position: Point::new(bounds.x + text.origin.x, bounds.y + text.origin.y),
                color: to_color(text.color),
                clip_bounds: to_rect(bounds, text.clip),
            });
        }

        if let Some(caret) = &list.caret {
            fill_quad(renderer, to_rect(bounds, caret.rect), to_color(caret.color));
        }

        if let Some(preedit) = &list.preedit {
            renderer.fill_text(
                adv_text::Text {
                    content: preedit.text.clone(),
                    bounds: Size::new(bounds.width, preedit.font_size * 1.4),
                    size: preedit.font_size.into(),
                    line_height: adv_text::LineHeight::Absolute(
                        (preedit.font_size * 4.0 / 3.0).into(),
                    ),
                    font: Font::MONOSPACE,
                    align_x: adv_text::Alignment::Left,
                    align_y: iced::alignment::Vertical::Top,
                    wrapping: adv_text::Wrapping::Word,
                    shaping: adv_text::Shaping::Advanced,
                },
                Point::new(bounds.x + preedit.origin.x, bounds.y + preedit.origin.y),
                to_color(preedit.color),
                bounds,
            );
            fill_quad(renderer, to_rect(bounds, preedit.underline), to_color(preedit.color));
        }

        if let Some(sb) = &list.scrollbar_v {
            fill_quad(renderer, to_rect(bounds, sb.thumb), to_color(sb.color));
        }
        if let Some(sb) = &list.scrollbar_h {
            fill_quad(renderer, to_rect(bounds, sb.thumb), to_color(sb.color));
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

fn to_rect(origin: Rectangle, r: draw::Rect) -> Rectangle {
    Rectangle::new(
        Point::new(origin.x + r.x, origin.y + r.y),
        Size::new(r.w, r.h),
    )
}

fn fill_quad(renderer: &mut iced::Renderer, rect: Rectangle, color: Color) {
    renderer.fill_quad(
        renderer::Quad {
            bounds: rect,
            ..renderer::Quad::default()
        },
        Background::Color(color),
    );
}

impl<'a, M: Clone + 'a> From<CodeEditor<'a, M>> for Element<'a, M> {
    fn from(editor: CodeEditor<'a, M>) -> Self {
        Element::new(editor)
    }
}
