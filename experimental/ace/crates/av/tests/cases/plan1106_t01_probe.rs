//! PLAN-1106 T-01 探针（临时证据面，T-02/03 演进为永久
//! `component_event_sessions_1106` 模块）：两条既有公共接缝的**最长真实链**
//! 先证 + 当前场景重放车道的执行层基线（V-01 事实面）。
//!
//! - P1 候选①（ProgramSnapshot/全局 cell/初始化纪律）：手装配的增量脱糖
//!   （`store` cell + init/handler 函数 + `module.global.get/set` 调用形态——
//!   与 T-02 将生成的形状一致）经**真实共享前门**
//!   （`compile_program_parts_age` + `verify_text_record_age`）→
//!   `lower_checked_owned_with_init` → `split_units` → encode/admit →
//!   `link_with_globals` → `execute_program_entry_at` 单事件派发 →
//!   `global_cells()` 纯读取。
//! - P2 候选②（owned 会话状态根 + handler 降级调用）：状态经实参/返回值
//!   穿线，每事件一次 `execute_entry`（既有 per-def 执行面）。
//! - P3 V-01 基线：当前场景车道（`observe_component_scenario_states`）每事件
//!   的真实派发身份与次数（`execution_seq` + `dispatched` 载荷名）。

use ace_av::age_component::{
    observe_component_scenario_states, ComponentFieldValue, ComponentInitials, ComponentScenario,
    LegacyWidgetAlias, ScenarioOp,
};
use ace_av_core::contract::{ExecOutcome, Value};
use ace_av_core::program_snapshot::ProgramSnapshot;
use ace_av_core::{execute_program_entry_at, CancelFlag, ExecLimits};
use ace_core::atom_text::Span;
use ace_core::source::adapter::compile_program_parts_age;
use ace_core::source::ast::{
    Block, Expr, ExprKind, FnItem, Ident, Item, Param, ParamMode, Program, Stmt, StoreItem,
    TypeAnn, TypeRef, TypeWord,
};
use ace_core::source::SourceProfile;
use ace_core::verify::verify_text_record_age;
use ace_lowering::lower_checked_owned_with_init;

/// 探针用的 Counter 形源（真实 node 形；span/哈希面与产线一致）。
const COUNTER_SRC: &str = "node Counter {\n    model {\n        count int = 0\n    }\n    on Inc {\n        count = count + 1\n    }\n    on Reset {\n        count = 0\n    }\n    view {\n        text \"x\" {}\n    }\n}\n";

fn sp() -> Span {
    Span {
        start: 0,
        end: COUNTER_SRC.len(),
    }
}

fn ident(name: &str) -> Ident {
    Ident {
        name: name.into(),
        span: sp(),
    }
}

fn int_lit(v: i32) -> Expr {
    Expr {
        span: sp(),
        kind: ExprKind::IntLit(v),
    }
}

fn int_ann() -> TypeAnn {
    TypeAnn {
        kind: TypeRef::Builtin(TypeWord::I32),
        span: sp(),
    }
}

/// `module.global.get:<sym>()`
fn g_get(sym: &str) -> Expr {
    Expr {
        span: sp(),
        kind: ExprKind::Call {
            callee: ident(&format!("module.global.get:{sym}")),
            args: Vec::new(),
        },
    }
}

/// `let <bind> = module.global.set:<sym>(<value>)`
fn g_set(bind: &str, sym: &str, value: Expr) -> Stmt {
    Stmt::Let {
        kw: sp(),
        name: ident(bind),
        ann: None,
        mutable: false,
        value: Expr {
            span: sp(),
            kind: ExprKind::Call {
                callee: ident(&format!("module.global.set:{sym}")),
                args: vec![ace_core::source::ast::Arg::Pos(value)],
            },
        },
        span: sp(),
    }
}

