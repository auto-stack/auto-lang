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
        selectable: false,
    }
}

fn bounds_of<M: Clone + std::fmt::Debug>(ui: &mut iced_test::Simulator<'_, M, iced::Theme, iced::Renderer>, needle: &str) -> (f32, f32, f32, f32) {
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
                onclick: None,
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
                selectable: false,
            },
            styled_view("SURVIVOR"),
        ],
        spacing: 0,
        padding: 0,
        style: None,
                onclick: None,
            };
    let mut ui = simulator(view.into_iced());
    let (x, _y, w, _h) = bounds_of(&mut ui, "SURVIVOR");
    assert!(w > 0.0, "sibling after a Fill child must keep width, got {w}");
    assert!(x >= 0.0);
}

/// Plan 448 续 (center parity, END-TO-END): the REAL 002-counter source
/// through parse → extract → DynamicComponent view → into_iced → headless
/// layout. The counter text must sit at the button row's horizontal center
/// (Vue `items-center` parity).
#[test]
fn center_parity_002_counter_end_to_end() {
    let src = concat!(
        "widget App {
",
        "    model { var count int = 0 }
",
        "    view {
",
        "        center {
",
        "            text `Counter: 0`
",
        "            row {
",
        "                button \"-\" { onclick: () => {.count -= 1} }
",
        "                button \"Reset\" { onclick: () => {.count = 0} }
",
        "                button \"+\" { onclick: () => {.count += 1} }
",
        "            }
",
        "        }
",
        "    }
",
        "}
"
    );
    let session = crate::session::CompilerSession::ui();
    let mut parser = crate::Parser::from(src).with_session(session);
    let ast = parser.parse().expect("parse");
    let decl = ast.stmts.iter().find_map(|s| match s {
        crate::ast::Stmt::WidgetDecl(d) => Some(d),
        _ => None,
    }).expect("decl");
    let widget = crate::aura::extract::extract_widget_from_decl(decl).expect("extract");
    let comp = crate::ui::dynamic::DynamicComponent::new(&widget).unwrap();
    let (view, _ids, _probe) = comp.view_with_debug_gated(false);
    let mut ui = simulator(view.into_iced());
    let (tx, _ty, tw, _th) = bounds_of(&mut ui, "Counter: 0");
    // button labels: find all three and take the row extent
    let (b1x, _b1y, b1w, _b1h) = bounds_of(&mut ui, "-");
    let (b3x, _b3y, b3w, _b3h) = bounds_of(&mut ui, "+");
    let tc = tx + tw / 2.0;
    let rc = (b1x + b1w / 2.0 + b3x + b3w / 2.0) / 2.0;
    assert!(
        (tc - rc).abs() < 1.5,
        "text center {tc} must equal button-row center {rc} (center items parity)"
    );
}

