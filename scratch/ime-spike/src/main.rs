// Plan 452 T6 — IME/焦点 spike 原型（一次性，验收后不并入产品路径）
//
// 验证项映射（Design 23 §9）：
//   ① iced 0.14 单窗口中文 IME 基线（事件流 + text_input 行为）
//   ② 组合输入进行中切换虚拟窗口的定性
//   ③ 全屏模式（Fullscreen）下候选框定位
//   ④ 双虚拟窗口焦点分区（Tab / 点击 / IME 跟随）
//   ⑥ 宿主级输入层降级预案原型
//
// 用法：cargo run 后用中文输入法在 A/B 两个"虚拟窗口"的输入框里打字；
// 底部事件日志面板实时记录 Keyboard / InputMethod(IME) 事件，截图即为证据。

use iced::{
    application, event,
    keyboard::{self, Key},
    widget::{
        button, column, container, mouse_area, row, scrollable, text, text_input,
    },
    window, Border, Color, Element, Event, Length, Size, Subscription, Task, Theme,
};

fn boot() -> (Spike, Task<Message>) {
    (Spike::default(), Task::none())
}

fn spike_theme(_state: &Spike) -> Theme {
    Theme::Dark
}

fn main() -> iced::Result {
    application(boot, update, view)
        .title("452 IME Spike")
        .theme(spike_theme)
        .window_size(Size::new(1180.0, 780.0))
        .subscription(subscription)
        .run()
}

// ---------- 状态与消息 ----------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Side {
    A,
    B,
}

fn side_name(side: Side) -> &'static str {
    match side {
        Side::A => "A(左)",
        Side::B => "B(右)",
    }
}

#[derive(Debug, Clone)]
enum Message {
    Event(String), // 订阅捕获的原始事件行（Keyboard / IME）
    WindowOpened(window::Id), // 主窗口 id 捕获（0.14 的 id 由 shell 内部生成）
    Esc,
    VwPressed(Side),
    LeftInput(String),
    RightInput(String),
    HostInput(String),
    LeftSubmit,
    RightSubmit,
    HostSubmit,
    FocusLeft,
    FocusRight,
    ToggleFullscreen,
    ToggleHostLayer,
}

#[derive(Debug, Default)]
struct Spike {
    left: String,
    right: String,
    host: String,
    host_on: bool,
    fullscreen: bool,
    last: Option<Side>,
    last_ime: Option<String>,
    main_id: Option<window::Id>,
    log: Vec<String>,
}

fn left_id() -> iced::widget::Id {
    iced::widget::Id::new("vw-a-input")
}
fn right_id() -> iced::widget::Id {
    iced::widget::Id::new("vw-b-input")
}
fn host_id() -> iced::widget::Id {
    iced::widget::Id::new("host-input")
}

fn push_log(state: &mut Spike, line: String) {
    state.log.push(line);
    if state.log.len() > 400 {
        state.log.remove(0);
    }
}

fn tail(s: &str) -> String {
    let t = s.chars().rev().take(24).collect::<Vec<_>>();
    t.into_iter().rev().collect()
}

// ---------- 订阅：捕获 IME / 键盘事件 ----------

fn subscription(_state: &Spike) -> Subscription<Message> {
    event::listen_with(|event, _status, window_id| match event {
        Event::InputMethod(ime) => Some(Message::Event(format!("IME {ime:?}"))),
        Event::Window(window::Event::Opened { .. }) => {
            Some(Message::WindowOpened(window_id))
        }
        Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => {
            if matches!(key, Key::Named(keyboard::key::Named::Escape)) {
                return Some(Message::Esc);
            }
            // F 键驱动（CUA 无像素点击备援）：F1 聚焦A / F2 聚焦B / F3 宿主层 / F11 全屏
            if let Key::Named(named) = key {
                return match named {
                    keyboard::key::Named::F1 => Some(Message::FocusLeft),
                    keyboard::key::Named::F2 => Some(Message::FocusRight),
                    keyboard::key::Named::F3 => Some(Message::ToggleHostLayer),
                    keyboard::key::Named::F11 => Some(Message::ToggleFullscreen),
                    _ => None,
                };
            }
            let k = match key {
                Key::Named(n) => format!("{n:?}"),
                Key::Character(c) => format!("'{c}'"),
                Key::Unidentified => "Unidentified".to_string(),
            };
            Some(Message::Event(format!("Key {k}")))
        }
        _ => None,
    })
}

