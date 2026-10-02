//! PLAN-727：共享 Rust 文件传输核心（VM/a2r/原生 Rust 三方单源）。
//!
//! 交付语义（PLAN-727 §2/§5 冻结；本模块 doc 注释即契约速览，canonical
//! Spec 由 SD-01 沉淀）：
//!
//! ```text
//! transfer_download/upload(url, path, options_json) → FileTransfer 句柄（非阻塞提交）
//!   │ 传输队列许可(try, 满 → 终结性 QueueFull 收据) → 传输活跃许可(await，
//!   │ 总期限自提交起覆盖排队/建立/传输/提交) → 准入等待可被取消
//!   ▼ 下载：验证成功状态 → 同目录独占 staging → 网络 bytes → 有限块(≤64KiB，
//!   │ 待处理 ≤2 背压) → 写盘/flush/sync → commit(rename) → Succeeded
//!   └ 上传：文件有限块 → raw / multipart → 有界响应(≤response_body_budget)
//!
//! 取消：FileTransfer Drop / transfer_cancel / transfer_wait_async future
//! Drop → 取消；写盘任务停止 issuing 新写、等在途写收口、清理 staging；
//! 原目标只在 commit 成功时被替换（迟到取消不回滚已提交结果）。
//! ```
//!
//! 状态机：`Queued → Opening → Transferring → Committing → Succeeded`，
//! 任何非终态可进入 `Failed/Cancelled`；只终结一次（首个终态胜出）。
//!
//! 排除面（PLAN-727 §0）：不做服务端任意文件路由、文件服务 Range 响应、
//! API/IPC 契约统一、CPU 抢占或部署配置；不承诺断电/崩溃持久性。

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::Deserialize;

// ============================================================================
// typed 错误 / 终态（收据契约的组成部分）
// ============================================================================

/// 传输终态种类（收据 `kind` 字段）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferOutcome {
    Success,
    Failed,
    Cancelled,
}

impl TransferOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            TransferOutcome::Success => "success",
            TransferOutcome::Failed => "failed",
            TransferOutcome::Cancelled => "cancelled",
        }
    }
}

/// 终结错误种类（收据 `error.kind` 字段）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferErrorKind {
    /// options JSON 解析失败/非法（请求未发出）。
    Options,
    /// 传输队列已满（提交即拒绝）。
    QueueFull,
    /// 上游返回非成功状态（含 404/416）。
    HttpStatus,
    /// 网络传输失败。
    Transport,
    /// 本机文件系统错误（staging/读写/替换/清理）。
    File,
    /// 文件/响应体超预算。
    Budget,
    /// 总期限或空闲期限到达。
    Timeout,
    /// 调用方取消（句柄 Drop / cancel / scope 回收）。
    Cancelled,
    /// 同目标已有在途传输（目标仲裁）。
    Conflict,
    /// 上传源文件在重试间被修改（len/mtime 复核不符）。
    SourceChanged,
}

impl TransferErrorKind {
    pub fn as_str(self) -> &'static str {
        match self {
            TransferErrorKind::Options => "options",
            TransferErrorKind::QueueFull => "queue_full",
            TransferErrorKind::HttpStatus => "http_status",
            TransferErrorKind::Transport => "transport",
            TransferErrorKind::File => "file",
            TransferErrorKind::Budget => "budget",
            TransferErrorKind::Timeout => "timeout",
            TransferErrorKind::Cancelled => "cancelled",
            TransferErrorKind::Conflict => "conflict",
            TransferErrorKind::SourceChanged => "source_changed",
        }
    }
}

/// typed 终结错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferError {
    pub kind: TransferErrorKind,
    pub message: String,
}

impl TransferError {
    pub fn new(kind: TransferErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

// ============================================================================
// 限额（PLAN-727 §5.3 冻结默认；env 首次使用读取一次；严格 options 可收窄）
// ============================================================================

/// 文件传输限额（独立于普通 HTTP/SSE 配额）。
#[derive(Debug, Clone, Copy)]
pub struct TransferLimits {
    /// 传输活跃许可。
    pub max_active: usize,
    /// 传输队列容量（提交即拒绝的上限）。
    pub queue_capacity: usize,
    /// 应用读写块上限（字节）。
    pub app_block_bytes: usize,
    /// 未落盘/未发送的待处理块上限（背压高水位）。
    pub max_pending_blocks: usize,
    /// 单文件字节预算（下载超限/上传源文件超限 → 终结性失败）。
    pub file_budget_bytes: u64,
    /// 准入至终结总期限（自提交起）。
    pub total_timeout: Duration,
    /// 上游读空闲期限（每 chunk 重置；写盘背压等待不计时）。
    pub idle_timeout: Duration,
    /// 内核级并发文件操作上限（受限阻塞池的可观测边界）。
    pub fs_ops_max: usize,
    /// 上传响应体预算（与普通 response 的 10 MiB 上限分开）。
    pub response_body_budget: usize,
}

impl Default for TransferLimits {
    fn default() -> Self {
        Self {
            max_active: 4,
            queue_capacity: 16,
            app_block_bytes: 64 * 1024,
            max_pending_blocks: 2,
            file_budget_bytes: 1024 * 1024 * 1024,
            total_timeout: Duration::from_secs(600),
            idle_timeout: Duration::from_secs(60),
            fs_ops_max: 4,
            response_body_budget: 10 * 1024 * 1024,
        }
    }
}

impl TransferLimits {
    pub(crate) fn from_env() -> Self {
        let d = Self::default();
        Self {
            max_active: env_usize("AUTO_A2R_TRANSFER_ACTIVE", d.max_active),
            queue_capacity: env_usize("AUTO_A2R_TRANSFER_QUEUE", d.queue_capacity),
            app_block_bytes: env_usize("AUTO_A2R_TRANSFER_BLOCK", d.app_block_bytes),
            max_pending_blocks: env_usize("AUTO_A2R_TRANSFER_PENDING", d.max_pending_blocks),
            file_budget_bytes: env_usize(
                "AUTO_A2R_TRANSFER_FILE_BUDGET",
                d.file_budget_bytes as usize,
            ) as u64,
            total_timeout: Duration::from_millis(
                env_usize("AUTO_A2R_TRANSFER_TIMEOUT_MS", 600_000) as u64,
            ),
            idle_timeout: Duration::from_millis(
                env_usize("AUTO_A2R_TRANSFER_IDLE_MS", 60_000) as u64
            ),
            fs_ops_max: env_usize("AUTO_A2R_TRANSFER_FS_OPS", d.fs_ops_max),
            response_body_budget: env_usize(
                "AUTO_A2R_TRANSFER_RESP_BUDGET",
                d.response_body_budget,
            ),
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

// ============================================================================
// options（严格解析：未知键/坏值 = Options 终态，请求不发出）
// ============================================================================

/// 续传版本验证器（发 If-Range；ETag 必须为强验证器）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Validator {
    /// 强 ETag（拒绝 `W/` 弱验证器）。
    Etag(String),
    /// Last-Modified 日期（原样发 If-Range）。
    LastModified(String),
}

/// 已存在目标策略（下载）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OnExists {
    /// 目标存在 → 传输成功后经 commit 原子替换（默认）。
    #[default]
    Overwrite,
    /// 目标已存在 → 终结性失败（目标保留）。
    Fail,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct DownloadOptionsRaw {
    #[serde(default)]
    headers: Option<std::collections::HashMap<String, String>>,
    #[serde(default)]
    offset: Option<u64>,
    #[serde(default)]
    validator: Option<ValidatorRaw>,
    #[serde(default)]
    max_bytes: Option<u64>,
    #[serde(default)]
    timeout_ms: Option<u64>,
    #[serde(default)]
    on_exists: Option<String>,
    #[serde(default)]
    idle_timeout_ms: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ValidatorRaw {
    #[serde(default)]
    etag: Option<String>,
    #[serde(default)]
    last_modified: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct UploadOptionsRaw {
    #[serde(default)]
    headers: Option<std::collections::HashMap<String, String>>,
    #[serde(default)]
    mode: Option<String>,
    #[serde(default)]
    field: Option<String>,
    #[serde(default)]
    filename: Option<String>,
    #[serde(default)]
    fields: Option<std::collections::HashMap<String, String>>,
    #[serde(default)]
    timeout_ms: Option<u64>,
    #[serde(default)]
    retries: Option<u32>,
}

/// 已解析的下载 options（owned）。
#[derive(Debug, Clone)]
pub struct DownloadOptions {
    pub headers: Vec<(String, String)>,
    /// 续传起点（字节）。>0 要求本地文件长度精确匹配；无 validator 时
    /// 仅接受严格 Content-Range 对齐的 206（版本一致性不承诺）。
    pub offset: Option<u64>,
    pub validator: Option<Validator>,
    pub max_bytes: Option<u64>,
    pub timeout_ms: Option<u64>,
    pub on_exists: OnExists,
    pub idle_timeout_ms: Option<u64>,
}

impl Default for DownloadOptions {
    fn default() -> Self {
        Self {
            headers: Vec::new(),
            offset: None,
            validator: None,
            max_bytes: None,
            timeout_ms: None,
            on_exists: OnExists::Overwrite,
            idle_timeout_ms: None,
        }
    }
}

impl DownloadOptions {
    pub(crate) fn parse(json: &str) -> Result<Self, TransferError> {
        if json.trim().is_empty() {
            return Ok(Self::default());
        }
        let raw: DownloadOptionsRaw = serde_json::from_str(json).map_err(|e| {
            TransferError::new(
                TransferErrorKind::Options,
                format!("invalid download options: {e}"),
            )
        })?;
        let on_exists = match raw.on_exists.as_deref() {
            None | Some("overwrite") => OnExists::Overwrite,
            Some("fail") => OnExists::Fail,
            Some(other) => {
                return Err(TransferError::new(
                    TransferErrorKind::Options,
                    format!("invalid on_exists {other:?}: expected \"overwrite\"|\"fail\""),
                ))
            }
        };
        let validator = match raw.validator {
            None => None,
            Some(v) => match (v.etag, v.last_modified) {
                (Some(e), None) => {
                    let e = e.trim().to_string();
                    if e.is_empty() {
                        return Err(TransferError::new(
                            TransferErrorKind::Options,
                            "empty etag validator",
                        ));
                    }
                    if e.starts_with("W/") || e.starts_with("w/") {
                        return Err(TransferError::new(
                            TransferErrorKind::Options,
                            "weak ETag (W/) is not a valid resume validator",
                        ));
                    }
                    Some(Validator::Etag(e))
                }
                (None, Some(m)) => {
                    let m = m.trim().to_string();
                    if m.is_empty() {
                        return Err(TransferError::new(
                            TransferErrorKind::Options,
                            "empty last_modified validator",
                        ));
                    }
                    Some(Validator::LastModified(m))
                }
                (Some(_), Some(_)) => {
                    return Err(TransferError::new(
                        TransferErrorKind::Options,
                        "validator accepts etag or last_modified, not both",
                    ))
                }
                (None, None) => {
                    return Err(TransferError::new(
                        TransferErrorKind::Options,
                        "validator requires etag or last_modified",
                    ))
                }
            },
        };
        Ok(Self {
            headers: headers_from_map(raw.headers, "download")?,
            offset: raw.offset,
            validator,
            max_bytes: raw.max_bytes,
            timeout_ms: raw.timeout_ms,
            on_exists,
            idle_timeout_ms: raw.idle_timeout_ms,
        })
    }
}

/// 上传模式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UploadMode {
    /// multipart/form-data，文件 part 字段名 `field`（默认；对齐 VM 既有
    /// upload 的 `"file"` 字段）。
    Multipart,
    /// 原始字节体（application/octet-stream）。
    Raw,
}

/// 已解析的上传 options（owned）。
#[derive(Debug, Clone)]
pub struct UploadOptions {
    pub headers: Vec<(String, String)>,
    pub mode: UploadMode,
    pub field: String,
    pub filename: Option<String>,
    pub text_fields: Vec<(String, String)>,
    pub timeout_ms: Option<u64>,
    /// 显式重试次数（默认 0：不自动重试非幂等 POST；每次重试重开文件并复核
    /// len/mtime，修改 → SourceChanged）。
    pub retries: u32,
}

impl Default for UploadOptions {
    fn default() -> Self {
        Self {
            headers: Vec::new(),
            mode: UploadMode::Multipart,
            field: "file".to_string(),
            filename: None,
            text_fields: Vec::new(),
            timeout_ms: None,
            retries: 0,
        }
    }
}

impl UploadOptions {
    pub(crate) fn parse(json: &str) -> Result<Self, TransferError> {
        if json.trim().is_empty() {
            return Ok(Self::default());
        }
        let raw: UploadOptionsRaw = serde_json::from_str(json).map_err(|e| {
            TransferError::new(
                TransferErrorKind::Options,
                format!("invalid upload options: {e}"),
            )
        })?;
        let mode = match raw.mode.as_deref() {
            None | Some("multipart") => UploadMode::Multipart,
            Some("raw") => UploadMode::Raw,
            Some(other) => {
                return Err(TransferError::new(
                    TransferErrorKind::Options,
                    format!("invalid mode {other:?}: expected \"multipart\"|\"raw\""),
                ))
            }
        };
        if mode == UploadMode::Raw
            && (raw.field.is_some() || raw.filename.is_some() || raw.fields.is_some())
        {
            return Err(TransferError::new(
                TransferErrorKind::Options,
                "raw mode rejects field/filename/fields",
            ));
        }
        Ok(Self {
            headers: headers_from_map(raw.headers, "upload")?,
            mode,
            field: {
                let f = raw.field.unwrap_or_else(|| "file".to_string());
                if f.trim().is_empty() {
                    return Err(TransferError::new(
                        TransferErrorKind::Options,
                        "empty field name",
                    ));
                }
                f
            },
            filename: raw.filename,
            text_fields: headers_from_map(raw.fields, "upload fields")?,
            timeout_ms: raw.timeout_ms,
            retries: raw.retries.unwrap_or(0),
        })
    }
}

fn headers_from_map(
    map: Option<std::collections::HashMap<String, String>>,
    what: &str,
) -> Result<Vec<(String, String)>, TransferError> {
    let mut out = Vec::new();
    if let Some(map) = map {
        for (k, v) in map {
            if k.trim().is_empty() {
                return Err(TransferError::new(
                    TransferErrorKind::Options,
                    format!("{what}: empty header/field name"),
                ));
            }
            out.push((k, v));
        }
    }
    Ok(out)
}

// ============================================================================
// 测试/故障注入钩子（doc(hidden)：fixture 与 T-07 矩阵专用，非公共语义）
// ============================================================================

pub type HookFn = Arc<dyn Fn() -> Result<(), String> + Send + Sync>;

/// FS 故障注入与 commit gate（`before_commit` 允许阻塞以冻结提交/取消竞态；
/// 经 spawn_blocking 调用，不占用内核 worker）。
#[derive(Default)]
pub struct TransferHooks {
    pub before_staging: Option<HookFn>,
    pub before_write: Option<HookFn>,
    pub before_flush: Option<HookFn>,
    pub before_commit: Option<HookFn>,
    pub before_cleanup: Option<HookFn>,
}

// ============================================================================
// 共享状态
// ============================================================================

/// 传输阶段（状态机；供观测/资源报告，不是公共契约）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferPhase {
    Queued,
    Opening,
    Transferring,
    Committing,
    Succeeded,
    Failed,
    Cancelled,
}

impl TransferPhase {
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            TransferPhase::Succeeded | TransferPhase::Failed | TransferPhase::Cancelled
        )
    }
}

/// 传输方向。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferKind {
    Download,
    Upload,
}

/// 源文件身份（上传重试复核：len + mtime）。
#[derive(Debug, Clone, Copy)]
pub(crate) struct SourceMeta {
    len: u64,
    mtime: Option<std::time::SystemTime>,
}

pub(crate) struct TransferShared {
    pub id: u64,
    pub target: PathBuf,
    pub phase: Mutex<TransferPhase>,
    pub progress_tx: tokio::sync::watch::Sender<Option<TransferProgress>>,
    pub progress_rx: tokio::sync::watch::Receiver<Option<TransferProgress>>,
    pub terminal_tx: tokio::sync::watch::Sender<Option<Arc<TransferReceipt>>>,
    pub terminal_rx: tokio::sync::watch::Receiver<Option<Arc<TransferReceipt>>>,
    pub cancel_tx: tokio::sync::watch::Sender<bool>,
    pub cancel_rx: tokio::sync::watch::Receiver<bool>,
    /// 上传读取流的 in-band 类型化错误（reqwest 只见 body error，这里还原）。
    pub file_error: Mutex<Option<String>>,
    pub hooks: Mutex<Option<Arc<TransferHooks>>>,
    pub source_meta: Mutex<Option<SourceMeta>>,
    /// 活动 staging 路径（清理失败时可追踪，不宣称零遗留）。
    pub staging: Mutex<Option<PathBuf>>,
    /// 终态收据已交付（next_progress 单次语义）。
    pub terminal_delivered: AtomicBool,
    /// 进度版本号（观察排序）。
    pub progress_version: AtomicU64,
    pub options_download: Mutex<Option<DownloadOptions>>,
    pub options_upload: Mutex<Option<UploadOptions>>,
}

impl TransferShared {
    pub(crate) fn phase(&self) -> TransferPhase {
        *self.phase.lock().unwrap()
    }

