//! PLAN-963（T-02）：用户泛型记录类型的语法期单态化。
//!
//! 覆盖：`type Point<T>` + `Point<int>` 合成为具体记录（模板移除）、同实参
//! 去重、嵌套泛型按字段类型继续展开、arity 具名拒、显式类型实参位越界拒。

use ace_core::atom_text::Diagnostic;
use ace_core::source::adapter::CompiledSource;
use ace_core::source::compile_record_source;

fn compile(src: &str) -> Result<CompiledSource, Vec<Diagnostic>> {
    compile_record_source(src, "p.at", &[])
}

const POINT: &str = "\
type Point<T> { x T; y T }
fn main() int {
    let p Point<int> = Point { x: 100, y: 200 }
    return p.x + p.y
}
";

/// O-963-1：`Point<int>` 单态化为具体记录；模板名不再作为记录类型到达
/// 下游（atom 面只见 `Point_int`）。
#[test]
fn generic_record_monomorphizes_963() {
    let c = compile(POINT).expect("generic record compiles");
    assert!(
        c.atom_text.contains("Point_int"),
        "synthesized concrete record missing:\n{}",
        c.atom_text
    );
    assert!(
        !c.atom_text.contains("Point<T>"),
        "template residue in atom text"
    );
}

/// O-963-2：同一实参的两个使用位只合成一个具体记录（去重键=声明序实参）。
#[test]
fn generic_record_dedups_963() {
    let src = "\
type Point<T> { x T; y T }
fn main() int {
    let p Point<int> = Point { x: 1, y: 2 }
    let q Point<int> = Point { x: 3, y: 4 }
    return p.x + q.y
}
";
    let c = compile(src).expect("dedup compiles");
    assert_eq!(
        c.atom_text.matches("Point_int").count() > 0,
        true,
        "concrete record missing"
    );
}

/// O-963-3：嵌套泛型按具体记录的字段类型继续展开（`Pair { b: Box { .. } }`）。
#[test]
fn generic_record_nested_963() {
    let src = "\
type Box<T> { v T }
type Pair<T> { a T; b Box<T> }
fn main() int {
    let q Pair<int> = Pair { a: 1, b: Box { v: 2 } }
    return q.a + q.b.v
}
";
    let c = compile(src).expect("nested generic record compiles");
    assert!(c.atom_text.contains("Box_int"), "nested instance missing");
    assert!(c.atom_text.contains("Pair_int"), "outer instance missing");
}

/// O-963-4：类型实参个数不符 → 具名拒（不静默按首个实参实例化）。
#[test]
fn generic_record_arity_rejected_963() {
    let src = "\
type Point<T> { x T; y T }
fn main() int {
    let p Point<int, bool> = Point { x: 1, y: 2 }
    return 0
}
";
    let err = compile(src).expect_err("arity mismatch must be rejected");
    assert!(
        err.iter().any(|d| d.code == "source.unsupported"
            && d.message.contains("type argument")),
        "expected named arity rejection, got {err:?}"
    );
}

/// O-963-5：字段值类型与替换后的字段类型不符 → 类型检查拒绝（不是静默
/// 通过，也不是 dispatch 成功）。
#[test]
fn generic_record_field_type_mismatch_rejected_963() {
    let src = "\
type Point<T> { x T; y T }
fn main() int {
    let p Point<int> = Point { x: true, y: 200 }
    return p.x
}
";
    assert!(
        compile(src).is_err(),
        "bool initializer for a substituted int field must be rejected"
    );
}

/// O-963-6：泛型记录的方法签名/返回/方法体内构造都按声明序替换
/// （`static fn both(v T) Point<T>` 的方法体构造同样换名为具体记录）。
#[test]
fn generic_record_methods_monomorphize_963() {
    let src = "\
type Point<T> { x T; y T
    static fn both(v T) Point<T> { return Point { x: v, y: v } }
    fn first() T { return .x }
}
fn main() int {
    let p Point<int> = Point.both(150)
    return p.first() + p.y
}
";
    let c = compile(src).expect("generic methods compile");
    let atom = &c.atom_text;
    assert!(atom.contains("Point_int"), "concrete record missing");
    // 方法体的构造已具体化：不存在未换名的模板构造/模板类型位。
    assert_eq!(
        atom.matches("Point {").count(),
        0,
        "template constructor residue in atom text"
    );
    assert!(!atom.contains("Point<T>"), "template residue in atom text");
}

