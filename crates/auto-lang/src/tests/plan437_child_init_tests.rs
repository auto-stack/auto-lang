//! Plan 437 Phase 2 —— VM 轨子组件 Init 生命周期钉子。
//!
//! 症状(2026-08-28 实证):VM 轨只有根 widget 的 Init 会触发(fire_init);
//! 路由页与视图中实例化的子组件 Init 从不触发 —— 子组件无法在渲染期做
//! 任何派生计算(chart 几何是典型)。vue 轨子组件 Init(onMounted)正常,
//! 属跨轨语义缺口。修复:render_child_widget 在 props 播种后补发子组件
//! Init(每渲染帧重放;统一 state 架构下逐实例顺序 props→Init→build,
//! 纯派生 Init 幂等)。
//!
//! 钉子:父组件持数据实例化子几何组件,断言渲染产物中出现 Init 计算的
//! path 字符串(修复前为空)。

#![cfg(test)]

#[cfg(test)]
mod plan437_child_init {
    /// 内联 fixture:父 widget 实例化 ChildGeo(props: data/field),
    /// 子 Init 从 props 计算 path 字符串。镜像 plan370_test_support 的
    /// 生产构建路径(parse → extract → registry+child_decls →
    /// with_registry_and_imports_from_decls)。
    #[cfg(feature = "ui-interpreter")]
    fn build_inline(src: &str) -> crate::ui::dynamic::DynamicComponent {
        use crate::ast::Stmt;
        let session = crate::session::CompilerSession::ui();
        let mut parser = crate::Parser::from(src).with_session(session);
        let ast = parser.parse().expect("parse");
        let mut root_decl = None;
        let mut view_widget = None;
        let mut child_decls = Vec::new();
        for stmt in &ast.stmts {
            if let Stmt::WidgetDecl(decl) = stmt {
                if root_decl.is_none() {
                    root_decl = Some(decl.clone());
                    view_widget = Some(
                        crate::aura::extract_widget_from_decl(decl)
                            .map_err(|e| e.to_string())
                            .unwrap(),
                    );
                } else {
                    child_decls.push(decl.clone());
                }
            }
        }
        let root_decl = root_decl.expect("root widget");
        let view_widget = view_widget.expect("view widget");

        let registry = crate::ui::widget_registry::WidgetRegistry::new();
        // 子 widget 注册(镜像 run_file_dynamic_ui_inner 的 child 收集)。
        let mut registry = registry;
        for d in &child_decls {
            let w = crate::aura::extract_widget_from_decl(d)
                .map_err(|e| e.to_string())
                .unwrap();
            registry.register(w);
        }
        crate::ui::dynamic::DynamicComponent::with_registry_and_imports_from_decls(
            &root_decl,
            &child_decls,
            &view_widget,
            registry,
            Vec::new(),
            &std::collections::HashMap::new(),
            false,
        )
        .expect("component builds")
    }

    const SRC: &str = r##"
widget Parent {
    model {
        series = [
            { m: "A", v: 30 },
            { m: "B", v: 80 },
            { m: "C", v: 10 }
        ]
    }
    view {
        col {
            child-geo (data: .series, field: "v") {}
        }
    }
}

widget ChildGeo (data: List, field: str = "v") {
    msg { Init }
    model {
        path str = ""
    }
    on {
        .Init -> {
            var left float = 40.0
            var bottom float = 260.0
            var vmax int = 0
            for d in .data {
                var v = d[.field]
                if v > vmax {
                    vmax = v
                }
            }
            var vmaxf float = vmax
            var span float = 250.0
            var dstr = ""
            var n int = .data.len()
            var i = 0
            for d in .data {
                var vf float = d[.field]
                var xf float = left + i * (510.0 / 2.0)
                var yf float = bottom - vf / vmaxf * span
                if i == 0 {
                    dstr = f"M ${xf} ${yf}"
                } else {
                    dstr = dstr + f" L ${xf} ${yf}"
                }
                i = i + 1
            }
            .path = dstr
        }
    }
    view {
        svg (viewBox: "0 0 560 300") {
            path (d: .path, fill: "none", stroke: "#2563eb") {}
        }
    }
}
"##;

    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn child_init_fires_during_render() {
        let mut dc = build_inline(SRC);
        dc.fire_init();
        let (view, _, _) = dc.view_with_debug_gated(true);
        let dump = format!("{:?}", view);
        assert!(
            dump.contains("M 40") && dump.contains("L 295"),
            "child Init geometry must land in the rendered view; dump head: {}",
            &dump[..dump.len().min(1200)]
        );
    }

    /// 端到端(生产构建路径):widgets-gallery 四条 chart 路由,每页消费
    /// 官方 Auto 组件(auto-*-chart),断言 Init 几何(M/L/A path 或
    /// 图例)进入渲染产物。缺 gallery 源(独立仓运行)时优雅跳过。
    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn gallery_chart_components_render_geometry() {
        let app = match crate::plan370_test_support::locate_example_app_at("widgets-gallery") {
            Some(p) => p,
            None => {
                eprintln!("plan437: SKIPPED — widgets-gallery sources not found");
                return;
            }
        };
        let code = std::fs::read_to_string(&app).unwrap();
        for (route, marker) in [
            ("/line-chart", "M 40 "),
            ("/bar-chart", "h6"),
            ("/area-chart", " L40 260 Z"),
            ("/donut-chart", "A100 100 0"),
        ] {
            // 生产构建路径(routes 页装载 + 组件包装载 + 单 VM 合成)。
            let mut dc = crate::build_dynamic_component(&code, Some(app.to_str().unwrap()))
                .unwrap_or_else(|e| panic!("{route}: gallery must build: {e}"));
            dc.fire_init();
            dc.set_route(route);
            let (view, _, _) = dc.view_with_debug_gated(true);
            let dump = format!("{:?}", view);
            assert!(
                dump.contains(marker),
                "{route}: geometry marker `{marker}` missing from render (child Init not firing?); dump head: {}",
                &dump[..dump.len().min(1000)]
            );
        }
    }
}
