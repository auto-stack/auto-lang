//! InspectorCache + ComputedNode data model (Plan 307, Phase 2).
//!
//! Pure data structures holding all "computed" probe data per VNode (layout,
//! computed style, state bindings, for-loop provenance, events, source
//! location) plus a cache keyed by [`VNodeId`] that also maintains the
//! bidirectional `VNodeId <-> iced widget id` mapping.
//!
//! Invariant (design doc §6.1): selecting any node, the right-hand panel must
//! always be able to render and never panic because a field is missing. This is
//! enforced by making every field an `Option`/`Vec` -- `None`/empty is the
//! valid "unknown" representation.

use std::collections::HashMap;

use crate::ui::vnode::VNodeId;

use super::primitives::{BoxModel, Rect};

// =====================================================================
// Sub-structures
// =====================================================================

/// 状态绑定：表达式 + 解析后的当前值。
#[derive(Debug, Clone, Default)]
pub struct StateBinding {
    pub expr: String,
    pub current_value: String,
}

/// for 循环迭代溯源。
#[derive(Debug, Clone, Default)]
pub struct ForIter {
    pub var: String,
    pub index: Option<usize>,
    pub value_repr: String,
    pub iterable_repr: String,
}

/// 事件处理器绑定。
#[derive(Debug, Clone, Default)]
pub struct EventHandlerInfo {
    pub event: String,
    pub handler: String,
}

// =====================================================================
// ComputedNode
// =====================================================================

/// 单个节点的所有 computed 探测数据。
///
/// 不变量：全字段 `Option`/`Vec`（空即可），右栏永远能渲染。
#[derive(Debug, Clone, Default)]
pub struct ComputedNode {
    /// Layout bounds reported by the backend.
    pub bounds: Option<Rect>,
    /// Full box model (content + padding + margin).
    pub box_model: Option<BoxModel>,
    /// Computed style key/value pairs (e.g. `("color", "red")`).
    pub computed_style: Vec<(String, String)>,
    /// Raw `class` attribute string from the source.
    pub raw_class: Option<String>,
    /// Reactive state bindings attached to this node.
    pub state_bindings: Vec<StateBinding>,
    /// If this node is a child of a `for` loop, the iteration context.
    pub for_context: Option<ForIter>,
    /// Event handlers attached to this node.
    pub events: Vec<EventHandlerInfo>,
    /// Source location, e.g. `"app.at:42"`.
    pub source: Option<String>,
}

impl ComputedNode {
    /// 右栏渲染摘要（不变量：永不 panic）。
    ///
    /// Even with every field `None`/empty this produces a usable one-line
    /// summary so the right panel can always render *something*.
    pub fn summary(&self, kind: &str, path: &[u16]) -> String {
        let mut s = format!("{} {:?}", kind, path);
        if let Some(b) = &self.bounds {
            s.push_str(&format!(
                " @ {:.0},{:.0} {:.0}×{:.0}",
                b.x, b.y, b.width, b.height
            ));
        }
        s
    }
}

// =====================================================================
// InspectorCache
// =====================================================================

/// F12 门控的检视缓存：按 [`VNodeId`] 索引每个节点的 computed 数据，
/// 并维护 `VNodeId <-> iced widget id`（形如 `"aura_N"`）双向映射。
///
/// The cache is populated lazily by later tasks (12-13) and read by the
/// inspector panels (tasks 15-16). It owns no I/O and performs no layout --
/// it is a pure data container.
#[derive(Debug, Clone, Default)]
pub struct InspectorCache {
    /// Per-node computed probe data.
    by_id: HashMap<VNodeId, ComputedNode>,
    /// Forward map: `VNodeId -> iced widget id`.
    id_to_iced: HashMap<VNodeId, String>,
    /// Reverse map: `iced widget id -> VNodeId`.
    iced_to_id: HashMap<String, VNodeId>,
}

impl InspectorCache {
    /// Create an empty cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Look up the computed data for a node.
    pub fn get(&self, id: VNodeId) -> Option<&ComputedNode> {
        self.by_id.get(&id)
    }

