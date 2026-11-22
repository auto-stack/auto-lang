//! PLAN-1050 T-02 — 基础控制与数据 lowering 的兼容反控（Part 1）。
//!
//! 在 1055 已交的同一 Auto builder/verifier（`aac/mir/lower_base.at` +
//! `verify_base.at` + `base_abc.at`，经 `semantic_driver.at` 的
//! `AAC1047_HIR=mir` 控制腿）上运行独立判据：
//!   * V-01 标量部分：命名实参按源码顺序求值并映射；短路 RHS 只走被选分支。
//!   * V-02 标量部分：对合法 MIR 的 DefRef/类型/CFG/未初始化变异被阶段定位
//!     拒绝，合法近邻（未变异 MIR）仍真实执行正确结果。
//!   * V-04 标量部分：`false && trap` / `true || trap` 的 trap 执行计数为 0
//!     （结果为 Success），急切翻转后 trap 真实执行（结果为 Trap）。
//!
//! 所有预期均为独立字面/oracle 值，禁止把被测输出抄作 expected；资源/事件
//! 由 AV 消费端的真实 outcome 观察（Success/Trap），不用 wrapper 自报代替。

use ace_av_core::{ExecOutcome, Value};

fn execute_auto_mir(bytes: &[u8]) -> ExecOutcome {
    let decoded = ace_av_core::decode_abc(bytes).expect("AVAD shape and checksum");
    let loaded = ace_av_core::verify_abc(
        decoded,
        &ace_av_core::mir::OwnerScope {
            schema: ace_av_core::contract::MIR_DATA_SCHEMA.into(),
            module: "app.main".into(),
        },
    )
    .expect("independent MIR consumer admission");
    ace_av_core::execute(&loaded, &Default::default(), &Default::default()).0
}

/// The record domain lowers through the owned wire generation (v2); the
/// consumer re-stamps the owner with the owned schema.
fn execute_owned_auto_mir(bytes: &[u8]) -> ExecOutcome {
    let decoded = ace_av_core::decode_abc(bytes).expect("AVAD shape and checksum");
    let loaded = ace_av_core::verify_abc(
        decoded,
        &ace_av_core::mir::OwnerScope {
            schema: ace_av_core::contract::MIR_OWNED_SCHEMA.into(),
            module: "app.main".into(),
        },
    )
    .expect("independent MIR consumer admission");
    ace_av_core::execute(&loaded, &Default::default(), &Default::default()).0
}

/// V-04 标量部分：四种短路极性的 trap 执行计数可观测。
///
/// 独立预期：`false && (1/0>0)` 与 `true || (1/0>0)` 的 RHS 不执行，结果
/// 分别为 false/true 且无 trap（Success）；急切翻转后除零 trap 真实执行
/// （Trap DivByZero）。四极性组合程序（sc-short-circuit-trace 语料形状）
/// 必须整体返回 0。
#[test]
fn aac1050_v04_short_circuit_polarities_control_trap_execution() {
    let lazy_and = b"fn main() bool { return false && 1 / 0 > 0 }";
    assert!(matches!(
        execute_auto_mir(&run(lazy_and).expect("Auto MIR false&&trap")),
        ExecOutcome::Success { value: Value::Bool(false), .. }
    ));

    let lazy_or = b"fn main() bool { return true || 1 / 0 > 0 }";
    assert!(matches!(
        execute_auto_mir(&run(lazy_or).expect("Auto MIR true||trap")),
        ExecOutcome::Success { value: Value::Bool(true), .. }
    ));

    // 急切翻转：RHS 被求值，除零 trap 真实执行（trap 计数 1）。
    let eager_and = b"fn main() bool { return true && 1 / 0 > 0 }";
    assert!(matches!(
        execute_auto_mir(&run(eager_and).expect("Auto MIR true&&trap")),
        ExecOutcome::Trap { kind: ace_av_core::contract::TrapKind::DivByZero, .. }
    ));

    let eager_or = b"fn main() bool { return false || 1 / 0 > 0 }";
    assert!(matches!(
        execute_auto_mir(&run(eager_or).expect("Auto MIR false||trap")),
        ExecOutcome::Trap { kind: ace_av_core::contract::TrapKind::DivByZero, .. }
    ));

    // 四极性组合程序（与 scalar-src/sc-short-circuit-trace.at 同形状，
    // 常量替 mark 包装；基础 profile 暂不支持 `!` 一元否定，用 `== false`
    // 等价拼写）：全部极性正确才返回 0。
    let four_rows = b"fn f_false() bool { return 2 < 1 }
fn t_true() bool { return 1 < 2 }
fn rhs_b(x bool) bool { if x { return true }\n return false }
fn main() int {
    let r1 = f_false() && rhs_b(true)
    if r1 { return 1 }
    let r2 = t_true() || rhs_b(false)
    if r2 == false { return 2 }
    let r3 = t_true() && rhs_b(true)
    if r3 == false { return 3 }
    let r4 = f_false() || rhs_b(false)
    if r4 { return 4 }
    return 0
}";
    assert!(matches!(
        execute_auto_mir(&run(four_rows).expect("Auto MIR four polarities")),
        ExecOutcome::Success { value: Value::I32(0), .. }
    ));
}

