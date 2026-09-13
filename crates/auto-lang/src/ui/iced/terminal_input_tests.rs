//! 014 直键入:terminal 组件键盘捕获的 headless 集成测试(iced_test
//! simulator 管线,terminal_pixel_tests 同款门控)。
//!
//! 断言面(端到端 widget 轨,非纯函数):
//! 1. 未聚焦(未点击)时键入不入队、不发消息——焦点门控;
//! 2. 点击终端聚焦后,typewrite 的字符逐键入队(FIFO)且每键发布一次
//!    `on_input` 消息;
//! 3. Enter 翻译为 CR 进队列(宿主引擎泵的裸写载荷)。
//!
//! Run with:
//! `cargo test -p auto-lang --features ui-iced,iced-layout-tests --lib terminal_input`
#![cfg(all(test, feature = "iced-layout-tests"))]

use crate::ui::iced::renderer::IntoIcedElement;
use crate::ui::terminal::{terminal, terminal_dispose, terminal_drain_input};
use crate::ui::view::View;
use iced_test::simulator;

const KEY: &str = "t8in";

#[derive(Clone, Debug, PartialEq, Eq)]
enum KeyMsg {
    KeyIn,
}

fn terminal_view() -> View<KeyMsg> {
    View::Terminal {
        key: KEY.to_string(),
        cols: 40,
        rows: 10,
        lines: vec![],
        scroll_offset: 0,
        preedit: None,
        on_select: None,
        on_menu: None,
        on_input: Some(KeyMsg::KeyIn),
        cursor_row: 0,
        cursor_col: 0,
        style: None,
    }
}

#[test]
fn keyboard_capture_gated_by_focus_then_queues_and_publishes() {
    terminal_dispose(KEY);
    let core = terminal(KEY, 40, 10);
    let mut ui = simulator(terminal_view().into_iced());

    // 未聚焦:键入被门控丢弃(不入队、不发消息)。
    ui.typewrite("x");
    assert_eq!(terminal_drain_input(core), None, "未聚焦不得入队");

    // 点击终端(组件中心)获得键入焦点。
    ui.point_at(iced::Point::new(100.0, 50.0));
    let _ = ui.simulate(simulator::click());

    // 聚焦后逐键入队 + 每键一次 on_input 消息。
    ui.typewrite("dir");
    assert_eq!(terminal_drain_input(core).as_deref(), Some("d"), "FIFO 首键");
    assert_eq!(terminal_drain_input(core).as_deref(), Some("i"));
    assert_eq!(terminal_drain_input(core).as_deref(), Some("r"));
    assert_eq!(terminal_drain_input(core), None, "恰好三键");

    // Enter → CR(VT 串,引擎泵裸写的回车载荷)。
    ui.tap_key(iced::keyboard::Key::Named(iced::keyboard::key::Named::Enter));
    assert_eq!(terminal_drain_input(core).as_deref(), Some("\r"));

    let msgs: Vec<KeyMsg> = ui.into_messages().collect();
    let hits = msgs.iter().filter(|m| **m == KeyMsg::KeyIn).count();
    assert_eq!(hits, 4, "三字符 + Enter 各发布一次 KeyIn,实际 {msgs:?}");

    terminal_dispose(KEY);
}
