//! PLAN-963（T-03）：通用 spec `spec S<T>` + `as S<int>` 实例化。
//!
//! spec 本体仍在投影期擦除；`as S<int>` 的实参按声明序替换 spec 方法的
//! 参数/返回/默认体里的类型参数，再参与遵从核验与方法注入。覆盖：int/bool
//! 两个实例各自具体化、arity 不符（少/多）具名拒、非通用 spec 被给实参
//! 具名拒、`const` 类型参数具名拒、默认体里的别名在注入后再展开。

use ace_core::atom_text::Diagnostic;
use ace_core::source::adapter::CompiledSource;
use ace_core::source::compile_record_source;

fn compile(src: &str) -> Result<CompiledSource, Vec<Diagnostic>> {
    compile_record_source(src, "p.at", &[])
}

fn codes(ds: &[Diagnostic]) -> Vec<String> {
    ds.iter().map(|d| d.code.clone()).collect()
}

const SPEC_INSTANCES: &str = "\
spec Boxed<T> { fn value(x T) T { return x } }
type AsInt as Boxed<int> { v int }
type AsBool as Boxed<bool> { v bool }
fn main() int {
    let a AsInt = AsInt { v: 1 }
    let b AsBool = AsBool { v: true }
    if b.value(true) { return a.value(41) + 1 }
    return 0
}
";

/// O-963-S1：同一个通用 spec 的两个实例各自按实参具体化（`T=int` 的默认体
/// 返回 `int`，`T=bool` 的返回 `bool`——任一未替换都会被下游以未知类型拒）。
#[test]
fn generic_spec_instantiates_per_conformance() {
    let c = compile(SPEC_INSTANCES).expect("generic spec instances compile");
    assert!(
        !c.atom_text.contains("Boxed"),
        "spec declaration leaked into the atom text:\n{}",
        c.atom_text
    );
}

/// O-963-S2：`as S` 少给实参 → `source.unsupported`（不按无约束 spec 放行）。
#[test]
fn generic_spec_missing_arguments_rejected() {
    let src = "\
spec Boxed<T> { fn value() T { return 0 } }
type AsInt as Boxed { v int }
fn main() int { return 0 }
";
    let err = compile(src).expect_err("missing type arguments must be rejected");
    assert!(
        codes(&err).iter().any(|c| c == "source.unsupported"),
        "{err:#?}"
    );
    assert!(
        err.iter()
            .any(|d| d.message.contains("expects 1 type argument(s), found 0")),
        "{err:#?}"
    );
}

/// O-963-S3：`as S<int, bool>` 多给实参 → `source.unsupported`。
#[test]
fn generic_spec_extra_arguments_rejected() {
    let src = "\
spec Boxed<T> { fn value() T { return 0 } }
type AsInt as Boxed<int, bool> { v int }
fn main() int { return 0 }
";
    let err = compile(src).expect_err("extra type arguments must be rejected");
    assert!(
        err.iter()
            .any(|d| d.message.contains("expects 1 type argument(s), found 2")),
        "{err:#?}"
    );
}

/// O-963-S4：非通用 spec 被给实参 → 具名拒（不静默丢实参）。
#[test]
fn non_generic_spec_rejects_type_arguments() {
    let src = "\
spec Plain { fn value() int { return 0 } }
type AsInt as Plain<int> { v int }
fn main() int { return 0 }
";
    let err = compile(src).expect_err("type arguments on a plain spec must be rejected");
    assert!(
        err.iter()
            .any(|d| d.message.contains("expects 0 type argument(s), found 1")),
        "{err:#?}"
    );
}

/// O-963-S5：spec 的类型参数表只接受类型参数——`<const N int>` 在记录面即被
/// 实例能力门具名拒（`source.instances-unsupported`），不静默忽略 `const`。
#[test]
fn generic_spec_const_parameter_rejected() {
    let src = "\
spec Boxed<const N int> { fn value() int { return 0 } }
type AsInt as Boxed<3> { v int }
fn main() int { return 0 }
";
    let err = compile(src).expect_err("const spec parameters must be rejected");
    assert!(
        codes(&err)
            .iter()
            .any(|c| c == "source.instances-unsupported"),
        "{err:#?}"
    );
    assert!(
        err.iter()
            .any(|d| d.message.contains("const (value) type parameters")),
        "{err:#?}"
    );
}

/// O-963-S6：spec 默认体里的别名在注入后再展开（spec 声明不在 items 里，
/// 首遍别名展开覆盖不到；未展开时下游会以未知类型拒）。
#[test]
fn spec_default_body_alias_is_expanded() {
    let src = "\
type Age = int
spec Aged { fn age() Age { return 41 } }
type Person as Aged { name int }
fn main() int {
    let p Person = Person { name: 1 }
    return p.age() + 1
}
";
    compile(src).expect("alias in an injected default body is expanded");
}
