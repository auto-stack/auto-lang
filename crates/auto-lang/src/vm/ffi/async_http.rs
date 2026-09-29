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
        .map(|map| map.get(&req_id).map(|op| op.state != OpState::Pending).unwrap_or(true))
        .unwrap_or(true)
}

/// 幂等终结入口（引擎超时放弃 / scope 取消 / deadline）：Pending/Completed
/// 一律移除；此后的 worker 迟到完成经 [`complete_live_op`] 的 presence
/// 守卫被丢弃。取消同样通知——等它的 owner loop/parked 项需要醒来观察。
///
/// PLAN-707 T-02：取消升格为**实际停止执行体**——同时 abort 该 req_id 的
/// managed job future（`JOB_ABORTS` 有则取）。abort 在 job 的下一个 await
/// 点生效（许可等待/请求建立/重试退避 sleep/读体/队满发送等待），wrapper
/// task 整树丢弃 → queue/active 许可随 Drop 释放。缺省（无登记 future，
/// 如 detached/已终结）只删槽——幂等。
pub(crate) fn cancel_live_op(req_id: u64) {
    let handle = JOB_ABORTS
        .lock()
        .ok()
        .and_then(|mut map| map.remove(&req_id));
    if let Some(handle) = handle {
        handle.abort();
    }
    if let Ok(mut map) = LIVE_OPS.lock() {
        map.remove(&req_id);
    }
    COMPLETION_NOTIFY.notify_waiters();
}

/// PLAN-707 T-02：managed job 的 abort 句柄表（req_id → 任务句柄）。
/// `submit_client_job` 在 spawn 后登记，`cancel_live_op` 取出并 abort。
/// 迟到 cancel 竞态由插入后复查闭合（见 submit_client_job 注释）。
static JOB_ABORTS: std::sync::LazyLock<
    std::sync::Mutex<std::collections::HashMap<u64, tokio::task::AbortHandle>>,
> = std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

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

// ============================================================================
// PLAN-705 T-03: 固定 async 客户端执行器（非流式 JSON/handle/builder 三路）
// ============================================================================
//
// 旧形态的三宗罪（AC-03 面）：JSON 池满退化为每 job `std::thread::spawn`
// 兜底；handle/auth/bearer/builder 每请求一个 detached 阻塞线程；无响应体
// 预算、无总期限上限。新形态：一个进程级固定 tokio runtime（线程数恒定），
// 提交经**有界队列**（队满立即终结性错误，绝不临时 spawn），执行经**活跃
// 许可**（超出排队等待），单 job 总期限 + 增量响应体预算兜底。取消 = 丢弃
// job future（重试退避的 tokio sleep 随之取消），迟到产物由 complete_live_op
// 的 presence 守卫吸收。

use std::time::Duration;

/// 客户端限额（决策报告 §4 冻结值；env 覆盖在首次使用时读取一次）。
#[derive(Debug, Clone, Copy)]
pub(crate) struct ClientLimits {
    /// 固定 runtime worker 线程数。
    pub workers: usize,
    /// 活跃 job 上限（超出排队等待）。
    pub max_active: usize,
    /// 等待队列容量（队满 → 提交即终结性错误）。
    pub queue_capacity: usize,
    /// 单 job 增量响应体预算（字节）。
    pub body_limit: usize,
    /// 单 job 总期限（发送 + 重试 + 读体全程）。
    pub total_timeout: Duration,
}

impl ClientLimits {
    fn from_env() -> Self {
        let env_usize = |key: &str, default: usize| {
            std::env::var(key)
                .ok()
                .and_then(|v| v.parse::<usize>().ok())
                .filter(|v| *v > 0)
                .unwrap_or(default)
        };
        Self {
            workers: env_usize("AUTO_HTTP_ASYNC_WORKERS", 2).min(16),
            max_active: env_usize("AUTO_HTTP_CLIENT_MAX_ACTIVE", 8),
            queue_capacity: env_usize("AUTO_HTTP_CLIENT_QUEUE", 64),
            body_limit: env_usize("AUTO_HTTP_CLIENT_BODY_LIMIT", 10 * 1024 * 1024),
            total_timeout: Duration::from_millis(
                env_usize("AUTO_HTTP_CLIENT_TIMEOUT_MS", 30_000) as u64,
            ),
        }
    }
}

