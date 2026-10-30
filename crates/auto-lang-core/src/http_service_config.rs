//! HTTP service runtime and public configuration facade.
//!
//! PLAN-890 bounded contract cut: configuration/validation/pure policy have
//! one source in contract.rs; mutable rate limits and active service assembly
//! stay here until the Web runtime migration. Public paths re-export the same
//! types rather than maintaining duplicate DTOs.

mod contract;
pub use contract::*;

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, Mutex, OnceLock};
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
        Self {
            cfg,
            buckets: Mutex::new(HashMap::new()),
        }
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
            Err(_) => {
                return RateDecision::Limited {
                    retry_after_secs: self.cfg.window_ms / 1000,
                }
            }
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
                    return RateDecision::Limited {
                        retry_after_secs: (remaining / 1000).max(1),
                    };
                }
                b.count += 1;
                b.last_seen_ms = now_ms;
                RateDecision::Allow
            }
            None => {
                if buckets.len() >= self.cfg.bucket_capacity {
                    // 满表且无可回收：保守拒绝新身份（不绕过限速）。
                    return RateDecision::Limited {
                        retry_after_secs: self.cfg.window_ms / 1000,
                    };
                }
                buckets.insert(
                    identity,
                    RateBucket {
                        window_start_ms: now_ms,
                        count: 1,
                        last_seen_ms: now_ms,
                    },
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
        assert!(
            e.field == "allowed_hosts" || e.field == "auth.responsibility",
            "{e}"
        );
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
        assert_eq!(
            c.cors.allowed_methods,
            ["GET", "POST", "PUT", "DELETE", "PATCH"]
        );
    }

    #[test]
    fn unknown_field_and_bad_profile_rejected() {
        let e =
            resolve_service_config(r#"{"profile":"development","nosuch":1}"#, None).unwrap_err();
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
        assert_eq!(
            bad(r#"{"max_connections":0}"#).field,
            "limits.max_connections"
        );
        assert_eq!(
            bad(r#"{"max_connections":9999}"#).field,
            "limits.max_connections"
        );
        assert_eq!(
            bad(r#"{"header_buf_bytes":1024}"#).field,
            "limits.header_buf_bytes"
        );
        assert_eq!(
            bad(r#"{"drain_timeout_ms":50}"#).field,
            "limits.drain_timeout_ms"
        );
        assert_eq!(
            bad(r#"{"request_timeout_ms":0}"#).field,
            "limits.request_timeout_ms"
        );
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
        let c = resolve_service_config(
            r#"{"profile":"development","listen":{"port":9000}}"#,
            Some(7777),
        )
        .unwrap();
        assert_eq!(c.listen_port, 7777);
        assert!(c
            .sources
            .iter()
            .any(|(f, s)| f == "listen.port" && *s == "cli_override"));
        let c = resolve_service_config(
            r#"{"profile":"development","listen":{"addr":"0.0.0.0","port":9000}}"#,
            None,
        )
        .unwrap();
        assert_eq!(c.listen_addr, IpAddr::from([0, 0, 0, 0]));
        assert_eq!(c.listen_port, 9000);
        // port=0（临时端口）合法。
        assert!(
            resolve_service_config(r#"{"profile":"development","listen":{"port":0}}"#, None)
                .is_ok()
        );
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
            CorsOutcome::Allowed {
                allow_origin: "http://a.com".into(),
                vary_origin: true
            }
        );
        // 不命中 → 无 CORS 头（不是拒绝响应）。
        assert_eq!(
            exact.actual_request(Some("http://evil.com")),
            CorsOutcome::Absent
        );
        assert_eq!(exact.actual_request(None), CorsOutcome::Absent);
        // preflight 三要素。
        let ok = exact.preflight(
            Some("http://a.com"),
            Some("POST"),
            &["content-type".to_string()],
        );
        assert_eq!(ok, (204, Some("http://a.com".into()), true));
        assert_eq!(
            exact.preflight(Some("http://evil.com"), Some("GET"), &[]).0,
            403
        );
        assert_eq!(
            exact.preflight(Some("http://a.com"), Some("TRACE"), &[]).0,
            403
        );
        assert_eq!(
            exact
                .preflight(Some("http://a.com"), Some("GET"), &["X-Custom".to_string()])
                .0,
            403
        );
        // wildcard：ACAO=* 且无 Vary。
        let star = resolve_service_config(DEV_MINIMAL, None).unwrap().cors;
        assert_eq!(
            star.actual_request(Some("http://any.com")),
            CorsOutcome::Allowed {
                allow_origin: "*".into(),
                vary_origin: false
            }
        );
        assert_eq!(
            star.preflight(Some("http://any.com"), Some("GET"), &[]),
            (204, Some("*".into()), false)
        );
        // service 空 origins：跨域一律 Absent。
        let svc = resolve_service_config(SVC_MINIMAL, None).unwrap().cors;
        assert_eq!(
            svc.actual_request(Some("http://a.com")),
            CorsOutcome::Absent
        );
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
        let cfg = RateLimitConfig {
            max_requests: 2,
            window_ms: 1000,
            bucket_capacity: 2,
            bucket_ttl_ms: 500,
        };
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
        let c = resolve_service_config(r#"{"profile":"development","listen":{"port":9000}}"#, None)
            .unwrap();
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
        assert_eq!(
            c2.trusted_proxy_ips,
            vec!["10.0.0.1".parse::<IpAddr>().unwrap()]
        );
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
