//! PLAN-963 T-02：实例车道的值域放宽（文本/记录）与约束见证。
//!
//! 覆盖 V-G01（int/bool/文本/记录实参各自成实例；同替换共享、异替换不串；
//! 错误类型在 dispatch 前拒）与 V-G03（`<T has Ord>` 满足即生成、不满足即
//! `source.instances-bound` 具名拒；记录字段/方法在延迟约束中解出）。
//!
//! 运行面（四端真实 consumer）由 T-03 接线；本文件只钉实例发现/身份/拒绝，
//! 不以旧 VM 语义或生成全文代运行。

use ace_core::source::cst::INSTANCES_CAPABILITY;
use ace_source_query::{
    compile_record_product, compile_text_record_product, decode_instance_page, encode_instance_page,
    CompileLane, InstancePage, InstanceRecord, InstanceTy, ProductCompile, SourceQuerySession,
};

const PKG: &str = "plan963vd";
const MOD: &str = "g.at";

fn caps() -> [String; 1] {
    [INSTANCES_CAPABILITY.to_string()]
}

/// 记录域实例编译（`compile_record_product`）。
fn record(src: &str) -> Result<ProductCompile, Vec<String>> {
    let mut s = SourceQuerySession::new(PKG, MOD);
    compile_record_product(src, MOD, &caps(), PKG, MOD, &mut s)
        .map_err(|ds| ds.iter().map(diag).collect())
}

