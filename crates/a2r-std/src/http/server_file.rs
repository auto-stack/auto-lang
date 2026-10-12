//! PLAN-729 T-02：服务端文件响应——可移植构造面与纯协议决策（单源）。
//!
//! 契约速览（canonical Spec 由 SD-01 沉淀；决策报告
//! `docs/plans/reports/729-server-files-decision.md` §4）：
//!
//! ```text
//! file_response(root, relative_path, options_json) → FileResponse 描述符
//!   零 I/O、零 VM 引用：只保存配置 + 构造期纯校验结果（options 严格解析、
//!   相对路径词法校验）。坏 options/坏路径不产生"成功描述符"——init_error
//!   记录原因，HTTP adapter 在发送 headers 前映射（Options→500 / Path→403）。
//! ```
//!
//! 纯协议决策（本模块，供 VM 与生成 Rust 两腿共用，绝不复制两套）：
//! - 单区间 Range 解析（有效可满足 206 / 有效不可满足 416 / 未知单位·畸形·
//!   多区间 → 忽略 Range → 200；全程 checked u64）。
//! - 前置条件评估（RFC 9110 §13.2.2 顺序：If-Match > If-Unmodified-Since >
//!   If-None-Match > If-Modified-Since；304/412 先于 Range）。
//! - If-Range（仅强 etag 精确匹配或日期不晚于 Last-Modified 时续传，否则完整
//!   200；无法解析 → 200）。
//! - HTTP 日期（IMF-fixdate 格式化/解析；秒级分辨率限制写入 Spec——同秒改写
//!   不可分辨，mtime+len 永不冒充强 ETag：默认不发 ETag，应用显式提供强
//!   etag 才发）。
//! - 响应头策略（Content-Type/Length/Range/Accept-Ranges/Last-Modified/
//!   ETag/Content-Disposition，注入值 CR/LF/NUL/非 ASCII 拒绝；download_name
//!   非 ASCII 走 RFC 5987 `filename*=UTF-8''` 编码 + ASCII 净化回退）。
//!
//! 排除面：本模块不打开文件、不读盘、不持有网络执行（宿主执行在消费端
//! `auto_lang::http_file_service`）；不做 multipart/byteranges、压缩表示、
//! 目录列表或 `Server.static`。

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Deserialize;

// ============================================================================
// 描述符与构造期错误
// ============================================================================

/// 构造期错误种类（HTTP adapter 映射：Options→500 / Path→403）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileInitErrorKind {
    /// options JSON 解析失败/未知键/非法值（应用错误 → 500）。
    Options,
    /// 相对路径词法校验失败（`..`/绝对/drive/UNC/设备名/NUL → 403）。
    Path,
}

impl FileInitErrorKind {
    pub fn as_str(self) -> &'static str {
        match self {
            FileInitErrorKind::Options => "options",
            FileInitErrorKind::Path => "path",
        }
    }
}

/// 构造期错误（描述符携带；adapter 在发送 headers 前映射）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileInitError {
    pub kind: FileInitErrorKind,
    pub message: String,
}

/// owned 文件响应描述符（PLAN-729 §2：只保存配置，不打开、不整文件读、
/// 不带 VM 引用）。VM 桥以 `id()` 登记/取出。
pub struct FileResponse {
    id: u64,
    root: PathBuf,
    relative: String,
    options: FileResponseOptions,
    init_error: Option<FileInitError>,
}

impl std::fmt::Debug for FileResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileResponse")
            .field("id", &self.id)
            .field("root", &self.root)
            .field("relative", &self.relative)
            .field("init_error", &self.init_error)
            .finish_non_exhaustive()
    }
}

impl FileResponse {
    /// 描述符 id（VM 登记与 scope 组收口用；全局单调，不复用）。
    pub fn id(&self) -> u64 {
        self.id
    }

    /// 应用受信的根目录（构造原样保存；打开在宿主执行）。
    pub fn root(&self) -> &std::path::Path {
        &self.root
    }

    /// 相对路径（可来自路由参数；不可信面——词法校验在构造期完成）。
    pub fn relative(&self) -> &str {
        &self.relative
    }

    /// 已解析 options（init_error 存在时为默认空 options）。
    pub fn options(&self) -> &FileResponseOptions {
        &self.options
    }

    /// 构造期错误（None = 描述符有效，可进入宿主打开阶段）。
    pub fn init_error(&self) -> Option<&FileInitError> {
        self.init_error.as_ref()
    }
}

fn next_file_response_id() -> u64 {
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// 构造文件响应描述符（非阻塞、零 I/O；`http.file_response(root,
/// relative_path, options)` 的共享实现，VM/a2r/原生 Rust 同词汇）。
///
/// - `options_json` 严格解析：空对象 `{}` 合法；未知键/错类型/非法头值/
///   弱 etag → 描述符携带 `Options` init_error（adapter 映射 500，不静默
///   回退默认成功）。`""` 按空对象处理（与传输 options 的宽容入口一致）。
/// - `relative_path` 词法校验：拒绝 `..`、绝对路径、Windows drive/UNC、
///   NUL、设备名 → `Path` init_error（adapter 映射 403，不泄露主机路径）。
pub fn file_response(root: &str, relative_path: &str, options_json: &str) -> FileResponse {
    let id = next_file_response_id();
    let options_json_normalized = if options_json.trim().is_empty() {
        "{}"
    } else {
        options_json
    };
    let (options, options_err) = parse_file_options(options_json_normalized);
    let path_err = validate_relative_path(relative_path)
        .err()
        .map(|message| FileInitError {
            kind: FileInitErrorKind::Path,
            message,
        });
    FileResponse {
        id,
        root: PathBuf::from(root),
        relative: relative_path.to_string(),
        options: options.unwrap_or_default(),
        init_error: options_err.or(path_err),
    }
}

// ============================================================================
// options（严格解析）
// ============================================================================

/// Content-Disposition 策略。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Disposition {
    /// 默认：显式 attachment 或提供 download_name 时使用。
    #[default]
    Attachment,
    /// 显式 inline。
    Inline,
}

/// 已解析的文件响应 options。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FileResponseOptions {
    /// 覆盖 MIME 推断（已通过头值安全校验）。
    pub content_type: Option<String>,
    /// 下载名（已剥离路径成分；CR/LF/NUL 已拒绝）。
    pub download_name: Option<String>,
    /// Content-Disposition 策略。
    pub disposition: Disposition,
    /// 应用提供的强 ETag 验证器（`W/` 已拒绝）。
    pub etag: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileOptionsRaw {
    #[serde(default)]
    content_type: Option<String>,
    #[serde(default)]
    download_name: Option<String>,
    #[serde(default)]
    disposition: Option<String>,
    #[serde(default)]
    etag: Option<String>,
}

/// 单个 HTTP 头字段值的安全校验：非空、ASCII 可见字符 + 空水平空白、
/// 无 CR/LF/NUL/控制字符（响应头注入防线，PLAN-729 §5.2）。
fn is_safe_header_value(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|b| {
            (0x20..=0x7E).contains(&b) // visible ASCII (space allowed)
                || b == b'\t'
        })
}

