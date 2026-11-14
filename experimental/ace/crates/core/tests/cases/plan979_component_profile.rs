//! PLAN-979 T-03：`age-component` profile 的接线面（H0 面不动的证明）。
//!
//! 本步只验证 profile 身份与入口闭合：裁剪程序可编译且与 `age-h0` 的 GrammarId
//! 隔离；所有通用入口（含片段）具名拒绝；age-h0 的既有行为不受影响。

use ace_core::source::cst::{
    parse_fragment_front, parse_front, parse_program_via_cst, profile_grammar, FrontBudget,
    FrontKind, FrontOutcome, FrontRequest,
};
use ace_core::source::SourceProfile;

const SRC: &str = "node Counter {\n    model {\n        count int = 0\n    }\n    view {\n        text \"x\" {}\n    }\n}\n";

#[test]
fn p979_component_profile_has_its_own_trimmed_grammar() {
    let component = profile_grammar(SourceProfile::AgeComponent).expect("component trim compiles");
    let h0 = profile_grammar(SourceProfile::AgeH0).expect("h0 trim compiles");
    // 组件面包含 H0 的规则 + 组件成员规则；H0 面不含组件规则。
    let component_rules: Vec<&str> = component
        .program()
        .rules
        .iter()
        .map(|r| r.name.as_str())
        .collect();
    let h0_rules: Vec<&str> = h0.program().rules.iter().map(|r| r.name.as_str()).collect();
    for rule in [
        "age-node-c-item",
        "age-model-member",
        "age-on-member",
        "age-view-member",
    ] {
        assert!(component_rules.contains(&rule), "component trim has {rule}");
        assert!(!h0_rules.contains(&rule), "h0 trim must not grow {rule}");
    }
    assert!(h0_rules.contains(&"age-node-item"));
    // 两个裁剪面的 identity 不同（同一 grammar 源、不同 profile）。
    assert_ne!(
        format!("{:?}", component.program().identity),
        format!("{:?}", h0.program().identity)
    );
}

#[test]
fn p979_component_profile_is_refused_by_generic_entries() {
    match parse_front(FrontRequest {
        src: SRC,
        profile: SourceProfile::AgeComponent,
        kind: FrontKind::WholeModule,
        budget: FrontBudget::WORK_OWNED_64MIB,
    }) {
        FrontOutcome::Diag(ds) => assert_eq!(ds[0].code, "source.domain-frontend"),
        other => panic!("expected the domain refusal, got {other:?}"),
    }
    let e = parse_program_via_cst(SRC, SourceProfile::AgeComponent)
        .expect_err("bridge refuses the component profile");
    assert_eq!(e.code, "source.domain-frontend");
    match parse_fragment_front(
        SRC,
        SourceProfile::AgeComponent,
        FrontBudget::WORK_OWNED_64MIB,
        &(0..SRC.len()),
        None,
        ace_core::source::ast::Receiver::Free,
    ) {
        FrontOutcome::Diag(ds) => assert_eq!(ds[0].code, "source.domain-frontend"),
        other => panic!("fragment entry refuses the component profile, got {other:?}"),
    }
}

// ---- T-03 组件投影（view 面）----

use ace_core::source::age_component::{
    compile_age_component, project_component_candidates, AGE_COMPONENT_DOMAIN,
};
use ace_core::source::age_h0::LegacyWidgetAlias;

const VIEW_ONLY: &str = "node Hello {\n    view {\n        text \"Hello\" {}\n    }\n}\n";

#[test]
fn p979_component_projection_yields_typed_candidates() {
    let nodes = project_component_candidates(VIEW_ONLY).expect("projection");
    assert_eq!(nodes.len(), 1);
    let n = &nodes[0];
    assert_eq!(n.head_role, "node");
    assert_eq!(n.head_spelling, "node");
    assert_eq!(n.name.name, "Hello");
    assert_eq!(n.view.role, "view");
    assert_eq!(n.view.texts.len(), 1);
    assert_eq!(n.view.texts[0].role, "text");
    assert!(n.fields.is_empty() && n.handlers.is_empty());
}

