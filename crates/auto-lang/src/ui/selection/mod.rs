//! Select Anything —— 任意 AutoUI 基面矩形框选 → 顶层节点集（PLAN-646）。
//!
//! 选择语义纯函数，VM/Iced 端、Vue 端（TS 侧同语义另实现）与 MCP 工具
//! `autoui_select_rect` 共享同一契约：
//!
//! 1. **中心点包含**：节点 bounds 矩形中心落在框选矩形内 → 命中。部分被
//!    框选边缘扫过但中心不在 → 不选中（避免框到半个大容器就吞掉整容器）。
//! 2. **顶层修剪**：命中集合中父节点也命中的子节点剔除（框选完整覆盖卡片
//!    → 只输出卡片本身 = 组织结构语义）。
//! 3. **文档序输出**：按 VTree 文档序（DFS 先序，即 `VTree::nodes()` 存储序）。
//!
//! 无 bounds 的节点不参与命中，但作为选中节点的子孙出现在 structure/source
//! 输出中（VTree 拓扑本身完整）。

pub mod output;

pub use output::{
    build_selection_result, node_to_json, select_envelope, value_to_json, SelectedNode,
    SelectionFormat, SelectionResult,
};

use std::collections::{HashMap, HashSet};

use crate::ui::debug::Rect;
use crate::ui::vnode::{VNodeId, VTree};

/// 拖拽死区：位移小于该值视同点击，不触发框选。
pub const MARQUEE_DRAG_THRESHOLD: f32 = 4.0;

/// Alt+拖拽 marquee 几何（anchor = 按下点，current = 当前点）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Marquee {
    pub anchor: (f32, f32),
    pub current: (f32, f32),
}

impl Marquee {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            anchor: (x, y),
            current: (x, y),
        }
    }

    pub fn update(&mut self, x: f32, y: f32) {
        self.current = (x, y);
    }

    /// 归一化矩形（支持任意拖拽方向；负向拖拽原点取左上角）。
    pub fn rect(&self) -> Rect {
        let (ax, ay) = self.anchor;
        let (cx, cy) = self.current;
        let x = ax.min(cx);
        let y = ay.min(cy);
        Rect::new(x, y, (ax - cx).abs(), (ay - cy).abs())
    }

    /// 拖拽位移是否超过死区（点击 vs 框选判定）。
    pub fn is_drag(&self) -> bool {
        let (ax, ay) = self.anchor;
        let (cx, cy) = self.current;
        ((ax - cx).abs() >= MARQUEE_DRAG_THRESHOLD) || ((ay - cy).abs() >= MARQUEE_DRAG_THRESHOLD)
    }
}

/// 中心点包含命中：节点 bounds 矩形中心落在 `rect` 内 → 命中。
///
/// 线性扫描所有 bounds（与 [`crate::ui::debug::hit_test`] 同复杂度量级）。
pub fn select_nodes(rect: Rect, bounds: &HashMap<VNodeId, Rect>) -> HashSet<VNodeId> {
    let mut hit = HashSet::new();
    for (&id, r) in bounds {
        let cx = r.x + r.width / 2.0;
        let cy = r.y + r.height / 2.0;
        if rect.contains(cx, cy) {
            hit.insert(id);
        }
    }
    hit
}

