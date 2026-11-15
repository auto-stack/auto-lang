//! PLAN-981（SOURCE 批，IF-981-ORIGINAL）：**AGE 资源类型与服务**回归。
//!
//! 覆盖作者面真实写法：`material2d.color(r,g,b,a)` → `Material2D`、
//! `mesh2d.quad(w,h)` / `mesh2d.circle(d)` → `Mesh2D`，以及模块级 `let` 从服务
//! 返回类型推断。负控证明这些不是"能解析就算支持"：
//! 参数类型/个数错、未知 op、端口互不相等、非服务初值不猜类型。
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::compile_age_package;
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};

/// 一个最小组件，保证文档里有一个合法 node（不与资源面互相干扰）。
const NODE: &str = "node N {\n    view { text \"t\" {} }\n}\n";

fn input(id: &str, source: &str) -> PackageModuleInput {
    PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: id.into(),
        source: source.into(),
        provider: None,
    }
}

fn compile(src: &str) -> Result<
    ace_core::source::age_package::AgePackageCompilation,
    Vec<ace_core::atom_text::Diagnostic>,
> {
    compile_age_package(
        &[input("main", src)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
    )
}

/// 三个资源服务构造器都能编译，且**模块级** `let` 从返回端口推断出类型。
///
/// 注意源里必须是**模块级** `let`（与测试名一致）：函数体内的 `let a = ...`
/// 只是局部绑定，不进 `description.resources` 面。此前这个用例的源是函数体内
/// `let`，却用一个恒真断言（复审 M6）掩盖了名实不符。
#[test]
fn p981_resource_services_compile_and_let_infers_the_port() {
    let src = format!(
        "{NODE}\
         let a = material2d.color(0.9, 0.3, 0.2, 0.5)\n\
         let m = mesh2d.quad(30.0, 30.0)\n\
         let c = mesh2d.circle(24.0)\n"
    );
    let out = compile(&src).expect("AGE resource services compile");
    // 三个 `let` 都被推断成**有类型的模块全局**，且类型是各自的端口。
    //
    // 旧断言是 `!consts.is_empty() || atom_text.contains("Material2D")`——
    // 后者一旦端口类型被登记就恒真，等于什么都没测（复审 M6）。
    let declared: Vec<(&str, &str)> = out
        .description
        .resources
        .iter()
        .map(|r| (r.name.as_str(), r.port.as_str()))
        .collect();
    assert!(
        declared.contains(&("a", "Material2D")),
        "`a` must be inferred as Material2D: {declared:?}"
    );
    assert!(
        declared.contains(&("m", "Mesh2D")),
        "`m` must be inferred as Mesh2D: {declared:?}"
    );
    assert!(
        declared.contains(&("c", "Mesh2D")),
        "`c` must be inferred as Mesh2D: {declared:?}"
    );
    assert_eq!(declared.len(), 3, "exactly the three bindings: {declared:?}");
}

/// 端口是**名义**类型：`Mesh2D` 不能赋给 `Material2D`。
#[test]
fn p981_resource_ports_are_nominal_not_interchangeable() {
    let src = format!(
        "{NODE}fn f() int {{\n    let a Material2D = mesh2d.circle(24.0)\n    return 1\n}}\n"
    );
    let ds = compile(&src).expect_err("Mesh2D is not a Material2D");
    assert!(
        ds.iter().any(|d| d.code.contains("type-mismatch")),
        "the mismatch is a TYPE error, not a missing symbol: {ds:?}"
    );
}

/// 参数类型必须是 `float`（f32）——整数不是 float。
#[test]
fn p981_resource_service_rejects_wrong_argument_type() {
    let src = format!(
        "{NODE}fn f() int {{\n    let a = material2d.color(1, 2, 3, 4)\n    return 1\n}}\n"
    );
    let ds = compile(&src).expect_err("integer components are not f32");
    assert_eq!(ds[0].code, "source.age.resource-argument", "{ds:?}");
    assert!(ds[0].message.contains("f32"), "{}", ds[0].message);
}

/// 参数个数必须与声明一致。
#[test]
fn p981_resource_service_rejects_wrong_arity() {
    let src = format!(
        "{NODE}fn f() int {{\n    let a = material2d.color(0.9, 0.3)\n    return 1\n}}\n"
    );
    let ds = compile(&src).expect_err("color takes four components");
    assert_eq!(ds[0].code, "source.age.resource-arity", "{ds:?}");
}

/// 未声明的服务操作不被臆造（与"名字不存在"同口径）。
#[test]
fn p981_unknown_resource_service_is_not_invented() {
    let src = format!(
        "{NODE}fn f() int {{\n    let a = material2d.nope(0.9)\n    return 1\n}}\n"
    );
    let ds = compile(&src).expect_err("an undeclared service op is not invented");
    assert_eq!(ds[0].code, "source.undefined", "{ds:?}");
}

/// 非服务初值的模块级 `let` 具名拒绝——不猜类型。
#[test]
fn p981_let_without_service_initializer_is_named() {
    let src = format!("{NODE}let x = 1\n");
    let ds = compile(&src).expect_err("a non-service initializer is not inferred");
    assert_eq!(ds[0].code, "source.module-let-inference", "{ds:?}");
}
