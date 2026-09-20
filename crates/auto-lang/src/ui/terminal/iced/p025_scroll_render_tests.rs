//! PLAN-025 T-01 判决工件 / T-03 修复断言:滚动闪屏机理的 headless 复现器。
//!
//! 复现链与实车逐层同构:
//! - 引擎侧:不可变 scrollback 模型(伪引擎;line id = 绝对行号,同 id
//!   恒同内容——对应 alacritty_terminal 0.26 不可变滚动存储);
//! - 泵侧(term.rs feed_snapshot_inner 同构):按 display_offset 重采视口
//!   窗投喂(terminal_feed_cells_for 逐行)+ offset/history/绝对行锚回写
//!   + 视口行入 window_store(T-02/T-03);
//! - draw 侧:refresh_row_cache 真码步进(重排计数埋点;行 id 寻址)。
//!
//! T-01 基线判决(槽位键:每 notch 整窗重排;泵滞后空白带 3-9 行×多帧)
//! 见 docs/plans/evidence/025/t01-verdict.md。T-03 后断言翻转:滚动重排
//! 收敛到"新暴露行数"量级(AC-01 headless 门)。
//!
//! Run: cargo test -p auto-lang --features ui-iced,iced-layout-tests --lib p025 -- --nocapture
#![cfg(all(test, feature = "iced-layout-tests"))]

use crate::ui::terminal::iced::widget::{refresh_row_cache, row_rebuilds_reset, VisibleRow, CELL_H};
use crate::ui::terminal::{
    terminal, terminal_dispose, terminal_feed_cells_for, terminal_feed_window_for,
    terminal_mark_view_bound, terminal_queue_scroll_delta, terminal_set_history,
    terminal_set_scroll_offset, terminal_set_window_anchor, terminal_take_scroll_delta,
    terminal_window_anchor, terminal_window_row, terminal_scroll_delta_pending, TermCell,
    TerminalCore, WINDOW_ANCHOR_UNSET,
};

/// 伪引擎行内容:行 id 即内容指纹(同 id 恒同文本)。
fn fake_line_cells(id: i64, cols: usize) -> Vec<TermCell> {
    let text = format!("L{id:06} the quick brown fox jumps over");
    text.chars().take(cols).map(TermCell::plain).collect()
}

/// 泵步进(term.rs feed_snapshot_inner 同构):o = 引擎 display_offset,
/// h = 引擎历史行数;重采视口窗 [h-o, h-o+rows) 逐行投喂 + 状态回写
/// (offset/history/T-02 绝对行锚)+ 视口行入 window_store(T-03)。
/// `prefetch_above`:额外把 [h-o-N, h-o) 预取行入 store(泵瞬态 scroll
/// 采样的同构面;0 = 关)。
fn pump(core: &TerminalCore, key: &str, o: usize, h: usize, rows: usize, cols: usize, prefetch_above: usize) {
    let anchor = h as i64 - o as i64;
    let mut window = Vec::with_capacity(rows);
    for y in 0..rows {
        let id = anchor + y as i64;
        let cells = fake_line_cells(id, cols);
        terminal_feed_cells_for(key, y, cells.clone());
        window.push(cells);
    }
    terminal_set_scroll_offset(core, o);
    terminal_set_history(core, h);
    terminal_set_window_anchor(core, anchor);
    terminal_feed_window_for(key, anchor, &window);
    if prefetch_above > 0 {
        let above: Vec<Vec<TermCell>> = (0..prefetch_above as i64)
            .rev()
            .map(|k| fake_line_cells(anchor - 1 - k, cols))
            .collect();
        terminal_feed_window_for(key, anchor - prefetch_above as i64, &above);
    }
}

/// draw 期缓存步进(真码;行 id 寻址同 draw):返回本帧行 Paragraph 重建
/// 数。`view_top_id` = 视口顶绝对行(锚定臂;None = 槽位回退臂)。
fn draw_step(key: &str, core: &TerminalCore, rows: usize, view_top_id: Option<i64>) -> usize {
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
    let anchor = terminal_window_anchor(core);
    let visible: Vec<VisibleRow> = match view_top_id {
        Some(top) => (top..top + rows as i64)
            .filter_map(|id| {
                if let Some(r) = terminal_window_row(core, id) {
                    Some(VisibleRow {
                        id,
                        y_px: id as f32 * CELL_H,
                        cells: r.cells,
                        digest: r.digest,
                    })
                } else if id >= anchor && (id - anchor) < cells.len() as i64 {
                    let y = (id - anchor) as usize;
                    Some(VisibleRow {
                        id,
                        y_px: id as f32 * CELL_H,
                        cells: cells[y].clone(),
                        digest: digests[y],
                    })
                } else {
                    None
                }
            })
            .collect(),
        None => cells
            .iter()
            .enumerate()
            .map(|(y, line)| VisibleRow {
                id: y as i64,
                y_px: 0.0,
                cells: line.clone(),
                digest: digests[y],
            })
            .collect(),
    };
    row_rebuilds_reset();
    refresh_row_cache(key, &visible, &palette, pal_key)
}

