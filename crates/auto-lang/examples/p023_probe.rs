// PLAN-023 T-01 划痕探针(未跟踪草稿,永不提交)——纯 iced 复现矩阵。
// 运行:cargo run -p auto-lang --example p023_probe --features ui-iced,iced-layout-tests
//
// 每变体一个 Simulator,交替点击 A(150,150)/B(450,150) 三轮;消息即计数
// (A 发 1,B 发 2);6 连击期望消息序列 [1,2,1,2,1,2],缺失=该击死亡。
use iced::widget::{button, column as col, opaque, space, stack, text};
use iced::{Element, Point};
use std::cell::Cell;
use std::rc::Rc;

fn layer_a() -> Element<'static, u32> {
    col![button(text("A").width(300).height(300).center())
        .width(300)
        .height(300)
        .on_press(1)]
    .width(300)
    .height(300)
    .into()
}

// 镜像 renderer.rs build_floating_layer 的 spacer 几何(左偏移 300)。
fn layer_b() -> Element<'static, u32> {
    col![
        space().height(0),
        row300(),
    ]
    .width(iced::Length::Shrink)
    .height(iced::Length::Shrink)
    .into()
}

fn row300() -> Element<'static, u32> {
    // iced Row 无直接 helper 混排 Space+col,用 row![] 宏。
    iced::widget::row![
        space().width(300),
        col![button(text("B").width(300).height(300).center())
            .width(300)
            .height(300)
            .on_press(2)]
        .width(300)
        .height(300),
    ]
    .into()
}

fn empty_base() -> Element<'static, u32> {
    col![].width(600).height(300).into()
}

enum Wrap {
    Opaque,
    Raw,
}

fn push(stk: iced::widget::Stack<'static, u32>, el: Element<'static, u32>, w: &Wrap) -> iced::widget::Stack<'static, u32> {
    match w {
        Wrap::Opaque => stk.push(opaque(el)),
        Wrap::Raw => stk.push(el),
    }
}

fn main() {
    real_main();
    real_vehicle();
    r4_case();
    r4b_bounds();
    r6_pure_fixed_geometry();
    r7_style_parse_dump();
    r8_run();

    // V1 镜像 p022:stack![空base, opaque(A层), opaque(B层spacer几何)]
    run("V1 镜像p022(空base+opaque双层)", || {
        let s = stack![empty_base()];
        let s = push(s, layer_a(), &Wrap::Opaque);
        let s = push(s, layer_b(), &Wrap::Opaque);
        s.into()
    }, false);

    // V2 无 opaque
    run("V2 无opaque(空base+裸双层)", || {
        let s = stack![empty_base()];
        let s = push(s, layer_a(), &Wrap::Raw);
        let s = push(s, layer_b(), &Wrap::Raw);
        s.into()
    }, false);

    // V3 无空 base:stack![opaque(A), opaque(B)]
    run("V3 无空base(opaque双层)", || {
        let s = stack![layer_a()];
        let s = push(s, layer_b(), &Wrap::Opaque);
        s.into()
    }, false);

    // V4 极简:stack![按钮A, 按钮B](无列无opaque)
    run("V4 极简(裸双按钮)", || {
        stack![button(text("A").width(300).height(300)).on_press(1),
               button(text("B").width(300).height(300)).on_press(2)]
            .into()
    }, false);

    // V5 同层连击:V1 结构,连点 A 三次(期望 [1,1,1])
    run("V5 同层连击AAA(V1结构)", || {
        let s = stack![empty_base()];
        let s = push(s, layer_a(), &Wrap::Opaque);
        let s = push(s, layer_b(), &Wrap::Opaque);
        s.into()
    }, true);
}

fn run(name: &str, build: fn() -> Element<'static, u32>, same_layer: bool) {
    let mut ui = iced_test::simulator::simulator(build());
    let mut st_log: Vec<String> = Vec::new();
    for r in 1..=3 {
        ui.point_at(Point::new(150.0, 150.0));
        let sa = ui.simulate(iced_test::simulator::click());
        st_log.push(format!("r{r}A={sa:?}"));
        if !same_layer {
            ui.point_at(Point::new(450.0, 150.0));
            let sb = ui.simulate(iced_test::simulator::click());
            st_log.push(format!("r{r}B={sb:?}"));
        }
    }
    let msgs: Vec<u32> = ui.into_messages().collect();
    println!("== {name}\n   statuses: {st_log:?}\n   msgs:     {msgs:?}\n");
}

// ===== PART 2: 真实 into_iced 路径重放(每击 statuses) =====
use auto_lang::ui::iced::renderer::IntoIcedElement;
use auto_lang::ui::view::View;

fn real_view(hit: Rc<Cell<u32>>, which: u32) -> View<u32> {
    View::col()
        .style("w-full h-full")
        .child(
            View::button("X")
                .style("w-full h-full")
                .on_click(move |_| {
                    hit.set(hit.get() + 1);
                    which
                })
                .build(),
        )
        .build()
}

