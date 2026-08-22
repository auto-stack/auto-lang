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
use crate::ui::view::{PopoverAnchor, PopoverPlacement, View};
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

// ── Plan 414 §8.1: nested-row icon-button disappearance ─────────────────
// The 414 record: an EE01 (PUA icon) button inside a NESTED row vanished
// while a text button in the same row survived; flat icons were fine. The
// text selector cannot see svg buttons, so collect ALL focusable bounds
// via a custom selector — every button must keep a non-zero rectangle.
use iced_test::selector::{Candidate, Selector};

/// Collects every container/focusable bounds (kind, w, h). Buttons surface
/// as plain containers in iced 0.14 (Button::operate calls only
/// `container`), and fixed-size buttons additionally get a centering
/// wrapper — hence two same-size entries per icon button. `select` never
/// "matches" (returns None), so the simulator traverses the WHOLE tree
/// before reporting SelectorNotFound — by then the store holds everything.
/// Verified outcome: the nested-row icon button keeps its full 28×28 —
/// 414 §8.1 does NOT reproduce through the current into_iced pipeline
/// (same verdict as §7.2); this test locks that good behavior.
#[derive(Clone)]
struct FocusableCollector(std::sync::Arc<std::sync::Mutex<Vec<(f32, f32, f32)>>>);
impl Selector for FocusableCollector {
    type Output = ();
    fn select(&mut self, candidate: Candidate<'_>) -> Option<()> {
        match candidate {
            Candidate::Focusable { bounds, .. } | Candidate::Container { bounds, .. } => {
                let kind = match candidate {
                    Candidate::Focusable { .. } => 0u8,
                    _ => 1u8,
                };
                self.0.lock().unwrap().push((kind as f32, bounds.width, bounds.height));
            }
            _ => {}
        }
        None
    }
    fn description(&self) -> String {
        "focusable-collector".into()
    }
}

fn all_button_bounds(view: View<()>) -> Vec<(f32, f32, f32)> {
    let mut ui = simulator(view.into_iced());
    let store: std::sync::Arc<std::sync::Mutex<Vec<(f32, f32, f32)>>> = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    // SelectorNotFound is the expected terminal state (see struct doc).
    let _ = ui.find(FocusableCollector(store.clone()));
    let out = store.lock().unwrap().clone();
    assert!(!out.is_empty(), "no focusables collected at all");
    out
}

/// 414 §8.1 exact repro shape: nested row containing an EE01 icon button
/// AND a text button, plus a flat icon control — all three must keep
/// non-zero bounds.
#[test]
fn nested_row_icon_button_keeps_bounds() {
    let icon_btn = |label: &str| View::<()>::Button {
        label: label.to_string(),
        onclick: (),
        style: crate::ui::style::Style::parse("h-7 w-7 px-0 py-0").ok(),
        on_right_click: None,
        content: None,
    };
    let inner = View::Row {
        children: vec![
            icon_btn("\u{EE01}file-plus\u{EE02}"),
            View::Button {
                label: "TB1".to_string(),
                onclick: (),
                style: None,
                on_right_click: None,
                content: None,
            },
        ],
        spacing: 0,
        padding: 0,
        style: None,
    };
    let view = View::Row {
        children: vec![inner, icon_btn("\u{EE01}save\u{EE02}")],
        spacing: 0,
        padding: 0,
        style: None,
    };
    let sizes = all_button_bounds(view);
    assert!(sizes.len() >= 5, "expected rows+3 buttons in the tree: {sizes:?}");
    let buttons: Vec<_> = sizes.iter().filter(|(k, w, _)| *k == 1.0 && *w <= 60.0).collect();
    assert!(buttons.len() >= 3, "three button-sized containers expected: {sizes:?}");
    for (i, (_, w, h)) in buttons.iter().enumerate() {
        assert!(*w > 0.0 && *h > 0.0, "button {i} collapsed to {w}x{h} (414 §8.1 regression)");
    }
}

// ── Plan 422 P1: popover overlay 定位断言 ─────────────────────────────────
// 弹层面板经 iced overlay 机制渲染;Panel overlay 实现了 operate 转发,
// 面板内文本对 iced_test 的 selector 可见(UserInterface::operate 会分派
// 到 overlay —— tooltip 没实现 operate 所以不可见,这里必须可见)。

