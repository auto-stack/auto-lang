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
    compile_diags(src, alias).map_err(|ds| {
        ds.into_iter()
            .map(|d| format!("{}: {}", d.code, d.message))
            .collect()
    })
}

/// 诊断原文（code + span），用于断言"具名码 + 原文件 span"。
fn compile_diags(
    src: &str,
    alias: LegacyWidgetAlias,
) -> Result<ace_core::source::age_h0::AgeH0Compilation, Vec<ace_core::atom_text::Diagnostic>> {
    compile_age_h0(src, "age.at", alias, &[])
}

fn first(ds: &[ace_core::atom_text::Diagnostic]) -> &ace_core::atom_text::Diagnostic {
    ds.first().expect("at least one diagnostic")
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

// ---- F1/F2/F3/F5/F6 repair regressions (independent review round 1) ----

#[test]
fn p979_age_h0_suffix_named_user_function_cannot_hijack_a_text_leaf() {
    // F1：含保留前缀的任意用户顶层名都是具名冲突（此前后缀匹配会把叶绑到它）。
    let hijack = format!(
        "fn x{AGE_H0_HIDDEN_PREFIX}0() Str {{\n    return \"HACK\".clone()\n}}\n{H0_LITERAL}"
    );
    let ds = compile_diags(&hijack, LegacyWidgetAlias::Reject)
        .expect_err("the hidden-name prefix family is reserved");
    assert_eq!(first(&ds).code, "source.age.hidden-name");
    let at = hijack.find(&format!("x{AGE_H0_HIDDEN_PREFIX}0")).unwrap();
    assert_eq!(first(&ds).span.start, at);

    // 近似但**不含**完整前缀的名字合法，且叶的 def id 仍是隐藏函数的精确 id。
    let near = format!("fn x_AgeH0Text0() Str {{\n    return \"HACK\".clone()\n}}\n{H0_LITERAL}");
    let ok = compile(&near, LegacyWidgetAlias::Reject).expect("near-miss name is legal");
    let t = &ok.description.nodes[0].view.texts[0];
    assert_eq!(t.def_id, "d___age_h0_text_0", "exact hidden def id");
}

#[test]
fn p979_age_h0_every_generic_entry_refuses_the_domain_profile() {
    use ace_core::source::cst::{
        parse_front, parse_program_via_cst, FrontBudget, FrontKind, FrontOutcome, FrontRequest,
    };
    use ace_core::source::SourceProfile;

    // 1) 兼容桥（ace-syntax-core 也重导出它——评审 F2 的绕过点）。
    let e = parse_program_via_cst(H0_LITERAL, SourceProfile::AgeH0)
        .expect_err("parse_program_via_cst must refuse the domain profile");
    assert_eq!(e.code, "source.domain-frontend");
    // 2) 统一前门（整模块与片段都必须拒）。
    for kind in [
        FrontKind::WholeModule,
        FrontKind::FreeFnFragment { range: 0..1 },
    ] {
        match parse_front(FrontRequest {
            src: H0_LITERAL,
            profile: SourceProfile::AgeH0,
            kind,
            budget: FrontBudget::WORK_OWNED_64MIB,
        }) {
            FrontOutcome::Diag(ds) => assert_eq!(ds[0].code, "source.domain-frontend"),
            other => panic!("expected the domain refusal, got {other:?}"),
        }
    }
    // 3) 已投影程序入口（compile_program_parts）也不得接受领域 profile。
    let program = ace_core::source::ast::Program { items: Vec::new() };
    let ds = match ace_core::source::adapter::compile_program_parts(
        &program,
        H0_LITERAL,
        "age.at",
        &[],
        SourceProfile::AgeH0,
    ) {
        Ok(_) => panic!("compile_program_parts must refuse the domain profile"),
        Err(ds) => ds,
    };
    assert_eq!(ds[0].code, "source.domain-frontend");
}

#[test]
fn p979_age_h0_rejections_are_domain_named_with_the_original_span() {
    // F3：结构/类型拒绝必须是领域具名码 + 原文件 span（不是通用 "expected fn"）。
    let missing_view = "node Hello {\n}\n";
    let ds = compile_diags(missing_view, LegacyWidgetAlias::Reject).expect_err("missing view");
    assert_eq!(first(&ds).code, "source.age.slot");
    assert!(first(&ds).message.contains("expected `view`"));

    let second_view = "node Hello {\n    view {\n        text \"a\" {}\n        view {\n            text \"b\" {}\n        }\n    }\n}\n";
    let ds = compile_diags(second_view, LegacyWidgetAlias::Reject).expect_err("second view");
    assert_eq!(first(&ds).code, "source.age.structure");
    let second_at = second_view.rfind("view").unwrap();
    assert_eq!(
        first(&ds).span.start,
        second_at,
        "points at the second `view`"
    );

    let attr = "node Hello {\n    view {\n        text \"Hello\" { size: 12 }\n    }\n}\n";
    let ds = compile_diags(attr, LegacyWidgetAlias::Reject).expect_err("attributes");
    assert_eq!(first(&ds).code, "source.age.structure");
    assert_eq!(first(&ds).span.start, attr.find("size").unwrap());

    let unclosed = "node Hello {\n    view {\n        text \"Hello\" {}\n    }\n";
    let ds = compile_diags(unclosed, LegacyWidgetAlias::Reject).expect_err("unclosed");
    assert_eq!(first(&ds).code, "source.age.structure");

    let non_text = "node Hello {\n    view {\n        text 1 + 2 {}\n    }\n}\n";
    let ds = compile_diags(non_text, LegacyWidgetAlias::Reject).expect_err("non-text leaf");
    assert_eq!(first(&ds).code, "source.age.text-type");
    assert_eq!(first(&ds).span.start, non_text.find("1 + 2").unwrap());
}

#[test]
fn p979_age_h0_import_front_door_has_text_record_parity() {
    // F5：`use.rs`/`use.c` 与 text-record 同面（导入语义仍归 F0 共享层）。
    let only = "use.rs provider_x\n";
    let age = compile(only, LegacyWidgetAlias::Reject).expect("age accepts the import front door");
    let plain = ace_core::source::compile_text_record_source(only, "age.at", &[])
        .expect("plain accepts it");
    assert_eq!(age.atom_text, plain.atom_text, "identical shared Atom text");
    assert_eq!(age.input_hash, plain.input_hash);
    assert!(age.description.nodes.is_empty());
}

#[test]
fn p979_age_h0_earlier_shared_errors_are_not_swallowed() {
    // F6：声明之前的真实错误优先于领域 hint。
    let src =
        "fn f() int {\n    return 1 +\n}\nnode A {\n    panel {\n        text \"x\" {}\n    }\n}\n";
    let ds = compile_diags(src, LegacyWidgetAlias::Reject).expect_err("errors");
    let d = first(&ds);
    assert_ne!(d.code, "source.age.slot", "{:?}", d);
    assert!(
        d.span.start < src.find("node A").unwrap(),
        "the earlier fn error keeps precedence: {d:?}"
    );
}

// ---- F9/F10/F11 + fragment-refusal regressions (independent review round 2) ----

#[test]
fn p979_age_h0_non_head_identifier_does_not_define_the_age_head() {
    // F9：前面合法函数里叫 `node` 的标识符不得被当成领域声明头（否则更早的
    // 真实错误被吞）。A/B 只差第一个函数名，首个诊断必须一致地落在 node 行之前。
    let with_node_fn = "fn node() int {\n    return 1\n}\nfn g() int {\n    return 1 +\n}\nnode A {\n    panel {\n        text \"x\" {}\n    }\n}\n";
    let with_foo_fn = with_node_fn.replace("fn node()", "fn foo()");
    let a = compile_diags(with_node_fn, LegacyWidgetAlias::Reject).expect_err("errors");
    let b = compile_diags(&with_foo_fn, LegacyWidgetAlias::Reject).expect_err("errors");
    let node_line = with_node_fn.find("node A").unwrap();
    assert!(first(&a).span.start < node_line, "{:?}", first(&a));
    assert!(first(&b).span.start < node_line, "{:?}", first(&b));
    assert_eq!(
        first(&a).code,
        first(&b).code,
        "A/B must agree on the class"
    );
}

#[test]
fn p979_age_h0_undefined_callee_is_not_reported_as_a_text_type_error() {
    // F10：叶内名字/解析错误保留共享语义（中立领域码），只有脱糖 `.clone()`
    // 引起的拒绝才是 `source.age.text-type`。
    let src = "node Hello {\n    view {\n        text no_such_fn() {}\n    }\n}\n";
    let ds = compile_diags(src, LegacyWidgetAlias::Reject).expect_err("undefined callee");
    let d = first(&ds);
    assert_eq!(d.code, "source.age.leaf-expr", "{d:?}");
    assert!(d.message.contains("no function named"), "{d:?}");
    assert_eq!(d.span.start, src.find("no_such_fn").unwrap());
    // 非文本值仍走 text-type。
    let non_text = "node Hello {\n    view {\n        text 1 + 2 {}\n    }\n}\n";
    let ds = compile_diags(non_text, LegacyWidgetAlias::Reject).expect_err("non-text");
    assert_eq!(first(&ds).code, "source.age.text-type");
}

#[test]
fn p979_age_h0_missing_node_name_is_a_domain_structure_error() {
    // F11：缺节点名也必须是领域具名结构诊断（span 指向 `{`）。
    let src = "node {\n    view {\n        text \"x\" {}\n    }\n}\n";
    let ds = compile_diags(src, LegacyWidgetAlias::Reject).expect_err("missing name");
    let d = first(&ds);
    assert_eq!(d.code, "source.age.structure", "{d:?}");
    assert_eq!(d.span.start, src.find('{').unwrap());
}

#[test]
fn p979_age_h0_fragment_entries_refuse_the_domain_profile() {
    // 评审观察：直调片段入口也必须具名拒绝（片段不承载领域声明）。
    use ace_core::source::cst::{parse_fragment_front, FrontBudget, FrontOutcome};
    use ace_core::source::SourceProfile;
    let src = "fn f() int {\n    return 1\n}\n";
    match parse_fragment_front(
        src,
        SourceProfile::AgeH0,
        FrontBudget::WORK_OWNED_64MIB,
        &(0..src.len()),
        None,
        ace_core::source::ast::Receiver::Free,
    ) {
        FrontOutcome::Diag(ds) => assert_eq!(ds[0].code, "source.domain-frontend"),
        other => panic!("expected the domain refusal, got {other:?}"),
    }
}
