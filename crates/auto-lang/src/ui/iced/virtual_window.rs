//! Plan 462 T3/T4: VirtualWindow 组合层（路线 A，单 OS 窗口多 App）。
//!
//! **T1 spike 定案（候选 B：组合现有 widget）**，依据（iced 0.14 实测源码）：
//! - `Stack`（iced_widget/stack.rs::update）自顶向下逐层转发事件、捕获即停
//!   —— z 序事件路由免费获得；空层事件自然穿透到下层。
//! - `mouse_area::on_press` 命中即 `shell.capture_event()`（且子组件优先，
//!   捕获后 mouse_area 让行）——客户区包裹层既承载"点击聚焦"，又阻断
//!   点击穿透，且不挡 App 自己的交互组件。
//! - `container::clip(true)` 以 viewport 交集裁剪子树绘制 —— 虚拟窗口
//!   内容溢出不再画出窗外。
//! - `mouse_area` 无 `on_drag`：拖拽/缩放改由**全局事件状态机**驱动 ——
//!   chrome `on_press` 发 `WmCommand::StartDrag/StartResize`，移动/松开走
//!   既有 `__mouse_moved`/`__mouse_released` 全局订阅（update 壳层拦截，
//!   见 renderer.rs `DM::Wm`/`DM::Window` 臂），偏移状态在 `WmState`。
//!
//! I4 注记：本层是 renderer 内部组合（chrome 不经 .at 声明）；
//! `virtual_window` 的 schema/WidgetRegistry 登记与 a2vue DOM 叶随 465
//! 一并落地（消费同一 WM 语义规范）。
//!
//! 位置/尺寸的唯一事实源是 `VWinState.rect`（R9：排布是 WM 策略）；
//! 本模块只做"读 WM state → 组 Element"，不持有任何窗口几何。

use iced::widget::{container, column, mouse_area, row, text};
use iced::widget::container::Style;
use iced::{Alignment, Border, Color, Element, Length, Padding, Shadow, Vector};

use crate::ui::session::{DesktopMessage, ResizeEdge, VWinState, WmCommand};

/// 标题条高度（Plan 473 T6：native slot chrome 与同步换算共用，pub(crate)）。
pub(crate) const TITLEBAR_H: f32 = 28.0;
const EDGE: f32 = 6.0;
const CORNER: f32 = 14.0;
/// 边框宽（Plan 473 T6：native slot chrome 与同步换算共用，pub(crate)）。
pub(crate) const BORDER: f32 = 1.0;

/// 语义色快捷访问（跟随 iced_adapter 的 dark/accent thread-local）。
fn token(c: crate::ui::style::Color) -> Color {
    crate::ui::style::iced_adapter::resolve_semantic_rgb(&c)
        .map(|(r, g, b)| Color::from_rgb8(r, g, b))
        .unwrap_or(Color::from_rgb8(0x1E, 0x1E, 0x24))
}

/// 桌面根：背景层 + 虚拟窗口 z-stack（back → front，调用方保证顺序）。
pub fn desktop_root(
    layers: Vec<Element<'_, DesktopMessage>>,
) -> Element<'_, DesktopMessage> {
    if layers.is_empty() {
        return container(text(""))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_t| Style {
                background: Some(token(crate::ui::style::Color::Background).into()),
                ..Default::default()
            })
            .into();
    }
    let stack = iced::widget::Stack::with_children(layers);
    container(stack)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_t| Style {
            background: Some(token(crate::ui::style::Color::Background).into()),
            ..Default::default()
        })
        .into()
}