fn popover_view(placement: PopoverPlacement, anchor_style: &str, panel_width: u16) -> View<()> {
    View::Column {
        children: vec![View::Popover {
            anchor: PopoverAnchor::Widget(Box::new(View::Button {
                label: "ANCHORBTN".to_string(),
                onclick: (),
                style: Style::parse(anchor_style).ok(),
                on_right_click: None,
                content: None,
            })),
            content: Box::new(View::Container {
                child: Box::new(styled_view("PANELTEXT")),
                padding: 0,
                width: Some(panel_width),
                height: None,
                center_x: false,
                center_y: false,
                style: None,
            }),
            placement,
            open: true,
            on_dismiss: None,
        }],
        spacing: 0,
        padding: 0,
        style: None,
    }
}

/// BottomStart:面板左缘对齐锚左缘、顶缘在锚正下方 —— menubar 下拉的
/// 核心定位(取代 418 的 top-[33px] left-[估] 像素估算)。
#[test]
fn popover_bottom_start_aligns_under_anchor() {
    let view = popover_view(PopoverPlacement::BottomStart, "h-7 px-3", 160);
    let mut ui = simulator(view.into_iced());
    let (ax, ay, aw, ah) = bounds_of(&mut ui, "ANCHORBTN");
    let (px, py, pw, ph) = bounds_of(&mut ui, "PANELTEXT");
    assert!(aw > 0.0 && ah > 0.0, "anchor button must keep bounds: {aw}x{ah}");
    assert!(pw > 0.0 && ph > 0.0, "panel must be visible through overlay operate: {pw}x{ph}");
    assert!(py >= ay + ah, "panel must start below anchor bottom: panel y {py} vs anchor {ay}+{ah}");
    // 容器给了固定宽 160;文字在其中。BottomStart = 面板左缘对齐锚按钮
    // 左缘 —— 文字选择器看到的是按钮内文字(px-3 内缩 12px),面板文字
    // 贴容器左缘,故面板文字 x 应落在 [按钮左缘, 按钮文字] 区间内。
    assert!(
        px >= ax - 16.0 && px <= ax,
        "panel left must align with anchor BUTTON left (text inset by px-3): panel x {px} vs anchor text x {ax}"
    );
}

/// 右缘越界 snap:锚贴近视口右缘、面板宽 320,右缘必须收回视口内
/// (~1024 默认视口;Tooltip 同款 snap_within_viewport 逻辑)。
#[test]
fn popover_snaps_within_viewport_right_edge() {
    // ml-auto 把锚组推到行右缘。
    let view = View::Row {
        children: vec![
            styled_view("LEFT"),
            popover_view(PopoverPlacement::BottomStart, "h-7 px-3 ml-auto", 320),
        ],
        spacing: 0,
        padding: 0,
        style: Some(Style::parse("w-full").ok().unwrap()),
    };
    let mut ui = simulator(view.into_iced());
    let (ax, _ay, _aw, ah) = bounds_of(&mut ui, "ANCHORBTN");
    let (px, py, pw, _ph) = bounds_of(&mut ui, "PANELTEXT");
    assert!(ax > 600.0, "anchor should sit near the right edge, got x {ax}");
    assert!(py >= ah, "panel below the 28px-tall anchor row: {py} vs {ah}");
    // snap 后:面板整体在视口内(1024 宽 + 0.5 容差),且左缘被推回
    // (否则 px ≈ ax > 700 且 px + 320 > 1024)。
    assert!(px + pw <= 1024.5, "panel right edge must stay in viewport: {px}+{pw}");
    assert!(px <= 1024.0 - 320.0 + 0.5, "panel must be snapped left: {px}");
}

/// 坐标锚(contextmenu 变体):面板左上角对齐 (x, y) 落点,不受布局影响。
#[test]
fn popover_point_anchor_places_panel_at_coordinate() {
    let view = View::Popover {
        anchor: PopoverAnchor::Point { x: 300.0, y: 200.0 },
        content: Box::new(View::Container {
            child: Box::new(styled_view("CTXITEM")),
            padding: 0,
            width: Some(180),
            height: None,
            center_x: false,
            center_y: false,
            style: None,
        }),
        placement: PopoverPlacement::BottomStart,
        open: true,
        on_dismiss: None,
    };
    let mut ui = simulator(view.into_iced());
    let (px, py, pw, _ph) = bounds_of(&mut ui, "CTXITEM");
    assert!(pw > 0.0, "context panel must be visible: {pw}");
    assert!((px - 300.0).abs() <= 8.0, "panel left at point x=300 (+text padding), got {px}");
    assert!((py - 200.0).abs() <= 8.0, "panel top at point y=200, got {py}");
}