fn fn_item(name: &str, params: Vec<Param>, body: Vec<Stmt>) -> Item {
    Item::Fn(FnItem {
        name: ident(name),
        type_params: Vec::new(),
        params,
        ret: int_ann(),
        body: Block {
            stmts: body,
            span: sp(),
        },
        span: sp(),
        receiver: ace_core::source::ast::Receiver::Free,
        owner: None,
        script: false,
        proto: false,
    })
}

fn param(name: &str) -> Param {
    Param {
        name: ident(name),
        ty: int_ann(),
        mode: ParamMode::Default,
        default: None,
        span: sp(),
    }
}

/// 候选①的手装配增量脱糖（T-02 目标形状的最小实例）：
/// `store __age_c_count: int = 0` + `__age_init()`（写声明初值）+
/// `__age_seed(count0 int)`（props 覆盖）+ `__age_on_inc()` +
/// `__age_on_add(amount int)` + `__age_on_reset()`。
fn incremental_program() -> Program {
    let mut items = Vec::new();
    items.push(Item::Store(StoreItem {
        name: ident("__age_c_count"),
        ty: int_ann(),
        value: int_lit(0),
        span: sp(),
    }));
    items.push(fn_item(
        "__age_init",
        Vec::new(),
        vec![
            g_set("__age_w0", "__age_c_count", int_lit(0)),
            Stmt::Return {
                value: int_lit(0),
                span: sp(),
            },
        ],
    ));
    items.push(fn_item(
        "__age_seed",
        vec![param("__age_p0")],
        vec![
            g_set(
                "__age_w0",
                "__age_c_count",
                Expr {
                    span: sp(),
                    kind: ExprKind::Var(ident("__age_p0")),
                },
            ),
            Stmt::Return {
                value: int_lit(0),
                span: sp(),
            },
        ],
    ));
    items.push(fn_item(
        "__age_on_inc",
        Vec::new(),
        vec![
            // let c = get(count); let w = set(count, c + 1); return w
            Stmt::Let {
                kw: sp(),
                name: ident("__age_r0"),
                ann: None,
                mutable: false,
                value: g_get("__age_c_count"),
                span: sp(),
            },
            g_set(
                "__age_w0",
                "__age_c_count",
                Expr {
                    span: sp(),
                    kind: ExprKind::Binary {
                        op: ace_core::source::ast::BinOp::Add,
                        lhs: Box::new(Expr {
                            span: sp(),
                            kind: ExprKind::Var(ident("__age_r0")),
                        }),
                        rhs: Box::new(int_lit(1)),
                    },
                },
            ),
            Stmt::Return {
                value: int_lit(0),
                span: sp(),
            },
        ],
    ));
    items.push(fn_item(
        "__age_on_add",
        vec![param("__age_p0")],
        vec![
            Stmt::Let {
                kw: sp(),
                name: ident("__age_r0"),
                ann: None,
                mutable: false,
                value: g_get("__age_c_count"),
                span: sp(),
            },
            g_set(
                "__age_w0",
                "__age_c_count",
                Expr {
                    span: sp(),
                    kind: ExprKind::Binary {
                        op: ace_core::source::ast::BinOp::Add,
                        lhs: Box::new(Expr {
                            span: sp(),
                            kind: ExprKind::Var(ident("__age_r0")),
                        }),
                        rhs: Box::new(Expr {
                            span: sp(),
                            kind: ExprKind::Var(ident("__age_p0")),
                        }),
                    },
                },
            ),
            Stmt::Return {
                value: int_lit(0),
                span: sp(),
            },
        ],
    ));
    items.push(fn_item(
        "__age_on_reset",
        Vec::new(),
        vec![
            g_set("__age_w0", "__age_c_count", int_lit(0)),
            Stmt::Return {
                value: int_lit(0),
                span: sp(),
            },
        ],
    ));
    Program { items }
}

