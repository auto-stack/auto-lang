//! PLAN-707 T-04：外部 HTTP/SSE 流统一资源表与生产者（707-stream-decision
//! D-2/D-3/D-8/D-10 冻结形态）。
//!
//! 拓扑（与非流式 705 执行器共享固定 runtime，许可独立）：
//!
//! ```text
//! shim open（owner 线程，零阻塞）
//!   └─ 登记 STREAMS[id]=Opening → stream queue 许可(try) → spawn 生产者
//!        生产者（client runtime 线程，可 abort）：
//!          stream active 许可(await) → 建立请求(10s) → 读循环(idle 60s/chunk)
//!            raw: Utf8Carry → ≤16KiB 文本块；sse: SseDecoder → 事件 data
//!            → 有界队列(≤16 条) + ready/space Notify + COMPLETION_NOTIFY
//! 消费者（owner 线程）：
//!   stream_pull → Data / Pending(挂 waiting_http_stream_id) / Eof / Failed
//!   close/break/断连/scope 取消 → stream_finalize(Cancelled) → abort 生产者
//! ```
//!
//! 单次终结：finalize 幂等（首个终态胜出）；迟到生产者在 enqueue 前复查
//! 终态被拒；EOF 先排空已入队 data 再报终结。背压：队满时生产者停在
//! space 等待（读空闲期限只包上游 read，不被暂停读误触）。

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::sse::decoder::{SseDecodeError, SseDecoder, Utf8Carry};

use super::async_http::COMPLETION_NOTIFY;
use super::stdlib::alloc_async_id;

// ============================================================================
// 限额（决策 D-10；env 首次使用读一次）
// ============================================================================

#[derive(Debug, Clone, Copy)]
pub(crate) struct StreamLimits {
    /// 流 active 许可（独立于非流式配额）。
    pub max_active: usize,
    /// 流 queue 许可（提交即拒绝的上限）。
    pub queue_capacity: usize,
    /// 每流已排队事件/文本块上限。
    pub max_queued_items: usize,
    /// 单事件/文本块字节上限（raw 文本块另按 16 KiB 切）。
    pub max_item_bytes: usize,
    /// 建立（headers 送达）期限。
    pub open_timeout: Duration,
    /// 上游读空闲期限（每 chunk 重置；队满暂停读不计时）。
    pub idle_timeout: Duration,
}

impl StreamLimits {
    fn from_env() -> Self {
        let env_usize = |key: &str, default: usize| {
            std::env::var(key)
                .ok()
                .and_then(|v| v.parse::<usize>().ok())
                .filter(|v| *v > 0)
                .unwrap_or(default)
        };
        Self {
            max_active: env_usize("AUTO_HTTP_STREAM_ACTIVE", 16),
            queue_capacity: env_usize("AUTO_HTTP_STREAM_QUEUE", 32),
            max_queued_items: env_usize("AUTO_HTTP_STREAM_MAX_QUEUED", 16),
            max_item_bytes: env_usize("AUTO_HTTP_STREAM_MAX_EVENT", 256 * 1024),
            open_timeout: Duration::from_millis(env_usize(
                "AUTO_HTTP_STREAM_OPEN_TIMEOUT_MS",
                10_000,
            ) as u64),
            idle_timeout: Duration::from_millis(env_usize(
                "AUTO_HTTP_STREAM_IDLE_TIMEOUT_MS",
                60_000,
            ) as u64),
        }
    }
}

/// raw 文本块切分上限（决策 D-10）。
pub(crate) const RAW_CHUNK_BYTES: usize = 16 * 1024;

static STREAM_LIMITS: Mutex<Option<StreamLimits>> = Mutex::new(None);
static STREAM_EXECUTOR: std::sync::OnceLock<StreamExecutor> = std::sync::OnceLock::new();