/// Plan 448 续 (center parity): `center { text; row{…} }` — the auto-wrapped
/// multi-child column carries ItemsCenter, so every child is horizontally
/// centered like Vue's `flex flex-col items-center`; a narrow text above a
/// wider row must sit at the row's CENTER, not at its left edge (002-counter
/// VM vs Vue divergence).
#[test]
fn center_column_items_center_centers_narrow_child() {
    use crate::ui::style::{SizeValue, StyleClass};
    let inner = View::<()>::Column {
        children: vec![
            View::Text {
                content: "Counter: 0".to_string(),
                style: None,
                selectable: false,
            },
            View::Row {
                children: vec![View::Text {
                    content: "AAAAAAAAAAAAAAAAAAAA".to_string(),
                    style: None,
                    selectable: false,
                }],
                spacing: 0,
                padding: 0,
                style: None,
                onclick: None,
            },
        ],
        spacing: 0,
        padding: 0,
        style: Some(Style::default().add(StyleClass::ItemsCenter)),
                onclick: None,
            };
    let view = View::container(inner)
        .center_x()
        .center_y()
        .with_style(
            Style::default()
                .add(StyleClass::Width(SizeValue::Full))
                .add(StyleClass::Height(SizeValue::Full)),
        )
        .build();
    let mut ui = simulator(view.into_iced());
    let (tx, _ty, tw, _th) = bounds_of(&mut ui, "Counter: 0");
    let (rx, _ry, rw, _rh) = bounds_of(&mut ui, "AAAAAAAAAAAAAAAAAAAA");
    let tc = tx + tw / 2.0;
    let rc = rx + rw / 2.0;
    assert!(
        (tc - rc).abs() < 1.0,
        "text center {tc} must equal row center {rc} (items-center column)"
    );
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
                onclick: None,
            },
        ],
        spacing: 0,
        padding: 0,
        style: Some(Style::parse("w-full").ok().unwrap()),
                onclick: None,
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
            disabled: false,
            style: Style::parse("h-7 w-7 px-0 py-0").ok(),
            on_right_click: None,
            content: None,
        }],
        spacing: 0,
        padding: 0,
        style: Some(Style::parse("items-center bg-[#1C1D24]").ok().unwrap()),
                onclick: None,
            };
    let view = View::Row {
        children: vec![
            inner,
            styled_view("OUTER"),
        ],
        spacing: 0,
        padding: 0,
        style: None,
                onclick: None,
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
        disabled: false,
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
                disabled: false,
                style: None,
                on_right_click: None,
                content: None,
            },
        ],
        spacing: 0,
        padding: 0,
        style: None,
                onclick: None,
            };
    let view = View::Row {
        children: vec![inner, icon_btn("\u{EE01}save\u{EE02}")],
        spacing: 0,
        padding: 0,
        style: None,
                onclick: None,
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
                disabled: false,
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
                onclick: None,
                style: None,
            }),
            placement,
            open: true,
            on_dismiss: None,
        }],
        spacing: 0,
        padding: 0,
        style: None,
                onclick: None,
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
                onclick: None,
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
                onclick: None,
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
                disabled: false,
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
                onclick: None,
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
        disabled: false,
        style: None,
        on_right_click: None,
        content: None,
    };
    View::Column {
        children: vec![View::Popover {
            anchor: PopoverAnchor::Widget(Box::new(View::Button {
                label: "TRIGBTN".to_string(),
                onclick: PopMsg::Trig,
                disabled: false,
                style: None,
                on_right_click: None,
                content: None,
            })),
            content: Box::new(View::Column {
                children: vec![panel_item("PANELITEM")],
                spacing: 0,
                padding: 0,
                style: None,
                onclick: None,
            }),
            placement: PopoverPlacement::BottomStart,
            open: true,
            on_dismiss: Some(PopMsg::Dismiss),
        }],
        spacing: 0,
        padding: 0,
        style: None,
                onclick: None,
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

/// Plan 496 M5 T3：桌面层 z 槽——App 虚拟窗覆盖桌面图标的装配几何断言。
/// 复刻 view() 的 Stack 装配序（壁纸层[省略，纯底] → 桌面图标面 → 虚拟窗
/// z_order），断言首枚图标格落在虚拟窗矩形内（Stack 底序绘制 → 窗口
/// 不透明 chrome 盖住图标，G3「窗口拖过时图标自然被覆盖」）。
#[test]
fn desktop_surface_z_slot_window_covers_icons() {
    // 真 desktop.at 装载 + 三条目注入（投影形状 = renderer
    // inject_desktop_surface 的 {id,icon,label,src}）。
    let mut comp = crate::ui::shell::build_desktop_surface_component().expect("desktop.at 装载");
    let entries: Vec<auto_val::Value> = ["011-calculator", "013-todo", "015-notes"]
        .iter()
        .map(|id| {
            auto_val::Value::Obj(auto_val::Obj::from_pairs([
                ("id", auto_val::Value::Str((*id).into())),
                ("icon", auto_val::Value::Str("app-window".into())),
                ("label", auto_val::Value::Str((*id).into())),
                ("src", auto_val::Value::Str("pinned".into())),
            ]))
        })
        .collect();
    let _ = comp.write_state_vec("__desktop_icons", entries);
    let (view, _, _) = comp.view_with_debug_gated(false);
    let surface_el: iced::Element<'static, ()> = view.map_msg(|_| ()).into_iced();

    // 虚拟窗（0,0 起 400×300——与左上角图标区重叠）：真 VWinState 经
    // 会话 wm_add_win 落（字段全、与实机同源）。
    let mut ds = crate::ui::session::DesktopSession::__test_session();
    ds.open_desktop(iced::window::Id::unique());
    let app = ds.allocate_app(crate::build_dynamic_component(
        "widget T3Stub {\n    model { var n int = 0 }\n    view { text \"WINCLIENT\" }\n}\n",
        None,
    )
    .unwrap());
    let wid = ds.wm_add_win(
        app,
        "T3W".to_string(),
        iced::Rectangle::new(iced::Point::new(0.0, 0.0), iced::Size::new(400.0, 300.0)),
    );
    let vwin = {
        let host = ds.host.as_ref().expect("desktop mode");
        host.wm.wins.get(&wid).expect("vwin 登记完成").clone()
    };
    let client: iced::Element<'_, crate::ui::session::DesktopMessage> =
        iced::widget::text("WINCLIENT").into();
    let win_el: iced::Element<'static, ()> =
        crate::ui::iced::virtual_window::virtual_window_element(&vwin, true, client)
            .map(|_| ());

    // Stack push 序 = view() 装配序（surface 先于虚拟窗 = 底序）。
    let stack: iced::Element<'static, ()> =
        iced::widget::Stack::new().push(surface_el).push(win_el).into();
    let mut ui = simulator(stack);
    let (ix, iy, iw, ih) = bounds_of(&mut ui, "011-calculator");
    assert!(iw > 0.0 && ih > 0.0, "图标 label 可见: {iw}x{ih}");
    // 图标格落在虚拟窗矩形 (0,0,400,300) 内 → Stack 底序绘制下被窗覆盖。
    assert!(
        ix >= 0.0 && iy >= 0.0 && ix + iw <= 400.5 && iy + ih <= 300.5,
        "首枚图标格应在虚拟窗矩形内（被覆盖几何）: ({ix},{iy})+{iw}x{ih}"
    );
    // 虚拟窗客户区文本同时渲染（同区域共存 = 层叠而非互斥布局）。
    let (wx, wy, ww, wh) = bounds_of(&mut ui, "WINCLIENT");
    assert!(ww > 0.0 && wh > 0.0, "虚拟窗客户区渲染: {ww}x{wh}");
    assert!(wy >= 0.0 && wy < 300.0, "窗客户区在窗矩形内: y={wy}");
    // 窗标题文本（chrome）也渲染于重叠区（覆盖关系的可见证据）。
    let (_tx, ty, _tw, th) = bounds_of(&mut ui, "T3W");
    assert!(th > 0.0 && ty < 40.0, "chrome 标题条在窗顶: y={ty}");
}

// ========== Plan 045 T2 — 表格列宽两态 lowering 结构 ==========

fn plan045_table_view<M: Clone + std::fmt::Debug>(col_widths: Option<Vec<f32>>) -> View<M> {
    View::Table {
        headers: vec![View::text("AAA"), View::text("BBB")],
        rows: vec![vec![View::text("aaa"), View::text("bbb")]],
        spacing: 0,
        col_spacing: 8,
        style: None,
        col_widths,
        on_col_resize: None,
    }
}

/// Plan 045 T2: col_widths Some → 列按固定 px 分列（表头与体列同源）。
/// 固定态 [200, 300]：col0 文本 x=16（cell 左 padding），col1 文本
/// x=200+8+16=224；体行同列同 x。截断/补自然宽的纯函数面由
/// renderer::test_plan045_col_fixed_width_dispatch 覆盖。
#[test]
fn plan045_table_fixed_col_widths_layout() {
    let view: View<()> = plan045_table_view(Some(vec![200.0, 300.0]));
    let mut ui = simulator(view.into_iced());
    let (ax, ay, _aw, _ah) = bounds_of(&mut ui, "AAA");
    let (bx, _by, _bw, _bh) = bounds_of(&mut ui, "BBB");
    let (ax2, ay2, _aw2, _ah2) = bounds_of(&mut ui, "aaa");
    let (bx2, _by2, _bw2, _bh2) = bounds_of(&mut ui, "bbb");
    assert!((ax - 16.0).abs() < 0.5, "col0 文本起于左 padding: {ax}");
    assert!((bx - 224.0).abs() < 0.5, "col1 文本起于 200+8+16: {bx}");
    assert!((ax2 - ax).abs() < 0.5 && (bx2 - bx).abs() < 0.5, "体列与表头列同源");
    assert!(ay2 > ay + 4.0 && ay > 0.0, "两行纵向分离: {ay} vs {ay2}");
}

/// Plan 045 T2: col_widths None → 自然宽（现状零回归）——col1 起点由
/// col0 内容自然宽决定（“AAA”远窄于 200），不在 224 固定锚点上。
#[test]
fn plan045_table_natural_col_widths_layout() {
    let view: View<()> = plan045_table_view(None);
    let mut ui = simulator(view.into_iced());
    let (ax, _ay, _aw, _ah) = bounds_of(&mut ui, "AAA");
    let (bx, _by, _bw, _bh) = bounds_of(&mut ui, "BBB");
    assert!((ax - 16.0).abs() < 0.5, "col0 文本仍起于左 padding: {ax}");
    assert!(bx < 224.0 - 40.0, "自然宽 col1 起点显著早于固定锚点: {bx}");
    assert!(bx > ax + 8.0, "col1 在 col0 之后: {ax} vs {bx}");
}

// ========== Plan 045 T3 — 列宽拖拽全链（命中→Drag→松手消息） ==========

#[derive(Debug, Clone, PartialEq)]
enum Plan045ResizeMsg {
    Resized(usize, f32),
}

fn plan045_resize_table_view(col_widths: Option<Vec<f32>>) -> View<Plan045ResizeMsg> {
    use crate::ui::view::ColResizeCallback;
    View::Table {
        headers: vec![View::text("AAA"), View::text("BBB")],
        rows: vec![vec![View::text("aaa"), View::text("bbb")]],
        spacing: 0,
        col_spacing: 8,
        style: None,
        col_widths,
        on_col_resize: Some(ColResizeCallback::new(|m| {
            Plan045ResizeMsg::Resized(m.col, m.width)
        })),
    }
}

/// Plan 045 T3: 全链——表头列边界（10px 带）按下 → 拖拽（临时宽实时进
/// 布局，BBB 列起点随动）→ 松手 publish 落定消息（拖拽中零消息）。
#[test]
fn plan045_table_resize_drag_chain_publishes_on_release() {
    use iced::event::Event;
    use iced::mouse;
    use iced::Point;

    let view = plan045_resize_table_view(Some(vec![200.0, 300.0]));
    let mut ui = simulator(view.into_iced());
    // col0 右边界 x=200；表头带 y=8（首行内）。
    ui.point_at(Point::new(200.0, 8.0));
    // 按下 + 拖 +50 → 临时宽 250（尚未发布）。
    ui.simulate([
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
        Event::Mouse(mouse::Event::CursorMoved { position: Point::new(250.0, 8.0) }),
    ]);
    let (_, _, _, _) = (0.0f32, 0.0, 0.0, 0.0);
    // 临时宽实时进布局：col1 起点从 200+8+16=224 移到 250+8+16=274。
    let (bx, _by, _bw, _bh) = bounds_of(&mut ui, "BBB");
    assert!(
        (bx - 274.0).abs() < 1.0,
        "拖拽中临时宽应实时生效（BBB 起点≈274，实测 {bx}）"
    );
    // 松手 → 落定消息（col 0, width 250）。
    ui.simulate([Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))]);
    let msgs: Vec<Plan045ResizeMsg> = ui.into_messages().collect();
    assert_eq!(msgs, vec![Plan045ResizeMsg::Resized(0, 250.0)]);
}

