//! PLAN-980 B-STATE：组件面的**状态执行服务**消费面——逐字段状态观察、
//! 每实例初值（props 种子）与 scheduler 一次性执行入口（UI 异步宿主）。
//!
//! 与 `plan979_age_component_cpu.rs` 的分工：979 交场景编码驱动（`__age_drive`
//! 打包 oracle）；本文件交 UI 桥的 `read_state` 通道（`observe_component_states`
//! 按 int/bool/str 全域取真值）与 `ComponentInitials` 种子/拒绝面。全部走
//! `ace_av::age_component` 的消费者再导出（与 UI 桥同路径，不直连 ace-core）。

use ace_av::age_component::{
    compile_component_scenario_states, compile_component_scenario_with_initials, observe_compiled,
    observe_component_scenario_states, observe_component_states, observe_component_states_and_texts,
    project_component_candidates, ComponentFieldValue, ComponentInitials, ComponentScenario,
    InitialValue, LegacyWidgetAlias, ScenarioOp,
};

/// int/bool/str 三型字段 + 三条消息的最小组件源（冻结输入；与 002 同族形状）。
const SEED_SRC: &str = "node Panel {\n    model {\n        count int = 0\n        enabled bool = true\n        label str = \"init\"\n    }\n    on Inc {\n        count = count + 1\n    }\n    on Off {\n        enabled = false\n    }\n    on Rename {\n        label = \"renamed\"\n    }\n    view {\n        text \"x\" {}\n    }\n}\n";

fn initials(per: Vec<Vec<(&str, InitialValue)>>) -> ComponentInitials {
    ComponentInitials {
        per_instance: per
            .into_iter()
            .map(|v| v.into_iter().map(|(n, x)| (n.to_string(), x)).collect())
            .collect(),
    }
}

fn empty_initials(instances: usize) -> ComponentInitials {
    ComponentInitials {
        per_instance: vec![Vec::new(); instances],
    }
}

fn field<'a>(row: &'a [(String, ComponentFieldValue)], name: &str) -> &'a ComponentFieldValue {
    &row.iter().find(|(n, _)| n == name).unwrap().1
}

/// 初值按实例种子：count=10/20，Inc 只推进实例 0；enabled/label 保持源内初值。
#[test]
fn p980_state_initials_seed_per_instance_and_ops_stay_directed() {
    let scenario = ComponentScenario {
        instances: 2,
        ops: vec![ScenarioOp::Send {
            instance: 0,
            msg: "Inc".to_string(),
        }],
    };
    let init = initials(vec![
        vec![("count", InitialValue::Int(10))],
        vec![("count", InitialValue::Int(20))],
    ]);
    let states = observe_component_scenario_states(
        SEED_SRC,
        "panel.at",
        LegacyWidgetAlias::Reject,
        &scenario,
        &init,
    )
    .expect("seeded scenario observes");
    assert_eq!(states.per_instance.len(), 2);
    // 实例 0：count 10+1；实例 1：count 20 未被定向 ops 触碰。
    assert_eq!(
        field(&states.per_instance[0], "count"),
        &ComponentFieldValue::Int(11)
    );
    assert_eq!(
        field(&states.per_instance[0], "enabled"),
        &ComponentFieldValue::Bool(true)
    );
    assert_eq!(
        field(&states.per_instance[0], "label"),
        &ComponentFieldValue::Str("init".into())
    );
    assert_eq!(
        field(&states.per_instance[1], "count"),
        &ComponentFieldValue::Int(20)
    );
    assert_eq!(
        field(&states.per_instance[1], "enabled"),
        &ComponentFieldValue::Bool(true)
    );
    assert_eq!(
        field(&states.per_instance[1], "label"),
        &ComponentFieldValue::Str("init".into())
    );
}

