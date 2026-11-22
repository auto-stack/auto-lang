//! PLAN-1119 AutoWeb HTTPS 只读导航——协议与缓存层（传输无关）。
//!
//! 职责边界（T-01 冻结）：地址/`autoweb://` 别名映射、v1 部署描述严格
//! 解析（重复键/未知字段/媒体/版本）、同源与重定向策略、读取限额、
//! ETag/`304`/`no-store`/`Vary` 缓存语义、只读文档渲染前准入扫描。
//! 网络执行不在本层：消费者实现 [`AutowebTransport`]（auto-ui 侧为
//! reqwest blocking 传输；测试用脚本化传输），本层只产请求/判响应。
//!
//! 限额与媒体合同（冻结，见 PLAN-1119 §8 T-01）：
//! - 描述 `application/vnd.autoweb.deployment+json` ≤ [`MAX_DESCRIPTOR_BYTES`]；
//! - 文档 `text/vnd.autodown`（仅允许附加 `charset=utf-8`）≤
//!   [`MAX_DOCUMENT_BYTES`]（解码后字节）；
//! - 单请求 [`DEFAULT_REQUEST_TIMEOUT_MS`]、单次导航总期限
//!   [`DEFAULT_NAV_DEADLINE_MS`]、重定向 ≤ [`MAX_REDIRECT_HOPS`] 跳且每跳
//!   https + 与初始来源同源；
//! - 只读准入拒绝 `![`（图片）、`<`+标签字符（HTML）、任何 `$`（数学与
//!   `$embed`/`$query`/`$callout` 组件指令同字面）、非白名单链接目标。

use crate::error::{AseError, Kind, Phase};
use std::collections::HashMap;
use std::time::Instant;

// ---------------------------------------------------------------------------
// 冻结常量
// ---------------------------------------------------------------------------

/// v1 部署描述媒体类型（协议 §2）。
pub const DESCRIPTOR_MEDIA: &str = "application/vnd.autoweb.deployment+json";
/// v1 只读 AutoDown 文档媒体类型（T-01 冻结）。
pub const DOCUMENT_MEDIA: &str = "text/vnd.autodown";
/// 描述请求的 `Accept` 值（两种地址写法一致；外壳无 HTML 转交能力）。
pub const ACCEPT_DESCRIPTOR: &str = "application/vnd.autoweb.deployment+json";
/// 文档请求的 `Accept` 值。
pub const ACCEPT_DOCUMENT: &str = "text/vnd.autodown";

/// 描述体上限（字节）。
pub const MAX_DESCRIPTOR_BYTES: usize = 64 * 1024;
/// 文档体上限（解码后字节）。
pub const MAX_DOCUMENT_BYTES: usize = 1024 * 1024;
/// 单请求缺省超时（毫秒）。
pub const DEFAULT_REQUEST_TIMEOUT_MS: u64 = 10_000;
/// 单次导航总期限（毫秒）。
pub const DEFAULT_NAV_DEADLINE_MS: u64 = 20_000;
/// 重定向跳数上限（每跳须 https + 与初始来源同源）。
pub const MAX_REDIRECT_HOPS: usize = 3;

// ---------------------------------------------------------------------------
// 地址与别名（协议 §1）
// ---------------------------------------------------------------------------

/// 地址写法：规范 HTTPS 或原生专用 `autoweb://` 别名。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AddressSpelling {
    Https,
    Autoweb,
}

/// 一个已解析的网络地址（规范形态恒为 HTTPS；别名保留写法用于历史/
/// 收藏与显示）。片段在解析期剥离，绝不发给服务端。
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct WebAddress {
    pub spelling: AddressSpelling,
    /// 小写主机名（`[a-z0-9.-]`；IPv6 字面量 v1 具名拒）。
    pub host: String,
    /// 显式端口（缺省 443 不存储）。
    pub port: Option<u16>,
    /// 以 `/` 起始的路径（空输入归一为 `/`）。
    pub path: String,
    /// 查询串（不含 `?`；无则 None）。
    pub query: Option<String>,
}

impl WebAddress {
    /// 规范 HTTPS URL（缓存键、请求目标；不含片段）。
    pub fn https_url(&self) -> String {
        match self.port {
            Some(port) => format!(
                "https://{}:{}{}{}",
                self.host,
                port,
                self.path,
                self.query
                    .as_deref()
                    .map(|q| format!("?{q}"))
                    .unwrap_or_default()
            ),
            None => format!(
                "https://{}{}{}",
                self.host,
                self.path,
                self.query
                    .as_deref()
                    .map(|q| format!("?{q}"))
                    .unwrap_or_default()
            ),
        }
    }

    /// 保留写法的显示/历史地址（别名由解析后的 URL 构造，不做字符串替换）。
    pub fn display_address(&self) -> String {
        let scheme = match self.spelling {
            AddressSpelling::Https => "https",
            AddressSpelling::Autoweb => "autoweb",
        };
        match self.port {
            Some(port) => format!(
                "{}://{}:{}{}{}",
                scheme,
                self.host,
                port,
                self.path,
                self.query
                    .as_deref()
                    .map(|q| format!("?{q}"))
                    .unwrap_or_default()
            ),
            None => format!(
                "{}://{}{}{}",
                scheme,
                self.host,
                self.path,
                self.query
                    .as_deref()
                    .map(|q| format!("?{q}"))
                    .unwrap_or_default()
            ),
        }
    }

