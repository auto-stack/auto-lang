//! PLAN-981（SOURCE 批，轮 46）：**资源的构造身份、实参逐位事实与引用解析**。
//!
//! 继续用"作者的哪一句输入没进来"这把尺子，这次对准 `AgeResource` 与场景行里的
//! 资源引用。实测三处静默丢失（探针输出见计划 §9）：
//!
//! 1. `AgeResource` 只有 `port`：`mesh2d.circle(30.0)` 与 `mesh2d.quad(30.0, 40.0)`
//!    都是 `Mesh2D`，**构造身份没了**；消费者只能按实参个数猜形状，而端口声明的
//!    字段（`Mesh2D` = w,h）对 circle 的单个直径根本不成立。
//! 2. 具名实参被整体 `continue`（`material2d.color(r: 0.9, …)` → 空表）；非字面量
//!    实参被跳过且**不留位**（`material2d.color(base.r, 0.3, 0.2, 0.5)` →
//!    `[0.3, 0.2, 0.5]`，被读成 r=0.3,g=0.2,b=0.5——静默错位，比空表更糟）。
//! 3. 场景资源引用只剩末段符号（`res2.red` → `"red"`）：两个模块各有一个 `red`
//!    时消费者无从知道是哪一个；而限定写法的可见性也只按裸名导入集判，
//!    `use res1: red` 居然能让 `res2.red` 通过。
//!
//! 修复后：`op` 是声明表的稳定符号（跨后端身份）；`args` 逐位保留、认不出就是
//! `Unresolved`；`components` 只在全字面量时才完整；`resource_refs` 给出检查器
//! 已解析的目标（声明模块 / node 参数），限制写法核到来源模块。
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::{
    compile_age_package, AgePackageCompilation, AgeResourceArg, AgeResourceTarget,
};
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};

const MAIN: &str = include_str!("../fixtures_g01/main.at");
const RESOURCES: &str = include_str!("../fixtures_g01/resources.at");
const SHAPES: &str = include_str!("../fixtures_g01/shapes.at");

fn input(id: &str, source: &str) -> PackageModuleInput {
    PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: id.into(),
        source: source.into(),
        provider: None,
    }
}

fn compile(
    inputs: Vec<PackageModuleInput>,
) -> Result<AgePackageCompilation, Vec<ace_core::atom_text::Diagnostic>> {
    compile_age_package(
        &inputs,
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
    )
}

fn compile_main(src: &str) -> AgePackageCompilation {
    compile(vec![input("main", src)]).expect("compiles")
}

/// 同一端口、不同构造器：`port` 不足以区分，`op` 必须给出稳定符号。
#[test]
fn p981_resource_constructor_identity_is_not_flattened() {
    let out = compile_main(
        "let round = mesh2d.circle(30.0)\n\
         let boxy = mesh2d.quad(30.0, 40.0)\n\
         node M {\n    view { text \"t\" {} }\n}\n",
    );
    let round = out
        .description
        .resources
        .iter()
        .find(|r| r.name == "round")
        .expect("round");
    let boxy = out
        .description
        .resources
        .iter()
        .find(|r| r.name == "boxy")
        .expect("boxy");
    // 旧描述面只有下面这两条相同的事实——构造身份完全丢失。
    assert_eq!(round.port, "Mesh2D");
    assert_eq!(boxy.port, "Mesh2D", "port alone cannot tell the two apart");
    // 新的身份事实。
    assert_eq!(round.op, "age.mesh2d.circle.v1");
    assert_eq!(boxy.op, "age.mesh2d.quad.v1");
    assert_eq!(round.arity, 1);
    assert_eq!(boxy.arity, 2);
    assert_eq!(round.components, vec![30.0]);
    assert_eq!(boxy.components, vec![30.0, 40.0]);
}

