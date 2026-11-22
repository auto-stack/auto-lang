//! PLAN-1115 T-01 — 语言矩阵与事实接面试点（有限正反例，编译探针）。
//!
//! 逐族钉住"普通 Auto 源（`auto.source.language.av/v1`）→ 共享前门 bind →
//! SemanticView 准入"今天停在哪一层：
//! - bind OK + view 具名拒绝 = 上游受检事实在场，缺口只在 a2x-core 投影
//!   （PLAN-1115 范围内可补）；
//! - bind 诊断 = 缺口在共享前门/resolver（记 owner 交接，不在本仓面补）。
//! 模块/全局试点沿 981 已验链（bind_packages → resolve → typecheck → emit
//! parts → verify → ModuleGlobalsPlan → view）走真实包闭包。
//! 全部为编译期探针：零 rustc spawn，秒级。

use ace_a2x_core::SemanticView;
use ace_core::source::{compile_language_source, CompiledSource};

const LANG_CAP: &str = "ac.source.language.av/v1";

fn compile_ordinary(src: &str, path: &str) -> CompiledSource {
    compile_language_source(src, path, &[LANG_CAP.to_string()])
        .unwrap_or_else(|ds| panic!("ordinary bind must hold the facts upstream: {ds:?}"))
}

fn view_rejects_named(src: &str, path: &str, needle: &str) -> String {
    let compiled = compile_ordinary(src, path);
    let checked = Box::leak(Box::new(compiled.checked));
    let err = SemanticView::from_checked(checked)
        .expect_err("the dynamic surface must be a NAMED view rejection today");
    let text = format!("{err:?}");
    assert!(
        text.to_lowercase().contains(needle),
        "rejection must name the {needle} family: {text}"
    );
    text
}

/// P1（1084 动态对象，反例）：受检事实在场（bind OK），view 以具名
/// `a2x.unsupported-type` 拒绝 Object 族 —— 缺口=投影层，不是上游。
#[test]
fn p1_dynamic_object_facts_bind_view_rejects_named() {
    let text = view_rejects_named(
        r#"fn main() int {
 var o = { inner: { x: 19, y: 20 } }
 o.inner.y = 23
 return o.inner.x + o.inner.y
}"#,
        "main.at",
        "object",
    );
    assert!(text.contains("a2x."), "a2x.* reject code: {text}");
}

/// P2（1084 显式 null，反例）：纯 null 源钉 Nil 族（对象字面量先撞
/// `object` 的变体见 P1）。
#[test]
fn p2_nil_and_coalesce_facts_bind_view_rejects_named() {
    view_rejects_named(
        r#"fn main() int {
    let a = null
    if a == null { return 42 }
    return 0
}"#,
        "main.at",
        "nil",
    );
}

/// P3（正例对照，T-02 基线）：已投影子面经**普通 Auto 入口**
/// （非 compile_scalar/aggregates_source）整链绿：bind → view → lower。
/// 冻结的规范拼法：record 字段 `name type`（`;`/换行分隔）、后缀数值
/// 字面量（`1i64`）、文本 `+`/`.len()`；转换拼法为 `.to(int)`。
#[test]
fn p3_supported_subset_through_ordinary_entry_lowers() {
    let sources = [
        (
            "record-canonical",
            "type Pair { left int; right int }\nfn main() int {\n    let p = Pair { right: 2, left: 40 }\n    return p.left + p.right\n}",
        ),
        (
            "suffix-lit",
            "fn main() int { let big = 9007199254740993i64\n if big - 9007199254740992i64 != 1i64 { return 1 }\n return 42 }",
        ),
        (
            "text-concat-len",
            "fn main() int { let label = \"a\" + \"b\"\n return label.len() }",
        ),
    ];
    for (name, src) in sources {
        let compiled = compile_ordinary(src, "main.at");
        let checked = Box::leak(Box::new(compiled.checked));
        let view = SemanticView::from_checked(checked)
            .unwrap_or_else(|e| panic!("{name}: subset must admit: {e:?}"));
        let program = ace_a2x_rust::lower_program(&view, "main")
            .unwrap_or_else(|e| panic!("{name}: subset must lower: {e:?}"));
        assert!(program.entry.ends_with("main"), "{name}: {}", program.entry);
    }
}