/// 仅供测试：覆写流限额（首次 stream_executor() 前调用生效）。
#[cfg(test)]
pub(crate) fn set_stream_limits_for_test(limits: StreamLimits) {
    *STREAM_LIMITS.lock().unwrap() = Some(limits);
}

struct StreamExecutor {
    limits: StreamLimits,
    active: Arc<tokio::sync::Semaphore>,
    queue: Arc<tokio::sync::Semaphore>,
}

fn stream_executor() -> &'static StreamExecutor {
    STREAM_EXECUTOR.get_or_init(|| {
        let limits = STREAM_LIMITS
            .lock()
            .unwrap()
            .take()
            .unwrap_or_else(StreamLimits::from_env);
        StreamExecutor {
            limits,
            active: Arc::new(tokio::sync::Semaphore::new(limits.max_active)),
            queue: Arc::new(tokio::sync::Semaphore::new(limits.queue_capacity)),
        }
    })
}

/// 流限额快照（资源报告/测试读取）。
#[cfg(test)]
pub(crate) fn stream_limits() -> StreamLimits {
    stream_executor().limits
}

#[cfg(test)]
pub(crate) fn stream_active_available() -> usize {
    stream_executor().active.available_permits()
}

#[cfg(test)]
pub(crate) fn stream_live_count() -> usize {
    STREAMS.lock().map(|m| m.len()).unwrap_or(0)
}

// ============================================================================
// 资源表
// ============================================================================

/// 流状态机（D-3：Opening/Ready 可取/Pending/EOF/Failed/Cancelled）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum StreamState {
    /// 建立中（headers 未送达）。
    Opening,
    /// 已建立，等待下一个 chunk/事件。
    Pending,
    /// 流终结：数据已全部交付（排空队列后可见）。
    Eof,
    /// 流终结：错误（诊断信息）。
    Failed(String),
    /// 流终结：被取消（close/break/断连/scope 取消）。
    Cancelled,
}

pub(crate) struct StreamCell {
    state: StreamState,
    /// 已解码待交付条目（≤ max_queued_items；单条 ≤ max_item_bytes）。
    queue: std::collections::VecDeque<String>,
}

pub(crate) struct StreamHandle {
    cell: Mutex<StreamCell>,
    /// 生产者 → 消费者：入队/终结通知（enable→检查→await 三段式防丢）。
    ready: tokio::sync::Notify,
    /// 消费者 → 生产者：队列腾位通知（背压恢复）。
    space: tokio::sync::Notify,
    /// 幂等终结标记（finalize 竞态防护的第二道闸）。
    finalized: AtomicBool,
}

static STREAMS: std::sync::LazyLock<Mutex<HashMap<u64, Arc<StreamHandle>>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

/// 生产者 abort 句柄（stream_id → 任务句柄），先登记后复查闭合取消竞态。
static STREAM_ABORTS: std::sync::LazyLock<Mutex<HashMap<u64, tokio::task::AbortHandle>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

fn is_terminal(state: &StreamState) -> bool {
    matches!(
        state,
        StreamState::Eof | StreamState::Failed(_) | StreamState::Cancelled
    )
}

/// 消费侧拉取结果（D-4 typed pull）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Pull {
    Data(String),
    /// 已建立但暂无数据（消费方挂等待凭据）。
    Pending,
    /// 未建立完成（open 返回后 headers 未到；消费方同 Pending 处理）。
    Opening,
    Eof,
    Failed(String),
}

/// 幂等终结：首个终态胜出；Cancelled 释放未消费数据；唤醒全部等待者。
fn finalize(handle: &StreamHandle, terminal: StreamState) {
    {
        let mut cell = handle.cell.lock().unwrap();
        if is_terminal(&cell.state) {
            return;
        }
        if matches!(terminal, StreamState::Cancelled) {
            cell.queue.clear(); // 取消释放数据（决策 §5.2）
        }
        cell.state = terminal;
    }
    handle.finalized.store(true, Ordering::SeqCst);
    handle.ready.notify_waiters();
    COMPLETION_NOTIFY.notify_waiters();
}

