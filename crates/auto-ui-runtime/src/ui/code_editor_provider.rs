// PLAN-802: UI host code editor provider adapter（779/785/790/794 装配
// 模式的 code_editor 批次，22 native 全族：2910..2915、2919..2924、
// 2932/2933、2939、9906、9908..9913）。
//
// 委托 `ui::code_editor` 既有注册表（rope/undo/fold/光标/delta 队列仍是
// **单一真源**：本 adapter 只做 core 值域 ↔ UI 函数返回形转换，不复制
// 状态、不新建第二个 registry、不触碰编辑算法）。editor 寄宿滚动三项
// 委托 `editor_scroll_handle` + scroll controller 注册表投影/intent 队列
// （**不是** 794 的通用 scroll 参数协议——handle 约定单源
// `core::editor_scroll_handle`，701 供②b 契约原样）。文件 IO/OS 剪贴板
// 留 UI 实现侧（load/save 直调 core 函数；cut/copy/paste 经既有
// `code_editor_clipboard_op`，不复制系统桥或状态）。
//
// 门控：本模块挂 `code-editor` 单门控（与被委托的 ui::code_editor 模块
// 一致）。cut/copy/paste 的**实现级**第二门控 `ui-clipboard` 保留在方法
// 体内：无 `ui-clipboard` 时返回 `None`（shim 侧 raise 组合特征文案
// ——= 迁移前 code-editor on + ui-clipboard off 组合的行为，T-01 冻结）。
// no-ui 构建不存在本 adapter——VM 缺席 provider → T-01 冻结缺席臂。
//
// 注册点（与 779/785/790/794 相同的宿主入口集合，T-03）：
// - `install_ui_host_capabilities`（lib.rs 统一 installer——解释器/
//   back-proxy/autovm_persistent 漏斗经此）
// - `run_file_dynamic_ui_inner`（`auto run` UI 入口）
// - `VmBridge` 两构造（UI 组件桥）

use auto_lang_core::vm::host_capabilities::{
    CodeEditorCursorDto, CodeEditorHostProvider, CodeEditorScrollOffsetDto, EditorSnapshotProvider,
};
use std::sync::Arc;

/// editor 注册表的 provider 形态（无状态——委托 core 注册表 lazy 存储）。
pub struct CodeEditorRegistryProvider;

impl CodeEditorHostProvider for CodeEditorRegistryProvider {
    fn text(&self, key: &str) -> Option<String> {
        crate::ui::code_editor::code_editor_text(key)
    }

    fn delta(&self, key: &str) -> Option<String> {
        // destructive read 单次调用即恰一次队列消费（673 §4.2 单源）。
        crate::ui::code_editor::code_editor_delta(key)
    }

    fn edit(&self, key: &str, start: usize, end: usize, replacement: &str) -> bool {
        crate::ui::code_editor::code_editor_edit(key, start, end, replacement)
    }

    fn cursor(&self, key: &str) -> Option<CodeEditorCursorDto> {
        crate::ui::code_editor::code_editor_cursor(key).map(|(line, char_col, selection_bytes)| {
            CodeEditorCursorDto {
                line,
                char_col,
                selection_bytes,
            }
        })
    }

    fn find(&self, key: &str) -> bool {
        crate::ui::code_editor::code_editor_find(key)
    }

    fn set_text(&self, key: &str, text: &str) -> bool {
        crate::ui::code_editor::code_editor_set_text(key, text)
    }

    fn load_file(&self, key: &str, path: &str) -> Option<i64> {
        crate::ui::code_editor::code_editor_load_file(key, path)
    }

    fn save(&self, key: &str, path: &str) -> bool {
        crate::ui::code_editor::code_editor_save(key, path)
    }

    fn set_cursor(&self, key: &str, line: usize, char_col: usize) -> bool {
        crate::ui::code_editor::code_editor_set_cursor(key, line, char_col)
    }

    fn scroll_offset(&self, key: &str) -> CodeEditorScrollOffsetDto {
        // 701 供②b：读=注册表投影单源（editor 句柄经 core::editor_scroll_handle
        // 约定解析；未绑定 handle → 注册表 default 全零，非 None）。
        let handle = crate::ui::code_editor::editor_scroll_handle(key);
        let snap = crate::ui::scroll::controller::controller_snapshot(&handle);
        CodeEditorScrollOffsetDto {
            x: snap.offset_x,
            y: snap.offset_y,
        }
    }

