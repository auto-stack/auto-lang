//! PLAN-981（SOURCE 批，IF-981-MODULE）：**AGE 多模块包前门**的真实输入回归。
//!
//! 输入就是 §5 拟最小两文件工程（`helper.at` 公共 helper + `main.at` AGE 组件
//! 跨模块引用），不是为本测试另造的等价形状：领域行与普通包项必须来自**同一
//! 次**解析、**同一**绑定图、**唯一** verify。
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::{compile_age_package, AGE_PACKAGE_DOMAIN};
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};

const HELPER: &str = "pub fn seed() int { return 0 }\n\
pub fn label(n int) str {\n    if n == 6 {\n        return \"six\"\n    }\n    return \"other\"\n}\n";

const MAIN: &str = "use helper.seed\n\
use helper.label\n\
node Counter {\n\
    model { count int = seed() }\n\
    on Inc { count = count + 1 }\n\
    on Life.Tick { count = count + 1 }\n\
    view { text label(count) {} }\n\
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

fn compile(helper: &str, main: &str) -> Result<
    ace_core::source::age_package::AgePackageCompilation,
    Vec<ace_core::atom_text::Diagnostic>,
> {
    compile_age_package(
        &[input("helper", helper), input("main", main)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &caps(),
    )
}

/// 两文件 AGE 工程：跨模块 helper 参与领域行为，领域行与普通项同源。
#[test]
fn p981_age_package_two_file_project_compiles_through_one_verifier() {
    let out = compile(HELPER, MAIN).expect("two-file AGE project compiles");
    assert_eq!(out.description.domain, AGE_PACKAGE_DOMAIN);
    assert_eq!(out.description.entry, "app::main");

    // 领域行只有一个组件，来自 main.at（不是 helper.at）。
    assert_eq!(out.description.components.len(), 1);
    let c = &out.description.components[0];
    assert_eq!(c.name, "Counter");
    assert_eq!(c.module, "app::main");
    assert!(!c.legacy_widget, "canonical `node` head");
    assert_eq!(c.fields, vec![("count".to_string(), "int".to_string())]);
    // 限定生命周期消息与普通消息都在，且保持作者拼写。
    assert_eq!(
        c.handler_msgs,
        vec!["Inc".to_string(), "Life.Tick".to_string()]
    );
    // 文本叶的值函数由唯一 verify 判定（种类来自共享 resolver，不是本层猜）。
    assert_eq!(c.texts.len(), 1);
    assert_eq!(c.texts[0].role, "view");
    assert_eq!(c.texts[0].module, "app::main");
    assert!(c.texts[0].fn_name.starts_with("__age_p_hidden_"));
    assert!(c.texts[0].def_id.starts_with('d'));

    // 模块图身份可读：两个模块都在，且 key 是共享绑定器的键格式。
    let keys: Vec<&str> = out.description.modules.iter().map(|m| m.key.as_str()).collect();
    assert_eq!(keys, vec!["app::helper", "app::main"]);
}

/// 跨模块 helper 的**行为**真的进了受检程序：改 helper 的初值/分支即换结果。
#[test]
fn p981_age_package_cross_module_helper_changes_the_checked_program() {
    let base = compile(HELPER, MAIN).expect("base compiles");
    // 改 helper 的返回常量 → 领域叶所调用的函数体变化 → 受检文本不同。
    let changed_helper = HELPER.replace("return 0 }", "return 10 }");
    let changed = compile(&changed_helper, MAIN).expect("changed helper compiles");
    assert_ne!(
        base.atom_text, changed.atom_text,
        "editing the imported helper must change the checked program"
    );
    assert_ne!(
        base.input_hash, changed.input_hash,
        "module graph identity follows the imported source"
    );
    // 领域行本身没有变（改的是 helper，不是 main）。
    assert_eq!(
        base.description.components[0].handler_msgs,
        changed.description.components[0].handler_msgs
    );
}

/// 领域叶的表达式由共享检查拒绝（不是"能解析就算支持"）。
///
/// 拒绝发生在**共享**绑定/检查阶段：本前门不自己判类型，故诊断码是共享链的
/// 真实结论（i32 上无 `.clone()`）。判据是"坏叶被拒 + 诊断能在原模块里定到
/// 坏表达式"，不是由本层造一个领域码。
#[test]
fn p981_age_package_non_text_leaf_is_rejected_by_the_shared_checker() {
    let bad = MAIN.replace("text label(count) {}", "text count + 1 {}");
    let ds = compile(HELPER, &bad).expect_err("an int leaf is not a text value");
    assert!(
        ds.iter().any(|d| d.message.contains("clone")),
        "the shared checker rejects the non-text leaf: {ds:?}"
    );
    // 诊断按前门自己的模块 source map 归位到 main.at 的源区间内（而不是落在
    // helper 段或整体越界）——这是跨模块定位的可验证性质。
    let base = compile(HELPER, MAIN).expect("base compiles");
    let main_mod = base
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
        "the rejection maps into main.at's source range (flat_start {}, len {}): {ds:?}",
        main_mod.flat_start,
        main_mod.source_len
    );
}

