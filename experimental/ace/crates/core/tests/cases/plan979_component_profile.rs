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
fn p979_component_scenario_observes_each_field_independently() {
    // F1 收口：多字段模型由 per-field 状态函数正确绑定（此前只返回首字段、会错绑）。
    let src = "node C {\n    model {\n        a int = 0\n        b int = 9\n    }\n    on Inc {\n        a = a + 1\n    }\n    view {\n        text \"x\" {}\n    }\n}\n";
    let scenario = ComponentScenario {
        instances: 1,
        ops: vec![ScenarioOp::Send {
            instance: 0,
            msg: "Inc".to_string(),
        }],
    };
    let out = compile_component_scenario(src, "c.at", LegacyWidgetAlias::Reject, &[], &scenario)
        .expect("multi-field models compile now");
    // 每个字段都有自己的状态函数（`__age_state_<inst>_<field>`）。
    assert!(
        out.atom_text.contains("__age_state_0_a"),
        "field a has its own state fn"
    );
    assert!(
        out.atom_text.contains("__age_state_0_b"),
        "field b has its own state fn"
    );
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
            // PLAN-980（B-DIALECT-I 完整批，2026-11-16）：`text <label> { 属性行 }`
            //（001-helloworld 的 selectable/style 形）已由 980 接线为合法
            // age-text-props（见 plan980_dialect_full 的 001 腿）；本反例改用
            // 仍然非法的"无冒号属性"形，具名结构拒绝合同不变。
            "unknown-attribute",
            "node C {\n    model {\n        count int = 0\n    }\n    on Inc {\n        count = count + 1\n    }\n    view {\n        text \"x\" { size }\n    }\n}\n",
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

// ---- V-04：只失效实际依赖（源/场景/profile/兼容配置各看各的）----

#[test]
fn p979_component_v04_invalidation_tracks_the_actual_dependency() {
    let src = "node C {\n    model {\n        count int = 0\n    }\n    on Inc {\n        count = count + 1\n    }\n    view {\n        text \"x\" {}\n    }\n}\n";
    let one = ComponentScenario {
        instances: 1,
        ops: vec![ScenarioOp::Send {
            instance: 0,
            msg: "Inc".to_string(),
        }],
    };
    let two = ComponentScenario {
        instances: 1,
        ops: vec![
            ScenarioOp::Send {
                instance: 0,
                msg: "Inc".to_string(),
            },
            ScenarioOp::Send {
                instance: 0,
                msg: "Inc".to_string(),
            },
        ],
    };
    let base = compile_component_scenario(src, "c.at", LegacyWidgetAlias::Reject, &[], &one)
        .expect("base");

    // ① 源变 → 源身份与脱糖身份都变。
    let src2 = format!("fn extra() int {{\n    return 1\n}}\n{src}");
    let changed_src =
        compile_component_scenario(&src2, "c.at", LegacyWidgetAlias::Reject, &[], &one)
            .expect("changed source");
    assert_ne!(base.input_hash, changed_src.input_hash);
    assert_ne!(base.desugar_hash, changed_src.desugar_hash);

    // ② 场景变 → 源身份不变、脱糖身份变（只失效受影响的依赖）。
    let changed_scenario =
        compile_component_scenario(src, "c.at", LegacyWidgetAlias::Reject, &[], &two)
            .expect("changed scenario");
    assert_eq!(base.input_hash, changed_scenario.input_hash);
    assert_ne!(base.desugar_hash, changed_scenario.desugar_hash);

    // ③ 兼容配置变 → 描述记录别名；**哈希对不变**（F12：接缓存时 alias 必须入键）。
    let alias_on = compile_component_scenario(src, "c.at", LegacyWidgetAlias::Accept, &[], &one)
        .expect("alias on");
    assert_eq!(base.input_hash, alias_on.input_hash);
    assert_eq!(base.desugar_hash, alias_on.desugar_hash);
    assert!(matches!(base.description.alias, LegacyWidgetAlias::Reject));
    assert!(matches!(
        alias_on.description.alias,
        LegacyWidgetAlias::Accept
    ));
}

// ---- F12：证据身份对源/兼容配置敏感（缓存键完整）----

#[test]
fn p979_component_f12_identity_is_sensitive_to_source_and_compat() {
    use ace_core::source::domain::same_evidence;
    let src = "node C {\n    model {\n        count int = 0\n    }\n    on Inc {\n        count = count + 1\n    }\n    view {\n        text \"x\" {}\n    }\n}\n";
    let one = ComponentScenario {
        instances: 1,
        ops: vec![ScenarioOp::Send {
            instance: 0,
            msg: "Inc".to_string(),
        }],
    };
    let base = compile_component_scenario(src, "c.at", LegacyWidgetAlias::Reject, &[], &one)
        .expect("base");
    let same = compile_component_scenario(src, "c.at", LegacyWidgetAlias::Reject, &[], &one)
        .expect("same");
    assert!(
        same_evidence(&base.identity(), &same.identity()),
        "same source + compat is stable evidence"
    );

    // 源变：源哈希入身份（此前 identity 只有 id/version/grammar）。
    let src2 = format!("fn extra() int {{\n    return 1\n}}\n{src}");
    let changed = compile_component_scenario(&src2, "c.at", LegacyWidgetAlias::Reject, &[], &one)
        .expect("changed source");
    assert!(
        !same_evidence(&base.identity(), &changed.identity()),
        "a different source must not reuse evidence"
    );

    // 兼容配置变：compat 入身份。
    let compat = compile_component_scenario(src, "c.at", LegacyWidgetAlias::Accept, &[], &one)
        .expect("compat on");
    assert!(
        !same_evidence(&base.identity(), &compat.identity()),
        "a different compat config must not reuse evidence"
    );
}

#[test]
fn p979_component_static_face_describes_every_node_in_the_module() {
    // 一个模块里的多个 view-only Node：各自描述、各自文本叶、互不串号。
    let src = "node First {\n    view {\n        text \"a\" {}\n    }\n}\nnode Second {\n    view {\n        text \"b\" {}\n        text \"c\" {}\n    }\n}\n";
    let out = compile_age_component(src, "two.at", LegacyWidgetAlias::Reject, &[])
        .expect("two view-only nodes compile");
    assert_eq!(out.description.nodes.len(), 2);
    assert_eq!(out.description.nodes[0].name, "First");
    assert_eq!(out.description.nodes[0].view.texts.len(), 1);
    assert_eq!(out.description.nodes[1].name, "Second");
    assert_eq!(out.description.nodes[1].view.texts.len(), 2);
    // 每个叶的 def id 不同（不共用求值函数）。
    let mut ids: Vec<_> = out
        .description
        .nodes
        .iter()
        .flat_map(|n| n.view.texts.iter().map(|t| t.def_id.clone()))
        .collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), 3, "each leaf has its own definition: {ids:?}");
    // 节点 span 各自落在自己的声明上。
    assert!(out.description.nodes[0].span.start < out.description.nodes[1].span.start);
}

