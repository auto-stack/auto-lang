//! PLAN-736: HTTP 服务观测——有界计数器组 + 非阻塞 JSONL 事件汇（T-03 核心，
//! T-05 补全事件面）。
//!
//! 设计约束（决策 §3.6 / AC-05）：
//! - 计数器 = 原子量，快照经 `/__auto/health/snapshot`（loopback-only）暴露；
//!   高基数字段（原始路径/IP 明细）**不做** counter 维度。
//! - 日志 sink = 有界 mpsc + 后台写线程（stderr JSONL）；队满丢弃并计
//!   `log_dropped`，业务**不等待**磁盘。
//! - 两轨共用（无 axum/hyper 类型）；生成 crate 经 `auto_lang` 依赖消费。

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, SyncSender, TrySendError};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// 服务级原子计数器组（snapshot 面）。
#[derive(Default)]
pub struct ServiceCounters {
    pub connections_accepted: AtomicU64,
    pub connections_rejected: AtomicU64,
    pub connections_active: AtomicU64,
    pub requests_total: AtomicU64,
    pub requests_rejected_503: AtomicU64,
    pub requests_canceled: AtomicU64,
    pub requests_completed: AtomicU64,
    pub stream_sessions: AtomicU64,
    pub file_sessions: AtomicU64,
    pub upload_sessions: AtomicU64,
    pub log_dropped: AtomicU64,
    pub bytes_sent: AtomicU64,
}

impl ServiceCounters {
    pub fn snapshot(&self) -> serde_json::Value {
        serde_json::json!({
            "connections_accepted": self.connections_accepted.load(Ordering::Relaxed),
            "connections_rejected": self.connections_rejected.load(Ordering::Relaxed),
            "connections_active": self.connections_active.load(Ordering::Relaxed),
            "requests_total": self.requests_total.load(Ordering::Relaxed),
            "requests_rejected_503": self.requests_rejected_503.load(Ordering::Relaxed),
            "requests_canceled": self.requests_canceled.load(Ordering::Relaxed),
            "requests_completed": self.requests_completed.load(Ordering::Relaxed),
            "stream_sessions": self.stream_sessions.load(Ordering::Relaxed),
            "file_sessions": self.file_sessions.load(Ordering::Relaxed),
            "upload_sessions": self.upload_sessions.load(Ordering::Relaxed),
            "log_dropped": self.log_dropped.load(Ordering::Relaxed),
            "bytes_sent": self.bytes_sent.load(Ordering::Relaxed),
        })
    }
}

/// One serving instance's identity, state, counters and bounded event sink.
#[derive(Default)]
pub struct ServiceObservation {
    counters: ServiceCounters,
    state: AtomicU64,
    instance_id: OnceLock<String>,
    identity: Mutex<Option<serde_json::Value>>,
    sink: OnceLock<SinkState>,
}

impl std::fmt::Debug for ServiceObservation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServiceObservation")
            .field("identity", &self.service_identity())
            .finish()
    }
}

/// Legacy single-process entrypoints share this observation. Explicit service
/// runtimes allocate their own instance instead of replacing this slot.
pub fn process_observation() -> std::sync::Arc<ServiceObservation> {
    process_observation_ref().clone()
}

fn process_observation_ref() -> &'static std::sync::Arc<ServiceObservation> {
    static OBS: OnceLock<std::sync::Arc<ServiceObservation>> = OnceLock::new();
    OBS.get_or_init(|| std::sync::Arc::new(ServiceObservation::default()))
}

/// Legacy process-level facade. Configured instances use their own counters.
pub fn service_counters() -> &'static ServiceCounters {
    &process_observation_ref().counters
}

pub fn counter_add(name: CounterName, delta: u64) {
    process_observation_ref().counter_add(name, delta);
}

impl ServiceObservation {
    pub fn counters(&self) -> &ServiceCounters {
        &self.counters
    }

    pub fn counter_add(&self, name: CounterName, delta: u64) {
        let c = &self.counters;
        match name {
            CounterName::ConnAccepted => c.connections_accepted.fetch_add(delta, Ordering::Relaxed),
            CounterName::ConnRejected => c.connections_rejected.fetch_add(delta, Ordering::Relaxed),
            CounterName::ConnActive => c.connections_active.fetch_add(delta, Ordering::Relaxed),
            CounterName::RequestsTotal => c.requests_total.fetch_add(delta, Ordering::Relaxed),
            CounterName::RequestsRejected503 => {
                c.requests_rejected_503.fetch_add(delta, Ordering::Relaxed)
            }
            CounterName::RequestsCanceled => {
                c.requests_canceled.fetch_add(delta, Ordering::Relaxed)
            }
            CounterName::RequestsCompleted => {
                c.requests_completed.fetch_add(delta, Ordering::Relaxed)
            }
            CounterName::StreamSessions => c.stream_sessions.fetch_add(delta, Ordering::Relaxed),
            CounterName::FileSessions => c.file_sessions.fetch_add(delta, Ordering::Relaxed),
            CounterName::UploadSessions => c.upload_sessions.fetch_add(delta, Ordering::Relaxed),
            CounterName::BytesSent => c.bytes_sent.fetch_add(delta, Ordering::Relaxed),
        };
    }

