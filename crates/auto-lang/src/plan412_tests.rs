//! Plan 412 — Layout Gallery + VM 布局引擎对齐: integration tests.
//!
//! Phase 3 验收的结构断言通道(Plan 411 双通道验收的延续):
//! 1. 12 个新 Layout 页面全部能解析 + 构建 VM view 树(不 panic);
//! 2. /grid 页的 style 写法 `col (style: "grid grid-cols-3 gap-4")` 重派生为
//!    真实 View::Grid(F1 验收,修复旧版纵向堆叠的双端不一致);
//! 3. /grid-span 页 cell 的 col-span-N 进入 View::Grid 的 cell style(F2);
//! 4. widgets-gallery 路由表含全部 12 条 Layout 路由。

#![cfg(test)]

use crate::ui::view::View;
use crate::ui::interpreter::DynamicMessage;
use crate::ui::style::StyleClass;

/// Collect every View::Grid in the tree (any depth), with its cols/gap.
fn collect_grids<'a>(
    view: &'a View<DynamicMessage>,
    out: &mut Vec<(usize, u16, &'a Vec<View<DynamicMessage>>)>,
) {
    match view {
        View::Grid { cols, gap, cells, .. } => {
            out.push((*cols, *gap, cells));
            for cell in cells {
                collect_grids(cell, out);
            }
        }
        View::Row { children, .. } | View::Column { children, .. } | View::List { items: children, .. } => {
            for child in children {
                collect_grids(child, out);
            }
        }
        View::Container { child, .. } | View::Scrollable { child, .. } => {
            collect_grids(child, out);
        }
        View::Table { headers, rows, .. } => {
            for h in headers {
                collect_grids(h, out);
            }
            for row in rows {
                for cell in row {
                    collect_grids(cell, out);
                }
            }
        }
        _ => {}
    }
}

/// Extract the style of a view node (the variants layout pages produce).
fn view_style(v: &View<DynamicMessage>) -> Option<&crate::ui::style::Style> {
    match v {
        View::Row { style, .. } | View::Column { style, .. } | View::Container { style, .. }
        | View::Grid { style, .. } | View::Text { style, .. } | View::Image { style, .. } => {
            style.as_ref()
        }
        _ => None,
    }
}

/// Build a gallery page widget in isolation (same harness as plan409_tests).
#[cfg(feature = "ui-iced")]
fn build_gallery_page(page_file: &str, widget_name: &str) -> Option<View<DynamicMessage>> {
    use crate::ui::aura_view_builder::AuraViewBuilder;
    use crate::ui::vm_bridge::VmBridge;
    use crate::ui::widget_registry::WidgetRegistry;

    let candidates = [
        std::env::var("CARGO_MANIFEST_DIR")
            .ok()
            .map(|d| std::path::PathBuf::from(d).join(format!("../../examples/widgets-gallery/src/front/pages/{}", page_file)))
            .filter(|p| p.exists()),
        Some(std::path::PathBuf::from(format!("examples/widgets-gallery/src/front/pages/{}", page_file)))
            .filter(|p| p.exists()),
    ];
    let path = candidates.into_iter().flatten().next()?;
    let code = std::fs::read_to_string(&path).ok()?;
    let session = crate::session::CompilerSession::ui();
    let mut parser = crate::Parser::from(code.as_str()).with_session(session);
    let ast = parser.parse().ok()?;
    for stmt in &ast.stmts {
        if let crate::ast::Stmt::WidgetDecl(decl) = stmt {
            let widget = crate::aura::extract_widget_from_decl(decl).ok()?;
            if widget.name != widget_name {
                continue;
            }
            let bridge = VmBridge::new(&widget).ok()?;
            let registry = WidgetRegistry::new();
            let builder = AuraViewBuilder::with_registry(&bridge, &widget.name, &registry);
            let view = builder.build(&widget.view_tree);
            return Some(view);
        }
    }
    None
}