/// P5（T-02 反例 oracle，现状冻结）：`.to(int)` 转换走嫁接的真 stdlib
/// 函数 `__std_conv_to_int`（公共 Auto 体），其 body 含 `VariantCtor`
/// （错误路径 `Err` 构造）；`lower_program` 的 compute 面今天具名拒它。
/// T-02 统一普通面降落后本断言翻绿（换 view+lower OK）。
#[test]
fn p5_conversion_graft_currently_named_rejected_at_lowering() {
    let src = "fn main() int { let wide = 40i64\n return (wide + 2i64).to(int) }";
    let compiled = compile_ordinary(src, "main.at");
    let checked = Box::leak(Box::new(compiled.checked));
    let view = SemanticView::from_checked(checked)
        .expect("bind+view hold: the graft is ordinary HIR");
    let err = ace_a2x_rust::lower_program(&view, "main")
        .expect_err("the compute surface must reject VariantCtor today");
    let text = format!("{err:?}");
    assert!(
        text.contains("VariantCtor") && text.contains("compute surface"),
        "frozen gap shape: {text}"
    );
}

/// P4（模块/全局试点，正例）：1085 module-global-once 的两文件包闭包沿
/// 981 已验链走到底 —— 绑定器全局图（符号/可变性/init 序）在场，
/// CheckedModule 钉 ModuleGlobalsPlan 后 view 放行标量全局。
/// 模块表示由此冻结：**扁平化单模块程序 + ModuleGlobalsPlan 事实**，
/// 不发明第二表示。
#[test]
fn p4_module_globals_package_chain_admits_scalar_globals() {
    use ace_core::source::packages::{
        bind_packages_auto, PackageModuleInput, PACKAGES_CAPABILITY,
    };
    use ace_core::source::cst::FrontBudget;

    let main = "use state.bump\nuse state.read\nfn main() int { bump(); return read() }";
    let state = "var n = 40\npub fn bump() { n = n + 2 }\npub fn read() int { return n }";
    let inputs = [
        PackageModuleInput {
            package: "auto".into(),
            id: "main".into(),
            version: "0".into(),
            source: main.into(),
            provider: None,
        },
        PackageModuleInput {
            package: "auto".into(),
            id: "state".into(),
            version: "0".into(),
            source: state.into(),
            provider: None,
        },
    ];
    let caps = [LANG_CAP.to_string(), PACKAGES_CAPABILITY.to_string()];
    let bound = bind_packages_auto(&inputs, "auto::main", &caps, FrontBudget::WORK_OWNED_64MIB)
        .expect("bind the ordinary package closure");
    assert!(!bound.globals.is_empty(), "binder global facts present");
    assert_eq!(bound.init_order.len(), 1, "one global in topo order");
    assert_eq!(bound.module_init_fns.len(), 1, "one module init fn");

    // 981 链：resolve → typecheck → emit parts → verify → 钉全局 plan → view。
    let resolved = ace_core::source::adapter::resolve_auto_package(&bound.program, &caps)
        .expect("resolve the bound package");
    ace_core::source::typecheck_with(&resolved, ace_core::source::SourceProfile::Auto)
        .expect("typecheck the bound package");
    let parts = ace_core::source::emit_resolved_packages_parts_profile(
        &resolved,
        &bound,
        "packages",
        ace_core::source::SourceProfile::Auto,
    )
    .expect("emit package parts");
    let mut checked = ace_core::verify::verify_auto_language(parts.bundle)
        .expect("verify the ordinary package bundle");
    let plan = ace_core::verify::ModuleGlobalsPlan {
        init_order: bound.init_order.clone(),
        init_fns: bound
            .module_init_order
            .iter()
            .filter_map(|k| bound.module_init_fns.get(k).cloned())
            .collect(),
        facts: bound
            .globals
            .iter()
            .map(|(sym, decl)| ace_core::verify::GlobalFact {
                symbol: sym.clone(),
                name: decl.name.clone(),
                mutable: decl.mutable,
            })
            .collect(),
    };
    checked.set_module_globals(plan);
    let checked = Box::leak(Box::new(checked));
    // 现状冻结（T-03 翻绿点）：整链到 view 体内准入，仅剩 `Unit` 表达式
    // （无返回值 fn 的 return unit）在 aggregates 面被具名拒。绑定器全局
    // 事实已到位——模块表示冻结为扁平化单模块+ModuleGlobalsPlan（981 链）。
    let err = SemanticView::from_checked(checked)
        .expect_err("the Unit expr is the frozen T-03 admission gap");
    let text = format!("{err:?}");
    assert!(
        text.contains("Unit") && text.contains("aggregates"),
        "frozen gap shape: {text}"
    );
}
