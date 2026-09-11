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
/// Plan 503 M5：28→36（stella 36-40px 标题栏带）。
pub(crate) const TITLEBAR_H: f32 = 36.0;
/// PLAN-526 T5：缩放把手命中区加宽（462 起边 6/角 14——透明无光标反馈
/// 被实测反馈为"无法缩放"；加宽 + 系统缩放光标双管齐下）。
const EDGE: f32 = 8.0;
const CORNER: f32 = 16.0;
/// 边框宽（Plan 473 T6：native slot chrome 与同步换算共用，pub(crate)）。
pub(crate) const BORDER: f32 = 1.0;
/// Plan 503 M5：窗体圆角 8→16（stella rounded-2xl 档）。
const WIN_RADIUS: f32 = 16.0;
/// PLAN-002 N2：fit 窗测量期隐藏上限（ServiceTick 400ms 节拍计数）。
/// 测量常态首帧内回执（开窗即终尺寸）；超限（app 异常、锚点恒缺席）
/// 强制显形，退化为旧观感（初值尺寸可见），杜绝"永不显形"死窗。
const FIT_HIDE_MAX_TICKS: u32 = 5;
/// PLAN-002 N2：隐藏期绘制偏移（px）——远超任何宿主视口高，窗体连同
/// 缩放把手/焦点环整体画出可视区；布局树保持原位，fit 测量锚点
/// （客户端内容 Shrink 布局）不受影响，Stack 子树零位移（T20 diff 错位
/// 类不触），越界层被宿主窗裁剪自然不可见、命中亦不可达。
const FIT_HIDE_OFFSET: f32 = 100_000.0;

/// PLAN-002 N2：fit 窗测量期隐藏判定（纯函数，测试靶）——测量落定前
/// 整窗不画（用户只看到窗口以终尺寸出现，不再"先 60% 大再缩"闪变）。
pub(crate) fn vwin_fit_hidden(fit_pending: bool, hidden_ticks: u32) -> bool {
    fit_pending && hidden_ticks < FIT_HIDE_MAX_TICKS
}

/// PLAN-526 T25：窗体圆角分角化——顶部 WIN_RADIUS 圆角、底部方角。
/// 根因：app 自绘背景方角且 iced 0.14 clip 为矩形（container.rs:351 仅
/// bounds 求交），圆角环底部两角必露内容方角；用户认可降级「环随内容
/// 方角」。顶部属 chrome（标题条带随 win_box 圆角绘制），保持圆角。
fn window_radius(maximized: bool) -> iced::border::Radius {
    if maximized {
        iced::border::Radius::default()
    } else {
        // PLAN-002 N5（用户复核裁定 2026-09-09）：四角全圆——T25 的"底边
        // 方角"降级撤销；内容探出由 app 根节点默认 rounded-b（renderer
        // round_bottom_root_default）系统性收口。
        iced::border::Radius::from(WIN_RADIUS)
    }
}