    pub(crate) fn set_phase(&self, p: TransferPhase) {
        if !p.is_terminal() {
            *self.phase.lock().unwrap() = p;
        }
    }

    pub(crate) fn request_cancel(&self) {
        let _ = self.cancel_tx.send(true);
    }

    pub(crate) fn cancel_requested(&self) -> bool {
        *self.cancel_rx.borrow()
    }

    pub(crate) fn set_progress(&self, p: TransferProgress) {
        let _ = self.progress_tx.send(Some(p));
        self.progress_version.fetch_add(1, Ordering::SeqCst);
    }

    /// 单次终结：首个终态胜出；终态时释放目标仲裁槽。
    pub(crate) fn finish(&self, receipt: TransferReceipt) {
        if self.terminal_rx.borrow().is_some() {
            return;
        }
        let phase = match receipt.kind {
            TransferOutcome::Success => TransferPhase::Succeeded,
            TransferOutcome::Failed => TransferPhase::Failed,
            TransferOutcome::Cancelled => TransferPhase::Cancelled,
        };
        *self.phase.lock().unwrap() = phase;
        release_target_slot(&self.target);
        let _ = self.terminal_tx.send(Some(Arc::new(receipt)));
    }

    pub fn is_terminal(&self) -> bool {
        self.terminal_rx.borrow().is_some()
    }

    fn hooks(&self) -> Option<Arc<TransferHooks>> {
        self.hooks.lock().unwrap().clone()
    }
}

// ---- 进度 / 收据（typed 值 + 跨后台 JSON 契约） ----

/// typed 进度（JSON 形态跨后台契约；total 未知时 percent 为 null）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransferProgress {
    pub bytes: u64,
    pub total: Option<u64>,
}

impl TransferProgress {
    pub fn percent(&self) -> Option<f64> {
        self.total.map(|t| {
            if t == 0 {
                // 空文件：无剩余可传，视为完成。
                100.0
            } else {
                (self.bytes as f64 / t as f64 * 100.0).clamp(0.0, 100.0)
            }
        })
    }

    pub(crate) fn json(&self) -> String {
        let total = match self.total {
            Some(t) => t.to_string(),
            None => "null".to_string(),
        };
        let percent = match self.percent() {
            Some(p) => format!("{p:.1}"),
            None => "null".to_string(),
        };
        format!(
            "{{\"kind\":\"progress\",\"bytes\":{},\"total\":{},\"percent\":{}}}",
            self.bytes, total, percent
        )
    }
}

/// typed 传输收据：`json()` 形态是跨后台契约（PLAN-727 §5.1 冻结）。
#[derive(Debug, Clone)]
pub struct TransferReceipt {
    pub kind: TransferOutcome,
    /// 上游响应状态；请求未发出/未收到响应时为 None。
    pub status: Option<u16>,
    /// 已落盘字节（下载，含续传前缀）/ 已发送字节（上传）。
    pub bytes: u64,
    /// 远端/本地总大小；未知为 None。
    pub total: Option<u64>,
    pub headers: Vec<(String, String)>,
    pub error: Option<TransferError>,
    /// 上传响应体（受 response_body_budget 约束；下载恒空）。
    pub body: Vec<u8>,
}

impl TransferReceipt {
    pub fn json(&self) -> String {
        let mut headers = String::from("{");
        for (i, (k, v)) in self.headers.iter().enumerate() {
            if i > 0 {
                headers.push(',');
            }
            headers.push_str(&format!(
                "{}:{}",
                serde_json::json!(k),
                serde_json::json!(v)
            ));
        }
        headers.push('}');
        let error = match &self.error {
            None => "null".to_string(),
            Some(e) => {
                serde_json::json!({"kind": e.kind.as_str(), "message": e.message}).to_string()
            }
        };
        let (total, status) = match (self.total, self.status) {
            (Some(t), Some(s)) => (t.to_string(), s.to_string()),
            (Some(t), None) => (t.to_string(), "null".to_string()),
            (None, Some(s)) => ("null".to_string(), s.to_string()),
            (None, None) => ("null".to_string(), "null".to_string()),
        };
        format!(
            "{{\"kind\":\"{}\",\"status\":{},\"bytes\":{},\"total\":{},\"headers\":{},\"error\":{},\"body\":{}}}",
            self.kind.as_str(),
            status,
            self.bytes,
            total,
            headers,
            error,
            serde_json::json!(String::from_utf8_lossy(&self.body).as_ref()),
        )
    }

    pub(crate) fn fail(
        status: Option<u16>,
        headers: Vec<(String, String)>,
        kind: TransferErrorKind,
        message: impl Into<String>,
    ) -> Self {
        Self {
            kind: TransferOutcome::Failed,
            status,
            bytes: 0,
            total: None,
            headers,
            error: Some(TransferError::new(kind, message)),
            body: Vec::new(),
        }
    }

    pub(crate) fn cancelled(status: Option<u16>, headers: Vec<(String, String)>) -> Self {
        Self {
            kind: TransferOutcome::Cancelled,
            status,
            bytes: 0,
            total: None,
            headers,
            error: Some(TransferError::new(
                TransferErrorKind::Cancelled,
                "transfer cancelled",
            )),
            body: Vec::new(),
        }
    }
}

// ---- 目标仲裁（同目标并发传输冲突报错） ----

fn target_key(path: &Path) -> PathBuf {
    if let Ok(p) = std::fs::canonicalize(path) {
        return p;
    }
    // 目标可能尚不存在：canonicalize 已存在的父目录，拼接文件名。
    let abs = std::env::current_dir()
        .map(|cwd| cwd.join(path))
        .unwrap_or_else(|_| path.to_path_buf());
    match path.parent().filter(|p| !p.as_os_str().is_empty()) {
        Some(parent) => match std::fs::canonicalize(parent) {
            Ok(cp) => cp.join(path.file_name().unwrap_or_default()),
            Err(_) => abs,
        },
        None => abs,
    }
}

