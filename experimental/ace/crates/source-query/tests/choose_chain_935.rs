//! PLAN-935 T-02：choose 双实例整链（查询层终点 = 具体 Checked HIR）。
//!
//! oracle 矩阵（独立手算，不用模型 echo；O-3/O-4 的**执行值**在 AV/native/
//! A2X 端各自的 case 里闭式断言——本文件断言查询层产物形状）：
//! * O-1/O-2 两类真实实例并存（T=bool 与 T=int），各有 producer 身份；
//! * O-3 main 经重绑后的具体程序真实 emit+verify（无模板/Unknown 残留）；
//! * O-5 异质实参具名拒；O-6 无解 T 具名拒；
//! * O-7 同 T 多 callsite → 单实例、demands=2；
//! * O-9 只改 caller（实例 key 不变）→ 实例复用（body 页 hit，0 重产）；
//! * O-4 模板 body 换版 → 实例键变化 + 该 body 页重产；
//! * plain 模块经实例 lane 与 plain 面逐字节同产物（identity 通道）。

use ace_core::atom_text::Diagnostic;
use ace_core::source::cst::INSTANCES_CAPABILITY;
use ace_source_query::{
    compile_scalar_product, instantiate_program, CompileLane, SourceQuerySession,
};
use ace_source_query::{InstanceRecord, INSTANCES_KEY_DOMAIN, INSTANCES_LOGICAL_DOMAIN};

const CHOOSE: &str = "\
fn choose<T>(a T, b T, flag bool) T {
    if flag { return a }
    return b
}
fn main() int {
    if choose(true, false, true) { return choose(40, 42, false) }
    return 0
}
";

/// O-4：交换模板两个 return 分支（调用点不动）。
const CHOOSE_SWAPPED: &str = "\
fn choose<T>(a T, b T, flag bool) T {
    if flag { return b }
    return a
}
fn main() int {
    if choose(true, false, true) { return choose(40, 42, false) }
    return 0
}
";

fn caps() -> Vec<String> {
    vec![INSTANCES_CAPABILITY.to_string()]
}

fn compile(src: &str) -> Result<ace_source_query::ProductCompile, Vec<Diagnostic>> {
    let mut session = SourceQuerySession::new("plan935", "choose.at");
    compile_scalar_product(
        src,
        "choose.at",
        &caps(),
        "plan935",
        "choose.at",
        &mut session,
    )
}

fn def_texts(p: &ace_source_query::ProductCompile) -> Vec<String> {
    p.compiled
        .checked
        .bundle()
        .modules
        .iter()
        .flat_map(|m| m.defs.iter().map(|d| d.id_text.clone()))
        .collect()
}

fn find_instance<'a>(ins: &'a [InstanceRecord], suffix: &str) -> &'a InstanceRecord {
    ins.iter()
        .find(|i| i.concrete_name.ends_with(suffix))
        .unwrap_or_else(|| panic!("no instance {suffix}: {ins:?}"))
}

