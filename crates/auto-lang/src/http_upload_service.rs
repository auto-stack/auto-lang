//! PLAN-730 T-03/T-04：服务端上传宿主 executor——共同接收/落盘/发布执行
//! （VM 默认 HTTP 与生成 Rust 服务共用单源；不含 axum 类型）。
//!
//! 模块边界（决策报告 §2）：
//! - a2r-std `server_upload` 提供 owned 类型/严格 options/async facade；本
//!   模块实现 [`a2r_std::http::UploadExecutor`] 并经
//!   [`install_upload_executor_service`] 注入（幂等）。
//! - 增量 multipart 状态机为本模块私有（拒绝 multer：per-part-header/carry
//!   预算必须外部可施；子集窄——单文件+有界文本、无嵌套、identity-only）。
//! - staging：受信 `staging_root` 内 `create_new` 独占文件（unix 0600）；
//!   EOF+flush+sync 后才 Staged；lease 到期/取消/失败清理。
//! - 发布：父目录 no-follow walk（729 同算法形态）→ `hard_link`（std 级
//!   原子 create-only，Windows ERROR_ALREADY_EXISTS/Linux EEXIST）→ 删
//!   staging。普通 rename 覆盖目标（探针 P2 反证），不符合合同。
//! - 取消仲裁：commit gate = `hard_link` syscall 本身；gate 前取消胜出
//!   （不发布、清理），gate 后迟到取消等实际结果（成功保留、失败清理）。
//! - 磁盘不强制 abort：停止 issuance、等在途写返回后清柄；writer 任务在
//!   通道关闭且未 Finish 时自清 staging（覆盖调用方 future 被 drop 的路径）。

use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use a2r_std::http::server_upload::{
    failed_session, UploadErrorKind, UploadExecutor, UploadPhase, UploadPhaseHook,
    UploadReceiveMode, UploadReceiveOptions, UploadReceivedMeta, UploadRequest, UploadReceipt,
    UploadServeLimits, UploadSession,
};
use tokio::io::AsyncWriteExt;

// ============================================================================
// 全局服务状态（限额信号量 + 会话注册表 + 计数探针）
// ============================================================================

struct UploadServiceState {
    queue: std::sync::Arc<tokio::sync::Semaphore>,
    active: std::sync::Arc<tokio::sync::Semaphore>,
    fs_ops: std::sync::Arc<tokio::sync::Semaphore>,
    registry: Mutex<HashMap<u64, Arc<SessionEntry>>>,
    limits: UploadServeLimits,
}

lazy_static::lazy_static! {
    static ref SERVICE: UploadServiceState = {
        let limits = UploadServeLimits::from_env();
        UploadServiceState {
            queue: std::sync::Arc::new(tokio::sync::Semaphore::new(limits.queue_capacity)),
            active: std::sync::Arc::new(tokio::sync::Semaphore::new(limits.max_active)),
            fs_ops: std::sync::Arc::new(tokio::sync::Semaphore::new(limits.fs_ops_max)),
            registry: Mutex::new(HashMap::new()),
            limits,
        }
    };
}

static SESSION_ID_GEN: AtomicU64 = AtomicU64::new(1);
static RESIDUAL_CLEANUP_FAILS: AtomicUsize = AtomicUsize::new(0);
static LATE_CANCELS: AtomicUsize = AtomicUsize::new(0);

/// lease 过期墓碑（有界环，容量 64）：过期会话已出注册表，迟到 commit 仍
/// 需要真实的 410 而非"unknown session"的 409（§5.4 lease 可观测性）。
static EXPIRED_TOMBSTONES: Mutex<std::collections::VecDeque<u64>> =
    Mutex::new(std::collections::VecDeque::new());

fn record_expired_tombstone(session_id: u64) {
    if let Ok(mut q) = EXPIRED_TOMBSTONES.lock() {
        if q.len() >= 64 {
            q.pop_front();
        }
        q.push_back(session_id);
    }
}

fn is_expired_tombstone(session_id: u64) -> bool {
    EXPIRED_TOMBSTONES
        .lock()
        .map(|q| q.contains(&session_id))
        .unwrap_or(false)
}

/// 会话生命周期状态（注册表内；终态不可逆）。
enum SessionPhase {
    /// 排队/接收中（active 许可由 entry 持有；lease 未开始）。
    Receiving,
    /// 接收完成、staged（lease 看门狗在跑；业务判定窗口）。
    Staged {
        meta: UploadReceivedMeta,
        staging_path: PathBuf,
    },
    /// 进入 commit gate（hard_link 在途；迟到取消不回滚）。收据快照在
    /// run_commit 局部——相内只携带发布路径（迟到取消的可观测面）。
    Committing { staging_path: PathBuf },
    /// 终态：committed/rejected/expired_lease/cancelled/failed。
    Terminal(&'static str),
}

struct SessionEntry {
    id: u64,
    phase: Mutex<SessionPhase>,
    /// 取消信号（scope 收口/断连级联；watch 允许多等待者）。
    cancel: tokio::sync::watch::Sender<bool>,
    /// active/queue 许可（Receiving/Staged/Committing 持 active 至终态）。
    active_permit: Mutex<Option<tokio::sync::OwnedSemaphorePermit>>,
    queue_permit: Mutex<Option<tokio::sync::OwnedSemaphorePermit>>,
    root: PathBuf,
    staging_root: PathBuf,
    /// lease 看门狗 abort 句柄（commit/reject/过期自身取消）。
    lease_guard: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

impl SessionEntry {
    fn phase_name(&self) -> &'static str {
        match &*self.phase.lock().unwrap() {
            SessionPhase::Receiving => "receiving",
            SessionPhase::Staged { .. } => "staged",
            SessionPhase::Committing { .. } => "committing",
            SessionPhase::Terminal(t) => t,
        }
    }
}

fn limits() -> UploadServeLimits {
    SERVICE.limits.clone()
}

/// 安装宿主 executor（幂等；VM serve_with 入口/生成 main/e2e fixture 调用）。
pub fn install_upload_executor_service() {
    a2r_std::http::install_upload_executor(Arc::new(UploadExecutorImpl));
}

// ── 测试/资源探针 ──────────────────────────────────────────────────────────

/// 存活会话条目数（资源回基线探针）。
pub fn upload_session_count() -> usize {
    SERVICE.registry.lock().map(|r| r.len()).unwrap_or(0)
}

/// staged 会话数。
pub fn upload_staged_count() -> usize {
    SERVICE
        .registry
        .lock()
        .map(|r| {
            r.values()
                .filter(|e| matches!(*e.phase.lock().unwrap(), SessionPhase::Staged { .. }))
                .count()
        })
        .unwrap_or(0)
}

/// 可用 active 许可数。
pub fn upload_active_available() -> usize {
    SERVICE.active.available_permits()
}

/// 可用 queue 许可数。
pub fn upload_queue_available() -> usize {
    SERVICE.queue.available_permits()
}

/// cleanup 失败累计（残留可观测性——不宣称零残留）。
pub fn upload_residual_cleanup_failures() -> usize {
    RESIDUAL_CLEANUP_FAILS.load(Ordering::SeqCst)
}

/// gate 后迟到取消计数（仲裁正确性探针）。
pub fn upload_late_cancel_count() -> usize {
    LATE_CANCELS.load(Ordering::SeqCst)
}

// ============================================================================
// 预检：受信目录配置（同卷/嵌套/存在性——读取 body 之前全部拒绝）
// ============================================================================

fn volume_key(p: &Path) -> io::Result<String> {
    let canon = std::fs::canonicalize(p)?;
    let s = canon.to_string_lossy().to_string();
    #[cfg(windows)]
    {
        if let Some(rest) = s.strip_prefix(r"\\?\") {
            if let Some(idx) = rest.find('\\') {
                return Ok(rest[..idx].to_ascii_lowercase());
            }
        }
        Ok(s.to_ascii_lowercase())
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        Ok(std::fs::metadata(&canon)?.dev().to_string())
    }
}

fn lexically_inside(parent: &Path, child: &Path) -> bool {
    match (std::fs::canonicalize(parent), std::fs::canonicalize(child)) {
        (Ok(p), Ok(c)) => c.starts_with(&p),
        _ => false,
    }
}

/// 受信目录配置校验（零 body 读取）：两目录存在且为目录、同卷、互不嵌套。
fn validate_roots(root: &str, staging_root: &str) -> Result<(), String> {
    let root_p = PathBuf::from(root);
    let staging_p = PathBuf::from(staging_root);
    if !root_p.is_absolute() || !staging_p.is_absolute() {
        return Err("upload roots must be absolute trusted paths".to_string());
    }
    for (label, p) in [("root", &root_p), ("staging_root", &staging_p)] {
        match std::fs::metadata(p) {
            Ok(md) if md.is_dir() => {}
            Ok(_) => return Err(format!("upload {label} is not a directory: {}", p.display())),
            Err(e) => return Err(format!("upload {label} missing: {e}")),
        }
    }
    if volume_key(&root_p).ok() != volume_key(&staging_p).ok() {
        return Err(format!(
            "upload staging_root is on a different volume than root (hard_link publish requires same volume): {} vs {}",
            staging_p.display(),
            root_p.display()
        ));
    }
    if lexically_inside(&root_p, &staging_p) || lexically_inside(&staging_p, &root_p) {
        return Err(
            "upload staging_root must live outside the public root (and not contain it)".to_string(),
        );
    }
    Ok(())
}

fn header_value<'a>(headers: &'a [(String, String)], name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(name))
        .map(|(_, v)| v.as_str())
}

