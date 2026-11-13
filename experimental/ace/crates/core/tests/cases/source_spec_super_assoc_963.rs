//! PLAN-963（T-05）：关联类型 + super-spec + `with` 旧拼写的具名裁定。
//!
//! 拼写取自旧用例，不猜：关联类型 `spec Container { type Item }` +
//! `as Container<Item=int>`（`crates/auto-lang/test/a2r/12_specs/011_associated_types`、
//! `012_generic_spec_assoc`）；super-spec `spec Tool: Send + Sync { .. }`
//! （`010_spec_supertrait`）；`#[with(T)]` / `#[with(T as Clone)]`
//! （`a2c/08_generics/004_with_constraint`、`a2r/16_interop/019_multi_bound`）。
//!
//! spec 仍在投影期擦除：关联类型绑定与 super-spec 义务只在投影期核验，下游
//! 只见具体方法签名。

use ace_core::atom_text::Diagnostic;
use ace_core::source::adapter::CompiledSource;
use ace_core::source::compile_record_source;

fn compile(src: &str) -> Result<CompiledSource, Vec<Diagnostic>> {
    compile_record_source(src, "p.at", &[])
}

fn codes(ds: &[Diagnostic]) -> Vec<String> {
    ds.iter().map(|d| d.code.clone()).collect()
}

/// O-963-SA1：关联类型绑定把 spec 方法签名具体化（`Item`→`int`），且实现者
/// 自身签名/体里保留 `Item` 拼写时按绑定替换（旧 417-E2 F1 行为）。
#[test]
fn associated_type_binding_instantiates_required_methods() {
    let src = "\
spec Container {
    type Item
    fn get(i int) Item
    fn first() Item
}
type IntBox as Container<Item=int> {
    data int
    fn get(i int) int { return self.data + i }
    fn first() Item { return self.data }
}
fn main() int {
    let b IntBox = IntBox { data: 40 }
    return b.get(1) + 1
}
";
    let c = compile(src).expect("associated type binding compiles");
    assert!(
        !c.atom_text.contains("Container"),
        "spec declaration leaked into the atom text:\n{}",
        c.atom_text
    );
    assert!(
        !c.atom_text.contains("Item"),
        "associated type name leaked past projection:\n{}",
        c.atom_text
    );
}

/// O-963-SA2：关联类型未绑定 → 具名拒（不按“无该成员”静默通过）。
#[test]
fn missing_associated_type_binding_rejected() {
    let src = "\
spec Container {
    type Item
    fn get(i int) Item
}
type IntBox as Container {
    data int
    fn get(i int) int { return self.data }
}
fn main() int { return 0 }
";
    let err = compile(src).expect_err("an unbound associated type must be rejected");
    assert!(
        err.iter().any(|d| d
            .message
            .contains("does not bind associated type `Item` required by spec `Container`")),
        "{err:#?}"
    );
}

/// O-963-SA3：绑定了未声明的关联类型名 → 具名拒。
#[test]
fn unknown_associated_type_binding_rejected() {
    let src = "\
spec Container {
    type Item
    fn get(i int) Item
}
type IntBox as Container<Item=int, Other=int> {
    data int
    fn get(i int) int { return self.data }
}
fn main() int { return 0 }
";
    let err = compile(src).expect_err("an unknown associated type must be rejected");
    assert!(
        err.iter().any(|d| d
            .message
            .contains("spec `Container` has no associated type `Other`")),
        "{err:#?}"
    );
}

/// O-963-SA4：同一关联类型绑定两次 → `source.duplicate`。
#[test]
fn duplicate_associated_type_binding_rejected() {
    let src = "\
spec Container {
    type Item
    fn get(i int) Item
}
type IntBox as Container<Item=int, Item=bool> {
    data int
    fn get(i int) int { return self.data }
}
fn main() int { return 0 }
";
    let err = compile(src).expect_err("a duplicate association binding must be rejected");
    assert!(
        codes(&err).iter().any(|c| c == "source.duplicate"),
        "{err:#?}"
    );
    assert!(
        err.iter().any(|d| d
            .message
            .contains("associated type `Item` is bound more than once for spec `Container`")),
        "{err:#?}"
    );
}

