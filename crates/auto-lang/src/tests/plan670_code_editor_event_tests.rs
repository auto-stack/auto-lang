//! PLAN-670 F-W3: code_editor 事件参数绑定回归（convert_code_editor 光杆
//! `event_to_message` → `event_to_message_with` 镜像，Plan 062 T9 input 同款）。
//!
//! 病灶：`oninput: .SrcChanged(i)` 的循环变量 i 此前不解析，落
//! `parse_event_param_literal` 空串 → handler `SrcChanged(int)` 收
//! `Str("")` → IndexError（auto-edit 矩阵 18 次，041 语料在案）。
//! 断言面：循环变量/字面量/点路径三类实参（对齐 062 T9 断言形态）。
#![cfg(test)]

use crate::ui::interpreter::DynamicMessage;
use crate::ui::view::View;

/// Build the view tree for a widget source (single decl, no imports).
fn build_view(src: &str) -> View<DynamicMessage> {
    let session = crate::session::CompilerSession::ui();
    let mut parser = crate::parser::Parser::from(src).with_session(session);
    let ast = parser.parse().expect("parse");
    let decls: Vec<crate::ast::WidgetDecl> = ast
        .stmts
        .iter()
        .filter_map(|st| match st {
            crate::ast::Stmt::WidgetDecl(d) => Some(d.clone()),
            _ => None,
        })
        .collect();
    let root_widget = crate::aura::extract_widget_from_decl(&decls[0]).expect("extract root");
    let comp = crate::ui::dynamic::DynamicComponent::with_registry_and_imports_from_decls(
        &decls[0],
        &decls[1..],
        &root_widget,
        crate::ui::widget_registry::WidgetRegistry::new(),
        vec![],
        &std::collections::HashMap::new(),
        false,
    )
    .expect("component");
    let (v, _, _) = comp.view_with_debug_gated(false);
    v
}

/// Recursively collect CodeEditor views by key (loop bodies nest in
/// containers/lists at arbitrary depth).
fn collect_editors<'a>(
    view: &'a View<DynamicMessage>,
    out: &mut Vec<(&'a str, &'a Option<DynamicMessage>, &'a Option<DynamicMessage>)>,
) {
    match view {
        View::CodeEditor { key, on_change, on_cursor, .. } => {
            out.push((key.as_str(), on_change, on_cursor));
        }
        View::Row { children, .. } | View::Column { children, .. } | View::List { items: children, .. } => {
            for child in children {
                collect_editors(child, out);
            }
        }
        View::Container { child, .. } | View::Scrollable { child, .. } => {
            collect_editors(child, out);
        }
        _ => {}
    }
}

/// The message-args shape shared by all three probes below: Typed message
/// whose args are the resolved ints — never `Str("")`.
fn assert_int_arg(msg: &DynamicMessage, event_name: &str, expected: i32, ctx: &str) {
    match msg {
        DynamicMessage::Typed { event_name: name, args, .. } => {
            assert_eq!(name, event_name, "{}: event name", ctx);
            assert_eq!(args.len(), 1, "{}: one arg ({:?})", ctx, args);
            match &args[0] {
                auto_val::Value::Int(v) => assert_eq!(*v, expected, "{}: int arg ({:?})", ctx, args),
                other => panic!("{}: expected Int({}), got {:?} (Str(\"\") = F-W3 病灶)", ctx, expected, other),
            }
        }
        other => panic!("{}: expected Typed message, got {:?}", ctx, other),
    }
}