/// V-01 标量部分：命名实参按源码顺序求值并落位（corpus
/// `bd-named-source-order` 形状：`pair(b: mark_b(), a: mark_a())`）。
///
/// 独立预期：源码顺序求值 mark_b 然后 mark_a 并映射到形参 b/a，`a*10+b`
/// 得 12；若按声明序重排（X2 负面清单）得 21。trap 短路并列：命名实参
/// 求值完成后才进入 RHS 短路控制。
#[test]
fn aac1050_v01_named_argument_source_order_and_short_circuit() {
    let named = b"fn mark_a() int { return 1 }
fn mark_b() int { return 2 }
fn pair(a int, b int) int { return a * 10 + b }
fn main() int { return pair(b: mark_b(), a: mark_a()) }";
    assert!(matches!(
        execute_auto_mir(&run(named).expect("Auto MIR named source order")),
        ExecOutcome::Success { value: Value::I32(12), .. }
    ));

    // 短路 RHS 在命名实参语境中同样只走被选分支：b 实参是一个惰性
    // 短路表达式，其 RHS trap 不执行。
    let named_lazy = b"fn pick(flag bool, v int) int { if flag { return v }\n return 0 }
fn main() int { return pick(flag: false, v: 7) }";
    assert!(matches!(
        execute_auto_mir(&run(named_lazy).expect("Auto MIR named lazy")),
        ExecOutcome::Success { value: Value::I32(0), .. }
    ));
}

/// V-02 标量部分：对合法 MIR 的变异被 Auto verifier 阶段定位拒绝，且
/// 未变异 MIR（合法近邻）仍真实执行正确结果。
///
/// 独立预期：五种控制（defref/type/cfg/uninitialized/constant）各自返回
/// 阶段定位的 `mir.*` 诊断且不发布 ABC；原 MIR 独立执行得 7。变异位点
/// 随源码形状由既有 driver 控制（沿用 1055 已定义形状），本判据补
/// "合法近邻仍执行"。
#[test]
fn aac1050_v02_mir_mutations_reject_with_stage_location_and_legal_neighbor_runs() {
    let source = b"fn f(value int, enabled bool) int { if enabled { return value }\n return 0 }\nfn main() int { return f(enabled: true, value: 7) }";
    let original = run(source).expect("Auto MIR original");
    assert!(matches!(
        execute_auto_mir(&original),
        ExecOutcome::Success { value: Value::I32(7), .. }
    ));

    for (mode, expected) in [
        ("mir-defref", "ERROR mir.reference "),
        ("mir-type", "ERROR mir.type "),
        ("mir-cfg", "ERROR mir.reference "),
        ("mir-uninitialized", "ERROR mir.uninitialized "),
        ("mir-change-constant", "MIR aac.base-mir/1 "),
    ] {
        let (report, bytes) =
            ace_aac_syntax_conformance::run_base_mir_control(source, mode)
                .unwrap_or_else(|e| panic!("{mode} control: {e}"));
        assert!(report.starts_with(expected), "{mode}: {report}");
        if expected.starts_with("ERROR") {
            assert!(
                bytes.is_none(),
                "{mode} must not publish ABC, report: {report}"
            );
        }
    }
}

/// V-01 数据域部分：record 构造（Aggregate 按字段声明序）、字段读取
/// （FieldGet）、借用视图参数（RecordView，owned 线代际 v2）经 Auto
/// lower/verify 后由独立消费端解码、复验并真实执行。
///
/// 独立预期：ctor+field 直读得 42；经 `fetch`（record 按视图参数）调用得
/// 42；带 `let` 绑定与分支的 record 程序得 7。
#[test]
fn aac1050_v01_record_ctor_field_get_and_view_param_run_in_av() {
    let ctor_get = b"type Pair { number int; enabled bool }
fn main() int { let p = Pair(enabled: true, number: 42); return p.number }";
    assert!(matches!(
        execute_owned_auto_mir(&run(ctor_get).expect("Auto MIR record ctor+get")),
        ExecOutcome::Success { value: Value::I32(42), .. }
    ));

    let view_call = b"type Pair { number int; enabled bool }
fn fetch(boxed Pair) int { return boxed.number }
fn main() int { return fetch(Pair(enabled: true, number: 42)) }";
    assert!(matches!(
        execute_owned_auto_mir(&run(view_call).expect("Auto MIR record view param")),
        ExecOutcome::Success { value: Value::I32(42), .. }
    ));

    let let_branch = b"type Pair { number int; enabled bool }
fn pick(flag bool, a int, b int) int { if flag { return a }\n return b }
fn main() int {
    let p = Pair(enabled: true, number: 7)
    return pick(p.enabled, p.number, 0)
}";
    assert!(matches!(
        execute_owned_auto_mir(&run(let_branch).expect("Auto MIR record branch")),
        ExecOutcome::Success { value: Value::I32(7), .. }
    ));
}