// ---------- update ----------

fn update(state: &mut Spike, message: Message) -> Task<Message> {
    match message {
        Message::Event(line) => {
            if line.starts_with("IME") {
                state.last_ime = Some(line.clone());
            }
            push_log(state, line);
            Task::none()
        }
        Message::WindowOpened(id) => {
            if state.main_id.is_none() {
                push_log(state, format!("主窗口 id = {id}"));
            }
            state.main_id = Some(id);
            Task::none()
        }
        Message::Esc => {
            push_log(state, "Esc".to_string());
            if state.fullscreen {
                state.fullscreen = false;
                push_log(state, "退出全屏".to_string());
                let id = state.main_id.unwrap_or_else(window::Id::unique);
                window::set_mode(id, window::Mode::Windowed)
            } else {
                Task::none()
            }
        }
        Message::VwPressed(side) => {
            state.last = Some(side);
            push_log(state, format!("Click → 聚焦 {}", side_name(side)));
            Task::none()
        }
        Message::LeftInput(v) => {
            if v != state.left {
                push_log(state, format!("A.on_input {:?}", tail(&v)));
                state.last = Some(Side::A);
            }
            state.left = v;
            Task::none()
        }
        Message::RightInput(v) => {
            if v != state.right {
                push_log(state, format!("B.on_input {:?}", tail(&v)));
                state.last = Some(Side::B);
            }
            state.right = v;
            Task::none()
        }
        Message::HostInput(v) => {
            state.host = v;
            Task::none()
        }
        Message::LeftSubmit => {
            push_log(state, "A Enter(on_submit)".to_string());
            Task::none()
        }
        Message::RightSubmit => {
            push_log(state, "B Enter(on_submit)".to_string());
            Task::none()
        }
        Message::HostSubmit => {
            let committed = state.host.clone();
            let target = state.last.unwrap_or(Side::A);
            match target {
                Side::A => state.left.push_str(&committed),
                Side::B => state.right.push_str(&committed),
            }
            state.host.clear();
            push_log(
                state,
                format!("HOST→{} 提交 {:?}", side_name(target), &committed),
            );
            widget_focus(host_id())
        }
        Message::FocusLeft => widget_focus(left_id()),
        Message::FocusRight => widget_focus(right_id()),
        Message::ToggleFullscreen => {
            state.fullscreen = !state.fullscreen;
            let mode = if state.fullscreen {
                window::Mode::Fullscreen
            } else {
                window::Mode::Windowed
            };
            push_log(
                state,
                format!("全屏切换 → {}", if state.fullscreen { "开" } else { "关" }),
            );
            let id = state.main_id.unwrap_or_else(window::Id::unique);
            window::set_mode(id, mode)
        }
        Message::ToggleHostLayer => {
            state.host_on = !state.host_on;
            push_log(
                state,
                format!(
                    "宿主输入层 → {}",
                    if state.host_on { "开" } else { "关" }
                ),
            );
            if state.host_on {
                widget_focus(host_id())
            } else {
                Task::none()
            }
        }
    }
}

fn widget_focus(id: iced::widget::Id) -> Task<Message> {
    iced::widget::operation::focus(id)
}

// ---------- view ----------

