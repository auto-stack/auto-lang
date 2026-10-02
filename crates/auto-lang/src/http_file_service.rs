//! PLAN-729 T-03：与 UI 无关的宿主文件响应执行（VM 默认 HTTP 与生成
//! Rust 服务共用的单源实现）。
//!
//! 输入是 [`a2r_std::http::server_file`] 的纯描述符/决策；本模块承担：
//! - **受限根目录打开**（决策报告 §2 冻结算法）：词法校验（描述符构造期
//!   完成）+ 逐段 no-follow walk（`symlink_metadata` 检出 symlink/junction，
//!   T-01 探针 P1b/P1c）+ 终段 `FILE_FLAG_OPEN_REPARSE_POINT`（Windows；
//!   P1e 证实对普通文件无害）+ 同句柄 metadata/seek/read（P4：路径替换不
//!   换文件；P5：EOF<承诺长度 = 发送失败）。
//! - **准入与排队**：active/queue 双信号量（排队期零句柄零缓冲，队满即时
//!   503）；从进入准备起的 30s 总期限（与调用方 deadline 取更早者）。
//! - **有界发送**：pump 任务拥有句柄，读驱动 ≤64KiB 单块 + 通道容量 1 =
//!   每 active ≤2 待发送块；fs op 并发 ≤4（信号量围每个读/seek）。
//! - **生命期**：body idle watchdog（60s 默认，独立计时任务覆盖不被 poll
//!   的黑洞客户端）+ 可选总期限；EOF/错误/Drop/断连（通道关闭）/超时/
//!   shutdown 恰一次收口（finish hook——VM 腿完成 scope，生成腿只回收
//!   本服务资源）。
//!
//! 版本无关：本模块**不含 axum 类型**——[`FileReply`] 是纯数据 + 自实现
//! `futures::Stream`（即 `futures_core::Stream` 再导出，0.7/0.8 两代 axum
//! 的 `Body::from_stream` 均可直接消费，生成 crate 无需新增命名依赖）。
//!
//! 排除面：不做目录列表/`Server.static`、multipart/byteranges、压缩、
//! TLS；不抵抗拥有 root 写权限的本地恶意用户（残余竞态见决策报告 §2.5）。

use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use a2r_std::http::server_file as proto;

// ============================================================================
// 配额/期限（决策报告 §5 冻结；env 首读一次）
// ============================================================================

/// 文件服务限额（独立于普通请求/SSE/传输配额）。
#[derive(Debug, Clone, Copy)]
pub struct FileServeLimits {
    /// 并发文件 body 上限（active）。
    pub max_active: usize,
    /// 等待上限（排队期零句柄零缓冲；满 → 即时 503）。
    pub queue_capacity: usize,
    /// 应用读块上限（字节）。
    pub app_block_bytes: usize,
    /// 每 active 待发送块上限（pump 读驱动：通道 1 + 在读 1 = 2）。
    pub max_pending_blocks: usize,
    /// 本服务并发文件操作上限（围每个 seek/read）。
    pub fs_ops_max: usize,
    /// 进入文件准备起（等许可+open+metadata+seek）总期限。
    pub prepare_timeout: Duration,
    /// body 空闲期限（两次活动之间；watchdog 独立计时）。
    pub body_idle_timeout: Duration,
    /// body 总期限（None = 不限；显式 env 0 = 不限，解析失败回默认）。
    pub body_total_timeout: Option<Duration>,
}

impl Default for FileServeLimits {
    fn default() -> Self {
        Self {
            max_active: 4,
            queue_capacity: 16,
            app_block_bytes: 64 * 1024,
            max_pending_blocks: 2,
            fs_ops_max: 4,
            prepare_timeout: Duration::from_secs(30),
            body_idle_timeout: Duration::from_secs(60),
            body_total_timeout: None,
        }
    }
}

fn env_usize(key: &str, default: usize) -> usize {
    std::env::var(key)
        .ok()
        .and_then(|v| v.trim().parse::<usize>().ok())
        .filter(|v| *v > 0) // 0/非法 → 回默认（不把 0 当无限）
        .unwrap_or(default)
}

fn env_ms(key: &str, default_ms: u64) -> Duration {
    Duration::from_millis(env_usize(key, default_ms as usize) as u64)
}

impl FileServeLimits {
    fn from_env() -> Self {
        let d = Self::default();
        Self {
            max_active: env_usize("AUTO_HTTP_FILE_ACTIVE", d.max_active),
            queue_capacity: env_usize("AUTO_HTTP_FILE_QUEUE", d.queue_capacity),
            app_block_bytes: env_usize("AUTO_HTTP_FILE_BLOCK", d.app_block_bytes),
            max_pending_blocks: env_usize("AUTO_HTTP_FILE_PENDING", d.max_pending_blocks),
            fs_ops_max: env_usize("AUTO_HTTP_FILE_FS_OPS", d.fs_ops_max),
            prepare_timeout: env_ms("AUTO_HTTP_FILE_PREPARE_MS", 30_000),
            body_idle_timeout: env_ms("AUTO_HTTP_FILE_IDLE_MS", 60_000),
            body_total_timeout: match std::env::var("AUTO_HTTP_FILE_TOTAL_MS") {
                Ok(v) => v
                    .trim()
                    .parse::<u64>()
                    .ok()
                    .filter(|ms| *ms > 0)
                    .map(Duration::from_millis),
                _ => None,
            },
        }
    }
}

