//! PLAN-964（T-02/T-08，AV 端）：动态值对象反射与通用闭包的 AV 真实验证。
//!
//! - 对象（L-16/L-20）：裸对象字面量 `{ x: 100 }` + 动态字符串键读写
//!   `obj["y"]=200`/`obj[k]=300`（map_bracket_write 旧语料 → 和 600）。
//! - nil（L-21）：nil 字面量 + nil==nil 常数化 + 对象别名共享写可见。
//! - 可调用值（L-22/L-23）：对象持闭包，`obj["twice"](3)` 动态派发。
//! - 通用闭包捕获（source02）：factor=3 捕获 + HOF，核 [3,6,9]（369 编码）。
//! - FStr（L-22）：插值表达式源序求值恰一次 + i32 显示（fstring 语料 → 42）。
//! - 负控：缺失键 trap `av.dyn.object-missing`；异型字段 `source.object-elem`；
//!   nil 运算 `source.nil-op`。
//!
//! oracle 全部为手算闭式，独立于实现。

use ace_av_core::abc::{decode_abc, encode_abc};
use ace_av_core::av::{execute, CancelFlag, ExecLimits};
use ace_av_core::contract::{ExecOutcome, Value, AV_DATA_PROFILE};
use ace_av_core::{abc_verify, mir_verify};
use ace_core::source::{compile_aggregates_source, compile_closures_source};
use ace_core::verify::CheckedModule;
use ace_av::closures_session::ClosuresUpdateSession;

fn checked_aggregates(src: &str) -> CheckedModule {
    compile_aggregates_source(src, "p964.at", &[])
        .expect("aggregates source compiles")
        .checked
}

fn checked_closures(src: &str) -> CheckedModule {
    compile_closures_source(src, "p964-closures.at", &[])
        .expect("closures source compiles")
        .checked
}

fn run_data(src: &str) -> Value {
    let checked = checked_aggregates(src);
    let mir = ace_lowering::lower_checked_closures(&checked, "d_main", 964)
        .expect("aggregates lowering");
    mir_verify::verify_mir(&mir).expect("MIR verifies");
    let wire = encode_abc(&mir).expect("AVAD encode").to_flat();
    let decoded = decode_abc(&wire).expect("AVAD decode");
    let loaded = abc_verify::verify_abc(decoded, &mir.owner).expect("AVAD admission");
    let (out, _trace) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    let ExecOutcome::Success { value, .. } = out else {
        panic!("expected success, got {out:?}")
    };
    value
}

fn run_data_trap(src: &str) -> ace_av_core::contract::TrapKind {
    let checked = checked_aggregates(src);
    let mir = ace_lowering::lower_checked_closures(&checked, "d_main", 964)
        .expect("aggregates lowering");
    mir_verify::verify_mir(&mir).expect("MIR verifies");
    let wire = encode_abc(&mir).expect("AVAD encode").to_flat();
    let decoded = decode_abc(&wire).expect("AVAD decode");
    let loaded = abc_verify::verify_abc(decoded, &mir.owner).expect("AVAD admission");
    let (out, _trace) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    let ExecOutcome::Trap { kind, .. } = out else {
        panic!("expected a trap, got {out:?}")
    };
    kind
}

/// The closures session path (live_after_release assertion — V-05/V-08).
fn run_closures(src: &str) -> (Option<i32>, u64) {
    let checked = checked_closures(src);
    let bundle = checked.bundle();
    let units: Vec<(String, ace_av_core::mir::MirModule)> = bundle
        .modules
        .iter()
        .flat_map(|module| module.defs.iter())
        .filter(|def| matches!(def.kind, ace_core::hir::DefKind::Function { .. }))
        .map(|def| {
            let unit = ace_lowering::lower_function_closures_unit(bundle, &def.id_text, &[], 1)
                .unwrap_or_else(|error| panic!("lower {}: {error:?}", def.id_text));
            (def.id_text.clone(), unit)
        })
        .collect();
    let submitted: Vec<(&str, ace_av_core::mir::MirModule)> =
        units.iter().map(|(n, m)| (n.as_str(), m.clone())).collect();
    let mut session = ClosuresUpdateSession::new(None, "d_main");
    let outcome = session
        .apply_revision(&submitted)
        .expect("source closure units link and execute");
    assert_eq!(outcome.live_after_release, 0, "last owner releases (V-05)");
    assert_eq!(outcome.allocs, outcome.frees);
    assert!(outcome.gate.pass());
    session.close().expect("session roots close cleanly");
    (outcome.value, outcome.live_after_release)
}

// ---------------------------------------------------------------------------
// 对象（L-16/L-20）
// ---------------------------------------------------------------------------

