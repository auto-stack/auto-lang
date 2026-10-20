//! Plan 413: code editor payload natives (UI bridge) 端到端。
//!
//! PLAN-814 T-02 迁移上移：本组测试消费上层 UI 面（ui::code_editor 真实
//! cosmic-text registry），原居 auto-lang-core vm/native.rs 的 `code-editor`
//! 门在 SDK 无定义——迁移后死门；PLAN-814 勘误为 SDK 侧空 feature（a2r
//! diff 双轨）后统一构建复活暴露滞留，按真实所有者上移本包恢复选择。

use super::*;

#[test]
fn vm_code_editor_natives_end_to_end() {
    // PLAN-814 T-04：随迁后测试进程不再链接 auto-lang 漏斗——provider 槽
    // 显式装配（r3 §5.3 显式优先；原 run 漏斗隐式安装面）。
    crate::install_ui_host_capabilities();
    use crate::ui::code_editor as ce;
    use std::sync::{OnceLock, RwLock};

    // Serialize against the registry-touching tests (LRU sweep).
    let _guard = crate::ui::code_editor::core::REGISTRY_TEST_LOCK
        .lock()
        .unwrap();

    fn install_fs(with: &mut dyn FnMut(&mut cosmic_text::FontSystem)) {
        static FS: OnceLock<RwLock<cosmic_text::FontSystem>> = OnceLock::new();
        let fs = FS.get_or_init(|| RwLock::new(cosmic_text::FontSystem::new()));
        let mut guard = fs.write().unwrap();
        with(&mut guard);
    }
    ce::set_font_system_call(install_fs);

    let key = ce::storage_key("vm-native-test");
    ce::code_editor_dispose(&key);
    let config = ce::CodeEditorConfig::default();
    ce::code_editor(&key, &config);
    assert!(ce::code_editor_set_text(&key, "line one\nline two"));

    // Script-level native calls through the full Codegen → VM → shim
    // pipeline.
    let (_result, out) = auto_lang_core::run_with_capture(
        r#"
let t = code_editor_text("__code_editor_vm-native-test")
print(t)
let l = code_editor_cursor_line("__code_editor_vm-native-test")
print(l)
let c = code_editor_cursor_col("__code_editor_vm-native-test")
print(c)
let ok = code_editor_set_text("__code_editor_vm-native-test", "rewritten")
print(ok)
let t2 = code_editor_text("__code_editor_vm-native-test")
print(t2)
"#,
    )
    .unwrap();
    assert!(out.contains("line one"), "text read: {out}");
    assert!(out.contains("0"), "cursor line: {out}");
    assert!(out.contains("rewritten"), "set+read roundtrip: {out}");

    // Plan 428 P1: fold natives round-trip (headless — no render).
    ce::code_editor_set_text(
        &key,
        "fn a() {
body()
}
",
    );
    let (_r2, out2) = auto_lang_core::run_with_capture(
        r#"
let folded = code_editor_fold_toggle("__code_editor_vm-native-test", 1)
print(folded)
let hidden = code_editor_fold_hidden_count("__code_editor_vm-native-test")
print(hidden)
let folded2 = code_editor_fold_toggle("__code_editor_vm-native-test", 1)
print(folded2)
let hidden2 = code_editor_fold_hidden_count("__code_editor_vm-native-test")
print(hidden2)
"#,
    )
    .unwrap();
    // Plan 489：断言对齐现 print 语义——bool 渲染为 true/false
    // 字面量（旧注释「renders bools as 1/0」停留在 shim 时代，
    // P487-2 既有红根因之一）；逐行精确比对防子串误判。
    assert_eq!(
        out2.lines().collect::<Vec<_>>(),
        vec!["true", "2", "false", "0"],
        "fold 往返：首 toggle true / 隐 2 行 / 次 toggle false / 归 0"
    );

    // Missing editor key raises a runtime error (not a panic).
    let err = auto_lang_core::run(r#"let x = code_editor_text("__code_editor_no-such")"#);
    assert!(err.is_err());
}

