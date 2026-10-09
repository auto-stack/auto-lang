//! PLAN-739：编辑回声→换页视面跟随回归（F-037-R1，jade-edit PLAN-037 探针
//! P2b/P5 探针面的本仓真实节拍围栏）。
//!
//! 缺陷形态（修前）：ADE `oninput` 事件不进 `input_state_map` →
//! `input_values` 键入条目经 retain 永生 → `patch_input_values` 每帧把
//! 模板求值的新页 content 覆写回旧页键入全文 → 编辑器视面粘滞钉死
//! （store 权威态正确）。修复 = patch 摘 ADE 臂 + update 记账单源化
//! （`track_input_text` / `retain_input_values_after_handler`）。
//!
//! 节拍口径 = 生产 update 链同构：记账（renderer.rs `track_input_text`，
//! 原 19586 位）→ 派发（`on_with_input_for`，原 19670 位）→ retain
//! （原 19981 位）→ 脏重建（`view` + `patch_input_values`，原 23870 位）。
//! 唯一不覆盖的是 iced OS 事件循环本身（同 plan732 六环边界）。

#[cfg(all(test, feature = "ui-iced"))]
mod plan739_input_pin_tests {
    use crate::ui::interpreter::DynamicMessage;
    use crate::ui::view::View;
    use std::collections::HashMap;

    fn locate_corpus() -> Option<std::path::PathBuf> {
        let rel = "test/ui/plan739_input_pin/src/front/app.at";
        let candidates = [
            std::env::var("CARGO_MANIFEST_DIR")
                .ok()
                .map(|d| std::path::PathBuf::from(d).join(rel)),
            Some(std::path::PathBuf::from(rel)),
            Some(std::path::PathBuf::from(format!("../../{}", rel))),
        ];
        candidates.into_iter().flatten().find(|p| p.exists())
    }

    /// 深走 View 树取首个 AutodownEditor 节点的 (key, value)。
    fn find_ade(v: &View<DynamicMessage>) -> Option<(String, String)> {
        match v {
            View::AutodownEditor { key, value, .. } => Some((key.clone(), value.clone())),
            View::AnchorSlot { child, .. }
            | View::Container { child, .. }
            | View::Scrollable { child, .. } => find_ade(child),
            View::MouseArea { content, .. } => find_ade(content),
            View::Column { children, .. } | View::Row { children, .. } => {
                children.iter().find_map(find_ade)
            }
            View::Grid { cells, .. } => cells.iter().find_map(find_ade),
            View::List { items, .. } => items.iter().find_map(find_ade),
            _ => None,
        }
    }

