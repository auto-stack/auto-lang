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
}

lazy_static::lazy_static! {
    static ref REGISTRY: Mutex<ControllerRegistry> = Mutex::new(ControllerRegistry {
        panes: HashMap::new(),
        intents: Vec::new(),
    });
}

/// pane build 期绑定：handle → 稳定 widget id（renderer 在 build_scrollable
/// 有 controller 时调用；绑定早于首次测量，snapshot 其余字段为零）。
pub fn bind_controller(handle: &str, widget_id: &str) {
    let mut reg = REGISTRY.lock().unwrap();
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
pub fn drain_resolved_intents() -> Vec<(String, f64, f64)> {
    let mut reg = REGISTRY.lock().unwrap();
    let queued = std::mem::take(&mut reg.intents);
    // handle → 终态聚合（同帧多条 intent 顺序应用；跨轴独立叠加）。
    let mut folded: HashMap<String, (f64, f64)> = HashMap::new();
    let mut order: Vec<String> = Vec::new();
    for (handle, intent) in queued {
        let Some(snapshot) = reg.panes.get(&handle) else {
            continue;
        };
        if snapshot.widget_id.is_empty() {
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
    order
        .into_iter()
        .filter_map(|handle| {
            let (x, y) = folded.get(&handle).copied()?;
            let widget_id = reg.panes.get(&handle)?.widget_id.clone();
            Some((widget_id, x, y))
        })
        .collect()
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
        // 未测量时 resolve 仍 total（退化态 → to_end 落 0）。
        enqueue_intent(
            &h,
            ScrollIntent::ToEnd {
                axis: Axis::Y,
                source: ScrollSource::Programmatic,
            },
        );
        let drained = drain_resolved_intents();
        assert_eq!(drained.len(), 1);
        assert_eq!(drained[0].0, "pane_a");
        assert_eq!(drained[0].2, 0.0); // (x, y)：y 终态

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
        let drained = drain_resolved_intents();
        assert_eq!(
            drained.len(),
            1,
            "same-handle intents fold into one final state"
        );
        assert_eq!(drained[0].2, 800.0); // range 1000-200（第二条覆盖第一条）
        assert_eq!(drained[0].1, 0.0); // x 轴不受 y intent 影响
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
        let drained = drain_resolved_intents();
        assert_eq!(drained[0].1, 100.0);
        assert_eq!(drained[0].2, 50.0);
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
        assert!(drain_resolved_intents().is_empty());
        reset_for_test();
    }
}
