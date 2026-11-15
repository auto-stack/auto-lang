//! PLAN-981（SOURCE 批，IF-981-REF）：**实例引用的可见性**回归。
//!
//! 979 V-10 要求：同类型的**合法引用**、**未声明引用**、**已声明但不可见引用**
//! 必须被隔离成三种不同的具名原因（"是引用/可见性诊断而非偶然类型错"），带原
//! 模块/字段 span；合法近邻两端真跑。本文件就是这三条 + 合法近邻。
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::compile_age_package;
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};

/// 本批的引用语义只覆盖**同模块**引用与三类失败；跨模块 `pub node` 可见性
/// 是具名缺口（见 Plan §9），故夹具不使用它。
const WIDGETS: &str = "node Meter {\n\
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


/// NAMED GAP (IF-981-REF, cross-module half): a top-level `pub node` is not yet
/// a declaration boundary in the CST engine, so it cannot reach projection.
/// This test exists to make sure that gap stays LOUD: such a source must either
/// be refused, or project the node — never compile into a silently smaller
/// program that merely omits it.
#[test]
fn p981_ref_pub_node_is_never_silently_dropped() {
    let src = "pub node Meter {\n    view { text \"m\" {} }\n}\n";
    let out = compile_age_package(
        &[input("main", src)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &caps(),
    );
    match out {
        Err(_) => {}
        Ok(c) => assert!(
            c.description.components.iter().any(|x| x.name == "Meter"),
            "`pub node Meter` must not be silently dropped from a compiling module"
        ),
    }
}
