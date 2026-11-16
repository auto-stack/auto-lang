//! PLAN-981（B-LANGUAGE-I / AGE-L02）：980 成员槽位在 age-package 的**完整投影**。
//!
//! R52-1 的修法是把这些槽位挡在 age-package 之外（否决静默丢）；LANGUAGE-I 的修法
//! 是相反方向：**槽位恢复 + 结构化投影**——每个槽位都在描述面留下角色、名字与作者
//! 值/文本/数字 + 原 span，且分母（构造表）逐行登记。语义执行归 980 面 consumer。
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

fn body(slot: &str) -> String {
    format!("node M {{\n    model {{\n        count int = 0\n    }}\n    {slot}\n    view {{\n        text \"t\" {{}}\n    }}\n}}\n")
}

/// 每个槽位都被接受，并在 `members` 里留下**对应角色**的行。
#[test]
fn p981_every_widget_member_slot_is_projected() {
    let cases = [
        ("named-view", "view main {\n        text \"x\" {}\n    }"),
        ("bind", "bind {\n        \"title\" -> .Title\n    }"),
        ("route", "routes {\n        \"/a\" -> use home\n    }"),
        ("expose", "expose {\n        count\n    }"),
        ("lifecycle", "lifecycle {\n        mounted => {\n            .count = count + 1\n        }\n    }"),
        ("computed", "computed {\n        double => count + count\n    }"),
        ("watch", "watch { .count -> { .count = count + 1 } }"),
        ("timer", "timer {\n        Tick (every_ms: 1)\n    }"),
    ];
    for (role, slot) in cases {
        let src = body(slot);
        let out = compile(&src).unwrap_or_else(|ds| {
            panic!("slot `{role}` must be accepted by age-package: {ds:?}")
        });
        let c = &out.description.components[0];
        let rows: Vec<&str> = c
            .members
            .iter()
            .filter(|m| m.role == role)
            .map(|m| m.role.as_str())
            .collect();
        assert!(
            !rows.is_empty(),
            "slot `{role}` must leave a projected row, got {:?}",
            c.members.iter().map(|m| m.role.as_str()).collect::<Vec<_>>()
        );
        assert!(
            c.members.iter().any(|m| m.role == role && m.span.end > m.span.start),
            "slot `{role}` must keep its original span"
        );
    }
}

/// 多个槽位并存：互不排斥，行按声明序保留。
#[test]
fn p981_member_slots_coexist_in_order() {
    let src = "node M {\n    model {\n        count int = 0\n    }\n    bind {\n        \"title\" -> .Title\n    }\n    computed {\n        double => count + count\n    }\n    timer {\n        Tick (every_ms: 1)\n    }\n    view {\n        text \"t\" {}\n    }\n}\n";
    let out = compile(src).expect("bind + computed + timer coexist");
    let c = &out.description.components[0];
    let roles: Vec<&str> = c.members.iter().map(|m| m.role.as_str()).collect();
    assert_eq!(roles, vec!["bind", "computed", "timer"], "declaration order");
}

/// 槽位内部的坏语法仍在作者源具名拒绝，不静默丢槽位。
#[test]
fn p981_malformed_member_slot_is_rejected() {
    let ds = compile(&body("routes {\n        \"/a\" -> \n    }"))
        .expect_err("a route without a target is a syntax error");
    assert_eq!(ds[0].code, "source.syntax", "{ds:?}");
}
/// R59-1：`expose { alpha, beta }` 的**每个**名字都要成行（此前只留第一个）。
#[test]
fn p981_expose_projects_every_name() {
    let out = compile(&body("expose {\n        alpha, beta\n    }")).expect("expose compiles");
    let names: Vec<&str> = out.description.components[0]
        .members
        .iter()
        .filter(|m| m.role == "expose")
        .map(|m| m.name.as_str())
        .collect();
    assert_eq!(names, vec!["alpha", "beta"], "every expose name becomes a row");
}

/// R59-4：`watch` 行必须带源（名字 `.count` 的 `count`，声明文本 `.count`），
/// 不能是空名字的空壳行。
#[test]
fn p981_watch_row_keeps_its_source() {
    let out = compile(&body("watch { .count -> { .count = count + 1 } }"))
        .expect("watch compiles");
    let row = out.description.components[0]
        .members
        .iter()
        .find(|m| m.role == "watch")
        .expect("a watch row");
    assert_eq!(row.name, "count", "the watch source name is preserved");
    assert_eq!(row.text.as_deref(), Some("count"), "the source name is preserved as declaration text");
}

/// R59-3：成员值表达式进唯一 verify——未声明名字在**作者 span** 具名拒绝。
#[test]
fn p981_member_value_expressions_are_verified() {
    let src = "node M {\n    design {\n        theme: nosuch_ident\n    }\n    view {\n        text \"t\" {}\n    }\n}\n";
    let want = src.find("nosuch_ident").expect("fixture has the bad name");
    let ds = match compile(src) {
        Ok(_) => panic!("an undefined design value must be rejected"),
        Err(ds) => ds,
    };
    assert_eq!(ds[0].code, "source.undefined", "{ds:?}");
    assert_eq!(ds[0].span.start, want, "the diagnostic points at the name");
}
