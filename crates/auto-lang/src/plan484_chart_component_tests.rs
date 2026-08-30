//! Plan 484 M4:024-charts 组件化迁移回归——内联几何退役后,四类图经
//! official 包裸名原语(line-chart/bar-chart/area-chart/donut-chart)消费,
//! 流式滑窗数据变化 → 子组件 Init 重放 → 几何自动重算(ADR-19)。

#[cfg(feature = "ui-iced")]
mod plan484_smoke {
    #[test]
    fn plan484_024_charts_component_smoke() {
        let app = match crate::plan370_test_support::locate_example_app_at("024-charts") {
            Some(p) => p,
            None => { eprintln!("plan484: SKIPPED — 024-charts sources not found"); return; }
        };
        let code = std::fs::read_to_string(&app).unwrap();
        let mut dc = crate::build_dynamic_component(&code, Some(app.to_str().unwrap()))
            .expect("024-charts must build");
        dc.fire_init();
        dc.set_route("/");
        let (view, _, _) = dc.view_with_debug_gated(true);
        let dump = format!("{:?}", view);
        assert!(dump.contains("svgdoc:"), "line chart svg must render");
        assert!(dump.contains("M 40"), "line path geometry must land (component Init replay)");
        assert!(dump.contains("图表工坊"), "app shell must render");
        assert!(!dump.contains("L 295"), "legacy inline geometry must be retired");
    }

    #[test]
    fn plan484_024_charts_streaming_recompute() {
        // 流式语义:Play 后 .Tick 追数据点,组件 Init 重放几何自动重算——
        // 30 ticks 后 x 轴出现 t30 标签且折线 path 已随滑窗数据更新。
        let app = match crate::plan370_test_support::locate_example_app_at("024-charts") {
            Some(p) => p,
            None => { eprintln!("plan484: SKIPPED — 024-charts sources not found"); return; }
        };
        let code = std::fs::read_to_string(&app).unwrap();
        let mut dc = crate::build_dynamic_component(&code, Some(app.to_str().unwrap()))
            .expect("024-charts must build");
        dc.fire_init();
        dc.set_route("/");
        dc.bridge_mut().call_handler("Play", &[]).expect("Play");
        for _ in 0..30 {
            dc.bridge_mut().call_handler("Tick", &[]).expect("Tick");
        }
        let (view, _, _) = dc.view_with_debug_gated(true);
        let dump = format!("{:?}", view);
        assert!(dump.contains("t29"), "streaming label t29 must reach the chart x-axis");
        assert!(dump.contains("M 40"), "recomputed path must render after 30 ticks");
    }

    #[test]
    fn plan484_charts_gallery_bare_names_render() {
        let app = match crate::plan370_test_support::locate_example_app_at("charts-gallery") {
            Some(p) => p,
            None => { eprintln!("plan484: SKIPPED — charts-gallery sources not found"); return; }
        };
        let code = std::fs::read_to_string(&app).unwrap();
        let mut dc = crate::build_dynamic_component(&code, Some(app.to_str().unwrap()))
            .expect("charts-gallery must build");
        dc.fire_init();
        dc.set_route("/");
        let (view, _, _) = dc.view_with_debug_gated(true);
        let dump = format!("{:?}", view);
        // 六图卡:四类几何全部落图(A 弧线/M 路径/L 折线)
        assert!(dump.contains("A100 100 0"), "donut arc must render");
        assert!(dump.contains("M 40"), "line/area path must render");
        assert!(dump.contains("h19"), "grouped bar width must render (slot*0.6/4)");
        assert!(dump.contains("8000"), "stacked domain nice-tick label must render");
        // 484 后续:y 刻度文本与图例名(经 loop-var 点路径)必须落进渲染树
        assert!(dump.contains("320"), "y tick label text must render");
        assert!(dump.contains("Desktop"), "legend series name must render");
    }
}

    #[test]
    fn body_props_flow_to_component() {
        fn gen_sfc(src: &str) -> String {
        let session = crate::session::CompilerSession::ui();
        let mut parser = crate::parser::Parser::from(src).with_session(session);
        let ast = parser.parse().expect("widget source must parse");
        let decl = ast.stmts.iter().find_map(|s| match s {
            crate::ast::Stmt::WidgetDecl(d) => Some(d),
            _ => None,
        }).expect("widget decl");
        let widget = crate::aura::extract_widget_from_decl(decl).expect("extract widget");
        let mut gen = crate::ui_gen::VueGenerator::new_shadcn();
        use crate::ui_gen::BackendGenerator;
        gen.generate(&widget).expect("generate SFC")
    }
        let sfc = gen_sfc(r##"
widget W {
    model { rows = [{a: 1, b: 2}] }
    view {
        col {
            bar-chart (id: "my-chart", type: "grouped", index: "q") {
                data: .rows
                fields: ["a"]
                colors: ["#2563eb"]
                grid: true
                legend: true
                tooltip: true
                labels: ["A"]
            }
        }
    }
}
"##);
        eprintln!("=== SFC ===\n{sfc}");
        assert!(sfc.contains(":data=\"rows\"") || sfc.contains("rows"), "data binding");
        assert!(sfc.contains("grouped"), "parens prop type");
        assert!(sfc.contains("my-chart"), "parens prop id");
        assert!(sfc.contains("'a'"), "body prop fields must emit");
        assert!(sfc.contains("#2563eb"), "body prop colors");
    }