// ============================================================================
// 增量 multipart 状态机（RFC 2046/7578 子集；决策报告 §4.8）
// ============================================================================
//
// 事件模型：`feed` 推进状态并把**文件内容块**交给调用方（转发 writer）；
// 文本字段由解析器内部累计（逐字段预算 + UTF-8 校验 + 有序入表）。carry
// 上限 = needle 长度（假前缀经后缀保留跨块匹配，回吐为内容）。

/// delimiter 后继判定（buf 以 needle 开头时调用）。
enum DelimStatus {
    /// 后继字节不足，等待更多（needle 保留在 buf）。
    NeedMore,
    /// 不是 delimiter（假前缀）：needle 字节按内容处理。
    NotDelimiter,
    /// part 边界（`CRLF` 或 ≤16B transport padding + `CRLF`）。
    PartBoundary,
    /// close delimiter（`--` + 可选 padding/CRLF——尾部一律按 epilogue 丢弃）。
    CloseDelimiter,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum MpState {
    Preamble,
    PartHeaders,
    FileContent,
    TextContent,
    Epilogue,
}

pub(crate) struct MultipartIncremental {
    /// 完整定界符 needle = CRLF + "--" + boundary（boundary 随 needle 携带）。
    delim: Vec<u8>,
    buf: Vec<u8>,
    state: MpState,
    header_block: Vec<u8>,
    max_part_header_bytes: usize,
    max_part_header_items: u32,
    file_field: String,
    text_fields: Vec<String>,
    // 结果累积（快照载荷）。
    file_seen: bool,
    file_size: u64,
    fields: Vec<(String, String)>,
    text_total: usize,
    file_content_type: Option<String>,
    file_filename: Option<String>,
    // 当前 part。
    cur_name: String,
    cur_text: Vec<u8>,
}

impl MultipartIncremental {
    pub(crate) fn new(
        boundary: &str,
        options: &UploadReceiveOptions,
        hard: &UploadServeLimits,
    ) -> Result<Self, UploadErrorKind> {
        if boundary.is_empty() || boundary.len() > hard.max_boundary_bytes {
            return Err(UploadErrorKind::BadMultipart);
        }
        // RFC 2046 bcharsnospace（0x21..0x7E 可见非空 ASCII；尾随 SP 子集不放宽）。
        if boundary.bytes().any(|b| !(0x21..=0x7E).contains(&b)) {
            return Err(UploadErrorKind::BadMultipart);
        }
        let mut delim = Vec::with_capacity(boundary.len() + 4);
        delim.extend_from_slice(b"\r\n--");
        delim.extend_from_slice(boundary.as_bytes());
        // 预置合成 CRLF：首边界（RFC 允许开头无 CRLF）与内容定界符共用
        // 同一 needle，Preamble 状态无需独立仲裁（wire 计数在调用方）。
        let mut buf = Vec::with_capacity(64);
        buf.extend_from_slice(b"\r\n");
        Ok(MultipartIncremental {
            delim,
            buf,
            state: MpState::Preamble,
            header_block: Vec::new(),
            max_part_header_bytes: options.max_part_header_bytes,
            max_part_header_items: options.max_part_header_items,
            file_field: options.file_field.clone(),
            text_fields: options.text_fields.clone(),
            file_seen: false,
            file_size: 0,
            fields: Vec::new(),
            text_total: 0,
            file_content_type: None,
            file_filename: None,
            cur_name: String::new(),
            cur_text: Vec::new(),
        })
    }

    /// 喂入一块字节；`out_file_chunks` 收集本批文件内容（调用方转发
    /// writer）。文本内容由解析器内部累计。
    pub(crate) fn feed(
        &mut self,
        chunk: &[u8],
        out_file_chunks: &mut Vec<Vec<u8>>,
        options: &UploadReceiveOptions,
    ) -> Result<(), UploadErrorKind> {
        self.buf.extend_from_slice(chunk);
        if cfg!(test) && std::env::var("P730_DEBUG").is_ok() {
            eprintln!(
                "[feed] state={:?} buf={:?} hdr={}",
                self.state,
                String::from_utf8_lossy(&self.buf),
                String::from_utf8_lossy(&self.header_block)
            );
        }
        loop {
            match self.state {
                MpState::Preamble => {
                    // 首个 dash-boundary（构造期合成 CRLF 使其与内容定界符
                    // 共用 needle）。仲裁期间 needle 保留在 buf 头部。
                    let Some(pos) = find_sub(&self.buf, &self.delim) else {
                        retain_suffix_prefix(&mut self.buf, &self.delim);
                        return Ok(());
                    };
                    self.buf.drain(..pos); // preamble 内容丢弃，buf 以 delim 开头
                    match self.delimiter_status() {
                        (DelimStatus::NeedMore, _) => return Ok(()),
                        (DelimStatus::NotDelimiter, _) => {
                            // 假前缀（`--boundaryX`）：跳过本候选继续找。
                            self.buf.drain(..self.delim.len());
                            continue;
                        }
                        (DelimStatus::PartBoundary, drain) => {
                            self.buf.drain(..drain);
                            self.enter_headers();
                            continue;
                        }
                        (DelimStatus::CloseDelimiter, _) => {
                            self.buf.drain(..self.delim.len() + 2);
                            self.close_delimiter(options)?;
                            continue;
                        }
                    }
                }
                MpState::PartHeaders => {
                    let Some(pos) = find_sub(&self.buf, b"\r\n\r\n") else {
                        let hold = suffix_prefix_len(&self.buf, b"\r\n\r\n");
                        let emit = self.buf.len() - hold;
                        self.header_block.extend_from_slice(&self.buf[..emit]);
                        self.buf.drain(..emit);
                        if self.header_block.len() > self.max_part_header_bytes {
                            return Err(UploadErrorKind::BadMultipart);
                        }
                        return Ok(());
                    };
                    self.header_block.extend_from_slice(&self.buf[..pos]);
                    self.buf.drain(..pos + 4);
                    if self.header_block.len() > self.max_part_header_bytes {
                        return Err(UploadErrorKind::BadMultipart);
                    }
                    self.start_part()?;
                    continue;
                }
                MpState::FileContent | MpState::TextContent => {
                    let Some(pos) = find_sub(&self.buf, &self.delim) else {
                        // 无完整 needle：保留可能的部分后缀，其余为内容。
                        let hold = suffix_prefix_len(&self.buf, &self.delim);
                        let emit = self.buf.len() - hold;
                        let content = self.buf[..emit].to_vec();
                        self.buf.drain(..emit);
                        self.deliver_content(&content, out_file_chunks, options)?;
                        return Ok(());
                    };
                    // 找到候选 needle：后继判定需要 needle 后至少 2 字节
                    //（或 padding 扫描）；不足则先产出确定内容再等待。
                    let content = self.buf[..pos].to_vec();
                    self.buf.drain(..pos);
                    self.deliver_content(&content, out_file_chunks, options)?;
                    match self.delimiter_status() {
                        (DelimStatus::NeedMore, _) => return Ok(()),
                        (DelimStatus::NotDelimiter, _) => {
                            // 假前缀：needle 字节作为内容产出后继续扫描。
                            let needle_len = self.delim.len();
                            let needle_bytes = self.buf[..needle_len].to_vec();
                            self.buf.drain(..needle_len);
                            self.deliver_content(&needle_bytes, out_file_chunks, options)?;
                            continue;
                        }
                        (DelimStatus::PartBoundary, drain) => {
                            self.buf.drain(..drain);
                            self.finish_current_field(options)?;
                            self.enter_headers();
                            continue;
                        }
                        (DelimStatus::CloseDelimiter, _) => {
                            self.buf.drain(..self.delim.len() + 2);
                            self.finish_current_field(options)?;
                            self.close_delimiter(options)?;
                            continue;
                        }
                    }
                }
                MpState::Epilogue => {
                    // 只计数（wire 计数在调用方）；缓冲清空防驻留。
                    self.buf.clear();
                    return Ok(());
                }
            }
        }
    }

    /// EOF：仅 close 之后的 Epilogue 合法（截断/缺 closing boundary → 400）。
    pub(crate) fn finish(&mut self) -> Result<(), UploadErrorKind> {
        match self.state {
            MpState::Epilogue => Ok(()),
            _ => Err(UploadErrorKind::BadMultipart),
        }
    }

    pub(crate) fn snapshot(&self, file_field: &str) -> UploadReceivedMeta {
        UploadReceivedMeta {
            mode: UploadReceiveMode::Multipart,
            field: file_field.to_string(),
            filename: self.file_filename.clone(),
            size: self.file_size,
            content_type: self.file_content_type.clone(),
            fields: self.fields.clone(),
        }
    }

    fn enter_headers(&mut self) {
        self.state = MpState::PartHeaders;
        self.header_block.clear();
    }

    /// buf 以 needle 开头时的后继判定（≤16B transport padding 容忍）。
    /// 纯读决策——PartBoundary 时返回应 drain 的字节数（调用方执行）。
    fn delimiter_status(&self) -> (DelimStatus, usize) {
        let n = self.delim.len();
        let b = &self.buf[..];
        if b.len() < n + 1 {
            return (DelimStatus::NeedMore, 0);
        }
        let next = b[n];
        if next == b'-' {
            if b.len() < n + 2 {
                return (DelimStatus::NeedMore, 0);
            }
            if b[n + 1] == b'-' {
                // 尾部 padding/CRLF/EOF 均按 epilogue 丢弃。
                return (DelimStatus::CloseDelimiter, 0);
            }
            return (DelimStatus::NotDelimiter, 0);
        }
        if next == b'\r' {
            if b.len() < n + 2 {
                return (DelimStatus::NeedMore, 0);
            }
            if b[n + 1] == b'\n' {
                return (DelimStatus::PartBoundary, n + 2);
            }
            return (DelimStatus::NotDelimiter, 0);
        }
        if next == b'\n' {
            // 宽容裸 LF（生成器现实）；part 头仍按 CRLF 解析。
            return (DelimStatus::PartBoundary, n + 1);
        }
        // transport padding（SP/HTAB，≤16B）后接 CRLF。
        if next == b' ' || next == b'\t' {
            let mut i = n;
            let pad_cap = n + 16;
            while i < b.len() && (b[i] == b' ' || b[i] == b'\t') && i < pad_cap {
                i += 1;
            }
            if i >= b.len() || i >= pad_cap {
                return (DelimStatus::NeedMore, 0);
            }
            if b[i] == b'\r' {
                if b.len() < i + 2 {
                    return (DelimStatus::NeedMore, 0);
                }
                if b[i + 1] == b'\n' {
                    return (DelimStatus::PartBoundary, i + 2);
                }
                return (DelimStatus::NotDelimiter, 0);
            }
            if b[i] == b'\n' {
                return (DelimStatus::PartBoundary, i + 1);
            }
        }
        (DelimStatus::NotDelimiter, 0)
    }