/// 关闭态:面板内容不可见(open=false → overlay 不产出)。
#[test]
fn popover_closed_hides_panel() {
    let view = View::Column {
        children: vec![View::Popover {
            anchor: PopoverAnchor::Widget(Box::new(View::Button {
                label: "CLOSEDBTN".to_string(),
                onclick: (),
                style: None,
                on_right_click: None,
                content: None,
            })),
            content: Box::new(styled_view("HIDDENPANEL")),
            placement: PopoverPlacement::BottomStart,
            open: false,
            on_dismiss: None,
        }],
        spacing: 0,
        padding: 0,
        style: None,
    };
    let mut ui = simulator(view.into_iced());
    let (_bx, _by, bw, bh) = bounds_of(&mut ui, "CLOSEDBTN");
    assert!(bw > 0.0 && bh > 0.0, "anchor button must render when closed: {bw}x{bh}");
    assert!(ui.find("HIDDENPANEL").is_err(), "panel content must NOT be reachable when closed");
}

// ── Plan 422 P1/P3: 弹层行为语义(捕获/dismiss)—— 消息级断言 ────────────
// Simulator 的事件先派发给 overlay(UserInterface::update 语义),据此断言:
// * 面板内点击 → 项消息发布、无 dismiss;
// * 面板外点击 → dismiss 发布且放行(此处无基础树按钮,仅 Dismiss);
// * 锚上点击 → dismiss 发布且【不】透传给锚按钮(点触发器 = 关);
// * Esc → dismiss 发布并捕获。

#[derive(Clone, Debug, PartialEq)]
enum PopMsg {
    Trig,
    Item,
    Dismiss,
}

/// 消息断言用的小视图:锚按钮 TRIGBTN + 开启面板(项按钮 PANELITEM)。
fn popover_semantics_view() -> View<PopMsg> {
    let panel_item = |label: &str| View::<PopMsg>::Button {
        label: label.to_string(),
        onclick: PopMsg::Item,
        style: None,
        on_right_click: None,
        content: None,
    };
    View::Column {
        children: vec![View::Popover {
            anchor: PopoverAnchor::Widget(Box::new(View::Button {
                label: "TRIGBTN".to_string(),
                onclick: PopMsg::Trig,
                style: None,
                on_right_click: None,
                content: None,
            })),
            content: Box::new(View::Column {
                children: vec![panel_item("PANELITEM")],
                spacing: 0,
                padding: 0,
                style: None,
            }),
            placement: PopoverPlacement::BottomStart,
            open: true,
            on_dismiss: Some(PopMsg::Dismiss),
        }],
        spacing: 0,
        padding: 0,
        style: None,
    }
}

#[test]
fn popover_panel_item_click_publishes_item() {
    let mut ui = simulator(popover_semantics_view().into_iced());
    ui.click("PANELITEM").expect("panel item clickable");
    let msgs: Vec<PopMsg> = ui.into_messages().collect();
    assert!(msgs.contains(&PopMsg::Item), "panel item message must publish: {msgs:?}");
    assert!(!msgs.contains(&PopMsg::Dismiss), "in-panel click must not dismiss: {msgs:?}");
}

#[test]
fn popover_outside_click_dismisses() {
    let mut ui = simulator(popover_semantics_view().into_iced());
    // 远离面板/锚的空白处点击。
    ui.point_at(iced::Point::new(900.0, 700.0));
    let _ = ui.simulate(iced_test::simulator::click());
    let msgs: Vec<PopMsg> = ui.into_messages().collect();
    assert!(msgs.contains(&PopMsg::Dismiss), "outside click must publish dismiss: {msgs:?}");
    assert!(!msgs.contains(&PopMsg::Item), "no item message on outside click: {msgs:?}");
    assert!(!msgs.contains(&PopMsg::Trig), "no trigger leak on outside click: {msgs:?}");
}

#[test]
fn popover_anchor_click_dismisses_without_trigger() {
    let mut ui = simulator(popover_semantics_view().into_iced());
    ui.click("TRIGBTN").expect("anchor button visible");
    let msgs: Vec<PopMsg> = ui.into_messages().collect();
    assert!(
        msgs.contains(&PopMsg::Dismiss) && !msgs.contains(&PopMsg::Trig),
        "clicking the anchor while open must dismiss WITHOUT re-triggering: {msgs:?}"
    );
}

#[test]
fn popover_escape_dismisses() {
    let mut ui = simulator(popover_semantics_view().into_iced());
    let status = ui.tap_key(iced::keyboard::Key::Named(
        iced::keyboard::key::Named::Escape,
    ));
    assert_eq!(status, iced::event::Status::Captured, "Esc must be captured by the popover");
    let msgs: Vec<PopMsg> = ui.into_messages().collect();
    assert!(msgs.contains(&PopMsg::Dismiss), "Esc must publish dismiss: {msgs:?}");
}
