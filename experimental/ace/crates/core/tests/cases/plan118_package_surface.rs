//! PLAN-118（用户裁定①）：AGE 包面两件的最小闭合集回归——
//! ① 显式空参数 handler `.Msg() ->`（088 ColorPicker 真语料形；绑定 0 参、
//!    消息 payload 丢弃）；② 包模块 fn 体 `for <v> in a..b { … }`（范围循环
//!    受理，resolve 脱糖为计数循环）。
//!
//! 两个正例都来自真实组件准入墙的最小复现（`plan-artifacts/plan-118/
//! micro-empty-paren-handler.at`、`micro-range-for-fn.at`）；同时保留负控：
//! 参数表不匹配仍拒、for 之外的范围值仍拒——不做面放宽。
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::compile_age_package;
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};

fn input(source: &str) -> PackageModuleInput {
    PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: "main".into(),
        source: source.into(),
        provider: None,
    }
}

fn caps() -> Vec<String> {
    vec![PACKAGES_CAPABILITY.to_string()]
}

fn compile(
    source: &str,
) -> Result<
    ace_core::source::age_package::AgePackageCompilation,
    Vec<ace_core::atom_text::Diagnostic>,
> {
    compile_age_package(
        &[input(source)],
        "app::main",
        LegacyWidgetAlias::Accept,
        &caps(),
    )
}

#[test]
fn p118_explicit_empty_handler_params_drop_declared_payload() {
    let source = "widget W {\n\
        msg { Go(str) }\n\
        model { var n int = 0 }\n\
        view { text \"x\" {} }\n\
        on {\n\
            .Go() -> { .n = 1 }\n\
        }\n\
    }\n";
    let compiled = compile(source).expect("explicit empty handler params compile");
    assert_eq!(compiled.description.components.len(), 1);

    // 负控：当作隐式零参处理（`Go` 声明 1 payload 槽）仍必须拒——
    // 显式 `()` 是唯一放行形。
    let implicit = source.replace(".Go() ->", ".Go ->");
    let refused = compile(&implicit).expect_err("implicit zero-arg handler must stay refused");
    assert_eq!(refused[0].code, "source.age.component-event");

    // 负控：显式参数个数与载荷不符仍拒。
    let too_many = source.replace(".Go() ->", ".Go(a, b) ->");
    let refused = compile(&too_many).expect_err("mismatched explicit params still refused");
    assert_eq!(refused[0].code, "source.age.component-event");
}

#[test]
fn p118_package_module_fn_accepts_range_for() {    let source = "fn sum_to(n int) int {\n\
        var total int = 0\n\
        for i in 0..n {\n\
            total = total + i\n\
        }\n\
        return total\n\
    }\n\
    widget W {\n\
        view { text \"x\" {} }\n\
    }\n";
    compile(source).expect("package module fn range-for compiles");

    // 负控：范围边界必须是 i32——字符串边界仍具名拒（新受理面不放宽语义）。
    let bad_bounds = "fn bad(n int) int {\n\
        for i in 0..\"x\" {\n\
            n = n + i\n\
        }\n\
        return n\n\
    }\n\
    widget W {\n\
        view { text \"x\" {} }\n\
    }\n";
    let refused = compile(bad_bounds).expect_err("non-i32 range bounds must stay refused");
    assert_eq!(refused[0].code, "source.range-bounds");
}

#[test]
fn p118_child_property_binds_lazy_view_data_list() {
    // 088 ColorPicker 形：父 model 的无类型列表（AGE 包面按惰性 view-data 承载）
    // 直接作为子组件 List 属性值（`palette: .palette`）。属性求值/视图值检查
    // 此前会留下未绑定 `self`（view-data 不进 model 字段表）。
    let parent = "widget App {\n\
        use { package: controls from \"../controls\" }\n\
        model {\n\
            var cur str = \"#000000FF\"\n\
            var palette = [\"#111827FF\"]\n\
        }\n\
        view {\n\
            Child (value: .cur, palette: .palette) {}\n\
        }\n\
    }\n";
    let child = "widget Child (value: str = \"#000000FF\", palette: List = []) {\n\
        view { text \"child\" {} }\n\
    }\n";
    let compiled = compile_age_package(
        &[
            PackageModuleInput {
                package: "app".into(),
                version: "1.0.0".into(),
                id: "main".into(),
                source: parent.into(),
                provider: None,
            },
            PackageModuleInput {
                package: "controls".into(),
                version: "1.0.0".into(),
                id: "child".into(),
                source: child.into(),
                provider: None,
            },
        ],
        "app::main",
        LegacyWidgetAlias::Accept,
        &caps(),
    )
    .expect("child property binds a lazy view-data list");
    assert_eq!(compiled.description.components.len(), 2);
}
