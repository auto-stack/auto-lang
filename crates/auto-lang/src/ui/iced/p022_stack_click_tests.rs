//! PLAN-022 T-05 → PLAN-023 T-03:分屏交互路由的 headless 回归——
//! relative 容器内两个 absolute 浮层(左右各半、各含一个满幅按钮),
//! 交替/反转/连击点击,断言消息流全数投递。
//!
//! T-01 判决(docs/plans/evidence/023/t01-verdict.md):本轮"冻结"
//! 有两案——①opaque 捕获边界 × spacer 几何错位(T-02 根修:opaque
//! 下沉 content 级);②`View::on_click` 构建期调用闭包(计数伪影)。
//! 故本套件一律以**消息流**为断言面(闭包计数不可用:它只在
//! into_iced 构建时执行一次)。
//!
//! Run: cargo test -p auto-lang --features ui-iced,iced-layout-tests --lib p022_stack
#![cfg(all(test, feature = "iced-layout-tests"))]

use crate::ui::iced::renderer::IntoIcedElement;
use crate::ui::view::View;
use iced_test::simulator;

#[derive(Clone, Debug, PartialEq)]
enum Msg {
    A,
    B,
}

fn two_layer_view() -> View<Msg> {
    let left = View::col()
        .style("absolute top-0 left-0 w-[300px] h-[300px] bg-[#202020]")
        .child(View::button("A").style("w-full h-full").on_click(|_| Msg::A).build())
        .build();
    let right = View::col()
        .style("absolute top-0 left-[300px] w-[300px] h-[300px] bg-[#303030]")
        .child(View::button("B").style("w-full h-full").on_click(|_| Msg::B).build())
        .build();
    View::col()
        .style("relative w-[600px] h-[300px] bg-background")
        .child(left)
        .child(right)
        .build()
}

fn click_at<M>(ui: &mut simulator::Simulator<'static, M>, x: f32, y: f32) {
    ui.point_at(iced::Point::new(x, y));
    let _ = ui.simulate(simulator::click());
}

#[test]
fn p022_both_absolute_float_layers_clickable() {
    let mut ui = simulator(two_layer_view().into_iced());

    // 左浮层中心 (150,150)
    click_at(&mut ui, 150.0, 150.0);
    // 右浮层中心 (450,150)
    click_at(&mut ui, 450.0, 150.0);

    let msgs: Vec<Msg> = ui.into_messages().collect();
    assert_eq!(msgs, vec![Msg::A, Msg::B], "双浮层单击必须全数投递");
}

/// 头台转正(PLAN-023 T-03,原 #[ignore] 复现用例):交替点击两浮层
/// 三轮,消息流必须 [A,B]×3 全数投递。修复前:右浮层 spacer 行
/// opaque 吞左浮层按压(r*A=[Captured,Ignored]),且计数伪影使轨迹
/// 恒为 1(见套件头注)。
#[test]
fn p022_repeat_clicks_on_both_layers() {
    let mut ui = simulator(two_layer_view().into_iced());

    for _ in 0..3 {
        click_at(&mut ui, 150.0, 150.0);
        click_at(&mut ui, 450.0, 150.0);
    }

    let msgs: Vec<Msg> = ui.into_messages().collect();
    assert_eq!(
        msgs,
        vec![Msg::A, Msg::B, Msg::A, Msg::B, Msg::A, Msg::B],
        "三轮交替必须无冻结全数投递"
    );
}

/// PLAN-023 T-03 回归①:单浮层连击 5 次 = 5 击全达。
#[test]
fn p023_single_layer_repeat_clicks() {
    let mut ui = simulator(two_layer_view().into_iced());
    for _ in 0..5 {
        click_at(&mut ui, 150.0, 150.0);
    }
    let msgs: Vec<Msg> = ui.into_messages().collect();
    assert_eq!(msgs, vec![Msg::A; 5], "同层连击不得衰减");
}

/// PLAN-023 T-03 回归②:先右后左顺序反转,投递与顺序无关。
#[test]
fn p023_reversed_click_order() {
    let mut ui = simulator(two_layer_view().into_iced());
    for _ in 0..2 {
        click_at(&mut ui, 450.0, 150.0);
        click_at(&mut ui, 150.0, 150.0);
    }
    let msgs: Vec<Msg> = ui.into_messages().collect();
    assert_eq!(msgs, vec![Msg::B, Msg::A, Msg::B, Msg::A]);
}

/// PLAN-023 T-03 回归③(T-02 根修钉):spacer 空白区穿透。
/// 浮层 C(top-[100px] left-[100px] w-[100px] h-[100px])的
/// build_floating_layer spacer 行,修复前 opaque 边界=(0..200)×(0..200),
/// 点击 (50,50)(行内空白区)被 C 层吞掉、无人收到;修复后
/// opaque 只包 C 内容,空白区穿透到 base 层按钮。
#[test]
fn p023_spacer_gap_click_falls_through() {
    #[derive(Clone, Debug, PartialEq)]
    enum M2 {
        Base,
        C,
    }
    let base = View::col()
        .style("w-full h-full")
        .child(View::button("BASE").style("w-full h-full").on_click(|_| M2::Base).build())
        .build();
    let c_layer = View::col()
        .style("absolute top-[100px] left-[100px] w-[100px] h-[100px] bg-[#404040]")
        .child(View::button("C").style("w-full h-full").on_click(|_| M2::C).build())
        .build();
    let root = View::col()
        .style("relative w-[600px] h-[300px] bg-background")
        .child(base)
        .child(c_layer)
        .build();
    let mut ui = simulator(root.into_iced());

    // (150,150):C 内容矩形内 → C 认领。
    click_at(&mut ui, 150.0, 150.0);
    // (50,50):spacer 行覆盖区内、C 内容外 → 穿透给 base。
    click_at(&mut ui, 50.0, 50.0);

    let msgs: Vec<M2> = ui.into_messages().collect();
    assert_eq!(
        msgs,
        vec![M2::C, M2::Base],
        "spacer 空白区必须穿透到下层(opaque 边界=内容矩形)"
    );
}
