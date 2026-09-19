//! PLAN-656 T-03/T-05: ScrollController 的 runtime 绑定存储。
//!
//! **层位注记**（plan r2 §5.2）：本文件不是 backend-independent 滚动语义
//! （State/Intent/Geometry 在 state.rs/intent.rs/geometry.rs）；它是 runtime
//! 私有层——controller logical handle 到 pending action queue 的落地，供
//! VM natives（`scroll_controller()` 族入队）与 backend renderer（注册表
//! 消费/排空）共用。不含任何 iced/DOM 类型。
//!
//! 数据流：
//! ```text
//! scroll_controller()          → 分配 handle（"@scrollctl:N"）
//! pane build（controller prop）→ bind(handle, widget_id)
//! on_scroll 测量               → note_state(handle, 六测量)
//! scroll_to_end(handle) 等     → enqueue(handle, ScrollIntent)
//! renderer update              → drain_intents() → resolve → scroll_to
//! ```

use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::ui::scroll::intent::{ScrollIntent, ScrollSource};
use crate::ui::scroll::state::{Axis, ScrollAxisState, ScrollState};

/// controller 句柄前缀（Value::Str 形态，DSL `let s = scroll_controller()`）。
pub const CONTROLLER_HANDLE_PREFIX: &str = "@scrollctl:";

static CONTROLLER_COUNTER: AtomicU64 = AtomicU64::new(1);

/// 分配下一个 controller 句柄字符串。
pub fn next_controller_handle() -> String {
    let n = CONTROLLER_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{CONTROLLER_HANDLE_PREFIX}{n}")
}

/// 句柄字符串校验（natives 侧弹参后验证）。
pub fn is_controller_handle(s: &str) -> bool {
    s.strip_prefix(CONTROLLER_HANDLE_PREFIX).is_some()
}

/// pane 侧最近测量快照（Viewport 六测量；progress 消费点现算）。
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ControllerPaneSnapshot {
    pub widget_id: String,
    pub offset_x: f64,
    pub offset_y: f64,
    pub viewport_w: f64,
    pub viewport_h: f64,
    pub content_w: f64,
    pub content_h: f64,
}

impl ControllerPaneSnapshot {
    /// 六测量 → 双轴 `ScrollState`（intent resolve 的输入）。
    pub fn to_scroll_state(&self) -> ScrollState {
        let x = ScrollAxisState {
            offset: self.offset_x,
            viewport_extent: self.viewport_w,
            content_extent: self.content_w,
        };
        let y = ScrollAxisState {
            offset: self.offset_y,
            viewport_extent: self.viewport_h,
            content_extent: self.content_h,
        };
        ScrollState {
            x: Some(x),
            y: Some(y),
        }
    }
}

struct ControllerRegistry {
    panes: HashMap<String, ControllerPaneSnapshot>,
    intents: Vec<(String, ScrollIntent)>,
    /// PLAN-656 F-4：新绑定 handle 的测量预热请求（update 期取走 → 推
    /// ScrollStateReader 读回操作补首用基线）。
    prime_requests: Vec<String>,
}

lazy_static::lazy_static! {
    static ref REGISTRY: Mutex<ControllerRegistry> = Mutex::new(ControllerRegistry {
        panes: HashMap::new(),
        intents: Vec::new(),
        prime_requests: Vec::new(),
    });
}

/// pane build 期绑定：handle → 稳定 widget id（renderer 在 build_scrollable
/// 有 controller 时调用；绑定早于首次测量，snapshot 其余字段为零）。
pub fn bind_controller(handle: &str, widget_id: &str) {
    let mut reg = REGISTRY.lock().unwrap();
    // 新绑定或换绑：请求一次测量预热（稳定句柄重建不重复请求）。
    if reg.panes.get(handle).map(|s| s.widget_id.as_str()) != Some(widget_id) {
        reg.prime_requests.push(handle.to_string());
    }
    let entry = reg.panes.entry(handle.to_string()).or_default();
    entry.widget_id = widget_id.to_string();
}

/// 读出臂测量更新（on_scroll 事件携带的六测量写入快照）。
pub fn note_controller_state(
    handle: &str,
    offset: (f64, f64),
    viewport: (f64, f64),
    content: (f64, f64),
) {
    let mut reg = REGISTRY.lock().unwrap();
    let entry = reg.panes.entry(handle.to_string()).or_default();
    entry.offset_x = offset.0;
    entry.offset_y = offset.1;
    entry.viewport_w = viewport.0;
    entry.viewport_h = viewport.1;
    entry.content_w = content.0;
    entry.content_h = content.1;
}

