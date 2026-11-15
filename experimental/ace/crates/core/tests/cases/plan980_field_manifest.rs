//! PLAN-980（B-DIALECT-I/V，IF-980-FIELDS）：字段级投影完整性清单——语言侧永久 case。
//!
//! 覆盖：通用清单/覆盖机制的合法与破坏近邻；`source → candidate → 共享 verify →
//! 字段覆盖` 的真实链路（AGE 组件面；`widget` 兼容头与 `node` 规范头同一 ID 连接）；
//! 删必需行/换映射实现 → 红、恢复 → 绿的反控；AGE 面对未接线 widget 成员的具名拒绝
//! （不静默略过）。
//!
//! ID 权威：`widget.*` 字段 ID 的完整分母由 auto-ui 侧清单（PLAN-980 IF-980-FIELDS
//! 交接件，`ui.widget-decl/v1`）持有；本文件只持 AGE 组件面已实现子集的演示行，
//! ID 字符串与该清单一致（UI 侧 `p980_required_fields_handoff` 对账同一 ID 集）。

use ace_core::atom_text::Span;
use ace_core::source::age_component::{
    compile_component_scenario_observing, project_component_candidates, ComponentScenario,
    ScenarioOp,
};
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::domain::DomainVersion;
use ace_core::source::field_manifest::{
    component_field_ids, field_coverage, validate_field_manifest, ComponentFieldVocab,
    FieldApplicability, FieldManifest, FieldPresented, RequiredFieldRow,
};

/// AGE 组件面已实现槽位 → WidgetDecl 字段 ID（与 auto-ui 清单一致的冻结连接）。
fn widget_vocab() -> ComponentFieldVocab {
    ComponentFieldVocab {
        node_head: "widget.decl.head".to_string(),
        node_name: "widget.decl.name".to_string(),
        model_block: "widget.model.block".to_string(),
        model_field: "widget.model.field".to_string(),
        on_block: "widget.on.block".to_string(),
        on_handler: "widget.on.handler".to_string(),
        view_block: "widget.view.block".to_string(),
        text_leaf: "widget.view.node.text".to_string(),
    }
}

fn row(id: &str, applicability: FieldApplicability) -> RequiredFieldRow {
    RequiredFieldRow {
        id: id.to_string(),
        origin: format!("plan980_field_manifest test subset ({id})"),
        source_span: Span { start: 0, end: 0 },
        parent_role: id
            .rsplit_once('.')
            .map(|(p, _)| p.to_string())
            .unwrap_or_default(),
        type_rule: "AGE component face subset".to_string(),
        applicability,
        projection_path: "ComponentNode candidates".to_string(),
        check_id: "p980_component_field_chain".to_string(),
        consumer_owner: "980/DIALECT".to_string(),
    }
}

fn subset_manifest() -> FieldManifest {
    FieldManifest {
        domain: "ui.widget-decl".to_string(),
        version: DomainVersion { major: 1, minor: 0 },
        rows: vec![
            row(
                "widget.decl.head",
                FieldApplicability::Required {
                    mode: "every decl".into(),
                },
            ),
            row(
                "widget.decl.name",
                FieldApplicability::Required {
                    mode: "every decl".into(),
                },
            ),
            row(
                "widget.model.block",
                FieldApplicability::Required {
                    mode: "002-counter".into(),
                },
            ),
            row(
                "widget.model.field",
                FieldApplicability::Required {
                    mode: "002-counter".into(),
                },
            ),
            row(
                "widget.on.block",
                FieldApplicability::Required {
                    mode: "002-counter".into(),
                },
            ),
            row(
                "widget.on.handler",
                FieldApplicability::Required {
                    mode: "002-counter".into(),
                },
            ),
            row(
                "widget.view.block",
                FieldApplicability::Required {
                    mode: "every decl".into(),
                },
            ),
            row(
                "widget.view.node.text",
                FieldApplicability::Required {
                    mode: "002-counter".into(),
                },
            ),
        ],
    }
}

// ---- 通用机制：清单自检与覆盖判定 ----

#[test]
fn p980_field_manifest_validate_rejects_broken_rows() {
    let mut broken = subset_manifest();
    broken.rows.push(row(
        "widget.view.block",
        FieldApplicability::Required { mode: "dup".into() },
    ));
    broken.rows.push(RequiredFieldRow {
        id: "widget.bad.empty-origin".to_string(),
        origin: String::new(),
        source_span: Span { start: 9, end: 3 },
        parent_role: "widget".to_string(),
        type_rule: "t".to_string(),
        applicability: FieldApplicability::Conditional {
            mode: String::new(),
            reason: String::new(),
        },
        projection_path: "p".to_string(),
        check_id: "c".to_string(),
        consumer_owner: "o".to_string(),
    });
    let problems = validate_field_manifest(&broken).expect_err("broken rows must be rejected");
    let joined = problems.join("; ");
    assert!(joined.contains("duplicate id"), "{joined}");
    assert!(joined.contains("origin is empty"), "{joined}");
    assert!(joined.contains("source_span is not ordered"), "{joined}");
    assert!(joined.contains("empty mode"), "{joined}");
    assert!(joined.contains("verifiable reason"), "{joined}");

    validate_field_manifest(&subset_manifest()).expect("the frozen subset manifest is executable");
}

