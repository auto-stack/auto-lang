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

impl crate::vm::host_capabilities::FrameProvider for FrameBenchProvider {
    fn begin_ms(&self) -> i64 {
        crate::ui::frame_bench::frame_begin_ms()
    }
    fn present_ms(&self) -> i64 {
        crate::ui::frame_bench::frame_present_ms()
    }
}

/// 注册进程默认 frame provider = frame_bench adapter（幂等替换）。
///
/// 幂等性说明：重复注册替换为等价的无状态 adapter，无观测差异；已建
/// VM 实例的构造期快照不受影响（host_capabilities 快照语义）。
pub fn install_process_default() {
    crate::vm::host_capabilities::set_process_default_frame_provider(Arc::new(FrameBenchProvider));
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
            crate::vm::host_capabilities::FrameProvider::begin_ms(&p),
            crate::ui::frame_bench::frame_begin_ms()
        );
        assert_eq!(
            crate::vm::host_capabilities::FrameProvider::present_ms(&p),
            crate::ui::frame_bench::frame_present_ms()
        );
    }
}
