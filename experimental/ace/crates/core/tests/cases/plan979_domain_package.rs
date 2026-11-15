//! PLAN-979 T-02：**通用领域包接面**（`ace_core::source::domain`）的独立性证明。
//!
//! 本文件不经过 `age_h0` 的公开入口，而是像外部领域 owner（auto-ui/980）那样
//! 只通过公开类型注册**第二个**领域包，验证：
//! - 共享驱动对任何包都只做 前门 → 包投影 → 兼容闸 → 隐藏名 → 脱糖 → 唯一 verify；
//! - 不同包对**同一份源**产出各自的结构行（无 AGE 硬编码）；
//! - 包级兼容规则（拼写/代码/消息）与隐藏前缀按包生效；
//! - 行为检查仍由共享 verify 完成（表达式拒绝保原 span）。

use ace_core::atom_text::{Diagnostic, Span};
use ace_core::source::cst::FrontOk;
use ace_core::source::domain::{
    compile_domain, CompatRule, DomainBehavior, DomainCandidate, DomainItem, DomainPackage,
    DomainVersion,
};
use ace_core::source::SourceProfile;

const SRC: &str = "node Hello {\n    view {\n        text \"Hello\" {}\n    }\n}\n";

/// 第二个领域包：同一 grammar 面，但自有结构（`pair` 行包两个 `card` 子行）、
/// 自有兼容规则与隐藏前缀。
static PAIR_PACKAGE: DomainPackage = DomainPackage {
    id: "test.pair/v1",
    version: DomainVersion { major: 0, minor: 1 },
    profile: SourceProfile::AgeH0,
    hidden_prefix: "__pair_val_",
    compat: Some(CompatRule {
        spelling: "widget",
        disabled_code: "test.pair.compat",
        disabled_message: "the pair package needs its own compat config",
    }),
    project: project_pair,
};

fn project_pair(ok: &FrontOk, _src: &str) -> Result<DomainCandidate, Vec<Diagnostic>> {
    let mut structure = Vec::new();
    let mut behaviors = Vec::new();
    for n in &ok.age_nodes {
        let mut cards = Vec::new();
        for leaf in &n.view.texts {
            let idx = behaviors.len();
            behaviors.push(DomainBehavior {
                expr: leaf.expr.clone(),
                span: leaf.expr.span,
            });
            cards.push(DomainItem {
                role: "card".to_string(),
                name: None,
                span: leaf.span,
                aux_span: Some(leaf.component_span),
                spelling: None,
                bindings: vec![idx],
                children: Vec::new(),
            });
        }
        structure.push(DomainItem {
            role: "pair".to_string(),
            name: Some(n.name.name.clone()),
            span: n.span,
            aux_span: Some(n.head_span),
            spelling: Some(n.head_spelling.clone()),
            bindings: Vec::new(),
            children: cards,
        });
    }
    Ok(DomainCandidate {
        structure,
        behaviors,
        requirements: vec![],
    })
}

/// 普通 profile 的包（应当被共享前门拒绝）。
static PLAIN_PACKAGE: DomainPackage = DomainPackage {
    id: "test.plain/v1",
    version: DomainVersion { major: 0, minor: 1 },
    profile: SourceProfile::TextRecord,
    hidden_prefix: "__plain_val_",
    compat: None,
    project: project_pair,
};

fn first(ds: &[Diagnostic]) -> &Diagnostic {
    ds.first().expect("at least one diagnostic")
}

#[test]
fn p979_domain_package_second_owner_plugs_in_through_the_public_api() {
    let out = compile_domain(&PAIR_PACKAGE, SRC, "pair.at", false, &[]).expect("pair compiles");
    assert_eq!(out.id, "test.pair/v1");
    assert_eq!(out.version, DomainVersion { major: 0, minor: 1 });
    assert_eq!(out.description.len(), 1);
    let pair = &out.description[0];
    assert_eq!(pair.role, "pair");
    assert_eq!(pair.name.as_deref(), Some("Hello"));
    assert_eq!(pair.children.len(), 1, "one card child");
    let card = &pair.children[0];
    assert_eq!(card.role, "card");
    assert_eq!(card.bindings.len(), 1);
    // 共享 verify 的结论：文本种类 + 共享 HIR def id（包自己的隐藏前缀）。
    assert_eq!(
        card.bindings[0].kind,
        ace_core::source::age_h0::TextKind::View
    );
    assert_eq!(card.bindings[0].def_id, "d___pair_val_0");
    // 同一份源经 AGE 包仍是 `node/view/text`（两个包互不影响）。
    let age = ace_core::source::age_h0::compile_age_h0(
        SRC,
        "age.at",
        ace_core::source::age_h0::LegacyWidgetAlias::Reject,
        &[],
    )
    .expect("age compiles");
    assert_eq!(age.description.nodes[0].head_role, "node");
    assert_eq!(age.description.nodes[0].view.role, "view");
}

