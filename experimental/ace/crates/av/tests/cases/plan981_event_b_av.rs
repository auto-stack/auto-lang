//! PLAN-981（B-W1，EVENT-B 三件）的 **AV 真值腿**：
//! `IF-981-EVENT-PAYLOAD-B` / `IF-981-MSG-B` / `IF-981-PARAMS-B`。
//!
//! V-EVT-B-PAYLOAD / V-EVT-B-BIND 的真实执行（共享 lower→MIR→ABC→AV）：
//! - 两实例定向载荷事件 `(i0,+3),(i1,+5),(i0,+4)` → **[7,5]**（独立算术 oracle，
//!   轨迹/隔离/置零/调换/重复求值负控都由该精确值兜住）；
//! - str 载荷绑定（`.QueryInput(v) -> { .query = v }`，两组不同字符串 + 中文 str）
//!   经文本叶取回真值；
//! - 多载荷 `(str, int)` 按位绑定；
//! - 实参**求值一次**（状态依赖实参 + 两次读取的 handler 体）。
use ace_core::atom_text::Span;
use ace_core::source::age_component::{ComponentScenario, InitialValue, ScenarioOp};
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::compile_age_package_with_scenario;
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};

fn input(src: &str) -> PackageModuleInput {
    PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: "main".into(),
        source: src.into(),
        provider: None,
    }
}

fn anchor(src: &str, needle: &str) -> Span {
    let start = src.find(needle).expect("anchor present");
    Span {
        start,
        end: start + needle.len(),
    }
}

/// str 字段的**状态通道**观察（`observe_component_states`——UI 桥 `read_state`
/// 的同一通道；int/bool/str 全域，不依赖文本叶）。
fn str_states(src: &str, ops: Vec<ScenarioOp>, instances: usize) -> Vec<Vec<String>> {
    let scenario = ComponentScenario { instances, ops };
    let compiled = ace_av::age_component::compile_component_scenario_states(
        src,
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
        &scenario,
        &ace_core::source::age_component::ComponentInitials {
            per_instance: vec![Vec::new(); instances],
        },
    )
    .expect("the bind scenario compiles");
    let observed = ace_av::age_component::observe_component_states(&compiled)
        .expect("the shared AV chain returns the str states");
    observed
        .per_instance
        .into_iter()
        .map(|row| {
            row.into_iter()
                .map(|(f, v)| match v {
                    ace_av::age_component::ComponentFieldValue::Str(s) => s,
                    other => panic!("field {f} is not str: {other:?}"),
                })
                .collect()
        })
        .collect()
}