    /// 来源（scheme://host[:port]）——同源判据与权限边界。
    pub fn origin(&self) -> String {
        match self.port {
            Some(port) => format!("https://{}:{port}", self.host),
            None => format!("https://{}", self.host),
        }
    }

    /// 以本地址为基准解析相对引用（描述 `entry.href`、重定向目标）。
    pub fn resolve(&self, reference: &str) -> Result<WebAddress, AutowebRefusal> {
        resolve_relative(self, reference)
    }
}

/// 严格解析网络地址：scheme 限 `https`/`autoweb`；`http` 具名降级拒。
///
/// - 拒 userinfo、空主机、IPv6 字面量、非 `[a-z0-9.-]` 主机、越界端口。
/// - 片段允许出现在输入中但剥离后不外发。
/// - 别名映射到同一 host/port/path/query 的 HTTPS 规范地址。
pub fn parse_web_address(input: &str) -> Result<WebAddress, AseError> {
    let refuse = |message: String| AseError::new(Phase::Resolve, Kind::Uri, message);
    let input = input.trim();
    let Some((scheme, rest)) = input.split_once("://") else {
        return Err(refuse(format!(
            "not a network address (expected https:// or autoweb://): {input:?}"
        )));
    };
    let spelling = match scheme.to_ascii_lowercase().as_str() {
        "https" => AddressSpelling::Https,
        "autoweb" => AddressSpelling::Autoweb,
        "http" => {
            return Err(refuse(
                "insecure scheme http is refused (HTTPS only)".to_string(),
            ))
        }
        other => {
            return Err(refuse(format!(
                "unsupported scheme {other:?} (only https/autoweb)"
            )))
        }
    };
    // 权威段到首个路径/查询/片段标记。
    let authority_len = rest
        .find(|c| c == '/' || c == '?' || c == '#')
        .unwrap_or(rest.len());
    let authority = &rest[..authority_len];
    let tail = &rest[authority_len..];
    if authority.is_empty() {
        return Err(refuse("empty host".to_string()));
    }
    if let Some(at) = authority.find('@') {
        return Err(refuse(format!(
            "userinfo is not allowed in a web address: {:?}",
            &authority[..at]
        )));
    }
    let (host, port) = match authority.rsplit_once(':') {
        // 仅当冒号后全数字才视为端口（主机名不含冒号；IPv6 括号形态 v1 拒）。
        Some((h, p)) if !h.is_empty() && !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()) => {
            let port: u32 = p
                .parse()
                .map_err(|_| refuse(format!("invalid port {p:?}")))?;
            if port == 0 || port > u32::from(u16::MAX) {
                return Err(refuse(format!("port out of range: {p}")));
            }
            (h, Some(port as u16))
        }
        Some((_, p)) => {
            return Err(refuse(format!(
                "invalid authority (IPv6 literals are not supported in v1): {p:?}"
            )))
        }
        None => (authority, None),
    };
    let host = host.to_ascii_lowercase();
    if !host
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '-')
    {
        return Err(refuse(format!(
            "host {host:?} must be lowercase [a-z0-9.-]"
        )));
    }
    // 尾段：路径/查询/片段。片段剥离（仅内容内部用途）。
    let before_fragment = tail.split_once('#').map(|(b, _)| b).unwrap_or(tail);
    let (path, query) = match before_fragment.split_once('?') {
        Some((p, q)) => (p, (!q.is_empty()).then(|| q.to_string())),
        None => (before_fragment, None),
    };
    if path.contains('\\') || path.contains('\0') {
        return Err(refuse(format!("invalid path {path:?}")));
    }
    let path = if path.is_empty() {
        "/".to_string()
    } else {
        path.to_string()
    };
    Ok(WebAddress {
        spelling,
        host,
        port,
        path,
        query,
    })
}

/// 相对引用解析（RFC 3986 §5 受限面：绝对 https URL、网络路径 `//…`、
/// 绝对路径、相对路径含点段消除；结果须 https 且与本基准同源）。
fn resolve_relative(base: &WebAddress, reference: &str) -> Result<WebAddress, AutowebRefusal> {
    let refuse = |message: String| AutowebRefusal {
        stage: NavStage::DescriptorParse,
        reason: message,
    };
    let reference = reference.trim();
    let no_fragment = reference
        .split_once('#')
        .map(|(b, _)| b)
        .unwrap_or(reference);
    let same = |path: String, query: Option<String>| WebAddress {
        spelling: AddressSpelling::Https,
        host: base.host.clone(),
        port: base.port,
        path,
        query,
    };
    if let Some(rest) = no_fragment.strip_prefix("https://") {
        let resolved = parse_web_address(&format!("https://{rest}"))
            .map_err(|e| refuse(format!("entry target is not a valid https URL: {e}")))?;
        if resolved.origin() != base.origin() {
            return Err(refuse(format!(
                "entry target {:?} is cross-origin (base origin {})",
                resolved.https_url(),
                base.origin()
            )));
        }
        return Ok(resolved);
    }
    if no_fragment.starts_with("//") {
        return Err(refuse(format!(
            "scheme-relative target is not accepted: {no_fragment:?}"
        )));
    }
    if let Some(other) = no_fragment.split_once("://") {
        let scheme = other.0.to_ascii_lowercase();
        if scheme != "https" {
            return Err(refuse(format!(
                "entry target scheme {scheme:?} is not https"
            )));
        }
    }
    if no_fragment.starts_with('/') {
        let (path, query) = split_path_query(no_fragment);
        return Ok(same(remove_dot_segments(&path), query));
    }
    if no_fragment.is_empty() {
        return Ok(same(base.path.clone(), base.query.clone()));
    }
    // 相对路径：与基准路径目录合并后消除点段。
    let (rel_path, query) = split_path_query(no_fragment);
    let base_dir = match base.path.rfind('/') {
        Some(index) => base.path[..=index].to_string(),
        None => "/".to_string(),
    };
    let merged = format!("{base_dir}{rel_path}");
    Ok(same(remove_dot_segments(&merged), query))
}

