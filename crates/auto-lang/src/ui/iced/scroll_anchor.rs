//! PLAN-781 T-03: 子内容锚点 wrapper widget + operate 期 bounds 采集 +
//! drain 期解析接线（接口合同见
//! docs/plans/reports/781-interface-decision.md §3/§4）。
//!
//! 数据流：
//! ```text
//! DSL scroll-anchor        → View::ScrollAnchorMark { pane, key }
//! convert_scroll 双臂       → inject_anchor_pane 填 pane（controller key）
//! renderer build           → 本 wrapper（合成 iced Id `scrollanchor:{seq}`，seq 经注册表还原 (pane,key)——T-19/R01）
//! drain 期（有 pending 请求）→ operate(AnchorBoundsCollector) + ScrollStateReader
//! __anchor_bounds_collected → stash；与 pane 原点 stash 齐备即解析
//! resolve_anchor_requests  → finals scroll_to（写臂投影回填既有纪律）
//! ```

use std::collections::HashMap;
use std::sync::Mutex;

use iced::advanced::widget::operation::Outcome;
use iced::advanced::widget::Operation;
use iced::widget::Id;
use iced::{Element, Length, Point, Rectangle, Size, Theme, Vector};

/// 合成 id 前缀（AnchorBoundsCollector 只认此前缀的 container 上报）。
pub const ANCHOR_ID_PREFIX: &str = "scrollanchor:";

// ---------------------------------------------------------------------------
// 稳定序号注册表（PLAN-783 T-19 / F-781-R01 修复）
//
// 旧实现把 (pane, key) 以冒号拼进合成 id、再按**第一个**冒号拆回——标准
// controller 句柄 `@scrollctl:N` 自带冒号即错拆（R01 审计反例），且任意
// key 字符还要经 Debug 字符串抓取往返。改为**不透明序号**：build 期
// `register_anchor` 分配 `scrollanchor:{seq}`（纯数字——分隔符/引号/反斜杠
// 不再进入 id），collector 只回传 seq，查表还原 (pane, key)。
//
// 生命周期 = 重注册驱逐：同一 (pane, key) 每帧重建时以新 seq 覆盖并驱逐
// 旧 seq——稳态大小 = 活跃锚点数（无帧钩子、无跨帧增长）；同 pane 重复
// key 的文档序 last-wins 语义保持（先注册者的旧 seq 被驱逐，上报侧跳过）。
// ---------------------------------------------------------------------------

static ANCHOR_SEQ_GEN: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);
lazy_static::lazy_static! {
    static ref ANCHOR_SEQ_FORWARD: Mutex<HashMap<(String, String), u32>> =
        Mutex::new(HashMap::new());
    static ref ANCHOR_SEQ_REVERSE: Mutex<HashMap<u32, (String, String)>> =
        Mutex::new(HashMap::new());
}

/// build 期注册 (pane, key) → 新 seq（驱逐同 pair 旧 seq）。
pub fn register_anchor(pane: &str, key: &str) -> u32 {
    let seq = ANCHOR_SEQ_GEN.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let pair = (pane.to_string(), key.to_string());
    let mut fwd = ANCHOR_SEQ_FORWARD.lock().unwrap();
    if let Some(old) = fwd.insert(pair.clone(), seq) {
        ANCHOR_SEQ_REVERSE.lock().unwrap().remove(&old);
    }
    ANCHOR_SEQ_REVERSE.lock().unwrap().insert(seq, pair);
    seq
}

/// 合成 id：`scrollanchor:{seq}`（纯数字——无歧义，任意 pane/key 字符安全）。
pub fn anchor_widget_id(seq: u32) -> String {
    format!("{ANCHOR_ID_PREFIX}{seq}")
}

/// 解析合成 id → seq（严格数字；非数字/越界 = None）。
pub fn parse_anchor_widget_seq(id: &str) -> Option<u32> {
    id.strip_prefix(ANCHOR_ID_PREFIX)?.parse().ok()
}

/// seq → (pane, key)（stale/未知 seq = None——调用方跳过该上报）。
pub fn lookup_anchor(seq: u32) -> Option<(String, String)> {
    ANCHOR_SEQ_REVERSE.lock().unwrap().get(&seq).cloned()
}

// ---------------------------------------------------------------------------
// 采集 Operation：container 钩子收 `scrollanchor:` 前缀上报（最终位置
// bounds——Operation 在布局完成后遍历，阶段 A LayoutCollector 同源）。
// ---------------------------------------------------------------------------

/// 一条锚点 rect 上报（pane 锚定坐标——与 LayoutCollector 同空间：pane 内
/// 子件 bounds 以 pane 原点为内容原点）。Vec 形态——JSON 对象键必须是
/// 字符串，元组键不可序列化（T-03 实测：to_string 失败→消息载荷空）。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AnchorRectEntry {
    pub pane: String,
    pub key: String,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

pub type AnchorRectMap = Vec<AnchorRectEntry>;