/// PLAN-526 T37：标题栏右键菜单面板（窗口 Stack 顶层）。
/// 项：最大化/还原、最小化、关闭（chrome 既有命令）；发送到下/上一分区
/// （SendFocusedTo 既有热键臂复用）。左键任意按压经 GlobalPress 关菜单。
/// PLAN-012：面板左上角跟随右键落点（`(mx, my)` 窗内坐标，调用侧经
/// [`title_menu_spot`] 钳制），弃 T37 固定右上（对齐三键）原设计。
fn title_menu_panel(
    wid: crate::ui::session::Wid,
    mx: f32,
    my: f32,
) -> Element<'static, DesktopMessage> {
    use iced::Padding;
    let surface = token(crate::ui::style::Color::Surface);
    let border_c = token(crate::ui::style::Color::Border);
    let fg = token(crate::ui::style::Color::OnSurface);
    let item = |label: &'static str, msg: DesktopMessage| -> iced::Element<'static, DesktopMessage> {
        iced::widget::button(
            iced::widget::container(iced::widget::text(label).size(12))
                .width(Length::Fill)
                .height(Length::Fixed(28.0))
                .align_y(Alignment::Center)
                .align_x(Alignment::Start),
        )
        .padding(Padding {
            top: 0.0,
            right: 10.0,
            bottom: 0.0,
            left: 10.0,
        })
        .style(move |_t, _st| iced::widget::button::Style {
            background: Some(Color::TRANSPARENT.into()),
            border: Border {
                radius: 6.0.into(),
                ..Default::default()
            },
            text_color: fg,
            ..Default::default()
        })
        .on_press(msg)
        .into()
    };
    let divider = || -> iced::Element<'static, DesktopMessage> {
        iced::widget::container(iced::widget::Space::new().height(Length::Fixed(0.0)))
            .width(Length::Fill)
            .height(Length::Fixed(1.0))
            .style(move |_t| Style {
                background: Some(Color::from_rgba(fg.r, fg.g, fg.b, 0.15).into()),
                ..Default::default()
            })
            .into()
    };
    let menu = iced::widget::column![
        item(
            "最大化 / 还原",
            DesktopMessage::Wm(WmCommand::ToggleMaximize(wid)),
        ),
        item("最小化", DesktopMessage::Wm(WmCommand::Minimize(wid))),
        item("关闭", DesktopMessage::Wm(WmCommand::Close(wid))),
        divider(),
        item(
            "发送到下一分区",
            DesktopMessage::Wm(WmCommand::SendFocusedTo(
                crate::ui::session::WorkspaceStep::Next,
            )),
        ),
        item(
            "发送到上一分区",
            DesktopMessage::Wm(WmCommand::SendFocusedTo(
                crate::ui::session::WorkspaceStep::Prev,
            )),
        ),
    ]
    .spacing(2.0)
    .padding(Padding {
        top: 4.0,
        right: 4.0,
        bottom: 4.0,
        left: 4.0,
    });
    let panel = iced::widget::container(menu)
        .width(Length::Fixed(180.0))
        .style(move |_t| Style {
            background: Some(Color::from_rgba(
                surface.r,
                surface.g,
                surface.b,
                0.98,
            ).into()),
            border: Border {
                color: border_c,
                width: 1.0,
                radius: 8.0.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.4),
                offset: Vector::new(0.0, 6.0),
                blur_radius: 16.0,
            },
            ..Default::default()
        });
    // 定位：PLAN-012 跟手——padding left/top = 钳制后的右键落点（窗内
    // 坐标），Start/Start 对齐使面板左上角即落点（弃 T37 固定右上）。
    iced::widget::container(panel)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding {
            top: my,
            right: 0.0,
            bottom: 0.0,
            left: mx,
        })
        .align_x(Alignment::Start)
        .align_y(Alignment::Start)
        .into()
}

/// PLAN-012：标题菜单面板尺寸/边距常量（钳制口径；PANEL_H 为 5 项+分隔线
/// 的估算高度，实机若溢出微调即可）。
const TITLE_MENU_PANEL_W: f32 = 180.0;
const TITLE_MENU_PANEL_H: f32 = 168.0;
const TITLE_MENU_MARGIN: f32 = 8.0;

/// PLAN-012：右键落点（host 窗逻辑坐标）→ 标题菜单面板左上角（窗内坐
/// 标）。x/y 跟随光标；面板尺寸 + 8px 边距做右/下缘钳制（窗过小
/// 时不越界），y 不低于标题条底（右键点在标题条上，菜单即其下方展开，
/// 不遮标题条）。纯函数供单测。
pub fn title_menu_spot(
    cx: f32,
    cy: f32,
    rx: f32,
    ry: f32,
    rw: f32,
    rh: f32,
) -> (f32, f32) {
    let lx = cx - rx;
    let ly = cy - ry;
    let x = lx.clamp(
        TITLE_MENU_MARGIN,
        (rw - TITLE_MENU_PANEL_W - TITLE_MENU_MARGIN).max(TITLE_MENU_MARGIN),
    );
    let y = ly.clamp(
        TITLEBAR_H,
        (rh - TITLE_MENU_PANEL_H - TITLE_MENU_MARGIN).max(TITLEBAR_H),
    );
    (x, y)
}

#[cfg(test)]
mod title_menu_spot_tests {
    use super::*;