/// 真实共享前门：AGE text-record 面 compile + 唯一 verify。
fn front_door(program: &Program) -> Result<ace_core::verify::CheckedModule, String> {
    let caps = ["ac.runtime.text.v1".to_string()];
    let parts = compile_program_parts_age(
        program,
        COUNTER_SRC,
        "ase_m0.counter.t01",
        &caps,
        SourceProfile::TextRecord,
    )
    .map_err(|ds| {
        ds.iter()
            .map(|d| format!("{}: {}", d.code, d.message))
            .collect::<Vec<_>>()
            .join("; ")
    })?;
    verify_text_record_age(parts.bundle).map_err(|ds| {
        ds.iter()
            .map(|d| format!("{}: {}", d.code, d.message))
            .collect::<Vec<_>>()
            .join("; ")
    })
}

/// 链接一个会话快照（共享代码 Arc 可跨快照复用——同一 codes 两次 link 即两个
/// 独立状态根）。
fn link_session(
    codes: &[std::sync::Arc<ace_av_core::function_code::FunctionCode>],
    globals: Vec<ace_av_core::mir::MirGlobal>,
) -> Result<ProgramSnapshot, String> {
    let span_maps = (0..codes.len())
        .map(|_| ace_av_core::span_map::SpanSegments::default())
        .collect();
    ProgramSnapshot::link_with_globals(
        codes.to_vec(),
        "d___age_init",
        0,
        span_maps,
        globals,
        &["d___age_init".to_string()],
    )
}

fn assert_success(name: &str, out: &ExecOutcome) {
    assert!(
        matches!(out, ExecOutcome::Success { .. }),
        "{name} did not succeed: {out:?}"
    );
}

/// P1：候选①最长链——前门→lower+init 链→链→单事件派发→cell 纯读取。
#[test]
fn p1_candidate1_globals_cells_longest_chain() {
    let checked =
        front_door(&incremental_program()).expect("front door admits the incremental form");
    let mir =
        lower_checked_owned_with_init(&checked, "d___age_init", 0, &["d___age_init".to_string()])
            .map_err(|e| format!("{e:?}"))
            .unwrap();
    assert!(
        !mir.globals.is_empty(),
        "lowering must register the store cell as a MIR global"
    );
    let globals = mir.globals.clone();
    // split → encode → admit → FunctionCode（无持久化 store 的纯内存车道）。
    let wire = ace_av::wire::WireCodec::LegacyAbc;
    let mut codes = Vec::new();
    for unit in ace_av::cache::split_units(&mir).unwrap() {
        let bytes = wire.encode(&unit).map_err(|e| format!("{e:?}")).unwrap();
        let loaded = wire
            .admit_bytes(&bytes, &unit.owner)
            .map_err(|e| format!("{e:?}"))
            .unwrap();
        codes.push(std::sync::Arc::new(
            ace_av_core::function_code::FunctionCode::admit_unit(loaded, 0).unwrap(),
        ));
    }
    // 会话 A：init（声明初值 0）→ Inc×2 → cell==2；读取零派发。
    let snap_a = link_session(&codes, globals.clone()).unwrap();
    let limits = ExecLimits::default();
    let cancel = CancelFlag::new(false);
    let seq0 = ace_av_core::av::execution_seq();
    let (out, trace) = execute_program_entry_at(&snap_a, "d___age_on_inc", &limits, &cancel, &[]);
    assert_success("first inc", &out);
    // `dispatched` 是本快照全部已链载荷的身份面（非单入口记录）：5 个函数全量。
    assert_eq!(trace.dispatched.len(), codes.len());
    let names: Vec<&str> = trace.dispatched.iter().map(|d| d.name.as_str()).collect();
    assert!(
        names.contains(&"d___age_on_inc"),
        "handler linked: {names:?}"
    );
    let (out, _t) = execute_program_entry_at(&snap_a, "d___age_on_inc", &limits, &cancel, &[]);
    assert_success("second inc", &out);
    let seq_after_two = ace_av_core::av::execution_seq();
    // 每事件恰一次执行器派发（seq 增量 2），读取不算在内。
    assert_eq!(seq_after_two - seq0, 2, "one dispatch per accepted event");
    assert_eq!(snap_a.global_cells(), vec![Value::I32(2)]);

    // 载荷绑定：add(5) → 7。
    let (out, _t) = execute_program_entry_at(
        &snap_a,
        "d___age_on_add",
        &limits,
        &cancel,
        &[Value::I32(5)],
    );
    assert_success("add", &out);
    assert_eq!(snap_a.global_cells(), vec![Value::I32(7)]);

    // seed 覆盖（props 车道）：seed(41) → add(1) → 42。
    let (out, _t) =
        execute_program_entry_at(&snap_a, "d___age_seed", &limits, &cancel, &[Value::I32(41)]);
    assert_success("seed", &out);
    let (out, _t) = execute_program_entry_at(
        &snap_a,
        "d___age_on_add",
        &limits,
        &cancel,
        &[Value::I32(1)],
    );
    assert_success("add after seed", &out);
    assert_eq!(snap_a.global_cells(), vec![Value::I32(42)]);

    // 会话 B：同一已验代码、独立状态根（3/7 隔离的最小形）。
    let snap_b = link_session(&codes, globals.clone()).unwrap();
    assert_eq!(snap_b.global_cells(), vec![Value::I32(0)]);
    let (out, _t) = execute_program_entry_at(&snap_b, "d___age_on_inc", &limits, &cancel, &[]);
    assert_success("b inc", &out);
    assert_eq!(snap_b.global_cells(), vec![Value::I32(1)]);
    // A 不受 B 影响；重置后 A 归 0。
    assert_eq!(snap_a.global_cells(), vec![Value::I32(42)]);
    let (out, _t) = execute_program_entry_at(&snap_a, "d___age_on_reset", &limits, &cancel, &[]);
    assert_success("reset", &out);
    assert_eq!(snap_a.global_cells(), vec![Value::I32(0)]);

    // 未链函数：具名 Infra 拒（不静默落 entry）。
    let (out, _t) = execute_program_entry_at(&snap_a, "d___age_missing", &limits, &cancel, &[]);
    assert!(
        matches!(out, ExecOutcome::Infra(_)),
        "missing def must be a named Infra refusal"
    );
}

