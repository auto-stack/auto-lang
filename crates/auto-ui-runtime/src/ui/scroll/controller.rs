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
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

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
    /// L0 fix(2026-10-11)：offset 来源标记——true=on_scroll 读出臂实测值
    /// （用户滚动/拖拽落点，注册表已是真源），false=写臂投影值（F-2 命令
    /// 投影，需重建后回显补落）。F-4 回显只发写臂投影：回显读臂值会把
    /// 「滚动期间快照滞后/夹取竞态」的 stale 值强行 scroll_to 回去——
    /// auto-edit diff 视图大幅滚轮后滚动条来回抖动的格斗源。
    pub offset_from_measure: bool,
    /// PLAN-781 T-01：managed content pane 标记（to_child 显式 unsupported）。
    pub managed: bool,
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
    /// PLAN-781 T-01：pane 绑定代际——bind_controller 检测到换绑（widget_id
    /// 变化）时 +1；锚点请求携带入队时代际，代际不符 → invalid:not_bound。
    generations: HashMap<String, u64>,
    /// PLAN-781 T-01：锚点请求表（(handle, key) 寻址，后写覆盖重新 pending）。
    anchor_requests: HashMap<(String, String), AnchorRequest>,
}

lazy_static::lazy_static! {
    static ref REGISTRY: Mutex<ControllerRegistry> = Mutex::new(ControllerRegistry {
        panes: HashMap::new(),
        intents: Vec::new(),
        prime_requests: Vec::new(),
        generations: HashMap::new(),
        anchor_requests: HashMap::new(),
    });
}

/// PLAN-781 T-01 冻结的锚点请求状态（`auto.scroll.anchor_state` 查询面）。
#[derive(Debug, Clone, PartialEq)]
pub enum AnchorRequestState {
    Pending,
    Applied,
    /// reason ∈ `not_found` / `not_bound` / `unsupported` / `unmeasured`。
    Invalid(String),
}

/// PLAN-781 T-01：一条锚点请求（(handle, key) 寻址）。
#[derive(Debug, Clone, PartialEq)]
pub struct AnchorRequest {
    pub generation: u64,
    pub state: AnchorRequestState,
}

/// pane build 期绑定：handle → 稳定 widget id（renderer 在 build_scrollable
/// 有 controller 时调用；绑定早于首次测量，snapshot 其余字段为零）。
/// PLAN-781 T-01：换绑（widget_id 变化）时绑定代际 +1。
pub fn bind_controller(handle: &str, widget_id: &str) {
    let mut reg = REGISTRY.lock().unwrap();
    // 新绑定或换绑：请求一次测量预热（稳定句柄重建不重复请求）。
    if reg.panes.get(handle).map(|s| s.widget_id.as_str()) != Some(widget_id) {
        reg.prime_requests.push(handle.to_string());
        let g = reg.generations.entry(handle.to_string()).or_insert(0);
        *g += 1;
    }
    let entry = reg.panes.entry(handle.to_string()).or_default();
    entry.widget_id = widget_id.to_string();
}

/// PLAN-781 T-01：pane 绑定代际读取（renderer drain 侧对照请求代际用）。
pub fn controller_generation(handle: &str) -> u64 {
    REGISTRY
        .lock()
        .unwrap()
        .generations
        .get(handle)
        .copied()
        .unwrap_or(0)
}

/// PLAN-781 T-01：managed pane 标记（renderer managed 桥 build 期调用）。
/// PLAN-781 T-05 勘定：upsert 形态——renderer 侧标记先于 build_scrollable
/// 内的首次 bind_controller（无条目即丢失标记 → to_child 回执 queued 而非
/// error:unsupported，实机矩阵 v06 锚定）；条目已存在时 bind 的 or_default
/// 保持本标记。
pub fn mark_controller_managed(handle: &str) {
    let mut reg = REGISTRY.lock().unwrap();
    reg.panes.entry(handle.to_string()).or_default().managed = true;
}

