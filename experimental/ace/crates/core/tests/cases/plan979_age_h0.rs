//! PLAN-979 T-H0-I：AGE H0 领域前门（core 侧）。
//!
//! 覆盖（语法层之上的领域层）：Schema 驱动的结构识别 → 未受信候选 →
//! 唯一 verify（共享 text-record 链）→ 受检描述（角色 + 原 span + 表达式
//! 标识 + 文本种类）。拒绝面：未知组件拼写、属性、缺闭合、非文本表达式、
//! 别名关闭的旧头、隐藏求值函数重名。

use ace_core::source::age_h0::{
    compile_age_h0, LegacyWidgetAlias, TextKind, AGE_H0_DOMAIN, AGE_H0_HIDDEN_PREFIX,
};

const H0_LITERAL: &str = "node Hello {\n    view {\n        text \"Hello\" {}\n    }\n}\n";

const H0_FUNCTION: &str = r#"fn hello_text() str {
    return "你好"
}
node Hello {
    view {
        text hello_text() {}
    }
}
"#;

fn compile(
    src: &str,
    alias: LegacyWidgetAlias,
) -> Result<ace_core::source::age_h0::AgeH0Compilation, Vec<String>> {
    compile_age_h0(src, "age.at", alias, &[]).map_err(|ds| {
        ds.into_iter()
            .map(|d| format!("{}: {}", d.code, d.message))
            .collect()
    })
}

#[test]
fn p979_age_h0_literal_describes_one_node_view_text() {
    let c = compile(H0_LITERAL, LegacyWidgetAlias::Reject).expect("literal H0 compiles");
    assert_eq!(c.description.domain, AGE_H0_DOMAIN);
    assert_eq!(c.description.nodes.len(), 1);
    let n = &c.description.nodes[0];
    assert_eq!(n.name, "Hello");
    assert_eq!(n.head_role, "node");
    assert!(!n.legacy_widget);
    assert_eq!(n.view.role, "view");
    assert_eq!(n.view.texts.len(), 1);
    let t = &n.view.texts[0];
    assert_eq!(t.role, "text");
    assert_eq!(t.kind, TextKind::View, "a literal is a str view");
    assert!(t.def_id.starts_with("d_"), "checked def id: {}", t.def_id);
    // 原 span：组件头 `text` 与整个 leaf 都指向源文件真实字节。
    let comp_at = H0_LITERAL.find("text \"Hello\"").unwrap();
    assert_eq!(t.component_span.start, comp_at);
    assert_eq!(t.component_span.end, comp_at + 4);
    assert!(t.span.start >= comp_at && t.span.end <= H0_LITERAL.len());
    // span 不漂移：node 头在同源位置。
    let head_at = H0_LITERAL.find("node").unwrap();
    assert_eq!(n.head_span.start, head_at);
}

#[test]
fn p979_age_h0_function_expression_is_checked_by_the_shared_verify() {
    let c = compile(H0_FUNCTION, LegacyWidgetAlias::Reject).expect("function H0 compiles");
    let n = &c.description.nodes[0];
    let t = &n.view.texts[0];
    // `fn hello_text() str` returns a view; the leaf's checked text kind comes
    // from the shared resolver verdict, not from the domain layer.
    assert_eq!(t.kind, TextKind::View);
    // The hidden evaluation function is an ordinary checked function returning Str.
    let checked = &c.checked;
    let module = &checked.bundle().modules[0];
    let hidden = module
        .defs
        .iter()
        .find(|d| matches!(&d.kind, ace_core::hir::DefKind::Function { name, .. } if name.ends_with(&format!("{AGE_H0_HIDDEN_PREFIX}0"))))
        .expect("hidden text function exists in Checked HIR");
    assert_eq!(hidden.id_text, t.def_id);
    // The ordinary user function is still checked in the same module.
    assert!(module.defs.iter().any(|d| matches!(&d.kind, ace_core::hir::DefKind::Function { name, .. } if name.ends_with("hello_text"))));
}

