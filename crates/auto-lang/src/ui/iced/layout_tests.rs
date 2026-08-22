//! Plan 414 §8.2 follow-up: headless iced layout testbench (`iced_test`).
//!
//! The 414 matrix documented VM Row layout bugs (Fill-sized / auto-margin
//! children making siblings vanish, §7.2; nested-row icon buttons
//! disappearing, §8.1) that could previously only be observed on a live
//! window — "iced 无 iced::test,需自建 headless 断言,单独立项". This
//! module is that testbench: bounds assertions against the FULL
//! `View → into_iced` pipeline with a headless wgpu renderer, no window.
//!
//! Run with:
//! `cargo test -p auto-lang --features ui-iced,iced-layout-tests --lib layout_tests`
#![cfg(all(test, feature = "iced-layout-tests"))]

use crate::ui::iced::renderer::IntoIcedElement;
use crate::ui::style::Style;
use crate::ui::view::View;
use iced_test::simulator;
use iced_test::selector::Bounded;

fn styled_view(style: &str) -> View<()> {
    View::Text {
        content: style.to_string(),
        style: Style::parse(style).ok(),
    }
}

fn bounds_of(ui: &mut iced_test::Simulator<'_, (), iced::Theme, iced::Renderer>, needle: &str) -> (f32, f32, f32, f32) {
    let t = ui.find(needle).expect("text not found");
    let b = t.bounds();
    (b.x, b.y, b.width, b.height)
}

/// Smoke: a plain row lays out both texts with non-zero bounds and no
/// overlap. Proves the headless renderer + text-selector plumbing works in
/// this environment before the bug-matrix assertions rely on it.
#[test]
fn row_smoke_two_texts() {
    let view = View::Row {
        children: vec![styled_view("L"), styled_view("R")],
        spacing: 0,
        padding: 0,
        style: None,
    };
    let mut ui = simulator(view.into_iced());
    let (x1, _y1, w1, _h1) = bounds_of(&mut ui, "L");
    let (x2, _y2, w2, _h2) = bounds_of(&mut ui, "R");
    assert!(w1 > 0.0 && w2 > 0.0, "both texts must have width: {w1} {w2}");
    assert!(x2 >= x1 + w1, "R must start after L: {x1}+{w1} vs {x2}");
}

/// Plan 414 §7.2 repro: a Fill-sized child (`flex-1 w-full`) inside a Row
/// must not make its siblings collapse to zero width / disappear.
#[test]
fn row_fill_child_keeps_sibling_visible() {
    let view = View::Row {
        children: vec![
            View::Text {
                content: "FILL".to_string(),
                style: Style::parse("flex-1 w-full").ok(),
            },
            styled_view("SURVIVOR"),
        ],
        spacing: 0,
        padding: 0,
        style: None,
    };
    let mut ui = simulator(view.into_iced());
    let (x, _y, w, _h) = bounds_of(&mut ui, "SURVIVOR");
    assert!(w > 0.0, "sibling after a Fill child must keep width, got {w}");
    assert!(x >= 0.0);
}

/// Plan 414 §7.2 (auto-margin variant) / 418 regression lock: `ml-auto`
/// pushes the wrapped group to the RIGHT edge of the row — the toolbar
/// right-alignment that 414 had to disable and 418 re-enabled.
#[test]
fn row_ml_auto_pushes_right() {
    let view = View::Row {
        children: vec![
            styled_view("LEFT"),
            View::Row {
                children: vec![styled_view("RIGHT")],
                spacing: 0,
                padding: 0,
                style: Style::parse("ml-auto").ok(),
            },
        ],
        spacing: 0,
        padding: 0,
        style: Some(Style::parse("w-full").ok().unwrap()),
    };
    let mut ui = simulator(view.into_iced());
    let (lx, _ly, lw, _lh) = bounds_of(&mut ui, "LEFT");
    let (rx, _ry, rw, _rh) = bounds_of(&mut ui, "RIGHT");
    assert!(rw > 0.0, "ml-auto group must keep width");
    assert!(
        rx >= lx + lw,
        "ml-auto group must sit right of the left group: LEFT {lx}+{lw}, RIGHT {rx}"
    );
    // Right group should hug the right edge of the ~1024px default viewport
    // (a few px of tolerance for the text baseline box).
    assert!(rx + rw > 900.0, "ml-auto group must reach the right edge: {rx}+{rw}");
}

/// Plan 414 §8.1 partial: a button inside a NESTED row must keep bounds
/// (the original repro — icon buttons vanishing — needs svg bounds which
/// the text selector cannot see; the text-button variant at least locks
/// the nested-row layout itself).
#[test]
fn nested_row_button_keeps_bounds() {
    let inner = View::Row {
        children: vec![View::Button {
            label: "NESTEDBTN".to_string(),
            onclick: (),
            style: Style::parse("h-7 w-7 px-0 py-0").ok(),
            on_right_click: None,
            content: None,
        }],
        spacing: 0,
        padding: 0,
        style: Some(Style::parse("items-center bg-[#1C1D24]").ok().unwrap()),
    };
    let view = View::Row {
        children: vec![
            inner,
            styled_view("OUTER"),
        ],
        spacing: 0,
        padding: 0,
        style: None,
    };
    let mut ui = simulator(view.into_iced());
    let (ox, _oy, ow, _oh) = bounds_of(&mut ui, "OUTER");
    assert!(ow > 0.0, "outer sibling must keep width");
    // The button's text label participates in layout — it must exist too
    // (if the nested-row bug resurfaces the label stops being found).
    let (_bx, _by, bw, bh) = bounds_of(&mut ui, "NESTEDBTN");
    assert!(bw > 0.0 && bh > 0.0, "nested button must have bounds: {bw}x{bh}");
}