/// 缺模块 / 私有 helper / 图环都在**绑定阶段**具名拒绝（不进入运行）。
#[test]
fn p981_age_package_module_graph_failures_are_named() {
    // 1) 缺模块：main 引用不存在的 helper。
    let ds = compile_age_package(
        &[input("main", MAIN)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &caps(),
    )
    .expect_err("missing module must be named");
    assert!(
        ds.iter()
            .any(|d| d.code == "source.module-missing" || d.code == "source.module-symbol"),
        "{ds:?}"
    );

    // 2) 私有 helper：去掉 `pub` 后导入必须被可见性拒绝。
    let private = HELPER.replace("pub fn seed()", "fn seed()");
    let ds = compile(&private, MAIN).expect_err("private helper import must be named");
    assert!(
        ds.iter().any(|d| d.code == "source.module-private"),
        "{ds:?}"
    );
}

/// 失败后同入口修复即恢复（无残留状态，不产半成品）。
#[test]
fn p981_age_package_recovers_on_the_same_entry() {
    let bad = MAIN.replace("node Counter", "node");
    assert!(compile(HELPER, &bad).is_err(), "malformed head is rejected");
    let out = compile(HELPER, MAIN).expect("the same entry recovers");
    assert_eq!(out.description.components.len(), 1);
}

/// 兼容闸：旧 `widget` 头只在显式开启时准入（与 979 同一口径）。
#[test]
fn p981_age_package_legacy_widget_alias_is_gated() {
    let legacy = MAIN.replace("node Counter", "widget Counter");
    let ds = compile(HELPER, &legacy).expect_err("legacy head needs the compat config");
    assert_eq!(ds[0].code, "source.age.legacy-widget", "{ds:?}");

    let on = compile_age_package(
        &[input("helper", HELPER), input("main", &legacy)],
        "app::main",
        LegacyWidgetAlias::Accept,
        &caps(),
    )
    .expect("compat on: the same Schema admits the legacy head");
    assert!(on.description.components[0].legacy_widget);
}

/// 通用入口不得静默丢弃领域声明（对称于 979 的通用入口拒绝）。
///
/// 复审 M5：旧版本只调用 `age_package_refusal` 这个**辅助函数**，从不经过真实
/// 入口——而那个辅助函数零生产调用、是死代码，通用入口对 `age-package` 实际
/// 返回 `Ok`（零项目、零领域行）。测试绿，缺陷在。现在直接跑真实入口。
#[test]
fn p981_age_package_profile_is_refused_by_generic_entries() {
    use ace_core::source::cst::{parse_front, FrontBudget, FrontKind, FrontRequest, FrontOutcome};
    let out = parse_front(FrontRequest {
        src: MAIN,
        profile: ace_core::source::SourceProfile::AgePackage,
        kind: FrontKind::WholeModule,
        budget: FrontBudget::WORK_OWNED_64MIB,
    });
    match out {
        FrontOutcome::Diag(ds) => {
            assert_eq!(ds[0].code, "source.domain-frontend", "{ds:?}");
            assert!(
                ds[0].message.contains("age_package"),
                "the refusal must point at the real front door: {}",
                ds[0].message
            );
        }
        _ => panic!("the generic front door must refuse the age-package profile"),
    }
    // 片段入口同样拒绝——必须**直接调用**该入口，且断言**诊断码**。
    //
    // 复审 M5 第二半指出两件事，都已实测确认：
    // ① `parse_front` 的整模块守卫会先拦住片段请求，所以从 `parse_front` 永远
    //    到不了片段守卫——用它来"验证片段拒绝"是**盲**的；
    // ② 旧断言 `matches!(frag, FrontOutcome::Diag(_))` 接受**任何**诊断，而探针源
    //    本身的语法错（`source.syntax`）就足以让它绿。
    //
    // 因此这里直接调 `parse_fragment_front`，并用**完全合法**的片段源。
    let frag_src = "fn f() int { return 1 }";
    let frag = ace_core::source::cst::parse_fragment_front(
        frag_src,
        ace_core::source::SourceProfile::AgePackage,
        FrontBudget::WORK_OWNED_64MIB,
        &(0..frag_src.len()),
        None,
        ace_core::source::ast::Receiver::Free,
    );
    match frag {
        FrontOutcome::Diag(ds) => assert_eq!(
            ds[0].code, "source.domain-frontend",
            "the fragment entry must refuse by the DOMAIN refusal, not by a syntax error: {ds:?}"
        ),
        _ => panic!("the fragment front door must refuse the age-package profile"),
    }
}