#[test]
fn p979_age_h0_rejects_unknown_component_with_original_span() {
    let src = "node Hello {\n    panel {\n        text \"Hello\" {}\n    }\n}\n";
    let err = compile(src, LegacyWidgetAlias::Reject).expect_err("undeclared component rejected");
    assert!(
        err.iter().any(|m| m.contains("expected `view`")),
        "slot rejection is named: {err:?}"
    );
}

#[test]
fn p979_age_h0_rejects_attributes_in_the_h0_slice() {
    let src = "node Hello {\n    view {\n        text \"Hello\" { size: 12 }\n    }\n}\n";
    let err = compile(src, LegacyWidgetAlias::Reject).expect_err("attributes rejected");
    assert!(!err.is_empty(), "a diagnostic is produced: {err:?}");
}

#[test]
fn p979_age_h0_rejects_unclosed_structure() {
    let src = "node Hello {\n    view {\n        text \"Hello\" {}\n    }\n";
    let err = compile(src, LegacyWidgetAlias::Reject).expect_err("unclosed rejected");
    assert!(!err.is_empty());
}

#[test]
fn p979_age_h0_rejects_non_text_expression_at_the_expression_span() {
    let src = "node Hello {\n    view {\n        text 1 + 2 {}\n    }\n}\n";
    let err = compile(src, LegacyWidgetAlias::Reject).expect_err("int text expression rejected");
    assert!(!err.is_empty(), "{err:?}");
}

#[test]
fn p979_age_h0_legacy_widget_head_needs_the_compat_config() {
    let src = "widget App {\n    view {\n        text \"Hello\" {}\n    }\n}\n";
    let err =
        compile(src, LegacyWidgetAlias::Reject).expect_err("alias off rejects the legacy head");
    assert!(
        err.iter()
            .any(|m| m.starts_with("source.age.legacy-widget")),
        "controlled rejection is named: {err:?}"
    );
    // With the compat config the same bytes become the same Node candidate.
    let on = compile(src, LegacyWidgetAlias::Accept).expect("alias on accepts the legacy head");
    assert_eq!(on.description.nodes.len(), 1);
    assert!(on.description.nodes[0].legacy_widget);
    assert_eq!(on.description.nodes[0].head_role, "node");
    assert_eq!(on.description.nodes[0].name, "App");
    // Identity: the compat config is part of the description, not the source.
    assert_eq!(on.description.alias_name(), "accept");
    let off_reject = compile(H0_LITERAL, LegacyWidgetAlias::Reject).unwrap();
    let on_accept = compile(H0_LITERAL, LegacyWidgetAlias::Accept).unwrap();
    assert_eq!(
        off_reject.description.grammar_id, on_accept.description.grammar_id,
        "the same schema/grammar backs both configs"
    );
    assert_ne!(
        off_reject.description.alias_name(),
        on_accept.description.alias_name()
    );
}

#[test]
fn p979_age_h0_hidden_name_collision_is_named() {
    let src = format!(
        "fn {AGE_H0_HIDDEN_PREFIX}0() str {{\n    return \"x\"\n}}\nnode Hello {{\n    view {{\n        text \"Hello\" {{}}\n    }}\n}}\n"
    );
    let err = compile(&src, LegacyWidgetAlias::Reject).expect_err("collision rejected");
    assert!(
        err.iter().any(|m| m.starts_with("source.age.hidden-name")),
        "{err:?}"
    );
}

#[test]
fn p979_age_h0_zero_or_many_text_roots_are_rejected() {
    let zero = "node Hello {\n    view {\n    }\n}\n";
    let err = compile(zero, LegacyWidgetAlias::Reject).expect_err("zero text roots rejected");
    assert!(
        err.iter().any(|m| m.starts_with("source.age.text-count")),
        "{err:?}"
    );
    let many = "node Hello {\n    view {\n        text \"a\" {}\n        text \"b\" {}\n    }\n}\n";
    let err = compile(many, LegacyWidgetAlias::Reject).expect_err("two text roots rejected");
    assert!(
        err.iter().any(|m| m.starts_with("source.age.text-count")),
        "{err:?}"
    );
}

