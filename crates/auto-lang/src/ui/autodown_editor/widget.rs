// Plan 019 Phase 3 ③ iced adapter — the autodown doc-editor widget.
//
// 蓝本 code_editor/iced/widget.rs。差异：无内部滚动/行号沟槽；高度按内容
// 实测（layout 期整形），宽随父约束；事件映射裁剪到 DocInput 面；鼠标
// press 严格过 is_over 门控（418 前车之鉴：越界捕获吞兄弟点击）。
//
// 此文件与本目录其余部分同 feature 门控（autodown × code-editor，其中
// autodown 已隐含 ui-iced）——是 autodown_editor 唯一触 iced 的位置。

use std::cell::RefCell;

use iced::advanced::text::Renderer as TextRenderer;
use iced::advanced::widget::Tree;
use iced::advanced::{
    input_method, layout::Node, mouse, renderer, text as adv_text, widget::Widget, Clipboard,
    Layout, Renderer as _, Shell,
};
use iced::event::Event;
use iced::keyboard::{self, key, Key, Modifiers};
use iced::mouse::ScrollDelta;
use iced::{Background, Color, Element, Font, Length, Point, Rectangle, Size, Theme};

use super::{
    AutodownEditorCore, DocDrawList, DocInput,
};
use crate::ui::code_editor::core::{EditorButton, EditorClipboard, EditorKey, EditorModifiers};
use crate::ui::code_editor::draw::{CaretDraw, PreeditDraw, Rect};
use crate::ui::code_editor::theme::Rgba;

/// Install the iced global font system as the core layer's font system
/// source (same instance as iced's own text pipeline). Idempotent
/// (first-writer-wins,与 code editor 同源调用).
fn install_font_system_source() {
    crate::ui::code_editor::core::set_font_system_call(|with| {
        let mut guard = iced::advanced::graphics::text::font_system().write().unwrap();
        with(guard.raw());
    });
}

struct IcedClipboard<'a> {
    inner: &'a mut dyn Clipboard,
}

impl EditorClipboard for IcedClipboard<'_> {
    fn read(&mut self) -> Option<String> {
        self.inner.read(iced::advanced::clipboard::Kind::Standard)
    }
    fn write(&mut self, text: &str) {
        self.inner.write(iced::advanced::clipboard::Kind::Standard, text.to_owned());
    }
}

/// 文档编辑器 widget。状态在全局注册表（key）；本体每帧可重建。
pub struct DocEditor<'a, M> {
    core: &'static AutodownEditorCore,
    on_change: Option<Box<dyn Fn() -> M + 'a>>,
    /// Plan 044 T2：块聚焦读出——焦点变化现场打包 (block_index, height)
    /// （高度取 core 布局快照，宿主零查询）。
    on_focus: Option<Box<dyn Fn(crate::ui::view::FocusMetrics) -> M + 'a>>,
    width: Length,
    /// 外层传入的基数前景色（正文基色；渲染器 lowering 由语义主题注入）。
    base_color: Rgba,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a, M: Clone> DocEditor<'a, M> {
    /// Get-or-create state for `key`（调用方先完成外部值 sync）。
    pub fn new(key: &str, base_color: Rgba) -> Self {
        install_font_system_source();
        Self {
            core: super::autodown_editor(key),
            on_change: None,
            on_focus: None,
            width: Length::Fill,
            base_color,
            _marker: std::marker::PhantomData,
        }
    }

    /// Fires whenever document text changed（payload 经
    /// `autodown_editor_text(key)` 读回）。
    pub fn on_change(mut self, f: impl Fn() -> M + 'a) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }

    /// Plan 044 T2：fires whenever block focus changed（点击/边界导航/失焦；
    /// 载荷 = 焦点块索引 + DocLayout 实测高，ghost 定高单源）。
    pub fn on_focus(mut self, f: impl Fn(crate::ui::view::FocusMetrics) -> M + 'a) -> Self {
        self.on_focus = Some(Box::new(f));
        self
    }

    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    pub fn core(&self) -> &'static AutodownEditorCore {
        self.core
    }

    fn publish(&self, out: &super::DocOutput, shell: &mut Shell<'_, M>) {
        if out.text_changed {
            if let Some(f) = &self.on_change {
                shell.publish(f());
            }
        }
        // Plan 044 T2：焦点变化现场打包 (block_index, height)。高度从
        // core 布局快照取（render_frame 每帧写回，点击后必已就绪）；
        // 失焦变体 block=None、height=0。
        if out.focus_changed {
            if let Some(f) = &self.on_focus {
                let block = self.core.focused_block();
                let height = block
                    .and_then(|i| self.core.block_rects().get(i).map(|r| r.h))
                    .unwrap_or(0.0);
                shell.publish(f(crate::ui::view::FocusMetrics { block, height }));
            }
        }
    }

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
            Key::Character(c) => c.chars().next().map(EditorKey::Char).unwrap_or_else(|| EditorKey::Other(c.to_string())),
            other => EditorKey::Other(format!("{other:?}")),
        }
    }

    fn map_modifiers(m: &Modifiers) -> EditorModifiers {
        EditorModifiers { shift: m.shift(), control: m.control(), alt: m.alt(), logo: m.logo() }
    }

    fn ime_request(&self, last_caret: Option<Rect>, bounds: Rectangle) -> input_method::InputMethod<&'static str> {
        let Some(caret) = last_caret else {
            return input_method::InputMethod::Disabled;
        };
        input_method::InputMethod::Enabled {
            cursor: Rectangle::new(
                Point::new(bounds.x + caret.x, bounds.y + caret.y),
                Size::new(caret.w.max(1.0), caret.h),
            ),
            purpose: input_method::Purpose::Normal,
            preedit: None,
        }
    }

    /// 内容高度实测（layout/draw 共用；整形缓存落在 cosmic 内部）。
    fn measure(&self, viewport_w: f32) -> super::DocFrame {
        crate::ui::code_editor::core::with_font_system(|fs| {
            self.core.render_frame(fs, viewport_w, self.base_color)
        })
    }
}