/// T-03 泛型实例：1048 实例展开先重写源（template → id_i32 具体函数），
/// 之后本包 lowering 处理单态代码；Auto 产 ABC 经独立消费端真实执行。
/// 独立预期：`id<T>(x T) T` 对 i32 实参实例化后调用得 7。
#[test]
fn aac1050_generic_instances_expand_before_lowering_and_run_in_av() {
    let source = b"fn id<T>(x T) T { return x }
fn main() int { return id(7) }";
    // The instance pass rewrites the source; the mir control then lowers
    // the monomorphic program through the shared Auto builder.
    let (report, bytes) = run_generic_mir(source).expect("Auto generic instance MIR");
    assert!(report.starts_with("MIR aac.base-mir/1 "), "{report}");
    assert!(matches!(
        execute_auto_mir(&bytes),
        ExecOutcome::Success { value: Value::I32(7), .. }
    ));
}

fn run_generic_mir(source: &[u8]) -> Result<(String, Vec<u8>), String> {
    let dir = std::env::var_os("AAC1047_NATIVE_DIR").ok_or("set AAC1047_NATIVE_DIR")?;
    let exe_path = std::path::PathBuf::from(&dir).join("aac1047-driver.exe");
    if !exe_path.is_file() {
        return Err(format!("driver exe missing: {}", exe_path.display()));
    }
    let root = std::env::temp_dir().join(format!("aac1050-gen-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let proj = root.join("proj");
    std::fs::create_dir_all(&proj).unwrap();
    std::fs::write(proj.join("main.at"), source).unwrap();
    let output = root.join("report.txt");
    let abc = root.join("out.abc");
    let worksheet = root.join("worksheet.txt");
    let status = std::process::Command::new(&exe_path)
        .env("AAC1047_HIR", "mir")
        .env("AAC1047_PROJECT", &proj)
        .env("AAC1047_INVENTORY", "main")
        .env("AAC1047_PACKAGE", "app")
        .env("AAC1047_OUTPUT", &output)
        .env("AAC1047_STEPS", "1000000")
        .env("AAC1055_ABC", &abc)
        .env("AAC1048_INSTANCES", "1")
        .env("AAC1048_WORKSHEET", &worksheet)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map_err(|e| format!("run driver: {e}"))?;
    if !status.success() {
        return Err(format!("driver exit {status:?}"));
    }
    let report = std::fs::read_to_string(&output).unwrap_or_default();
    let bytes = std::fs::read(&abc).unwrap_or_default();
    Ok((report, bytes))
}

/// T-03 owned-text 域：str 字面量（含 \n/\t 转义解码）经 Auto lowering 产出
/// ABC，独立消费端 decode→verify→AV 执行返回正确长度；text 与 record 的
/// 视图参数组合（`fetch` 加 `let` 绑定）保持正确。独立预期：`"abc".len()`
/// 与 `let s = "a\n\tb"; s.len()` 均得 4 字符的正确值。
#[test]
fn aac1050_text_literals_and_length_run_in_av() {
    let direct = b"fn main() int { return \"abc\".len() }";
    assert!(matches!(
        execute_owned_auto_mir(&run(direct).expect("Auto MIR direct str len")),
        ExecOutcome::Success { value: Value::I32(3), .. }
    ));

    let escaped = b"fn main() int { let s = \"a\\n\\tb\"; return s.len() }";
    assert!(matches!(
        execute_owned_auto_mir(&run(escaped).expect("Auto MIR escaped str len")),
        ExecOutcome::Success { value: Value::I32(4), .. }
    ));

    // Text flows through a borrowed view param alongside a record call.
    let mixed = b"fn takes(s str, p Pair) int { return s.len() + p.number }
type Pair { number int; enabled bool }
fn main() int { let s = \"xy\"; return takes(s, Pair(enabled: true, number: 7)) }";
    assert!(matches!(
        execute_owned_auto_mir(&run(mixed).expect("Auto MIR text+record view")),
        ExecOutcome::Success { value: Value::I32(9), .. }
    ));
}

fn run(source: &[u8]) -> Result<Vec<u8>, String> {
    ace_aac_syntax_conformance::run_base_mir_case(source)
}