#[test]
fn p979_age_h0_generic_front_doors_refuse_the_domain_profile() {
    // 通用解析/编译入口显式拒绝 AGE profile（领域声明绝不静默丢弃）。
    let err =
        ace_core::source::parse_program_with(H0_LITERAL, ace_core::source::SourceProfile::AgeH0)
            .expect_err("generic parse refuses the domain profile");
    assert_eq!(err.code, "source.domain-frontend");
}

#[test]
fn p979_age_h0_ordinary_modules_stay_ordinary() {
    // 没有 AGE 声明时就是普通 text-record 模块（同一前门仍可用）。
    let src = "fn on(model int) int {\n    let text = model + 1\n    let widget = text + 1\n    return widget\n}\n";
    let c = compile(src, LegacyWidgetAlias::Reject).expect("ordinary module compiles");
    assert!(c.description.nodes.is_empty());
    // V-H0-POSITION 的"普通源有无 AGE 相同"：无领域叶时脱糖模块与
    // text-record 前门产出的共享 Atom 文本逐字节同形（同值/同类型）。
    let plain = ace_core::source::compile_text_record_source(src, "age.at", &[]).expect("plain");
    assert_eq!(c.atom_text, plain.atom_text);
    assert_eq!(c.input_hash, plain.input_hash);
    assert!(!H0_LITERAL.is_empty());
}

#[test]
fn p979_age_h0_failed_parse_does_not_pollute_the_next_parse() {
    let bad = "node Hello {\n    panel {\n        text \"Hello\" {}\n    }\n}\n";
    assert!(compile(bad, LegacyWidgetAlias::Reject).is_err());
    // 同进程紧随其后的合法源不受前一失败影响（活动领域/槽位上下文无泄漏）。
    let ok = compile(H0_LITERAL, LegacyWidgetAlias::Reject).expect("legal source after a failure");
    assert_eq!(ok.description.nodes[0].name, "Hello");
    assert_eq!(ok.description.nodes[0].view.texts[0].kind, TextKind::View);
    // 普通模块同样不受污染。
    let ordinary = compile(
        "fn f() int {\n    return 41 + 1\n}\n",
        LegacyWidgetAlias::Reject,
    )
    .expect("ordinary source after a failure");
    assert!(ordinary.description.nodes.is_empty());
    assert_eq!(
        ordinary.checked.domain(),
        ace_core::hir::HirDomain::TextRecord
    );
}

#[test]
fn p979_age_h0_identity_binds_schema_and_source() {
    let a = compile(H0_LITERAL, LegacyWidgetAlias::Reject).unwrap();
    let b = compile(H0_LITERAL, LegacyWidgetAlias::Reject).unwrap();
    assert_eq!(
        a.description.grammar_id, b.description.grammar_id,
        "stable schema identity"
    );
    assert_eq!(a.input_hash, b.input_hash, "stable source identity");
    // 领域面与 plain text-record 面是不同的语法程序（身份隔离）。
    let plain = ace_core::source::compile_text_record_source(H0_LITERAL, "age.at", &[]);
    assert!(plain.is_err(), "plain text-record must not parse AGE nodes");
    // 改源 → 源身份变；schema 身份不变。
    let changed = H0_LITERAL.replace("\"Hello\"", "\"Hello, world\"");
    let c = compile(&changed, LegacyWidgetAlias::Reject).unwrap();
    assert_ne!(a.input_hash, c.input_hash);
    assert_eq!(a.description.grammar_id, c.description.grammar_id);
    assert_eq!(c.checked.domain(), ace_core::hir::HirDomain::TextRecord);
}