const MAP_BRACKET_WRITE: &str = "\
fn main() int {
    var obj = { x: 100 }
    obj[\"y\"] = 200
    var k = \"z\"
    obj[k] = 300
    return obj[\"y\"] + obj[k] + obj[\"x\"]
}
";

/// map_bracket_write 旧语料：y=200/z=300/x=100 → 和 600。
#[test]
fn av_object_map_bracket_write_600() {
    assert_eq!(
        run_data(MAP_BRACKET_WRITE),
        Value::I32(600),
        "map_bracket_write oracle"
    );
}

const NIL_AND_ALIAS: &str = "\
fn main() int {
    var o1 = { n: 10 }
    let o2 = o1
    o2[\"n\"] = 20
    let seen = o1[\"n\"]
    let n = nil
    let same = n == nil
    if seen != 20 { return 0 }
    if !same { return 0 }
    return o1[\"n\"]
}
";

/// 对象别名（Rc 共享）写经任一名可见（AC-06 共享单元格）+ nil 常数化。
#[test]
fn av_object_alias_shared_write_visible_20() {
    assert_eq!(
        run_data(NIL_AND_ALIAS),
        Value::I32(20),
        "alias write visible through the other name"
    );
}

const OBJECT_MISSING: &str = "\
fn main() int {
    var obj = { x: 1 }
    return obj[\"missing\"]
}
";

/// 缺失键读 = 稳定运行期拒绝（av.dyn.object-missing）——V-04。
#[test]
fn av_object_missing_key_traps() {
    let kind = run_data_trap(OBJECT_MISSING);
    assert_eq!(kind, ace_av_core::contract::TrapKind::ObjectMissing);
}

#[test]
fn av_object_mixed_fields_reject_source_object_elem() {
    let diags = compile_aggregates_source(
        "fn main() int { var obj = { x: 1, y: \"text\" }; return obj[\"x\"] }",
        "p964.at",
        &[],
    )
    .expect_err("mixed object fields must be rejected");
    assert!(
        diags.iter().any(|d| d.code == "source.object-elem"),
        "expected source.object-elem, got {diags:#?}"
    );
}

#[test]
fn av_nil_arithmetic_reject_source_nil_op() {
    let diags =
        compile_aggregates_source("fn main() int { let n = nil; return n + 1 }", "p964.at", &[])
            .expect_err("nil arithmetic must be rejected");
    assert!(
        diags.iter().any(|d| d.code == "source.nil-op"),
        "expected source.nil-op, got {diags:#?}"
    );
}

// ---------------------------------------------------------------------------
// FStr（L-22）
// ---------------------------------------------------------------------------

const FSTRING: &str = "\
fn main() int {
    let name = \"AutoLang\"
    let age = 1
    let greeting = f\"Hello, $name\"
    let info = f\"Name: $name, Age: $age\"
    let x = 10
    let y = 20
    let result = f\"Result: ${x + y}\"
    let msg = f\"The value is $x and ${y * 2}\"
    let a = 5
    let b = 3
    let sum = f\"Sum: ${a + b}, Product: ${a * b}\"
    if greeting != \"Hello, AutoLang\" { return 0 }
    if info != \"Name: AutoLang, Age: 1\" { return 0 }
    if result != \"Result: 30\" { return 0 }
    if msg != \"The value is 10 and 40\" { return 0 }
    if sum != \"Sum: 8, Product: 15\" { return 0 }
    return 42
}
";

/// fstring 语料：插值源序 + i32 显示 → 42。
#[test]
fn av_fstring_interpolation_executes_42() {
    assert_eq!(run_data(FSTRING), Value::I32(42), "fstring oracle");
}

const FSTR_TWO_SIDE_EFFECTS: &str = "\
fn main() int {
    let log = List<i32>.new()
    let s = f\"${mark(log, 1)} ${mark(log, 2)}\"
    if s != \"1 2\" { return 0 }
    if log.len() != 2 { return 0 }
    if log.get(0) != 1 { return 0 }
    if log.get(1) != 2 { return 0 }
    return 42
}
fn mark(log List<i32>, v int) int {
    log.push(v)
    return v
}
";

/// V-08：两个有副作用的插值按源序各求值一次（TextConcat 链左结合）。
#[test]
fn av_fstr_side_effectful_parts_source_order_once_each() {
    assert_eq!(
        run_data(FSTR_TWO_SIDE_EFFECTS),
        Value::I32(42),
        "each interpolation evaluates exactly once in source order"
    );
}

// ---------------------------------------------------------------------------
// 闭包（L-22）
// ---------------------------------------------------------------------------

const CLOSURE_CAPTURE_HOF: &str = "\
fn main() int {
    let factor = 3
    let nums = List<i32>.new()
    nums.push(1)
    nums.push(2)
    nums.push(3)
    let scaled = List<i32>.new()
    for x in nums {
        scaled.push(((y int) => y * factor)(x))
    }
    if scaled.len() != 3 { return 0 }
    return scaled.get(0) * 100 + scaled.get(1) * 10 + scaled.get(2)
}
";

/// source02：factor=3 捕获 + HOF → [3,6,9] 编码 369（非只 len）。
#[test]
fn av_closure_capture_hof_values_369() {
    let (value, _) = run_closures(CLOSURE_CAPTURE_HOF);
    assert_eq!(value, Some(369), "mapped [3,6,9] oracle");
}

const OBJECT_CALLABLE: &str = "\
fn main() int {
    var fns = { twice: (y int) => y * 2, half: (y int) => y / 2 }
    let a = fns[\"twice\"](3)
    let b = fns[\"half\"](a)
    return a + b
}
";

/// 对象持闭包 + 动态键派发（callable/member/index 派发）→ 9。
#[test]
fn av_object_callable_dynamic_dispatch_9() {
    let (value, _) = run_closures(OBJECT_CALLABLE);
    assert_eq!(value, Some(9), "callable dispatch oracle");
}

const SHARED_OBJECT_TWO_CLOSURES: &str = "\
fn main() int {
    var cell = { n: 10 }
    let bump = move () => { cell[\"n\"] = cell[\"n\"] + 1 }
    let read = move () => cell[\"n\"]
    bump()
    bump()
    return read()
}
";

/// V-03/V-05：两闭包共享一个对象单元格，更新互相可见；
/// 最后 owner 释放（live_after_release==0 已由 run_closures 断言）。
#[test]
fn av_two_closures_share_object_cell_visible_12() {
    let (value, live) = run_closures(SHARED_OBJECT_TWO_CLOSURES);
    assert_eq!(value, Some(12), "shared object cell increments visible");
    assert_eq!(live, 0, "last owner releases the shared cell");
}