#[cfg(feature = "ui-iced")]
#[test]
fn plan412_all_layout_pages_build() {
    // 12 页(含重写的 /grid)全部解析 + 构建成功,根节点为 Column(页面骨架)。
    let pages = [
        ("row.at", "RowPage"),
        ("col.at", "ColumnPage"),
        ("center.at", "CenterPage"),
        ("flex.at", "FlexPage"),
        ("alignment.at", "AlignmentPage"),
        ("spacing.at", "SpacingPage"),
        ("sizing.at", "SizingPage"),
        ("scroll.at", "ScrollPage"),
        ("position.at", "PositionPage"),
        ("responsive.at", "ResponsivePage"),
        ("grid.at", "GridPage"),
        ("grid-span.at", "GridSpanPage"),
    ];
    let mut built = 0;
    for (file, widget) in pages {
        match build_gallery_page(file, widget) {
            Some(View::Column { .. }) => built += 1,
            Some(other) => panic!("{} built to non-column root {:?}", file, std::mem::discriminant(&other)),
            None => eprintln!("plan412: SKIPPED — {} not found", file),
        }
    }
    assert_eq!(built, 12, "all 12 layout pages must build (built {})", built);
}

#[cfg(feature = "ui-iced")]
#[test]
fn plan412_grid_page_style_classes_rederive_to_grid() {
    // F1 验收:/grid 页 `col (style: "grid grid-cols-3 gap-4 w-full max-w-lg")`
    // 的 demo 在 VM 必须出现 View::Grid { cols: 3, gap: 16 } —— 修复旧版
    // "CSS grid 写法渲染成纵向堆叠" 的双端不一致。
    let Some(view) = build_gallery_page("grid.at", "GridPage") else {
        eprintln!("plan412: SKIPPED — grid.at not found");
        return;
    };
    let mut grids = Vec::new();
    collect_grids(&view, &mut grids);
    assert!(
        grids.iter().any(|(cols, gap, cells)| *cols == 3 && *gap == 16 && cells.len() >= 6),
        "expected a 3-col 16px-gap grid with 6+ cells, found {:?}",
        grids.iter().map(|(c, g, n)| (*c, *g, n.len())).collect::<Vec<_>>()
    );
    // 响应式 demo:md:grid-cols-2 lg:grid-cols-4 → 剥离后取最后 → cols=4。
    assert!(
        grids.iter().any(|(cols, _, _)| *cols == 4),
        "responsive demo should rederive to 4 columns, found {:?}",
        grids.iter().map(|(c, g, n)| (*c, *g, n.len())).collect::<Vec<_>>()
    );
}

#[cfg(feature = "ui-iced")]
#[test]
fn plan412_grid_span_page_carries_span_metadata() {
    // F2 验收:/grid-span 页 cell 的 col-span-N 落在 cell style 上,
    // 由 build_grid 的行分配器消费(等分轨道 + 槽位 padding 补偿)。
    let Some(view) = build_gallery_page("grid-span.at", "GridSpanPage") else {
        eprintln!("plan412: SKIPPED — grid-span.at not found");
        return;
    };
    let mut grids = Vec::new();
    collect_grids(&view, &mut grids);
    assert!(!grids.is_empty(), "grid-span page must contain View::Grid nodes");
    let mut spans_found = Vec::new();
    for (_, _, cells) in &grids {
        for cell in cells.iter() {
            if let Some(style) = view_style(cell) {
                for c in &style.classes {
                    if let StyleClass::ColSpan(n) = c {
                        spans_found.push(*n);
                    }
                }
            }
        }
    }
    assert!(
        spans_found.contains(&2) && spans_found.contains(&3),
        "expected col-span-2 and col-span-3 cells, found {:?}",
        spans_found
    );
}

