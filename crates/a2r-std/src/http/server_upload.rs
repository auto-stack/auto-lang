//! PLAN-730 T-02：服务端上传接收——owned 公共面与 async facade（单源）。
//!
//! 契约速览（canonical Spec 由 SD-01 沉淀；决策报告
//! `docs/plans/reports/730-upload-decision.md` §2/§3）：
//!
//! ```text
//! upload_receive(req, root, staging_root, options).await → UploadSession
//! upload_metadata(session) → str            # 有界 JSON（接收快照）
//! upload_commit(session, relative_target).await → UploadReceipt   # create-only 发布
//! upload_reject(session, status, message).await → UploadReceipt   # 清理收口
//! upload_error(status, message) → UploadReceipt                   # 零 I/O 早拒
//! ```
//!
//! 拆分（依赖方向冻结）：本模块只含 owned 类型、严格 options、限额表、
//! 错误种类→HTTP status 纯映射与 async facade；**接收/落盘/发布执行在宿主
//! executor**（`auto_lang::http_upload_service` 经 `install_upload_executor`
//! 注入本模块的 owned async hook）。executor 未安装时 facade 返回确定性诊断
//! 失败（零 I/O）——非 HTTP 形态（IPC/merged 滥用）不静默假成功。
//!
//! `UploadRequest` 只能由宿主构造（`upload_request_from_parts`——VM 桥与
//! 生成 Rust 腿的 adapter 调用）；Auto 面无用户构造 native，请求字段不能
//! 指定 root/staging_root/最终 target。

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};

use serde::Deserialize;

use super::server_file::validate_relative_path;

// ============================================================================
// 错误种类与 status 纯映射
// ============================================================================

/// 上传失败/收据种类（receipt JSON `kind` 字段；HTTP status 映射见表）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UploadErrorKind {
    /// options 严格解析失败/非法配置（root/staging 越权形态等）→ 500。
    InvalidOptions,
    /// wire 总量或单文件超限 → 413。
    BodyTooLarge,
    /// Content-Encoding 非 identity / 媒体类型不允许 → 415。
    UnsupportedMedia,
    /// multipart 结构坏（缺 closing boundary/坏 headers/坏 UTF-8 文本/
    /// 缺 file/第二 file/未知字段/截断）→ 400。
    BadMultipart,
    /// 接收 idle 超时 → 408。
    IdleTimeout,
    /// 上传总期限（headers 入 scope 起）→ 408。
    TotalTimeout,
    /// active/queue 满或队列期限 → 503。
    QueueFull,
    /// 目标已存在（create-only 发布拒绝）→ 409。
    TargetConflict,
    /// 目标路径越界/权限 → 403。
    ForbiddenPath,
    /// 磁盘/IO/发布故障 → 500。
    Io,
    /// 会话冲突（重复 receive/commit/reject、failed 会话 commit、句柄未知）→ 409。
    SessionConflict,
    /// 取消/断连/shutdown 收口（未提交）→（应用侧不可见时为连接终结）。
    Cancelled,
    /// staged lease 到期未 commit/reject → 410。
    ExpiredLease,
    /// 应用显式拒绝（upload_reject）。
    Rejected,
    /// executor 未安装（非 HTTP 形态滥用）→ 500。
    NoExecutor,
}

impl UploadErrorKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            UploadErrorKind::InvalidOptions => "invalid_options",
            UploadErrorKind::BodyTooLarge => "body_too_large",
            UploadErrorKind::UnsupportedMedia => "unsupported_media",
            UploadErrorKind::BadMultipart => "bad_multipart",
            UploadErrorKind::IdleTimeout => "idle_timeout",
            UploadErrorKind::TotalTimeout => "total_timeout",
            UploadErrorKind::QueueFull => "queue_full",
            UploadErrorKind::TargetConflict => "target_conflict",
            UploadErrorKind::ForbiddenPath => "forbidden_path",
            UploadErrorKind::Io => "io_error",
            UploadErrorKind::SessionConflict => "session_conflict",
            UploadErrorKind::Cancelled => "cancelled",
            UploadErrorKind::ExpiredLease => "expired_lease",
            UploadErrorKind::Rejected => "rejected",
            UploadErrorKind::NoExecutor => "no_executor",
        }
    }

    /// 建议的 HTTP status（§5.3 状态表；应用 reject 自选状态不受此限）。
    pub fn suggested_status(&self) -> u16 {
        match self {
            UploadErrorKind::InvalidOptions => 500,
            UploadErrorKind::BodyTooLarge => 413,
            UploadErrorKind::UnsupportedMedia => 415,
            UploadErrorKind::BadMultipart => 400,
            UploadErrorKind::IdleTimeout => 408,
            UploadErrorKind::TotalTimeout => 408,
            UploadErrorKind::QueueFull => 503,
            UploadErrorKind::TargetConflict => 409,
            UploadErrorKind::ForbiddenPath => 403,
            UploadErrorKind::Io => 500,
            UploadErrorKind::SessionConflict => 409,
            UploadErrorKind::Cancelled => 499,
            UploadErrorKind::ExpiredLease => 410,
            UploadErrorKind::Rejected => 400,
            UploadErrorKind::NoExecutor => 500,
        }
    }
}