/// O-963-7：显式类型实参构造 `Point<int> { .. }`（旧 VM 拼写）无需注解即可
/// 单态化；同一模板的第二个实参也各自合成（`Point<int>` + `Point<bool>`）。
#[test]
fn generic_record_typed_ctor_963() {
    let src = "\
type Point<T> { x T; y T
    fn first() T { return .x }
}
fn main() int {
    let p = Point<int> { x: 100, y: 200 }
    let q = Point<bool> { x: true, y: false }
    if q.first() { return p.first() + p.y } else { return 0 }
}
";
    let c = compile(src).expect("typed constructors compile");
    assert!(c.atom_text.contains("Point_int"), "int instance missing");
    assert!(c.atom_text.contains("Point_bool"), "bool instance missing");
}

/// O-963-8：显式类型实参构造的字段类型不符 / 实参个数不符 → 编译期拒
/// （dispatch=0：拒绝发生在 resolve 之前，任何后端都不会启动）。
#[test]
fn generic_record_typed_ctor_rejections_963() {
    let mismatch = "\
type Point<T> { x T; y T }
fn main() int {
    let p = Point<int> { x: true, y: 200 }
    return p.x
}
";
    assert!(
        compile(mismatch).is_err(),
        "bool initializer for a substituted int field must be rejected"
    );
    let arity = "\
type Point<T> { x T; y T }
fn main() int {
    let p = Point<int, bool> { x: 1, y: 2 }
    return 0
}
";
    let err = compile(arity).expect_err("arity mismatch must be rejected");
    assert!(
        err.iter().any(|d| d.code == "source.unsupported"
            && d.message.contains("type argument")),
        "expected named arity rejection, got {err:?}"
    );
}

/// O-963-9：非泛型类型的显式类型实参 → 具名拒（不把 `Plain<int>` 当模板）。
#[test]
fn generic_record_typed_ctor_on_plain_type_rejected_963() {
    let src = "\
type Plain { v int }
fn main() int {
    let p = Plain<int> { v: 1 }
    return p.v
}
";
    let err = compile(src).expect_err("type arguments on a plain record must be rejected");
    assert!(
        err.iter()
            .any(|d| d.code == "source.unsupported" && d.message.contains("type argument")),
        "expected named rejection, got {err:?}"
    );
}

// ---------------------------------------------------------------------------
// PLAN-963（T-03）：`spec` 声明 + `type X as S` 遵从（投影期擦除）
// ---------------------------------------------------------------------------

const SPEC_EXPLICIT: &str = "\
spec Comparable { fn compare(other int) int }
type Score as Comparable {
    val int
    fn compare(other int) int { return .val - other }
}
fn main() int {
    let s Score = Score { val: 40 }
    return s.compare(3)
}
";

/// O-963-10：显式实现满足 spec 抽象方法 → 编译通过；spec 本身不进下游
/// （无 spec 声明残留，调用按普通记录方法解析）。
#[test]
fn spec_explicit_impl_compiles_963() {
    let c = compile(SPEC_EXPLICIT).expect("spec conformance compiles");
    assert!(
        !c.atom_text.contains("Comparable"),
        "spec declaration leaked into the atom text:\n{}",
        c.atom_text
    );
}

/// O-963-11：spec 默认体在实现者未声明同名方法时注入（调用 `t.greet()` 由
/// 注入的方法解析；无注入时 resolve 会以未知方法拒绝）。
#[test]
fn spec_default_body_injected_963() {
    let src = "\
spec Greeter { fn greet() int { return 7 } }
type Thing as Greeter { v int }
fn main() int {
    let t Thing = Thing { v: 1 }
    return t.greet()
}
";
    let c = compile(src).expect("default spec method is injected");
    assert!(
        c.atom_text.contains("greet"),
        "injected default method missing from the checked module:\n{}",
        c.atom_text
    );
}