/// F-W3 主体（041 形状）：`for i, t in` 循环内 code_editor 的
/// oninput/oncursor 循环变量实参解析——修复前两事件均落 Str("")。
#[test]
fn code_editor_loop_var_event_args_resolve() {
    let v = build_view(concat!(
        "widget P670Fw3Probe {\n",
        "    msg { SrcChanged(int), CursorMoved(int) }\n",
        "    model {\n",
        "        var tabs list = [ {key: \"a.at\", id: 3, src: \"x\"} ]\n",
        "    }\n",
        "    view {\n",
        "        col {\n",
        "            for i, t in .tabs {\n",
        "                code_editor (key: t.key, lang: \"none\") {\n",
        "                    content: t.src\n",
        "                    oninput: .SrcChanged(i)\n",
        "                    oncursor: .CursorMoved(i)\n",
        "                }\n",
        "                code_editor (key: \"lit\") {\n",
        "                    oninput: .SrcChanged(7)\n",
        "                }\n",
        "                code_editor (key: \"dot\") {\n",
        "                    oninput: .SrcChanged(t.id)\n",
        "                }\n",
        "            }\n",
        "        }\n",
        "    }\n",
        "    on {\n",
        "        .SrcChanged(idx) -> { }\n",
        "        .CursorMoved(idx) -> { }\n",
        "    }\n",
        "}\n",
    ));

    let mut editors = Vec::new();
    collect_editors(&v, &mut editors);
    assert_eq!(editors.len(), 3, "three editors built: {:?}", editors.iter().map(|(k, _, _)| k).collect::<Vec<_>>());

    let by_key = |k: &str| editors.iter().find(|(key, _, _)| *key == k).expect(k);
    let (key, on_change, on_cursor) = by_key("a.at");
    assert_eq!(*key, "a.at");
    // 041 形状：i = 循环索引 0。
    assert_int_arg(on_change.as_ref().expect("on_change baked"), "SrcChanged", 0, "loop-var oninput");
    assert_int_arg(on_cursor.as_ref().expect("on_cursor baked"), "CursorMoved", 0, "loop-var oncursor");
    // 字面量实参直取。
    assert_int_arg(by_key("lit").1.as_ref().unwrap(), "SrcChanged", 7, "literal oninput");
    // 前导绑定路径（t.id 多段 field 访问）经 bindings 解析。
    assert_int_arg(by_key("dot").1.as_ref().unwrap(), "SrcChanged", 3, "dotted-path oninput");
}

/// 无参事件回归锁：`.EditorCtx` 光杆形态（041 的 oncontextmenu 用法）
/// 经 _with 版仍烘焙为零参 Typed 消息——两版对无参事件等价（062 T9 同注记）。
#[test]
fn code_editor_no_param_event_still_bakes() {
    let v = build_view(concat!(
        "widget P670Fw3Bare {\n",
        "    msg { EditorCtx(float, float) }\n",
        "    view {\n",
        "        col {\n",
        "            code_editor (key: \"bare\") {\n",
        "                oninput: .SrcChanged\n",
        "            }\n",
        "        }\n",
        "    }\n",
        "}\n",
    ));

    let mut editors = Vec::new();
    collect_editors(&v, &mut editors);
    assert_eq!(editors.len(), 1);
    match editors[0].1.as_ref().expect("on_change baked") {
        DynamicMessage::Typed { event_name, args, .. } => {
            assert_eq!(event_name, "SrcChanged");
            assert!(args.is_empty(), "bare handler: zero args ({:?})", args);
        }
        other => panic!("expected Typed, got {:?}", other),
    }
}