struct FileServeState {
    limits: FileServeLimits,
    /// 准入排队信号量（排队期持有；Arc 克隆供 acquire_owned）。
    queue: std::sync::Arc<tokio::sync::Semaphore>,
    /// 活跃许可（body 全程持有；pump 退出即释放）。
    active: std::sync::Arc<tokio::sync::Semaphore>,
    /// 文件操作并发信号量。
    fs_ops: std::sync::Arc<tokio::sync::Semaphore>,
    active_count: AtomicUsize,
    queued_count: AtomicUsize,
}

impl FileServeState {
    fn from_limits(limits: FileServeLimits) -> Self {
        let queue = std::sync::Arc::new(tokio::sync::Semaphore::new(limits.queue_capacity));
        let active = std::sync::Arc::new(tokio::sync::Semaphore::new(limits.max_active));
        let fs_ops = std::sync::Arc::new(tokio::sync::Semaphore::new(limits.fs_ops_max));
        Self {
            limits,
            queue,
            active,
            fs_ops,
            active_count: AtomicUsize::new(0),
            queued_count: AtomicUsize::new(0),
        }
    }
}

lazy_static::lazy_static! {
    static ref FILE_SERVE: FileServeState = FileServeState::from_limits(FileServeLimits::from_env());
}

/// 当前生效限额（测试探针/报告用）。
pub fn file_serve_limits() -> FileServeLimits {
    FILE_SERVE.limits
}

/// 活跃文件 body 数（测试探针）。
#[doc(hidden)]
pub fn file_active_count() -> usize {
    FILE_SERVE.active_count.load(Ordering::SeqCst)
}

/// 排队中文件请求数（测试探针）。
#[doc(hidden)]
pub fn file_queued_count() -> usize {
    FILE_SERVE.queued_count.load(Ordering::SeqCst)
}

// ============================================================================
// 受限根目录打开（决策报告 §2）
// ============================================================================

/// 打开失败分类（→ HTTP 状态映射）。
#[derive(Debug, Clone, PartialEq, Eq)]
enum OpenError {
    /// 路径在 root 下不存在（含中间目录缺失/ENOTDIR）→ 404。
    NotFound,
    /// 存在但非普通文件（目录/设备/管道）→ 404。
    NotARegularFile,
    /// 越界/链接/reparse → 403。
    Escape(String),
    /// 其他 I/O 故障 → 500。
    Io(String),
}

impl OpenError {
    fn status(&self) -> u16 {
        match self {
            OpenError::NotFound | OpenError::NotARegularFile => 404,
            OpenError::Escape(_) => 403,
            OpenError::Io(_) => 500,
        }
    }

    /// 诊断消息（不含主机绝对路径）。
    fn message(&self) -> String {
        match self {
            OpenError::NotFound => "file not found under root".to_string(),
            OpenError::NotARegularFile => "not a regular file".to_string(),
            OpenError::Escape(why) => format!("path rejected: {why}"),
            OpenError::Io(_) => "file open failed".to_string(),
        }
    }
}

/// 已打开文件的表示 metadata（同句柄取得）。
struct OpenedFile {
    file: std::fs::File,
    len: u64,
    modified: Option<std::time::SystemTime>,
}

/// Windows 终段打开标志：reparse point 本身（sylink/junction）不跟随——
/// 打开成功后 `file_type().is_symlink()` 为真即终段是链接（拒绝）。
/// `FILE_FLAG_BACKUP_SEMANTICS`：允许打开目录（随后 is_file 门 404），
/// 不加则目录 open 直接 ERROR_ACCESS_DENIED → 误映射 500。
#[cfg(windows)]
const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
#[cfg(windows)]
const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;

/// 受限打开：逐段 no-follow walk + 终段 reparse 拒绝 + 句柄 is_file 门。
/// 词法校验已在描述符构造期完成（这里对相对路径再走一遍——防御描述符
/// 外的调用面）。全部 std 同步操作：调用方包在 spawn_blocking。
fn safe_open_file(root: &Path, relative: &str) -> Result<OpenedFile, OpenError> {
    if proto::validate_relative_path(relative).is_err() {
        return Err(OpenError::Escape("relative path rejected".to_string()));
    }
    let mut prefix = root.to_path_buf();
    let segments: Vec<&str> = relative
        .split(['/', '\\'])
        .filter(|s| !s.is_empty() && *s != ".")
        .collect();
    let (dirs, last) = match segments.split_last() {
        Some((l, d)) => (d, *l),
        None => return Err(OpenError::NotFound), // 空相对路径（词法已拒，防御）
    };
    // 逐段 walk：中间段必须存在、是目录、非 symlink（junction 同检——
    // T-01 P1b：symlink_metadata 的 is_symlink 覆盖两类 reparse）。
    for seg in dirs {
        prefix.push(seg);
        match std::fs::symlink_metadata(&prefix) {
            Ok(md) => {
                if md.file_type().is_symlink() {
                    return Err(OpenError::Escape("symlink in path".to_string()));
                }
                if !md.is_dir() {
                    return Err(OpenError::NotFound);
                }
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                return Err(OpenError::NotFound);
            }
            Err(_) => return Err(OpenError::Io("metadata".into())),
        }
    }
    prefix.push(last);
    // 终段打开（Windows：不跟随 reparse——终段是链接则句柄指向链接本身）。
    let mut opts = std::fs::OpenOptions::new();
    opts.read(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        opts.custom_flags(FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS);
    }
    let file = match opts.open(&prefix) {
        Ok(f) => f,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Err(OpenError::NotFound),
        Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
            return Err(OpenError::Io("permission".into()))
        }
        Err(_) => return Err(OpenError::Io("open".into())),
    };
    // 句柄 metadata：is_file 门（目录/命名管道等 → 404；reparse → 403）。
    let md = match file.metadata() {
        Ok(m) => m,
        Err(_) => return Err(OpenError::Io("metadata".into())),
    };
    let ft = md.file_type();
    if ft.is_symlink() {
        return Err(OpenError::Escape("final component is a link".to_string()));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::FileTypeExt;
        if ft.is_block_device() || ft.is_char_device() || ft.is_fifo() || ft.is_socket() {
            return Err(OpenError::NotARegularFile);
        }
    }
    if !ft.is_file() {
        return Err(OpenError::NotARegularFile);
    }
    let modified = md.modified().ok();
    Ok(OpenedFile {
        file,
        len: md.len(),
        modified,
    })
}