/// 解析严格 options。返回 (options, error)；error 存在时 options 为空默认。
fn parse_file_options(json: &str) -> (Option<FileResponseOptions>, Option<FileInitError>) {
    let raw: FileOptionsRaw = match serde_json::from_str(json) {
        Ok(r) => r,
        Err(e) => {
            return (
                None,
                Some(FileInitError {
                    kind: FileInitErrorKind::Options,
                    message: format!("invalid file_response options JSON: {e}"),
                }),
            );
        }
    };
    let options_err = |msg: String| {
        (
            None,
            Some(FileInitError {
                kind: FileInitErrorKind::Options,
                message: msg,
            }),
        )
    };
    if let Some(ct) = &raw.content_type {
        if !is_safe_header_value(ct) {
            return options_err("content_type is not a safe header value".to_string());
        }
    }
    if let Some(name) = &raw.download_name {
        if name.contains('\r') || name.contains('\n') || name.contains('\0') {
            return option_err("download_name contains control characters");
        }
    }
    let disposition = match raw.disposition.as_deref() {
        None => Disposition::Attachment,
        Some("attachment") => Disposition::Attachment,
        Some("inline") => Disposition::Inline,
        Some(other) => {
            return option_err(format!(
                "disposition must be \"attachment\" or \"inline\", got {other:?}"
            ));
        }
    };
    if let Some(etag) = &raw.etag {
        if !is_safe_header_value(etag) {
            return option_err("etag is not a safe header value".to_string());
        }
        if etag.starts_with("W/") || etag.starts_with("w/") {
            return option_err(
                "etag must be a strong validator (weak \"W/\" forms are rejected)".to_string(),
            );
        }
    }
    (
        Some(FileResponseOptions {
            content_type: raw.content_type,
            // 剥离路径成分（防 header/路径混合注入；打开侧同样再剥一次）。
            download_name: raw
                .download_name
                .as_deref()
                .map(basename_of)
                .filter(|s| !s.is_empty()),
            disposition,
            etag: raw.etag,
        }),
        None,
    )
}

fn option_err(msg: impl Into<String>) -> (Option<FileResponseOptions>, Option<FileInitError>) {
    (
        None,
        Some(FileInitError {
            kind: FileInitErrorKind::Options,
            message: msg.into(),
        }),
    )
}

/// 剥离路径成分：按 `/` 与 `\` 取最后一段，再去 Windows drive 前缀。
fn basename_of(name: &str) -> String {
    let after_slashes = name.rsplit(['/', '\\']).next().unwrap_or("");
    // "C:foo" → "foo"；裸 "C:" → ""。
    match after_slashes.find(':') {
        Some(idx) if idx == 1 => after_slashes[idx + 1..].to_string(),
        _ => after_slashes.to_string(),
    }
}

// ============================================================================
// 相对路径词法校验（§5.3）
// ============================================================================

/// Windows 传统 DOS 设备名（基名剥离扩展后匹配，大小写不敏感）——T-01 探针
/// P3a 证实 `root\NUL` 打开的是设备句柄且 metadata 报错，is_file 门拦不住，
/// 必须词法拒绝。
const DOS_DEVICE_NAMES: [&str; 22] = [
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

fn is_dos_device_component(component: &str) -> bool {
    let base = component.split('.').next().unwrap_or(component);
    let upper = base.to_ascii_uppercase();
    DOS_DEVICE_NAMES.contains(&upper.as_str())
}

/// 词法校验相对路径。拒绝：空、NUL、绝对路径（`/`、`\` 开头）、Windows
/// drive（第二字符 `:`）、UNC（`\\`）、`..` 段、设备名段、非 UTF-8 边界
/// 由 &str 保证。返回 Err(message) 描述原因（不包含主机绝对路径）。
pub fn validate_relative_path(relative: &str) -> Result<(), String> {
    if relative.is_empty() {
        return Err("relative path is empty".to_string());
    }
    if relative.contains('\0') {
        return Err("relative path contains NUL".to_string());
    }
    if relative.starts_with('/') || relative.starts_with('\\') {
        return Err("relative path must not be absolute".to_string());
    }
    // Windows drive（"C:…"）与 UNC（"\\…"，上面已拒 `\\` 开头，双检中段）。
    let chars: Vec<char> = relative.chars().collect();
    if chars.len() >= 2 && chars[1] == ':' {
        return Err("relative path must not contain a drive prefix".to_string());
    }
    for component in relative.split(['/', '\\']) {
        if component == ".." {
            return Err("relative path must not traverse upward (\"..\")".to_string());
        }
        if is_dos_device_component(component) {
            return Err("relative path must not name a device".to_string());
        }
    }
    Ok(())
}

// ============================================================================
// 请求条件提取（头名小写、first-wins；VM/生成两腿统一从这里取值）
// ============================================================================

/// 影响文件表示选择的请求头（已提取的 owned 快照）。
#[derive(Debug, Clone, Default)]
pub struct FileRequestConditions {
    pub range: Option<String>,
    pub if_match: Option<String>,
    pub if_none_match: Option<String>,
    pub if_modified_since: Option<String>,
    pub if_unmodified_since: Option<String>,
    pub if_range: Option<String>,
}

/// 从请求头对提取条件（头名大小写无关；同名多值 first-wins——与
/// `ApiRequest`/axum HeaderMap 迭代序一致，重复头不合成，行为写入 Spec）。
pub fn extract_request_conditions(headers: &[(String, String)]) -> FileRequestConditions {
    let mut c = FileRequestConditions::default();
    for (name, value) in headers {
        let v = value.trim();
        if v.is_empty() {
            continue;
        }
        let set = |slot: &mut Option<String>| {
            if slot.is_none() {
                *slot = Some(v.to_string());
            }
        };
        match name.to_ascii_lowercase().as_str() {
            "range" => set(&mut c.range),
            "if-match" => set(&mut c.if_match),
            "if-none-match" => set(&mut c.if_none_match),
            "if-modified-since" => set(&mut c.if_modified_since),
            "if-unmodified-since" => set(&mut c.if_unmodified_since),
            "if-range" => set(&mut c.if_range),
            _ => {}
        }
    }
    c
}

impl FileRequestConditions {
    fn header(&self, name: &str) -> Option<&str> {
        match name {
            "range" => self.range.as_deref(),
            "if-match" => self.if_match.as_deref(),
            "if-none-match" => self.if_none_match.as_deref(),
            "if-modified-since" => self.if_modified_since.as_deref(),
            "if-unmodified-since" => self.if_unmodified_since.as_deref(),
            "if-range" => self.if_range.as_deref(),
            _ => None,
        }
    }
}

// ============================================================================
// 单区间 Range 解析（RFC 9110 §14 子集）
// ============================================================================

/// 单区间 Range 解析结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileRangePlan {
    /// 无/忽略 Range（未知单位、畸形、多区间）：完整 200。
    Full,
    /// 有效可满足：206，闭区间窗口（end 已截至 EOF）。
    Partial { start: u64, end: u64 },
    /// 有效但不可满足：416 + `Content-Range: bytes */len` + 空体。
    Unsatisfiable,
}