#[test]
fn p980_field_coverage_reports_missing_and_unknown() {
    let manifest = subset_manifest();
    let present = |ids: &[&str]| -> Vec<FieldPresented> {
        ids.iter()
            .map(|id| FieldPresented {
                id: id.to_string(),
                span: Span { start: 0, end: 0 },
            })
            .collect()
    };
    // 必需行缺席 → missing；未知 ID → unknown（两侧都红，不静默丢）。
    let cov = field_coverage(
        &present(&["widget.decl.head", "widget.bogus.slot"]),
        &manifest,
    );
    assert_eq!(cov.missing.len(), manifest.rows.len() - 1);
    assert_eq!(cov.unknown, vec!["widget.bogus.slot".to_string()]);
    assert!(!cov.is_complete());
    // 全槽位出现 → 闭合；重复出现不改判定。
    let full: Vec<&str> = manifest.rows.iter().map(|r| r.id.as_str()).collect();
    let cov = field_coverage(&present(&full), &manifest);
    assert!(cov.is_complete(), "{cov:?}");
    let doubled: Vec<&str> = full.iter().chain(full.iter()).copied().collect();
    assert!(field_coverage(&present(&doubled), &manifest).is_complete());
}

// ---- 真实链路：source → candidate → 共享 verify → 字段覆盖 ----

/// AGE 组件面合法形态（002-counter 的成员子集：model/view/on；`widget` 兼容头
/// 变体见别名 case；`msg`/`design` 等未接线成员的拒绝面见 unknown-members case）。
const NODE_SRC: &str = "node Counter {\n    model {\n        count int = 0\n    }\n    on Inc {\n        count = count + 1\n    }\n    view {\n        text \"x\" {}\n    }\n}\n";

#[test]
fn p980_component_field_chain_source_candidate_verify_coverage() {
    // 1) source → candidate（未受信投影，带原 span）。
    let candidates = project_component_candidates(NODE_SRC).expect("candidates project");
    assert_eq!(candidates.len(), 1);
    // 2) candidate → 共享 verify（场景编译把 model/on/text 交唯一 verify）。
    compile_component_scenario_observing(
        NODE_SRC,
        "plan980_field_chain.at",
        LegacyWidgetAlias::Reject,
        &[],
        &ComponentScenario {
            instances: 1,
            ops: vec![ScenarioOp::Send {
                instance: 0,
                msg: "Inc".to_string(),
            }],
        },
        None,
    )
    .expect("shared verify accepts the field-bearing source");
    // 3) candidate → 字段级出现（原 span）→ 覆盖闭合。
    let presented = component_field_ids(&candidates, &widget_vocab());
    for (id, needle) in [
        ("widget.decl.head", "node"),
        ("widget.decl.name", "Counter"),
        ("widget.model.field", "count int"),
        ("widget.on.handler", "on Inc"),
        ("widget.view.node.text", "text \"x\""),
    ] {
        let hit = presented
            .iter()
            .find(|p| p.id == id && NODE_SRC[p.span.start..p.span.end].contains(needle));
        assert!(
            hit.is_some(),
            "{id} must be presented with its original span covering {needle:?}: {presented:?}"
        );
    }
    let manifest = subset_manifest();
    let cov = field_coverage(&presented, &manifest);
    assert!(cov.is_complete(), "field coverage must close: {cov:?}");
    assert_eq!(cov.required.len(), 8);
}

#[test]
fn p980_component_field_coverage_delete_and_restore_controls() {
    let candidates = project_component_candidates(NODE_SRC).expect("candidates project");
    let presented = component_field_ids(&candidates, &widget_vocab());
    let manifest = subset_manifest();

    // 反控 A：删一条必需**清单行**——投影侧仍产出 text，清单已不认识它 → 该 ID
    // 进 unknown（红：删除清单行被对账抓住，不是静默缩表）；恢复 → 绿。
    let mut without_row = manifest.clone();
    without_row.rows.retain(|r| r.id != "widget.view.node.text");
    let cov = field_coverage(&presented, &without_row);
    assert!(cov.missing.is_empty(), "{cov:?}");
    assert_eq!(cov.unknown, vec!["widget.view.node.text".to_string()]);
    assert!(!cov.is_complete());
    assert!(
        field_coverage(&presented, &manifest).is_complete(),
        "restored row is green"
    );

    // 反控 B：映射实现删一项（text 叶的 ID 映射被换掉）→ 该槽 unknown + 原必需行
    // missing，两条红可区分；恢复同一入口后绿。
    let mut broken_vocab = widget_vocab();
    broken_vocab.text_leaf = "widget.bogus.leaf".to_string();
    let broken = component_field_ids(&candidates, &broken_vocab);
    let cov = field_coverage(&broken, &manifest);
    assert_eq!(cov.unknown, vec!["widget.bogus.leaf".to_string()]);
    assert_eq!(cov.missing, vec!["widget.view.node.text".to_string()]);
    assert!(!cov.is_complete());
    assert!(
        field_coverage(&presented, &manifest).is_complete(),
        "restored mapping is green"
    );
}