/// 文本记录域实例编译（`compile_text_record_product`）。
fn text_record(src: &str) -> Result<ProductCompile, Vec<String>> {
    let mut s = SourceQuerySession::new(PKG, MOD);
    compile_text_record_product(src, MOD, &caps(), PKG, MOD, &mut s)
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

/// 期望具名拒（`ProductCompile` 不带 Debug，不用 `expect_err`）。
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

const SCALARS: &str = "\
fn pick<T>(a T, b T) T { return a }
fn main() int {
    let i int = pick(1, 2)
    let b bool = pick(true, false)
    return i
}
";

const RECORD_ARG: &str = "\
type Score { val int }
fn pick<T>(a T, b T) T { return a }
fn main() int {
    let s Score = Score { val: 7 }
    return pick(s, s).val
}
";

const TEXT_ARG: &str = "\
fn pick<T>(a T, b T) T { return a }
fn main() int {
    let s str = \"hi\"
    let t str = pick(s, s)
    return 1
}
";

const BOUND_OK: &str = "\
spec Ord { fn rank() int }
type Score2 as Ord { val int
    fn rank() int { return .val }
}
fn max_of<T has Ord>(a T, b T) T {
    if a.rank() > b.rank() { return a }
    return b
}
fn main() int {
    let a Score2 = Score2 { val: 3 }
    let b Score2 = Score2 { val: 9 }
    return max_of(a, b).val
}
";

const BOUND_MISSING: &str = "\
type Score2 { val int }
fn max_of<T has Ord>(a T, b T) T {
    if a.rank() > b.rank() { return a }
    return b
}
fn main() int {
    let a Score2 = Score2 { val: 3 }
    let b Score2 = Score2 { val: 9 }
    return max_of(a, b).val
}
";

const BOUND_MISSING_METHOD: &str = "\
spec Ord { fn rank() int }
type Score2 as Ord { val int }
fn max_of<T has Ord>(a T, b T) T {
    if a.rank() > b.rank() { return a }
    return b
}
fn main() int {
    let a Score2 = Score2 { val: 3 }
    let b Score2 = Score2 { val: 9 }
    return max_of(a, b).val
}
";

/// V-G01：标量面每个替换各自成实例（i32/bool 不串），同替换两点共享一例。
#[test]
fn vg01_scalar_substitutions_are_distinct_instances() {
    let p = ok(SCALARS);
    assert_eq!(p.lane, CompileLane::Instances);
    assert_eq!(p.instances.len(), 2, "{:?}", p.instances);
    let i32_ = by_name(&p, "pick_i32");
    let bool_ = by_name(&p, "pick_bool");
    assert_eq!(i32_.args[0].ty, InstanceTy::I32);
    assert_eq!(bool_.args[0].ty, InstanceTy::Bool);
    assert_ne!(i32_.key, bool_.key, "不同替换必须是不同实现身份");
    assert_eq!(i32_.logical, bool_.logical, "逻辑实例只按声明 owner 取 id");
    assert_eq!(i32_.template, bool_.template);
}

/// V-G01：记录实参解出记录实例；字段读取（`pick(s, s).val`）在接收者类型
/// 晚于使用点可知时仍解出（延迟约束），不因后序栈而漏。
#[test]
fn vg01_record_argument_and_field_read() {
    let p = ok(RECORD_ARG);
    assert_eq!(p.lane, CompileLane::Instances);
    let inst = by_name(&p, "pick_Score");
    assert_eq!(
        inst.args,
        vec![ace_source_query::InstanceArg {
            param: "T".into(),
            ty: InstanceTy::Record("Score".into()),
        }],
        "记录替换按声明序落位"
    );
    assert_eq!(p.instances.len(), 1, "同一替换只产一个实例");
}

/// V-G01：文本实参同样成实例（值域不退回两标量）。
#[test]
fn vg01_text_argument_instance() {
    let p = match text_record(TEXT_ARG) {
        Ok(p) => p,
        Err(ds) => panic!("expected a text instance, got {ds:#?}"),
    };
    assert_eq!(p.lane, CompileLane::Instances);
    let inst = by_name(&p, "pick_text");
    assert_eq!(inst.args[0].ty, InstanceTy::Text);
}

/// V-G01：不同替换的记录实例互不串（同模板两实例、键/简化名各异）。
#[test]
fn vg01_distinct_record_substitutions_do_not_collide() {
    let src = "\
type A { val int }
type B { val int }
fn pick<T>(a T, b T) T { return a }
fn main() int {
    let a A = A { val: 1 }
    let b B = B { val: 2 }
    return pick(a, a).val + pick(b, b).val
}
";
    let p = ok(src);
    assert_eq!(p.instances.len(), 2, "{:?}", p.instances);
    let a = by_name(&p, "pick_A");
    let b = by_name(&p, "pick_B");
    assert_ne!(a.key, b.key);
    assert_eq!(a.logical, b.logical);
}

/// V-G03：`<T has Ord>` 的实参满足 spec → 生成实例并解出 `a.rank()` 方法
/// 调用（方法返回类型进入调用点绑定）。
#[test]
fn vg03_bound_satisfied_generates_instance() {
    let p = ok(BOUND_OK);
    assert_eq!(p.lane, CompileLane::Instances);
    assert_eq!(p.instances.len(), 1, "{:?}", p.instances);
    let inst = by_name(&p, "max_of_Score2");
    assert_eq!(inst.args[0].ty, InstanceTy::Record("Score2".into()));
}

/// V-G03 负控：实参未 `as Ord` → 具名拒（`source.instances-bound`），
/// 不静默生成实例、不把它降级成标量/未知。
#[test]
fn vg03_bound_unsatisfied_is_named_rejection() {
    let ds = rejected(BOUND_MISSING, "unsatisfied bound must be rejected");
    let hit = ds
        .iter()
        .find(|d| d.contains("source.instances-bound"))
        .unwrap_or_else(|| panic!("expected source.instances-bound, got {ds:#?}"));
    assert!(hit.contains("does not satisfy `Ord`"), "{hit}");
    assert!(hit.contains("max_of"), "诊断须点名模板: {hit}");
}

/// V-G03 负控：`as Ord` 声明了但缺 spec 要求的抽象方法 `rank` → 仍是拒
/// （合规核验在单态化后按 spec 成员进行）。
#[test]
fn vg03_missing_spec_member_is_rejected() {
    let ds = rejected(BOUND_MISSING_METHOD, "missing spec member must be rejected");
    assert!(
        ds.iter().any(|d| d.contains("rank")),
        "拒绝须点名缺失成员: {ds:#?}"
    );
}

/// V-G01 负控：错误类型在 dispatch 前拒（记录位填标量、标量位填记录）。
#[test]
fn vg01_wrong_type_is_rejected_before_dispatch() {
    let scalar_into_record = "\
type Score { val int }
fn take(a Score) int { return a.val }
fn main() int { return take(1) }
";
    let ds = rejected(
        scalar_into_record,
        "scalar into record position must be rejected",
    );
    assert!(!ds.is_empty());

    let record_into_scalar = "\
type Score { val int }
fn take(a int) int { return a }
fn main() int {
    let s Score = Score { val: 1 }
    return take(s)
}
";
    let ds = rejected(
        record_into_scalar,
        "record into int position must be rejected",
    );
    assert!(!ds.is_empty());
}

/// V-G03：记录方法调用的实参绑定（按名/按位）与返回类型进入调用点；
/// 记录上没有该方法 → 具名拒而不是静默穿透。
#[test]
fn vg03_record_method_call_binding() {
    let src = "\
type Score { val int
    fn bigger(o Score) bool { return .val > o.val }
}
fn pick<T>(a T, b T) T { return a }
fn main() int {
    let a Score = Score { val: 3 }
    let b Score = Score { val: 5 }
    if pick(a, b).bigger(b) { return 1 }
    return 0
}
";
    let p = ok(src);
    assert_eq!(p.instances.len(), 1);

    let unknown = "\
type Score { val int }
fn main() int {
    let a Score = Score { val: 3 }
    if a.missing() { return 1 }
    return 0
}
";
    let ds = rejected(unknown, "unknown method must be rejected");
    assert!(
        ds.iter().any(|d| d.contains("missing")),
        "拒绝须点名未知方法: {ds:#?}"
    );
}