fn split_path_query(input: &str) -> (String, Option<String>) {
    match input.split_once('?') {
        Some((p, q)) => (p.to_string(), (!q.is_empty()).then(|| q.to_string())),
        None => (input.to_string(), None),
    }
}

/// RFC 3986 §5.2.4 remove_dot_segments（单绝对路径输入）。
fn remove_dot_segments(path: &str) -> String {
    let mut output: Vec<&str> = Vec::new();
    for segment in path.split('/') {
        match segment {
            "." => {}
            ".." => {
                output.pop();
            }
            other => output.push(other),
        }
    }
    let mut resolved = output.join("/");
    if !resolved.starts_with('/') {
        resolved.insert(0, '/');
    }
    // 尾部 `/..`、`/.` 需补回斜杠（目录语义）。
    if (path.ends_with("/..") || path.ends_with("/."))
        && !resolved.ends_with('/')
        && !resolved.is_empty()
    {
        resolved.push('/');
    }
    resolved
}

// ---------------------------------------------------------------------------
// 传输抽象（网络执行归消费者；测试用脚本化传输）
// ---------------------------------------------------------------------------

/// 一次网络请求（GET；头部含协商/条件验证）。
#[derive(Clone, Debug)]
pub struct TransportRequest {
    pub url: String,
    pub headers: Vec<(&'static str, String)>,
    pub timeout_ms: u64,
}

/// 一次网络响应（3xx 交协议层处理；transport 不跟随重定向）。
#[derive(Clone, Debug)]
pub struct TransportResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl TransportResponse {
    /// 大小写无关 header 查找；缺失返回 ""。
    pub fn header_get(&self, key: &str) -> &str {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(key))
            .map(|(_, v)| v.as_str())
            .unwrap_or("")
    }
}

/// 传输层失败（具名；不吞成成功）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TransportFailure {
    Cancelled,
    Timeout,
    Transport(String),
}

impl std::fmt::Display for TransportFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransportFailure::Cancelled => write!(f, "request cancelled"),
            TransportFailure::Timeout => write!(f, "request timed out"),
            TransportFailure::Transport(e) => write!(f, "transport error: {e}"),
        }
    }
}

/// 网络执行面（auto-ui reqwest 实现；测试脚本化实现）。
pub trait AutowebTransport {
    /// 执行一次 GET。逐块读取的实现应在读取间检查取消。
    fn fetch(&mut self, req: &TransportRequest) -> Result<TransportResponse, TransportFailure>;
    /// 取消旗标（导航器在每跳/每次准入前检查）。
    fn cancelled(&self) -> bool {
        false
    }
}

// ---------------------------------------------------------------------------
// v1 部署描述（严格解析：重复键/未知字段经 serde derive 拒绝）
// ---------------------------------------------------------------------------