/// O-1/O-2/O-3：双实例并存，全部真实走完 parse→instantiate→facts→emit→
/// verify；执行 HIR 里只有具体函数（choose_i32/choose_bool），没有模板、
/// 没有 Unknown/Any 残留。
#[test]
fn choose_dual_instances_real_checked_hir() {
    let p = compile(CHOOSE).expect("choose compiles through the instances lane");
    assert_eq!(p.lane, CompileLane::Instances);
    assert_eq!(p.instances.len(), 2, "exactly two demanded instances");
    let bool_inst = find_instance(&p.instances, "_bool");
    let int_inst = find_instance(&p.instances, "_i32");
    assert_eq!(bool_inst.demands, 1);
    assert_eq!(int_inst.demands, 1);
    assert_ne!(bool_inst.key, int_inst.key, "substitution enters the key");
    assert_eq!(bool_inst.key, {
        // 键 = 逻辑实例 + 签名/体版本 + substitution + 算法/域（可复算）。
        let logical = ace_source_query::sq_hash(
            INSTANCES_LOGICAL_DOMAIN,
            &[
                b"plan935".as_slice(),
                b"choose.at".as_slice(),
                b"tpl".as_slice(),
                b"choose".as_slice(),
            ],
        );
        let k = ace_source_query::sq_hash(
            INSTANCES_KEY_DOMAIN,
            &[
                logical.as_bytes(),
                b"sig",
                bool_inst.sig_version.as_bytes(),
                b"body",
                bool_inst.body_version.as_bytes(),
                b"T",
                b"bool",
                ace_source_query::INSTANCES_ALGO.as_bytes(),
                ace_source_query::INSTANCES_SCHEMA.as_bytes(),
            ],
        );
        k
    });
    // 具体 HIR 面：实例与 main 都在，无模板残留。
    let defs = def_texts(&p);
    let has = |s: &str| defs.iter().any(|d| d.contains(s));
    assert!(has("choose_i32"), "defs: {defs:?}");
    assert!(has("choose_bool"), "defs: {defs:?}");
    assert!(has("main"), "defs: {defs:?}");
    assert!(
        !defs.iter().any(|d| d
            .trim_end_matches("choose_i32")
            .trim_end_matches("choose_bool")
            == "d_choose"
            || d == "choose"),
        "no template def residue: {defs:?}"
    );
    // supply/apply：每个自由函数（main + 两实例）都有事实。
    assert!(p.applied.iter().any(|a| a.name.contains("choose_i32")));
    assert!(p.applied.iter().any(|a| a.name.contains("choose_bool")));
    assert!(p.applied.iter().any(|a| a.name == "main"));
}

/// O-5：异质实参（40 与 true 争 T）→ 具名拒（无隐式转换/Any）。
#[test]
fn heterogeneous_arguments_named_rejection() {
    let src = "\
fn choose<T>(a T, b T, flag bool) T {
    if flag { return a }
    return b
}
fn main() int {
    return choose(40, true, false)
}
";
    match compile(src) {
        Err(ds) => assert!(
            ds.iter().any(|d| d.code == "type-mismatch"
                && d.message.contains("bool") && d.message.contains("i32")),
            "expected the real solver heterogeneous rejection, got {ds:?}"
        ),
        Ok(p) => panic!(
            "must reject, got lane={:?} instances={}",
            p.lane,
            p.instances.len()
        ),
    }
}

/// O-6：T 无法由实参唯一解出（只在返回位出现）→ 具名拒（不猜默认 T）。
#[test]
fn unresolved_t_named_rejection() {
    // T 只出现在返回位且实参约束不到它（x 未用、无期望传播——T-02 车辆
    // 边界，期望类型解 T 属 T-04 求解器 Check 任务）。
    let src = "\
fn probe<T>(a int) T {
    return a
}
fn main() int {
    let x = probe(1)
    return 0
}
";
    match compile(src) {
        Err(ds) => assert!(
            ds.iter().any(|d| d.code == "unsolved-variable"),
            "expected the real solver unsolved rejection, got {ds:?}"
        ),
        Ok(_p) => panic!("must reject, got acceptance"),
    }
}

/// O-7 + O-9：同 T 两个 callsite → 一实例 demands=2；只改 caller 常量
/// （实例 key 不变）→ 同 session 复编译实例 body 页 hit、solver 0 重跑；
/// 未引用模板不生产（choose 无调用时不产任何实例）。
#[test]
fn same_t_single_instance_and_caller_edit_reuse() {
    let same_two_calls = "\
fn choose<T>(a T, b T, flag bool) T {
    if flag { return a }
    return b
}
fn main() int {
    let x = choose(40, 42, false)
    let y = choose(50, 60, false)
    return x + y
}
";
    let mut session = SourceQuerySession::new("plan935", "reuse.at");
    let caps = caps();
    let p1 = compile_scalar_product(
        same_two_calls,
        "reuse.at",
        &caps,
        "plan935",
        "reuse.at",
        &mut session,
    )
    .expect("compiles");
    assert_eq!(p1.instances.len(), 1, "both callsites share one instance");
    assert_eq!(p1.instances[0].demands, 2);
    let key1 = p1.instances[0].key.clone();
    let runs1 = session.counters().solver_runs;
    assert!(runs1 > 0, "cold compile actually solves");
    // O-9：只改 caller 常量（choose 体与签名不动）→ 实例 key 不变、实例
    // body 页 **hit**。允许的 solver 重跑恰为 main 自身那一页：runs 恰 +1
    // （若实例也重产会是 +2）。
    let edited = same_two_calls.replace("50, 60", "51, 61");
    let p2 = compile_scalar_product(
        &edited,
        "reuse.at",
        &caps,
        "plan935",
        "reuse.at",
        &mut session,
    )
    .expect("edited caller still compiles");
    assert_eq!(p2.instances.len(), 1);
    assert_eq!(
        p2.instances[0].key, key1,
        "instance key is caller-edit stable"
    );
    assert_eq!(
        session.counters().solver_runs,
        runs1 + 1,
        "only the edited caller re-solves; the instance body page hits"
    );
    assert!(session.counters().body_hits > 0);
    // 同源第三遍：全 hit，0 重跑（页缓存稳定）。
    let _p3 = compile_scalar_product(
        &edited,
        "reuse.at",
        &caps,
        "plan935",
        "reuse.at",
        &mut session,
    )
    .expect("third compile");
    assert_eq!(
        session.counters().solver_runs,
        runs1 + 1,
        "a recompile of the same source solves nothing new"
    );
}

