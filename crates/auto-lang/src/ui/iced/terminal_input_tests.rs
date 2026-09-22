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
use crate::ui::terminal::{terminal, terminal_claim_focus, terminal_dispose, terminal_drain_input};
use crate::ui::view::View;
use iced_test::simulator;

const KEY: &str = "t8in";

/// 两个测试共享进程级 TERMINALS/FOCUS_OWNER 注册表,
/// 并行线程互踩会互相破坊焦点前提(023 实测),串行化。
static REGISTRY_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

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
        scheme: crate::ui::terminal::TERMINAL_SCHEME_FOLLOW_THEME,
        shortcuts: Vec::new(),
        on_select: None,
        on_menu: None,
        on_input: Some(KeyMsg::KeyIn),
        cursor_row: 0,
        cursor_col: 0,
        history: 0,
        style: None,
    }
}

#[test]
fn keyboard_capture_gated_by_focus_then_queues_and_publishes() {
    let _registry = REGISTRY_LOCK.lock().unwrap();
    terminal_dispose(KEY);
    terminal_dispose("t8in-holder");
    let core = terminal(KEY, 40, 10);
    // 预占焦点(他端持焦形态)——本测试确定性化:
    // 旧版无预占,靠并行测试碰巧占住 FOCUS_OWNER 才能过,
    // solo 跑法必挂(PLAN-023 甄别,启动自动聚焦会自领)。
    let holder = terminal("t8in-holder", 40, 10);
    terminal_claim_focus(&holder);
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
    terminal_dispose("t8in-holder");
}

/// PLAN-023 T-04 用户门双播回归：双终端分屏,点 B 换焦后键入
/// 必须单播(A 不得残留入队)。实车症状:右槽打字左槽同步、
/// Enter 两边同跐——per-widget focused 残留态致键盘门控双放行。
#[test]
fn p023_focus_switch_single_cast() {
    const KA: &str = "p23fa";
    const KB: &str = "p23fb";
    let _registry = REGISTRY_LOCK.lock().unwrap();
    terminal_dispose(KA);
    terminal_dispose(KB);
    let core_a = terminal(KA, 40, 10);
    let core_b = terminal(KB, 40, 10);
    // 预注焦 A:Stack 逆序下自动聚焦竞速由顶层赢,预注焦消除次序依赖。
    terminal_claim_focus(&core_a);
    let term = |key: &str| View::Terminal {
        key: key.to_string(),
        cols: 40,
        rows: 10,
        lines: vec![],
        scroll_offset: 0,
        preedit: None,
        scheme: crate::ui::terminal::TERMINAL_SCHEME_FOLLOW_THEME,
        shortcuts: Vec::new(),
        on_select: None,
        on_menu: None,
        on_input: Some(KeyMsg::KeyIn),
        cursor_row: 0,
        cursor_col: 0,
        history: 0,
        style: None,
    };
    let root = View::col()
        .style("relative w-[600px] h-[300px] bg-background")
        .child(
            View::col()
                .style("absolute top-0 left-0 w-[300px] h-[300px]")
                .child(term(KA))
                .build(),
        )
        .child(
            View::col()
                .style("absolute top-0 left-[300px] w-[300px] h-[300px]")
                .child(term(KB))
                .build(),
        )
        .build();
    let mut ui = simulator(root.into_iced());

    // boot:A 预持焦;键入单播到 A。
    ui.typewrite("a");
    let da = terminal_drain_input(core_a);
    let db = terminal_drain_input(core_b);
    assert_eq!(
        (da.as_deref(), db.as_deref()),
        (Some("a"), None),
        "单播基线 a={da:?} b={db:?}"
    );

    // 点 B 层中心换焦,再键入:必须单播到 B(A 不得残留)。
    ui.point_at(iced::Point::new(450.0, 150.0));
    let _ = ui.simulate(simulator::click());
    ui.typewrite("x");
    assert_eq!(
        terminal_drain_input(core_a),
        None,
        "换焦后旧端不得入队(双播回归)"
    );
    assert_eq!(
        terminal_drain_input(core_b).as_deref(),
        Some("x"),
        "新持焦端入队"
    );

    terminal_dispose(KA);
    terminal_dispose(KB);
}