/// 入队一条已解码条目（队满 → 等 space 背压；终态 → 迟到生产退出）。
async fn enqueue(handle: &Arc<StreamHandle>, limits: StreamLimits, item: String) -> bool {
    loop {
        let mut space_wait = handle.space.notified();
        tokio::pin!(space_wait);
        // 先登记兴趣再复查——覆盖"入队前刚被取走"的丢唤醒窗口。
        space_wait.as_mut().enable();
        {
            let mut cell = handle.cell.lock().unwrap();
            if is_terminal(&cell.state) {
                return false; // 迟到生产者：禁止 insert 重建
            }
            if cell.queue.len() < limits.max_queued_items {
                cell.queue.push_back(item);
                drop(cell);
                handle.ready.notify_waiters();
                COMPLETION_NOTIFY.notify_waiters();
                return true;
            }
        }
        space_wait.await;
    }
}

/// 消费者取走一条（EOF 前先排空队列——决策 §5.2）。
pub(crate) fn stream_pull(stream_id: u64) -> Pull {
    let handle = STREAMS.lock().ok().and_then(|m| m.get(&stream_id).cloned());
    let Some(handle) = handle else {
        // 条目已被显式 close/清理：消费端视为终结（不复活）。
        return Pull::Eof;
    };
    let mut cell = handle.cell.lock().unwrap();
    if let Some(item) = cell.queue.pop_front() {
        drop(cell);
        handle.space.notify_waiters();
        return Pull::Data(item);
    }
    match &cell.state {
        StreamState::Opening => Pull::Opening,
        StreamState::Pending => Pull::Pending,
        StreamState::Eof => Pull::Eof,
        StreamState::Failed(e) => Pull::Failed(e.clone()),
        StreamState::Cancelled => Pull::Eof,
    }
}

/// 非消费式就绪探测（ParkedWait::HttpStream / 泵扫描用）：
/// 有数据或已终结 → true；Opening/Pending 无数据 → false。
pub(crate) fn stream_ready(stream_id: u64) -> bool {
    let handle = STREAMS.lock().ok().and_then(|m| m.get(&stream_id).cloned());
    let Some(handle) = handle else {
        return true; // 条目消失 → 唤醒（pull 落 Eof 终结臂）
    };
    let cell = handle.cell.lock().unwrap();
    !cell.queue.is_empty() || is_terminal(&cell.state)
}

/// 资源观测探针（测试）：队列当前条数（背压钉界判据）。
#[cfg(test)]
pub(crate) fn stream_queue_len_for_test(stream_id: u64) -> usize {
    STREAMS
        .lock()
        .ok()
        .and_then(|m| m.get(&stream_id).cloned())
        .map(|h| h.cell.lock().map(|c| c.queue.len()).unwrap_or(0))
        .unwrap_or(0)
}

/// 流 id 是否存活（for-in 惰性消费的判定门——未知 id 不进流路径）。
pub(crate) fn stream_is_live(stream_id: u64) -> bool {
    STREAMS
        .lock()
        .map(|m| m.contains_key(&stream_id))
        .unwrap_or(false)
}

/// 非消费式终态查询（is_done / sse_error 用）。
pub(crate) fn stream_terminal_error(stream_id: u64) -> Option<Option<String>> {
    let handle = STREAMS
        .lock()
        .ok()
        .and_then(|m| m.get(&stream_id).cloned())?;
    let cell = handle.cell.lock().unwrap();
    match &cell.state {
        StreamState::Eof => Some(None),
        StreamState::Failed(e) => Some(Some(e.clone())),
        StreamState::Cancelled => Some(Some("stream cancelled".to_string())),
        _ => None,
    }
}

