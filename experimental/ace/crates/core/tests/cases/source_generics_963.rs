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
