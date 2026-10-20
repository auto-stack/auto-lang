// PLAN-790: UI host clipboard media provider adapter（779/785 装配模式的
// files/image 剪贴板 2934..2937 批次）。
//
// 委托 `ui::clipboard_native`（Plan 485 CF_HDROP/DIBV5+PNG 桥）既有实现
// ——OS 语义原样：files_get 失败=空表、files_set/image_set 失败=false、
// image_get 无图像格式=None（值降级非 raise）。core（vm/native.rs shim
// 2934..2937）自本批次起不再直引 ui::clipboard_native，经
// `vm::host_capabilities::ClipboardMediaProvider` 装配回供值。
//
// 门控：本模块在 `ui/mod.rs` 挂 `windows × native-clipboard` 双门控（与
// 被委托的 Win32 桥函数同一双门控；`native-clipboard` 不蕴含 `ui`）。
// 非 Windows 不存在本 adapter——宿主入口的调用点带同形 cfg 不装配，
// VM 缺席 provider → G3 值降级（空表/false/null），**不把 UI 默认伪造为
// 可用 OS**（计划 §5）。临时 PNG 生命周期沿用既有 OS 策略：落 `%TEMP%`
// 不即删，返回路径在消费期可读。
//
// 注册点（与 779/785 相同的宿主入口集合，T-04）：
// - `install_ui_host_capabilities`（lib.rs 统一 installer）
// - `run_file_dynamic_ui_inner`（`auto run` UI 入口）
// - `VmBridge` 两构造（UI 组件桥）
// - auto CLI main（cfg ui-iced）

use auto_lang_core::vm::host_capabilities::ClipboardImage;
use std::sync::Arc;

/// Win32 桥的 provider 形态（无状态读写——与 `ui::clipboard_native`
/// 现实现一致）。
pub struct NativeClipboardMediaProvider;

impl auto_lang_core::vm::host_capabilities::ClipboardMediaProvider
    for NativeClipboardMediaProvider
{
    fn files_get(&self) -> Vec<std::path::PathBuf> {
        crate::ui::clipboard_native::clipboard_files_get()
            .into_iter()
            .map(std::path::PathBuf::from)
            .collect()
    }

    fn files_set(&self, paths: &[std::path::PathBuf]) -> bool {
        crate::ui::clipboard_native::clipboard_files_set(paths)
    }

    fn image_get(&self) -> Option<ClipboardImage> {
        crate::ui::clipboard_native::clipboard_image_get().map(|img| ClipboardImage {
            path: img.path,
            width: img.width,
            height: img.height,
        })
    }

    fn image_set(&self, png_path: &std::path::Path) -> bool {
        crate::ui::clipboard_native::clipboard_image_set(png_path)
    }
}

/// 装配 UI 兼容默认 clipboard media provider = Win32 adapter（**仅当
/// 进程默认槽为空时**安装——`set_..._if_absent` 同锁原子；优先关系与
/// 779 F779-R2-01 / 785 逐条相同）：
/// 1. 调用方**显式**注册（`set_process_default_clipboard_media_provider`）
///    恒胜——本兼容安装不覆盖；
/// 2. 空槽时补缺（windows+native-clipboard 的 ui 构建恢复迁移前 OS 接线）；
/// 3. 实例级 `AutoVM::set_clipboard_media_provider` 在 1/2 之上按 VM 独立
///    生效（构造期快照 + 实例覆盖，不受后续进程默认变化影响）。
pub fn install_process_default() {
    auto_lang_core::vm::host_capabilities::set_process_default_clipboard_media_provider_if_absent(
        Arc::new(NativeClipboardMediaProvider),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use auto_lang_core::vm::host_capabilities::ClipboardMediaProvider;
    use std::sync::Mutex;

    /// F779-R2-01 优先关系镜像锚（785 文本族同款纪律）：兼容安装不覆盖
    /// 显式默认；空槽才补缺；显式 set 恒可替换。Fake provider——不触碰
    /// 真实 OS 剪贴板（红线：测试不写用户剪贴板；真 OS 往返由
    /// clipboard_native 既有 plan485 测试与本计划 V04 定向档覆盖）。
    /// F-785-R1-01：本测试 clear/set/restore 写默认槽——须持共享隔离锁
    /// （crate 级 test_default_slot_lock），与缺席断言/桥装配测试互斥。
    #[test]
    fn compat_install_never_overrides_explicit_default() {
        struct Fake {
            files: Vec<std::path::PathBuf>,
            received: Mutex<Vec<Vec<std::path::PathBuf>>>,
        }
        impl ClipboardMediaProvider for Fake {
            fn files_get(&self) -> Vec<std::path::PathBuf> {
                self.files.clone()
            }
            fn files_set(&self, paths: &[std::path::PathBuf]) -> bool {
                self.received.lock().unwrap().push(paths.to_vec());
                true
            }
            fn image_get(&self) -> Option<ClipboardImage> {
                None
            }
            fn image_set(&self, _png_path: &std::path::Path) -> bool {
                false
            }
        }

        let _slot = auto_lang_core::vm::host_capabilities::test_default_slot_lock();
        let saved =
            auto_lang_core::vm::host_capabilities::process_default_clipboard_media_provider();

        // 空槽 → 补缺成功（仅入槽，不触 OS）。
        auto_lang_core::vm::host_capabilities::clear_process_default_clipboard_media_provider();
        install_process_default();
        assert!(
            auto_lang_core::vm::host_capabilities::process_default_clipboard_media_provider()
                .is_some(),
            "空槽时兼容默认须补缺"
        );

        // 显式默认 → 兼容安装不覆盖（sentinel 路径可区分）。
        let sentinel = std::path::PathBuf::from("C:\\plan790-explicit\\哨兵 🎉.png");
        auto_lang_core::vm::host_capabilities::set_process_default_clipboard_media_provider(
            Arc::new(Fake {
                files: vec![sentinel.clone()],
                received: Mutex::new(Vec::new()),
            }),
        );
        install_process_default();
        let cur = auto_lang_core::vm::host_capabilities::process_default_clipboard_media_provider()
            .unwrap();
        assert_eq!(
            ClipboardMediaProvider::files_get(&*cur),
            vec![sentinel],
            "兼容安装不得覆盖显式默认"
        );

        // 还原。
        match saved {
            Some(p) => {
                auto_lang_core::vm::host_capabilities::set_process_default_clipboard_media_provider(p)
            }
            None => auto_lang_core::vm::host_capabilities::clear_process_default_clipboard_media_provider(),
        }
    }
}
