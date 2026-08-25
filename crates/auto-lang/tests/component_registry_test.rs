//! Plan 435 P4 —— 统一组件注册表(ComponentRegistry)集成测试。
//!
//! 覆盖四个验收面:
//! 1. 解析优先级:Builtin > Local > Package(显式化,Plan 408 推广);
//! 2. 内置不可被 shadow:同名本地组件注册被拒并记录 violation;
//! 3. 官方包自举:gallery components 目录经统一机制注册(无特殊通道);
//! 4. 端到端:`use { package: ... }` 引用 + 包组件 tag → SFC 生成可用。

use auto_lang::ui_gen::widget::{ComponentRegistry, ComponentResolution, ComponentSource};

fn gallery_components_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/widgets-gallery/src/front/components")
}

/// 构造一个最小 AuraWidget(名字即可;其余字段默认)。
fn minimal_widget(name: &str) -> auto_lang::aura::AuraWidget {
    let code = format!(
        "widget {} {{\n    msg M {{ Go }}\n    model {{ n int = 0 }}\n    on {{ .Go -> {{ }} }}\n    view {{ col {{ text \"x\" {{}} }} }}\n}}\n",
        name
    );
    let session = auto_lang::session::CompilerSession::new(
        auto_lang::session::Scenario::UI,
    );
    let mut parser = auto_lang::Parser::from(code.as_str());
    parser = parser.with_session(session);
    let ast = parser.parse().expect("parse minimal widget");
    for stmt in &ast.stmts {
        if let auto_lang::ast::Stmt::WidgetDecl(d) = stmt {
            return auto_lang::aura::extract_widget_from_decl(d).expect("extract");
        }
    }
    panic!("widget decl not found");
}

#[test]
fn resolution_priority_builtin_local_package() {
    let mut reg = ComponentRegistry::new();
    // Local:与内置无关的名字
    let rejected = reg.register_local(&[minimal_widget("MyLocalThing")]);
    assert!(rejected.is_empty(), "non-colliding local should register");
    // Package:官方包(gallery components)
    reg.load_package(&gallery_components_dir(), std::path::Path::new("."))
        .expect("official package loads");

    // 1) 内置优先:button 是内置 tag —— 即使本地注册了 Button 也不 shadow
    //    (下一测试显式验证拒绝;此处验证 resolve 结果)
    assert!(matches!(
        reg.resolve("button"),
        ComponentResolution::Builtin { .. }
    ));
    // 折叠别名同样命中内置
    assert!(matches!(
        reg.resolve("alert-dialog-action"),
        ComponentResolution::Builtin { .. }
    ));
    // 2) Local
    match reg.resolve("my-local-thing") {
        ComponentResolution::Component { source, .. } => {
            assert_eq!(source, ComponentSource::Local);
        }
        other => panic!("expected Local, got {:?}", other),
    }
    // 3) Package(官方包组件;carousel-content 与内置 carousel_content 折叠
    //    冲突,builtin 优先 —— 用无冲突的 copy-button 验证包解析)
    match reg.resolve("copy-button") {
        ComponentResolution::Component { name, source } => {
            assert_eq!(source, ComponentSource::Package);
            assert_eq!(name, "CopyButton");
        }
        other => panic!("expected Package CopyButton, got {:?}", other),
    }
    // 冲突者归内置(Plan 408/435:builtin wins;P7-1 carousel 家族已退役
    // 交还官方 .at 组件 —— 换 dialog-content 验证同一语义)
    assert!(matches!(
        reg.resolve("dialog-content"),
        ComponentResolution::Builtin { .. }
    ));
    // 4) Unknown
    assert!(matches!(
        reg.resolve("definitely-not-a-thing"),
        ComponentResolution::Unknown
    ));
}

#[test]
fn builtin_tags_cannot_be_shadowed() {
    let mut reg = ComponentRegistry::new();
    let rejected = reg.register_local(&[
        minimal_widget("Button"),     // 与内置 button 折叠冲突
        minimal_widget("AlertDialog"), // 与内置 alert-dialog 折叠冲突
        minimal_widget("FineWidget"), // 无冲突
    ]);
    assert_eq!(rejected.len(), 2, "Button/AlertDialog 应被拒绝: {rejected:?}");
    assert_eq!(reg.shadow_violations().len(), 2);
    // 被拒后 resolve 仍命中内置
    assert!(matches!(
        reg.resolve("button"),
        ComponentResolution::Builtin { .. }
    ));
    // 无冲突的正常注册
    match reg.resolve("fine-widget") {
        ComponentResolution::Component { source, .. } => {
            assert_eq!(source, ComponentSource::Local);
        }
        other => panic!("expected Local FineWidget, got {:?}", other),
    }
}

