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

/// P5（通道边界钉）：`.to(int)` 转换走嫁接的真 stdlib 函数
/// `__std_conv_to_int`（公共 Auto 体，body 含 `VariantCtor` 错误路径）；
/// compute 通道（`lower_program`，892/902 冻结面）**仍然具名拒**它——
/// 双通道边界不静默加宽。
#[test]
fn p5_conversion_graft_still_named_rejected_on_the_compute_channel() {
    let src = "fn main() int { let wide = 40i64\n return (wide + 2i64).to(int) }";
    let compiled = compile_ordinary(src, "main.at");
    let checked = Box::leak(Box::new(compiled.checked));
    let view = SemanticView::from_checked(checked)
        .expect("bind+view hold: the graft is ordinary HIR");
    let err = ace_a2x_rust::lower_program(&view, "main")
        .expect_err("the compute channel must keep rejecting VariantCtor");
    let text = format!("{err:?}");
    assert!(
        text.contains("VariantCtor") && text.contains("compute surface"),
        "channel boundary shape: {text}"
    );
}

/// P6（T-02 翻绿点）：同一源在**language 通道**（collections 族统一
/// 降落）整链绿——数值/转换/嫁接 stdlib/变体构造同链。
#[test]
fn p6_conversion_graft_lowers_on_the_language_channel() {
    let src = "fn main() int { let wide = 40i64\n return (wide + 2i64).to(int) }";
    let compiled = compile_ordinary(src, "main.at");
    let checked = Box::leak(Box::new(compiled.checked));
    let view = SemanticView::from_checked(checked).expect("view admits");
    let program = ace_a2x_rust::lower_language_program(&view, "main")
        .expect("the unified ordinary-face lowering must cover the graft");
    assert!(program.entry.ends_with("main"), "{}", program.entry);
}

/// P7（T-02 闭包/HOF 试点，1081 规范式）：捕获闭包经 HOF 参数传递并
/// 调用（`fn(int)int` 参数 + for-in 列表迭代 + 不可变捕获）。
/// 记录 language 通道当前对该面的实际降落结论。
#[test]
fn p7_hof_capturing_closure_language_channel() {
    let src = r#"fn each_count(f fn(int)int, xs List<int>) int {
    var n = 0
    for x in xs {
        let y = f(x)
        n = n + 1
    }
    return n
}
fn main() int {
    let factor = 3
    let nums = [1, 2, 3]
    return each_count((x int) => x * factor, nums)
}"#;
    let compiled = compile_ordinary(src, "main.at");
    let checked = Box::leak(Box::new(compiled.checked));
    let view = SemanticView::from_checked(checked).expect("view admits the HOF face");
    let program = ace_a2x_rust::lower_language_program(&view, "main")
        .expect("the language channel must lower the capturing-closure HOF face");
    assert!(program.entry.ends_with("main"), "{}", program.entry);
}

/// P8（T-02 插值试点，1079 规范式）：f-string（`${}`→TextDisplay+TextConcat
/// 链）+ char_at/to_codepoint 的 Unicode 精确窗口。
#[test]
fn p8_fstring_interpolation_language_channel() {
    let src = r#"fn main() int {
 let s = "A中😀\u{0}".clone()
 if s.char_at(2).to_codepoint() != 128512 { return 4 }
 let f = f"v=${20 + 22}"
 if f.len() != 4 { return 6 }
 return 42
}"#;
    let compiled = compile_ordinary(src, "main.at");
    let checked = Box::leak(Box::new(compiled.checked));
    let view = SemanticView::from_checked(checked).expect("view admits the fstr face");
    let program = ace_a2x_rust::lower_language_program(&view, "main")
        .expect("the language channel must lower the f-string chain");
    assert!(program.entry.ends_with("main"), "{}", program.entry);
}

/// P9（T-02 模式试点，1083 规范式原文）：`!?int`（Result<Option<int>>）
/// 嵌套 `is` 解构（`->` 臂），经 Variant guard/project 链。
#[test]
fn p9_nested_is_destructuring_language_channel() {
    let src = r#"fn obtain(ok bool) !?int {
    if ok {
        return Ok(Some(42))
    }
    return Err("bad")
}
fn main() int {
 is obtain(true) {
  Ok(v) -> {
   is v {
    Some(n) -> { return n }
    None -> { return 0 }
   }
  }
  Err(e) -> { return 0 }
 }
}"#;
    let compiled = compile_ordinary(src, "main.at");
    let checked = Box::leak(Box::new(compiled.checked));
    let view = SemanticView::from_checked(checked).expect("view admits the is-destructuring face");
    let program = ace_a2x_rust::lower_language_program(&view, "main")
        .expect("the language channel must lower nested is destructuring");
    assert!(program.entry.ends_with("main"), "{}", program.entry);
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
    // T-02 起（Unit 准入落地）：整链绿 —— 模块表示冻结为扁平化单模块
    // 程序+ModuleGlobalsPlan（981 已验模式，零新表示）。
    let view = SemanticView::from_checked(checked)
        .expect("scalar globals with the binder plan must admit (981 pattern)");
    let model = view
        .globals_model()
        .expect("globals model over the binder plan");
    assert_eq!(model.len(), 1, "one global in the view model");
    assert!(model[0].mutable, "module var n is mutable");
    assert!(!view.global_init_fns().is_empty(), "init chain present");
}

/// P-DIAG（临时）：分段定位 500 深嵌套的栈溢出阶段。
#[test]
fn p11_depth_budget_named_rejection() {
    let mut body = String::from("fn deep(x int) int { return ");
    body.push_str(&"(x + ".repeat(500));
    body.push('1');
    body.push_str(&")".repeat(500));
    body.push_str(" }");
    body.push_str("fn main() int { return deep(1) }");
    eprintln!("stage: bind");
    let compiled = ace_core::source::compile_scalar_source(&body, "diag", &[]).unwrap();
    eprintln!("stage: view");
    let checked = Box::leak(Box::new(compiled.checked));
    let view = SemanticView::from_checked(checked).unwrap();
    eprintln!("stage: lower");
    let r = ace_a2x_rust::lower_program(&view, "main");
    eprintln!("stage: done -> {r:?}");
}



/// P10（宽度守卫）：RExpr 的宽度冻结——深度守卫递归的 debug 帧由其槽位
/// 尺寸决定，枚举意外增长会在 400 层预算测试上翻成栈溢出（2026-11-22
/// 实测：+24B/槽即翻）。
#[test]
fn p10_rexpr_width_frozen() {
    assert_eq!(
        std::mem::size_of::<ace_a2x_rust::RExpr>(),
        72,
        "RExpr width is frozen for the depth-budgeted lowering recursion"
    );
}