/// 基线:静止零重排(digest 门控有效性;非滚动帧零扰动)。
#[test]
fn p025_static_frame_zero_rebuild() {
    terminal_dispose("p025-static");
    let core = terminal("p025-static", 80, 30);
    let (rows, cols, h) = (30usize, 80usize, 500usize);
    pump(core, "p025-static", 0, h, rows, cols, 0);
    let first = draw_step("p025-static", core, rows, Some(h as i64));
    assert_eq!(first, rows, "首帧整窗成形");
    pump(core, "p025-static", 0, h, rows, cols, 0);
    assert_eq!(draw_step("p025-static", core, rows, Some(h as i64)), 0, "静止帧零重排");
    terminal_dispose("p025-static");
}

/// AC-01 headless 门:滚轮连续回滚 K notch,行 Paragraph 重建数收敛到
/// "新暴露行数"量级(每 notch ≤ notch 行数 + 预取增量;T-01 基线为每
/// notch 整窗 rows 行重排——判决 evidence/025/t01-verdict.md)。
#[test]
fn p025_scroll_rebuild_curve_converges() {
    terminal_dispose("p025-curve");
    let core = terminal("p025-curve", 80, 30);
    let (rows, cols, h) = (30usize, 80usize, 500usize);
    let notch = 3usize;
    let mut o = 0usize;
    pump(core, "p025-curve", o, h, rows, cols, 0);
    let first = draw_step("p025-curve", core, rows, Some((h - o) as i64));
    assert_eq!(first, rows, "首帧整窗成形");
    let mut curve = vec![first];
    for _ in 0..10 {
        o += notch;
        pump(core, "p025-curve", o, h, rows, cols, 0);
        curve.push(draw_step("p025-curve", core, rows, Some((h - o) as i64)));
    }
    eprintln!("[P025-CURVE] anchored per-notch rebuilds: {curve:?} (rows={rows}, notch={notch})");
    assert!(
        curve[1..].iter().all(|&n| n <= notch),
        "绝对行号键:每 notch 重排 ≤ 新暴露行数({notch}),got {curve:?}"
    );
    assert!(
        curve[1..].iter().any(|&n| n > 0),
        "每 notch 应有新暴露行重排(曲线非全零——数据面在动)"
    );
    terminal_dispose("p025-curve");
}

/// 输出流对照:贴底打印新行 → 重排收敛到 1 行(T-01 基线:整窗 30/30)。
#[test]
fn p025_output_stream_single_row_rebuild() {
    terminal_dispose("p025-stream");
    let core = terminal("p025-stream", 80, 30);
    let (rows, cols) = (30usize, 80usize);
    let mut h = 500usize;
    pump(core, "p025-stream", 0, h, rows, cols, 0);
    assert_eq!(draw_step("p025-stream", core, rows, Some(h as i64)), rows);
    h += 1;
    pump(core, "p025-stream", 0, h, rows, cols, 0);
    let rebuilt = draw_step("p025-stream", core, rows, Some(h as i64));
    eprintln!("[P025-STREAM] 1 printed line → {rebuilt}/{rows} rows rebuilt");
    assert_eq!(rebuilt, 1, "绝对行号键:打印 1 行仅新行重排");
    terminal_dispose("p025-stream");
}

/// 预取窗:视图先行(泵未追上)时,上方预取行自 store 覆盖——无空白
/// 行,且重排仍只发生在首次见到的新 id(时序收益:与视图位移同帧,
/// 不等泵)。
#[test]
fn p025_prefetch_covers_view_ahead_of_engine() {
    terminal_dispose("p025-prefetch");
    let core = terminal("p025-prefetch", 80, 30);
    let (rows, cols, h) = (30usize, 80usize, 500usize);
    let o0 = 0usize;
    pump(core, "p025-prefetch", o0, h, rows, cols, 8);
    assert_eq!(draw_step("p025-prefetch", core, rows, Some(h as i64)), rows);
    // 用户滚轮上翻 5 行(引擎尚未泵,视图先行);上方预取 8 行覆盖。
    let k = 5i64;
    let view_top = h as i64 - (o0 as i64 + k);
    let covered: usize = (view_top..view_top + rows as i64)
        .filter(|id| terminal_window_row(core, *id).is_some())
        .count();
    assert_eq!(covered, rows, "预取 8 行覆盖 5 行先行:视口无空白行");
    let rebuilt = draw_step("p025-prefetch", core, rows, Some(view_top));
    assert_eq!(
        rebuilt, k as usize,
        "先行帧重排 = 首见新 id 数(k 行),其余全复用"
    );
    // 同视图重绘零重排;泵追上后(o=k)再绘仍零重排(同 id 集)。
    assert_eq!(draw_step("p025-prefetch", core, rows, Some(view_top)), 0);
    pump(core, "p025-prefetch", k as usize, h, rows, cols, 8);
    assert_eq!(draw_step("p025-prefetch", core, rows, Some(view_top)), 0);
    terminal_dispose("p025-prefetch");
}