fn observe(
    src: &str,
    ops: Vec<ScenarioOp>,
    instances: usize,
) -> ace_av::age_package::ObservedAgePackage {
    let compiled = compile_age_package_with_scenario(
        &[input(src)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
        &ComponentScenario { instances, ops },
    )
    .expect("the payload scenario compiles");
    ace_av::age_package::observe_age_package(&compiled, instances)
        .expect("the shared AV chain runs the payload events")
}

/// V-EVT-B-PAYLOAD 主行：`msg Msg { Add(int) }` / `.Add(delta) -> { .count += delta }`；
/// 两实例依次 (i0,+3),(i1,+5),(i0,+4) → **[7,5]**；轨迹前缀对应（[3,0]→[3,5]→[7,5]）。
#[test]
fn p981_evt_b_payload_two_instances_independent_values() {
    const SRC: &str = "node Counter {\n\
    msg Msg { Add(int) }\n\
    on {\n\
        .Add(delta) -> {\n\
            .count += delta\n\
        }\n\
    }\n\
    model {\n\
        count int = 0\n\
    }\n\
    view {\n\
        text \"c\" {}\n\
    }\n\
}\n";
    let a = anchor(SRC, "Add(int)");
    let mk =
        |i: usize, v: i64| ScenarioOp::send_args_from_values(i, "Add", &[InitialValue::Int(v)], a);
    let full = || vec![mk(0, 3), mk(1, 5), mk(0, 4)];

    // 轨迹：每条前缀独立编译观察（事件轨迹/提交数对应）。
    let p1 = observe(SRC, vec![mk(0, 3)], 2);
    assert_eq!(p1.components[0].instance_states, vec![vec![3], vec![0]]);
    let p2 = observe(SRC, vec![mk(0, 3), mk(1, 5)], 2);
    assert_eq!(p2.components[0].instance_states, vec![vec![3], vec![5]]);
    let p3 = observe(SRC, full(), 2);
    assert_eq!(
        p3.components[0].instance_states,
        vec![vec![7], vec![5]],
        "independent arithmetic oracle: i0 = 3+4, i1 = 5"
    );

    // 置零负控由精确 oracle 兜住：任一实参置 0 都必须改变对应实例的终值。
    let zeroed = observe(SRC, vec![mk(0, 3), mk(1, 5), mk(0, 0)], 2);
    assert_eq!(zeroed.components[0].instance_states, vec![vec![3], vec![5]]);

    // 改任一实参只改目标实例（i1 的第二次 +5 不影响 i0）。
    let swapped_target = observe(SRC, vec![mk(0, 3), mk(1, 5), mk(1, 4)], 2);
    assert_eq!(
        swapped_target.components[0].instance_states,
        vec![vec![3], vec![9]]
    );
}

/// V-EVT-B-PAYLOAD：实参**求值一次**——状态依赖实参（`count + 2`）在 handler 体
/// 读两次的载荷上仍只求值一次（let 绑定；重求值会给出不同终值）。
#[test]
fn p981_evt_b_payload_args_are_evaluated_once() {
    // handler 体读 delta 两次：count = (count + delta) + delta。
    // 实参 = `count + 2`（求值时 count = 1）→ delta = 3 → 终值 1 + 3 + 3 = 7。
    // 若实参在每次读取处重求值：第二次会看到 count = 4 → 1 + 3 + 6 = 10 ≠ 7。
    const SRC: &str = "node Once {\n\
    msg Msg { Add(int) }\n\
    on {\n\
        .Add(delta) -> {\n\
            .count = count + delta\n\
            .count = count + delta\n\
        }\n\
    }\n\
    model {\n\
        count int = 1\n\
    }\n\
    view {\n\
        text \"o\" {}\n\
    }\n\
}\n";
    let arg = ace_core::source::ast::Expr {
        span: anchor(SRC, "Add(int)"),
        kind: ace_core::source::ast::ExprKind::Binary {
            op: ace_core::source::ast::BinOp::Add,
            lhs: Box::new(ace_core::source::ast::Expr {
                span: anchor(SRC, "count int = 1"),
                kind: ace_core::source::ast::ExprKind::Var(ace_core::source::ast::Ident {
                    name: "count".into(),
                    span: anchor(SRC, "count int = 1"),
                }),
            }),
            rhs: Box::new(ace_core::source::ast::Expr {
                span: anchor(SRC, "Add(int)"),
                kind: ace_core::source::ast::ExprKind::IntLit(2),
            }),
        },
    };
    let ops = vec![ScenarioOp::SendArgs {
        instance: 0,
        msg: "Add".into(),
        args: vec![arg],
    }];
    let observed = observe(SRC, ops, 1);
    assert_eq!(
        observed.components[0].instance_states,
        vec![vec![7]],
        "1 + (1+2) + (1+2) = 7 — the arg is evaluated exactly once"
    );
}

/// V-EVT-B-BIND：`.QueryInput(v) -> { .query = v }`（cap_unused_handler_still_emitted
/// 原形）——两组不同字符串分别落到对应实例的 query（含中文 str）。str 经**状态
/// 通道**（`observe_component_states`，UI 桥 `read_state` 的同一通道）取回真值。
#[test]
fn p981_evt_b_bind_two_strings_reach_their_own_instances() {
    const SRC: &str = "node Panel {\n\
    msg Msg { QueryInput(str), ExtOnly(str) }\n\
    on {\n\
        .QueryInput(v) -> {\n\
            .query = v\n\
        }\n\
        .ExtOnly(v) -> {\n\
            .query = v\n\
        }\n\
    }\n\
    model {\n\
        query str = \"\"\n\
    }\n\
    view {\n\
        text \"p\" {}\n\
    }\n\
}\n";
    let a = anchor(SRC, "QueryInput(str)");
    let got = str_states(
        SRC,
        vec![
            ScenarioOp::send_args_from_values(
                0,
                "QueryInput",
                &[InitialValue::Str("alpha".into())],
                a,
            ),
            ScenarioOp::send_args_from_values(
                1,
                "QueryInput",
                &[InitialValue::Str("中文查询".into())],
                a,
            ),
        ],
        2,
    );
    assert_eq!(
        got,
        vec![vec!["alpha".to_string()], vec!["中文查询".to_string()]],
        "each instance keeps its own query; Chinese str round-trips"
    );
    // 负控（精确值兜住）：交换两组字符串的实例归属必须改变观察。
    let swapped = str_states(
        SRC,
        vec![
            ScenarioOp::send_args_from_values(
                0,
                "QueryInput",
                &[InitialValue::Str("中文查询".into())],
                a,
            ),
            ScenarioOp::send_args_from_values(
                1,
                "QueryInput",
                &[InitialValue::Str("alpha".into())],
                a,
            ),
        ],
        2,
    );
    assert_eq!(
        swapped,
        vec![vec!["中文查询".to_string()], vec!["alpha".to_string()]]
    );
}

/// V-EVT-B-BIND：多载荷 `(str, int)` 逐位绑定（cap_emit_tuple_all_payload_types
/// 原形 `.Resize(name, n)`）——str 落 label（状态通道）、int 落 width（int 状态/
/// 打包车道），两个通道各自对齐独立 oracle。
#[test]
fn p981_evt_b_bind_multi_payload_positional_binding() {
    const SRC: &str = "node W {\n\
    msg Msg { Resize(str, int) }\n\
    on {\n\
        .Resize(name, n) -> {\n\
            .label = name\n\
            .width = n\n\
        }\n\
    }\n\
    model {\n\
        label str = \"\"\n\
        width int = 0\n\
    }\n\
    view {\n\
        text \"w\" {}\n\
    }\n\
}\n";
    let run = |v_str: &str, v_int: i64| -> (Vec<String>, Vec<i32>) {
        let scenario = ComponentScenario {
            instances: 1,
            ops: vec![ScenarioOp::send_args_from_values(
                0,
                "Resize",
                &[InitialValue::Str(v_str.into()), InitialValue::Int(v_int)],
                anchor(SRC, "Resize(str, int)"),
            )],
        };
        let compiled = ace_av::age_component::compile_component_scenario_states(
            SRC,
            "app::main",
            LegacyWidgetAlias::Reject,
            &[PACKAGES_CAPABILITY.to_string()],
            &scenario,
            &ace_core::source::age_component::ComponentInitials {
                per_instance: vec![Vec::new()],
            },
        )
        .expect("the multi-payload scenario compiles");
        let observed = ace_av::age_component::observe_component_states(&compiled)
            .expect("AV returns both channels");
        let row = &observed.per_instance[0];
        let label = row
            .iter()
            .find(|(f, _)| f == "label")
            .map(|(_, v)| match v {
                ace_av::age_component::ComponentFieldValue::Str(s) => s.clone(),
                other => panic!("label is not str: {other:?}"),
            })
            .expect("label field");
        let width = row
            .iter()
            .find(|(f, _)| f == "width")
            .map(|(_, v)| match v {
                ace_av::age_component::ComponentFieldValue::Int(i) => *i,
                other => panic!("width is not int: {other:?}"),
            })
            .expect("width field");
        (vec![label], vec![width])
    };
    let (labels, widths) = run("x", 3);
    assert_eq!(labels, vec!["x".to_string()]);
    assert_eq!(widths, vec![3]);

    // 实参值整体平移（不改顺序）：两个通道都跟着变（同入口可重复观察）。
    let (labels2, widths2) = run("y", 9);
    assert_eq!(labels2, vec!["y".to_string()]);
    assert_eq!(widths2, vec![9]);
}

/// V-EVT-B-PAYLOAD（review-B1）：bool 载荷的正例（cap_event_arg_bool_and_float_literals
/// 的 bool 半边——float 半边是具名拒车道，见 core 侧 plan981_event_b）。
/// `.Toggle(on) -> { .flag = on }` 两实例分别 true/false，经状态通道取回。
#[test]
fn p981_evt_b_bool_payload_binds_positively() {
    const SRC: &str = "node T {\n\
    msg Msg { Toggle(bool) }\n\
    on {\n\
        .Toggle(on) -> {\n\
            .flag = on\n\
        }\n\
    }\n\
    model {\n\
        flag bool = false\n\
    }\n\
    view {\n\
        text \"t\" {}\n\
    }\n\
}\n";
    let a = anchor(SRC, "Toggle(bool)");
    let scenario = ComponentScenario {
        instances: 2,
        ops: vec![
            ScenarioOp::send_args_from_values(0, "Toggle", &[InitialValue::Bool(true)], a),
            ScenarioOp::send_args_from_values(1, "Toggle", &[InitialValue::Bool(false)], a),
        ],
    };
    let compiled = ace_av::age_component::compile_component_scenario_states(
        SRC,
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
        &scenario,
        &ace_core::source::age_component::ComponentInitials {
            per_instance: vec![Vec::new(), Vec::new()],
        },
    )
    .expect("the bool payload scenario compiles");
    let observed = ace_av::age_component::observe_component_states(&compiled)
        .expect("AV returns the bool states");
    let flags: Vec<bool> = observed
        .per_instance
        .iter()
        .map(|row| match &row[0].1 {
            ace_av::age_component::ComponentFieldValue::Bool(b) => *b,
            other => panic!("flag is not bool: {other:?}"),
        })
        .collect();
    assert_eq!(flags, vec![true, false], "each instance keeps its own bool");
}

/// V-EVT-B-BIND：param 与 model **同拼写**——裸 `count` 是形参、`.count` 恒指
/// 实例字段（负控：删除 binder 映射把裸名当 model 读，本例终值必须从 5 变 0）。
#[test]
fn p981_evt_b_bind_param_shadows_same_spelled_model_field() {
    const SRC: &str = "node S {\n\
    msg Msg { Set(int) }\n\
    on {\n\
        .Set(count) -> {\n\
            .count = count\n\
        }\n\
    }\n\
    model {\n\
        count int = 0\n\
    }\n\
    view {\n\
        text \"s\" {}\n\
    }\n\
}\n";
    let a = anchor(SRC, "Set(int)");
    let observed = observe(
        SRC,
        vec![ScenarioOp::send_args_from_values(
            0,
            "Set",
            &[InitialValue::Int(5)],
            a,
        )],
        1,
    );
    assert_eq!(
        observed.components[0].instance_states,
        vec![vec![5]],
        "bare `count` binds the payload (5); treating it as the model read would leave 0"
    );
}

/// SESSION 队列语义（IF-981-EVENT-PAYLOAD-B 的 AV 侧）：载荷事件与 unit 事件
/// 在同一场景混排，轨迹仍按实例/按序精确对应（宿主队列重放的同一模型）。
#[test]
fn p981_evt_b_payload_and_unit_events_interleave_on_one_queue() {
    const SRC: &str = "node M {\n\
    msg Msg { Add(int), Inc }\n\
    on {\n\
        .Add(delta) -> {\n\
            .count += delta\n\
        }\n\
        .Inc -> {\n\
            .count += 1\n\
        }\n\
    }\n\
    model {\n\
        count int = 0\n\
    }\n\
    view {\n\
        text \"m\" {}\n\
    }\n\
}\n";
    let a = anchor(SRC, "Add(int)");
    let ops = vec![
        ScenarioOp::send_args_from_values(0, "Add", &[InitialValue::Int(2)], a),
        ScenarioOp::Send {
            instance: 0,
            msg: "Inc".into(),
        },
        ScenarioOp::send_args_from_values(1, "Add", &[InitialValue::Int(10)], a),
        ScenarioOp::Send {
            instance: 1,
            msg: "Inc".into(),
        },
        ScenarioOp::send_args_from_values(0, "Add", &[InitialValue::Int(1)], a),
    ];
    let observed = observe(SRC, ops, 2);
    assert_eq!(
        observed.components[0].instance_states,
        vec![vec![4], vec![11]],
        "i0 = 2 + 1 (Inc) + 1; i1 = 10 + 1 (Inc)"
    );
}

/// PLAN-981 B-W1（D-981-SESSION-MULTI ①余量）：**双组件 2:1** 的 AV 真值——
/// 同一场景 `ForNode{0}×2 + ForNode{1}×1`，组件 A 计 2、组件 B 计 1
/// （独立算术 oracle；未包裹 op 的旧"全组件重放"语义由对照断言兜住）。
#[test]
fn p981_evt_b_dual_component_two_to_one_events() {
    use ace_core::source::age_component::ScenarioOp;
    const SRC: &str = "node A {\n    msg {\n        Inc\n    }\n    on {\n        .Inc -> {\n            .count += 1\n        }\n    }\n    model {\n        count int = 0\n    }\n    view {\n        text \"a\" {}\n    }\n}\nnode B {\n    msg {\n        Inc\n    }\n    on {\n        .Inc -> {\n            .count += 1\n        }\n    }\n    model {\n        count int = 0\n    }\n    view {\n        text \"b\" {}\n    }\n}\n";
    let for_node = |n: usize| ScenarioOp::ForNode {
        node: n,
        op: Box::new(ScenarioOp::Send {
            instance: 0,
            msg: "Inc".into(),
        }),
    };
    // 2:1 跨组件。
    let o = observe(SRC, vec![for_node(0), for_node(0), for_node(1)], 1);
    assert_eq!(o.components.len(), 2);
    assert_eq!(
        o.components[0].instance_states,
        vec![vec![2]],
        "component A receives its two events"
    );
    assert_eq!(
        o.components[1].instance_states,
        vec![vec![1]],
        "component B receives exactly one"
    );
    // 对照：未包裹 op 重放到两个组件（旧语义不回退）。
    let o2 = observe(
        SRC,
        vec![ScenarioOp::Send {
            instance: 0,
            msg: "Inc".into(),
        }],
        1,
    );
    assert_eq!(o2.components[0].instance_states, vec![vec![1]]);
    assert_eq!(o2.components[1].instance_states, vec![vec![1]]);
    // 组件级带载荷：B 定向 +5（typed 队列与组件限定正交）。
    let o3 = observe(
        "node A {\n    msg Msg { Add(int) }\n    on {\n        .Add(d) -> {\n            .count += d\n        }\n    }\n    model {\n        count int = 0\n    }\n    view {\n        text \"a\" {}\n    }\n}\nnode B {\n    msg Msg { Add(int) }\n    on {\n        .Add(d) -> {\n            .count += d\n        }\n    }\n    model {\n        count int = 0\n    }\n    view {\n        text \"b\" {}\n    }\n}\n",
        vec![ScenarioOp::ForNode {
            node: 1,
            op: Box::new(ScenarioOp::send_args_from_values(
                0,
                "Add",
                &[InitialValue::Int(5)],
                ace_core::atom_text::Span { start: 0, end: 0 },
            )),
        }],
        1,
    );
    assert_eq!(o3.components[0].instance_states, vec![vec![0]]);
    assert_eq!(o3.components[1].instance_states, vec![vec![5]]);
}