static IN_FLIGHT_TARGETS: Mutex<Option<std::collections::HashMap<PathBuf, u64>>> = Mutex::new(None);

fn register_target_slot(target: &Path, id: u64) -> Result<(), TransferError> {
    let mut guard = IN_FLIGHT_TARGETS.lock().unwrap();
    let map = guard.get_or_insert_with(Default::default);
    let key = target_key(target);
    if map.contains_key(&key) {
        return Err(TransferError::new(
            TransferErrorKind::Conflict,
            format!("target already transferring: {}", key.display()),
        ));
    }
    map.insert(key, id);
    Ok(())
}

fn release_target_slot(target: &Path) {
    let mut guard = IN_FLIGHT_TARGETS.lock().unwrap();
    if let Some(map) = guard.as_mut() {
        map.remove(&target_key(target));
    }
}

// ---- 活跃传输登记（VM scope 级联取消入口） ----

static ACTIVE_TRANSFERS: Mutex<Option<std::collections::HashMap<u64, Arc<TransferShared>>>> =
    Mutex::new(None);

fn register_active(shared: &Arc<TransferShared>) {
    let mut guard = ACTIVE_TRANSFERS.lock().unwrap();
    guard
        .get_or_insert_with(Default::default)
        .insert(shared.id, shared.clone());
}

fn unregister_active(id: u64) {
    let mut guard = ACTIVE_TRANSFERS.lock().unwrap();
    if let Some(map) = guard.as_mut() {
        map.remove(&id);
    }
}

/// 宿主按 id 级联取消（VM scope finalize；句柄独立于注册表生命周期）。
pub fn cancel_transfer_by_id(id: u64) -> bool {
    let guard = ACTIVE_TRANSFERS.lock().unwrap();
    match guard.as_ref().and_then(|m| m.get(&id)) {
        Some(shared) => {
            shared.request_cancel();
            true
        }
        None => false,
    }
}

// ============================================================================
// 句柄与公共入口
// ============================================================================

/// 可取消文件传输句柄。最后拥有者 Drop 触发取消（幂等；终态后 no-op）。
pub struct FileTransfer {
    shared: Arc<TransferShared>,
}

impl FileTransfer {
    /// 传输 id（VM 登记与 scope 级联取消用）。
    pub fn id(&self) -> u64 {
        self.shared.id
    }

    /// 是否已终结。
    pub fn is_terminal(&self) -> bool {
        self.shared.is_terminal()
    }

    /// 当前阶段（观测）。
    pub fn phase(&self) -> TransferPhase {
        self.shared.phase()
    }

    /// 注入测试钩子（doc(hidden)：fixture/矩阵专用）。
    #[doc(hidden)]
    pub fn with_hooks(self, hooks: Arc<TransferHooks>) -> Self {
        *self.shared.hooks.lock().unwrap() = Some(hooks);
        self
    }

    fn shared(&self) -> &Arc<TransferShared> {
        &self.shared
    }

    fn request_cancel(&self) {
        self.shared.request_cancel();
    }
}

impl Drop for FileTransfer {
    fn drop(&mut self) {
        if !self.shared.is_terminal() {
            self.request_cancel();
        }
    }
}

fn next_transfer_id() -> u64 {
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// 启动下载（非阻塞提交；坏 options → 已终结的 Failed 句柄，请求不发出）。
pub fn transfer_download(url: &str, path: &str, options_json: &str) -> FileTransfer {
    start_transfer(TransferKind::Download, url, path, options_json)
}

/// 启动上传（非阻塞提交；坏 options → 已终结的 Failed 句柄）。
pub fn transfer_upload(url: &str, path: &str, options_json: &str) -> FileTransfer {
    start_transfer(TransferKind::Upload, url, path, options_json)
}

fn start_transfer(kind: TransferKind, url: &str, path: &str, options_json: &str) -> FileTransfer {
    let id = next_transfer_id();
    let target = PathBuf::from(path);

    // 静态准备：options 严格解析 + 目标仲裁。
    let prep = prepare(kind, id, url, &target, options_json);
    let shared = match prep {
        Ok(s) => s,
        Err((s, err)) => {
            s.finish(TransferReceipt {
                kind: TransferOutcome::Failed,
                status: None,
                bytes: 0,
                total: None,
                headers: Vec::new(),
                error: Some(err),
                body: Vec::new(),
            });
            return FileTransfer { shared: s };
        }
    };

    // 队列许可（try）：满 → 终结性 QueueFull（提交即拒绝）。
    let Ok(queue_slot) = Arc::clone(super::client::transfer_queue()).try_acquire_owned() else {
        shared.finish(TransferReceipt::fail(
            None,
            Vec::new(),
            TransferErrorKind::QueueFull,
            "transfer queue full",
        ));
        return FileTransfer { shared };
    };

    register_active(&shared);
    let url = url.to_string();
    let job_shared = shared.clone();
    super::client::kernel_handle().spawn(async move {
        let _queue_slot = queue_slot;
        let receipt = match kind {
            TransferKind::Download => run_download(job_shared.clone(), url).await,
            TransferKind::Upload => run_upload(job_shared.clone(), url).await,
        };
        job_shared.finish(receipt);
        unregister_active(job_shared.id);
    });
    FileTransfer { shared }
}

fn prepare(
    kind: TransferKind,
    id: u64,
    url: &str,
    target: &Path,
    options_json: &str,
) -> Result<Arc<TransferShared>, (Arc<TransferShared>, TransferError)> {
    let shared = make_shared(kind, id, target);
    if url.trim().is_empty() {
        return Err((
            shared,
            TransferError::new(TransferErrorKind::Options, "empty url"),
        ));
    }
    if target.as_os_str().is_empty() {
        return Err((
            shared,
            TransferError::new(TransferErrorKind::Options, "empty path"),
        ));
    }
    match kind {
        TransferKind::Download => match DownloadOptions::parse(options_json) {
            Ok(o) => *shared.options_download.lock().unwrap() = Some(o),
            Err(e) => return Err((shared, e)),
        },
        TransferKind::Upload => match UploadOptions::parse(options_json) {
            Ok(o) => *shared.options_upload.lock().unwrap() = Some(o),
            Err(e) => return Err((shared, e)),
        },
    }
    if let Err(e) = register_target_slot(target, id) {
        return Err((shared, e));
    }
    Ok(shared)
}

fn make_shared(_kind: TransferKind, id: u64, target: &Path) -> Arc<TransferShared> {
    let (ptx, prx) = tokio::sync::watch::channel(None);
    let (ttx, trx) = tokio::sync::watch::channel(None);
    let (ctx, crx) = tokio::sync::watch::channel(false);
    Arc::new(TransferShared {
        id,
        target: target.to_path_buf(),
        phase: Mutex::new(TransferPhase::Queued),
        progress_tx: ptx,
        progress_rx: prx,
        terminal_tx: ttx,
        terminal_rx: trx,
        cancel_tx: ctx,
        cancel_rx: crx,
        file_error: Mutex::new(None),
        hooks: Mutex::new(None),
        source_meta: Mutex::new(None),
        staging: Mutex::new(None),
        terminal_delivered: AtomicBool::new(false),
        progress_version: AtomicU64::new(0),
        options_download: Mutex::new(None),
        options_upload: Mutex::new(None),
    })
}

// ============================================================================
// 公共查询面（自由函数；Auto/Rust/VM 同词汇）
// ============================================================================

/// 同步桥接等待终态收据（JSON）。仅允许阻塞的同步边界；async 用
/// [`transfer_wait_async`]（响亮边界：async 上下文调用会 panic）。
pub fn transfer_wait(t: &FileTransfer) -> String {
    super::client::kernel_handle().block_on(wait_receipt(t.shared()))
}

/// async 等待终态收据（JSON）。**丢弃本 future = 取消传输**（结构化取消，
/// 与内核 execute 同语义）；progress 观察不取消传输。
pub async fn transfer_wait_async(t: &FileTransfer) -> String {
    let guard = WaitCancelGuard { shared: t.shared() };
    let json = wait_receipt(t.shared()).await;
    drop(guard); // 完成后 is_terminal → no-op
    json
}

/// 结构化取消守卫：中途丢弃 wait future → 取消传输。
struct WaitCancelGuard<'a> {
    shared: &'a Arc<TransferShared>,
}

impl Drop for WaitCancelGuard<'_> {
    fn drop(&mut self) {
        if !self.shared.is_terminal() {
            self.shared.request_cancel();
        }
    }
}

async fn wait_receipt(shared: &Arc<TransferShared>) -> String {
    let mut rx = shared.terminal_rx.clone();
    loop {
        if let Some(r) = rx.borrow().clone() {
            return r.json();
        }
        if rx.changed().await.is_err() {
            // 发送端消失（内核关闭）：以取消收据兜底。
            return TransferReceipt::cancelled(None, Vec::new()).json();
        }
    }
}

/// 同步等待 typed 收据（legacy adapter 与原生 Rust 消费者面；async 上下文
/// 用 [`transfer_wait_async`]）。
pub fn transfer_wait_typed(t: &FileTransfer) -> TransferReceipt {
    let shared = t.shared();
    super::client::kernel_handle().block_on(async {
        let mut rx = shared.terminal_rx.clone();
        loop {
            if let Some(r) = rx.borrow().clone() {
                return (*r).clone();
            }
            if rx.changed().await.is_err() {
                return TransferReceipt::cancelled(None, Vec::new());
            }
        }
    })
}

/// 读取最新进度（JSON）："" = 暂无新内容；进度保留最新值（可合并）；
/// 终态收据单次交付（不因查询丢失）。
pub fn transfer_next_progress(t: &FileTransfer) -> String {
    let shared = t.shared();
    if shared.is_terminal() {
        if shared.terminal_delivered.swap(true, Ordering::SeqCst) {
            return String::new();
        }
        if let Some(r) = shared.terminal_rx.borrow().clone() {
            return r.json();
        }
        return String::new();
    }
    match shared.progress_rx.borrow().clone() {
        Some(p) => p.json(),
        None => String::new(),
    }
}

/// 终结错误查询："" = 无错误/未终结。
pub fn transfer_error(t: &FileTransfer) -> String {
    match t.shared().terminal_rx.borrow().clone() {
        Some(r) => r
            .error
            .as_ref()
            .map(|e| e.message.clone())
            .unwrap_or_default(),
        None => String::new(),
    }
}

/// 显式取消（幂等；在途 FS 操作收口后清理 staging，原目标保持）。
pub fn transfer_cancel(t: &FileTransfer) {
    t.request_cancel();
}

// ============================================================================
// 下载执行
// ============================================================================

