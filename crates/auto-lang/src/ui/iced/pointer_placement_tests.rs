//! PLAN-631 F-7: pointer placement 原语 headless 测试（iced_test 管线，
//! layout_tests 同款门控 `iced-layout-tests`）。
//!
//! 覆盖：PointerPressArea 根 wrapper 的真实事件记账（CursorMoved + 右键
//! 事件序列 → 会话单槽）——AC-03 的框架内自动化面；单槽"最近按下"语义
//! （plan §10 Q3 v1）。面板锚归一几何的单测在 popover.rs tests 模块
//! （pointer_panel_anchor 私有函数同文件可达）。
//!
//! Run with:
//! `cargo test -p auto-lang --features ui-iced,iced-layout-tests --lib pointer_placement`
#![cfg(all(test, feature = "iced-layout-tests"))]

use crate::ui::iced::right_press_area::test_support::slot_lock;
use crate::ui::iced::right_press_area::{clear_pointer_press, last_pointer_press, PointerPressArea};

use iced::widget::mouse_area;
use iced::widget::text;
use iced::widget::column;
use iced::{Event, Point};
use iced_test::simulator;

fn right_press_events(pos: Point) -> [Event; 3] {
    [
        Event::Mouse(iced::mouse::Event::CursorMoved { position: pos }),
        Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Right)),
        Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Right)),
    ]
}

fn pointer_root<'a>() -> iced::Element<'a, ()> {
    PointerPressArea::new(column![
        mouse_area(text("TRIGGER").size(20)).on_right_press(()),
    ])
    .into()
}

#[test]
fn pointer_press_records_and_slot_holds_latest() {
    // 单槽为进程级(并发测试互斥不可假定),记账+覆盖语义合并串行断言:
    // 首记 → 槽 = 首点;再按他点 → 后写覆盖,槽 = 最近按下位置(计划
    // §10 Q3 v1 单槽语义)。
    let _guard = slot_lock();
    clear_pointer_press();
    let mut ui = simulator(pointer_root());
    let first = Point::new(137.0, 91.0);
    ui.point_at(first);
    ui.simulate(right_press_events(first));
    assert_eq!(
        last_pointer_press().expect("right-press must be recorded"),
        first,
        "记账坐标 = 事件现场指针坐标"
    );

    let second = Point::new(300.0, 150.0);
    ui.point_at(second);
    ui.simulate(right_press_events(second));
    assert_eq!(
        last_pointer_press().expect("recorded"),
        second,
        "后写覆盖:槽持有最近按下位置"
    );
}
