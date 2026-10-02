// PLAN-731 T-00: S5 段根包装探针（帧尾段增量化件——layout/draw 分段计量）。
//
// 纯委托根 wrapper（PointerPressArea 同型）：iced 0.14 内部段（UserInterface::
// build 的全树 layout、RedrawRequested 臂的全树 draw）无公开钩子，但两段
// 均自根 widget 递归下行——根级 wrapper 的 layout()/draw() 括号即得全树
// 时段（S5a/S5c，SD-B 增面）。门控纪律承袭：AUTO_FRAME_BENCH 未设时
// `Instant::now` 都不取（OnceLock 布尔读+分支），行为零扰动（不捕获事件、
// 不改内层语义、不参与命中——716/725 同门）。
//
// 计量落点：
// - layout() → frame_segments::note_s5_layout（含非编辑器文本件布局期整形）。
// - draw()   → frame_segments::note_s5_draw（含编辑器 render/光栅与全树绘制
//   入队）。present 侧 wgpu flush 不在内（谱分析残差单列）。
// - 纯滚动帧不重建 view → 无 layout 调用，s5_layout=0（如实反映）。

use iced::advanced::layout::{self, Layout};
use iced::advanced::mouse;
use iced::advanced::renderer;
use iced::advanced::widget::{self, Tree, Widget};
use iced::advanced::{Clipboard, Shell};
use iced::event::Event;
use iced::{Element, Rectangle, Size, Vector};

/// S5 分段计量根 wrapper（窗口根单包装，纯委托）。
pub struct FrameProbe<'a, Message> {
    content: Element<'a, Message>,
}

impl<'a, Message> FrameProbe<'a, Message> {
    pub fn new(content: impl Into<Element<'a, Message>>) -> Self {
        Self {
            content: content.into(),
        }
    }
}

/// 门开时取起始时刻（门关=None——零开销纪律：不取时刻不进分支）。
fn probe_start() -> Option<std::time::Instant> {
    crate::ui::frame_bench::segments_gate().then(std::time::Instant::now)
}

fn note_elapsed(t0: Option<std::time::Instant>, f: fn(std::time::Duration)) {
    if let Some(t0) = t0 {
        f(t0.elapsed());
    }
}

impl<Message> Widget<Message, iced::Theme, iced::Renderer> for FrameProbe<'_, Message>
where
    Message: Clone + 'static,
{
    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<()>()
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[self.content.as_widget()]);
    }

    fn size(&self) -> Size<iced::Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<iced::Length> {
        self.content.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let t0 = probe_start();
        let node = self
            .content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);
        note_elapsed(t0, crate::ui::frame_segments::note_s5_layout);
        node
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        inherited_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let t0 = probe_start();
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            inherited_style,
            layout,
            cursor,
            viewport,
        );
        note_elapsed(t0, crate::ui::frame_segments::note_s5_draw);
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, Message, iced::Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message> From<FrameProbe<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'static,
{
    fn from(probe: FrameProbe<'a, Message>) -> Self {
        Self::new(probe)
    }
}