    fn scroll_to(&self, key: &str, x: f64, y: f64) -> bool {
        // 701 供②b：写=intent 队列（双轴 ScrollTo、X 先 Y 后、Programmatic；
        // 未绑定 handle 由 renderer drain 的 unprimed 规则静默丢弃——恒
        // true=已入队，迁移前 shim 语义）。
        use crate::ui::scroll::{enqueue_intent, Axis, ScrollIntent, ScrollSource};
        let handle = crate::ui::code_editor::editor_scroll_handle(key);
        enqueue_intent(
            &handle,
            ScrollIntent::ScrollTo {
                axis: Axis::X,
                offset: x,
                source: ScrollSource::Programmatic,
            },
        );
        enqueue_intent(
            &handle,
            ScrollIntent::ScrollTo {
                axis: Axis::Y,
                offset: y,
                source: ScrollSource::Programmatic,
            },
        );
        true
    }

    fn fold_toggle(&self, key: &str, line_0: usize) -> bool {
        // 与迁移前 shim 体同形（code_editor_with 闭包；无编辑器 → false）。
        crate::ui::code_editor::code_editor_with(key, |core| core.fold_toggle(line_0))
            .unwrap_or(false)
    }

    fn fold_hidden_count(&self, key: &str) -> usize {
        crate::ui::code_editor::code_editor_with(key, |core| core.fold_hidden_count()).unwrap_or(0)
    }

    fn undo(&self, key: &str) -> bool {
        crate::ui::code_editor::code_editor_undo(key)
    }

    fn redo(&self, key: &str) -> bool {
        crate::ui::code_editor::code_editor_redo(key)
    }

    fn select_all(&self, key: &str) -> bool {
        crate::ui::code_editor::code_editor_select_all(key)
    }

    fn cut(&self, key: &str) -> Option<bool> {
        #[cfg(feature = "ui-clipboard")]
        {
            Some(crate::ui::code_editor::code_editor_clipboard_op(
                key,
                crate::ui::code_editor::ClipboardOp::Cut,
            ))
        }
        // 无 ui-clipboard：宿主无编辑器剪贴板面（shim raise 组合特征文案，
        // T-01 冻结——镜像迁移前 code-editor on + ui-clipboard off 行为）。
        #[cfg(not(feature = "ui-clipboard"))]
        {
            let _ = key;
            None
        }
    }

    fn copy(&self, key: &str) -> Option<bool> {
        #[cfg(feature = "ui-clipboard")]
        {
            Some(crate::ui::code_editor::code_editor_clipboard_op(
                key,
                crate::ui::code_editor::ClipboardOp::Copy,
            ))
        }
        #[cfg(not(feature = "ui-clipboard"))]
        {
            let _ = key;
            None
        }
    }

    fn paste(&self, key: &str) -> Option<bool> {
        #[cfg(feature = "ui-clipboard")]
        {
            Some(crate::ui::code_editor::code_editor_clipboard_op(
                key,
                crate::ui::code_editor::ClipboardOp::Paste,
            ))
        }
        #[cfg(not(feature = "ui-clipboard"))]
        {
            let _ = key;
            None
        }
    }
}

/// 装配 UI 兼容默认 code editor host provider = editor 注册表 adapter
///（**仅当进程默认槽为空时**安装——`set_..._if_absent` 同锁原子；优先
/// 关系与 779 F779-R2-01 / 785 / 790 / 794 逐条相同）：
/// 1. 调用方**显式**注册（`set_process_default_code_editor_host_provider`）
///    恒胜——本兼容安装不覆盖；
/// 2. 空槽时补缺（ui 构建下内建宿主入口恢复迁移前 editor 注册表接线）；
/// 3. 实例级 `AutoVM::set_code_editor_provider` 在 1/2 之上按 VM 独立
///    生效（构造期快照 + 实例覆盖，不受后续进程默认变化影响）。
pub fn install_process_default() {
    auto_lang_core::vm::host_capabilities::set_process_default_code_editor_host_provider_if_absent(
        Arc::new(CodeEditorRegistryProvider),
    );
    // PLAN-807: 同点位补装编辑器快照 adapter（独立槽 if-absent——与 22
    // native 槽互不掩盖；diff_snapshots 9916 经此供值）。
    auto_lang_core::vm::host_capabilities::set_process_default_editor_snapshot_provider_if_absent(
        Arc::new(EditorSnapshotRegistryProvider),
    );
}

