//! PLAN-981（SOURCE 批，轮 49；**D-981-AGE-F32-SURFACE 关闭批，2026-11-18**）：
//! **`model` 字段类型在两个 AGE 面之间必须一致**。
//!
//! 历史（轮 48/R47-3）：同一 node body 在两个前门被判成不同结果——包面把状态
//! 函数返回类型硬编码成 i32；`f float`/`f32` 在包面因关键字 token 无 alt 而整
//! 个 node 在解析期退化。轮 49 修了两面一致性（状态函数返回字段自己的类型），
//! 但 f32/float 仍按"切片不支持"在类型 token 处具名拒。
//!
//! 本批（D-981-AGE-F32-SURFACE）：`f32`（及历史同义拼写 `float`）成为合法
//! model 字段类型（`field_type` → `TypeWord::Float`），两面同一码；f32 **注解位**
//! （fn 参数/返回/`let`）在 age-package 面开放（`ann-ty`/`ret-ty` 增 f32 臂）；
//! f32 同型四则/比较由共享 resolver/adapter 的 typed-numeric 面承载——AV 执行
//! 证据见 `av/tests/cases/plan981_f32_surface.rs`（`x = x + vx * dt` 真落值）。
use ace_core::source::age_component::{
    compile_component_scenario, compile_component_scenario_states, ComponentInitials,
    ComponentScenario,
};
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

/// `float` / `f32` 现在都是**合法** model 字段类型，两个面同一结论
///（D-981-AGE-F32-SURFACE；此前两条用例断言"类型 token 处具名拒"，本批翻正）。
#[test]
fn p981_model_float_and_f32_fields_are_supported_in_both_faces() {
    for ty in ["float", "f32"] {
        let src = format!(
            "node M {{\n    model {{\n        f {ty} = 1.0\n    }}\n    view {{ text \"t\" {{}} }}\n}}\n"
        );
        // 包面：字段类型/状态函数返回类型都跟着 f32 走。
        let out = compile_pkg(&src).expect("the package face holds an f32 model field");
        let c = &out.description.components[0];
        assert_eq!(c.fields, vec![("f".to_string(), ty.to_string())]);
        assert!(
            out.symbol_of("__age_state_0_f").is_some(),
            "the f32 state function is generated for `{ty}`"
        );
        // 组件面：同一输入在 states-only 车道（免驱动打包）同样接受。
        let scenario = ComponentScenario {
            instances: 1,
            ops: Vec::new(),
        };
        let initials = ComponentInitials {
            per_instance: vec![Vec::new()],
        };
        let c979 = compile_component_scenario_states(
            &src,
            "app::main",
            LegacyWidgetAlias::Reject,
            &[PACKAGES_CAPABILITY.to_string()],
            &scenario,
            &initials,
        )
        .expect("the component face holds the same f32 model field");
        assert_eq!(
            c979.description.nodes[0].fields,
            vec![("f".to_string(), ty.to_string())]
        );
    }
}

/// f32 **注解位**在 age-package 面开放：fn 参数/返回/`let` 注解三处同词，
/// 同型四则经共享 typed-numeric 面通过；混宽仍具名拒（不做隐式提升）。
#[test]
fn p981_package_f32_annotation_positions_parse_and_type() {
    let ok = "node M {\n    model {\n        x f32 = 0.0\n    }\n    view { text \"t\" {} }\n}\n\npub fn step(x f32, vx f32, dt f32) f32 {\n    return x + vx * dt\n}\n\npub fn scaled() f32 {\n    let k f32 = 2.0\n    return k * 0.5\n}\n";
    compile_pkg(ok).expect("f32 annotations in fn params/return/let verify");

    let mixed = ok.replace("return x + vx * dt", "return x + 1");
    let ds = compile_pkg(&mixed).expect_err("mixed f32/int must be a named type mismatch");
    assert!(
        ds.iter().any(|d| d.code == "source.type-mismatch"),
        "named mixed-width rejection: {ds:?}"
    );
}

/// resync（并入 980 B-DIALECT 面）后 `str`/`Str` 成为合法 model 字段类型：状态函数
/// 按 `Str` 返回（975/980 的传统 widget 002 形）。本用例钉住合并后的真实行为——
/// 旧的"str 在类型 token 具名拒"是当时实现的限制，不是契约。
#[test]
fn p981_model_str_field_is_supported_after_the_980_surface() {
    let src = "node M {\n    model {\n        s str = \"x\"\n    }\n    view { text \"t\" {} }\n}\n";
    let out = compile_pkg(src).expect("str model fields are supported by the merged surface");
    let c = &out.description.components[0];
    assert_eq!(c.fields, vec![("s".to_string(), "str".to_string())]);
    assert!(c.initialized, "the Str state function is generated");
    assert!(out.symbol_of("__age_state_0_s").is_some());
}