#[test]
fn p979_domain_package_compat_rule_is_per_package() {
    let widget = "widget App {\n    view {\n        text \"Hi\" {}\n    }\n}\n";
    // 关闭：包自己的具名码 + 声明头 span（不是 AGE 的码）。
    let ds = compile_domain(&PAIR_PACKAGE, widget, "pair.at", false, &[])
        .expect_err("pair compat off rejects");
    assert_eq!(first(&ds).code, "test.pair.compat");
    assert_eq!(first(&ds).span.start, 0);
    // 打开：同字节 → 同结构，且行标记 compat。
    let out = compile_domain(&PAIR_PACKAGE, widget, "pair.at", true, &[]).expect("pair compat on");
    assert!(out.compat);
    assert_eq!(out.description[0].role, "pair");
    assert_eq!(out.description[0].spelling.as_deref(), Some("widget"));
}

#[test]
fn p979_domain_package_hidden_prefix_is_per_package() {
    // 包前缀冲突 = 具名拒绝；AGE 前缀在 pair 包里是普通名字（互不干涉）。
    let collide = format!("fn x__pair_val_0() Str {{\n    return \"HACK\".clone()\n}}\n{SRC}");
    let ds = compile_domain(&PAIR_PACKAGE, &collide, "pair.at", false, &[])
        .expect_err("pair prefix is reserved for the pair package");
    assert_eq!(first(&ds).code, "source.age.hidden-name");
    let age_prefix_name = format!("fn x__age_h0_text_0() str {{\n    return \"ok\"\n}}\n{SRC}");
    compile_domain(&PAIR_PACKAGE, &age_prefix_name, "pair.at", false, &[])
        .expect("another package's hidden prefix is an ordinary name here");
}

#[test]
fn p979_domain_package_behaviour_check_stays_in_the_shared_verify() {
    // 非文本叶：共享检查拒绝，原 span 保留，领域码由共享驱动标注。
    let bad = "node Hello {\n    view {\n        text 1 + 2 {}\n    }\n}\n";
    let ds = compile_domain(&PAIR_PACKAGE, bad, "pair.at", false, &[])
        .expect_err("non-text behaviour rejected");
    assert_eq!(first(&ds).code, "source.age.text-type");
    let at = bad.find("1 + 2").unwrap_or(0);
    assert_eq!(
        first(&ds).span,
        Span {
            start: at,
            end: at + 5
        }
    );
    // 名字错误：不是文本类型问题（中立领域码 + 共享消息）。
    let undefined = "node Hello {\n    view {\n        text no_such_fn() {}\n    }\n}\n";
    let ds = compile_domain(&PAIR_PACKAGE, undefined, "pair.at", false, &[])
        .expect_err("undefined callee rejected");
    assert_eq!(first(&ds).code, "source.age.leaf-expr");
    assert!(first(&ds).message.contains("no function named"));
}

#[test]
fn p979_domain_package_refuses_generic_profiles() {
    // 包驱动只接受领域 profile；普通 profile 由共享前门拒绝（不静默丢声明）。
    let ds =
        compile_domain(&PLAIN_PACKAGE, SRC, "plain.at", false, &[]).expect_err("plain profile");
    assert_eq!(first(&ds).code, "source.domain-package");
}

// ---- V-02：冲突包拒加载，旧注册保留 ----

#[test]
fn p979_domain_package_registry_refuses_conflicts_and_keeps_the_old_package() {
    use ace_core::source::domain::{DomainPackage, DomainRegistry, DomainVersion};

    // 与 PAIR_PACKAGE 同 id 但不同 profile 的变体（模拟"冲突包"）。
    static CONFLICT: DomainPackage = DomainPackage {
        id: "test.pair/v1",
        version: DomainVersion { major: 0, minor: 1 },
        profile: SourceProfile::AgeComponent,
        hidden_prefix: "__pair_val_",
        compat: None,
        project: project_pair,
    };
    // 同 id 同版本同 profile 的重复注册（幂等）。
    static SAME: DomainPackage = DomainPackage {
        id: "test.pair/v1",
        version: DomainVersion { major: 0, minor: 1 },
        profile: SourceProfile::AgeH0,
        hidden_prefix: "__pair_val_",
        compat: None,
        project: project_pair,
    };

    let mut registry = DomainRegistry::new();
    assert!(registry.register(&PAIR_PACKAGE).is_ok());
    assert!(
        registry.register(&SAME).is_ok(),
        "same id/version/profile is idempotent"
    );
    assert_eq!(registry.len(), 1);

    let conflict = registry
        .register(&CONFLICT)
        .expect_err("a same-id package with another profile is refused");
    assert_eq!(conflict.id, "test.pair/v1");
    assert_eq!(conflict.existing_profile, SourceProfile::AgeH0);
    assert_eq!(conflict.incoming_profile, SourceProfile::AgeComponent);
    // 旧注册保留：仍能取到原包并用它编译（旧程序不受影响）。
    assert_eq!(registry.len(), 1);
    let kept = registry.get("test.pair/v1").expect("old package survives");
    assert_eq!(kept.profile, SourceProfile::AgeH0);
    let out = compile_domain(kept, SRC, "pair.at", false, &[]).expect("old package still compiles");
    assert_eq!(out.id, "test.pair/v1");
}
