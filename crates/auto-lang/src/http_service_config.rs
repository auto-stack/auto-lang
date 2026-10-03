//! PLAN-736: HTTP 服务部署配置——版本中立层（无 axum/hyper/VM 依赖）。
//!
//! 两轨共同消费（VM 轨 axum 0.8 / 生成 Rust 轨 axum 0.7 各自装配自己的
//! 连接驱动与 Response 类型；本模块只提供可共享的纯数据与纯策略）：
//!
//! - 配置解析：严格 JSON（未知字段/坏值/非法组合指名失败，非零退出）
//! - profile 缺省（development / proxy_service）+ 每字段来源记录
//! - 合法范围冻结（决策报告 §3.1；0 永不表示无限）
//! - 纯策略：CORS 决策（preflight/实际请求）、Host 允许表、单层可信代理身份
//! - 有界限速器（桶容量 + TTL 回收，满表新身份保守 429）
//!
//! effective_config hash 与 734 的 generation/implementation hash 语义分开：
//! 本 hash 只标识"解析后的服务运行配置"，不冒称业务实现新鲜度。
//!
//! 契约：[docs/specs/stdlib/design/http-service-deployment.md]（SD-01，T-08 沉淀）、
//! 决策报告 [docs/plans/reports/736-http-decision.md](../docs/plans/reports/736-http-decision.md)。

use serde::Deserialize;
use std::collections::HashMap;
use std::fmt;
use std::net::IpAddr;
use std::sync::{Arc, Mutex, OnceLock};

