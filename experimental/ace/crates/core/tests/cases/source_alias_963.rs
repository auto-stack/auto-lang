//! PLAN-963（T-03）：类型别名 `type Name = T`（投影期展开）。
//!
//! 覆盖：别名在字段/参数/返回/局部注解各位置展开；别名链与**泛型目标**
//! （`type P = Point<int>`，展开先于记录单态化）；重复名与环具名拒；未知目标
//! 不被静默接受。

use ace_core::atom_text::Diagnostic;
use ace_core::source::adapter::CompiledSource;
use ace_core::source::compile_record_source;

fn compile(src: &str) -> Result<CompiledSource, Vec<Diagnostic>> {
    compile_record_source(src, "p.at", &[])
}

fn codes(ds: &[Diagnostic]) -> Vec<String> {
    ds.iter().map(|d| d.code.clone()).collect()
}

/// O-963-A1：`Age = int` / `Flag = bool` 在字段、参数、返回与局部注解里都
/// 展开为具体类型——atom 面不再残留别名名。
#[test]
fn alias_expands_in_every_type_position() {
    let src = "\
type Age = int
type Flag = bool
type Score2 { val Age; ok Flag }
fn bump(x Age) Age { return x + 1 }
fn main() int {
    let s Score2 = Score2 { val: 41, ok: true }
    let n Age = bump(s.val)
    if s.ok { return n }
    return 0
}
";
    let c = compile(src).expect("aliases compile");
    assert!(
        !c.atom_text.contains("Age") && !c.atom_text.contains("Flag"),
        "alias residue in the artifact:\n{}",
        c.atom_text
    );
    assert!(
        c.atom_text.contains("Score2"),
        "record type missing:\n{}",
        c.atom_text
    );
}

/// O-963-A2：别名链 + 泛型目标（`Q = P`, `P = Point<int>`）——展开先于单态化，
/// 因此目标里的 `Point<int>` 落在使用位再被换名为具体记录。
#[test]
fn alias_chain_to_generic_target() {
    let src = "\
type P = Point<int>
type Q = P
type Point<T> { x T; y T }
fn main() int {
    let q Q = Point { x: 2, y: 3 }
    return q.x + q.y
}
";
    let c = compile(src).expect("alias chain compiles");
    assert!(c.atom_text.contains("Point_int"), "{}", c.atom_text);
    assert!(
        !c.atom_text.contains("Point<T>"),
        "template residue:\n{}",
        c.atom_text
    );
}

/// O-963-A3：别名环具名拒（`source.alias-cycle`），不无限展开。
#[test]
fn alias_cycle_is_rejected() {
    let src = "\
type A = B
type B = A
fn main() int {
    let a A = 0
    return a
}
";
    let err = compile(src).expect_err("cycle must be rejected");
    assert!(
        codes(&err).iter().any(|c| c == "source.alias-cycle"),
        "{err:#?}"
    );
}

/// O-963-A4：重复别名 / 别名与记录同名 → `source.duplicate`。
#[test]
fn duplicate_alias_is_rejected() {
    let dup = "\
type Age = int
type Age = bool
fn main() int { return 0 }
";
    let err = compile(dup).expect_err("duplicate alias must be rejected");
    assert!(
        codes(&err).iter().any(|c| c == "source.duplicate"),
        "{err:#?}"
    );

    let clash = "\
type Point { x int }
type Point = int
fn main() int { return 0 }
";
    let err = compile(clash).expect_err("alias/record clash must be rejected");
    assert!(
        codes(&err).iter().any(|c| c == "source.duplicate"),
        "{err:#?}"
    );
}

/// O-963-A5：未知目标不被静默接受——展开后按普通类型解析失败。
#[test]
fn unknown_alias_target_is_rejected() {
    let src = "\
type Missing = Nope
type Box2 { m Missing }
fn main() int {
    let b Box2 = Box2 { m: 0 }
    return b.m
}
";
    let err = compile(src).expect_err("unknown target must be rejected");
    assert!(err.iter().any(|d| d.message.contains("Nope")), "{err:#?}");
}