#[cfg(feature = "ui-iced")]
#[test]
fn plan412_layout_pages_generate_vue_sfc() {
    // 双端验收的 vue 通道:12 页走真实 parse → extract → VueGenerator 管线,
    // SFC 生成成功且关键类透传(grid 页的 grid-cols-3 / span 页的 col-span-2)。
    use crate::ui_gen::{BackendGenerator, VueGenerator};

    let manifest = std::env::var("CARGO_MANIFEST_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_default();
    let base = manifest.join("../../examples/widgets-gallery/src/front/pages");
    let base = if base.exists() { base } else { std::path::PathBuf::from("examples/widgets-gallery/src/front/pages") };
    if !base.exists() {
        eprintln!("plan412: SKIPPED — pages dir not found");
        return;
    }
    let pages = [
        ("row.at", "RowPage", "flex"),
        ("col.at", "ColumnPage", "flex-col"),
        ("center.at", "CenterPage", "justify-center"),
        ("flex.at", "FlexPage", "flex-1"),
        ("alignment.at", "AlignmentPage", "justify-between"),
        ("spacing.at", "SpacingPage", "gap-4"),
        ("sizing.at", "SizingPage", "max-w-xs"),
        ("scroll.at", "ScrollPage", "overflow-y-auto"),
        ("position.at", "PositionPage", "z-30"),
        ("responsive.at", "ResponsivePage", "md:grid-cols-2"),
        ("grid.at", "GridPage", "grid-cols-3"),
        ("grid-span.at", "GridSpanPage", "col-span-2"),
    ];
    for (file, widget_name, needle) in pages {
        let code = std::fs::read_to_string(base.join(file))
            .unwrap_or_else(|e| panic!("read {}: {}", file, e));
        let session = crate::session::CompilerSession::ui();
        let mut parser = crate::Parser::from(code.as_str()).with_session(session);
        let ast = parser.parse().unwrap_or_else(|e| panic!("parse {}: {:?}", file, e));
        let decl = ast
            .stmts
            .iter()
            .find_map(|s| match s {
                crate::ast::Stmt::WidgetDecl(d) => Some(d),
                _ => None,
            })
            .expect("widget decl");
        let widget = crate::aura::extract_widget_from_decl(decl)
            .unwrap_or_else(|e| panic!("extract {}: {:?}", widget_name, e));
        let sfc = VueGenerator::new()
            .generate(&widget)
            .unwrap_or_else(|e| panic!("vue gen {}: {:?}", widget_name, e));
        assert!(
            sfc.contains(needle),
            "{}: generated SFC must contain `{}`",
            file,
            needle
        );
    }
}

/// Build the VM view tree from a widget source string (real parse pipeline).
#[cfg(feature = "ui-iced")]
fn build_view_from_src(src: &str) -> View<DynamicMessage> {
    use crate::ui::aura_view_builder::AuraViewBuilder;
    use crate::ui::vm_bridge::VmBridge;
    use crate::ui::widget_registry::WidgetRegistry;

    let session = crate::session::CompilerSession::ui();
    let mut parser = crate::Parser::from(src).with_session(session);
    let ast = parser.parse().expect("widget source must parse");
    let decl = ast
        .stmts
        .iter()
        .find_map(|s| match s {
            crate::ast::Stmt::WidgetDecl(d) => Some(d),
            _ => None,
        })
        .expect("widget decl");
    let widget = crate::aura::extract_widget_from_decl(decl).expect("extract widget");
    let bridge = VmBridge::new(&widget).expect("bridge");
    let registry = WidgetRegistry::new();
    let builder = AuraViewBuilder::with_registry(&bridge, &widget.name, &registry);
    builder.build(&widget.view_tree)
}

#[cfg(feature = "ui-iced")]
#[test]
fn plan412_square_builds_centered_container() {
    // §4.3 占位块:VM 端 = Container center_x/center_y + 色块类;
    // w: "full" 进等宽 grid 轨道,class prop 附加(col-span-2)。
    let src = r#"
widget Sq {
    view {
        row (style: "gap-2") {
            square (color: "purple", h: 10, w: "full", class: "col-span-2", text: "span-2") {}
        }
    }
}
"#;
    let view = build_view_from_src(src);
    let View::Row { children, .. } = view else {
        panic!("root must be Row");
    };
    let square = children.first().expect("square child");
    match square {
        View::Container { center_x, center_y, style, child, .. } => {
            assert!(*center_x, "square 双轴居中(center_x)");
            assert!(*center_y, "square 双轴居中(center_y)");
            let s = style.as_ref().expect("square carries style");
            let has = |pred: &dyn Fn(&StyleClass) -> bool| s.classes.iter().any(pred);
            assert!(has(&|c| matches!(c, StyleClass::BackgroundColor(_))), "bg color class present");
            assert!(has(&|c| matches!(c, StyleClass::Width(crate::ui::style::SizeValue::Full))), "w-full");
            assert!(has(&|c| matches!(c, StyleClass::Height(crate::ui::style::SizeValue::Fixed(10)))), "h-10");
            assert!(has(&|c| matches!(c, StyleClass::ColSpan(2))), "class prop 附加 col-span-2");
            match &**child {
                View::Text { content, .. } => assert_eq!(content, "span-2"),
                other => panic!("square text prop → Text child, got {:?}", std::mem::discriminant(other)),
            }
        }
        other => panic!("square → Container, got {:?}", std::mem::discriminant(other)),
    }
}

#[cfg(feature = "ui-iced")]
#[test]
fn plan412_square_generates_vue_div() {
    // vue 端:square → div + flex 居中类 + text prop 内容(继承 text-{c}-600)。
    use crate::ui_gen::{BackendGenerator, VueGenerator};
    let src = r#"
widget Sq {
    view {
        row (style: "gap-2") {
            square (color: "blue", size: 8, text: "1") {}
            square (color: "emerald", w: "full", h: 12, class: "col-span-2", text: "wide") {}
        }
    }
}
"#;
    let session = crate::session::CompilerSession::ui();
    let mut parser = crate::Parser::from(src).with_session(session);
    let ast = parser.parse().expect("parse");
    let decl = ast.stmts.iter().find_map(|s| match s {
        crate::ast::Stmt::WidgetDecl(d) => Some(d),
        _ => None,
    }).expect("decl");
    let widget = crate::aura::extract_widget_from_decl(decl).expect("extract");
    let sfc = VueGenerator::new().generate(&widget).expect("generate");
    for needle in [
        "h-8", "w-8", "bg-blue-500/40", "border-blue-500", "text-blue-600",
        "flex", "items-center", "justify-center", ">1</div>",
        "h-12", "w-full", "col-span-2", ">wide</div>",
    ] {
        if !sfc.contains(needle) {
            panic!("square SFC must contain `{}`. SFC:
{}", needle, sfc);
        }
    }
    // 尺寸/颜色 props 已转为类,不再透传为无效绑定属性。
    for banned in [":size", ":h=", ":w=", ":color"] {
        assert!(!sfc.contains(banned), "square SFC must not pass through `{}`", banned);
    }
}

#[cfg(feature = "ui-iced")]
#[test]
fn plan412_routes_registered() {
    // app.at 的 routes 块含全部 12 条 Layout 路由(结构断言)。直接解析
    // app.at(DynamicComponent::routes 为私有字段,不宜为测试开洞)。
    let candidates = [
        std::env::var("CARGO_MANIFEST_DIR")
            .ok()
            .map(|d| std::path::PathBuf::from(d).join("../../examples/widgets-gallery/src/front/app.at"))
            .filter(|p| p.exists()),
        Some(std::path::PathBuf::from("examples/widgets-gallery/src/front/app.at"))
            .filter(|p| p.exists()),
    ];
    let Some(path) = candidates.into_iter().flatten().next() else {
        eprintln!("plan412: SKIPPED — widgets-gallery app.at not found");
        return;
    };
    let code = std::fs::read_to_string(&path).expect("read app.at");
    let session = crate::session::CompilerSession::ui();
    let mut parser = crate::Parser::from(code.as_str()).with_session(session);
    let ast = parser.parse().expect("parse app.at");
    let mut paths: Vec<String> = Vec::new();
    for stmt in &ast.stmts {
        if let crate::ast::Stmt::WidgetDecl(decl) = stmt {
            let widget = crate::aura::extract_widget_from_decl(decl).expect("extract App widget");
            if let Some(routes) = widget.routes {
                paths.extend(routes.routes.iter().map(|r| r.path.clone()));
            }
        }
    }
    assert!(!paths.is_empty(), "app.at must declare routes");
    for expected in [
        "/row", "/col", "/center", "/flex", "/alignment", "/spacing",
        "/sizing", "/scroll", "/position", "/responsive", "/grid", "/grid-span",
    ] {
        assert!(
            paths.iter().any(|p| p == expected),
            "route {} missing from app.at (found {:?})",
            expected,
            paths
        );
    }
}