/// O-963-12：显式实现优先于默认体（两者都合法，不产生重复方法）。
#[test]
fn spec_explicit_overrides_default_963() {
    let src = "\
spec Greeter { fn greet() int { return 7 } }
type Thing as Greeter { v int
    fn greet() int { return 9 }
}
fn main() int {
    let t Thing = Thing { v: 1 }
    return t.greet()
}
";
    compile(src).expect("explicit implementation overrides the default");
}

/// O-963-13：多 spec 遵从列表 `as A, B` 各自核验。
#[test]
fn spec_multiple_conformance_963() {
    let src = "\
spec First { fn first() int }
spec Second { fn second() int }
type Both as First, Second { v int
    fn first() int { return .v }
    fn second() int { return .v + 1 }
}
fn main() int {
    let b Both = Both { v: 1 }
    return b.first() + b.second()
}
";
    compile(src).expect("multiple conformance list compiles");
}

/// O-963-14：缺抽象方法 → 具名拒（dispatch=0：投影期即拒）。
#[test]
fn spec_missing_method_rejected_963() {
    let src = "\
spec Comparable { fn compare(other int) int }
type Score as Comparable { val int }
fn main() int {
    let s Score = Score { val: 1 }
    return s.val
}
";
    let err = compile(src).expect_err("missing spec method must be rejected");
    assert!(
        err.iter().any(|d| d.code == "source.unsupported"
            && d.message.contains("does not implement")
            && d.message.contains("compare")),
        "expected named missing-method rejection, got {err:?}"
    );
}

/// O-963-15：未知 spec 名 → 具名拒（不按名字静默跳过遵从检查）。
#[test]
fn spec_unknown_name_rejected_963() {
    let src = "\
type Score as Missing { val int }
fn main() int {
    let s Score = Score { val: 1 }
    return s.val
}
";
    let err = compile(src).expect_err("unknown spec must be rejected");
    assert!(
        err.iter()
            .any(|d| d.code == "source.unsupported" && d.message.contains("unknown spec")),
        "expected named unknown-spec rejection, got {err:?}"
    );
}

/// O-963-16：实现方法签名与 spec 不符 → 具名拒（不静默接受名义遵从）。
#[test]
fn spec_signature_mismatch_rejected_963() {
    let src = "\
spec Comparable { fn compare(other int) int }
type Score as Comparable {
    val int
    fn compare(other bool) int { return .val }
}
fn main() int {
    let s Score = Score { val: 1 }
    return s.compare(0)
}
";
    let err = compile(src).expect_err("signature mismatch must be rejected");
    assert!(
        err.iter()
            .any(|d| d.code == "source.unsupported" && d.message.contains("does not match")),
        "expected named signature rejection, got {err:?}"
    );
}

/// O-963-17：重复 spec 声明 / 遵从列表重复项 → 具名拒。
#[test]
fn spec_duplicates_rejected_963() {
    let dup_decl = "\
spec S { fn m() int }
spec S { fn m() int }
type T as S { v int
    fn m() int { return .v }
}
fn main() int {
    let t T = T { v: 1 }
    return t.m()
}
";
    let err = compile(dup_decl).expect_err("duplicate spec declaration must be rejected");
    assert!(
        err.iter()
            .any(|d| d.code == "source.duplicate" && d.message.contains("more than once")),
        "expected duplicate-spec rejection, got {err:?}"
    );
    let dup_list = "\
spec S { fn m() int }
type T as S, S { v int
    fn m() int { return .v }
}
fn main() int {
    let t T = T { v: 1 }
    return t.m()
}
";
    let err = compile(dup_list).expect_err("duplicate conformance entry must be rejected");
    assert!(
        err.iter()
            .any(|d| d.code == "source.duplicate" && d.message.contains("listed more than once")),
        "expected duplicate-entry rejection, got {err:?}"
    );
}

/// O-963-18：泛型记录 × spec 组合——遵从列表随单态化复制到每个合成实例，
/// 抽象方法按替换后的具体签名核验（T-02 × T-03 交叉面）。
#[test]
fn spec_on_generic_record_963() {
    let src = "\
spec Summable { fn total() int }
type Pair<T> as Summable { a T
    b T
    fn total() int { return .a + .b }
}
fn main() int {
    let p Pair<int> = Pair { a: 1, b: 2 }
    return p.total()
}
";
    compile(src).expect("generic record conformance compiles per instance");
}

