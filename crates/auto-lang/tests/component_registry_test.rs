//! Plan 435 P4 —— 统一组件注册表(ComponentRegistry)集成测试。
//!
//! 覆盖四个验收面:
//! 1. 解析优先级:Builtin > Local > Package(显式化,Plan 408 推广);
//! 2. 内置不可被 shadow:同名本地组件注册被拒并记录 violation;
//! 3. 官方包自举:gallery components 目录经统一机制注册(无特殊通道);
//! 4. 端到端:`use { package: ... }` 引用 + 包组件 tag → SFC 生成可用。

use auto_lang::ui_gen::widget::{ComponentRegistry, ComponentResolution, ComponentSource};

/// PLAN-590(Stage B P-5):widgets-gallery 迁 auto-os 顶层,components 包
/// 目录经 `resolve_os_top_dir` 解析序定位(env AUTO_OS_ROOT → 兄弟 → 主
/// 检出);solo 检出 → None,依赖画廊包的测试整体 SKIP(不炸)。
fn gallery_components_dir() -> Option<std::path::PathBuf> {
    let sibling_base =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    auto_lang::os_paths::resolve_os_top_dir(&sibling_base, "widgets-gallery")
        .map(|g| g.join("src/front/components"))
}

/// 画廊包缺席的统一 SKIP 提示。
macro_rules! gallery_or_skip {
    () => {
        match gallery_components_dir() {
            Some(dir) => dir,
            None => {
                eprintln!(
                    "component_registry: SKIPPED — auto-os/widgets-gallery 未解析\
                     (solo 检出;设 AUTO_OS_ROOT 或并置 auto-os 兄弟检出可启用)"
                );
                return;
            }
        }
    };
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
    let components = gallery_or_skip!();
    let mut reg = ComponentRegistry::new();
    // Local:与内置无关的名字
    let rejected = reg.register_local(&[minimal_widget("MyLocalThing")]);
    assert!(rejected.is_empty(), "non-colliding local should register");
    // Package:官方包(gallery components)
    reg.load_package(&components, std::path::Path::new("."))
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
    let components = gallery_or_skip!();
    let mut reg = ComponentRegistry::new();
    let pkg = reg
        .load_package(&components, std::path::Path::new("."))
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
    // PLAN-590:画廊迁 auto-os 顶层——fixtures/pkg_app.at 的相对 `from`
    // 路径无法承载解析序,改为测试内物化临时 fixture 并注入解析出的
    // components 绝对路径(solo 检出 SKIP)。
    let components = gallery_or_skip!();
    let from_path = components.display().to_string().replace('\\', "/");
    let src = format!(
        "// Plan 435 P4 e2e —— use package 引用官方 .at 组件包（PLAN-590：路径注入）\n\
         widget PkgApp {{\n\
         \x20   use {{ package: official from \"{from_path}\" }}\n\
         \x20   msg {{ Go }}\n\
         \x20   model {{ n int = 0 }}\n\
         \x20   on {{ .Go -> {{ }} }}\n\
         \x20   view {{\n\
         \x20       col {{\n\
         \x20           copy-button {{}}\n\
         \x20           button \"native still works\" {{}}\n\
         \x20       }}\n\
         \x20   }}\n\
         }}\n"
    );
    let fixture =
        std::env::temp_dir().join(format!("pkg_app_p590_{}.at", std::process::id()));
    std::fs::write(&fixture, src).expect("write temp pkg_app fixture");
    let opts = auto_lang::ui_gen::ComponentGenOptions::default();
    let result = match auto_lang::ui_gen::generate_component_from_file(&fixture, opts) {
        Ok(r) => r,
        Err(e) => {
            let _ = std::fs::remove_file(&fixture);
            panic!("package app generates: {e}");
        }
    };
    let _ = std::fs::remove_file(&fixture);
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

/// Plan 435 P8-2(D6):LoadedPackage 家族建模 —— schema sub_widgets 折叠匹配
/// + 包内严格前缀兜底。gallery components 包:Carousel 全家、Combobox 全家。
#[test]
fn package_families_modeled() {
    let components = gallery_or_skip!();
    let mut reg = ComponentRegistry::new();
    let pkg = reg
        .load_package(&components, std::path::Path::new("."))
        .expect("gallery components load");
    let carousel = pkg.families.get("Carousel").expect("Carousel 家族");
    for child in ["CarouselContent", "CarouselItem", "CarouselNext", "CarouselPrevious"] {
        assert!(
            carousel.contains(&child.to_string()),
            "Carousel 子件缺 {child}: {carousel:?}"
        );
    }
    let combobox = pkg.families.get("Combobox").expect("Combobox 家族");
    assert!(
        combobox.iter().any(|c| c == "ComboboxItem"),
        "Combobox 子件: {combobox:?}"
    );
    // 访问器:未知 widget 返回空
    assert!(reg.family_children_of("NoSuchWidget").is_empty());
    // 第 5 个子件是 CarouselDemo(同文件 demo widget,前缀推导合理收入)
    assert_eq!(reg.family_children_of("Carousel").len(), 5);
    assert!(reg.family_children_of("Carousel").contains(&"CarouselDemo".to_string()));
}

/// Plan 435 P8-6(D13):桌面端接入 —— ①WidgetRegistry 折叠桥接(kebab tag
/// 命中 Pascal widget,与 vue map_tag 同语义);②包组件可经
/// load_package 的 full_widgets 注册进 WidgetRegistry(视图 + decl)。
#[test]
fn desktop_registry_bridges_package_components() {
    #[cfg(feature = "ui-interpreter")]
    use auto_lang::ui::widget_registry::WidgetRegistry;

    // ① 折叠桥接(ui feature 门控:WidgetRegistry 在 iced 后端)
    #[cfg(feature = "ui-interpreter")]
    {
        let mut reg_ui = WidgetRegistry::new();
    reg_ui.register(minimal_widget("CopyButton"));
    assert!(reg_ui.get("CopyButton").is_some(), "精确命中");
    assert!(reg_ui.get("copy-button").is_some(), "kebab 折叠命中");
    assert!(reg_ui.get("Copy-Button").is_some(), "混合折叠命中");
    }

    // ② 包组件全量对:视图 + decl 齐备
    let components = gallery_or_skip!();
    let mut reg = ComponentRegistry::new();
    let pkg = reg
        .load_package(&components, std::path::Path::new("."))
        .expect("gallery components load");
    assert!(
        pkg.full_widgets.iter().any(|(_, w)| w.name == "Carousel"),
        "full_widgets 应含 Carousel: {:?}",
        pkg.full_widgets.iter().map(|(_, w)| w.name.clone()).collect::<Vec<_>>()
    );
    assert!(
        pkg.full_widgets
            .iter()
            .all(|(d, w)| d.name.as_str() == w.name),
        "decl 与 widget 名一致"
    );
}
