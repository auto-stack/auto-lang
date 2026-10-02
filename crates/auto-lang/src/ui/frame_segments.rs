// PLAN-725 T-00：键入帧五段成本链分段探针（S1 消息载荷/S2 VM 段/
// S3a MCP 同步块建/S3b 主重建/Element 段 S4）——`[P725-FRAME]` 行随帧吐出，
// 供阶梯谱脚本分段归因（改前基线/改后对照同一通道）。
//
// 通道语义（沿 PLAN-716 组B frame_bench 门控纪律）：
// - **门控零开销**：`AUTO_FRAME_BENCH=1` 与 frame_bench 同门（复用其
//   `enabled()` 读数）；未设=所有 note 系列零分支零写入。
// - **帧生命周期（prev/cur 双槽配对）**：`frame_begin`（update 入口）把
//   cur 收纳进 prev（prev 未发布且有数据 → 以 `present=-1` 孤儿行落账，
//   fall-through 帧成本由此显形，T-00① 勘定面）；`frame_present`（__frame_
//   pump 消费点）发布 prev 并清零。泵消费序=begin→present（同一 update
//   内），故脏帧分段先被收纳进 prev、紧随的 present 行配对发布；无泵帧
//   （如非脏 fall-through 自身）滞留 prev 至被覆写 → 孤儿行。
// - **残差=layout+draw**：S5 不插桩（iced 内部段无公开钩子）——分析侧以
//   `total=present-begin` 减 S1..S4 得残差（含 iced 内务开销，归因上界口径）。
// - **线程域**：update/view 同主线程——thread_local 累加，无锁。
//
// 段定义（与 plan §5 T-00 对齐）：
// - S1 消息载荷：code_editor on_change 闭包全文 clone（O(doc)）。
// - S2 VM 段：update_inner 通用派发（on_with_input_for 解释 handler）。
// - S3a builder×2 之 MCP 同步块：view_with_debug_gated + read_all_state_
//   materialized + vtree 转换（MCP 活跃脏帧的第一遍全量建）。
// - S3b builder 主重建：脏帧主建（view_with_debug_gated→convert_view_
//   messages→缓存写回）或非脏 fall-through（cached_converted_view 深克隆）。
// - S4 Element 段：render_dynamic_view 全树 Element 新建。

use std::cell::RefCell;

/// 单帧分段累加器。所有时长单位=微秒。
#[derive(Default, Debug)]
struct Segments {
    begin_ms: i64,
    s1_payload_us: u64,
    s2_vm_us: u64,
    s3a_mcp_us: u64,
    s3b_build_us: u64,
    s4_element_us: u64,
    builds: u32,
    dirty: Option<bool>,
}

thread_local! {
    static PAIR: RefCell<(Option<Segments>, Option<Segments>)> = const { RefCell::new((None, None)) };
    //                                          prev            cur
}

fn enabled() -> bool {
    super::frame_bench::segments_gate()
}

fn elapsed_ms() -> i64 {
    super::frame_bench::process_elapsed_ms()
}

fn with_current<R>(f: impl FnOnce(&mut Segments) -> R) -> Option<R> {
    if !enabled() {
        return None;
    }
    PAIR.with(|c| {
        let mut slot = c.borrow_mut();
        let (_, cur) = &mut *slot;
        let seg = cur.get_or_insert_with(|| Segments {
            begin_ms: elapsed_ms(),
            ..Default::default()
        });
        Some(f(seg))
    })
}

/// 帧开始（update 入口；与 frame_bench::note_frame_begin 同点调用）。
/// cur 收纳进 prev；prev 若滞留未发布分段（无泵呈现帧），以 present=-1
/// 孤儿行落账后被覆写。
pub fn frame_begin() {
    if !enabled() {
        return;
    }
    let now = elapsed_ms();
    PAIR.with(|c| {
        let mut slot = c.borrow_mut();
        let (prev, cur) = &mut *slot;
        let incoming = cur.take();
        if let Some(stale) = prev.take() {
            if stale.has_data() {
                emit(stale, -1);
            }
        }
        *prev = incoming;
        *cur = Some(Segments {
            begin_ms: now,
            ..Default::default()
        });
    });
}

/// 呈现完成（__frame_pump 消费点；与 frame_bench::note_frame_present 同点）。
/// 发布 prev（泵序 begin→present——prev 恰为上一完成帧的分段）。
pub fn frame_present() {
    if !enabled() {
        return;
    }
    let now = elapsed_ms();
    PAIR.with(|c| {
        let mut slot = c.borrow_mut();
        let (prev, _) = &mut *slot;
        if let Some(done) = prev.take() {
            if done.has_data() {
                emit(done, now);
            }
        }
    });
}

impl Segments {
    fn has_data(&self) -> bool {
        self.s1_payload_us > 0
            || self.s2_vm_us > 0
            || self.s3a_mcp_us > 0
            || self.s3b_build_us > 0
            || self.s4_element_us > 0
    }
}

