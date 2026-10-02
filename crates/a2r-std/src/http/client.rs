//! PLAN-724 T-03/T-04：共享 async HTTP 客户端内核（Rust 网络执行单源）。
//!
//! 两个 Rust facade —— 独立 [`crate::http`]（a2r 转译产物的历史面）与
//! auto_lang `a2r_std::http`（生成服务兼容面）—— 的**全部**网络执行经本
//! 内核；任何 facade 不得再持有 reqwest blocking client、spawn_blocking
//! 兜底、每请求线程或无界通道。
//!
//! 形态（PLAN-724 §2 冻结；与 VM 侧 PLAN-705/707 执行器同构、协议独立）：
//!
//! ```text
//! 消费者 runtime（generated #[tokio::main] / VM owner 线程 / 同步线程）
//!   │ execute(req).await          （async 上下文；丢弃 future = 取消）
//!   │ execute_blocking(req)       （同步桥接；仅允许阻塞的边界）
//!   ▼
//! 固定内核 runtime（OnceLock，multi-thread、线程数恒定）+ 复用连接 Client
//!   │ 队列许可(try, 队满即终结性错误) → 活跃许可(await) → 总期限包裹
//!   ▼ reqwest async 建立/读体（增量 body 预算）
//! owned HttpResponse { status, headers, body ≤ body_limit }
//!
//! open_stream(spec)（非阻塞）
//!   │ 流队列许可(try) → spawn 生产者 → 流活跃许可(await) → 建立(10s)
//!   ▼ 读循环(idle 60s/chunk)：raw=Utf8Carry→≤16KiB / sse=SseDecoder→事件
//!   ▼ 有界队列(≤16 条 × ≤256KiB) + ready/space Notify（背压逐项生产）
//! 消费者 stream.next().await → Data / Eof / Failed  （单次终结）
//! close()/Drop → finalize(Cancelled) + abort 生产者（幂等、不复活）
//! ```
//!
//! 取消语义：非流式 = 调用方丢弃 `execute` 返回的 future → oneshot 关闭 →
//! 内核 job 在下一 await 点（许可等待/建立/读体）退出并归还许可；流 =
//! close()/Drop/scope 回收 → finalize + abort，绝不只丢 receiver。
//! 同步桥接是**响亮边界**：在 async 上下文调用 `*_blocking`/同步流 next
//! 会被 tokio 检测为嵌套执行而 panic——已知 async 上下文必须走 async 路径。

use std::sync::Arc;
use std::time::Duration;

use super::transfer::TransferLimits;

// ============================================================================
// 限额（默认对齐 705/707 交付值；env 首次使用时读取一次）
// ============================================================================

/// 非流式客户端限额。
#[derive(Debug, Clone, Copy)]
pub struct ClientLimits {
    /// 内核 runtime worker 线程数（固定，非每请求）。
    pub workers: usize,
    /// 活跃 job 上限（超出排队等待，等待计入总期限）。
    pub max_active: usize,
    /// 等待队列容量（队满 → 提交即终结性错误，绝不临时 spawn）。
    pub queue_capacity: usize,
    /// 单请求增量响应体预算（字节）。
    pub body_limit: usize,
    /// 单请求总期限（排队 + 建立 + 读体全程）。
    pub total_timeout: Duration,
}

/// 流式客户端限额。
#[derive(Debug, Clone, Copy)]
pub struct StreamLimits {
    /// 流活跃许可（独立于非流式配额）。
    pub max_active: usize,
    /// 流队列许可（提交即拒绝的上限）。
    pub queue_capacity: usize,
    /// 每流已排队条目上限（背压高水位）。
    pub max_queued_items: usize,
    /// 单条目（raw 文本块/SSE 事件 data）字节上限。
    pub max_item_bytes: usize,
    /// 建立期限（headers 送达）。
    pub open_timeout: Duration,
    /// 上游读空闲期限（每 chunk 重置；队满暂停读不计时）。
    pub idle_timeout: Duration,
}

impl Default for ClientLimits {
    fn default() -> Self {
        Self {
            workers: 2,
            max_active: 8,
            queue_capacity: 64,
            body_limit: 10 * 1024 * 1024,
            total_timeout: Duration::from_secs(30),
        }
    }
}

impl Default for StreamLimits {
    fn default() -> Self {
        Self {
            max_active: 16,
            queue_capacity: 32,
            max_queued_items: 16,
            max_item_bytes: 256 * 1024,
            open_timeout: Duration::from_secs(10),
            idle_timeout: Duration::from_secs(60),
        }
    }
}

fn env_usize(key: &str, default: usize) -> usize {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(default)
}

impl ClientLimits {
    fn from_env() -> Self {
        Self {
            workers: env_usize("AUTO_A2R_HTTP_WORKERS", 2).min(16),
            max_active: env_usize("AUTO_A2R_HTTP_MAX_ACTIVE", 8),
            queue_capacity: env_usize("AUTO_A2R_HTTP_QUEUE", 64),
            body_limit: env_usize("AUTO_A2R_HTTP_BODY_LIMIT", 10 * 1024 * 1024),
            total_timeout: Duration::from_millis(
                env_usize("AUTO_A2R_HTTP_TIMEOUT_MS", 30_000) as u64
            ),
        }
    }
}

impl StreamLimits {
    fn from_env() -> Self {
        Self {
            max_active: env_usize("AUTO_A2R_HTTP_STREAM_ACTIVE", 16),
            queue_capacity: env_usize("AUTO_A2R_HTTP_STREAM_QUEUE", 32),
            max_queued_items: env_usize("AUTO_A2R_HTTP_STREAM_MAX_QUEUED", 16),
            max_item_bytes: env_usize("AUTO_A2R_HTTP_STREAM_MAX_ITEM", 256 * 1024),
            open_timeout: Duration::from_millis(env_usize(
                "AUTO_A2R_HTTP_STREAM_OPEN_TIMEOUT_MS",
                10_000,
            ) as u64),
            idle_timeout: Duration::from_millis(env_usize(
                "AUTO_A2R_HTTP_STREAM_IDLE_TIMEOUT_MS",
                60_000,
            ) as u64),
        }
    }
}

/// raw 文本块切分上限（707 决策 D-10 同值）。
pub const RAW_CHUNK_BYTES: usize = 16 * 1024;

// ============================================================================
// 内核实例：固定 runtime + 复用连接 Client + 有界准入
// ============================================================================

/// 一个独立内核实例。全局单例（[`kernel_handle`] 等入口）服务两个 HTTP
/// facade；测试经 [`KernelInstance::new`] 构造独立小预算实例，不碰进程
/// env、不受全局初始化顺序影响（PLAN-724 §6 资源/计数隔离要求）。
///
/// PLAN-727：文件传输在实例内持**独立**许可组（`transfer_active/queue`）
/// 与受限 FS 并发（`fs_ops`）——大文件传输不占用普通 HTTP/SSE 配额。
pub struct KernelInstance {
    rt: tokio::runtime::Runtime,
    client: reqwest::Client,
    active: Arc<tokio::sync::Semaphore>,
    queue: Arc<tokio::sync::Semaphore>,
    limits: ClientLimits,
    stream_active: Arc<tokio::sync::Semaphore>,
    stream_queue: Arc<tokio::sync::Semaphore>,
    stream_limits: StreamLimits,
    transfer_active: Arc<tokio::sync::Semaphore>,
    transfer_queue: Arc<tokio::sync::Semaphore>,
    transfer_limits: TransferLimits,
    fs_ops: Arc<tokio::sync::Semaphore>,
}

impl KernelInstance {
    pub fn new(client_limits: ClientLimits, stream_limits: StreamLimits) -> Self {
        Self::new_with_transfers(client_limits, stream_limits, TransferLimits::default())
    }

