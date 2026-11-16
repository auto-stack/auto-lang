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

/// R59-3 尾项：lifecycle/watch 体内语句的 **RHS** 进唯一 verify（未声明名字具名拒）。
#[test]
fn p981_member_statement_rhs_is_verified() {
    let src = "node M {\n    model {\n        count int = 0\n    }\n    lifecycle {\n        mounted => {\n            .count = nosuch_ident\n        }\n    }\n    view {\n        text \"t\" {}\n    }\n}\n";
    let want = src.find("nosuch_ident").expect("fixture has the bad name");
    let ds = match compile(src) {
        Ok(_) => panic!("an undefined lifecycle RHS must be rejected"),
        Err(ds) => ds,
    };
    assert_eq!(ds[0].code, "source.undefined", "{ds:?}");
    assert_eq!(ds[0].span.start, want, "the diagnostic points at the name");
}

/// R59-3 尾项：语句 LHS 必须是本 node 的 `model` 字段（写不存在的字段具名拒）。
#[test]
fn p981_member_statement_field_must_exist() {
    let src = "node M {\n    model {\n        count int = 0\n    }\n    lifecycle {\n        mounted => {\n            .nosuch = 1\n        }\n    }\n    view {\n        text \"t\" {}\n    }\n}\n";
    let ds = match compile(src) {
        Ok(_) => panic!("assigning an undeclared model field must be rejected"),
        Err(ds) => ds,
    };
    assert_eq!(ds[0].code, "source.age.member-field", "{ds:?}");
}
/// R60-1：成员值必须保留**完整表达式**（`1 + 2` 是 Binary，不是首原子 IntLit）。
#[test]
fn p981_member_value_keeps_the_full_expression() {
    use ace_core::source::ast::ExprKind;
    let out = compile(&body("computed {\n        d => 1 + 2\n    }")).expect("computed compiles");
    let row = out.description.components[0]
        .members
        .iter()
        .find(|m| m.role == "computed")
        .expect("a computed row");
    let value = row.value.as_ref().expect("the expression is projected");
    assert!(
        matches!(value.kind, ExprKind::Binary { .. }),
        "the projected value must be the whole binary expression, got {:?}",
        value.kind
    );
}

/// R60-1：成员值/语句 RHS 的**右链**也必须进唯一 verify（截断即绿是缺陷）。
#[test]
fn p981_member_value_tail_is_verified() {
    for src in [
        "node M {\n    model {\n        count int = 0\n    }\n    computed {\n        d => count + nosuch_ident\n    }\n    view {\n        text \"t\" {}\n    }\n}\n",
        "node M {\n    model {\n        count int = 0\n    }\n    lifecycle {\n        m => {\n            .count = count + nosuch_ident\n        }\n    }\n    view {\n        text \"t\" {}\n    }\n}\n",
    ] {
        let want = src.find("nosuch_ident").expect("fixture has the bad name");
        let ds = match compile(src) {
            Ok(_) => panic!("the right-hand tail must be verified: {src}"),
            Err(ds) => ds,
        };
        assert_eq!(ds[0].code, "source.undefined", "{ds:?}");
        assert_eq!(ds[0].span.start, want, "diagnostic at the name");
    }
}

/// R60-2：`view <name> { … }` 的体不能丢——文本叶投影（role 带命名面）且进 verify。
#[test]
fn p981_named_view_body_is_verified_and_attributed() {
    let good = "node M {\n    view main {\n        text \"hi\" {}\n    }\n    view {\n        text \"t\" {}\n    }\n}\n";
    let out = compile(good).expect("a named view with a valid text body compiles");
    let texts = &out.description.components[0].texts;
    assert_eq!(texts.len(), 2, "both leaves are projected");
    assert!(
        texts.iter().any(|t| t.role == "named-view:main"),
        "the named-view leaf keeps its face attribution: {texts:?}"
    );
    let bad = "node M {\n    view main {\n        text nosuch_fn() {}\n    }\n    view {\n        text \"t\" {}\n    }\n}\n";
    match compile(bad) {
        Ok(_) => panic!("an undefined name inside `view <name>` must be rejected"),
        Err(ds) => assert_eq!(ds[0].code, "source.module-symbol", "{ds:?}"),
    }
}

/// R60-3：watch 支持多行与多源声明（此前 `dot(same-line)` 让两者都 `@0`）。
#[test]
fn p981_watch_multiline_and_multisource_parse() {
    let multiline = "node M {\n    model {\n        count int = 0\n    }\n    watch {\n        .count -> {\n            .count = count + 1\n        }\n    }\n    view {\n        text \"t\" {}\n    }\n}\n";
    let out = compile(multiline).expect("a multi-line watch parses");
    let row = out.description.components[0]
        .members
        .iter()
        .find(|m| m.role == "watch")
        .expect("a watch row");
    assert_eq!(row.name, "count");
    let multi = "node M {\n    model {\n        a int = 0\n        b int = 1\n    }\n    watch {\n        .a, .b -> {\n            .a = a + 1\n        }\n    }\n    view {\n        text \"t\" {}\n    }\n}\n";
    let out = compile(multi).expect("a multi-source watch parses");
    assert!(
        out.description.components[0].members.iter().any(|m| m.role == "watch"),
        "the multi-source watch leaves a row"
    );
}