/// 逐字段观察反映每实例定向 ops：Off/Rename 只改实例 1，实例 0 全保持。
#[test]
fn p980_state_per_field_observation_reflects_directed_ops() {
    let scenario = ComponentScenario {
        instances: 2,
        ops: vec![
            ScenarioOp::Send {
                instance: 1,
                msg: "Off".to_string(),
            },
            ScenarioOp::Send {
                instance: 1,
                msg: "Rename".to_string(),
            },
        ],
    };
    let states = observe_component_scenario_states(
        SEED_SRC,
        "panel.at",
        LegacyWidgetAlias::Reject,
        &scenario,
        &empty_initials(2),
    )
    .expect("directed ops observe");
    assert_eq!(
        field(&states.per_instance[0], "enabled"),
        &ComponentFieldValue::Bool(true)
    );
    assert_eq!(
        field(&states.per_instance[0], "label"),
        &ComponentFieldValue::Str("init".into())
    );
    assert_eq!(
        field(&states.per_instance[1], "enabled"),
        &ComponentFieldValue::Bool(false)
    );
    assert_eq!(
        field(&states.per_instance[1], "label"),
        &ComponentFieldValue::Str("renamed".into())
    );
    assert_eq!(
        field(&states.per_instance[0], "count"),
        &ComponentFieldValue::Int(0)
    );
}

/// 初值拒绝面：未声明字段 / 类型不匹配 / 超 i32 域 / 单实例重复 / 覆盖数错位。
#[test]
fn p980_state_initials_rejections_are_named() {
    let scenario = ComponentScenario {
        instances: 2,
        ops: vec![],
    };
    let run = |init: ComponentInitials| {
        observe_component_scenario_states(
            SEED_SRC,
            "panel.at",
            LegacyWidgetAlias::Reject,
            &scenario,
            &init,
        )
        .expect_err("must refuse")
    };
    // 未声明字段。
    let err = run(initials(vec![vec![("nope", InitialValue::Int(1))], vec![]]));
    assert!(err.contains("source.age.component-unknown-field"), "{err}");
    // 类型不匹配：int 字段携带 bool。
    let err = run(initials(vec![
        vec![("count", InitialValue::Bool(false))],
        vec![],
    ]));
    assert!(err.contains("source.age.component-unsupported"), "{err}");
    assert!(err.contains("count"), "{err}");
    // int 初值超 i32 域。
    let err = run(initials(vec![
        vec![("count", InitialValue::Int(i64::from(i32::MAX) + 1))],
        vec![],
    ]));
    assert!(err.contains("source.age.component-unsupported"), "{err}");
    assert!(err.contains("i32"), "{err}");
    // 单实例内重复种子。
    let err = run(initials(vec![
        vec![
            ("count", InitialValue::Int(1)),
            ("count", InitialValue::Int(2)),
        ],
        vec![],
    ]));
    assert!(err.contains("source.age.component-duplicate"), "{err}");
    // 覆盖数与实例数错位。
    let err = run(initials(vec![vec![("count", InitialValue::Int(1))]]));
    assert!(err.contains("source.age.component-scenario"), "{err}");
}

/// 状态观察与既有驱动编码互补：同编译下逐字段 int 视图与打包编码一致
/// （实例 0 个位 + 实例 1 的 256 位）。
#[test]
fn p980_state_field_view_agrees_with_packed_driver() {
    let scenario = ComponentScenario {
        instances: 2,
        ops: vec![
            ScenarioOp::Send {
                instance: 0,
                msg: "Inc".to_string(),
            },
            ScenarioOp::Send {
                instance: 1,
                msg: "Inc".to_string(),
            },
            ScenarioOp::Send {
                instance: 1,
                msg: "Inc".to_string(),
            },
        ],
    };
    let init = initials(vec![
        vec![("count", InitialValue::Int(3))],
        vec![("count", InitialValue::Int(4))],
    ]);
    let compiled = compile_component_scenario_with_initials(
        SEED_SRC,
        "panel.at",
        LegacyWidgetAlias::Reject,
        &["ac.runtime.text.v1".to_string()],
        &scenario,
        None,
        &init,
    )
    .expect("compile with initials");
    let states = observe_component_states(&compiled).expect("field view");
    let packed = observe_compiled(&compiled).expect("packed view");
    let c0 = match field(&states.per_instance[0], "count") {
        ComponentFieldValue::Int(v) => *v,
        other => panic!("instance 0 count not int: {other:?}"),
    };
    let c1 = match field(&states.per_instance[1], "count") {
        ComponentFieldValue::Int(v) => *v,
        other => panic!("instance 1 count not int: {other:?}"),
    };
    assert_eq!((c0, c1), (4, 6), "3+1 / 4+2");
    assert_eq!(
        packed.encoded,
        c0 + c1 * 256,
        "packed driver must agree with field view"
    );
}

