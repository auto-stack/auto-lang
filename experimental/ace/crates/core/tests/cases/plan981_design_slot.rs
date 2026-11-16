//! PLAN-981（B-LANGUAGE-I / AGE-L02）：`design` 条目进入 age-package 描述面。
//!
//! 与 `msg` 同一原则：槽位恢复必须是**结构化投影**（key + 作者表达式 + span），
//! 不是"接受但看不见"。表达式求值/消费由 V-L-UI 在 980 正式面上核验。
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::{compile_age_package, AgePackageCompilation};
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};

fn compile(src: &str) -> Result<AgePackageCompilation, Vec<ace_core::atom_text::Diagnostic>> {
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
}

/// `design { theme: "dark", mode: 1 }`：key 与作者表达式都进描述面，span 指向原源。
#[test]
fn p981_design_entries_reach_the_description() {
    let src = "node M {\n    design {\n        theme: \"dark\"\n        scale: 2\n    }\n    view {\n        text \"t\" {}\n    }\n}\n";
    let out = compile(src).expect("a design block is part of the AGE author surface");
    let c = &out.description.components[0];
    let keys: Vec<&str> = c.design.iter().map(|d| d.key.as_str()).collect();
    assert_eq!(keys, vec!["theme", "scale"], "entry order is preserved");
    assert!(
        c.design.iter().all(|d| d.value.is_some()),
        "each entry keeps its author expression"
    );
    assert!(
        c.design.iter().all(|d| d.span.end > d.span.start),
        "each entry keeps its original span"
    );
    // span 落在原文范围内（不是 0）。
    assert!(c.design.iter().all(|d| (d.span.end as usize) <= src.len()));
}

/// 坏条目（缺冒号）在作者源具名拒绝，而不是静默丢。
#[test]
fn p981_malformed_design_entry_is_rejected() {
    let ds = compile(
        "node M {\n    design {\n        theme\n    }\n    view {\n        text \"t\" {}\n    }\n}\n",
    )
    .expect_err("a design entry without a value is a syntax error");
    assert_eq!(ds[0].code, "source.syntax", "{ds:?}");
}

/// `design` 与 `msg`/`model`/`on` 并存：槽位互不排斥。
#[test]
fn p981_design_coexists_with_other_slots() {
    let out = compile(
        "node M {\n    design {\n        theme: \"dark\"\n    }\n    msg {\n        Inc\n    }\n    model {\n        count int = 0\n    }\n    view {\n        text \"t\" {}\n    }\n}\n",
    )
    .expect("design + msg + model coexist");
    let c = &out.description.components[0];
    assert_eq!(c.design.len(), 1);
    assert_eq!(c.messages.len(), 1);
    assert_eq!(c.fields.len(), 1);
}
/// R60-4：design 的 `if` 形态三段（条件/Then/Else）都进唯一 verify。
#[test]
fn p981_design_if_value_is_verified() {
    let good = "node M {\n    design {\n        theme: if true { \"a\" } else { \"b\" }\n    }\n    view {\n        text \"t\" {}\n    }\n}\n";
    compile(good).expect("a valid if-valued design entry compiles");
    for src in [
        "node M {\n    design {\n        theme: if nosuch_cond { 1 } else { 2 }\n    }\n    view {\n        text \"t\" {}\n    }\n}\n",
        "node M {\n    design {\n        theme: if true { nosuch_then } else { 2 }\n    }\n    view {\n        text \"t\" {}\n    }\n}\n",
        "node M {\n    design {\n        theme: if true { 1 } else { nosuch_else }\n    }\n    view {\n        text \"t\" {}\n    }\n}\n",
    ] {
        let want = src.find("nosuch_").expect("fixture has a bad name");
        let ds = match compile(src) {
            Ok(_) => panic!("undefined if-parts must be rejected: {src}"),
            Err(ds) => ds,
        };
        assert_eq!(ds[0].code, "source.undefined", "{ds:?}");
        assert_eq!(ds[0].span.start, want, "diagnostic at the name");
    }
}