/// 具名实参在共享调用检查里按**书写序**对齐；描述面也必须按书写序取值。
#[test]
fn p981_named_constructor_args_are_kept_in_written_order() {
    let out = compile_main(
        "let tinted = material2d.color(r: 0.9, g: 0.3, b: 0.2, a: 0.5)\n\
         node M {\n    view { text \"t\" {} }\n}\n",
    );
    let r = out
        .description
        .resources
        .iter()
        .find(|r| r.name == "tinted")
        .expect("tinted");
    assert_eq!(r.arity, 4);
    assert_eq!(
        r.components,
        vec![0.9, 0.3, 0.2, 0.5],
        "named args must not be dropped: {:#?}",
        r.args
    );
}

/// 非字面量实参**保留位置**：不能把第 2 个分量当成第 1 个。
#[test]
fn p981_non_literal_argument_keeps_its_slot() {
    let out = compile_main(
        "let base = material2d.color(0.9, 0.3, 0.2, 0.5)\n\
         let tinted = material2d.color(base.r, 0.3, 0.2, 0.5)\n\
         node M {\n    view { text \"t\" {} }\n}\n",
    );
    let r = out
        .description
        .resources
        .iter()
        .find(|r| r.name == "tinted")
        .expect("tinted");
    assert_eq!(r.arity, 4, "the author wrote four arguments");
    assert_eq!(
        r.args,
        vec![
            AgeResourceArg::Unresolved,
            AgeResourceArg::Float(0.3),
            AgeResourceArg::Float(0.2),
            AgeResourceArg::Float(0.5),
        ]
    );
    assert!(
        r.components.is_empty(),
        "a partial component list would be read as r=0.3,g=0.2,b=0.5 — \
         misaligned values are worse than none: {:?}",
        r.components
    );
}

/// 单个字符串实参仍是符号（`Model3D` 的 GLB 路径），且不再用"最后一个字符串"冒充。
#[test]
fn p981_single_string_argument_is_the_symbol() {
    let out = compile_main(
        "let ship = model3d.glb(\"assets/ship.glb\")\n\
         node T {\n    view { text \"t\" {} }\n}\n",
    );
    let r = out
        .description
        .resources
        .iter()
        .find(|r| r.name == "ship")
        .expect("ship");
    assert_eq!(r.op, "age.model3d.glb.v1");
    assert_eq!(r.arity, 1);
    assert_eq!(
        r.args,
        vec![AgeResourceArg::Str("assets/ship.glb".into())]
    );
    assert_eq!(r.symbol.as_deref(), Some("assets/ship.glb"));
    assert!(r.components.is_empty());
}

/// 场景资源引用必须解析到**声明模块**，而不是只剩末段名字。
#[test]
fn p981_scene_resource_reference_resolves_to_the_declaring_module() {
    let out = compile(vec![
        input("res1", "let red = material2d.color(0.1, 0.2, 0.3, 0.5)\n"),
        input("res2", "let red = material2d.color(0.9, 0.8, 0.7, 0.5)\n"),
        input(
            "main",
            "use res2: red\nnode M {\n    view-2d {\n        shape2d q {\n            kind: circle\n            material: res2.red\n        }\n    }\n}\n",
        ),
    ])
    .expect("compiles");
    let m = out
        .description
        .components
        .iter()
        .find(|c| c.name == "M")
        .expect("M");
    // CPU 值面仍然只有末段符号（既有契约，不改）。
    let cpu = m.scene[0]
        .cpu
        .iter()
        .find(|(k, _)| k == "material")
        .expect("material")
        .1
        .clone();
    assert_eq!(cpu, ace_core::source::age_package::AgeSceneValue::Symbol("red".into()));
    // 解析后的身份在 resource_refs 里——两个模块都有 `red`，这里必须指对。
    assert_eq!(m.resource_refs.len(), 1, "{:#?}", m.resource_refs);
    let r = &m.resource_refs[0];
    assert_eq!(r.row, "q");
    assert_eq!(r.attr, "material");
    assert_eq!(r.spelling, "res2.red", "the author's qualifier is kept");
    assert_eq!(
        r.target,
        AgeResourceTarget::Resource {
            module: "app::res2".into(),
            name: "red".into(),
            port: "Material2D".into(),
        },
        "the checker resolved res2, not res1"
    );
}