    #[test]
    fn p012_spot_follows_cursor_inside_window() {
        // 窗 rect (100, 80) 800×600；落点在窗中部——原样跟随（相对化）。
        assert_eq!(title_menu_spot(300.0, 200.0, 100.0, 80.0, 800.0, 600.0), (200.0, 120.0));
    }

    #[test]
    fn p012_spot_clamps_right_and_bottom_edges() {
        // 右缘：落点靠右 → 面板左移收进窗内；下缘：y 钳到面板高上限。
        let (x, y) = title_menu_spot(890.0, 650.0, 100.0, 80.0, 800.0, 600.0);
        assert_eq!(x, 800.0 - 180.0 - 8.0);
        assert_eq!(y, 600.0 - 168.0 - 8.0);
    }

    #[test]
    fn p012_spot_titlebar_click_drops_below_titlebar() {
        // 标题条上右键（ly < TITLEBAR_H=36）→ y 钳到标题条底。
        let (x, y) = title_menu_spot(150.0, 95.0, 100.0, 80.0, 800.0, 600.0);
        assert_eq!(x, 50.0);
        assert_eq!(y, TITLEBAR_H);
    }

    #[test]
    fn p012_spot_degenerate_window_stays_in_bounds() {
        // 退化小窗（宽/高小于面板+边距）：钳到下限而非负坐标。
        let (x, y) = title_menu_spot(0.0, 0.0, 0.0, 0.0, 50.0, 40.0);
        assert_eq!(x, TITLE_MENU_MARGIN);
        assert_eq!(y, TITLEBAR_H);
    }
}
/// 语义色快捷访问（跟随 iced_adapter 的 dark/accent thread-local）。
fn token(c: crate::ui::style::Color) -> Color {
    crate::ui::style::iced_adapter::resolve_semantic_rgb(&c)
        .map(|(r, g, b)| Color::from_rgb8(r, g, b))
        .unwrap_or(Color::from_rgb8(0x1E, 0x1E, 0x24))
}