/// 未锚回退臂(vm/desktop 泵臂):槽位语义零回归(行为对齐 T-01 前的
/// draw——静止零重排、内容变化按槽重建,不崩不劣化;行源 = core 槽位,
/// 锚/store 不参与)。
#[test]
fn p025_unanchored_fallback_slot_semantics() {
    terminal_dispose("p025-fallback");
    let core = terminal("p025-fallback", 80, 30);
    let (rows, cols, h) = (30usize, 80usize, 500usize);
    // vm 臂泵:只喂槽位 + 回写 offset/history,不写锚(T-02 未覆盖面)。
    for y in 0..rows {
        terminal_feed_cells_for("p025-fallback", y, fake_line_cells(h as i64 + y as i64, cols));
    }
    terminal_set_scroll_offset(core, 0);
    terminal_set_history(core, h);
    assert_eq!(terminal_window_anchor(core), WINDOW_ANCHOR_UNSET, "泵未写锚 = 未锚");
    assert_eq!(draw_step("p025-fallback", core, rows, None), rows, "首帧整窗成形(槽位臂)");
    assert_eq!(draw_step("p025-fallback", core, rows, None), 0, "静止零重排");
    // 内容变化按槽重建(槽位语义面仍在)。
    terminal_feed_cells_for("p025-fallback", 3, fake_line_cells(9_999_999, cols));
    assert_eq!(draw_step("p025-fallback", core, rows, None), 1, "单槽变化单槽重建");
    terminal_dispose("p025-fallback");
}

/// 锚/存储 core glue:锚缺省哨兵、回写读出、store 读数与内容往返。
#[test]
fn p025_anchor_and_store_roundtrip() {
    terminal_dispose("p025-glue");
    let core = terminal("p025-glue", 80, 30);
    assert_eq!(terminal_window_anchor(core), WINDOW_ANCHOR_UNSET);
    terminal_set_window_anchor(core, 4711);
    assert_eq!(terminal_window_anchor(core), 4711);
    // store 喂入/读出/覆盖(重喂同 id 覆盖)。
    let cells = fake_line_cells(4711, 80);
    terminal_feed_window_for("p025-glue", 4711, &[cells.clone()]);
    let got = terminal_window_row(core, 4711).expect("store 命中");
    assert_eq!(got.cells.len(), cells.len());
    assert_eq!(got.cells.iter().map(|c| c.ch).collect::<String>().len(), cells.len());
    assert!(terminal_window_row(core, 9999).is_none(), "未喂 id = None");
    // 几何替换(terminal() 同 key 新几何)承载锚(泵回写不丢)。
    let core2 = terminal("p025-glue", 100, 40);
    assert_eq!(terminal_window_anchor(core2), 4711, "几何替换后锚保持");
    terminal_dispose("p025-glue");
}

/// T-05 实机复测修的 bind 防回拉语义(滚动条抖动根修):用户滚动回灌
/// (泵 delta≠0 → mark_view_bound)后,bind 写臂不得再 scroll_to 行量化
/// 回拉;外部源 offset 变化(不标)仍正常触发 bind。iced 滚轮 60px/notch
/// ≠ CELL_H 行距——回拉 = 每拍 4px 抖动(2026-09-20 用户实机实录)。
#[test]
fn p025_user_scroll_marks_bound_external_still_binds() {
    use crate::ui::terminal::iced::widget::Terminal;
    terminal_dispose("p025-bind");
    let core = terminal("p025-bind", 80, 30);
    terminal_set_history(core, 500);
    // 用户滚动路径:泵排水 → 引擎 offset → set_scroll_offset → 标 bound。
    terminal_set_scroll_offset(core, 64);
    terminal_mark_view_bound(core);
    assert_eq!(
        Terminal::<u8>::bind_request_y(core),
        None,
        "用户滚动回灌后视图已知位:bind 不得回拉(防抖)"
    );
    // 外部源(键入贴底/程序滚动):set_scroll_offset 不标 bound。
    terminal_set_scroll_offset(core, 40);
    let y = Terminal::<u8>::bind_request_y(core).expect("外部变化必须 bind 跟随");
    assert_eq!(y, (500 - 40) as f32 * CELL_H);
    // 绑定后回声抑制挂起(022 语义不变)。
    assert!(core.scroll_bind_suppress_pending());
    assert_eq!(Terminal::<u8>::bind_request_y(core), None, "已绑定去重");
    terminal_dispose("p025-bind");
}

