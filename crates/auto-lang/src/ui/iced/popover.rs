// Plan 422: 锚定弹层原语 —— iced overlay 机制的真弹层 widget。
//
// Wrapper 模式(iced Tooltip 同型):`layout` 委托 anchor(触发按钮),
// `overlay()` 在 open 时把 content 作为 overlay::Element 交还 runtime。
// runtime 先于基础树给 overlay 派发事件(iced_runtime::UserInterface::
// update),overlay 报告非 None 鼠标交互时基础树 cursor 置 Unavailable
// (第五会话 tooltip 遮蔽分析实读)。面板 chrome(bg/border/shadow)由
// content 元素自带(apply_column_style 的 visual wrap),本 widget 只负责
// 定位/事件捕获/dismiss,不画背景 —— 与 Stack 常驻槽的 toast 层(点击
// 穿透,Plan 412)互补。
//
// 捕获语义(取代 418 menubar 的 2000px 隐形 catch 按钮):
// * 点击面板 bounds 内 —— 转发给 content,若 content 未捕获则补捕获
//   (基础树永远收不到落在面板上的点击)。
// * 点击 anchor(触发按钮,仅 widget 锚)—— 发布 on_dismiss 并捕获:
//   基础树收不到 toggle,菜单经 on_dismiss 干净关闭(点触发器 = 关)。
// * 点击外部 —— 发布 on_dismiss 但【不】捕获:别处的 menubar 触发器
//   仍能收到这次点击直接切换菜单(pick_list 会整吞外部点击,这里选择
//   放行以保留 menubar 切换语义)。
// * Esc —— 发布 on_dismiss 并捕获;窗口失焦 —— 仅发布。
//
// 定位:BottomStart = 面板左缘对齐锚左缘、顶缘在锚下方 gap 处(menubar
// 下拉);BottomEnd/Bottom(居中)类似;Left/Right 垂直居中。坐标锚
// (`at_point`)把锚当作位于该点的零尺寸矩形 —— 面板左上角对齐落点
// (contextmenu 约定)。`snap_within_viewport` 默认开启:越界时收回到
// 视口内(Tooltip 同款逻辑)。

use iced::advanced::layout::{self, Layout};
use iced::advanced::mouse;
use iced::advanced::overlay;
use iced::advanced::renderer;
use iced::advanced::widget::{self, Tree, Widget};
use iced::advanced::Clipboard;
use iced::advanced::Shell;
use iced::event::Event;
use iced::touch;
use iced::{keyboard, window, Element, Point, Rectangle, Size, Vector};

use crate::ui::view::PopoverPlacement;

/// 面板与锚之间留出的间隙(menubar 语义:面板紧贴按钮条下缘)。
const DEFAULT_GAP: f32 = 0.0;

pub struct Popover<'a, Message>
where
    Message: Clone + 'static,
{
    anchor: Element<'a, Message>,
    content: Element<'a, Message>,
    placement: PopoverPlacement,
    open: bool,
    /// 坐标锚(contextmenu);None = widget 锚(anchor 元素的 bounds)。
    at_point: Option<(f32, f32)>,
    on_dismiss: Option<Message>,
    gap: f32,
    snap_within_viewport: bool,
}

