// PLAN-785: UI host clipboard text provider adapter（779 装配模式 UI 侧，
// 文本剪贴板 2925/2926 批次）。
//
// 委托 `ui::clipboard`（Plan 418 arboard 桥）既有实现——OS 语义原样：
// 读侧 None=不可用（shim 落空串）、写侧 false=失败；空剪贴板=Some("")。
// core（vm/native.rs shim 2925/2926）自本批次起不再直引 ui::clipboard，
// 经 `vm::host_capabilities::ClipboardTextProvider` 装配回供值。
//
// 注册点（与 779 frame adapter 相同的宿主入口集合，T-04）：
// - `install_ui_host_capabilities`（lib.rs 统一 installer——run 家族/测试/
//   调试/ABT/配置/REPL/解释器/back-proxy 漏斗 + autovm_persistent）
// - `run_file_dynamic_ui_inner`（`auto run` UI 入口）
// - `VmBridge` 两构造（UI 组件桥）
// - auto CLI main（cfg ui-iced）
//
// 门控：本模块挂 `ui-clipboard`（`ui/mod.rs`）——`{ui-clipboard, 无 ui}`
// 组合下 mod ui 不编译，无 adapter 可装（T01 矩阵登记的缺席错误形态）。

use std::sync::Arc;

/// arboard 桥的 provider 形态（无状态读写——每次调用新建 Clipboard 句柄，
/// 与 `ui::clipboard` 现实现一致）。
pub struct ArboardClipboardProvider;

impl auto_lang_core::vm::host_capabilities::ClipboardTextProvider for ArboardClipboardProvider {
    fn text(&self) -> Option<String> {
        crate::ui::clipboard::clipboard_get()
    }
    fn set_text(&self, text: &str) -> bool {
        crate::ui::clipboard::clipboard_set(text)
    }
}

/// 装配 UI 兼容默认 clipboard text provider = arboard adapter（**仅当
/// 进程默认槽为空时**安装——`set_..._if_absent` 同锁原子；优先关系与
/// 779 F779-R2-01 逐条相同）：
/// 1. 调用方**显式**注册（`set_process_default_clipboard_text_provider`）
///    恒胜——本兼容安装不覆盖；
/// 2. 空槽时补缺（ui 构建下内建宿主恢复迁移前 OS 接线）；
/// 3. 实例级 `AutoVM::set_clipboard_text_provider` 在 1/2 之上按 VM 独立
///    生效（构造期快照 + 实例覆盖，不受后续进程默认变化影响）。
pub fn install_process_default() {
    auto_lang_core::vm::host_capabilities::set_process_default_clipboard_text_provider_if_absent(
        Arc::new(ArboardClipboardProvider),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// F779-R2-01 优先关系镜像锚：兼容安装不覆盖显式默认；空槽才补缺；
    /// 显式 set 恒可替换（Fake provider——不触碰真实 OS 剪贴板；计划 §5
    /// 禁止随意改写用户剪贴板的验收步骤，真 OS 往返由 ui::clipboard 既有
    /// plan418 测试覆盖）。
    /// F-785-R1-01：本测试 clear/set/restore 写默认槽——须持共享隔离锁
    ///（crate 级 test_default_slot_lock），与缺席断言/桥装配测试互斥。
    #[test]
    fn compat_install_never_overrides_explicit_default() {
        struct Fake(Option<&'static str>, bool);
        impl auto_lang_core::vm::host_capabilities::ClipboardTextProvider for Fake {
            fn text(&self) -> Option<String> {
                self.0.map(|s| s.to_string())
            }
            fn set_text(&self, _text: &str) -> bool {
                self.1
            }
        }
        let _slot = auto_lang_core::vm::host_capabilities::test_default_slot_lock();
        let saved =
            auto_lang_core::vm::host_capabilities::process_default_clipboard_text_provider();

        // 空槽 → 补缺成功（装的是真 arboard adapter——仅入槽，不触 OS）。
        auto_lang_core::vm::host_capabilities::clear_process_default_clipboard_text_provider();
        install_process_default();
        assert!(
            auto_lang_core::vm::host_capabilities::process_default_clipboard_text_provider()
                .is_some(),
            "空槽时兼容默认须补缺"
        );

        // 显式默认 → 兼容安装不覆盖。
        auto_lang_core::vm::host_capabilities::set_process_default_clipboard_text_provider(
            Arc::new(Fake(Some("explicit-α"), false)),
        );
        install_process_default();
        let cur = auto_lang_core::vm::host_capabilities::process_default_clipboard_text_provider()
            .unwrap();
        assert_eq!(
            auto_lang_core::vm::host_capabilities::ClipboardTextProvider::text(&*cur).as_deref(),
            Some("explicit-α"),
            "兼容安装不得覆盖显式默认"
        );

        // 还原。
        match saved {
            Some(p) => {
                auto_lang_core::vm::host_capabilities::set_process_default_clipboard_text_provider(
                    p,
                )
            }
            None => {
                auto_lang_core::vm::host_capabilities::clear_process_default_clipboard_text_provider(
                )
            }
        }
    }
}
