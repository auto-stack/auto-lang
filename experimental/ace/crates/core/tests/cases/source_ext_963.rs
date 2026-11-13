//! PLAN-963（T-03）：扩展块 `ext Type { members }`。
//!
//! 覆盖：ext 方法在类型上可调用（`.` 接收者与显式参数）；ext 可满足
//! `type X as Spec` 的抽象方法（legacy `generic_spec_ext` 形态）；未知类型、
//! 重复方法、ext 里声明字段各自具名拒。

use ace_core::atom_text::Diagnostic;
use ace_core::source::adapter::CompiledSource;
use ace_core::source::compile_record_source;

fn compile(src: &str) -> Result<CompiledSource, Vec<Diagnostic>> {
    compile_record_source(src, "p.at", &[])
}

fn codes(ds: &[Diagnostic]) -> Vec<String> {
    ds.iter().map(|d| d.code.clone()).collect()
}

/// O-963-E1：ext 方法挂到已有类型，接收者与显式参数都可用。
#[test]
fn ext_methods_attach_to_the_type() {
    let src = "\
type Heap { ptr int }
ext Heap {
    fn get() int { return .ptr }
    fn bump(by int) int { return .ptr + by }
}
fn main() int {
    let h Heap = Heap { ptr: 40 }
    return h.bump(h.get() - 38)
}
";
    let c = compile(src).expect("ext methods compile");
    assert!(c.atom_text.contains("bump"), "{}", c.atom_text);
}

/// O-963-E2：ext 满足 spec 抽象方法（legacy `generic_spec_ext` 形态的整数版）；
/// 去掉 ext 后同一类型必须被拒（不静默通过）。
#[test]
fn ext_satisfies_a_spec_requirement() {
    let with_ext = "\
spec Storage { fn get() int }
type Heap as Storage { ptr int }
ext Heap {
    fn get() int { return .ptr }
}
fn main() int {
    let h Heap = Heap { ptr: 7 }
    return h.get()
}
";
    compile(with_ext).expect("ext satisfies the spec");

    let without_ext = "\
spec Storage { fn get() int }
type Heap as Storage { ptr int }
fn main() int { return 0 }
";
    let err = compile(without_ext).expect_err("missing implementation must be rejected");
    assert!(
        err.iter().any(|d| d.message.contains("does not implement")),
        "{err:#?}"
    );
}

/// O-963-E3：`ext` 指向未知类型 → `source.unsupported`。
#[test]
fn ext_of_unknown_type_is_rejected() {
    let src = "\
ext Nope {
    fn get() int { return 0 }
}
fn main() int { return 0 }
";
    let err = compile(src).expect_err("unknown type in ext must be rejected");
    assert!(
        codes(&err).iter().any(|c| c == "source.unsupported"),
        "{err:#?}"
    );
    assert!(
        err.iter()
            .any(|d| d.message.contains("unknown type `Nope`")),
        "{err:#?}"
    );
}

/// O-963-E4：ext 里重复声明已有方法 → `source.duplicate`。
#[test]
fn ext_duplicate_method_is_rejected() {
    let src = "\
type Heap { ptr int
    fn get() int { return .ptr }
}
ext Heap {
    fn get() int { return .ptr + 1 }
}
fn main() int { return 0 }
";
    let err = compile(src).expect_err("duplicate ext method must be rejected");
    assert!(
        codes(&err).iter().any(|c| c == "source.duplicate"),
        "{err:#?}"
    );
}

/// O-963-E5：ext 里声明字段被具名拒（布局只能在声明处固定）。
#[test]
fn ext_field_is_rejected() {
    let src = "\
type Heap { ptr int }
ext Heap {
    extra int
}
fn main() int { return 0 }
";
    let err = compile(src).expect_err("ext field must be rejected");
    assert!(
        codes(&err).iter().any(|c| c == "source.unsupported"),
        "{err:#?}"
    );
    assert!(
        err.iter().any(|d| d.message.contains("methods only")),
        "{err:#?}"
    );
}
