//! PLAN-781 T-03: 子内容锚点 wrapper widget + operate 期 bounds 采集 +
//! drain 期解析接线（接口合同见
//! docs/plans/reports/781-interface-decision.md §3/§4）。
//!
//! 数据流：
//! ```text
//! DSL scroll-anchor        → View::ScrollAnchorMark { pane, key }
//! convert_scroll 双臂       → inject_anchor_pane 填 pane（controller key）
//! renderer build           → 本 wrapper（合成 iced Id `scrollanchor:{pane}:{key}`）
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

/// 合成 id：`scrollanchor:{pane_controller_key}:{anchor_key}`
///（pane 为空串 = 无 controller 归属——按裸 key 兜底上报）。
pub fn anchor_widget_id(pane: &str, key: &str) -> String {
    format!("{ANCHOR_ID_PREFIX}{pane}:{key}")
}

/// 解析合成 id → (pane, key)。
pub fn parse_anchor_widget_id(id: &str) -> Option<(String, String)> {
    let rest = id.strip_prefix(ANCHOR_ID_PREFIX)?;
    let (pane, key) = rest.split_once(':')?;
    Some((pane.to_string(), key.to_string()))
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
            // Id Debug 形态 `Id(Custom("scrollanchor:..."))`——取内层串。
            let inner = debug
                .split('"')
                .nth(1)
                .unwrap_or_default();
            if let Some((pane, key)) = parse_anchor_widget_id(inner) {
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
pub fn try_resolve_pending()
-> Vec<(String, String, f64, f64)> {
    let rects_taken = ANCHOR_RECTS.lock().unwrap().take();
    let origins_taken = PANE_ORIGINS.lock().unwrap().take();
    let dbg = std::env::var("P781_DEBUG").is_ok();
    if dbg {
        eprintln!("[P781] try_resolve: rects={} origins={}",
            rects_taken.as_ref().map(|m| m.len()).unwrap_or(0),
            origins_taken.as_ref().map(|m| m.len()).unwrap_or(0));
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
        Self {
            qualified_id: anchor_widget_id(pane, key),
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
