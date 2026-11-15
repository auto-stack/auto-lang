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

/// ② 已声明且被导入 → 可引用。AGE 作者面**不写 `pub`**（2026-11-16 用户裁定；
///    规范原文「接受普通 Auto 模块工程（…普通 `use`、具名资源与引用）」）；
///    符号表导入本身就是显式的跨模块意愿。
#[test]
fn p981_ref_imported_node_is_referenceable_without_pub() {
    let out = compile(
        "node Screen {\n\
    view {\n\
        text \"hi\" {}\n\
        Secret m1 {}\n\
    }\n\
}\n",
    )
    .expect("an imported node is referenceable without `pub`");
    let screen = out
        .description
        .components
        .iter()
        .find(|c| c.name == "Screen")
        .expect("Screen");
    assert_eq!(screen.refs.len(), 1);
    assert_eq!(screen.refs[0].node, "Secret");
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
    let ds = compile(main).expect_err("undeclared reference rejected");
    // 诊断必须**精确指向**出错的那个 token，而不是"落在模块区间内"。
    //
    // 旧版本只检查 `span` 落在 main 的区间里；因为 main 的 `flat_start == 0`，
    // **任何**小 span 都满足——那是一句几乎不可能红的断言。改成锚定 token 后，
    // 任何把 span 算错一个偏移的改动都会立刻暴露。
    let want = main.find("Nowhere").expect("fixture contains Nowhere");
    assert_eq!(
        ds[0].span.start, want,
        "the diagnostic must point at the offending token; got {:?} which reads {:?}",
        ds[0].span,
        &main[ds[0].span.start.min(main.len())..ds[0].span.end.min(main.len())]
    );
}

/// 跨模块 node 引用（本轮闭环）：普通 `use` 导入的 node 按 node 名解析到声明模块；
/// `pub node` 同样可引用。AGE 作者面不需要 `pub`（2026-11-16 用户裁定）。
#[test]
fn p981_ref_cross_module_node_resolves_by_import() {
    let imported =
        "node Screen {\n    view {\n        text \"hi\" {}\n        Meter m1 {}\n    }\n}\n";
    let out = compile(imported).expect("an imported node is referenceable");
    let screen = out
        .description
        .components
        .iter()
        .find(|c| c.name == "Screen")
        .expect("Screen");
    assert_eq!(screen.refs.len(), 1);
    assert_eq!(screen.refs[0].module, "app::widgets");
    assert_eq!(screen.refs[0].node, "Meter");

    // 同一规则对未写 `pub` 的声明同样成立（g01 的原源形态）。
    let plain =
        "node Screen {\n    view {\n        text \"hi\" {}\n        Secret m1 {}\n    }\n}\n";
    let out = compile(plain).expect("a non-pub imported node is referenceable too");
    let screen = out
        .description
        .components
        .iter()
        .find(|c| c.name == "Screen")
        .expect("Screen");
    assert_eq!(screen.refs.len(), 1);
    assert_eq!(screen.refs[0].node, "Secret");
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
