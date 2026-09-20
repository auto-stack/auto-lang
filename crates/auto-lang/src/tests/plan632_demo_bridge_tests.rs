//! PLAN-632: 画廊内嵌 demo 语境的模块组件桥接回归。
//!
//! 双缺陷最小复现（T-01 判定，2026-09-15）：
//! - **F1/016 形**：demo 源经 `use.web component` 适配器链装载时，其
//!   `use <mod>: Store` 引入的 StoreDecl 在 `load_ext_imports_for_vm`
//!   才进 import_stmts，晚于 store→child 转换（lib.rs 原 порядка）——
//!   store model 不并入统一根态、store handler 不编译 → `.store.*`
//!   读落空（f-string 原样输出）。
//! - **F2+F3/006 形**：`use <dep>: Component` 的 dep 目录只含 item
//!   命名文件（`settings_popover.at`）时解析失败；即便解析成功，
//!   ext 适配器 use 链上的 widget 也从不进 registry/child_decls。
//!
//! 结构对齐 ui-gallery 实测布局：host(内嵌宿主) → demo(.at 适配器) →
//! 自有模块/dep 组件。语料最小化但链路同构。

#![cfg(feature = "ui-iced")]

use std::path::PathBuf;

/// 内嵌三文件布局：host.at / demo.at /（按需）模块文件。返回 host 路径。
struct HostFixture {
    _dir: tempfile::TempDir,
    host_path: PathBuf,
}

fn write(dir: &std::path::Path, rel: &str, src: &str) {
    let p = dir.join(rel);
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(p, src).unwrap();
}

fn build_embedded(files: &[(&str, &str)]) -> (crate::ui::dynamic::DynamicComponent, PathBuf) {
    let dir = tempfile::TempDir::new().unwrap();
    for (rel, src) in files {
        write(dir.path(), rel, src);
    }
    let host_path = dir.path().join("host.at");
    let path_str = host_path.to_string_lossy().to_string();
    let host_src = std::fs::read_to_string(&host_path).unwrap();
    let comp = crate::build_dynamic_component(&host_src, Some(&path_str))
        .expect("内嵌 demo 宿主编译");
    (comp, host_path)
}

// ── F1/016 形：store 型组件经 use.web 适配器链的状态桥接 ──

const F1_DEMO: &str = r#"
use counter_store: CounterStore

widget Demo016 {
    model {
        var tag str = "n"
    }
    view {
        col {
            text f"count=${.store.count}"
            button "inc" { onclick: .Bump }
        }
    }
    on {
        .Init -> { store.Init() }
        .Bump -> { store.Inc() }
    }
}
"#;

const F1_STORE: &str = r#"
store CounterStore {
    model {
        var count int = 7
    }
    msg {
        Init,
        Inc,
    }
    on {
        .Init -> { .count = 7 }
        .Inc -> { .count = .count + 1 }
    }
}
"#;

/// F4 锁：demo（适配器）自带 `use <mod>: fn` 的模块 fn 供 computed 消费
/// ——别名表必须覆盖适配器 use 链（016 的 month_name/build_month_grid 形）。
const F1_UTIL: &str = r#"
pub fn double_it(n int) int {
    n * 2
}
"#;

const F1_DEMO_WITH_COMPUTED: &str = r#"
use counter_store: CounterStore
use counter_util: double_it

widget Demo016 {
    model {
        var tag str = "n"
    }
    computed {
        twice => double_it(.store.count)
    }
    view {
        col {
            text f"count=${.store.count}"
            text .twice
            button "inc" { onclick: .Bump }
        }
    }
    on {
        .Init -> { store.Init() }
        .Bump -> { store.Inc() }
    }
}
"#;

const F1_HOST_COMPUTED: &str = r#"
use.web component Demo016 from "demo016.at"

widget App {
    view {
        Demo016 {}
    }
}
"#;

const F1_HOST: &str = r#"
use.web component Demo016 from "demo016.at"

widget App {
    view {
        Demo016 {}
    }
}
"#;

/// store model 并入统一根态：`.store.count` 可读、f-string 求值非原样。
#[test]
fn f1_embedded_store_state_seeds_and_interpolates() {
    let (mut comp, _keep) = build_embedded(&[
        ("host.at", F1_HOST),
        ("demo016.at", F1_DEMO),
        ("counter_store.at", F1_STORE),
    ]);
    // 触发渲染（子件 Init 挂载/派生求值走渲染帧）。
    let (view, _, _) = comp.view_with_debug();
    let rendered = format!("{view:?}");
    // store 字段直接可读（model 并根的直接证据）。
    let count = comp.read_state("count").expect("store 字段 count 应在统一根态");
    assert_eq!(count, auto_val::Value::Int(7), "store 默认值应播种: {count:?}");
    // f-string 内插求值成功（非原样 `${.store.count}`）。
    assert!(
        rendered.contains("count=7"),
        "内嵌 demo 的 .store.* f-string 应求值: {rendered:.600}"
    );
    assert!(
        !rendered.contains("${.store.count}"),
        "不允许 raw 模板残留: {rendered:.600}"
    );
}