fn emit(s: Segments, present_ms: i64) {
    let dirty = match s.dirty {
        Some(true) => 1,
        Some(false) => 0,
        None => -1,
    };
    eprintln!(
        "[P725-FRAME] begin={} present={} s1_payload_us={} s2_vm_us={} s3a_mcp_us={} s3b_build_us={} s4_element_us={} builds={} dirty={}",
        s.begin_ms,
        present_ms,
        s.s1_payload_us,
        s.s2_vm_us,
        s.s3a_mcp_us,
        s.s3b_build_us,
        s.s4_element_us,
        s.builds,
        dirty
    );
}

// ── 分段打点（门关=即时返回，零开销） ────────────────────────────────

/// S1：on_change 消息载荷构造耗时（全文 clone 形）。
pub fn note_s1_payload(elapsed: std::time::Duration) {
    with_current(|s| s.s1_payload_us += elapsed.as_micros() as u64);
}

/// S2：VM 段（通用派发→handler 解释）耗时。
pub fn note_s2_vm(elapsed: std::time::Duration) {
    with_current(|s| s.s2_vm_us += elapsed.as_micros() as u64);
}

/// S3a：MCP 同步块第一遍全量建耗时（含就地自建）。
pub fn note_s3a_mcp(elapsed: std::time::Duration) {
    with_current(|s| {
        s.s3a_mcp_us += elapsed.as_micros() as u64;
        s.builds += 1;
    });
}

/// S3a（T-01 复用臂）：同步块耗时但**无建**（复用主重建产物——不计
/// builds，单帧单建断言的口径支撑）。
pub fn note_s3a_mcp_nobuild(elapsed: std::time::Duration) {
    with_current(|s| s.s3a_mcp_us += elapsed.as_micros() as u64);
}

/// S3b：主重建（脏帧模板建 or 非脏 fall-through 克隆）耗时。
pub fn note_s3b_build(elapsed: std::time::Duration) {
    with_current(|s| {
        s.s3b_build_us += elapsed.as_micros() as u64;
        s.builds += 1;
    });
}

/// S3b 克隆-only 形态（fall-through 帧 cached_converted_view 深克隆）——
/// 计入 s3b 但不计 build 次数（无 view_with_debug_gated 调用）。
pub fn note_s3b_clone(elapsed: std::time::Duration) {
    with_current(|s| s.s3b_build_us += elapsed.as_micros() as u64);
}

/// S4：Element 段（render_dynamic_view 全树新建）耗时。
pub fn note_s4_element(elapsed: std::time::Duration) {
    with_current(|s| s.s4_element_us += elapsed.as_micros() as u64);
}

/// 帧脏标记（view() 主路径取 view_dirty 时调用）。
pub fn note_dirty(dirty: bool) {
    with_current(|s| s.dirty = Some(dirty));
}

/// T-00 勘定子探针：Element 臂内部计时行（`[P725-ARM]`——不进分段累加器，
/// 独立行吐出，供勘定期钻取；收口时评估去留）。
pub fn arm_probe(name: &str, elapsed: std::time::Duration) {
    if !enabled() {
        return;
    }
    eprintln!("[P725-ARM] {} us={}", name, elapsed.as_micros());
}

thread_local! {
    static ARM_ACC: RefCell<std::collections::BTreeMap<&'static str, (u64, u64)>> =
        const { RefCell::new(std::collections::BTreeMap::new()) };
}

/// T-00 勘定：render_dynamic_view 每臂累积耗时（µs, 次数）——s4 发布点随行
/// 吐出（`[P725-ARMS]`），定位 Element 段内 O(n) 臂。
pub fn arm_acc(name: &'static str, elapsed: std::time::Duration) {
    if !enabled() {
        return;
    }
    ARM_ACC.with(|m| {
        m.borrow_mut()
            .entry(name)
            .and_modify(|(us, n)| {
                *us += elapsed.as_micros() as u64;
                *n += 1;
            })
            .or_insert((elapsed.as_micros() as u64, 1));
    });
}

/// s4 发布点：吐出并清零臂累积表。
pub fn flush_arms() {
    if !enabled() {
        return;
    }
    ARM_ACC.with(|m| {
        let mut map = m.borrow_mut();
        if map.is_empty() {
            return;
        }
        let parts: Vec<String> = map
            .iter()
            .map(|(k, (us, n))| format!("{}={}/{}", k, us, n))
            .collect();
        eprintln!("[P725-ARMS] {}", parts.join(" "));
        map.clear();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 门关（默认测试进程未设 AUTO_FRAME_BENCH）：打点为 no-op，
    /// 累加器不建条目（零开销路径的结构断言）。
    #[test]
    fn gate_off_no_accumulation() {
        if std::env::var("AUTO_FRAME_BENCH").is_ok() {
            return; // 门开环境跳过（探针族负责开态）
        }
        note_s1_payload(std::time::Duration::from_micros(5));
        note_s2_vm(std::time::Duration::from_micros(5));
        frame_begin();
        frame_present();
        PAIR.with(|c| assert!(c.borrow().0.is_none() && c.borrow().1.is_none()));
    }
}
