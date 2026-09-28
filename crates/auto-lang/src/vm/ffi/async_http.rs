//! PLAN-705 T-02: 统一 live-op 登记表（register → complete → take/cancel
//! 单次终结协议）+ 全局完成通知。
//!
//! 取代 `ASYNC_RESULTS` 的裸 `HashMap<Option<..>>` 形态。旧表的两个协议
//! 缺陷（本模块的立约动机，源码级确认见 docs/plans/reports/705-async-decision.md）：
//! 1. 完成端无条件 `map.insert` —— 取消后迟到的 worker 完成可**复活**
//!    条目（req_id 单调不复用，复活即永驻泄漏）；
//! 2. `drop_async_result` 裸 remove 与完成写无同步 —— "超时即删、泄漏
//!    归零"的强结论不成立。
//!
//! 协议（写进 SD-02 规范增量）：
//! - **register**：提交 worker **之前**登记 Pending（调用方顺序保证）；
//! - **complete**：仅 Pending → Completed 一次转变，返回 false 即迟到/
//!   重复完成被丢弃（已取消/已消费/缺席一律禁止 insert 重建）；
//! - **take**：仅消费 Completed；Pending → None（探测/重入不得删除
//!   live 令牌）；
//! - **cancel**：幂等终结入口（引擎 30s 超时臂、scope 取消、deadline）；
//! - **notify**：complete/cancel 后 `COMPLETION_NOTIFY.notify_waiters()`；
//!   owner loop 侧 `notified().enable()` → 检查 → await（spike 已证
//!   enable 先于检查即零丢唤醒、零固定间隔轮询）。
//!
//! 消费者兼容面：stdlib 的既有 helper（`async_http_result_ready` /
//! `check_async_http_result*` / `drop_async_result`）签名与语义逐字节保留，
//! 内部改指本表；`vm.futures`（External Future）不物理合并，仅经
//! `complete_external_future` 补完成通知（决策报告 §6）。

use std::collections::HashMap;

use super::stdlib::AsyncResult;

lazy_static::lazy_static! {
    /// 统一 live-op 表：req_id（单调，永不复用）→ 操作状态机。
    pub(crate) static ref LIVE_OPS: std::sync::Mutex<HashMap<u64, LiveOp>> =
        std::sync::Mutex::new(HashMap::new());
}

/// 全局完成通知：完成端（任意线程）`notify_waiters()`；owner loop / 测试
/// 泵用 `notified().enable()` → 检查 → await 三段式等待。
pub(crate) static COMPLETION_NOTIFY: tokio::sync::Notify = tokio::sync::Notify::const_new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OpState {
    /// 已登记、未完成（worker 执行中）。
    Pending,
    /// 结果已落表、待消费（take 的唯一可消费态）。
    Completed,
}

// LiveOp 不派生 Debug：payload（AsyncResult）无 Debug 形态，保持旧表约束。
pub(crate) struct LiveOp {
    pub state: OpState,
    pub result: Option<Result<AsyncResult, String>>,
}

/// 提交 worker **之前**登记 live 令牌。幂等（or 语义：已完成条目不覆盖）。
pub(crate) fn register_live_op(req_id: u64) {
    if let Ok(mut map) = LIVE_OPS.lock() {
        map.entry(req_id).or_insert_with(|| LiveOp {
            state: OpState::Pending,
            result: None,
        });
    }
}

/// 完成端唯一入口：只对仍 live（Pending）的令牌提交一次结果并通知。
/// 返回 `false` = 迟到/重复完成被丢弃（已取消/已消费/缺席 —— 禁止复活）。
pub(crate) fn complete_live_op(req_id: u64, result: Result<AsyncResult, String>) -> bool {
    let delivered = if let Ok(mut map) = LIVE_OPS.lock() {
        match map.get_mut(&req_id) {
            Some(op) if op.state == OpState::Pending => {
                op.state = OpState::Completed;
                op.result = Some(result);
                true
            }
            // 已取消/已消费/缺席 → 丢弃数据（含 drop 结果体），不重建。
            _ => false,
        }
    } else {
        false
    };
    if delivered {
        COMPLETION_NOTIFY.notify_waiters();
    }
    delivered
}

/// take：只消费 Completed（条目终结移除）；Pending → None（保持 live）。
pub(crate) fn take_live_op(req_id: u64) -> Option<Result<AsyncResult, String>> {
    if let Ok(mut map) = LIVE_OPS.lock() {
        let consumable = matches!(
            map.get(&req_id).map(|op| op.state),
            Some(OpState::Completed)
        );
        if consumable {
            return map.remove(&req_id).and_then(|op| op.result);
        }
    }
    None
}

/// 非夺取式就绪探测（UI resume 泵 / 引擎 drain 消费）：Completed → true，
/// Pending/缺席/已消费 → false。
pub(crate) fn live_op_ready(req_id: u64) -> bool {
    LIVE_OPS
        .lock()
        .ok()
        .and_then(|map| {
            map.get(&req_id)
                .map(|op| op.state == OpState::Completed)
        })
        .unwrap_or(false)
}

/// 引擎 wake source 5 的极性（条目缺席/已终结 → true，让 CALL_NAT 重入
/// 落入"无结果"错误 fallback 而非永久等待）：Pending → false，其余 → true。
pub(crate) fn live_op_ready_or_gone(req_id: u64) -> bool {
    LIVE_OPS
        .lock()
        .ok()
        .and_then(|map| map.get(&req_id).map(|op| op.state != OpState::Pending))
        .unwrap_or(true)
}

/// 幂等终结入口（引擎超时放弃 / scope 取消 / deadline）：Pending/Completed
/// 一律移除；此后的 worker 迟到完成经 [`complete_live_op`] 的 presence
/// 守卫被丢弃。取消同样通知——等它的 owner loop/parked 项需要醒来观察。
pub(crate) fn cancel_live_op(req_id: u64) {
    if let Ok(mut map) = LIVE_OPS.lock() {
        map.remove(&req_id);
    }
    COMPLETION_NOTIFY.notify_waiters();
}

/// 登记表是否仍持有该令牌（parked 不回收断言 / 泄漏探针）。
#[cfg(test)]
pub(crate) fn live_op_exists(req_id: u64) -> bool {
    LIVE_OPS.lock().ok().map(|map| map.contains_key(&req_id)).unwrap_or(false)
}

/// 登记表条目总数（资源回基线探针）。
#[cfg(test)]
pub(crate) fn live_op_count() -> usize {
    LIVE_OPS.lock().ok().map(|map| map.len()).unwrap_or(0)
}

/// External Future 的跨线程完成入口（不持 AutoVM——AutoVM/AutoTask/RC
/// 不跨线程）：写 state/result 后发全局完成通知。stdlib 的 test.delay /
/// future_all / future_race 等 native worker 用本函数取代直接字段写。
/// Engine 内部（owner 线程上）的 Internal future 完成不需要它——owner
/// 本线程写、本线程恢复，无跨线程唤醒面。
pub(crate) fn complete_external_future(
    future: &std::sync::Arc<std::sync::RwLock<crate::vm::engine::FutureValue>>,
    result: Result<auto_val::Value, String>,
) {
    {
        let mut f = future.write().unwrap();
        match result {
            Ok(v) => {
                f.result = Some(v);
                f.state = crate::vm::engine::FutureState::Ready;
            }
            Err(_) => {
                f.state = crate::vm::engine::FutureState::Failed;
            }
        }
    }
    COMPLETION_NOTIFY.notify_waiters();
}