    /// Look up or insert-default the computed data for a node, returning a
    /// mutable handle so callers can fill in fields.
    pub fn get_mut_or_default(&mut self, id: VNodeId) -> &mut ComputedNode {
        self.by_id.entry(id).or_default()
    }

    /// Record/replace the bidirectional `VNodeId <-> iced widget id` mapping.
    ///
    /// When an existing mapping for `id` is overwritten, the stale reverse
    /// entry is removed so the two maps never diverge.
    pub fn set_iced_map(&mut self, id: VNodeId, iced_id: String) {
        if let Some(old) = self.id_to_iced.insert(id, iced_id.clone()) {
            self.iced_to_id.remove(&old);
        }
        self.iced_to_id.insert(iced_id, id);
    }

    /// Resolve `VNodeId -> iced widget id`.
    pub fn vnode_to_iced(&self, id: VNodeId) -> Option<&String> {
        self.id_to_iced.get(&id)
    }

    /// Resolve `iced widget id -> VNodeId`.
    pub fn iced_to_vnode(&self, iced_id: &str) -> Option<VNodeId> {
        self.iced_to_id.get(iced_id).copied()
    }

    /// Drop all cached data and id mappings.
    pub fn clear(&mut self) {
        self.by_id.clear();
        self.id_to_iced.clear();
        self.iced_to_id.clear();
    }

    /// Iterate over every [`VNodeId`] that has cached data.
    pub fn ids(&self) -> impl Iterator<Item = VNodeId> + '_ {
        self.by_id.keys().copied()
    }
}

// =====================================================================
// Free helpers
// =====================================================================

/// Backfill layout bounds (as reported by the iced backend) into the cache.
///
/// For each `(id_str, (x, y, w, h))`, if `id_str` maps to a known [`VNodeId`],
/// set that node's `bounds` and a zero-padding `box_model` (so `content ==
/// bounds`). Padding/margin refinement from `raw_class` is deferred to a later
/// task (the declared `class` value is not yet populated into `ComputedNode`).
///
/// (Plan 307, Task 13.)
pub fn backfill_bounds(
    cache: &mut InspectorCache,
    bounds: &HashMap<String, (f32, f32, f32, f32)>,
) {
    for (id_str, (x, y, w, h)) in bounds {
        if let Some(vnid) = cache.iced_to_vnode(id_str) {
            let node = cache.get_mut_or_default(vnid);
            let rect = Rect::new(*x, *y, *w, *h);
            node.bounds = Some(rect);
            // Zero padding/margin for now: content == bounds. Padding refinement
            // (content = bounds − declared padding) is deferred until raw_class
            // is populated by a later task.
            node.box_model = Some(BoxModel::from_bounds(rect));
        }
    }
}

