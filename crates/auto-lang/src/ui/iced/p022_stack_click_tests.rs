//! PLAN-022 T-05:分屏交互路由的 headless 回归——relative 容器内两个
//! absolute 浮层(左右各半、各含一个计数按钮),模拟点击两侧,断言
//! 两个按钮都能收到(此前右侧浮层交互全灭:Opaque 满幅捕获 + 光标
//! 抬升级联)。
//!
//! Run: cargo test -p auto-lang --features ui-iced,iced-layout-tests --lib p022_stack
#![cfg(all(test, feature = "iced-layout-tests"))]

use crate::ui::iced::renderer::IntoIcedElement;
use crate::ui::view::View;
use iced_test::simulator;
use std::rc::Rc;
use std::cell::Cell;

#[derive(Clone, Debug, PartialEq)]
enum Msg {
    A,
    B,
}

fn two_layer_view(hits: (Rc<Cell<u32>>, Rc<Cell<u32>>)) -> View<Msg> {
    let left = View::col()
        .style("absolute top-0 left-0 w-[300px] h-[300px] bg-[#202020]")
        .child({
            let a = hits.0.clone();
            View::button("A")
                .style("w-full h-full")
                .on_click(move |_| {
                    a.set(a.get() + 1);
                    Msg::A
                })
                .build()
        })
        .build();
    let right = View::col()
        .style("absolute top-0 left-[300px] w-[300px] h-[300px] bg-[#303030]")
        .child({
            let b = hits.1.clone();
            View::button("B")
                .style("w-full h-full")
                .on_click(move |_| {
                    b.set(b.get() + 1);
                    Msg::B
                })
                .build()
        })
        .build();
    View::col()
        .style("relative w-[600px] h-[300px] bg-background")
        .child(left)
        .child(right)
        .build()
}

#[test]
fn p022_both_absolute_float_layers_clickable() {
    let hits: (Rc<Cell<u32>>, Rc<Cell<u32>>) = (Default::default(), Default::default());
    let view = two_layer_view(hits.clone());
    let mut ui = simulator(view.into_iced());

    // 左浮层中心 (150,150)
    ui.point_at(iced::Point::new(150.0, 150.0));
    let _ = ui.simulate(simulator::click());
    // 右浮层中心 (450,150)
    ui.point_at(iced::Point::new(450.0, 150.0));
    let _ = ui.simulate(simulator::click());

    assert_eq!(hits.0.get(), 1, "左浮层按钮必须收到点击");
    assert_eq!(hits.1.get(), 1, "右浮层按钮必须收到点击(此前右侧交互全灭)");
}

/// 头台确定性复现:交替点击两浮层,第一轮后全部交互死亡(计数冻结)。
/// 归属 653 T-00(iced_test 模拟器/运行时交互投递层);本标记挂 ignore,
/// 653 根修后转正。
#[test]
#[ignore = "交互死亡复现:归属 PLAN-653 T-00(653 创建会话被并行清理,重做后转正)"]
fn p022_repeat_clicks_on_both_layers() {
    let hits: (Rc<Cell<u32>>, Rc<Cell<u32>>) = (Default::default(), Default::default());
    let view = two_layer_view(hits.clone());
    let mut ui = simulator(view.into_iced());

    let mut fired: Vec<String> = Vec::new();
    for round in 1..=3 {
        ui.point_at(iced::Point::new(150.0, 150.0));
        let _ = ui.simulate(simulator::click());
        fired.push(format!("r{}A->{}", round, hits.0.get()));
        ui.point_at(iced::Point::new(450.0, 150.0));
        let _ = ui.simulate(simulator::click());
        fired.push(format!("r{}B->{}", round, hits.1.get()));
    }
    panic!("轨迹: {:?}", fired);
}
