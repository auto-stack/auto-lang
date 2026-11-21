//! PLAN-981（B-W1，EVENT-B 三件）：`IF-981-EVENT-PAYLOAD-B` / `IF-981-MSG-B` /
//! `IF-981-PARAMS-B` 的**编译面**正负控（AV 真值腿见
//! `av/tests/cases/plan981_event_b_av.rs`）。
//!
//! - **V-EVT-B-MSG**：具名/匿名集合、多载荷类型保真与**原 span**、
//!   重名/未知变体/引用错误与恢复；
//! - **V-EVT-B-PAYLOAD**：`Send` = 空载荷（向载荷变体 Send 是 arity 域错误）；
//!   `SendArgs` 的 arity/类型在**效果前**拒（类型错由唯一 verify 在原 span 报）；
//!   本切片不可承载的载荷拼写具名拒（fail-closed）；
//! - **V-EVT-B-BIND**：handler 形参 ↔ 载荷槽**声明级**一致（未派发也核对）、
//!   重复 binder 拒、param 与 model 同拼写时裸名归 param、未绑定名拒、
//!   未调用 handler 亦按声明语义 verify。
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

fn compile(
    src: &str,
) -> Result<
    ace_core::source::age_package::AgePackageDescription,
    Vec<ace_core::atom_text::Diagnostic>,
