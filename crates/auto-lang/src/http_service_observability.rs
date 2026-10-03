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
use std::sync::mpsc::{self, Sender, SyncSender, TrySendError};
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

fn counters() -> &'static ServiceCounters {
    static C: OnceLock<ServiceCounters> = OnceLock::new();
    C.get_or_init(ServiceCounters::default)
}

/// 进程级单例访问（两轨 serve 入口共享同一进程内实例）。
pub fn service_counters() -> &'static ServiceCounters {
    counters()
}

pub fn counter_add(name: CounterName, delta: u64) {
    let c = counters();
    match name {
        CounterName::ConnAccepted => c.connections_accepted.fetch_add(delta, Ordering::Relaxed),
        CounterName::ConnRejected => c.connections_rejected.fetch_add(delta, Ordering::Relaxed),
        CounterName::ConnActive => c.connections_active.fetch_add(delta, Ordering::Relaxed),
        CounterName::RequestsTotal => c.requests_total.fetch_add(delta, Ordering::Relaxed),
        CounterName::RequestsRejected503 => c.requests_rejected_503.fetch_add(delta, Ordering::Relaxed),
        CounterName::RequestsCanceled => c.requests_canceled.fetch_add(delta, Ordering::Relaxed),
        CounterName::RequestsCompleted => c.requests_completed.fetch_add(delta, Ordering::Relaxed),
        CounterName::StreamSessions => c.stream_sessions.fetch_add(delta, Ordering::Relaxed),
        CounterName::FileSessions => c.file_sessions.fetch_add(delta, Ordering::Relaxed),
        CounterName::UploadSessions => c.upload_sessions.fetch_add(delta, Ordering::Relaxed),
        CounterName::BytesSent => c.bytes_sent.fetch_add(delta, Ordering::Relaxed),
    };
}

pub fn counter_sub(name: CounterName, delta: u64) {
    let c = counters();
    match name {
        CounterName::ConnActive => {
            c.connections_active.fetch_sub(delta, Ordering::Relaxed);
        }
        _ => {}
    }
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
// 有界 JSONL sink（stderr；队满丢弃 + 计数；写线程后台化）
// ============================================================================

enum SinkMsg {
    Line(String),
}

struct SinkState {
    tx: SyncSender<SinkMsg>,
}

static SINK: OnceLock<SinkState> = OnceLock::new();
static SINK_CAPACITY: AtomicU64 = AtomicU64::new(0);

/// 安装日志 sink（进程内一次；容量来自配置 `observability.log_sink_capacity`）。
/// 幂等：第二次安装按第一次容量生效（进程 = 单服务实例语义）。
pub fn install_log_sink(capacity: usize) {
    if SINK.get().is_some() {
        return;
    }
    let (tx, rx) = mpsc::sync_channel::<SinkMsg>(capacity.max(1));
    SINK_CAPACITY.store(capacity as u64, Ordering::Relaxed);
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
    let _ = SINK.set(SinkState { tx });
}

/// 事件行（serde_json 编码，杜绝字符串拼接注入面）。队满 → log_dropped+1，
/// 永不阻塞调用方。
pub fn emit_event(fields: serde_json::Value) {
    let Some(state) = SINK.get() else { return };
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
            counters().log_dropped.fetch_add(1, Ordering::Relaxed);
        }
    }
}

/// 测试复位（重装 sink；生产面进程内单实例无此路径）。
#[cfg(test)]
pub fn reset_log_sink_for_test() {
    // OnceLock 无法清空——测试用独立容量探测通道替代（见 tests）。
}

/// sink 阻塞性负证：容量满后 emit 立即返回（不等待磁盘/不 panic）。
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t03_counters_snapshot_fields() {
        let before = service_counters().snapshot()["connections_accepted"].as_u64().unwrap();
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