pub struct AnchorBoundsCollector {
    out: Vec<AnchorRectEntry>,
}

impl AnchorBoundsCollector {
    pub fn new() -> Self {
        Self { out: Vec::new() }
    }
}

impl Default for AnchorBoundsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl Operation<AnchorRectMap> for AnchorBoundsCollector {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<AnchorRectMap>)) {
        operate(self);
    }

    fn container(&mut self, id: Option<&Id>, bounds: Rectangle) {
        if let Some(id) = id {
            let debug = format!("{id:?}");
            // Id Debug 形态 `Id(Custom("scrollanchor:42"))`——取内层串。
            // T-19 后 id 体为纯数字 seq（pane/key 不进入 id——R01 的分隔符
            // 与 Debug 转义两类面根除），查注册表还原。
            let inner = debug.split('"').nth(1).unwrap_or_default();
            if let Some((pane, key)) = parse_anchor_widget_seq(inner).and_then(lookup_anchor) {
                self.out.push(AnchorRectEntry {
                    pane,
                    key,
                    x: bounds.x as f64,
                    y: bounds.y as f64,
                    w: bounds.width as f64,
                    h: bounds.height as f64,
                });
            }
        }
    }

    fn finish(&self) -> Outcome<AnchorRectMap> {
        if std::env::var("P781_DEBUG").is_ok() {
            eprintln!("[P781] collector finish: {} entries", self.out.len());
        }
        Outcome::Some(self.out.clone())
    }
}

// ---------------------------------------------------------------------------
// drain 期 stash：锚点 rect 与 pane 原点两路消息（到达顺序不定，齐备即解析）。
// ---------------------------------------------------------------------------

static ANCHOR_RECTS: Mutex<Option<AnchorRectMap>> = Mutex::new(None);
/// pane widget id → viewport 原点 (vx, vy)（__scroll_state_read 原始图旁路）。
static PANE_ORIGINS: Mutex<Option<HashMap<String, (f64, f64)>>> = Mutex::new(None);

pub fn note_anchor_rects(map: AnchorRectMap) {
    *ANCHOR_RECTS.lock().unwrap() = Some(map);
}

pub fn note_pane_origins(map: HashMap<String, (f64, f64)>) {
    *PANE_ORIGINS.lock().unwrap() = Some(map);
}

/// 解析产物暂存（消息处理器侧生产；drain 块统一消费落盘——任务推送
/// 只在一个位置发生，沿既有 finals scroll_to 通路）。
static RESOLVED_FINALS: Mutex<Vec<(String, String, f64, f64)>> = Mutex::new(Vec::new());

pub fn stash_resolved_finals(finals: Vec<(String, String, f64, f64)>) {
    RESOLVED_FINALS.lock().unwrap().extend(finals);
}

pub fn take_resolved_finals() -> Vec<(String, String, f64, f64)> {
    std::mem::take(&mut *RESOLVED_FINALS.lock().unwrap())
}

/// 两路 stash 齐备时取走并联动解析（controller 请求队列 → finals）。
/// 未齐备/无 pending 返回空（下一拍重试——drain/心跳节拍既有形态）。
pub fn try_resolve_pending() -> Vec<(String, String, f64, f64)> {
    let rects_taken = ANCHOR_RECTS.lock().unwrap().take();
    let origins_taken = PANE_ORIGINS.lock().unwrap().take();
    let dbg = std::env::var("P781_DEBUG").is_ok();
    if dbg {
        eprintln!(
            "[P781] try_resolve: rects={} origins={}",
            rects_taken.as_ref().map(|m| m.len()).unwrap_or(0),
            origins_taken.as_ref().map(|m| m.len()).unwrap_or(0)
        );
    }
    let (rects, origins) = match (rects_taken, origins_taken) {
        (Some(r), Some(o)) => (r, o),
        (r, o) => {
            // 归还非空侧（另一路未到，下一拍重试）。
            if r.is_some() {
                *ANCHOR_RECTS.lock().unwrap() = r;
            }
            if o.is_some() {
                *PANE_ORIGINS.lock().unwrap() = o;
            }
            return Vec::new();
        }
    };
    // (handle, key) → pane 锚定 (y, h)——pane 原点换算在 controller 侧完成
    //（请求持有 handle→widget_id→viewport 原点）。
    let mut bounds: HashMap<(String, String), (f64, f64)> = HashMap::new();
    for e in rects {
        bounds.insert((e.pane, e.key), (e.y, e.h));
    }
    crate::ui::scroll::controller::resolve_anchor_requests(&bounds, &origins, true)
}

// ---------------------------------------------------------------------------
// wrapper widget（AnchorSlot 同构零布局：layout/draw/update 全委托；
// operate 期以合成 id 上报自身 bounds 后再委托子件）。
// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct ScrollAnchorState;

pub struct ScrollAnchor<'a, M: Clone + std::fmt::Debug + 'static> {
    qualified_id: String,
    child: Element<'a, M>,
}

