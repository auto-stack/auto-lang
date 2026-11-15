//! PLAN-981（IF-981-REF / IF-981-FIELDS）：**同模块多 node 不得互相覆盖**。
//!
//! 这是一个真实的数据丢失缺陷，与场景出口无关，是在把 `view.scene` 加进构造表时
//! 顺带暴露的：
//!
//! `resolve_refs` 原先对每个 node 做 `out.insert(module, resolved)`。同一模块的
//! 第二个 node 于是用**它自己的**（通常为空的）引用表顶掉第一个 node 的结果：
//!
//! ```text
//! Widget + Counter（Counter 的 view 里有 `Widget w1 {}`）  -> refs = ["Widget:w1"]
//! 同样的源 + 任意第三个 node（哪怕毫无引用）               -> refs = []
//! ```
//!
//! 也就是说：**多声明一个 node 就会丢掉整个模块的实例引用**，且没有任何诊断。
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::compile_age_package;
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

fn refs_of(src: &str) -> Vec<String> {
    let out = compile_age_package(
        &[input("main", src)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
    )
    .expect("the fixture compiles");
    let mut names: Vec<String> = out
        .description
        .components
        .iter()
        .flat_map(|c| c.refs.iter().map(|r| format!("{}:{}", r.node, r.name)))
        .collect();
    names.sort();
    names.dedup();
    names
}

const BASE: &str = "node Widget {\n    view { text \"w\" {} }\n}\nnode Counter {\n    view {\n        text \"c\" {}\n        Widget w1 {}\n    }\n}\n";

/// 基线：两个 node，引用被解析。
#[test]
fn p981_refs_survive_two_nodes_in_one_module() {
    assert_eq!(refs_of(BASE), vec!["Widget:w1".to_string()]);
}

/// 再加**一个普通 node**（无任何引用）之后，原引用**不得**消失。
#[test]
fn p981_refs_survive_a_third_node_in_the_same_module() {
    let with_extra = format!("{BASE}node Extra {{\n    view {{ text \"e\" {{}} }}\n}}\n");
    assert_eq!(
        refs_of(&with_extra),
        vec!["Widget:w1".to_string()],
        "declaring one more node must not erase the module's references"
    );
}

/// 再加一个**场景** node 之后同样不得消失（这正是当初暴露缺陷的形态）。
#[test]
fn p981_refs_survive_a_scene_node_in_the_same_module() {
    let with_scene = format!(
        "{BASE}node Scene {{\n    view-2d {{\n        shape2d b {{\n            kind: circle\n        }}\n    }}\n}}\n"
    );
    assert_eq!(
        refs_of(&with_scene),
        vec!["Widget:w1".to_string()],
        "a scene exit must not erase instance references either"
    );
}

/// 两个独立引用都应保留（不是"只留第一个"）。
#[test]
fn p981_multiple_refs_across_nodes_are_all_kept() {
    let src = "node Widget {\n    view { text \"w\" {} }\n}\nnode A {\n    view {\n        text \"a\" {}\n        Widget x1 {}\n    }\n}\nnode B {\n    view {\n        text \"b\" {}\n        Widget y1 {}\n    }\n}\n";
    assert_eq!(
        refs_of(src),
        vec!["Widget:x1".to_string(), "Widget:y1".to_string()]
    );
}

/// **每个组件只拿到自己的引用**——不是整个模块的。
///
/// 用模块聚合时，同一模块的每个组件都收到整份列表：实测 `A` 引用 `Widget x1`、
/// `B` 引用 `Widget y1` 时，`A`、`B`、以及毫无引用的 `Unrelated` 三者拿到的都是
/// `["Widget:x1","Widget:y1"]`；连被引用的 `Widget` 自己都拿到这两条（等于"自己
/// 引用自己"）。那是 979 V-10 实例身份的实质错误。
#[test]
fn p981_each_component_gets_only_its_own_refs() {
    let src = "node Widget {\n    view { text \"w\" {} }\n}\n\
node A {\n    view {\n        text \"a\" {}\n        Widget x1 {}\n    }\n}\n\
node B {\n    view {\n        text \"b\" {}\n        Widget y1 {}\n    }\n}\n\
node Unrelated {\n    view { text \"u\" {} }\n}\n";
    let out = compile_age_package(
        &[input("main", src)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
    )
    .expect("compiles");
    let refs_for = |name: &str| -> Vec<String> {
        out.description
            .components
            .iter()
            .find(|c| c.name == name)
            .unwrap_or_else(|| panic!("component {name}"))
            .refs
            .iter()
            .map(|r| format!("{}:{}", r.node, r.name))
            .collect()
    };
    assert_eq!(refs_for("A"), vec!["Widget:x1".to_string()], "A's own only");
    assert_eq!(refs_for("B"), vec!["Widget:y1".to_string()], "B's own only");
    assert!(
        refs_for("Unrelated").is_empty(),
        "a component with no reference rows must get none: {:?}",
        refs_for("Unrelated")
    );
    assert!(
        refs_for("Widget").is_empty(),
        "the referenced node must not receive the referring rows: {:?}",
        refs_for("Widget")
    );
}
