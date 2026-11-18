//! PLAN-981（SOURCE 批，IF-981-REF）：**实例引用的可见性**回归。
//!
//! 979 V-10 要求：同类型的**合法引用**、**未声明引用**、**已声明但不可见引用**
//! 必须被隔离成不同的具名原因（"是引用/可见性诊断而非偶然类型错"），带原
//! 模块/字段 span；合法近邻两端真跑。本文件就是这几条 + 合法近邻。
//!
//! 轮 48 复审（R47-1）发现本文件曾**声称**验证"导入即可引用"，但两个名为
//! imported 的 fixture 里**没有任何 `use`**——实现当时按全包 node 名直接匹配，
//! 于是"导入"这一条件从未被验证。现在实现必须**从声明模块导入**才可见，
//! 三个 fixture 都真的写了 `use`；另补 invisible / ambiguous / 导入择一 / 修复恢复。
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::compile_age_package;
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};

/// 被引用的 node 声明模块：`Meter`/`Secret` 都是 node（AGE 作者面不写 `pub`，
/// 导入即用）；`Plain` 是普通 type（不是 node）。
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

fn compile(
    main: &str,
) -> Result<
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

/// 依赖模块可变体的版本（`widgets` 键，便于构造扁平偏移布局）。
fn compile_two(
    dep: &str,
    main: &str,
) -> Result<
    ace_core::source::age_package::AgePackageCompilation,
    Vec<ace_core::atom_text::Diagnostic>,
