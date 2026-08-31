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