    /// 带自定义文件传输限额的实例构造（PLAN-727 测试隔离入口）。
    pub fn new_with_transfers(
        client_limits: ClientLimits,
        stream_limits: StreamLimits,
        transfer_limits: TransferLimits,
    ) -> Self {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(client_limits.workers)
            .thread_name("a2r-http-kernel")
            .enable_all()
            .build()
            .expect("a2r http kernel runtime");
        // 复用连接的共享 Client：所有请求/流/传输同池，无每请求 Client。
        let client = reqwest::Client::builder()
            .build()
            .expect("a2r http kernel client");
        Self {
            rt,
            client,
            active: Arc::new(tokio::sync::Semaphore::new(client_limits.max_active)),
            queue: Arc::new(tokio::sync::Semaphore::new(client_limits.queue_capacity)),
            limits: client_limits,
            stream_active: Arc::new(tokio::sync::Semaphore::new(stream_limits.max_active)),
            stream_queue: Arc::new(tokio::sync::Semaphore::new(stream_limits.queue_capacity)),
            stream_limits,
            transfer_active: Arc::new(tokio::sync::Semaphore::new(transfer_limits.max_active)),
            transfer_queue: Arc::new(tokio::sync::Semaphore::new(transfer_limits.queue_capacity)),
            transfer_limits,
            fs_ops: Arc::new(tokio::sync::Semaphore::new(transfer_limits.fs_ops_max)),
        }
    }

    /// 内核 runtime 句柄（同步桥接用；async 消费者不得用其 block_on）。
    pub fn handle(&self) -> tokio::runtime::Handle {
        self.rt.handle().clone()
    }

    /// 非流式活跃许可探针（测试/资源报告：取消后须回基线）。
    pub fn active_available(&self) -> usize {
        self.active.available_permits()
    }

    /// 流活跃许可探针。
    pub fn stream_active_available(&self) -> usize {
        self.stream_active.available_permits()
    }

    /// 非流式队列许可探针（PLAN-727 排队取消测试用）。
    pub fn queue_available(&self) -> usize {
        self.queue.available_permits()
    }

    /// 传输活跃许可探针（PLAN-727 资源报告）。
    pub fn transfer_active_available(&self) -> usize {
        self.transfer_active.available_permits()
    }

    /// 传输队列许可探针。
    pub fn transfer_queue_available(&self) -> usize {
        self.transfer_queue.available_permits()
    }

    /// 受限文件操作许可探针。
    pub fn fs_ops_available(&self) -> usize {
        self.fs_ops.available_permits()
    }

    /// 在 async 上下文执行一个请求（见模块级拓扑）。
    ///
    /// PLAN-727 T-02 闭合：排队等待同样被调用方取消与总期限覆盖——
    /// 先等 active 许可的旧形态下，排队 job 既不响应丢弃 future 也不受
    /// 总期限约束（静态差距实证）。现在 select 三臂同权：
    /// `acquire(active)` / `tx.closed()`（调用方取消）/ `total`（自提交起）。
    pub async fn execute(&self, req: HttpRequest) -> Result<HttpResponse, ClientError> {
        let Ok(queue_permit) = Arc::clone(&self.queue).try_acquire_owned() else {
            return Err(ClientError::QueueFull);
        };
        let (mut tx, rx) = tokio::sync::oneshot::channel();
        let active = Arc::clone(&self.active);
        let client = self.client.clone();
        let limits = self.limits;
        self.rt.spawn(async move {
            let _queue_slot = queue_permit;
            // 总期限自 job 提交（排队起点）计时——等待计入总期限。
            let total = tokio::time::sleep(limits.total_timeout);
            tokio::pin!(total);
            let acquire = active.acquire_owned();
            tokio::pin!(acquire);
            let slot = tokio::select! {
                slot = &mut acquire => slot,
                // 取消传播（排队期）：调用方丢弃 rx → tx.closed() → 退出并
                // 归还队列/活跃许可。
                _ = tx.closed() => { return; }
                _ = &mut total => {
                    let _ = tx.send(Err(ClientError::Timeout));
                    return;
                }
            };
            let Ok(_active_slot) = slot else {
                return; // runtime 关闭：许可随 guard 释放，调用方拿 Cancelled
            };
            let work =
                tokio::time::timeout_at(total.deadline(), execute_owned(client, limits, req));
            tokio::pin!(work);
            // 取消传播（执行期）：调用方丢弃 rx → tx.closed() → work future
            // 在当前 await 点被丢弃（建立/读体随之停止），许可随任务退出归还。
            tokio::select! {
                r = &mut work => {
                    let result = match r {
                        Ok(inner) => inner,
                        Err(_) => Err(ClientError::Timeout),
                    };
                    let _ = tx.send(result);
                }
                _ = tx.closed() => {}
            }
        });
        match rx.await {
            Ok(r) => r,
            // tx 侧放弃只发生在 runtime 关闭/任务被外力 abort 时。
            Err(_) => Err(ClientError::Cancelled),
        }
    }

    /// 同步桥接：仅在允许阻塞的边界调用（见模块级拓扑）。async 上下文
    /// 调用会被 tokio 嵌套执行检测 panic——刻意的响亮诊断。
    pub fn execute_blocking(&self, req: HttpRequest) -> Result<HttpResponse, ClientError> {
        self.rt.handle().block_on(self.execute(req))
    }

    /// 非阻塞打开流（见模块级拓扑）。
    pub fn open_stream(&self, spec: StreamSpec) -> HttpClientStream {
        let shared = Arc::new(StreamShared {
            cell: std::sync::Mutex::new(StreamCell {
                state: StreamState::Opening,
                queue: std::collections::VecDeque::new(),
            }),
            ready: tokio::sync::Notify::new(),
            space: tokio::sync::Notify::new(),
            status: std::sync::Mutex::new(None),
            abort: std::sync::Mutex::new(None),
        });
        let Ok(queue_permit) = Arc::clone(&self.stream_queue).try_acquire_owned() else {
            finalize(
                &shared,
                StreamState::Failed("stream queue full".to_string()),
            );
            return HttpClientStream {
                shared,
                done: false,
            };
        };
        let active = Arc::clone(&self.stream_active);
        let client = self.client.clone();
        let limits = self.stream_limits;
        let task = self.rt.spawn(producer_task(
            shared.clone(),
            spec,
            limits,
            active,
            client,
            queue_permit,
        ));
        // 登记生产者 abort 句柄（spawn→登记窗口内的 close：finalize 已生效，
        // 生产者在首个终态检查点自行退出——abort 缺席不漏取消）。
        *shared.abort.lock().unwrap() = Some(task.abort_handle());
        HttpClientStream {
            shared,
            done: false,
        }
    }
}

static KERNEL: std::sync::OnceLock<KernelInstance> = std::sync::OnceLock::new();

fn kernel() -> &'static KernelInstance {
    KERNEL.get_or_init(|| {
        KernelInstance::new_with_transfers(
            ClientLimits::from_env(),
            StreamLimits::from_env(),
            TransferLimits::from_env(),
        )
    })
}

/// 内核 runtime 句柄（同步桥接用；async 消费者不得用其 block_on）。
pub fn kernel_handle() -> tokio::runtime::Handle {
    kernel().handle()
}

// ---- PLAN-727 全局传输子系统入口（transfer.rs 消费） ----

pub(crate) fn transfer_active() -> &'static Arc<tokio::sync::Semaphore> {
    &kernel().transfer_active
}

pub(crate) fn transfer_queue() -> &'static Arc<tokio::sync::Semaphore> {
    &kernel().transfer_queue
}

pub(crate) fn fs_ops() -> &'static Arc<tokio::sync::Semaphore> {
    &kernel().fs_ops
}

pub(crate) fn transfer_limits() -> TransferLimits {
    kernel().transfer_limits
}

pub(crate) fn shared_http_client() -> reqwest::Client {
    kernel().client.clone()
}

/// 非 streaming 响应体预算（builder multipart 面复用同一上限）。
pub(crate) fn kernel_response_body_limit() -> usize {
    kernel().limits.body_limit
}

// ============================================================================
// owned 请求/响应/错误
// ============================================================================