    pub fn counter_sub(&self, name: CounterName, delta: u64) {
        let c = &self.counters;
        match name {
            CounterName::ConnActive => {
                c.connections_active.fetch_sub(delta, Ordering::Relaxed);
            }
            _ => {}
        }
    }
}

pub fn counter_sub(name: CounterName, delta: u64) {
    process_observation_ref().counter_sub(name, delta);
}

#[derive(Clone, Copy)]
pub enum CounterName {
    ConnAccepted,
    ConnRejected,
    ConnActive,
    RequestsTotal,
    RequestsRejected503,
    RequestsCanceled,
    RequestsCompleted,
    StreamSessions,
    FileSessions,
    UploadSessions,
    BytesSent,
}

// ============================================================================
// 服务状态机 + 身份（AC-02：ready 语义可证；drain/stop 事件）
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceState {
    Starting,
    Ready,
    Draining,
    Stopped,
}

impl ServiceState {
    pub fn as_str(self) -> &'static str {
        match self {
            ServiceState::Starting => "starting",
            ServiceState::Ready => "ready",
            ServiceState::Draining => "draining",
            ServiceState::Stopped => "stopped",
        }
    }
}

fn state_from_u64(v: u64) -> ServiceState {
    match v {
        1 => ServiceState::Ready,
        2 => ServiceState::Draining,
        3 => ServiceState::Stopped,
        _ => ServiceState::Starting,
    }
}

fn state_to_u64(s: ServiceState) -> u64 {
    match s {
        ServiceState::Starting => 0,
        ServiceState::Ready => 1,
        ServiceState::Draining => 2,
        ServiceState::Stopped => 3,
    }
}

/// 进程级实例身份：serve 入口调用一次（pid + 启动纳秒 + 随机尾）。
pub fn init_instance_id() -> String {
    instance_id()
}

pub fn instance_id() -> String {
    process_observation_ref().instance_id()
}

impl ServiceObservation {
    pub fn instance_id(&self) -> String {
        // 注意：闭包内不得再调 init_instance_id()——同一 OnceLock 同线程重入
        // get_or_init = 死锁（T-05 实测：VM ready 臂挂死于身份设置）。
        self.instance_id
            .get_or_init(|| {
                let nanos = std::time::SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.subsec_nanos() as u64 + d.as_secs().wrapping_mul(1_000_000_000))
                    .unwrap_or(0);
                static LAST_STAMP: AtomicU64 = AtomicU64::new(0);
                let previous = LAST_STAMP
                    .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |last| {
                        last.checked_add(1).map(|next| nanos.max(next))
                    })
                    .expect("service instance identity space exhausted");
                let stamp = nanos.max(previous + 1);
                format!("svc-{}-{:x}", std::process::id(), stamp)
            })
            .clone()
    }

    /// 身份四元组（bound/profile/config_hash + instance_id）——ready 行、health
    /// body、父进程核对共用同一事实源。
    pub fn set_service_identity(&self, bound: &str, profile: &str, config_hash: u64) {
        if let Ok(mut slot) = self.identity.lock() {
            *slot = Some(serde_json::json!({
                "instance_id": self.instance_id(),
                "bound": bound,
                "profile": profile,
                "config_hash": format!("{:016x}", config_hash),
            }));
        }
    }

    pub fn service_identity(&self) -> Option<serde_json::Value> {
        self.identity.lock().ok().and_then(|s| s.clone())
    }

    pub fn set_service_state(&self, s: ServiceState) {
        self.state.store(state_to_u64(s), Ordering::SeqCst);
        self.emit_event(serde_json::json!({ "event": "state", "state": s.as_str() }));
    }

    pub fn service_state(&self) -> ServiceState {
        state_from_u64(self.state.load(Ordering::SeqCst))
    }

    /// ready 判定（AC-02）：Ready 且未 drain——health/ready 与父进程轮询共用。
    pub fn is_ready(&self) -> bool {
        self.service_state() == ServiceState::Ready
    }
}