/// 描述里的入口声明（v1 仅 `kind=autodown`）。
#[derive(serde::Deserialize)]
struct DescriptorEntryDto {
    kind: String,
    href: String,
    sha256: String,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct DeploymentDescriptorDto {
    protocol_version: u32,
    app_id: String,
    release: String,
    title: String,
    entry: DescriptorEntryDto,
    assets: Vec<serde_json::Value>,
    services: Vec<serde_json::Value>,
}

/// 已验证的 v1 部署描述。
#[derive(Clone, Debug)]
pub struct DeploymentDescriptor {
    pub protocol_version: u32,
    pub app_id: String,
    pub release: String,
    pub title: String,
    /// 入口文档的绝对 https URL（相对基准=描述最终 URL）。
    pub entry_url: String,
    /// 入口文档字节 SHA-256。
    pub entry_sha256: [u8; 32],
}

/// 严格解析描述：UTF-8 → serde 严格结构（重复键/未知字段具名拒）→
/// 字段域校验（版本=1、kind=autodown、64 小写 hex、assets/services 空、
/// 入口同源 https）。
pub fn parse_descriptor(
    bytes: &[u8],
    base: &WebAddress,
) -> Result<DeploymentDescriptor, AutowebRefusal> {
    let refuse = |message: String| AutowebRefusal {
        stage: NavStage::DescriptorParse,
        reason: message,
    };
    if bytes.len() > MAX_DESCRIPTOR_BYTES {
        return Err(refuse(format!(
            "descriptor exceeds limit: {} > {MAX_DESCRIPTOR_BYTES} bytes",
            bytes.len()
        )));
    }
    let text = std::str::from_utf8(bytes)
        .map_err(|e| refuse(format!("descriptor is not valid UTF-8: {e}")))?;
    let dto: DeploymentDescriptorDto = serde_json::from_str(text).map_err(|e| {
        refuse(format!(
            "descriptor is not a strict v1 document: {e} (duplicate keys, unknown \
             fields, missing required fields and unsupported values are all refused)"
        ))
    })?;
    if dto.protocol_version != 1 {
        return Err(refuse(format!(
            "unsupported descriptor protocolVersion {} (only 1)",
            dto.protocol_version
        )));
    }
    if dto.app_id.trim().is_empty() || dto.release.trim().is_empty() || dto.title.trim().is_empty()
    {
        return Err(refuse(
            "descriptor appId/release/title must be non-empty".to_string(),
        ));
    }
    if dto.entry.kind != "autodown" {
        return Err(refuse(format!(
            "unsupported entry kind {:?} (only \"autodown\" in the readonly prototype)",
            dto.entry.kind
        )));
    }
    if !dto.assets.is_empty() || !dto.services.is_empty() {
        return Err(refuse(format!(
            "readonly v1 requires empty assets/services (got {} assets, {} services)",
            dto.assets.len(),
            dto.services.len()
        )));
    }
    let sha256 = parse_sha256_hex(&dto.entry.sha256)
        .ok_or_else(|| refuse(format!("entry sha256 must be 64 lowercase hex characters")))?;
    let entry = base
        .resolve(&dto.entry.href)
        .map_err(|r| refuse(format!("entry href rejected: {}", r.reason)))?;
    Ok(DeploymentDescriptor {
        protocol_version: dto.protocol_version,
        app_id: dto.app_id,
        release: dto.release,
        title: dto.title,
        entry_url: entry.https_url(),
        entry_sha256: sha256,
    })
}

/// 64 位小写十六进制 SHA-256。
fn parse_sha256_hex(text: &str) -> Option<[u8; 32]> {
    if text.len() != 64
        || !text
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
    {
        return None;
    }
    let mut out = [0u8; 32];
    for (index, byte) in out.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&text[index * 2..index * 2 + 2], 16).ok()?;
    }
    Some(out)
}

// ---------------------------------------------------------------------------
// 响应准入（媒体/状态/限额）
// ---------------------------------------------------------------------------

/// 媒体类型准入：type/subtype 大小写无关精确匹配；文档允许且仅允许附加
/// `charset=utf-8` 参数。HTML 显式具名（普通写法=不支持、原生写法=拒）。
pub fn admit_media(
    actual: &str,
    expected: &str,
    allow_charset: bool,
) -> Result<(), AutowebRefusal> {
    let refuse = |message: String| AutowebRefusal {
        stage: if expected == DOCUMENT_MEDIA {
            NavStage::DocumentMedia
        } else {
            NavStage::DescriptorMedia
        },
        reason: message,
    };
    let actual = actual.trim();
    let mut parts = actual.split(';');
    let type_subtype = parts.next().unwrap_or("").trim().to_ascii_lowercase();
    if type_subtype == expected {
        for param in parts {
            let param = param.trim();
            if param.is_empty() {
                continue;
            }
            let ok = allow_charset && param.eq_ignore_ascii_case("charset=utf-8");
            if !ok {
                return Err(refuse(format!(
                    "unsupported media parameter {param:?} on {expected}"
                )));
            }
        }
        return Ok(());
    }
    let lower = actual.to_ascii_lowercase();
    if lower.starts_with("text/html") {
        return Err(refuse(format!(
            "server returned HTML ({actual:?}); the AutoWeb readonly prototype does not \
             hand off HTML documents"
        )));
    }
    Err(refuse(format!(
        "unexpected media type {actual:?} (expected {expected})"
    )))
}

/// HTTP 状态 → 状态栏语义映射（协议 §2：404=不存在、406=不可用、5xx=服务
/// 端失败；其余 4xx 具名）。
fn status_reason(status: u16) -> String {
    match status {
        404 => "404: target does not exist".to_string(),
        406 => "406: no acceptable representation on this server".to_string(),
        500..=599 => format!("{status}: server error"),
        (200..300) => format!("{status}: unexpected success status (only 200 is accepted)"),
        other => format!("HTTP {other}"),
    }
}

// ---------------------------------------------------------------------------
// 只读文档准入（渲染前；冻结拒绝清单）
// ---------------------------------------------------------------------------

