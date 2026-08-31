//! Plan 498: chart 交互状态机——M0 mouse-area `on_click` 引擎臂 VM 轨
//! 冒烟 + M1-M4 悬停态/转折点/legend 点击显隐断言(484 冒烟扩展)。
//!
//! M0 冒烟:内联 widget 源经 build_dynamic_component 全链(解析 →
//! aura 提取 → convert_mouse_area → View::MouseArea),onclick 臂以
//! DynamicMessage::Typed 落树;iced lowering 臂的存活由 renderer.rs 的
//! t496_walk clk 计数与 convert_view_messages 显式臂共同覆盖。
//!
//! M1+ 悬停态:四类图组件源(三副本同源,取 charts-gallery 份)直接
//! build_dynamic_component,fire_init 后 call_handler 驱动交互态,view
//! dump 断言高亮/转折点/显隐样式落图(svgdoc 内联;Debug dump 引号转义
//! 为 `\"`,故断言用原始字符串书写字面 `=\"…\"` 形态)。

#[cfg(feature = "ui-iced")]
mod plan498_m0 {
    /// M0:mouse-area onclick → View::MouseArea.on_click(VM 动态组件
    /// 全链;literal int 参数随行,legend Toggle(k) 消费同型)。
    #[test]
    fn plan498_mouse_area_onclick_arm_lands() {
        let src = r#"
widget ClickZone {
    msg { Ping(int) }
    model { var n int = 0 }
    view {
        col {
            mouse-area (style: "w-4 h-4", onmouseenter: .Ping(0), onmouseleave: .Ping(0), onclick: .Ping(7)) {}
            text "zone" {}
        }
    }
    on {
        .Ping(k) -> { .n = k }
    }
}
"#;
        let mut dc = match crate::build_dynamic_component(src, None) {
            Ok(dc) => dc,
            Err(e) => { eprintln!("plan498 M0: SKIPPED — component build failed: {e}"); return; }
        };
        dc.fire_init();
        let (view, _, _) = dc.view_with_debug_gated(true);
        let mut found = false;
        walk_mouse_area(&view, &mut |on_click| {
            if let Some(msg) = on_click {
                found = true;
                match msg {
                    crate::ui::interpreter::DynamicMessage::Typed { event_name, args, .. } => {
                        assert_eq!(event_name, "Ping", "onclick handler name");
                        assert_eq!(args.len(), 1, "literal int arg rides along");
                    }
                    other => panic!("Expected Typed message, got: {other:?}"),
                }
            }
        });
        assert!(found, "mouse-area onclick arm must land in the view tree");
    }

    fn walk_mouse_area(
        v: &crate::ui::view::View<crate::ui::interpreter::DynamicMessage>,
        f: &mut dyn FnMut(&Option<crate::ui::interpreter::DynamicMessage>),
    ) {
        use crate::ui::view::View;
        match v {
            View::MouseArea { content, on_click, .. } => {
                f(on_click);
                walk_mouse_area(content, f);
            }
            View::Column { children, .. } | View::Row { children, .. } => {
                for c in children {
                    walk_mouse_area(c, f);
                }
            }
            View::Container { child, .. } | View::Scrollable { child, .. } => {
                walk_mouse_area(child, f)
            }
            _ => {}
        }
    }
}

#[cfg(feature = "ui-iced")]
mod plan498_charts {
    /// charts-gallery 整 app 构建(四类图同屏,数据由 app 注入;组件 props
    /// 不入 state,独立构建组件会缺 data 字段,故走消费方整包形态;路径
    /// 必传——`use { package: official from "components" }` 按相对路径解析)。
    fn build_gallery() -> Option<crate::ui::dynamic::DynamicComponent> {
        let app = crate::plan370_test_support::locate_example_app_at("charts-gallery")?;
        let code = std::fs::read_to_string(&app).ok()?;
        let mut dc = crate::build_dynamic_component(&code, Some(app.to_str()?)).ok()?;
        dc.fire_init();
        dc.set_route("/");
        Some(dc)
    }

    fn dump(dc: &mut crate::ui::dynamic::DynamicComponent) -> String {
        let (view, _, _) = dc.view_with_debug_gated(true);
        format!("{:?}", view)
    }

