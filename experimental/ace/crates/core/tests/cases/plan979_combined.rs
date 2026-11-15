//! PLAN-979 T-08：普通语言+领域组合相容、包冲突/版本、坏候选推进性与热换身份。
//!
//! V-08 断言（本文件逐条落地）：
//! - 同源同时含普通函数、旧 `widget`、新 `node`：各自按配置准入（别名关闭时旧头
//!   具名拒绝且不静默丢，新头/普通项不受影响）；
//! - 坏包/歧义（越界/重复绑定、空角色、过深结构）不死循环、不静默丢字段；
//! - 领域新增不改变无关普通标识符语义（`node`/`widget` 作普通名时字节同形）；
//! - 热换 schema/包后旧证据失效（身份元组变化）。

use ace_core::atom_text::Diagnostic;
use ace_core::source::age_h0::{compile_age_h0, LegacyWidgetAlias};
use ace_core::source::cst::FrontOk;
use ace_core::source::domain::{
    compile_domain, same_evidence, CompatRule, DomainBehavior, DomainCandidate, DomainItem,
    DomainPackage, DomainVersion,
};
use ace_core::source::SourceProfile;

const MIXED: &str = r#"fn helper(x int) str {
    if x > 0 {
        return "v"
    }
    return "w"
}
node A {
    view {
        text helper(1) {}
    }
}
widget B {
    view {
        text "legacy" {}
    }
}
"#;

const ORDINARY: &str = r#"fn node() int {
    return 1
}
fn widget() int {
    return 2
}
fn main() int {
    let a = node()
    let b = widget()
    return a + b
}
"#;

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

static PAIR_V1: DomainPackage = DomainPackage {
    id: "test.pair/v1",
    version: DomainVersion { major: 0, minor: 1 },
    profile: SourceProfile::AgeH0,
    hidden_prefix: "__pair_val_",
    compat: Some(CompatRule {
        spelling: "widget",
        disabled_code: "test.pair.compat",
        disabled_message: "pair compat",
    }),
    project: project_pair,
};

static PAIR_V2: DomainPackage = DomainPackage {
    id: "test.pair/v1",
    version: DomainVersion { major: 0, minor: 2 },
    profile: SourceProfile::AgeH0,
    hidden_prefix: "__pair_val_",
    compat: Some(CompatRule {
        spelling: "widget",
        disabled_code: "test.pair.compat",
        disabled_message: "pair compat",
    }),
    project: project_pair,
};

/// 坏投影：绑定下标越界。
static BAD_BINDING: DomainPackage = DomainPackage {
    id: "test.bad-binding/v1",
    version: DomainVersion { major: 0, minor: 1 },
    profile: SourceProfile::AgeH0,
    hidden_prefix: "__bad_",
    compat: None,
    project: project_bad_binding,
};

/// 坏投影：同一行为被两行绑定（歧义）。
static DUP_BINDING: DomainPackage = DomainPackage {
    id: "test.dup-binding/v1",
    version: DomainVersion { major: 0, minor: 1 },
    profile: SourceProfile::AgeH0,
    hidden_prefix: "__dup_",
    compat: None,
    project: project_dup_binding,
};

/// 坏投影：空角色。
static EMPTY_ROLE: DomainPackage = DomainPackage {
    id: "test.empty-role/v1",
    version: DomainVersion { major: 0, minor: 1 },
    profile: SourceProfile::AgeH0,
    hidden_prefix: "__empty_",
    compat: None,
    project: project_empty_role,
};

/// 坏投影：过深结构（>64）。
static DEEP: DomainPackage = DomainPackage {
    id: "test.deep/v1",
    version: DomainVersion { major: 0, minor: 1 },
    profile: SourceProfile::AgeH0,
    hidden_prefix: "__deep_",
    compat: None,
    project: project_deep,
};

fn one_card(role: &str, bindings: Vec<usize>) -> DomainItem {
    DomainItem {
        role: role.to_string(),
        name: None,
        span: ace_core::atom_text::Span { start: 0, end: 1 },
        aux_span: None,
        spelling: None,
        bindings,
        children: Vec::new(),
    }
}

fn project_bad_binding(_ok: &FrontOk, _src: &str) -> Result<DomainCandidate, Vec<Diagnostic>> {
    Ok(DomainCandidate {
        structure: vec![one_card("row", vec![0])],
        behaviors: vec![],
        requirements: vec![],
    })
}

fn project_dup_binding(_ok: &FrontOk, _src: &str) -> Result<DomainCandidate, Vec<Diagnostic>> {
    Ok(DomainCandidate {
        structure: vec![one_card("a", vec![0]), one_card("b", vec![0])],
        behaviors: vec![],
        requirements: vec![],
    })
}