/// 显式关闭/消费退出/组取消的唯一收口：终结 + abort 生产者 + 出表。
pub(crate) fn stream_cancel(stream_id: u64) {
    let handle = STREAMS.lock().ok().and_then(|m| m.get(&stream_id).cloned());
    if let Some(handle) = handle {
        finalize(&handle, StreamState::Cancelled);
    }
    if let Some(h) = STREAM_ABORTS.lock().unwrap().remove(&stream_id) {
        h.abort();
    }
    if let Ok(mut map) = STREAMS.lock() {
        map.remove(&stream_id);
    }
}

/// 终结态流出表（close/清理用；不重复终结）。
pub(crate) fn stream_remove(stream_id: u64) {
    if let Some(h) = STREAM_ABORTS.lock().unwrap().remove(&stream_id) {
        h.abort();
    }
    if let Ok(mut map) = STREAMS.lock() {
        map.remove(&stream_id);
    }
}

// ============================================================================
// 打开 + 生产者
// ============================================================================

/// 流模式：raw 文本块 / SSE data 事件。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StreamMode {
    Raw,
    Sse,
}

/// 打开请求的完整描述（shim 组装；headers/body 原样透传）。
pub(crate) struct StreamOpen {
    pub url: String,
    pub method: String,
    pub body: Option<String>,
    pub headers: Vec<(String, String)>,
    pub mode: StreamMode,
}

/// 非阻塞打开：登记 STREAMS（Opening）→ 队列许可 → spawn 生产者。
/// 返回 stream_id（终态错误也不抛——消费端经 Pull::Failed 可见）。
pub(crate) fn open_stream(open: StreamOpen) -> u64 {
    let stream_id = alloc_async_id();
    let handle = Arc::new(StreamHandle {
        cell: Mutex::new(StreamCell {
            state: StreamState::Opening,
            queue: std::collections::VecDeque::new(),
        }),
        ready: tokio::sync::Notify::new(),
        space: tokio::sync::Notify::new(),
        finalized: AtomicBool::new(false),
    });
    if let Ok(mut map) = STREAMS.lock() {
        map.insert(stream_id, handle.clone());
    }
    let ex = stream_executor();
    let Ok(queue_permit) = Arc::clone(&ex.queue).try_acquire_owned() else {
        finalize(
            &handle,
            StreamState::Failed("stream queue full".to_string()),
        );
        return stream_id;
    };
    let limits = ex.limits;
    let active = Arc::clone(&ex.active);
    let rt = super::async_http::client_runtime();
    let task = rt.spawn(producer_task(
        stream_id,
        handle,
        open,
        limits,
        active,
        queue_permit,
    ));
    STREAM_ABORTS
        .lock()
        .unwrap()
        .insert(stream_id, task.abort_handle());
    // Cancel-before-install 闭合（同 submit_client_job D-1）：登记窗口内
    // 取消则条目已不在 → 补 abort。
    let still_live = STREAMS
        .lock()
        .map(|m| m.contains_key(&stream_id))
        .unwrap_or(false);
    if !still_live {
        if let Some(h) = STREAM_ABORTS.lock().unwrap().remove(&stream_id) {
            h.abort();
        }
    }
    stream_id
}