// ============================================================================
// Raw JSON schema（serde 层——Option 全开放，语义校验在 resolve）
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawServiceConfig {
    schema_version: Option<u32>,
    profile: Option<String>,
    listen: Option<RawListen>,
    limits: Option<RawLimits>,
    cors: Option<RawCors>,
    allowed_hosts: Option<Vec<String>>,
    trusted_proxy_ips: Option<Vec<String>>,
    auth: Option<RawAuth>,
    rate_limit: Option<RawRateLimit>,
    features: Option<RawFeatures>,
    observability: Option<RawObservability>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawListen {
    addr: Option<String>,
    port: Option<u16>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawLimits {
    max_connections: Option<usize>,
    max_inflight_requests: Option<usize>,
    header_buf_bytes: Option<usize>,
    header_read_timeout_ms: Option<u64>,
    body_limit_bytes: Option<usize>,
    body_timeout_ms: Option<u64>,
    request_timeout_ms: Option<u64>,
    drain_timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCors {
    allowed_origins: Option<Vec<String>>,
    allowed_methods: Option<Vec<String>>,
    allowed_headers: Option<Vec<String>>,
    exposed_headers: Option<Vec<String>>,
    allow_credentials: Option<bool>,
    max_age_seconds: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawAuth {
    responsibility: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRateLimit {
    max_requests: u32,
    window_ms: u64,
    bucket_capacity: Option<usize>,
    bucket_ttl_ms: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawFeatures {
    websocket_echo: Option<bool>,
    media_scan: Option<bool>,
    photo_scan: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawObservability {
    log_sink_capacity: Option<usize>,
}

// ============================================================================
// Resolved config
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceProfile {
    Development,
    ProxyService,
}

impl ServiceProfile {
    pub fn as_str(self) -> &'static str {
        match self {
            ServiceProfile::Development => "development",
            ServiceProfile::ProxyService => "proxy_service",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthResponsibility {
    App,
    Edge,
}

impl AuthResponsibility {
    pub fn as_str(self) -> &'static str {
        match self {
            AuthResponsibility::App => "app",
            AuthResponsibility::Edge => "edge",
        }
    }
}

/// 普通请求/连接预算（决策报告 §3.1 冻结范围；file/upload/SSE 期限归 729/730
/// 专属契约，不经此表）。
#[derive(Debug, Clone, Copy)]
pub struct ServiceLimits {
    pub max_connections: usize,
    pub max_inflight_requests: usize,
    pub header_buf_bytes: usize,
    pub header_read_timeout_ms: u64,
    pub body_limit_bytes: usize,
    pub body_timeout_ms: u64,
    pub request_timeout_ms: u64,
    pub drain_timeout_ms: u64,
}

#[derive(Debug, Clone)]
pub struct CorsPolicy {
    /// 精确 origin 列表，或唯一 "*"（development 缺省）。
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
    pub exposed_headers: Vec<String>,
    /// 本期恒 false；true = 配置错误（`*`+credentials 组合在解析期拒绝）。
    pub allow_credentials: bool,
    pub max_age_seconds: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct RateLimitConfig {
    pub max_requests: u32,
    pub window_ms: u64,
    pub bucket_capacity: usize,
    pub bucket_ttl_ms: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct ServiceFeatures {
    /// 简化 echo WS（既有行为）；proxy_service 缺省关闭。
    pub websocket_echo: bool,
    pub media_scan: bool,
    pub photo_scan: bool,
}

#[derive(Debug, Clone)]
pub struct ObservabilityConfig {
    pub log_sink_capacity: usize,
    pub health_request_budget: usize,
}

/// 解析后的不可变服务配置（两轨共同消费的唯一事实源）。
#[derive(Debug, Clone)]
pub struct HttpServiceConfig {
    pub profile: ServiceProfile,
    pub listen_addr: IpAddr,
    pub listen_port: u16,
    pub limits: ServiceLimits,
    pub cors: CorsPolicy,
    pub allowed_hosts: Vec<String>,
    pub trusted_proxy_ips: Vec<IpAddr>,
    pub auth_responsibility: AuthResponsibility,
    pub rate_limit: Option<RateLimitConfig>,
    pub features: ServiceFeatures,
    pub observability: ObservabilityConfig,
    /// 每个生效值的来源：`config_default` / `profile_default` / `explicit` /
    /// `cli_override`（键为点分字段路径）。
    pub sources: Vec<(String, &'static str)>,
}

impl HttpServiceConfig {
    /// FNV-1a 64 over the canonical JSON form（不含 sources）。
    /// 与 734 generation/implementation hash 分开——只标识服务运行配置。
    pub fn effective_config_hash(&self) -> u64 {
        let canonical = self.canonical_json();
        let mut hash: u64 = 0xcbf29ce484222325;
        for b in canonical.as_bytes() {
            hash ^= *b as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash
    }

    /// 规范 JSON 形（全字段显式）——进程间传递解析后配置的唯一形态
    /// （`auto serve` 父进程 resolve → env `AUTO_HTTP_SERVICE_JSON` → 生成
    /// 子进程 re-resolve；两端 hash 必须一致）。
    pub fn to_json(&self) -> String {
        use serde_json::json;
        let lim = &self.limits;
        let cors = &self.cors;
        let mut root = json!({
            "schema_version": 1,
            "profile": self.profile.as_str(),
            "listen": { "addr": self.listen_addr.to_string(), "port": self.listen_port },
            "limits": {
                "max_connections": lim.max_connections,
                "max_inflight_requests": lim.max_inflight_requests,
                "header_buf_bytes": lim.header_buf_bytes,
                "header_read_timeout_ms": lim.header_read_timeout_ms,
                "body_limit_bytes": lim.body_limit_bytes,
                "body_timeout_ms": lim.body_timeout_ms,
                "request_timeout_ms": lim.request_timeout_ms,
                "drain_timeout_ms": lim.drain_timeout_ms,
            },
            "cors": {
                "allowed_origins": cors.allowed_origins,
                "allowed_methods": cors.allowed_methods,
                "allowed_headers": cors.allowed_headers,
                "exposed_headers": cors.exposed_headers,
                "allow_credentials": cors.allow_credentials,
                "max_age_seconds": cors.max_age_seconds,
            },
            "allowed_hosts": self.allowed_hosts,
            "trusted_proxy_ips": self.trusted_proxy_ips.iter().map(|i| i.to_string()).collect::<Vec<_>>(),
            "auth": { "responsibility": self.auth_responsibility.as_str() },
            "features": {
                "websocket_echo": self.features.websocket_echo,
                "media_scan": self.features.media_scan,
                "photo_scan": self.features.photo_scan,
            },
            "observability": { "log_sink_capacity": self.observability.log_sink_capacity },
        });
        if let Some(rl) = &self.rate_limit {
            root["rate_limit"] = json!({
                "max_requests": rl.max_requests,
                "window_ms": rl.window_ms,
                "bucket_capacity": rl.bucket_capacity,
                "bucket_ttl_ms": rl.bucket_ttl_ms,
            });
        }
        root.to_string()
    }

    fn canonical_json(&self) -> String {
        // 手写稳定序列化（字段顺序固定；不用 serde_json::Value 以避免 map 序）。
        let mut s = String::with_capacity(512);
        let lim = &self.limits;
        s.push_str(&format!("profile={};", self.profile.as_str()));
        s.push_str(&format!("listen={}:{};", self.listen_addr, self.listen_port));
        s.push_str(&format!(
            "limits={},{},{},{},{},{},{},{};",
            lim.max_connections,
            lim.max_inflight_requests,
            lim.header_buf_bytes,
            lim.header_read_timeout_ms,
            lim.body_limit_bytes,
            lim.body_timeout_ms,
            lim.request_timeout_ms,
            lim.drain_timeout_ms
        ));
        s.push_str(&format!("origins={:?};", self.cors.allowed_origins));
        s.push_str(&format!("methods={:?};", self.cors.allowed_methods));
        s.push_str(&format!("headers={:?};", self.cors.allowed_headers));
        s.push_str(&format!("exposed={:?};", self.cors.exposed_headers));
        s.push_str(&format!("creds={};maxage={};", self.cors.allow_credentials, self.cors.max_age_seconds));
        s.push_str(&format!("hosts={:?};", self.allowed_hosts));
        s.push_str(&format!("proxies={:?};", self.trusted_proxy_ips));
        s.push_str(&format!("auth={};", self.auth_responsibility.as_str()));
        if let Some(rl) = &self.rate_limit {
            s.push_str(&format!(
                "ratelimit={},{},{},{};",
                rl.max_requests, rl.window_ms, rl.bucket_capacity, rl.bucket_ttl_ms
            ));
        }
        s.push_str(&format!(
            "features={},{},{};",
            self.features.websocket_echo, self.features.media_scan, self.features.photo_scan
        ));
        s.push_str(&format!(
            "obs={},{};",
            self.observability.log_sink_capacity, self.observability.health_request_budget
        ));
        s
    }
}

// ============================================================================
// Errors
// ============================================================================

#[derive(Debug, Clone)]
pub struct ConfigError {
    pub field: String,
    pub reason: String,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "http-service config error at `{}`: {}", self.field, self.reason)
    }
}

impl std::error::Error for ConfigError {}

fn err(field: &str, reason: impl Into<String>) -> ConfigError {
    ConfigError { field: field.to_string(), reason: reason.into() }
}

// ============================================================================
// Resolve：raw JSON → validated immutable config
// ============================================================================

/// 冻结范围（决策报告 §3.1 表；越界=指名错误）。
mod range {
    pub const MAX_CONNECTIONS: (usize, usize) = (1, 4096);
    pub const MAX_INFLIGHT: (usize, usize) = (1, 1024);
    pub const HEADER_BUF: (usize, usize) = (8192, 1_048_576);
    pub const HEADER_READ_TIMEOUT_MS: (u64, u64) = (1_000, 60_000);
    pub const BODY_LIMIT: (usize, usize) = (1_024, 104_857_600);
    pub const BODY_TIMEOUT_MS: (u64, u64) = (1_000, 300_000);
    pub const REQUEST_TIMEOUT_MS: (u64, u64) = (1_000, 600_000);
    pub const DRAIN_TIMEOUT_MS: (u64, u64) = (100, 60_000);
    pub const LOG_SINK_CAPACITY: (usize, usize) = (64, 65_536);
    pub const HEALTH_BUDGET: (usize, usize) = (1, 64);
    pub const BUCKET_CAPACITY: (usize, usize) = (1, 65_536);
    pub const BUCKET_TTL_MS: (u64, u64) = (1_000, 3_600_000);
}

fn check_range_usize(field: &str, v: usize, (lo, hi): (usize, usize)) -> Result<(), ConfigError> {
    if v < lo || v > hi {
        Err(err(field, format!("{v} out of legal range [{lo}, {hi}]")))
    } else {
        Ok(())
    }
}

fn check_range_u64(field: &str, v: u64, (lo, hi): (u64, u64)) -> Result<(), ConfigError> {
    if v < lo || v > hi {
        Err(err(field, format!("{v} out of legal range [{lo}, {hi}]")))
    } else {
        Ok(())
    }
}

const DEV_CORS_METHODS: &[&str] = &["GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS"];
const DEV_CORS_HEADERS: &[&str] = &["Content-Type", "Authorization"];
const SERVICE_CORS_METHODS: &[&str] = &["GET", "POST", "PUT", "DELETE", "PATCH"];

/// 解析并校验服务配置。
///
/// `cli_port_override`：`auto serve -B <port>` 显式传入（覆盖 `listen.port`，
/// 来源 `cli_override`）；`None` = 未显式传。
pub fn resolve_service_config(
    json: &str,
    cli_port_override: Option<u16>,
) -> Result<HttpServiceConfig, ConfigError> {
    let raw: RawServiceConfig = serde_json::from_str(json)
        .map_err(|e| err("(root)", format!("invalid JSON or unknown field: {e}")))?;

    let sources: std::cell::RefCell<Vec<(String, &'static str)>> =
        std::cell::RefCell::new(Vec::new());
    let mark = |field: &str, explicit: bool| {
        sources.borrow_mut().push((
            field.to_string(),
            if explicit { "explicit" } else { "profile_default" },
        ));
    };

    // ---- schema_version ----
    match raw.schema_version {
        None => {}
        Some(1) => mark("schema_version", true),
        Some(other) => return Err(err("schema_version", format!("unsupported version {other}; this build understands 1 only"))),
    }

    // ---- profile（必填性按有无显式配置面区分：文件/inline 都必须写）----
    let profile = match raw.profile.as_deref() {
        Some("development") => ServiceProfile::Development,
        Some("proxy_service") => ServiceProfile::ProxyService,
        Some(other) => {
            return Err(err("profile", format!("unknown profile `{other}`; expected `development` or `proxy_service`")))
        }
        None => return Err(err("profile", "profile is required (`development` or `proxy_service`)")),
    };
    let is_service = profile == ServiceProfile::ProxyService;

    // ---- listen ----
    let dev_default_port: u16 = 8080;
    let (listen_addr, listen_port) = match &raw.listen {
        None => {
            mark("listen.addr", false);
            mark("listen.port", false);
            (IpAddr::from([127, 0, 0, 1]), dev_default_port)
        }
        Some(l) => {
            let addr = match &l.addr {
                Some(a) => {
                    let ip: IpAddr = a.parse().map_err(|_| err("listen.addr", format!("`{a}` is not a valid IP address (explicit non-loopback listening must be an explicit address; hostnames are not resolved)")))?;
                    mark("listen.addr", true);
                    ip
                }
                None => {
                    mark("listen.addr", false);
                    IpAddr::from([127, 0, 0, 1])
                }
            };
            let port = match l.port {
                Some(p) => {
                    mark("listen.port", true);
                    p
                }
                None => {
                    mark("listen.port", false);
                    dev_default_port
                }
            };
            (addr, port)
        }
    };
    // CLI 显式 -B 覆盖 config 端口（决策 §3.1：仅显式 CLI 旗标可覆盖）。
    let listen_port = match cli_port_override {
        Some(p) => {
            sources.borrow_mut().push(("listen.port".to_string(), "cli_override"));
            p
        }
        None => listen_port,
    };
    if listen_port == 0 {
        // port=0 仅独立 serve/test：ready 报真实 bound 地址（装配层负责）。
        let was_explicit = sources
            .borrow()
            .iter()
            .any(|(f, s)| f == "listen.port" && *s == "explicit");
        mark("listen.port", was_explicit);
    }

    // ---- limits ----
    let (dev_conn, svc_conn) = (128usize, 128usize);
    let limits = {
        let l = raw.limits.clone();
        let mut take = |field: &str, v: Option<usize>, dev: usize, svc: usize| -> Result<usize, ConfigError> {
            let (val, explicit) = match v {
                Some(v) => (v, true),
                None => (if is_service { svc } else { dev }, false),
            };
            check_range_usize(field, val, range_for_usize(field))?;
            mark(field, explicit);
            Ok(val)
        };
        let mut take_u64 = |field: &str, v: Option<u64>, dev: u64, svc: u64| -> Result<u64, ConfigError> {
            let (val, explicit) = match v {
                Some(v) => (v, true),
                None => (if is_service { svc } else { dev }, false),
            };
            check_range_u64(field, val, range_for_u64(field))?;
            mark(field, explicit);
            Ok(val)
        };
        // 数值相同/不同按冻结表：
        ServiceLimits {
            max_connections: take("limits.max_connections", l.as_ref().and_then(|x| x.max_connections), dev_conn, svc_conn)?,
            max_inflight_requests: take("limits.max_inflight_requests", l.as_ref().and_then(|x| x.max_inflight_requests), 64, 64)?,
            header_buf_bytes: take("limits.header_buf_bytes", l.as_ref().and_then(|x| x.header_buf_bytes), 65_536, 65_536)?,
            header_read_timeout_ms: take_u64("limits.header_read_timeout_ms", l.as_ref().and_then(|x| x.header_read_timeout_ms), 10_000, 10_000)?,
            body_limit_bytes: take("limits.body_limit_bytes", l.as_ref().and_then(|x| x.body_limit_bytes), 10_485_760, 10_485_760)?,
            body_timeout_ms: take_u64("limits.body_timeout_ms", l.as_ref().and_then(|x| x.body_timeout_ms), 10_000, 10_000)?,
            request_timeout_ms: take_u64("limits.request_timeout_ms", l.as_ref().and_then(|x| x.request_timeout_ms), 30_000, 30_000)?,
            drain_timeout_ms: take_u64("limits.drain_timeout_ms", l.as_ref().and_then(|x| x.drain_timeout_ms), 10_000, 10_000)?,
        }
    };

    // ---- cors ----
    let cors = {
        let c = raw.cors.clone();
        let origins: Vec<String> = match c.as_ref().and_then(|x| x.allowed_origins.clone()) {
            Some(o) => {
                mark("cors.allowed_origins", true);
                if o.len() > 1 && o.iter().any(|x| x == "*") {
                    return Err(err("cors.allowed_origins", "`*` cannot be combined with explicit origins"));
                }
                for origin in &o {
                    if origin != "*" && (origin.contains('*') || !origin.starts_with("http://") && !origin.starts_with("https://")) {
                        return Err(err("cors.allowed_origins", format!("`{origin}` is not an exact origin (scheme://host[:port]) or the single `*`")));
                    }
                }
                o
            }
            None => {
                mark("cors.allowed_origins", false);
                if is_service {
                    Vec::new() // service：缺省不授权任何跨域
                } else {
                    vec!["*".to_string()] // development：legacy `*`
                }
            }
        };
        let credentials = match c.as_ref().and_then(|x| x.allow_credentials) {
            Some(true) => return Err(err("cors.allow_credentials", "credential CORS is out of scope for PLAN-736 (constant false)")),
            _ => {
                mark("cors.allow_credentials", false);
                false
            }
        };
        let mut take_list = |field: &str, v: Option<Vec<String>>, dev: &[&str], svc: &[&str]| -> Vec<String> {
            match v {
                Some(list) => {
                    mark(field, true);
                    list
                }
                None => {
                    mark(field, false);
                    (if is_service { svc } else { dev }).iter().map(|s| s.to_string()).collect()
                }
            }
        };
        let methods = take_list("cors.allowed_methods", c.as_ref().and_then(|x| x.allowed_methods.clone()), DEV_CORS_METHODS, SERVICE_CORS_METHODS);
        let headers = take_list("cors.allowed_headers", c.as_ref().and_then(|x| x.allowed_headers.clone()), DEV_CORS_HEADERS, DEV_CORS_HEADERS);
        let exposed = take_list("cors.exposed_headers", c.as_ref().and_then(|x| x.exposed_headers.clone()), &[], &[]);
        let max_age = match c.as_ref().and_then(|x| x.max_age_seconds) {
            Some(v) => {
                mark("cors.max_age_seconds", true);
                v
            }
            None => {
                mark("cors.max_age_seconds", false);
                86400
            }
        };
        CorsPolicy {
            allowed_origins: origins,
            allowed_methods: methods,
            allowed_headers: headers,
            exposed_headers: exposed,
            allow_credentials: credentials,
            max_age_seconds: max_age,
        }
    };

    // ---- allowed_hosts ----
    let allowed_hosts = match raw.allowed_hosts.clone() {
        Some(h) => {
            mark("allowed_hosts", true);
            if h.is_empty() && is_service {
                return Err(err("allowed_hosts", "proxy_service profile requires a non-empty allowed_hosts list"));
            }
            h.into_iter().map(|x| x.to_ascii_lowercase()).collect()
        }
        None => {
            mark("allowed_hosts", false);
            Vec::new() // development：不校验 Host
        }
    };

    // ---- trusted_proxy_ips ----
    let trusted_proxy_ips = match raw.trusted_proxy_ips.clone() {
        Some(list) => {
            mark("trusted_proxy_ips", true);
            let mut ips = Vec::with_capacity(list.len());
            for s in &list {
                let ip: IpAddr = s.parse().map_err(|_| err("trusted_proxy_ips", format!("`{s}` is not a numeric IP address (no DNS resolution in PLAN-736)")))?;
                ips.push(ip);
            }
            ips
        }
        None => {
            mark("trusted_proxy_ips", false);
            Vec::new()
        }
    };

    // ---- auth ----
    let auth_responsibility = match raw.auth.as_ref().and_then(|a| a.responsibility.as_deref()) {
        Some("app") => {
            mark("auth.responsibility", true);
            AuthResponsibility::App
        }
        Some("edge") => {
            mark("auth.responsibility", true);
            AuthResponsibility::Edge
        }
        Some(other) => return Err(err("auth.responsibility", format!("unknown responsibility `{other}`; expected `app` or `edge`"))),
        None if is_service => return Err(err("auth.responsibility", "proxy_service profile requires an explicit auth responsibility (`app` or `edge`)")),
        None => {
            mark("auth.responsibility", false);
            AuthResponsibility::App
        }
    };

    // ---- rate_limit ----
    let rate_limit = match raw.rate_limit.clone() {
        Some(rl) => {
            mark("rate_limit", true);
            let window = rl.window_ms;
            check_range_u64("rate_limit.window_ms", window, (100, 3_600_000))?;
            let capacity = rl.bucket_capacity.unwrap_or(4096);
            check_range_usize("rate_limit.bucket_capacity", capacity, range::BUCKET_CAPACITY)?;
            let ttl = rl.bucket_ttl_ms.unwrap_or(60_000);
            check_range_u64("rate_limit.bucket_ttl_ms", ttl, range::BUCKET_TTL_MS)?;
            if rl.max_requests == 0 {
                return Err(err("rate_limit.max_requests", "0 is not legal (no implicit unbounded mode)"));
            }
            Some(RateLimitConfig {
                max_requests: rl.max_requests,
                window_ms: window,
                bucket_capacity: capacity,
                bucket_ttl_ms: ttl,
            })
        }
        None => None,
    };

    // ---- features ----
    let features = {
        let f = raw.features.clone();
        let mut take = |field_end: &str, v: Option<bool>, dev: bool, svc: bool| -> bool {
            match v {
                Some(b) => {
                    mark(&format!("features.{field_end}"), true);
                    b
                }
                None => {
                    mark(&format!("features.{field_end}"), false);
                    if is_service { svc } else { dev }
                }
            }
        };
        ServiceFeatures {
            websocket_echo: take("websocket_echo", f.as_ref().and_then(|x| x.websocket_echo), true, false),
            media_scan: take("media_scan", f.as_ref().and_then(|x| x.media_scan), true, false),
            photo_scan: take("photo_scan", f.as_ref().and_then(|x| x.photo_scan), true, false),
        }
    };

    // ---- observability ----
    let observability = {
        let o = raw.observability.clone();
        let sink = match o.as_ref().and_then(|x| x.log_sink_capacity) {
            Some(v) => {
                check_range_usize("observability.log_sink_capacity", v, range::LOG_SINK_CAPACITY)?;
                mark("observability.log_sink_capacity", true);
                v
            }
            None => {
                mark("observability.log_sink_capacity", false);
                1024
            }
        };
        let health_budget = 8usize; // 决策 §3.1：v1 不从文件配置（固定 8），范围见 range::HEALTH_BUDGET
        ObservabilityConfig { log_sink_capacity: sink, health_request_budget: health_budget }
    };

    Ok(HttpServiceConfig {
        profile,
        listen_addr,
        listen_port,
        limits,
        cors,
        allowed_hosts,
        trusted_proxy_ips,
        auth_responsibility,
        rate_limit,
        features,
        observability,
        sources: sources.into_inner(),
    })
}

fn range_for_usize(field: &str) -> (usize, usize) {
    match field {
        "limits.max_connections" => range::MAX_CONNECTIONS,
        "limits.max_inflight_requests" => range::MAX_INFLIGHT,
        "limits.header_buf_bytes" => range::HEADER_BUF,
        "limits.body_limit_bytes" => range::BODY_LIMIT,
        _ => (1, usize::MAX),
    }
}

fn range_for_u64(field: &str) -> (u64, u64) {
    match field {
        "limits.header_read_timeout_ms" => range::HEADER_READ_TIMEOUT_MS,
        "limits.body_timeout_ms" => range::BODY_TIMEOUT_MS,
        "limits.request_timeout_ms" => range::REQUEST_TIMEOUT_MS,
        "limits.drain_timeout_ms" => range::DRAIN_TIMEOUT_MS,
        _ => (1, u64::MAX),
    }
}

// ============================================================================
// 纯策略：CORS / Host / 代理身份
// ============================================================================

fn origin_allowed(cors: &CorsPolicy, origin: &str) -> bool {
    if cors.allowed_origins.iter().any(|o| o == "*") {
        return true;
    }
    cors.allowed_origins.iter().any(|o| o.eq_ignore_ascii_case(origin))
}

/// 实际请求（非 preflight）的 CORS 决策。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CorsOutcome {
    /// 允许：附带的 ACAO 值与是否加 `Vary: Origin`。
    Allowed { allow_origin: String, vary_origin: bool },
    /// 不允许：不产生任何 CORS 头（响应照常走业务语义）。
    Absent,
}

impl CorsPolicy {
    pub fn actual_request(&self, origin: Option<&str>) -> CorsOutcome {
        let Some(origin) = origin else {
            return CorsOutcome::Absent; // 无 Origin：同源/非浏览器，不加头
        };
        if !origin_allowed(self, origin) {
            return CorsOutcome::Absent;
        }
        if self.allowed_origins.iter().any(|o| o == "*") {
            CorsOutcome::Allowed { allow_origin: "*".to_string(), vary_origin: false }
        } else {
            CorsOutcome::Allowed { allow_origin: origin.to_string(), vary_origin: true }
        }
    }

    /// preflight 判定：返回 (status, allow_origin, vary)。
    /// 204=允许；403=拒绝（origin/方法/请求头任一不命中；不附 CORS 头）。
    pub fn preflight(
        &self,
        origin: Option<&str>,
        request_method: Option<&str>,
        request_headers: &[String],
    ) -> (u16, Option<String>, bool) {
        let Some(origin) = origin else {
            return (403, None, false);
        };
        if !origin_allowed(self, origin) {
            return (403, None, false);
        }
        let Some(method) = request_method else {
            return (403, None, false);
        };
        if !self.allowed_methods.iter().any(|m| m.eq_ignore_ascii_case(method)) {
            return (403, None, false);
        }
        for h in request_headers {
            if !self.allowed_headers.iter().any(|a| a.eq_ignore_ascii_case(h)) {
                return (403, None, false);
            }
        }
        let vary = !self.allowed_origins.iter().any(|o| o == "*");
        let acao = if vary {
            origin.to_string()
        } else {
            "*".to_string() // wildcard 语义统一：ACAO=*（与 actual_request 一致）
        };
        (204, Some(acao), vary)
    }
}

impl HttpServiceConfig {
    /// Host 允许表（decision §3.5：hostname-only、大小写不敏感、精确匹配；
    /// 空表 = 不校验（development）。**不**读 X-Forwarded-Host。
    pub fn host_allowed(&self, authority: Option<&str>) -> bool {
        if self.allowed_hosts.is_empty() {
            return true;
        }
        let Some(host) = authority else {
            return false;
        };
        // Host/authority 可能带端口：取 hostname 段（IPv6 [::1]:8080 形态处理）。
        let hostname = normalize_hostname(host);
        self.allowed_hosts.iter().any(|h| {
            let h = normalize_hostname(h);
            h == hostname
        })
    }

    /// 单层可信代理身份（decision §3.5）：
    /// 仅 socket peer 精确命中 trusted 列表时，消费**单个**重写 XFF IP；
    /// 多段/坏值/空 = 拒绝该头（按 peer 记身份，`xff_rejected=true`）。
    /// X-Forwarded-Proto 仅在经可信代理时消费（http|https）。
    pub fn effective_client_identity(
        &self,
        peer: IpAddr,
        x_forwarded_for: Option<&str>,
        x_forwarded_proto: Option<&str>,
    ) -> (EffectiveClient, Option<String>) {
        let trusted = self.trusted_proxy_ips.iter().any(|ip| *ip == peer);
        if !trusted {
            return (
                EffectiveClient { client: peer, via_trusted_proxy: false, xff_rejected: false },
                None,
            );
        }
        let Some(xff) = x_forwarded_for.map(str::trim) else {
            return (
                EffectiveClient { client: peer, via_trusted_proxy: true, xff_rejected: false },
                None,
            );
        };
        // 单一重写 IP 语义：不允许链（多段=歧义拒绝）。
        if xff.is_empty() || xff.contains(',') {
            return (
                EffectiveClient { client: peer, via_trusted_proxy: true, xff_rejected: true },
                None,
            );
        }
        match xff.parse::<IpAddr>() {
            Ok(ip) => {
                let proto = x_forwarded_proto
                    .map(str::trim)
                    .filter(|p| *p == "http" || *p == "https")
                    .map(str::to_string);
                (
                    EffectiveClient { client: ip, via_trusted_proxy: true, xff_rejected: false },
                    proto,
                )
            }
            Err(_) => (
                EffectiveClient { client: peer, via_trusted_proxy: true, xff_rejected: true },
                None,
            ),
        }
    }
}

fn normalize_hostname(host: &str) -> String {
    let h = host.trim();
    let h = h.trim_end_matches('.');
    if let Some(rest) = h.strip_prefix('[') {
        // [::1]:8080 → ::1
        if let Some(end) = rest.find(']') {
            return rest[..end].to_ascii_lowercase();
        }
    }
    // host:port → host（IPv4/域名；IPv6 裸地址含冒号的情形仅来自 config 侧
    // 规范写法 [..] 或裸 IP——裸 IPv6 也按最后冒号切分会错，先试整体解析）。
    if h.parse::<std::net::Ipv6Addr>().is_ok() {
        return h.to_ascii_lowercase();
    }
    match h.rsplit_once(':') {
        Some((host, port)) if port.chars().all(|c| c.is_ascii_digit()) && !port.is_empty() => {
            host.to_ascii_lowercase()
        }
        _ => h.to_ascii_lowercase(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectiveClient {
    pub client: IpAddr,
    pub via_trusted_proxy: bool,
    pub xff_rejected: bool,
}

// ============================================================================
// 有界限速器（decision §3.5/§3.1：桶容量 + TTL；满表新身份保守 429）
// ============================================================================

pub enum RateDecision {
    Allow,
    Limited { retry_after_secs: u64 },
}

struct RateBucket {
    window_start_ms: u64,
    count: u32,
    last_seen_ms: u64,
}

pub struct RateLimiter {
    cfg: RateLimitConfig,
    buckets: Mutex<HashMap<IpAddr, RateBucket>>,
}

impl RateLimiter {
    pub fn new(cfg: RateLimitConfig) -> Self {
        Self { cfg, buckets: Mutex::new(HashMap::new()) }
    }

    pub fn config(&self) -> &RateLimitConfig {
        &self.cfg
    }

    pub fn active_buckets(&self) -> usize {
        self.buckets.lock().map(|b| b.len()).unwrap_or(0)
    }

    /// `now_ms` 由调用方供给（生成/VM 轨各自时钟），保持本模块无时钟依赖。
    pub fn check(&self, identity: IpAddr, now_ms: u64) -> RateDecision {
        let mut buckets = match self.buckets.lock() {
            Ok(b) => b,
            Err(_) => return RateDecision::Limited { retry_after_secs: self.cfg.window_ms / 1000 },
        };
        // TTL 回收：先按 last_seen 清理过期桶（有界表的核心约束）。
        if buckets.len() >= self.cfg.bucket_capacity {
            let ttl = self.cfg.bucket_ttl_ms;
            buckets.retain(|_, b| now_ms.saturating_sub(b.last_seen_ms) < ttl);
        }
        match buckets.get_mut(&identity) {
            Some(b) => {
                if now_ms.saturating_sub(b.window_start_ms) >= self.cfg.window_ms {
                    b.window_start_ms = now_ms;
                    b.count = 1;
                    b.last_seen_ms = now_ms;
                    return RateDecision::Allow;
                }
                if b.count >= self.cfg.max_requests {
                    let remaining = self
                        .cfg
                        .window_ms
                        .saturating_sub(now_ms.saturating_sub(b.window_start_ms));
                    return RateDecision::Limited { retry_after_secs: (remaining / 1000).max(1) };
                }
                b.count += 1;
                b.last_seen_ms = now_ms;
                RateDecision::Allow
            }
            None => {
                if buckets.len() >= self.cfg.bucket_capacity {
                    // 满表且无可回收：保守拒绝新身份（不绕过限速）。
                    return RateDecision::Limited { retry_after_secs: self.cfg.window_ms / 1000 };
                }
                buckets.insert(
                    identity,
                    RateBucket { window_start_ms: now_ms, count: 1, last_seen_ms: now_ms },
                );
                RateDecision::Allow
            }
        }
    }
}

// ============================================================================
// 服务运行面（配置 + 有界限速器实例）——网络层策略消费的单点
// ============================================================================

/// VM 轨网络层与生成轨 gate 层共同消费的运行时面：不可变配置 + 可变限速器。
/// 放进 TransportConfig / 生成静态量时以 `Arc` 持有。
pub struct ServiceRuntime {
    pub config: Arc<HttpServiceConfig>,
    pub limiter: Option<RateLimiter>,
}

impl std::fmt::Debug for ServiceRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServiceRuntime")
            .field("config_hash", &self.config.effective_config_hash())
            .field("rate_limited", &self.limiter.is_some())
            .finish()
    }
}

impl ServiceRuntime {
    pub fn new(config: Arc<HttpServiceConfig>) -> Self {
        let limiter = config.rate_limit.map(RateLimiter::new);
        Self { config, limiter }
    }

    pub fn now_ms() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }
}

// ============================================================================
// 服务策略激活旗标（dispatch legacy CORS 让位于桥层 per-origin 附件）
// ============================================================================

static SERVICE_POLICY_ACTIVE: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// serve 入口在启动服务前置位：dispatch 既有 legacy CORS 头块（`*`）停用，
/// CORS 附件由桥层按配置决策（T-04）。
pub fn set_service_policy_active() {
    SERVICE_POLICY_ACTIVE.store(true, std::sync::atomic::Ordering::SeqCst);
}

pub fn service_policy_active() -> bool {
    SERVICE_POLICY_ACTIVE.load(std::sync::atomic::Ordering::SeqCst)
}

// ============================================================================
// 进程内活动配置 seam（VM 轨 serve 入口 → lib.rs 自动启动钩子）
// ============================================================================

static ACTIVE_SERVICE_CONFIG: OnceLock<Mutex<Option<Arc<HttpServiceConfig>>>> = OnceLock::new();

fn active_slot() -> &'static Mutex<Option<Arc<HttpServiceConfig>>> {
    ACTIVE_SERVICE_CONFIG.get_or_init(|| Mutex::new(None))
}

/// serve 入口在 spawn VM 线程**之前**安装；lib.rs 自动启动钩子 take 一次。
pub fn set_active_service_config(cfg: Arc<HttpServiceConfig>) {
    if let Ok(mut slot) = active_slot().lock() {
        *slot = Some(cfg);
    }
}

pub fn take_active_service_config() -> Option<Arc<HttpServiceConfig>> {
    active_slot().lock().ok().and_then(|mut s| s.take())
}

/// 由环境注入的配置 JSON 读取（生成轨/子进程形态）：
/// `AUTO_HTTP_SERVICE_JSON` 为完整配置文档；不存在 = legacy 面（返回 None）。
pub fn service_config_json_from_env() -> Option<String> {
    std::env::var("AUTO_HTTP_SERVICE_JSON").ok().filter(|s| !s.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEV_MINIMAL: &str = r#"{"profile":"development"}"#;
    const SVC_MINIMAL: &str = r#"{"profile":"proxy_service","allowed_hosts":["api.example.com"],"auth":{"responsibility":"edge"}}"#;

    #[test]
    fn dev_minimal_defaults_loopback_and_star() {
        let c = resolve_service_config(DEV_MINIMAL, None).unwrap();
        assert_eq!(c.profile, ServiceProfile::Development);
        assert_eq!(c.listen_addr, IpAddr::from([127, 0, 0, 1]));
        assert_eq!(c.listen_port, 8080);
        assert_eq!(c.cors.allowed_origins, vec!["*"]);
        assert!(c.allowed_hosts.is_empty());
        assert_eq!(c.auth_responsibility, AuthResponsibility::App);
        assert!(c.features.websocket_echo && c.features.media_scan && c.features.photo_scan);
        assert_eq!(c.limits.max_connections, 128);
        assert_eq!(c.limits.max_inflight_requests, 64);
    }

    #[test]
    fn service_profile_requires_hosts_and_auth() {
        let e = resolve_service_config(r#"{"profile":"proxy_service"}"#, None).unwrap_err();
        assert!(e.field == "allowed_hosts" || e.field == "auth.responsibility", "{e}");
        let e = resolve_service_config(
            r#"{"profile":"proxy_service","allowed_hosts":["a.com"]}"#,
            None,
        )
        .unwrap_err();
        assert_eq!(e.field, "auth.responsibility");
    }

    #[test]
    fn service_defaults_no_cross_origin_and_scan_off() {
        let c = resolve_service_config(SVC_MINIMAL, None).unwrap();
        assert!(c.cors.allowed_origins.is_empty());
        assert!(!c.features.media_scan && !c.features.photo_scan && !c.features.websocket_echo);
        assert_eq!(c.cors.allowed_methods, SERVICE_CORS_METHODS);
    }

    #[test]
    fn unknown_field_and_bad_profile_rejected() {
        let e = resolve_service_config(r#"{"profile":"development","nosuch":1}"#, None).unwrap_err();
        assert!(e.reason.contains("unknown field"), "{e}");
        let e = resolve_service_config(r#"{"profile":"prod"}"#, None).unwrap_err();
        assert_eq!(e.field, "profile");
        // 空 JSON 缺 profile 也拒。
        assert!(resolve_service_config("{}", None).is_err());
    }

    #[test]
    fn ranges_enforced_no_zero_unbounded() {
        let bad = |patch: &str| {
            let json = format!(r#"{{"profile":"development","limits":{patch}}}"#);
            resolve_service_config(&json, None).unwrap_err()
        };
        assert_eq!(bad(r#"{"max_connections":0}"#).field, "limits.max_connections");
        assert_eq!(bad(r#"{"max_connections":9999}"#).field, "limits.max_connections");
        assert_eq!(bad(r#"{"header_buf_bytes":1024}"#).field, "limits.header_buf_bytes");
        assert_eq!(bad(r#"{"drain_timeout_ms":50}"#).field, "limits.drain_timeout_ms");
        assert_eq!(bad(r#"{"request_timeout_ms":0}"#).field, "limits.request_timeout_ms");
        // 合法下界可用（低预算测试路径依赖）。
        let c = resolve_service_config(
            r#"{"profile":"development","limits":{"max_connections":1,"max_inflight_requests":1,"request_timeout_ms":1000}}"#,
            None,
        )
        .unwrap();
        assert_eq!(c.limits.max_connections, 1);
    }

    #[test]
    fn cli_port_override_and_explicit_listen() {
        let c = resolve_service_config(r#"{"profile":"development","listen":{"port":9000}}"#, Some(7777)).unwrap();
        assert_eq!(c.listen_port, 7777);
        assert!(c.sources.iter().any(|(f, s)| f == "listen.port" && *s == "cli_override"));
        let c = resolve_service_config(r#"{"profile":"development","listen":{"addr":"0.0.0.0","port":9000}}"#, None).unwrap();
        assert_eq!(c.listen_addr, IpAddr::from([0, 0, 0, 0]));
        assert_eq!(c.listen_port, 9000);
        // port=0（临时端口）合法。
        assert!(resolve_service_config(r#"{"profile":"development","listen":{"port":0}}"#, None).is_ok());
    }

    #[test]
    fn cors_star_rules() {
        // `*` 不可与精确列表混用。
        let e = resolve_service_config(
            r#"{"profile":"development","cors":{"allowed_origins":["*","http://a.com"]}}"#,
            None,
        )
        .unwrap_err();
        assert_eq!(e.field, "cors.allowed_origins");
        // 精确 origin 必须带 scheme。
        assert!(resolve_service_config(
            r#"{"profile":"development","cors":{"allowed_origins":["a.com"]}}"#,
            None
        )
        .is_err());
        // credentials 恒 false。
        assert!(resolve_service_config(
            r#"{"profile":"development","cors":{"allow_credentials":true}}"#,
            None
        )
        .is_err());
    }

    #[test]
    fn cors_policy_decisions() {
        let exact = CorsPolicy {
            allowed_origins: vec!["http://a.com".into(), "https://b.com:8443".into()],
            allowed_methods: vec!["GET".into(), "POST".into()],
            allowed_headers: vec!["Content-Type".into()],
            exposed_headers: vec![],
            allow_credentials: false,
            max_age_seconds: 600,
        };
        // 命中 origin → ACAO=origin + Vary。
        assert_eq!(
            exact.actual_request(Some("http://a.com")),
            CorsOutcome::Allowed { allow_origin: "http://a.com".into(), vary_origin: true }
        );
        // 不命中 → 无 CORS 头（不是拒绝响应）。
        assert_eq!(exact.actual_request(Some("http://evil.com")), CorsOutcome::Absent);
        assert_eq!(exact.actual_request(None), CorsOutcome::Absent);
        // preflight 三要素。
        let ok = exact.preflight(Some("http://a.com"), Some("POST"), &["content-type".to_string()]);
        assert_eq!(ok, (204, Some("http://a.com".into()), true));
        assert_eq!(exact.preflight(Some("http://evil.com"), Some("GET"), &[]).0, 403);
        assert_eq!(exact.preflight(Some("http://a.com"), Some("TRACE"), &[]).0, 403);
        assert_eq!(exact.preflight(Some("http://a.com"), Some("GET"), &["X-Custom".to_string()]).0, 403);
        // wildcard：ACAO=* 且无 Vary。
        let star = resolve_service_config(DEV_MINIMAL, None).unwrap().cors;
        assert_eq!(
            star.actual_request(Some("http://any.com")),
            CorsOutcome::Allowed { allow_origin: "*".into(), vary_origin: false }
        );
        assert_eq!(star.preflight(Some("http://any.com"), Some("GET"), &[]), (204, Some("*".into()), false));
        // service 空 origins：跨域一律 Absent。
        let svc = resolve_service_config(SVC_MINIMAL, None).unwrap().cors;
        assert_eq!(svc.actual_request(Some("http://a.com")), CorsOutcome::Absent);
    }

    #[test]
    fn host_allowlist_semantics() {
        let svc = resolve_service_config(
            r#"{"profile":"proxy_service","allowed_hosts":["API.Example.com","internal.corp"],"auth":{"responsibility":"edge"}}"#,
            None,
        )
        .unwrap();
        assert!(svc.host_allowed(Some("api.example.com")));
        assert!(svc.host_allowed(Some("API.EXAMPLE.COM:8443"))); // 大小写+端口剥离
        assert!(svc.host_allowed(Some("internal.corp")));
        assert!(!svc.host_allowed(Some("evil.com")));
        assert!(!svc.host_allowed(Some("api.example.com.evil.com")));
        assert!(!svc.host_allowed(None));
        // dev 空表：不校验。
        let dev = resolve_service_config(DEV_MINIMAL, None).unwrap();
        assert!(dev.host_allowed(Some("anything.local")));
        // IPv6 host 规范化。
        let v6 = resolve_service_config(
            r#"{"profile":"proxy_service","allowed_hosts":["[::1]","my.host"],"auth":{"responsibility":"app"}}"#,
            None,
        )
        .unwrap();
        assert!(v6.host_allowed(Some("[::1]:8443")));
        assert!(v6.host_allowed(Some("my.host")));
    }

    #[test]
    fn proxy_identity_single_rewrite_only() {
        let svc = resolve_service_config(
            r#"{"profile":"proxy_service","trusted_proxy_ips":["10.0.0.1"],"allowed_hosts":["a.com"],"auth":{"responsibility":"edge"}}"#,
            None,
        )
        .unwrap();
        let peer: IpAddr = "10.0.0.1".parse().unwrap();
        // 可信 peer + 单 IP → 消费。
        let (id, proto) = svc.effective_client_identity(peer, Some("203.0.113.7"), Some("https"));
        assert_eq!(id.client, "203.0.113.7".parse::<IpAddr>().unwrap());
        assert!(id.via_trusted_proxy && !id.xff_rejected);
        assert_eq!(proto.as_deref(), Some("https"));
        // 链式多段 → 拒绝该头。
        let (id, _) = svc.effective_client_identity(peer, Some("203.0.113.7, 10.0.0.9"), None);
        assert!(id.xff_rejected && id.client == peer);
        // 坏值 → 拒绝。
        let (id, _) = svc.effective_client_identity(peer, Some("not-an-ip"), None);
        assert!(id.xff_rejected);
        // 非 trusted peer → 头完全不消费（伪造 XFF 无效）。
        let attacker: IpAddr = "6.6.6.6".parse().unwrap();
        let (id, proto) = svc.effective_client_identity(attacker, Some("1.2.3.4"), Some("https"));
        assert!(!id.via_trusted_proxy && id.client == attacker && proto.is_none());
        // XFP 非 http/https → 不消费。
        let (_, proto) = svc.effective_client_identity(peer, Some("203.0.113.7"), Some("ftp"));
        assert!(proto.is_none());
    }

    #[test]
    fn rate_limiter_bounded_and_ttl() {
        let cfg = RateLimitConfig { max_requests: 2, window_ms: 1000, bucket_capacity: 2, bucket_ttl_ms: 500 };
        let rl = RateLimiter::new(cfg);
        let a: IpAddr = "1.1.1.1".parse().unwrap();
        let b: IpAddr = "2.2.2.2".parse().unwrap();
        let c: IpAddr = "3.3.3.3".parse().unwrap();
        assert!(matches!(rl.check(a, 0), RateDecision::Allow));
        assert!(matches!(rl.check(a, 100), RateDecision::Allow));
        assert!(matches!(rl.check(a, 200), RateDecision::Limited { .. })); // 窗内超 2
        assert!(matches!(rl.check(a, 1200), RateDecision::Allow)); // 新窗
        assert!(matches!(rl.check(b, 1200), RateDecision::Allow));
        // 满表（a,b 未过期）→ 新身份保守 429。
        assert!(matches!(rl.check(c, 1300), RateDecision::Limited { .. }));
        // TTL 过期回收 → 新身份可进。
        assert!(matches!(rl.check(c, 1900), RateDecision::Allow));
        // a、b 的 last_seen 均已超 TTL → 双双回收，仅剩 c。
        assert_eq!(rl.active_buckets(), 1);
    }

    #[test]
    fn effective_config_hash_stable_and_sensitive() {
        let a = resolve_service_config(DEV_MINIMAL, None).unwrap();
        let b = resolve_service_config(DEV_MINIMAL, None).unwrap();
        assert_eq!(a.effective_config_hash(), b.effective_config_hash());
        let c = resolve_service_config(r#"{"profile":"development","listen":{"port":9000}}"#, None).unwrap();
        assert_ne!(a.effective_config_hash(), c.effective_config_hash());
    }

    #[test]
    fn to_json_round_trips_with_same_hash() {
        let c = resolve_service_config(
            r#"{"profile":"proxy_service","listen":{"port":9100},"allowed_hosts":["a.com","b.org"],"auth":{"responsibility":"edge"},"trusted_proxy_ips":["10.0.0.1"],"rate_limit":{"max_requests":50,"window_ms":1000}}"#,
            None,
        )
        .unwrap();
        let json = c.to_json();
        let c2 = resolve_service_config(&json, None).unwrap();
        assert_eq!(c.effective_config_hash(), c2.effective_config_hash());
        assert_eq!(c2.listen_port, 9100);
        assert_eq!(c2.allowed_hosts, vec!["a.com", "b.org"]);
        assert_eq!(c2.rate_limit.unwrap().max_requests, 50);
        assert_eq!(c2.trusted_proxy_ips, vec!["10.0.0.1".parse::<IpAddr>().unwrap()]);
        // dev 全默认形也 round-trip。
        let d = resolve_service_config(DEV_MINIMAL, None).unwrap();
        let d2 = resolve_service_config(&d.to_json(), None).unwrap();
        assert_eq!(d.effective_config_hash(), d2.effective_config_hash());
    }

    #[test]
    fn active_config_seam_take_once() {
        let cfg = Arc::new(resolve_service_config(DEV_MINIMAL, None).unwrap());
        set_active_service_config(cfg.clone());
        let taken = take_active_service_config().unwrap();
        assert_eq!(taken.effective_config_hash(), cfg.effective_config_hash());
        assert!(take_active_service_config().is_none()); // take 一次即空
    }
}
