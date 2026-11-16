//! PLAN-981（IF-981-ORIGINAL）：**CPU 值面**——979 的消费入口。
//!
//! 979 V-DETAIL-01 要"CPU 读取入口"。只给 `Expr`（AST）等于把形状判断推给每个
//! 消费者：它得自己判"是不是 `TypedLit{Float}`、要不要处理一元负号、元组怎么拆"。
//! `AgeSceneRow.cpu` 给出**已解码**的值，原始 `attrs` 仍在旁边。
//!
//! 关键是 **`Unresolved` 不猜**：参数引用（`position: (x, y)`）、方法链、计算值
//! 都返回 `Unresolved`，消费者必须回看 `attrs`——不给假默认值。
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::{compile_age_package, AgeSceneValue};
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};

fn input(id: &str, source: &str) -> PackageModuleInput {
    PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: id.into(),
        source: source.into(),
        provider: None,
    }
}

fn compile(src: &str) -> ace_core::source::age_package::AgePackageCompilation {
    compile_age_package(
        &[input("main", src)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
    )
    .expect("compiles")
}

/// 标量、元组、bool、符号各自解码成对应的值变体。
#[test]
fn p981_cpu_surface_decodes_each_value_kind() {
    let out = compile(
        "node M {\n    view-2d {\n        shape2d q {\n            kind: circle\n            position: (1.5, -2.5)\n            x: -20.0\n            visible: false\n        }\n    }\n}\n",
    );
    let row = &out.description.components[0].scene[0];
    let get = |k: &str| {
        row.cpu
            .iter()
            .find(|(n, _)| n == k)
            .unwrap_or_else(|| panic!("{k}"))
            .1
            .clone()
    };
    assert_eq!(get("kind"), AgeSceneValue::Symbol("circle".into()));
    assert_eq!(get("position"), AgeSceneValue::Floats(vec![1.5, -2.5]));
    assert_eq!(get("x"), AgeSceneValue::Float(-20.0));
    assert_eq!(get("visible"), AgeSceneValue::Bool(false));
}

/// 资源引用解码为**末段符号**（不是整条路径）。
#[test]
fn p981_cpu_surface_decodes_a_qualified_resource_to_its_symbol() {
    let out = compile(
        "let red = material2d.color(0.9, 0.3, 0.2, 0.5)\n\
node M {\n    view-2d {\n        shape2d q {\n            kind: circle\n            material: resources.red\n        }\n    }\n}\n",
    );
    let row = &out.description.components[0].scene[0];
    let mat = row
        .cpu
        .iter()
        .find(|(k, _)| k == "material")
        .expect("material")
        .1
        .clone();
    assert_eq!(mat, AgeSceneValue::Symbol("red".into()));
}

/// 参数引用**不猜**：`position: (x, y)` 里 `x`/`y` 是 node 参数，CPU 面给
/// `Unresolved`，而不是填 0 或某默认值。
#[test]
fn p981_cpu_surface_leaves_parameter_references_unresolved() {
    let out = compile(
        "node Disc(x: float, y: float) {\n    view-2d {\n        shape2d d {\n            kind: circle\n            position: (x, y)\n        }\n    }\n}\n",
    );
    let d = out
        .description
        .components
        .iter()
        .find(|c| c.name == "Disc")
        .expect("Disc");
    let pos = d.scene[0]
        .cpu
        .iter()
        .find(|(k, _)| k == "position")
        .expect("position")
        .1
        .clone();
    assert_eq!(
        pos,
        AgeSceneValue::Unresolved,
        "a parameter reference must NOT be given a fabricated value"
    );
    // 但原始表达式仍在，消费者可以回看。
    assert!(d.scene[0].attrs.iter().any(|(k, _)| k == "position"));
}

/// 原始 `attrs` 与 `cpu` **并存**：同一属性在两处都能找到，且顺序一致。
#[test]
fn p981_cpu_surface_and_raw_attrs_coexist() {
    let out = compile(
        "node M {\n    view-2d {\n        shape2d q {\n            kind: quad\n            x: 1.0\n        }\n    }\n}\n",
    );
    let row = &out.description.components[0].scene[0];
    let raw: Vec<&str> = row.attrs.iter().map(|(k, _)| k.as_str()).collect();
    let resolved: Vec<&str> = row.cpu.iter().map(|(k, _)| k.as_str()).collect();
    assert_eq!(raw, resolved, "both faces keep written order");
}
