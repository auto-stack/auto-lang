// Plan 212 Phase 3D.1: FFI Dual-Test Infrastructure
//
// Tests that FFI functions produce consistent output through the AutoVM path.
// Each test reads input.at and compares stdout against expected_output.txt.

use crate::error::AutoResult;
use crate::run_with_capture;
use std::fs::read_to_string;
use std::path::PathBuf;

fn test_ffi_dual(case: &str) -> AutoResult<()> {
    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut src = read_to_string(d.join(format!("test/ffi_dual/{}/input.at", case)))?;
    // PLAN-592: dep 路径占位符——语料文件化,VM/a2r/oracle 三腿共享同一份 input.at
    let ffi_dual_dir = d.join("test/ffi_dual").to_string_lossy().replace('\\', "/");
    src = src.replace("{{FFI_DUAL_DIR}}", &ffi_dual_dir);
    let expected =
        read_to_string(d.join(format!("test/ffi_dual/{}/expected_output.txt", case)))?;

    let (_, stdout) = run_with_capture(&src)?;
    let trimmed = stdout.trim();
    let expected_trimmed = expected.trim();
    if trimmed != expected_trimmed {
        let wrong_path = d.join(format!("test/ffi_dual/{}/.wrong.out", case));
        std::fs::write(&wrong_path, &stdout)?;
    }
    assert_eq!(
        trimmed, expected_trimmed,
        "VM output mismatch for {}",
        case
    );
    Ok(())
}

// === FFI Dual Tests ===

#[test]
fn ffi_dual_001_file_exists() {
    test_ffi_dual("001_file_exists").unwrap();
}

#[test]
fn ffi_dual_002_str_operations() {
    test_ffi_dual("002_str_operations").unwrap();
}

#[test]
fn ffi_dual_003_json_encode_parse() {
    test_ffi_dual("003_json_encode_parse").unwrap();
}

#[test]
fn ffi_dual_004_math_abs() {
    test_ffi_dual("004_math_abs").unwrap();
}

#[test]
fn ffi_dual_005_url_parts() {
    test_ffi_dual("005_url_parts").unwrap();
}

#[test]
fn ffi_dual_006_regex_is_match() {
    test_ffi_dual("006_regex_is_match").unwrap();
}

#[test]
fn ffi_dual_007_path_join() {
    test_ffi_dual("007_path_join").unwrap();
}

#[test]
fn ffi_dual_008_json_array() {
    test_ffi_dual("008_json_array").unwrap();
}

#[test]
fn ffi_dual_009_json_keys() {
    test_ffi_dual("009_json_keys").unwrap();
}

#[test]
fn ffi_dual_010_env_get_set() {
    test_ffi_dual("010_env_get_set").unwrap();
}

#[test]
fn ffi_dual_011_char_operations() {
    test_ffi_dual("011_char_operations").unwrap();
}

#[test]
fn ffi_dual_012_str_find_replace() {
    test_ffi_dual("012_str_find_replace").unwrap();
}