fn project_empty_role(_ok: &FrontOk, _src: &str) -> Result<DomainCandidate, Vec<Diagnostic>> {
    Ok(DomainCandidate {
        structure: vec![one_card("", vec![])],
        behaviors: vec![],
        requirements: vec![],
    })
}

fn project_deep(_ok: &FrontOk, _src: &str) -> Result<DomainCandidate, Vec<Diagnostic>> {
    let mut item = one_card("leaf", vec![]);
    for _ in 0..80 {
        item = DomainItem {
            role: "wrap".to_string(),
            name: None,
            span: ace_core::atom_text::Span { start: 0, end: 1 },
            aux_span: None,
            spelling: None,
            bindings: vec![],
            children: vec![item],
        };
    }
    Ok(DomainCandidate {
        structure: vec![item],
        behaviors: vec![],
        requirements: vec![],
    })
}

fn first(ds: &[Diagnostic]) -> &Diagnostic {
    ds.first().expect("at least one diagnostic")
}

#[test]
fn p979_combined_mixed_source_admits_each_construct_by_config() {
    // 别名开：同一份源里普通函数 + 新 `node` + 旧 `widget` 都准入，原 span 保留。
    let on = compile_age_h0(MIXED, "mixed.at", LegacyWidgetAlias::Accept, &[]).expect("compat on");
    assert_eq!(on.description.nodes.len(), 2);
    assert_eq!(on.description.nodes[0].name, "A");
    assert!(!on.description.nodes[0].legacy_widget);
    assert_eq!(on.description.nodes[1].name, "B");
    assert!(on.description.nodes[1].legacy_widget);
    let widget_at = MIXED.find("widget B").expect("widget decl");
    assert_eq!(on.description.nodes[1].head_span.start, widget_at);
    // 普通函数仍在：其文本叶求值来自 helper(1)。
    assert_eq!(
        on.description.nodes[0].view.texts[0].kind,
        ace_core::source::age_h0::TextKind::View
    );

    // 别名关：旧头在**自己的**声明头具名拒绝（不是整份源一起丢）。
    let err = compile_age_h0(MIXED, "mixed.at", LegacyWidgetAlias::Reject, &[])
        .expect_err("compat off rejects the legacy head");
    assert_eq!(first(&err).code, "source.age.legacy-widget");
    assert_eq!(first(&err).span.start, widget_at);

    // 只含新 `node` 的同形源在别名关闭时照常准入（新头不被旧兼容连坐）。
    let node_only = MIXED.replace("widget B", "node B");
    let off = compile_age_h0(&node_only, "mixed.at", LegacyWidgetAlias::Reject, &[])
        .expect("no legacy head");
    assert_eq!(off.description.nodes.len(), 2);
}

#[test]
fn p979_combined_domain_does_not_change_ordinary_identifier_semantics() {
    // `node`/`widget` 作普通函数名：AGE 前门与 plain text-record 前门字节同形。
    let age = compile_age_h0(ORDINARY, "ordinary.at", LegacyWidgetAlias::Accept, &[])
        .expect("ordinary module on the age front door");
    let plain = ace_core::source::compile_text_record_source(ORDINARY, "ordinary.at", &[])
        .expect("plain text-record");
    assert_eq!(age.atom_text, plain.atom_text);
    assert_eq!(age.input_hash, plain.input_hash);
    assert!(age.description.nodes.is_empty(), "no domain declarations");
}

#[test]
fn p979_combined_bad_package_candidates_fail_closed() {
    for (pkg, why) in [
        (&BAD_BINDING, "out of range binding"),
        (&DUP_BINDING, "duplicate binding"),
        (&EMPTY_ROLE, "empty role"),
        (&DEEP, "too deep"),
    ] {
        match compile_domain(pkg, MIXED, "bad.at", false, &[]) {
            Ok(_) => panic!("{why}: expected a named rejection"),
            Err(ds) => assert_eq!(first(&ds).code, "source.domain-candidate", "{why}: {ds:?}"),
        }
    }
}

#[test]
fn p979_combined_hot_swap_invalidates_old_evidence() {
    // 同一 grammar、不同包版本 → 身份不同（旧证据失效）。
    let v1 = compile_domain(&PAIR_V1, MIXED, "pair.at", true, &[]).expect("v1");
    let v2 = compile_domain(&PAIR_V2, MIXED, "pair.at", true, &[]).expect("v2");
    let i1 = v1.identity();
    let i2 = v2.identity();
    assert_eq!(i1.grammar_id, i2.grammar_id, "same grammar trim");
    assert!(
        !same_evidence(&i1, &i2),
        "version change invalidates evidence"
    );
    assert!(same_evidence(&i1, &v1.identity()), "same input is stable");
    // 换包（AGE vs pair）在同一 grammar 上同样失效。
    let age = compile_age_h0(MIXED, "mixed.at", LegacyWidgetAlias::Accept, &[]).expect("age");
    let age_identity = ace_core::source::domain::DomainIdentity {
        id: ae_id(),
        version: DomainVersion { major: 1, minor: 0 },
        grammar_id: age.description.grammar_id.clone(),
        input_hash: age.input_hash.clone(),
        compat: true,
    };
    assert!(
        !same_evidence(&i1, &age_identity),
        "package swap invalidates evidence"
    );
}