/// 041-auto-edit 语料级验证（真实组件树：editor_store 双 tab 种子，仅激活
/// tab 的 code_editor 进视图——app.at `if t.key == .store.active_key` 门）。
/// 双证：①烘焙实参 Int(0) 非 Str("")；②经 comp.on()（生产派发入口，
/// DynamicMessage::Typed → VM handler）驱动 SrcChanged → .edits 自增 +
/// tabs[0].dirty 置真——Str("") 时代 handler `.tabs[i].dirty = true` 在此
/// IndexError 中断（auto-edit 矩阵 18 次的真身）。
#[cfg(feature = "ui-iced")]
#[test]
fn corpus_041_code_editor_events_reach_handler() {
    use crate::ui::component::Component;

    let Some(mut comp) = crate::plan370_test_support::build_example_component("041-auto-edit")
    else {
        eprintln!("skipping: 041-auto-edit corpus not found");
        return;
    };
    let (v, _, _) = comp.view_with_debug_gated(false);

    let mut editors = Vec::new();
    collect_editors(&v, &mut editors);
    assert_eq!(
        editors.len(), 1,
        "active tab only (active_key=tab-main): {:?}",
        editors.iter().map(|(k, _, _)| k).collect::<Vec<_>>()
    );
    assert_eq!(editors[0].0, "tab-main");
    let (on_change, on_cursor) = (
        editors[0].1.clone().expect("on_change baked"),
        editors[0].2.clone().expect("on_cursor baked"),
    );
    // ①烘焙断言：循环变量 i=0 以 Int 到达（修复前 Str("")）。
    assert_int_arg(&on_change, "SrcChanged", 0, "041 oninput bake");
    assert_int_arg(&on_cursor, "CursorMoved", 0, "041 oncursor bake");

    // ②派发断言：生产入口 comp.on(Typed) → EditorStore handler。
    let edits_before = match comp.read_state("edits").expect("edits readable") {
        auto_val::Value::Int(n) => n,
        other => panic!("edits seed: {:?}", other),
    };
    comp.on(on_change);
    let edits_after = match comp.read_state("edits").expect("edits readable") {
        auto_val::Value::Int(n) => n,
        other => panic!("edits after: {:?}", other),
    };
    assert_eq!(edits_after, edits_before + 1, "SrcChanged handler ran (edits+1)");
    // tabs 元素是堆上 VmRef——经桥物化后读字段（plan370 note_field 同款）。
    let tabs = comp.read_state_as_vec("tabs").expect("tabs readable");
    let elem = tabs
        .first()
        .unwrap_or_else(|| panic!("tabs[0] missing (len {})", tabs.len()))
        .clone();
    match comp.bridge().materialize_obj_ref(&elem) {
        auto_val::Value::Obj(fields) => {
            assert_eq!(
                fields.get("dirty"),
                Some(auto_val::Value::Bool(true)),
                "tabs[0].dirty flipped (IndexError 时代不可达)"
            );
        }
        other => panic!("tabs[0] not an Obj: {:?}", other),
    }

    // oncursor 同链路派发（SyncCursor 副链：.line/.col 状态面不动即达——
    // 烘焙 Int 已证，此处证 handler 不再中断：.edits 不受影响、无 panic）。
    comp.on(on_cursor);
}


/// PLAN-089(musk T-09): `readonly` prop reaches `View::CodeEditor` —
/// literal `true` bakes true, state binding resolves, absence defaults to
/// false (existing corpus untouched).
#[test]
fn code_editor_readonly_prop_bakes() {
    fn collect_readonly<'a>(
        view: &'a View<DynamicMessage>,
        out: &mut Vec<(&'a str, bool)>,
    ) {
        match view {
            View::CodeEditor { key, readonly, .. } => {
                out.push((key.as_str(), *readonly));
            }
            View::Row { children, .. } | View::Column { children, .. } | View::List { items: children, .. } => {
                for child in children {
                    collect_readonly(child, out);
                }
            }
            View::Container { child, .. } | View::Scrollable { child, .. } => {
                collect_readonly(child, out);
            }
            _ => {}
        }
    }

    let src = r#"
widget Demo {
    model {
        ro bool = true
    }
    view {
        col (style: "w-full") {
            code_editor (key: "ro-lit", lang: "rust", readonly: true) {}
            code_editor (key: "ro-state", lang: "rust", readonly: .ro) {}
            code_editor (key: "ro-default", lang: "rust") {}
        }
    }
}
"#;
    let v = build_view(src);
    let mut editors = Vec::new();
    collect_readonly(&v, &mut editors);
    assert_eq!(editors.len(), 3, "all three editors built: {:?}", editors);
    let get = |k: &str| -> bool {
        editors
            .iter()
            .find(|(key, _)| *key == k)
            .map(|(_, ro)| *ro)
            .unwrap_or_else(|| panic!("editor {k} missing"))
    };
    assert!(get("ro-lit"), "literal readonly: true bakes");
    assert!(get("ro-state"), "state binding .ro resolves");
    assert!(!get("ro-default"), "absent prop defaults false");
}