fn real_case(name: &str, left_style: &str, right_style: &str) {
    let ha = Rc::new(Cell::new(0u32));
    let hb = Rc::new(Cell::new(0u32));
    let left = View::col()
        .style(left_style)
        .child(real_view(ha.clone(), 1))
        .build();
    let right = View::col()
        .style(right_style)
        .child(real_view(hb.clone(), 2))
        .build();
    let root = View::col()
        .style("relative w-[600px] h-[300px] bg-background")
        .child(left)
        .child(right)
        .build();
    let mut ui = iced_test::simulator::simulator(root.into_iced());
    let mut st_log: Vec<String> = Vec::new();
    for r in 1..=3 {
        ui.point_at(Point::new(150.0, 150.0));
        let sa = ui.simulate(iced_test::simulator::click());
        st_log.push(format!("r{r}A={sa:?}a{}", ha.get()));
        ui.point_at(Point::new(450.0, 150.0));
        let sb = ui.simulate(iced_test::simulator::click());
        st_log.push(format!("r{r}B={sb:?}b{}", hb.get()));
    }
    println!("== {name}\n   {st_log:?}\n");
}

fn real_main() {
    // R1:p022 测试原样式
    real_case(
        "R1 p022原样式(left-0/left-300)",
        "absolute top-0 left-0 w-[300px] h-[300px] bg-[#202020]",
        "absolute top-0 left-[300px] w-[300px] h-[300px] bg-[#303030]",
    );
    // R2:两层都零偏移(都走裸 opaque,同点叠原点)
    real_case(
        "R2 双零偏移(叠原点)",
        "absolute top-0 left-0 w-[300px] h-[300px] bg-[#202020]",
        "absolute top-0 left-0 w-[300px] h-[300px] bg-[#303030]",
    );
}

// ===== PART 3: 载具树形状重建(槽位+分隔条+捕获层,真实 View API) =====
// 消息值:左槽按钮=1 右槽按钮=2 分隔条按压=7 Drop=8
fn real_vehicle() {
    let btn = |label: &'static str, msg: u32| {
        View::button(label)
            .style("w-full h-full")
            .on_click(move |_| msg)
            .build()
    };
    let base = View::col().style("w-full h-full bg-background").build();
    let slot1 = View::col()
        .style("absolute z-10 top-[0px] left-[0px] w-[300px] h-[300px]")
        .child(btn("A", 1))
        .build();
    let slot2 = View::col()
        .style("absolute z-10 top-[0px] left-[300px] w-[300px] h-[300px]")
        .child(btn("B", 2))
        .build();
    let dv1 = View::col()
        .style("absolute z-20 top-[0px] left-[300px] w-[8px] h-[300px] bg-[#8899aa]")
        .child(
            View::MouseArea {
                content: Box::new(View::Empty),
                on_enter: None,
                on_exit: None,
                on_double_click: None,
                on_click: Some(7),
                on_context_menu: None,
                on_release: Some(8),
                on_move: None,
                logical_extent: None,
                style: auto_lang::ui::style::Style::parse("w-full h-full").ok(),
            },
        )
        .build();
    let catcher = View::MouseArea {
        content: Box::new(View::Empty),
        on_enter: None,
        on_exit: None,
        on_double_click: None,
        on_click: None,
        on_context_menu: None,
        on_release: Some(8),
        on_move: Some(auto_lang::ui::view::PointerMoveHandler::new(
            move |_x: f32, _y: f32| 9u32,
        )),
        logical_extent: Some((600.0, 300.0)),
        style: auto_lang::ui::style::Style::parse(
            "absolute z-30 top-0 left-0 w-[0px] h-[0px]",
        )
        .ok(),
    };
    let root = View::col()
        .style("relative w-[600px] h-[300px] bg-background")
        .child(base)
        .child(slot1)
        .child(slot2)
        .child(dv1)
        .child(catcher)
        .build();
    let mut ui = iced_test::simulator::simulator(root.into_iced());
    let mut log: Vec<String> = Vec::new();
    let pts: [(&str, f32); 3] = [("L", 150.0), ("R", 450.0), ("DIV", 302.0)];
    for round in 1..=2 {
        for (name, x) in pts {
            ui.point_at(Point::new(x, 150.0));
            let st = ui.simulate(iced_test::simulator::click());
            log.push(format!("r{round}{name}={st:?}"));
        }
    }
    let msgs: Vec<u32> = ui.into_messages().collect();
    println!("== R3 载具树形状(左300+右300+分隔条8px+0x0捕获层)\n   {log:?}\n   msgs: {msgs:?}\n   期望[1,2,7,8]*2 若全通;实测见上");
}

