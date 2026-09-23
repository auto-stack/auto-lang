// PLAN-042 T-01：宿主系统日志环（桌面内实时诊断层）。
//!
//! 三层采集归一入一个进程内有界 FIFO 环：
//! ① 宿主 `log` crate trap（[`HostLogger`]，boot 期 [`install_host_logger`]）；
//! ② `syslog!` 双写宏（eprintln 诊断家族逐站点双写——stderr 归档层不退役）；
//! ③ App `log` 出向动词（session.rs `DesktopCommand::Syslog`，notify 同型
//!    分段归因 notify_source → registry_id）。
//!
//! 性能红线（PLAN-042 §1.3）：环写入 O(1)，**绝不触发任何 view 重建**——
//! 视图刷新由 renderer ServiceTick 注入泵全量快照下行（`injection_due`
//! 判定：seq 无变化零注入、≥[`MIN_INJECT_INTERVAL`] 一拍、窗关零扫描）。
//! 环有界 FIFO [`SYSLOG_CAP`] 条，满弹头淘汰最旧；seq 全局单调分配是
//! 注入面的脏判据（[`dirty_seq`]）。

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// 环容量（条）——PLAN-042 §5.1 定案：1000 条（info 默认收，内存预算可控）。
pub const SYSLOG_CAP: usize = 1000;

/// 注入泵最小节拍（PLAN-042 AC-06：注入频率 ≤2Hz）。
pub const MIN_INJECT_INTERVAL: std::time::Duration = std::time::Duration::from_millis(500);

/// 日志级别（v1 三级；动词词面小写）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyslogLevel {
    Error,
    Warn,
    Info,
}

impl SyslogLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            SyslogLevel::Error => "error",
            SyslogLevel::Warn => "warn",
            SyslogLevel::Info => "info",
        }
    }

    /// 词面解析（T-04 动词臂共用）：未知级别兜底 info 不弃单——notify
    /// kind 兜底同款（session.rs notify parse 臂先例）。
    pub fn parse(s: &str) -> SyslogLevel {
        match s {
            "error" => SyslogLevel::Error,
            "warn" => SyslogLevel::Warn,
            _ => SyslogLevel::Info,
        }
    }
}

impl std::fmt::Display for SyslogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// 单条日志（seq 单调；ts_ms = UNIX epoch 毫秒，宿主侧格式化——.at 算术
/// 避开，通知面板 at 串先例同型）。
#[derive(Debug, Clone, PartialEq)]
pub struct SyslogEntry {
    pub seq: u64,
    pub ts_ms: u64,
    pub level: SyslogLevel,
    pub source: String,
    pub msg: String,
}

static RING: Mutex<VecDeque<SyslogEntry>> = Mutex::new(VecDeque::new());
static SEQ: AtomicU64 = AtomicU64::new(0);

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// 入环（三层采集唯一写入口）：seq 单调分配 + 时间戳；满 [`SYSLOG_CAP`]
/// 弹头（淘汰最旧）。返回本条 seq。
pub fn push(level: SyslogLevel, source: &str, msg: String) -> u64 {
    let seq = SEQ.fetch_add(1, Ordering::Relaxed);
    let entry = SyslogEntry {
        seq,
        ts_ms: now_ms(),
        level,
        source: source.to_string(),
        msg,
    };
    if let Ok(mut ring) = RING.lock() {
        if ring.len() >= SYSLOG_CAP {
            ring.pop_front();
        }
        ring.push_back(entry);
    }
    seq
}

/// 注入面脏判据（PLAN-042 §5.5）：最新已分配 seq。0 = 环自 boot 起无写入；
/// 与注入方记录的 `last_injected` 比对——相等即无新行零注入。
pub fn dirty_seq() -> u64 {
    SEQ.load(Ordering::Relaxed)
}

/// 全量快照（环序 = 最旧→最新；克隆——注入面全量快照替换语义，v1 不做
/// 增量协议）。锁内拷贝，调用方持锁零交叉。
pub fn snapshot() -> Vec<SyslogEntry> {
    RING
        .lock()
        .map(|ring| ring.iter().cloned().collect())
        .unwrap_or_default()
}

/// 环内条数（观测/单测面）。
pub fn len() -> usize {
    RING.lock().map(|r| r.len()).unwrap_or(0)
}

/// 注入节流判定（T-06 纯函数——假时钟单测面；renderer ServiceTick 段消费）：
/// seq 无变化零注入 → 距上次注入不足 [`MIN_INJECT_INTERVAL`] 攒批 → 放行。
/// `last_at = None` = 本窗首拍（立即放行，一次基线注入）。
pub fn injection_due(
    dirty: u64,
    last_injected: u64,
    last_at: Option<Instant>,
    now: Instant,
) -> bool {
    if dirty == 0 || dirty == last_injected {
        return false;
    }
    match last_at {
        Some(t) => now.duration_since(t) >= MIN_INJECT_INTERVAL,
        None => true,
    }
}

/// T-02：宿主 `log` crate trap（error/warn/info 三级入环，source=`host`，
/// target 透传进 msg 前缀；Debug/Trace 不入环——容量预算面）。转发形态：
/// 不接管既有终端输出链——桌面轨（ui_desktop）本无 logger，装环即全量；
/// 若入口已装 logger 则 [`install_host_logger`] 返回 false，宏面双写兜底
/// （不强拆既有链，PLAN-042 §5.2 定案）。
pub struct HostLogger;

impl log::Log for HostLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Info
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let level = match record.level() {
            log::Level::Error => SyslogLevel::Error,
            log::Level::Warn => SyslogLevel::Warn,
            _ => SyslogLevel::Info,
        };
        push(
            level,
            "host",
            format!("[{}] {}", record.target(), record.args()),
        );
    }

    fn flush(&self) {}
}