/// 解析单个 `bytes=` 区间（决策报告 §4 冻结语义）：
/// - `S-E`：S<E 校验；`S >= len` → 416；`E` 截至 `len-1`。
/// - `S-`：`S < len` → 到 EOF；`S == len`/越界 → 416。
/// - `-N`：`N == 0` → 416；`N >= len` → 整文件窗口；否则后缀窗口。
/// - `len == 0` 时任何有效 bytes 区间 → 416（零长表示无可满足区间）。
/// - 未知单位/畸形（含 S>E、非数字、溢出）/多区间 → 忽略 → Full。
pub fn parse_range_header(header: Option<&str>, file_len: u64) -> FileRangePlan {
    let raw = match header.map(str::trim).filter(|h| !h.is_empty()) {
        Some(h) => h,
        None => return FileRangePlan::Full,
    };
    let spec = match raw
        .strip_prefix("bytes=")
        .or_else(|| raw.strip_prefix("BYTES="))
    {
        Some(s) => s.trim(),
        None => return FileRangePlan::Full, // unknown unit: RFC MUST ignore
    };
    if spec.contains(',') {
        return FileRangePlan::Full; // multi-range: ignore (no multipart/byteranges)
    }
    let Some((a, b)) = spec.split_once('-') else {
        return FileRangePlan::Full; // malformed: ignore
    };
    let (a, b) = (a.trim(), b.trim());
    if a.contains('-') || b.contains('-') || b.starts_with('+') || a.starts_with('+') {
        return FileRangePlan::Full;
    }
    if file_len == 0 {
        // 任何语法有效的 bytes 区间在零长表示上都不可满足（suffix 除外见下：
        // -N with N>0 整文件=空窗口同样不可满足）。
        if a.is_empty() && b.is_empty() {
            return FileRangePlan::Full; // "bytes=-" 畸形 → 忽略
        }
        return FileRangePlan::Unsatisfiable;
    }
    let last = file_len - 1; // file_len >= 1；end 上界（u64 安全）
    match (a, b) {
        // bytes=S-
        (start_s, "") => match start_s.parse::<u64>() {
            Ok(s) if s <= last => FileRangePlan::Partial {
                start: s,
                end: last,
            },
            Ok(_) => FileRangePlan::Unsatisfiable,
            Err(_) => FileRangePlan::Full,
        },
        // bytes=-N (suffix)
        ("", suffix_s) => match suffix_s.parse::<u64>() {
            Ok(0) => FileRangePlan::Unsatisfiable,
            Ok(n) => {
                let start = file_len.saturating_sub(n); // n >= len → 0
                FileRangePlan::Partial { start, end: last }
            }
            Err(_) => FileRangePlan::Full,
        },
        // bytes=S-E
        (start_s, end_s) => match (start_s.parse::<u64>(), end_s.parse::<u64>()) {
            (Ok(s), Ok(e)) if s <= e && s <= last => FileRangePlan::Partial {
                start: s,
                end: e.min(last),
            },
            (Ok(s), Ok(e)) if s > e => FileRangePlan::Full, // 畸形（S>E）→ 忽略
            (Ok(_), Ok(_)) => FileRangePlan::Unsatisfiable, // s > last
            _ => FileRangePlan::Full,
        },
    }
}

impl FileRangePlan {
    /// 窗口字节数（416/忽略 = 0；content_length 用）。
    pub fn window_len(&self, file_len: u64) -> u64 {
        match self {
            FileRangePlan::Full => file_len,
            FileRangePlan::Partial { start, end } => end - start + 1, // end>=start 已保证
            FileRangePlan::Unsatisfiable => 0,
        }
    }
}

// ============================================================================
// HTTP 日期（IMF-fixdate only）
// ============================================================================

const WEEKDAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// days-from-civil（Howard Hinnant 算法；1970-01-01=0）。
fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400; // [0, 399]
    let mp = (m as i64 + 9) % 12; // [0, 11]
    let doy = (153 * mp + 2) / 5 + d as i64 - 1; // [0, 365]
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy; // [0, 146096]
    era * 146097 + doe - 719468
}

/// civil-from-days（逆算法；返回 (y, m, d)）。
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32; // [1, 12]
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// SystemTime → IMF-fixdate（"Sun, 06 Nov 1994 08:49:37 GMT"）。
/// 精度：秒（HTTP 日期规范；写入 Spec 的秒级分辨率限制）。
pub fn format_http_date(t: SystemTime) -> String {
    let secs = t
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let days = secs.div_euclid(86_400);
    let sod = secs.rem_euclid(86_400);
    let (y, m, d) = civil_from_days(days);
    let weekday = ((days % 7 + 7 + 4) % 7) as usize; // 1970-01-01 是周四(4)
    format!(
        "{}, {:02} {} {:04} {:02}:{:02}:{:02} GMT",
        WEEKDAYS[weekday],
        d,
        MONTHS[(m - 1) as usize],
        y,
        sod / 3600,
        (sod % 3600) / 60,
        sod % 60
    )
}

/// 解析 IMF-fixdate → epoch 秒（决策比较用；None = 非 IMF-fixdate）。
pub fn parse_http_date_secs(s: &str) -> Option<i64> {
    parse_http_date(s).map(|t| {
        t.duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0)
    })
}

/// 解析 IMF-fixdate（仅接受该格式；RFC 9110 §5.6.7 发送方义务的接收侧
/// 严格面——废弃两格式不收）。大小写敏感（协议字面量）。
pub fn parse_http_date(s: &str) -> Option<SystemTime> {
    let s = s.trim();
    let bytes = s.as_bytes();
    // "Sun, 06 Nov 1994 08:49:37 GMT" = 29 字符
    if bytes.len() != 29 {
        return None;
    }
    if bytes[3] != b','
        || bytes[4] != b' '
        || bytes[7] != b' '
        || bytes[11] != b' '
        || bytes[16] != b' '
        || bytes[19] != b':'
        || bytes[22] != b':'
        || bytes[25] != b' '
    {
        return None;
    }
    let weekday = &s[0..3];
    if !WEEKDAYS.contains(&weekday) {
        return None;
    }
    let day: u32 = s[5..7].parse().ok()?;
    let month = MONTHS.iter().position(|m| *m == &s[8..11])? as u32 + 1;
    let year: i64 = s[12..16].parse().ok()?;
    let hour: i64 = s[17..19].parse().ok()?;
    let min: i64 = s[20..22].parse().ok()?;
    let sec: i64 = s[23..25].parse().ok()?;
    if &s[26..29] != "GMT" {
        return None;
    }
    if !(1..=31).contains(&day) || hour > 23 || min > 59 || sec > 59 {
        return None;
    }
    let days = days_from_civil(year, month, day);
    let secs = days * 86_400 + hour * 3600 + min * 60 + sec;
    UNIX_EPOCH.checked_add(std::time::Duration::from_secs(secs as u64))
}

// ============================================================================
// 前置条件与 If-Range
// ============================================================================