async fn producer_task(
    stream_id: u64,
    handle: Arc<StreamHandle>,
    open: StreamOpen,
    limits: StreamLimits,
    active: Arc<tokio::sync::Semaphore>,
    _queue_slot: tokio::sync::OwnedSemaphorePermit,
) {
    let Ok(_active_slot) = active.acquire_owned().await else {
        finalize(
            &handle,
            StreamState::Failed("stream executor closed".to_string()),
        );
        return;
    };
    // 建立：headers 送达期限 10s（D-10）。SSE 非 2xx → Failed（D-4）；
    // raw 保留读取非 2xx body 的既有可观察行为。
    let mut builder = match reqwest::Client::new().request(
        reqwest::Method::from_bytes(open.method.as_bytes()).unwrap_or(reqwest::Method::GET),
        &open.url,
    ) {
        b => b,
    };
    for (k, v) in &open.headers {
        builder = builder.header(k.as_str(), v.as_str());
    }
    if let Some(ref body) = open.body {
        builder = builder.body(body.clone());
    }
    let resp = match tokio::time::timeout(limits.open_timeout, builder.send()).await {
        Err(_) => {
            finalize(
                &handle,
                StreamState::Failed(format!(
                    "stream open timed out after {:?}",
                    limits.open_timeout
                )),
            );
            return;
        }
        Ok(Err(e)) => {
            finalize(
                &handle,
                StreamState::Failed(format!("stream open failed: {e}")),
            );
            return;
        }
        Ok(Ok(resp)) => resp,
    };
    if open.mode == StreamMode::Sse && resp.status().as_u16() >= 400 {
        finalize(
            &handle,
            StreamState::Failed(format!("sse upstream status {}", resp.status().as_u16())),
        );
        return;
    }
    // 已建立 → Pending（queue 有数据则状态仍以队列为准，pull 先出队）。
    {
        let mut cell = handle.cell.lock().unwrap();
        if is_terminal(&cell.state) {
            return; // 建立期间被取消
        }
        cell.state = StreamState::Pending;
    }
    COMPLETION_NOTIFY.notify_waiters();

    let mut decoder = SseDecoder::new();
    let mut carry = Utf8Carry::default();
    use futures::StreamExt;
    let mut upstream = resp.bytes_stream();
    loop {
        // 读空闲期限只包上游 read；队满背压等待发生在 enqueue 内、不计时
        //（决策 D-8/D-12：暂停读不误触读空闲）。
        let chunk = match tokio::time::timeout(limits.idle_timeout, upstream.next()).await {
            Err(_) => {
                finalize(
                    &handle,
                    StreamState::Failed("stream idle timeout".to_string()),
                );
                return;
            }
            Ok(None) => break, // 上游 EOF
            Ok(Some(Err(e))) => {
                finalize(
                    &handle,
                    StreamState::Failed(format!("stream read error: {e}")),
                );
                return;
            }
            Ok(Some(Ok(bytes))) => bytes,
        };
        match open.mode {
            StreamMode::Raw => {
                let text = carry.push(&chunk);
                for piece in split_utf8_chunks(&text, RAW_CHUNK_BYTES) {
                    if piece.len() > limits.max_item_bytes {
                        finalize(
                            &handle,
                            StreamState::Failed(format!(
                                "stream item exceeds budget {}",
                                limits.max_item_bytes
                            )),
                        );
                        return;
                    }
                    if !enqueue(&handle, limits, piece).await {
                        return; // 终态/取消
                    }
                }
            }
            StreamMode::Sse => {
                let mut events = Vec::new();
                match decoder.feed(&chunk, &mut events) {
                    Ok(()) => {}
                    Err(SseDecodeError::Budget { kind, limit }) => {
                        finalize(
                            &handle,
                            StreamState::Failed(format!(
                                "sse decode budget exceeded ({kind} > {limit})"
                            )),
                        );
                        return;
                    }
                    Err(SseDecodeError::InvalidUtf8) => {
                        finalize(
                            &handle,
                            StreamState::Failed("sse decode invalid utf-8".to_string()),
                        );
                        return;
                    }
                }
                for ev in events {
                    if ev.data.len() > limits.max_item_bytes {
                        finalize(
                            &handle,
                            StreamState::Failed(format!(
                                "sse event exceeds budget {}",
                                limits.max_item_bytes
                            )),
                        );
                        return;
                    }
                    // 空 data 事件不分发由 decoder 保证；此处 data 一定非空。
                    if !enqueue(&handle, limits, ev.data).await {
                        return;
                    }
                }
            }
        }
    }
    // 自然 EOF：先排空队列（pull 已保证），再置终态。
    decoder.finish();
    finalize(&handle, StreamState::Eof);
}