#[test]
fn p979_component_bool_fields_are_typed_and_an_int_field_is_observed() {
    // 第二字段 bool 可用（状态函数按字段类型返回）。
    let ok = "node S {\n    model {\n        count int = 0\n        on bool = true\n    }\n    on Off {\n        on = false\n    }\n    view {\n        text \"x\" {}\n    }\n}\n";
    let scenario = ComponentScenario {
        instances: 1,
        ops: vec![ScenarioOp::Send {
            instance: 0,
            msg: "Off".to_string(),
        }],
    };
    let out = compile_component_scenario(ok, "s.at", LegacyWidgetAlias::Reject, &[], &scenario)
        .expect("a bool second field is supported");
    assert!(
        out.atom_text.contains("__age_state_0_on"),
        "bool field keeps its own state fn"
    );

    // 只有 bool 字段（没有 int 可观测）：具名拒，span 指向该字段类型。
    let bad = "node S {\n    model {\n        on bool = true\n    }\n    on Off {\n        on = false\n    }\n    view {\n        text \"x\" {}\n    }\n}\n";
    let ds = compile_component_scenario(bad, "s.at", LegacyWidgetAlias::Reject, &[], &scenario)
        .expect_err("a bool-only model has nothing to observe");
    assert_eq!(ds[0].code, "source.age.component-unsupported");
    assert!(
        ds[0].message.contains("at least one `int` model field"),
        "{:?}",
        ds[0]
    );
    let at = bad.find("bool").expect("ty token");
    assert_eq!(ds[0].span.start, at, "span points at the field type");
}

