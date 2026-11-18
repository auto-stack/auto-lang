//! PLAN-981（SOURCE 批）：**场景元素的种类必须真实存在**。
//!
//! 独立复审把这条列为 blocker（B3）：投影层只看得到名字的拼写，于是用「首字母
//! 大写即当组件实例」放行——拼错的 `Discc`（`Disc` 的 typo）与根本不存在的
//! `NotANode` 都被**静默接受**，产出一条绑定到空处的场景行。
//!
//! 现在用 binder 的既定事实裁定：元素要么是冻结的 3D 元素，要么是 `shape2d`，
//! 要么是某模块里**真实声明**的 AGE node。跨模块引用沿用同一「符号表导入即可见」
//! 规则（不需要 `pub`），不新建第二套可见性。
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::compile_age_package;
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};

const DEP: &str = "node Disc {\n    view { text \"d\" {} }\n}\n";

fn input(id: &str, source: &str) -> PackageModuleInput {
    PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: id.into(),
        source: source.into(),
        provider: None,
    }
}

fn compile_with(
    dep: &str,
    main: &str,
) -> Result<
    ace_core::source::age_package::AgePackageCompilation,
    Vec<ace_core::atom_text::Diagnostic>,
> {
    compile_age_package(
        &[input("libx", dep), input("main", main)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
    )
}

/// 拼错的组件名必须被具名拒绝（不是静默接受）。
#[test]
fn p981_scene_element_typo_is_named() {
    let ds = compile_with(
        DEP,
        "use libx.Disc\nnode M {\n    view-2d {\n        Discc d1 {\n            x: 1.0\n        }\n    }\n}\n",
    )
    .expect_err("a misspelled component kind is rejected");
    assert_eq!(ds[0].code, "source.age.scene-element", "{ds:?}");
    assert!(ds[0].message.contains("Discc"), "{}", ds[0].message);
}

/// 完全没有声明的名字同样被拒（不是"首字母大写即可"）。
#[test]
fn p981_scene_element_unknown_is_named() {
    let ds = compile_with(
        DEP,
        "node M {\n    view-2d {\n        NotANode thing {\n            x: 1.0\n        }\n    }\n}\n",
    )
    .expect_err("an undeclared component kind is rejected");
    assert_eq!(ds[0].code, "source.age.scene-element", "{ds:?}");
}

/// 真实存在的跨模块 node 仍然是合法元素（修复不能过度收紧）。
#[test]
fn p981_scene_element_imported_node_is_accepted() {
    let out = compile_with(
        DEP,
        "use libx.Disc\nnode M {\n    view-2d {\n        Disc d1 {\n            x: 1.0\n        }\n    }\n}\n",
    )
    .expect("an imported node is a legal scene element");
    let m = out
        .description
        .components
        .iter()
        .find(|c| c.name == "M")
        .expect("M");
    assert_eq!(m.scene.len(), 1);
    assert_eq!(m.scene[0].kind, "Disc");
}

/// 同模块声明的 node 也是合法元素。
#[test]
fn p981_scene_element_same_module_node_is_accepted() {
    let out = compile_with(
        DEP,
        "node Inner {\n    view { text \"i\" {} }\n}\nnode M {\n    view-2d {\n        Inner x1 {\n            x: 1.0\n        }\n    }\n}\n",
    )
    .expect("a same-module node is a legal scene element");
    let m = out
        .description
        .components
        .iter()
        .find(|c| c.name == "M")
        .expect("M");
    assert_eq!(m.scene[0].kind, "Inner");
}

/// **未被导入**的跨模块 node 不能当场景元素。
///
/// 复审指出：此前的实现把**全项目** node 名并成一个集合，于是"某个与入口毫无
/// 导入关系的模块里恰好有个同名 node"也会被接受——比同族的 `resolve_refs`
/// （查导入）松。拼写撞上无关模块的名字就能过关，是真实的静默接受。
#[test]
fn p981_unimported_node_is_not_a_scene_element() {
    let ds = compile_with(
        "node Far {\n    view { text \"f\" {} }\n}\n",
        "node M {\n    view-2d {\n        Far f1 {\n            x: 1.0\n        }\n    }\n}\n",
    )
    .expect_err("an unimported node is not usable as a scene element");
    assert_eq!(ds[0].code, "source.age.scene-element", "{ds:?}");
    assert!(
        ds[0]
            .message
            .contains("not a node declared in or imported by"),
        "{}",
        ds[0].message
    );
}

/// 同一形状但**写了 `use`** → 通过。与上一条构成同型合法/非法对照。
#[test]
fn p981_imported_node_is_a_scene_element() {
    let out = compile_with(
        "node Far {\n    view { text \"f\" {} }\n}\n",
        "use libx.Far\nnode M {\n    view-2d {\n        Far f1 {\n            x: 1.0\n        }\n    }\n}\n",
    )
    .expect("an imported node is usable as a scene element");
    let m = out
        .description
        .components
        .iter()
        .find(|c| c.name == "M")
        .expect("M");
    assert_eq!(m.scene[0].kind, "Far");
}

/// 冻结的 3D 元素与 `shape2d` 不受该收紧影响。
#[test]
fn p981_frozen_elements_are_still_legal() {
    let out = compile_with(
        DEP,
        "node T {\n    view-3d {\n        cylinder3d c {\n            position: (0.0, 0.0, 0.0)\n        }\n    }\n}\n",
    )
    .expect("cylinder3d is legal");
    let t = out
        .description
        .components
        .iter()
        .find(|c| c.name == "T")
        .expect("T");
    assert_eq!(t.scene[0].kind, "cylinder3d");

    let out2 = compile_with(
        DEP,
        "node D {\n    view-2d {\n        shape2d q {\n            kind: quad\n        }\n    }\n}\n",
    )
    .expect("shape2d is legal");
    let d = out2
        .description
        .components
        .iter()
        .find(|c| c.name == "D")
        .expect("D");
    assert_eq!(d.scene[0].kind, "shape2d");
}