/// 只读文档准入：零隐式资源、零执行面、链接目标白名单。
///
/// 拒绝清单（T-01 冻结）：`![`（图片）；`<`+`[A-Za-z/!?]`（HTML）；任何
/// `$`（数学与 `$embed`/`$query`/`$callout`/`$details` 组件指令）；链接
/// 目标仅 `https://`、`autoweb://`、`#锚`、相对引用——其余 scheme 具名拒。
pub fn admit_readonly_document(body: &str) -> Result<(), AutowebRefusal> {
    let refuse = |message: String| AutowebRefusal {
        stage: NavStage::DocumentAdmission,
        reason: message,
    };
    if body.contains("![") {
        return Err(refuse(
            "document contains an image (implicit resources are outside the readonly v1 \
             domain)"
                .to_string(),
        ));
    }
    for (index, c) in body.char_indices() {
        if c == '<' {
            let next = body[index + 1..].chars().next();
            if matches!(
                next,
                Some('a'..='z') | Some('A'..='Z') | Some('/') | Some('!') | Some('?')
            ) {
                return Err(refuse(
                    "document contains raw HTML (execution/embed surface is outside the \
                     readonly v1 domain)"
                        .to_string(),
                ));
            }
        }
        if c == '$' {
            return Err(refuse(
                "document contains `$` (math and component directives are outside the \
                 readonly v1 domain)"
                    .to_string(),
            ));
        }
    }
    // 链接目标白名单：扫描 [text](target) 形态（含图片已在上面的 `![` 拒）。
    let mut rest = body;
    while let Some(open) = rest.find(']') {
        let after = &rest[open + 1..];
        if let Some(target) = after.strip_prefix('(') {
            if let Some(close) = target.find(')') {
                let target = &target[..close];
                if !target.is_empty() && !target.starts_with(' ') {
                    admit_link_target(target)?;
                }
            }
        }
        rest = &rest[open + 1..];
    }
    Ok(())
}

fn admit_link_target(target: &str) -> Result<(), AutowebRefusal> {
    let lower = target.to_ascii_lowercase();
    let allowed = lower.starts_with("https://")
        || lower.starts_with("autoweb://")
        || lower.starts_with('#')
        || lower.starts_with('/')
        || lower.starts_with("./")
        || lower.starts_with("../")
        || !lower.contains(':');
    if allowed {
        Ok(())
    } else {
        Err(AutowebRefusal {
            stage: NavStage::DocumentAdmission,
            reason: format!(
                "link target {target:?} is outside the readonly v1 whitelist (https, \
                 autoweb, anchor, relative)"
            ),
        })
    }
}

// ---------------------------------------------------------------------------
// 缓存（协议 §4：规范 URL × 表示 × 来源；ETag 条件验证；no-store；Vary）
// ---------------------------------------------------------------------------

/// 缓存表示维度（`Vary: Accept` 的分离由表示键承担）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Representation {
    Descriptor,
    Document,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CacheKey {
    /// 规范 https URL（地址写法共享缓存）。
    pub url: String,
    pub representation: Representation,
}

#[derive(Clone, Debug)]
pub struct CacheEntry {
    pub etag: Option<String>,
    pub content_type: String,
    pub body: Vec<u8>,
}

/// 一次响应入缓存的结果。
#[derive(Clone, Debug)]
pub enum IngestOutcome {
    /// 存入新表示。
    Stored { etag: Option<String> },
    /// `304` 复用已缓存有效表示（未重下正文）。
    Reused,
    /// `no-store`：正文有效但不入缓存（导航继续用新鲜正文）。
    FreshNotStored,
}

/// `304` 语义违规（无匹配条件请求/无可复用条目）——具名拒绝。
#[derive(Clone, Debug)]
pub struct IngestRefusal(pub String);

/// AutoWeb 只读缓存（worker 单线程持有；无持久化——会话内有效）。
#[derive(Default, Debug)]
pub struct AutowebCache {
    entries: HashMap<CacheKey, CacheEntry>,
    pub revalidations: usize,
    pub full_downloads: usize,
    pub reuse_hits: usize,
}

impl AutowebCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn entry(&self, key: &CacheKey) -> Option<&CacheEntry> {
        self.entries.get(key)
    }

    /// 请求条件头：存在带 ETag 的条目 → 重访必发 `If-None-Match` 复验
    /// （V-02 语义：已验缓存再访问实际发送条件请求）。
    pub fn conditional_headers(&mut self, key: &CacheKey) -> Vec<(&'static str, String)> {
        match self.entries.get(key).and_then(|e| e.etag.clone()) {
            Some(etag) => {
                self.revalidations += 1;
                vec![("If-None-Match", etag)]
            }
            None => Vec::new(),
        }
    }

    /// 响应入缓存。`sent_if_none_match` = 本次请求是否携带了匹配条目的
    /// 条件头（`304` 仅在该前提与条目仍存在时接受）。
    pub fn ingest(
        &mut self,
        key: &CacheKey,
        response: &TransportResponse,
        sent_if_none_match: bool,
    ) -> Result<IngestOutcome, IngestRefusal> {
        if response.status == 304 {
            if !sent_if_none_match {
                return Err(IngestRefusal(
                    "304 without a matching If-None-Match request is refused".to_string(),
                ));
            }
            return if self.entries.contains_key(key) {
                self.reuse_hits += 1;
                Ok(IngestOutcome::Reused)
            } else {
                Err(IngestRefusal(
                    "304 has no cached representation to reuse (refused)".to_string(),
                ))
            };
        }
        let no_store = response
            .header_get("Cache-Control")
            .split(',')
            .any(|directive| directive.trim().eq_ignore_ascii_case("no-store"));
        self.full_downloads += 1;
        if no_store {
            self.entries.remove(key);
            return Ok(IngestOutcome::FreshNotStored);
        }
        let etag = match response.header_get("ETag") {
            "" => None,
            value => Some(value.to_string()),
        };
        self.entries.insert(
            key.clone(),
            CacheEntry {
                etag: etag.clone(),
                content_type: response.header_get("Content-Type").to_string(),
                body: response.body.clone(),
            },
        );
        Ok(IngestOutcome::Stored { etag })
    }
}