/// Plan 518 G6：Transparency 三档 → 虚拟窗底色 alpha（off=0.95 / low=0.80 /
/// high=0.62，初值实机可调）。决策纯函数；档位坏值 = off。仅底色——chrome
/// 窗口键/描边/文字不透明保可用性（计划条款）；alpha 由调用方自单源
/// `config.transparency` 映射传入（Plan 540 T2，storage 每帧读退役），
/// 设置窗写后下一帧即时生效（壁纸键先例为 boot 读，此处面板同屏切换
/// 要求即时）。
pub(crate) fn transparency_alpha_for(level: &str) -> f32 {
    match level.trim() {
        "low" => 0.80,
        "high" => 0.62,
        _ => 0.95,
    }
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

/// PLAN-526 T16：标题栏窗口键（右置 `– ▢ ×`，Windows 惯例序——关闭
/// 最右；T3 一轮版的左置组实测间隔失控 + 无 hover 反馈）。根因：
/// `container.center(Fill)` 同时把宽高置 Fill（按钮被均分拉散）——
/// 改回 Fixed 命中盒 + align 双中。hover 反馈走 iced `button` 原生件：
/// 天然 Pointer 光标 + `Status::Hovered/Pressed` 背景提亮（仅图标
/// 包围容器提亮，圆角小块——用户截图3 VSCode 形态，非全条填充）。
/// PLAN-526 T22：命中盒 30×24 → 24×24 正方形（用户反馈长方形盒让
/// 三键横向松散；等宽=等高后三键紧拢，glyph 13/10/13 均可容纳）。
/// PLAN-526 T28：button 显式 `.padding(0)`——T22 只收了内容盒，iced
/// button 主题默认 padding（水平更宽）仍让 hover 提亮盒呈宽体长方
/// （用户七轮截图实录）；padding=0 后 hover 盒=内容盒=24×24 正方形。
fn title_button(glyph: &'static str, size: f32, msg: DesktopMessage) -> Element<'static, DesktopMessage> {
    let fg = token(crate::ui::style::Color::OnSurface);
    let hover_tint = if crate::ui::style::theme::dark_mode() { 1.0 } else { 0.0 };
    iced::widget::button(
        container(text(glyph).size(size))
            .width(Length::Fixed(24.0))
            .height(Length::Fixed(24.0))
            .align_x(Alignment::Center)
            .align_y(Alignment::Center),
    )
    .padding(0.0)
    .style(move |_t, status| {
        let alpha = match status {
            iced::widget::button::Status::Hovered => 0.10,
            iced::widget::button::Status::Pressed => 0.18,
            _ => 0.0,
        };
        iced::widget::button::Style {
            background: Some(Color::from_rgba(hover_tint, hover_tint, hover_tint, alpha).into()),
            border: Border {
                radius: 6.0.into(),
                ..Default::default()
            },
            text_color: fg,
            ..Default::default()
        }
    })
    .on_press(msg)
    .into()
}

/// 组装一个虚拟窗口层：定位包裹 + 窗体（标题栏 chrome + 客户区 + 八向
/// 缩放把手）。`client` 已由调用方打标 `DM::App(app_id, ·)` 并包好 panic
/// 边界；本函数只读 `vwin` 几何，不发 VM 消息。
pub fn virtual_window_element<'a>(
    vwin: &VWinState,
    focused: bool,
    title_menu_pos: Option<(f32, f32)>,
    t_alpha: f32,
    client: Element<'a, DesktopMessage>,
) -> Element<'a, DesktopMessage> {
    let rect = *vwin.rect.borrow();
    let wid = vwin.wid;

    // --- 标题栏（整条为拖拽把手；窗口键优先捕获）---
    // PLAN-526 T16：三键右置（Windows 惯例序 `– ▢ ×`，关闭最右；紧排
    // 间距 2px、右收边 8px），左配重列对称保持标题窗口级居中（518 G5
    // 形态不变）。行内垂直居中由容器 center_y 承载（T3 一轮修复保留）。
    let win_buttons = row![
        title_button("–", 13.0, DesktopMessage::Wm(WmCommand::Minimize(wid))),
        title_button("□", 10.0, DesktopMessage::Wm(WmCommand::ToggleMaximize(wid))),
        title_button("×", 13.0, DesktopMessage::Wm(WmCommand::Close(wid))),
    ]
    .spacing(2.0);

    let titlebar = mouse_area(
        row![
            container(text("")).width(Length::Fixed(102.0)),
            container(text(vwin.title.clone()).size(12))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill),
            container(win_buttons)
                .padding(Padding {
                    top: 0.0,
                    right: 8.0,
                    bottom: 0.0,
                    left: 0.0,
                })
                .center_y(Length::Fill),
        ]
        .width(Length::Fill)
        .height(Length::Fixed(TITLEBAR_H)),
    )
    .interaction(iced::mouse::Interaction::Idle)
    .on_press(DesktopMessage::Wm(WmCommand::StartDrag { wid }))
    // PLAN-526 T37：右键 → 标题栏菜单（最大化/最小化/关闭/送分区）。
    .on_right_press(DesktopMessage::Wm(WmCommand::TitleMenuOpen(wid)));

    // --- 客户区（点击聚焦 + 阻断穿透；App 组件优先捕获不受影响）---
    // Plan 518 G6：底色乘 Transparency 档位 alpha（内容文字照常绘制其上）。
    // PLAN-526 T23：`.interaction(Arrow)` —— 客户区整块对 mouse_interaction
    // 查询回报 Arrow，iced Stack 自顶向下的 levitate 语义随之生效：光标
    // 悬在本窗 bounds 内时下层窗口的 button 不再收到 hover（半透明是
    // 纯视觉，遮挡语义由此收口）；客户区内 App 自己的交互件（Pointer）
    // 仍优先（mouse_area 的 child interaction 优先于 interaction 兜底）。
    let mut client_bg = token(crate::ui::style::Color::Background);
    client_bg.a = t_alpha;
    // PLAN-002 N5：客户区底色与窗框同步底角圆角（最大化=全屏方角）。
    let maximized = vwin.maximized.get();
    let client_area = container(
        mouse_area(container(client).width(Length::Fill).height(Length::Fill))
            .interaction(iced::mouse::Interaction::Idle)
            .on_press(DesktopMessage::Wm(WmCommand::Focus(wid))),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(move |_t| Style {
        background: Some(client_bg.into()),
        border: Border {
            radius: window_radius(maximized),
            ..Default::default()
        },
        ..Default::default()
    });

    let body = column![titlebar, client_area]
        .width(Length::Fill)
        .height(Length::Fill);

    // --- 窗体容器：裁剪 + 阴影 + 常驻弱描边 ---
    // Plan 503 M5：柔影 (0,8)/32px——light 12% / dark 40%，focused 加深。
    // Plan 518 G5 重校：柔影 (0,10)/40——dark 40–52%（聚焦区间上限即
    // 0.52）、light 12–18%（聚焦 0.18,原 0.20 收敛对齐 stella 轻影）。
    // PLAN-526 T2/T4：最大化改读真状态（462 的"rect≈全桌面 98%"派生判定
    // 退役）；描边职责移交 Stack 顶层焦点环（本框只留常驻弱描边——
    // 整框 1px 会被客户区不透明底色盖住，实测只剩标题栏三边可见）。
    let accent = token(crate::ui::style::Color::Primary);
    let dark = crate::ui::style::theme::dark_mode();
    let (base_alpha, focus_boost): (f32, f32) = if dark { (0.40, 0.12) } else { (0.12, 0.06) };
    let shadow_alpha = if focused {
        (base_alpha + focus_boost).min(0.7)
    } else {
        base_alpha
    };
    // Plan 518 G6：窗体底色（Surface，标题栏带）同乘档位 alpha。
    let mut win_surface = token(crate::ui::style::Color::Surface);
    win_surface.a = t_alpha;
    let win_box = container(body)
        .width(Length::Fill)
        .height(Length::Fill)
        .clip(true)
        .style(move |_t| Style {
            background: Some(win_surface.into()),
            border: Border {
                color: token(crate::ui::style::Color::Surface),
                width: BORDER,
                radius: window_radius(maximized),
            },
            shadow: if maximized {
                Shadow::default()
            } else {
                Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, shadow_alpha),
                    offset: Vector::new(0.0, 10.0),
                    blur_radius: 40.0,
                }
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

    // --- 焦点环 overlay（PLAN-526 T4；Stack 最顶层）---
    // 聚焦 accent 2px 环：非捕获空层事件穿透到下层把手，且永不被客户区
    // 不透明底色覆盖（503 的"描边只围标题栏"缺口随此闭合）。失焦透明。
    let ring = container(text(""))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_t| Style {
            border: Border {
                color: if focused {
                    Color::from_rgba(accent.r, accent.g, accent.b, 0.9)
                } else {
                    Color::TRANSPARENT
                },
                width: if focused { 2.0 } else { 0.0 },
                radius: window_radius(maximized),
            },
            ..Default::default()
        });
    layers.push(ring.into());

    // PLAN-526 T37：标题栏右键菜单浮层（窗口 Stack 顶层；chrome 自绘，
    // 不经 .at——标题栏本为 I4 chrome 域）。任意左键按压经 GlobalPress
    // 清 title_menu 关闭（点外部关语义，T29 对齐）。
    // PLAN-012：面板左上角随右键落点（窗内坐标，钳制见 title_menu_spot）。
    if let Some((mx, my)) = title_menu_pos {
        layers.push(title_menu_panel(wid, mx, my));
    }

    // 定位包裹：padding 出窗口原点，Start/Start 对齐（Stack 每层布局原点
    // 在桌面左上，见 T1 spike 记录）。
    let win_stack = iced::widget::Stack::with_children(layers)
        .width(Length::Fixed(rect.width))
        .height(Length::Fixed(rect.height));
    // PLAN-002 N2：fit 测量期隐藏——绘制位置整体越界（布局树原位不动，
    // 锚点可测、子树零位移），`vwin_fit_hidden` 谓词见常量注。
    let hide = vwin_fit_hidden(vwin.fit_pending.get(), vwin.fit_hidden_ticks.get());
    container(win_stack)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding {
            top: rect.y + if hide { FIT_HIDE_OFFSET } else { 0.0 },
            left: rect.x,
            right: 0.0,
            bottom: 0.0,
        })
        .align_x(Alignment::Start)
        .align_y(Alignment::Start)
        .into()
}