> {
    compile_age_package(
        &[input("widgets", dep), input("main", main)],
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

/// ② 已声明**且被导入** → 可引用。AGE 作者面**不写 `pub`**（2026-11-16 用户裁定；
///    规范原文「接受普通 Auto 模块工程（…普通 `use`、具名资源与引用）」）；
///    符号表导入本身就是显式的跨模块意愿。
#[test]
fn p981_ref_imported_node_is_referenceable_without_pub() {
    let out = compile(
        "use widgets.Secret\n\
         node Screen {\n\
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
    assert_eq!(screen.refs[0].module, "app::widgets");
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
        ds[0].span.start,
        want,
        "the diagnostic must point at the offending token; got {:?} which reads {:?}",
        ds[0].span,
        &main[ds[0].span.start.min(main.len())..ds[0].span.end.min(main.len())]
    );
}

//// 诊断带**原模块** span，且**在能暴露偏移的布局下**精确指向出错 token。
///
/// 判据必须这样构造，否则缺陷隐身（我因此一度误判"减法必需"）：
/// - 故障必须在**非入口**模块（入口的 `flat_start` 为 0，减 0 等于没减）；
/// - 故障的**本地偏移**必须**大于**其它模块的 `flat_start`，否则减法会
///   `saturating_sub` 到 0 附近而"碰巧正确"。
///
/// 布局：`main` 很短且是入口；`zlib` 里先放一个正常 node，再放 200 字节填充，
/// 最后才是带 `Nowhere` 的 node —— 使 `Nowhere` 的本地偏移 (~316) 远大于
/// `main` 的长度。任何"再减一次 flat_start"的实现都会落到填充注释上。
#[test]
fn p981_ref_span_is_correct_for_a_deep_fault_in_a_non_entry_module() {
    let main = "use widgets.Screen\nnode Main {\n    view {\n        text \"t\" {}\n    }\n}\n";
    let filler = format!("// {}\n", "q".repeat(200));
    let zlib = format!(
        "pub node Screen {{\n    view {{\n        text \"s\" {{}}\n    }}\n}}\n{filler}\
         pub node Late {{\n    view {{\n        text \"l\" {{}}\n        Nowhere m1 {{}}\n    }}\n}}\n"
    );
    let ds = compile_two(&zlib, main).expect_err("Nowhere is undeclared");
    let want = zlib.find("Nowhere").expect("fixture has Nowhere");
    assert_eq!(
        ds[0].span.start,
        want,
        "the diagnostic must point at `Nowhere` inside zlib (local {want}); \
         got {:?} which reads {:?}",
        ds[0].span,
        &zlib[ds[0].span.start.min(zlib.len())..ds[0].span.end.min(zlib.len())]
    );
    // 同时确认这个布局确实"能暴露偏移"：want 必须大于 main 的长度。
    assert!(
        want > main.len(),
        "the fixture must place the fault beyond main's length to be a real probe"
    );
}

// 跨模块 node 引用（本轮闭环）的既有用例在下方。
/// `pub node` 同样可引用（`pub` 不是必要条件，但也不妨碍）；导入是必要条件。
#[test]
fn p981_ref_cross_module_node_resolves_by_import() {
    let imported = "use widgets.Meter\n\
         node Screen {\n    view {\n        text \"hi\" {}\n        Meter m1 {}\n    }\n}\n";
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
    let plain = "use widgets.Secret\n\
         node Screen {\n    view {\n        text \"hi\" {}\n        Secret m1 {}\n    }\n}\n";
    let out = compile(plain).expect("a non-pub imported node is referenceable too");
    let screen = out
        .description
        .components
        .iter()
        .find(|c| c.name == "Screen")
        .expect("Screen");
    assert_eq!(screen.refs.len(), 1);
    assert_eq!(screen.refs[0].module, "app::widgets");
    assert_eq!(screen.refs[0].node, "Secret");
}

/// 已声明但**没有导入** → `source.age.ref-invisible`（不是静默接受，也不是 undeclared）。
#[test]
fn p981_ref_unimported_node_is_invisible() {
    let main =
        "node Screen {\n    view {\n        text \"hi\" {}\n        Secret m1 {}\n    }\n}\n";
    let ds = compile(main).expect_err("an unimported node must not be referenceable");
    assert_eq!(ds[0].code, "source.age.ref-invisible", "{ds:?}");
    assert!(
        ds[0].message.contains("app::widgets"),
        "the diagnostic names the declaring module: {}",
        ds[0].message
    );
    assert!(
        ds[0].message.contains("use widgets: Secret"),
        "and the exact fix: {}",
        ds[0].message
    );
}

/// 同名 node 出现在**多个**模块、而本模块一个都没导入 → `source.age.ref-ambiguous`。
///
/// 旧实现会按模块键**任选一个**并静默接受；这正是 R47-1 的第二个实证。
#[test]
fn p981_ref_same_named_nodes_are_ambiguous() {
    let aa = "node Widget {\n    view { text \"a\" {} }\n}\n";
    let bb = "node Widget {\n    view { text \"b\" {} }\n}\n";
    let main =
        "node Screen {\n    view {\n        text \"hi\" {}\n        Widget w1 {}\n    }\n}\n";
    let ds = compile_age_package(
        &[input("aa", aa), input("bb", bb), input("main", main)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &caps(),
    )
    .expect_err("two candidate modules and no import must be ambiguous");
    assert_eq!(ds[0].code, "source.age.ref-ambiguous", "{ds:?}");
    assert!(ds[0].message.contains("app::aa"), "{}", ds[0].message);
    assert!(ds[0].message.contains("app::bb"), "{}", ds[0].message);
}

/// 导入哪一个，就解析到哪一个（歧义由显式导入裁定）。
#[test]
fn p981_ref_import_selects_between_same_named_nodes() {
    let aa = "node Widget {\n    view { text \"a\" {} }\n}\n";
    let bb = "node Widget {\n    view { text \"b\" {} }\n}\n";
    let main = "use bb: Widget\nnode Screen {\n    view {\n        text \"hi\" {}\n        Widget w1 {}\n    }\n}\n";
    let out = compile_age_package(
        &[input("aa", aa), input("bb", bb), input("main", main)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &caps(),
    )
    .expect("an explicit import disambiguates");
    let screen = out
        .description
        .components
        .iter()
        .find(|c| c.name == "Screen")
        .expect("Screen");
    assert_eq!(screen.refs.len(), 1);
    assert_eq!(
        screen.refs[0].module, "app::bb",
        "the imported module wins, not the first module key"
    );
}

/// 失败后**同入口**修好导入即可恢复（REF 面自己的恢复例，R47-5）。
#[test]
fn p981_ref_recovers_after_the_import_is_added() {
    let broken =
        "node Screen {\n    view {\n        text \"hi\" {}\n        Secret m1 {}\n    }\n}\n";
    let ds = compile(broken).expect_err("broken reference");
    assert_eq!(ds[0].code, "source.age.ref-invisible");
    let fixed = "use widgets.Secret\n\
         node Screen {\n    view {\n        text \"hi\" {}\n        Secret m1 {}\n    }\n}\n";
    let out = compile(fixed).expect("same entry compiles after the fix");
    assert_eq!(
        out.description.components.len(),
        3,
        "widgets' Meter+Secret and Screen"
    );
    let screen = out
        .description
        .components
        .iter()
        .find(|c| c.name == "Screen")
        .expect("Screen");
    assert_eq!(screen.refs[0].node, "Secret");
}

/// `pub node` 自身必须留下可见性事实（声明侧，不只是引用侧）。
#[test]
fn p981_ref_pub_node_records_its_visibility() {
    let out =
        compile("node Screen {\n    view { text \"x\" {} }\n}\n").expect("plain fixture compiles");
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
