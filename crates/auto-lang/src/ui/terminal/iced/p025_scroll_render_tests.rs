//! PLAN-025 T-01 判决工件:滚动闪屏机理的 headless 复现器。
//!
//! 复现链与实车逐层同构:
//! - 引擎侧:不可变 scrollback 模型(伪引擎;line id = 绝对行号,同 id
//!   恒同内容——对应 alacritty_terminal 0.26 不可变滚动存储);
//! - 泵侧(term.rs feed_snapshot_inner 同构):按 display_offset 重采视口
//!   窗投喂(terminal_feed_cells_for 逐行)+ offset/history 回写;
//! - draw 侧:refresh_row_cache 真码步进(重排计数埋点)。
//!
//! 判决问题:滚动每帧行 Paragraph 重建数(候选 1:槽位键整窗重排)与
//! 泵滞后空白带(候选 2:视图先行/窗口等泵)的量级与配比。基线断言 =
//! 现行槽位键语义(每 notch 整窗重建——缺陷即证据);T-03 绝对行号键
//! 落地后,p025 曲线断言翻转为"≤ 新暴露行 + 预取增量"。
//!
//! Run: cargo test -p auto-lang --features ui-iced,iced-layout-tests --lib p025 -- --nocapture
#![cfg(all(test, feature = "iced-layout-tests"))]

use crate::ui::terminal::iced::widget::{refresh_row_cache, row_rebuilds_reset};
use crate::ui::terminal::{
    terminal, terminal_dispose, terminal_feed_cells_for, terminal_set_history,
    terminal_set_scroll_offset, TermCell,
};

/// 伪引擎行内容:行 id 即内容指纹(同 id 恒同文本)。
fn fake_line_cells(id: i64, cols: usize) -> Vec<TermCell> {
    let text = format!("L{id:06} the quick brown fox jumps over");
    text.chars().take(cols).map(TermCell::plain).collect()
}

/// 泵步进(term.rs feed_snapshot_inner 同构):o = 引擎 display_offset,
/// h = 引擎历史行数;重采视口窗 [h-o, h-o+rows) 逐行投喂 + 状态回写。
fn pump(core: &crate::ui::terminal::TerminalCore, key: &str, o: usize, h: usize, rows: usize, cols: usize) {
    for y in 0..rows {
        let id = h as i64 - o as i64 + y as i64;
        terminal_feed_cells_for(key, y, fake_line_cells(id, cols));
    }
    terminal_set_scroll_offset(core, o);
    terminal_set_history(core, h);
}

/// draw 期缓存步进(真码):返回本帧行 Paragraph 重建数。
fn draw_step(key: &str, core: &crate::ui::terminal::TerminalCore, rows: usize) -> usize {
    let (cells, digests) = core.snapshot();
    let palette = crate::ui::terminal::terminal_effective_palette(
        crate::ui::terminal::TERMINAL_SCHEME_CLASSIC_DARK,
    );
    let pal_key: u64 = {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        palette.hash(&mut hasher);
        hasher.finish()
    };
    row_rebuilds_reset();
    refresh_row_cache(key, &cells, &digests, &palette, pal_key, rows)
}

/// 基线曲线:静止零重排(digest 门控有效性)。
#[test]
fn p025_static_frame_zero_rebuild() {
    terminal_dispose("p025-static");
    let core = terminal("p025-static", 80, 30);
    let (rows, cols, h) = (30usize, 80usize, 500usize);
    pump(core, "p025-static", 0, h, rows, cols);
    let first = draw_step("p025-static", core, rows);
    assert_eq!(first, rows, "首帧整窗成形");
    // 同窗重泵 + 重绘:零重建(静止不闪)。
    pump(core, "p025-static", 0, h, rows, cols);
    assert_eq!(draw_step("p025-static", core, rows), 0, "静止帧零重排");
    terminal_dispose("p025-static");
}

/// 判决面 1(候选 1):滚轮连续回滚时每 notch 的重建数曲线。现行槽位键
/// 语义 = 视口窗整体位移 → 每槽 digest 全失效 → 每 notch 整窗重排
/// (rows 行 shaping 集中在一帧 = 帧预算尖峰/闪屏主嫌疑)。T-03 落地后
/// 本断言翻转:≤ notch 行数 + 预取增量。
#[test]
fn p025_baseline_rebuild_curve_full_window_per_notch() {
    terminal_dispose("p025-curve");
    let core = terminal("p025-curve", 80, 30);
    let (rows, cols, h) = (30usize, 80usize, 500usize);
    let mut o = 0usize;
    pump(core, "p025-curve", o, h, rows, cols);
    let first = draw_step("p025-curve", core, rows);
    assert_eq!(first, rows, "首帧整窗成形");
    let mut curve = vec![first];
    for _ in 0..10 {
        o += 3; // 一 notch 上翻(引擎约定正 = 上翻历史)
        pump(core, "p025-curve", o, h, rows, cols);
        curve.push(draw_step("p025-curve", core, rows));
    }
    eprintln!("[P025-CURVE] baseline per-notch rebuilds: {curve:?}");
    assert!(
        curve[1..].iter().all(|&n| n == rows),
        "现行槽位键:每 notch 整窗 {rows} 行全量重排(候选 1 判决面),got {curve:?}"
    );
    terminal_dispose("p025-curve");
}