async fn run_download(shared: Arc<TransferShared>, url: String) -> TransferReceipt {
    let opts = shared
        .options_download
        .lock()
        .unwrap()
        .clone()
        .unwrap_or_default();
    let limits = super::client::transfer_limits();
    let deadline = tokio::time::Instant::now()
        + Duration::from_millis(
            opts.timeout_ms
                .unwrap_or(limits.total_timeout.as_millis() as u64),
        );

    shared.set_phase(TransferPhase::Opening);
    let _active = match acquire_active(&shared, deadline).await {
        Ok(s) => s,
        Err(receipt) => return receipt,
    };

    // ---- 本地前置：续传对齐 / 已存在目标策略 ----
    let offset = opts.offset.unwrap_or(0);
    let mut resume = false;
    if offset > 0 {
        match std::fs::metadata(&shared.target) {
            Ok(m) if m.len() == offset => resume = true,
            Ok(m) => {
                return TransferReceipt::fail(
                    None,
                    Vec::new(),
                    TransferErrorKind::File,
                    format!(
                        "resume offset {offset} does not match local length {}",
                        m.len()
                    ),
                )
            }
            Err(e) => {
                return TransferReceipt::fail(
                    None,
                    Vec::new(),
                    TransferErrorKind::File,
                    format!("resume target missing: {e}"),
                )
            }
        }
    }
    if opts.on_exists == OnExists::Fail && !resume && shared.target.exists() {
        return TransferReceipt::fail(
            None,
            Vec::new(),
            TransferErrorKind::File,
            format!("target exists: {}", shared.target.display()),
        );
    }

    // ---- 建立请求 ----
    let mut builder = super::client::shared_http_client().get(&url);
    for (k, v) in &opts.headers {
        builder = builder.header(k.as_str(), v.as_str());
    }
    if resume {
        // identity：防止解压后字节位置漂移（PLAN-727 §5.2）。
        builder = builder.header("Range", format!("bytes={offset}-"));
        builder = builder.header("Accept-Encoding", "identity");
        match &opts.validator {
            Some(Validator::Etag(etag)) => {
                builder = builder.header("If-Range", etag.as_str());
            }
            Some(Validator::LastModified(m)) => {
                builder = builder.header("If-Range", m.as_str());
            }
            None => {
                // 无 validator：不能证明远端版本一致（新面默认完整重下的例外
                // 仅当 206 且 Content-Range 严格对齐；版本一致性不承诺）。
            }
        }
    }
    let send = builder.send();
    tokio::pin!(send);
    let resp = tokio::select! {
        r = &mut send => match r {
            Ok(resp) => resp,
            Err(e) if e.is_timeout() => return TransferReceipt::fail(None, Vec::new(), TransferErrorKind::Timeout, "transfer open timed out"),
            Err(e) => return TransferReceipt::fail(None, Vec::new(), TransferErrorKind::Transport, format!("transfer open failed: {e}")),
        },
        _ = wait_cancel(&shared) => return TransferReceipt::cancelled(None, Vec::new()),
        _ = tokio::time::sleep_until(deadline) => return TransferReceipt::fail(None, Vec::new(), TransferErrorKind::Timeout, "total deadline while opening"),
    };
    let status = resp.status().as_u16();
    let headers: Vec<(String, String)> = resp
        .headers()
        .iter()
        .filter_map(|(k, v)| Some((k.to_string(), v.to_str().ok()?.to_string())))
        .collect();

    // ---- 状态验证先行：非 2xx = 失败（原目标保持） ----
    if !(200..300).contains(&status) {
        return TransferReceipt::fail(
            Some(status),
            headers,
            TransferErrorKind::HttpStatus,
            format!("upstream status {status}"),
        );
    }

    // ---- 续传响应判定 ----
    let mut total_size: Option<u64> = resp.content_length();
    let mut prefix_len = 0u64;
    if resume {
        if status == 206 {
            let cr = resp
                .headers()
                .get("Content-Range")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");
            let Some((start, end, cr_total)) = parse_content_range(cr) else {
                return TransferReceipt::fail(
                    Some(status),
                    headers,
                    TransferErrorKind::HttpStatus,
                    format!("206 missing/invalid Content-Range: {cr:?}"),
                );
            };
            if start != offset {
                return TransferReceipt::fail(
                    Some(status),
                    headers,
                    TransferErrorKind::HttpStatus,
                    format!("Content-Range start {start} != requested offset {offset}"),
                );
            }
            prefix_len = offset;
            total_size = cr_total.or(Some(end + 1));
        } else {
            // 200：上游忽略 Range 或 If-Range 失配 → 完整重启（绝不 append）。
            resume = false;
            prefix_len = 0;
        }
    }

    shared.set_phase(TransferPhase::Transferring);

    // ---- staging 写盘管道 ----
    let staging = staging_path(&shared.target, shared.id);
    *shared.staging.lock().unwrap() = Some(staging.clone());
    let max_bytes = opts.max_bytes.unwrap_or(limits.file_budget_bytes);
    let idle = Duration::from_millis(
        opts.idle_timeout_ms
            .unwrap_or(limits.idle_timeout.as_millis() as u64),
    );

    let (tx, rx) = tokio::sync::mpsc::channel::<WriteCmd>(limits.max_pending_blocks);
    let writer = super::client::kernel_handle().spawn(writer_task(
        shared.clone(),
        rx,
        staging.clone(),
        resume,
        offset,
        limits.app_block_bytes,
        Arc::clone(super::client::fs_ops()),
    ));

    // 生产端：网络 bytes_stream → 有限块 → 背压通道（满 = 慢磁盘反压网络）。
    use futures::StreamExt;
    let mut upstream = resp.bytes_stream();
    let mut received: u64 = 0;
    let mut producer_error: Option<TransferError> = None;
    loop {
        let chunk = tokio::select! {
            c = upstream.next() => match c {
                Some(Ok(bytes)) => bytes,
                Some(Err(e)) => {
                    producer_error = Some(TransferError::new(
                        TransferErrorKind::Transport,
                        format!("download read: {e}"),
                    ));
                    break;
                }
                None => break, // 上游 EOF（干净终结）
            },
            _ = wait_cancel(&shared) => {
                producer_error = Some(TransferError::new(TransferErrorKind::Cancelled, "transfer cancelled"));
                break;
            }
            _ = tokio::time::sleep_until(deadline) => {
                producer_error = Some(TransferError::new(TransferErrorKind::Timeout, "total deadline while transferring"));
                break;
            }
            _ = tokio::time::sleep(idle) => {
                producer_error = Some(TransferError::new(TransferErrorKind::Timeout, format!("upstream idle {idle:?} during transfer")));
                break;
            }
        };
        for block in chunk.chunks(limits.app_block_bytes) {
            if prefix_len + received + block.len() as u64 > max_bytes {
                producer_error = Some(TransferError::new(
                    TransferErrorKind::Budget,
                    format!("download exceeds file budget {max_bytes}"),
                ));
                break;
            }
            received += block.len() as u64;
            let send_block = tx.send(WriteCmd::Data(block.to_vec()));
            tokio::select! {
                r = send_block => {
                    if r.is_err() {
                        producer_error = Some(TransferError::new(TransferErrorKind::File, "download writer closed unexpectedly"));
                        break;
                    }
                }
                _ = wait_cancel(&shared) => {
                    producer_error = Some(TransferError::new(TransferErrorKind::Cancelled, "transfer cancelled"));
                    break;
                }
            }
        }
        if producer_error.is_some() {
            break;
        }
        shared.set_progress(TransferProgress {
            bytes: prefix_len + received,
            total: total_size,
        });
    }
    if producer_error.is_none() {
        // 干净 EOF → 显式 Finish 标记（writer 仅在此后才允许提交）。
        let _ = tx.send(WriteCmd::Finish).await;
    }
    drop(tx);
    let writer_outcome = match writer.await {
        Ok(outcome) => outcome,
        Err(join_err) => WriterOutcome::Failed(TransferError::new(
            TransferErrorKind::File,
            format!("download writer task failed: {join_err}"),
        )),
    };

    // ---- 收据合成 ----
    match writer_outcome {
        WriterOutcome::Committed { bytes } => {
            shared.set_progress(TransferProgress {
                bytes,
                total: total_size,
            });
            TransferReceipt {
                kind: TransferOutcome::Success,
                status: Some(status),
                bytes,
                total: total_size,
                headers,
                error: None,
                body: Vec::new(),
            }
        }
        _ => {
            // 未提交：producer 端错误（传输/预算/超时/取消）优先于 writer
            // 放弃细节——writer 的 Cancelled 只是「未达 Finish」的形态。
            let (kind, message) = match (&producer_error, &writer_outcome) {
                (Some(pe), _) => (pe.kind, pe.message.clone()),
                (None, WriterOutcome::Failed(e)) => (e.kind, e.message.clone()),
                _ => (
                    TransferErrorKind::Cancelled,
                    "transfer cancelled".to_string(),
                ),
            };
            if kind == TransferErrorKind::Cancelled {
                TransferReceipt::cancelled(Some(status), headers)
            } else {
                TransferReceipt::fail(Some(status), headers, kind, message)
            }
        }
    }
}

// ============================================================================
// 下载写盘任务
// ============================================================================

/// 生产端 → 写盘端命令。`Finish` 是唯一允许进入提交路径的信号；
/// 通道无 Finish 关闭（producer 错误/取消 drop）= 放弃提交。
enum WriteCmd {
    Data(Vec<u8>),
    Finish,
}

enum WriterOutcome {
    Committed { bytes: u64 },
    Cancelled,
    Failed(TransferError),
}