/// PLAN-781 T-01 冻结：`auto.scroll.to_child(handle, key)` 入队面。
/// 回执错误 reason：`bad_handle` / `empty_key` / `unsupported`。
/// 入队成功 → 请求置 pending（同 (handle,key) 后写覆盖重新 pending）。
pub fn enqueue_anchor_request(handle: &str, key: &str) -> Result<(), &'static str> {
    // 句柄语义与 9900 族 pop_controller_handle 等价：任意非空串合法
    //（"@scrollctl:N" 与 .at 侧裸字符串柄等价——Reader 先例）；仅空串 bad_handle。
    if handle.is_empty() {
        return Err("bad_handle");
    }
    if key.is_empty() {
        return Err("empty_key");
    }
    let mut reg = REGISTRY.lock().unwrap();
    if let Some(snap) = reg.panes.get(handle) {
        if snap.managed {
            return Err("unsupported");
        }
    }
    // handle 尚未绑定（pane 未 build）允许入队——pending 等待绑定/预热；
    // 解析期按 not_bound/未预热语义处置（§2 状态机）。
    let generation = reg.generations.get(handle).copied().unwrap_or(0);
    reg.anchor_requests.insert(
        (handle.to_string(), key.to_string()),
        AnchorRequest {
            generation,
            state: AnchorRequestState::Pending,
        },
    );
    Ok(())
}

/// PLAN-781 T-01 冻结：`auto.scroll.anchor_state(handle, key)` 查询面。
pub fn anchor_request_state(handle: &str, key: &str) -> Option<AnchorRequestState> {
    REGISTRY
        .lock()
        .unwrap()
        .anchor_requests
        .get(&(handle.to_string(), key.to_string()))
        .map(|r| r.state.clone())
}

/// PLAN-781：是否有 pending 锚点请求（renderer drain 期决定是否推采集）。
pub fn has_pending_anchor_requests() -> bool {
    REGISTRY
        .lock()
        .unwrap()
        .anchor_requests
        .values()
        .any(|r| r.state == AnchorRequestState::Pending)
}

