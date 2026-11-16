//! PLAN-981：**文本叶必须暴露它依赖的名字**。
//!
//! 实测发现的缺口：`text label(c)` 的描述里只有隐藏函数名与 def id，**实参 `c`
//! 完全丢了**。979 要核"文本是否由 model 字段驱动"时，拿不到这层关系，只能自己
//! 去反查隐藏函数的 HIR 体——那正是"把工作推给消费者"。
//!
//! `AgePackageText.references` 现给出该文本表达式引用到的名字（按出现序、去重）。
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::compile_age_package;
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};

fn compile(src: &str) -> ace_core::source::age_package::AgePackageCompilation {
    compile_age_package(
        &[PackageModuleInput {
            package: "app".into(),
            version: "1.0.0".into(),
            id: "main".into(),
            source: src.into(),
            provider: None,
        }],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
    )
    .expect("compiles")
}

/// `text label(c)` → 依赖里有 `c`（model 字段），979 据此可核"文本由 model 驱动"。
#[test]
fn p981_text_leaf_exposes_the_names_it_depends_on() {
    let out = compile(
        "fn label(n int) str {\n    return \"x\"\n}\n\
node M {\n    model { c int = 1 }\n    view {\n        text label(c) {}\n    }\n}\n",
    );
    let m = out
        .description
        .components
        .iter()
        .find(|c| c.name == "M")
        .expect("M");
    let t = &m.texts[0];
    assert!(
        t.references.contains(&"c".to_string()),
        "the model field `c` must appear in the text's references: {:?}",
        t.references
    );
    assert!(
        m.fields.iter().any(|(n, _)| n == "c"),
        "and `c` really is a model field"
    );
}

/// 纯字面量文本不依赖任何名字。
#[test]
fn p981_literal_text_has_no_references() {
    let out = compile("node M {\n    view {\n        text \"hello\" {}\n    }\n}\n");
    let t = &out.description.components[0].texts[0];
    assert!(
        t.references.is_empty(),
        "a literal depends on nothing: {:?}",
        t.references
    );
}

/// 多个名字按出现序、去重。
#[test]
fn p981_text_references_are_ordered_and_deduped() {
    let out = compile(
        "fn pair(a int, b int) str {\n    return \"p\"\n}\n\
node M {\n    model { a int = 1\n b int = 2 }\n    view {\n        text pair(a, b) {}\n    }\n}\n",
    );
    let t = &out.description.components[0].texts[0];
    assert_eq!(t.references, vec!["pair", "a", "b"]);
}
