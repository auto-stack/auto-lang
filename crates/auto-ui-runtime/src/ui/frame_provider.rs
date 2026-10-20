// PLAN-779: UI host frame provider adapter（Design 37 §4 "装配"层 UI 侧）。
//
// 读现有 `frame_bench` 通道（PLAN-716 组B——AUTO_FRAME_BENCH 门控零开销
// 纪律、未捕获=0、呈现=711 帧泵消费时刻代理；通道本身零改动）。core
// （vm/native.rs shim 9918/9919）自本 pilot 起不再直引 frame_bench——
// 本 adapter 经显式注册装配回去，行为等价：无泵进程两形（有无 provider）
// 读侧均 0；有泵进程（iced 渲染循环）读真实捕获值。
//
// 注册点（UI 所有受支持 VM 创建路径的上游，计划 AC-03/V-04）：
// - `VmBridge::new_with_children`（UI 组件桥——DynamicComponent/探针路径）
// - `run_file_dynamic_ui_inner`（`auto run` UI 入口；-q rqhost 客户端下游）
// - `auto` CLI main 启动（cfg ui-iced；覆盖 REPL/back_proxy 等同进程路径）

use std::sync::Arc;

/// frame_bench 通道的 provider 形态（无状态读侧——静态原子读）。
pub struct FrameBenchProvider;

impl auto_lang_core::vm::host_capabilities::FrameProvider for FrameBenchProvider {
    fn begin_ms(&self) -> i64 {
        crate::ui::frame_bench::frame_begin_ms()
    }
    fn present_ms(&self) -> i64 {
        crate::ui::frame_bench::frame_present_ms()
    }
}

/// 装配 UI 兼容默认 frame provider = frame_bench adapter（F779-R2-01：
/// **仅当进程默认槽为空时**安装——`set_..._if_absent` 同锁原子，检查与
/// 安装无并发窗口）。
///
/// 优先关系（装配契约）：
/// 1. 调用方**显式**注册（`set_process_default_frame_provider`，权威 API）
///    恒胜——本兼容安装不覆盖它；
/// 2. 无人注册时空槽由本兼容默认补缺（ui 构建下内建宿主入口恢复 779 前
///    的帧观测接线）；
/// 3. 实例级 `AutoVM::set_frame_provider` 在 1/2 之上按 VM 独立生效
///    （构造期快照 + 实例覆盖，均不受后续进程默认变化影响）。
/// 需要强制替换显式默认时，调用方使用显式 `set_` API 表达。
///
/// 本函数供全部内建隐式入口统一调用：VmBridge 构造、
/// `run_file_dynamic_ui_inner`、`install_ui_host_capabilities`（run 家族/
/// test/debug/ABT/config/REPL/解释器/back-proxy 漏斗）、auto CLI main。
pub fn install_process_default() {
    auto_lang_core::vm::host_capabilities::set_process_default_frame_provider_if_absent(Arc::new(
        FrameBenchProvider,
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// adapter 同源读：经 provider 接口读出值与 frame_bench 直读逐值相等
    /// （测试进程默认门关未捕获=0 形；门开真值形由 plan716 探针覆盖）。
    #[test]
    fn adapter_reads_same_source() {
        let p = FrameBenchProvider;
        assert_eq!(
            auto_lang_core::vm::host_capabilities::FrameProvider::begin_ms(&p),
            crate::ui::frame_bench::frame_begin_ms()
        );
        assert_eq!(
            auto_lang_core::vm::host_capabilities::FrameProvider::present_ms(&p),
            crate::ui::frame_bench::frame_present_ms()
        );
    }

    /// F779-R2-01 优先关系单元锚：兼容安装不覆盖显式默认；空槽才补缺；
    /// 显式 set 恒可替换（save/restore 串行，独占进程默认槽）。
    /// F-785-R1-01 修正：save/restore 写默认槽——改持 crate 级共享隔离锁
    ///（本地无锁与其他槽写/缺席断言测试同进程并行可互踩）。
    #[test]
    fn compat_install_never_overrides_explicit_default() {
        struct Fake(i64);
        impl auto_lang_core::vm::host_capabilities::FrameProvider for Fake {
            fn begin_ms(&self) -> i64 {
                self.0
            }
            fn present_ms(&self) -> i64 {
                self.0
            }
        }
        let _slot = auto_lang_core::vm::host_capabilities::test_default_slot_lock();
        let saved = auto_lang_core::vm::host_capabilities::process_default_frame_provider();

        // 空槽 → 补缺成功。
        auto_lang_core::vm::host_capabilities::clear_process_default_frame_provider();
        install_process_default();
        assert!(
            auto_lang_core::vm::host_capabilities::process_default_frame_provider().is_some(),
            "空槽时兼容默认须补缺"
        );

        // 显式默认 → 兼容安装不覆盖。
        auto_lang_core::vm::host_capabilities::set_process_default_frame_provider(Arc::new(Fake(
            17,
        )));
        install_process_default();
        let cur = auto_lang_core::vm::host_capabilities::process_default_frame_provider().unwrap();
        assert_eq!(
            auto_lang_core::vm::host_capabilities::FrameProvider::begin_ms(&*cur),
            17,
            "兼容安装不得覆盖显式默认"
        );

        // 还原。
        match saved {
            Some(p) => auto_lang_core::vm::host_capabilities::set_process_default_frame_provider(p),
            None => auto_lang_core::vm::host_capabilities::clear_process_default_frame_provider(),
        }
    }
}