struct ClientExecutor {
    limits: ClientLimits,
    rt: tokio::runtime::Runtime,
    active: std::sync::Arc<tokio::sync::Semaphore>,
    queue: std::sync::Arc<tokio::sync::Semaphore>,
}

static CLIENT_EXECUTOR: std::sync::OnceLock<ClientExecutor> = std::sync::OnceLock::new();

// 测试覆写限额（set_client_limits_for_test 在首次 executor() 前调用生效）；
// 非测试构建恒为 None，零成本。
static TEST_LIMITS: std::sync::Mutex<Option<ClientLimits>> = std::sync::Mutex::new(None);

/// 仅供测试：覆写客户端限额（首次 executor() 前调用生效）。
#[cfg(test)]
pub(crate) fn set_client_limits_for_test(limits: ClientLimits) {
    *TEST_LIMITS.lock().unwrap() = Some(limits);
}

fn executor() -> &'static ClientExecutor {
    CLIENT_EXECUTOR.get_or_init(|| {
        let mut limits = TEST_LIMITS.lock().unwrap().take();
        if limits.is_none() {
            limits = Some(ClientLimits::from_env());
        }
        let limits = limits.unwrap();
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(limits.workers)
            .thread_name("auto-http-async")
            .enable_all()
            .build()
            .expect("http async client runtime");
        ClientExecutor {
            rt,
            active: std::sync::Arc::new(tokio::sync::Semaphore::new(limits.max_active)),
            queue: std::sync::Arc::new(tokio::sync::Semaphore::new(limits.queue_capacity)),
            limits,
        }
    })
}

/// 客户端限额快照（响应体预算等消费方读取）。
pub(crate) fn client_limits() -> ClientLimits {
    executor().limits
}

/// PLAN-707 T-04：共享固定 client runtime 的句柄（流生产者与非流式 job
/// 同 runtime，无新增线程池——决策 D-2）。
pub(crate) fn client_runtime() -> tokio::runtime::Handle {
    executor().rt.handle().clone()
}

/// 提交一个**managed** 客户端 job 到固定 async runtime。
///
/// 队列已满 → 立即以终结性错误完成该 req_id（消费端拿到既有错误形态）并
/// 返回 `false`——**绝不临时 spawn 兜底线程**。job 的完成统一经
/// [`complete_live_op`]（presence 守卫拒迟到复活），总期限兜底由本函数
/// 包裹（job 自身的重试退避 sleep 在取消/超时时随 future 一起丢弃）。
///
/// PLAN-707 T-02：spawn 后登记 [`JOB_ABORTS`]（managed 取消面），随后复查
/// live-op 仍存在——spawn→登记窗口内的 `cancel_live_op` 抢先时，条目已
/// 消失 → 此处立即补 abort，cancel 先于 handle 安装不漏失。
pub(crate) fn submit_client_job(
    req_id: u64,
    job: impl std::future::Future<Output = Result<AsyncResult, String>> + Send + 'static,
) -> bool {
    let ex = executor();
    let Ok(queue_permit) = std::sync::Arc::clone(&ex.queue).try_acquire_owned() else {
        let _ = complete_live_op(req_id, Err("http client queue full".to_string()));
        return false;
    };
    let active = std::sync::Arc::clone(&ex.active);
    let limits = ex.limits;
    let task = ex.rt.spawn(async move {
        let _queue_slot = queue_permit;
        // 活跃许可：等待期间占队列槽（有限在途 + 有限等待队列）。
        let Ok(_active_slot) = active.acquire_owned().await else {
            let _ = complete_live_op(req_id, Err("http client executor closed".to_string()));
            JOB_ABORTS.lock().unwrap().remove(&req_id);
            return;
        };
        let outcome = tokio::time::timeout(limits.total_timeout, job).await;
        let result = match outcome {
            Ok(r) => r,
            Err(_) => Err(format!(
                "http client job timed out after {:?}",
                limits.total_timeout
            )),
        };
        let _ = complete_live_op(req_id, result);
        // PLAN-707 T-07：abort 句柄随 job 终结出表（与 cancel 路径双写幂等
        //）——否则每个成功 job 泄漏一个 AbortHandle（慢性增长）。
        JOB_ABORTS.lock().unwrap().remove(&req_id);
    });
    JOB_ABORTS
        .lock()
        .unwrap()
        .insert(req_id, task.abort_handle());
    // Cancel-before-install 闭合：spawn→insert 窗口内取消则条目已不在，
    // 立即补 abort（幂等：cancel_live_op 侧 remove 已发生，此处句柄仍在表）。
    let still_live = LIVE_OPS
        .lock()
        .map(|map| map.contains_key(&req_id))
        .unwrap_or(false);
    if !still_live {
        if let Some(h) = JOB_ABORTS.lock().unwrap().remove(&req_id) {
            h.abort();
        }
    }
    true
}