// ============================================================================
// 响应形态（版本无关 FileReply + 流）
// ============================================================================

/// 有界文件 body 流：pump 任务的通道接收端包装（`futures::Stream` 即
/// `futures_core::Stream`——两代 axum 的 `Body::from_stream` 直接可用）。
pub struct FileBodyStream {
    rx: tokio::sync::mpsc::Receiver<Result<Vec<u8>, io::Error>>,
}

impl futures::Stream for FileBodyStream {
    type Item = Result<Vec<u8>, io::Error>;
    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.rx.poll_recv(cx)
    }
}

/// 文件回复的 body 形态。
pub enum FileReplyBody {
    /// HEAD / 304 / 412 / 416：wire 无 body（Content-Length 语义见决策头）。
    None,
    /// 错误诊断（短 JSON；发送 headers 前的错误状态）。
    Inline(Vec<u8>),
    /// 文件窗口字节（有界块流）。
    Stream(FileBodyStream),
}

/// 一个文件响应的完整 wire 形态（status/headers 已定）。
pub struct FileReply {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: FileReplyBody,
}

fn json_escape(message: &str) -> String {
    let mut out = String::with_capacity(message.len() + 2);
    for c in message.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

impl FileReply {
    /// 错误回复（诊断 JSON 内联体 + 可选 Retry-After）。
    fn error(status: u16, message: &str, retry_after: bool) -> Self {
        let body = format!("{{\"error\":\"{}\"}}", json_escape(message)).into_bytes();
        let mut headers = vec![
            ("Content-Type".to_string(), "application/json".to_string()),
            ("Content-Length".to_string(), body.len().to_string()),
        ];
        if retry_after {
            headers.push(("Retry-After".to_string(), "1".to_string()));
        }
        Self {
            status,
            headers,
            body: FileReplyBody::Inline(body),
        }
    }
}

/// 收口种类（finish hook 参数；VM 腿据此完成 scope——幂等）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileFinish {
    /// 正常 EOF（窗口发送完毕）。
    Completed,
    /// 客户端侧流丢弃/断连（通道接收端关闭）。
    ClientGone,
    /// idle 期限到达（watchdog 触发取消）。
    IdleTimeout,
    /// 总期限到达。
    TotalTimeout,
    /// 读/发送失败（含提前 EOF 截断）。
    Failed(String),
}

/// 恰一次收口钩子（VM 腿：完成 scope；生成腿：资源已随 pump 回收）。
pub type FileFinishHook = Arc<dyn Fn(&FileFinish) + Send + Sync>;

struct BodyShared {
    /// 取消信号（watch → pump 的发送等待；值 = 触发种类）。
    cancelled: tokio::sync::watch::Sender<Option<FileFinish>>,
    finished: AtomicBool,
    last_activity: std::sync::Mutex<Instant>,
    active_permit: std::sync::Mutex<Option<tokio::sync::OwnedSemaphorePermit>>,
    finish_hook: Option<FileFinishHook>,
}

impl BodyShared {
    /// 恰一次终结：归还执行槽 + 触发 hook。计数只随真实 active 许可递减
    /// （直接驱动 pump 的测试面传 None——不污染全局计数）。
    fn finish(&self, outcome: FileFinish) {
        if self.finished.swap(true, Ordering::SeqCst) {
            return;
        }
        let held_active = self.active_permit.lock().unwrap().take().is_some();
        if held_active {
            FILE_SERVE.active_count.fetch_sub(1, Ordering::SeqCst);
        }
        if let Some(hook) = &self.finish_hook {
            hook(&outcome);
        }
    }

    /// watchdog 触发：终结 + 唤醒 pump 的发送等待（watch 保值——订阅后
    /// 不漏触发）。
    fn trigger(&self, outcome: FileFinish) {
        self.finish(outcome.clone());
        let _ = self.cancelled.send(Some(outcome));
    }

    fn is_finished(&self) -> bool {
        self.finished.load(Ordering::SeqCst)
    }

    fn touch(&self) {
        if let Ok(mut t) = self.last_activity.lock() {
            *t = Instant::now();
        }
    }

    fn last_activity(&self) -> Instant {
        self.last_activity
            .lock()
            .map(|t| *t)
            .unwrap_or_else(|_| Instant::now())
    }
}

// ============================================================================
// serve：准入 + 打开 + 决策 + 组装
// ============================================================================

