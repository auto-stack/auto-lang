// PLAN-716 组B: 帧时间戳观测通道（M4 供料包供②——type_latency/scroll_fps
// 测量解锁 + steady_start 首帧段分解）。
//
// 通道语义（供② 原文承接 + SD-B 契约）：
// - **帧开始**：桌面渲染 update 入口（iced TickWrap::update——全消息流经点）
//   的到达时刻；**呈现完成**：`__frame_pump`（711 帧通知泵——`listen_raw`
//   过滤 RedrawRequested）消息的**消费**时刻。R-1 序障（711 实证）：泵消息
//   消费时刻 ≥ 该帧 present 返回——present 完成的合法代理（硬 post-present
//   屏障需 fork iced_winit，计划约束禁止）。
// - **门控零开销纪律**（AUTO_BENCH 族，PLAN-005 先例；711 AUTO_SCHED_DIAG
//   同域先例）：`AUTO_FRAME_BENCH=1` 时捕获开启；未设=零写入零分配零系统
//   调用（单 OnceLock 布尔读+分支）。
// - **单调时源**：`Instant`（进程起点起算）→ 毫秒 i64（可换算，供②验收
//   形）；首帧段分解（spawn→首帧 begin→首帧 present）由读侧差分得出。
// - **只读可达**：VM 内建 9918/9919（auto.frame.begin_ms/present_ms）+
//   a2r 臂同源（a2r_std::frame::{begin_ms,present_ms}——701 time 族先例）。

use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::OnceLock;

/// 进程起点（模块首次触及时定格——捕获与读侧同一坐标系）。
fn process_start() -> &'static std::time::Instant {
    static START: OnceLock<std::time::Instant> = OnceLock::new();
    START.get_or_init(std::time::Instant::now)
}

/// AUTO_FRAME_BENCH=1 门（读一次定格；未设=捕获关闭）。
fn bench_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("AUTO_FRAME_BENCH").ok().as_deref() == Some("1"))
}

/// PLAN-725 T-00：门控读数开放（frame_segments 分段探针同门复用——
/// 单一 AUTO_FRAME_BENCH 门语义，不引第二门）。
pub fn segments_gate() -> bool {
    bench_enabled()
}

/// PLAN-725 T-00：进程起点起算毫秒（frame_segments 同坐标系读数）。
pub fn process_elapsed_ms() -> i64 {
    elapsed_ms()
}

static FRAME_BEGIN_MS: AtomicI64 = AtomicI64::new(0);
static FRAME_PRESENT_MS: AtomicI64 = AtomicI64::new(0);

/// 帧开始时间戳捕获（update 入口调用；门控关闭时零开销路径）。
pub fn note_frame_begin() {
    if bench_enabled() {
        FRAME_BEGIN_MS.store(elapsed_ms(), Ordering::Relaxed);
    }
}

/// 呈现完成时间戳捕获（`__frame_pump` 消息消费点调用；门控同上）。
pub fn note_frame_present() {
    if bench_enabled() {
        FRAME_PRESENT_MS.store(elapsed_ms(), Ordering::Relaxed);
    }
}

/// 帧开始时间戳（毫秒，单调；未捕获/未发生=0）。
pub fn frame_begin_ms() -> i64 {
    FRAME_BEGIN_MS.load(Ordering::Relaxed)
}

/// 呈现完成时间戳（毫秒，单调；未捕获/未发生=0）。
pub fn frame_present_ms() -> i64 {
    FRAME_PRESENT_MS.load(Ordering::Relaxed)
}

fn elapsed_ms() -> i64 {
    process_start().elapsed().as_millis() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 门控关闭（测试进程默认未设 AUTO_FRAME_BENCH）：note 系列为零开销
    /// 路径且不写入——读侧恒 0（未捕获语义，SD-B）。
    #[test]
    fn gate_off_no_capture() {
        if std::env::var("AUTO_FRAME_BENCH").is_ok() {
            return; // 门开环境下跳过（探针测试负责开态断言）
        }
        note_frame_begin();
        note_frame_present();
        assert_eq!(frame_begin_ms(), 0);
        assert_eq!(frame_present_ms(), 0);
    }

    /// 到达序语义：present 捕获时刻 ≥ begin 捕获时刻（同帧先后序）。
    /// （门开态由 plan716_supply_probes 探针驱动；此处用直接写路径
    /// 不可行——note 系列被门控，单测以读侧单调性为锚。）
    #[test]
    fn read_side_monotonic_shape() {
        let a = frame_begin_ms();
        let p = frame_present_ms();
        assert!(a >= 0 && p >= 0);
        // 未捕获态双零；捕获态 present ≥ begin——两种形态都合法。
        assert!(p == 0 || p >= a || a == 0);
    }
}