    /// AC-01/AC-02 主围栏：键入节拍 → 换页节拍 → 重建断言 ADE value 随
    /// store 切换（修前 = 键入全文钉死，红）。视面核随动断言：未覆写值
    /// 对换页新核必真重建。
    #[cfg(all(
        feature = "ui-interpreter",
        feature = "autodown",
        feature = "code-editor"
    ))]
    #[test]
    fn plan739_edit_then_page_switch_view_follows_store() {
        use crate::plan370_test_support::build_component_from_app;
        use crate::ui::component::Component;
        use crate::ui::iced::renderer::{
            patch_input_values, retain_input_values_after_handler, track_input_text,
        };

        let Some(mut dc) = locate_corpus()
            .as_deref()
            .and_then(|p| build_component_from_app(std::path::Path::new(p)))
        else {
            eprintln!("plan739: SKIPPED — corpus app.at not found");
            return;
        };

        // 首建：ADE value = state 权威投影（第一页）。
        let v0 = dc.view();
        let (key0, val0) = find_ade(&v0).expect("corpus must render autodown_editor");
        assert_eq!(key0, "p1");
        assert_eq!(val0, "第一页正文。");

        // ── 键入节拍（update 19586/19670/19981 生产记账序）──────────
        let typed = "第一页正文。XYZ";
        let mut iv: HashMap<String, String> = HashMap::new();
        track_input_text(&mut iv, "Edit", typed);
        dc.on_with_input_for("App", "Edit", Some(typed.to_string()));
        retain_input_values_after_handler(&mut iv, &dc, "Edit");
        assert!(
            iv.contains_key("Edit"),
            "ADE oninput 不在 input_state_map → 触发事件条目保留（根因前提在案）"
        );
        match dc.read_state("doc") {
            Ok(auto_val::Value::Str(s)) => assert_eq!(s.as_str(), typed, "键入回写 .at 落 state"),
            other => panic!("read_state(doc) 形态异常: {other:?}"),
        }

        // ── 换页节拍（nav 消息：无 input_value——派发 + handler 后 retain）──
        dc.on_with_input_for("App", "GoP2", None);
        retain_input_values_after_handler(&mut iv, &dc, "GoP2");
        assert!(
            iv.contains_key("Edit"),
            "Edit 不在 input_state_map → retain 不清除（永生条目=缺陷前提，F-037-R1 面向记录）"
        );
        match dc.read_state("page") {
            Ok(auto_val::Value::Str(s)) => assert_eq!(s.as_str(), "p2", "store 权威态已切页"),
            other => panic!("read_state(page) 形态异常: {other:?}"),
        }
        match dc.read_state("doc") {
            Ok(auto_val::Value::Str(s)) => assert_eq!(s.as_str(), "第二页正文。"),
            other => panic!("read_state(doc) 形态异常: {other:?}"),
        }

        // ── 脏重建（view + patch——渲染主链 23870 位同构）──────────────
        let mut v1 = dc.view();
        patch_input_values(&mut v1, &iv);
        let (key1, val1) = find_ade(&v1).expect("rebuild must keep autodown_editor");
        assert_eq!(key1, "p2", "key 随 page 投影换装（jade active_key 同构）");
        assert_eq!(
            val1, "第二页正文。",
            "F-037-R1：键入后换页视面必须随 store（被键入条目覆写钉死=本断言红）"
        );

        // 视面核随动（两段降层生产节律——PLAN-057/PLAN-732 注记：首帧
        // sync 先于 DocEditor::new 注册 = no-op，次帧 sync 装值入核）。
        // 修前此处次帧装的是键入覆写值 → 视面核钉死旧页（红）。
        use crate::ui::iced::renderer::IntoIcedElement;
        let _ = dc.view().into_iced();
        let mut v2 = dc.view();
        patch_input_values(&mut v2, &iv);
        let _ = v2.into_iced();
        let sk2 = crate::ui::autodown_editor::storage_key("p2");
        let core_text = crate::ui::autodown_editor::autodown_editor_text(&sk2).unwrap_or_default();
        assert_eq!(
            core_text, "第二页正文。",
            "F-037-R1 视面核半边：编辑器核必须装新页值（用户可见面跟随）"
        );
    }

    /// AC-03 单元锚：ADE 不被视面补丁覆写（摘臂语义）；Input 仍被覆写
    /// （既有单向 value 语义反例保持——Plan 319 grid 测试的同族锚）。
    #[test]
    fn plan739_patch_input_values_leaves_autodown_editor() {
        use crate::ui::iced::renderer::patch_input_values;

        let mut iv: HashMap<String, String> = HashMap::new();
        iv.insert("Edit".to_string(), "键入全文".to_string());

        let mut ade = View::<DynamicMessage>::AutodownEditor {
            key: "ed".to_string(),
            value: "模板求值值".to_string(),
            is_final: true,
            on_change: Some(DynamicMessage::String(".Edit".to_string())),
            on_focus: None,
            on_link: None,
            placeholder: None,
            style: None,
        };
        patch_input_values(&mut ade, &iv);
        match &ade {
            View::AutodownEditor { value, .. } => {
                assert_eq!(
                    value, "模板求值值",
                    "ADE 视面权威序列=state→sync 守卫→核；键入条目零作用"
                );
            }
            _ => unreachable!(),
        }

        let mut input = View::<DynamicMessage>::Input {
            placeholder: "p".to_string(),
            value: String::new(),
            on_change: Some(DynamicMessage::String(".Edit".to_string())),
            on_submit: None,
            width: None,
            password: false,
            style: None,
        };
        patch_input_values(&mut input, &iv);
        match &input {
            View::Input { value, .. } => {
                assert_eq!(
                    value, "键入全文",
                    "Input 单向 value 仍由补丁承载（零回退反例锚）"
                );
            }
            _ => unreachable!(),
        }
    }

    /// AC-03 守卫回退围栏（§5.1 案例二）：字面 content 载体键入后，重建帧
    /// sync(字面量原文) 必须零重建——摘除 patch 臂后历史动机（防逐帧重建
    /// 清焦点/丢用户文本）由 sync_external 三层守卫承担的实证锁定。
    #[cfg(all(
        feature = "ui-interpreter",
        feature = "autodown",
        feature = "code-editor"
    ))]
    #[test]
    fn plan739_literal_content_typing_sync_no_rebuild() {
        use crate::ui::autodown_editor::DocInput;
        use crate::ui::autodown_editor::{autodown_editor, storage_key};
        use crate::ui::code_editor::core::{
            set_font_system_call, EditorKey, EditorModifiers, NullClipboard,
        };

        // 测试装制 font system（core.rs tests run_fs 同款单例锁形态）。
        static FS: std::sync::OnceLock<std::sync::RwLock<cosmic_text::FontSystem>> =
            std::sync::OnceLock::new();
        set_font_system_call(|with| {
            let mut guard = FS
                .get_or_init(|| std::sync::RwLock::new(cosmic_text::FontSystem::new()))
                .write()
                .unwrap();
            with(&mut guard);
        });

        let sk = storage_key("p739lit");
        let core = autodown_editor(&sk);
        // 无 wikilink 语法的字面量（emit 全文保真；`[[..]]` 会被结构化
        // span 脱括号渲染——探针注记口径）。
        let literal = "甲编辑普通正文。";
        // 初装：字面量首见 = 真变化重建（last_external 记字面量）。
        assert!(core.sync_external(literal, true), "初装必须重建");

        // 键入（生产节律）：渲染帧排布几何（rects 由 render_frame WRITE
        // 填充——PLAN-737 回执口径）→ 点击建焦（重建后焦点=None，裸按键
        // 被丢——plan057 同因）→ End + 'X'：核内文本领先于模板字面量。
        let _ = crate::ui::code_editor::core::with_font_system(|fs| {
            core.render_frame(
                fs,
                400.0,
                crate::ui::code_editor::theme::Rgba {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                },
                None,
            )
        });
        let rect0 = core
            .block_rects()
            .first()
            .copied()
            .expect("render_frame must lay out block 0");
        {
            use crate::ui::code_editor::core::EditorButton;
            let (cx, cy) = (rect0.x + 8.0, rect0.y + 4.0);
            crate::ui::code_editor::core::with_font_system(|fs| {
                core.handle_input(
                    fs,
                    DocInput::MousePressed {
                        button: EditorButton::Left,
                        x: cx,
                        y: cy,
                    },
                    &mut NullClipboard,
                )
            });
            crate::ui::code_editor::core::with_font_system(|fs| {
                core.handle_input(
                    fs,
                    DocInput::MouseReleased {
                        button: EditorButton::Left,
                        x: cx,
                        y: cy,
                    },
                    &mut NullClipboard,
                )
            });
        }
        assert_eq!(
            core.focused_block(),
            Some(0),
            "点击必须建焦块 0（键入前置）"
        );
        let press = |key: EditorKey| {
            crate::ui::code_editor::core::with_font_system(|fs| {
                core.handle_input(
                    fs,
                    DocInput::KeyPressed {
                        key,
                        text: None,
                        modifiers: EditorModifiers::none(),
                    },
                    &mut NullClipboard,
                )
            });
        };
        press(EditorKey::End);
        press(EditorKey::Char('X'));

        // 重建帧：模板字面量原样 sync —— 守卫必须零重建（rebuild=true 红
        // = 守卫回退复活连打风暴，摘臂安全性前提被破坏）。
        assert!(
            !core.sync_external(literal, true),
            "字面 content 重建帧 sync 必须零重建（last_external 差分快路承担）"
        );
        assert_eq!(
            core.emit_document(),
            "甲编辑普通正文。X",
            "用户键入存活于核"
        );
    }
}