/// serve 调用上下文。
pub struct ServeFileRequest<'a> {
    /// HTTP 方法（GET/HEAD；其他方法上游已 405）。
    pub method: &'a str,
    /// 请求头对（条件/Range 提取源）。
    pub request_headers: &'a [(String, String)],
    /// 准备总期限（等许可+open+metadata+seek；调用方取 min(30s, scope 剩余)）。
    pub prepare_deadline: Instant,
    /// 恰一次收口钩子（VM 腿完成 scope；body 非 Stream 回复不触发——调用
    /// 方对无 body 回复自行处理）。
    pub finish_hook: Option<FileFinishHook>,
    /// body idle 期限覆写（测试用；None = 全局限额默认。生产调用面恒 None）。
    pub idle_timeout: Option<std::time::Duration>,
}

/// 服务一个文件响应描述符（async；网络/读盘均不在 VM owner——调用点在
/// transport/生成 handler 的 tokio 上下文）。
pub async fn serve_file_response(
    descriptor: &proto::FileResponse,
    req: ServeFileRequest<'_>,
) -> FileReply {
    // 构造期错误映射（决策报告 §4：Options→500 / Path→403）。
    if let Some(err) = descriptor.init_error() {
        let status = match err.kind {
            proto::FileInitErrorKind::Options => 500,
            proto::FileInitErrorKind::Path => 403,
        };
        return FileReply::error(status, &err.message, false);
    }

    // 准入：排队 try（满 → 即时 503）→ active acquire（期限内）。
    let queue_permit = match FILE_SERVE.queue.clone().try_acquire_owned() {
        Ok(p) => p,
        Err(_) => {
            return FileReply::error(503, "file queue full", true);
        }
    };
    FILE_SERVE.queued_count.fetch_add(1, Ordering::SeqCst);
    let active_permit = {
        let acquire = FILE_SERVE.active.clone().acquire_owned();
        let deadline = tokio::time::Instant::from_std(req.prepare_deadline);
        match tokio::time::timeout_at(deadline, acquire).await {
            Ok(Ok(p)) => p,
            _ => {
                FILE_SERVE.queued_count.fetch_sub(1, Ordering::SeqCst);
                return FileReply::error(503, "file preparation timed out", true);
            }
        }
    };
    FILE_SERVE.queued_count.fetch_sub(1, Ordering::SeqCst);
    drop(queue_permit); // 进入 active：排队许可立即归还
    FILE_SERVE.active_count.fetch_add(1, Ordering::SeqCst);

    // 打开 + 表示 metadata（blocking pool——不占调用线程）。
    let root: PathBuf = descriptor.root().to_path_buf();
    let relative = descriptor.relative().to_string();
    let opened = {
        let deadline = req.prepare_deadline;
        let relative_for_open = relative.clone();
        let task = tokio::task::spawn_blocking(move || safe_open_file(&root, &relative_for_open));
        let deadline_tokio = tokio::time::Instant::from_std(deadline);
        match tokio::time::timeout_at(deadline_tokio, task).await {
            Ok(Ok(o)) => o,
            Ok(Err(_)) => {
                release_active_only();
                return FileReply::error(500, "file open failed", false);
            }
            Err(_) => {
                release_active_only();
                return FileReply::error(503, "file preparation timed out", true);
            }
        }
    };
    let opened = match opened {
        Ok(o) => o,
        Err(e) => {
            release_active_only();
            return FileReply::error(e.status(), &e.message(), false);
        }
    };

    // 协议决策（纯函数单源：a2r_std::http::server_file）。
    let conds = proto::extract_request_conditions(req.request_headers);
    let validators = proto::RepresentationValidators {
        etag: descriptor.options().etag.clone(),
        last_modified: opened.modified,
    };
    let decision = proto::decide_file_protocol(req.method, &conds, opened.len, &validators);
    let inferred_mime = infer_content_type(&relative);
    let headers =
        proto::build_file_headers(descriptor.options(), &decision, &validators, &inferred_mime);

    if !decision.include_body {
        // HEAD / 304 / 412 / 416：无文件字节；句柄即弃，资源即收。
        drop(opened.file);
        release_active_only();
        return FileReply {
            status: decision.status,
            headers,
            body: FileReplyBody::None,
        };
    }

    // body 路径：句柄转 tokio；准备段最后一歩 = seek（fs op 许可围住）。
    // 窗口是闭区间 (start, end)；pump 消费字节数 = end-start+1（零长文件
    // → None → 空窗口）。
    let (start, len) = decision
        .window
        .map(|(s, e)| (s, e - s + 1))
        .unwrap_or((0, 0));
    let mut tokio_file = tokio::fs::File::from_std(opened.file);
    {
        let permit = acquire_fs_op(&req.prepare_deadline).await;
        if permit.is_none() {
            let shared = finish_shared(Some(active_permit), req.finish_hook.as_ref());
            shared.finish(FileFinish::Failed("prepare deadline".into()));
            return FileReply::error(503, "file preparation timed out", true);
        }
        use tokio::io::AsyncSeekExt;
        let seek = tokio_file.seek(io::SeekFrom::Start(start)).await;
        drop(permit);
        if seek.is_err() {
            let shared = finish_shared(Some(active_permit), req.finish_hook.as_ref());
            shared.finish(FileFinish::Failed("seek failed".into()));
            return FileReply::error(500, "file read failed", false);
        }
    }

    // pump + watchdog：通道容量 = max_pending_blocks - 1（在读 1 块 + 通道
    // N-1 块 ≤ 每活跃待发送块上限）。pump 拥有句柄/active 许可——退出即
    // 全量回收（决策报告 §5：执行槽在在途读退出后复用）。
    let shared = finish_shared(Some(active_permit), req.finish_hook.as_ref());
    let (tx, rx) = tokio::sync::mpsc::channel::<Result<Vec<u8>, io::Error>>(
        FILE_SERVE
            .limits
            .max_pending_blocks
            .saturating_sub(1)
            .max(1),
    );
    let limits = FILE_SERVE.limits;
    let pump_shared = Arc::clone(&shared);
    tokio::spawn(async move {
        pump_file_body(tokio_file, len, limits.app_block_bytes, tx, pump_shared).await;
    });
    let watch_shared = Arc::clone(&shared);
    let idle = req.idle_timeout.unwrap_or(limits.body_idle_timeout);
    tokio::spawn(async move {
        watchdog(watch_shared, idle, limits.body_total_timeout).await;
    });

    FileReply {
        status: decision.status,
        headers,
        body: FileReplyBody::Stream(FileBodyStream { rx }),
    }
}