/// 顶层修剪 + 文档序输出。
///
/// 命中集合中某节点的任一祖先也在命中集合中 → 该节点被其祖先吸收（剔除）。
/// 输出按 VTree 文档序（`nodes()` DFS 先序存储序）。
///
/// 祖先链从 children 列表派生（不依赖 `VNode.parent` 字段——构造器/夹具
/// 可能不回填；children 列表是拓扑事实源）。
pub fn trim_to_topmost(selected: &HashSet<VNodeId>, vtree: &VTree) -> Vec<VNodeId> {
    if selected.is_empty() {
        return Vec::new();
    }
    let mut parent_of: HashMap<VNodeId, VNodeId> = HashMap::new();
    for node in vtree.nodes() {
        for &c in &node.children {
            parent_of.insert(c, node.id);
        }
    }
    let has_selected_ancestor = |id: VNodeId| -> bool {
        let mut cur = id;
        // 步数上界 = 节点数（防御环数据；正常树必在祖先链终止）。
        for _ in 0..=vtree.node_count() {
            match parent_of.get(&cur) {
                Some(&p) => {
                    if selected.contains(&p) {
                        return true;
                    }
                    cur = p;
                }
                None => return false,
            }
        }
        false
    };
    vtree
        .nodes()
        .iter()
        .filter(|n| selected.contains(&n.id) && !has_selected_ancestor(n.id))
        .map(|n| n.id)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::vnode::{VNode, VNodeProps};

    fn tree_with_parents() -> VTree {
        // root(1) → child(2)；root → other(4)（存储序 = 文档序 1,2,4）
        let mut t = VTree::new();
        use crate::ui::vnode::VNodeKind;
        let root =
            VNode::new(VNodeId::new(1), VNodeKind::Column, VNodeProps::Empty).with_label("root");
        let r = t.set_root(root);
        let child = VNode::new(VNodeId::new(2), VNodeKind::Row, VNodeProps::Empty).with_parent(r);
        t.add_node(child);
        t.get_mut(r).unwrap().add_child(VNodeId::new(2));
        let other =
            VNode::new(VNodeId::new(4), VNodeKind::Button, VNodeProps::Empty).with_parent(r);
        t.add_node(other);
        t.get_mut(r).unwrap().add_child(VNodeId::new(4));
        t
    }

    #[test]
    fn marquee_rect_normalizes_both_directions() {
        let mut m = Marquee::new(100.0, 100.0);
        m.update(40.0, 160.0);
        let r = m.rect();
        assert_eq!((r.x, r.y, r.width, r.height), (40.0, 100.0, 60.0, 60.0));
        // 反向
        let mut m2 = Marquee::new(0.0, 0.0);
        m2.update(-10.0, -20.0);
        let r2 = m2.rect();
        assert_eq!((r2.x, r2.y), (-10.0, -20.0));
    }

    #[test]
    fn marquee_dead_zone() {
        let mut m = Marquee::new(10.0, 10.0);
        assert!(!m.is_drag());
        m.update(12.0, 10.0);
        assert!(!m.is_drag());
        m.update(20.0, 10.0);
        assert!(m.is_drag());
    }

    #[test]
    fn select_nodes_empty_bounds_is_empty() {
        let bounds = HashMap::new();
        assert!(select_nodes(Rect::new(0.0, 0.0, 100.0, 100.0), &bounds).is_empty());
    }

    #[test]
    fn select_nodes_center_inside_hits_edge_sweep_misses() {
        let mut bounds = HashMap::new();
        let id = VNodeId::new(1);
        // 节点 (10,10)-(110,110)，中心 (60,60)
        bounds.insert(id, Rect::new(10.0, 10.0, 100.0, 100.0));
        // 框选完整覆盖 → 中心在内 → 命中
        assert!(select_nodes(Rect::new(0.0, 0.0, 200.0, 200.0), &bounds).contains(&id));
        // 边缘扫过（只盖到节点左上角，中心不在）→ 不命中
        assert!(!select_nodes(Rect::new(0.0, 0.0, 50.0, 50.0), &bounds).contains(&id));
        // 框选恰好含中心（点框）→ 命中
        assert!(select_nodes(Rect::new(58.0, 58.0, 4.0, 4.0), &bounds).contains(&id));
    }

    #[test]
    fn select_nodes_nested_all_hit() {
        let mut bounds = HashMap::new();
        let outer = VNodeId::new(1);
        let inner = VNodeId::new(2);
        bounds.insert(outer, Rect::new(0.0, 0.0, 200.0, 200.0));
        bounds.insert(inner, Rect::new(50.0, 50.0, 50.0, 50.0));
        let hit = select_nodes(Rect::new(10.0, 10.0, 180.0, 180.0), &bounds);
        assert!(hit.contains(&outer) && hit.contains(&inner));
    }

    #[test]
    fn trim_all_hit_returns_topmost_single_in_doc_order() {
        let t = tree_with_parents();
        let all: HashSet<VNodeId> = [1, 2].iter().map(|&n| VNodeId::new(n)).collect();
        let top = trim_to_topmost(&all, &t);
        assert_eq!(top, vec![VNodeId::new(1)]);
    }

    #[test]
    fn trim_disjoint_hits_keep_multiple_in_doc_order() {
        let t = tree_with_parents();
        // 命中 child(2) 与 other(4)——不存在祖先链重叠
        let sel: HashSet<VNodeId> = [2, 4].iter().map(|&n| VNodeId::new(n)).collect();
        let top = trim_to_topmost(&sel, &t);
        // 文档序：2 先于 4（nodes() 存储序）
        assert_eq!(top.len(), 2);
        assert_eq!(top[0], VNodeId::new(2));
    }

    #[test]
    fn trim_child_hit_parent_miss_keeps_child() {
        let t = tree_with_parents();
        let sel: HashSet<VNodeId> = [2].iter().map(|&n| VNodeId::new(n)).collect();
        assert_eq!(trim_to_topmost(&sel, &t), vec![VNodeId::new(2)]);
    }

    #[test]
    fn trim_empty_selection_is_empty() {
        let t = tree_with_parents();
        assert!(trim_to_topmost(&HashSet::new(), &t).is_empty());
    }

    #[test]
    fn trim_derives_ancestry_from_children_not_parent_field() {
        // 夹具/构造器可能不回填 VNode.parent——children 列表是拓扑事实源。
        let mut t = VTree::new();
        t.set_root(VNode::new(
            VNodeId::new(1),
            crate::ui::vnode::VNodeKind::Column,
            VNodeProps::Empty,
        ));
        t.add_node(VNode::new(
            VNodeId::new(2),
            crate::ui::vnode::VNodeKind::Row,
            VNodeProps::Empty,
        ));
        t.get_mut(VNodeId::new(1))
            .unwrap()
            .add_child(VNodeId::new(2));
        let all: HashSet<VNodeId> = [1u64, 2u64].iter().map(|&n| VNodeId::new(n)).collect();
        assert_eq!(trim_to_topmost(&all, &t), vec![VNodeId::new(1)]);
    }
}