// ============================================================================
// UploadRequest（宿主构造；一次性消费）
// ============================================================================

/// 版本无关的请求体流（VM 桥与两代 axum 消费端各自投影；不含 axum 类型）。
pub type UploadBodyStream =
    std::pin::Pin<Box<dyn futures::Stream<Item = std::io::Result<Vec<u8>>> + Send>>;

/// HTTP 注入的上传请求能力：method/path/headers 快照 + 原始 body 流。
/// 只能由宿主 adapter 经 [`upload_request_from_parts`] 构造（VM 桥 / 生成
/// Rust 服务的 extractor glue）；Auto 面无构造 native——请求字段不能伪造。
/// 值按 move 传递（一次性消费由结构保证）。
pub struct UploadRequest {
    pub(crate) method: String,
    pub(crate) path: String,
    pub(crate) headers: Vec<(String, String)>,
    pub(crate) body: UploadBodyStream,
}

impl UploadRequest {
    pub fn method(&self) -> &str {
        &self.method
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn headers(&self) -> &[(String, String)] {
        &self.headers
    }

    /// 消费分解（宿主 executor 读取 headers/body 用）。
    pub fn into_parts(self) -> (String, String, Vec<(String, String)>, UploadBodyStream) {
        (self.method, self.path, self.headers, self.body)
    }
}

/// 宿主 adapter 构造入口（VM 桥 / 生成 Rust extractor glue）。
pub fn upload_request_from_parts(
    method: &str,
    path: &str,
    headers: Vec<(String, String)>,
    body: UploadBodyStream,
) -> UploadRequest {
    UploadRequest {
        method: method.to_string(),
        path: path.to_string(),
        headers,
        body,
    }
}

// ============================================================================
// 接收快照（UploadSession 载荷）与 metadata JSON
// ============================================================================

/// 接收完成后的有界快照（fields 有序保留重复；filename 仅 metadata）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UploadReceivedMeta {
    pub mode: UploadReceiveMode,
    /// 声明的 file 字段名（multipart；raw 为空串）。
    pub field: String,
    /// 客户端 filename（可缺/空；lossy；绝不作为存储路径）。
    pub filename: Option<String>,
    /// 文件字节数（staged 内容长度）。
    pub size: u64,
    /// 文件 part 的 Content-Type（multipart part 头或 raw 请求头；可缺）。
    pub content_type: Option<String>,
    /// 有序文本字段（含重复，不静默覆盖）。
    pub fields: Vec<(String, String)>,
}

/// 会话接收状态快照（receive 完成时冻结；资源在宿主 executor 注册表）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UploadSessionState {
    Received(UploadReceivedMeta),
    Failed {
        kind: UploadErrorKind,
        message: String,
    },
}

/// owned 会话句柄：id + 接收快照。拷贝不复制资源（资源按 id 归 executor
/// 注册表；commit/reject 单次消费由 executor 终态门保证）。
#[derive(Debug, Clone)]
pub struct UploadSession {
    id: u64,
    state: UploadSessionState,
}

impl UploadSession {
    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn state(&self) -> &UploadSessionState {
        &self.state
    }

    pub fn is_received(&self) -> bool {
        matches!(self.state, UploadSessionState::Received(_))
    }
}