/// `material: tint` 指向本 node 的 typed 参数：解析结果不是某个模块资源。
#[test]
fn p981_scene_resource_reference_can_name_a_node_parameter() {
    let out = compile_main(
        "node Disc(x: float, tint: Material2D) {\n    view-2d {\n        shape2d disc {\n            kind: circle\n            position: (x, 0.0)\n            material: tint\n        }\n    }\n}\n",
    );
    let d = out
        .description
        .components
        .iter()
        .find(|c| c.name == "Disc")
        .expect("Disc");
    assert_eq!(d.resource_refs.len(), 1, "{:#?}", d.resource_refs);
    let r = &d.resource_refs[0];
    assert_eq!(r.spelling, "tint");
    assert_eq!(
        r.target,
        AgeResourceTarget::Param {
            name: "tint".into(),
            port: "Material2D".into(),
        }
    );
}

/// 限定写法必须核到**限定词所指的模块**：导入同名符号不能替代模块导入。
#[test]
fn p981_qualified_resource_requires_importing_that_module() {
    let ds = compile(vec![
        input("res1", "let red = material2d.color(0.1, 0.2, 0.3, 0.5)\n"),
        input("res2", "let red = material2d.color(0.9, 0.8, 0.7, 0.5)\n"),
        input(
            "main",
            "use res1: red\nnode M {\n    view-2d {\n        shape2d q {\n            kind: circle\n            material: res2.red\n        }\n    }\n}\n",
        ),
    ])
    .expect_err("res2 was never imported");
    assert_eq!(ds[0].code, "source.age.scene-resource-invisible", "{ds:?}");
}

/// g01 原源：`tint: resources.red_alpha` 必须解析到 `app::resources`，`Disc` 的
/// `material: tint` 必须是参数而非资源——两条都是 979 V-DETAIL-01 的直接输入。
#[test]
fn p981_g01_resource_references_have_resolved_targets() {
    let out = compile(vec![
        input("resources", RESOURCES),
        input("shapes", SHAPES),
        input("main", MAIN),
    ])
    .expect("g01 compiles");
    let main = out
        .description
        .components
        .iter()
        .find(|c| c.name == "Main")
        .expect("Main");
    assert_eq!(main.resource_refs.len(), 4, "{:#?}", main.resource_refs);
    for r in &main.resource_refs {
        let AgeResourceTarget::Resource { module, port, .. } = &r.target else {
            panic!("`{}` must resolve to a module resource: {:#?}", r.spelling, r.target);
        };
        assert_eq!(module, "app::resources", "{}: wrong module", r.spelling);
        assert_eq!(port, "Material2D");
        assert!(
            r.spelling.starts_with("resources."),
            "the qualifier is part of the author's spelling: {}",
            r.spelling
        );
    }
    let disc = out
        .description
        .components
        .iter()
        .find(|c| c.name == "Disc")
        .expect("Disc");
    assert_eq!(disc.resource_refs.len(), 1, "{:#?}", disc.resource_refs);
    assert_eq!(
        disc.resource_refs[0].target,
        AgeResourceTarget::Param {
            name: "tint".into(),
            port: "Material2D".into(),
        }
    );
    // g01 的颜色分量仍逐值可读，且构造身份是稳定符号。
    let red = out
        .description
        .resources
        .iter()
        .find(|r| r.name == "red_alpha")
        .expect("red_alpha");
    assert_eq!(red.op, "age.material2d.color.v1");
    assert_eq!(red.module, "app::resources");
    assert_eq!(red.components, vec![0.9, 0.3, 0.2, 0.5]);
}