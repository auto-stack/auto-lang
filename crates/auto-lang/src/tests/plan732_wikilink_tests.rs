//! PLAN-732：wikilink 激活供给六环贯通语料测试。
//!
//! 六环 = autodown-core parser（`[[t#a]]` → attr wikilink span）→
//! 编辑壳 core（flatten_inlines 链接区间 + 完整点击门 + DocOutput 激活）
//! → DocEditor.publish（Shell 真实路径）→ View callback（on_link）→
//! iced lowering（renderer VM 臂）→ AURA（`on "open-wiki-link"` 装配）→
//! DynamicMessage/IcedMessage（encode_payload 过 Send 边界）→ 真实 VM
//! handler（`.OpenWikiLink(target, anchor)` 形参绑定，state 落地）。
//!
//! 驱动面 = 生产同构：`build_component_from_app`（生产 parse/build 路径）
//! + `comp.view()`（AURA 装配）+ `into_iced()`（renderer lowering）+
//! `UserInterface::build/update`（widget.layout/update 生产事件臂）+
//! `comp.on(msg)`（VM 派发）。唯一不覆盖的是 OS 事件循环本身。

#[cfg(test)]
mod plan732_wikilink_tests {
    use auto_val::Value;

    fn locate_corpus() -> Option<std::path::PathBuf> {
        let rel = "test/ui/plan732_wikilink/src/front/app.at";
        let candidates = [
            std::env::var("CARGO_MANIFEST_DIR")
                .ok()
                .map(|d| std::path::PathBuf::from(d).join(rel)),
            Some(std::path::PathBuf::from(rel)),
            Some(std::path::PathBuf::from(format!("../../{}", rel))),
        ];
        candidates.into_iter().flatten().find(|p| p.exists())
    }

    fn state_str(dc: &crate::ui::dynamic::DynamicComponent, field: &str) -> String {
        match dc.read_state(field) {
            Ok(Value::Str(s)) => s.as_str().to_string(),
            Ok(other) => format!("{other:?}"),
            Err(e) => panic!("read_state('{field}') failed: {e}"),
        }
    }

    fn state_int(dc: &crate::ui::dynamic::DynamicComponent, field: &str) -> i32 {
        match dc.read_state(field) {
            Ok(Value::Int(i)) => i,
            Ok(other) => panic!("state '{field}' not int: {other:?}"),
            Err(e) => panic!("read_state('{field}') failed: {e}"),
        }
    }