> {
    compile_age_package_with_scenario(
        &[input(src)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
        &ComponentScenario {
            instances: 0,
            ops: Vec::new(),
        },
    )
    .map(|out| out.description)
}

fn dispatch(
    src: &str,
    ops: Vec<ScenarioOp>,
    instances: usize,
) -> Result<
    ace_core::source::age_package::AgePackageCompilation,
    Vec<ace_core::atom_text::Diagnostic>,
> {
    compile_age_package_with_scenario(
        &[input(src)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
        &ComponentScenario { instances, ops },
    )
}

fn anchor_of(src: &str, needle: &str) -> Span {
    let start = src.find(needle).expect("fixture contains the anchor");
    Span {
        start,
        end: start + needle.len(),
    }
}

fn int_lit(src: &str, anchor: &str, v: i32) -> ace_core::source::ast::Expr {
    ace_core::source::ast::Expr {
        span: anchor_of(src, anchor),
        kind: ace_core::source::ast::ExprKind::IntLit(v),
    }
}

fn str_lit(src: &str, anchor: &str, v: &str) -> ace_core::source::ast::Expr {
    ace_core::source::ast::Expr {
        span: anchor_of(src, anchor),
        kind: ace_core::source::ast::ExprKind::StrLit(v.to_string()),
    }
}

/// V-EVT-B-MSG：多载荷变体（`Resize(str, int)`，cap_emit_tuple_all_payload_types
/// 原形）保类型拼写、声明序与**原 span**（span 指向作者字节，不是脱糖坐标）。
#[test]
fn p981_evt_b_msg_multi_payload_types_and_span_fidelity() {
    let src = "node W {\n    msg Msg {\n        Resize(str, int),\n        Inc\n    }\n    on {\n        .Resize(name, n) -> {\n            .width = n\n        }\n    }\n    model {\n        width int = 0\n    }\n    view {\n        text \"w\" {}\n    }\n}\n";
    let d = compile(src).expect("a multi-payload named set compiles");
    let c = &d.components[0];
    assert_eq!(c.message_set_name.as_deref(), Some("Msg"));
    let resize = c
        .messages
        .iter()
        .find(|m| m.name == "Resize")
        .expect("Resize variant");
    assert_eq!(resize.payload, vec!["str".to_string(), "int".to_string()]);
    let want = src.find("Resize(str, int)").expect("variant bytes");
    assert_eq!(
        resize.span.start, want,
        "variant span points at author bytes"
    );
    let inc = c.messages.iter().find(|m| m.name == "Inc").expect("Inc");
    assert!(inc.payload.is_empty());
    // handler 形参（名与序）可读回：IF-981-PARAMS-B 的保真面。
    assert_eq!(
        c.handler_params,
        vec![(
            "Resize".to_string(),
            vec!["name".to_string(), "n".to_string()]
        )]
    );
}

/// V-EVT-B-MSG：重名变体、未知变体引用、handler 指向未声明消息——错误表面
/// 具名，**修复同入口绿**（恢复）。
#[test]
fn p981_evt_b_msg_error_surface_and_recovery() {
    let dup = "node M {\n    msg Msg {\n        Add(int),\n        Add(int)\n    }\n    view {\n        text \"t\" {}\n    }\n}\n";
    match compile(dup) {
        Ok(_) => panic!("a duplicate variant must be rejected"),
        Err(ds) => assert_eq!(ds[0].code, "source.age.msg-duplicate", "{ds:?}"),
    }
    let unknown = "node M {\n    msg Msg {\n        Add(int)\n    }\n    on {\n        .Add(x) -> {\n            .count = x\n        }\n        .Rogue(x) -> {\n            .count = x\n        }\n    }\n    model {\n        count int = 0\n    }\n    view {\n        text \"t\" {}\n    }\n}\n";
    match compile(unknown) {
        Ok(_) => panic!("a handler for an undeclared message must be rejected"),
        Err(ds) => assert_eq!(ds[0].code, "source.age.msg-unknown", "{ds:?}"),
    }
    // 恢复：去掉 Rogue handler 后同入口绿。
    let fixed = "node M {\n    msg Msg {\n        Add(int)\n    }\n    on {\n        .Add(x) -> {\n            .count = x\n        }\n    }\n    model {\n        count int = 0\n    }\n    view {\n        text \"t\" {}\n    }\n}\n";
    compile(fixed).expect("the repaired source compiles through the same entry");
}

/// V-EVT-B-PAYLOAD：`Send` 恒等价空载荷——向载荷变体 Send 是 arity 域错误；
/// `SendArgs` 多一个/少一个值同样在**效果前**拒（不丢参、不补零执行）。
#[test]
fn p981_evt_b_payload_arity_is_checked_before_effects() {
    let src = "node M {\n    msg Msg {\n        Add(int)\n    }\n    on {\n        .Add(delta) -> {\n            .count += delta\n        }\n    }\n    model {\n        count int = 0\n    }\n    view {\n        text \"t\" {}\n    }\n}\n";
    let anchor = anchor_of(src, "Add(int)");
    let send = vec![ScenarioOp::Send {
        instance: 0,
        msg: "Add".into(),
    }];
    match dispatch(src, send, 1) {
        Ok(_) => panic!("an empty Send to a payload variant must be rejected"),
        Err(ds) => {
            assert_eq!(ds[0].code, "source.age.component-event", "{ds:?}");
            assert_eq!(ds[0].span.start, anchor.start);
        }
    }
    let two = vec![ScenarioOp::send_args_from_values(
        0,
        "Add",
        &[InitialValue::Int(1), InitialValue::Int(2)],
        anchor,
    )];
    match dispatch(src, two, 1) {
        Ok(_) => panic!("a 2-value dispatch to a 1-slot payload must be rejected"),
        Err(ds) => assert_eq!(ds[0].code, "source.age.component-event", "{ds:?}"),
    }
    // 向 unit 变体带值派发同样拒（declared 0, carried 1）。
    let unit = "node M {\n    msg Msg {\n        Inc\n    }\n    on {\n        .Inc -> {\n            .count += 1\n        }\n    }\n    model {\n        count int = 0\n    }\n    view {\n        text \"t\" {}\n    }\n}\n";
    let bad = vec![ScenarioOp::send_args_from_values(
        0,
        "Inc",
        &[InitialValue::Int(1)],
        anchor_of(unit, "Inc"),
    )];
    match dispatch(unit, bad, 1) {
        Ok(_) => panic!("a valued dispatch to a unit variant must be rejected"),
        Err(ds) => assert_eq!(ds[0].code, "source.age.component-event", "{ds:?}"),
    }
}

/// V-EVT-B-PAYLOAD：错 type 在效果前拒——类型错由**唯一 verify** 报，且 span
/// 指向作者/宿主实参锚（不是脱糖坐标）；同入口正确类型绿。
#[test]
fn p981_evt_b_payload_type_errors_come_from_the_unique_verify() {
    let src = "node M {\n    msg Msg {\n        Add(int)\n    }\n    on {\n        .Add(delta) -> {\n            .count += delta\n        }\n    }\n    model {\n        count int = 0\n    }\n    view {\n        text \"t\" {}\n    }\n}\n";
    let anchor = anchor_of(src, "Add(int)");
    let wrong = vec![ScenarioOp::send_args_from_values(
        0,
        "Add",
        &[InitialValue::Bool(true)],
        anchor,
    )];
    match dispatch(src, wrong, 1) {
        Ok(_) => panic!("a bool value for an int payload must be rejected"),
        Err(ds) => {
            // 共享类型错误（唯一 verify 原则）——不是领域层自判的字符串表。
            assert_eq!(ds[0].code, "source.type-mismatch", "{ds:?}");
            assert!(
                ds[0].message.contains("bool") && ds[0].message.contains("i32"),
                "the type mismatch is reported with both spellings: {ds:?}"
            );
            assert_eq!(ds[0].span.start, anchor.start);
        }
    }
    let good = vec![ScenarioOp::send_args_from_values(
        0,
        "Add",
        &[InitialValue::Int(7)],
        anchor,
    )];
    dispatch(src, good, 1).expect("an int value for an int payload dispatches");
}

/// V-EVT-B-PAYLOAD：多载荷（str, int）实参**调换**必须被判错（str↔int 互换
/// 落到对方的 let 注解上，唯一 verify 在效果前拒）；正确顺序绿。
#[test]
fn p981_evt_b_payload_swapped_multi_args_are_rejected() {
    let src = "node W {\n    msg Msg {\n        Resize(str, int)\n    }\n    on {\n        .Resize(name, n) -> {\n            .label = name\n            .width = n\n        }\n    }\n    model {\n        label str = \"\"\n        width int = 0\n    }\n    view {\n        text \"w\" {}\n    }\n}\n";
    let anchor = anchor_of(src, "Resize(str, int)");
    let swapped = vec![ScenarioOp::SendArgs {
        instance: 0,
        msg: "Resize".into(),
        args: vec![
            int_lit(src, "Resize(str, int)", 3),
            str_lit(src, "Resize(str, int)", "x"),
        ],
    }];
    match dispatch(src, swapped, 1) {
        Ok(_) => panic!("swapping a (str, int) payload must be rejected before effects"),
        Err(ds) => {
            assert_eq!(ds[0].code, "source.type-mismatch", "{ds:?}");
            assert!(
                ds.iter()
                    .any(|d| d.message.contains("Str") && d.message.contains("i32")),
                "the swap is a type error naming both slots: {ds:?}"
            );
        }
    }
    let good = vec![ScenarioOp::SendArgs {
        instance: 0,
        msg: "Resize".into(),
        args: vec![
            str_lit(src, "Resize(str, int)", "x"),
            int_lit(src, "Resize(str, int)", 3),
        ],
    }];
    dispatch(src, good, 1).expect("the correctly ordered payload dispatches");
}

/// V-EVT-B-PAYLOAD：本切片不可承载的载荷拼写（`Opaque`，age-package 面合法
/// 声明）在**派发时**具名拒（`source.age.component-unsupported`，指向原
/// 声明），零效果——fail-closed，不静默丢值执行。
#[test]
fn p981_evt_b_unsupported_payload_lane_is_named_refused() {
    let src = "node P {\n    msg Msg {\n        Receive(Opaque),\n        Bump(int)\n    }\n    on {\n        .Receive(v) -> {\n            .hits += 1\n        }\n        .Bump(n) -> {\n            .hits += n\n        }\n    }\n    model {\n        hits int = 0\n    }\n    view {\n        text \"p\" {}\n    }\n}\n";
    let anchor = anchor_of(src, "Receive(Opaque)");
    compile(src).expect("declaration can retain an unsupported payload type");
    let ops = vec![ScenarioOp::send_args_from_values(
        0,
        "Receive",
        &[InitialValue::Int(1)],
        anchor,
    )];
    match dispatch(src, ops, 1) {
        Ok(_) => panic!("dispatching an unsupported payload must be named-refused"),
        Err(ds) => {
            assert_eq!(ds[0].code, "source.age.component-unsupported", "{ds:?}");
            assert_eq!(ds[0].span.start, anchor.start);
        }
    }
    // 同入口 int 载荷照常（unsupported 只拒该变体的车道，不拦整 node）。
    let ok = vec![ScenarioOp::send_args_from_values(
        0,
        "Bump",
        &[InitialValue::Int(2)],
        anchor_of(src, "Bump(int)"),
    )];
    dispatch(src, ok, 1).expect("the int payload next door still dispatches");
}

/// V-EVT-B-BIND：形参数 ↔ 载荷槽数**声明级**一致（未派发也核对）；重复
/// binder 具名拒；修复同入口绿。
#[test]
fn p981_evt_b_bind_declaration_level_checks() {
    // 槽 1、形参 0：声明即拒（不需要派发）。
    let missing = "node M {\n    msg Msg {\n        Add(int)\n    }\n    on {\n        .Add -> {\n            .count += 1\n        }\n    }\n    model {\n        count int = 0\n    }\n    view {\n        text \"t\" {}\n    }\n}\n";
    match compile(missing) {
        Ok(_) => panic!("a payload variant without its binder must be rejected"),
        Err(ds) => {
            assert_eq!(ds[0].code, "source.age.component-event", "{ds:?}");
            assert!(ds[0].message.contains("payload slot"), "{ds:?}");
        }
    }
    // 槽 2、形参 1：同样拒，span 指到 handler。
    let short = "node M {\n    msg Msg {\n        Add(int, int)\n    }\n    on {\n        .Add(a) -> {\n            .count += a\n        }\n    }\n    model {\n        count int = 0\n    }\n    view {\n        text \"t\" {}\n    }\n}\n";
    match compile(short) {
        Ok(_) => panic!("one binder for two payload slots must be rejected"),
        Err(ds) => assert_eq!(ds[0].code, "source.age.component-event", "{ds:?}"),
    }
    // 重复 binder：`.Add(t, t)` 拒，span 指第二个 `t`。
    let dup = "node M {\n    msg Msg {\n        Add(int, int)\n    }\n    on {\n        .Add(t, t) -> {\n            .count += t\n        }\n    }\n    model {\n        count int = 0\n    }\n    view {\n        text \"t\" {}\n    }\n}\n";
    let want = dup.rfind("t)").map(|p| p).expect("second binder");
    match compile(dup) {
        Ok(_) => panic!("a duplicate binder must be rejected"),
        Err(ds) => {
            assert_eq!(ds[0].code, "source.age.component-event", "{ds:?}");
            assert!(ds[0].message.contains("twice"), "{ds:?}");
            assert_eq!(ds[0].span.start, want);
        }
    }
    // 修复：`(t, u)` 同入口绿。
    let fixed = "node M {\n    msg Msg {\n        Add(int, int)\n    }\n    on {\n        .Add(t, u) -> {\n            .count += t\n            .count += u\n        }\n    }\n    model {\n        count int = 0\n    }\n    view {\n        text \"t\" {}\n    }\n}\n";
    compile(fixed).expect("distinct binders compile through the same entry");
}

/// V-EVT-B-BIND：param 与 model 字段**同拼写**——裸名归 param、`.field` 恒指
/// 实例 model（负控：删除 binder 映射把裸 `count` 当 model 读，本例必须红）；
/// 未绑定名由唯一 verify 拒；**未调用** handler 亦按声明语义 verify。
#[test]
fn p981_evt_b_bind_shadowing_unbound_and_unused_handlers() {
    // 同拼写遮蔽：`.Set(count) -> { .count = count }`——RHS 裸 `count` 是形参。
    let shadow = "node S {\n    msg Msg {\n        Set(int)\n    }\n    on {\n        .Set(count) -> {\n            .count = count\n        }\n    }\n    model {\n        count int = 0\n    }\n    view {\n        text \"s\" {}\n    }\n}\n";
    let anchor = anchor_of(shadow, "Set(int)");
    let out = dispatch(
        shadow,
        vec![ScenarioOp::send_args_from_values(
            0,
            "Set",
            &[InitialValue::Int(5)],
            anchor,
        )],
        1,
    )
    .expect("same-spelling param shadows the model field for bare names");
    let _ = out;
    // `.count`（点语法）恒指 model：`.count = count + 1` 里点号是字段、裸名是形参。
    let mixed = "node S {\n    msg Msg {\n        Set(int)\n    }\n    on {\n        .Set(count) -> {\n            .count = count + 1\n        }\n    }\n    model {\n        count int = 0\n    }\n    view {\n        text \"s\" {}\n    }\n}\n";
    dispatch(
        mixed,
        vec![ScenarioOp::send_args_from_values(
            0,
            "Set",
            &[InitialValue::Int(5)],
            anchor_of(mixed, "Set(int)"),
        )],
        1,
    )
    .expect("dotted access stays on the model, bare names bind to the param");

    // 未绑定名：`.query = zz` —— 唯一 verify 拒（span 落作者源）。
    let unbound = "node P {\n    msg Msg {\n        QueryInput(str)\n    }\n    on {\n        .QueryInput(v) -> {\n            .query = zz\n        }\n    }\n    model {\n        query str = \"\"\n    }\n    view {\n        text \"p\" {}\n    }\n}\n";
    let want = unbound.find("zz").expect("unbound name bytes");
    match dispatch(
        unbound,
        vec![ScenarioOp::send_args_from_values(
            0,
            "QueryInput",
            &[InitialValue::Str("x".into())],
            anchor_of(unbound, "QueryInput(str)"),
        )],
        1,
    ) {
        Ok(_) => panic!("an unbound name in a handler body must be rejected"),
        Err(ds) => assert_eq!(ds[0].span.start, want, "{ds:?}"),
    }

    // 未调用 handler 亦按声明语义 verify：ExtOnly 从未被派发，但赋值了不存在的
    // 字段——无场景（instances 0）也拒。
    let unused = "node P {\n    msg Msg {\n        QueryInput(str),\n        ExtOnly(str)\n    }\n    on {\n        .QueryInput(v) -> {\n            .query = v\n        }\n        .ExtOnly(v) -> {\n            .nope = v\n        }\n    }\n    model {\n        query str = \"\"\n    }\n    view {\n        text \"p\" {}\n    }\n}\n";
    match compile(unused) {
        Ok(_) => panic!("an unused handler must still verify against declarations"),
        Err(ds) => {
            assert_eq!(ds[0].code, "source.age.component-event", "{ds:?}");
            assert!(ds[0].message.contains("nope"), "{ds:?}");
        }
    }
}

/// PLAN-981 B-W1（D-981-SESSION-MULTI ①余量）：**组件级**定向（`ForNode`）——
/// 同一场景只推进目标组件；限定到不存在的组件具名拒；未包裹 op 对双组件
/// 重放的旧语义不变（对照组）。
#[test]
fn p981_evt_b_for_node_scopes_ops_to_one_component() {
    use ace_core::source::age_component::ScenarioOp;
    let src = "node A {\n    msg {\n        Inc\n    }\n    on {\n        .Inc -> {\n            .count += 1\n        }\n    }\n    model {\n        count int = 0\n    }\n    view {\n        text \"a\" {}\n    }\n}\nnode B {\n    msg {\n        Inc\n    }\n    on {\n        .Inc -> {\n            .count += 1\n        }\n    }\n    model {\n        count int = 0\n    }\n    view {\n        text \"b\" {}\n    }\n}\n";
    let for_node = |n: usize| ScenarioOp::ForNode {
        node: n,
        op: Box::new(ScenarioOp::Send {
            instance: 0,
            msg: "Inc".into(),
        }),
    };
    // A×2、B×1（2:1 跨组件）：只动目标组件。
    let out = dispatch(src, vec![for_node(0), for_node(0), for_node(1)], 1)
        .expect("the component-scoped 2:1 scenario compiles");
    let _ = out;
    // 限定到不存在的组件：具名拒（不静默丢弃）。
    match dispatch(src, vec![for_node(2)], 1) {
        Ok(_) => panic!("an op scoped to a missing node must be rejected"),
        Err(ds) => {
            assert_eq!(ds[0].code, "source.age.component-scenario", "{ds:?}");
            assert!(ds[0].message.contains("node 2"), "{ds:?}");
        }
    }
    // 对照：未包裹的 op 对两个组件都重放（旧语义）。
    dispatch(
        src,
        vec![ScenarioOp::Send {
            instance: 0,
            msg: "Inc".into(),
        }],
        1,
    )
    .expect("unscoped ops keep replaying on every component");
}