/// store handler 派发链：demo 的 .Bump → store.Inc() → count 增。
#[test]
fn f1_embedded_store_handler_dispatch() {
    let (mut comp, _keep) = build_embedded(&[
        ("host.at", F1_HOST),
        ("demo016.at", F1_DEMO),
        ("counter_store.at", F1_STORE),
    ]);
    let _ = comp.view_with_debug();
    comp.on_with_input_for("Demo016", "Bump", None);
    let count = comp.read_state("count").expect("Bump 后 count 仍可读");
    assert_eq!(count, auto_val::Value::Int(8), "store.Inc() 应经 sibling 调用生效: {count:?}");
}

/// F4 锁：适配器 use 链的模块 fn 经别名进 computed 求值（twice = 7×2）。
#[test]
fn f4_embedded_adapter_module_fn_computed() {
    let (mut comp, _keep) = build_embedded(&[
        ("host.at", F1_HOST_COMPUTED),
        ("demo016.at", F1_DEMO_WITH_COMPUTED),
        ("counter_store.at", F1_STORE),
        ("counter_util.at", F1_UTIL),
    ]);
    let (view, _, _) = comp.view_with_debug();
    let rendered = format!("{view:?}");
    assert!(
        rendered.contains("count=7") && rendered.contains(">14<") || rendered.contains("14"),
        "适配器 computed 的模块 fn 调用应求值(twice=14): {rendered:.800}"
    );
    assert!(
        !rendered.contains("${twice}") && !rendered.contains("${month"),
        "computed 失败不得回退 raw 模板: {rendered:.800}"
    );
}

// ── F2+F3/006 形：参数型视图组件经 dep 目录 item 文件的解析与注册 ──

const F2_POPOVER: &str = r#"
widget SettingsPopover(open: bool) {
    msg {
        Close,
    }
    view {
        if .open {
            text "POPOVER-OPEN"
        }
    }
    on {
        .Close -> { }
    }
}
"#;

const F2_DEMO: &str = r#"
use settings: SettingsPopover

widget Demo006 {
    model {
        var open bool = false
    }
    view {
        col {
            SettingsPopover(open: .open)
        }
    }
    on {
        .Init -> { .open = true }
    }
}
"#;

const F2_HOST: &str = r#"
use.web component Demo006 from "demo006.at"

widget App {
    view {
        Demo006 {}
    }
}
"#;

/// dep 目录 item 命名文件（settings_popover.at）解析 + 适配器链 widget 注册，
/// 组件实例按 props 展开视图子树。
#[test]
fn f2_embedded_dep_component_expands_with_props() {
    // PLAN-668 A-03：夹具按 PLAN-635 D4/D5 声明门控契约补 pac.at——
    // deps/<name> 物化但未声明 = 幽灵依赖，解析被阻断（632 夹具早于
    // 635 契约，真实 demo（ui-gallery）均带 pac.at 声明）。
    let (mut comp, _keep) = build_embedded(&[
        ("host.at", F2_HOST),
        ("demo006.at", F2_DEMO),
        ("deps/settings/settings_popover.at", F2_POPOVER),
        ("pac.at", "dep \"settings\" { path: \"deps/settings\" }"),
    ]);
    let (view, _, _) = comp.view_with_debug();
    let rendered = format!("{view:?}");
    assert!(
        rendered.contains("POPOVER-OPEN"),
        "模块组件实例应展开视图子树（open=true 时面板内容可见）: {rendered:.800}"
    );
}

/// F3 隔离面：组件自身 handler 编译进单 VM（Close 派发不 HandlerNotFound）。
#[test]
fn f3_component_own_handler_compiles_into_vm() {
    // PLAN-668 A-03：夹具按 PLAN-635 D4/D5 声明门控契约补 pac.at——
    // deps/<name> 物化但未声明 = 幽灵依赖，解析被阻断（632 夹具早于
    // 635 契约，真实 demo（ui-gallery）均带 pac.at 声明）。
    let (mut comp, _keep) = build_embedded(&[
        ("host.at", F2_HOST),
        ("demo006.at", F2_DEMO),
        ("deps/settings/settings_popover.at", F2_POPOVER),
        ("pac.at", "dep \"settings\" { path: \"deps/settings\" }"),
    ]);
    let _ = comp.view_with_debug();
    // 不 panic / 不 Err 即为通过：Close 的 synthesized fn 必须可派发。
    comp.on_with_input_for("SettingsPopover", "Close", None);
}
