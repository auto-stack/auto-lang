//! PLAN-979 T-H0-V：AGE H0 的 CPU 观察链（领域描述 → 共享 lowering/MIR/ABC/AV）。
//!
//! 断言面：文本值必须来自真实共享执行（换文本/换函数体改变观察值），
//! 结构角色与原始 span 由 Schema 决定，兼容配置只改接纳面不改领域语义。

use ace_av::age_h0::observe_age_h0;
use ace_core::source::age_h0::{LegacyWidgetAlias, TextKind};

const LITERAL: &str = "node Hello {\n    view {\n        text \"Hello\" {}\n    }\n}\n";

const FN_VIEW: &str = r#"fn hello_text() str {
    return "你好"
}
node Hello {
    view {
        text hello_text() {}
    }
}
"#;

const FN_CONCAT_OWNED: &str = r#"fn hello_text() Str {
    let head = "你"
    return head + "好"
}
node Hello {
    view {
        text hello_text() {}
    }
}
"#;

const FN_OWNED: &str = r#"fn owned_text() Str {
    return "Owned".clone()
}
node Owned {
    view {
        text owned_text() {}
    }
}
"#;

#[test]
fn p979_age_h0_literal_value_comes_from_the_shared_execution_chain() {
    let o = observe_age_h0(LITERAL, "age.at", LegacyWidgetAlias::Reject).expect("observes");
    assert_eq!(o.nodes.len(), 1);
    let n = &o.nodes[0];
    assert_eq!(n.name, "Hello");
    assert_eq!(n.head_role, "node");
    assert!(!n.legacy_widget);
    assert_eq!(n.view.role, "view");
    assert_eq!(n.view.texts.len(), 1);
    let t = &n.view.texts[0];
    assert_eq!(t.role, "text");
    assert_eq!(t.value, "Hello");
    assert_eq!(t.kind, TextKind::View);
    let at = LITERAL.rfind("Hello").unwrap();
    assert_eq!(t.component_span.0, LITERAL.find("text").unwrap());
    assert_eq!(t.component_span.1, LITERAL.find("text").unwrap() + 4);
    assert!(t.span.0 <= at && t.span.1 >= at + 5);
}

#[test]
fn p979_age_h0_function_value_is_executed_not_faked() {
    let o = observe_age_h0(FN_VIEW, "age.at", LegacyWidgetAlias::Reject).expect("observes");
    let t = &o.nodes[0].view.texts[0];
    assert_eq!(t.value, "你好", "the function really ran in AV");
    assert_eq!(t.kind, TextKind::View);
    // Distinctive control: a different function body produces a different value
    // through the same chain (no Rust constant can satisfy both).
    let other = FN_VIEW.replace("return \"你好\"", "return \"Bon好\"");
    let o2 = observe_age_h0(&other, "age.at", LegacyWidgetAlias::Reject).expect("observes");
    let t2 = &o2.nodes[0].view.texts[0];
    assert_eq!(t2.value, "Bon好");
    assert_ne!(t.value, t2.value);
    // Concatenation (an owned `Str` value) goes through the same chain and is
    // reported with the checker's owned text kind.
    let c = observe_age_h0(FN_CONCAT_OWNED, "age.at", LegacyWidgetAlias::Reject).expect("observes");
    let tc = &c.nodes[0].view.texts[0];
    assert_eq!(tc.value, "你好");
    assert_eq!(tc.kind, TextKind::Owned);
}

#[test]
fn p979_age_h0_owned_text_kind_is_carried_from_the_shared_checker() {
    let o = observe_age_h0(FN_OWNED, "age.at", LegacyWidgetAlias::Reject).expect("observes");
    let t = &o.nodes[0].view.texts[0];
    assert_eq!(t.kind, TextKind::Owned);
    assert_eq!(t.value, "Owned");
}