/// natives 侧：controller 方法 → pending intent 入队（axis 缺省 Y——
/// 单轴 pane 的简写；双轴 pane 显式轴见 natives 族签名）。
pub fn enqueue_intent(handle: &str, intent: ScrollIntent) {
    let mut reg = REGISTRY.lock().unwrap();
    reg.intents.push((handle.to_string(), intent));
}

/// renderer update 期排空：同 handle 的 intent 序列对快照状态顺序折叠成
/// 双轴终态 offset，产出 (widget_id, x, y)。无绑定/无测量的 handle 静默
/// 丢弃（pane 未 build 或尚未布局——controller 允许先于 pane 使用，非错误）。
/// 快照是否已有测量基线（F-4：viewport 无测量 = 未预热）。
fn snapshot_primed(s: &ControllerPaneSnapshot) -> bool {
    s.viewport_w > 0.0 || s.viewport_h > 0.0
}

/// 取走预热请求（update 期消费——推读回操作）。
pub fn take_prime_requests() -> Vec<String> {
    let mut reg = REGISTRY.lock().unwrap();
    std::mem::take(&mut reg.prime_requests)
}

/// 反查：widget id → 持有该绑定 id 的全部 handle（读回结果落库寻址）。
pub fn handles_for_widget(widget_id: &str) -> Vec<String> {
    let reg = REGISTRY.lock().unwrap();
    reg.panes
        .iter()
        .filter(|(_, s)| s.widget_id == widget_id)
        .map(|(h, _)| h.clone())
        .collect()
}

/// drain 消费端语义（F-4 后）：已预热 handle 的 intent 序列折叠为双轴终态
/// scroll_to；**未预热 handle 的 intents 留队**（返回 prime 名单——下一
/// tick 读回补基线后 drain 正常解析，首用动作晚一拍而非解析为 0）。
pub fn drain_resolved_intents() -> (Vec<(String, String, f64, f64)>, Vec<String>) {
    let mut reg = REGISTRY.lock().unwrap();
    let queued = std::mem::take(&mut reg.intents);
    // handle → 终态聚合（同帧多条 intent 顺序应用；跨轴独立叠加）。
    let mut folded: HashMap<String, (f64, f64)> = HashMap::new();
    let mut order: Vec<String> = Vec::new();
    let mut prime: Vec<String> = Vec::new();
    for (handle, intent) in queued {
        let Some(snapshot) = reg.panes.get(&handle) else {
            continue;
        };
        if snapshot.widget_id.is_empty() {
            continue;
        }
        // F-4：未预热——intent 回队（保持顺序），handle 进 prime 名单。
        if !snapshot_primed(snapshot) {
            reg.intents.push((handle.clone(), intent));
            if !prime.contains(&handle) {
                prime.push(handle.clone());
            }
            continue;
        }
        // 每条 intent 对「当前聚合态」resolve（首条用快照基线）。
        let (bx, by) = folded
            .get(&handle)
            .copied()
            .unwrap_or((snapshot.offset_x, snapshot.offset_y));
        let mut state = snapshot.to_scroll_state();
        if let Some(x) = state.x.as_mut() {
            x.offset = bx;
        }
        if let Some(y) = state.y.as_mut() {
            y.offset = by;
        }
        let resolved = intent.resolve(&state);
        let (nx, ny) = match resolved.axis {
            Axis::X => (resolved.offset, by),
            Axis::Y => (bx, resolved.offset),
        };
        if !folded.contains_key(&handle) {
            order.push(handle.clone());
        }
        folded.insert(handle, (nx, ny));
    }
    let finals: Vec<(String, String, f64, f64)> = order
        .into_iter()
        .filter_map(|handle| {
            let (x, y) = folded.get(&handle).copied()?;
            let widget_id = reg.panes.get(&handle)?.widget_id.clone();
            Some((handle, widget_id, x, y))
        })
        .collect();
    (finals, prime)
}

/// 单轴 pane 语义辅助：唯一启用轴（natives 的 axis 简写与 pane 配置对齐
/// 由调用面负责；此处只提供 Y 缺省）。
pub fn default_axis() -> Axis {
    Axis::Y
}

/// 读出某句柄的最近快照（未绑定 → 全零默认；`scroll_state()` native 与
/// capability 验证断言面消费）。
pub fn controller_snapshot(handle: &str) -> ControllerPaneSnapshot {
    REGISTRY
        .lock()
        .unwrap()
        .panes
        .get(handle)
        .cloned()
        .unwrap_or_default()
}