/// 无 body 路径的 active 计数归还（信号量许可由作用域 drop 归还）。
fn release_active_only() {
    FILE_SERVE.active_count.fetch_sub(1, Ordering::SeqCst);
}

fn finish_shared(
    active_permit: Option<tokio::sync::OwnedSemaphorePermit>,
    hook: Option<&FileFinishHook>,
) -> Arc<BodyShared> {
    let (cancelled_tx, _) = tokio::sync::watch::channel(None::<FileFinish>);
    Arc::new(BodyShared {
        cancelled: cancelled_tx,
        finished: AtomicBool::new(false),
        last_activity: std::sync::Mutex::new(Instant::now()),
        active_permit: std::sync::Mutex::new(active_permit),
        finish_hook: hook.cloned(),
    })
}

async fn acquire_fs_op(deadline: &Instant) -> Option<tokio::sync::OwnedSemaphorePermit> {
    let acquire = FILE_SERVE.fs_ops.clone().acquire_owned();
    match tokio::time::timeout_at(tokio::time::Instant::from_std(*deadline), acquire).await {
        Ok(Ok(p)) => Some(p),
        _ => None,
    }
}

/// MIME 推断（宿主侧 mime_guess；不明 → octet-stream）。content_type 选项
/// 覆盖已在 build_file_headers 处理。
fn infer_content_type(relative: &str) -> String {
    mime_guess::from_path(Path::new(relative))
        .first_raw()
        .map(|s| s.to_string())
        .unwrap_or_else(|| "application/octet-stream".to_string())
}

/// 失败终结帧：尽力交付 Err（通道满时与取消 select——黑洞消费者由
/// watchdog 收口，错误帧可弃）。
async fn send_terminal_error(
    tx: &tokio::sync::mpsc::Sender<Result<Vec<u8>, io::Error>>,
    err: io::Error,
    cancel_rx: &mut tokio::sync::watch::Receiver<Option<FileFinish>>,
) {
    let send = tx.send(Err(err));
    tokio::select! {
        sent = send => {
            let _ = sent;
        }
        _ = cancel_rx.changed() => {}
    }
}

/// pump：读驱动发送。取消语义（决策报告 §5）：停止 issuance 新读、在途读
/// 自然收口后退出；发送等待（背压/黑洞客户端）经 watch 取消立即退出——
/// 句柄随本任务 Drop 回收。
async fn pump_file_body(
    mut file: tokio::fs::File,
    len: u64,
    block_bytes: usize,
    tx: tokio::sync::mpsc::Sender<Result<Vec<u8>, io::Error>>,
    shared: Arc<BodyShared>,
) {
    use tokio::io::AsyncReadExt;
    let mut cancel_rx = shared.cancelled.subscribe();
    let mut remaining = len;
    let mut buf = vec![0u8; block_bytes.max(1)];
    let outcome = loop {
        // 取消检查（watch 保值：订阅前触发的终结也能在此观察到）。
        if let Some(kind) = cancel_rx.borrow().clone() {
            break kind; // IdleTimeout / TotalTimeout（watchdog 已收口）
        }
        if remaining == 0 {
            break FileFinish::Completed;
        }
        let want = remaining.min(buf.len() as u64) as usize;
        // fs op 并发许可：try + 有界小睡重试（持许可者不等待其他许可——
        // 无死锁；重试不越过取消检查下一轮）。
        let permit = loop {
            if cancel_rx.borrow().is_some() {
                break None;
            }
            match FILE_SERVE.fs_ops.clone().try_acquire_owned() {
                Ok(p) => break Some(p),
                Err(_) => tokio::time::sleep(Duration::from_millis(5)).await,
            }
        };
        if permit.is_none() {
            continue; // 已取消 → 回循环顶退出
        }
        // 在途读不取消（in-flight FS 收口语义——取消只停止 issuance）。
        let read = file.read(&mut buf[..want]).await;
        drop(permit);
        shared.touch();
        match read {
            Ok(0) => {
                // 提前 EOF：句柄被截断——发送失败帧（消费端可观察），不当
                // 正常完成（T-01 P5）。
                send_terminal_error(
                    &tx,
                    io::Error::new(io::ErrorKind::UnexpectedEof, "file truncated during send"),
                    &mut cancel_rx,
                )
                .await;
                break FileFinish::Failed("file truncated during send".into());
            }
            Ok(n) => {
                remaining -= n as u64;
                let chunk = buf[..n].to_vec();
                tokio::select! {
                    sent = tx.send(Ok(chunk)) => {
                        if sent.is_err() {
                            break FileFinish::ClientGone; // 接收端 Drop（断连/丢弃）
                        }
                    }
                    changed = cancel_rx.changed() => {
                        if changed.is_ok() {
                            continue; // 循环顶观察取消值并退出
                        }
                        break FileFinish::ClientGone;
                    }
                }
            }
            Err(e) => {
                let msg = format!("file read failed: {e}");
                send_terminal_error(
                    &tx,
                    io::Error::new(io::ErrorKind::Other, msg.clone()),
                    &mut cancel_rx,
                )
                .await;
                break FileFinish::Failed(msg);
            }
        }
    };
    shared.finish(outcome);
}

