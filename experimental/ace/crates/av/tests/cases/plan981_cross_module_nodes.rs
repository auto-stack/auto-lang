//! PLAN-981（W1 真消费者缺口，2026-11-18 第六批）：**跨模块实例身份**。
//!
//! UI058 T-05 实测（`058-age-topdown2d-queries-and-sprites.md` r7 §T-05）：
//! 「场景 node 序号**每模块局部**（跨模块时各模块 node 0 撞号、`ForNode` 定向
//! 失效→双 node 须同模块；Target=0/Player=1）」。本文件把它做成**永久反例**：
//! 两个真实模块各声明一个 node，`ForNode` 按**包级组件声明序**定向，两个目标
//! 的状态必须是独立的 2:1——不靠"把两个 node 塞进同一模块"绕过。
//!
//! 同时钉住相邻的具名性：越界组件、以及"旧实例/旧世代"的定向事件不得静默
//! 落到别的组件上。

use ace_core::source::age_component::{ComponentScenario, ScenarioOp};
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::compile_age_package_with_scenario;
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};

fn module(id: &str, source: &str) -> PackageModuleInput {
    PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: id.into(),
        source: source.into(),
        provider: None,
    }
}

/// 目标组件（模块 `targets`）：`hits` 计数 + `Touch` unit 消息。
const TARGETS: &str = "node Target {\n    msg {\n        Touch\n    }\n    on {\n        .Touch -> {\n            .hits += 1\n        }\n    }\n    model {\n        hits int = 0\n    }\n    view {\n        text \"t\" {}\n    }\n}\n";

/// 玩家组件（模块 `players`）：同形但独立字段名，用来证明两模块各自有 node 0。
const PLAYERS: &str = "node Player {\n    msg {\n        Touch\n    }\n    on {\n        .Touch -> {\n            .hits += 1\n        }\n    }\n    model {\n        hits int = 0\n    }\n    view {\n        text \"p\" {}\n    }\n}\n";

fn for_node(node: usize) -> ScenarioOp {
    ScenarioOp::ForNode {
        node,
        op: Box::new(ScenarioOp::Send {
            instance: 0,
            msg: "Touch".into(),
        }),
    }
}

/// **跨模块 2:1**：`Target`（模块 targets）与 `Player`（模块 players）各自声明
/// node——包级组件声明序 = `targets` 先、`players` 后（输入序）。场景
/// `ForNode{0}×2 + ForNode{1}×1` 必须让 Target 计 2、Player 计 1。
///
/// 修复前实测：前门在**逐模块**循环里用包级全局 node 下标索引**本模块**的
/// nodes，于是 `ForNode{1}` 在"单 node 模块"里直接报
/// `the scenario targets node 1 but the package declares 1 component(s)`——
/// 跨模块定向根本进不来（这正是 UI058 把两个 node 塞进同一模块的原因）。
#[test]
fn p981_cross_module_dual_node_two_to_one_events() {
    let compiled = compile_age_package_with_scenario(
        &[module("main", TARGETS), module("players", PLAYERS)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
        &ComponentScenario {
            instances: 1,
            ops: vec![for_node(0), for_node(0), for_node(1)],
        },
    )
    .unwrap_or_else(|ds| {
        panic!(
            "cross-module ForNode must compile: {}",
            ds.iter()
                .map(|d| format!("{}: {}", d.code, d.message))
                .collect::<Vec<_>>()
                .join("; ")
        )
    });
    assert_eq!(
        compiled.description.components.len(),
        2,
        "both modules' nodes are components of the package"
    );
    let names: Vec<&str> = compiled
        .description
        .components
        .iter()
        .map(|c| c.name.as_str())
        .collect();
    assert_eq!(
        names,
        vec!["Target", "Player"],
        "package-global component order is the input (module) order"
    );
    let observed = ace_av::age_package::observe_age_package(&compiled, 1)
        .expect("the shared AV chain runs the cross-module events");
    assert_eq!(
        observed.components[0].instance_states,
        vec![vec![2]],
        "Target (package node 0) receives its two events"
    );
    assert_eq!(
        observed.components[1].instance_states,
        vec![vec![1]],
        "Player (package node 1) receives exactly one — no cross-module collision"
    );
}

/// 组件定义在**后**的模块里也能被定向：输入序调换后组件序随之调换，
/// `ForNode{1}` 指向 `Target`——证明定向是"包级声明序"，不是"模块内序"。
#[test]
fn p981_cross_module_for_node_follows_package_order_not_module_order() {
    let compiled = compile_age_package_with_scenario(
        &[module("players", PLAYERS), module("main", TARGETS)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
        &ComponentScenario {
            instances: 1,
            ops: vec![for_node(1), for_node(1), for_node(0)],
        },
    )
    .expect("swapped module order compiles");
    let names: Vec<&str> = compiled
        .description
        .components
        .iter()
        .map(|c| c.name.as_str())
        .collect();
    assert_eq!(names, vec!["Player", "Target"]);
    let observed = ace_av::age_package::observe_age_package(&compiled, 1)
        .expect("the shared AV chain runs the swapped events");
    assert_eq!(
        observed.components[0].instance_states,
        vec![vec![1]],
        "Player (package node 0) gets the single directed event"
    );
    assert_eq!(
        observed.components[1].instance_states,
        vec![vec![2]],
        "Target (package node 1) gets its two"
    );
}

/// 越界组件仍是**包级**具名拒：`ForNode{2}` 在双模块包里报"package declares
/// 2 component(s)"（不是某个模块的局部数字）。
#[test]
fn p981_cross_module_out_of_range_for_node_names_the_package_count() {
    let err = compile_age_package_with_scenario(
        &[module("main", TARGETS), module("players", PLAYERS)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
        &ComponentScenario {
            instances: 1,
            ops: vec![for_node(2)],
        },
    )
    .expect_err("out-of-range component is refused");
    let codes: Vec<&str> = err.iter().map(|d| d.code.as_str()).collect();
    assert!(
        codes.contains(&"source.age.component-scenario"),
        "named scenario domain error: {codes:?}"
    );
    assert!(
        err.iter()
            .any(|d| d.message.contains("declares 2 component(s)")),
        "the count is the package's, not a module's: {:?}",
        err.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

/// 跨模块下**未包裹**的 `Send` 仍重放到两个组件（旧语义不回退）——定向失效
/// 时不会把事件悄悄只送给第一个组件。
#[test]
fn p981_cross_module_unscoped_send_still_replays_every_component() {
    let compiled = compile_age_package_with_scenario(
        &[module("main", TARGETS), module("players", PLAYERS)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
        &ComponentScenario {
            instances: 1,
            ops: vec![ScenarioOp::Send {
                instance: 0,
                msg: "Touch".into(),
            }],
        },
    )
    .expect("unscoped cross-module send compiles");
    let observed = ace_av::age_package::observe_age_package(&compiled, 1)
        .expect("the shared AV chain runs the unscoped events");
    assert_eq!(observed.components[0].instance_states, vec![vec![1]]);
    assert_eq!(observed.components[1].instance_states, vec![vec![1]]);
}
