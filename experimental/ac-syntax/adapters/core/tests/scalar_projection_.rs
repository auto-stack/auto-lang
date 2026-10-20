//! PLAN-828 T-02：auto-scalar grammar + 投影 vs ac-core scalar parser 的
//! 差分对拍（oracle = 旧手写 parser，仅测试层使用）。
//! - 旧 parser 接受的源 → 投影必须接受且 AST（结构/顺序/span/字段）逐项一致；
//! - 旧 parser 拒绝的源 → 投影必须拒绝；
//! - 覆盖 scalar 全表面：二元 6 级优先级、一元 `- !`、带符号字面量、
//!   条件 `for`、`while` 拒绝、命名实参、短路逻辑、括号/换行自由。

use ac_syntax_core_adapter::{project_strict, CoreKinds, ProjectionError};
use ac_syntax_grammar::compile;
use ac_syntax_rt::lexer::CompiledScanner;
use ac_syntax_rt::parse::parse_snapshot;
use ac_syntax_rt::program::{Budget, GrammarProgram};
use ac_syntax_rt::snapshot::{PageStore, Snapshot};
use auto_ac_prototype::source::parse_program_with;
use auto_ac_prototype::source::SourceProfile;
use std::rc::Rc;

pub fn scalar_grammar() -> GrammarProgram {
    let src = std::fs::read_to_string(format!(
        "{}/../../grammars/auto-scalar.at",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    compile(&src).unwrap()
}

/// 投影结果（成功 = AST Debug 形态；失败 = 拒绝）。
fn project_scalar(
    p: &GrammarProgram,
    cs: &CompiledScanner,
    src: &str,
) -> Result<String, ProjectionError> {
    let snap = Snapshot::from_text(&Rc::new(PageStore::new()), src);
    let out = parse_snapshot(p, cs, &snap, &Budget::default()).unwrap();
    if !out.tree.diags.is_empty() {
        return Err(ProjectionError::HasErrors(format!(
            "{:?}",
            out.tree.diags
        )));
    }
    let k = CoreKinds::from_program(p);
    let program = project_strict(p, &k, &out.tree, &snap)?;
    Ok(format!("{program:?}"))
}

fn oracle(src: &str) -> Result<String, String> {
    match parse_program_with(src, SourceProfile::Scalar) {
        Ok(p) => Ok(format!("{p:?}")),
        Err(d) => Err(format!("{d:?}")),
    }
}

/// (名称, 源) 对拍矩阵：期望 = oracle 的接受/拒绝结论，投影必须同结论且
/// AST Debug 完全一致（含 span）。
fn differential(cases: &[(&str, &str)]) {
    let p = scalar_grammar();
    let cs = CompiledScanner::compile(&p).unwrap();
    for (name, src) in cases {
        let oracle = oracle(src);
        let projected = project_scalar(&p, &cs, src);
        match (&oracle, &projected) {
            (Ok(a), Ok(b)) => {
                if a != b {
                    let mut i = 0;
                    let (x, y) = (a.as_bytes(), b.as_bytes());
                    while i < x.len().min(y.len()) && x[i] == y[i] {
                        i += 1;
                    }
                    panic!(
                        "case `{name}`: AST mismatch at byte {i}
  oracle: ...{}...
  project: ...{}...
  src: {src}",
                        &a[i.saturating_sub(60)..(i + 90).min(a.len())],
                        &b[i.saturating_sub(60)..(i + 90).min(b.len())],
                    );
                }
            }
            (Err(_), Err(_)) => {}
            (Ok(_), Err(e)) => panic!("case `{name}`: parser accepts, projection rejects: {e:?}\nsource: {src}"),
            (Err(e), Ok(_)) => panic!("case `{name}`: parser rejects, projection accepts: {e:?}\nsource: {src}"),
        }
    }
}

#[test]
fn scalar_projection_operator_surface_parity() {
    differential(&[
        ("arith all", "fn main() int { return 1 + 2 - 3 * 4 / 5 % 6 }"),
        ("precedence chain", "fn f(a int, b int) int { return a + b * 2 - a / b % 3 }"),
        ("relational", "fn f(a int, b int) bool { return a < b }"),
        ("relational le", "fn f(a int, b int) bool { return a <= b }"),
        ("relational gt", "fn f(a int, b int) bool { return a > b }"),
        ("relational ge", "fn f(a int, b int) bool { return a >= b }"),
        ("eq", "fn f(a int, b int) bool { return a == b }"),
        ("ne", "fn f(a int, b int) bool { return a != b }"),
        (
            "logic precedence",
            "fn f(a int, b int) bool { return a < b && b < a || a == b }",
        ),
        (
            "tight logic",
            "fn f(a bool, b bool) bool { return a && b || !a && !b }",
        ),
        (
            "mixed deep",
            "fn f(a int, b int, c bool) bool { return (a + b) * 2 < a % b == c }",
        ),
    ]);
}

#[test]
fn scalar_projection_unary_and_literal_parity() {
    differential(&[
        ("unary neg ident", "fn f(x int) int { return -x }"),
        ("unary not", "fn f(b bool) bool { return !b }"),
        ("unary neg group", "fn f(a int, b int) int { return -(a + b) }"),
        ("unary not group", "fn f(a bool, b bool) bool { return !(a && b) }"),
        ("unary neg call", "fn g() int { return 3 }\nfn main() int { return -g() }"),
        ("unary double neg", "fn f(x int) int { return - -x }"),
        ("neg literal tight", "fn f() int { return -5 }"),
        ("neg literal space", "fn f() int { return - 5 }"),
        ("neg literal min", "fn f() int { return -2147483648 }"),
        ("pos literal overflow", "fn f() int { return 2147483648 }"),
        ("neg literal underflow", "fn f() int { return -2147483649 }"),
        ("bin minus vs neg", "fn f(a int) int { return a -1 }"),
        ("neg then mul", "fn f(a int) int { return -a * 2 }"),
        ("neg after lt", "fn f(a int, b int) bool { return a < -b }"),
    ]);
}

#[test]
fn scalar_projection_loop_and_stmt_parity() {
    differential(&[
        (
            "conditional for",
            "fn f(n int) int { var a = 0; for a < n { a = a + 1 } return a }",
        ),
        (
            "for with logic cond",
            "fn f(n int, m int) int { var a = 0; for a < n && a < m { a = a + 2 } return a }",
        ),
        (
            "for nested if",
            "fn f(n int) int { var a = 0; for a < n { if a % 2 == 0 { a = a + 1 } else { a = a + 3 } } return a }",
        ),
        ("while rejected", "fn f() int { var a = 0; while a < 3 { a = a + 1 } return a }"),
        ("loop stmt", "fn f() int { var a = 0; loop { a = a + 1; if a > 3 { break } } return a }"),
        ("continue stmt", "fn f(n int) int { var a = 0; var s = 0; for a < n { a = a + 1; if a % 2 == 0 { continue } s = s + a } return s }"),
        (
            "let ann",
            "fn f() int { let a int = 3; let b bool = true; if b { return a } return 0 }",
        ),
        (
            "assign chain",
            "fn f(n int) int { var a = n; a = a + 1; a = a * 2; return a }",
        ),
    ]);
}

#[test]
fn scalar_projection_calls_and_layout_parity() {
    differential(&[
        (
            "named args",
            "fn f(a int, b int) int { return a * 10 + b }\nfn main() int { return f(b: 2, a: 1) }",
        ),
        (
            "mixed args multiline",
            "fn f(a int, b int) int { return a + b }\nfn main() int { return f(\n 1,\n b: 2\n) + 3 }",
        ),
        (
            "nested calls",
            "fn g(x int) int { return x * 2 }\nfn h(x int, y int) int { return x + y }\nfn main() int { return h(g(1), h(2, 3)) }",
        ),
        (
            "paren newline",
            "fn main() int { return (1 +\n 2) * 3 }",
        ),
        (
            "comments and blank",
            "// leading\nfn main() int {\n    // inner\n    return 1 // trailing\n}",
        ),
        (
            "recursive call",
            "fn fact(n int) int { if n <= 1 { return 1 } return n * fact(n - 1) }\nfn main() int { return fact(5) }",
        ),
        (
            "bool returns",
            "fn t() bool { return true }\nfn f() bool { return t() && !t() }",
        ),
        ("reserved word name", "fn f(x int) int { return x }"),
        ("reserved rejected", "fn alias() int { return 1 }"),
        ("empty source", ""),
    ]);
}

#[test]
fn scalar_projection_reject_shape_parity() {
    differential(&[
        ("dangling op", "fn f() int { return 1 + }"),
        ("missing paren", "fn f() int { return (1 + 2 }"),
        ("unclosed block", "fn f() int { return 1"),
        ("type word in expr", "fn f() int { let a = int; return 1 }"),
        ("bad char", "fn f() int { return 1 # 2 }"),
    ]);
}