#[test]
fn official_package_bootstrap_via_unified_mechanism() {
    // 自举验收:官方包(gallery components)通过与第三方完全相同的
    // load_package 机制注册 —— 无任何官方特例。
    let mut reg = ComponentRegistry::new();
    let pkg = reg
        .load_package(&gallery_components_dir(), std::path::Path::new("."))
        .expect("official package");
    assert_eq!(pkg.manifest.name, "official");
    assert_eq!(pkg.manifest.version, "0.1.0");
    assert!(
        pkg.widgets.contains_key("carouselcontent"),
        "carousel 家族应注册: {:?}",
        pkg.widgets.keys().take(8).collect::<Vec<_>>()
    );
    assert!(!pkg.widgets.contains_key("button"), "内置冲突者不入包注册表");
}

#[test]
fn e2e_use_package_generates_component() {
    // 端到端:use { package: ... } + 包组件 tag → SFC 引用生成。
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pkg_app.at");
    let opts = auto_lang::ui_gen::ComponentGenOptions::default();
    let result = auto_lang::ui_gen::generate_component_from_file(&fixture, opts)
        .expect("package app generates");
    let sfc = result
        .all_widget_codes
        .iter()
        .find(|(n, _)| n == "PkgApp")
        .map(|(_, c)| c.clone())
        .expect("PkgApp SFC");
    assert!(
        sfc.contains("CopyButton"),
        "SFC 应引用包组件 CarouselContent:\n{}",
        sfc.lines().take(20).collect::<Vec<_>>().join("\n")
    );
    // 包加载无告警(S003 不应出现)
    assert!(
        !result.validation_warnings.iter().any(|w| w.rule == "S003"),
        "package load warning: {:?}",
        result
            .validation_warnings
            .iter()
            .filter(|w| w.rule == "S003")
            .collect::<Vec<_>>()
    );
}

/// Plan 435 P7-3(D7):load_package 逐文件容错 —— 单个坏文件只记
/// parse_warning,不废整个包;合法组件照常注册。全坏才报错(带文件清单)。
#[test]
fn load_package_survives_single_bad_file() {
    let tmp = std::env::temp_dir().join(format!("p7pkg_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    std::fs::write(
        tmp.join("good.at"),
        "widget GoodThing {\n    view { col { text \"x\" {} } }\n}\n",
    )
    .unwrap();
    std::fs::write(tmp.join("bad.at"), "this is :: not valid autolang {{{\n").unwrap();

    let mut reg = ComponentRegistry::new();
    let pkg = reg
        .load_package(&tmp, std::path::Path::new("."))
        .expect("单文件失败不应废包");
    assert!(
        pkg.widgets.values().any(|n| n == "GoodThing"),
        "合法组件应注册: {:?}",
        pkg.widgets
    );
    assert_eq!(
        pkg.parse_warnings.len(),
        1,
        "坏文件应恰好记录一条 warning: {:?}",
        pkg.parse_warnings
    );
    assert!(
        pkg.parse_warnings[0].contains("bad.at"),
        "warning 应含失败文件路径: {}",
        pkg.parse_warnings[0]
    );

    // 全坏 → 报错,错误信息带文件清单
    let tmp2 = std::env::temp_dir().join(format!("p7pkg2_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp2);
    std::fs::create_dir_all(&tmp2).unwrap();
    std::fs::write(tmp2.join("bad1.at"), "}}} garbage\n").unwrap();
    let err = reg
        .load_package(&tmp2, std::path::Path::new("."))
        .expect_err("全坏应报错");
    assert!(err.contains("bad1.at"), "错误应列出失败文件: {err}");
    let _ = std::fs::remove_dir_all(&tmp);
    let _ = std::fs::remove_dir_all(&tmp2);
}
