//! PLAN-1072 刀A（组件面 style）：`style NAME = "literal"` 作者面在
//! age-component 的开口永久用例。
//!
//! 语义（沿 981 r18 会签 (c)3 投影期字面量替换——**非** const 路径）：
//! 组件面 style = 惰性视图数据（不产 Item::Const/Fn、不进 resolve/
//! ModuleGlobalsPlan——981 三锁条件不触发）；view 内 `style: NAME` ident
//! 引用在投影期替换为字面量；重名/跨模块引用/带参调用沿包面同名拒。
//! 参数化/`pub style` 仍是包面专属（组件面文法层边界——style-params
//! 步骤 profile 门控缺席）。
//!
//! 红相（刀前）：`style` 声明使组件面整源解析失败（style-item 仅
//! age-package）——005-login 真源 lane 全体缺席的实底。
use ace_core::source::age_component::{
    compile_component_scenario_states, ComponentInitials, ComponentScenario,
};
use ace_core::source::age_h0::LegacyWidgetAlias;

fn compile(src: &str) -> Result<(), Vec<ace_core::atom_text::Diagnostic>> {
    let caps = vec!["ac.runtime.text.v1".to_string()];
    compile_component_scenario_states(
        src,
        "p1072-style.at",
        LegacyWidgetAlias::Accept,
        &caps,
        &ComponentScenario {
            instances: 1,
            ops: vec![ace_core::source::age_component::ScenarioOp::Send {
                instance: 0,
                msg: "Inc".to_string(),
            }],
        },
        &ComponentInitials {
            per_instance: vec![Vec::new()],
        },
    )
    .map(|_| ())
}

const WIDGET: &str = "\nwidget App {\n    model {\n        var n int = 0\n    }\n    view {\n        col { text \"x\" { style: card } }\n    }\n    on {\n        .Inc -> { .n += 1 }\n    }\n}\n";

/// ①字面量 style 声明 + view `style: NAME` 引用——整源编译通过（替换后
/// 下游无 ident 引用残留）。
#[test]
fn p1072_component_style_literal_compiles() {
    compile(&format!("style card = \"bg-card rounded\"\n{WIDGET}"))
        .expect("literal style compiles");
}

/// ②本地 ident 复制（`style alias_card = card`）——折叠为同值字面量。
/// （`alias` 是全语言面保留字 token——保留字名走文法层拒绝，不在此测。）
#[test]
fn p1072_component_style_local_copy_folds() {
    compile(&format!(
        "style card = \"bg-card rounded\"\nstyle alias_card = card\n{WIDGET}"
    ))
    .expect("local ident copy folds to the literal");
}

/// ③重名 → `source.module-duplicate-declaration`（981 (c)1 同码）。
#[test]
fn p1072_component_style_duplicate_is_named() {
    let err = compile(&format!("style card = \"a\"\nstyle card = \"b\"\n{WIDGET}"))
        .expect_err("duplicate style names are rejected");
    assert_eq!(
        err[0].code, "source.module-duplicate-declaration",
        "{err:?}"
    );
}

/// ④导入名引用（`use themes.themes: Auto` 后 `style: Auto`）→ `style-call`
/// 具名拒（跨模块=包面同族第四臂）。
#[test]
fn p1072_component_style_imported_reference_is_named() {
    let src = "\nwidget App {\n    model {\n        var n int = 0\n    }\n    view {\n        col { text \"x\" { style: Auto } }\n    }\n    on {\n        .Inc -> { .n += 1 }\n    }\n}\n";
    let err = compile(&format!("use themes.themes: Auto, Solarized\n{src}"))
        .expect_err("imported style references are a named pending surface");
    assert_eq!(err[0].code, "source.age.style-call", "{err:?}");
}

/// ⑤带参调用形（`style: card(k: "v")`）→ `style-call` 具名拒。
#[test]
fn p1072_component_style_call_form_is_named() {
    let src = "\nwidget App {\n    model {\n        var n int = 0\n    }\n    view {\n        col { text \"x\" { style: card(k: \"v\") } }\n    }\n}\n";
    let err = compile(&format!("style card = \"a\"\n{src}"))
        .expect_err("style call forms are not folded");
    assert_eq!(err[0].code, "source.age.style-call", "{err:?}");
}

/// ⑥未声明名引用（`style: ghost`）→ 具名拒（不静默落 ident）。
#[test]
fn p1072_component_style_unknown_reference_is_named() {
    let src = "\nwidget App {\n    model {\n        var n int = 0\n    }\n    view {\n        col { text \"x\" { style: ghost } }\n    }\n    on {\n        .Inc -> { .n += 1 }\n    }\n}\n";
    let err = compile(src).expect_err("unknown style references are rejected, not silently kept");
    assert_eq!(err[0].code, "source.age.style-call", "{err:?}");
}

/// ⑦复制目标非本模块字面量（`style alias_card = ghost`）→ `style-call` 具名拒。
#[test]
fn p1072_component_style_copy_of_missing_target_is_named() {
    let err = compile(&format!("style alias_card = ghost\n{WIDGET}"))
        .expect_err("copies of unknown targets are rejected");
    assert_eq!(err[0].code, "source.age.style-call", "{err:?}");
}

/// ⑧参数化声明在组件面=文法层边界（style-params 步骤缺席——解析期拒，
/// 非投影期具名拒；与 `pub style` 同为包面专属形）。
#[test]
fn p1072_component_style_params_stay_package_face() {
    let err = compile(&format!("style sized(p: str) = \"text-sm\"\n{WIDGET}"))
        .expect_err("parameterised styles are a package-face form");
    assert!(
        err[0].code.starts_with("source."),
        "a syntax-level boundary, still a named source diagnostic: {:?}",
        err[0].code
    );
}

/// ⑨`style` 作普通标识符仍合法（981 (d) 负控——不进 token 表）。
#[test]
fn p1072_component_style_as_plain_identifier_stays_legal() {
    let src = "\nwidget App {\n    model {\n        var style int = 1\n    }\n    view {\n        col { text \"x\" {} }\n    }\n    on {\n        .Inc -> { .style += 1 }\n    }\n}\n";
    let _ = compile(src);
}

/// ⑩刀A 第二刀（005 真语料）：`type: "password"` 条目键——`type` 是全局
/// token（type-item 声明头），age-event-key 按 token 直收（spelling 兼容投
/// 影）。红相实证：修复前 005 整源解析失败（错误经恢复链归因到 λ 收尾
/// `}`——"unexpected token in an if branch"）。
#[test]
fn p1072_component_view_entry_type_key_parses() {
    compile(
        "widget App {
    model {
        var email str = \"\"
    }
    view {
        col { input (value: .email) {
            type: \"password\"
        } }
    }
    on {
        .Inc -> { .email = \"\" }
    }
}
",
    )
    .expect("the `type:` entry key parses on the component face");
}