    /// 内容交付（文件 → out；文本 → cur_text 累计，预算即时检查）。
    fn deliver_content(
        &mut self,
        bytes: &[u8],
        out_file_chunks: &mut Vec<Vec<u8>>,
        options: &UploadReceiveOptions,
    ) -> Result<(), UploadErrorKind> {
        match self.state {
            MpState::FileContent => {
                self.file_size += bytes.len() as u64;
                if self.file_size > options.max_file_bytes {
                    return Err(UploadErrorKind::BodyTooLarge);
                }
                if !bytes.is_empty() {
                    out_file_chunks.push(bytes.to_vec());
                }
            }
            MpState::TextContent => {
                if self.cur_text.len() + bytes.len() > options.max_text_field_bytes {
                    return Err(UploadErrorKind::BadMultipart);
                }
                if self.text_total + self.cur_text.len() + bytes.len()
                    > options.max_text_total_bytes
                {
                    return Err(UploadErrorKind::BadMultipart);
                }
                self.cur_text.extend_from_slice(bytes);
            }
            _ => unreachable!("deliver_content only from content states"),
        }
        Ok(())
    }

    /// 当前字段收尾（part 边界/close 前）：文本 UTF-8 校验 + 有序入表。
    fn finish_current_field(&mut self, options: &UploadReceiveOptions) -> Result<(), UploadErrorKind> {
        if self.state == MpState::TextContent {
            if self.fields.len() + 1 > options.max_text_fields as usize {
                return Err(UploadErrorKind::BadMultipart);
            }
            let value = String::from_utf8(std::mem::take(&mut self.cur_text))
                .map_err(|_| UploadErrorKind::BadMultipart)?;
            self.text_total += value.len();
            self.fields.push((self.cur_name.clone(), value));
        }
        Ok(())
    }

    /// close delimiter：无 file part → 错误；进入 epilogue。
    fn close_delimiter(&mut self, _options: &UploadReceiveOptions) -> Result<(), UploadErrorKind> {
        if !self.file_seen {
            return Err(UploadErrorKind::BadMultipart); // 无 file part
        }
        self.state = MpState::Epilogue;
        self.buf.clear();
        Ok(())
    }

    /// part 头块解析完毕：字段分派（file/text/未知）+ 不支持形式拒绝。
    fn start_part(&mut self) -> Result<(), UploadErrorKind> {
        let headers = String::from_utf8_lossy(&self.header_block).to_string();
        let mut items = 0u32;
        let mut name: Option<String> = None;
        let mut filename: Option<String> = None;
        let mut content_type: Option<String> = None;
        let mut has_cte = false;
        for line in headers.split("\r\n") {
            if line.is_empty() {
                continue;
            }
            items += 1;
            if items > self.max_part_header_items {
                return Err(UploadErrorKind::BadMultipart);
            }
            let lower = line.to_ascii_lowercase();
            if lower.starts_with("content-disposition:") {
                for attr in line.split(';').skip(1) {
                    let attr = attr.trim();
                    if let Some(v) = attr.strip_prefix("name=") {
                        name = Some(unquote(v));
                    } else if let Some(v) = attr.strip_prefix("filename=") {
                        filename = Some(unquote(v));
                    }
                }
            } else if lower.starts_with("content-type:") {
                let v = line[13..].trim().to_string();
                if v.starts_with("multipart/") {
                    return Err(UploadErrorKind::BadMultipart); // 嵌套 multipart 拒绝
                }
                if let Some(cs) = charset_of(&v) {
                    if !cs.eq_ignore_ascii_case("utf-8") {
                        return Err(UploadErrorKind::UnsupportedMedia);
                    }
                }
                content_type = Some(media_of(&v));
            } else if lower.starts_with("content-transfer-encoding:") {
                has_cte = true;
            }
        }
        if has_cte {
            return Err(UploadErrorKind::BadMultipart); // CTE 不在 identity 子集
        }
        let Some(name) = name else {
            return Err(UploadErrorKind::BadMultipart);
        };
        if name == self.file_field {
            if self.file_seen {
                return Err(UploadErrorKind::BadMultipart); // 第二 file
            }
            self.file_seen = true;
            // filename 仅 metadata（lossy 截断；绝不作为存储路径）。
            self.file_filename = filename.map(|f| {
                String::from_utf8_lossy(&f.into_bytes())
                    .chars()
                    .take(255)
                    .collect::<String>()
            });
            self.file_content_type = content_type;
            self.state = MpState::FileContent;
        } else if self.text_fields.iter().any(|f| f == &name) {
            self.state = MpState::TextContent;
        } else {
            return Err(UploadErrorKind::BadMultipart); // 未知字段
        }
        self.cur_name = name;
        self.cur_text = Vec::new();
        Ok(())
    }
}

fn find_sub(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    haystack.windows(needle.len()).position(|w| w == needle)
}

/// buf 尾部中 needle 真前缀的长度（跨块匹配 hold；≤ needle.len()-1）。
fn suffix_prefix_len(buf: &[u8], needle: &[u8]) -> usize {
    let max = needle.len() - 1;
    for k in (1..=max.min(buf.len())).rev() {
        if buf[buf.len() - k..] == needle[..k] {
            return k;
        }
    }
    0
}

fn retain_suffix_prefix(buf: &mut Vec<u8>, needle: &[u8]) {
    let hold = suffix_prefix_len(buf, needle);
    let emit = buf.len() - hold;
    buf.drain(..emit);
}

fn unquote(v: &str) -> String {
    let v = v.trim();
    if v.len() >= 2 && v.starts_with('"') && v.ends_with('"') {
        v[1..v.len() - 1]
            .replace("\\\"", "\"")
            .replace("\\\\", "\\")
    } else {
        v.to_string()
    }
}

fn media_of(ct: &str) -> String {
    ct.split(';').next().unwrap_or("").trim().to_ascii_lowercase()
}

fn charset_of(ct: &str) -> Option<String> {
    ct.split(';').skip(1).find_map(|p| {
        p.trim()
            .strip_prefix("charset=")
            .map(|s| s.trim().trim_matches('"').to_string())
    })
}

// ============================================================================
// Executor 实现
// ============================================================================

struct UploadExecutorImpl;

/// VM 桥直连入口（phase hook 透传——scope 期限切换）；生成腿经 a2r facade。
pub fn receive_with_phase(
    req: UploadRequest,
    root: &str,
    staging_root: &str,
    options_json: &str,
    phase: UploadPhaseHook,
) -> futures::future::BoxFuture<'static, UploadSession> {
    let hard = limits();
    let options = match a2r_std::http::parse_upload_receive_options(options_json, &hard) {
        Ok(o) => o,
        Err(message) => {
            let session = failed_session(UploadErrorKind::InvalidOptions, message);
            return Box::pin(async move { session });
        }
    };
    let root = root.to_string();
    let staging_root = staging_root.to_string();
    Box::pin(async move { run_receive(req, &root, &staging_root, options, hard, phase).await })
}

impl UploadExecutor for UploadExecutorImpl {
    fn receive(
        &self,
        req: UploadRequest,
        root: &str,
        staging_root: &str,
        options: UploadReceiveOptions,
        phase: UploadPhaseHook,
    ) -> futures::future::BoxFuture<'static, UploadSession> {
        let hard = limits();
        let root = root.to_string();
        let staging_root = staging_root.to_string();
        Box::pin(async move { run_receive(req, &root, &staging_root, options, hard, phase).await })
    }

    fn commit(
        &self,
        session_id: u64,
        relative_target: &str,
    ) -> futures::future::BoxFuture<'static, UploadReceipt> {
        let target = relative_target.to_string();
        Box::pin(async move { run_commit(session_id, target).await })
    }

    fn reject(
        &self,
        session_id: u64,
        status: i64,
        message: String,
    ) -> futures::future::BoxFuture<'static, UploadReceipt> {
        Box::pin(async move { run_reject(session_id, status, message).await })
    }

    fn cancel_session(&self, session_id: u64) {
        cancel_session(session_id);
    }
}

fn with_entry<R>(id: u64, f: impl FnOnce(&Arc<SessionEntry>) -> R) -> Option<R> {
    let entry = SERVICE.registry.lock().ok().and_then(|r| r.get(&id).cloned());
    entry.map(|e| f(&e))
}

