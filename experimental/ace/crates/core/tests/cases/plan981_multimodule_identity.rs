//! PLAN-981（SOURCE 批，轮 47）：**多模块下的名字不唯一**。
//!
//! 继续"作者的哪一句输入没进来"的机械审计，这轮对准两张"按名字"的面：
//!
//! 1. `AgePackageCompilation.symbols` 的键是**源名**。两个模块各有一个
//!    `pub fn seed()` 时，映射只剩一条（键冲突被静默覆盖）：实测
//!    `{"seed": "p3_appm6_helper_seed"}`——`other::seed` 的符号根本没进来。
//!    多模块消费者（979 的跨模块 helper 用例）会拿到另一个模块的函数。
//! 2. 场景行只有 `kind` 拼写。`Disc d1 { … }` 里 `Disc` 声明在哪个模块，描述面
//!    不说；同包两个模块各有一个 `Disc` 时消费者只能按名字字符串匹配——正是
//!    §5 禁止的做法。元素检查当时也只核"这个名字在导入集里吗"，于是从一个
//!    模块导入的同名**普通函数**也能让场景行过关（它根本不是 node）。
//!
//! 修复：`symbols_by_module`（`(模块全键, 源名) → 符号`，权威面）与
//! `AgeSceneRow.element`（已解析到声明模块的 node / 冻结叶）。
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::{compile_age_package, AgePackageCompilation, AgeSceneElement};
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};

const DISC: &str = "node Disc(x: float) {\n    view-2d {\n        shape2d disc {\n            kind: circle\n            position: (x, 0.0)\n        }\n    }\n}\n";

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

fn component<'a>(
    out: &'a AgePackageCompilation,
    name: &str,
) -> &'a ace_core::source::age_package::AgePackageComponent {
    out.description
        .components
        .iter()
        .find(|c| c.name == name)
        .unwrap_or_else(|| panic!("component {name}"))
}

/// 两个模块的同名 helper 都必须留在符号面上，且各自的符号不同。
#[test]
fn p981_symbols_keep_every_module() {
    let out = compile(vec![
        input("helper", "pub fn seed() int {\n    return 0\n}\n"),
        input("other", "pub fn seed() int {\n    return 10\n}\n"),
        input(
            "main",
            "use helper.seed\nnode M {\n    model { n int = seed() }\n    view { text \"t\" {} }\n}\n",
        ),
    ])
    .expect("compiles");
    let helper = out
        .symbol_of_in("app::helper", "seed")
        .expect("helper::seed is on the authoritative face");
    let other = out
        .symbol_of_in("app::other", "seed")
        .expect("other::seed is on the authoritative face");
    assert_ne!(
        helper, other,
        "two modules' helpers must not collapse into one symbol"
    );
    assert!(helper.contains("helper"), "{helper}");
    assert!(other.contains("other"), "{other}");
    // 旧的按源名映射**仍保留**（兼容），但它在多模块下是有损的——这正是新面存在的理由：
    // 两个模块的 seed 符号里，只有**一个**能从按源名的映射里找到。
    let reachable = [helper.as_str(), other.as_str()]
        .iter()
        .filter(|sym| out.symbols.values().any(|v| v.as_str() == **sym))
        .count();
    assert_eq!(
        reachable, 1,
        "the legacy source-name face loses one module: {:?}",
        out.symbols
    );
}

/// 场景元素解析到**导入的那个模块**，而不是按名字字符串匹配第一个声明。
#[test]
fn p981_scene_element_resolves_to_the_imported_module() {
    let out = compile(vec![
        input("shapes", DISC),
        input("other", DISC),
        input(
            "main",
            "use shapes: Disc\nnode Main {\n    view-2d {\n        Disc d1 {\n            x: -20.0\n        }\n    }\n}\n",
        ),
    ])
    .expect("compiles");
    // 两个模块各有一份 Disc：描述面上确实存在歧义。
    let discs: Vec<&str> = out
        .description
        .components
        .iter()
        .filter(|c| c.name == "Disc")
        .map(|c| c.module.as_str())
        .collect();
    assert_eq!(discs, vec!["app::shapes", "app::other"]);
    let main = component(&out, "Main");
    assert_eq!(main.scene[0].kind, "Disc");
    assert_eq!(
        main.scene[0].element,
        AgeSceneElement::Node {
            module: "app::shapes".into(),
            node: "Disc".into(),
        },
        "the imported module is the resolved target"
    );
}

