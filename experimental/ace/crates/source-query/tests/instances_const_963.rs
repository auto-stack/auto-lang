//! PLAN-963 T-04 / V-G04：值参数（`<const N>`）的实例身份、去重与拒绝。
//!
//! 覆盖：
//! - 合法形 `<const N>` 与 `<const N u32>`（声明类型缺省 u32）；
//! - `rep<2>(21)` 的值实参进入**实例身份**：N=2/N=3 是两个不同实例（不同
//!   键、不同装配名），同值调用点复用同一实例（同键，需求计数累加）；
//! - 值参数在模板体里被代入为字面量（`N * x` → `2 * x`；体里没有名为 `N`
//!   的运行时变量）；
//! - 值实参与类型参数混排时按**声明序**成身份；
//! - 拒绝：缺值实参、值实参过多、值给无值参数模板、值域越界、非 u32 声明
//!   类型。
//!
//! 真实运行 oracle（`rep<2>(21)`=42 / `rep<3>(21)`=63）在四端用例
//! （`native/tests/cases/const_instances_963.rs` 等）里执行，本文件只钉
//! 发现/身份/拒绝面。

use ace_core::source::cst::INSTANCES_CAPABILITY;
use ace_source_query::{
    compile_record_product, InstanceRecord, ProductCompile, SourceQuerySession,
};

const PKG: &str = "plan963cg";
const MOD: &str = "g.at";

fn caps() -> [String; 1] {
    [INSTANCES_CAPABILITY.to_string()]
}

fn record(src: &str) -> Result<ProductCompile, Vec<String>> {
    let mut s = SourceQuerySession::new(PKG, MOD);
    compile_record_product(src, MOD, &caps(), PKG, MOD, &mut s)
        .map_err(|ds| ds.iter().map(diag).collect())
}

fn diag(d: &ace_core::atom_text::Diagnostic) -> String {
    format!("{} @{}-{}: {}", d.code, d.span.start, d.span.end, d.message)
}

fn ok(src: &str) -> ProductCompile {
    match record(src) {
        Ok(p) => p,
        Err(ds) => panic!("expected an instance product, got {ds:#?}"),
    }
}

fn rejected(src: &str, why: &str) -> Vec<String> {
    match record(src) {
        Ok(_) => panic!("expected a rejection: {why}"),
        Err(ds) => ds,
    }
}

fn by_name<'a>(p: &'a ProductCompile, name: &str) -> &'a InstanceRecord {
    p.instances
        .iter()
        .find(|i| i.concrete_name == name)
        .unwrap_or_else(|| panic!("no instance `{name}`: {:?}", p.instances))
}

/// 值参数模板：`rep<const N>(x int) int` 把 N 乘进结果。
const REP: &str = "\
fn rep<const N>(x int) int { return N * x }
fn main() int { return rep<2>(21) }
";

/// 同一模板两个值实参 → 两个实例（值进身份）。
const REP_TWO_VALUES: &str = "\
fn rep<const N>(x int) int { return N * x }
fn main() int { return rep<2>(21) + rep<3>(21) }
";

/// 同值两个调用点 → 一个实例、两个需求。
const REP_SAME_VALUE: &str = "\
fn rep<const N>(x int) int { return N * x }
fn main() int { return rep<2>(21) + rep<2>(1) }
";

/// 显式声明类型 `u32`（与缺省同义）。
const REP_EXPLICIT_U32: &str = "\
fn rep<const N u32>(x int) int { return N * x }
fn main() int { return rep<2>(21) }
";

/// 值参数与类型参数混排（声明序 `[const N, T]`）。
const MIXED_DECL: &str = "\
fn wrap<const N, T>(x T) int { return N }
fn main() int { return wrap<7>(9) }
";

/// 值参与条件（体里 `N` 是字面量）。
const COND: &str = "\
fn clamp<const N>(x int) int {
    if x > N { return N }
    return x
}
fn main() int { return clamp<5>(9) + clamp<5>(3) }
";

#[test]
fn const_param_instance_is_produced() {
    let p = ok(REP);
    assert_eq!(
        p.instances.len(),
        1,
        "one value instance: {:?}",
        p.instances
    );
    let i = &p.instances[0];
    assert_eq!(i.template, "rep");
    assert_eq!(i.args.len(), 0, "no type parameters");
    assert_eq!(i.consts.len(), 1, "one value argument");
    assert_eq!(i.consts[0].param, "N");
    assert_eq!(i.consts[0].value, 2);
    // 装配名是拼写（含声明序值位），不是身份。
    assert_eq!(i.concrete_name, "rep_u32_2");
}

/// V-G04：N=2/N=3 = 不同身份（不同键）且各自布局/值不同。
#[test]
fn distinct_values_are_distinct_instances() {
    let p = ok(REP_TWO_VALUES);
    assert_eq!(
        p.instances.len(),
        2,
        "two value instances: {:?}",
        p.instances
    );
    let two = by_name(&p, "rep_u32_2");
    let three = by_name(&p, "rep_u32_3");
    assert_eq!(two.consts[0].value, 2);
    assert_eq!(three.consts[0].value, 3);
    assert_ne!(two.key, three.key, "value is part of the identity");
    assert_eq!(
        two.logical, three.logical,
        "logical is the template's slot, not the identity"
    );
}