/// V-G03：记录构造的字段面负控（未知字段 / 缺字段）在记录域具名拒。
#[test]
fn vg03_record_construction_field_negatives() {
    let unknown_field = "\
type Score { val int }
fn main() int {
    let s Score = Score { val: 1, extra: 2 }
    return s.val
}
";
    let ds = rejected(unknown_field, "unknown field must be rejected");
    assert!(
        ds.iter().any(|d| d.contains("extra")),
        "拒绝须点名未知字段: {ds:#?}"
    );

    let missing_field = "\
type Pair { a int
    b int }
fn main() int {
    let p Pair = Pair { a: 1 }
    return p.a
}
";
    let ds = rejected(missing_field, "missing field must be rejected");
    assert!(
        ds.iter()
            .any(|d| d.contains("missing: b") || d.contains("`b`")),
        "拒绝须点名缺失字段: {ds:#?}"
    );
}

/// V-G01：实例页 codec v3 往返保留文本/记录实参（类型码 + 记录名），且
/// 记录名被截断时按截断拒——不把旧长度当新布局重释。
#[test]
fn vg01_page_codec_v3_carries_text_and_record_args() {
    let rec = ok(RECORD_ARG);
    let page = InstancePage::from(&rec.instances[0]);
    let bytes = encode_instance_page(&page).expect("encode v3 record arg");
    assert_eq!(
        decode_instance_page(&bytes).expect("decode v3 record arg"),
        page,
        "记录实参往返（含记录名）"
    );
    // 记录名块截断：不得解出别的记录或静默补齐。
    for cut in 1..=3 {
        let mut bad = bytes.clone();
        bad.truncate(bytes.len() - cut);
        assert!(
            decode_instance_page(&bad).is_err(),
            "记录名截断 {cut} 字节必须拒，不得重释"
        );
    }

    let text = text_record(TEXT_ARG).expect("text instance");
    let tpage = InstancePage::from(&text.instances[0]);
    let tbytes = encode_instance_page(&tpage).expect("encode v3 text arg");
    assert_eq!(
        decode_instance_page(&tbytes).expect("decode v3 text arg"),
        tpage,
        "文本实参往返"
    );
    assert_eq!(tpage.args[0].1, InstanceTy::Text);
}