impl<'a, M: Clone + std::fmt::Debug + 'static> ScrollAnchor<'a, M> {
    pub fn new(pane: &str, key: &str, child: Element<'a, M>) -> Self {
        // T-19（R01）：注册表分配 seq（重注册驱逐），id 不含 pane/key。
        Self {
            qualified_id: anchor_widget_id(register_anchor(pane, key)),
            child,
        }
    }
}

impl<M: Clone + std::fmt::Debug + 'static> iced::advanced::Widget<M, Theme, iced::Renderer>
    for ScrollAnchor<'_, M>
{
    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        iced::advanced::widget::tree::Tag::of::<ScrollAnchor<M>>()
    }

    fn state(&self) -> iced::advanced::widget::tree::State {
        iced::advanced::widget::tree::State::new(ScrollAnchorState)
    }

    fn children(&self) -> Vec<iced::advanced::widget::Tree> {
        vec![iced::advanced::widget::Tree::new(&self.child)]
    }

    fn diff(&self, tree: &mut iced::advanced::widget::Tree) {
        tree.diff_children(std::slice::from_ref(&self.child));
    }

    fn size(&self) -> Size<Length> {
        self.child.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        renderer: &iced::Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        self.child
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn update(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        event: &iced::Event,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, M>,
        viewport: &Rectangle,
    ) {
        // 子件消费自己的树槽（anchor_slot 同款三方 zip 纪律）。
        self.child.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn draw(
        &self,
        tree: &iced::advanced::widget::Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.child.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> iced::mouse::Interaction {
        self.child.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn size_hint(&self) -> Size<Length> {
        self.child.as_widget().size_hint()
    }

    fn operate(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        // PLAN-781: 以合成 id 上报自身 bounds（最终位置——operate 在布局
        // 完成后遍历），再委托子件（子件自身的 vnode/aura id 上报不受扰）。
        operation.container(Some(&Id::from(self.qualified_id.clone())), layout.bounds());
        self.child
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, M, Theme, iced::Renderer>> {
        self.child.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

// Point 导入保留给未来 X 轴扩展（与 anchor_slot 对齐的形状契约）。
#[allow(dead_code)]
fn _point_marker(_: Point) {}

#[cfg(test)]
mod plan783_tests {
    use super::*;

    /// F-781-R01 反例复现锁：标准 `@scrollctl:N` 句柄（自带冒号）与含
    /// 分隔符/引号/反斜杠的 key 必须经 id 往返无损；旧实现首冒号拆分
    /// 会把 ("@scrollctl:1","k") 拆成 ("@scrollctl","1:k")。
    #[test]
    fn plan783_r01_standard_handle_and_delimiter_keys_roundtrip() {
        let cases = [
            ("@scrollctl:1", "chapter-50"),
            ("@scrollctl:42", "a:b:c"),
            ("main-pane", "k:nested-colon"),
            ("", "bare-key"),
            ("@scrollctl:7", "quote\"key"),
            ("@scrollctl:8", "back\\slash"),
            ("@scrollctl:9", "空格 与中文：键"),
        ];
        for (pane, key) in cases {
            let seq = register_anchor(pane, key);
            let id = anchor_widget_id(seq);
            assert!(id.starts_with(ANCHOR_ID_PREFIX));
            let got = parse_anchor_widget_seq(&id)
                .and_then(lookup_anchor)
                .unwrap_or_else(|| panic!("roundtrip failed for {pane:?}/{key:?}"));
            assert_eq!(got, (pane.to_string(), key.to_string()), "{pane:?}/{key:?}");
        }
        // id 体为纯数字（无 pane/key 字符进入——注入面不存在）。
        let seq = register_anchor("@scrollctl:1", "q\"k");
        let id = anchor_widget_id(seq);
        let rest = id.strip_prefix(ANCHOR_ID_PREFIX).unwrap();
        assert!(
            rest.bytes().all(|b| b.is_ascii_digit()),
            "id body digits only: {rest}"
        );
    }

    /// 重注册驱逐：同 (pane,key) 重建后旧 seq 失效（lookup miss = 上报侧
    /// 跳过）、新 seq 生效；重复 key 的文档序 last-wins 语义保持。
    #[test]
    fn plan783_r01_reregistration_evicts_stale_seq() {
        let (p, k) = ("@scrollctl:3", "dup");
        let s1 = register_anchor(p, k);
        let s2 = register_anchor(p, k);
        assert_ne!(s1, s2);
        assert_eq!(lookup_anchor(s1), None, "stale seq must miss");
        assert_eq!(lookup_anchor(s2), Some((p.to_string(), k.to_string())));
        // 未知 seq（从未分配/已驱逐）不 panic、不误配。
        assert_eq!(lookup_anchor(u32::MAX), None);
        assert_eq!(parse_anchor_widget_seq("scrollanchor:abc"), None);
        assert_eq!(parse_anchor_widget_seq("scrollanchor:"), None);
        assert_eq!(parse_anchor_widget_seq("unrelated:1"), None);
    }
}