// =====================================================================
// Tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::vnode::VNodeId;

    #[test]
    fn computed_node_default_renders_without_panic() {
        let cn = ComputedNode::default();
        assert!(cn.bounds.is_none());
        assert!(cn.computed_style.is_empty());
        assert!(cn.state_bindings.is_empty());
        assert!(cn.for_context.is_none());
        // 右栏不变量：即便全空也能产出摘要
        let summary = cn.summary("Button", &[0, 1, 2]);
        assert!(summary.contains("Button"));
    }

    #[test]
    fn inspector_cache_round_trip_id_map() {
        let mut cache = InspectorCache::new();
        cache.set_iced_map(VNodeId::new(7), "aura_3_42".into());
        assert_eq!(cache.iced_to_vnode("aura_3_42"), Some(VNodeId::new(7)));
        assert_eq!(
            cache.vnode_to_iced(VNodeId::new(7)).map(|s| s.as_str()),
            Some("aura_3_42")
        );
    }

    #[test]
    fn inspector_cache_get_mut_or_default_inserts() {
        let mut cache = InspectorCache::new();
        let id = VNodeId::new(3);
        cache.get_mut_or_default(id).raw_class = Some("p-4".into());
        assert_eq!(cache.get(id).unwrap().raw_class.as_deref(), Some("p-4"));
    }

    // -----------------------------------------------------------------
    // Additional invariants
    // -----------------------------------------------------------------

    #[test]
    fn set_iced_map_overwrite_removes_stale_reverse_entry() {
        // Reassigning a VNodeId to a new iced id must not leave the old
        // reverse mapping dangling.
        let mut cache = InspectorCache::new();
        let id = VNodeId::new(1);
        cache.set_iced_map(id, "aura_0_0".into());
        cache.set_iced_map(id, "aura_0_9".into());

        assert_eq!(
            cache.vnode_to_iced(id).map(|s| s.as_str()),
            Some("aura_0_9")
        );
        // Old reverse entry gone.
        assert_eq!(cache.iced_to_vnode("aura_0_0"), None);
        assert_eq!(cache.iced_to_vnode("aura_0_9"), Some(id));
    }

    #[test]
    fn clear_empties_all_maps() {
        let mut cache = InspectorCache::new();
        cache.get_mut_or_default(VNodeId::new(1)).raw_class = Some("p-4".into());
        cache.set_iced_map(VNodeId::new(1), "aura_0_0".into());

        cache.clear();

        assert!(cache.get(VNodeId::new(1)).is_none());
        assert!(cache.vnode_to_iced(VNodeId::new(1)).is_none());
        assert!(cache.iced_to_vnode("aura_0_0").is_none());
        assert_eq!(cache.ids().count(), 0);
    }

    #[test]
    fn summary_with_bounds_includes_geometry() {
        let mut cn = ComputedNode::default();
        cn.bounds = Some(Rect::new(10.0, 20.0, 120.0, 36.0));
        let s = cn.summary("Button", &[0]);
        assert!(s.contains("Button"));
        assert!(s.contains("10"));
        assert!(s.contains("120"));
    }

    // -----------------------------------------------------------------
    // backfill_bounds (Plan 307, Task 13)
    // -----------------------------------------------------------------

    #[test]
    fn backfill_bounds_sets_bounds_and_box_model() {
        let mut cache = InspectorCache::new();
        let id = VNodeId::new(7);
        cache.set_iced_map(id, "aura_3_42".into());

        let mut bounds = std::collections::HashMap::new();
        bounds.insert("aura_3_42".to_string(), (10.0, 20.0, 120.0, 36.0));

        backfill_bounds(&mut cache, &bounds);

        let node = cache.get(id).expect("node inserted");
        let b = node.bounds.expect("bounds set");
        assert_eq!((b.x, b.y, b.width, b.height), (10.0, 20.0, 120.0, 36.0));

        let bm = node.box_model.as_ref().expect("box_model set");
        // Zero padding/margin for now → content == bounds.
        assert!((bm.content.x - 10.0).abs() < f32::EPSILON);
        assert!((bm.content.y - 20.0).abs() < f32::EPSILON);
        assert!((bm.content.width - 120.0).abs() < f32::EPSILON);
        assert!((bm.content.height - 36.0).abs() < f32::EPSILON);
        assert!(bm.padding.is_zero());
        assert!(bm.margin.is_zero());
    }

    #[test]
    fn backfill_bounds_skips_unknown_id_str() {
        let mut cache = InspectorCache::new();
        cache.set_iced_map(VNodeId::new(7), "aura_3_42".into());

        let mut bounds = std::collections::HashMap::new();
        // No mapping for "aura_9_99".
        bounds.insert("aura_9_99".to_string(), (1.0, 2.0, 3.0, 4.0));

        backfill_bounds(&mut cache, &bounds);

        // Known node untouched, and no spurious entry created.
        assert!(cache.get(VNodeId::new(7)).is_none());
        assert_eq!(cache.ids().count(), 0);
    }

    #[test]
    fn backfill_bounds_empty_map_is_noop() {
        let mut cache = InspectorCache::new();
        cache.set_iced_map(VNodeId::new(1), "aura_0_0".into());
        let bounds = std::collections::HashMap::new();
        backfill_bounds(&mut cache, &bounds);
        assert_eq!(cache.ids().count(), 0);
    }
}