/// 取消会话（幂等；scope 收口/断连级联）。Committing 期取消记迟到（不回滚）。
pub fn cancel_session(session_id: u64) {
    let Some(entry) = with_entry(session_id, |e| e.clone()) else { return };
    let staged_path: Option<PathBuf> = {
        let mut phase = entry.phase.lock().unwrap();
        match &*phase {
            SessionPhase::Committing { staging_path, .. } => {
                // 迟到取消（gate 后不回滚）——保留发布路径可观测。
                eprintln!(
                    "[upload730] session {session_id} late cancel during commit ({} stays published)",
                    staging_path.display()
                );
                None
            }
            SessionPhase::Staged { staging_path, .. } => {
                let p = staging_path.clone();
                *phase = SessionPhase::Terminal("cancelled");
                Some(p)
            }
            _ => None,
        }
    };
    match staged_path {
        Some(path) => {
            release_permits(&entry);
            match tokio::runtime::Handle::try_current() {
                Ok(handle) => {
                    handle.spawn(cleanup_staging_path(path, session_id));
                }
                // 无 runtime 上下文：同步有界清理（删除单文件，快）。
                Err(_) => {
                    let _ = std::fs::remove_file(&path);
                }
            }
        }
        None => {
            if matches!(
                &*entry.phase.lock().unwrap(),
                SessionPhase::Committing { .. }
            ) {
                LATE_CANCELS.fetch_add(1, Ordering::SeqCst);
            } else {
                let _ = entry.cancel.send(true);
            }
        }
    }
}

// ── writer 任务（staging 落盘；通道关闭未 Finish 时自清） ────────────────

enum WriteCmd {
    Block(Vec<u8>),
    /// flush + sync + close + 回执（接收完成的唯一提交信号）。
    Finish,
}

struct WriterHandle {
    tx: tokio::sync::mpsc::Sender<WriteCmd>,
    finish_rx: tokio::sync::oneshot::Receiver<io::Result<()>>,
}

async fn spawn_writer(path: PathBuf, session_id: u64) -> io::Result<WriterHandle> {
    // fs_ops 许可覆盖 open+write+sync 生命期（writer 退出即释放）。
    let permit = SERVICE
        .fs_ops
        .clone()
        .acquire_owned()
        .await
        .map_err(|e| io::Error::other(format!("fs ops pool: {e}")))?;
    let file = tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .await?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
    }
    let (tx, mut rx) = tokio::sync::mpsc::channel::<WriteCmd>(1); // +1 在写 = ≤2 待写块
    let (finish_tx, finish_rx) = tokio::sync::oneshot::channel::<io::Result<()>>();
    tokio::spawn(async move {
        let _permit = permit;
        let mut outcome: io::Result<()> = Ok(());
        let mut finished = false;
        let mut file = Some(file);
        while let Some(cmd) = rx.recv().await {
            match cmd {
                WriteCmd::Block(b) => {
                    if outcome.is_ok() {
                        if let Some(f) = file.as_mut() {
                            outcome = f.write_all(&b).await;
                        }
                    }
                }
                WriteCmd::Finish => {
                    finished = true;
                    if outcome.is_ok() {
                        if let Some(f) = file.as_mut() {
                            outcome = async {
                                f.flush().await?;
                                f.sync_all().await?;
                                Ok(())
                            }
                            .await;
                        }
                    }
                    file = None;
                    let _ = finish_tx.send(outcome);
                    break;
                }
            }
        }
        // 通道关闭（取消/调用方 drop）且未 Finish：关柄自清 staging
        //（不残留临时文件；磁盘慢时在途写返回后退出——不强制 abort）。
        drop(file);
        if !finished {
            if let Err(e) = tokio::fs::remove_file(&path).await {
                RESIDUAL_CLEANUP_FAILS.fetch_add(1, Ordering::SeqCst);
                eprintln!(
                    "[upload730] staging cleanup after writer drop failed (session {session_id}): {e} — residual {} (private dir)",
                    path.display()
                );
            }
        }
    });
    Ok(WriterHandle { tx, finish_rx })
}

// ── 接收主流程 ────────────────────────────────────────────────────────────

async fn run_receive(
    req: UploadRequest,
    root: &str,
    staging_root: &str,
    options: UploadReceiveOptions,
    hard: UploadServeLimits,
    phase: UploadPhaseHook,
) -> UploadSession {
    let started = Instant::now();
    let total_deadline = started + Duration::from_millis(hard.total_timeout_ms);
    let idle_timeout = Duration::from_millis(hard.idle_timeout_ms);

    // 排队（不读 body、不开文件）：queue 许可或 queue 期限/total。
    let queue_permit = {
        let deadline =
            total_deadline.min(started + Duration::from_millis(hard.queue_timeout_ms));
        tokio::select! {
            p = SERVICE.queue.clone().acquire_owned() => match p {
                Ok(p) => p,
                Err(_) => return failed_session(UploadErrorKind::Io, "upload queue closed"),
            },
            _ = tokio::time::sleep_until(tokio::time::Instant::from_std(deadline)) => {
                return failed_session(UploadErrorKind::QueueFull, "upload queue wait timed out")
            }
        }
    };

    let session_id = SESSION_ID_GEN.fetch_add(1, Ordering::SeqCst);
    let (cancel_tx, cancel_rx) = tokio::sync::watch::channel(false);
    let entry = Arc::new(SessionEntry {
        id: session_id,
        phase: Mutex::new(SessionPhase::Receiving),
        cancel: cancel_tx,
        active_permit: Mutex::new(None),
        queue_permit: Mutex::new(Some(queue_permit)),
        root: PathBuf::from(root),
        staging_root: PathBuf::from(staging_root),
        lease_guard: Mutex::new(None),
    });
    if let Ok(mut r) = SERVICE.registry.lock() {
        r.insert(session_id, entry.clone());
    }

    // active 许可（total/cancel 内）。
    let active_permit = tokio::select! {
        p = SERVICE.active.clone().acquire_owned() => match p {
            Ok(p) => p,
            Err(_) => {
                let s = fail_receive(&entry, UploadErrorKind::Io, "upload active pool closed").await;
                return s;
            }
        },
        _ = tokio::time::sleep_until(tokio::time::Instant::from_std(total_deadline)) => {
            return fail_receive(&entry, UploadErrorKind::QueueFull, "upload admission timed out").await;
        }
        _ = wait_cancel(&cancel_rx) => {
            return fail_receive(&entry, UploadErrorKind::Cancelled, "upload cancelled").await;
        }
    };
    *entry.queue_permit.lock().unwrap() = None;
    *entry.active_permit.lock().unwrap() = Some(active_permit);

    // 预检（零 body 读取）。
    if let Err(message) = validate_roots(root, staging_root) {
        return fail_receive(&entry, UploadErrorKind::InvalidOptions, message).await;
    }
    let req_headers = req.headers().to_vec();
    if let Some(ce) = header_value(&req_headers, "content-encoding") {
        if !ce.trim().eq_ignore_ascii_case("identity") {
            return fail_receive(
                &entry,
                UploadErrorKind::UnsupportedMedia,
                format!("Content-Encoding {ce:?} not supported (identity only)"),
            )
            .await;
        }
    }
    let content_type_raw = header_value(&req_headers, "content-type").unwrap_or("").to_string();
    let content_type = media_of(&content_type_raw);

    let mut parser = None;
    match options.mode {
        UploadReceiveMode::Multipart => {
            if content_type != "multipart/form-data" {
                return fail_receive(
                    &entry,
                    UploadErrorKind::UnsupportedMedia,
                    format!("mode=multipart requires multipart/form-data (got {content_type:?})"),
                )
                .await;
            }
            let boundary = content_type_raw
                .split(';')
                .find_map(|p| p.trim().strip_prefix("boundary="))
                .map(|b| b.trim().trim_matches('"').to_string());
            let Some(boundary) = boundary else {
                return fail_receive(
                    &entry,
                    UploadErrorKind::BadMultipart,
                    "multipart/form-data without boundary".to_string(),
                )
                .await;
            };
            match MultipartIncremental::new(&boundary, &options, &hard) {
                Ok(p) => parser = Some(p),
                Err(kind) => {
                    return fail_receive(
                        &entry,
                        kind,
                        "invalid multipart boundary".to_string(),
                    )
                    .await
                }
            }
        }
        UploadReceiveMode::Raw => {
            if content_type.starts_with("multipart/") {
                return fail_receive(
                    &entry,
                    UploadErrorKind::UnsupportedMedia,
                    "mode=raw rejects multipart bodies (use mode=multipart)".to_string(),
                )
                .await;
            }
            if let Some(allowed) = &options.raw_content_types {
                let ct = if content_type.is_empty() {
                    "application/octet-stream".to_string()
                } else {
                    content_type.clone()
                };
                if !allowed.iter().any(|a| a.eq_ignore_ascii_case(&ct)) {
                    return fail_receive(
                        &entry,
                        UploadErrorKind::UnsupportedMedia,
                        format!("content type {ct:?} not in raw allowlist"),
                    )
                    .await;
                }
            }
        }
    }
    if let Some(cl) = header_value(&req_headers, "content-length") {
        if let Ok(declared) = cl.trim().parse::<u64>() {
            if declared > options.max_wire_bytes {
                return fail_receive(
                    &entry,
                    UploadErrorKind::BodyTooLarge,
                    format!(
                        "declared Content-Length {declared} > wire cap {}",
                        options.max_wire_bytes
                    ),
                )
                .await;
            }
        }
    }

    // 期限切换通知（VM 腿 scope 延伸；生成腿 no-op）。
    phase(UploadPhase::ReceiveStarted { total_deadline });

    // staging 文件（create_new 独占）。
    let staging_path = entry
        .staging_root
        .join(format!(".upload730-{session_id}.part"));
    let writer = match spawn_writer(staging_path.clone(), session_id).await {
        Ok(w) => w,
        Err(e) => {
            return fail_receive(
                &entry,
                UploadErrorKind::Io,
                format!(
                    "staging create failed in {}: {e}",
                    entry.staging_root.display()
                ),
            )
            .await;
        }
    };

    // 接收泵：网络块（idle）/ 取消 / total。
    use futures::StreamExt;
    let (_m, _p, _h, mut body) = req.into_parts();
    let mut wire_total: u64 = 0;
    let idle = tokio::time::sleep(idle_timeout);
    tokio::pin!(idle);
    let outcome: Result<(), (UploadErrorKind, String)> = loop {
        tokio::select! {
            biased;
            _ = wait_cancel(&cancel_rx) => {
                break Err((UploadErrorKind::Cancelled, "upload cancelled".into()));
            }
            _ = tokio::time::sleep_until(tokio::time::Instant::from_std(total_deadline)) => {
                break Err((UploadErrorKind::TotalTimeout, "upload total deadline exceeded".into()));
            }
            _ = &mut idle => {
                break Err((UploadErrorKind::IdleTimeout, "upload idle timeout".into()));
            }
            chunk = body.next() => {
                match chunk {
                    None => break Ok(()), // EOF
                    Some(Err(e)) => {
                        break Err((UploadErrorKind::BadMultipart, format!("body read failed: {e}")));
                    }
                    Some(Ok(bytes)) => {
                        wire_total += bytes.len() as u64;
                        if wire_total > options.max_wire_bytes {
                            break Err((UploadErrorKind::BodyTooLarge, format!(
                                "wire total {wire_total} > {}", options.max_wire_bytes
                            )));
                        }
                        match &mut parser {
                            None => {
                                // raw：整 body 为文件。
                                if let Err(e) = writer.tx.send(WriteCmd::Block(bytes)).await {
                                    break Err((UploadErrorKind::Io, format!("staging write failed: {e}")));
                                }
                            }
                            Some(mp) => {
                                let mut file_chunks = Vec::new();
                                if let Err(kind) = mp.feed(&bytes, &mut file_chunks, &options) {
                                    break Err((kind, "multipart parse failed".into()));
                                }
                                let mut send_err: Option<String> = None;
                                for block in file_chunks {
                                    if let Err(e) = writer.tx.send(WriteCmd::Block(block)).await {
                                        send_err = Some(format!("staging write failed: {e}"));
                                        break;
                                    }
                                }
                                if let Some(e) = send_err {
                                    break Err((UploadErrorKind::Io, e));
                                }
                            }
                        }
                        idle.as_mut().reset(tokio::time::Instant::now() + idle_timeout);
                    }
                }
            }
        }
    };

    let result: Result<UploadReceivedMeta, (UploadErrorKind, String)> = match outcome {
        Err(e) => Err(e),
        Ok(()) => {
            let finish_meta = match &mut parser {
                None => Ok(UploadReceivedMeta {
                    mode: UploadReceiveMode::Raw,
                    field: String::new(),
                    filename: None,
                    size: wire_total,
                    content_type: if content_type.is_empty() {
                        None
                    } else {
                        Some(content_type.clone())
                    },
                    fields: Vec::new(),
                }),
                Some(mp) => mp
                    .finish()
                    .map_err(|kind| {
                        (kind, "truncated multipart body (missing closing boundary)".to_string())
                    })
                    .map(|_| mp.snapshot(&options.file_field)),
            };
            match finish_meta {
                Err(e) => Err(e),
                Ok(meta) => {
                    // flush + sync + close（staged 的唯一完成路径）。
                    if let Err(e) = writer.tx.send(WriteCmd::Finish).await {
                        return fail_receive(
                            &entry,
                            UploadErrorKind::Io,
                            format!("staging finish send failed: {e}"),
                        )
                        .await;
                    }
                    match writer.finish_rx.await {
                        Ok(Ok(())) => Ok(meta),
                        Ok(Err(e)) => {
                            Err((UploadErrorKind::Io, format!("staging finish failed: {e}")))
                        }
                        Err(_) => {
                            Err((UploadErrorKind::Io, "staging writer exited unexpectedly".into()))
                        }
                    }
                }
            }
        }
    };

    match result {
        Ok(meta) => {
            *entry.phase.lock().unwrap() = SessionPhase::Staged {
                meta: meta.clone(),
                staging_path: staging_path.clone(),
            };
            let lease_deadline = Instant::now() + Duration::from_millis(hard.lease_timeout_ms);
            phase(UploadPhase::Staged { lease_deadline });
            let guard = tokio::spawn(lease_watchdog(session_id, lease_deadline));
            *entry.lease_guard.lock().unwrap() = Some(guard);
            a2r_std::http::server_upload::received_session(session_id, meta)
        }
        Err((kind, message)) => fail_receive(&entry, kind, message).await,
    }
}