fn view(state: &Spike) -> Element<'_, Message> {
    let toolbar = row![
        button(text(if state.fullscreen { "退出全屏" } else { "进入全屏" }).size(13))
            .on_press(Message::ToggleFullscreen),
        button(
            text(if state.host_on { "关闭宿主层" } else { "打开宿主层" }).size(13)
        )
        .on_press(Message::ToggleHostLayer),
        button(text("聚焦 A").size(13)).on_press(Message::FocusLeft),
        button(text("聚焦 B").size(13)).on_press(Message::FocusRight),
        text(format!(
            "最近聚焦: {}｜最近 IME: {}",
            state.last.map(side_name).unwrap_or("-"),
            state.last_ime.as_deref().unwrap_or("-")
        ))
        .size(12),
    ]
    .spacing(8);

    let host_layer = state.host_on.then(|| {
        container(row![
            text("宿主 IME 输入层").size(13),
            text_input("组合输入落于此层，Enter 提交到最近聚焦的虚拟窗口", &state.host)
                .id(host_id())
                .on_input(Message::HostInput)
                .on_submit(Message::HostSubmit)
                .size(13),
            button(text("提交→最近聚焦").size(13)).on_press(Message::HostSubmit),
        ]
        .spacing(10))
        .padding(10)
        .style(|_t| host_style())
    });

    let vw_a = virtual_window(
        "Virtual Window A（左）",
        &state.left,
        left_id(),
        Message::LeftInput,
        Message::LeftSubmit,
        Message::VwPressed(Side::A),
        Color::from_rgb(0.25, 0.55, 0.95),
    );
    let vw_b = virtual_window(
        "Virtual Window B（右）",
        &state.right,
        right_id(),
        Message::RightInput,
        Message::RightSubmit,
        Message::VwPressed(Side::B),
        Color::from_rgb(0.85, 0.45, 0.85),
    );

    let desktop = row![vw_a, vw_b].spacing(16).height(Length::Fill);

    // 最新日志在顶部（截图友好）
    let lines: Vec<Element<'_, Message>> = state
        .log
        .iter()
        .rev()
        .take(16)
        .map(|l| text(l.clone()).size(11).into())
        .collect();
    let log_panel = container(
        scrollable(column(lines).spacing(2)).height(Length::Fixed(150.0)),
    )
    .padding(8)
    .style(|_t| log_style());

    let mut col = column![toolbar].spacing(10);
    if let Some(h) = host_layer {
        col = col.push(h);
    }
    col = col.push(desktop).push(text("事件日志（▲最新在上，Keyboard/InputMethod 实时流）").size(10)).push(log_panel);

    col.padding(12).height(Length::Fill).into()
}

fn virtual_window<'a>(
    title: &str,
    value: &str,
    id: iced::widget::Id,
    on_input: fn(String) -> Message,
    on_submit: Message,
    pressed: Message,
    accent: Color,
) -> Element<'a, Message> {
    let title_bar = container(row![text(title.to_string()).size(13)].spacing(6))
        .padding(8)
        .style(move |_t| title_style(accent));

    let body = column![
        text_input("在此输入中文…", value)
            .id(id)
            .on_input(on_input)
            .on_submit(on_submit),
        text("点击或 Tab 切换窗口焦点；输入中文观察组合(preedit)与提交(commit)")
            .size(10),
    ]
    .spacing(8);

    let card = container(column![title_bar, body].spacing(0))
        .style(move |_t| card_style(accent))
        .width(Length::Fill)
        .height(Length::Fill)
        .clip(true);

    mouse_area(card).on_press(pressed).into()
}

// ---------- 样式 ----------

fn card_style(accent: Color) -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(Color::from_rgb(0.09, 0.12, 0.17).into()),
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: accent,
        },
        ..Default::default()
    }
}

fn title_style(accent: Color) -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(accent.into()),
        ..Default::default()
    }
}

fn host_style() -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(Color::from_rgb(0.35, 0.22, 0.08).into()),
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: Color::from_rgb(0.95, 0.65, 0.2),
        },
        ..Default::default()
    }
}

fn log_style() -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(Color::from_rgb(0.02, 0.02, 0.03).into()),
        border: Border {
            radius: 4.0.into(),
            width: 1.0,
            color: Color::from_rgb(0.3, 0.3, 0.35),
        },
        ..Default::default()
    }
}