// Plan 430 C2: 端到端 —— dep 声明的三方 crate 自动出方法 shim 包,
// VM 侧经 dispatch 3000 兜底段调用其方法(构造器/&mut/&self/静态/字符串/i64)。
// 依赖 nightly rustdoc 提取元信息 + cargo 编译 shim 包,任一缺失时跳过(非失败)。
#[test]
fn ffi_dual_013_dep_method() {
    if !auto_cache::methods_pack::nightly_available() {
        eprintln!("skipped: nightly toolchain unavailable for methods pack");
        return;
    }
    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture = d
        .join("test/ffi_dual/013_dep_method/fixture/autolang_counter")
        .to_string_lossy()
        .replace('\\', "/");
    let src = format!(
        r#"dep autolang_counter(path: "{fixture}")
use.rs autolang_counter::{{Counter, Config, Point}}
let c = Counter.new("hits")
c.increment()
c.increment()
print(c.value())
print(c.label())
c.set_label("misses")
print(c.label())
print(c.add(5))
let d = c.clone_reset()
print(d.value())
print(Counter.version())

let cfg = Config.new()
let c2 = cfg.verbose(true)
print(c2.is_verbose())
let c3 = c2.level(7)
print(c3.level_value())
print(cfg.is_verbose())
let m = c3.merge(cfg)
print(m.level_value())
let p = Config.parse("42")
print(p.level_value())

let pt = Point.new(3, 4, "origin")
print(pt.x())
print(pt.y())
print(pt.tag())
print(pt.to_string())
"#
    );
    let (_, stdout) = run_with_capture(&src).expect("run");
    let expected = "2\nhits\nmisses\n7\n7\n1.0.0\ntrue\n7\ntrue\n7\n42\n3\n4\norigin\n(3, 4) origin"; // Plan 474 待澄清#3: bool 显示形态 true/false
    assert_eq!(stdout.trim(), expected, "dep method e2e output mismatch:\n{stdout}");

    // unwrap_ok 错误传播:Result 构造失败 → VMError(带 cdylib 侧错误消息)
    let bad = format!(
        r#"dep autolang_counter(path: "{fixture}")
use.rs autolang_counter::{{Config}}
let p = Config.parse("not-a-number")
"#
    );
    let err = run_with_capture(&bad).expect_err("parse error must propagate");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("Config.parse") && msg.contains("invalid level"),
        "error should carry dep-side message, got: {msg}"
    );
}

// PLAN-592 T4: dep marshalling 全矩阵(三腿共享语料 016_dep_abi_matrix)。
// 主段走 test_ffi_dual({{FFI_DUAL_DIR}} 占位);附加段为 VM 侧特征化钉死
// (宽槽收窄 / u64 槽不往返,不入三腿语料——a2r 腿对超范围字面量编译不过)
// 与负面断言(arity 上限即刻可绿;未覆盖签名/未知字段在 T7/T8 收口后翻绿,TDD 先红)。
#[test]
fn ffi_dual_016_dep_abi_matrix() {
    if !auto_cache::methods_pack::nightly_available() {
        eprintln!("skipped: nightly toolchain unavailable for methods pack");
        return;
    }
    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture = d
        .join("test/ffi_dual/016_dep_abi_matrix/fixture/autolang_abi_matrix")
        .to_string_lossy()
        .replace('\\', "/");
    let matrix_src = |imports: &str, body: &str| {
        format!(
            r#"dep autolang_abi_matrix(path: "{fixture}")
use.rs autolang_abi_matrix::{{{imports}}}
{body}
"#
        )
    };

    // 主段:三腿共享 golden(字段访问 p.a 段在 T7 GET_FIELD 桥接落地前为已知红)
    test_ffi_dual("016_dep_abi_matrix").unwrap();

    // —— VM 特征化钉死(设计限制的行为记录,非三腿面)——
    // 宽槽收窄:i64 槽 1000 → wrapper `as u8` → 232(Rust `as` 语义,oracle 侧
    // 无法直写超范围字面量,故不入共享语料)
    let narrowing = matrix_src(
        "Num",
        r#"let n = Num.new()
print(n.echo_u8(1000))"#,
    );
    let (_, out) = crate::run_with_capture(&narrowing).expect("echo_u8 narrowing runs");
    assert_eq!(out.trim(), "232", "wide-slot narrowing (i64→u8 `as` cast)");

    // u64::MAX 经 i64 槽不往返(DIV-DEP-4 特征化:oracle=18446744073709551615,
    // VM 槽位呈现 -1——430 协议 v1 的 i64 槽限制)
    let u64_max = matrix_src(
        "Num",
        r#"let n = Num.new()
print(n.u64_max())"#,
    );
    let (_, out) = crate::run_with_capture(&u64_max).expect("u64_max runs");
    assert_eq!(out.trim(), "-1", "u64::MAX through i64 slot (DIV-DEP-4 pin)");

    // —— 负面断言 ——
    // arity 上限:接收者 + 3 参 = 4 ABI 参数 → RuntimeError(v1 supports ≤3)
    let quad = matrix_src(
        "Num",
        r#"let n = Num.new()
print(n.quad(1, 2, 3))"#,
    );
    let err = crate::run_with_capture(&quad).expect_err("4-ABI-param method must error");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("≤3") || msg.contains("<=3"),
        "arity limit error should mention v1 ≤3, got: {msg}"
    );

    // 未覆盖自由函数签名:此前 warn+静默 0,T8 收口为 VMError(先红后绿)。
    // let 绑定形态——print 参数路径会把裸 Ident 调用改写为构造器语义
    // (登记 DIV-DEP-5),标准路径经此形态进入。
    let uncovered = matrix_src(
        "free_uncovered",
        r#"let f = free_uncovered(2.5, "abcd")
print(f)"#,
    );
    let err = crate::run_with_capture(&uncovered)
        .expect_err("uncovered free-function signature must error after PLAN-592 T8");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("unsupported free-function signature"),
        "uncovered signature error should name the policy, got: {msg}"
    );

    // 未知字段:现状静默 0,T7 收口为报错(先红后绿);正确字段 p.a 同批翻绿
    let unknown_field = matrix_src(
        "Pt",
        r#"let p = Pt.new(9, "mid", -7)
print(p.nope)"#,
    );
    let err = crate::run_with_capture(&unknown_field)
        .expect_err("unknown field on dep object must error after PLAN-592 T7");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("unknown field"),
        "unknown field error should be explicit, got: {msg}"
    );
}