/// PLAN-807: 编辑器快照 adapter——委托 ui::code_editor 既有注册表
/// （`code_editor_with(key, doc_snapshot)`，COW 冻结视图；注册表仍是
/// 单一真源，本 adapter 无状态）。读取期间编辑器被删/换不影响已取得
/// 的快照（COW 冻结语义由 rope 保证）。
struct EditorSnapshotRegistryProvider;

impl EditorSnapshotProvider for EditorSnapshotRegistryProvider {
    fn snapshot(&self, key: &str) -> Option<auto_text_core::RopeSnapshot> {
        crate::ui::code_editor::code_editor_with(key, |core| core.doc_snapshot())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// F779-R2-01 优先关系镜像锚（779/785/790/794 同款纪律）：兼容安装
    /// 不覆盖显式默认；空槽才补缺；显式 set 恒可替换。Fake provider——
    /// 不触碰全局 editor 注册表（真注册表往返由 plan802 in-crate 测试与
    /// fixture 覆盖）。F-785-R1-01：本测试 clear/set/restore 写默认槽——
    /// 须持 crate 级共享隔离锁（test_default_slot_lock），与缺席断言/
    /// 桥装配测试互斥。
    #[test]
    fn compat_install_never_overrides_explicit_default() {
        struct Fake(&'static str);
        impl CodeEditorHostProvider for Fake {
            fn text(&self, _key: &str) -> Option<String> {
                Some(self.0.to_string())
            }
            fn delta(&self, _key: &str) -> Option<String> {
                None
            }
            fn edit(&self, _key: &str, _s: usize, _e: usize, _r: &str) -> bool {
                false
            }
            fn cursor(&self, _key: &str) -> Option<CodeEditorCursorDto> {
                None
            }
            fn find(&self, _key: &str) -> bool {
                false
            }
            fn set_text(&self, _key: &str, _t: &str) -> bool {
                false
            }
            fn load_file(&self, _key: &str, _p: &str) -> Option<i64> {
                None
            }
            fn save(&self, _key: &str, _p: &str) -> bool {
                false
            }
            fn set_cursor(&self, _key: &str, _l: usize, _c: usize) -> bool {
                false
            }
            fn scroll_offset(&self, _key: &str) -> CodeEditorScrollOffsetDto {
                CodeEditorScrollOffsetDto::default()
            }
            fn scroll_to(&self, _k: &str, _x: f64, _y: f64) -> bool {
                false
            }
            fn fold_toggle(&self, _k: &str, _l: usize) -> bool {
                false
            }
            fn fold_hidden_count(&self, _k: &str) -> usize {
                0
            }
            fn undo(&self, _k: &str) -> bool {
                false
            }
            fn redo(&self, _k: &str) -> bool {
                false
            }
            fn select_all(&self, _k: &str) -> bool {
                false
            }
            fn cut(&self, _k: &str) -> Option<bool> {
                None
            }
            fn copy(&self, _k: &str) -> Option<bool> {
                None
            }
            fn paste(&self, _k: &str) -> Option<bool> {
                None
            }
        }

        let _slot = auto_lang_core::vm::host_capabilities::test_default_slot_lock();
        let saved =
            auto_lang_core::vm::host_capabilities::process_default_code_editor_host_provider();

        // 空槽 → 补缺成功（仅入槽，不触注册表）。
        auto_lang_core::vm::host_capabilities::clear_process_default_code_editor_host_provider();
        install_process_default();
        assert!(
            auto_lang_core::vm::host_capabilities::process_default_code_editor_host_provider()
                .is_some(),
            "空槽时兼容默认须补缺"
        );

        // 显式默认 → 兼容安装不覆盖（哨兵文本可区分）。
        auto_lang_core::vm::host_capabilities::set_process_default_code_editor_host_provider(
            Arc::new(Fake("@plan802-explicit:哨兵-🎉")),
        );
        install_process_default();
        let cur =
            auto_lang_core::vm::host_capabilities::process_default_code_editor_host_provider()
                .unwrap();
        assert_eq!(
            CodeEditorHostProvider::text(&*cur, "任意键"),
            Some("@plan802-explicit:哨兵-🎉".to_string()),
            "兼容安装不得覆盖显式默认"
        );

        // 还原。
        match saved {
            Some(p) => {
                auto_lang_core::vm::host_capabilities::set_process_default_code_editor_host_provider(p)
            }
            None => auto_lang_core::vm::host_capabilities::clear_process_default_code_editor_host_provider(),
        }
    }
}