/// PLAN-781 T-02/T-03：drain 期锚点解析（renderer 调用；决策报告 §2/§3）。
/// `anchor_bounds` = (controller key, anchor key) → pane 锚定 `(y, h)`
///（AnchorBoundsCollector 产物，pane 原点为内容原点）；`pane_origins` =
/// pane widget id → viewport 原点 `(vx, vy)`（换算 content-space target）；
/// `collected` = 本帧采集器确实跑过（false = pending 保持，不作 not_found）。
/// 返回 (handle, widget_id, x, y) 终态 offset（并入既有 finals scroll_to
/// 通路）；applied/invalid 状态就地回写。未预热 pane 的请求保持 pending
///（沿 F-4 晚一拍纪律）。
pub fn resolve_anchor_requests(
    anchor_bounds: &HashMap<(String, String), (f64, f64)>,
    pane_origins: &HashMap<String, (f64, f64)>,
    collected: bool,
) -> Vec<(String, String, f64, f64)> {
    let mut reg = REGISTRY.lock().unwrap();
    let requests: Vec<(String, String)> = reg
        .anchor_requests
        .iter()
        .filter(|(_, r)| r.state == AnchorRequestState::Pending)
        .map(|((h, k), _)| (h.clone(), k.clone()))
        .collect();
    let mut finals: Vec<(String, String, f64, f64)> = Vec::new();
    for (handle, key) in requests {
        let Some(snapshot) = reg.panes.get(&handle) else {
            // 无绑定记录 → not_bound（一次性失败）。
            if let Some(req) = reg.anchor_requests.get_mut(&(handle.clone(), key.clone())) {
                req.state = AnchorRequestState::Invalid("not_bound".into());
            }
            continue;
        };
        if snapshot.widget_id.is_empty() {
            if let Some(req) = reg.anchor_requests.get_mut(&(handle.clone(), key.clone())) {
                req.state = AnchorRequestState::Invalid("not_bound".into());
            }
            continue;
        }
        // 代际不符（入队后换绑）→ not_bound：旧请求不落到新内容。
        // PLAN-781 T-05 勘定：gen==0 = 入队于任何绑定之前（首次 bind 使
        // 0→1）——冻结合同 §2「请求首次可在布局前发出，等待当前绑定的
        // 有效布局后消费」：此类请求采用当前绑定（首绑或其后的现状），
        // 不作 not_bound（Init 先发目标 → 首绑后 applied 的实机矩阵锚定；
        // gen≥1 的请求仍严格校验换绑）。
        let current_gen = reg.generations.get(&handle).copied().unwrap_or(0);
        let req_gen = reg
            .anchor_requests
            .get(&(handle.clone(), key.clone()))
            .map(|r| r.generation)
            .unwrap_or(u64::MAX);
        if req_gen != 0 && req_gen != current_gen {
            if let Some(req) = reg.anchor_requests.get_mut(&(handle.clone(), key.clone())) {
                req.state = AnchorRequestState::Invalid("not_bound".into());
            }
            continue;
        }
        if snapshot.managed {
            if let Some(req) = reg.anchor_requests.get_mut(&(handle.clone(), key.clone())) {
                req.state = AnchorRequestState::Invalid("unsupported".into());
            }
            continue;
        }
        // 未预热（无测量基线）→ 保持 pending 晚一拍（F-4 纪律）。
        if !snapshot_primed(snapshot) {
            if !reg.prime_requests.contains(&handle) {
                reg.prime_requests.push(handle.clone());
            }
            continue;
        }
        // 无当帧采集 → 保持 pending（布局时序，非失败）。
        if !collected {
            continue;
        }
        let Some(&(pane_y, th)) = anchor_bounds.get(&(handle.clone(), key.clone())) else {
            if let Some(req) = reg.anchor_requests.get_mut(&(handle.clone(), key.clone())) {
                req.state = AnchorRequestState::Invalid("not_found".into());
            }
            continue;
        };
        // pane 锚定 y → content-space（pane viewport 原点为内容原点）。
        let Some(&(_vx, vy)) = pane_origins.get(&snapshot.widget_id) else {
            if let Some(req) = reg.anchor_requests.get_mut(&(handle.clone(), key.clone())) {
                req.state = AnchorRequestState::Invalid("not_found".into());
            }
            continue;
        };
        let ty = pane_y - vy;
        // nearest（决策报告 §3 冻结公式）——x 轴不动（v1 单轴 Y）。
        let y_axis =
            snapshot
                .to_scroll_state()
                .y
                .unwrap_or(crate::ui::scroll::state::ScrollAxisState {
                    offset: 0.0,
                    viewport_extent: 0.0,
                    content_extent: 0.0,
                });
        let nearest_y = crate::ui::scroll::geometry::nearest_child_offset(&y_axis, ty, th);
        let x = snapshot.offset_x;
        let widget_id = snapshot.widget_id.clone();
        if let Some(req) = reg.anchor_requests.get_mut(&(handle.clone(), key.clone())) {
            req.state = AnchorRequestState::Applied;
        }
        finals.push((handle.clone(), widget_id, x, nearest_y));
    }
    finals
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
    entry.offset_from_measure = true;
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
        entry.offset_from_measure = false;
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
    reg.generations.clear();
    reg.anchor_requests.clear();
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
        assert!(
            !take_prime_requests().is_empty(),
            "new binding requests priming"
        );
        assert!(
            take_prime_requests().is_empty(),
            "stable rebuild does not re-request"
        );
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

    // ---------------- PLAN-781 T-02: 锚点请求生命周期 ----------------

    fn assert_state(handle: &str, key: &str, expected: AnchorRequestState) {
        assert_eq!(
            anchor_request_state(handle, key).as_ref(),
            Some(&expected),
            "state mismatch for {handle}/{key}"
        );
    }

    #[test]
    fn plan781_anchor_enqueue_validation() {
        reset_for_test();
        // 空 handle / 空 key（非空裸串句柄合法——Reader 先例语义）。
        assert_eq!(enqueue_anchor_request("", "k"), Err("bad_handle"));
        assert_eq!(enqueue_anchor_request("@scrollctl:1", ""), Err("empty_key"));
        enqueue_anchor_request("bare-key", "k").unwrap();
        assert_eq!(
            anchor_request_state("bare-key", "k"),
            Some(AnchorRequestState::Pending)
        );
        // managed pane → unsupported（入队即拒）。
        let h = next_controller_handle();
        bind_controller(&h, "pane_m");
        mark_controller_managed(&h);
        assert_eq!(enqueue_anchor_request(&h, "k"), Err("unsupported"));
        reset_for_test();
    }

    #[test]
    fn plan781_anchor_pending_then_applied_with_nearest() {
        reset_for_test();
        let h = next_controller_handle();
        bind_controller(&h, "pane_a"); // gen=1
        take_prime_requests();
        note_controller_state(&h, (0.0, 100.0), (400.0, 200.0), (800.0, 1000.0));
        enqueue_anchor_request(&h, "p50").unwrap();
        assert_state(&h, "p50", AnchorRequestState::Pending);
        // 无采集帧 → pending 保持。
        assert!(resolve_anchor_requests(&HashMap::new(), &HashMap::new(), false).is_empty());
        assert_state(&h, "p50", AnchorRequestState::Pending);
        // 当帧采集：p50 位于 content y=500..540（底出：540 > 100+200）→
        // 底对齐 offset = 540-200 = 340。
        let bounds = HashMap::from([((h.clone(), "p50".to_string()), (515.5_f64, 40.0_f64))]);
        let origins = HashMap::from([("pane_a".to_string(), (0.0_f64, 15.5_f64))]);
        let finals = resolve_anchor_requests(&bounds, &origins, true);
        assert_eq!(finals.len(), 1);
        assert_eq!(finals[0].1, "pane_a");
        assert_eq!(finals[0].2, 0.0); // x 不动
        assert_eq!(finals[0].3, 340.0);
        assert_state(&h, "p50", AnchorRequestState::Applied);
        // 已消费：再次 drain 无产出（不驻留拉回）。
        assert!(resolve_anchor_requests(&bounds, &HashMap::new(), true).is_empty());
        reset_for_test();
    }

    #[test]
    fn plan781_anchor_unprimed_stays_pending_then_not_found() {
        reset_for_test();
        let h = next_controller_handle();
        bind_controller(&h, "pane_b");
        take_prime_requests();
        // 未预热（无测量）：pending 晚一拍 + 进 prime 名单。
        enqueue_anchor_request(&h, "ghost").unwrap();
        assert!(resolve_anchor_requests(&HashMap::new(), &HashMap::new(), true).is_empty());
        assert_state(&h, "ghost", AnchorRequestState::Pending);
        assert!(take_prime_requests().contains(&h));
        // 预热后 key 不在当帧注册表 → not_found（一次性，不重试）。
        note_controller_state(&h, (0.0, 0.0), (400.0, 200.0), (800.0, 1000.0));
        let origins = HashMap::from([("pane_b".to_string(), (0.0_f64, 0.0_f64))]);
        assert!(resolve_anchor_requests(&HashMap::new(), &origins, true).is_empty());
        assert_state(&h, "ghost", AnchorRequestState::Invalid("not_found".into()));
        assert!(resolve_anchor_requests(&HashMap::new(), &origins, true).is_empty());
        reset_for_test();
    }

    #[test]
    fn plan781_anchor_rebind_generation_invalidates() {
        reset_for_test();
        let h = next_controller_handle();
        bind_controller(&h, "pane_c1"); // gen=1
        take_prime_requests();
        note_controller_state(&h, (0.0, 100.0), (400.0, 200.0), (800.0, 1000.0));
        enqueue_anchor_request(&h, "p10").unwrap();
        // 换绑（换章/重建到新 widget id）→ generation=2。
        bind_controller(&h, "pane_c2");
        take_prime_requests();
        note_controller_state(&h, (0.0, 0.0), (400.0, 200.0), (800.0, 1000.0));
        let bounds = HashMap::from([((h.clone(), "p10".to_string()), (50.0_f64, 40.0_f64))]);
        let origins = HashMap::from([("pane_c2".to_string(), (0.0_f64, 0.0_f64))]);
        let finals = resolve_anchor_requests(&bounds, &origins, true);
        assert!(
            finals.is_empty(),
            "stale request must not land on new content"
        );
        assert_state(&h, "p10", AnchorRequestState::Invalid("not_bound".into()));
        // 重新入队（新代际）正常解析。
        enqueue_anchor_request(&h, "p10").unwrap();
        let finals = resolve_anchor_requests(&bounds, &origins, true);
        assert_eq!(finals.len(), 1);
        assert_eq!(finals[0].1, "pane_c2");
        assert_state(&h, "p10", AnchorRequestState::Applied);
        reset_for_test();
    }

    /// PLAN-781 T-05 实机矩阵勘定（vm_probe v03_init_state_applied 红）：
    /// 入队于任何绑定之前（gen=0，Init 先发目标）的请求采用**当前**绑定
    /// ——首次 bind 的 0→1 自增不是换绑；冻结合同 §2「请求首次可在布局
    /// 前发出，等待当前绑定的有效布局后消费」。
    #[test]
    fn plan781_anchor_pre_bind_request_adopts_first_binding() {
        reset_for_test();
        let h = next_controller_handle();
        // Init 时序：pane 尚未 build → 入队记录 gen=0。
        enqueue_anchor_request(&h, "c3").unwrap();
        assert_state(&h, "c3", AnchorRequestState::Pending);
        // 首绑（gen 0→1）+ 预热 + 当帧采集 → applied（不是 not_bound）。
        bind_controller(&h, "pane_init"); // gen=1
        take_prime_requests();
        note_controller_state(&h, (0.0, 50.0), (400.0, 200.0), (800.0, 1000.0));
        let bounds = HashMap::from([((h.clone(), "c3".to_string()), (0.0_f64, 36.0_f64))]);
        let origins = HashMap::from([("pane_init".to_string(), (0.0_f64, 50.0_f64))]);
        let finals = resolve_anchor_requests(&bounds, &origins, true);
        assert_eq!(finals.len(), 1, "pre-bind request must adopt first binding");
        assert_eq!(finals[0].1, "pane_init");
        assert_state(&h, "c3", AnchorRequestState::Applied);
        // 对照：gen≥1 的请求在换绑后仍严格 not_bound（上一测覆盖）。
        reset_for_test();
    }

    #[test]
    fn plan781_anchor_reequeue_supersedes() {
        reset_for_test();
        let h = next_controller_handle();
        bind_controller(&h, "pane_d");
        take_prime_requests();
        note_controller_state(&h, (0.0, 100.0), (400.0, 200.0), (800.0, 1000.0));
        let bounds = HashMap::from([((h.clone(), "p30".to_string()), (900.0_f64, 40.0_f64))]);
        let origins = HashMap::from([("pane_d".to_string(), (0.0_f64, 0.0_f64))]);
        enqueue_anchor_request(&h, "p30").unwrap();
        let _ = resolve_anchor_requests(&bounds, &origins, true);
        assert_state(&h, "p30", AnchorRequestState::Applied);
        // 重复入队 → 覆盖重新 pending（用户重定位语义）。
        enqueue_anchor_request(&h, "p30").unwrap();
        assert_state(&h, "p30", AnchorRequestState::Pending);
        let _ = resolve_anchor_requests(&bounds, &origins, true);
        assert_state(&h, "p30", AnchorRequestState::Applied);
        // 无记录 → unknown（anchor_state 查询语义）。
        assert_eq!(anchor_request_state(&h, "none"), None);
        reset_for_test();
    }
}
