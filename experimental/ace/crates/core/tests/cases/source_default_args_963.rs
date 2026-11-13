//! PLAN-963（T-08，L-24）：声明位默认实参（`fn f(n int = 42)`）。
//!
//! 语义边界（V-08）：缺席实参才补默认值；补位在**调用者**作用域解析，故
//! 默认值不得命名参数/局部（fail-closed，具名拒）；位置式只能补末尾缺席位，
//! 命名式可补任意缺席位；显式传入的实参一律优先，默认值不再参与。

use ace_core::atom_text::Diagnostic;
use ace_core::source::adapter::CompiledSource;
use ace_core::source::compile_record_source;

fn compile(src: &str) -> Result<CompiledSource, Vec<Diagnostic>> {
    compile_record_source(src, "d963.at", &[])
}

/// O-963-D1：位置式调用省略末尾默认实参可编译并通过约束（补位后 arity 满）。
#[test]
fn default_argument_fills_trailing_positional_963() {
    let src = "\
fn rep(x int, n int = 3) int { return x * n }
fn main() int {
    return rep(14)
}
";
    let c = compile(src).expect("omitted trailing default compiles");
    assert!(c.atom_text.contains("rep"), "callee missing:\n{}", c.atom_text);
}

/// O-963-D2：命名式调用可省略**任意**带默认值的参数。
#[test]
fn default_argument_fills_named_hole_963() {
    let src = "\
fn box(a int = 40, b int = 2) int { return a + b }
fn main() int {
    return box(b: 2)
}
";
    compile(src).expect("named call with a defaulted hole compiles");
}

/// O-963-D3：显式实参优先，默认值不覆盖；两个调用位互不影响。
#[test]
fn explicit_argument_wins_over_default_963() {
    let src = "\
fn rep(x int, n int = 3) int { return x * n }
fn main() int {
    let a int = rep(14)
    let b int = rep(14, 2)
    return a + b
}
";
    compile(src).expect("explicit argument overrides the default");
}

/// O-963-D4：默认值命名参数/局部 → 具名拒（不在调用者作用域静默绑定）。
#[test]
fn default_argument_naming_a_param_is_rejected_963() {
    let src = "\
fn scale(x int, k int = x) int { return x * k }
fn main() int {
    return scale(2)
}
";
    let err = compile(src).expect_err("a default naming a parameter must be rejected");
    assert!(
        err.iter()
            .any(|d| d.code == "source.undefined" && d.message.contains('x')),
        "expected a named undefined-name rejection, got {err:?}"
    );
}

/// O-963-D5：省略**无默认值**的参数仍是 arity 拒（默认值不是任意参数兜底）。
#[test]
fn missing_argument_without_default_still_rejected_963() {
    let src = "\
fn rep(x int, n int = 3) int { return x * n }
fn main() int {
    return rep()
}
";
    let err = compile(src).expect_err("missing required argument stays an arity error");
    assert!(
        err.iter().any(|d| d.code == "source.argument"),
        "expected an arity rejection, got {err:?}"
    );
}

/// O-963-D16（V-G05「可变参数」边界）：AutoLang 无用户声明位可变参数拼写
/// （legacy `Param`/AST 无 rest/variadic 字段，唯一「varargs」用例是 builtin
/// `List.new(1,2,3)`）；ACE 侧同样不可表达，`...` 形参数在语法层具名拒，
/// 不静默接受成普通参数。
#[test]
fn variadic_parameter_form_is_rejected_963() {
    let src = "\
fn sum(xs ...) int { return 0 }
fn main() int {
    return sum(1, 2)
}
";
    let err = compile(src).expect_err("a variadic parameter form is not part of the surface");
    assert!(
        err.iter().any(|d| d.code == "source.syntax"),
        "expected a syntax-stage rejection, got {err:?}"
    );
}

/// O-963-D6：实参过多仍是 arity 拒；默认值不改变声明的可见元数上限。
#[test]
fn too_many_arguments_still_rejected_963() {
    let src = "\
fn rep(x int, n int = 3) int { return x * n }
fn main() int {
    return rep(1, 2, 3)
}
";
    let err = compile(src).expect_err("extra argument stays an arity error");
    assert!(
        err.iter().any(|d| d.code == "source.argument"),
        "expected an arity rejection, got {err:?}"
    );
}