/// PLAN-707 T-02：提交一个 **detached** 客户端 job（fire-and-forget，如
/// 消息桥 `spawn_async_http_msg_get`）。
///
/// 与 managed 形态的显式边界：不写 live-op 表、不登记 [`JOB_ABORTS`]——
/// `cancel_live_op` 对其零影响（detached 提交形态不得因 LIVE_OPS 缺席被
/// 误杀）。executor 的 queue/active 许可与总期限同样适用；完成产物落
/// complete_live_op 时被 presence 守卫丢弃（无人在等）。
pub(crate) fn submit_detached_client_job(
    job: impl std::future::Future<Output = Result<AsyncResult, String>> + Send + 'static,
) -> bool {
    let ex = executor();
    let Ok(queue_permit) = std::sync::Arc::clone(&ex.queue).try_acquire_owned() else {
        return false;
    };
    let active = std::sync::Arc::clone(&ex.active);
    let limits = ex.limits;
    let req_id = crate::vm::ffi::stdlib::alloc_async_id();
    ex.rt.spawn(async move {
        let _queue_slot = queue_permit;
        let Ok(_active_slot) = active.acquire_owned().await else {
            return;
        };
        let outcome = tokio::time::timeout(limits.total_timeout, job).await;
        if let Ok(result) = outcome {
            // fire-and-forget：无 live-op 令牌，完成被 presence 守卫丢弃。
            let _ = complete_live_op(req_id, result);
        }
    });
    true
}

/// 增量读取响应体并强制预算：Content-Length 预检 + 分块累计超限即终结性
/// 错误（不把无限响应体读进内存）。
pub(crate) async fn read_body_capped(
    resp: reqwest::Response,
    cap: usize,
) -> Result<Vec<u8>, String> {
    if let Some(len) = resp.content_length() {
        if len > cap as u64 {
            return Err(format!(
                "http response body {len} exceeds budget {cap}"
            ));
        }
    }
    use futures::StreamExt;
    let mut out: Vec<u8> = Vec::new();
    let mut stream = resp.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("http response body read error: {e}"))?;
        if out.len() + chunk.len() > cap {
            return Err(format!("http response body exceeds budget {cap}"));
        }
        out.extend_from_slice(&chunk);
    }
    Ok(out)
}

// ============================================================================
// PLAN-707 T-02/T-07 资源观测面（测试专用）
// ============================================================================

/// 资源回基线探针：managed abort 登记数（job 完成不主动出表——abort 表
/// 与 live-op 表生命周期一致，cancel 时出表；泄漏探针=完成/取消后计数
/// 不随轮次增长）。
#[cfg(test)]
pub(crate) fn job_abort_count() -> usize {
    JOB_ABORTS.lock().map(|m| m.len()).unwrap_or(0)
}

/// 资源回基线探针：非流式 executor 的可用 active 许可（取消后必须归还）。
#[cfg(test)]
pub(crate) fn client_active_available() -> usize {
    executor().active.available_permits()
}