/// O-4：模板 body 换版（交换分支）→ 实例键变化、实例 body 页重产；
/// 签名轴不动（caller 只读签名不受扰动——签名页 hit）。
#[test]
fn template_body_edit_changes_key_and_reproduces() {
    let mut session = SourceQuerySession::new("plan935", "swap.at");
    let caps = caps();
    let p1 = compile_scalar_product(CHOOSE, "swap.at", &caps, "plan935", "swap.at", &mut session)
        .expect("original compiles");
    let key1 = find_instance(&p1.instances, "_i32").key.clone();
    let runs1 = session.counters().solver_runs;
    let p2 = compile_scalar_product(
        CHOOSE_SWAPPED,
        "swap.at",
        &caps,
        "plan935",
        "swap.at",
        &mut session,
    )
    .expect("swapped compiles");
    let key2 = find_instance(&p2.instances, "_i32").key.clone();
    assert_ne!(key1, key2, "body version enters the instance key");
    assert!(
        session.counters().solver_runs > runs1,
        "swapped template body re-produces its instances"
    );
    // 签名轴不变：两版的 sig_version 相同。
    assert_eq!(
        find_instance(&p1.instances, "_i32").sig_version,
        find_instance(&p2.instances, "_i32").sig_version
    );
}

/// 未引用模板：无实例、不生产（不无限实例化）；模块照常编译。
#[test]
fn unreferenced_template_not_instantiated() {
    let src = "\
fn unused<T>(a T) T { return a }
fn main() int { return 7 }
";
    let p = compile(src).expect("compiles");
    assert_eq!(p.lane, CompileLane::Facts, "no demands → facts lane");
    assert!(p.instances.is_empty());
}

/// plain 模块（无模板）经实例 lane 与 plain 面逐字节同产物（identity
/// 通道——携带能力不改变无模板源的字节）。
#[test]
fn plain_module_identical_through_instances_lane() {
    let src = "\
fn add(a int, b int) int { return a + b }
fn main() int { return add(20, 22) }
";
    let mut lane_session = SourceQuerySession::new("plan935", "same.at");
    let via_lane = compile_scalar_product(
        src,
        "same.at",
        &caps(),
        "plan935",
        "same.at",
        &mut lane_session,
    )
    .expect("instances-lane compiles");
    let mut plain_session = SourceQuerySession::new("plan935", "same.at");
    let plain = compile_scalar_product(
        src,
        "same.at",
        &[],
        "plan935",
        "same.at",
        &mut plain_session,
    )
    .expect("plain compiles");
    assert_eq!(via_lane.compiled.input_hash, plain.compiled.input_hash);
    assert_eq!(via_lane.compiled.atom_text, plain.compiled.atom_text);
    assert_eq!(via_lane.lane, CompileLane::Facts);
}