/// O-963-SA5：通用 spec 的位置实参与关联类型命名绑定混用（旧 012 拼写
/// `as Store<int, Item=str>`，此处用 record 面可用的 `int`）。
#[test]
fn generic_spec_with_associated_type_binding() {
    let src = "\
spec Store<T> {
    type Item
    fn make(v T) Item
}
type IntStore as Store<int, Item=int> {
    data int
    fn make(v int) int { return self.data + v }
}
fn main() int {
    let s IntStore = IntStore { data: 40 }
    return s.make(1) + 1
}
";
    compile(src).expect("generic spec + associated type compiles");
}

/// O-963-SA6：super-spec 的方法义务并入遵从核验——两处都实现才通过。
#[test]
fn super_spec_obligations_are_enforced() {
    let src = "\
spec Named {
    fn name() int
}
spec Tool: Named {
    fn size() int
}
type Wrench as Tool {
    v int
    fn name() int { return 1 }
    fn size() int { return 41 }
}
fn main() int {
    let w Wrench = Wrench { v: 0 }
    return w.name() + w.size()
}
";
    compile(src).expect("both super-spec and own methods are implemented");
}

/// O-963-SA7：缺少 super-spec 的抽象方法 → 具名拒（义务来自 `Named`，不是 `Tool`）。
#[test]
fn missing_super_spec_method_rejected() {
    let src = "\
spec Named {
    fn name() int
}
spec Tool: Named {
    fn size() int
}
type Wrench as Tool {
    v int
    fn size() int { return 41 }
}
fn main() int { return 0 }
";
    let err = compile(src).expect_err("a missing super-spec method must be rejected");
    assert!(
        err.iter().any(|d| d
            .message
            .contains("does not implement `fn name` required by spec `Named`")),
        "{err:#?}"
    );
}

/// O-963-SA8：未声明的 super-spec → 具名拒（不当“无约束”静默放行）。
#[test]
fn unknown_super_spec_rejected() {
    let src = "\
spec Tool: Send {
    fn size() int
}
type Wrench as Tool {
    v int
    fn size() int { return 41 }
}
fn main() int { return 0 }
";
    let err = compile(src).expect_err("an undeclared super-spec must be rejected");
    assert!(
        err.iter().any(|d| d
            .message
            .contains("unknown super-spec `Send` of spec `Tool`")),
        "{err:#?}"
    );
}

/// O-963-SA9：super-spec 成环 → 具名拒（不无限展开）。
#[test]
fn super_spec_cycle_rejected() {
    let src = "\
spec A: B {
    fn a() int
}
spec B: A {
    fn b() int
}
type T as A {
    v int
    fn a() int { return 1 }
    fn b() int { return 2 }
}
fn main() int { return 0 }
";
    let err = compile(src).expect_err("a super-spec cycle must be rejected");
    assert!(
        err.iter().any(|d| d.message.contains("super-spec cycle")),
        "{err:#?}"
    );
}

/// O-963-SA10：通用 spec 作 super-spec → 具名拒（super-spec 只有裸名形）。
#[test]
fn generic_super_spec_rejected() {
    let src = "\
spec Base<T> {
    fn v() T
}
spec Sub: Base {
    fn s() int
}
type T as Sub {
    x int
    fn v() int { return 1 }
    fn s() int { return 2 }
}
fn main() int { return 0 }
";
    let err = compile(src).expect_err("a generic super-spec bound must be rejected");
    assert!(
        err.iter().any(|d| d
            .message
            .contains("is generic; super-spec bounds take no type arguments")),
        "{err:#?}"
    );
}

/// O-963-SA11：`#[with(T)]` 旧注解拼写（Plan 061）在 ACE 无语言属性语法——
/// `semantics-contract` 的 `visibility.attributes` 仍是 SchemaOnly、future_owner
/// 为 "language attribute grammar plan"，故本包按**具名迁移拒**处理，不静默
/// 当作无约束泛型；规范形为 `fn f<T>` / `fn f<T has S>`。
#[test]
fn legacy_with_attribute_is_rejected_with_migration_note() {
    let src = "\
#[with(T)]
fn identity(x T) T { return x }
fn main() int { return 0 }
";
    let err = compile(src).expect_err("an attribute spelling must be rejected, not ignored");
    assert!(
        codes(&err).iter().any(|c| c == "source.syntax"),
        "expected a parse-level rejection, got {err:#?}"
    );
}