impl<'a, Message> Popover<'a, Message>
where
    Message: Clone + 'static,
{
    /// 包住触发按钮(anchor)与面板内容(content)。
    pub fn new(
        anchor: impl Into<Element<'a, Message>>,
        content: impl Into<Element<'a, Message>>,
    ) -> Self {
        Self {
            anchor: anchor.into(),
            content: content.into(),
            placement: PopoverPlacement::default(),
            open: false,
            at_point: None,
            on_dismiss: None,
            gap: DEFAULT_GAP,
            snap_within_viewport: true,
        }
    }

    pub fn placement(mut self, placement: PopoverPlacement) -> Self {
        self.placement = placement;
        self
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn on_dismiss(mut self, msg: Message) -> Self {
        self.on_dismiss = Some(msg);
        self
    }

    /// 坐标锚:面板定位到视口坐标 (x, y)(widget 锚失效)。
    pub fn at_point(mut self, x: f32, y: f32) -> Self {
        self.at_point = Some((x, y));
        self
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }
}

impl<Message> Widget<Message, iced::Theme, iced::Renderer> for Popover<'_, Message>
where
    Message: Clone + 'static,
{
    fn children(&self) -> Vec<Tree> {
        vec![
            Tree::new(&self.anchor),
            Tree::new(&self.content),
        ]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[
            self.anchor.as_widget(),
            self.content.as_widget(),
        ]);
    }

    fn size(&self) -> Size<iced::Length> {
        self.anchor.as_widget().size()
    }

    fn size_hint(&self) -> Size<iced::Length> {
        self.anchor.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.anchor.as_widget_mut().layout(
            &mut tree.children[0],
            renderer,
            limits,
        )
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
        self.anchor.as_widget_mut().update(
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
        self.anchor.as_widget().mouse_interaction(
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
        self.anchor.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            inherited_style,
            layout,
            cursor,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, iced::Theme, iced::Renderer>> {
        let mut children = tree.children.iter_mut();

        let anchor_overlays = self.anchor.as_widget_mut().overlay(
            children.next().unwrap(),
            layout,
            renderer,
            viewport,
            translation,
        );

        let panel = if self.open {
            Some(overlay::Element::new(Box::new(Panel {
                // 坐标锚优先;widget 锚取 wrapper 自身(base 树)的绝对
                // bounds —— overlay 层的 layout 与基础树同坐标系。
                anchor_position: layout.position() + translation,
                anchor_bounds: layout.bounds(),
                at_point: self.at_point,
                content: &mut self.content,
                tree: children.next().unwrap(),
                placement: self.placement,
                gap: self.gap,
                snap_within_viewport: self.snap_within_viewport,
                on_dismiss: self.on_dismiss.clone(),
            })))
        } else {
            None
        };

        let elements: Vec<_> = anchor_overlays.into_iter().chain(panel).collect();
        (!elements.is_empty()).then(|| overlay::Group::with_children(elements).overlay())
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        operation.container(None, layout.bounds());
        operation.traverse(&mut |operation| {
            self.anchor.as_widget_mut().operate(
                &mut tree.children[0],
                layout,
                renderer,
                operation,
            );
        });
    }
}

impl<'a, Message> From<Popover<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'static,
{
    fn from(popover: Popover<'a, Message>) -> Self {
        Element::new(popover)
    }
}

/// open 时置顶的面板 overlay 元素。
struct Panel<'a, 'b, Message>
where
    Message: Clone + 'static,
{
    anchor_position: Point,
    anchor_bounds: Rectangle,
    at_point: Option<(f32, f32)>,
    content: &'b mut Element<'a, Message>,
    tree: &'b mut Tree,
    placement: PopoverPlacement,
    gap: f32,
    snap_within_viewport: bool,
    on_dismiss: Option<Message>,
}

impl<Message> overlay::Overlay<Message, iced::Theme, iced::Renderer> for Panel<'_, '_, Message>
where
    Message: Clone + 'static,
{
    fn layout(&mut self, renderer: &iced::Renderer, bounds: Size) -> layout::Node {
        let viewport = Rectangle::with_size(bounds);

        let (position, anchor_bounds) = match self.at_point {
            Some((x, y)) => (
                Point::new(x, y),
                Rectangle::new(Point::new(x, y), Size::ZERO),
            ),
            None => (self.anchor_position, self.anchor_bounds),
        };

        let content_layout = self.content.as_widget_mut().layout(
            self.tree,
            renderer,
            &layout::Limits::new(
                Size::ZERO,
                if self.snap_within_viewport {
                    viewport.size()
                } else {
                    Size::INFINITE
                },
            ),
        );
        let size = content_layout.bounds().size();

        let x_start = position.x;
        let x_end = position.x + anchor_bounds.width - size.width;
        let x_center = position.x + (anchor_bounds.width - size.width) / 2.0;
        let y_center = position.y + (anchor_bounds.height - size.height) / 2.0;

        let mut panel_bounds = match self.placement {
            PopoverPlacement::Bottom => Rectangle::new(
                Point::new(x_center, position.y + anchor_bounds.height + self.gap),
                size,
            ),
            PopoverPlacement::BottomStart => Rectangle::new(
                Point::new(x_start, position.y + anchor_bounds.height + self.gap),
                size,
            ),
            PopoverPlacement::BottomEnd => Rectangle::new(
                Point::new(x_end, position.y + anchor_bounds.height + self.gap),
                size,
            ),
            PopoverPlacement::Top => Rectangle::new(
                Point::new(x_center, position.y - size.height - self.gap),
                size,
            ),
            PopoverPlacement::TopStart => Rectangle::new(
                Point::new(x_start, position.y - size.height - self.gap),
                size,
            ),
            PopoverPlacement::TopEnd => Rectangle::new(
                Point::new(x_end, position.y - size.height - self.gap),
                size,
            ),
            PopoverPlacement::Left => Rectangle::new(
                Point::new(position.x - size.width - self.gap, y_center),
                size,
            ),
            PopoverPlacement::Right => Rectangle::new(
                Point::new(
                    position.x + anchor_bounds.width + self.gap,
                    y_center,
                ),
                size,
            ),
        };

        if self.snap_within_viewport {
            if panel_bounds.x < viewport.x {
                panel_bounds.x = viewport.x;
            } else if viewport.x + viewport.width
                < panel_bounds.x + panel_bounds.width
            {
                panel_bounds.x = viewport.x + viewport.width - panel_bounds.width;
            }

            if panel_bounds.y < viewport.y {
                panel_bounds.y = viewport.y;
            } else if viewport.y + viewport.height
                < panel_bounds.y + panel_bounds.height
            {
                panel_bounds.y =
                    viewport.y + viewport.height - panel_bounds.height;
            }
        }

        layout::Node::with_children(size, vec![content_layout])
            .move_to(panel_bounds.position())
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        let panel_bounds = layout.bounds();

        let dismiss = |shell: &mut Shell<'_, Message>, on_dismiss: &Option<Message>| {
            if let Some(msg) = on_dismiss {
                shell.publish(msg.clone());
            }
        };

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                let over_panel = cursor.is_over(panel_bounds);
                let over_anchor = self.at_point.is_none() && cursor.is_over(self.anchor_bounds);
                if over_anchor {
                    // 锚上点击:dismiss 并捕获 —— 基础树收不到 toggle,菜单经
                    // on_dismiss 干净关闭(点触发器 = 关,menubar 语义)。
                    dismiss(shell, &self.on_dismiss);
                }
                if !over_panel && !over_anchor {
                    // 面板/锚之外的点击:dismiss 但放行给基础树 —— 别的
                    // menubar 触发器可以直接切换菜单。
                    dismiss(shell, &self.on_dismiss);
                    return;
                }
                // 面板内(或锚上):转发给 content(菜单项自行发布/捕获),
                // 未捕获时兜底捕获,基础树收不到这次点击。
                let content_layout = layout.children().next().expect("panel has content child");
                self.content.as_widget_mut().update(
                    self.tree,
                    event,
                    content_layout,
                    cursor,
                    renderer,
                    clipboard,
                    shell,
                    &Rectangle::with_size(Size::INFINITE),
                );
                if !shell.is_event_captured() {
                    shell.capture_event();
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Escape),
                ..
            }) => {
                dismiss(shell, &self.on_dismiss);
                shell.capture_event();
            }
            Event::Window(window::Event::Unfocused) => {
                dismiss(shell, &self.on_dismiss);
            }
            _ => {
                // 其余事件(移动/滚轮/键盘输入等)转发给 content,面板内
                // 交互(hover 态、焦点)保持活着;content 自行决定捕获。
                let content_layout = layout.children().next().expect("panel has content child");
                self.content.as_widget_mut().update(
                    self.tree,
                    event,
                    content_layout,
                    cursor,
                    renderer,
                    clipboard,
                    shell,
                    &Rectangle::with_size(Size::INFINITE),
                );
            }
        }
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        if !cursor.is_over(layout.bounds()) {
            return mouse::Interaction::None;
        }
        let content_layout = layout.children().next().expect("panel has content child");
        self.content.as_widget().mouse_interaction(
            self.tree,
            content_layout,
            cursor,
            &Rectangle::with_size(Size::INFINITE),
            renderer,
        )
    }

    fn draw(
        &self,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        inherited_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        // chrome(bg/border/shadow)由 content 元素自带的 visual wrap 绘制。
        let content_layout = layout.children().next().expect("panel has content child");
        self.content.as_widget().draw(
            self.tree,
            renderer,
            theme,
            inherited_style,
            content_layout,
            cursor,
            &Rectangle::with_size(Size::INFINITE),
        );
    }

    fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        // 转发给 content:iced_test selector / MCP bounds 收集能看见面板
        // 内部节点(tooltip 的 overlay 没实现 operate,所以不可见 —— 我们
        // 要可见,Plan 422 §3 P1 的定位断言依赖它)。
        operation.container(None, layout.bounds());
        operation.traverse(&mut |operation| {
            let content_layout = layout.children().next().expect("panel has content child");
            self.content.as_widget_mut().operate(
                self.tree,
                content_layout,
                renderer,
                operation,
            );
        });
    }

    /// 高于 tooltip(默认 1.0):弹层面板盖住提示。
    fn index(&self) -> f32 {
        10.0
    }
}