#[test]
fn p979_component_view_only_source_compiles_through_the_shared_verify() {
    let out = compile_age_component(VIEW_ONLY, "component.at", LegacyWidgetAlias::Reject, &[])
        .expect("view-only component compiles");
    assert_eq!(out.description.domain, AGE_COMPONENT_DOMAIN);
    assert_eq!(out.description.nodes.len(), 1);
    let n = &out.description.nodes[0];
    assert_eq!(n.view.role, "view");
    assert_eq!(n.view.texts[0].fn_name, "__age_c_text_0");
    assert_eq!(n.view.texts[0].def_id, "d___age_c_text_0");
}

#[test]
fn p979_component_compat_and_prefix_rules_apply() {
    let widget = "widget App {\n    view {\n        text \"hi\" {}\n    }\n}\n";
    let ds = compile_age_component(widget, "component.at", LegacyWidgetAlias::Reject, &[])
        .expect_err("compat off rejects the legacy head");
    assert_eq!(ds[0].code, "source.age.legacy-widget");
    let on = compile_age_component(widget, "component.at", LegacyWidgetAlias::Accept, &[])
        .expect("compat on admits the legacy head");
    assert!(on.description.nodes[0].legacy_widget);
    let collide =
        format!("fn x__age_c_text_0() Str {{\n    return \"HACK\".clone()\n}}\n{VIEW_ONLY}");
    let ds = compile_age_component(&collide, "component.at", LegacyWidgetAlias::Reject, &[])
        .expect_err("reserved prefix family is a named conflict");
    assert_eq!(ds[0].code, "source.age.hidden-name");
}

#[test]
fn p979_component_probe_member_shapes() {
    let cases = [
        ("model_only", "node C {\n    model {\n        count int = 0\n    }\n}\n"),
        ("view_then_model", "node C {\n    view {\n        text \"x\" {}\n    }\n    model {\n        count int = 0\n    }\n}\n"),
        ("model_then_view", "node C {\n    model {\n        count int = 0\n    }\n    view {\n        text \"x\" {}\n    }\n}\n"),
        ("model_assign_only", "node C {\n    model {\n        count = 0\n    }\n}\n"),
        ("on_only", "node C {\n    on Inc {\n        count = 1\n    }\n}\n"),
        ("model_two", "node C {\n    model {\n        a int = 0\n        b int = 1\n    }\n}\n"),
    ];
    for (name, src) in cases {
        match project_component_candidates(src) {
            Ok(ns) => eprintln!(
                "PROBE {name}: ok nodes={} fields={} handlers={}",
                ns.len(),
                ns.first().map(|n| n.fields.len()).unwrap_or(0),
                ns.first().map(|n| n.handlers.len()).unwrap_or(0)
            ),
            Err(ds) => eprintln!(
                "PROBE {name}: err {} @{}..{} {}",
                ds[0].code, ds[0].span.start, ds[0].span.end, ds[0].message
            ),
        }
    }
}
#[test]
fn p979_component_model_and_on_members_project_typed_candidates() {
    let model = "node Counter {\n    model {\n        count int = 0\n        label str = \"x\"\n    }\n    view {\n        text \"x\" {}\n    }\n}\n";
    let nodes = project_component_candidates(model).expect("model projection");
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].fields.len(), 2);
    assert_eq!(nodes[0].fields[0].name, "count");
    assert_eq!(nodes[0].fields[0].ty, "int");
    assert_eq!(nodes[0].fields[1].name, "label");
    assert_eq!(nodes[0].fields[1].ty, "str");
    assert_eq!(nodes[0].view.texts.len(), 1);
    // 原 span：字段与初值都指向源文件。
    assert!(nodes[0].fields[0].span.start < nodes[0].fields[0].init.span.start);

    let on = "node Counter {\n    on Inc {\n        count = count + 1\n    }\n    view {\n        text \"x\" {}\n    }\n}\n";
    let nodes = project_component_candidates(on).expect("on projection");
    assert_eq!(nodes[0].handlers.len(), 1);
    assert_eq!(nodes[0].handlers[0].msg, "Inc");
    assert_eq!(nodes[0].handlers[0].assigns.len(), 1);
    assert_eq!(nodes[0].handlers[0].assigns[0].field, "count");
}