/// 已打开表示的验证器快照（同句柄 metadata + 应用 etag）。
#[derive(Debug, Clone, Default)]
pub struct RepresentationValidators {
    /// 应用提供的强 ETag（None = 不发 ETag，也不参与 If-Match 匹配）。
    pub etag: Option<String>,
    /// 句柄 Last-Modified（None = 不可得——某些 FS 不提供）。
    pub last_modified: Option<SystemTime>,
}

impl RepresentationValidators {
    fn last_modified_secs(&self) -> Option<i64> {
        self.last_modified.map(|t| {
            t.duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0)
        })
    }
}

/// etag 字符串匹配（形态感知）：`strong` 要求去掉引号的裸比较之外，还要求
/// 两形态一致——`tag`（标准 `"x"` 形）之间做字符串全等；非标准形态做裸等。
fn etag_literal_match(a: &str, b: &str) -> bool {
    a == b
}

/// If-Match/If-None-Match 列表解析：`*` → None（通配）；否则按逗号拆分
/// （引号内逗号不拆——ETag 值不含裸逗号，简单 split 够用且与主流实现一致）。
fn parse_etag_list(value: &str) -> Option<Vec<String>> {
    let v = value.trim();
    if v == "*" {
        return None; // wildcard
    }
    Some(v.split(',').map(|s| s.trim().to_string()).collect())
}

/// 弱形态剥离（`W/"x"` → `"x"`；用于 If-None-Match 的弱比较）。
fn unweak(etag: &str) -> &str {
    etag.strip_prefix("W/").unwrap_or(etag)
}

/// 前置条件评估结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreconditionOutcome {
    /// 全部通过（或无前置条件）。
    Proceed,
    /// If-None-Match/If-Modified-Since 命中：304（GET/HEAD）。
    NotModified,
    /// If-Match/If-Unmodified-Since 失败：412。
    PreconditionFailed,
}

/// RFC 9110 §13.2.2 顺序评估前置条件。调用前提：资源存在（句柄已打开）。
/// 日期仅接受 IMF-fixdate；无法解析 → 忽略该头。
pub fn evaluate_preconditions(
    conds: &FileRequestConditions,
    validators: &RepresentationValidators,
    method: &str,
) -> PreconditionOutcome {
    // 1. If-Match
    if let Some(im) = conds.header("if-match") {
        match parse_etag_list(im) {
            None => { // "*"：资源存在 → 通过
            }
            Some(list) => {
                let matched = validators
                    .etag
                    .as_deref()
                    .map(|etag| {
                        // 强比较：弱验证器永不匹配 If-Match。
                        list.iter()
                            .any(|c| !c.starts_with("W/") && etag_literal_match(c, etag))
                    })
                    .unwrap_or(false);
                if !matched {
                    return PreconditionOutcome::PreconditionFailed;
                }
            }
        }
    } else if let Some(ius) = conds.header("if-unmodified-since") {
        // 2. If-Unmodified-Since（仅无 If-Match 时评估）
        if let (Some(date), Some(lm)) = (parse_http_date_secs(ius), validators.last_modified_secs())
        {
            if lm > date {
                return PreconditionOutcome::PreconditionFailed;
            }
        }
    }
    // 3. If-None-Match（GET/HEAD 才有 304 语义；其他方法命中=412，但文件
    //    端点仅 GET/HEAD——非 GET/HEAD 已在上游 405，这里按 GET/HEAD 处理）。
    if let Some(inm) = conds.header("if-none-match") {
        match parse_etag_list(inm) {
            None => {
                // "*"：资源存在 → 304
                if method.eq_ignore_ascii_case("GET") || method.eq_ignore_ascii_case("HEAD") {
                    return PreconditionOutcome::NotModified;
                }
            }
            Some(list) => {
                let matched = validators
                    .etag
                    .as_deref()
                    .map(|etag| {
                        // 弱比较允许（If-None-Match 语义）。
                        list.iter()
                            .any(|c| etag_literal_match(unweak(c), unweak(etag)))
                    })
                    .unwrap_or(false);
                if matched
                    && (method.eq_ignore_ascii_case("GET") || method.eq_ignore_ascii_case("HEAD"))
                {
                    return PreconditionOutcome::NotModified;
                }
            }
        }
    } else if let Some(ims) = conds.header("if-modified-since") {
        // 4. If-Modified-Since（仅无 If-None-Match；GET/HEAD）
        if method.eq_ignore_ascii_case("GET") || method.eq_ignore_ascii_case("HEAD") {
            if let (Some(date), Some(lm)) =
                (parse_http_date_secs(ims), validators.last_modified_secs())
            {
                if lm <= date {
                    return PreconditionOutcome::NotModified;
                }
            }
        }
    }
    PreconditionOutcome::Proceed
}

/// If-Range 评估（仅在 Range 将产 206 时调用）。决策报告 §4：
/// 强 etag 精确匹配 → 续传；失配/弱标签 → 完整 200；HTTP-date：
/// `lm 截秒 ≤ date 截秒` → 续传（日期验证器秒级分辨率的明示限制），否则 200；
/// 无法解析/无验证器可比 → 200。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IfRangeOutcome {
    ServePartial,
    FullResponse,
}

pub fn evaluate_if_range(
    if_range: Option<&str>,
    validators: &RepresentationValidators,
) -> IfRangeOutcome {
    let Some(value) = if_range.map(str::trim).filter(|v| !v.is_empty()) else {
        return IfRangeOutcome::ServePartial; // 无 If-Range：Range 正常生效
    };
    // HTTP-date 形态（含 GMT 结尾）→ 日期验证器。
    if value.ends_with("GMT") {
        if let (Some(date), Some(lm)) =
            (parse_http_date_secs(value), validators.last_modified_secs())
        {
            return if lm <= date {
                IfRangeOutcome::ServePartial
            } else {
                IfRangeOutcome::FullResponse
            };
        }
        return IfRangeOutcome::FullResponse; // 坏日期/无 LM → 200
    }
    // ETag 形态：应用未提供强验证器 → 无法确定为强验证 → 200。
    let Some(etag) = validators.etag.as_deref() else {
        return IfRangeOutcome::FullResponse;
    };
    if value.starts_with("W/") || value.starts_with("w/") {
        return IfRangeOutcome::FullResponse; // 弱标签 → 200
    }
    if etag_literal_match(value, etag) {
        IfRangeOutcome::ServePartial
    } else {
        IfRangeOutcome::FullResponse
    }
}

// ============================================================================
// 组合决策（宿主一次调用）
// ============================================================================

/// 一个请求对已打开文件的完整协议决策（纯函数）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileProtocolDecision {
    /// 200 / 206 / 304 / 412 / 416。
    pub status: u16,
    /// 是否发送文件窗口字节（HEAD 与 304/412/416 为 false）。
    pub include_body: bool,
    /// 响应 Content-Length 值（200=表示长；206=窗口长；304=表示长；412/416=0）。
    pub content_length: u64,
    /// 206 发送窗口 [start, end]（include_body 时 Some）。
    pub window: Option<(u64, u64)>,
    /// 416 诊断用：完整表示长（`Content-Range: bytes */len`）。
    pub representation_len: u64,
}