fn ae_id() -> &'static str {
    ace_core::source::age_h0::AGE_H0_DOMAIN
}

/// 投影省略 `view` 行（删除一条必需映射的坏近邻）。
static DROPS_VIEW: DomainPackage = DomainPackage {
    id: "test.drops-view/v1",
    version: DomainVersion { major: 0, minor: 1 },
    profile: SourceProfile::AgeH0,
    hidden_prefix: "__drop_",
    compat: None,
    project: project_drops_view,
};

fn project_drops_view(ok: &FrontOk, _src: &str) -> Result<DomainCandidate, Vec<Diagnostic>> {
    // 直接挂 text 行到 node 行：投影里没有 `view` 这一层。
    let mut structure = Vec::new();
    let mut behaviors = Vec::new();
    for n in &ok.age_nodes {
        let mut texts = Vec::new();
        for leaf in &n.view.texts {
            let idx = behaviors.len();
            behaviors.push(DomainBehavior {
                expr: leaf.expr.clone(),
                span: leaf.expr.span,
            });
            texts.push(DomainItem {
                role: "text".to_string(),
                name: None,
                span: leaf.span,
                aux_span: Some(leaf.component_span),
                spelling: None,
                bindings: vec![idx],
                children: Vec::new(),
            });
        }
        structure.push(DomainItem {
            role: "node".to_string(),
            name: Some(n.name.name.clone()),
            span: n.span,
            aux_span: Some(n.head_span),
            spelling: Some(n.head_spelling.clone()),
            bindings: Vec::new(),
            children: texts,
        });
    }
    Ok(DomainCandidate {
        structure,
        behaviors,
        requirements: vec![],
    })
}

const NODE_ONLY: &str = "node Hello {\n    view {\n        text \"Hello\" {}\n    }\n}\n";

#[test]
fn p979_combined_mapping_completeness_check_goes_red_when_a_required_row_is_dropped() {
    use ace_core::source::domain::mapping_complete;
    // 规范 AGE 投影：node/view/text 三层齐。
    let age = compile_age_h0(NODE_ONLY, "m.at", LegacyWidgetAlias::Reject, &[]).expect("age");
    let rows = age_rows(&age);
    assert_eq!(mapping_complete(&rows, &["node", "view", "text"]), Ok(()));
    // 删除 `view` 映射的坏近邻：完整性检查红并指出缺项。
    let dropped =
        compile_domain(&DROPS_VIEW, NODE_ONLY, "m.at", false, &[]).expect("drops compiles");
    assert_eq!(
        mapping_complete(&dropped.description, &["node", "view", "text"]),
        Err(vec!["view".to_string()])
    );
    // 合法近邻（多出 role）仍接受。
    assert_eq!(
        mapping_complete(&dropped.description, &["node", "text"]),
        Ok(())
    );
}

/// AGE 描述 → 通用行（测试内适配器；不引入第二套投影）。
fn age_rows(
    age: &ace_core::source::age_h0::AgeH0Compilation,
) -> Vec<ace_core::source::domain::DomainRow> {
    use ace_core::source::domain::{DomainRow, ResolvedBinding};
    age.description
        .nodes
        .iter()
        .map(|n| DomainRow {
            role: "node".to_string(),
            name: Some(n.name.clone()),
            span: n.span,
            aux_span: Some(n.head_span),
            spelling: Some(n.head_role.clone()),
            compat: n.legacy_widget,
            bindings: Vec::new(),
            children: vec![DomainRow {
                role: n.view.role.clone(),
                name: None,
                span: n.view.span,
                aux_span: None,
                spelling: None,
                compat: false,
                bindings: Vec::new(),
                children: n
                    .view
                    .texts
                    .iter()
                    .map(|t| DomainRow {
                        role: t.role.clone(),
                        name: None,
                        span: t.span,
                        aux_span: Some(t.component_span),
                        spelling: None,
                        compat: false,
                        bindings: vec![ResolvedBinding {
                            hidden_fn: t.fn_name.clone(),
                            def_id: t.def_id.clone(),
                            kind: t.kind,
                        }],
                        children: Vec::new(),
                    })
                    .collect(),
            }],
        })
        .collect()
}
