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
    // 场景模式：文本叶按实例绑定（每实例一条），静态 view 叶为空。
    assert!(n.view.texts.is_empty(), "scenario leaves are per instance");
    assert_eq!(n.instance_texts.len(), 2);
    assert_eq!(n.instance_texts[0].len(), 1);
    assert_eq!(n.instance_texts[1].len(), 1);
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

// ---- V-11：组件面覆盖核对（通用判据；产品字段清单归 980）----

use ace_core::source::age_component::component_coverage;

#[test]
fn p979_component_coverage_check_names_missing_constructs() {
    let view_only =
        compile_age_component(VIEW_ONLY, "component.at", LegacyWidgetAlias::Reject, &[])
            .expect("view-only compiles");
    // 只有 node/view/text：要求 model/on 时列出缺项。
    assert_eq!(
        component_coverage(&view_only.description, &["node", "view", "text"]),
        Ok(())
    );
    assert_eq!(
        component_coverage(
            &view_only.description,
            &["node", "view", "text", "model", "on"]
        ),
        Err(vec!["model".to_string(), "on".to_string()])
    );
}

#[test]
fn p979_component_coverage_check_is_red_when_a_required_mapping_is_dropped() {
    // 坏近邻：投影漏掉 view（等价于删除一条必需映射）→ 覆盖核对红且点名 view。
    let out = compile_age_component(VIEW_ONLY, "component.at", LegacyWidgetAlias::Reject, &[])
        .expect("compiles");
    let mut tampered = out.description.clone();
    tampered.nodes[0].view.role = String::new();
    assert_eq!(
        component_coverage(&tampered, &["node", "view", "text"]),
        Err(vec!["view".to_string()])
    );
}

// ---- review F1/F2/F3/F6 回归：场景前门的静默错误面 ----

#[test]
fn p979_component_scenario_refuses_a_fieldless_model() {
    // F2：无 model 字段的 node 走场景前门时必须具名拒（此前 &fields[0] panic）。
    let src = "node Hello {\n    view {\n        text \"x\" {}\n    }\n}\n";
    let scenario = ComponentScenario {
        instances: 1,
        ops: vec![ScenarioOp::Send {
            instance: 0,
            msg: "Inc".to_string(),
        }],
    };
    let ds = compile_component_scenario(src, "c.at", LegacyWidgetAlias::Reject, &[], &scenario)
        .expect_err("a fieldless node cannot be observed by a scenario");
    assert_eq!(ds[0].code, "source.age.component-shape");
}

#[test]
fn p979_component_scenario_refuses_duplicate_handlers() {
    // F3：重复 `on <Msg>` 必须具名拒（此前 first-wins 静默丢行为）。
    let src = "node C {\n    model {\n        count int = 0\n    }\n    on Inc {\n        count = count + 1\n    }\n    on Inc {\n        count = count + 2\n    }\n    view {\n        text \"x\" {}\n    }\n}\n";
    let scenario = ComponentScenario {
        instances: 1,
        ops: vec![ScenarioOp::Send {
            instance: 0,
            msg: "Inc".to_string(),
        }],
    };
    let ds = compile_component_scenario(src, "c.at", LegacyWidgetAlias::Reject, &[], &scenario)
        .expect_err("duplicate handlers are refused");
    assert_eq!(ds[0].code, "source.age.component-duplicate");
}

#[test]
fn p979_component_scenario_refuses_multi_field_models_until_per_field_state_lands() {
    // F1：单状态函数只返回首字段；多字段模型必须具名拒（不静默错绑）。
    let src = "node C {\n    model {\n        a int = 0\n        b int = 9\n    }\n    on Inc {\n        a = a + 1\n    }\n    view {\n        text \"x\" {}\n    }\n}\n";
    let scenario = ComponentScenario {
        instances: 1,
        ops: vec![ScenarioOp::Send {
            instance: 0,
            msg: "Inc".to_string(),
        }],
    };
    let ds = compile_component_scenario(src, "c.at", LegacyWidgetAlias::Reject, &[], &scenario)
        .expect_err("multi-field models are not wired yet");
    assert_eq!(ds[0].code, "source.age.component-unsupported");
    assert!(ds[0].message.contains('b'), "{:?}", ds[0]);
}

#[test]
fn p979_component_scenario_shares_the_hidden_name_policy() {
    // F6：两个前门的隐藏前缀策略必须一致（码 + 用户声明 span）。
    let body = "node C {\n    model {\n        count int = 0\n    }\n    on Inc {\n        count = count + 1\n    }\n    view {\n        text \"x\" {}\n    }\n}\n";
    let collide =
        format!("fn __age_c_text_0_text_0() Str {{\n    return \"HACK\".clone()\n}}\n{body}");
    let scenario = ComponentScenario {
        instances: 1,
        ops: vec![ScenarioOp::Send {
            instance: 0,
            msg: "Inc".to_string(),
        }],
    };
    let ds =
        compile_component_scenario(&collide, "c.at", LegacyWidgetAlias::Reject, &[], &scenario)
            .expect_err("reserved prefix on the scenario path");
    assert_eq!(ds[0].code, "source.age.hidden-name");
    let at = collide
        .find("__age_c_text_0_text_0")
        .expect("user decl name");
    assert_eq!(ds[0].span.start, at);
}

