//! PLAN-980 lambda-if 刀（av 面）：view 内联 lambda 体 if 形的**值函数
//! 铸造**——`__age_vl_` 出现在受检模块（旧 assigns-only 面=foreign 不铸→
//! 缺席；铸造事实在模块内容上可判别，F-R1 同族：不靠消费侧掩蔽）+ 深验
//! 证面负控（分支内未声明字段具名拒、cond 未绑定名 verify 红）+ 负控
//! （分支内调用形仍不铸——#6p fail-closed 合同不变）。
#![allow(dead_code)]

use ace_av::age_component::{
    compile_component_scenario_states, ComponentInitials, ComponentScenario, LegacyWidgetAlias,
    ScenarioOp,
};

fn compile(
    src: &str,
) -> Result<ace_av::age_component::ComponentCompilation, Vec<ace_core::atom_text::Diagnostic>> {
    compile_component_scenario_states(
        src,
        "p980-lambda-if.at",
        LegacyWidgetAlias::Accept,
        &[],
        &ComponentScenario {
            instances: 1,
            ops: vec![ScenarioOp::Send {
                instance: 0,
                msg: "R".into(),
            }],
        },
        &ComponentInitials {
            per_instance: vec![vec![]],
        },
    )
}

/// 受检模块里的 `__age_vl_*` 值函数名清单（铸造事实判据）。
fn cast_value_fns(c: &ace_av::age_component::ComponentCompilation) -> Vec<String> {
    c.checked.bundle().modules[0]
        .defs
        .iter()
        .map(|d| d.name_for_diagnostic().to_string())
        .filter(|n| n.contains("__age_vl_"))
        .collect()
}

/// ① if-else 形 lambda 铸值函数：编译 OK（含铸造+共享 verify——cond 与分
/// 支赋值全链通过）+ `__age_vl_0` 在受检模块。旧面（assigns-only）此源
/// =foreign 不铸——本断言即刀面判别器。
#[test]
fn p980_lambda_if_casts_value_fn() {
    let src = "widget W {
    msg Msg { R }
    model { var x int = 0 }
    view { col { text \"t\" { oninput: () => { if .x == 1 { .x = 2 } else { .x = 3 } } } } }
    on { .R -> { .x = 0 } }
}
";
    let c = compile(src).expect("if-form lambda compiles and casts its value fn");
    let fns = cast_value_fns(&c);
    assert_eq!(fns.len(), 1, "one lambda, one value fn in mint order");
    assert!(
        fns[0].ends_with("__age_vl_0"),
        "module-def name carries the mint name (mangled): {fns:?}"
    );
}

/// ② else 分支内未声明字段：深降版到达分支，具名拒（不静默丢句）。
#[test]
fn p980_lambda_if_branch_undeclared_field_rejects() {
    let src = "widget W {
    msg Msg { R }
    model { var x int = 0 }
    view { col { text \"t\" { oninput: () => { if .x == 1 { .x = 2 } else { .nope = 3 } } } } }
    on { .R -> { .x = 0 } }
}
";
    let err = compile(src).expect_err("undeclared field in a branch must reject");
    assert_eq!(err[0].code, "source.age.component-event");
    assert!(
        err[0].message.contains("nope"),
        "names the undeclared field: {}",
        err[0].message
    );
}

/// ③ cond 未绑定名：编译红（条件表达式进唯一 verify——深走查面；裸名
/// 形在绑定段具名拒 `no local named \`y\``，`.y` 字段形同闭）。
#[test]
fn p980_lambda_if_undefined_cond_name_rejects() {
    let src = "widget W {
    msg Msg { R }
    model { var x int = 0 }
    view { col { text \"t\" { oninput: () => { if y == 1 { .x = 2 } } } } }
    on { .R -> { .x = 0 } }
}
";
    let err = compile(src).expect_err("unknown name in a lambda cond must fail verify");
    assert!(
        err[0].message.contains("`y`"),
        "names the unknown cond name: {}",
        err[0].message
    );
    // `.y` 字段形：同样编译红（铸造段 rename 拒——深面不静默放行）。
    let src = "widget W {
    msg Msg { R }
    model { var x int = 0 }
    view { col { text \"t\" { oninput: () => { if .y == 1 { .x = 2 } } } } }
    on { .R -> { .x = 0 } }
}
";
    assert!(
        compile(src).is_err(),
        "field-form unknown cond name also rejects"
    );
}

/// ④ 负控：分支内调用形 → foreign 不铸（#6p 合同不变——与 6p⑥ 单语句调
/// 用形互补，本例为 if 分支形）。
#[test]
fn p980_lambda_if_call_in_branch_stays_uncast() {
    let src = "widget W {
    msg Msg { R }
    model { var x int = 0 }
    view { col { text \"t\" { oninput: () => { if .x == 1 {
        .x = 2
        side()
    } } } } }
    on { .R -> { .x = 0 } }
}
";
    let c = compile(src).expect("foreign lambda compiles (value fn not cast)");
    let fns = cast_value_fns(&c);
    assert!(
        fns.is_empty(),
        "call inside a branch: no partial-body splice, no value fn: {fns:?}"
    );
}