/// 组装一个虚拟窗口层：定位包裹 + 窗体（标题栏 chrome + 客户区 + 八向
/// 缩放把手）。`client` 已由调用方打标 `DM::App(app_id, ·)` 并包好 panic
/// 边界；本函数只读 `vwin` 几何，不发 VM 消息。
pub fn virtual_window_element<'a>(
    vwin: &VWinState,
    focused: bool,
    client: Element<'a, DesktopMessage>,
) -> Element<'a, DesktopMessage> {
    let rect = *vwin.rect.borrow();
    let wid = vwin.wid;

    // --- 标题栏（整条为拖拽把手；关闭按钮优先捕获）---
    let close_btn = mouse_area(
        container(text("×").size(13))
            .width(Length::Fixed(22.0))
            .height(Length::Fixed(TITLEBAR_H - 6.0))
            .center(Length::Fill),
    )
    .on_press(DesktopMessage::Wm(WmCommand::Close(wid)));

    let titlebar = mouse_area(
        row![
            container(text(vwin.title.clone()).size(12))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_y(Length::Fill)
                .padding(Padding { top: 0.0, right: 0.0, bottom: 0.0, left: 8.0 }),
            close_btn,
        ]
        .spacing(4.0)
        .width(Length::Fill)
        .height(Length::Fixed(TITLEBAR_H)),
    )
    .on_press(DesktopMessage::Wm(WmCommand::StartDrag { wid }));

    // --- 客户区（点击聚焦 + 阻断穿透；App 组件优先捕获不受影响）---
    let client_area = container(
        mouse_area(container(client).width(Length::Fill).height(Length::Fill))
            .on_press(DesktopMessage::Wm(WmCommand::Focus(wid))),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|_t| Style {
        background: Some(token(crate::ui::style::Color::Background).into()),
        ..Default::default()
    });

    let body = column![titlebar, client_area]
        .width(Length::Fill)
        .height(Length::Fill);

    // --- 窗体容器：裁剪 + 阴影 + 焦点描边 ---
    let accent = token(crate::ui::style::Color::Primary);
    let win_box = container(body)
        .width(Length::Fill)
        .height(Length::Fill)
        .clip(true)
        .style(move |_t| Style {
            background: Some(token(crate::ui::style::Color::Surface).into()),
            border: Border {
                color: if focused { accent } else { token(crate::ui::style::Color::Surface) },
                width: if focused { BORDER + 1.0 } else { BORDER },
                radius: 8.0.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.45),
                offset: Vector::new(0.0, 8.0),
                blur_radius: 24.0,
            },
            ..Default::default()
        });

    // --- 八向缩放把手（stack 顶层；边 6px、角 14px）---
    let mut layers: Vec<Element<'a, DesktopMessage>> = Vec::with_capacity(9);
    layers.push(win_box.into());
    layers.push(handle(0.0, 0.0,
        Length::Fill, Length::Fixed(EDGE), wid, ResizeEdge::North));
    layers.push(handle(rect.height - EDGE, 0.0,
        Length::Fill, Length::Fill, wid, ResizeEdge::South));
    layers.push(handle(0.0, 0.0,
        Length::Fixed(EDGE), Length::Fill, wid, ResizeEdge::West));
    layers.push(handle(0.0, rect.width - EDGE,
        Length::Fill, Length::Fill, wid, ResizeEdge::East));
    layers.push(handle(0.0, 0.0,
        Length::Fixed(CORNER), Length::Fixed(CORNER), wid, ResizeEdge::NorthWest));
    layers.push(handle(0.0, rect.width - CORNER,
        Length::Fill, Length::Fixed(CORNER), wid, ResizeEdge::NorthEast));
    layers.push(handle(rect.height - CORNER, 0.0,
        Length::Fixed(CORNER), Length::Fill, wid, ResizeEdge::SouthWest));
    layers.push(handle(rect.height - CORNER, rect.width - CORNER,
        Length::Fill, Length::Fill, wid, ResizeEdge::SouthEast));

    // 定位包裹：padding 出窗口原点，Start/Start 对齐（Stack 每层布局原点
    // 在桌面左上，见 T1 spike 记录）。
    let win_stack = iced::widget::Stack::with_children(layers)
        .width(Length::Fixed(rect.width))
        .height(Length::Fixed(rect.height));
    container(win_stack)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding { top: rect.y, left: rect.x, right: 0.0, bottom: 0.0 })
        .align_x(Alignment::Start)
        .align_y(Alignment::Start)
        .into()
}

/// 缩放把手：透明命中区（pad 定位于窗体局部坐标，其余维度 Fill 补齐）。
fn handle<'a>(
    pad_top: f32,
    pad_left: f32,
    w: Length,
    h: Length,
    wid: crate::ui::session::Wid,
    edge: ResizeEdge,
) -> Element<'a, DesktopMessage> {
    container(
        mouse_area(
            container(text("")).width(w).height(h),
        )
        .on_press(DesktopMessage::Wm(WmCommand::StartResize { wid, edge })),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(Padding { top: pad_top.max(0.0), left: pad_left.max(0.0), right: 0.0, bottom: 0.0 })
    .align_x(Alignment::Start)
    .align_y(Alignment::Start)
    .into()
}

/// Plan 473 T6：原生窗口槽位框 chrome——槽位顶部标题条（标题 + 最小化 +
/// 关闭）+ 1px 边框环；中央透明不绘制（假洞：原生窗口在 OS z 序上盖住
/// 槽位客户区）。原生窗口实际摆放到标题条以下客户区——内缩换算见
/// renderer `sync_native_geometry`（与本模块 TITLEBAR_H/BORDER 同源）。
pub fn native_slot_element<'a>(
    slot_id: crate::ui::native_dock::NativeSlotId,
    title: &str,
    rect: iced::Rectangle,
) -> Element<'a, DesktopMessage> {
    let min_btn = mouse_area(
        container(text("—").size(12))
            .width(Length::Fixed(22.0))
            .height(Length::Fixed(TITLEBAR_H - 6.0))
            .center(Length::Fill),
    )
    .on_press(DesktopMessage::Wm(WmCommand::NativeSlotMin(slot_id)));

    let close_btn = mouse_area(
        container(text("×").size(13))
            .width(Length::Fixed(22.0))
            .height(Length::Fixed(TITLEBAR_H - 6.0))
            .center(Length::Fill),
    )
    .on_press(DesktopMessage::Wm(WmCommand::NativeSlotClose(slot_id)));

    let titlebar = row![
        container(text(title.to_string()).size(12))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_y(Length::Fill)
            .padding(Padding { top: 0.0, right: 0.0, bottom: 0.0, left: 8.0 }),
        min_btn,
        close_btn,
    ]
    .spacing(4.0)
    .width(Length::Fill)
    .height(Length::Fixed(TITLEBAR_H));

    // 标题条 + 透明客户区（槽位洞：原生窗口从桌面窗 z 上方露出）。
    let body = column![
        titlebar,
        container(text("")).width(Length::Fill).height(Length::Fill),
    ]
    .width(Length::Fill)
    .height(Length::Fill);

    let slot_box = container(body)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_t| Style {
            border: Border {
                color: token(crate::ui::style::Color::Surface),
                width: BORDER,
                radius: 0.0.into(),
            },
            ..Default::default()
        });

    container(slot_box)
        .width(Length::Fixed(rect.width))
        .height(Length::Fixed(rect.height))
        .padding(Padding { top: rect.y, left: rect.x, right: 0.0, bottom: 0.0 })
        .align_x(Alignment::Start)
        .align_y(Alignment::Start)
        .into()
}