#[test]
fn p979_age_h0_legacy_widget_alias_is_explicit_and_same_candidate() {
    let src = "widget App {\n    view {\n        text \"Hi\" {}\n    }\n}\n";
    let err = observe_age_h0(src, "age.at", LegacyWidgetAlias::Reject)
        .expect_err("alias off rejects the legacy head");
    assert!(err.contains("source.age.legacy-widget"), "{err}");
    let on = observe_age_h0(src, "age.at", LegacyWidgetAlias::Accept).expect("alias on observes");
    assert_eq!(on.nodes.len(), 1);
    assert_eq!(on.nodes[0].name, "App");
    assert!(on.nodes[0].legacy_widget);
    assert_eq!(on.nodes[0].head_role, "node", "same role as `node`");
    assert_eq!(on.nodes[0].view.texts[0].value, "Hi");
    // The canonical spelling produces the identical observed candidate.
    let node_src = src.replace("widget App", "node App");
    let canonical = observe_age_h0(&node_src, "age.at", LegacyWidgetAlias::Reject)
        .expect("canonical node observes");
    assert_eq!(
        canonical.nodes[0].view.texts[0].value,
        on.nodes[0].view.texts[0].value
    );
    assert_eq!(canonical.nodes[0].head_role, on.nodes[0].head_role);
    assert!(!canonical.nodes[0].legacy_widget);
}

#[test]
fn p979_age_h0_rejections_surface_with_the_original_span() {
    let bad = "node Hello {\n    panel {\n        text \"Hello\" {}\n    }\n}\n";
    let err = observe_age_h0(bad, "age.at", LegacyWidgetAlias::Reject).expect_err("rejects");
    assert!(err.contains("expected `view`"), "{err}");
    // Non-text leaves are rejected by the shared type rules (not by a second checker).
    let int_text = "node Hello {\n    view {\n        text 1 + 2 {}\n    }\n}\n";
    assert!(observe_age_h0(int_text, "age.at", LegacyWidgetAlias::Reject).is_err());
}

#[test]
fn p979_age_h0_two_independent_nodes_observe_independently() {
    let src = "node A {\n    view {\n        text \"one\" {}\n    }\n}\nnode B {\n    view {\n        text \"two\" {}\n    }\n}\n";
    let o = observe_age_h0(src, "age.at", LegacyWidgetAlias::Reject).expect("observes");
    assert_eq!(o.nodes.len(), 2);
    assert_eq!(o.nodes[0].name, "A");
    assert_eq!(o.nodes[0].view.texts[0].value, "one");
    assert_eq!(o.nodes[1].name, "B");
    assert_eq!(o.nodes[1].view.texts[0].value, "two");
    assert_ne!(
        o.nodes[0].view.texts[0].value,
        o.nodes[1].view.texts[0].value
    );
}

#[test]
fn p979_age_h0_near_miss_user_function_cannot_bind_the_text_leaf() {
    // F1 值层回归：用户函数名与隐藏前缀相似但不含完整前缀时合法，叶值仍来自
    // 叶表达式（不是那个函数）。
    let src = "fn x_AgeH0Text0() Str {\n    return \"HACK\".clone()\n}\nnode Hello {\n    view {\n        text \"Hello\" {}\n    }\n}\n";
    let o = observe_age_h0(src, "age.at", LegacyWidgetAlias::Reject).expect("observes");
    assert_eq!(o.nodes[0].view.texts[0].value, "Hello");
    // 而真正落在保留前缀家族里的名字是具名冲突（不静默绑定）。
    let hijack = "fn x__age_h0_text_0() Str {\n    return \"HACK\".clone()\n}\nnode Hello {\n    view {\n        text \"Hello\" {}\n    }\n}\n";
    let err = observe_age_h0(hijack, "age.at", LegacyWidgetAlias::Reject)
        .expect_err("reserved prefix family is rejected");
    assert!(err.contains("source.age.hidden-name"), "{err}");
}

#[test]
fn p979_age_same_source_feeds_both_backend_legs() {
    // T-03：同一份 AGE 源（含普通 helper 与 i32 main），AV 腿取文本叶值；
    // A2R 腿由 `crates/a2x-worker/tests/cases/plan979_age_a2r.rs` 用同一源
    // 真实 rustc 编译运行（value:4）。这里只核 AV 侧的领域值。
    let src = "fn counter_text(n int) str {\n    if n > 2 {\n        return \"many\"\n    }\n    return \"few\"\n}\nnode Counter {\n    view {\n        text counter_text(2) {}\n    }\n}\nfn main() int {\n    let t = counter_text(3)\n    return t.len_bytes()\n}\n";
    let o = observe_age_h0(src, "age.at", LegacyWidgetAlias::Reject).expect("observes");
    assert_eq!(o.nodes.len(), 1);
    assert_eq!(o.nodes[0].view.texts[0].value, "few");
    assert_eq!(o.nodes[0].view.texts[0].role, "text");
}