// ===== PART 4: 穿透钉复现(base 流子 + C 偏移浮层) =====
fn r4_case() {
    let base = View::col()
        .style("w-full h-full")
        .child(View::button("BASE").style("w-full h-full").on_click(|_| 0u32).build())
        .build();
    let c_layer = View::col()
        .style("absolute top-[100px] left-[100px] w-[100px] h-[100px] bg-[#404040]")
        .child(View::button("C").style("w-full h-full").on_click(|_| 1u32).build())
        .build();
    let root = View::col()
        .style("relative w-[600px] h-[300px] bg-background")
        .child(base)
        .child(c_layer)
        .build();
    let mut ui = iced_test::simulator::simulator(root.into_iced());
    let mut log = Vec::new();
    for (name, x, y) in [("C内(150,150)", 150.0, 150.0), ("空区(50,50)", 50.0, 50.0), ("C内(110,110)", 110.0, 110.0)] {
        ui.point_at(iced::Point::new(x, y));
        let st = ui.simulate(iced_test::simulator::click());
        log.push(format!("{name}={st:?}"));
    }
    let msgs: Vec<u32> = ui.into_messages().collect();
    println!("== R4 穿透钉(base流子+C浮层)\n   {log:?}\n   msgs: {msgs:?}\n");
}

fn r4b_bounds() {
    let base = View::col()
        .style("w-full h-full")
        .child(View::button("BASE").style("w-full h-full").on_click(|_| 0u32).build())
        .build();
    let c_layer = View::col()
        .style("absolute top-[100px] left-[100px] w-[100px] h-[100px] bg-[#404040]")
        .child(View::button("C").style("w-full h-full").on_click(|_| 1u32).build())
        .build();
    let root = View::col()
        .style("relative w-[600px] h-[300px] bg-background")
        .child(base)
        .child(c_layer)
        .build();
    let mut ui = iced_test::simulator::simulator(root.into_iced());
    for label in ["BASE", "C"] {
        match ui.find(label) {
            Ok(t) => println!("R4b {label}: bounds={:?} visible={:?}",
                t.bounds(), t.visible_bounds()),
            Err(e) => println!("R4b {label}: NOT FOUND ({e:?})"),
        }
    }
}

fn r6_root() -> Element<'static, u32> {
    use iced::Length::Shrink;
    stack![
        opaque(col![button(text("BASE").width(600).height(300)).on_press(0u32)]
            .width(600)
            .height(300)),
        col![
            space().width(0).height(100),
            iced::widget::row![
                space().width(100),
                opaque(col![button(text("C").width(100).height(100)).on_press(1u32)]
                    .width(100)
                    .height(100)),
            ],
        ]
        .width(Shrink)
        .height(Shrink),
    ]
    .into()
}
fn r6_pure_fixed_geometry() {
    let mut ui = iced_test::simulator::simulator(r6_root());
    for label in ["BASE", "C"] {
        match ui.find(label) {
            Ok(t) => println!("R6b {label}: bounds={:?}",
                t.bounds()),
            Err(e) => println!("R6b {label}: NOT FOUND ({e:?})"),
        }
    }
    for (name, x, y) in [("C内(150,150)", 150.0, 150.0), ("空区(50,50)", 50.0, 50.0)] {
        ui.point_at(iced::Point::new(x, y));
        let st = ui.simulate(iced_test::simulator::click());
        println!("R6 {name}={st:?}");
    }
    let msgs: Vec<u32> = ui.into_messages().collect();
    println!("R6 msgs: {msgs:?} (期望 [1, 0])");
}

fn r7_style_parse_dump() {
    let s = auto_lang::ui::style::Style::parse(
        "absolute top-[100px] left-[100px] w-[100px] h-[100px] bg-[#404040]",
    )
    .unwrap();
    let is = auto_lang::ui::style::iced_adapter::IcedStyle::from_style(&s);
    println!(
        "R7 top_offset={:?} left_offset={:?} width={:?} height={:?}",
        is.top_offset, is.left_offset, is.width, is.height
    );
    println!(
        "R7 classes={:?}",
        s.classes.iter().map(|c| format!("{c:?}")).collect::<Vec<_>>()
    );
}

fn r8_space_probe() -> Element<'static, u32> {
    stack![
        opaque(col![button(text("BASE").width(600).height(300)).on_press(0u32)]
            .width(600)
            .height(300)),
        col![
            space().width(0).height(100),
            button(text("C1").width(100).height(100)).on_press(1u32),
        ]
        .width(iced::Length::Shrink)
        .height(iced::Length::Shrink),
    ]
    .into()
}

fn r8_space_probe2() -> Element<'static, u32> {
    stack![
        opaque(col![button(text("BASE").width(600).height(300)).on_press(0u32)]
            .width(600)
            .height(300)),
        col![
            space().width(1).height(100),
            button(text("C2").width(100).height(100)).on_press(2u32),
        ]
        .width(iced::Length::Shrink)
        .height(iced::Length::Shrink),
    ]
    .into()
}

fn r8_run() {
    for (name, root) in [("w0", r8_space_probe()), ("w1", r8_space_probe2())] {
        let mut ui = iced_test::simulator::simulator(root);
        let target = if name == "w0" { "C1" } else { "C2" };
        match ui.find(target) {
            Ok(t) => println!("R8[{name}] {target} text bounds={:?}", t.bounds()),
            Err(e) => println!("R8[{name}] NOT FOUND {e:?}"),
        }
    }
}