/// 按 ≤max 字节切文本（码点边界安全；max ≥ 4 字节时单码点必不超限）。
fn split_utf8_chunks(text: &str, max: usize) -> Vec<String> {
    if text.len() <= max {
        return vec![text.to_string()];
    }
    let mut out = Vec::new();
    let mut start = 0;
    while start < text.len() {
        let mut end = (start + max).min(text.len());
        while end < text.len() && !text.is_char_boundary(end) {
            end -= 1;
        }
        out.push(text[start..end].to_string());
        start = end;
    }
    out
}

// ============================================================================
// legacy 通道桥（AsyncStreamEvent 消费者兼容面）
// ============================================================================

/// 把 STREAMS 流桥接为 705 前形态的 AsyncStreamEvent 通道（Data/Done/Error），
/// 供 `Iterator::AsyncHttpStream`（sse_get_stream 的 for-in）与 sse_open/
/// sse_poll 的 try_recv 消费面使用（决策 D-4：语义逐字节保留）。
///
/// 泵在本模块的生产者协程上运行（共享 runtime）：rx drop（消费端清理）→
/// send 失败 → 泵退出并 stream_cancel（消费退出=上游回收的接入点，T-06
/// scope 组取消前的独立路径）。
pub(crate) fn spawn_legacy_channel_bridge(stream_id: u64) -> Arc<super::stdlib::AsyncStreamHandle> {
    let (tx, rx) = tokio::sync::mpsc::channel::<super::stdlib::AsyncStreamEvent>(64);
    let handle = Arc::new(super::stdlib::AsyncStreamHandle {
        rx: Mutex::new(rx),
        done: AtomicBool::new(false),
    });
    let done_flag = Arc::clone(&handle);
    let rt = super::async_http::client_runtime();
    rt.spawn(async move {
        loop {
            match stream_pull(stream_id) {
                Pull::Data(s) => {
                    if tx
                        .send(super::stdlib::AsyncStreamEvent::Data(s))
                        .await
                        .is_err()
                    {
                        stream_cancel(stream_id); // 消费端已丢 → 上游回收
                        return;
                    }
                    COMPLETION_NOTIFY.notify_waiters();
                }
                Pull::Pending | Pull::Opening => {
                    // enable → 检查 → await：零固定间隔轮询（决策 D-6）。
                    let stream_ready_fut = wait_stream_ready(stream_id);
                    if wait_stream_ready_or_cancel(stream_ready_fut)
                        .await
                        .is_none()
                    {
                        return;
                    }
                }
                Pull::Eof => {
                    let _ = tx.send(super::stdlib::AsyncStreamEvent::Done).await;
                    done_flag.done.store(true, Ordering::SeqCst);
                    COMPLETION_NOTIFY.notify_waiters();
                    return;
                }
                Pull::Failed(e) => {
                    let _ = tx.send(super::stdlib::AsyncStreamEvent::Error(e)).await;
                    let _ = tx.send(super::stdlib::AsyncStreamEvent::Done).await;
                    done_flag.done.store(true, Ordering::SeqCst);
                    COMPLETION_NOTIFY.notify_waiters();
                    return;
                }
            }
        }
    });
    handle
}

/// 流就绪等待（ready Notify + 全局完成通知双通道，enable 三段式防丢）。
/// T-05 generator 等待凭据化（next_sse_generator_value 的 Ok(None) 臂）。
pub(crate) async fn wait_stream_ready(stream_id: u64) {
    loop {
        let handle = STREAMS.lock().ok().and_then(|m| m.get(&stream_id).cloned());
        let Some(handle) = handle else { return };
        let ready_fut = handle.ready.notified();
        tokio::pin!(ready_fut);
        ready_fut.as_mut().enable();
        if stream_ready(stream_id) {
            return;
        }
        // 双等待：本流 ready / 全局完成通知（终结也经 COMPLETION_NOTIFY）。
        let global = COMPLETION_NOTIFY.notified();
        tokio::pin!(global);
        global.as_mut().enable();
        if stream_ready(stream_id) {
            return;
        }
        tokio::select! {
            _ = &mut ready_fut => {}
            _ = &mut global => {}
        }
    }
}

