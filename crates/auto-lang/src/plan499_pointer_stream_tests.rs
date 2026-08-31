//! Plan 499 M2:鼠标移动限频流管道 e2e——mouse-area onmousemove 事件经
//! PointerArea widget(坐标换算+限频,单元级见 `ui/iced/pointer_area.rs`)
//! 编码为 pipe-payload 消息后,`on_with_input → decode_payload → VM handler`
//! 全链把逻辑坐标送达组件状态(双 float 形参);coords 逻辑幅面随视图树
//! 下发(iced PointerArea extent 消费)。

#[cfg(feature = "ui-iced")]
mod plan499_pipe {
    const SRC: &str = r#"
widget Chart {
    msg Msg { PointerMove, HoverOut }
    model {
        px float = -1.0
        py float = -1.0
    }
    view {
        col {
            mouse-area (coords: "560x300", onmousemove: .PointerMove, onmouseleave: .HoverOut) {}
        }
    }
    on {
        .PointerMove(x, y) -> {
            .px = x
            .py = y
        }
        .HoverOut -> {
            .px = -1.0
        }
    }
}
"#;

    #[test]
    fn plan499_pointer_stream_pipe_delivers_logical_coords() {
        let mut dc = crate::build_dynamic_component(SRC, None)
            .expect("pointer-stream widget must build");
        dc.fire_init();

        // 视图树:on_move handler + 逻辑幅面必须接线(Debug dump 可见)。
        let (view, _, _) = dc.view_with_debug_gated(true);
        let dump = format!("{:?}", view);
        assert!(
            dump.contains("PointerMoveHandler"),
            "onmousemove must wire a move handler:\n{}",
            dump
        );
        assert!(
            dump.contains("560.0, 300.0") || dump.contains("(560.0, 300.0)"),
            "coords extent must ride the view tree:\n{}",
            dump
        );

        // 全链管道:PointerArea 发布的 pipe-payload 消息 → handler 双 float 形参。
        // (encode 形态 = encode_payload("PointerMove", [Float(x), Float(y)]))
        let msg = "PointerMove\u{1F}f\u{1F}280\u{1F}f\u{1F}150";
        dc.on_with_input(msg, None);
        let px = dc
            .bridge_mut()
            .read_state("px")
            .expect("px state must exist");
        let py = dc
            .bridge_mut()
            .read_state("py")
            .expect("py state must exist");
        assert!(
            matches!(px, auto_val::Value::Float(f) if (f - 280.0).abs() < 1e-6),
            "logical x must reach handler param, got {:?}",
            px
        );
        assert!(
            matches!(py, auto_val::Value::Float(f) if (f - 150.0).abs() < 1e-6),
            "logical y must reach handler param, got {:?}",
            py
        );

        // 节奏对照:同一坐标重复流(量化去重后的极端 = 静止)下 handler
        // 仍幂等(末值不变);不同坐标覆盖旧值。
        dc.on_with_input("PointerMove\u{1F}f\u{1F}420\u{1F}f\u{1F}90", None);
        let px2 = dc.bridge_mut().read_state("px").unwrap();
        assert!(
            matches!(px2, auto_val::Value::Float(f) if (f - 420.0).abs() < 1e-6),
            "subsequent move must overwrite, got {:?}",
            px2
        );
    }
}

#[cfg(feature = "ui-iced")]
mod plan499_axispointer {
    /// charts-gallery 整 app 构建(四类图同屏;line 卡 n=6 数据点,
    /// monthlyRevenue;left=40/right=550/step=102)。
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

    /// M3:axisPointer 十字线 + tooltip 跟随。
    /// x=145 → fi=(145-40)/510*5+0.5=1.53 → 吸附 i=1(Jan..Jun 第 2 点)
    /// → 竖线 cx=142("M 142 20 V 260");水平线 y=100 跟随("M 40 100 H 550");
    /// tooltip 标题随吸附点变 Feb、left 跟随 62px;PointerOut 复原。
    #[test]
    fn plan499_line_axispointer_crosshair_and_follow_tooltip() {
        let Some(mut dc) = build_gallery() else {
            eprintln!("plan499 M3: SKIPPED — charts-gallery not found");
            return;
        };
        let dump0 = dump(&mut dc);
        assert!(
            !dump0.contains(r#"d=\"M 142 20 V 260\""#),
            "无指针时不得有十字线竖线"
        );

        // PointerArea 发布形态的 pipe-payload 消息(逻辑坐标 145,100)
        dc.on_with_input_for(
            "LineChart",
            "PointerMove\u{1F}f\u{1F}145\u{1F}f\u{1F}100",
            None,
        );
        let dump1 = dump(&mut dc);
        assert!(
            dump1.contains(r#"d=\"M 142 20 V 260\""#),
            "竖线吸附数据点 i=1 → cx=142"
        );
        assert!(
            dump1.contains(r#"d=\"M 40 100 H 550\""#),
            "水平线跟随光标 y=100"
        );
        assert!(
            dump1.contains(r#"stroke-dasharray=\"3 3\""#),
            "axisPointer 虚线形态"
        );
        assert!(dump1.contains("Feb"), "tooltip 标题随吸附点(i=1=Feb)");
        assert!(
            dump1.contains("LeftOffset(62.0)"),
            "tooltip left 跟随(cx-80=62;动态类串在 dump 为解析后 LeftOffset 形态)"
        );

        // 边界钳制:y 超上限钳 260,tooltip left 超上限钳 400
        dc.on_with_input_for(
            "LineChart",
            "PointerMove\u{1F}f\u{1F}560\u{1F}f\u{1F}290",
            None,
        );
        let dump2 = dump(&mut dc);
        assert!(
            dump2.contains(r#"d=\"M 40 260 H 550\""#),
            "水平线 y 钳制 260(超上限)"
        );
        assert!(
            dump2.contains("LeftOffset(400.0)"),
            "tooltip left 钳制 400(右缘溢出保护)"
        );
        assert!(dump2.contains("Jun"), "x=560 吸附末点 i=5(Jun)");

        // PointerOut → 十字线消失
        dc.on_with_input_for("LineChart", "PointerOut", None);
        let dump3 = dump(&mut dc);
        assert!(
            !dump3.contains(r#"d=\"M 142 20 V 260\""#),
            "离场后十字线消失"
        );
        assert!(!dump3.contains("LeftOffset(62.0)"), "离场后 tooltip 消失");
    }

    /// M3:命中层重构——N 竖带 bands 退役,单全图 mouse-area(coords
    /// 逻辑幅面 + on_move handler)落树。
    #[test]
    fn plan499_line_single_pointer_hit_area() {
        let Some(mut dc) = build_gallery() else {
            eprintln!("plan499 M3: SKIPPED — charts-gallery not found");
            return;
        };
        let (view, _, _) = dc.view_with_debug_gated(true);
        let dump = format!("{:?}", view);
        assert!(
            dump.contains("PointerMoveHandler"),
            "折线卡命中区必须是带 on_move 的 mouse-area"
        );
        assert!(
            dump.contains("(560.0, 300.0)"),
            "coords 逻辑幅面 560x300 必须随树下发"
        );
        assert!(
            !dump.contains("w-[102px]"),
            "竖带命中区(w-[slot])必须退役"
        );
    }
}