/// P2：候选②——handler 作为纯函数、状态经实参/返回值穿线（既有 per-def 执行面）。
/// 机械上可行；对照记录：状态根落在宿主侧 Rust 值（非执行器 cell）。
#[test]
fn p2_candidate2_state_threading_per_def() {
    // fn __age_s2_step(state int, amount int) -> int { return state + amount }
    let program = Program {
        items: vec![fn_item(
            "__age_s2_step",
            vec![param("__age_p0"), param("__age_p1")],
            vec![Stmt::Return {
                value: Expr {
                    span: sp(),
                    kind: ExprKind::Binary {
                        op: ace_core::source::ast::BinOp::Add,
                        lhs: Box::new(Expr {
                            span: sp(),
                            kind: ExprKind::Var(ident("__age_p0")),
                        }),
                        rhs: Box::new(Expr {
                            span: sp(),
                            kind: ExprKind::Var(ident("__age_p1")),
                        }),
                    },
                },
                span: sp(),
            }],
        )],
    };
    let checked = front_door(&program).expect("per-def lane admits the pure step fn");
    let mir = ace_lowering::lower_checked_owned(&checked, "d___age_s2_step", 0)
        .map_err(|e| format!("{e:?}"))
        .unwrap();
    let wire = ace_av::wire::WireCodec::LegacyAbc;
    let bytes = wire.encode(&mir).map_err(|e| format!("{e:?}")).unwrap();
    let loaded = wire
        .admit_bytes(&bytes, &mir.owner)
        .map_err(|e| format!("{e:?}"))
        .unwrap();
    // 状态根 = Rust 侧局部值；每事件一次真实执行。
    let mut state = Value::I32(0);
    let limits = ExecLimits::default();
    let cancel = CancelFlag::new(false);
    for _ in 0..3 {
        let (out, _trace) =
            ace_av_core::execute_entry(&loaded, &limits, &cancel, &[state.clone(), Value::I32(1)]);
        match out {
            ExecOutcome::Success { value, .. } => state = value,
            other => panic!("step did not succeed: {other:?}"),
        }
    }
    assert_eq!(
        state,
        Value::I32(3),
        "candidate-2 mechanically threads state"
    );
}