#[test]
fn p979_component_observed_field_can_be_chosen_by_name() {
    let src = "node C {\n    model {\n        first int = 0\n        second int = 0\n    }\n    on Bump {\n        second = second + 1\n    }\n    view {\n        text \"x\" {}\n    }\n}\n";
    let scenario = ComponentScenario {
        instances: 1,
        ops: vec![ScenarioOp::Send {
            instance: 0,
            msg: "Bump".to_string(),
        }],
    };
    // 缺省观测首个 int 字段（first）；显式指定 second。
    let default =
        compile_component_scenario(src, "c.at", LegacyWidgetAlias::Reject, &[], &scenario)
            .expect("default observed field");
    let chosen = ace_core::source::age_component::compile_component_scenario_observing(
        src,
        "c.at",
        LegacyWidgetAlias::Reject,
        &[],
        &scenario,
        Some("second"),
    )
    .expect("chosen observed field");
    assert!(default.atom_text.contains("__age_state_0_first"));
    assert!(chosen.atom_text.contains("__age_state_0_second"));
    fn driver_observes(body: &str) -> String {
        // 驱动体（b___age_drive）里唯一被调用的状态函数名即观测字段。
        let tail = &body[body.find("body b___age_drive").expect("driver body")..];
        let at = tail
            .find("callee: d___age_state_0_")
            .expect("driver callee");
        let rest = &tail[at + "callee: d___age_state_0_".len()..];
        rest.split(|c: char| !(c.is_alphanumeric() || c == '_'))
            .next()
            .unwrap_or_default()
            .to_string()
    }
    assert_eq!(
        driver_observes(&default.atom_text),
        "first",
        "default observes the first int field"
    );
    assert_eq!(
        driver_observes(&chosen.atom_text),
        "second",
        "the chosen field is observed"
    );
}

#[test]
fn p979_component_observed_field_must_exist_and_be_int() {
    let src = "node C {\n    model {\n        count int = 0\n        flag bool = true\n    }\n    on Inc {\n        count = count + 1\n    }\n    view {\n        text \"x\" {}\n    }\n}\n";
    let scenario = ComponentScenario {
        instances: 1,
        ops: vec![ScenarioOp::Send {
            instance: 0,
            msg: "Inc".to_string(),
        }],
    };
    let missing = ace_core::source::age_component::compile_component_scenario_observing(
        src,
        "c.at",
        LegacyWidgetAlias::Reject,
        &[],
        &scenario,
        Some("nope"),
    )
    .expect_err("unknown observed field");
    assert_eq!(missing[0].code, "source.age.component-unknown-field");
    let bad_ty = ace_core::source::age_component::compile_component_scenario_observing(
        src,
        "c.at",
        LegacyWidgetAlias::Reject,
        &[],
        &scenario,
        Some("flag"),
    )
    .expect_err("bool cannot be observed");
    assert_eq!(bad_ty[0].code, "source.age.component-unsupported");
    assert!(bad_ty[0].message.contains('`'), "{:?}", bad_ty[0]);
}

#[test]
fn p979_component_generated_name_families_are_reserved_and_duplicate_fields_are_named() {
    let scenario = ComponentScenario {
        instances: 1,
        ops: vec![ScenarioOp::Send {
            instance: 0,
            msg: "Inc".to_string(),
        }],
    };
    // 用户占用状态函数族名 → hidden-name（两个前门一致）。
    let body = "node C {\n    model {\n        count int = 0\n    }\n    on Inc {\n        count = count + 1\n    }\n    view {\n        text \"x\" {}\n    }\n}\n";
    for reserved in ["__age_state_0_count", "__age_drive"] {
        let src = format!("fn {reserved}() int {{\n    return 0\n}}\n{body}");
        let ds =
            compile_component_scenario(&src, "c.at", LegacyWidgetAlias::Reject, &[], &scenario)
                .expect_err("generated name families are reserved");
        assert_eq!(ds[0].code, "source.age.hidden-name", "{reserved}");
        let at = src.find(reserved).expect("user decl");
        assert_eq!(
            ds[0].span.start, at,
            "{reserved}: span at the user declaration"
        );
    }

    // 重复 model 字段名 → component-duplicate（不是生成符号处的通用 duplicate）。
    let dup = "node C {\n    model {\n        count int = 0\n        count int = 1\n    }\n    on Inc {\n        count = count + 1\n    }\n    view {\n        text \"x\" {}\n    }\n}\n";
    let ds = compile_component_scenario(dup, "c.at", LegacyWidgetAlias::Reject, &[], &scenario)
        .expect_err("duplicate model fields are named");
    assert_eq!(ds[0].code, "source.age.component-duplicate");
    assert!(ds[0].message.contains("count"), "{:?}", ds[0]);
}