/// watch 取消等待（sender 掉落视为取消——不产生忙等）。
async fn wait_cancel(rx: &tokio::sync::watch::Receiver<bool>) {
    let mut rx = rx.clone();
    let _ = rx.changed().await;
}

/// 失败收口：writer 通道关闭（在途写返回后自清）+ 许可释放 + 失败会话。
async fn fail_receive(
    entry: &Arc<SessionEntry>,
    kind: UploadErrorKind,
    message: impl Into<String>,
) -> UploadSession {
    *entry.phase.lock().unwrap() = SessionPhase::Terminal("failed");
    release_permits(entry);
    failed_session(kind, message)
}

/// lease 看门狗：Staged 期无人 commit/reject → 过期清理（许可释放）。
async fn lease_watchdog(session_id: u64, deadline: Instant) {
    tokio::time::sleep_until(tokio::time::Instant::from_std(deadline)).await;
    let Some(entry) = with_entry(session_id, |e| e.clone()) else { return };
    let expired_staging: Option<PathBuf> = {
        let mut phase = entry.phase.lock().unwrap();
        if matches!(&*phase, SessionPhase::Staged { .. }) {
            match std::mem::replace(&mut *phase, SessionPhase::Terminal("expired_lease")) {
                SessionPhase::Staged { staging_path, .. } => Some(staging_path),
                _ => None,
            }
        } else {
            None
        }
    };
    if let Some(staging_path) = expired_staging {
        eprintln!("[upload730] session {session_id} staged lease expired — cleaning staging");
        record_expired_tombstone(session_id);
        // 先摘除自身句柄再 release——release_permits 会 abort lease_guard，
        // 看门狗任务不能自 abort（清 staging 的 await 在其后）。
        *entry.lease_guard.lock().unwrap() = None;
        release_permits(&entry);
        cleanup_staging_path(staging_path, session_id).await;
    }
}

// ── commit / reject ───────────────────────────────────────────────────────

async fn run_commit(session_id: u64, relative_target: String) -> UploadReceipt {
    let Some(entry) = with_entry(session_id, |e| e.clone()) else {
        if is_expired_tombstone(session_id) {
            return UploadReceipt::failed(
                UploadErrorKind::ExpiredLease,
                "upload_commit: staged lease expired before commit",
            );
        }
        return UploadReceipt::failed(
            UploadErrorKind::SessionConflict,
            format!("upload_commit: unknown session {session_id}"),
        );
    };
    // 状态仲裁：仅 Staged 可提交（gate 进入 = phase 迁移 Committing；载荷
    // 随相携带——不消费丢失）。并发双 commit 由 hard_link 的 create-only
    // 原语收敛（第二个 AlreadyExists → 409）。
    let (meta, staging_path) = {
        let phase = entry.phase.lock().unwrap();
        match &*phase {
            SessionPhase::Staged { meta, staging_path } => (meta.clone(), staging_path.clone()),
            other => {
                let kind = match other {
                    SessionPhase::Terminal("expired_lease") => UploadErrorKind::ExpiredLease,
                    _ => UploadErrorKind::SessionConflict,
                };
                let name = entry.phase_name();
                return UploadReceipt::failed(
                    kind,
                    format!("upload_commit: session is {name} (not staged)"),
                );
            }
        }
    };
    *entry.phase.lock().unwrap() = SessionPhase::Committing {
        staging_path: staging_path.clone(),
    };
    let fail_commit = |terminal: &'static str, kind: UploadErrorKind, message: String| {
        let entry = entry.clone();
        let staging_path = staging_path.clone();
        async move {
            *entry.phase.lock().unwrap() = SessionPhase::Terminal(terminal);
            release_permits(&entry);
            tokio::spawn(cleanup_staging_path(staging_path, entry.id));
            UploadReceipt::failed(kind, message)
        }
    };
    // 词法校验（facade 已验；防御重复）。
    if a2r_std::http::validate_relative_path(&relative_target).is_err() {
        return fail_commit(
            "bad_target",
            UploadErrorKind::ForbiddenPath,
            format!("target {relative_target:?} rejected"),
        )
        .await;
    }
    // 安全目标：父目录 no-follow walk（已存在的安全父目录；不自动创建）。
    let target_path = match safe_target_in_root(&entry.root, &relative_target) {
        Ok(p) => p,
        Err(kind) => {
            return fail_commit(
                "bad_target",
                kind,
                format!("target {relative_target:?} rejected"),
            )
            .await;
        }
    };

    // gate：hard_link（原子 create-only）。迟到取消不回滚——等实际结果。
    let _fs_permit = SERVICE.fs_ops.clone().acquire_owned().await;
    let link_result = {
        let staging = staging_path.clone();
        let target = target_path.clone();
        match tokio::task::spawn_blocking(move || std::fs::hard_link(&staging, &target)).await {
            Ok(r) => r,
            Err(e) => Err(io::Error::other(format!("publish task: {e}"))),
        }
    };
    match link_result {
        Ok(()) => {
            // 发布成功：内容改由 target 名持有——删 staging 链接（失败=可观测残留）。
            if let Err(e) = tokio::fs::remove_file(&staging_path).await {
                RESIDUAL_CLEANUP_FAILS.fetch_add(1, Ordering::SeqCst);
                eprintln!(
                    "[upload730] session {session_id} published but staging unlink failed: {e} — residual {} (private dir)",
                    staging_path.display()
                );
            }
            *entry.phase.lock().unwrap() = SessionPhase::Terminal("committed");
            release_permits(&entry);
            UploadReceipt::committed(&relative_target, &meta)
        }
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
            // 原文件字节不变（hard_link 拒绝覆盖——探针 P1）。
            *entry.phase.lock().unwrap() = SessionPhase::Terminal("conflict");
            release_permits(&entry);
            tokio::spawn(cleanup_staging_path(staging_path.clone(), session_id));
            UploadReceipt::failed(
                UploadErrorKind::TargetConflict,
                format!("target {relative_target:?} already exists (original untouched)"),
            )
        }
        Err(e) => {
            let kind = if matches!(e.raw_os_error(), Some(17) | Some(18)) {
                UploadErrorKind::InvalidOptions // EXDEV/EXFULL：跨卷配置
            } else {
                UploadErrorKind::Io
            };
            *entry.phase.lock().unwrap() = SessionPhase::Terminal("publish_failed");
            release_permits(&entry);
            tokio::spawn(cleanup_staging_path(staging_path.clone(), session_id));
            UploadReceipt::failed(kind, format!("publish failed: {e}"))
        }
    }
}