pub fn set_service_identity(bound: &str, profile: &str, hash: u64) {
    process_observation_ref().set_service_identity(bound, profile, hash);
}
pub fn service_identity() -> Option<serde_json::Value> {
    process_observation_ref().service_identity()
}
pub fn set_service_state(state: ServiceState) {
    process_observation_ref().set_service_state(state);
}
pub fn service_state() -> ServiceState {
    process_observation_ref().service_state()
}
pub fn is_ready() -> bool {
    process_observation_ref().is_ready()
}

// ============================================================================
// 请求事件上下文（恰一次终态：scope finalize/complete/cancel 三路收敛）
// ============================================================================

/// 桥层在请求入队时挂到 scope 上；scope 终结点（幂等）发出恰一次终态事件。
/// bytes 在回复已知时填充；流/文件以 scope 终结为终态（不 headers 即记）。
#[derive(Debug)]
pub struct PendingRequestEvent {
    pub method: String,
    /// route 模板（去 query；参数化路由为具体路径——v1 边界，T-08 复核）。
    pub route: String,
    pub status: Option<u16>,
    pub bytes_sent: u64,
    /// 已发终态（幂等闸）。
    sent: std::sync::atomic::AtomicBool,
    observation: std::sync::Arc<ServiceObservation>,
}

impl PendingRequestEvent {
    pub fn new(method: &str, route: &str) -> Self {
        Self::new_in(process_observation(), method, route)
    }

    pub fn new_in(
        observation: std::sync::Arc<ServiceObservation>,
        method: &str,
        route: &str,
    ) -> Self {
        Self {
            method: method.to_string(),
            route: route.chars().take(128).collect(),
            status: None,
            bytes_sent: 0,
            sent: std::sync::atomic::AtomicBool::new(false),
            observation,
        }
    }

    fn emit_once(&self, outcome: &str) {
        if self.sent.swap(true, std::sync::atomic::Ordering::SeqCst) {
            return; // cancel/drop 双路径只发一次
        }
        // 计数与事件同点收敛（snapshot 面与 JSONL 面一致）。
        let counter = if outcome == "completed" {
            CounterName::RequestsCompleted
        } else {
            CounterName::RequestsCanceled
        };
        self.observation.counter_add(counter, 1);
        self.observation
            .counter_add(CounterName::BytesSent, self.bytes_sent);
        self.observation.emit_event(serde_json::json!({
            "event": "request",
            "method": self.method,
            "route": self.route,
            "status": self.status,
            "bytes_sent": self.bytes_sent,
            "outcome": outcome,
        }));
    }

    pub fn complete(&self) {
        self.emit_once("completed");
    }

    pub fn cancel(&self) {
        self.emit_once("canceled");
    }
}

// ============================================================================
// 有界 JSONL sink（stderr；队满丢弃 + 计数；写线程后台化）
// ============================================================================

enum SinkMsg {
    Line(String),
}

struct SinkState {
    tx: SyncSender<SinkMsg>,
}

/// 安装日志 sink（进程内一次；容量来自配置 `observability.log_sink_capacity`）。
/// Legacy facade; each explicit observation independently installs one sink.
pub fn install_log_sink(capacity: usize) {
    process_observation_ref().install_log_sink(capacity);
}

impl ServiceObservation {
    pub fn install_log_sink(&self, capacity: usize) {
        if self.sink.get().is_some() {
            return;
        }
        let (tx, rx) = mpsc::sync_channel::<SinkMsg>(capacity.max(1));
        let _ = std::thread::Builder::new()
            .name("auto-http-log-sink".into())
            .spawn(move || {
                use std::io::Write;
                let stderr = std::io::stderr();
                for msg in rx {
                    if let SinkMsg::Line(l) = msg {
                        let mut h = stderr.lock();
                        let _ = writeln!(h, "{l}");
                    }
                }
            });
        let _ = self.sink.set(SinkState { tx });
    }