/// 读回通道的 extent-only 落库（F-4 终段）：offset 语义单源纪律——注册表
/// offset 由写臂回填/on_scroll 回声/用户滚动维护，读回只补 viewport/content
/// 基线（读回 offset 与注册表同源同号已修正，v1 保留 extent-only 是
/// 投影链已闭合 + 避免排空同帧的 pre-scroll 读值回写竞态，非读不准）。
pub fn note_controller_extents(handle: &str, viewport: (f64, f64), content: (f64, f64)) {
    let mut reg = REGISTRY.lock().unwrap();
    if let Some(entry) = reg.panes.get_mut(handle) {
        entry.viewport_w = viewport.0;
        entry.viewport_h = viewport.1;
        entry.content_w = content.0;
        entry.content_h = content.1;
    }
}

/// drain 消费端回填：controller 意图落盘 scroll_to 后，把已解析的双轴目标
/// offset 写回注册表投影（iced 程序化 scroll_to 不触发 on_scroll 回声——
/// 043 写臂同款盲区；用户滚动后 on_scroll 测量会校正 viewport/content）。
pub fn note_controller_offset(handle: &str, offset_x: f64, offset_y: f64) {
    let mut reg = REGISTRY.lock().unwrap();
    if let Some(entry) = reg.panes.get_mut(handle) {
        entry.offset_x = offset_x;
        entry.offset_y = offset_y;
    }
}

/// 测试/调试面：intent 队列长度（不消费）。
#[cfg(test)]
pub fn pending_intent_count() -> usize {
    REGISTRY.lock().unwrap().intents.len()
}

/// 测试面：清空注册表（测试隔离用）。
#[cfg(test)]
pub fn reset_for_test() {
    let mut reg = REGISTRY.lock().unwrap();
    reg.panes.clear();
    reg.intents.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handle_allocates_and_validates() {
        let h1 = next_controller_handle();
        let h2 = next_controller_handle();
        assert_ne!(h1, h2);
        assert!(is_controller_handle(&h1));
        assert!(!is_controller_handle("random"));
    }

    #[test]
    fn bind_note_enqueue_drain_round_trip() {
        reset_for_test();
        let h = next_controller_handle();
        bind_controller(&h, "pane_a");
        // F-4：未预热（无测量基线）→ intent 留队 + prime 名单（不再解析为 0）。
        assert!(!take_prime_requests().is_empty(), "new binding requests priming");
        assert!(take_prime_requests().is_empty(), "stable rebuild does not re-request");
        enqueue_intent(
            &h,
            ScrollIntent::ToEnd {
                axis: Axis::Y,
                source: ScrollSource::Programmatic,
            },
        );
        let (drained, prime) = drain_resolved_intents();
        assert!(drained.is_empty(), "unprimed handle defers resolution");
        assert_eq!(prime.as_slice(), [h.clone()], "handle flagged for priming");
        assert_eq!(pending_intent_count(), 1, "intent stays queued");

        note_controller_state(&h, (0.0, 100.0), (300.0, 200.0), (500.0, 1000.0));
        enqueue_intent(
            &h,
            ScrollIntent::ScrollBy {
                axis: Axis::Y,
                delta: 150.0,
                source: ScrollSource::Programmatic,
            },
        );
        enqueue_intent(
            &h,
            ScrollIntent::ToEnd {
                axis: Axis::Y,
                source: ScrollSource::Programmatic,
            },
        );
        let (drained, prime) = drain_resolved_intents();
        assert!(prime.is_empty(), "primed handle resolves immediately");
        assert_eq!(
            drained.len(),
            1,
            "same-handle intents fold into one final state"
        );
        assert_eq!(drained[0].3, 800.0); // range 1000-200（第二条覆盖第一条）
        assert_eq!(drained[0].2, 0.0); // x 轴不受 y intent 影响
        assert_eq!(pending_intent_count(), 0, "drain empties queue");

        // 双轴独立叠加：x by → x 终态变化、y 保持。
        note_controller_state(&h, (10.0, 0.0), (300.0, 200.0), (1000.0, 1000.0));
        enqueue_intent(
            &h,
            ScrollIntent::ScrollBy {
                axis: Axis::X,
                delta: 90.0,
                source: ScrollSource::Programmatic,
            },
        );
        enqueue_intent(
            &h,
            ScrollIntent::ScrollBy {
                axis: Axis::Y,
                delta: 50.0,
                source: ScrollSource::Programmatic,
            },
        );
        let (drained, _) = drain_resolved_intents();
        assert_eq!(drained[0].2, 100.0);
        assert_eq!(drained[0].3, 50.0);
    }

    #[test]
    fn unbound_handle_silently_dropped() {
        reset_for_test();
        enqueue_intent(
            "@scrollctl:999",
            ScrollIntent::ToStart {
                axis: Axis::Y,
                source: ScrollSource::Programmatic,
            },
        );
        let (drained, _) = drain_resolved_intents();
        assert!(drained.is_empty());
        reset_for_test();
    }
}