/// watchdog：独立计时（覆盖 body 不再被 poll 的慢客户端——决策报告 §5）。
/// 触发 = 标记终结 + watch 唤醒 pump 的发送等待；资源实际归还由 pump 退出
/// 完成（在途读自然收口后）。
async fn watchdog(
    shared: Arc<BodyShared>,
    idle_timeout: Duration,
    total_timeout: Option<Duration>,
) {
    let total_deadline = total_timeout.map(|d| Instant::now() + d);
    loop {
        if shared.is_finished() {
            return;
        }
        let idle_deadline = shared.last_activity() + idle_timeout;
        let next = match total_deadline {
            Some(td) if td < idle_deadline => td,
            _ => idle_deadline,
        };
        let now = Instant::now();
        if now >= next {
            let hit_total = total_deadline.is_some_and(|td| now >= td);
            let hit_idle = now >= shared.last_activity() + idle_timeout;
            if hit_total || hit_idle {
                let outcome = if hit_total {
                    FileFinish::TotalTimeout
                } else {
                    FileFinish::IdleTimeout
                };
                shared.trigger(outcome);
                return;
            }
        }
        tokio::time::sleep_until(tokio::time::Instant::from_std(next)).await;
    }
}

// ============================================================================
// 单元测试（T-03 验证：gate/FS 注入与 feature 无关面；wire 矩阵在
// plan729 测试族）
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("plan729-fs-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn std_headers(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    fn serve_simple(
        root: &Path,
        rel: &str,
        method: &str,
        headers: &[(&str, &str)],
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = FileReply> + Send>> {
        let desc = proto::file_response(root.to_str().unwrap(), rel, "{}");
        let hdrs = std_headers(headers);
        let method = method.to_string();
        Box::pin(async move {
            serve_file_response(
                &desc,
                ServeFileRequest {
                    method: &method,
                    request_headers: &hdrs,
                    prepare_deadline: Instant::now() + Duration::from_secs(10),
                    finish_hook: None,
                    idle_timeout: None,
                },
            )
            .await
        })
    }

    async fn drain(reply: FileReply) -> Result<Vec<u8>, io::Error> {
        use futures::StreamExt;
        let FileReplyBody::Stream(mut body) = reply.body else {
            panic!("expected stream body");
        };
        let mut out = Vec::new();
        while let Some(chunk) = body.next().await {
            out.extend_from_slice(&chunk?);
        }
        Ok(out)
    }

    fn header<'a>(reply: &'a FileReply, name: &str) -> String {
        reply
            .headers
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.clone())
            .unwrap_or_default()
    }

    #[tokio::test]
    async fn plan729_open_and_full_get() {
        let root = temp_root("get");
        std::fs::write(root.join("hello.txt"), b"hello plan729").unwrap();
        let reply = serve_simple(&root, "hello.txt", "GET", &[]).await;
        assert_eq!(reply.status, 200);
        assert_eq!(header(&reply, "Content-Length"), "13");
        assert_eq!(header(&reply, "Content-Type"), "text/plain");
        assert_eq!(header(&reply, "Accept-Ranges"), "bytes");
        assert!(!header(&reply, "Last-Modified").is_empty());
        let body = drain(reply).await.unwrap();
        assert_eq!(body, b"hello plan729");
        std::fs::remove_dir_all(&root).ok();
    }

    #[tokio::test]
    async fn plan729_head_no_body() {
        let root = temp_root("head");
        std::fs::write(root.join("f.bin"), vec![1u8; 100]).unwrap();
        let reply = serve_simple(&root, "f.bin", "HEAD", &[("range", "bytes=0-9")]).await;
        assert_eq!(reply.status, 200);
        assert!(matches!(reply.body, FileReplyBody::None));
        assert_eq!(header(&reply, "Content-Length"), "100"); // 表示长度（Range 忽略）
        std::fs::remove_dir_all(&root).ok();
    }

    #[tokio::test]
    async fn plan729_missing_dir_and_notfile_404() {
        let root = temp_root("404");
        std::fs::create_dir_all(root.join("sub")).unwrap();
        let r1 = serve_simple(&root, "nope.txt", "GET", &[]).await;
        assert_eq!(r1.status, 404);
        let r2 = serve_simple(&root, "sub", "GET", &[]).await; // 目录
        assert_eq!(r2.status, 404);
        let r3 = serve_simple(&root, "sub/deep/missing.bin", "GET", &[]).await;
        assert_eq!(r3.status, 404);
        std::fs::remove_dir_all(&root).ok();
    }

    #[tokio::test]
    async fn plan729_escape_traversal_403() {
        let root = temp_root("esc");
        std::fs::create_dir_all(root.join("d")).unwrap();
        let r1 = serve_simple(&root, "../outside/secret.txt", "GET", &[]).await;
        assert_eq!(r1.status, 403);
        let r2 = serve_simple(&root, "C:/windows/win.ini", "GET", &[]).await;
        assert_eq!(r2.status, 403);
        let r3 = serve_simple(&root, "\\\\server\\share\\f", "GET", &[]).await;
        assert_eq!(r3.status, 403);
        let r4 = serve_simple(&root, "/etc/passwd", "GET", &[]).await;
        assert_eq!(r4.status, 403); // 前导 / = 绝对路径 → 词法拒绝
        let r5 = serve_simple(&root, "NUL", "GET", &[]).await;
        assert_eq!(r5.status, 403); // 设备名 → 词法拒绝
        std::fs::remove_dir_all(&root).ok();
    }

    #[tokio::test]
    async fn plan729_junction_escape_rejected() {
        // T-01 P1 复现（walk 层）：junction 指向 root 外 → 403。
        // 链接只在测试独占临时目录创建（红线：不进工作检出）。
        let base = temp_root("junc");
        let root = base.join("root");
        let secret = base.join("secret");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(&secret).unwrap();
        std::fs::write(secret.join("esc.txt"), b"ESCAPED").unwrap();
        std::fs::write(root.join("ok.txt"), b"ok").unwrap();
        let link = root.join("jdir");
        let st = std::process::Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(&link)
            .arg(&secret)
            .status()
            .unwrap();
        if !st.success() {
            eprintln!("SKIP: junction 权限缺失（环境缺项如实记录，不以假证据计数）");
            std::fs::remove_dir_all(&base).ok();
            return;
        }
        // walk 检出中间段 junction → 403（不经由跟随读到 ESCAPED）。
        let r = serve_simple(&root, "jdir/esc.txt", "GET", &[]).await;
        assert_eq!(r.status, 403);
        // 正常文件不受影响。
        let ok = serve_simple(&root, "ok.txt", "GET", &[]).await;
        assert_eq!(ok.status, 200);
        assert_eq!(drain(ok).await.unwrap(), b"ok");
        std::fs::remove_dir_all(&base).ok();
    }

    #[tokio::test]
    async fn plan729_zero_byte_file() {
        let root = temp_root("zero");
        std::fs::write(root.join("empty.bin"), b"").unwrap();
        let get = serve_simple(&root, "empty.bin", "GET", &[]).await;
        assert_eq!(get.status, 200);
        assert_eq!(header(&get, "Content-Length"), "0"); // 不是 1
        let body = drain(get).await.unwrap();
        assert!(body.is_empty());
        // 带 Range → 416。
        let r416 = serve_simple(&root, "empty.bin", "GET", &[("range", "bytes=0-")]).await;
        assert_eq!(r416.status, 416);
        assert!(matches!(r416.body, FileReplyBody::None));
        assert_eq!(header(&r416, "Content-Range"), "bytes */0");
        std::fs::remove_dir_all(&root).ok();
    }

    #[tokio::test]
    async fn plan729_range_window_body() {
        let root = temp_root("range");
        let data: Vec<u8> = (0u8..=199).collect();
        std::fs::write(root.join("r.bin"), &data).unwrap();
        let reply = serve_simple(&root, "r.bin", "GET", &[("range", "bytes=10-19")]).await;
        assert_eq!(reply.status, 206);
        assert_eq!(header(&reply, "Content-Range"), "bytes 10-19/200");
        assert_eq!(header(&reply, "Content-Length"), "10");
        let body = drain(reply).await.unwrap();
        assert_eq!(body, (10u8..=19).collect::<Vec<u8>>());
        // suffix。
        let suf = serve_simple(&root, "r.bin", "GET", &[("range", "bytes=-7")]).await;
        assert_eq!(suf.status, 206);
        assert_eq!(
            drain(suf).await.unwrap(),
            (193u8..=199).collect::<Vec<u8>>()
        );
        // 416。
        let un = serve_simple(&root, "r.bin", "GET", &[("range", "bytes=500-")]).await;
        assert_eq!(un.status, 416);
        assert!(matches!(un.body, FileReplyBody::None));
        std::fs::remove_dir_all(&root).ok();
    }

    #[tokio::test]
    async fn plan729_non_utf8_content() {
        let root = temp_root("bin");
        let data: Vec<u8> = vec![0x00, 0xFF, 0xFE, 0x80, 0x01];
        std::fs::write(root.join("raw.bin"), &data).unwrap();
        let reply = serve_simple(&root, "raw.bin", "GET", &[]).await;
        assert_eq!(reply.status, 200);
        assert_eq!(header(&reply, "Content-Type"), "application/octet-stream");
        assert_eq!(drain(reply).await.unwrap(), data);
        std::fs::remove_dir_all(&root).ok();
    }

    #[tokio::test]
    async fn plan729_truncation_mid_send_fails() {
        // T-01 P5 复现：句柄 metadata（承诺长度）后、发送中被截断 → 提前
        // EOF = 发送失败（不当正常完成）。确定性形态：句柄打开取 len 后，
        // pump 启动前用另一句柄截断——pump 读到 16 字节即提前 EOF。
        let root = temp_root("trunc");
        let path = root.join("t.bin");
        std::fs::write(&path, vec![7u8; 64]).unwrap();
        let std_file = std::fs::File::open(&path).unwrap();
        let promised = std_file.metadata().unwrap().len();
        assert_eq!(promised, 64);
        let tokio_file = tokio::fs::File::from_std(std_file);
        // 截断（原句柄仍持有——同句柄语义下句柄继续提供旧内容直到越过新
        // EOF，随后提前 EOF）。
        let f = std::fs::OpenOptions::new().write(true).open(&path).unwrap();
        f.set_len(16).unwrap();
        drop(f);
        let shared = finish_shared(None, None);
        let (tx, mut rx) = tokio::sync::mpsc::channel::<Result<Vec<u8>, io::Error>>(1);
        let pump_shared = Arc::clone(&shared);
        tokio::spawn(async move {
            pump_file_body(tokio_file, promised, 16, tx, pump_shared).await;
        });
        let mut saw_err = false;
        let mut total = 0usize;
        while let Some(chunk) = rx.recv().await {
            match chunk {
                Ok(b) => total += b.len(),
                Err(_) => {
                    saw_err = true;
                    break;
                }
            }
        }
        assert!(saw_err, "截断必须报错，不当正常完成");
        assert_eq!(total, 16, "只发到新 EOF 为止");
        std::fs::remove_dir_all(&root).ok();
    }

    #[tokio::test]
    async fn plan729_finish_hook_and_counters() {
        let root = temp_root("hook");
        std::fs::write(root.join("h.bin"), b"hookdata").unwrap();
        let finish: Arc<std::sync::Mutex<Vec<FileFinish>>> =
            Arc::new(std::sync::Mutex::new(Vec::new()));
        let f2 = Arc::clone(&finish);
        let hook: FileFinishHook = Arc::new(move |kind: &FileFinish| {
            f2.lock().unwrap().push(kind.clone());
        });
        let desc = proto::file_response(root.to_str().unwrap(), "h.bin", "{}");
        let hdrs = Vec::new();
        let reply = serve_file_response(
            &desc,
            ServeFileRequest {
                method: "GET",
                request_headers: &hdrs,
                prepare_deadline: Instant::now() + Duration::from_secs(10),
                finish_hook: Some(hook),
                idle_timeout: None,
            },
        )
        .await;
        assert_eq!(reply.status, 200);
        let drained = drain(reply).await.unwrap();
        assert_eq!(drained, b"hookdata");
        // 恰一次 Completed。
        let deadline = Instant::now() + Duration::from_secs(5);
        while finish.lock().unwrap().is_empty() && Instant::now() < deadline {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        assert_eq!(*finish.lock().unwrap(), vec![FileFinish::Completed]);
        assert_eq!(file_active_count(), 0);
        std::fs::remove_dir_all(&root).ok();
    }

    #[tokio::test]
    async fn plan729_client_gone_reclaims() {
        // 接收端丢弃（drop body）：pump 发送失败 → ClientGone → 资源回收。
        let root = temp_root("gone");
        std::fs::write(root.join("g.bin"), vec![9u8; 256 * 1024]).unwrap();
        let desc = proto::file_response(root.to_str().unwrap(), "g.bin", "{}");
        let hdrs = Vec::new();
        let reply = serve_file_response(
            &desc,
            ServeFileRequest {
                method: "GET",
                request_headers: &hdrs,
                prepare_deadline: Instant::now() + Duration::from_secs(10),
                finish_hook: None,
                idle_timeout: None,
            },
        )
        .await;
        assert_eq!(reply.status, 200);
        drop(reply.body); // 客户端断开
        let deadline = Instant::now() + Duration::from_secs(5);
        while file_active_count() > 0 && Instant::now() < deadline {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        assert_eq!(file_active_count(), 0, "断连后资源回基线");
        std::fs::remove_dir_all(&root).ok();
    }

    #[tokio::test]
    async fn plan729_percent_named_missing_file_404() {
        let root = temp_root("pct");
        std::fs::write(root.join("ok.txt"), b"x").unwrap();
        // % 字面名（双重编码路由输入的单次解码形态）：缺失文件必须 404
        // 而非 500（Windows ERROR_INVALID_NAME 防御——见下 NotFound 分类）。
        let r = serve_simple(&root, "%2e%2e%2fsecret.txt", "GET", &[]).await;
        assert_eq!(r.status, 404, "percent-named missing → 404");
        let r2 = serve_simple(&root, "a%25b.txt", "GET", &[]).await;
        assert_eq!(r2.status, 404);
    }

    #[tokio::test]
    async fn plan729_options_error_maps_500_path_maps_403() {
        let root = temp_root("opt");
        std::fs::write(root.join("a.txt"), b"x").unwrap();
        let desc_bad_opts =
            proto::file_response(root.to_str().unwrap(), "a.txt", r#"{"unknown_key":true}"#);
        let hdrs = Vec::new();
        let r = serve_file_response(
            &desc_bad_opts,
            ServeFileRequest {
                method: "GET",
                request_headers: &hdrs,
                prepare_deadline: Instant::now() + Duration::from_secs(10),
                finish_hook: None,
                idle_timeout: None,
            },
        )
        .await;
        assert_eq!(r.status, 500);
        assert!(matches!(r.body, FileReplyBody::Inline(_)));
        // 诊断体不泄露主机绝对路径。
        if let FileReplyBody::Inline(b) = &r.body {
            let s = String::from_utf8_lossy(b);
            assert!(!s.contains(&root.to_string_lossy().to_string()));
        }
        std::fs::remove_dir_all(&root).ok();
    }
}