fn next_upload_session_id() -> u64 {
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// 成功会话构造（宿主 executor：接收完成交付）。
pub fn received_session(id: u64, meta: UploadReceivedMeta) -> UploadSession {
    UploadSession {
        id,
        state: UploadSessionState::Received(meta),
    }
}

/// 失败会话构造（宿主 executor/facade 诊断路径共用）。
pub fn failed_session(kind: UploadErrorKind, message: impl Into<String>) -> UploadSession {
    UploadSession {
        id: next_upload_session_id(),
        state: UploadSessionState::Failed {
            kind,
            message: message.into(),
        },
    }
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for c in s.chars() {
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

/// metadata JSON（`http.upload_metadata(session)` 的载荷；有界——字段值
/// 已受接收预算限制）。失败会话含 kind/message/suggested_status（应用
/// `upload_reject(session, 0, "")` 可直接采用建议状态）。
pub fn upload_metadata_json(session: &UploadSession) -> String {
    match &session.state {
        UploadSessionState::Received(meta) => {
            let fields = meta
                .fields
                .iter()
                .map(|(k, v)| {
                    format!(
                        "{{\"name\":\"{}\",\"value\":\"{}\"}}",
                        json_escape(k),
                        json_escape(v)
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "{{\"state\":\"received\",\"mode\":\"{}\",\"field\":\"{}\",\"filename\":{},\"size\":\"{}\",\"content_type\":{},\"fields\":[{}]}}",
                meta.mode.as_str(),
                json_escape(&meta.field),
                meta.filename
                    .as_deref()
                    .map(|f| format!("\"{}\"", json_escape(f)))
                    .unwrap_or_else(|| "null".to_string()),
                meta.size,
                meta.content_type
                    .as_deref()
                    .map(|c| format!("\"{}\"", json_escape(c)))
                    .unwrap_or_else(|| "null".to_string()),
                fields,
            )
        }
        UploadSessionState::Failed { kind, message } => format!(
            "{{\"state\":\"failed\",\"kind\":\"{}\",\"message\":\"{}\",\"suggested_status\":{}}}",
            kind.as_str(),
            json_escape(message),
            kind.suggested_status(),
        ),
    }
}

// ============================================================================
// UploadReceipt（owned 终态；不持在途文件资源）
// ============================================================================

/// owned 收据：真实 HTTP status + 有界 JSON 体。拷贝不重复提交。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UploadReceipt {
    pub status: u16,
    pub kind: UploadErrorKind,
    pub json: String,
}

impl UploadReceipt {
    /// 成功收据（commit 201；size 十进制字符串防 TS 精度丢失）。
    pub fn committed(relative_path: &str, meta: &UploadReceivedMeta) -> UploadReceipt {
        let fields = meta
            .fields
            .iter()
            .map(|(k, v)| {
                format!(
                    "{{\"name\":\"{}\",\"value\":\"{}\"}}",
                    json_escape(k),
                    json_escape(v)
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        UploadReceipt {
            status: 201,
            kind: UploadErrorKind::Rejected, // ok=true 时 kind 不参与（保持枚举非空）
            json: format!(
                "{{\"ok\":true,\"path\":\"{}\",\"size\":\"{}\",\"field\":\"{}\",\"filename\":{},\"fields\":[{}]}}",
                json_escape(relative_path),
                meta.size,
                json_escape(&meta.field),
                meta.filename
                    .as_deref()
                    .map(|f| format!("\"{}\"", json_escape(f)))
                    .unwrap_or_else(|| "null".to_string()),
                fields,
            ),
        }
    }

    /// 失败收据（kind 的建议 status，除非应用自选合法状态覆盖）。
    pub fn failed(kind: UploadErrorKind, message: impl Into<String>) -> UploadReceipt {
        let message = message.into();
        UploadReceipt {
            status: kind.suggested_status(),
            kind: kind.clone(),
            json: format!(
                "{{\"ok\":false,\"kind\":\"{}\",\"message\":\"{}\"}}",
                kind.as_str(),
                json_escape(&message),
            ),
        }
    }
}

/// `http.upload_error(status, message)`：零 I/O 早拒构造（不等待、不打开
/// 文件、不能伪造 201 成功）。status 域 400..=599；域外 → 500 + 可观察
/// 诊断（不静默钳制）。
pub fn upload_error(status: i64, message: impl AsRef<str>) -> UploadReceipt {
    let message = message.as_ref();
    if !(400..=599).contains(&status) {
        return UploadReceipt::failed(
            UploadErrorKind::InvalidOptions,
            format!("upload_error: invalid status {status} (must be 400..=599)"),
        );
    }
    UploadReceipt {
        status: status as u16,
        kind: UploadErrorKind::Rejected,
        json: format!(
            "{{\"ok\":false,\"kind\":\"rejected\",\"message\":\"{}\"}}",
            json_escape(message),
        ),
    }
}

// ============================================================================
// 严格 options 与服务硬上限
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UploadReceiveMode {
    Multipart,
    Raw,
}

impl UploadReceiveMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            UploadReceiveMode::Multipart => "multipart",
            UploadReceiveMode::Raw => "raw",
        }
    }
}

/// 接收 options（严格 JSON；未知键/错类型 → `Err`，零 I/O 拒绝）。
/// 全部限额只可下调（`min(options, 硬上限)`）；mode 显式声明——不从
/// Content-Type 猜测能力。
#[derive(Debug, Clone, PartialEq)]
pub struct UploadReceiveOptions {
    pub mode: UploadReceiveMode,
    /// multipart 声明的 file 字段名（默认 "file"）。
    pub file_field: String,
    /// 允许的文本字段名（不在表内的字段 = 明确错误，不偷偷接第一份）。
    pub text_fields: Vec<String>,
    /// raw 模式允许的媒体类型（缺省 = 允许除 multipart/* 外任意）。
    pub raw_content_types: Option<Vec<String>>,
    pub max_file_bytes: u64,
    pub max_wire_bytes: u64,
    pub max_text_fields: u32,
    pub max_text_field_bytes: usize,
    pub max_text_total_bytes: usize,
    pub max_parts: u32,
    pub max_part_header_bytes: usize,
    pub max_part_header_items: u32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UploadOptionsRaw {
    mode: String,
    #[serde(default)]
    file_field: Option<String>,
    #[serde(default)]
    text_fields: Option<Vec<String>>,
    #[serde(default)]
    raw_content_types: Option<Vec<String>>,
    #[serde(default)]
    max_file_bytes: Option<u64>,
    #[serde(default)]
    max_wire_bytes: Option<u64>,
    #[serde(default)]
    max_text_fields: Option<u32>,
    #[serde(default)]
    max_text_field_bytes: Option<usize>,
    #[serde(default)]
    max_text_total_bytes: Option<usize>,
    #[serde(default)]
    max_parts: Option<u32>,
    #[serde(default)]
    max_part_header_bytes: Option<usize>,
    #[serde(default)]
    max_part_header_items: Option<u32>,
}

/// 服务硬上限（env `AUTO_HTTP_UPLOAD_*`；0/非法回默认——零/非法预算
/// 不变成无限）。§5.3 默认合同的机器面。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UploadServeLimits {
    pub max_active: usize,
    pub queue_capacity: usize,
    pub app_block_bytes: usize,
    pub max_pending_blocks: usize,
    pub fs_ops_max: usize,
    pub max_file_bytes: u64,
    pub max_wire_bytes: u64,
    pub max_text_fields: u32,
    pub max_text_field_bytes: usize,
    pub max_text_total_bytes: usize,
    pub max_parts: u32,
    pub max_part_header_bytes: usize,
    pub max_part_header_items: u32,
    pub max_boundary_bytes: usize,
    pub total_timeout_ms: u64,
    pub idle_timeout_ms: u64,
    pub lease_timeout_ms: u64,
    pub queue_timeout_ms: u64,
}

impl Default for UploadServeLimits {
    fn default() -> Self {
        UploadServeLimits {
            max_active: 4,
            queue_capacity: 16,
            app_block_bytes: 64 * 1024,
            max_pending_blocks: 2,
            fs_ops_max: 4,
            max_file_bytes: 64 * 1024 * 1024,
            max_wire_bytes: 65 * 1024 * 1024,
            max_text_fields: 16,
            max_text_field_bytes: 16 * 1024,
            max_text_total_bytes: 64 * 1024,
            max_parts: 32,
            max_part_header_bytes: 16 * 1024,
            max_part_header_items: 32,
            max_boundary_bytes: 70,
            total_timeout_ms: 10 * 60 * 1000,
            idle_timeout_ms: 60 * 1000,
            lease_timeout_ms: 30 * 1000,
            queue_timeout_ms: 30 * 1000,
        }
    }
}

fn env_usize(key: &str, default: usize) -> usize {
    std::env::var(key)
        .ok()
        .and_then(|v| v.trim().parse::<usize>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(default)
}

fn env_u64(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.trim().parse::<u64>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(default)
}

impl UploadServeLimits {
    /// env 覆写（测试/运维旋钮；语义同 729 `FileServeLimits::from_env`）。
    pub fn from_env() -> Self {
        let d = UploadServeLimits::default();
        UploadServeLimits {
            max_active: env_usize("AUTO_HTTP_UPLOAD_ACTIVE", d.max_active),
            queue_capacity: env_usize("AUTO_HTTP_UPLOAD_QUEUE", d.queue_capacity),
            app_block_bytes: env_usize("AUTO_HTTP_UPLOAD_BLOCK", d.app_block_bytes),
            max_pending_blocks: env_usize("AUTO_HTTP_UPLOAD_PENDING", d.max_pending_blocks),
            fs_ops_max: env_usize("AUTO_HTTP_UPLOAD_FS_OPS", d.fs_ops_max),
            max_file_bytes: env_u64("AUTO_HTTP_UPLOAD_MAX_FILE_BYTES", d.max_file_bytes),
            max_wire_bytes: env_u64("AUTO_HTTP_UPLOAD_MAX_WIRE_BYTES", d.max_wire_bytes),
            max_text_fields: env_usize(
                "AUTO_HTTP_UPLOAD_MAX_TEXT_FIELDS",
                d.max_text_fields as usize,
            ) as u32,
            max_text_field_bytes: env_usize(
                "AUTO_HTTP_UPLOAD_MAX_TEXT_FIELD_BYTES",
                d.max_text_field_bytes,
            ),
            max_text_total_bytes: env_usize(
                "AUTO_HTTP_UPLOAD_MAX_TEXT_TOTAL_BYTES",
                d.max_text_total_bytes,
            ),
            max_parts: env_usize("AUTO_HTTP_UPLOAD_MAX_PARTS", d.max_parts as usize) as u32,
            max_part_header_bytes: env_usize(
                "AUTO_HTTP_UPLOAD_MAX_PART_HEADER_BYTES",
                d.max_part_header_bytes,
            ),
            max_part_header_items: env_usize(
                "AUTO_HTTP_UPLOAD_MAX_PART_HEADER_ITEMS",
                d.max_part_header_items as usize,
            ) as u32,
            max_boundary_bytes: env_usize("AUTO_HTTP_UPLOAD_BOUNDARY", d.max_boundary_bytes),
            total_timeout_ms: env_u64("AUTO_HTTP_UPLOAD_TOTAL_MS", d.total_timeout_ms),
            idle_timeout_ms: env_u64("AUTO_HTTP_UPLOAD_IDLE_MS", d.idle_timeout_ms),
            lease_timeout_ms: env_u64("AUTO_HTTP_UPLOAD_LEASE_MS", d.lease_timeout_ms),
            queue_timeout_ms: env_u64("AUTO_HTTP_UPLOAD_QUEUE_TIMEOUT_MS", d.queue_timeout_ms),
        }
    }
}

/// 严格解析接收 options（`""` 归一化 `{}`；mode 必填）。限额与硬上限取
/// min——应用 options 不得无界扩大。所有错误在零 I/O 处拒绝。
pub fn parse_upload_receive_options(
    options_json: &str,
    hard: &UploadServeLimits,
) -> Result<UploadReceiveOptions, String> {
    let normalized = if options_json.trim().is_empty() {
        "{}"
    } else {
        options_json
    };
    let raw: UploadOptionsRaw =
        serde_json::from_str(normalized).map_err(|e| format!("upload options: {e}"))?;
    let mode = match raw.mode.as_str() {
        "multipart" => UploadReceiveMode::Multipart,
        "raw" => UploadReceiveMode::Raw,
        other => {
            return Err(format!(
                "upload options: unknown mode {other:?} (multipart|raw)"
            ))
        }
    };
    if raw.max_file_bytes.unwrap_or(1) == 0
        || raw.max_wire_bytes.unwrap_or(1) == 0
        || raw.max_text_fields.unwrap_or(1) == 0
        || raw.max_text_field_bytes.unwrap_or(1) == 0
        || raw.max_text_total_bytes.unwrap_or(1) == 0
        || raw.max_parts.unwrap_or(1) == 0
        || raw.max_part_header_bytes.unwrap_or(1) == 0
        || raw.max_part_header_items.unwrap_or(1) == 0
    {
        return Err(
            "upload options: zero budgets are not a valid way to disable limits".to_string(),
        );
    }
    let file_field = match (&raw.file_field, mode) {
        (Some(f), UploadReceiveMode::Multipart) => {
            if f.is_empty() || f.len() > 128 {
                return Err("upload options: file_field must be 1..=128 chars".to_string());
            }
            f.clone()
        }
        (Some(_), UploadReceiveMode::Raw) => {
            return Err("upload options: file_field is multipart-only".to_string())
        }
        (None, UploadReceiveMode::Multipart) => "file".to_string(),
        (None, UploadReceiveMode::Raw) => String::new(),
    };
    if let Some(fields) = &raw.text_fields {
        if mode == UploadReceiveMode::Raw {
            return Err("upload options: text_fields is multipart-only".to_string());
        }
        if fields.len() > hard.max_text_fields as usize {
            return Err(format!(
                "upload options: text_fields exceeds server hard cap {}",
                hard.max_text_fields
            ));
        }
        for f in fields {
            if f.is_empty() || f.len() > 128 {
                return Err("upload options: text field names must be 1..=128 chars".to_string());
            }
        }
    }
    if let Some(cts) = &raw.raw_content_types {
        if mode == UploadReceiveMode::Multipart {
            return Err("upload options: raw_content_types is raw-only".to_string());
        }
        for ct in cts {
            if ct.len() > 128 || ct.contains('\r') || ct.contains('\n') {
                return Err("upload options: bad raw content type".to_string());
            }
        }
    }
    Ok(UploadReceiveOptions {
        mode,
        file_field,
        text_fields: raw.text_fields.clone().unwrap_or_default(),
        raw_content_types: raw.raw_content_types.clone(),
        max_file_bytes: raw
            .max_file_bytes
            .unwrap_or(u64::MAX)
            .min(hard.max_file_bytes),
        max_wire_bytes: raw
            .max_wire_bytes
            .unwrap_or(u64::MAX)
            .min(hard.max_wire_bytes),
        max_text_fields: raw
            .max_text_fields
            .unwrap_or(u32::MAX)
            .min(hard.max_text_fields),
        max_text_field_bytes: raw
            .max_text_field_bytes
            .unwrap_or(usize::MAX)
            .min(hard.max_text_field_bytes),
        max_text_total_bytes: raw
            .max_text_total_bytes
            .unwrap_or(usize::MAX)
            .min(hard.max_text_total_bytes),
        max_parts: raw.max_parts.unwrap_or(u32::MAX).min(hard.max_parts),
        max_part_header_bytes: raw
            .max_part_header_bytes
            .unwrap_or(usize::MAX)
            .min(hard.max_part_header_bytes),
        max_part_header_items: raw
            .max_part_header_items
            .unwrap_or(u32::MAX)
            .min(hard.max_part_header_items),
    })
}

// ============================================================================
// 宿主 executor hook 与 async facade
// ============================================================================

/// 执行阶段通知（VM 腿用于 scope 期限切换；生成腿为服务级 no-op）。
#[derive(Debug, Clone)]
pub enum UploadPhase {
    /// receive 开始（total deadline = headers 入 scope 起 + total_timeout）。
    ReceiveStarted { total_deadline: std::time::Instant },
    /// 接收完成、staged（lease deadline；业务判定窗口）。
    Staged { lease_deadline: std::time::Instant },
}

pub type UploadPhaseHook = Arc<dyn Fn(UploadPhase) + Send + Sync>;

/// 宿主执行器（`auto_lang::http_upload_service` 注入；owned async hook）。
pub trait UploadExecutor: Send + Sync {
    /// 接收（排队→准入→预检→staging 写入→EOF 校验）；终态含失败快照。
    fn receive(
        &self,
        req: UploadRequest,
        root: &str,
        staging_root: &str,
        options: UploadReceiveOptions,
        phase: UploadPhaseHook,
    ) -> futures::future::BoxFuture<'static, UploadSession>;

    /// create-only 发布（目标冲突 409；gate 后迟到取消不回滚）。
    fn commit(
        &self,
        session_id: u64,
        relative_target: &str,
    ) -> futures::future::BoxFuture<'static, UploadReceipt>;

    /// 清理收口（staged/failed 会话；failed 会话 status=0 → 建议状态）。
    fn reject(
        &self,
        session_id: u64,
        status: i64,
        message: String,
    ) -> futures::future::BoxFuture<'static, UploadReceipt>;

    /// 取消/断连/scope 收口（幂等；commit gate 后的迟到取消不回滚）。
    fn cancel_session(&self, session_id: u64);
}

static UPLOAD_EXECUTOR: OnceLock<Arc<dyn UploadExecutor>> = OnceLock::new();

/// 安装宿主执行器（幂等；首个安装者胜出）。安装点：VM `serve_with` 入口、
/// 生成 main.rs、独立 axum 生成器 main、e2e fixture。
pub fn install_upload_executor(executor: Arc<dyn UploadExecutor>) {
    let _ = UPLOAD_EXECUTOR.set(executor);
}

fn executor() -> Option<Arc<dyn UploadExecutor>> {
    UPLOAD_EXECUTOR.get().cloned()
}

/// `http.upload_receive(req, root, staging_root, options)`：接收完整请求
/// 至 staging（路由/鉴权已先行；body 此前不被解析）。返回接收快照会话
/// （失败也返回 failed 会话——inspect/reject 可用，commit 不能成功）。
/// executor 未安装 → 确定性诊断 failed 会话（零 I/O）。
pub async fn upload_receive(
    req: UploadRequest,
    root: impl AsRef<str>,
    staging_root: impl AsRef<str>,
    options_json: impl AsRef<str>,
) -> UploadSession {
    let Some(exec) = executor() else {
        return failed_session(
            UploadErrorKind::NoExecutor,
            "upload executor not installed (requires the HTTP server host)",
        );
    };
    let hard = UploadServeLimits::from_env();
    let options = match parse_upload_receive_options(options_json.as_ref(), &hard) {
        Ok(o) => o,
        Err(message) => {
            return failed_session(UploadErrorKind::InvalidOptions, message);
        }
    };
    let root = root.as_ref().to_string();
    let staging_root = staging_root.as_ref().to_string();
    exec.receive(req, &root, &staging_root, options, Arc::new(|_| {}))
        .await
}

/// `http.upload_commit(session, relative_target)`：受信 root 内原子
/// create-only 发布（hard_link 原语；存在即 409，绝不先删/覆盖）。
/// 目标词法校验在委托前完成（零 I/O 拒绝）。
pub async fn upload_commit(
    session: UploadSession,
    relative_target: impl AsRef<str>,
) -> UploadReceipt {
    if !session.is_received() {
        return UploadReceipt::failed(
            UploadErrorKind::SessionConflict,
            "upload_commit: cannot commit a failed receive session",
        );
    }
    let target = relative_target.as_ref();
    if let Err(message) = validate_relative_path(target) {
        // PLAN-734 T-05 修复（泄漏）：词法拒绝此前短路返回——Staged 会话的
        // staging 文件与 active 许可滞留到 lease 过期（同目标矩阵多笔时
        // 准入饿死；lease≈准入期限的时序巧合掩盖了它）。拒绝 = 终结会话：
        // 级联 cancel（executor 清理；未安装时 no-op）。
        cancel_upload_session(session.id());
        return UploadReceipt::failed(UploadErrorKind::ForbiddenPath, message);
    }
    let Some(exec) = executor() else {
        return UploadReceipt::failed(
            UploadErrorKind::NoExecutor,
            "upload executor not installed (requires the HTTP server host)",
        );
    };
    exec.commit(session.id, target).await
}

/// `http.upload_reject(session, status, message)`：清理 staging 并返回
/// 收据。failed 会话 `status=0` → 采用失败种类建议状态（省应用样板）。
pub async fn upload_reject(
    session: UploadSession,
    status: i64,
    message: impl AsRef<str>,
) -> UploadReceipt {
    let effective_status = match (&session.state, status) {
        (UploadSessionState::Failed { kind, .. }, 0) => i64::from(kind.suggested_status()),
        (_, s) => s,
    };
    // failed 接收会话：staging 已在失败收口清理——收据直接构造，不经
    // executor（注册表无此 id；Failed 会话唯一合法后续即本形态）。
    let message = message.as_ref();
    if let UploadSessionState::Failed { kind, .. } = &session.state {
        return UploadReceipt {
            status: kind.suggested_status(),
            kind: kind.clone(),
            json: format!(
                "{{\"ok\":false,\"kind\":\"{}\",\"message\":\"{}\"}}",
                kind.as_str(),
                json_escape(message),
            ),
        };
    }
    if !(400..=599).contains(&effective_status) {
        return UploadReceipt::failed(
            UploadErrorKind::InvalidOptions,
            format!("upload_reject: invalid status {effective_status} (must be 400..=599 or 0 for failed sessions)"),
        );
    }
    let Some(exec) = executor() else {
        return UploadReceipt::failed(
            UploadErrorKind::NoExecutor,
            "upload executor not installed (requires the HTTP server host)",
        );
    };
    exec.reject(session.id, effective_status, message.to_string())
        .await
}

/// `http.upload_metadata(session)`：接收快照 JSON（同步零 I/O；转译
/// Rust 直调面——与 `upload_metadata_json` 同实现）。
pub fn upload_metadata(session: &UploadSession) -> String {
    upload_metadata_json(session)
}

/// 会话取消（VM scope 收口/断连级联；幂等）。非 facade 公共 API——宿主桥
/// 专用（Auto 面无对应 native；cancel 由 scope 生命周期驱动）。
pub fn cancel_upload_session(session_id: u64) {
    if let Some(exec) = executor() {
        exec.cancel_session(session_id);
    }
}

// ============================================================================
// 表驱动测试（options 严格性 / receipt / metadata / status 映射）
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn hard() -> UploadServeLimits {
        UploadServeLimits::default()
    }

    #[test]
    fn plan730_options_mode_required() {
        assert!(parse_upload_receive_options("{}", &hard()).is_err());
        assert!(parse_upload_receive_options("", &hard()).is_err());
        let err = parse_upload_receive_options(r#"{"mode":"tus"}"#, &hard()).unwrap_err();
        assert!(err.contains("unknown mode"));
    }

    #[test]
    fn plan730_options_unknown_key_rejected() {
        let err =
            parse_upload_receive_options(r#"{"mode":"raw","root":"/x"}"#, &hard()).unwrap_err();
        assert!(
            err.contains("unknown field") || err.contains("root"),
            "{err}"
        );
    }

    #[test]
    fn plan730_options_mode_specific_keys() {
        // file_field/text_fields 仅 multipart；raw_content_types 仅 raw。
        assert!(
            parse_upload_receive_options(r#"{"mode":"raw","file_field":"f"}"#, &hard()).is_err()
        );
        assert!(
            parse_upload_receive_options(r#"{"mode":"raw","text_fields":["a"]}"#, &hard()).is_err()
        );
        assert!(parse_upload_receive_options(
            r#"{"mode":"multipart","raw_content_types":["a/b"]}"#,
            &hard()
        )
        .is_err());
        let ok = parse_upload_receive_options(
            r#"{"mode":"multipart","text_fields":["note"],"file_field":"doc"}"#,
            &hard(),
        )
        .unwrap();
        assert_eq!(ok.file_field, "doc");
        assert_eq!(ok.text_fields, vec!["note".to_string()]);
    }

    #[test]
    fn plan730_options_cannot_exceed_hard_caps() {
        let opts = parse_upload_receive_options(
            r#"{"mode":"raw","max_file_bytes":999999999999}"#,
            &hard(),
        )
        .unwrap();
        assert_eq!(opts.max_file_bytes, hard().max_file_bytes);
        // 未提供 → 取硬上限值（不是无限）。
        assert_eq!(opts.max_wire_bytes, hard().max_wire_bytes);
    }

    #[test]
    fn plan730_options_zero_budget_rejected() {
        let err = parse_upload_receive_options(r#"{"mode":"raw","max_file_bytes":0}"#, &hard())
            .unwrap_err();
        assert!(err.contains("zero budgets"), "{err}");
    }

    #[test]
    fn plan730_upload_error_status_domain() {
        assert_eq!(upload_error(401, "no auth").status, 401);
        assert_eq!(upload_error(503, "busy").kind, UploadErrorKind::Rejected);
        let bad = upload_error(201, "not an error status");
        assert_eq!(bad.status, 500);
        assert!(bad.json.contains("invalid status 201"));
    }

    #[test]
    fn plan730_failed_session_metadata_shape() {
        let s = failed_session(UploadErrorKind::BodyTooLarge, "wire 70000000 > 65000000");
        let json = upload_metadata_json(&s);
        assert!(json.contains("\"state\":\"failed\""));
        assert!(json.contains("\"kind\":\"body_too_large\""));
        assert!(json.contains("\"suggested_status\":413"));
    }

    #[test]
    fn plan730_received_metadata_and_receipt_shape() {
        let meta = UploadReceivedMeta {
            mode: UploadReceiveMode::Multipart,
            field: "file".to_string(),
            filename: Some("résumé.bin".to_string()),
            size: 12_582_912,
            content_type: Some("application/octet-stream".to_string()),
            fields: vec![
                ("note".to_string(), "a".to_string()),
                ("note".to_string(), "b".to_string()),
            ],
        };
        let s = UploadSession {
            id: 7,
            state: UploadSessionState::Received(meta.clone()),
        };
        let json = upload_metadata_json(&s);
        assert!(json.contains("\"size\":\"12582912\""), "{json}");
        assert!(json.contains("\"state\":\"received\""));
        // 重复字段保序不覆盖。
        let note_pos = json.find("\"note\"").unwrap();
        let note_pos2 = json.rfind("\"note\"").unwrap();
        assert_ne!(note_pos, note_pos2, "duplicate fields preserved");
        let r = UploadReceipt::committed("public/x.bin", &meta);
        assert_eq!(r.status, 201);
        assert!(r.json.contains("\"ok\":true"));
        assert!(r.json.contains("\"path\":\"public/x.bin\""));
        assert!(r.json.contains("\"size\":\"12582912\""));
    }

    #[test]
    fn plan730_kind_status_table() {
        assert_eq!(UploadErrorKind::BodyTooLarge.suggested_status(), 413);
        assert_eq!(UploadErrorKind::UnsupportedMedia.suggested_status(), 415);
        assert_eq!(UploadErrorKind::BadMultipart.suggested_status(), 400);
        assert_eq!(UploadErrorKind::IdleTimeout.suggested_status(), 408);
        assert_eq!(UploadErrorKind::QueueFull.suggested_status(), 503);
        assert_eq!(UploadErrorKind::TargetConflict.suggested_status(), 409);
        assert_eq!(UploadErrorKind::ForbiddenPath.suggested_status(), 403);
        assert_eq!(UploadErrorKind::ExpiredLease.suggested_status(), 410);
    }

    #[test]
    fn plan730_facade_without_executor_is_diagnostic() {
        // facade 在 executor 未安装时返回确定性诊断（零 I/O）——本测试
        // 进程若已安装（其他测试）则跳过语义面，仅当未安装时断言。
        if executor().is_some() {
            return; // 已安装环境（auto-lang 宿主测试进程）不适用本断言。
        }
        let body: UploadBodyStream = Box::pin(futures::stream::empty());
        let req = upload_request_from_parts("POST", "/up", Vec::new(), body);
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let session = rt.block_on(upload_receive(
            req,
            "C:/root",
            "C:/staging",
            r#"{"mode":"raw"}"#,
        ));
        assert!(!session.is_received());
        assert!(upload_metadata_json(&session).contains("no_executor"));
    }

    #[test]
    fn plan730_commit_validates_target_before_delegate() {
        let meta = UploadReceivedMeta {
            mode: UploadReceiveMode::Raw,
            field: String::new(),
            filename: None,
            size: 1,
            content_type: None,
            fields: Vec::new(),
        };
        let s = UploadSession {
            id: 1,
            state: UploadSessionState::Received(meta),
        };
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        for bad in ["..", "../x", "C:/x", "\\\\srv\\share", "a/../../b", ""] {
            let r = rt.block_on(upload_commit(s.clone(), bad));
            assert_eq!(r.status, 403, "target {bad:?} must be lexically rejected");
        }
        // failed 会话直接 409 冲突。
        let f = failed_session(UploadErrorKind::BadMultipart, "truncated");
        let r = rt.block_on(upload_commit(f, "ok.bin"));
        assert_eq!(r.status, 409);
        assert!(r.json.contains("session_conflict"));
    }
}