/// 直接实例化 API 的身份面：脱糖程序确定（两次运行同字节）、重绑计数、
/// 实例记录可枚举（936 消费本包实例的接缝）。
#[test]
fn instantiate_api_deterministic() {
    use ace_core::source::cst::{
        parse_front_with_caps, FrontBudget, FrontKind, FrontOutcome, FrontRequest,
    };
    use ace_core::source::SourceProfile;
    let parse = |src: &str| match parse_front_with_caps(
        FrontRequest {
            src,
            profile: SourceProfile::Scalar,
            kind: FrontKind::WholeModule,
            budget: FrontBudget::WORK_OWNED_64MIB,
        },
        &caps(),
    ) {
        FrontOutcome::Ok(ok) => ok.program,
        other => panic!("parse failed: {other:?}"),
    };
    let a = instantiate_program(&parse(CHOOSE), "plan935", "choose.at", &caps()).expect("ok");
    let b = instantiate_program(&parse(CHOOSE), "plan935", "choose.at", &caps()).expect("ok");
    assert_eq!(a.rebinding_count, 2, "two template callsites rebound");
    assert_eq!(format!("{:?}", a.program), format!("{:?}", b.program));
    assert_eq!(a.instances, b.instances);
    assert_eq!(a.instances.len(), 2);
    // 域身份：新版本域，不混 934 事实 schema。
    assert_eq!(ace_source_query::INSTANCES_SCHEMA, "ac935/instances/v1");
    assert_eq!(
        ace_source_query::INSTANCES_PRODUCER,
        "plan935/source-instances"
    );
}
/// T-04：期望类型解 T——`make<T>() T` 无实参可约束，`let x int = make()`
/// 的注解位经真求解器 Check 任务 unify 出 T=int（T-02 车辆拒的形态）。
#[test]
fn t04_expected_type_solves_t() {
    let src = "\
fn make<T>() T {
    return 0
}
fn main() int {
    let x int = make()
    return x
}
";
    let p = compile(src).expect("expected-type solves T");
    assert_eq!(p.lane, CompileLane::Instances);
    assert_eq!(p.instances.len(), 1);
    assert!(p.instances[0].concrete_name.contains("make_i32"));
    assert!(def_texts(&p).iter().any(|d| d.contains("make_i32")));
}

/// T-04：跨模板调用（wrap<T> 体调 choose）——fixpoint 物化把实例体内的
/// 模板调用再发现（T-02 具名拒的形态），两层 substitution 正确传播。
#[test]
fn t04_cross_template_fixpoint() {
    let src = "\
fn choose<T>(a T, b T, flag bool) T {
    if flag { return a }
    return b
}
fn wrap<T>(a T, b T) T {
    return choose(a, b, true)
}
fn main() int {
    if wrap(true, false) { return wrap(40, 42) }
    return 0
}
";
    let p = compile(src).expect("cross-template compiles");
    assert_eq!(p.instances.len(), 4, "wrap i32+bool and choose i32+bool");
    for name in ["wrap_i32", "wrap_bool", "choose_i32", "choose_bool"] {
        assert!(
            p.instances.iter().any(|i| i.concrete_name == name),
            "missing {name}: {:?}",
            p.instances.iter().map(|i| i.concrete_name.clone()).collect::<Vec<_>>()
        );
    }
    // 实例体内的跨模板调用已重绑到同型实例（脱糖程序形状）。
    use ace_core::source::cst::{parse_front_with_caps, FrontBudget, FrontKind, FrontOutcome, FrontRequest};
    use ace_core::source::SourceProfile;
    let parse = |s: &str| match parse_front_with_caps(
        FrontRequest { src: s, profile: SourceProfile::Scalar, kind: FrontKind::WholeModule, budget: FrontBudget::WORK_OWNED_64MIB },
        &caps(),
    ) {
        FrontOutcome::Ok(ok) => ok.program,
        other => panic!("{other:?}"),
    };
    let desugared = instantiate_program(&parse(src), "plan935", "choose.at", &caps()).expect("ok");
    let body_of = |p: &ace_core::source::ast::Program, name: &str| {
        p.items.iter().find_map(|i| match i {
            ace_core::source::ast::Item::Fn(f) if f.name.name == name => Some(format!("{:?}", f.body)),
            _ => None,
        }).unwrap_or_else(|| panic!("no {name}"))
    };
    assert!(body_of(&desugared.program, "wrap_i32").contains("choose_i32"));
    assert!(body_of(&desugared.program, "wrap_bool").contains("choose_bool"));
    assert!(!body_of(&desugared.program, "wrap_i32").contains("\"choose\""));
}