/// scheduler 一次性入口：真实 task/go/sleep 到静止，stdout 保序（UI 异步宿主）。
#[test]
fn p980_state_run_task_source_executes_async_to_quiescence() {
    // 968 的 V-01 形：go sleeper() 先返回，main 先打印，虚拟时钟把 sleeper 排后。
    let src = "fn main() {\n    go sleeper()\n    print(\"main\")\n}\nfn sleeper() {\n    \
               sleep(100)\n    print(\"later\")\n}\n";
    let r = ace_av::scheduler_run::run_task_source(src, "async_probe.at")
        .expect("scheduler source runs");
    assert!(r.success, "outcome: {}", r.outcome_debug);
    assert_eq!(r.stdout, "main\nlater\n");
}

/// f-string 视图叶边界（具名）：states-only 通道对 f-string 源可观察状态
/// （002 形 `` `Counter: ${.count}` ``）；完整文本观察（`observe_compiled`
/// 的整模块 lowering）在此类源上保持拒绝——`TextDisplay` 在 av.owned/1
/// 车道之外，待后续 profile/接线工作关闭（不是本批静默降级）。
#[test]
fn p980_state_fstring_view_leaf_states_observe_text_lane_refuses() {
    const FSTR_SRC: &str = concat!(
        "node Fid {\n",
        "    model {\n",
        "        count int = 0\n",
        "    }\n",
        "    on Inc {\n",
        "        count = count + 1\n",
        "    }\n",
        "    view {\n",
        "        text `c: ${count}` {}\n",
        "    }\n",
        "}\n",
    );
    let scenario = ComponentScenario {
        instances: 1,
        ops: vec![ScenarioOp::Send {
            instance: 0,
            msg: "Inc".to_string(),
        }],
    };
    // states-only：状态通道可用，Inc 生效。
    let states = observe_component_scenario_states(
        FSTR_SRC,
        "fid.at",
        LegacyWidgetAlias::Reject,
        &scenario,
        &empty_initials(1),
    )
    .expect("states-only observes f-string sources");
    assert_eq!(
        field(&states.per_instance[0], "count"),
        &ComponentFieldValue::Int(1)
    );

    // 完整编译（发射文本叶）→ 整模块 lowering 拒绝 text-display（边界在案）。
    let compiled = compile_component_scenario_with_initials(
        FSTR_SRC,
        "fid.at",
        LegacyWidgetAlias::Reject,
        &["ac.runtime.text.v1".to_string()],
        &scenario,
        None,
        &empty_initials(1),
    )
    .expect("full compile verifies (HIR-level)");
    let err = observe_compiled(&compiled).expect_err("text lane must refuse TextDisplay");
    assert!(err.contains("text-display"), "{err}");
}