// PLAN-592 T5: dep 生命周期与 skip 面负面(三腿共享语料 017_dep_lifecycle,
// fixture 复用 013 扩展件:drop_count 自由函数 + Drop 计数器)。
// 正面:chain(ChainInPlace)同句柄语义、clone_reset 独立性、drop_count 基线
// (自由函数 ()→u64 冒烟);负面:classify skip 面(Option 返回/按值 self)在 VM 侧
// 必须显式报 "Unknown Rust stdlib call",不得静默返 0(DIV-DEP-1/2)。
#[test]
fn ffi_dual_017_dep_lifecycle() {
    if !auto_cache::methods_pack::nightly_available() {
        eprintln!("skipped: nightly toolchain unavailable for methods pack");
        return;
    }
    test_ffi_dual("017_dep_lifecycle").unwrap();

    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture = d
        .join("test/ffi_dual/013_dep_method/fixture/autolang_counter")
        .to_string_lossy()
        .replace('\\', "/");
    let counter_src = |imports: &str, body: &str| {
        format!(
            r#"dep autolang_counter(path: "{fixture}")
use.rs autolang_counter::{{{imports}}}
{body}
"#
        )
    };

    // Option 返回(maybe):v1 分类跳过 → 显式 Unknown,非静默 0(DIV-DEP-1)
    let maybe = counter_src(
        "Counter",
        r#"let c = Counter.new("x")
print(c.maybe())"#,
    );
    let err = crate::run_with_capture(&maybe).expect_err("Option-return method is skipped in v1");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("Unknown Rust stdlib call"),
        "skipped (Option-return) method must error explicitly, got: {msg}"
    );

    // 按值 self(bump):v1 分类跳过 → 显式 Unknown(DIV-DEP-2;move marshaller 无覆盖)
    let bump = counter_src(
        "Config",
        r#"let c = Config.new()
print(c.bump().level_value())"#,
    );
    let err = crate::run_with_capture(&bump).expect_err("by-value self method is skipped in v1");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("Unknown Rust stdlib call"),
        "skipped (by-value self) method must error explicitly, got: {msg}"
    );
}

// PLAN-596 T-03/T-08: 591 V2——trait 白名单转发(Clone)/既有 Display 合成/
// V1×V2 组合(make→to_string+字段读/clone 深拷贝独立性+字段写)。
// V2-3/V2-4(pick/pick_max 泛型 mono)与 V2-6(回调 adapter)随 T-04/T-05 增补。
#[test]
fn ffi_dual_020_dep_traits_generics() {
    if !auto_cache::methods_pack::nightly_available() {
        eprintln!("skipped: nightly toolchain unavailable for methods pack");
        return;
    }
    test_ffi_dual("020_dep_traits_generics").unwrap();
}