/// 组合决策（决策报告 §4 冻结顺序：前置条件先于 Range；HEAD 忽略 Range、
/// Content-Length=GET 表示长度；GET 的 Range 经 If-Range 短路）。
pub fn decide_file_protocol(
    method: &str,
    conds: &FileRequestConditions,
    representation_len: u64,
    validators: &RepresentationValidators,
) -> FileProtocolDecision {
    let is_head = method.eq_ignore_ascii_case("HEAD");
    let pre = evaluate_preconditions(conds, validators, method);
    match pre {
        PreconditionOutcome::NotModified => FileProtocolDecision {
            status: 304,
            include_body: false,
            content_length: representation_len, // RFC 9110 §8.6：允许且须等于 200 值
            window: None,
            representation_len,
        },
        PreconditionOutcome::PreconditionFailed => FileProtocolDecision {
            status: 412,
            include_body: false,
            content_length: 0,
            window: None,
            representation_len,
        },
        PreconditionOutcome::Proceed => {
            if is_head {
                // HEAD：按 GET 表示 metadata，忽略 Range；body 为 0 但
                // Content-Length 保持表示长度（§5.2）。
                return FileProtocolDecision {
                    status: 200,
                    include_body: false,
                    content_length: representation_len,
                    window: None,
                    representation_len,
                };
            }
            let plan = parse_range_header(conds.header("range"), representation_len);
            match plan {
                FileRangePlan::Full => FileProtocolDecision {
                    status: 200,
                    include_body: true,
                    content_length: representation_len,
                    window: if representation_len == 0 {
                        None
                    } else {
                        Some((0, representation_len - 1))
                    },
                    representation_len,
                },
                FileRangePlan::Unsatisfiable => FileProtocolDecision {
                    status: 416,
                    include_body: false,
                    content_length: 0,
                    window: None,
                    representation_len,
                },
                FileRangePlan::Partial { start, end } => {
                    // If-Range 短路：失配 → 完整 200。
                    match evaluate_if_range(conds.header("if-range"), validators) {
                        IfRangeOutcome::ServePartial => FileProtocolDecision {
                            status: 206,
                            include_body: true,
                            content_length: end - start + 1,
                            window: Some((start, end)),
                            representation_len,
                        },
                        IfRangeOutcome::FullResponse => FileProtocolDecision {
                            status: 200,
                            include_body: true,
                            content_length: representation_len,
                            window: if representation_len == 0 {
                                None
                            } else {
                                Some((0, representation_len - 1))
                            },
                            representation_len,
                        },
                    }
                }
            }
        }
    }
}

// ============================================================================
// 响应头构建
// ============================================================================