/// owned HTTP 请求描述（跨线程，禁止携带 AutoVM/AutoTask/借用）。
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<Vec<u8>>,
    /// 单请求超时（毫秒）；None 用内核总期限兜底。
    pub timeout_ms: Option<u64>,
}

impl HttpRequest {
    pub fn new(method: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            method: method.into(),
            url: url.into(),
            headers: Vec::new(),
            body: None,
            timeout_ms: None,
        }
    }
}

/// owned HTTP 响应：状态、headers 与 body 归属本请求（无线程局部关联）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl HttpResponse {
    /// 大小写无关 header 查找；缺失返回 ""。
    pub fn header_get(&self, key: &str) -> &str {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(key))
            .map(|(_, v)| v.as_str())
            .unwrap_or("")
    }
}

/// 内核终结性错误（不吞成空成功）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientError {
    /// 队列已满（提交即拒绝）。
    QueueFull,
    /// 总期限/单请求超时。
    Timeout,
    /// 响应体超预算。
    BodyTooLarge { limit: usize },
    /// 传输失败（连接/解析等）。
    Transport(String),
    /// 内核 runtime 已关闭（进程退出竞态）。
    ExecutorClosed,
    /// 调用方在结果交付前放弃（oneshot 关闭竞态）。
    Cancelled,
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientError::QueueFull => write!(f, "http client queue full"),
            ClientError::Timeout => write!(f, "http client timed out"),
            ClientError::BodyTooLarge { limit } => {
                write!(f, "http response body exceeds budget {limit}")
            }
            ClientError::Transport(e) => write!(f, "transport error: {e}"),
            ClientError::ExecutorClosed => write!(f, "http client executor closed"),
            ClientError::Cancelled => write!(f, "http client call cancelled"),
        }
    }
}

impl std::error::Error for ClientError {}

/// 增量读取响应体并强制预算：Content-Length 预检 + 分块累计超限即终结性
/// 错误（不把无限响应体读进内存）。
async fn read_body_capped(resp: reqwest::Response, cap: usize) -> Result<Vec<u8>, ClientError> {
    if let Some(len) = resp.content_length() {
        if len > cap as u64 {
            return Err(ClientError::BodyTooLarge { limit: cap });
        }
    }
    use futures::StreamExt;
    let mut out: Vec<u8> = Vec::new();
    let mut stream = resp.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| ClientError::Transport(format!("body read: {e}")))?;
        if out.len() + chunk.len() > cap {
            return Err(ClientError::BodyTooLarge { limit: cap });
        }
        out.extend_from_slice(&chunk);
    }
    Ok(out)
}

async fn execute_owned(
    client: reqwest::Client,
    limits: ClientLimits,
    req: HttpRequest,
) -> Result<HttpResponse, ClientError> {
    let method = reqwest::Method::from_bytes(req.method.to_ascii_uppercase().as_bytes())
        .map_err(|e| ClientError::Transport(format!("invalid method {}: {e}", req.method)))?;
    let mut builder = client.request(method, &req.url);
    for (k, v) in &req.headers {
        builder = builder.header(k.as_str(), v.as_str());
    }
    if let Some(ms) = req.timeout_ms {
        builder = builder.timeout(Duration::from_millis(ms));
    }
    if let Some(body) = &req.body {
        builder = builder.body(body.clone());
    }
    let resp = builder.send().await.map_err(|e| {
        if e.is_timeout() {
            ClientError::Timeout
        } else {
            ClientError::Transport(e.to_string())
        }
    })?;
    let status = resp.status().as_u16();
    let headers: Vec<(String, String)> = resp
        .headers()
        .iter()
        .filter_map(|(k, v)| Some((k.to_string(), v.to_str().ok()?.to_string())))
        .collect();
    let body = read_body_capped(resp, limits.body_limit).await?;
    Ok(HttpResponse {
        status,
        headers,
        body,
    })
}

// ============================================================================
// 非流式执行：async + 同步桥接（全局单例入口）
// ============================================================================

/// 在 async 上下文执行一个请求（网络在固定内核 runtime 上跑，本 future
/// 只做通知等待——current-thread 消费者 runtime 不会被阻塞）。
///
/// 队列已满 → 立即 [`ClientError::QueueFull`]（终结性，不排队不 spawn）。
/// **取消**：丢弃返回的 future → 内核 job 在下一 await 点退出并归还许可。
pub async fn execute(req: HttpRequest) -> Result<HttpResponse, ClientError> {
    kernel().execute(req).await
}

/// 同步桥接：仅在允许阻塞的边界（同步 facade 入口、非 runtime 线程）使用。
/// 在 async 上下文调用会 panic（tokio 嵌套执行检测）——这是刻意的响亮诊断，
/// 不做静默阻塞兜底；async 上下文请用 [`execute`]。
pub fn execute_blocking(req: HttpRequest) -> Result<HttpResponse, ClientError> {
    kernel().execute_blocking(req)
}

// ============================================================================
// headers 适配器（Auto JSON 形态 / 历史 Rust 行形态 → typed headers）
// ============================================================================

/// 解析 Auto 表面的 JSON headers（对象，值为字符串）。格式错误 = 终结性
/// 错误——不按空 headers 成功发射。
pub fn parse_json_headers(headers_json: &str) -> Result<Vec<(String, String)>, String> {
    let trimmed = headers_json.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    let value: serde_json::Value =
        serde_json::from_str(trimmed).map_err(|e| format!("invalid headers json: {e}"))?;
    let obj = value
        .as_object()
        .ok_or_else(|| "invalid headers json: expected object".to_string())?;
    let mut out = Vec::with_capacity(obj.len());
    for (k, v) in obj {
        let v = v
            .as_str()
            .ok_or_else(|| format!("invalid header value for {k:?}: expected string"))?;
        if k.trim().is_empty() {
            return Err("invalid header: empty name".to_string());
        }
        out.push((k.to_string(), v.to_string()));
    }
    Ok(out)
}

/// 解析历史 Rust facade 的行格式 headers（每行 `Key: Value`）。缺冒号或
/// 空键 = 终结性错误。
pub fn parse_line_headers(headers_text: &str) -> Result<Vec<(String, String)>, String> {
    let mut out = Vec::new();
    for line in headers_text.split('\n') {
        let line = line.trim_end_matches('\r').trim();
        if line.is_empty() {
            continue;
        }
        let Some((k, v)) = line.split_once(':') else {
            return Err(format!("invalid header line (missing ':'): {line:?}"));
        };
        let k = k.trim();
        if k.is_empty() {
            return Err(format!("invalid header line (empty name): {line:?}"));
        }
        out.push((k.to_string(), v.trim().to_string()));
    }
    Ok(out)
}

// ============================================================================
// 流式执行：typed 内核流（有界队列 / 背压 / 真取消 / 单次终结）
// ============================================================================

/// 流模式：raw 文本块 / SSE data 事件。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamMode {
    Raw,
    Sse,
}

/// 流打开请求的完整描述（owned）。
#[derive(Debug, Clone)]
pub struct StreamSpec {
    pub method: String,
    pub url: String,
    pub body: Option<Vec<u8>>,
    pub headers: Vec<(String, String)>,
    pub mode: StreamMode,
}