// ---------------------------------------------------------------------------
// 导航器（描述 → 验证 → 同源文档 → 摘要/准入 → 成功；失败具名）
// ---------------------------------------------------------------------------

/// 导航阶段（具名；`已下载描述 ≠ 已打开文档`——Success 只在文档完整验证
/// 后产生）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavStage {
    Address,
    DescriptorFetch,
    DescriptorMedia,
    DescriptorParse,
    DocumentFetch,
    DocumentMedia,
    DocumentIntegrity,
    DocumentAdmission,
    Cancelled,
    Deadline,
}

impl NavStage {
    pub fn as_str(self) -> &'static str {
        match self {
            NavStage::Address => "address",
            NavStage::DescriptorFetch => "descriptor-fetch",
            NavStage::DescriptorMedia => "descriptor-media",
            NavStage::DescriptorParse => "descriptor-parse",
            NavStage::DocumentFetch => "document-fetch",
            NavStage::DocumentMedia => "document-media",
            NavStage::DocumentIntegrity => "document-integrity",
            NavStage::DocumentAdmission => "document-admission",
            NavStage::Cancelled => "cancelled",
            NavStage::Deadline => "deadline",
        }
    }
}

/// 具名拒绝（不动已提交页面；旧有效缓存保留）。
#[derive(Clone, Debug)]
pub struct AutowebRefusal {
    pub stage: NavStage,
    pub reason: String,
}

impl AutowebRefusal {
    pub fn message(&self) -> String {
        format!("{}: {}", self.stage.as_str(), self.reason)
    }
}

impl std::fmt::Display for AutowebRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message())
    }
}

/// 导航限额（T-01 冻结缺省；测试可注入更小值）。
#[derive(Clone, Copy, Debug)]
pub struct NavLimits {
    pub descriptor_bytes: usize,
    pub document_bytes: usize,
    pub request_timeout_ms: u64,
    pub nav_deadline_ms: u64,
    pub max_hops: usize,
}

impl Default for NavLimits {
    fn default() -> Self {
        NavLimits {
            descriptor_bytes: MAX_DESCRIPTOR_BYTES,
            document_bytes: MAX_DOCUMENT_BYTES,
            request_timeout_ms: DEFAULT_REQUEST_TIMEOUT_MS,
            nav_deadline_ms: DEFAULT_NAV_DEADLINE_MS,
            max_hops: MAX_REDIRECT_HOPS,
        }
    }
}

/// 一次导航中的单条请求轨迹（证据面：服务器可对账）。
#[derive(Clone, Debug)]
pub struct TraceRequest {
    pub url: String,
    pub accept: &'static str,
    pub if_none_match: Option<String>,
    pub status: u16,
    pub outcome: &'static str,
}

#[derive(Clone, Debug, Default)]
pub struct NavTrace {
    pub requests: Vec<TraceRequest>,
    /// 描述体实际下载字节（`304` 复用为 0）。
    pub descriptor_bytes_downloaded: usize,
    pub document_bytes_downloaded: usize,
    pub cache_reused: bool,
}

/// 导航成功（一次可提交的只读文档；携带提交所需元数据）。
#[derive(Clone, Debug)]
pub struct NavSuccess {
    /// 保留写法的地址（历史/收藏）。
    pub display_address: String,
    /// 规范 https URL。
    pub canonical_url: String,
    pub title: String,
    pub release: String,
    pub app_id: String,
    pub document: String,
    pub document_sha256_hex: String,
    pub trace: NavTrace,
}

#[derive(Debug)]
pub enum NavOutcome {
    Success(NavSuccess),
    Refused(AutowebRefusal, NavTrace),
}

/// 只读导航器：描述 GET（协商/缓存）→ 严格验证 → 同源文档 GET（sha256/
/// 媒体/限额）→ 渲染前准入。重定向逐跳同源校验，跨源/降级在下一请求前拒。
pub struct ReadonlyNavigator<'a, T: AutowebTransport> {
    transport: &'a mut T,
    cache: &'a mut AutowebCache,
    limits: NavLimits,
    started: Instant,
    trace: NavTrace,
}

impl<'a, T: AutowebTransport> ReadonlyNavigator<'a, T> {
    pub fn new(transport: &'a mut T, cache: &'a mut AutowebCache, limits: NavLimits) -> Self {
        ReadonlyNavigator {
            transport,
            cache,
            limits,
            started: Instant::now(),
            trace: NavTrace::default(),
        }
    }