/// Per-widget-tree transient frame cache（draw 写、update 的 IME 路径读）。
struct WidgetState {
    last_frame: RefCell<Option<std::sync::Arc<DocDrawList>>>,
    last_caret: RefCell<Option<Rect>>,
}

impl<M: Clone> Widget<M, Theme, iced::Renderer> for DocEditor<'_, M> {
    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        iced::advanced::widget::tree::Tag::of::<WidgetState>()
    }

    fn state(&self) -> iced::advanced::widget::tree::State {
        iced::advanced::widget::tree::State::new(WidgetState {
            last_frame: RefCell::new(None),
            last_caret: RefCell::new(None),
        })
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, Length::Shrink)
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &iced::Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> Node {
        // Fill 宽语义下解析（418 修复口径）；高为实测内容高（Shrink）。
        let max = limits.max();
        let width = limits.resolve(self.width, Length::Shrink, max).width;
        let frame = self.measure(width.max(1.0));
        Node::new(Size::new(width, frame.height.max(BODY_MIN_H)))
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, M>,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        if self.core.take_external_dirty() {
            if let Some(f) = &self.on_change {
                shell.publish(f());
            }
        }

        let local = |p: Point| (p.x - bounds.x, p.y - bounds.y);
        let input = match event {
            Event::Mouse(mouse::Event::ButtonPressed(button)) => {
                if !cursor.is_over(bounds) {
                    return; // 越界不捕获：点击继续传播给兄弟控件（418 教训）
                }
                let button = match button {
                    mouse::Button::Left => EditorButton::Left,
                    mouse::Button::Right => EditorButton::Right,
                    mouse::Button::Middle => EditorButton::Middle,
                    _ => EditorButton::Other,
                };
                cursor.position().map(|p| {
                    let (x, y) = local(p);
                    DocInput::MousePressed { button, x, y }
                })
            }
            Event::Mouse(mouse::Event::ButtonReleased(button)) => {
                if !cursor.is_over(bounds) {
                    return;
                }
                let b = match button {
                    mouse::Button::Left => EditorButton::Left,
                    mouse::Button::Right => EditorButton::Right,
                    mouse::Button::Middle => EditorButton::Middle,
                    _ => EditorButton::Other,
                };
                Some(DocInput::MouseReleased { button: b })
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                // 拖选移动通路（PLAN-048 T3）：core 侧以 drag 状态门控，
                // 非拖选零操作零捕获；界外丢弃（413 同款取舍：拖选停在边缘）。
                if !cursor.is_over(bounds) {
                    return;
                }
                let (x, y) = local(*position);
                Some(DocInput::MouseDragged { x, y })
            }
            Event::Mouse(mouse::Event::WheelScrolled { .. }) => None, // 页面滚动透传
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, text, .. }) => {
                Some(DocInput::KeyPressed {
                    key: Self::map_key(key),
                    text: text.as_ref().map(|t| t.to_string()),
                    modifiers: Self::map_modifiers(modifiers),
                })
            }
            Event::Keyboard(keyboard::Event::ModifiersChanged(m)) => {
                Some(DocInput::ModifiersChanged(Self::map_modifiers(m)))
            }
            Event::InputMethod(ime) => match ime {
                input_method::Event::Opened => None,
                input_method::Event::Preedit(p, _) => Some(DocInput::ImePreedit(p.clone())),
                input_method::Event::Commit(c) => Some(DocInput::ImeCommit(c.clone())),
                input_method::Event::Closed => Some(DocInput::ImeClosed),
            },
            Event::Window(iced::window::Event::Focused) => Some(DocInput::FocusGained),
            Event::Window(iced::window::Event::Unfocused) => Some(DocInput::FocusLost),
            _ => None,
        };

        let Some(input) = input else {
            // 滚轮每帧高频——直接放行，零分配。
            return;
        };

        // 未聚焦且非点击类输入时忽略按键噪声（焦点经 MousePressed 建立）。
        if matches!(
            input,
            DocInput::KeyPressed { .. } | DocInput::ImePreedit(_) | DocInput::ImeCommit(_)
        ) && self.core.focused_block().is_none()
        {
            return;
        }

        let out = crate::ui::code_editor::core::with_font_system(|fs| {
            let mut clip = IcedClipboard { inner: clipboard };
            self.core.handle_input(fs, input, &mut clip)
        });

        if out.request_redraw {
            shell.request_redraw();
        }
        if out.captured {
            shell.capture_event();
        }
        self.publish(&out, shell);
        let _ = (&tree, &ScrollDelta::Lines { x: 0.0, y: 0.0 }); // 保持导入面稳定
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

        let frame = self.measure(bounds.width.max(1.0));
        let list = &frame.list;

        *state.last_frame.borrow_mut() = Some(std::sync::Arc::new(list.clone()));
        *state.last_caret.borrow_mut() = list.caret.as_ref().map(|c| c.rect);

        let to_color = |c: Rgba| {
            Color::from_rgba(
                c.r.clamp(0.0, 1.0),
                c.g.clamp(0.0, 1.0),
                c.b.clamp(0.0, 1.0),
                c.a.clamp(0.0, 1.0),
            )
        };

        // chrome 填充（PLAN-041 T3：fence header/边框/底色，家族单源）——
        // 先于选区与文本绘制。
        for (rect, color) in &list.fills {
            fill_quad(renderer, to_rect(bounds, *rect), to_color(*color));
        }

        for (rect, color) in &list.selection {
            fill_quad(renderer, to_rect(bounds, *rect), to_color(*color));
        }

        // 样式化文本段。粗斜体走 iced Font weight/style —— 与 buffer 自身
        // 的整形家族不必逐字形一致（v1 观感目标；像素级 advance 对齐登记
        // 余量）。strike 以近似宽度横杆绘制。
        for run in &list.runs {
            let font = run_font(run.mono, run.bold, run.italic);
            renderer.fill_text(
                adv_text::Text {
                    content: run.text.clone(),
                    bounds: Size::new(f32::MAX / 2.0, run.line_height),
                    size: run.size.into(),
                    line_height: adv_text::LineHeight::Absolute(run.line_height.into()),
                    font,
                    align_x: adv_text::Alignment::Left,
                    align_y: iced::alignment::Vertical::Top,
                    wrapping: adv_text::Wrapping::None,
                    shaping: adv_text::Shaping::Advanced,
                },
                Point::new(bounds.x + run.x, bounds.y + run.y),
                to_color(run.color),
                bounds,
            );
            if run.strike {
                let approx_w = run.text.chars().count() as f32 * run.size * if run.mono { 0.6 } else { 0.52 };
                let mid_y = run.y + run.size * 0.55;
                fill_quad(
                    renderer,
                    Rectangle::new(Point::new(bounds.x + run.x, bounds.y + mid_y), Size::new(approx_w, 1.4)),
                    to_color(run.color),
                );
            }
            if run.underline {
                let approx_w = run.text.chars().count() as f32 * run.size * if run.mono { 0.6 } else { 0.52 };
                let base_y = run.y + run.size * 0.88;
                fill_quad(
                    renderer,
                    Rectangle::new(Point::new(bounds.x + run.x, bounds.y + base_y), Size::new(approx_w, 1.2)),
                    to_color(run.color),
                );
            }
        }

        if let Some((rect, color)) = list.focus_frame {
            stroke_rect(renderer, bounds, rect, to_color(color));
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
                    line_height: adv_text::LineHeight::Absolute((preedit.font_size * 4.0 / 3.0).into()),
                    font: Font::default(),
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
        // CaretDraw/PreeditDraw 类型再导出用于状态缓存（防未用告警）。
        let _ = (std::mem::size_of::<CaretDraw>(), std::mem::size_of::<PreeditDraw>());
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

const BODY_MIN_H: f32 = 24.0;

fn run_font(mono: bool, bold: bool, italic: bool) -> Font {
    let family = if mono { mono_iced_font() } else { Font::default() };
    let family = if bold {
        Font { weight: iced::font::Weight::Bold, ..family }
    } else {
        family
    };
    if italic {
        Font { style: iced::font::Style::Italic, ..family }
    } else {
        family
    }
}

fn mono_iced_font() -> Font {
    if cfg!(windows) {
        Font::with_name("Consolas")
    } else {
        Font::MONOSPACE
    }
}

fn to_rect(origin: Rectangle, r: Rect) -> Rectangle {
    Rectangle::new(Point::new(origin.x + r.x, origin.y + r.y), Size::new(r.w, r.h))
}

fn fill_quad(renderer: &mut iced::Renderer, rect: Rectangle, color: Color) {
    renderer.fill_quad(
        renderer::Quad { bounds: rect, ..renderer::Quad::default() },
        Background::Color(color),
    );
}

/// 焦点框：四条 1px 边（iced 无描边 quad 原语，v1 用细 quad 拼）。
fn stroke_rect(renderer: &mut iced::Renderer, origin: Rectangle, r: Rect, color: Color) {
    let px = 1.0f32;
    let abs = to_rect(origin, r);
    let edges = [
        Rectangle::new(abs.position(), Size::new(abs.width, px)),
        Rectangle::new(Point::new(abs.x, abs.y + abs.height - px), Size::new(abs.width, px)),
        Rectangle::new(Point::new(abs.x, abs.y), Size::new(px, abs.height)),
        Rectangle::new(Point::new(abs.x + abs.width - px, abs.y), Size::new(px, abs.height)),
    ];
    for e in edges {
        fill_quad(renderer, e, color);
    }
}

impl<'a, M: Clone + 'a> From<DocEditor<'a, M>> for Element<'a, M> {
    fn from(editor: DocEditor<'a, M>) -> Self {
        Element::new(editor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::autodown_editor::{autodown_editor, storage_key, DocInput};

    const WHITE: Rgba = Rgba { r: 1., g: 1., b: 1., a: 1. };

    /// PLAN-044 T2：消息可达——模拟点击第一块 → focus_changed →
    /// publish 把 on_focus 消息发进 shell，载荷 (block, height) 与
    /// core 布局快照一致（高度 > 0 即 layout 现场取值）。
    #[test]
    fn on_focus_message_reaches_shell_on_click() {
        // 测试字体系统引导（镜像 core.rs tests::run_fs）：先装 callback
        // 再走 with_font_system 路径。
        static FS: std::sync::OnceLock<std::sync::RwLock<cosmic_text::FontSystem>> =
            std::sync::OnceLock::new();
        crate::ui::code_editor::core::set_font_system_call(|with| {
            let mut guard =
                FS.get_or_init(|| std::sync::RwLock::new(cosmic_text::FontSystem::new())).write().unwrap();
            with(&mut guard);
        });
        let sk = storage_key("p044_t2_focus_msg");
        let core = autodown_editor(&sk);
        // sync_external 内部自调 with_font_system——必须在外层调用（同线程
        // RwLock 写锁不可重入），否则死锁。
        core.sync_external("甲段。\n\n乙段。\n", true);
        crate::ui::code_editor::core::with_font_system(|fs| {
            let _ = core.render_frame(fs, 400.0, WHITE);
        });
        // 点击第一块中部 → 建焦块 0。
        let out = crate::ui::code_editor::core::with_font_system(|fs| {
            core.handle_input(
                fs,
                DocInput::MousePressed { button: EditorButton::Left, x: 10.0, y: 8.0 },
                &mut crate::ui::code_editor::core::NullClipboard,
            )
        });
        assert!(out.focus_changed, "{out:?}");
        assert_eq!(core.focused_block(), Some(0));

        let expected_h = core.block_rects()[0].h;
        assert!(expected_h > 0.0);
        #[derive(Debug, Clone, PartialEq)]
        enum FocusMsg {
            F(Option<usize>, f32),
        }
        let editor = DocEditor::<FocusMsg>::new(&sk, WHITE)
            .on_focus(|m| FocusMsg::F(m.block, m.height));
        let mut msgs: Vec<FocusMsg> = Vec::new();
        let mut shell = Shell::new(&mut msgs);
        editor.publish(&out, &mut shell);
        assert_eq!(msgs, vec![FocusMsg::F(Some(0), expected_h)]);
    }
}
