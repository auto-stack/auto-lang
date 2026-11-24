//! PLAN-118（用户裁定①）：`xs.join(sep)` 的执行真值回归。
//!
//! 088 ColorPicker / 031-paint 真语料形：文本列表连接（快照 `join("|")`）。
//! 实现为 resolve 级脱糖（无新 opcode）：`out = ""`，`i < len` 循环里先补
//! 分隔符再拼元素（owned 元素走非破坏元素视图），返回 owned 文本。
use ace_core::source::age_component::{ComponentScenario, ScenarioOp};
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::compile_age_package_with_scenario;
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};

fn caps() -> Vec<String> {
    vec![PACKAGES_CAPABILITY.to_string()]
}

fn input(package: &str, id: &str, source: &str) -> PackageModuleInput {
    PackageModuleInput {
        package: package.into(),
        version: "1.0.0".into(),
        id: id.into(),
        source: source.into(),
        provider: None,
    }
}

#[test]
fn plan118_join_renders_separated_text_and_empty_for_empty_list() {
    use ace_av::age_package::{evaluate_widget_view, ExecLimits, WidgetViewItem};

    let root = r#"
widget Root {
    use { package: app from "../app" }
    view { App {} }
}
"#;
    let app = r#"
widget App {
    msg { Load }
    model {
        var joined Str = ""
        var empty Str = ""
        var single Str = ""
        var px = []
        var picked Str = ""
    }
    on {
        .Load -> {
            var xs List = []
            xs.push("a")
            xs.push("b")
            xs.push("c")
            .joined = xs.join("-")
            var none List = []
            .empty = none.join(",")
            var one List = []
            one.push("only")
            .single = one.join("/")
            .px = xs
            .picked = .px[1]
        }
    }
    view {
        text .joined {}
        text .empty {}
        text .single {}
        text .picked {}
    }
}
"#;
    let compiled = compile_age_package_with_scenario(
        &[
            input("root", "root", root),
            input("app", "app", app),
        ],
        "root::root",
        LegacyWidgetAlias::Accept,
        &caps(),
        &ComponentScenario {
            instances: 1,
            ops: vec![ScenarioOp::Send {
                instance: 0,
                msg: "Load".into(),
            }],
        },
    )
    .expect("join desugar compiles through the package face");
    let site = compiled.first_widget_ref().expect("authored App site");
    let view = evaluate_widget_view(&compiled, site, 0, &ExecLimits::default())
        .expect("execute the authored App view");
    assert_eq!(
        view,
        [
            WidgetViewItem::Text("a-b-c".into()),
            WidgetViewItem::Text(String::new()),
            WidgetViewItem::Text("only".into()),
            WidgetViewItem::Text("b".into()),
        ]
    );
}
#[test]
fn plan118_entry_widget_view_is_emitted_for_the_live_root() {
    // T-03 真窗入口：live 根组件的视图必须恒发（无子站点/定时器/宿主绑定
    // 也要有），否则宿主报 "entry component has no checked view"。
    use ace_av::age_package::{evaluate_package_view, ExecLimits, WidgetViewItem};

    let source = r#"
widget App {
    msg { Load }
    model { var label Str = "ready" }
    on { .Load -> { .label = "loaded" } }
    view { text .label {} }
}
"#;
    let compiled = compile_age_package_with_scenario(
        &[input("app", "app", source)],
        "app::app",
        LegacyWidgetAlias::Accept,
        &caps(),
        &ComponentScenario {
            instances: 1,
            ops: vec![ScenarioOp::Send { instance: 0, msg: "Load".into() }],
        },
    )
    .expect("live root package compiles");
    let view = evaluate_package_view(&compiled, 0, &ExecLimits::default())
        .expect("the live root view executes without any child site");
    assert_eq!(view, [WidgetViewItem::Text("loaded".into())]);
}