async fn run_reject(session_id: u64, status: i64, message: String) -> UploadReceipt {
    let Some(entry) = with_entry(session_id, |e| e.clone()) else {
        return UploadReceipt::failed(
            UploadErrorKind::SessionConflict,
            format!("upload_reject: unknown session {session_id}"),
        );
    };
    let staging_path: Option<PathBuf> = {
        let mut phase = entry.phase.lock().unwrap();
        match &*phase {
            SessionPhase::Staged { staging_path, .. } => {
                let p = staging_path.clone();
                *phase = SessionPhase::Terminal("rejected");
                Some(p)
            }
            SessionPhase::Receiving => {
                *phase = SessionPhase::Terminal("rejected");
                None // 未 staged（理论不可达——会话尚未交付）
            }
            SessionPhase::Committing { .. } => {
                return UploadReceipt::failed(
                    UploadErrorKind::SessionConflict,
                    "upload_reject: session is committing (publish in flight)",
                );
            }
            SessionPhase::Terminal(_) => {
                return UploadReceipt::failed(
                    UploadErrorKind::SessionConflict,
                    "upload_reject: session already terminal",
                );
            }
        }
    };
    release_permits(&entry);
    if let Some(path) = staging_path {
        cleanup_staging_path(path, session_id).await;
    }
    UploadReceipt {
        status: status as u16,
        kind: UploadErrorKind::Rejected,
        json: format!(
            "{{\"ok\":false,\"kind\":\"rejected\",\"message\":\"{}\"}}",
            json_escape(&message),
        ),
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

// ── 清理/许可 ─────────────────────────────────────────────────────────────

fn release_permits(entry: &Arc<SessionEntry>) {
    entry.active_permit.lock().unwrap().take();
    entry.queue_permit.lock().unwrap().take();
    if let Some(guard) = entry.lease_guard.lock().unwrap().take() {
        guard.abort();
    }
    if let Ok(mut r) = SERVICE.registry.lock() {
        r.remove(&entry.id);
    }
    // writer（若在途）：通道发送端随本 entry 的最后一个引用 Drop 而关闭，
    // writer 收尾（在途写返回）后自清 staging——不提前腾 FS 槽。
}

async fn cleanup_staging_path(path: PathBuf, session_id: u64) {
    if tokio::fs::metadata(&path).await.is_err() {
        return; // 不存在（writer 已自清/未创建/已发布改名）
    }
    let _permit = SERVICE.fs_ops.clone().try_acquire_owned();
    if let Err(e) = tokio::fs::remove_file(&path).await {
        RESIDUAL_CLEANUP_FAILS.fetch_add(1, Ordering::SeqCst);
        eprintln!(
            "[upload730] staging cleanup failed (session {session_id}): {e} — residual {} (private dir)",
            path.display()
        );
    }
}

/// 安全目标解析：词法校验（facade 层已过）+ 父目录逐段 no-follow walk
///（已存在的安全父目录；不自动创建目录）。终段 create-only 语义由
/// `hard_link` 承担（目标存在即 409——包括终段为 symlink 的情形）。
fn safe_target_in_root(root: &Path, relative: &str) -> Result<PathBuf, UploadErrorKind> {
    let mut prefix = root.to_path_buf();
    let segments: Vec<&str> = relative
        .split(['/', '\\'])
        .filter(|s| !s.is_empty() && *s != ".")
        .collect();
    let Some((last, dirs)) = segments.split_last() else {
        return Err(UploadErrorKind::ForbiddenPath);
    };
    for seg in dirs {
        prefix.push(*seg);
        match std::fs::symlink_metadata(&prefix) {
            Ok(md) => {
                if md.file_type().is_symlink() || !md.is_dir() {
                    return Err(UploadErrorKind::ForbiddenPath);
                }
            }
            Err(_) => return Err(UploadErrorKind::ForbiddenPath), // 父目录须已存在
        }
    }
    prefix.push(*last);
    Ok(prefix)
}

// ============================================================================
// T-03/T-04 表驱动测试（parser 矩阵 + 接收/提交/取消/租约）
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use a2r_std::http::server_upload::upload_request_from_parts;
    use a2r_std::http::UploadBodyStream;
    use a2r_std::http::UploadErrorKind as K;

    fn opts(json: &str) -> UploadReceiveOptions {
        a2r_std::http::parse_upload_receive_options(json, &UploadServeLimits::default()).unwrap()
    }

    fn mp_new(boundary: &str, options: &UploadReceiveOptions) -> MultipartIncremental {
        MultipartIncremental::new(boundary, options, &UploadServeLimits::default()).unwrap()
    }

    /// 标准 multipart 体：文本 a=1、file、文本 b=2（有序）。
    fn standard_body() -> Vec<u8> {
        let mut b = Vec::new();
        b.extend_from_slice(b"--BOUND\r\n");
        b.extend_from_slice(b"Content-Disposition: form-data; name=\"a\"\r\n\r\n");
        b.extend_from_slice(b"1\r\n");
        b.extend_from_slice(b"--BOUND\r\n");
        b.extend_from_slice(
            b"Content-Disposition: form-data; name=\"file\"; filename=\"x.bin\"\r\n",
        );
        b.extend_from_slice(b"Content-Type: application/octet-stream\r\n\r\n");
        b.extend_from_slice(b"FILEDATA");
        b.extend_from_slice(b"\r\n--BOUND\r\n");
        b.extend_from_slice(b"Content-Disposition: form-data; name=\"b\"\r\n\r\n");
        b.extend_from_slice(b"2\r\n");
        b.extend_from_slice(b"--BOUND--\r\n");
        b
    }

    fn drive_parser(
        boundary: &str,
        body: &[u8],
        chunk_size: usize,
        options: &UploadReceiveOptions,
    ) -> Result<(Vec<Vec<u8>>, MultipartIncremental), UploadErrorKind> {
        let mut mp = mp_new(boundary, options);
        let mut all_file = Vec::new();
        for c in body.chunks(chunk_size.max(1)) {
            let mut out = Vec::new();
            mp.feed(c, &mut out, options)?;
            all_file.extend(out);
        }
        mp.finish()?;
        Ok((all_file, mp))
    }

    /// 逐字节/多粒度切块（boundary/header 跨任意 TCP chunk）内容一致。
    #[test]
    fn plan730_parser_every_chunk_split_yields_same_bytes() {
        let body = standard_body();
        let options = opts(r#"{"mode":"multipart","text_fields":["a","b"]}"#);
        for cs in [1usize, 2, 3, 5, 7, 11, 17, 64, 1024] {
            let (file_chunks, mp) = drive_parser("BOUND", &body, cs, &options)
                .unwrap_or_else(|e| panic!("chunk size {cs}: {e:?}"));
            let joined: Vec<u8> = file_chunks.concat();
            assert_eq!(joined, b"FILEDATA", "chunk size {cs}");
            let meta = mp.snapshot("file");
            assert_eq!(meta.size, 8);
            assert_eq!(meta.filename.as_deref(), Some("x.bin"));
            assert_eq!(
                meta.fields,
                vec![
                    ("a".to_string(), "1".to_string()),
                    ("b".to_string(), "2".to_string())
                ],
                "chunk size {cs}"
            );
        }
    }

    /// 内容含假前缀/相似 delimiter：不截断文件。
    #[test]
    fn plan730_parser_false_prefix_boundaries_are_content() {
        let mut b = Vec::new();
        b.extend_from_slice(b"--BOUND\r\n");
        b.extend_from_slice(
            b"Content-Disposition: form-data; name=\"file\"; filename=\"t\"\r\n\r\n",
        );
        b.extend_from_slice(b"A\r\n--BOUNDXB\r\n--BOUN\r\n--BOUNDZ");
        b.extend_from_slice(b"\r\n--BOUND--\r\n");
        let options = opts(r#"{"mode":"multipart"}"#);
        let (file_chunks, mp) = drive_parser("BOUND", &b, 3, &options).unwrap();
        let joined: Vec<u8> = file_chunks.concat();
        assert_eq!(joined, b"A\r\n--BOUNDXB\r\n--BOUN\r\n--BOUNDZ");
        assert_eq!(mp.file_size, joined.len() as u64);
    }

    /// 空 filename/Unicode filename/缺 filename 均合法（filename 仅 metadata）。
    #[test]
    fn plan730_parser_filename_variants() {
        let mk = |fname: &str| {
            format!(
                "--B\r\nContent-Disposition: form-data; name=\"file\"; filename={f}\r\n\r\nX\r\n--B--\r\n",
                f = fname
            )
            .into_bytes()
        };
        let options = opts(r#"{"mode":"multipart"}"#);
        let body = mk("\"\"");
        let (_, mp) = drive_parser("B", &body, 4, &options).unwrap();
        assert_eq!(mp.file_filename.as_deref(), Some(""));
        let body = mk("\"报告 v1.bin\"");
        let (_, mp) = drive_parser("B", &body, 4, &options).unwrap();
        assert_eq!(mp.file_filename.as_deref(), Some("报告 v1.bin"));
        let body = b"--B\r\nContent-Disposition: form-data; name=\"file\"\r\n\r\nX\r\n--B--\r\n";
        let (_, mp) = drive_parser("B", body, 4, &options).unwrap();
        assert_eq!(mp.file_filename, None);
    }

    /// 错误矩阵：缺 file/第二 file/未知字段/截断/坏 UTF-8/CTE/嵌套/charset。
    #[test]
    fn plan730_parser_error_matrix() {
        let options = opts(r#"{"mode":"multipart","text_fields":["a"]}"#);
        let body = b"--B\r\nContent-Disposition: form-data; name=\"a\"\r\n\r\n1\r\n--B--\r\n";
        assert_eq!(
            drive_parser("B", body, 5, &options).map(|_| ()).unwrap_err(),
            K::BadMultipart
        );
        let body = b"--B\r\nContent-Disposition: form-data; name=\"file\"\r\n\r\n1\r\n--B\r\nContent-Disposition: form-data; name=\"file\"\r\n\r\n2\r\n--B--\r\n";
        assert_eq!(
            drive_parser("B", body, 5, &options).map(|_| ()).unwrap_err(),
            K::BadMultipart
        );
        let body = b"--B\r\nContent-Disposition: form-data; name=\"nope\"\r\n\r\n1\r\n--B--\r\n";
        assert_eq!(
            drive_parser("B", body, 5, &options).map(|_| ()).unwrap_err(),
            K::BadMultipart
        );
        let body = b"--B\r\nContent-Disposition: form-data; name=\"file\"\r\n\r\nDATA";
        assert_eq!(
            drive_parser("B", body, 5, &options).map(|_| ()).unwrap_err(),
            K::BadMultipart
        );
        let body = b"--B\r\nContent-Disposition: form-data; name=\"a\"\r\n\r\n\xFF\xFE\r\n--B--\r\n";
        assert_eq!(
            drive_parser("B", body, 5, &options).map(|_| ()).unwrap_err(),
            K::BadMultipart
        );
        let body = b"--B\r\nContent-Disposition: form-data; name=\"file\"\r\nContent-Transfer-Encoding: base64\r\n\r\nWFla\r\n--B--\r\n";
        assert_eq!(
            drive_parser("B", body, 5, &options).map(|_| ()).unwrap_err(),
            K::BadMultipart
        );
        let body = b"--B\r\nContent-Disposition: form-data; name=\"file\"\r\nContent-Type: multipart/mixed; boundary=X\r\n\r\n.\r\n--B--\r\n";
        assert_eq!(
            drive_parser("B", body, 5, &options).map(|_| ()).unwrap_err(),
            K::BadMultipart
        );
        let body = b"--B\r\nContent-Disposition: form-data; name=\"a\"\r\nContent-Type: text/plain; charset=iso-8859-1\r\n\r\nx\r\n--B--\r\n";
        assert_eq!(
            drive_parser("B", body, 5, &options).map(|_| ()).unwrap_err(),
            K::UnsupportedMedia
        );
    }

    /// 预算矩阵：单文件上限 ±1、文本单项预算、重复字段保序。
    #[test]
    fn plan730_parser_budget_boundaries() {
        let file_n = |n: usize| {
            let data = vec![b'X'; n];
            let mut b = Vec::new();
            b.extend_from_slice(b"--B\r\nContent-Disposition: form-data; name=\"file\"\r\n\r\n");
            b.extend_from_slice(&data);
            b.extend_from_slice(b"\r\n--B--\r\n");
            b
        };
        let options = opts(r#"{"mode":"multipart","max_file_bytes":10,"text_fields":["k"]}"#);
        assert!(drive_parser("B", &file_n(10), 3, &options).is_ok());
        assert_eq!(
            drive_parser("B", &file_n(11), 3, &options).map(|_| ()).unwrap_err(),
            K::BodyTooLarge
        );
        // 文本预算样例须带 file part（无 file part 本身即 BadMultipart）。
        let tf = |v: &str| {
            format!(
                "--B\r\nContent-Disposition: form-data; name=\"file\"\r\n\r\nF\r\n--B\r\nContent-Disposition: form-data; name=\"k\"\r\n\r\n{v}\r\n--B--\r\n"
            )
            .into_bytes()
        };
        let options = opts(r#"{"mode":"multipart","text_fields":["k"],"max_text_field_bytes":4}"#);
        assert!(drive_parser("B", &tf("1234"), 3, &options).is_ok());
        assert_eq!(
            drive_parser("B", &tf("12345"), 3, &options).map(|_| ()).unwrap_err(),
            K::BadMultipart
        );
        // 重复字段保序（不静默覆盖）。
        let mut body = Vec::new();
        body.extend_from_slice(b"--B\r\nContent-Disposition: form-data; name=\"file\"\r\n\r\nF\r\n");
        for v in ["1", "2"] {
            body.extend_from_slice(b"--B\r\n");
            body.extend_from_slice(b"Content-Disposition: form-data; name=\"k\"\r\n\r\n");
            body.extend_from_slice(v.as_bytes());
            body.extend_from_slice(b"\r\n");
        }
        body.extend_from_slice(b"--B--\r\n");
        let options = opts(r#"{"mode":"multipart","text_fields":["k"]}"#);
        let (_, mp) = drive_parser("B", &body, 7, &options).unwrap();
        assert_eq!(
            mp.fields,
            vec![
                ("k".to_string(), "1".to_string()),
                ("k".to_string(), "2".to_string())
            ],
            "重复字段保序"
        );
    }

    /// preamble/epilogue 字节不参与内容（计入 wire 由调用方负责）。
    #[test]
    fn plan730_parser_preamble_epilogue_ignored() {
        let mut b = Vec::new();
        b.extend_from_slice(b"this is preamble\r\njunk--Bnot\r\n");
        b.extend_from_slice(b"--B\r\n");
        b.extend_from_slice(b"Content-Disposition: form-data; name=\"file\"\r\n\r\n");
        b.extend_from_slice(b"D\r\n--B--\r\n");
        b.extend_from_slice(b"epilogue bytes here");
        let options = opts(r#"{"mode":"multipart"}"#);
        let (file_chunks, mp) = drive_parser("B", &b, 6, &options).unwrap();
        assert_eq!(file_chunks.concat(), b"D");
        assert_eq!(mp.file_size, 1);
    }

    // ── receive/commit/reject/lease 集成（内存流 + 独占临时目录） ──────

    fn temp_roots(tag: &str) -> (std::path::PathBuf, std::path::PathBuf) {
        let base = std::env::temp_dir().join(format!(
            "plan730-svc-{tag}-{}",
            std::process::id()
        ));
        let root = base.join("public");
        let staging = base.join("private-staging");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(&staging).unwrap();
        (root, staging)
    }

    fn stream_of(chunks: Vec<Vec<u8>>) -> UploadBodyStream {
        use futures::stream;
        let items: Vec<std::io::Result<Vec<u8>>> = chunks.into_iter().map(Ok).collect();
        Box::pin(stream::iter(items))
    }

    fn req_from(body_chunks: Vec<Vec<u8>>, content_type: &str) -> UploadRequest {
        let headers = if content_type.is_empty() {
            Vec::new()
        } else {
            vec![("content-type".to_string(), content_type.to_string())]
        };
        upload_request_from_parts("POST", "/up", headers, stream_of(body_chunks))
    }

    fn rt() -> tokio::runtime::Runtime {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
    }

    fn install() {
        install_upload_executor_service();
    }

    fn drive_until(runtime: &tokio::runtime::Runtime, cond: impl Fn() -> bool, secs: u64) -> bool {
        let deadline = Instant::now() + Duration::from_secs(secs);
        while Instant::now() < deadline {
            if cond() {
                return true;
            }
            runtime.block_on(async {
                tokio::time::sleep(Duration::from_millis(25)).await;
            });
        }
        cond()
    }

    /// raw 接收 → staged 字节一致 → create-only 提交 201 + hash 相同 +
    /// staging 清零 + 同名冲突 409（原文件字节不变）。
    #[test]
    fn plan730_raw_receive_commit_conflict_roundtrip() {
        install();
        let (root, staging) = temp_roots("raw");
        let payload: Vec<u8> = (0..100_000u32).map(|i| (i % 251) as u8).collect();
        let runtime = rt();
        let session = runtime.block_on(a2r_std::http::upload_receive(
            req_from(
                payload.chunks(7000).map(|c| c.to_vec()).collect(),
                "application/octet-stream",
            ),
            root.to_str().unwrap(),
            staging.to_str().unwrap(),
            r#"{"mode":"raw"}"#,
        ));
        assert!(session.is_received(), "raw receive stages");
        let meta_json = a2r_std::http::upload_metadata_json(&session);
        assert!(meta_json.contains("\"size\":\"100000\""), "{meta_json}");
        assert_eq!(upload_staged_count(), 1);
        // 合同：只允许已存在的安全父目录——测试预建子目录。
        std::fs::create_dir_all(root.join("ok")).unwrap();
        let receipt =
            runtime.block_on(a2r_std::http::upload_commit(session.clone(), "ok/x.bin"));
        assert_eq!(receipt.status, 201, "{}", receipt.json);
        let on_disk = std::fs::read(root.join("ok/x.bin")).unwrap();
        assert_eq!(on_disk, payload, "published bytes identical");
        assert_eq!(upload_staged_count(), 0);
        assert_eq!(upload_session_count(), 0, "terminal session leaves registry");
        // 同名冲突：原文件不变。
        let another = runtime.block_on(a2r_std::http::upload_receive(
            req_from(vec![b"second".to_vec()], "application/octet-stream"),
            root.to_str().unwrap(),
            staging.to_str().unwrap(),
            r#"{"mode":"raw"}"#,
        ));
        let receipt = runtime.block_on(a2r_std::http::upload_commit(another, "ok/x.bin"));
        assert_eq!(receipt.status, 409, "{}", receipt.json);
        let still = std::fs::read(root.join("ok/x.bin")).unwrap();
        assert_eq!(still, payload, "conflict leaves original untouched");
    }

    /// multipart 接收（字段有序 + 12MiB>普通 10MiB 上限）+ reject 清零。
    #[test]
    fn plan730_multipart_receive_and_reject_cleanup() {
        install();
        let (root, staging) = temp_roots("mp");
        let big: Vec<u8> = vec![0xA7; 12 * 1024 * 1024];
        let mut body = Vec::new();
        body.extend_from_slice(b"--B730\r\n");
        body.extend_from_slice(
            b"Content-Disposition: form-data; name=\"note\"\r\n\r\nhello\r\n",
        );
        body.extend_from_slice(b"--B730\r\n");
        body.extend_from_slice(
            b"Content-Disposition: form-data; name=\"file\"; filename=\"big.bin\"\r\n\r\n",
        );
        body.extend_from_slice(&big);
        body.extend_from_slice(b"\r\n--B730--\r\n");
        let chunks: Vec<Vec<u8>> = body.chunks(64 * 1024).map(|c| c.to_vec()).collect();
        let ct = "multipart/form-data; boundary=B730".to_string();
        let runtime = rt();
        let session = runtime.block_on(a2r_std::http::upload_receive(
            req_from(chunks, &ct),
            root.to_str().unwrap(),
            staging.to_str().unwrap(),
            r#"{"mode":"multipart","text_fields":["note"]}"#,
        ));
        assert!(session.is_received(), "12MiB multipart stages");
        let meta = a2r_std::http::upload_metadata_json(&session);
        assert!(meta.contains("\"size\":\"12582912\""), "{meta}");
        assert!(meta.contains("\"filename\":\"big.bin\""));
        assert!(meta.contains("\"name\":\"note\",\"value\":\"hello\""));
        assert_eq!(upload_staged_count(), 1);
        let receipt = runtime
            .block_on(a2r_std::http::upload_reject(session, 422, "business check failed"));
        assert_eq!(receipt.status, 422);
        assert_eq!(upload_staged_count(), 0);
        assert_eq!(upload_session_count(), 0);
        assert!(!root.join("anything").exists(), "nothing published");
        let staging_files: Vec<_> = std::fs::read_dir(&staging).unwrap().collect();
        assert!(staging_files.is_empty(), "staging cleaned");
    }

    /// wire 超限 413 失败会话（staging 清零；failed 会话 commit 409 /
    /// reject(status=0) 采用建议状态）。
    #[test]
    fn plan730_wire_over_limit_fails_with_cleanup() {
        install();
        let (root, staging) = temp_roots("cap");
        let runtime = rt();
        let over: Vec<u8> = vec![b'Z'; 1024];
        let session = runtime.block_on(a2r_std::http::upload_receive(
            req_from(vec![over], "application/octet-stream"),
            root.to_str().unwrap(),
            staging.to_str().unwrap(),
            r#"{"mode":"raw","max_wire_bytes":512}"#,
        ));
        assert!(!session.is_received());
        let meta = a2r_std::http::upload_metadata_json(&session);
        assert!(meta.contains("body_too_large"), "{meta}");
        assert!(meta.contains("\"suggested_status\":413"));
        let receipt = runtime.block_on(a2r_std::http::upload_commit(session.clone(), "x"));
        assert_eq!(receipt.status, 409);
        let receipt = runtime.block_on(a2r_std::http::upload_reject(session, 0, ""));
        assert_eq!(receipt.status, 413, "status=0 adopts suggested");
        // writer 自清是异步的（通道关闭→在途写返回→remove）：驱动到落定。
        assert!(
            drive_until(
                &runtime,
                || std::fs::read_dir(&staging).unwrap().next().is_none(),
                5
            ),
            "no staging left after failure"
        );
        assert_eq!(upload_session_count(), 0);
    }

    /// 目标矩阵：traversal/绝对/drive/UNC/父目录缺失 → 403（零发布）。
    #[test]
    fn plan730_commit_target_matrix() {
        install();
        let (root, staging) = temp_roots("tgt");
        let runtime = rt();
        for target in [
            "../escape.bin",
            "a/../../b.bin",
            "C:/abs.bin",
            "\\\\srv\\s\\b.bin",
            "missing_dir/x.bin",
            "..",
        ] {
            let session = runtime.block_on(a2r_std::http::upload_receive(
                req_from(vec![b"T".to_vec()], "application/octet-stream"),
                root.to_str().unwrap(),
                staging.to_str().unwrap(),
                r#"{"mode":"raw"}"#,
            ));
            let receipt = runtime.block_on(a2r_std::http::upload_commit(session, target));
            assert_eq!(receipt.status, 403, "target {target:?} -> {}", receipt.json);
        }
        assert!(
            std::fs::read_dir(&root).unwrap().next().is_none(),
            "nothing published"
        );
    }

    /// lease 到期：无人 commit/reject → 自动清理 + 后续 commit 410。
    #[test]
    fn plan730_staged_lease_expiry_cleans() {
        std::env::set_var("AUTO_HTTP_UPLOAD_LEASE_MS", "300");
        install();
        let (root, staging) = temp_roots("lease");
        let runtime = rt();
        let session = runtime.block_on(a2r_std::http::upload_receive(
            req_from(vec![b"L".to_vec()], "application/octet-stream"),
            root.to_str().unwrap(),
            staging.to_str().unwrap(),
            r#"{"mode":"raw"}"#,
        ));
        assert!(session.is_received());
        assert!(drive_until(&runtime, || upload_session_count() == 0, 5));
        assert!(drive_until(
            &runtime,
            || std::fs::read_dir(&staging).unwrap().next().is_none(),
            5
        ), "lease cleanup settles");
        let receipt = runtime.block_on(a2r_std::http::upload_commit(session, "late.bin"));
        assert_eq!(receipt.status, 410, "{}", receipt.json);
        let _ = root;
    }

    /// staged 期取消（scope 收口语义）：立即清理、不发布。
    #[test]
    fn plan730_staged_cancel_cleans_without_publish() {
        install();
        let (root, staging) = temp_roots("cancel");
        let runtime = rt();
        let session = runtime.block_on(a2r_std::http::upload_receive(
            req_from(vec![b"C".to_vec()], "application/octet-stream"),
            root.to_str().unwrap(),
            staging.to_str().unwrap(),
            r#"{"mode":"raw"}"#,
        ));
        assert!(session.is_received());
        let session_id = session.id();
        cancel_session(session_id);
        assert!(drive_until(&runtime, || upload_session_count() == 0, 5));
        assert!(drive_until(
            &runtime,
            || std::fs::read_dir(&staging).unwrap().next().is_none(),
            5
        ));
        let receipt = runtime.block_on(a2r_std::http::upload_commit(session, "gone.bin"));
        assert_eq!(receipt.status, 409, "cancelled session cannot commit");
        let _ = root;
    }

    /// 接收配置矩阵：嵌套/缺失 staging 在读取 body 前拒绝。
    #[test]
    fn plan730_root_config_rejected_before_body_read() {
        install();
        let (root, staging) = temp_roots("cfg");
        let runtime = rt();
        let nested = root.join("nested-staging");
        std::fs::create_dir_all(&nested).unwrap();
        let session = runtime.block_on(a2r_std::http::upload_receive(
            req_from(Vec::new(), ""),
            root.to_str().unwrap(),
            nested.to_str().unwrap(),
            r#"{"mode":"raw"}"#,
        ));
        assert!(!session.is_received());
        assert!(
            a2r_std::http::upload_metadata_json(&session).contains("outside"),
            "nested staging rejected"
        );
        let session = runtime.block_on(a2r_std::http::upload_receive(
            req_from(Vec::new(), ""),
            root.to_str().unwrap(),
            root.parent()
                .unwrap()
                .join("no-such-staging")
                .to_str()
                .unwrap(),
            r#"{"mode":"raw"}"#,
        ));
        assert!(!session.is_received());
        assert!(
            a2r_std::http::upload_metadata_json(&session)
                .contains("missing"),
            "missing staging rejected"
        );
        let _ = staging;
    }

    /// 非 identity Content-Encoding → 415。
    #[test]
    fn plan730_content_encoding_rejected() {
        install();
        let (root, staging) = temp_roots("ce");
        let runtime = rt();
        let req = upload_request_from_parts(
            "POST",
            "/up",
            vec![
                ("content-type".to_string(), "application/octet-stream".to_string()),
                ("content-encoding".to_string(), "gzip".to_string()),
            ],
            stream_of(vec![vec![1, 2, 3]]),
        );
        let session = runtime.block_on(a2r_std::http::upload_receive(
            req,
            root.to_str().unwrap(),
            staging.to_str().unwrap(),
            r#"{"mode":"raw"}"#,
        ));
        let meta = a2r_std::http::upload_metadata_json(&session);
        assert!(meta.contains("unsupported_media"), "{meta}");
        assert!(meta.contains("\"suggested_status\":415"));
    }
}