/// T-04：模板自递归——同实例先注册具体签名后检查体；自调用重绑到同型
/// 实例，具体体由真求解器检查（递归不逃实例化）。
#[test]
fn t04_template_self_recursion_same_instance() {
    let src = "\
fn pick<T>(n int, seed T) T {
    if n <= 0 { return seed }
    return pick(n - 1, seed)
}
fn main() int {
    return pick(2, 40)
}
";
    let p = compile(src).expect("recursive template compiles");
    assert_eq!(p.instances.len(), 1);
    assert_eq!(p.instances[0].concrete_name, "pick_i32");
    let defs = def_texts(&p);
    assert!(defs.iter().any(|d| d.contains("pick_i32")));
    // 递归体真实走完 emit+verify（无未解残留）。
    assert!(p.applied.iter().any(|a| a.name == "pick_i32"));
}

/// T-06（§5.5 caller/target 行）：模板 body-only 换版——普通 caller 只读
/// 具体签名（choose_i32 的签名轴不含 body），main 的 body 页 **hit、
/// 推导 0**；仅两个实例体重解（+2）；实现版本换新而调用引用（具体名）
/// 不变——不是把所有 caller 重算来掩盖绑定。
#[test]
fn t06_template_body_edit_caller_solve_zero() {
    let mut session = SourceQuerySession::new("plan935", "t06.at");
    let caps = caps();
    let p1 = compile_scalar_product(CHOOSE, "t06.at", &caps, "plan935", "t06.at", &mut session)
        .expect("cold");
    assert_eq!(p1.instances.len(), 2);
    let runs1 = session.counters().solver_runs;
    // 模板 body 换版（交换分支）；调用点与模板签名不动。
    let p2 = compile_scalar_product(CHOOSE_SWAPPED, "t06.at", &caps, "plan935", "t06.at", &mut session)
        .expect("recompile");
    assert_eq!(p2.instances.len(), 2);
    // 实例实现版本换新：choose_i32/choose_bool 的键变化。
    for (a, b) in p1.instances.iter().zip(p2.instances.iter()) {
        assert_ne!(a.key, b.key, "body version moves the instance key");
        assert_eq!(a.concrete_name, b.concrete_name, "assembly name stable");
        assert_eq!(a.sig_version, b.sig_version, "signature axis stable");
    }
    // 计量：恰两个实例体重解；main 推导 0。
    assert_eq!(
        session.counters().solver_runs,
        runs1 + 2,
        "only the two instance bodies re-solve; the caller never does"
    );
    // 暖 session 再编同源：0（全 hit）。
    let _p3 = compile_scalar_product(CHOOSE_SWAPPED, "t06.at", &caps, "plan935", "t06.at", &mut session)
        .expect("third");
    assert_eq!(session.counters().solver_runs, runs1 + 2);
}

/// T-06：调用者改 T（int 实例 → bool 实例）——该 caller 的实例集合需求
/// 变化：新实例生产、旧实例不再被该点需求（demands 计数迁移）。
#[test]
fn t06_caller_switches_t_demand_moves() {
    let int_call = "\
fn choose<T>(a T, b T, flag bool) T {
    if flag { return a }
    return b
}
fn main() int {
    let x = choose(40, 42, false)
    return x
}
";
    let bool_call = int_call.replace(
        "let x = choose(40, 42, false)
    return x",
        "if choose(true, false, true) {
        return 1
    }
    return 0",
    );
    let mut session = SourceQuerySession::new("plan935", "t06b.at");
    let caps = caps();
    let p1 = compile_scalar_product(int_call, "t06b.at", &caps, "plan935", "t06b.at", &mut session)
        .expect("int call");
    assert_eq!(p1.instances.len(), 1);
    assert_eq!(p1.instances[0].demands, 1);
    assert!(p1.instances[0].concrete_name.contains("i32"));
    let p2 = compile_scalar_product(&bool_call, "t06b.at", &caps, "plan935", "t06b.at", &mut session)
        .expect("bool call");
    assert_eq!(p2.instances.len(), 1);
    assert!(p2.instances[0].concrete_name.contains("bool"));
    assert_eq!(p2.instances[0].demands, 1);
    // 实例身份隔离：两个需求是不同键。
    assert_ne!(p1.instances[0].key, p2.instances[0].key);
}