/// boot 期安装（ui_desktop 入口一次）。true = 本进程环 logger 就位；
/// false = 已有 logger 占槽（降级宏面，零干预）。
pub fn install_host_logger() -> bool {
    log::set_boxed_logger(Box::new(HostLogger)).is_ok()
}

/// 诊断家族双写宏（PLAN-042 §5.1）：环（桌面内实时层）+ 原 eprintln
/// （stderr 文件事后归档层，保留不退役）。`$level` 取 [`SyslogLevel`]
/// 值；`$source` 归因串（`host`/`vm:<app>`/registry_id）。
#[macro_export]
macro_rules! syslog {
    ($level:expr, $source:expr, $($arg:tt)*) => {{
        let __lvl = $level;
        let __src = $source;
        eprintln!("[syslog][{}][{}] {}", __lvl, __src, format!($($arg)*));
        // 语句宏语义（恒 ()）——match 臂/表达式语句位直用。
        $crate::ui::syslog::push(__lvl, &__src, format!($($arg)*));
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    // 全局环被同进程其他测试并发写入——各测试用唯一 source 前缀过滤
    // 自证，不假设独占（断言不依赖全局绝对量）。

    fn mine(source: &str) -> Vec<SyslogEntry> {
        snapshot()
            .into_iter()
            .filter(|e| e.source == source)
            .collect()
    }

    #[test]
    fn seq_strictly_monotonic_across_pushes() {
        let src = "t01-mono";
        let a = push(SyslogLevel::Info, src, "one".into());
        let b = push(SyslogLevel::Warn, src, "two".into());
        let c = push(SyslogLevel::Error, src, "three".into());
        assert!(a < b && b < c);
        let got = mine(src);
        assert_eq!(got.len(), 3);
        let seqs: Vec<u64> = got.iter().map(|e| e.seq).collect();
        assert_eq!(seqs, vec![a, b, c]);
        // 环序 = 最旧→最新（snapshot 全序）。
        assert_eq!(got[0].msg, "one");
        assert_eq!(got[2].level, SyslogLevel::Error);
    }

    #[test]
    fn capacity_evicts_oldest_fifo() {
        let src = "t01-cap";
        for i in 0..(SYSLOG_CAP + 50) {
            push(SyslogLevel::Info, src, format!("line-{i}"));
        }
        let got = mine(src);
        // 全环有界（含并发写入者，总长不超容量）。
        assert!(len() <= SYSLOG_CAP);
        // 本测试自己的行：首批 50 条被弹头淘汰（并发写入者可能再多挤掉
        // 几条——最旧者优先，单调性不受影响）。
        assert!(got.len() >= SYSLOG_CAP.saturating_sub(64));
        let seqs: Vec<u64> = got.iter().map(|e| e.seq).collect();
        let mut sorted = seqs.clone();
        sorted.sort_unstable();
        assert_eq!(seqs, sorted, "snapshot 序应保持 seq 单调（淘汰只弹头）");
        let first = got.first().expect("t01-cap survivors");
        assert!(
            first.msg.starts_with("line-5") || got.len() == SYSLOG_CAP,
            "最旧幸存条应靠近淘汰边界: {:?}",
            first
        );
    }

    #[test]
    fn dirty_seq_advances_with_pushes() {
        let before = dirty_seq();
        let seq = push(SyslogLevel::Info, "t01-dirty", "tick".into());
        let after = dirty_seq();
        // 并发测试进程内其他写入者也在推 seq——只断言相对不变量
        // （单调前进 + 本条 seq 不超前最新）。
        assert!(after >= seq);
        assert!(after > before || before == 0);
    }

    #[test]
    fn level_parse_unknown_falls_back_to_info() {
        assert_eq!(SyslogLevel::parse("error"), SyslogLevel::Error);
        assert_eq!(SyslogLevel::parse("warn"), SyslogLevel::Warn);
        assert_eq!(SyslogLevel::parse("info"), SyslogLevel::Info);
        assert_eq!(SyslogLevel::parse("verbose"), SyslogLevel::Info);
        assert_eq!(SyslogLevel::parse(""), SyslogLevel::Info);
    }

    #[test]
    fn injection_due_throttle_semantics() {
        let t0 = Instant::now();
        // 环无写入：零注入。
        assert!(!injection_due(0, 0, None, t0));
        // seq 无变化：零注入（哪怕已过任意时长）。
        let later = t0 + Duration::from_secs(10);
        assert!(!injection_due(7, 7, Some(t0), later));
        // 首拍（无上次注入记录）：立即放行。
        assert!(injection_due(7, 0, None, t0));
        // seq 变化但距上次 <500ms：攒批拒绝。
        assert!(!injection_due(8, 7, Some(t0), t0 + Duration::from_millis(499)));
        // seq 变化且 ≥500ms：放行。
        assert!(injection_due(8, 7, Some(t0), t0 + Duration::from_millis(500)));
    }

    #[test]
    fn macro_dual_writes_eprintln_and_ring() {
        // eprintln 侧由捕获_stderr 测试框架覆盖成本高——本测锚环侧写入 +
        // 展开形态（eprintln 臂同表达式共存，stderr 输出为可接受副产物）。
        // 语句宏语义（恒 ()）：match 臂位直用可编译。
        let src = "t01-macro";
        let mut landed = false;
        if src.starts_with("t01") {
            crate::syslog!(SyslogLevel::Warn, src, "macro line {}", 42);
            landed = true;
        }
        assert!(landed);
        let got = mine(src);
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].msg, "macro line 42");
        assert_eq!(got[0].level, SyslogLevel::Warn);
    }
}