#[test]
fn p979_component_model_and_on_still_fail_closed_until_the_runtime_lands() {
    let model = "node Counter {\n    model {\n        count int = 0\n    }\n    view {\n        text \"x\" {}\n    }\n}\n";
    let ds = compile_age_component(model, "component.at", LegacyWidgetAlias::Reject, &[])
        .expect_err("model runtime is not wired yet");
    assert_eq!(ds[0].code, "source.age.component-unsupported");
    assert!(ds[0].message.contains("model"), "{:?}", ds[0]);
    assert_eq!(
        ds[0].span.start,
        model.find("count int").expect("first model field")
    );

    let on = "node Counter {\n    on Inc {\n        count = 1\n    }\n    view {\n        text \"x\" {}\n    }\n}\n";
    let ds = compile_age_component(on, "component.at", LegacyWidgetAlias::Reject, &[])
        .expect_err("event runtime is not wired yet");
    assert_eq!(ds[0].code, "source.age.component-unsupported");
    assert!(ds[0].message.contains("on Inc"), "{:?}", ds[0]);
}

// ---- T-03 场景驱动（model/on 脱糖为普通 Auto 驱动）----

use ace_core::source::age_component::{compile_component_scenario, ComponentScenario, ScenarioOp};

const COUNTER: &str = "node Counter {\n    model {\n        count int = 0\n    }\n    on Inc {\n        count = count + 1\n    }\n    on Tick {\n        count = count + 1\n    }\n    view {\n        text \"c\" {}\n    }\n}\n";

#[test]
fn p979_component_scenario_desugars_to_a_verified_driver() {
    let scenario = ComponentScenario {
        instances: 2,
        ops: vec![
            ScenarioOp::Send {
                instance: 1,
                msg: "Inc".to_string(),
            },
            ScenarioOp::Send {
                instance: 0,
                msg: "Inc".to_string(),
            },
            ScenarioOp::Send {
                instance: 0,
                msg: "Inc".to_string(),
            },
            ScenarioOp::Tick { instance: 1 },
        ],
    };
    let out = compile_component_scenario(
        &COUNTER,
        "counter.at",
        LegacyWidgetAlias::Reject,
        &[],
        &scenario,
    )
    .expect("scenario compiles through the shared verify");
    // 驱动是普通 i32 函数，位于共享 Checked HIR（名字由 adapter 唯一决定）。
    let atom = &out.atom_text;
    assert!(atom.contains("__age_drive"), "driver in the checked module");
    assert!(
        atom.contains("__age_h0_text_") || atom.contains("__age_c_text_0"),
        "text evaluator present"
    );
    let n = &out.description.nodes[0];
    assert_eq!(n.fields.len(), 1);
    assert_eq!(n.handler_msgs, vec!["Inc".to_string(), "Tick".to_string()]);
    assert_eq!(n.view.texts.len(), 1);
}

#[test]
fn p979_component_scenario_rejects_undeclared_events_and_instances() {
    let scenario = ComponentScenario {
        instances: 2,
        ops: vec![ScenarioOp::Send {
            instance: 0,
            msg: "Nope".to_string(),
        }],
    };
    let ds = compile_component_scenario(
        &COUNTER,
        "counter.at",
        LegacyWidgetAlias::Reject,
        &[],
        &scenario,
    )
    .expect_err("undeclared event");
    assert_eq!(ds[0].code, "source.age.component-event");

    let scenario = ComponentScenario {
        instances: 1,
        ops: vec![ScenarioOp::Send {
            instance: 5,
            msg: "Inc".to_string(),
        }],
    };
    let ds = compile_component_scenario(
        &COUNTER,
        "counter.at",
        LegacyWidgetAlias::Reject,
        &[],
        &scenario,
    )
    .expect_err("out-of-range instance");
    assert_eq!(ds[0].code, "source.age.component-scenario");
}
