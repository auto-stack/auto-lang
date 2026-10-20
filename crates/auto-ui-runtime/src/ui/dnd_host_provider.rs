// PLAN-804: UI host dnd provider adapter（779/785/790/794 装配模式的
// dnd 批次，2938）。
//
// 委托 `ui::native_dnd` 既有载荷 parser 与 Win32 会话实现（PLAN-488）：
// parse_payload_json（text/files/virtual_files、base64 与裸反斜杠重试）
// → win32::start_drag（**调用线程内联阻塞**的 DoDragDrop，自带消息泵；
// 完成效果经 DONE 通道 + on_dnd_finished 事件回发起方 App）——本 adapter
// 零状态零复制，不触碰 COM 会话/完成通道/会话代号，不复制第二个 parser。
// core（vm/native.rs shim 2938）自本批次起不再直引 ui::native_dnd，经
// `vm::host_capabilities::DndHostProvider` 装配回供值。
//
// 门控：本模块挂 `windows ∧ native-dnd` 双门控（模块本身在 ui 门内 =
// 有效三分 windows ∧ native-dnd ∧ ui，与迁移前 shim 门逐字相同——
// install 点在 lib.rs/ui 入点补第三重）。no-provider 形态（非 Windows /
// 未开 native-dnd / no-ui / 未装配）= shim 返 false（G5 值降级，与迁移
// 前 off 臂同形）。
//
// 注册点（与 779/785/790/794 相同的宿主入口集合）：
// - `install_ui_host_capabilities`（lib.rs 统一 installer，ui 块内）
// - `run_file_dynamic_ui_inner`（`auto run` UI 入口）
// - `VmBridge` 两构造（UI 组件桥）

use auto_lang_core::vm::host_capabilities::DndHostProvider;

/// 委托既有 ui::native_dnd 管线的 provider 形态（无状态）。
pub struct UiDndHostProvider;

impl DndHostProvider for UiDndHostProvider {
    /// owned JSON → parse_payload_json → win32::start_drag；坏 JSON =
    /// false（迁移前 shim 臂逐字语义）。阻塞形态保持：真 DoDragDrop 在
    /// 调用线程内联（不得 spawn/async 提前返 true——T-01 冻结）。
    fn start(&self, payload_json: String) -> bool {
        crate::ui::native_dnd::parse_payload_json(&payload_json)
            .map(crate::ui::native_dnd::win32::start_drag)
            .unwrap_or(false)
    }
}

/// 装配 UI 兼容默认 dnd host provider = ui::native_dnd adapter
///（**仅当进程默认槽为空时**安装——`set_..._if_absent` 同锁原子；优先
/// 关系与 779/785/790/794 逐条相同）：
/// 1. 调用方**显式**注册（`set_process_default_dnd_host_provider`）
///    恒胜——本兼容安装不覆盖；
/// 2. 空槽时补缺（windows+native-dnd+ui 构建下内建宿主入口恢复迁移前
///    拖出接线）；
/// 3. 实例级 `AutoVM::set_dnd_host_provider` 在 1/2 之上按 VM 独立生效。
pub fn install_process_default() {
    auto_lang_core::vm::host_capabilities::set_process_default_dnd_host_provider_if_absent(
        std::sync::Arc::new(UiDndHostProvider),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// F779-R2-01 优先关系镜像锚（同款纪律）：兼容安装不覆盖显式默认；
    /// 空槽才补缺。Fake provider——不触发真 COM/拖拽会话（真拖出证据 =
    /// V05 env 门控实机档）。F-785-R1-01：写默认槽须持 crate 级共享
    /// 隔离锁（test_default_slot_lock）。
    #[test]
    fn compat_install_never_overrides_explicit_default() {
        struct Fake(bool);
        impl DndHostProvider for Fake {
            fn start(&self, _payload_json: String) -> bool {
                self.0
            }
        }

        let _slot = auto_lang_core::vm::host_capabilities::test_default_slot_lock();
        let saved = auto_lang_core::vm::host_capabilities::process_default_dnd_host_provider();

        // 空槽 → 补缺成功（仅入槽；UiDndHostProvider 惰性——不调用即零
        // COM 面）。
        auto_lang_core::vm::host_capabilities::clear_process_default_dnd_host_provider();
        install_process_default();
        assert!(
            auto_lang_core::vm::host_capabilities::process_default_dnd_host_provider().is_some(),
            "空槽时兼容默认须补缺"
        );

        // 显式默认 → 兼容安装不覆盖（哨兵布尔可区分）。
        auto_lang_core::vm::host_capabilities::set_process_default_dnd_host_provider(
            std::sync::Arc::new(Fake(true)),
        );
        install_process_default();
        let cur =
            auto_lang_core::vm::host_capabilities::process_default_dnd_host_provider().unwrap();
        assert!(
            DndHostProvider::start(&*cur, "{}".to_string()),
            "兼容安装不得覆盖显式默认（空对象载荷应返回哨兵 true 而非走真 adapter 的 false）"
        );

        // 还原。
        match saved {
            Some(p) => {
                auto_lang_core::vm::host_capabilities::set_process_default_dnd_host_provider(p)
            }
            None => {
                auto_lang_core::vm::host_capabilities::clear_process_default_dnd_host_provider()
            }
        }
    }
}