/// typed 流条目：数据 / EOF（排空后一次）/ 终结错误（一次）。
/// 控制串（`__status__` 等）不是内核状态——只在 facade 兼容层出现。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamItem {
    Data(String),
    Eof,
    Failed(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum StreamState {
    Opening,
    Pending,
    Eof,
    Failed(String),
    Cancelled,
}

impl StreamState {
    fn is_terminal(&self) -> bool {
        matches!(
            self,
            StreamState::Eof | StreamState::Failed(_) | StreamState::Cancelled
        )
    }
}

struct StreamCell {
    state: StreamState,
    queue: std::collections::VecDeque<String>,
}

struct StreamShared {
    cell: std::sync::Mutex<StreamCell>,
    /// 生产者 → 消费者：入队/终结通知。
    ready: tokio::sync::Notify,
    /// 消费者 → 生产者：队列腾位通知（背压恢复）。
    space: tokio::sync::Notify,
    /// 已建立后的响应状态（headers 元数据；typed，不走控制串）。
    status: std::sync::Mutex<Option<u16>>,
    /// 生产者 abort 句柄（生产者自装自卸；cancel 时取用）。
    abort: std::sync::Mutex<Option<tokio::task::AbortHandle>>,
}

/// typed HTTP 流句柄。Drop = close（幂等取消）。
pub struct HttpClientStream {
    shared: Arc<StreamShared>,
    /// 消费侧终结标记：Eof/Failed/Cancelled 已交付后 next 恒 None。
    done: bool,
}

fn finalize(shared: &StreamShared, terminal: StreamState) {
    {
        let mut cell = shared.cell.lock().unwrap();
        if cell.state.is_terminal() {
            return; // 单次终结：首个终态胜出
        }
        if matches!(terminal, StreamState::Cancelled) {
            cell.queue.clear(); // 取消释放未消费数据
        }
        cell.state = terminal;
    }
    shared.ready.notify_waiters();
}

/// 入队一条已解码条目（队满 → 等 space 背压；终态 → 迟到生产退出）。
async fn enqueue(shared: &StreamShared, limits: StreamLimits, item: String) -> bool {
    loop {
        let space_wait = shared.space.notified();
        tokio::pin!(space_wait);
        space_wait.as_mut().enable(); // 先登记兴趣再复查，覆盖丢唤醒窗口
        {
            let mut cell = shared.cell.lock().unwrap();
            if cell.state.is_terminal() {
                return false; // 迟到生产者：禁止重建
            }
            if cell.queue.len() < limits.max_queued_items {
                cell.queue.push_back(item);
                drop(cell);
                shared.ready.notify_waiters();
                return true;
            }
        }
        space_wait.await;
    }
}

impl HttpClientStream {
    /// 流是否已终结且队列排空（is_done 语义：不因上游写完而丢尾）。
    pub fn is_finished(&self) -> bool {
        let cell = self.shared.cell.lock().unwrap();
        cell.queue.is_empty() && cell.state.is_terminal()
    }

    /// 已排队条目数（测试/资源报告探针：背压高水位断言用）。
    pub fn queued_len(&self) -> usize {
        self.shared.cell.lock().unwrap().queue.len()
    }

    /// 终结错误查询：None = 未终结；Some(None) = EOF；Some(Some(e)) = 错误/
    /// 取消（707 同形）。
    pub fn terminal_error(&self) -> Option<Option<String>> {
        let cell = self.shared.cell.lock().unwrap();
        match &cell.state {
            StreamState::Eof => Some(None),
            StreamState::Failed(e) => Some(Some(e.clone())),
            StreamState::Cancelled => Some(Some("stream cancelled".to_string())),
            _ => None,
        }
    }

    /// 已建立流的响应状态（headers 未到 → None；非 2xx 也如实呈现）。
    pub fn status(&self) -> Option<u16> {
        *self.shared.status.lock().unwrap()
    }

    /// 等待下一条目。终结条目（Eof/Failed）只交付一次，其后恒 None；
    /// close/Drop 后恒 None。等待是纯通知等待——不占执行线程。
    pub async fn next(&mut self) -> Option<StreamItem> {
        if self.done {
            return None;
        }
        loop {
            let ready_wait = self.shared.ready.notified();
            tokio::pin!(ready_wait);
            ready_wait.as_mut().enable();
            {
                let mut cell = self.shared.cell.lock().unwrap();
                if let Some(item) = cell.queue.pop_front() {
                    drop(cell);
                    self.shared.space.notify_waiters();
                    return Some(StreamItem::Data(item));
                }
                match &cell.state {
                    StreamState::Opening | StreamState::Pending => {}
                    StreamState::Eof => {
                        self.done = true;
                        return Some(StreamItem::Eof);
                    }
                    StreamState::Failed(e) => {
                        self.done = true;
                        return Some(StreamItem::Failed(e.clone()));
                    }
                    StreamState::Cancelled => {
                        self.done = true;
                        return None;
                    }
                }
            }
            ready_wait.await;
        }
    }

    /// 幂等关闭：终结（Cancelled，丢弃未消费数据）+ abort 生产者。
    /// 连接回收、许可归还随生产者任务退出完成。
    pub fn close(&self) {
        finalize(&self.shared, StreamState::Cancelled);
        if let Some(h) = self.shared.abort.lock().unwrap().take() {
            h.abort();
        }
    }
}

impl Drop for HttpClientStream {
    fn drop(&mut self) {
        self.close();
    }
}

/// 非阻塞打开流（全局单例入口）：登记（Opening）→ 流队列许可（满 → 流以
/// Failed("queue full") 终结呈现，不 panic）→ spawn 生产者。
pub fn open_stream(spec: StreamSpec) -> HttpClientStream {
    kernel().open_stream(spec)
}

async fn producer_task(
    shared: Arc<StreamShared>,
    spec: StreamSpec,
    limits: StreamLimits,
    active: Arc<tokio::sync::Semaphore>,
    client: reqwest::Client,
    _queue_slot: tokio::sync::OwnedSemaphorePermit,
) {
    let _ = run_producer(&shared, spec, limits, active, client).await;
    // 无论自然终结/错误/取消/运行时关闭：卸载自己的 abort 句柄（cancel 侧
    // 已取走则 take 为空操作）。
    let _ = shared.abort.lock().unwrap().take();
}

async fn run_producer(
    shared: &Arc<StreamShared>,
    spec: StreamSpec,
    limits: StreamLimits,
    active: Arc<tokio::sync::Semaphore>,
    client: reqwest::Client,
) {
    let Ok(_active_slot) = active.acquire_owned().await else {
        finalize(
            shared,
            StreamState::Failed("stream executor closed".to_string()),
        );
        return;
    };
    if shared.cell.lock().unwrap().state.is_terminal() {
        return; // 准入等待期间被取消
    }
    let method = reqwest::Method::from_bytes(spec.method.to_ascii_uppercase().as_bytes())
        .unwrap_or(reqwest::Method::GET);
    let mut builder = client.request(method, &spec.url);
    for (k, v) in &spec.headers {
        builder = builder.header(k.as_str(), v.as_str());
    }
    if let Some(body) = &spec.body {
        builder = builder.body(body.clone());
    }
    let resp = match tokio::time::timeout(limits.open_timeout, builder.send()).await {
        Err(_) => {
            finalize(
                shared,
                StreamState::Failed(format!(
                    "stream open timed out after {:?}",
                    limits.open_timeout
                )),
            );
            return;
        }
        Ok(Err(e)) => {
            finalize(
                shared,
                StreamState::Failed(format!("stream open failed: {e}")),
            );
            return;
        }
        Ok(Ok(resp)) => resp,
    };
    // 状态元数据先行：非 2xx 也如实呈现（typed metadata，不经控制串）。
    *shared.status.lock().unwrap() = Some(resp.status().as_u16());
    // SSE 非 2xx → Failed；raw 保留读取非 2xx body 的可观察行为（707 同）。
    if spec.mode == StreamMode::Sse && resp.status().as_u16() >= 400 {
        finalize(
            shared,
            StreamState::Failed(format!("sse upstream status {}", resp.status().as_u16())),
        );
        return;
    }
    {
        let mut cell = shared.cell.lock().unwrap();
        if cell.state.is_terminal() {
            return; // 建立期间被取消
        }
        cell.state = StreamState::Pending;
    }
    shared.ready.notify_waiters();

    let mut decoder = crate::sse::SseDecoder::new();
    let mut carry = crate::sse::Utf8Carry::new();
    use futures::StreamExt;
    let mut upstream = resp.bytes_stream();
    loop {
        // 读空闲期限只包上游 read；队满背压等待在 enqueue 内、不计时。
        let chunk = match tokio::time::timeout(limits.idle_timeout, upstream.next()).await {
            Err(_) => {
                finalize(
                    shared,
                    StreamState::Failed("stream idle timeout".to_string()),
                );
                return;
            }
            Ok(None) => break, // 上游 EOF
            Ok(Some(Err(e))) => {
                finalize(
                    shared,
                    StreamState::Failed(format!("stream read error: {e}")),
                );
                return;
            }
            Ok(Some(Ok(bytes))) => bytes,
        };
        match spec.mode {
            StreamMode::Raw => {
                let text = carry.push(&chunk);
                // raw 单块上限 = min(16 KiB 默认, 实例 item 预算)——切分后
                // 结构性满足预算，无需逐块报错路径。
                let raw_chunk = RAW_CHUNK_BYTES.min(limits.max_item_bytes);
                for piece in split_utf8_chunks(&text, raw_chunk) {
                    if !enqueue(shared, limits, piece).await {
                        return; // 终态/取消
                    }
                }
            }
            StreamMode::Sse => {
                let mut events = Vec::new();
                match decoder.feed(&chunk, &mut events) {
                    Ok(()) => {}
                    Err(crate::sse::SseDecodeError::Budget { kind, limit }) => {
                        finalize(
                            shared,
                            StreamState::Failed(format!(
                                "sse decode budget exceeded ({kind} > {limit})"
                            )),
                        );
                        return;
                    }
                    Err(crate::sse::SseDecodeError::InvalidUtf8) => {
                        finalize(
                            shared,
                            StreamState::Failed("sse decode invalid utf-8".to_string()),
                        );
                        return;
                    }
                }
                for ev in events {
                    if ev.data.len() > limits.max_item_bytes {
                        finalize(
                            shared,
                            StreamState::Failed(format!(
                                "sse event exceeds budget {}",
                                limits.max_item_bytes
                            )),
                        );
                        return;
                    }
                    // 空 data 事件不分发由 decoder 保证。
                    if !enqueue(shared, limits, ev.data).await {
                        return;
                    }
                }
            }
        }
    }
    decoder.finish();
    finalize(shared, StreamState::Eof);
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
// 内核测试：独立小预算实例 + 回环 TCP stub（PLAN-724 §6 布局：内核单测
// 在本文件；串行 TCP 集成在 tests/http_client.rs）
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};

    fn small_limits() -> (ClientLimits, StreamLimits) {
        (
            ClientLimits {
                workers: 2,
                max_active: 2,
                queue_capacity: 4,
                body_limit: 64 * 1024,
                total_timeout: Duration::from_secs(5),
            },
            StreamLimits {
                max_active: 2,
                queue_capacity: 2,
                max_queued_items: 4,
                max_item_bytes: 4096,
                open_timeout: Duration::from_secs(2),
                idle_timeout: Duration::from_secs(2),
            },
        )
    }

    fn test_instance() -> &'static KernelInstance {
        let (c, s) = small_limits();
        Box::leak(Box::new(KernelInstance::new(c, s)))
    }

    /// 回环服务器：accept 一次 → 读请求头（ready 信号）→ 交 handler 处理
    /// 流 → 退出（done 信号）。断言用 [`wait_server_ready`]/
    /// [`join_server`] 有界等待（防测试挂死 + 保证关闭/取消发生在连接
    /// 建立之后——open/abort 竞态不产生悬空 accept）。
    fn spawn_server(
        handler: impl FnOnce(String, TcpStream) + Send + 'static,
    ) -> (
        u16,
        std::sync::mpsc::Receiver<()>,
        std::sync::mpsc::Receiver<()>,
    ) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let (ready_tx, ready_rx) = std::sync::mpsc::channel();
        let (done_tx, done_rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let head = read_request_head(&mut stream);
            let _ = ready_tx.send(());
            handler(head, stream);
            let _ = done_tx.send(());
        });
        (port, ready_rx, done_rx)
    }

    fn wait_server_ready(ready: &std::sync::mpsc::Receiver<()>) {
        ready
            .recv_timeout(Duration::from_secs(5))
            .expect("服务器 5s 内未收到请求（客户端未落线）");
    }

    fn join_server(done: std::sync::mpsc::Receiver<()>) {
        done.recv_timeout(Duration::from_secs(5))
            .expect("服务器线程 5s 内未结束（客户端断连未传导）");
    }

    fn read_request_head(stream: &mut TcpStream) -> String {
        let mut buf = [0u8; 8192];
        let n = stream.read(&mut buf).unwrap();
        String::from_utf8_lossy(&buf[..n]).to_string()
    }

    fn respond(stream: &mut TcpStream, status_line: &str, headers: &[(&str, &str)], body: &[u8]) {
        let mut resp = format!("{status_line}\r\n");
        for (k, v) in headers {
            resp.push_str(&format!("{k}: {v}\r\n"));
        }
        resp.push_str(&format!("Content-Length: {}\r\n\r\n", body.len()));
        stream.write_all(resp.as_bytes()).unwrap();
        stream.write_all(body).unwrap();
        let _ = stream.flush();
    }

    fn get(url: String) -> HttpRequest {
        HttpRequest::new("GET", url)
    }

    #[test]
    fn plan724_kernel_roundtrip_status_headers_body() {
        let inst = test_instance();
        let (port, _ready, server) = spawn_server(|head, mut s| {
            assert!(head.starts_with("GET /probe "), "request line: {head}");
            respond(
                &mut s,
                "HTTP/1.1 200 OK",
                &[("X-Test-Header", "Alpha"), ("content-type", "text/plain")],
                b"hello-kernel",
            );
        });
        let resp = inst
            .handle()
            .block_on(inst.execute(get(format!("http://127.0.0.1:{port}/probe"))));
        let resp = resp.expect("roundtrip ok");
        assert_eq!(resp.status, 200);
        assert_eq!(resp.body, b"hello-kernel");
        // 大小写无关 header 查找。
        assert_eq!(resp.header_get("x-test-header"), "Alpha");
        assert_eq!(resp.header_get("X-TEST-HEADER"), "Alpha");
        assert_eq!(resp.header_get("missing"), "");
        join_server(server);
    }

    #[test]
    fn plan724_kernel_500_status_and_body_preserved() {
        let inst = test_instance();
        let (port, _ready, server) = spawn_server(|_head, mut s| {
            respond(&mut s, "HTTP/1.1 500 Internal Server Error", &[], b"boom");
        });
        let resp = inst
            .handle()
            .block_on(inst.execute(get(format!("http://127.0.0.1:{port}/boom"))));
        let resp = resp.expect("非 2xx 是正常结果值，不是传输错误");
        assert_eq!(resp.status, 500);
        assert_eq!(resp.body, b"boom");
        join_server(server);
    }

    #[test]
    fn plan724_kernel_transport_failure_is_terminal_error() {
        let inst = test_instance();
        // 占住端口后立刻关闭 listener → 连接必被拒。
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        let err = inst
            .handle()
            .block_on(inst.execute(get(format!("http://127.0.0.1:{port}/dead"))));
        match err {
            Err(ClientError::Transport(msg)) => assert!(!msg.is_empty()),
            other => panic!("期望 Transport 终结错误，得到 {other:?}"),
        }
    }

    #[test]
    fn plan724_kernel_body_budget_observable_terminal() {
        let inst = test_instance();
        let big = vec![b'x'; 80 * 1024]; // > 64 KiB body_limit
        let (port, _ready, server) = spawn_server(move |_head, mut s| {
            respond(&mut s, "HTTP/1.1 200 OK", &[], &big);
        });
        let err = inst
            .handle()
            .block_on(inst.execute(get(format!("http://127.0.0.1:{port}/big"))));
        assert_eq!(err, Err(ClientError::BodyTooLarge { limit: 64 * 1024 }));
        join_server(server);
    }

    #[test]
    fn plan724_kernel_queue_full_is_immediate_terminal() {
        let inst = test_instance();
        // 不 accept 的服务器：进入 active 的 job 挂在响应等待，占住队列槽。
        let (port, _ready, server) = spawn_server(|_head, mut s| {
            // 阻塞读直到客户端断连（job 取消 → 连接关闭 → 本线程退出）。
            let _ = std::io::Read::read(&mut s, &mut [0u8; 1]);
        });
        let h = inst.handle();
        // active=2 + queue=4：前 6 个被接受，第 7 个 QueueFull。
        let mut inflight = Vec::new();
        for i in 0..6 {
            inflight.push(h.spawn(inst.execute(get(format!("http://127.0.0.1:{port}/hold{i}")))));
        }
        // 等 active 许可占满（确定性判据），队列槽随后被其余 job 占住。
        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        while inst.active_available() > 0 {
            assert!(std::time::Instant::now() < deadline, "active 许可未占满");
            std::thread::sleep(Duration::from_millis(10));
        }
        wait_server_ready(&_ready); // 首个 active job 的请求已落线
        let seventh = h.block_on(inst.execute(get(format!("http://127.0.0.1:{port}/overflow"))));
        assert_eq!(seventh, Err(ClientError::QueueFull));
        // 清理：取消全部在途 → 许可回基线。
        for j in inflight {
            j.abort();
        }
        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        while inst.active_available() < 2 {
            assert!(std::time::Instant::now() < deadline, "取消后许可未归还");
            std::thread::sleep(Duration::from_millis(10));
        }
        join_server(server);
    }

    #[test]
    fn plan724_kernel_cancel_dropped_future_stops_job_and_returns_permit() {
        let inst = test_instance();
        let (port, _ready, server) = spawn_server(|_head, mut s| {
            // 阻塞读直到客户端断连（job/流取消 → 连接关闭 → 本线程退出）。
            let _ = std::io::Read::read(&mut s, &mut [0u8; 1]);
        });
        let h = inst.handle();
        let caller = h.spawn(inst.execute(get(format!("http://127.0.0.1:{port}/cancel-me"))));
        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        while inst.active_available() > 1 {
            assert!(std::time::Instant::now() < deadline, "job 未进入 active");
            std::thread::sleep(Duration::from_millis(10));
        }
        wait_server_ready(&_ready); // 请求已落线，连接可被取消关闭
                                    // 调用方取消：丢弃等待 future → job 必须退出并归还许可。
        caller.abort();
        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        while inst.active_available() < 2 {
            assert!(std::time::Instant::now() < deadline, "取消后许可未归还");
            std::thread::sleep(Duration::from_millis(10));
        }
        join_server(server);
    }

    #[test]
    fn plan724_kernel_stream_raw_chunks_eof_and_status() {
        let inst = test_instance();
        let body = "x".repeat(40 * 1024); // > 16 KiB → 多块
        let body_clone = body.clone();
        let (port, _ready, server) = spawn_server(move |_head, mut s| {
            respond(&mut s, "HTTP/1.1 200 OK", &[], body_clone.as_bytes());
        });
        let mut stream = inst.open_stream(StreamSpec {
            method: "GET".into(),
            url: format!("http://127.0.0.1:{port}/raw"),
            body: None,
            headers: Vec::new(),
            mode: StreamMode::Raw,
        });
        let h = inst.handle();
        let mut received = Vec::new();
        loop {
            match h.block_on(stream.next()) {
                Some(StreamItem::Data(s)) => received.push(s),
                Some(StreamItem::Eof) => break,
                other => panic!("期望 Eof，得到 {other:?}"),
            }
        }
        assert_eq!(received.concat(), body, "raw 块顺序完整");
        assert!(received.iter().all(|p| p.len() <= RAW_CHUNK_BYTES));
        assert_eq!(stream.status(), Some(200));
        assert_eq!(stream.terminal_error(), Some(None));
        assert!(stream.is_finished());
        // 终结后 next 恒 None（单次终结）。
        assert!(h.block_on(stream.next()).is_none());
        join_server(server);
    }

    #[test]
    fn plan724_kernel_stream_sse_events_and_crlf() {
        let inst = test_instance();
        let (port, _ready, server) = spawn_server(|_head, mut s| {
            respond(
                &mut s,
                "HTTP/1.1 200 OK",
                &[("Content-Type", "text/event-stream")],
                b"data: a\r\ndata: b\r\n\r\ndata: [DONE]\n\n",
            );
        });
        let mut stream = inst.open_stream(StreamSpec {
            method: "GET".into(),
            url: format!("http://127.0.0.1:{port}/sse"),
            body: None,
            headers: Vec::new(),
            mode: StreamMode::Sse,
        });
        let h = inst.handle();
        let mut events = Vec::new();
        loop {
            match h.block_on(stream.next()) {
                Some(StreamItem::Data(s)) => events.push(s),
                Some(StreamItem::Eof) => break,
                other => panic!("期望 Eof，得到 {other:?}"),
            }
        }
        // CRLF 是一个终结符：a/b 同事件；[DONE] 是数据（707 子集）。
        assert_eq!(events, vec!["a\nb".to_string(), "[DONE]".to_string()]);
        assert_eq!(stream.status(), Some(200));
        join_server(server);
    }

    #[test]
    fn plan724_kernel_stream_sse_non2xx_fails_typed() {
        let inst = test_instance();
        let (port, _ready, server) = spawn_server(|_head, mut s| {
            respond(&mut s, "HTTP/1.1 404 Not Found", &[], b"nope");
        });
        let stream = inst.open_stream(StreamSpec {
            method: "GET".into(),
            url: format!("http://127.0.0.1:{port}/missing"),
            body: None,
            headers: Vec::new(),
            mode: StreamMode::Sse,
        });
        wait_server_ready(&_ready);
        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        while stream.terminal_error().is_none() {
            assert!(std::time::Instant::now() < deadline, "SSE 404 未终结");
            std::thread::sleep(Duration::from_millis(10));
        }
        assert_eq!(
            stream.terminal_error(),
            Some(Some("sse upstream status 404".to_string()))
        );
        assert_eq!(stream.status(), Some(404));
        join_server(server);
    }

    #[test]
    fn plan724_kernel_stream_close_cancels_producer_and_returns_permit() {
        let inst = test_instance();
        // 慢上游：headers 后持续滴流，永不到 EOF。
        let (port, _ready, server) = spawn_server(|_head, mut s| {
            s.write_all(
                b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nTransfer-Encoding: chunked\r\n\r\n",
            )
            .unwrap();
            let _ = s.flush();
            // 滴流 30s（进程退出回收；测试经 close 提前 abort 生产者）。
            for _ in 0..300 {
                if s.write_all(b"5\r\nabcde\r\n").is_err() {
                    break;
                }
                let _ = s.flush();
                std::thread::sleep(Duration::from_millis(100));
            }
        });
        let stream = inst.open_stream(StreamSpec {
            method: "GET".into(),
            url: format!("http://127.0.0.1:{port}/slow"),
            body: None,
            headers: Vec::new(),
            mode: StreamMode::Raw,
        });
        wait_server_ready(&_ready); // 慢上游已收到请求（headers 已写）
        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        while inst.stream_active_available() > 1 {
            assert!(
                std::time::Instant::now() < deadline,
                "流生产者未进入 active"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
        stream.close();
        assert_eq!(
            stream.terminal_error(),
            Some(Some("stream cancelled".to_string()))
        );
        // close 幂等。
        stream.close();
        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        while inst.stream_active_available() < 2 {
            assert!(std::time::Instant::now() < deadline, "取消后流许可未归还");
            std::thread::sleep(Duration::from_millis(10));
        }
        // close 后消费端终结（不复活）。
        let mut stream = stream;
        assert!(inst.handle().block_on(stream.next()).is_none());
        join_server(server);
    }

    #[test]
    fn plan724_kernel_stream_queue_full_fails_typed() {
        let inst = test_instance();
        let (port, _ready, server) = spawn_server(|_head, mut s| {
            // 阻塞读直到客户端断连（job/流取消 → 连接关闭 → 本线程退出）。
            let _ = std::io::Read::read(&mut s, &mut [0u8; 1]);
        });
        // stream_queue=2：占满两个槽，第三个 open 终结为 queue full。
        let s1 = inst.open_stream(StreamSpec {
            method: "GET".into(),
            url: format!("http://127.0.0.1:{port}/s1"),
            body: None,
            headers: Vec::new(),
            mode: StreamMode::Raw,
        });
        let s2 = inst.open_stream(StreamSpec {
            method: "GET".into(),
            url: format!("http://127.0.0.1:{port}/s2"),
            body: None,
            headers: Vec::new(),
            mode: StreamMode::Raw,
        });
        let s3 = inst.open_stream(StreamSpec {
            method: "GET".into(),
            url: format!("http://127.0.0.1:{port}/s3"),
            body: None,
            headers: Vec::new(),
            mode: StreamMode::Raw,
        });
        assert_eq!(
            s3.terminal_error(),
            Some(Some("stream queue full".to_string()))
        );
        wait_server_ready(&_ready); // s1/s2 生产者已建立并发出请求
        s1.close();
        s2.close();
        join_server(server);
    }

    #[test]
    fn plan724_kernel_stream_backpressure_high_watermark_bounded() {
        let inst = test_instance();
        let big = "y".repeat(100 * 1024); // ~7 块 > 队列容量 4
        let big_clone = big.clone();
        let (port, _ready, server) = spawn_server(move |_head, mut s| {
            respond(&mut s, "HTTP/1.1 200 OK", &[], big_clone.as_bytes());
        });
        let stream = inst.open_stream(StreamSpec {
            method: "GET".into(),
            url: format!("http://127.0.0.1:{port}/bp"),
            body: None,
            headers: Vec::new(),
            mode: StreamMode::Raw,
        });
        wait_server_ready(&_ready);
        // 消费者暂停：生产者最多排队 4 条（高水位钉界）。
        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        while stream.queued_len() < 4 {
            assert!(std::time::Instant::now() < deadline, "队列未达到高水位");
            std::thread::sleep(Duration::from_millis(5));
        }
        std::thread::sleep(Duration::from_millis(100));
        assert!(
            stream.queued_len() <= 4,
            "高水位超预算: {}",
            stream.queued_len()
        );
        // 恢复消费：顺序完整 + EOF。
        let mut stream = stream;
        let h = inst.handle();
        let mut received = Vec::new();
        loop {
            match h.block_on(stream.next()) {
                Some(StreamItem::Data(s)) => received.push(s),
                Some(StreamItem::Eof) => break,
                other => panic!("期望 Eof，得到 {other:?}"),
            }
        }
        assert_eq!(received.concat(), big);
        join_server(server);
    }

    #[test]
    fn plan724_kernel_stream_item_budget_terminal() {
        let inst = test_instance();
        // SSE 事件 data 超实例 item 预算（4096 < 8 KiB < decoder 256 KiB）
        // → 内核级预算终结可观察。
        let huge_event = format!(
            "data: {}

",
            "z".repeat(8 * 1024)
        );
        let (port, _ready, server) = spawn_server(move |_head, mut s| {
            respond(&mut s, "HTTP/1.1 200 OK", &[], huge_event.as_bytes());
        });
        let stream = inst.open_stream(StreamSpec {
            method: "GET".into(),
            url: format!("http://127.0.0.1:{port}/huge"),
            body: None,
            headers: Vec::new(),
            mode: StreamMode::Sse,
        });
        wait_server_ready(&_ready);
        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        while stream.terminal_error().is_none() {
            assert!(std::time::Instant::now() < deadline, "超预算流未终结");
            std::thread::sleep(Duration::from_millis(10));
        }
        match stream.terminal_error() {
            Some(Some(msg)) => assert!(msg.contains("budget"), "错误应可观察: {msg}"),
            other => panic!("期望预算终结，得到 {other:?}"),
        }
        join_server(server);
    }

    #[test]
    fn plan724_kernel_stream_open_timeout_terminal() {
        let inst = test_instance();
        // 不 accept：建立超时（2s）→ 终结为 Failed。
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        // listener 保留（未 accept）→ TCP 层可连入但 headers 永不送达。
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_secs(5));
            drop(listener);
        });
        let stream = inst.open_stream(StreamSpec {
            method: "GET".into(),
            url: format!("http://127.0.0.1:{port}/never"),
            body: None,
            headers: Vec::new(),
            mode: StreamMode::Raw,
        });
        let deadline = std::time::Instant::now() + Duration::from_secs(4);
        while stream.terminal_error().is_none() {
            assert!(std::time::Instant::now() < deadline, "建立超时未终结");
            std::thread::sleep(Duration::from_millis(20));
        }
        match stream.terminal_error() {
            Some(Some(msg)) => assert!(msg.contains("timed out"), "{msg}"),
            other => panic!("期望建立超时终结，得到 {other:?}"),
        }
    }

    #[test]
    fn plan724_kernel_headers_adapters_strict() {
        // JSON 形态（Auto 表面）。
        let h = parse_json_headers(r#"{"Authorization":"Bearer t","X-A":"1"}"#).unwrap();
        assert_eq!(h.len(), 2);
        assert!(
            parse_json_headers("not-json").is_err(),
            "坏 JSON 必须终结性报错"
        );
        assert!(
            parse_json_headers(r#"{"K":123}"#).is_err(),
            "非字符串值必须报错"
        );
        assert!(parse_json_headers("[]").is_err(), "非对象必须报错");
        assert!(parse_json_headers("").unwrap().is_empty());

        // 行形态（历史 Rust facade）。
        let h = parse_line_headers("K: v\r\nB: 2\n").unwrap();
        assert_eq!(h, vec![("K".into(), "v".into()), ("B".into(), "2".into())]);
        assert!(
            parse_line_headers("no-colon-line").is_err(),
            "缺冒号必须报错"
        );
        assert!(parse_line_headers(": value-only").is_err(), "空键必须报错");
        assert!(parse_line_headers("").unwrap().is_empty());
    }

    #[test]
    fn plan724_kernel_sync_bridge_works_on_plain_thread() {
        let inst = test_instance();
        let (port, _ready, server) = spawn_server(|_head, mut s| {
            respond(&mut s, "HTTP/1.1 200 OK", &[], b"bridge");
        });
        let resp = inst.execute_blocking(get(format!("http://127.0.0.1:{port}/bridge")));
        let resp = resp.expect("同步桥接在非 runtime 线程必须可用");
        assert_eq!(resp.body, b"bridge");
        join_server(server);
    }

    /// AC-02：current-thread 消费者 runtime 的 reactor 不被内核等待阻塞——
    /// 慢上游（500ms 响应）期间本地定时器持续触发（健康事件先于 gate 放行）。
    #[test]
    fn plan724_kernel_current_thread_consumer_reactor_stays_live() {
        let inst = test_instance();
        let (port, _ready, server) = spawn_server(|_head, mut s| {
            std::thread::sleep(Duration::from_millis(500)); // 慢建立
            respond(&mut s, "HTTP/1.1 200 OK", &[], b"slow");
        });
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let url = format!("http://127.0.0.1:{port}/slow-open");
        let ticks = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let ticks2 = std::sync::Arc::clone(&ticks);
        rt.block_on(async move {
            let ticker = tokio::spawn(async move {
                for _ in 0..50 {
                    tokio::time::sleep(Duration::from_millis(20)).await;
                    ticks2.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                }
            });
            let result = inst
                .execute(get(url))
                .await
                .expect("current-thread 消费者上的内核 async 请求");
            assert_eq!(result.body, b"slow");
            let ticks_during = ticks.load(std::sync::atomic::Ordering::SeqCst);
            ticker.abort();
            // 请求等待期间（≥500ms）reactor 持续调度定时器（≥10 tick）。
            assert!(
                ticks_during >= 10,
                "reactor 被阻塞：等待期间仅 {ticks_during} tick"
            );
        });
        join_server(server);
    }

    /// AC-02：current-thread 消费者上的流等待不阻塞 reactor（逐块让出）。
    #[test]
    fn plan724_kernel_current_thread_stream_wait_yields() {
        let inst = test_instance();
        let (port, _ready, server) = spawn_server(|_head, mut s| {
            // 分帧 drip：先给半截 body，停 300ms 再补全——next() 必须真实
            // 挂起等待，期间的 reactor 定时器 tick 是「不阻塞」的判据。
            s.write_all(
                b"HTTP/1.1 200 OK
Content-Length: 11

stream-",
            )
            .unwrap();
            let _ = s.flush();
            std::thread::sleep(Duration::from_millis(300));
            s.write_all(b"body").unwrap();
            let _ = s.flush();
        });
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let url = format!("http://127.0.0.1:{port}/s");
        let ticks = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let ticks2 = std::sync::Arc::clone(&ticks);
        rt.block_on(async move {
            let ticker = tokio::spawn(async move {
                for _ in 0..100 {
                    tokio::time::sleep(Duration::from_millis(20)).await;
                    ticks2.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                }
            });
            let mut stream = inst.open_stream(StreamSpec {
                method: "GET".into(),
                url,
                body: None,
                headers: Vec::new(),
                mode: StreamMode::Raw,
            });
            let mut got = String::new();
            loop {
                match stream.next().await {
                    Some(StreamItem::Data(s)) => got.push_str(&s),
                    Some(StreamItem::Eof) => break,
                    other => panic!("期望 Eof，得到 {other:?}"),
                }
            }
            assert_eq!(got, "stream-body");
            let ticks_during = ticks.load(std::sync::atomic::Ordering::SeqCst);
            ticker.abort();
            assert!(
                ticks_during >= 5,
                "流等待阻塞了 reactor（等待期间仅 {ticks_during} tick）"
            );
            stream.close();
        });
        join_server(server);
    }

    /// 同步桥接响亮边界：async 上下文内调用 *_blocking 必须 panic
    /// （tokio 嵌套执行检测）——不做静默阻塞兜底。
    #[test]
    fn plan724_kernel_sync_bridge_panics_loudly_in_async_ctx() {
        let inst = test_instance();
        let (port, _ready, server) = spawn_server(|_head, mut s| {
            respond(&mut s, "HTTP/1.1 200 OK", &[], b"x");
        });
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let url = format!("http://127.0.0.1:{port}/bridge-panic");
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            rt.block_on(async {
                // async 上下文内的同步桥接：设计上响亮失败。
                let _ = inst.execute_blocking(get(url));
            });
        }));
        assert!(
            result.is_err(),
            "async 上下文内的同步桥接必须 panic（响亮诊断），不得静默阻塞"
        );
        // 请求在 block_on 边界就被拒绝——连接从未建立，不 join（stub 线程
        // 阻塞在 accept，随测试进程回收）。
        drop(server);
    }

    #[test]
    fn plan724_kernel_split_utf8_chunks_boundary_safe() {
        let text = "你好世界".repeat(1000);
        let pieces = split_utf8_chunks(&text, RAW_CHUNK_BYTES);
        assert!(pieces.iter().all(|p| p.len() <= RAW_CHUNK_BYTES));
        assert_eq!(pieces.concat(), text);
        assert_eq!(split_utf8_chunks("abc", 16), vec!["abc".to_string()]);
    }

    // ========================================================================
    // PLAN-727 T-02：排队取消与准入期限闭合（active=1 gate 不开的复现）
    // ========================================================================

    /// active=1：job1 占住活跃许可且 gate 不放行；排队中的 job2 被调用方
    /// 取消（abort 等待 future）——必须在未获得活跃许可的情况下退出并归还
    /// 队列许可（旧形态下排队 job 不响应调用方取消，静态差距的运行时实证）。
    #[test]
    fn plan727_execute_queued_cancel_returns_permit_without_gate() {
        let inst = Box::leak(Box::new(KernelInstance::new(
            ClientLimits {
                workers: 2,
                max_active: 1,
                queue_capacity: 4,
                body_limit: 64 * 1024,
                total_timeout: Duration::from_secs(30),
            },
            StreamLimits::default(),
        )));
        // 服务器 accept 后只读不响应（gate 不开），客户端断连后退出。
        let (port, _ready, server) = spawn_server(|_head, mut s| {
            let _ = std::io::Read::read(&mut s, &mut [0u8; 1]);
        });
        let h = inst.handle();
        let job1 = h.spawn(inst.execute(get(format!("http://127.0.0.1:{port}/hold"))));
        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        while inst.active_available() > 0 {
            assert!(std::time::Instant::now() < deadline, "job1 未进入 active");
            std::thread::sleep(Duration::from_millis(10));
        }
        // job2 排队：job1 已占 1 个队列许可（基线 q=3），job2 外层入队后 q=2。
        let job2 = h.spawn(inst.execute(get(format!("http://127.0.0.1:{port}/queued"))));
        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        while inst.queue_available() > 2 {
            assert!(std::time::Instant::now() < deadline, "job2 未进入队列");
            std::thread::sleep(Duration::from_millis(10));
        }
        // 内核 job 已 spawn 并即将在 acquire 上排队（active=1 被 job1 占满，
        // 不可能前进一步）；给一次 poll 机会后取消。
        std::thread::sleep(Duration::from_millis(50));
        // 取消排队 job2：gate 未开也要退出并归还队列许可。
        job2.abort();
        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        while inst.queue_available() < 3 {
            assert!(
                std::time::Instant::now() < deadline,
                "排队取消后队列许可未归还 q={}",
                inst.queue_available()
            );
            std::thread::sleep(Duration::from_millis(10));
        }
        // 清理 job1：活跃许可回基线。
        job1.abort();
        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        while inst.active_available() < 1 {
            assert!(std::time::Instant::now() < deadline, "取消后活跃许可未归还");
            std::thread::sleep(Duration::from_millis(10));
        }
        join_server(server);
    }

    /// active=1 占满时，排队 job 的总期限自提交起覆盖排队等待——到期收到
    /// Timeout（旧形态排队 job 无期限约束）。
    #[test]
    fn plan727_execute_queued_deadline_bounded() {
        let inst = Box::leak(Box::new(KernelInstance::new(
            ClientLimits {
                workers: 2,
                max_active: 1,
                queue_capacity: 4,
                body_limit: 64 * 1024,
                total_timeout: Duration::from_millis(700),
            },
            StreamLimits::default(),
        )));
        // 服务器 accept job1 后保持连接（gate 不开）直到客户端断连。
        let (port, _ready, server) = spawn_server(|_head, mut s| {
            let _ = std::io::Read::read(&mut s, &mut [0u8; 1]);
        });
        let h = inst.handle();
        let job1 = h.spawn(inst.execute(get(format!("http://127.0.0.1:{port}/hold"))));
        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        while inst.active_available() > 0 {
            assert!(std::time::Instant::now() < deadline, "job1 未进入 active");
            std::thread::sleep(Duration::from_millis(10));
        }
        // job2 排队：总期限 700ms 到期必须以 Timeout 终结（等待计入期限）。
        let started = std::time::Instant::now();
        let job2 = h.block_on(inst.execute(get(format!("http://127.0.0.1:{port}/queued"))));
        assert_eq!(
            job2,
            Err(ClientError::Timeout),
            "排队 job 须在总期限到期时 Timeout"
        );
        let elapsed = started.elapsed();
        assert!(
            elapsed >= Duration::from_millis(600) && elapsed < Duration::from_secs(5),
            "排队期限耗时异常: {elapsed:?}"
        );
        job1.abort();
        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        while inst.active_available() < 1 {
            assert!(std::time::Instant::now() < deadline, "取消后活跃许可未归还");
            std::thread::sleep(Duration::from_millis(10));
        }
        join_server(server);
    }
}
