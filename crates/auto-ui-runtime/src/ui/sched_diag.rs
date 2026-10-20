// PLAN-721 T-1: 桌面轨更新路径家族诊断通道（P712-D1 承接——712-r2 T-16
// 定位记录的 trace 面归一）。
//
// 背景：动态 app 的恢复泵（`__parked_resume_tick`）与帧泵（`__frame_pump`）
// 消息在独立轨全链工作（712-r2 实测 51 条到达），桌面内嵌轨 r2 测得 0 条
// ——断链点（订阅装配门 / 消息路由 / 泵臂消费 / view_dirty 传播）待
// 双轨对照定案。本模块提供统一定时轴 + 门控打印，四类 trace 行：
// - `sub_parked` / `sub_frame`：订阅装配点（泵订阅 per-app 推入）
// - `msg`：dispatch_app 入口到达（泵族事件过滤，`__` 前缀）
// - `arm_parked` / `arm_frame`：update_inner 泵臂消费（到达 + 置脏传播）
//
// 门控纪律（AUTO_FRAME_BENCH / PLAN-005 先例）：`AUTO_SCHED_DIAG=1` 时
// 开启；未设 = 单 OnceLock 布尔读 + 分支，零写入零分配。

use std::sync::OnceLock;

/// AUTO_SCHED_DIAG=1 门（读一次定格；未设=关闭）。
pub fn enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("AUTO_SCHED_DIAG").ok().as_deref() == Some("1"))
}

/// 泵/恢复链路 trace 行。`app` 为 None 表示会话级（订阅装配汇总等）。
/// 时间基准复用 [`crate::ui::dynamic::sched_diag_t0`]（与既有 SCHED-DIAG
/// /MEMO-DIAG 行同轴对读）。
pub fn trace(app: Option<crate::ui::session::AppId>, phase: &str, detail: &str) {
    if enabled() {
        let t0 = crate::ui::dynamic::sched_diag_t0();
        match app {
            Some(id) => eprintln!(
                "[SCHED-DIAG] {} t={}ms app={:?} {}",
                phase,
                t0.elapsed().as_millis(),
                id,
                detail
            ),
            None => eprintln!(
                "[SCHED-DIAG] {} t={}ms {}",
                phase,
                t0.elapsed().as_millis(),
                detail
            ),
        }
    }
}