    /// 事件行（serde_json 编码，杜绝字符串拼接注入面）。队满 → log_dropped+1，
    /// 永不阻塞调用方。
    pub fn emit_event(&self, fields: serde_json::Value) {
        let Some(state) = self.sink.get() else { return };
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        let mut obj = serde_json::json!({ "ts": ts });
        if let (Some(dst), Some(src)) = (obj.as_object_mut(), fields.as_object()) {
            for (k, v) in src {
                dst.insert(k.clone(), v.clone());
            }
        }
        match state.tx.try_send(SinkMsg::Line(obj.to_string())) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) | Err(TrySendError::Disconnected(_)) => {
                self.counters.log_dropped.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}

pub fn emit_event(fields: serde_json::Value) {
    process_observation_ref().emit_event(fields);
}

/// 测试复位（重装 sink；生产面进程内单实例无此路径）。

/// sink 阻塞性负证：容量满后 emit 立即返回（不等待磁盘/不 panic）。
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instance_observation_and_terminal_event_are_owned_and_exactly_once() {
        let first = std::sync::Arc::new(ServiceObservation::default());
        let second = std::sync::Arc::new(ServiceObservation::default());
        first.set_service_identity("127.0.0.1:1", "development", 1);
        second.set_service_identity("127.0.0.1:2", "development", 2);
        assert_ne!(first.instance_id(), second.instance_id());
        first.set_service_state(ServiceState::Ready);
        assert!(first.is_ready());
        assert!(!second.is_ready());
        let mut event = PendingRequestEvent::new_in(first.clone(), "GET", "/owned");
        event.bytes_sent = 42;
        std::thread::spawn(move || {
            event.complete();
            event.complete();
            event.cancel();
        })
        .join()
        .unwrap();
        let a = first.counters().snapshot();
        let b = second.counters().snapshot();
        assert_eq!(a["requests_completed"], 1);
        assert_eq!(a["requests_canceled"], 0);
        assert_eq!(a["bytes_sent"], 42);
        assert_eq!(b["requests_completed"], 0);
        assert_eq!(b["bytes_sent"], 0);
        let mut canceled = PendingRequestEvent::new_in(second.clone(), "POST", "/canceled");
        canceled.bytes_sent = 9;
        canceled.cancel();
        canceled.complete();
        let b = second.counters().snapshot();
        assert_eq!(b["requests_canceled"], 1);
        assert_eq!(b["requests_completed"], 0);
        assert_eq!(b["bytes_sent"], 9);
        assert_eq!(first.counters().snapshot()["bytes_sent"], 42);
        assert_eq!(first.service_identity().unwrap()["bound"], "127.0.0.1:1");
    }

    #[test]
    fn instance_sink_overflow_uses_real_emit_path_and_own_drop_counter() {
        let first = ServiceObservation::default();
        let second = ServiceObservation::default();
        let (tx, rx) = mpsc::sync_channel(1);
        assert!(first.sink.set(SinkState { tx }).is_ok());
        first.emit_event(serde_json::json!({"message": "line\nquote\""}));
        first.emit_event(serde_json::json!({"message": "full"}));
        second.emit_event(serde_json::json!({"message": "no sink"}));
        assert_eq!(first.counters().snapshot()["log_dropped"], 1);
        assert_eq!(second.counters().snapshot()["log_dropped"], 0);
        let SinkMsg::Line(line) = rx.recv().unwrap();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&line).unwrap()["message"],
            "line\nquote\""
        );
        assert!(!line.contains('\n'));
    }

    #[test]
    fn t03_counters_snapshot_fields() {
        let before = service_counters().snapshot()["connections_accepted"]
            .as_u64()
            .unwrap();
        counter_add(CounterName::ConnAccepted, 3);
        counter_add(CounterName::ConnActive, 2);
        counter_sub(CounterName::ConnActive, 1);
        let snap = service_counters().snapshot();
        assert_eq!(snap["connections_accepted"].as_u64().unwrap(), before + 3);
        // 并行测试共享进程计数——active 只断言自增自减差值方向正确：
        assert!(snap["connections_active"].as_u64().unwrap() >= 1);
    }

    #[test]
    fn t03_sink_drop_overflows_without_blocking() {
        // 独立通道验证满载丢弃语义（进程级 SINK 已被其它测试占用形态，
        // 这里直接构造 SyncSender 复刻同一判定路径）。
        let (tx, rx) = mpsc::sync_channel::<SinkMsg>(1);
        tx.try_send(SinkMsg::Line("a".into())).unwrap();
        let dropped = std::thread::spawn(move || {
            // 消费端不读取：第二次 try_send 必须立即失败（非阻塞证据）。
            let t0 = std::time::Instant::now();
            let r = tx.try_send(SinkMsg::Line("b".into()));
            let elapsed = t0.elapsed();
            (r.is_err(), elapsed < std::time::Duration::from_millis(50))
        });
        let (dropped_fast, non_blocking) = dropped.join().unwrap();
        assert!(dropped_fast && non_blocking, "满载必须立即拒绝");
        drop(rx);
    }

    #[test]
    fn t03_emit_event_before_install_is_noop() {
        // 未安装 sink 时 emit 不 panic 不计数（SINK 可能已被其它测试安装——
        // 此处只验证调用面安全）。
        emit_event(serde_json::json!({"event": "test-noop"}));
    }
}
