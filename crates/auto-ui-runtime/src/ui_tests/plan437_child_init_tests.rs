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
        use auto_lang_core::ast::Stmt;
        let session = auto_lang_core::session::CompilerSession::ui();
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
                        auto_ui_compiler::aura::extract_widget_from_decl(decl)
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
            let w = auto_ui_compiler::aura::extract_widget_from_decl(d)
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

    /// PLAN-762 T-02: 多片 Init 夹具——250k 次自旋迭代，实测 ~16.7 VM
    /// 指令/迭代 ≈ 4.2M 步，> 2×1M 指令片帽 → 必跨 ≥3 片；< 10M 累计
    /// 护栏（余量约 2.4×）。片内让出点由指令帽绑定，与机器速度/负载
    /// 无关（快机恰 1M 指令/片封顶，慢机 4ms 墙钟帽只让片更短、片数
    /// 更多）——确定性复现 761 tt 批量档 flake 的早退机制。
    /// 250040 = 40 + 自旋计数，marker 本身即"Init 真跑完"的证据。
    const SRC_SPIN: &str = r##"
widget ParentSpin {
    model {
        spins int = 250000
    }
    view {
        col {
            child-spin (spins: .spins) {}
        }
    }
}

widget ChildSpin (spins: int = 0) {
    msg { Init }
    model {
        path str = ""
    }
    on {
        .Init -> {
            var n int = .spins
            var i int = 0
            var acc int = 0
            while i < n {
                acc = acc + 1
                i = i + 1
            }
            var x float = 40.0 + acc
            .path = f"M ${x} 260"
        }
    }
    view {
        svg (viewBox: "0 0 560 300") {
            path (d: .path, fill: "none", stroke: "#2563eb") {}
        }
    }
}
"##;

    /// PLAN-762 T-02: 完成态 + 有界截止时间驱动。单次
    /// `drive_scheduler_to_quiescence` 的 `progressed` 判据只认派发/完成，
    /// 不计"续跑片让出未完成"——多片 Init 单调用不保证收敛（761 tt
    /// flake 机制，分诊见 docs/plans/reports/762-chart-init-attribution.md）。
    /// 本循环驱动到终态（无排队 demand / 无在途 CPU / 无 I/O 凭据）才
    /// 放行断言；有界超时红并带 pending/CPU/I/O 诊断。循环内重建视图：
    /// 新落地状态可能登记下一批子件 demand（plan502 完成态驱动同
    /// precedent，PLAN-759 T-08）。
    #[cfg(feature = "ui-interpreter")]
    fn drive_to_completion(
        dc: &mut crate::ui::dynamic::DynamicComponent,
        label: &str,
        deadline: std::time::Duration,
    ) {
        let start = std::time::Instant::now();
        loop {
            dc.drive_scheduler_to_quiescence(10_000);
            if !(dc.has_pending_init_work()
                || dc.has_cpu_continuations()
                || dc.has_parked_io_tasks())
            {
                return;
            }
            assert!(
                start.elapsed() < deadline,
                "{label}: Init/CPU 续体 {deadline:?} 未收敛（完成态驱动超时；pending_init={} cpu_cont={} parked_total={}）",
                dc.bridge_mut().pending_init_demand_count(),
                dc.bridge_mut().cpu_continuation_count(),
                dc.bridge_mut().parked_count(),
            );
            let _ = dc.view_with_debug_gated(true);
        }
    }

    /// PLAN-762 T-02: 机制钉子——必跨 ≥3 片的 Init 确定性复现单调用早退
    /// （续体在途即出循环）；完成态截止时间驱动仍可收敛且几何落地。
    /// 注：前半断言钉的是 helper 现行 `progressed` 判据（KNOWN-DEBT
    /// D-762-1）；若日后修复 helper 退出判据，本断言按新语义更新。
    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn child_init_multi_slice_early_exit_and_completion_driving() {
        let mut dc = build_inline(SRC_SPIN);
        dc.fire_init();
        let (view0, _, _) = dc.view_with_debug_gated(true);
        let dump0 = format!("{view0:?}");
        assert!(
            dump0.contains("Loading…"),
            "pending Init demand must render the skeleton placeholder; dump head: {}",
            &dump0[..dump0.len().min(800)]
        );
        // 复现：单次驱动对多片 Init 早退（progressed 不计片让出）。
        dc.drive_scheduler_to_quiescence(10_000);
        assert!(
            dc.has_pending_init_work() || dc.has_cpu_continuations(),
            "多片 Init 单次驱动后仍应在途（早退机制复现）；若本断言翻红，helper 退出判据已变更——按 docs/plans/reports/762-chart-init-attribution.md 更新机制钉子"
        );
        // 阻断：完成态截止时间驱动收敛，自旋计数几何落地。
        drive_to_completion(&mut dc, "multi-slice", std::time::Duration::from_secs(30));
        let (view, _, _) = dc.view_with_debug_gated(true);
        let dump = format!("{view:?}");
        assert!(
            dump.contains("M 250040 260"),
            "multi-slice Init geometry must land (250040 = 40 + 250000 spins proves completion); dump head: {}",
            &dump[..dump.len().min(800)]
        );
    }

    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn child_init_fires_during_render() {
        let mut dc = build_inline(SRC);
        dc.fire_init();
        // PLAN-711 T-05 (Q-01)：pending demand 首建渲染轻量骨架占位（不渲染
        // 子树产物、不永久早退）；驱动至终态后重建出全量几何。
        let (view0, _, _) = dc.view_with_debug_gated(true);
        let dump0 = format!("{view0:?}");
        assert!(
            dump0.contains("Loading…"),
            "pending Init demand must render the skeleton placeholder; dump head: {}",
            &dump0[..dump0.len().min(800)]
        );
        dc.drive_scheduler_to_quiescence(10_000);
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
            let _ = dc.view_with_debug_gated(true);
            // PLAN-762 T-02：完成态截止时间驱动——单次驱动对多片 Init
            // 早退（progressed 不计片让出，761 tt flake），续体在途不
            // 提前验收；marker 原断言不动。
            drive_to_completion(&mut dc, route, std::time::Duration::from_secs(30));
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
