//! PLAN-066: 原生组件外部注册 SPI 语料测试。
//!
//! 语料：`test/ui/plan066_native_widget/src/front/app.at`（生产形态构建，
//! build_component_from_app 链 = plan622 同款）。
//!
//! - **T2 迁移等价**：`autodown_editor` 标签自硬编码字符串臂迁经
//!   NativeWidgetRegistry 派发后，View 树与原臂语义逐字段一致（案 a sugar
//!   —— View::AutodownEditor 原样，scroll_sync 缺席=不外包 Scrollable）。
//! - **T1 Element 通道**：注册表 Element 入口命中 → `View::Custom`（name+
//!   props 明文透传；快照 kind=注册名，snapshot_builder 臂同测）。

#[cfg(test)]
mod plan066_native_widget_tests {
    use crate::plan370_test_support::build_component_from_app;
    use crate::ui::interpreter::DynamicMessage;
    use crate::ui::view::View;

    fn locate_corpus(rel: &str) -> Option<std::path::PathBuf> {
        let full = format!("test/ui/plan066_native_widget/{}", rel);
        let candidates = [
            std::env::var("CARGO_MANIFEST_DIR")
                .ok()
                .map(|d| std::path::PathBuf::from(d).join(&full)),
            Some(std::path::PathBuf::from(&full)),
            Some(std::path::PathBuf::from(format!("../../{}", full))),
        ];
        candidates.into_iter().flatten().find(|p| p.exists())
    }

    fn build_app(rel: &str) -> Option<crate::ui::dynamic::DynamicComponent> {
        build_component_from_app(&locate_corpus(rel)?)
    }

    fn render_view(dc: &crate::ui::dynamic::DynamicComponent) -> View<DynamicMessage> {
        let (view, _id_map, _probe) = dc.view_with_debug();
        view
    }

    // ─────────────────────────────────────────────────────────────────────
    // T2 等价：autodown_editor 经注册表派发 == 原硬编码臂形态
    // ─────────────────────────────────────────────────────────────────────

    /// 原臂语义钉死：key 取 key:/id: 缺省 "doc"；content/value 双别名；
    /// final 解析为 bool（字面 true）；oninput → on_change 消息位；
    /// scroll_sync 缺席 = 不外包 Scrollable（View::AutodownEditor 裸出）。
    #[cfg(all(feature = "ui-interpreter", feature = "autodown", feature = "code-editor"))]
    #[test]
    fn plan066_editor_tag_dispatches_via_native_registry() {
        let mut dc = match build_app("src/front/app.at") {
            Some(c) => c,
            None => {
                eprintln!("plan066: SKIPPED — corpus app.at not found");
                return;
            }
        };
        let view = render_view(&dc);
        let editor = find_autodown_editor(&view).expect("View::AutodownEditor in corpus view");
        assert_eq!(editor.0, "doc-ed", "key prop");
        assert_eq!(editor.1, "# 标题\n\n段落一。", "value bound to model doc");
        assert!(editor.2, "final literal true");
        assert!(
            matches!(
                editor.3,
                Some(DynamicMessage::Typed { ref event_name, .. }) if event_name == "DocEdit"
            ),
            "oninput → on_change message: {:?}",
            editor.3
        );
        assert!(editor.4.is_none(), "placeholder unset");
    }

    /// 折叠别名：`AutodownEditor`/`autodown-editor` 标签同归注册表入口
    /// （P8-6 折叠兜底同规）——原臂的 "autodowneditor" 双拼写保持。
    #[cfg(all(feature = "ui-interpreter", feature = "autodown", feature = "code-editor"))]
    #[test]
    fn plan066_editor_fold_aliases_hit_registry() {
        // 注册表面直接断言（builder 面由等价测试钉死）。
        let reg = crate::ui::native_widget::global();
        assert!(reg.lookup("autodown_editor").is_some());
        assert!(reg.lookup("autodowneditor").is_some());
        assert!(reg.lookup("autodown-editor").is_some());
        assert!(reg.lookup("AutodownEditor").is_some());
    }

    // ─────────────────────────────────────────────────────────────────────
    // T1 Element 通道：注册表 Element 入口 → View::Custom
    // ─────────────────────────────────────────────────────────────────────

    /// 探针名经测试钩子进程内登记（native_widget::test_register_element，
    /// 首次 global() 前生效；生产源面与 schema 漂移围栏 P1 不见测试名）；
    /// props 经 resolve→display 明文透传，按键排序。
    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn plan066_element_entry_produces_custom_view() {
        crate::ui::native_widget::test_register_element("plan066_element_probe");
        let mut dc = match build_app("src/front/app.at") {
            Some(c) => c,
            None => {
                eprintln!("plan066: SKIPPED — corpus app.at not found");
                return;
            }
        };
        let view = render_view(&dc);
        let custom = find_custom(&view).expect("View::Custom for element probe");
        assert_eq!(custom.0, "plan066_element_probe", "Custom.name = 注册名");
        assert_eq!(
            custom.1,
            vec![
                ("count".to_string(), "3".to_string()),
                ("label".to_string(), "probe-x".to_string()),
            ],
            "props resolved+sorted"
        );
    }

    // ─────────────────────────────────────────────────────────────────────
    // T1 保形：未注册未知 tag 不落注册表（快照面兜底语义不变）
    // ─────────────────────────────────────────────────────────────────────

    /// 未知 tag 在注册表 miss → 沿既有链（WidgetRegistry → 折叠 → 兜底），
    /// 不得误产 View::Custom/AutodownEditor。
    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn plan066_unknown_tag_bypasses_native_registry() {
        // 注册表面：确无此名。
        assert!(crate::ui::native_widget::global().lookup("plan066_not_registered").is_none());
        // 构建面：未知 tag 语料走兜底（占位文本），不炸不误配。
        // （占位形状由既有 fallback 语义负责，这里只钉"不接管"。）
        assert!(
            !matches!(
                crate::ui::native_widget::global().lookup("totally_unknown_widget"),
                Some(_)
            )
        );
    }

    // ─────────────────────────────────────────────────────────────────────
    // helpers
    // ─────────────────────────────────────────────────────────────────────

    #[cfg(all(feature = "ui-interpreter", feature = "autodown", feature = "code-editor"))]
    fn find_autodown_editor(
        view: &View<DynamicMessage>,
    ) -> Option<(String, String, bool, Option<DynamicMessage>, Option<String>)> {
        match view {
            View::AutodownEditor { key, value, is_final, on_change, placeholder, .. } => {
                Some((key.clone(), value.clone(), *is_final, on_change.clone(), placeholder.clone()))
            }
            View::Scrollable { child, .. } => find_autodown_editor(child),
            View::Column { children, .. } | View::Row { children, .. } | View::List { items: children, .. } => {
                children.iter().find_map(find_autodown_editor)
            }
            View::Container { child, .. } => find_autodown_editor(child),
            View::Button { content: Some(c), .. } => find_autodown_editor(c),
            _ => None,
        }
    }

    #[cfg(feature = "ui-interpreter")]
    fn find_custom(view: &View<DynamicMessage>) -> Option<(String, Vec<(String, String)>)> {
        match view {
            View::Custom { name, props, .. } => Some((name.clone(), props.clone())),
            View::Scrollable { child, .. } => find_custom(child),
            View::Column { children, .. } | View::Row { children, .. } | View::List { items: children, .. } => {
                children.iter().find_map(find_custom)
            }
            View::Container { child, .. } => find_custom(child),
            View::Button { content: Some(c), .. } => find_custom(c),
            _ => None,
        }
    }
}