/// V-G04：同值规范化复用（一个实例，两个调用点需求）。
#[test]
fn same_value_is_normalized_to_one_instance() {
    let p = ok(REP_SAME_VALUE);
    assert_eq!(p.instances.len(), 1, "same value reuses: {:?}", p.instances);
    assert_eq!(p.instances[0].consts[0].value, 2);
    assert_eq!(p.instances[0].demands, 2, "two call sites demand it");
}

#[test]
fn explicit_u32_declaration_is_accepted() {
    let p = ok(REP_EXPLICIT_U32);
    assert_eq!(p.instances.len(), 1);
    assert_eq!(p.instances[0].consts[0].value, 2);
    assert_eq!(p.instances[0].concrete_name, "rep_u32_2");
}

/// 值位与类型位按声明序共同成身份；类型仍由实参推断。
#[test]
fn mixed_value_and_type_params() {
    let p = ok(MIXED_DECL);
    assert_eq!(p.instances.len(), 1, "{:?}", p.instances);
    let i = &p.instances[0];
    assert_eq!(i.args.len(), 1, "T solved from the argument");
    assert_eq!(i.consts.len(), 1);
    assert_eq!(i.consts[0].value, 7);
    assert_eq!(i.concrete_name, "wrap_u32_7_i32");
}

/// 体里的 `N` 已被代入字面量（若仍是变量，resolve 会报未声明名字）。
#[test]
fn value_is_substituted_in_the_template_body() {
    let p = ok(COND);
    assert_eq!(
        p.instances.len(),
        1,
        "same N → one instance: {:?}",
        p.instances
    );
    assert_eq!(p.instances[0].demands, 2);
}

/// 缺值实参：模板必须写 `<..>`。
#[test]
fn missing_value_argument_is_rejected() {
    let ds = rejected(
        "fn rep<const N>(x int) int { return N * x }\nfn main() int { return rep(21) }\n",
        "a const-parameter template called without value arguments",
    );
    assert!(
        ds.iter().any(|d| d.contains("value-argument-required")),
        "{ds:#?}"
    );
}

/// 值实参过多：arity 具名拒。
#[test]
fn extra_value_arguments_are_rejected() {
    let ds = rejected(
        "fn rep<const N>(x int) int { return N * x }\nfn main() int { return rep<2, 3>(21) }\n",
        "one value parameter, two value arguments",
    );
    assert!(
        ds.iter().any(|d| d.contains("value-arity-mismatch")),
        "{ds:#?}"
    );
}

/// 值给无值参数的函数：同样 arity 具名拒。
#[test]
fn value_argument_to_plain_fn_is_rejected() {
    let ds = rejected(
        "fn id(x int) int { return x }\nfn main() int { return id<2>(1) }\n",
        "a fn with no value parameter called with one",
    );
    assert!(
        ds.iter().any(|d| d.contains("value-arity-mismatch")),
        "{ds:#?}"
    );
}

/// 值域：u32 之外的实参在投影期具名拒（不静默截断）；u32 内但超出
/// i32 体域的实参同样具名拒（ACE `int` 表达不了，代进体会截断）。
#[test]
fn out_of_domain_value_is_rejected() {
    let ds = rejected(
        "fn rep<const N>(x int) int { return N * x }\nfn main() int { return rep<4294967296>(21) }\n",
        "value argument beyond u32",
    );
    assert!(
        ds.iter().any(|d| d.contains("source.literal-range")),
        "{ds:#?}"
    );
    let ds = rejected(
        "fn rep<const N>(x int) int { return N * x }\nfn main() int { return rep<2147483648>(21) }\n",
        "value argument beyond the i32 body domain",
    );
    assert!(
        ds.iter()
            .any(|d| d.contains("source.literal-range") && d.contains("i32")),
        "{ds:#?}"
    );
}

/// 声明类型：唯一 `<const N>` / `<const N u32>`，其他拼写具名拒。
#[test]
fn non_u32_declared_type_is_rejected() {
    let ds = rejected(
        "fn rep<const N bool>(x int) int { return N * x }\nfn main() int { return rep<2>(21) }\n",
        "const parameter declared bool",
    );
    assert!(
        ds.iter()
            .any(|d| d.contains("source.instances-unsupported") && d.contains("u32")),
        "{ds:#?}"
    );
}

/// 无能力面：值参数与值实参调用都不成立（935 面冻结语义不变）。
#[test]
fn without_capability_value_params_are_rejected() {
    let mut s = SourceQuerySession::new(PKG, MOD);
    let r = compile_record_product(REP, MOD, &[], PKG, MOD, &mut s);
    assert!(r.is_err(), "plain record face must reject `<const N>`");
}