/// 下载写盘任务：同目录独占 staging →（续传）前缀分块复制 → 追加网络块 →
/// flush/sync → commit(rename)。取消 = 停止新写、等在途写收口、清理 staging；
/// commit gate 通过后的迟到取消不回滚（替换已成功 = 成功）。
#[allow(clippy::too_many_arguments)]
async fn writer_task(
    shared: Arc<TransferShared>,
    mut rx: tokio::sync::mpsc::Receiver<WriteCmd>,
    staging: PathBuf,
    resume: bool,
    offset: u64,
    block_bytes: usize,
    fs_ops: Arc<tokio::sync::Semaphore>,
) -> WriterOutcome {
    // 单 FS 许可覆盖整个文件阶段：内核级并发磁盘传输上限 = fs_ops_max。
    let _permit = match fs_ops.acquire().await {
        Ok(p) => p,
        Err(_) => {
            return WriterOutcome::Failed(TransferError::new(
                TransferErrorKind::Transport,
                "transfer executor closed",
            ))
        }
    };
    let hooks = || shared.hooks();

    if let Some(h) = hooks().and_then(|h| h.before_staging.clone()) {
        if let Err(e) = h() {
            return WriterOutcome::Failed(TransferError::new(
                TransferErrorKind::File,
                format!("injected staging failure: {e}"),
            ));
        }
    }
    if shared.cancel_requested() {
        return cleanup_staging(&shared, staging, WriterOutcome::Cancelled).await;
    }
    let mut file = match tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&staging)
        .await
    {
        Ok(f) => f,
        Err(e) => {
            return WriterOutcome::Failed(TransferError::new(
                TransferErrorKind::File,
                format!("staging create failed: {e}"),
            ))
        }
    };

    // 续传前缀：分块复制旧目标前 offset 字节到 staging（内存不随前缀增长）。
    if resume {
        let mut src = match tokio::fs::File::open(&shared.target).await {
            Ok(f) => f,
            Err(e) => {
                return abandon(
                    &shared,
                    file,
                    staging,
                    TransferError::new(
                        TransferErrorKind::File,
                        format!("resume prefix open failed: {e}"),
                    ),
                )
                .await
            }
        };
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let mut buf = vec![0u8; block_bytes];
        let mut remaining = offset;
        loop {
            if remaining == 0 {
                break;
            }
            if shared.cancel_requested() {
                return abandon(
                    &shared,
                    file,
                    staging,
                    TransferError::new(TransferErrorKind::Cancelled, "transfer cancelled"),
                )
                .await;
            }
            let want = buf.len().min(remaining as usize);
            match src.read(&mut buf[..want]).await {
                Ok(0) => {
                    return abandon(
                        &shared,
                        file,
                        staging,
                        TransferError::new(
                            TransferErrorKind::File,
                            "resume prefix shorter than offset (file changed)",
                        ),
                    )
                    .await;
                }
                Ok(n) => {
                    if let Err(e) = file.write_all(&buf[..n]).await {
                        return abandon(
                            &shared,
                            file,
                            staging,
                            TransferError::new(
                                TransferErrorKind::File,
                                format!("prefix copy write failed: {e}"),
                            ),
                        )
                        .await;
                    }
                    remaining -= n as u64;
                }
                Err(e) => {
                    return abandon(
                        &shared,
                        file,
                        staging,
                        TransferError::new(
                            TransferErrorKind::File,
                            format!("prefix copy read failed: {e}"),
                        ),
                    )
                    .await;
                }
            }
        }
    }

    // 消费网络块。`Finish` 是唯一允许进入提交路径的信号；通道在 Finish 前
    // 关闭（producer 错误/取消）= 放弃提交。
    use tokio::io::AsyncWriteExt;
    let mut written: u64 = if resume { offset } else { 0 };
    let mut abort: Option<WriterOutcome> = None;
    let mut producer_finished = false;
    while let Some(cmd) = rx.recv().await {
        match cmd {
            WriteCmd::Data(block) => {
                if let Some(h) = hooks().and_then(|h| h.before_write.clone()) {
                    if let Err(e) = h() {
                        abort = Some(WriterOutcome::Failed(TransferError::new(
                            TransferErrorKind::File,
                            format!("injected write failure: {e}"),
                        )));
                        break;
                    }
                }
                if shared.cancel_requested() {
                    abort = Some(WriterOutcome::Cancelled);
                    break;
                }
                match file.write_all(&block).await {
                    Ok(()) => written += block.len() as u64,
                    Err(e) => {
                        abort = Some(WriterOutcome::Failed(TransferError::new(
                            TransferErrorKind::File,
                            format!("staging write failed: {e}"),
                        )));
                        break;
                    }
                }
            }
            WriteCmd::Finish => {
                producer_finished = true;
                break;
            }
        }
    }

    if let Some(outcome) = abort {
        return abandon(
            &shared,
            file,
            staging,
            match outcome {
                WriterOutcome::Failed(e) => e,
                _ => TransferError::new(TransferErrorKind::Cancelled, "transfer cancelled"),
            },
        )
        .await;
    }
    if !producer_finished {
        // producer 未发 Finish 就关通道 → 干净 EOF 未达成，一律不提交。
        return abandon(
            &shared,
            file,
            staging,
            TransferError::new(
                TransferErrorKind::Cancelled,
                "download aborted before finish",
            ),
        )
        .await;
    }

    // ---- 提交路径 ----
    shared.set_phase(TransferPhase::Committing);
    if let Some(h) = hooks().and_then(|h| h.before_flush.clone()) {
        if let Err(e) = h() {
            return abandon(
                &shared,
                file,
                staging,
                TransferError::new(
                    TransferErrorKind::File,
                    format!("injected flush failure: {e}"),
                ),
            )
            .await;
        }
    }
    if let Err(e) = file.flush().await {
        return abandon(
            &shared,
            file,
            staging,
            TransferError::new(
                TransferErrorKind::File,
                format!("staging flush failed: {e}"),
            ),
        )
        .await;
    }
    if let Err(e) = file.sync_all().await {
        return abandon(
            &shared,
            file,
            staging,
            TransferError::new(TransferErrorKind::File, format!("staging sync failed: {e}")),
        )
        .await;
    }
    drop(file); // Windows：替换打开中的文件必败，先落柄再 rename

    // commit gate（可能阻塞；spawn_blocking 执行，不占内核 worker）。
    if let Some(h) = hooks().and_then(|h| h.before_commit.clone()) {
        let gate = tokio::task::spawn_blocking(move || h());
        match gate.await {
            Ok(Ok(())) => {}
            Ok(Err(e)) => {
                return cleanup_staging(
                    &shared,
                    staging,
                    WriterOutcome::Failed(TransferError::new(
                        TransferErrorKind::File,
                        format!("injected commit failure: {e}"),
                    )),
                )
                .await
            }
            Err(join_err) => {
                return cleanup_staging(
                    &shared,
                    staging,
                    WriterOutcome::Failed(TransferError::new(
                        TransferErrorKind::File,
                        format!("commit gate failed: {join_err}"),
                    )),
                )
                .await
            }
        }
    }
    // gate 之后 = 提交区：此后的迟到取消不回滚（替换成功 = 成功）。
    // Windows：std::fs::rename = MoveFileExW(REPLACE_EXISTING)——不先删原文件
    // （见 src/state_file.rs:10 既有记载）；失败原目标保持。
    match tokio::fs::rename(&staging, &shared.target).await {
        Ok(()) => {
            *shared.staging.lock().unwrap() = None;
            WriterOutcome::Committed { bytes: written }
        }
        Err(e) => {
            cleanup_staging(
                &shared,
                staging,
                WriterOutcome::Failed(TransferError::new(
                    TransferErrorKind::File,
                    format!("commit replace failed: {e}"),
                )),
            )
            .await
        }
    }
}

/// 放弃：等在途写收口（本任务即写者；已完成的写不回滚），关柄、清理 staging。
async fn abandon(
    shared: &Arc<TransferShared>,
    file: tokio::fs::File,
    staging: PathBuf,
    err: TransferError,
) -> WriterOutcome {
    drop(file);
    let outcome = if err.kind == TransferErrorKind::Cancelled {
        WriterOutcome::Cancelled
    } else {
        WriterOutcome::Failed(err)
    };
    cleanup_staging(shared, staging, outcome).await
}

/// staging 清理：失败保留可追踪路径与错误（不宣称零遗留）。
async fn cleanup_staging(
    shared: &Arc<TransferShared>,
    staging: PathBuf,
    mut outcome: WriterOutcome,
) -> WriterOutcome {
    if let Some(h) = shared.hooks().and_then(|h| h.before_cleanup.clone()) {
        if let Err(e) = h() {
            let note = format!(
                " (injected cleanup failure: {e}; leftover staging at {})",
                staging.display()
            );
            outcome = match outcome {
                WriterOutcome::Failed(err) => WriterOutcome::Failed(TransferError::new(
                    err.kind,
                    format!("{}{note}", err.message),
                )),
                WriterOutcome::Cancelled => WriterOutcome::Failed(TransferError::new(
                    TransferErrorKind::Cancelled,
                    format!("transfer cancelled;{note}"),
                )),
                committed => committed,
            };
            return outcome;
        }
    }
    match tokio::fs::remove_file(&staging).await {
        Ok(()) => {
            *shared.staging.lock().unwrap() = None;
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            *shared.staging.lock().unwrap() = None;
        }
        Err(e) => {
            let note = format!(
                " (staging cleanup failed: {e}; leftover at {})",
                staging.display()
            );
            outcome = match outcome {
                WriterOutcome::Failed(err) => WriterOutcome::Failed(TransferError::new(
                    err.kind,
                    format!("{}{note}", err.message),
                )),
                WriterOutcome::Cancelled => WriterOutcome::Failed(TransferError::new(
                    TransferErrorKind::Cancelled,
                    format!("transfer cancelled;{note}"),
                )),
                committed => committed,
            };
        }
    }
    outcome
}

// ============================================================================
// 上传执行
// ============================================================================

async fn run_upload(shared: Arc<TransferShared>, url: String) -> TransferReceipt {
    let opts = shared
        .options_upload
        .lock()
        .unwrap()
        .clone()
        .unwrap_or_default();
    let limits = super::client::transfer_limits();
    let deadline = tokio::time::Instant::now()
        + Duration::from_millis(
            opts.timeout_ms
                .unwrap_or(limits.total_timeout.as_millis() as u64),
        );

    shared.set_phase(TransferPhase::Opening);
    let _active = match acquire_active(&shared, deadline).await {
        Ok(s) => s,
        Err(receipt) => return receipt,
    };

    // ---- 源文件前置：存在、非常规目录/设备、预算内；身份记录供重试复核 ----
    let meta = match std::fs::metadata(&shared.target) {
        Ok(m) => m,
        Err(e) => {
            return TransferReceipt::fail(
                None,
                Vec::new(),
                TransferErrorKind::File,
                format!("source file missing: {e}"),
            )
        }
    };
    if !meta.is_file() {
        return TransferReceipt::fail(
            None,
            Vec::new(),
            TransferErrorKind::File,
            format!(
                "source is not a regular file (dir/device/symlink target rejected): {}",
                shared.target.display()
            ),
        );
    }
    if meta.len() > limits.file_budget_bytes {
        return TransferReceipt::fail(
            None,
            Vec::new(),
            TransferErrorKind::Budget,
            format!(
                "source file {} exceeds file budget {}",
                meta.len(),
                limits.file_budget_bytes
            ),
        );
    }
    let src_meta = SourceMeta {
        len: meta.len(),
        mtime: meta.modified().ok(),
    };
    *shared.source_meta.lock().unwrap() = Some(src_meta);

    shared.set_phase(TransferPhase::Transferring);
    let max_attempts = 1 + opts.retries as usize;
    let mut attempt = 0usize;
    loop {
        attempt += 1;
        let receipt = upload_attempt(&shared, &url, &opts, src_meta, deadline, limits).await;
        let retryable = matches!(
            receipt.error.as_ref().map(|e| e.kind),
            Some(TransferErrorKind::Transport) | Some(TransferErrorKind::Timeout)
        ) && receipt.kind == TransferOutcome::Failed;
        if retryable && attempt < max_attempts && !shared.cancel_requested() {
            continue; // 每次重试重开文件并复核身份（upload_attempt 内做）
        }
        return receipt;
    }
}

