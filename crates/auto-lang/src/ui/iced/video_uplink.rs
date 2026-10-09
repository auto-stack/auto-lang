//! video 契约上行事件泵（PLAN-712 **T-04**）——`ontimeupdate` 等 §2.3 上行
//! 事件在 iced 端的分发承载。
//!
//! # 为什么需要它
//!
//! `mpv::widget::drain_events` 是契约事件队列的唯一出口，但此前**没有任何
//! 生产调用方**——事件在 `VideoPrimitive::prepare` 里堆积，应用侧
//! `OnTime`/`OnDuration` handler 永不触发（进度条拖动无效、时长恒 00:00）。
//!
//! # 为什么是 wrapper 而不是 Subscription 轮询（DP-1 定案）
//!
//! 运行时注册表（`RUNTIMES`）是 **thread-local**（mpv 引擎非 Send，只能住
//! 主线程），而 iced Subscription 闭包跑在执行器线程——从那里 drain 永远
//! 落空。wrapper 的 `update` 在 widget 树上执行，恰是主线程。iced_winit 在
//! 每次重绘时把 `RedrawRequested` 事件传播给整棵树（应用 tick → 消息 →
//! 重绘请求，`unconditional-rendering` 下持续成立），所以本 wrapper 的
//! `update` 每帧可达，事件天然与画面同帧。
//!
//! # 薄包装纪律（precedent: `seek_area.rs`）
//!
//! * 只在 **真有契约事件** 时 `shell.publish`——空队列（每帧常态）零消息，
//!   不触发 iced_winit 的 RedrawRequested 处理循环重建视图；
//! * 其余事件与全部 widget 语义**原样转发**给内层 shader，不捕获、不改布局。

use iced::advanced::layout::{self, Layout};
use iced::advanced::mouse;
use iced::advanced::renderer;
use iced::advanced::widget::{self, Tree, Widget};
use iced::advanced::{Clipboard, Shell};
use iced::event::Event;
use iced::{Element, Rectangle, Size, Vector};
use std::sync::Arc;

use crate::ui::mpv::contract::VideoContractEvent;
use crate::ui::mpv::widget::{drain_events, VideoWidgetState};
use crate::ui::view::{MediaEventHandler, MediaEventPayload};

/// 各 `on*` 上行 handler 的集合（与 [`crate::ui::view::View::Video`] 的
/// 上行字段一一对应；`None` = 作者未声明，事件被丢弃）。
///
/// Default 手写：`Option<M>` 的默认不需要 `M: Default`，derive 会多加约束。
pub struct VideoUplinkHandlers<M> {
    pub on_time_update: Option<MediaEventHandler<M>>,
    pub on_loaded_metadata: Option<MediaEventHandler<M>>,
    pub on_play_state: Option<MediaEventHandler<M>>,
    pub on_ended: Option<M>,
    pub on_media_error: Option<MediaEventHandler<M>>,
}

impl<M> Default for VideoUplinkHandlers<M> {
    fn default() -> Self {
        Self {
            on_time_update: None,
            on_loaded_metadata: None,
            on_play_state: None,
            on_ended: None,
            on_media_error: None,
        }
    }
}

/// 纯映射：契约事件 → 宿主消息（经 handler）。**单测锚点**——widget 包装
/// 层之外，这里可脱离 iced 树独立断言合成序列。
pub fn synth_messages<M: Clone>(
    handlers: &VideoUplinkHandlers<M>,
    events: &[VideoContractEvent],
    mut publish: impl FnMut(M),
) {
    for ev in events {
        match ev {
            VideoContractEvent::TimeUpdate(t) => {
                if let Some(h) = &handlers.on_time_update {
                    publish(h.call(MediaEventPayload::Time(*t)));
                }
            }
            VideoContractEvent::LoadedMetadata(d) => {
                if let Some(h) = &handlers.on_loaded_metadata {
                    publish(h.call(MediaEventPayload::Duration(*d)));
                }
            }
            VideoContractEvent::PlayStateChange(p) => {
                if let Some(h) = &handlers.on_play_state {
                    publish(h.call(MediaEventPayload::Playing(*p)));
                }
            }
            VideoContractEvent::Ended => {
                if let Some(m) = &handlers.on_ended {
                    publish(m.clone());
                }
            }
            VideoContractEvent::MediaError(msg) => {
                if let Some(h) = &handlers.on_media_error {
                    publish(h.call(MediaEventPayload::Error(msg.clone())));
                }
            }
        }
    }
}

pub struct VideoUplink<'a, Message> {
    content: Element<'a, Message>,
    handlers: Arc<VideoUplinkHandlers<Message>>,
}