/// 判决面 2(候选 2):泵滞后空白带的时间线模型(T-01 判决工件留档;
/// N=8 取值依据)。模型参数对齐实车:帧 60fps(17ms 取整)、泵周期
/// 50ms、notch 3 行;两档滚轮速率:常规 30ms/notch 与快速 10ms/notch。
#[test]
fn p025_pump_lag_model_blank_band_extent() {
    const FRAME_MS: i64 = 17; // 60fps 取整
    const PUMP_MS: i64 = 50;
    const NOTCH: i64 = 3;
    const BURST_NOTCHES: i64 = 10;

    /// 单速率推演:返回 (无预取最大空白带行数, 无预取滞后帧数,
    /// 预取 prefetch_n 最大空白带行数, 预取滞后帧数)。T-04 落地后
    /// 滚动期泵周期 = 16ms(节拍门),预取 N=24(见 term.rs)。
    fn run(notch_every_ms: i64) -> (i64, i64, i64, i64) {
        run_with(notch_every_ms, PUMP_MS, 8)
    }
    fn run_with(notch_every_ms: i64, pump_ms: i64, prefetch_n: i64) -> (i64, i64, i64, i64) {
        let mut t_view: i64 = 0; // 视图目标 offset(行)
        let mut o_eng: i64 = 0; // 引擎已确认 offset(行)
        let mut next_notch: i64 = 0;
        let mut next_pump: i64 = 0;
        let (mut max0, mut frames0, mut max8, mut frames8) = (0i64, 0i64, 0i64, 0i64);
        let end = BURST_NOTCHES * notch_every_ms + pump_ms + FRAME_MS;
        let mut t: i64 = 0;
        while t <= end {
            // 本帧前的滚轮与泵事件追认(时刻 ≤ t 全部生效)。
            while next_notch <= t && next_notch < BURST_NOTCHES * notch_every_ms {
                t_view += NOTCH;
                next_notch += notch_every_ms;
            }
            while next_pump <= t {
                o_eng = t_view;
                next_pump += pump_ms;
            }
            // draw:视图窗 [h-t_view, ..],已喂窗 [h-o_eng, ..](无预取);
            // 上翻(t_view > o_eng)时顶部超出已喂窗 = 空白带。
            let lead = (t_view - o_eng).max(0);
            if lead > 0 {
                frames0 += 1;
                max0 = max0.max(lead);
            }
            let lead_prefetch = (lead - prefetch_n).max(0);
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
    // 由泵周期兜底——T-04 即时泵豁免的取舍依据)。
    assert_eq!((slow.2, slow.3), (0, 0), "预取 8 行覆盖常规速率泵滞后窗口");
    assert!(
        fast.2 < fast.0 && fast.3 < fast.1,
        "预取 8 行对快速速率严格收窄: {}→{} 行,{}→{} 帧",
        fast.0, fast.2, fast.1, fast.3
    );
    // T-04 落地(2026-09-20 二轮实机:快拖半屏空白):滚动期泵 16ms +
    // 预取 N=24 → 两档速率空白带均清零。
    let gated_slow = run_with(30, 16, 24);
    let gated_fast = run_with(10, 16, 24);
    eprintln!(
        "[P025-LAG] 门控泵 16ms + N=24:常规 {}行 {}帧 / 快速 {}行 {}帧",
        gated_slow.2, gated_slow.3, gated_fast.2, gated_fast.3
    );
    assert_eq!((gated_slow.2, gated_slow.3), (0, 0), "门控+预取:常规速率零空白");
    assert_eq!((gated_fast.2, gated_fast.3), (0, 0), "门控+预取:快速速率零空白");
}

/// T-04 即时泵探针:queue 后 pending ≠ 0(peek 不排空);drain 后归零。
#[test]
fn p025_scroll_pending_peek_semantics() {
    terminal_dispose("p025-pending");
    let core = terminal("p025-pending", 80, 30);
    assert_eq!(terminal_scroll_delta_pending(core), 0, "初态无待排");
    terminal_queue_scroll_delta(core, 3);
    assert_eq!(terminal_scroll_delta_pending(core), 3, "peek 不排空");
    assert_eq!(terminal_take_scroll_delta(core), 3, "drain 取走");
    assert_eq!(terminal_scroll_delta_pending(core), 0, "drain 后归零");
    terminal_dispose("p025-pending");
}