/// R61-1/R61-2：`watch` 单行体 + 多源声明可解析；**每个**源都进描述面并被检查。
#[test]
fn p981_watch_multisource_single_line_body() {
    let two_sources = "node M {\n    model {\n        alpha int = 0\n        beta int = 1\n    }\n    watch { .alpha, .beta -> { .alpha = alpha + 1 } }\n    view {\n        text \"t\" {}\n    }\n}\n";
    let out = compile(two_sources).expect("a single-line body with two sources parses");
    let row = out.description.components[0]
        .members
        .iter()
        .find(|m| m.role == "watch")
        .expect("a watch row");
    assert_eq!(row.sources, vec!["alpha".to_string(), "beta".to_string()]);
    // 第二个源若是未声明名字 → 具名拒（不能只验第一个）。
    let bad = "node M {\n    model {\n        alpha int = 0\n    }\n    watch { .alpha, .nosuch -> { .alpha = alpha + 1 } }\n    view {\n        text \"t\" {}\n    }\n}\n";
    let ds = match compile(bad) {
        Ok(_) => panic!("the second watch source must be checked"),
        Err(ds) => ds,
    };
    assert_eq!(ds[0].code, "source.age.member-field", "{ds:?}");
}

/// R62-2：watch 源拒绝的 span 指向**源 token**（不是整个成员）。
#[test]
fn p981_watch_source_rejection_points_at_the_source() {
    let bad = "node M {\n    model {\n        alpha int = 0\n    }\n    watch { .alpha, .nosuch -> { .alpha = alpha + 1 } }\n    view {\n        text \"t\" {}\n    }\n}\n";
    let want = bad.find("nosuch").expect("fixture has the bad source");
    let ds = match compile(bad) {
        Ok(_) => panic!("the second watch source must be checked"),
        Err(ds) => ds,
    };
    assert_eq!(ds[0].code, "source.age.member-field", "{ds:?}");
    assert_eq!(ds[0].span.start, want, "span points at the source token");
}

/// R61-4：view 元素属性值表达式进唯一 verify（`class: nosuch_ident` 不能静默通过）。
#[test]
fn p981_view_attribute_expressions_are_verified() {
    let good = "node M {\n    model {\n        count int = 0\n    }\n    view {\n        col {\n            class: \"w-full\"\n        }\n    }\n}\n";
    compile(good).expect("a literal attribute value compiles");
    let bad = "node M {\n    model {\n        count int = 0\n    }\n    view {\n        col {\n            class: nosuch_ident\n        }\n    }\n}\n";
    let want = bad.find("nosuch_ident").expect("fixture has the bad name");
    let ds = match compile(bad) {
        Ok(_) => panic!("an undefined attribute value must be rejected"),
        Err(ds) => ds,
    };
    assert_eq!(ds[0].code, "source.undefined", "{ds:?}");
    assert_eq!(ds[0].span.start, want, "diagnostic at the attribute value");
}

/// R61-4/R62-5：命名 view 的结构事实（名字/文本数/项数）进描述面，且树里的
/// 属性表达式进唯一 verify。
#[test]
fn p981_named_view_structure_is_described_and_verified() {
    let good = "node M {\n    view main {\n        col {\n            class: \"w\"\n            text \"hi\" {}\n        }\n    }\n    view {\n        text \"t\" {}\n    }\n}\n";
    let out = compile(good).expect("a named view with an element compiles");
    let facts = &out.description.components[0].named_views;
    assert_eq!(facts.len(), 1, "{facts:?}");
    assert_eq!(facts[0].name, "main");
    assert_eq!(facts[0].texts, 1, "the text leaf is counted");
    assert_eq!(facts[0].items, 3, "element + class entry + text");
    let bad = "node M {\n    view main {\n        col {\n            class: nosuch_ident\n        }\n    }\n    view {\n        text \"t\" {}\n    }\n}\n";
    let want = bad.find("nosuch_ident").expect("fixture has the bad name");
    let ds = match compile(bad) {
        Ok(_) => panic!("an undefined attribute inside a named view must be rejected"),
        Err(ds) => ds,
    };
    assert_eq!(ds[0].code, "source.undefined", "{ds:?}");
    assert_eq!(ds[0].span.start, want);
}

/// R64-4：同一 AGE 包内两个 node 使用**同名字段**不再撞状态函数名。
#[test]
fn p981_dual_nodes_with_same_field_compile() {
    let src = "node A {\n    model { count int = 0 }\n    on Tick { count = count + 1 }\n    view {\n        text \"a\" {}\n    }\n}\nnode B {\n    model { count int = 0 }\n    view {\n        text \"b\" {}\n    }\n}\n";
    let out = compile(src).expect("two nodes with the same field name compile in one package");
    let cs = &out.description.components;
    assert_eq!(cs.len(), 2, "{cs:?}");
    assert_eq!(cs[0].name, "A");
    assert_eq!(cs[1].name, "B");
    assert!(cs[0].fields.iter().any(|(n, _)| n == "count"));
    assert!(cs[1].fields.iter().any(|(n, _)| n == "count"));
}

/// R62-7：watch 修饰形 `.a.b`（源 token + 同行修饰）现在可解析并保留完整拼写。
#[test]
fn p981_watch_modifier_source_parses_and_keeps_its_spelling() {
    for src in [
        "node M {\n    model {\n        alpha int = 0\n    }\n    watch { .alpha.x -> { .alpha = alpha + 1 } }\n    view {\n        text \"t\" {}\n    }\n}\n",
        "node M {\n    model {\n        alpha int = 0\n    }\n    watch {\n        .alpha.x -> {\n            .alpha = alpha + 1\n        }\n    }\n    view {\n        text \"t\" {}\n    }\n}\n",
    ] {
        let out = compile(src).expect("a modifier watch source parses");
        let row = out.description.components[0]
            .members
            .iter()
            .find(|m| m.role == "watch")
            .expect("a watch row");
        assert_eq!(row.sources, vec!["alpha.x".to_string()], "{src}");
    }
}