impl<'a, Message> VideoUplink<'a, Message>
where
    Message: Clone + 'static,
{
    pub fn new(content: impl Into<Element<'a, Message>>) -> Self {
        Self {
            content: content.into(),
            handlers: Arc::new(VideoUplinkHandlers::default()),
        }
    }

    fn with_handlers(mut self, f: impl FnOnce(&mut VideoUplinkHandlers<Message>)) -> Self {
        // Arc::get_mut 安全：构造期引用计数恒为 1（widget 每次 view 重建都
        // 重新构造，不存在共享态）。
        f(Arc::get_mut(&mut self.handlers).expect("fresh handlers"));
        self
    }

    pub fn on_time_update(mut self, h: Option<MediaEventHandler<Message>>) -> Self {
        self.with_handlers(|hs| hs.on_time_update = h)
    }

    pub fn on_loaded_metadata(mut self, h: Option<MediaEventHandler<Message>>) -> Self {
        self.with_handlers(|hs| hs.on_loaded_metadata = h)
    }

    pub fn on_play_state(mut self, h: Option<MediaEventHandler<Message>>) -> Self {
        self.with_handlers(|hs| hs.on_play_state = h)
    }

    pub fn on_ended(mut self, m: Option<Message>) -> Self {
        self.with_handlers(|hs| hs.on_ended = m)
    }

    pub fn on_media_error(mut self, h: Option<MediaEventHandler<Message>>) -> Self {
        self.with_handlers(|hs| hs.on_media_error = h)
    }

    /// 事件泵：取走内层 shader widget 的挂起契约事件并合成宿主消息。
    fn pump(&self, tree: &Tree, shell: &mut Shell<'_, Message>) {
        // 内层直接是 shader widget（render_video 保证），其 Tree 状态即
        // VideoWidgetState——widget id 在 iced 生命周期内稳定（重建视图不改）。
        // （iced 0.14 的 State::downcast_ref 直接返回 &T，不匹配即 panic——
        // 本 wrapper 只在 render_video 的 shader 子节点上使用，形态恒定。）
        let id = tree
            .children
            .first()
            .map(|child| child.state.downcast_ref::<VideoWidgetState>().id());
        if let Some(id) = id {
            let events = drain_events(id);
            if !events.is_empty() {
                synth_messages(&self.handlers, &events, |m| shell.publish(m));
            }
        }
    }
}

impl<Message> Widget<Message, iced::Theme, iced::Renderer> for VideoUplink<'_, Message>
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
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
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
        // 每次重绘抽干一次事件队列（见模块文档「为什么是 wrapper」）。
        if matches!(
            event,
            Event::Window(iced::window::Event::RedrawRequested(_))
        ) {
            self.pump(tree, shell);
        }

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
        self.content.as_widget().draw(
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

impl<'a, Message> From<VideoUplink<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'static,
{
    fn from(uplink: VideoUplink<'a, Message>) -> Self {
        Element::new(uplink)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::mpv::contract::VideoContractEvent;
    use std::cell::RefCell;

    /// 契约事件 → 消息映射：按声明的 handler 合成、未声明的丢弃（PLAN-712
    /// 测试设计「uplink 单测」）。
    #[test]
    fn synth_maps_declared_and_drops_undeclared() {
        // 只声明 ontimeupdate 与 onended。
        let handlers: VideoUplinkHandlers<String> = VideoUplinkHandlers {
            on_time_update: Some(MediaEventHandler::new(|p| match p {
                MediaEventPayload::Time(t) => format!("time:{t}"),
                _ => unreachable!(),
            })),
            on_ended: Some("ended".to_string()),
            ..Default::default()
        };
        let out: RefCell<Vec<String>> = RefCell::new(Vec::new());
        synth_messages(
            &handlers,
            &[
                VideoContractEvent::TimeUpdate(1.5),
                VideoContractEvent::LoadedMetadata(90.0), // 未声明 → 丢弃
                VideoContractEvent::PlayStateChange(true), // 未声明 → 丢弃
                VideoContractEvent::Ended,
                VideoContractEvent::MediaError("boom".into()), // 未声明 → 丢弃
                VideoContractEvent::TimeUpdate(1.75),
            ],
            |m| out.borrow_mut().push(m),
        );
        assert_eq!(
            out.into_inner(),
            vec![
                "time:1.5".to_string(),
                "ended".to_string(),
                "time:1.75".to_string()
            ]
        );
    }

    /// 载荷形状：Duration/Playing/Error 各按契约值合成。
    #[test]
    fn synth_carries_payload_shapes() {
        let handlers: VideoUplinkHandlers<String> = VideoUplinkHandlers {
            on_loaded_metadata: Some(MediaEventHandler::new(|p| match p {
                MediaEventPayload::Duration(d) => format!("dur:{d}"),
                _ => panic!("duration handler 只收 Duration"),
            })),
            on_play_state: Some(MediaEventHandler::new(|p| match p {
                MediaEventPayload::Playing(b) => format!("playing:{b}"),
                _ => panic!("play_state handler 只收 Playing"),
            })),
            on_media_error: Some(MediaEventHandler::new(|p| match p {
                MediaEventPayload::Error(s) => format!("err:{s}"),
                _ => panic!("media_error handler 只收 Error"),
            })),
            ..Default::default()
        };
        let out: RefCell<Vec<String>> = RefCell::new(Vec::new());
        synth_messages(
            &handlers,
            &[
                VideoContractEvent::LoadedMetadata(12.5),
                VideoContractEvent::PlayStateChange(false),
                VideoContractEvent::MediaError("无法播放".into()),
            ],
            |m| out.borrow_mut().push(m),
        );
        assert_eq!(
            out.into_inner(),
            vec![
                "dur:12.5".to_string(),
                "playing:false".to_string(),
                "err:无法播放".to_string(),
            ]
        );
    }

    /// 空 handler 集：任何事件都不产出消息（零消息常态不被破坏）。
    #[test]
    fn synth_without_handlers_publishes_nothing() {
        let handlers: VideoUplinkHandlers<()> = VideoUplinkHandlers::default();
        let mut count = 0;
        synth_messages(
            &handlers,
            &[
                VideoContractEvent::TimeUpdate(0.0),
                VideoContractEvent::Ended,
            ],
            |_| count += 1,
        );
        assert_eq!(count, 0);
    }
}
