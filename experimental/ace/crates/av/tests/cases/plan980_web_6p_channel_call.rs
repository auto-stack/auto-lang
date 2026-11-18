//! PLAN-980 WEB 切片#6p（调用表达式形·通道调用承载·永久用例）：widget
//! 头参数（宿主通道 prop，`widget Counter(on_select: msg)`）作 callee 的
//! 调用语句——通道行投影（原源 span）、实参死 let 验证（名字/类型真核）、
//! 值位置/ops 派发具名拒（fail-closed）、域外体（非头参数调用）缺席标记。
//!
//! 背景（#6o 收据的"wide-ops 无调用后缀"经探针证伪）：pratt 调用后缀在场
//! （`.x = on_select(1)` 解析通过、Bind 拒）；真缺口=通道调用无承载——
//! 检查函数只铸赋值、调用语句被静默丢弃（[65] `.Bump` 部分体拼接丢句）。
//! 本切片：语句位通道行 + 完整发射合同（§13.17）。

use ace_av::age_component::{
    compile_component_scenario_states, ComponentInitials, ComponentScenario, LegacyWidgetAlias,
    ScenarioOp,
};

fn compile(
    src: &str,
    ops: Vec<ScenarioOp>,
) -> Result<ace_av::age_component::ComponentCompilation, Vec<ace_core::atom_text::Diagnostic>> {
    compile_component_scenario_states(
        src,
        "p980-6p.at",
        LegacyWidgetAlias::Accept,
        &[],
        &ComponentScenario { instances: 1, ops },
        &ComponentInitials {
            per_instance: vec![vec![]],
        },
    )
}

/// ① [65] 形：通道语句编译 OK + 描述出口行（channel/args 原源 span）。
#[test]
fn p980_web_6p_channel_stmt_carries_rows() {
    let src = "widget Counter(on_select: msg) {
    msg Msg { Select(int), Bump(int) }
    model { var n int = 0 }
    view { col { button \"increment\" { onclick: .Bump(.n + 1) } text .n { } } }
    on { .Bump(next) -> { .n = next; on_select(next) } }
}
";
    let c = compile(src, vec![]).expect("channel stmt compiles");
    let node = c.description.nodes.first().expect("node out");
    let bump = node
        .handler_channel_calls
        .iter()
        .find(|(m, _, _)| m == "Bump")
        .expect("Bump facts");
    assert!(!bump.2, "channel stmt is NOT body_foreign");
    assert_eq!(bump.1.len(), 1, "one channel row");
    let row = &bump.1[0];
    assert_eq!(row.channel, "on_select");
    assert_eq!(row.args.len(), 1);
    // 实参原源 span（lane 按精确 span 经 a2x-ts 发射）。
    let ds_len = src.len();
    assert!(
        row.args[0].span.start < row.args[0].span.end && row.args[0].span.end <= ds_len,
        "arg span in source bounds: {:?}",
        row.args[0].span
    );
}

/// ② 值位置（赋值 RHS）通道调用：效应作值——域内具名拒（不是泛化
/// `source.undefined`；fail-closed 不降级）。
#[test]
fn p980_web_6p_channel_value_position_named_reject() {
    let src = "widget T(on_select: msg) {
    msg Msg { Go }
    model { var x int = 0 }
    view { col { button \"b\" { } } }
    on { .Go -> { .x = on_select(1) } }
}
";
    let err = compile(src, vec![]).expect_err("value-position channel must reject");
    assert_eq!(err[0].code, "source.age.component-unsupported");
    assert!(
        err[0].message.contains("effect statement"),
        "named domain message: {}",
        err[0].message
    );
}

/// ③ ops 派发通道 handler：场景重放车道无宿主——零效果具名拒（不做
/// 静默丢通知的部分重放）。
#[test]
fn p980_web_6p_channel_ops_dispatch_named_reject() {
    let src = "widget T(on_go: msg) {
    msg Msg { Go }
    model { var x int = 0 }
    view { col { button \"b\" { } } }
    on { .Go -> { .x = 1; on_go(.x) } }
}
";
    let err = compile(
        src,
        vec![ScenarioOp::Send {
            instance: 0,
            msg: "Go".to_string(),
        }],
    )
    .expect_err("dispatched channel handler must reject");
    assert_eq!(err[0].code, "source.age.component-unsupported");
    assert!(
        err[0].message.contains("on_go"),
        "names the channel: {}",
        err[0].message
    );
}

/// ④ 未声明实参：死 let 验证红（`source.undefined`——实参经共享 verify
/// 真核，未验证拼写绝不进发射）。对照：合法实参（③源）编译绿。
#[test]
fn p980_web_6p_channel_arg_dead_let_verifies_names() {
    let src = "widget T(on_go: msg) {
    msg Msg { Go }
    model { var x int = 0 }
    view { col { button \"b\" { } } }
    on { .Go -> { .x = 1; on_go(typo) } }
}
";
    let err = compile(src, vec![]).expect_err("undeclared arg must fail verification");
    assert_eq!(err[0].code, "source.undefined");
    assert!(
        err[0].message.contains("typo"),
        "names the undeclared arg: {}",
        err[0].message
    );
}

/// ⑤ 域外体（非头参数调用，④外部名面族）：未派发编译 OK + body_foreign
/// 出口（检查函数不铸——#6o 合同闭合；lane 缺席旧道承载）。
#[test]
fn p980_web_6p_foreign_body_flagged_not_failed() {
    let src = "widget F {
    msg Msg { Go }
    model { var x int = 0 }
    view { col { button \"b\" { } } }
    on { .Go -> { .x = 1; fs_read(2) } }
}
";
    let c = compile(src, vec![]).expect("undispatched foreign body compiles (lane absent)");
    let node = c.description.nodes.first().expect("node out");
    let go = node
        .handler_channel_calls
        .iter()
        .find(|(m, _, _)| m == "Go")
        .expect("Go facts");
    assert!(go.2, "foreign body flagged (body_foreign)");
    assert!(go.1.is_empty(), "no channel rows for a foreign callee");
}

/// ⑥ lambda 调用体：foreign lambda 不铸值函数（不再静默跳过调用语句后
/// 部分拼接）——编译 OK（缺席由 lane 侧消费合同断言，见 UI 永久用例）。
/// 边界注记：lambda 体 `list-nl`（换行分隔）——单行多语句形（`.x = 1;
/// on_go(.x)`）是文法层既定边界，本例用单语句形。
#[test]
fn p980_web_6p_foreign_lambda_compiles_uncast() {
    let src = "widget L(on_go: msg) {
    msg Msg { Go }
    model { var x int = 0 }
    view { col { button \"b\" { onclick: () => { on_go(.x) } } } }
}
";
    let c = compile(src, vec![]).expect("foreign lambda compiles (value fn not cast)");
    let _ = c;
}