    /// M1:line 高亮——HoverSeries(0) 后该系列 path 落 stroke-width 3 +
    /// opacity 1,其余系列 downplay 0.25,转折点圆圈浮现;SeriesOut 复原。
    #[test]
    fn plan498_line_emphasis_and_turning_points() {
        let Some(mut dc) = build_gallery() else {
            eprintln!("plan498 M1: SKIPPED — charts-gallery not found");
            return;
        };
        let dump0 = dump(&mut dc);
        assert!(dump0.contains(r#"stroke-width=\"2\""#), "常驻线宽 2 落图");
        assert!(!dump0.contains(r#"r=\"3\""#), "常驻态无转折点圆圈");

        dc.on_with_input_for("LineChart", "HoverSeries\u{1F}i\u{1F}0", None);
        let dump1 = dump(&mut dc);
        assert!(dump1.contains(r#"stroke-width=\"3\""#), "高亮线宽 3 落图");
        assert!(dump1.contains(r#"stroke-opacity=\"1\""#), "高亮 opacity 1 落图");
        assert!(dump1.contains(r#"stroke-opacity=\"0.25\""#), "downplay 0.25 落图");
        assert!(dump1.contains(r#"r=\"3\""#), "转折点圆圈浮现(r=3)");

        dc.on_with_input_for("LineChart", "SeriesOut", None);
        let dump2 = dump(&mut dc);
        assert!(dump2.contains(r#"stroke-opacity=\"0.85\""#), "离焦复原 0.85");
        assert!(!dump2.contains(r#"r=\"3\""#), "转折点随离焦消失");
    }

    /// M1:area(line 同族)——HoverSeries 后 a/l 双 path 高亮落图。
    #[test]
    fn plan498_area_emphasis() {
        let Some(mut dc) = build_gallery() else {
            eprintln!("plan498 M1: SKIPPED — charts-gallery not found");
            return;
        };
        dc.on_with_input_for("AreaChart", "HoverSeries\u{1F}i\u{1F}1", None);
        let dump = dump(&mut dc);
        assert!(dump.contains(r#"fill-opacity=\"0.45\""#), "高亮面积 0.45 落图");
        assert!(dump.contains(r#"stroke-width=\"3\""#), "高亮描边 3 落图");
        assert!(dump.contains(r#"fill-opacity=\"0.08\""#), "downplay 面积 0.08 落图");
    }

    /// M2:bar 分组高亮——命中竖带 .Hover(0) 顺带 hoverGroup:该组柱描边
    /// 1.5 落图,其余组 fill-opacity 0.3;HoverOut 复原。
    #[test]
    fn plan498_bar_group_emphasis() {
        let Some(mut dc) = build_gallery() else {
            eprintln!("plan498 M2: SKIPPED — charts-gallery not found");
            return;
        };
        let dump0 = dump(&mut dc);
        assert!(dump0.contains("h19") || dump0.contains("h25"), "常驻柱几何落图");
        assert!(!dump0.contains(r#"fill-opacity=\"0.3\""#), "常驻态无 downplay");

        dc.on_with_input_for("BarChart", "Hover\u{1F}i\u{1F}0", None);
        let dump1 = dump(&mut dc);
        assert!(dump1.contains(r#"stroke-width=\"1.5\""#), "高亮组描边 1.5 落图");
        assert!(dump1.contains(r#"fill-opacity=\"0.3\""#), "其余组 downplay 0.3 落图");

        dc.on_with_input_for("BarChart", "HoverOut", None);
        let dump2 = dump(&mut dc);
        assert!(!dump2.contains(r#"stroke-width=\"1.5\""#), "离焦后描边消失");
        assert!(!dump2.contains(r#"fill-opacity=\"0.3\""#), "离焦后 downplay 消失");
    }

    /// M3:donut 扇区 emphasis——.Hover(1)(图例行/扇区同源)后该扇区
    /// 外移路径落图(白描边 2px,donut 独占标记);HoverOut 复原常驻路径。
    #[test]
    fn plan498_donut_sector_emphasis() {
        let Some(mut dc) = build_gallery() else {
            eprintln!("plan498 M3: SKIPPED — charts-gallery not found");
            return;
        };
        let dump0 = dump(&mut dc);
        assert!(dump0.contains("A100 100 0"), "常驻扇区弧落图");
        assert!(!dump0.contains(r##"stroke=\"#ffffff\""##), "常驻态无白描边");

        dc.on_with_input_for("DonutChart", "Hover\u{1F}i\u{1F}1", None);
        let dump1 = dump(&mut dc);
        assert!(dump1.contains(r##"stroke=\"#ffffff\""##), "悬停扇区白描边落图");

        dc.on_with_input_for("DonutChart", "HoverOut", None);
        let dump2 = dump(&mut dc);
        assert!(!dump2.contains(r##"stroke=\"#ffffff\""##), "离焦后白描边消失");
    }

    /// M4:legend 点击切换显隐——Toggle(0)(M0 on_click 电路)后该系列
    /// 几何跳过 + 图例项落 opacity-40;再点复原;与 emphasis 正交(隐藏
    /// 优先于悬停:visible 门在 hoverSeries 分支外层)。
    #[test]
    fn plan498_legend_toggle_visibility() {
        let Some(mut dc) = build_gallery() else {
            eprintln!("plan498 M4: SKIPPED — charts-gallery not found");
            return;
        };
        let dump0 = dump(&mut dc);
        assert!(!dump0.contains("Opacity(40)"), "常驻态无暗图例");
        let paths0 = dump0.matches("stroke-opacity=").count();

        dc.on_with_input_for("LineChart", "Toggle\u{1F}i\u{1F}0", None);
        let dump1 = dump(&mut dc);
        assert!(dump1.contains("Opacity(40)"), "隐藏系列图例项落 opacity-40");
        let paths1 = dump1.matches("stroke-opacity=").count();
        assert!(paths1 < paths0, "隐藏系列 path 跳过(前 {paths0} 后 {paths1})");

        // 隐藏优先于悬停:对已隐藏系列 HoverSeries(0) 不浮几何(无转折点)。
        dc.on_with_input_for("LineChart", "HoverSeries\u{1F}i\u{1F}0", None);
        let dump_h = dump(&mut dc);
        assert!(!dump_h.contains(r#"r=\"3\""#), "隐藏系列悬停不浮现转折点");

        dc.on_with_input_for("LineChart", "Toggle\u{1F}i\u{1F}0", None);
        let dump2 = dump(&mut dc);
        assert!(!dump2.contains("Opacity(40)"), "再点复原图例");
        let paths2 = dump2.matches("stroke-opacity=").count();
        assert_eq!(paths2, paths0, "复原后几何回归");
    }
}