/// A bridged iterator consumes ASYNC_STREAMS, whose queue can be ready after
/// the bridge has drained STREAMS. Subscribe before checking that queue.
pub(crate) async fn wait_legacy_stream_ready(stream_id: u64) {
    loop {
        let notified = COMPLETION_NOTIFY.notified();
        tokio::pin!(notified);
        notified.as_mut().enable();
        let handle = super::stdlib::ASYNC_STREAMS
            .lock()
            .ok()
            .and_then(|streams| streams.get(&stream_id).cloned());
        let Some(handle) = handle else { return };
        if handle.done.load(Ordering::SeqCst)
            || handle
                .rx
                .lock()
                .map(|rx| !rx.is_empty() || rx.is_closed())
                .unwrap_or(true)
        {
            return;
        }
        notified.await;
    }
}

async fn wait_stream_ready_or_cancel(fut: impl std::future::Future<Output = ()>) -> Option<()> {
    // 被 abort 时此 await 直接丢弃 —— None 分支仅在未来自身返回时不可达，
    // 保留 Option 以便将来接入显式取消信号。
    fut.await;
    Some(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan738_bridge_wait_uses_consumer_queue_after_raw_queue_drained() {
        let id = alloc_async_id();
        let raw = Arc::new(StreamHandle {
            cell: Mutex::new(StreamCell {
                state: StreamState::Pending,
                queue: Default::default(),
            }),
            ready: Default::default(),
            space: Default::default(),
            finalized: AtomicBool::new(false),
        });
        STREAMS.lock().unwrap().insert(id, raw);
        let (tx, rx) = tokio::sync::mpsc::channel(4);
        let legacy = Arc::new(super::super::stdlib::AsyncStreamHandle {
            rx: Mutex::new(rx),
            done: AtomicBool::new(false),
        });
        super::super::stdlib::ASYNC_STREAMS
            .lock()
            .unwrap()
            .insert(id, legacy.clone());
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .unwrap();
        rt.block_on(async {
            tx.send(super::super::stdlib::AsyncStreamEvent::Data("alpha".into()))
                .await
                .unwrap();
            // The former credential blocks in this exact, valid bridge state.
            assert!(
                tokio::time::timeout(Duration::from_millis(30), wait_stream_ready(id))
                    .await
                    .is_err()
            );
            assert!(
                tokio::time::timeout(Duration::from_millis(30), wait_legacy_stream_ready(id))
                    .await
                    .is_ok()
            );
            legacy.rx.lock().unwrap().try_recv().unwrap();
            // Arrival after parking must also wake the consumer immediately.
            let arrival = async {
                tokio::task::yield_now().await;
                tx.send(super::super::stdlib::AsyncStreamEvent::Data("beta".into()))
                    .await
                    .unwrap();
                COMPLETION_NOTIFY.notify_waiters();
            };
            let wait =
                tokio::time::timeout(Duration::from_millis(100), wait_legacy_stream_ready(id));
            let (result, _) = tokio::join!(wait, arrival);
            assert!(result.is_ok());
        });
        super::super::stdlib::ASYNC_STREAMS
            .lock()
            .unwrap()
            .remove(&id);
        STREAMS.lock().unwrap().remove(&id);
    }

    #[test]
    fn plan707_client_split_utf8_chunks_boundary_safe() {
        let text = "你好世界".repeat(1000); // 12 KiB 码点密度
        let pieces = split_utf8_chunks(&text, RAW_CHUNK_BYTES);
        assert!(pieces.iter().all(|p| p.len() <= RAW_CHUNK_BYTES));
        assert_eq!(pieces.concat(), text);
        assert!(pieces.iter().all(|p| p.chars().next().is_some()));
        // 短文本单块。
        assert_eq!(split_utf8_chunks("abc", 16), vec!["abc".to_string()]);
    }
}
