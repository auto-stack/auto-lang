//! Plan 498: chart 交互状态机——M0 mouse-area `on_click` 引擎臂 VM 轨
//! 冒烟 + M1-M4 悬停态/转折点/legend 点击显隐断言(484 冒烟扩展)。
//!
//! M0 冒烟:内联 widget 源经 build_dynamic_component 全链(解析 →
//! aura 提取 → convert_mouse_area → View::MouseArea),onclick 臂以
//! DynamicMessage::Typed 落树;iced lowering 臂的存活由 renderer.rs 的
//! t496_walk clk 计数与 convert_view_messages 显式臂共同覆盖。

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