/// 百分号编码（RFC 5987 attr-chr 之外的字符；UTF-8 字节序列逐字节）。
fn percent_encode_utf8(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        let keep = b.is_ascii_alphanumeric()
            || matches!(
                b,
                b'!' | b'#' | b'$' | b'&' | b'+' | b'-' | b'.' | b'^' | b'_' | b'`' | b'|' | b'~'
            );
        if keep {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// quoted-string 转义（ASCII 回退文件名：`"` 与 `\` 反斜杠转义；其余
/// 可见 ASCII 原样；非可见字符替换 `_`）。
fn quote_ascii_filename(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .map(|c| {
            if c.is_ascii() && (0x20..=0x7E).contains(&(c as u8)) && c != '"' && c != '\\' {
                c
            } else {
                '_'
            }
        })
        .collect();
    format!("\"{sanitized}\"")
}

/// Content-Disposition 值构建。默认（无 download_name 且未显式 disposition
/// 选项）不生成该头；有名字或显式 inline 时生成（§4 冻结）。非 ASCII 名走
/// `filename*=UTF-8''…`（RFC 5987/6266）+ 净化 ASCII 回退 filename。
pub fn build_content_disposition(options: &FileResponseOptions) -> Option<String> {
    let name = options.download_name.as_deref();
    match (options.disposition, name) {
        (Disposition::Inline, None) => Some("inline".to_string()),
        (_, None) => None, // attachment 且无名字：不发头（浏览器按 content-type 内联渲染）
        (_, Some(name)) => {
            let kind = match options.disposition {
                Disposition::Attachment => "attachment",
                Disposition::Inline => "inline",
            };
            if name.is_ascii() {
                Some(format!("{kind}; filename={}", quote_ascii_filename(name)))
            } else {
                Some(format!(
                    "{kind}; filename={}; filename*=UTF-8''{}",
                    quote_ascii_filename(name),
                    percent_encode_utf8(name)
                ))
            }
        }
    }
}

/// 构建文件响应头（纯；宿主把推断的 MIME 作为 fallback 传入）。
/// 覆盖：Content-Type、Content-Length、Accept-Ranges（200/206）、
/// Content-Range（206 窗口 / 416 诊断）、Last-Modified（200/206/304）、
/// ETag（仅应用提供时）、Content-Disposition。
pub fn build_file_headers(
    options: &FileResponseOptions,
    decision: &FileProtocolDecision,
    validators: &RepresentationValidators,
    inferred_content_type: &str,
) -> Vec<(String, String)> {
    let mut headers: Vec<(String, String)> = Vec::new();
    let content_type = options
        .content_type
        .as_deref()
        .unwrap_or(inferred_content_type);
    headers.push(("Content-Type".to_string(), content_type.to_string()));
    headers.push((
        "Content-Length".to_string(),
        decision.content_length.to_string(),
    ));
    match decision.status {
        200 | 206 => {
            headers.push(("Accept-Ranges".to_string(), "bytes".to_string()));
        }
        _ => {}
    }
    match decision.status {
        206 => {
            if let Some((start, end)) = decision.window {
                headers.push((
                    "Content-Range".to_string(),
                    format!("bytes {}-{}/{}", start, end, decision.representation_len),
                ));
            }
        }
        416 => {
            headers.push((
                "Content-Range".to_string(),
                format!("bytes */{}", decision.representation_len),
            ));
        }
        _ => {}
    }
    if matches!(decision.status, 200 | 206 | 304) {
        if let Some(lm) = validators.last_modified {
            headers.push(("Last-Modified".to_string(), format_http_date(lm)));
        }
        if let Some(etag) = validators.etag.as_deref() {
            headers.push(("ETag".to_string(), etag.to_string()));
        }
    }
    if let Some(cd) = build_content_disposition(options) {
        headers.push(("Content-Disposition".to_string(), cd));
    }
    headers
}

// ============================================================================
// 表驱动测试（T-02 验证：零长度、大位置、畸形/多范围、前置条件全覆盖）
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn conds(pairs: &[(&str, &str)]) -> FileRequestConditions {
        let owned: Vec<(String, String)> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        extract_request_conditions(&owned)
    }

    fn validators(etag: Option<&str>, lm_secs: Option<i64>) -> RepresentationValidators {
        RepresentationValidators {
            etag: etag.map(|s| s.to_string()),
            last_modified: lm_secs.map(|s| UNIX_EPOCH + std::time::Duration::from_secs(s as u64)),
        }
    }

    // ---- options 严格解析 ----

    #[test]
    fn server_file_options_empty_and_valid() {
        let d = file_response("/srv", "a.txt", "{}");
        assert!(d.init_error().is_none());
        assert_eq!(d.relative(), "a.txt");
        assert_eq!(d.options().disposition, Disposition::Attachment);
        // 空串按空对象（宽容入口同传输 options）。
        assert!(file_response("/srv", "a.txt", "").init_error().is_none());
        let d2 = file_response(
            "/srv",
            "a.txt",
            r#"{"content_type":"text/plain","download_name":"x.bin","disposition":"inline","etag":"\"v1\""}"#,
        );
        assert!(d2.init_error().is_none());
        assert_eq!(d2.options().content_type.as_deref(), Some("text/plain"));
        assert_eq!(d2.options().etag.as_deref(), Some("\"v1\""));
        assert_eq!(d2.options().disposition, Disposition::Inline);
    }

    #[test]
    fn server_file_options_strict_rejects() {
        // 未知键
        assert_eq!(
            file_response("/srv", "a", r#"{"bogus":1}"#)
                .init_error()
                .unwrap()
                .kind,
            FileInitErrorKind::Options
        );
        // 错类型
        assert!(file_response("/srv", "a", r#"{"etag":5}"#)
            .init_error()
            .is_some());
        // 坏 JSON
        assert!(file_response("/srv", "a", "{oops").init_error().is_some());
        // 弱 etag
        let e = file_response("/srv", "a", r#"{"etag":"W/\"x\""}"#)
            .init_error()
            .unwrap()
            .clone();
        assert_eq!(e.kind, FileInitErrorKind::Options);
        assert!(e.message.contains("strong"));
        // 头值注入
        assert!(
            file_response("/srv", "a", "{\"content_type\":\"a\\r\\nb: 1\"}")
                .init_error()
                .is_some()
        );
        // 非 ASCII content_type
        assert!(file_response("/srv", "a", "{\"content_type\":\"类型\"}")
            .init_error()
            .is_some());
        // 非法 disposition
        assert!(file_response("/srv", "a", r#"{"disposition":"form-data"}"#)
            .init_error()
            .is_some());
        // download_name 控制字符
        assert!(file_response("/srv", "a", "{\"download_name\":\"x\\n\"}")
            .init_error()
            .is_some());
    }

    #[test]
    fn server_file_download_name_path_stripped() {
        let d = file_response("/srv", "a", "{\"download_name\":\"../../etc/passwd\"}");
        assert!(d.init_error().is_none());
        assert_eq!(d.options().download_name.as_deref(), Some("passwd"));
        let d2 = file_response("/srv", "a", r#"{"download_name":"C:\\win\\evil.exe"}"#);
        assert_eq!(d2.options().download_name.as_deref(), Some("evil.exe"));
        let d3 = file_response("/srv", "a", r#"{"download_name":"C:"}"#);
        assert_eq!(d3.options().download_name, None); // 剥完为空 → None
    }

    // ---- 路径词法校验 ----

    #[test]
    fn server_file_path_validation() {
        for bad in [
            "",
            "..",
            "a/../b",
            "../x",
            "a/..",
            "/abs",
            "\\abs",
            "C:\\x",
            "C:x",
            "\\\\srv\\share",
            "a\0b",
            "NUL",
            "nul.bin",
            "CON.txt",
            "dir/COM1",
            "aux",
            "lpt9.tar.gz",
        ] {
            assert!(validate_relative_path(bad).is_err(), "应拒绝 {bad:?}");
        }
        for good in [
            "a.txt",
            "dir/sub/file.bin",
            "中文 名.txt",
            "a.b.c",
            ".",
            "dir/./x",
            "a\\b",
        ] {
            assert!(validate_relative_path(good).is_ok(), "应接受 {good:?}");
        }
        // 构造面：路径错误 → Path kind（403）
        let d = file_response("/srv", "../etc/passwd", "{}");
        assert_eq!(d.init_error().unwrap().kind, FileInitErrorKind::Path);
    }

    // ---- Range 解析 ----

    #[test]
    fn server_file_range_table() {
        let len = 100u64;
        let cases: Vec<(Option<&str>, FileRangePlan)> = vec![
            (None, FileRangePlan::Full),
            (Some(""), FileRangePlan::Full),
            // bounded
            (
                Some("bytes=0-9"),
                FileRangePlan::Partial { start: 0, end: 9 },
            ),
            (
                Some("bytes=50-99"),
                FileRangePlan::Partial { start: 50, end: 99 },
            ),
            (
                Some("bytes=90-200"),
                FileRangePlan::Partial { start: 90, end: 99 },
            ), // end 截至 EOF
            (
                Some("bytes=99-99"),
                FileRangePlan::Partial { start: 99, end: 99 },
            ),
            (Some("bytes=100-200"), FileRangePlan::Unsatisfiable), // start==EOF
            (Some("bytes=150-"), FileRangePlan::Unsatisfiable),
            // open
            (
                Some("bytes=0-"),
                FileRangePlan::Partial { start: 0, end: 99 },
            ),
            (
                Some("bytes=42-"),
                FileRangePlan::Partial { start: 42, end: 99 },
            ),
            (Some("bytes=100-"), FileRangePlan::Unsatisfiable), // 起点=EOF
            // suffix
            (
                Some("bytes=-10"),
                FileRangePlan::Partial { start: 90, end: 99 },
            ),
            (
                Some("bytes=-100"),
                FileRangePlan::Partial { start: 0, end: 99 },
            ), // suffix==len → 整文件
            (
                Some("bytes=-5000"),
                FileRangePlan::Partial { start: 0, end: 99 },
            ), // suffix>len → 整文件
            (Some("bytes=-0"), FileRangePlan::Unsatisfiable),
            // 忽略类（→ 200 full）
            (Some("bytes=abc"), FileRangePlan::Full),
            (Some("bytes=5-2"), FileRangePlan::Full), // S>E 畸形
            (Some("bytes=,"), FileRangePlan::Full),
            (Some("bytes=1-2,5-9"), FileRangePlan::Full), // 多区间
            (Some("chunks=0-9"), FileRangePlan::Full),    // 未知单位
            (
                Some("BYTES=0-9"),
                FileRangePlan::Partial { start: 0, end: 9 },
            ), // 单位大小写不敏感
            (
                Some("bytes=99999999999999999999999999-"),
                FileRangePlan::Full,
            ), // u64 溢出 → 忽略
            (Some("bytes=+5-9"), FileRangePlan::Full),
        ];
        for (header, want) in cases {
            assert_eq!(parse_range_header(header, len), want, "header={header:?}");
        }
    }

    #[test]
    fn server_file_range_zero_len() {
        // 零字节文件：无 Range → 200/0；带任何有效 bytes 区间 → 416。
        assert_eq!(parse_range_header(None, 0), FileRangePlan::Full);
        assert_eq!(
            parse_range_header(Some("bytes=0-"), 0),
            FileRangePlan::Unsatisfiable
        );
        assert_eq!(
            parse_range_header(Some("bytes=0-0"), 0),
            FileRangePlan::Unsatisfiable
        );
        assert_eq!(
            parse_range_header(Some("bytes=-5"), 0),
            FileRangePlan::Unsatisfiable
        );
        assert_eq!(
            parse_range_header(Some("bytes=1-2,3-4"), 0),
            FileRangePlan::Full
        ); // 多区间仍忽略
    }

    #[test]
    fn server_file_range_u64_boundaries() {
        // u64 大位置（稀疏/metadata 探针语义；不读数 GiB）。
        let len = u64::MAX;
        assert_eq!(
            parse_range_header(Some("bytes=0-0"), len),
            FileRangePlan::Partial { start: 0, end: 0 }
        );
        assert_eq!(
            parse_range_header(Some(&format!("bytes={}-", u64::MAX - 1)), len),
            FileRangePlan::Partial {
                start: u64::MAX - 1,
                end: u64::MAX - 1
            }
        );
        // window_len checked：end-start+1 不溢出（end ≤ len-1）。
        let p = parse_range_header(Some(&format!("bytes=0-{}", u64::MAX)), len);
        assert_eq!(p.window_len(len), len);
    }

    // ---- HTTP 日期 ----

    #[test]
    fn server_file_http_date_roundtrip() {
        let t = UNIX_EPOCH + std::time::Duration::from_secs(784_111_777); // 1994-11-06 08:49:37
        assert_eq!(format_http_date(t), "Sun, 06 Nov 1994 08:49:37 GMT");
        let parsed = parse_http_date("Sun, 06 Nov 1994 08:49:37 GMT").unwrap();
        assert_eq!(parsed, t);
        // 星期与日期一致（算法自洽）：2000-02-29（周二）闰年。
        let t2 = UNIX_EPOCH + std::time::Duration::from_secs(951_827_682);
        assert_eq!(format_http_date(t2), "Tue, 29 Feb 2000 12:34:42 GMT");
        // 解析拒绝：废弃格式/坏月份/坏长度/非 GMT。
        assert!(parse_http_date("Sunday, 06-Nov-94 08:49:37 GMT").is_none());
        assert!(parse_http_date("Sun, 06 Nov 1994 08:49:37 EST").is_none());
        assert!(parse_http_date("Sun, 06 Xxx 1994 08:49:37 GMT").is_none());
        assert!(parse_http_date("Sun, 06 Nov 1994 08:49:37").is_none());
        assert!(parse_http_date("").is_none());
    }

    // ---- 前置条件 ----

    #[test]
    fn server_file_preconditions_if_match() {
        let v = validators(Some("\"v1\""), Some(1_000));
        // 匹配
        assert_eq!(
            evaluate_preconditions(&conds(&[("if-match", "\"v1\"")]), &v, "GET"),
            PreconditionOutcome::Proceed
        );
        // 列表
        assert_eq!(
            evaluate_preconditions(&conds(&[("if-match", "\"v0\", \"v1\"")]), &v, "GET"),
            PreconditionOutcome::Proceed
        );
        // 通配符（资源存在 → 通过）
        assert_eq!(
            evaluate_preconditions(&conds(&[("if-match", "*")]), &v, "GET"),
            PreconditionOutcome::Proceed
        );
        // 失配 → 412
        assert_eq!(
            evaluate_preconditions(&conds(&[("if-match", "\"v2\"")]), &v, "GET"),
            PreconditionOutcome::PreconditionFailed
        );
        // 弱形态永不匹配 If-Match（强比较）。
        assert_eq!(
            evaluate_preconditions(&conds(&[("if-match", "W/\"v1\"")]), &v, "GET"),
            PreconditionOutcome::PreconditionFailed
        );
        // 无 etag 可比 → 412（RFC：If-Match 且无 ETag → 失败）。
        let no_etag = validators(None, Some(1_000));
        assert_eq!(
            evaluate_preconditions(&conds(&[("if-match", "\"v1\"")]), &no_etag, "GET"),
            PreconditionOutcome::PreconditionFailed
        );
    }

    #[test]
    fn server_file_preconditions_dates() {
        let v = validators(None, Some(1_000));
        // If-Unmodified-Since 晚于 lm → 通过；早于 → 412。
        let later = format_http_date(UNIX_EPOCH + std::time::Duration::from_secs(2_000));
        let earlier = format_http_date(UNIX_EPOCH + std::time::Duration::from_secs(500));
        assert_eq!(
            evaluate_preconditions(&conds(&[("if-unmodified-since", &later)]), &v, "GET"),
            PreconditionOutcome::Proceed
        );
        assert_eq!(
            evaluate_preconditions(&conds(&[("if-unmodified-since", &earlier)]), &v, "GET"),
            PreconditionOutcome::PreconditionFailed
        );
        // If-Match 存在时 If-Unmodified-Since 被跳过（优先级）。
        assert_eq!(
            evaluate_preconditions(
                &conds(&[("if-match", "*"), ("if-unmodified-since", &earlier)]),
                &v,
                "GET"
            ),
            PreconditionOutcome::Proceed
        );
        // 坏日期忽略。
        assert_eq!(
            evaluate_preconditions(&conds(&[("if-unmodified-since", "yesterday")]), &v, "GET"),
            PreconditionOutcome::Proceed
        );
        // If-Modified-Since：lm <= date → 304；lm > date → Proceed。
        assert_eq!(
            evaluate_preconditions(&conds(&[("if-modified-since", &later)]), &v, "GET"),
            PreconditionOutcome::NotModified
        );
        assert_eq!(
            evaluate_preconditions(&conds(&[("if-modified-since", &earlier)]), &v, "GET"),
            PreconditionOutcome::Proceed
        );
        // If-None-Match 优先于 If-Modified-Since。
        assert_eq!(
            evaluate_preconditions(
                &conds(&[("if-none-match", "\"zz\""), ("if-modified-since", &earlier)]),
                &v,
                "GET"
            ),
            PreconditionOutcome::Proceed
        );
    }

    #[test]
    fn server_file_preconditions_if_none_match() {
        let v = validators(Some("\"v1\""), None);
        assert_eq!(
            evaluate_preconditions(&conds(&[("if-none-match", "\"v1\"")]), &v, "GET"),
            PreconditionOutcome::NotModified
        );
        // 弱比较允许（If-None-Match）。
        assert_eq!(
            evaluate_preconditions(&conds(&[("if-none-match", "W/\"v1\"")]), &v, "GET"),
            PreconditionOutcome::NotModified
        );
        assert_eq!(
            evaluate_preconditions(&conds(&[("if-none-match", "\"v2\"")]), &v, "HEAD"),
            PreconditionOutcome::Proceed
        );
        assert_eq!(
            evaluate_preconditions(&conds(&[("if-none-match", "*")]), &v, "GET"),
            PreconditionOutcome::NotModified
        );
    }

    // ---- If-Range ----

    #[test]
    fn server_file_if_range() {
        let lm = format_http_date(UNIX_EPOCH + std::time::Duration::from_secs(1_000));
        let v = validators(Some("\"v1\""), Some(1_000));
        // 强 etag 匹配 → 206
        assert_eq!(
            evaluate_if_range(Some("\"v1\""), &v),
            IfRangeOutcome::ServePartial
        );
        // 失配 → 200
        assert_eq!(
            evaluate_if_range(Some("\"v2\""), &v),
            IfRangeOutcome::FullResponse
        );
        // 弱标签 → 200
        assert_eq!(
            evaluate_if_range(Some("W/\"v1\""), &v),
            IfRangeOutcome::FullResponse
        );
        // 日期：等于 lm（截秒）→ 206；晚于 lm → 206；早于 lm → 200。
        assert_eq!(
            evaluate_if_range(Some(&lm), &v),
            IfRangeOutcome::ServePartial
        );
        let later = format_http_date(UNIX_EPOCH + std::time::Duration::from_secs(2_000));
        assert_eq!(
            evaluate_if_range(Some(&later), &v),
            IfRangeOutcome::ServePartial
        );
        let earlier = format_http_date(UNIX_EPOCH + std::time::Duration::from_secs(500));
        assert_eq!(
            evaluate_if_range(Some(&earlier), &v),
            IfRangeOutcome::FullResponse
        );
        // 无 etag 且非日期 → 200；坏日期 → 200；无 If-Range → Range 正常。
        let no_etag = validators(None, Some(1_000));
        assert_eq!(
            evaluate_if_range(Some("\"x\""), &no_etag),
            IfRangeOutcome::FullResponse
        );
        assert_eq!(
            evaluate_if_range(Some("garbage"), &v),
            IfRangeOutcome::FullResponse
        );
        assert_eq!(evaluate_if_range(None, &v), IfRangeOutcome::ServePartial);
    }

    // ---- 组合决策 ----

    #[test]
    fn server_file_decide_get_head_and_conditions_first() {
        let v = validators(Some("\"v1\""), Some(1_000));
        // GET plain → 200 full。
        let d = decide_file_protocol("GET", &conds(&[]), 100, &v);
        assert_eq!(
            (d.status, d.include_body, d.content_length),
            (200, true, 100)
        );
        assert_eq!(d.window, Some((0, 99)));
        // HEAD：忽略 Range、Content-Length=表示长、无 body。
        let d2 = decide_file_protocol("HEAD", &conds(&[("range", "bytes=0-9")]), 100, &v);
        assert_eq!(
            (d2.status, d2.include_body, d2.content_length),
            (200, false, 100)
        );
        assert_eq!(d2.window, None);
        // 前置条件先于 Range：304 与 Range 并存 → 304（无 body、长度=表示长）。
        let d3 = decide_file_protocol(
            "GET",
            &conds(&[("range", "bytes=0-9"), ("if-none-match", "\"v1\"")]),
            100,
            &v,
        );
        assert_eq!(
            (d3.status, d3.include_body, d3.content_length),
            (304, false, 100)
        );
        // 412 同理。
        let d4 = decide_file_protocol(
            "GET",
            &conds(&[("range", "bytes=0-9"), ("if-match", "\"vX\"")]),
            100,
            &v,
        );
        assert_eq!((d4.status, d4.include_body), (412, false));
        // 206 精确窗口。
        let d5 = decide_file_protocol("GET", &conds(&[("range", "bytes=10-19")]), 100, &v);
        assert_eq!(
            (d5.status, d5.content_length, d5.window),
            (206, 10, Some((10, 19)))
        );
        // 416。
        let d6 = decide_file_protocol("GET", &conds(&[("range", "bytes=500-")]), 100, &v);
        assert_eq!((d6.status, d6.include_body), (416, false));
        // If-Range 失配 → 完整 200。
        let d7 = decide_file_protocol(
            "GET",
            &conds(&[("range", "bytes=10-19"), ("if-range", "\"v2\"")]),
            100,
            &v,
        );
        assert_eq!(
            (d7.status, d7.content_length, d7.window),
            (200, 100, Some((0, 99)))
        );
        // 零字节文件：200 长度 0（window None）。
        let d8 = decide_file_protocol("GET", &conds(&[]), 0, &v);
        assert_eq!((d8.status, d8.content_length, d8.window), (200, 0, None));
        // 零字节 HEAD：Content-Length=0（不是 1）。
        let d9 = decide_file_protocol("HEAD", &conds(&[]), 0, &v);
        assert_eq!(
            (d9.status, d9.include_body, d9.content_length),
            (200, false, 0)
        );
    }

    // ---- 响应头构建 ----

    #[test]
    fn server_file_headers_and_disposition() {
        let v = validators(Some("\"v1\""), Some(784_111_777));
        let opts = FileResponseOptions::default();
        let d = decide_file_protocol("GET", &conds(&[]), 100, &v);
        let h = build_file_headers(&opts, &d, &v, "text/plain");
        let get = |k: &str| {
            h.iter()
                .find(|(n, _)| n.eq_ignore_ascii_case(k))
                .map(|(_, v)| v.clone())
                .unwrap_or_default()
        };
        assert_eq!(get("Content-Type"), "text/plain");
        assert_eq!(get("Content-Length"), "100");
        assert_eq!(get("Accept-Ranges"), "bytes");
        assert_eq!(get("Last-Modified"), "Sun, 06 Nov 1994 08:49:37 GMT");
        assert_eq!(get("ETag"), "\"v1\"");
        assert_eq!(get("Content-Disposition"), ""); // 默认无名：不发

        // 206 → Content-Range。
        let d206 = decide_file_protocol("GET", &conds(&[("range", "bytes=10-19")]), 100, &v);
        let h206 = build_file_headers(&opts, &d206, &v, "application/octet-stream");
        assert!(h206
            .iter()
            .any(|(n, val)| n == "Content-Range" && val == "bytes 10-19/100"));

        // 416 → bytes */100。
        let d416 = decide_file_protocol("GET", &conds(&[("range", "bytes=500-")]), 100, &v);
        let h416 = build_file_headers(&opts, &d416, &v, "application/octet-stream");
        assert!(h416
            .iter()
            .any(|(n, val)| n == "Content-Range" && val == "bytes */100"));

        // 非 ASCII download_name → RFC 5987。
        let opts_cn = FileResponseOptions {
            download_name: Some("报告 第1期.pdf".to_string()),
            ..Default::default()
        };
        let cd = build_content_disposition(&opts_cn).unwrap();
        assert!(cd.starts_with("attachment; filename="));
        assert!(cd.contains("filename*=UTF-8''"));
        assert!(cd.contains("%E6%8A%A5%E5%91%8A")); // "报告" 的 UTF-8 百分号编码

        // ASCII 名引号/反斜杠转义。
        let opts_q = FileResponseOptions {
            download_name: Some("a\"b\\c.txt".to_string()),
            ..Default::default()
        };
        let cd2 = build_content_disposition(&opts_q).unwrap();
        assert_eq!(
            cd2,
            "attachment; filename=\"a_b_c.txt\"" // " 和 \ 非法字符 → _
        );

        // inline 显式无名字 → inline。
        let opts_in = FileResponseOptions {
            disposition: Disposition::Inline,
            ..Default::default()
        };
        assert_eq!(
            build_content_disposition(&opts_in).as_deref(),
            Some("inline")
        );
    }

    #[test]
    fn server_file_descriptor_ids_monotonic() {
        let a = file_response("/r", "a", "{}");
        let b = file_response("/r", "b", "{}");
        assert!(b.id() > a.id());
        assert_ne!(a.id(), 0);
    }
}
