//! PLAN-981（SOURCE 批，IF-981-REF）：**实例引用的可见性**回归。
//!
//! 979 V-10 要求：同类型的**合法引用**、**未声明引用**、**已声明但不可见引用**
//! 必须被隔离成三种不同的具名原因（"是引用/可见性诊断而非偶然类型错"），带原
//! 模块/字段 span；合法近邻两端真跑。本文件就是这三条 + 合法近邻。
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::compile_age_package;
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};

/// 被引用的 node 声明模块：`Meter` 是 `pub node`（跨模块可见），`Secret`
/// 不是（跨模块不可见），`Plain` 是普通 type（不是 node）。
const WIDGETS: &str = "pub node Meter {\n\
    view { text \"m\" {} }\n\
}\n\
node Secret {\n\
    view { text \"s\" {} }\n\
}\n\
pub type Plain {\n\
    x int\n\
}\n";

fn input(id: &str, source: &str) -> PackageModuleInput {
    PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: id.into(),
        source: source.into(),
        provider: None,
    }
}

fn caps() -> Vec<String> {
    vec![PACKAGES_CAPABILITY.to_string()]
}

fn compile(main: &str) -> Result<
    ace_core::source::age_package::AgePackageCompilation,
    Vec<ace_core::atom_text::Diagnostic>,
> {
    compile_age_package(
        &[input("widgets", WIDGETS), input("main", main)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &caps(),
    )
}



/// 同模块引用不需要导入（可见性只约束跨模块）。
#[test]
fn p981_ref_same_module_reference_needs_no_import() {
    let main = "node Inner {\n\
    view { text \"i\" {} }\n\
}\n\
node Outer {\n\
    view {\n\
        text \"o\" {}\n\
        Inner one {}\n\
    }\n\
}\n";
    let out = compile(main).expect("same-module reference is legal");
    let outer = out
        .description
        .components
        .iter()
        .find(|c| c.name == "Outer")
        .expect("Outer");
    assert_eq!(outer.refs.len(), 1);
    assert_eq!(outer.refs[0].module, "app::main");
    assert_eq!(outer.refs[0].node, "Inner");
}

/// ① 未声明：名字在任何模块都不存在 → `source.age.ref-undeclared`。
#[test]
fn p981_ref_undeclared_is_named_distinctly() {
    let main = "node Screen {\n\
    view {\n\
        text \"hi\" {}\n\
        Nowhere m1 {}\n\
    }\n\
}\n";
    let ds = compile(main).expect_err("an undeclared node reference is rejected");
    assert_eq!(ds[0].code, "source.age.ref-undeclared", "{ds:?}");
}

/// ② 已声明但不可见：`Secret` 存在但没有 `pub`，且未被导入 →
///    `source.age.ref-invisible`（与"不存在"分开报）。
#[test]
fn p981_ref_invisible_is_named_distinctly() {
    let main = "node Screen {\n\
    view {\n\
        text \"hi\" {}\n\
        Secret m1 {}\n\
    }\n\
}\n";
    let ds = compile(main).expect_err("a non-public node reference is rejected");
    assert_eq!(ds[0].code, "source.age.ref-invisible", "{ds:?}");
}

/// ③ 已声明且导入，但**不是 node**（是普通 type）→ `source.age.ref-not-a-node`。
///    这三条互斥可辨，正是 V-10 要的"引用/可见性诊断，而非偶然类型错"。
#[test]
fn p981_ref_non_node_type_is_named_distinctly() {
    let main = "use widgets.Plain\n\
node Screen {\n\
    view {\n\
        text \"hi\" {}\n\
        Plain m1 {}\n\
    }\n\
}\n";
    let ds = compile(main).expect_err("a non-node type cannot be an instance");
    assert_eq!(ds[0].code, "source.age.ref-not-a-node", "{ds:?}");
}

/// 诊断带**原模块** span（不落在 helper 段、不越界）。
#[test]
fn p981_ref_diagnostics_carry_the_original_module_span() {
    let main = "node Screen {\n\
    view {\n\
        text \"hi\" {}\n\
        Nowhere m1 {}\n\
    }\n\
}\n";
    let out = compile(main);
    let ds = out.expect_err("undeclared reference rejected");
    // 用同一工程的成功编译取得 main 模块的源坐标。
    let ok = compile("node Screen {\n    view { text \"hi\" {} }\n}\n").expect("base compiles");
    let main_mod = ok
        .description
        .modules
        .iter()
        .find(|m| m.key == "app::main")
        .expect("main module");
    assert!(
        ds.iter().any(|d| {
            d.span.start >= main_mod.flat_start
                && d.span.start < main_mod.flat_start + main_mod.source_len
        }),
        "the reference diagnostic maps into main.at: {ds:?}"
    );
}



/// 跨模块可见性（本轮闭环）：`pub node` 可被别的模块按 node 名引用并解析到声明
/// 模块；非 `pub` 的同名声明则报 `ref-invisible`。
#[test]
fn p981_ref_cross_module_pub_node_resolves_and_private_does_not() {
    let public =
        "node Screen {\n    view {\n        text \"hi\" {}\n        Meter m1 {}\n    }\n}\n";
    let out = compile(public).expect("a public cross-module node is referenceable");
    let screen = out
        .description
        .components
        .iter()
        .find(|c| c.name == "Screen")
        .expect("Screen");
    assert_eq!(screen.refs.len(), 1);
    assert_eq!(screen.refs[0].module, "app::widgets");
    assert_eq!(screen.refs[0].node, "Meter");

    let private =
        "node Screen {\n    view {\n        text \"hi\" {}\n        Secret m1 {}\n    }\n}\n";
    let ds = compile(private).expect_err("a private cross-module node is rejected");
    assert_eq!(ds[0].code, "source.age.ref-invisible", "{ds:?}");
}

/// `pub node` 自身必须留下可见性事实（声明侧，不只是引用侧）。
#[test]
fn p981_ref_pub_node_records_its_visibility() {
    let out = compile("node Screen {\n    view { text \"x\" {} }\n}\n")
        .expect("plain fixture compiles");
    let _ = out; // plain nodes stay private
    let src = "pub node Only {\n    view { text \"o\" {} }\n}\n";
    let out = compile_age_package(
        &[input("main", src)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &caps(),
    )
    .expect("a public node compiles");
    let only = out
        .description
        .components
        .iter()
        .find(|c| c.name == "Only")
        .expect("Only projects (it is no longer silently dropped)");
    assert!(only.public, "the `pub` declaration must be recorded");
}
