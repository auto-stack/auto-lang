//! PLAN-981（SOURCE 批，轮 49）：**`model` 字段类型在两个 AGE 面之间必须一致**。
//!
//! 轮 48 复审（R47-3）实测同一 node body 在两个前门被判成不同结果：
//!
//! - `count int + flag bool`：979 组件面 **接受**，包面 `source.type-mismatch:
//!   expected i32, found bool`——因为包面注入的状态函数把返回类型**硬编码成 i32**。
//! - `f float`：979 具名 `source.age.component-unsupported`（指向类型 token），
//!   包面却是 `source.syntax: expected \`fn\`, found \`node\`` **@0**——因为 `float`/
//!   `f32` 在 age-package 面是**关键字**（场景属性/参数需要），而 `age-model-ty`
//!   当时没有对应的 alt，整个 node 在解析期就退化。
//!
//! 现在：状态函数返回**字段自己的类型**（int/i32/bool），不支持的类型在**类型
//! token 处**具名拒绝（两个面同一诊断码）；包面补了 float/f32 的 alt。
use ace_core::source::age_component::{compile_component_scenario, ComponentScenario};
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::{compile_age_package, AgePackageCompilation};
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};

fn input(id: &str, source: &str) -> PackageModuleInput {
    PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: id.into(),
        source: source.into(),
        provider: None,
    }
}

fn compile_pkg(src: &str) -> Result<AgePackageCompilation, Vec<ace_core::atom_text::Diagnostic>> {
    compile_age_package(
        &[input("main", src)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
    )
}

fn compile_979(src: &str) -> Result<(), Vec<ace_core::atom_text::Diagnostic>> {
    let scenario = ComponentScenario {
        instances: 1,
        ops: Vec::new(),
    };
    compile_component_scenario(
        src,
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
        &scenario,
    )
    .map(|_| ())
}

/// 包面接受 `int` + `bool` 混合 model，且两个字段都真的进了求值链。
#[test]
fn p981_package_model_accepts_int_and_bool() {
    let out = compile_pkg(
        "node M {\n    model {\n        count int = 0\n        flag bool = true\n    }\n    view { text \"t\" {} }\n}\n",
    )
    .expect("int + bool model fields are representable");
    let c = &out.description.components[0];
    assert_eq!(
        c.fields,
        vec![("count".into(), "int".into()), ("flag".into(), "bool".into())]
    );
    assert!(c.initialized, "every field got a state function");
    assert_eq!(
        c.field_init_cpu
            .iter()
            .find(|(k, _)| k == "flag")
            .expect("flag init")
            .1,
        ace_core::source::age_package::AgeSceneValue::Bool(true)
    );
    // 状态函数的返回类型跟着字段走（不再是硬编码 i32）：两个符号都在。
    assert!(out.symbol_of("__age_state_0_flag").is_some());
    assert!(out.symbol_of("__age_state_0_count").is_some());
}

/// 纯 `bool` model 在包面同样可编（bool 不是"不可观测字段"就被拒的字段）。
#[test]
fn p981_package_model_accepts_a_bool_only_model() {
    let out = compile_pkg(
        "node M {\n    model {\n        flag bool = false\n    }\n    view { text \"t\" {} }\n}\n",
    )
    .expect("a bool-only model compiles in the package face");
    assert!(out.description.components[0].initialized);
}

/// 979 组件面的同一输入也接受——两面结论一致（R47-3 的核心）。
#[test]
fn p981_component_scenario_accepts_int_and_bool() {
    compile_979(
        "node M {\n    model {\n        count int = 0\n        flag bool = true\n    }\n    on Inc { count = count + 1 }\n    view { text \"t\" {} }\n}\n",
    )
    .expect("the 979 scenario face accepts the same model");
}

/// `float` 在两个面都**具名**拒绝，且诊断指向类型 token（不是 @0 的语法错）。
#[test]
fn p981_model_float_is_named_rejected_at_the_type_token_in_both_faces() {
    let src = "node M {\n    model {\n        f float = 1.0\n    }\n    view { text \"t\" {} }\n}\n";
    let want = src.find("float").expect("fixture has float");
    let ds = compile_pkg(src).expect_err("float is not a supported model field type");
    assert_eq!(ds[0].code, "source.age.component-unsupported", "{ds:?}");
    assert_eq!(
        ds[0].span.start, want,
        "the package face must point at the type token, not @0"
    );
    let ds979 = compile_979(src).expect_err("float is rejected by the component face too");
    assert_eq!(
        ds979[0].code, "source.age.component-unsupported",
        "the two faces must use the same named reason: {ds979:?}"
    );
}

/// `f32` 同样在类型 token 处具名拒绝（它也是 age-package 面的关键字）。
#[test]
fn p981_model_f32_is_named_rejected_at_the_type_token() {
    let src = "node M {\n    model {\n        f f32 = 1.0\n    }\n    view { text \"t\" {} }\n}\n";
    let want = src.find("f32").expect("fixture has f32");
    let ds = compile_pkg(src).expect_err("f32 is not a supported model field type");
    assert_eq!(ds[0].code, "source.age.component-unsupported", "{ds:?}");
    assert_eq!(ds[0].span.start, want);
}

/// `str` model 字段仍按具名原因拒绝（回归：这条本来就对）。
#[test]
fn p981_model_str_is_named_rejected_at_the_type_token() {
    let src = "node M {\n    model {\n        s str = \"x\"\n    }\n    view { text \"t\" {} }\n}\n";
    let want = src.find("str").expect("fixture has str");
    let ds = compile_pkg(src).expect_err("str is not a supported model field type");
    assert_eq!(ds[0].code, "source.age.component-unsupported", "{ds:?}");
    assert_eq!(ds[0].span.start, want);
}