/// Plan 045 T3: 负向拖拽 clamp 到最小宽 40（vue 金标 max(40,…)）。
#[test]
fn plan045_table_resize_drag_clamps_to_min_width() {
    use iced::event::Event;
    use iced::mouse;
    use iced::Point;

    let view = plan045_resize_table_view(Some(vec![200.0, 300.0]));
    let mut ui = simulator(view.into_iced());
    ui.point_at(Point::new(200.0, 8.0));
    ui.simulate([
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
        Event::Mouse(mouse::Event::CursorMoved { position: Point::new(-500.0, 8.0) }),
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
    ]);
    let msgs: Vec<Plan045ResizeMsg> = ui.into_messages().collect();
    assert_eq!(msgs, vec![Plan045ResizeMsg::Resized(0, 40.0)]);
}

/// Plan 045 T3: 列内远离边界（命中带外）按下拖拽不发消息——只有列边界
/// 10px 带是拖拽把手。
#[test]
fn plan045_table_resize_out_of_band_press_is_inert() {
    use iced::event::Event;
    use iced::mouse;
    use iced::Point;

    let view = plan045_resize_table_view(Some(vec![200.0, 300.0]));
    let mut ui = simulator(view.into_iced());
    // x=100 是 col0 正中（边界 200 的带外）。
    ui.point_at(Point::new(100.0, 8.0));
    ui.simulate([
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
        Event::Mouse(mouse::Event::CursorMoved { position: Point::new(150.0, 8.0) }),
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
    ]);
    let msgs: Vec<Plan045ResizeMsg> = ui.into_messages().collect();
    assert!(msgs.is_empty(), "带外按压不应产生消息: {msgs:?}");
}

/// Plan 045 T3: 表头带外（体行内）的边界位置按压不触发——命中区域限定
/// 表头行。
#[test]
fn plan045_table_resize_body_row_press_is_inert() {
    use iced::event::Event;
    use iced::mouse;
    use iced::Point;

    let view = plan045_resize_table_view(Some(vec![200.0, 300.0]));
    let mut ui = simulator(view.into_iced());
    // 体行 y（表头高≈文本高+24+1，取 60）在边界 x=200 按压。
    ui.point_at(Point::new(200.0, 60.0));
    ui.simulate([
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
        Event::Mouse(mouse::Event::CursorMoved { position: Point::new(250.0, 60.0) }),
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
    ]);
    let msgs: Vec<Plan045ResizeMsg> = ui.into_messages().collect();
    assert!(msgs.is_empty(), "体行按压不应触发拖拽: {msgs:?}");
}