/// 复核反例（B-STATE-R）：states-only 是工件级事实，不是运行时偶然——
/// states-only 编译的 atom_text 不含任何文本叶函数；完整编译含之。
/// 删掉发射门控（emit_view_texts 恒真）会让本例红。
#[test]
fn p980_state_states_only_emits_no_text_fns_in_artifact() {
    const SRC: &str = concat!(
        "node T {\n",
        "    model {\n",
        "        n int = 0\n",
        "    }\n",
        "    on Inc {\n",
        "        n = n + 1\n",
        "    }\n",
        "    view {\n",
        "        text \"t\" {}\n",
        "    }\n",
        "}\n",
    );
    let scenario = ComponentScenario {
        instances: 1,
        ops: vec![],
    };
    let caps = ["ac.runtime.text.v1".to_string()];
    let states_only = compile_component_scenario_states(
        SRC,
        "t.at",
        LegacyWidgetAlias::Reject,
        &caps,
        &scenario,
        &empty_initials(1),
    )
    .expect("states-only compiles");
    assert!(states_only.atom_text.contains("__age_state_0_n"));
    assert!(
        !states_only.atom_text.contains("__age_c_text_"),
        "states-only must not emit text leaf functions"
    );
    assert!(
        states_only.description.nodes[0].instance_texts[0].is_empty(),
        "states-only description carries no text rows"
    );
    let full = compile_component_scenario_with_initials(
        SRC,
        "t.at",
        LegacyWidgetAlias::Reject,
        &caps,
        &scenario,
        None,
        &empty_initials(1),
    )
    .expect("full compiles");
    assert!(
        full.atom_text.contains("__age_c_text_"),
        "full compile emits text leaf functions (control)"
    );
}

/// PLAN-980 NATIVE：**免驱动模式**——001 形（model 无 int 字段：str/bool +
/// 文本叶带属性体）经完整场景编译（observed 缺省）不发射驱动；states+texts
/// 双观察可用；既有口径不变（有 int 仍发射；显式观测非 int 仍拒）。
#[test]
fn p980_native_driverless_compile_observes_states_and_texts() {
    const SRC_001_FORM: &str = concat!(
        "node App {\n",
        "    model {\n",
        "        var theme_name str = \"Auto\"\n",
        "        var dark_mode bool = false\n",
        "    }\n",
        "    view {\n",
        "        text \"Hello, World!\" { style: \"text-4xl\", selectable: true }\n",
        "    }\n",
        "}\n",
    );
    let scenario = ComponentScenario {
        instances: 1,
        ops: vec![],
    };
    let caps = ["ac.runtime.text.v1".to_string()];
    let compiled = compile_component_scenario_with_initials(
        SRC_001_FORM,
        "app.at",
        LegacyWidgetAlias::Reject,
        &caps,
        &scenario,
        None,
        &empty_initials(1),
    )
    .expect("driverless compile (str/bool model, no int)");
    assert!(
        !compiled.atom_text.contains("__age_drive"),
        "no packed driver in driverless mode"
    );
    let (states, texts) = observe_component_states_and_texts(&compiled).expect("dual observe");
    assert_eq!(
        field(&states.per_instance[0], "theme_name"),
        &ComponentFieldValue::Str("Auto".into())
    );
    assert_eq!(
        field(&states.per_instance[0], "dark_mode"),
        &ComponentFieldValue::Bool(false)
    );
    assert_eq!(texts[0], vec!["Hello, World!".to_string()]);

    // 结构面：完整 view 树（元素/属性行）可从候选投影消费（selectable 进树）。
    let candidates = project_component_candidates(SRC_001_FORM).expect("candidates project");
    assert_eq!(candidates.len(), 1);
    assert!(!candidates[0].view.tree.is_empty(), "view tree delivered");

    // 口径不变：同源显式观测非 int 字段仍拒；int 源缺省仍发射驱动。
    let err = compile_component_scenario_with_initials(
        SRC_001_FORM,
        "app.at",
        LegacyWidgetAlias::Reject,
        &caps,
        &scenario,
        Some("theme_name"),
        &empty_initials(1),
    )
    .expect_err("explicit non-int observed still refuses");
    assert!(err.iter().any(|d| d.code == "source.age.component-unsupported"));

    let with_int = "node C {\n    model {\n        n int = 0\n    }\n    view {\n        text \"x\" {}\n    }\n}\n";
    let compiled2 = compile_component_scenario_with_initials(
        with_int,
        "c.at",
        LegacyWidgetAlias::Reject,
        &caps,
        &scenario,
        None,
        &empty_initials(1),
    )
    .expect("int source default still compiles");
    assert!(
        compiled2.atom_text.contains("__age_drive"),
        "int source keeps the packed driver"
    );
}