    /// 六环一次激活贯通：真实 iced 事件（press+release 同链接区间）→
    /// Shell 消息 → VM handler 双参绑定 → state 落地（target/anchor 逐值、
    /// 调用恰一次）。负例：非链接点完整点击零派发。
    #[cfg(all(feature = "ui-interpreter", feature = "autodown", feature = "code-editor"))]
    #[test]
    fn plan732_six_ring_full_click_reaches_vm_handler() {
        use crate::ui::component::Component;
        use crate::ui::iced::renderer::IntoIcedElement;

        let mut dc = match crate::plan370_test_support::build_component_from_app(
            &locate_corpus().expect("corpus app.at missing"),
        ) {
            Some(c) => c,
            None => {
                eprintln!("plan732: SKIPPED — corpus app.at not found");
                return;
            }
        };

        // 环④⑤：comp.view()（AURA 装配 on_link）→ into_iced()（renderer
        // VM 臂 lowering，convert_view_messages 换型 IcedMessage）。
        let view = dc.view();
        // 首帧降层（建 core 注册表槽位；autodown_editor_sync 在槽位缺席时
        // no-op——生产动态循环每轮重降层，第二帧起内容入核，此处显式两段
        // 模拟同一节律）。
        let _ = dc.view().into_iced();
        let element: iced::Element<'_, crate::ui::interpreter::DynamicMessage> = view.into_iced();

        // 环①②前置：DocEditor 建核 + 布局（UserInterface::build 内走
        // widget.layout → measure → render_frame 生产路径）。
        let mut renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            crate::ui::iced::renderer::INTER_FONT,
            iced::Pixels(16.0),
        ));
        let mut ui = iced_runtime::user_interface::UserInterface::build(
            element,
            iced::Size::new(400.0, 600.0),
            iced_runtime::user_interface::Cache::default(),
            &mut renderer,
        );
        let mut messages: Vec<crate::ui::interpreter::DynamicMessage> = Vec::new();
        // RedrawRequested 先行（iced_test snapshot 同款驱动——首帧状态推进）。
        let _ = ui.update(
            &[iced::event::Event::Window(iced::window::Event::RedrawRequested(
                iced::time::Instant::now(),
            ))],
            iced::mouse::Cursor::Unavailable,
            &mut renderer,
            &mut iced::advanced::clipboard::Null,
            &mut messages,
        );

        // 取真实布局链接区（core 注册表，key = corpus 声明的 "p732doc"）。
        let sk = crate::ui::autodown_editor::storage_key("p732doc");
        let core = crate::ui::autodown_editor::autodown_editor(&sk);
        let regions = core.link_regions();
        assert_eq!(regions.len(), 2, "双链接命中区在册：{regions:?}");
        let r = &regions[0];
        let (cx, cy) = (r.rect.x + r.rect.w / 2.0, r.rect.y + r.rect.h / 2.0);

        // 环③（widget update 生产事件臂 + Shell.publish）。
        let _ = ui.update(
            &[iced::event::Event::Mouse(iced::mouse::Event::ButtonPressed(
                iced::mouse::Button::Left,
            ))],
            iced::mouse::Cursor::Available(iced::Point::new(cx, cy)),
            &mut renderer,
            &mut iced::advanced::clipboard::Null,
            &mut messages,
        );
        assert!(messages.is_empty(), "按下零消息：{messages:?}");
        let _ = ui.update(
            &[iced::event::Event::Mouse(iced::mouse::Event::ButtonReleased(
                iced::mouse::Button::Left,
            ))],
            iced::mouse::Cursor::Available(iced::Point::new(cx, cy)),
            &mut renderer,
            &mut iced::advanced::clipboard::Null,
            &mut messages,
        );
        assert_eq!(messages.len(), 1, "完整点击恰一条消息：{messages:?}");
        match &messages[0] {
            crate::ui::interpreter::DynamicMessage::Typed { widget_name, event_name, args } => {
                assert_eq!(widget_name, "App");
                assert_eq!(event_name, "OpenWikiLink");
                assert_eq!(args.len(), 2);
                assert!(matches!(&args[0], Value::Str(s) if s.as_str() == "目标页"), "{args:?}");
                assert!(matches!(&args[1], Value::Str(s) if s.as_str() == "锚点甲"), "{args:?}");
            }
            other => panic!("expected Typed message, got {other:?}"),
        }

        // 环⑥：VM 派发（on → decode_payload → call_handler 形参绑定）。
        dc.on(messages.remove(0));
        assert_eq!(state_str(&dc, "wiki_target"), "目标页");
        assert_eq!(state_str(&dc, "wiki_anchor"), "锚点甲");
        assert_eq!(state_int(&dc, "wiki_calls"), 1, "恰一次激活");

        // 负例：非链接点完整点击零派发（state 不再变化）。
        let plain = core.block_rects()[0];
        let (px, py) = (plain.x + 4.0, plain.y + 4.0);
        let _ = ui.update(
            &[iced::event::Event::Mouse(iced::mouse::Event::ButtonPressed(
                iced::mouse::Button::Left,
            ))],
            iced::mouse::Cursor::Available(iced::Point::new(px, py)),
            &mut renderer,
            &mut iced::advanced::clipboard::Null,
            &mut messages,
        );
        let _ = ui.update(
            &[iced::event::Event::Mouse(iced::mouse::Event::ButtonReleased(
                iced::mouse::Button::Left,
            ))],
            iced::mouse::Cursor::Available(iced::Point::new(px, py)),
            &mut renderer,
            &mut iced::advanced::clipboard::Null,
            &mut messages,
        );
        assert!(messages.is_empty(), "非链接零派发：{messages:?}");
        assert_eq!(state_int(&dc, "wiki_calls"), 1, "计数不变");
    }
}