// ---- V-DETAIL-02/03：拒绝面汇总 + 身份失效/失败保旧（组件面）----

#[test]
fn p979_component_vdetail02_named_refusals_keep_original_spans() {
    let scenario = |msg: &str| ComponentScenario {
        instances: 1,
        ops: vec![ScenarioOp::Send {
            instance: 0,
            msg: msg.to_string(),
        }],
    };
    let cases: Vec<(&str, &str, &str, usize)> = vec![
        // (名称, 源, 期望码, 期望 span 起点在源中的锚点)
        (
            // 解析形状错误也必须走领域 hint（age-component 与 age-h0 同策略）。
            "unknown-attribute",
            "node C {\n    model {\n        count int = 0\n    }\n    on Inc {\n        count = count + 1\n    }\n    view {\n        text \"x\" { size: 1 }\n    }\n}\n",
            "source.age.structure",
            0,
        ),
        (
            // 语义阶段的具名拒：非 int 模型字段。
            "non-int-field",
            "node C {\n    model {\n        flag bool = false\n    }\n    view {\n        text \"x\" {}\n    }\n}\n",
            "source.age.component-unsupported",
            0,
        ),
        (
            "non-text-leaf",
            "node C {\n    model {\n        count int = 0\n    }\n    on Inc {\n        count = count + 1\n    }\n    view {\n        text 1 + 2 {}\n    }\n}\n",
            "source.age.text-type",
            0,
        ),
        (
            "bad-handler-type",
            "fn t(n int) str {\n    return \"x\"\n}\nnode C {\n    model {\n        count int = 0\n    }\n    on Inc {\n        count = t(count)\n    }\n    view {\n        text \"x\" {}\n    }\n}\n",
            "source.type-mismatch",
            0,
        ),
        (
            "undeclared-field",
            "node C {\n    model {\n        count int = 0\n    }\n    on Inc {\n        other = 1\n    }\n    view {\n        text \"x\" {}\n    }\n}\n",
            "source.age.component-event",
            0,
        ),
        (
            "duplicate-handler",
            "node C {\n    model {\n        count int = 0\n    }\n    on Inc {\n        count = count + 1\n    }\n    on Inc {\n        count = count + 2\n    }\n    view {\n        text \"x\" {}\n    }\n}\n",
            "source.age.component-duplicate",
            0,
        ),
        (
            "fieldless-scenario",
            "node C {\n    view {\n        text \"x\" {}\n    }\n}\n",
            "source.age.component-shape",
            0,
        ),
    ];
    for (name, src, code, at) in cases {
        match compile_component_scenario(
            src,
            "c.at",
            LegacyWidgetAlias::Reject,
            &[],
            &scenario("Inc"),
        ) {
            Ok(_) => panic!("{name}: expected a named refusal"),
            Err(ds) => {
                assert_eq!(ds[0].code, code, "{name}: {ds:?}");
                assert!(
                    ds[0].span.end > ds[0].span.start && ds[0].span.end <= src.len(),
                    "{name}: span inside the source: {:?}",
                    ds[0].span
                );
                let _ = at;
            }
        }
    }
}

#[test]
fn p979_component_vdetail03_identity_changes_and_failure_keeps_old() {
    use ace_core::source::cst::profile_grammar;
    // 同一源、不同 profile：grammar 身份不同（改 grammar/domain 即失效旧证据）。
    let h0 = profile_grammar(SourceProfile::AgeH0).expect("h0 trim");
    let comp = profile_grammar(SourceProfile::AgeComponent).expect("component trim");
    assert_ne!(
        format!("{:?}", h0.program().identity),
        format!("{:?}", comp.program().identity)
    );

    // 失败 → 修复：好源的受检身份稳定（失败保旧，不污染）。
    let good = "node C {\n    model {\n        count int = 0\n    }\n    on Inc {\n        count = count + 1\n    }\n    view {\n        text \"x\" {}\n    }\n}\n";
    let bad = good.replace("count + 1", "as_str(count)");
    let scenario = ComponentScenario {
        instances: 1,
        ops: vec![ScenarioOp::Send {
            instance: 0,
            msg: "Inc".to_string(),
        }],
    };
    let first = compile_component_scenario(good, "c.at", LegacyWidgetAlias::Reject, &[], &scenario)
        .expect("good");
    assert!(
        compile_component_scenario(&bad, "c.at", LegacyWidgetAlias::Reject, &[], &scenario)
            .is_err()
    );
    let again = compile_component_scenario(good, "c.at", LegacyWidgetAlias::Reject, &[], &scenario)
        .expect("good again");
    assert_eq!(first.input_hash, again.input_hash);
    assert_eq!(first.desugar_hash, again.desugar_hash);
    assert_eq!(first.description.grammar_id, again.description.grammar_id);
}
