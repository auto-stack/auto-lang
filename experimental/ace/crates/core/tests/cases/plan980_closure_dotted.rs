//! PLAN-980 ④-A 首切片：AGE 包面的**点分成员导入投影 + 引用驱动（惰性）绑定**。
//!
//! 输入由真实例派生（`examples/ui/ace-node-counter` 的 node 形态 + 计数业务），
//! 仅追加两条真实作者拼写的导入：被引用的跨包 helper（点分 `use util.steps: step`）
//! 与未被引用的 themes（点分 `use themes.themes: Auto, Solarized`，目标模块**缺席**）。
//! 判据（用户 2026-11-18）：①未引用 themes 不拉入模块；②被引用 helper 进闭包并改变
//! 结果；③改 helper 源改受检程序；④缺模块/缺符号/私有符号分别具名拒绝。
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::compile_age_package;
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};

/// ace-node-counter 派生的作者源：计数业务保持（0→1、重置），新增两条点分导入。
fn derived_counter(helper_import: &str, body_extra: &str) -> String {
    format!(
        "use {helper_import}\n\
use themes.themes: Auto, Solarized\n\
\n\
node Counter {{\n\
    msg {{ Inc, Reset }}\n\
\n\
    model {{\n\
        var count int = step(0)\n\
    }}\n\
\n\
    view {{\n\
        col {{\n\
            text \"Counter\"\n\
            row {{\n\
                button \"Reset\" {{ onclick: .Reset }}\n\
                button \"+\" {{ onclick: .Inc }}\n\
            }}\n\
        }}\n\
    }}\n\
\n\
    on {{\n\
        .Inc -> {{ .count = step(.count) }}\n\
        .Reset -> {{ .count = 0 }}\n\
    }}\n\
}}{body_extra}\n"
    )
}

fn unit(pkg: &str, id: &str, src: &str) -> PackageModuleInput {
    PackageModuleInput {
        package: pkg.into(),
        version: "1.0.0".into(),
        id: id.into(),
        source: src.into(),
        provider: None,
    }
}

fn compile(
    main: &str,
    helper: &str,
) -> Result<
    ace_core::source::age_package::AgePackageCompilation,
    Vec<ace_core::atom_text::Diagnostic>,
> {
    let caps = vec![PACKAGES_CAPABILITY.to_string()];
    compile_age_package(
        &[unit("app", "main", main), unit("util", "steps", helper)],
        "app::main",
        LegacyWidgetAlias::Accept,
        &caps,
    )
}

const HELPER: &str = "pub fn step(n int) int {\n    return n + 2\n}\n";

/// ①点分导入被引用 → helper 进闭包（模块图含 util::steps）；未引用的 themes 不拉入。
#[test]
fn p980_closure_dotted_use_binds_helper_and_skips_inert_import() {
    let main = derived_counter("util.steps: step", "");
    let out = compile(&main, HELPER).expect("derived counter + dotted helper compiles");
    let keys: Vec<&str> = out
        .description
        .modules
        .iter()
        .map(|m| m.key.as_str())
        .collect();
    assert_eq!(
        keys,
        vec!["app::main", "util::steps"],
        "referenced helper is in the closure; unreferenced themes is not pulled in"
    );
    assert_eq!(out.description.components.len(), 1);
    assert_eq!(out.description.components[0].name, "Counter");
}

/// ②改 helper 源 → 受检程序变（helper 真的参与编译，不是声明摆设）。
#[test]
fn p980_closure_helper_edit_changes_the_checked_program() {
    let main = derived_counter("util.steps: step", "");
    let base = compile(&main, HELPER).expect("base compiles");
    let changed = HELPER.replace("return n + 2", "return n + 10");
    let edited = compile(&main, &changed).expect("edited helper compiles");
    assert_ne!(
        base.atom_text, edited.atom_text,
        "editing the imported helper must change the checked program"
    );
}

/// ③缺模块 / 缺符号 / 私有符号：具名拒绝（不静默、不退化成 undefined）。
#[test]
fn p980_closure_missing_module_symbol_and_private_are_named() {
    let caps = vec![PACKAGES_CAPABILITY.to_string()];
    let main = derived_counter("util.steps: step", "");

    // 缺模块：被引用的导入指向缺席模块。
    let missing = compile_age_package(
        &[unit("app", "main", &main)],
        "app::main",
        LegacyWidgetAlias::Accept,
        &caps,
    )
    .expect_err("missing module must refuse");
    assert_eq!(missing[0].code, "source.module-missing", "{:?}", missing);

    // 缺符号：模块在场但声明没这个名字（被引用）。
    let wrong_symbol = derived_counter("util.steps: nope", "");
    let err = compile(&wrong_symbol, HELPER).expect_err("missing symbol must refuse");
    assert_eq!(err[0].code, "source.module-symbol", "{:?}", err);

    // 私有符号：目标声明没有 `pub`。
    let private_helper = "fn step(n int) int {\n    return n + 2\n}\n";
    let err = compile(&main, private_helper).expect_err("private symbol must refuse");
    assert_eq!(err[0].code, "source.module-private", "{:?}", err);
}

/// ④未引用的互指导入**不建模块边**（A3 的直接推论）：原 981 环用例的源在
/// ④-A 合同下是**正例**——两边导入都只出现在导入行（未被引用），闭包照常成立。
#[test]
fn p980_lazy_unreferenced_cyclic_imports_do_not_form_edges() {
    let caps = vec![PACKAGES_CAPABILITY.to_string()];
    let aa = "use bb.B\nnode A {\n    view { text \"a\" {} }\n}\n";
    let bb = "use aa.A\nnode B {\n    view { text \"b\" {} }\n}\n";
    let main = "use aa.A\nnode Main {\n    view { text \"m\" {} }\n}\n";
    let out = compile_age_package(
        &[
            unit("app", "aa", aa),
            unit("app", "bb", bb),
            unit("app", "main", main),
        ],
        "app::main",
        LegacyWidgetAlias::Accept,
        &caps,
    )
    .expect("unreferenced imports create no module edges, so no cycle");
    assert_eq!(out.description.components.len(), 3);
}