/// 同模块声明的 node 解析到本模块。
#[test]
fn p981_scene_element_resolves_to_its_own_module() {
    let out = compile(vec![input(
        "main",
        "node Inner {\n    view { text \"i\" {} }\n}\nnode M {\n    view-2d {\n        Inner x1 {\n            x: 1.0\n        }\n    }\n}\n",
    )])
    .expect("compiles");
    assert_eq!(
        component(&out, "M").scene[0].element,
        AgeSceneElement::Node {
            module: "app::main".into(),
            node: "Inner".into(),
        }
    );
}

/// 冻结叶保留自己的类别（不是 node 引用）。
#[test]
fn p981_frozen_scene_elements_keep_their_kind() {
    let out = compile(vec![input(
        "main",
        "node D {\n    view-2d {\n        shape2d q {\n            kind: quad\n            position: (0.0, 0.0)\n        }\n    }\n}\nnode T {\n    view-3d {\n        cylinder3d tower {\n            position: (1.0, 2.0, 3.0)\n        }\n    }\n}\n",
    )])
    .expect("compiles");
    assert_eq!(
        component(&out, "D").scene[0].element,
        AgeSceneElement::Shape2d
    );
    assert_eq!(
        component(&out, "T").scene[0].element,
        AgeSceneElement::Element3d
    );
}

/// 从一个模块导入的**同名普通函数**不是场景元素——它没有 node 声明。
///
/// 旧判据只看"这个名字在导入集里"，于是 `use libx: Disc`（libx 里是 `fn Disc`）
/// 也能让 `Disc d1 { … }` 过关，产出一条指向空处的场景行。
#[test]
fn p981_imported_function_is_not_a_scene_element() {
    let ds = compile(vec![
        input("libx", "pub fn Disc() int {\n    return 1\n}\n"),
        input(
            "main",
            "use libx: Disc\nnode Main {\n    view-2d {\n        Disc d1 {\n            x: -20.0\n        }\n    }\n}\n",
        ),
    ])
    .expect_err("a function is not a node");
    assert_eq!(ds[0].code, "source.age.scene-element", "{ds:?}");
    assert!(ds[0].message.contains("Disc"), "{}", ds[0].message);
}

/// `pub use` 重导出链也要解析到最终的声明模块（不是中间模块）。
#[test]
fn p981_reexported_node_resolves_through_the_chain() {
    let out = compile(vec![
        input("shapes", DISC),
        input("mid", "pub use shapes.Disc\n"),
        input(
            "main",
            "use mid: Disc\nnode Main {\n    view-2d {\n        Disc d1 {\n            x: -20.0\n        }\n    }\n}\n",
        ),
    ])
    .expect("compiles");
    assert_eq!(
        component(&out, "Main").scene[0].element,
        AgeSceneElement::Node {
            module: "app::shapes".into(),
            node: "Disc".into(),
        },
        "the re-export chain ends at the declaring module"
    );
}

/// g01 原源：`Disc red_disc` 解析到 shapes 模块，`shape2d` 行是冻结叶。
#[test]
fn p981_g01_scene_elements_are_resolved() {
    let out = compile(vec![
        input("resources", include_str!("../fixtures_g01/resources.at")),
        input("shapes", include_str!("../fixtures_g01/shapes.at")),
        input("main", include_str!("../fixtures_g01/main.at")),
    ])
    .expect("g01 compiles");
    let main = component(&out, "Main");
    assert_eq!(main.scene.len(), 4);
    assert_eq!(
        main.scene[0].element,
        AgeSceneElement::Node {
            module: "app::shapes".into(),
            node: "Disc".into(),
        }
    );
    assert_eq!(main.scene[1].element, AgeSceneElement::Shape2d);
    assert_eq!(
        main.scene[2].element,
        AgeSceneElement::Node {
            module: "app::shapes".into(),
            node: "Disc".into(),
        }
    );
    assert_eq!(main.scene[3].element, AgeSceneElement::Shape2d);
}