// Plan 430 复审补网:std 臂 VM 路径回归网。
// 守护 dispatch 3000 生成段(generated_std.rs):Vec 14 臂/Duration 5 臂/
// Instant 2 臂/PathBuf.from/String.new|from。复审发现 430 迁移 std 手写臂后,
// VM 模式的 19_rust_std goldens 当时全部 #[ignore],迁移主战场零活跃守护——
// 本用例补上;19_rust_std 的陈旧 ignore 亦已解除。
#[test]
fn ffi_dual_015_musk_backend_wave1() {
    test_ffi_dual("015_musk_backend_wave1").unwrap();
}

#[test]
fn ffi_dual_014_std_generated_segment() {
    test_ffi_dual("014_std_generated_segment").unwrap();
}

// 跨测试路由污染回归(ffi_dual_014 发现,2026-08-25):BIGVM_NATIVES 惰性注册 +
// "已有 native 优先"启发式,使 use.rs 的 String.from 路由取决于同进程内是否有
// 先前程序用过原生 String API。本测试在**同一测试内**先跑原生(无 use.rs)
// String.from——修复前第二条会 print 出裸堆 ID(4000011 形态)而非 "42"。
#[test]
fn ffi_dual_015_rust_type_route_not_hijacked_by_native_registry() {
    // 1. 原生 String API(无 use.rs)——副作用:auto.str.from 惰性注册进全局表
    let (_, out1) = crate::run_with_capture(r#"
fn main() {
    let s = String.from("native")
    print(s.len())
}
"#).expect("native String.from runs");
    assert_eq!(out1.trim(), "6");

    // 2. use.rs 的 String.from——必须仍走 dispatch 3000 生成段,
    //    不得被已注册的 auto.str.from 劫持
    let (_, out2) = crate::run_with_capture(r#"
use.rs std::string::String
fn main() {
    print(String.from("42"))
}
"#).expect("rust String.from runs");
    assert_eq!(out2.trim(), "42", "rust type import must not be hijacked by lazily-registered auto.str native");
}

// PLAN-591 T3/T7: V1 布局/语义语料(三腿共享 018_dep_fields,fixture autolang_shapes)。
// 主段走 test_ffi_dual:V1-4 Option nullable(hit/miss == null 三轨一致)、
// V1-5 Result fallible(合法输入 .unwrap() 后字段可读)、V1-3 嵌套两级
// (o.inner.n / o.inner.label)、V1-7 enum 判别(is_circle 等匹配语义探针)。
// 附加段为 VM 腿负面断言(错误输出无法进 stdout golden,592 惯例):
// Result Err → VMError 含 fixture 固定锚文案;None 后 .unwrap() → VMError。
#[test]
fn ffi_dual_018_dep_fields() {
    if !auto_cache::methods_pack::nightly_available() {
        eprintln!("skipped: nightly toolchain unavailable for methods pack");
        return;
    }
    test_ffi_dual("018_dep_fields").unwrap();

    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture = d
        .join("test/ffi_dual/018_dep_fields/fixture/autolang_shapes")
        .to_string_lossy()
        .replace('\\', "/");
    let shapes_src = |imports: &str, body: &str| {
        format!(
            r#"dep autolang_shapes(path: "{fixture}")
use.rs autolang_shapes::{{{imports}}}
{body}
"#
        )
    };

    // V1-5 负面:parse 非法输入 → wrapper Err 写错误通道 → VMError 含锚文案
    let bad_parse = shapes_src(
        "Point",
        r#"let p = Point.parse("bad").unwrap()
print(p.x)"#,
    );
    let err = crate::run_with_capture(&bad_parse)
        .expect_err("Result Err must surface as VMError (591 V1-5)");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("bad-input"),
        "Result Err message should carry fixture anchor, got: {msg}"
    );

    // None 表达注记(V1-4):VM 侧 T2 以 null 表达 None,语料以 `== null` 断言
    // (三轨一致:VM null ≡ a2r None ≡ oracle is_none)。null 接收者的方法调用
    // 走 VM 既有 musk/Python parity 惯例(null.method() → "None" 文本),早于本
    // 计划且非 dep 面,不在此改动——登记为既有语义观察。

    // 标量槽 nullable 显式跳过(哨兵歧义,V1 已知限制):Option<i64> 面不进包
    // (fixture 无该面;017 的 maybe(-> Option<i64>) 负面断言已覆盖同一守卫路径)
}

// PLAN-591 T8: 布局不变量对抗测试(019)。
// 对抗①孪生 fixture:autolang_shapes_b 与 autolang_shapes 的 Messy 同类型名/
// 同方法签名集,异字段声明序 + 异哨兵(a=9/b=13 vs a=7/b=11)——连续装载下
// offset 直读必须各自读回各自真值;布局信息混淆(陈旧 pack 复用/layouts 串键)
// 即当场暴露。对抗②features 变体:FeatCfg.width 字段型随 features 组合切换
// (i32/i64),同构造实参在宽窄变体呈现不同值;指纹含 features 行(排序等价),
// 异变体必异指纹 → 各自重建。V1-6 的同 crate 源码变更 stale 防护由
// auto-cache source-hash 通道单测覆盖(path_source_staleness_detection)。
#[test]
fn ffi_dual_019_dep_layout_invariants() {
    if !auto_cache::methods_pack::nightly_available() {
        eprintln!("skipped: nightly toolchain unavailable for methods pack");
        return;
    }
    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let ffi = d.join("test/ffi_dual").to_string_lossy().replace('\\', "/");
    let src_of = |dep_line: &str, imports: &str, body: &str| {
        format!("{dep_line}\nuse.rs {imports}\nfn main() {{\n{body}\n}}\n")
    };

    // 对抗②(features):窄变体(无 features)——width: i32 对 5000000000 截断
    let narrow = src_of(
        &format!(
            "dep autolang_shapes(path: \"{ffi}/018_dep_fields/fixture/autolang_shapes\")"
        ),
        "autolang_shapes::{FeatCfg}",
        r#"    let f = FeatCfg.new(5000000000, "narrow")
    print(f.width)"#,
    );
    let (_, out) = crate::run_with_capture(&narrow).expect("FeatCfg narrow runs");
    assert_eq!(out.trim(), "705032704", "narrow variant: i32 wraparound pin");

    // 对抗②(features):宽变体(features: ["wide"])——width: i64 全值保留。
    // 若异 features 共用陈旧 pack(指纹未含 features),此处会拿到 705032704。
    let wide = src_of(
        &format!(
            "dep autolang_shapes(path: \"{ffi}/018_dep_fields/fixture/autolang_shapes\", features: [\"wide\"])"
        ),
        "autolang_shapes::{FeatCfg}",
        r#"    let f = FeatCfg.new(5000000000, "wide")
    print(f.width)"#,
    );
    let (_, out) = crate::run_with_capture(&wide).expect("FeatCfg wide runs");
    assert_eq!(out.trim(), "5000000000", "wide variant: i64 full value pin");

    // 对抗①(孪生):先 _b 后 shapes 连续装载,各自读回各自哨兵。
    // 若两 crate 的 layouts/方法面混淆,offset 直读返回对方真值。
    let twin_b = src_of(
        &format!(
            "dep autolang_shapes_b(path: \"{ffi}/019_dep_layout_invariants/fixture/autolang_shapes_b\")"
        ),
        "autolang_shapes_b::{Messy}",
        r#"    let m = Messy.new(1000, "beta", false)
    print(m.a)
    print(m.b)
    print(m.total)
    print(m.flag)"#,
    );
    let (_, out) = crate::run_with_capture(&twin_b).expect("twin _b runs");
    assert_eq!(
        out.trim(),
        "9\n13\n1000\nfalse",
        "twin _b truth (异字段序偏移正确性)"
    );

    let twin_a = src_of(
        &format!(
            "dep autolang_shapes(path: \"{ffi}/018_dep_fields/fixture/autolang_shapes\")"
        ),
        "autolang_shapes::{Messy}",
        r#"    let m = Messy.new(1000, "alpha", true)
    print(m.a)
    print(m.b)
    print(m.total)
    print(m.flag)"#,
    );
    let (_, out) = crate::run_with_capture(&twin_a).expect("twin a runs");
    assert_eq!(
        out.trim(),
        "7\n11\n1000\ntrue",
        "twin a truth(与 _b 同进程先后装载,布局各归各)"
    );
}