// ---- AGE 面对未接线 widget 成员：具名拒绝，不静默略过 ----

/// 含 `msg`/`design`（WidgetDecl 分母行存在、AGE 组件面未接线）的最小源。
const UNKNOWN_MEMBERS_SRC: &str =
    "node App {\n    msg { Go }\n    view {\n        text \"x\" {}\n    }\n}\n";

#[test]
fn p980_age_face_rejects_unknown_widget_decl_members() {
    let diags = project_component_candidates(UNKNOWN_MEMBERS_SRC)
        .expect_err("msg is not wired in the AGE component face; it must be rejected, not dropped");
    assert!(!diags.is_empty());
    // 具名拒绝必须指到未接线成员的原位置（msg 声明处），不是整文件兜底或静默空候选。
    let msg_at = UNKNOWN_MEMBERS_SRC.find("msg").expect("msg anchor");
    let pointed = diags.iter().any(|d| {
        d.span.start >= msg_at && d.span.start < UNKNOWN_MEMBERS_SRC.find("view").unwrap()
    });
    assert!(
        pointed,
        "a diagnostic must point at the `msg` member span: {diags:?}"
    );

    // design 成员同理（002-counter 的 design 块成员）。
    let design_src = "node App {\n    design {\n        theme: .t\n    }\n    view {\n        text \"x\" {}\n    }\n}\n";
    let diags = project_component_candidates(design_src)
        .expect_err("design is not wired in the AGE component face");
    let design_at = design_src.find("design").expect("design anchor");
    assert!(
        diags
            .iter()
            .any(|d| d.span.start >= design_at && d.span.start < design_src.find("view").unwrap()),
        "a diagnostic must point at the `design` member span: {diags:?}"
    );
}

// ---- `widget` 兼容头与 `node` 规范头：同一字段 ID 连接 ----

const WIDGET_SRC: &str = "widget Counter {\n    model {\n        count int = 0\n    }\n    view {\n        text \"x\" {}\n    }\n}\n";

#[test]
fn p980_age_face_legacy_widget_alias_head_join() {
    // 兼容别名关闭：compile 路径的兼容闸在原声明头 span 具名拒绝（不回落旧引擎）。
    let diags = compile_component_scenario_observing(
        WIDGET_SRC,
        "plan980_alias.at",
        LegacyWidgetAlias::Reject,
        &[],
        &ComponentScenario {
            instances: 1,
            ops: vec![],
        },
        None,
    )
    .expect_err("legacy head requires the compat config");
    assert!(
        diags.iter().any(|d| d.code == "source.age.legacy-widget"
            && WIDGET_SRC[d.span.start..d.span.end].contains("widget")),
        "named rejection at the head span: {diags:?}"
    );
    // 兼容别名开启：同一共享链路（含唯一 verify）接受旧声明头。
    let compiled = compile_component_scenario_observing(
        WIDGET_SRC,
        "plan980_alias.at",
        LegacyWidgetAlias::Accept,
        &[],
        &ComponentScenario {
            instances: 1,
            ops: vec![],
        },
        None,
    )
    .expect("compat alias accepts the legacy widget head");
    assert!(compiled.description.nodes[0].legacy_widget);
    // 字段 ID 连接：两种头拼写（候选投影不过闸）产出同一 ID 集——head 单行连接
    // `widget`（legacy 别名）与 `node`（规范拼写），拼写不换角色。
    let widget_candidates = project_component_candidates(WIDGET_SRC).expect("widget candidates");
    assert!(widget_candidates[0].legacy_widget);
    let node_src = WIDGET_SRC.replacen("widget", "node", 1);
    let node_candidates = project_component_candidates(&node_src).expect("node head projects");
    assert!(!node_candidates[0].legacy_widget);
    let widget_presented = component_field_ids(&widget_candidates, &widget_vocab());
    let node_presented = component_field_ids(&node_candidates, &widget_vocab());
    assert_eq!(
        widget_presented
            .iter()
            .map(|p| p.id.clone())
            .collect::<Vec<_>>(),
        node_presented
            .iter()
            .map(|p| p.id.clone())
            .collect::<Vec<_>>(),
        "both head spellings present the same field-id set"
    );
    // 本 case 不做 002-counter 模式的完整闭合（WIDGET_SRC 无 on 块；on 行按其
    // 模式在链路 case 的 002 形态源上闭合）；此处只核未知面为零。
    let cov = field_coverage(&widget_presented, &subset_manifest());
    assert!(cov.unknown.is_empty(), "no unknown ids: {cov:?}");
}