    fn check_alive(&self) -> Result<(), AutowebRefusal> {
        if self.transport.cancelled() {
            return Err(AutowebRefusal {
                stage: NavStage::Cancelled,
                reason: "navigation cancelled before completion".to_string(),
            });
        }
        if self.started.elapsed().as_millis() as u64 > self.limits.nav_deadline_ms {
            return Err(AutowebRefusal {
                stage: NavStage::Deadline,
                reason: format!(
                    "navigation exceeded deadline of {} ms",
                    self.limits.nav_deadline_ms
                ),
            });
        }
        Ok(())
    }

    /// 单资源 GET：同源重定向 ≤ max_hops（每跳 https+同源，跨源/降级/超
    /// 跳在下一次请求前拒）；`304` 复用缓存条目；200 的新鲜正文经
    /// 媒体/限额/`validate` 全验证**之后**才入缓存（未验证完整的内容不
    /// 进入可复用缓存——协议 §4）。返回最终（缓存或新鲜）字节与最终 URL。
    fn fetch_resource(
        &mut self,
        mut url: WebAddress,
        representation: Representation,
        media: &'static str,
        max_bytes: usize,
        validate: &dyn Fn(&WebAddress, &[u8]) -> Result<(), AutowebRefusal>,
    ) -> Result<(Vec<u8>, String), AutowebRefusal> {
        let origin = url.origin();
        let fetch_stage = if representation == Representation::Descriptor {
            NavStage::DescriptorFetch
        } else {
            NavStage::DocumentFetch
        };
        let media_stage = if representation == Representation::Descriptor {
            NavStage::DescriptorMedia
        } else {
            NavStage::DocumentMedia
        };
        let mut hops = 0;
        loop {
            self.check_alive()?;
            if url.origin() != origin {
                return Err(AutowebRefusal {
                    stage: fetch_stage,
                    reason: format!(
                        "redirect target {:?} is cross-origin (base origin {origin}); \
                         refused before the next request",
                        url.https_url()
                    ),
                });
            }
            // 缓存键跟随当前跳的 URL（重定向后的资源独立验证/缓存）。
            let key = key_for(url.clone(), representation);
            let conditional = self.cache.conditional_headers(&key);
            let sent_match = conditional.iter().any(|(k, _)| *k == "If-None-Match");
            let mut headers: Vec<(&'static str, String)> = vec![("Accept", media.to_string())];
            headers.extend(conditional);
            let request = TransportRequest {
                url: url.https_url(),
                headers,
                timeout_ms: self.limits.request_timeout_ms,
            };
            let response = self
                .transport
                .fetch(&request)
                .map_err(|failure| match failure {
                    TransportFailure::Cancelled => AutowebRefusal {
                        stage: NavStage::Cancelled,
                        reason: "request cancelled mid-flight".to_string(),
                    },
                    other => AutowebRefusal {
                        stage: fetch_stage,
                        reason: other.to_string(),
                    },
                })?;
            self.trace.requests.push(TraceRequest {
                url: url.https_url(),
                accept: media,
                if_none_match: if sent_match {
                    self.trace_if_none_match(&request)
                } else {
                    None
                },
                status: response.status,
                outcome: "fetched",
            });
            // `304 Not Modified` 不是重定向：先于 3xx 分支交条件验证处理。
            if response.status != 304 && (300..400).contains(&response.status) {
                let location = response.header_get("Location");
                if location.is_empty() {
                    return Err(AutowebRefusal {
                        stage: fetch_stage,
                        reason: format!(
                            "redirect status {} without a Location header",
                            response.status
                        ),
                    });
                }
                hops += 1;
                if hops > self.limits.max_hops {
                    return Err(AutowebRefusal {
                        stage: fetch_stage,
                        reason: format!("redirect chain exceeds {} hops", self.limits.max_hops),
                    });
                }
                let next = url.resolve(location).map_err(|mut r| {
                    r.stage = fetch_stage;
                    r
                })?;
                // 降级（非 https）由 resolve/parse 直接具名拒；跨源在循环顶拒。
                url = next;
                continue;
            }
            if response.status != 200 && response.status != 304 {
                let reason = status_reason(response.status);
                return Err(AutowebRefusal {
                    stage: fetch_stage,
                    reason,
                });
            }
            // 新鲜 200：媒体 → 限额 → 语义验证全部通过后才入缓存并返回。
            if response.status == 200 {
                admit_media(
                    response.header_get("Content-Type"),
                    media,
                    media == DOCUMENT_MEDIA,
                )
                .map_err(|mut r| {
                    if r.stage != NavStage::DescriptorMedia && r.stage != NavStage::DocumentMedia {
                        r.stage = media_stage;
                    }
                    r
                })?;
                if response.body.len() > max_bytes {
                    return Err(AutowebRefusal {
                        stage: media_stage,
                        reason: format!(
                            "representation exceeds limit: {} > {max_bytes} bytes",
                            response.body.len()
                        ),
                    });
                }
                validate(&url, &response.body)?;
            }
            let (bytes, from_cache) = match self.cache.ingest(&key, &response, sent_match) {
                Ok(IngestOutcome::Reused) => {
                    let entry = self.cache.entry(&key).expect("reuse implies entry");
                    (entry.body.clone(), true)
                }
                Ok(IngestOutcome::Stored { .. }) | Ok(IngestOutcome::FreshNotStored) => {
                    (response.body.clone(), false)
                }
                Err(IngestRefusal(reason)) => {
                    return Err(AutowebRefusal {
                        stage: fetch_stage,
                        reason,
                    })
                }
            };
            let downloaded = if from_cache { 0 } else { bytes.len() };
            if representation == Representation::Descriptor {
                self.trace.descriptor_bytes_downloaded += downloaded;
            } else {
                self.trace.document_bytes_downloaded += downloaded;
            }
            if from_cache {
                self.trace.cache_reused = true;
            }
            return Ok((bytes, url.https_url()));
        }
    }

    fn trace_if_none_match(&self, request: &TransportRequest) -> Option<String> {
        request
            .headers
            .iter()
            .find(|(k, _)| *k == "If-None-Match")
            .map(|(_, v)| v.clone())
    }

    /// 一次只读导航（地址已由调用方解析；描述基准=描述最终 URL）。
    pub fn navigate(&mut self, address: &WebAddress) -> NavOutcome {
        match self.navigate_inner(address) {
            Ok(success) => NavOutcome::Success(success),
            Err(refusal) => {
                let trace = std::mem::take(&mut self.trace);
                NavOutcome::Refused(refusal, trace)
            }
        }
    }

    fn navigate_inner(&mut self, address: &WebAddress) -> Result<NavSuccess, AutowebRefusal> {
        self.check_alive()?;
        use sha2::{Digest, Sha256};
        let sha_hex_of = |bytes: &[u8]| -> String {
            let digest: [u8; 32] = Sha256::new().chain_update(bytes).finalize().into();
            digest.iter().map(|b| format!("{b:02x}")).collect()
        };
        // 1. 描述（规范 https 目标；别名已在地址层映射）。缓存准入验证=
        //    严格 v1 解析（基准=描述最终 URL）。
        let https = WebAddress {
            spelling: AddressSpelling::Https,
            host: address.host.clone(),
            port: address.port,
            path: address.path.clone(),
            query: address.query.clone(),
        };
        let (descriptor_bytes, descriptor_url) = self.fetch_resource(
            https,
            Representation::Descriptor,
            DESCRIPTOR_MEDIA,
            self.limits.descriptor_bytes,
            &|url, body| {
                parse_descriptor(body, url).map(|_| ()).map_err(|mut r| {
                    if r.stage == NavStage::DescriptorParse {
                        r.reason = format!("descriptor rejected before caching: {}", r.reason);
                    }
                    r
                })
            },
        )?;
        let descriptor_base = parse_web_address(&descriptor_url).map_err(|e| AutowebRefusal {
            stage: NavStage::DescriptorParse,
            reason: format!("final descriptor URL is not representable: {e}"),
        })?;
        let descriptor = parse_descriptor(&descriptor_bytes, &descriptor_base)?;
        // 2. 入口文档（描述声明的绝对同源 URL）。缓存准入验证=sha256+
        //    UTF-8+只读准入——未验证完整的内容不进入可复用缓存。
        let entry = parse_web_address(&descriptor.entry_url).map_err(|e| AutowebRefusal {
            stage: NavStage::DocumentFetch,
            reason: format!("entry URL is not representable: {e}"),
        })?;
        let declared_sha = descriptor.entry_sha256;
        let (document_bytes, _final_url) = self.fetch_resource(
            entry,
            Representation::Document,
            DOCUMENT_MEDIA,
            self.limits.document_bytes,
            &move |_url, body| {
                let digest: [u8; 32] = Sha256::new().chain_update(body).finalize().into();
                if digest != declared_sha {
                    return Err(AutowebRefusal {
                        stage: NavStage::DocumentIntegrity,
                        reason: format!(
                            "document sha256 mismatch: descriptor declares {}, bytes hash to {}",
                            hex(&declared_sha),
                            hex(&digest)
                        ),
                    });
                }
                let text = std::str::from_utf8(body).map_err(|e| AutowebRefusal {
                    stage: NavStage::DocumentAdmission,
                    reason: format!("document is not valid UTF-8: {e}"),
                })?;
                admit_readonly_document(text)
            },
        )?;
        let document = String::from_utf8(document_bytes.clone()).map_err(|e| AutowebRefusal {
            stage: NavStage::DocumentAdmission,
            reason: format!("document is not valid UTF-8: {e}"),
        })?;
        let document_sha256_hex = sha_hex_of(&document_bytes);
        Ok(NavSuccess {
            display_address: address.display_address(),
            canonical_url: address.https_url(),
            title: descriptor.title.clone(),
            release: descriptor.release.clone(),
            app_id: descriptor.app_id.clone(),
            document,
            document_sha256_hex,
            trace: std::mem::take(&mut self.trace),
        })
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// 字节 SHA-256（小写 hex；测试/证据面与协议层同源）。
pub fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let digest: [u8; 32] = Sha256::new().chain_update(bytes).finalize().into();
    hex(&digest)
}

fn key_for(url: WebAddress, representation: Representation) -> CacheKey {
    CacheKey {
        url: url.https_url(),
        representation,
    }
}