/// Plan 673 T-01: `code_editor_delta` end-to-end through the full
/// Codegen → VM → shim pipeline — destructive read with a constant
/// JSON shape (`{"revision":N,"deltas":[...]}`, empty → `"deltas":[]`).
#[test]
fn vm_code_editor_delta_end_to_end() {
    // PLAN-814 T-04：随迁后测试进程不再链接 auto-lang 漏斗——provider 槽
    // 显式装配（r3 §5.3 显式优先；原 run 漏斗隐式安装面）。
    crate::install_ui_host_capabilities();
    use crate::ui::code_editor as ce;
    use std::sync::{OnceLock, RwLock};

    // Serialize against the registry-touching tests (LRU sweep).
    let _guard = crate::ui::code_editor::core::REGISTRY_TEST_LOCK
        .lock()
        .unwrap();

    fn install_fs(with: &mut dyn FnMut(&mut cosmic_text::FontSystem)) {
        static FS: OnceLock<RwLock<cosmic_text::FontSystem>> = OnceLock::new();
        let fs = FS.get_or_init(|| RwLock::new(cosmic_text::FontSystem::new()));
        let mut guard = fs.write().unwrap();
        with(&mut guard);
    }
    ce::set_font_system_call(install_fs);

    let key = ce::storage_key("vm-native-delta-test");
    ce::code_editor_dispose(&key);
    let config = ce::CodeEditorConfig::default();
    ce::code_editor(&key, &config);
    assert!(ce::code_editor_set_text(&key, "hello"));

    let (_result, out) = auto_lang_core::run_with_capture(
        r#"
let d = code_editor_delta("__code_editor_vm-native-delta-test")
print(d)
let d2 = code_editor_delta("__code_editor_vm-native-delta-test")
print(d2)
"#,
    )
    .unwrap();
    // First call drains the set_text full-replace delta; the second
    // call at the same watermark returns empty deltas (shape恒定).
    assert_eq!(
        out.lines().collect::<Vec<_>>(),
        vec![
            r#"{"revision":1,"deltas":[{"start":0,"end":0,"replacement":"hello"}]}"#,
            r#"{"revision":1,"deltas":[]}"#,
        ],
        "destructive read roundtrip: {out}"
    );
}

/// Plan 673 T-02: `code_editor_edit` end-to-end — structured write
/// (replace form) through the full pipeline; the queued delta carries
/// the exact caller interval in the same stream as set_text.
#[test]
fn vm_code_editor_edit_end_to_end() {
    // PLAN-814 T-04：随迁后测试进程不再链接 auto-lang 漏斗——provider 槽
    // 显式装配（r3 §5.3 显式优先；原 run 漏斗隐式安装面）。
    crate::install_ui_host_capabilities();
    use crate::ui::code_editor as ce;
    use std::sync::{OnceLock, RwLock};

    let _guard = crate::ui::code_editor::core::REGISTRY_TEST_LOCK
        .lock()
        .unwrap();

    fn install_fs(with: &mut dyn FnMut(&mut cosmic_text::FontSystem)) {
        static FS: OnceLock<RwLock<cosmic_text::FontSystem>> = OnceLock::new();
        let fs = FS.get_or_init(|| RwLock::new(cosmic_text::FontSystem::new()));
        let mut guard = fs.write().unwrap();
        with(&mut guard);
    }
    ce::set_font_system_call(install_fs);

    let key = ce::storage_key("vm-native-edit-test");
    ce::code_editor_dispose(&key);
    let config = ce::CodeEditorConfig::default();
    ce::code_editor(&key, &config);
    assert!(ce::code_editor_set_text(&key, "hello world"));

    let (_result, out) = auto_lang_core::run_with_capture(
        r#"
let d0 = code_editor_delta("__code_editor_vm-native-edit-test")
print(d0)
let ok = code_editor_edit("__code_editor_vm-native-edit-test", 0, 5, "goodbye")
print(ok)
let t = code_editor_text("__code_editor_vm-native-edit-test")
print(t)
let d = code_editor_delta("__code_editor_vm-native-edit-test")
print(d)
let bad = code_editor_edit("__code_editor_vm-native-edit-test", -1, 2, "x")
print(bad)
"#,
    )
    .unwrap();
    assert_eq!(
        out.lines().collect::<Vec<_>>(),
        vec![
            // set_text full-replace delta (revision 1).
            r#"{"revision":1,"deltas":[{"start":0,"end":0,"replacement":"hello world"}]}"#,
            "true",
            "goodbye world",
            // The edit's exact caller interval, next watermark (2).
            r#"{"revision":2,"deltas":[{"start":0,"end":5,"replacement":"goodbye"}]}"#,
            "false",
        ],
        "structured write roundtrip: {out}"
    );
}