/// 判决面 2(候选 2):泵滞后空白带的时间线模型。视图随 iced scrollable
/// 即时位移,窗口内容等泵(50ms Tick);模型量化"视图超出已喂窗"的
/// 行数(空白带高)与持续帧数。预取 N 行后覆盖之——N 的取值依据。
///
/// 模型参数对齐实车:帧 60fps(17ms 取整)、泵周期 50ms、notch 3 行;
/// 两档滚轮速率:常规 30ms/notch 与快速 10ms/notch(一帧内可积 2 notch)。
#[test]
fn p025_pump_lag_model_blank_band_extent() {
    const FRAME_MS: i64 = 17; // 60fps 取整
    const PUMP_MS: i64 = 50;
    const NOTCH: i64 = 3;
    const BURST_NOTCHES: i64 = 10;

    /// 单速率推演:返回 (无预取最大空白带行数, 无预取滞后帧数,
    /// 预取 N=8 最大空白带行数, 预取滞后帧数)。
    fn run(notch_every_ms: i64) -> (i64, i64, i64, i64) {
        let mut t_view: i64 = 0; // 视图目标 offset(行)
        let mut o_eng: i64 = 0; // 引擎已确认 offset(行)
        let mut next_notch: i64 = 0;
        let mut next_pump: i64 = 0;
        let (mut max0, mut frames0, mut max8, mut frames8) = (0i64, 0i64, 0i64, 0i64);
        let end = BURST_NOTCHES * notch_every_ms + PUMP_MS + FRAME_MS;
        let mut t: i64 = 0;
        while t <= end {
            // 本帧前的滚轮与泵事件追认(时刻 ≤ t 全部生效)。
            while next_notch <= t && next_notch < BURST_NOTCHES * notch_every_ms {
                t_view += NOTCH;
                next_notch += notch_every_ms;
            }
            while next_pump <= t {
                o_eng = t_view;
                next_pump += PUMP_MS;
            }
            // draw:视图窗 [h-t_view, ..],已喂窗 [h-o_eng, ..](无预取);
            // 上翻(t_view > o_eng)时顶部超出已喂窗 = 空白带。
            let lead = (t_view - o_eng).max(0);
            if lead > 0 {
                frames0 += 1;
                max0 = max0.max(lead);
            }
            let lead_prefetch = (lead - 8).max(0);
            if lead_prefetch > 0 {
                frames8 += 1;
                max8 = max8.max(lead_prefetch);
            }
            t += FRAME_MS;
        }
        (max0, frames0, max8, frames8)
    }

    let slow = run(30);
    let fast = run(10);
    eprintln!(
        "[P025-LAG] 常规滚轮(30ms/notch): baseline max_blank={}行 frames={}; N=8: {}行 {}帧",
        slow.0, slow.1, slow.2, slow.3
    );
    eprintln!(
        "[P025-LAG] 快速滚轮(10ms/notch): baseline max_blank={}行 frames={}; N=8: {}行 {}帧",
        fast.0, fast.1, fast.2, fast.3
    );
    // 判决断言(基线):常规速率即达 1 notch 空白带且跨多帧;快速速率
    // 达 ≥2 notch(6 行)——顶/底缘空白带 + 窗口回中跳变的量级证据。
    assert!(slow.0 >= NOTCH, "常规速率基线空白带 ≥1 notch,got {}", slow.0);
    assert!(slow.1 > 0, "常规速率基线存在滞后帧");
    assert!(fast.0 >= 2 * NOTCH, "快速速率基线空白带 ≥2 notch,got {}", fast.0);
    // 预取 N=8:常规速率清零;快速速率显著收窄(极值仍可能余薄带,
    // 由泵周期兜底——T-04 即时泵的取舍依据)。
    assert_eq!((slow.2, slow.3), (0, 0), "预取 8 行覆盖常规速率泵滞后窗口");
    assert!(
        fast.2 < fast.0 && fast.3 < fast.1,
        "预取 8 行对快速速率严格收窄: {}→{} 行,{}→{} 帧",
        fast.0, fast.2, fast.1, fast.3
    );
}

/// 判决面 1 的输出流对照:贴底(o=0)时打印新行,现行槽位键同样整窗
/// 重排(全槽内容位移)。此面 T-03 后同样收敛到 1 行/新行——绝对行号
/// 键对输出流与滚动同构受益的证据位。
#[test]
fn p025_baseline_output_stream_full_window_reshape() {
    terminal_dispose("p025-stream");
    let core = terminal("p025-stream", 80, 30);
    let (rows, cols) = (30usize, 80usize);
    let mut h = 500usize;
    pump(core, "p025-stream", 0, h, rows, cols);
    assert_eq!(draw_step("p025-stream", core, rows), rows, "首帧整窗成形");
    // 打印 1 行:历史 +1,窗整体上移 1 行(槽位键全变)。
    h += 1;
    pump(core, "p025-stream", 0, h, rows, cols);
    let rebuilt = draw_step("p025-stream", core, rows);
    eprintln!("[P025-STREAM] 1 printed line → {rebuilt}/{rows} rows rebuilt");
    assert_eq!(rebuilt, rows, "现行槽位键:打印 1 行 = 整窗重排(候选 1 输出面)");
    terminal_dispose("p025-stream");
}