/// 缩放把手：透明命中区（pad 定位于窗体局部坐标，其余维度 Fill 补齐）。
/// PLAN-526 T5：挂系统缩放光标（拖拽可供性）。
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
        .interaction(resize_cursor(edge))
        .on_press(DesktopMessage::Wm(WmCommand::StartResize { wid, edge })),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(Padding { top: pad_top.max(0.0), left: pad_left.max(0.0), right: 0.0, bottom: 0.0 })
    .align_x(Alignment::Start)
    .align_y(Alignment::Start)
    .into()
}

/// PLAN-526 T5：八向把手 → 系统缩放光标映射。
fn resize_cursor(edge: ResizeEdge) -> iced::mouse::Interaction {
    use iced::mouse::Interaction;
    match edge {
        ResizeEdge::North | ResizeEdge::South => Interaction::ResizingVertically,
        ResizeEdge::East | ResizeEdge::West => Interaction::ResizingHorizontally,
        ResizeEdge::NorthWest | ResizeEdge::SouthEast => Interaction::ResizingDiagonallyDown,
        ResizeEdge::NorthEast | ResizeEdge::SouthWest => Interaction::ResizingDiagonallyUp,
    }
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

/// Plan 486：拖入手势落点高亮（DragWatch::Over 时绘制；463 snap 预览同款
/// 语义——主色半透明填充 + 描边，提示松手即收编的槽位）。纯视觉层：
/// 无鼠标区、无 chrome（拖动进行中，被拖原生窗在 OS z 序上方）。
pub fn native_drag_over_element<'a>(rect: iced::Rectangle) -> Element<'a, DesktopMessage> {
    let accent = token(crate::ui::style::Color::Primary);
    let hint_box = container(text(""))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_t| Style {
            background: Some(Color::from_rgba(accent.r, accent.g, accent.b, 0.18).into()),
            border: Border {
                color: accent,
                width: BORDER + 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        });
    container(hint_box)
        .width(Length::Fixed(rect.width))
        .height(Length::Fixed(rect.height))
        .padding(Padding { top: rect.y, left: rect.x, right: 0.0, bottom: 0.0 })
        .align_x(Alignment::Start)
        .align_y(Alignment::Start)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// PLAN-002 N2：fit 窗测量期隐藏判定——pending 且未超 tick 上限时
    /// 隐藏（开窗即终尺寸）；超限强制显形（防测量永不回执死窗）；
    /// 非 fit 窗恒显。
    #[test]
    fn vwin_fit_hidden_caps_measurement_period() {
        // 测量常态：pending + 计数在上限内 → 隐藏。
        assert!(vwin_fit_hidden(true, 0));
        assert!(vwin_fit_hidden(true, FIT_HIDE_MAX_TICKS - 1));
        // 护栏：计数达上限 → 强制显形（退化为旧观感）。
        assert!(!vwin_fit_hidden(true, FIT_HIDE_MAX_TICKS));
        // 测量已回执（pending 翻 false）→ 显形（终尺寸）。
        assert!(!vwin_fit_hidden(false, 0));
        // 非 fit 窗恒显。
        assert!(!vwin_fit_hidden(false, u32::MAX));
    }

    /// Plan 518 G6 T1 决策单测:三档 alpha 映射 + 缺席/坏值回退 off。
    /// 初值 off=0.95 / low=0.80 / high=0.62（实机调参时同步本表）。
    #[test]
    fn transparency_levels_map_to_alpha() {
        assert_eq!(transparency_alpha_for("off"), 0.95);
        assert_eq!(transparency_alpha_for("low"), 0.80);
        assert_eq!(transparency_alpha_for("high"), 0.62);
        // 容错:空/未知/带空白 → off。
        assert_eq!(transparency_alpha_for(""), 0.95);
        assert_eq!(transparency_alpha_for("bogus"), 0.95);
        assert_eq!(transparency_alpha_for(" low "), 0.80, "首尾空白容忍");
        // Plan 540 T2：单源映射（config.transparency → alpha；storage
        // 每帧读已退役，缺省档 off 同型兜底）。
        assert_eq!(
            transparency_alpha_for(
                &crate::ui::desktop_config::DesktopConfig::default().transparency
            ),
            0.95
        );
    }
}