#[allow(clippy::too_many_arguments)]
async fn upload_attempt(
    shared: &Arc<TransferShared>,
    url: &str,
    opts: &UploadOptions,
    src_meta: SourceMeta,
    deadline: tokio::time::Instant,
    limits: TransferLimits,
) -> TransferReceipt {
    // 重试复核：源文件 len/mtime 必须与首测一致（修改 → SourceChanged）。
    if let Ok(m) = std::fs::metadata(&shared.target) {
        let now = SourceMeta {
            len: m.len(),
            mtime: m.modified().ok(),
        };
        if now.len != src_meta.len || now.mtime != src_meta.mtime {
            return TransferReceipt::fail(
                None,
                Vec::new(),
                TransferErrorKind::SourceChanged,
                format!(
                    "source file changed between attempts (len {} -> {}, mtime changed: {})",
                    src_meta.len,
                    now.len,
                    now.mtime != src_meta.mtime
                ),
            );
        }
    } else {
        return TransferReceipt::fail(
            None,
            Vec::new(),
            TransferErrorKind::SourceChanged,
            "source file disappeared between attempts",
        );
    }

    let (tx, rx) = tokio::sync::mpsc::channel::<Vec<u8>>(limits.max_pending_blocks);
    let reader_shared = shared.clone();
    let block = limits.app_block_bytes;
    let fs_ops = super::client::fs_ops();
    let reader = super::client::kernel_handle().spawn(upload_reader_task(
        reader_shared,
        tx,
        block,
        Arc::clone(&fs_ops),
    ));

    // 请求构造：raw 体 / multipart（文件 part 流式，保持路径描述直到发送）。
    let mut builder = super::client::shared_http_client().post(url);
    for (k, v) in &opts.headers {
        builder = builder.header(k.as_str(), v.as_str());
    }
    let file_len = src_meta.len;
    let body_stream = RxStream { rx };
    match opts.mode {
        UploadMode::Raw => {
            builder = builder.header("Content-Type", "application/octet-stream");
            // 流式体 + 显式长度：hyper 按给定长度发送（否则退化为 chunked）。
            builder = builder.header("Content-Length", file_len);
            builder = builder.body(reqwest::Body::wrap_stream(body_stream));
        }
        UploadMode::Multipart => {
            let filename = opts
                .filename
                .clone()
                .unwrap_or_else(|| default_filename(&shared.target));
            let part = reqwest::multipart::Part::stream_with_length(
                reqwest::Body::wrap_stream(body_stream),
                file_len,
            )
            .file_name(filename.clone())
            .mime_str(guess_mime(&filename))
            .expect("static mime type is always valid");
            let mut form = reqwest::multipart::Form::new();
            for (k, v) in &opts.text_fields {
                form = form.text(k.clone(), v.clone());
            }
            form = form.part(opts.field.clone(), part);
            builder = builder.multipart(form);
        }
    }

    let send = builder.send();
    tokio::pin!(send);
    let resp = tokio::select! {
        r = &mut send => match r {
            Ok(resp) => resp,
            Err(e) => {
                // reader 的 in-band 文件错误优先于传输错误分类。
                let reader_result = reader.await.ok();
                if let Some(file_err) = shared.file_error.lock().unwrap().clone() {
                    return TransferReceipt::fail(None, Vec::new(), TransferErrorKind::File, file_err);
                }
                let _ = reader_result;
                let kind = if e.is_timeout() { TransferErrorKind::Timeout } else { TransferErrorKind::Transport };
                return TransferReceipt::fail(None, Vec::new(), kind, format!("upload send failed: {e}"));
            }
        },
        _ = wait_cancel(shared) => {
            reader.abort();
            return TransferReceipt::cancelled(None, Vec::new());
        },
        _ = tokio::time::sleep_until(deadline) => {
            reader.abort();
            return TransferReceipt::fail(None, Vec::new(), TransferErrorKind::Timeout, "total deadline while uploading");
        },
    };
    let status = resp.status().as_u16();
    let headers: Vec<(String, String)> = resp
        .headers()
        .iter()
        .filter_map(|(k, v)| Some((k.to_string(), v.to_str().ok()?.to_string())))
        .collect();
    // 等读取任务收口（读错误在 file_error；成功时 join 返回 Ok）。
    let _ = reader.await;
    if let Some(file_err) = shared.file_error.lock().unwrap().clone() {
        return TransferReceipt::fail(Some(status), headers, TransferErrorKind::File, file_err);
    }

    // 有界响应体。
    let body = match read_body_capped(resp, limits.response_body_budget, deadline).await {
        Ok(b) => b,
        Err(kind_msg) => {
            return TransferReceipt::fail(Some(status), headers, kind_msg.0, kind_msg.1)
        }
    };

    if (200..300).contains(&status) {
        TransferReceipt {
            kind: TransferOutcome::Success,
            status: Some(status),
            bytes: file_len,
            total: Some(file_len),
            headers,
            error: None,
            body,
        }
    } else {
        TransferReceipt::fail(
            Some(status),
            headers,
            TransferErrorKind::HttpStatus,
            format!("upstream status {status}"),
        )
    }
}

/// 上传读取任务：重开文件 → 有限块读取（≤64KiB，待处理 ≤2 背压）→ EOF。
/// 读失败：设置 in-band file_error 并终止（不省略、不跳过 part）。
async fn upload_reader_task(
    shared: Arc<TransferShared>,
    tx: tokio::sync::mpsc::Sender<Vec<u8>>,
    block_bytes: usize,
    fs_ops: Arc<tokio::sync::Semaphore>,
) {
    let _permit = match fs_ops.acquire().await {
        Ok(p) => p,
        Err(_) => return,
    };
    let mut file = match tokio::fs::File::open(&shared.target).await {
        Ok(f) => f,
        Err(e) => {
            *shared.file_error.lock().unwrap() = Some(format!("source open failed: {e}"));
            return;
        }
    };
    use tokio::io::AsyncReadExt;
    let mut buf = vec![0u8; block_bytes];
    loop {
        if shared.cancel_requested() {
            return;
        }
        let n = match file.read(&mut buf).await {
            Ok(0) => break, // EOF：正常关通道（截断由 Content-Length 检测）
            Ok(n) => n,
            Err(e) => {
                *shared.file_error.lock().unwrap() = Some(format!("source read failed: {e}"));
                return; // 不发 Finish：流提前结束 → reqwest 侧长度失配报错
            }
        };
        let send = tx.send(buf[..n].to_vec());
        tokio::select! {
            r = send => {
                if r.is_err() {
                    return; // 消费端消失（请求失败/取消）
                }
            }
            _ = wait_cancel(&shared) => return,
        }
    }
}

/// mpsc → 字节流适配（Vec<u8> 实现 Into<Bytes>；错误经 file_error 带外还原）。
struct RxStream {
    rx: tokio::sync::mpsc::Receiver<Vec<u8>>,
}

impl futures::Stream for RxStream {
    type Item = Result<Vec<u8>, std::io::Error>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        match self.rx.poll_recv(cx) {
            std::task::Poll::Ready(Some(v)) => std::task::Poll::Ready(Some(Ok(v))),
            std::task::Poll::Ready(None) => std::task::Poll::Ready(None),
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}

/// 有界响应体读取：(错误种类, 消息)。
async fn read_body_capped(
    resp: reqwest::Response,
    cap: usize,
    deadline: tokio::time::Instant,
) -> Result<Vec<u8>, (TransferErrorKind, String)> {
    if let Some(len) = resp.content_length() {
        if len > cap as u64 {
            return Err((
                TransferErrorKind::Budget,
                format!("response body exceeds budget {cap}"),
            ));
        }
    }
    use futures::StreamExt;
    let mut out: Vec<u8> = Vec::new();
    let mut stream = resp.bytes_stream();
    loop {
        let chunk = tokio::select! {
            c = stream.next() => match c {
                Some(Ok(b)) => b,
                Some(Err(e)) => return Err((TransferErrorKind::Transport, format!("response read: {e}"))),
                None => break,
            },
            _ = tokio::time::sleep_until(deadline) => {
                return Err((TransferErrorKind::Timeout, "total deadline while reading response".to_string()));
            }
        };
        if out.len() + chunk.len() > cap {
            return Err((
                TransferErrorKind::Budget,
                format!("response body exceeds budget {cap}"),
            ));
        }
        out.extend_from_slice(&chunk);
    }
    Ok(out)
}

// ============================================================================
// 共享工具
// ============================================================================

async fn acquire_active(
    shared: &Arc<TransferShared>,
    deadline: tokio::time::Instant,
) -> Result<tokio::sync::OwnedSemaphorePermit, TransferReceipt> {
    let acquire = Arc::clone(super::client::transfer_active()).acquire_owned();
    tokio::pin!(acquire);
    tokio::select! {
        slot = &mut acquire => slot.map_err(|_| {
            TransferReceipt::fail(None, Vec::new(), TransferErrorKind::Transport, "transfer executor closed")
        }),
        _ = wait_cancel(shared) => Err(TransferReceipt::cancelled(None, Vec::new())),
        _ = tokio::time::sleep_until(deadline) => Err(TransferReceipt::fail(
            None, Vec::new(), TransferErrorKind::Timeout, "total deadline while queued",
        )),
    }
}

/// 等取消信号（select 臂）。
async fn wait_cancel(shared: &TransferShared) {
    let mut rx = shared.cancel_rx.clone();
    loop {
        if *rx.borrow() {
            return;
        }
        if rx.changed().await.is_err() {
            // 发送端随 shared 消失：不再取消。
            std::future::pending::<()>().await;
        }
    }
}

/// 同目录独占 staging 路径（同卷保证 rename 原子性）。
fn staging_path(target: &Path, id: u64) -> PathBuf {
    let name = target
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "download".to_string());
    let dir = target
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    dir.join(format!(".{name}.plan727-{id}.part"))
}

/// Content-Range: `bytes start-end/total|*`。
fn parse_content_range(s: &str) -> Option<(u64, u64, Option<u64>)> {
    let rest = s.trim().strip_prefix("bytes")?.trim_start();
    let (range, total) = rest.trim().split_once('/')?;
    let (start, end) = range.trim().split_once('-')?;
    let start: u64 = start.trim().parse().ok()?;
    let end: u64 = end.trim().parse().ok()?;
    if end < start {
        return None;
    }
    let total = match total.trim() {
        "*" => None,
        t => Some(t.parse().ok()?),
    };
    Some((start, end, total))
}

fn default_filename(target: &Path) -> String {
    target
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "file".to_string())
}

/// 常见扩展名 → MIME（对齐 reqwest Part::file 的扩展推断行为的最小子集）。
fn guess_mime(filename: &str) -> &'static str {
    let ext = filename
        .rsplit_once('.')
        .map(|(_, e)| e.to_ascii_lowercase())
        .unwrap_or_default();
    match ext.as_str() {
        "txt" | "text" | "md" | "log" => "text/plain",
        "csv" => "text/csv",
        "html" | "htm" => "text/html",
        "css" => "text/css",
        "js" => "text/javascript",
        "json" => "application/json",
        "xml" => "application/xml",
        "pdf" => "application/pdf",
        "zip" => "application/zip",
        "gz" => "application/gzip",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "mp3" => "audio/mpeg",
        "mp4" => "video/mp4",
        "wasm" => "application/wasm",
        _ => "application/octet-stream",
    }
}