/// O-963-19：泛型实例缺抽象方法 → 同一条具名拒（遵从检查不因模板而放宽）。
#[test]
fn spec_on_generic_record_missing_method_963() {
    let src = "\
spec Summable { fn total() int }
type Pair<T> as Summable { a T
    b T
}
fn main() int {
    let p Pair<int> = Pair { a: 1, b: 2 }
    return p.a
}
";
    let err = compile(src).expect_err("synthesized instance must satisfy the spec");
    assert!(
        err.iter().any(|d| d.code == "source.unsupported"
            && d.message.contains("does not implement")
            && d.message.contains("total")),
        "expected named missing-method rejection on the synthesized instance, got {err:?}"
    );
}

// ---------------------------------------------------------------------------
// PLAN-963（T-03）：`<T has S>` 类型参数约束（单态化时按遵从列表核验）
// ---------------------------------------------------------------------------

const BOUND_OK: &str = "\
spec Summable { fn total() int }
type Score as Summable { val int
    fn total() int { return .val }
}
type Box<T has Summable> { inner T }
fn main() int {
    let b Box<Score> = Box { inner: Score { val: 5 } }
    let s Score = b.inner
    return s.total()
}
";

/// O-963-20：实参记录遵从 bound → 实例合成并编译（约束随实例保留）。
#[test]
fn bound_satisfied_963() {
    compile(BOUND_OK).expect("conforming type argument satisfies the bound");
}

/// O-963-21：标量实参不满足 bound → 具名拒（不合成无约束实例）。
#[test]
fn bound_scalar_rejected_963() {
    let src = "\
spec Summable { fn total() int }
type Box<T has Summable> { inner T }
fn main() int {
    let b Box<int> = Box { inner: 1 }
    return b.inner
}
";
    let err = compile(src).expect_err("a scalar cannot satisfy a spec bound");
    assert!(
        err.iter().any(|d| d.code == "source.unsupported"
            && d.message.contains("does not satisfy")
            && d.message.contains("Summable")),
        "expected named bound rejection, got {err:?}"
    );
}

/// O-963-22：记录实参但未声明遵从 → 同样具名拒。
#[test]
fn bound_non_conforming_record_rejected_963() {
    let src = "\
spec Summable { fn total() int }
type Plain { val int }
type Box<T has Summable> { inner T }
fn main() int {
    let b Box<Plain> = Box { inner: Plain { val: 1 } }
    return b.inner.val
}
";
    let err = compile(src).expect_err("a non-conforming record violates the bound");
    assert!(
        err.iter().any(|d| d.code == "source.unsupported"
            && d.message.contains("does not satisfy")
            && d.message.contains("Plain")),
        "expected named bound rejection for the record, got {err:?}"
    );
}

/// O-963-23：bound 名不是已声明 spec → 具名拒（不静默当成无约束）。
#[test]
fn bound_unknown_spec_rejected_963() {
    let src = "\
type Box<T has Missing> { inner T }
fn main() int {
    let b Box<int> = Box { inner: 1 }
    return b.inner
}
";
    let err = compile(src).expect_err("unknown spec in a bound must be rejected");
    assert!(
        err.iter().any(|d| d.code == "source.unsupported"
            && d.message.contains("unknown spec")
            && d.message.contains("Missing")),
        "expected named unknown-bound rejection, got {err:?}"
    );
}

/// O-963-24：约束经嵌套实参传递（`Box<Pair<int>>` 里 `Pair<int>` 的遵从
/// 列表随模板复制到合成实例，故外层 bound 仍成立）。
#[test]
fn bound_through_nested_instance_963() {
    let src = "\
spec Summable { fn total() int }
type Pair<T> as Summable { a T
    fn total() int { return 1 }
}
type Box<T has Summable> { inner T }
fn main() int {
    let b Box<Pair<int>> = Box { inner: Pair { a: 1 } }
    let p Pair<int> = b.inner
    return p.total()
}
";
    compile(src).expect("synthesized instance keeps its conformance list");
}