/// P3：V-01 事实基线——当前场景车道每事件的真实派发身份与次数。
/// 事实：
/// (a) 当前编译产物的 def 集只有**状态函数**（`d___age_state_0_count`——整段
///     历史内联），不存在任何 `d___age_on_*` handler 入口；
/// (b) 单次执行该状态函数即返回**整段历史**的终值（3 个 Inc → 一次调用出 3）
///     ——历史在函数体内重放，不是每事件一次入口；
/// (c) per-def 执行车道（`execute`）不进入进程级 `EXECUTION_SEQ` 计数（只有
///     快照车道入口计数）——旧车道没有可用的执行入口观察口，这正是新会话
///     面要补的 oracle 位置。
/// 这是修复目标（V-02 恰一次 handler 入口）的反面事实。
#[test]
fn p3_v01_baseline_current_lane_replays_history_per_observation() {
    let alias = LegacyWidgetAlias::Accept;
    let initials = ComponentInitials {
        per_instance: vec![Vec::new()],
    };
    let inc = || ScenarioOp::Send {
        instance: 0,
        msg: "Inc".to_string(),
    };

    // (a)+(b)：编译 3 个 Inc 的场景，产物 def 集只含状态函数；单次执行该
    // def 得整段历史终值 3。
    let caps = ["ac.runtime.text.v1".to_string()];
    let compiled = ace_av::age_component::compile_component_scenario_states(
        COUNTER_SRC,
        "ase_m0.counter.t01",
        alias,
        &caps,
        &ComponentScenario {
            instances: 1,
            ops: vec![inc(), inc(), inc()],
        },
        &initials,
    )
    .unwrap();
    let states = &compiled.description.nodes[0].instance_states[0];
    assert_eq!(states.len(), 1, "1 field = 1 state fn");
    assert_eq!(states[0].def_id, "d___age_state_0_count");
    // 真实执行层：一次执行这个 def → 整段历史（3 个事件）的终值。
    let v =
        ace_av::age_component::execute_component_def(&compiled, "d___age_state_0_count").unwrap();
    assert_eq!(
        v,
        Value::I32(3),
        "one state-def call replays the WHOLE history"
    );

    // (c)：per-def 车道不进进程级 seq（无入口计数端口）。
    let seq0 = ace_av_core::av::execution_seq();
    let _ =
        ace_av::age_component::execute_component_def(&compiled, "d___age_state_0_count").unwrap();
    assert_eq!(
        ace_av_core::av::execution_seq() - seq0,
        0,
        "per-def lane has no entry-count port (snapshot lane only)"
    );

    // K 增长对照：5 个 Inc 的场景重新编译后（每次观察都是一次新编译），
    // 同一 def 一次执行出 5——历史长度直接决定单次执行的工作量。
    let compiled5 = ace_av::age_component::compile_component_scenario_states(
        COUNTER_SRC,
        "ase_m0.counter.t01",
        alias,
        &caps,
        &ComponentScenario {
            instances: 1,
            ops: vec![inc(); 5],
        },
        &initials,
    )
    .unwrap();
    let v5 =
        ace_av::age_component::execute_component_def(&compiled5, "d___age_state_0_count").unwrap();
    assert_eq!(
        v5,
        Value::I32(5),
        "history length drives per-call work (replay)"
    );
}