// ============================================================================
// 内核测试：独立小预算 + 回环 TCP stub（串行矩阵在 tests/http_transfer.rs；
// 本节为内核快速回归，临时目录隔离可并行）
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};

    fn temp_dir(tag: &str) -> PathBuf {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir =
            std::env::temp_dir().join(format!("plan727-{}-{}-{}", tag, std::process::id(), n));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// 回环服务器：accept 一次 → 读请求头 → handler → done 信号。
    fn spawn_server(
        handler: impl FnOnce(String, TcpStream) + Send + 'static,
    ) -> (u16, std::sync::mpsc::Receiver<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let (done_tx, done_rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0u8; 8192];
            let n = stream.read(&mut buf).unwrap_or(0);
            let head = String::from_utf8_lossy(&buf[..n]).to_string();
            handler(head, stream);
            let _ = done_tx.send(());
        });
        (port, done_rx)
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

    fn join_server(done: std::sync::mpsc::Receiver<()>) {
        let _ = done.recv_timeout(Duration::from_secs(10));
    }

    /// 死端口（无 listener）：任何发出的请求都会立即连接失败——
    /// 「请求不应发出」断言的确定性载体。
    fn dead_port() -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        port
    }

    fn wait_terminal(t: &FileTransfer) -> TransferReceipt {
        let shared = t.shared();
        super::super::client::kernel_handle().block_on(async {
            let mut rx = shared.terminal_rx.clone();
            loop {
                if let Some(r) = rx.borrow().clone() {
                    return (*r).clone();
                }
                if rx.changed().await.is_err() {
                    panic!("terminal sender closed");
                }
            }
        })
    }

    // ---- 纯逻辑单测 ----

    #[test]
    fn plan727_parse_content_range_strict() {
        assert_eq!(
            parse_content_range("bytes 100-199/1000"),
            Some((100, 199, Some(1000)))
        );
        assert_eq!(parse_content_range("bytes 0-0/*"), Some((0, 0, None)));
        assert_eq!(
            parse_content_range("bytes 5-4/10"),
            None,
            "end < start 拒绝"
        );
        assert_eq!(parse_content_range("bytes abc-9/10"), None);
        assert_eq!(
            parse_content_range("chunks 5-9/10"),
            None,
            "缺 bytes 前缀拒绝"
        );
        assert_eq!(parse_content_range(""), None);
    }

    #[test]
    fn plan727_receipt_json_contract() {
        let r = TransferReceipt {
            kind: TransferOutcome::Success,
            status: Some(206),
            bytes: 10,
            total: Some(110),
            headers: vec![("Content-Type".into(), "text/plain".into())],
            error: None,
            body: Vec::new(),
        };
        assert_eq!(
            r.json(),
            "{\"kind\":\"success\",\"status\":206,\"bytes\":10,\"total\":110,\"headers\":{\"Content-Type\":\"text/plain\"},\"error\":null,\"body\":\"\"}"
        );
        let f = TransferReceipt::fail(
            Some(404),
            Vec::new(),
            TransferErrorKind::HttpStatus,
            "upstream status 404",
        );
        assert!(f.json().starts_with("{\"kind\":\"failed\",\"status\":404,"));
        assert!(f
            .json()
            .contains("\"error\":{\"kind\":\"http_status\",\"message\":\"upstream status 404\"}"));
        let n = TransferReceipt::fail(None, Vec::new(), TransferErrorKind::File, "boom");
        assert!(n.json().contains("\"status\":null"));
    }

    #[test]
    fn plan727_progress_json_contract() {
        let p = TransferProgress {
            bytes: 50,
            total: Some(200),
        };
        assert_eq!(
            p.json(),
            "{\"kind\":\"progress\",\"bytes\":50,\"total\":200,\"percent\":25.0}"
        );
        let u = TransferProgress {
            bytes: 5,
            total: None,
        };
        assert_eq!(
            u.json(),
            "{\"kind\":\"progress\",\"bytes\":5,\"total\":null,\"percent\":null}"
        );
        assert_eq!(
            TransferProgress {
                bytes: 0,
                total: Some(0)
            }
            .percent(),
            Some(100.0)
        );
    }

    #[test]
    fn plan727_download_options_strict_parse() {
        let o = DownloadOptions::parse(r#"{"headers":{"A":"b"},"offset":1024,"validator":{"etag":"\"x\""},"max_bytes":9,"on_exists":"fail"}"#).unwrap();
        assert_eq!(o.headers, vec![("A".into(), "b".into())]);
        assert_eq!(o.offset, Some(1024));
        assert_eq!(o.validator, Some(Validator::Etag("\"x\"".into())));
        assert_eq!(o.on_exists, OnExists::Fail);

        // 未知键 = Options 错误（严格解析）。
        assert!(DownloadOptions::parse(r#"{"unknown":1}"#).is_err());
        // 弱 ETag 拒绝。
        match DownloadOptions::parse(r#"{"validator":{"etag":"W/\"weak\""}}"#) {
            Err(e) => assert_eq!(e.kind, TransferErrorKind::Options),
            Ok(_) => panic!("弱 ETag 必须拒绝"),
        }
        // validator 双字段拒绝。
        assert!(
            DownloadOptions::parse(r#"{"validator":{"etag":"\"a\"","last_modified":"x"}}"#)
                .is_err()
        );
        // 坏 on_exists 拒绝。
        assert!(DownloadOptions::parse(r#"{"on_exists":"clobber"}"#).is_err());
        // 空 = 默认。
        let d = DownloadOptions::parse("").unwrap();
        assert_eq!(d.on_exists, OnExists::Overwrite);
        assert!(d.offset.is_none());
    }

    #[test]
    fn plan727_upload_options_strict_parse() {
        let o = UploadOptions::parse(r#"{"mode":"raw","headers":{"A":"b"},"retries":2}"#).unwrap();
        assert_eq!(o.mode, UploadMode::Raw);
        assert_eq!(o.retries, 2);
        // raw + multipart 专属键 = Options 错误。
        assert!(UploadOptions::parse(r#"{"mode":"raw","field":"f"}"#).is_err());
        // 坏模式拒绝。
        assert!(UploadOptions::parse(r#"{"mode":"chunked"}"#).is_err());
        let d = UploadOptions::parse("").unwrap();
        assert_eq!(d.mode, UploadMode::Multipart);
        assert_eq!(d.field, "file");
        assert_eq!(d.retries, 0);
    }

    #[test]
    fn plan727_bad_options_terminal_no_request() {
        // 坏 options：句柄已终结为 Failed(options)，请求不发出（无服务器）。
        let dir = temp_dir("bad-opts");
        let t = transfer_download(
            "http://127.0.0.1:9/x",
            dir.join("f.bin").to_str().unwrap(),
            "{bad json",
        );
        let r = wait_terminal(&t);
        assert_eq!(r.kind, TransferOutcome::Failed);
        assert_eq!(r.error.as_ref().unwrap().kind, TransferErrorKind::Options);
        assert_eq!(r.status, None, "请求未发出");
        assert!(transfer_error(&t).contains("invalid download options"));
        // 进度面：终态收据单次交付。
        assert!(transfer_next_progress(&t).contains("\"kind\":\"failed\""));
        assert_eq!(transfer_next_progress(&t), "");
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn plan727_staging_path_same_dir() {
        let p = staging_path(Path::new("/tmp/data/file.bin"), 7);
        assert_eq!(p, Path::new("/tmp/data/.file.bin.plan727-7.part"));
        let q = staging_path(Path::new("file.bin"), 8);
        assert_eq!(q, Path::new("./.file.bin.plan727-8.part"));
    }

    // ---- 回环集成（内核快速回归） ----

    #[test]
    fn plan727_download_full_commit_replaces_target() {
        let dir = temp_dir("dl-full");
        let target = dir.join("file.bin");
        std::fs::write(&target, b"OLD").unwrap();
        let payload = vec![7u8; 200 * 1024]; // > 64KiB：多块
        let payload_clone = payload.clone();
        let (port, done) = spawn_server(move |head, mut s| {
            assert!(head.contains("GET /f "), "request line: {head}");
            respond(
                &mut s,
                "HTTP/1.1 200 OK",
                &[("ETag", "\"v1\"")],
                &payload_clone,
            );
        });
        let t = transfer_download(
            &format!("http://127.0.0.1:{port}/f"),
            target.to_str().unwrap(),
            "",
        );
        let receipt = wait_terminal(&t);
        assert_eq!(
            receipt.kind,
            TransferOutcome::Success,
            "{:?}",
            receipt.error
        );
        assert_eq!(receipt.status, Some(200));
        assert_eq!(receipt.bytes, payload.len() as u64);
        assert_eq!(receipt.total, Some(payload.len() as u64));
        // 原目标被替换为新内容；无 staging 遗留。
        assert_eq!(std::fs::read(&target).unwrap(), payload);
        let entries: Vec<_> = std::fs::read_dir(&dir).unwrap().collect();
        assert_eq!(entries.len(), 1, "staging 必须清理: {entries:?}");
        join_server(done);
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn plan727_download_http_failure_preserves_target() {
        let dir = temp_dir("dl-404");
        let target = dir.join("keep.bin");
        std::fs::write(&target, b"PRECIOUS").unwrap();
        let (port, done) = spawn_server(move |_head, mut s| {
            respond(&mut s, "HTTP/1.1 404 Not Found", &[], b"nope");
        });
        let t = transfer_download(
            &format!("http://127.0.0.1:{port}/f"),
            target.to_str().unwrap(),
            "",
        );
        let receipt = wait_terminal(&t);
        assert_eq!(receipt.kind, TransferOutcome::Failed);
        assert_eq!(receipt.status, Some(404));
        assert_eq!(
            receipt.error.as_ref().unwrap().kind,
            TransferErrorKind::HttpStatus
        );
        assert_eq!(
            std::fs::read(&target).unwrap(),
            b"PRECIOUS",
            "原目标必须保持"
        );
        let entries: Vec<_> = std::fs::read_dir(&dir).unwrap().collect();
        assert_eq!(entries.len(), 1, "无 staging 遗留");
        join_server(done);
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn plan727_resume_206_appends_via_staging() {
        let dir = temp_dir("dl-resume");
        let target = dir.join("large.bin");
        let prefix = vec![1u8; 100];
        let suffix = vec![2u8; 50];
        let prefix_len = prefix.len();
        std::fs::write(&target, &prefix).unwrap();
        let suffix_clone = suffix.clone();
        let (port, done) = spawn_server(move |head, mut s| {
            // hyper 线上头名小写——断言用小写形态。
            let head_l = head.to_ascii_lowercase();
            assert!(
                head_l.contains(&format!("range: bytes={}-", prefix_len)),
                "Range 头: {head}"
            );
            assert!(
                head_l.contains("accept-encoding: identity"),
                "identity 必需: {head}"
            );
            respond(
                &mut s,
                "HTTP/1.1 206 Partial Content",
                &[(
                    "Content-Range",
                    &format!(
                        "bytes {}-{}/{}",
                        prefix_len,
                        prefix_len + suffix_clone.len() - 1,
                        prefix_len + suffix_clone.len()
                    ),
                )],
                &suffix_clone,
            );
        });
        let t = transfer_download(
            &format!("http://127.0.0.1:{port}/f"),
            target.to_str().unwrap(),
            &format!("{{\"offset\":{}}}", prefix_len),
        );
        let receipt = wait_terminal(&t);
        assert_eq!(
            receipt.kind,
            TransferOutcome::Success,
            "{:?}",
            receipt.error
        );
        assert_eq!(receipt.status, Some(206));
        assert_eq!(receipt.bytes, (prefix_len + suffix.len()) as u64);
        assert_eq!(receipt.total, Some((prefix_len + suffix.len()) as u64));
        let mut expect = prefix.clone();
        expect.extend_from_slice(&suffix);
        assert_eq!(std::fs::read(&target).unwrap(), expect, "前缀复制+后缀追加");
        join_server(done);
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn plan727_resume_200_full_restart_never_appends() {
        let dir = temp_dir("dl-restart");
        let target = dir.join("large.bin");
        std::fs::write(&target, vec![1u8; 100]).unwrap();
        let fresh = vec![9u8; 80];
        let fresh_clone = fresh.clone();
        let (port, done) = spawn_server(move |_head, mut s| {
            // 服务器忽略 Range → 200 全量。
            respond(&mut s, "HTTP/1.1 200 OK", &[], &fresh_clone);
        });
        let t = transfer_download(
            &format!("http://127.0.0.1:{port}/f"),
            target.to_str().unwrap(),
            "{\"offset\":100}",
        );
        let receipt = wait_terminal(&t);
        assert_eq!(
            receipt.kind,
            TransferOutcome::Success,
            "{:?}",
            receipt.error
        );
        assert_eq!(receipt.status, Some(200));
        assert_eq!(
            std::fs::read(&target).unwrap(),
            fresh,
            "200 = 完整重启，不追加"
        );
        join_server(done);
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn plan727_resume_offset_mismatch_preserves_target() {
        let dir = temp_dir("dl-mismatch");
        let target = dir.join("large.bin");
        std::fs::write(&target, vec![1u8; 40]).unwrap();
        let port = dead_port(); // offset 不符时请求不应发出
        let t = transfer_download(
            &format!("http://127.0.0.1:{port}/f"),
            target.to_str().unwrap(),
            "{\"offset\":100}",
        );
        let receipt = wait_terminal(&t);
        assert_eq!(receipt.kind, TransferOutcome::Failed);
        assert_eq!(
            receipt.error.as_ref().unwrap().kind,
            TransferErrorKind::File
        );
        assert_eq!(receipt.status, None);
        assert_eq!(std::fs::read(&target).unwrap().len(), 40, "原目标保持");
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn plan727_budget_exceeded_preserves_target() {
        let dir = temp_dir("dl-budget");
        let target = dir.join("f.bin");
        std::fs::write(&target, b"OLD").unwrap();
        let big = vec![b'x'; 300 * 1024]; // > max_bytes 128KiB
        let big_clone = big.clone();
        let (port, done) = spawn_server(move |_head, mut s| {
            respond(&mut s, "HTTP/1.1 200 OK", &[], &big_clone);
        });
        let t = transfer_download(
            &format!("http://127.0.0.1:{port}/f"),
            target.to_str().unwrap(),
            "{\"max_bytes\":131072}",
        );
        let receipt = wait_terminal(&t);
        assert_eq!(receipt.kind, TransferOutcome::Failed);
        assert_eq!(
            receipt.error.as_ref().unwrap().kind,
            TransferErrorKind::Budget
        );
        assert_eq!(std::fs::read(&target).unwrap(), b"OLD", "超限保留原目标");
        let entries: Vec<_> = std::fs::read_dir(&dir).unwrap().collect();
        assert_eq!(entries.len(), 1, "staging 清理");
        join_server(done);
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn plan727_cancel_mid_transfer_preserves_target_and_cleans_staging() {
        let dir = temp_dir("dl-cancel");
        let target = dir.join("f.bin");
        std::fs::write(&target, b"OLD").unwrap();
        // 慢上游：headers 后滴流不断。
        let (port, done) = spawn_server(move |_head, mut s| {
            s.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 1000000\r\n\r\n")
                .unwrap();
            let _ = s.flush();
            for _ in 0..600 {
                if s.write_all(&[b'z'; 1024]).is_err() {
                    break;
                }
                let _ = s.flush();
                std::thread::sleep(Duration::from_millis(10));
            }
        });
        let t = transfer_download(
            &format!("http://127.0.0.1:{port}/f"),
            target.to_str().unwrap(),
            "",
        );
        // 等进入传输态（staging 已建）。
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while t.phase() != TransferPhase::Transferring {
            assert!(std::time::Instant::now() < deadline, "未进入传输态");
            std::thread::sleep(Duration::from_millis(5));
        }
        transfer_cancel(&t);
        let receipt = wait_terminal(&t);
        assert_eq!(receipt.kind, TransferOutcome::Cancelled, "{receipt:?}");
        assert_eq!(std::fs::read(&target).unwrap(), b"OLD", "取消保留原目标");
        let entries: Vec<_> = std::fs::read_dir(&dir).unwrap().collect();
        assert_eq!(entries.len(), 1, "取消后 staging 清理");
        join_server(done);
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn plan727_commit_gate_late_cancel_does_not_rollback() {
        let dir = temp_dir("dl-gate");
        let target = dir.join("f.bin");
        std::fs::write(&target, b"OLD").unwrap();
        let (port, done) = spawn_server(move |_head, mut s| {
            respond(&mut s, "HTTP/1.1 200 OK", &[], b"NEW-CONTENT");
        });
        // commit gate：挂住提交；gate 期间取消、再放行 → 提交成功 = 成功收据。
        let (gate_tx, gate_rx) = std::sync::mpsc::channel::<()>();
        let (proceed_tx, proceed_rx) = std::sync::mpsc::channel::<()>();
        let proceed = std::sync::Mutex::new(proceed_rx);
        let hooks = Arc::new(TransferHooks {
            before_commit: Some(Arc::new(move || {
                let _ = gate_tx.send(());
                let _ = proceed.lock().unwrap().recv();
                Ok(())
            })),
            ..Default::default()
        });
        let t = transfer_download(
            &format!("http://127.0.0.1:{port}/f"),
            target.to_str().unwrap(),
            "",
        )
        .with_hooks(hooks);
        // 等 writer 到达 gate（有限等待防挂死）。
        assert!(
            gate_rx.recv_timeout(Duration::from_secs(10)).is_ok(),
            "commit gate 未到达"
        );
        transfer_cancel(&t);
        let _ = proceed_tx.send(());
        let receipt = wait_terminal(&t);
        assert_eq!(
            receipt.kind,
            TransferOutcome::Success,
            "gate 后迟到取消不回滚: {receipt:?}"
        );
        assert_eq!(std::fs::read(&target).unwrap(), b"NEW-CONTENT");
        join_server(done);
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn plan727_write_failure_injection_preserves_target() {
        let dir = temp_dir("dl-wfail");
        let target = dir.join("f.bin");
        std::fs::write(&target, b"KEEPME").unwrap();
        let (port, done) = spawn_server(move |_head, mut s| {
            respond(&mut s, "HTTP/1.1 200 OK", &[], b"abcdef");
        });
        let hooks = Arc::new(TransferHooks {
            before_write: Some(Arc::new(|| Err("disk on fire".to_string()))),
            ..Default::default()
        });
        let t = transfer_download(
            &format!("http://127.0.0.1:{port}/f"),
            target.to_str().unwrap(),
            "",
        )
        .with_hooks(hooks);
        let receipt = wait_terminal(&t);
        assert_eq!(receipt.kind, TransferOutcome::Failed);
        assert_eq!(
            receipt.error.as_ref().unwrap().kind,
            TransferErrorKind::File
        );
        assert!(receipt
            .error
            .as_ref()
            .unwrap()
            .message
            .contains("disk on fire"));
        assert_eq!(std::fs::read(&target).unwrap(), b"KEEPME");
        join_server(done);
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn plan727_same_target_conflict() {
        let dir = temp_dir("dl-conflict");
        let target = dir.join("f.bin");
        let (port, done) = spawn_server(move |_head, mut s| {
            s.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 100000\r\n\r\n")
                .unwrap();
            let _ = s.flush();
            for _ in 0..600 {
                if s.write_all(&[b'q'; 1024]).is_err() {
                    break;
                }
                let _ = s.flush();
                std::thread::sleep(Duration::from_millis(10));
            }
        });
        let t1 = transfer_download(
            &format!("http://127.0.0.1:{port}/f"),
            target.to_str().unwrap(),
            "",
        );
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while t1.phase() != TransferPhase::Transferring {
            assert!(std::time::Instant::now() < deadline, "t1 未进入传输态");
            std::thread::sleep(Duration::from_millis(5));
        }
        let t2 = transfer_download(
            &format!("http://127.0.0.1:{port}/f"),
            target.to_str().unwrap(),
            "",
        );
        let r2 = wait_terminal(&t2);
        assert_eq!(r2.kind, TransferOutcome::Failed);
        assert_eq!(
            r2.error.as_ref().unwrap().kind,
            TransferErrorKind::Conflict,
            "{r2:?}"
        );
        transfer_cancel(&t1);
        let _ = wait_terminal(&t1);
        join_server(done);
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn plan727_upload_multipart_wire_and_text_fields() {
        let dir = temp_dir("up-mp");
        let src = dir.join("report.txt");
        std::fs::write(&src, b"FILE-BODY").unwrap();
        let captured = Arc::new(Mutex::new(None::<String>));
        let captured2 = captured.clone();
        let (port, done) = spawn_server(move |head, mut s| {
            *captured2.lock().unwrap() = Some(head);
            respond(&mut s, "HTTP/1.1 201 Created", &[], b"saved");
        });
        let t = transfer_upload(
            &format!("http://127.0.0.1:{port}/up"),
            src.to_str().unwrap(),
            r#"{"fields":{"note":"hello"},"filename":"report.txt"}"#,
        );
        let receipt = wait_terminal(&t);
        assert_eq!(
            receipt.kind,
            TransferOutcome::Success,
            "{:?}",
            receipt.error
        );
        assert_eq!(receipt.status, Some(201));
        assert_eq!(receipt.bytes, 9);
        assert_eq!(receipt.body, b"saved");
        let head = captured.lock().unwrap().clone().unwrap();
        assert!(head.contains("POST /up "), "{head}");
        assert!(head.contains("multipart/form-data"), "{head}");
        assert!(head.contains("name=\"note\""), "text 字段缺失: {head}");
        assert!(
            head.contains("filename=\"report.txt\""),
            "filename 缺失: {head}"
        );
        join_server(done);
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn plan727_upload_raw_octet_stream() {
        let dir = temp_dir("up-raw");
        let src = dir.join("blob.bin");
        std::fs::write(&src, vec![5u8; 300]).unwrap();
        let (port, done) = spawn_server(move |head, mut s| {
            let head_l = head.to_ascii_lowercase();
            assert!(
                head_l.contains("content-type: application/octet-stream"),
                "{head}"
            );
            assert!(
                head_l.contains("content-length: 300"),
                "raw 须带精确长度: {head}"
            );
            respond(&mut s, "HTTP/1.1 200 OK", &[], b"ok");
        });
        let t = transfer_upload(
            &format!("http://127.0.0.1:{port}/up"),
            src.to_str().unwrap(),
            "{\"mode\":\"raw\"}",
        );
        let receipt = wait_terminal(&t);
        assert_eq!(
            receipt.kind,
            TransferOutcome::Success,
            "{:?}",
            receipt.error
        );
        assert_eq!(receipt.bytes, 300);
        join_server(done);
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn plan727_upload_missing_file_fails_not_success() {
        let dir = temp_dir("up-missing");
        let src = dir.join("ghost.bin");
        let port = dead_port(); // 缺失文件不应发出请求
        let t = transfer_upload(
            &format!("http://127.0.0.1:{port}/up"),
            src.to_str().unwrap(),
            "",
        );
        let receipt = wait_terminal(&t);
        assert_eq!(receipt.kind, TransferOutcome::Failed);
        assert_eq!(
            receipt.error.as_ref().unwrap().kind,
            TransferErrorKind::File
        );
        assert_eq!(receipt.status, None);
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn plan727_upload_dir_rejected() {
        let dir = temp_dir("up-dir");
        let port = dead_port(); // 目录上传不应发出请求
        let t = transfer_upload(
            &format!("http://127.0.0.1:{port}/up"),
            dir.to_str().unwrap(),
            "",
        );
        let receipt = wait_terminal(&t);
        assert_eq!(receipt.kind, TransferOutcome::Failed);
        assert_eq!(
            receipt.error.as_ref().unwrap().kind,
            TransferErrorKind::File
        );
        assert!(receipt
            .error
            .as_ref()
            .unwrap()
            .message
            .contains("not a regular file"));
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn plan727_zero_byte_download_succeeds() {
        let dir = temp_dir("dl-zero");
        let target = dir.join("empty.bin");
        let (port, done) = spawn_server(move |_head, mut s| {
            respond(&mut s, "HTTP/1.1 200 OK", &[], b"");
        });
        let t = transfer_download(
            &format!("http://127.0.0.1:{port}/f"),
            target.to_str().unwrap(),
            "",
        );
        let receipt = wait_terminal(&t);
        assert_eq!(
            receipt.kind,
            TransferOutcome::Success,
            "{:?}",
            receipt.error
        );
        assert_eq!(receipt.bytes, 0);
        assert_eq!(receipt.total, Some(0));
        assert!(std::fs::metadata(&target).unwrap().len() == 0);
        join_server(done);
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn plan727_zero_byte_resume_requires_exact_empty_target() {
        let dir = temp_dir("dl-zero-resume");
        let target = dir.join("e.bin");
        std::fs::write(&target, b"").unwrap();
        let (port, done) = spawn_server(move |_head, mut s| {
            respond(
                &mut s,
                "HTTP/1.1 206 Partial Content",
                &[("Content-Range", "bytes 0-4/5")],
                b"12345",
            );
        });
        let t = transfer_download(
            &format!("http://127.0.0.1:{port}/f"),
            target.to_str().unwrap(),
            "{\"offset\":0}",
        );
        let receipt = wait_terminal(&t);
        // offset=0 不启用 resume 路径（>0 才续传）——等价全量下载。
        assert_eq!(
            receipt.kind,
            TransferOutcome::Success,
            "{:?}",
            receipt.error
        );
        assert_eq!(std::fs::read(&target).unwrap(), b"12345");
        join_server(done);
        std::fs::remove_dir_all(dir).ok();
    }
}
