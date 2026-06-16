//! # VTree → Atom 序列化器（Plan 314）
//!
//! 把一帧的实时 VTree（[`crate::ui::vnode::VTree`]）+ 按 `VNodeId` 索引的
//! computed 子集（[`crate::ui::mcp_server::ComputedNodeLite`]）转成一棵
//! [`auto_val::Node`]，再经 `Display` 序列化为 Atom 文本，供 MCP 工具
//! `autoui_vtree` 返回。
//!
//! ## 不变量
//!
//! - **拓扑 1:1**：Atom node 的 children 严格 = VTree 的 children。盒模型、
//!   computed 样式、events、source 等元数据全部是 node 的 **props**（`Value::Obj`/
//!   `Value::Str`），**不是 children**，故不破坏层级对应。
//! - **降级安全**：任一 computed 字段缺失 → 该 prop 省略，node 照常输出，永不 panic。
//! - **node 名字** = 源 widget 关键字（`col`/`button`/`center`…），经
//!   [`crate::ui::vnode::kind_keyword`]。

use std::collections::HashMap;

use auto_val::{Node, Obj, Value};

use crate::ui::debug::BoxModel;
use crate::ui::mcp_server::{ComputedNodeLite, StyledNodeSnapshot};
use crate::ui::vnode::{VNode, VNodeProps, VTree, VNodeId, kind_keyword};

// ============================================================================
// Options
// ============================================================================

/// 控制 Atom VTree 序列化的选项。
#[derive(Clone, Debug)]
pub struct VTreeAtomOptions {
    /// 只返回该 VNodeId 子树（`None` = 整棵树）。
    pub scope: Option<VNodeId>,
    /// 相对 scope 根的最大深度（`None` = 不限）。
    pub depth: Option<usize>,
    /// 是否输出 `bbox` + `box`（盒模型）。
    pub include_box: bool,
    /// 是否输出 `style` + `class`（computed 样式）。
    pub include_style: bool,
    /// 是否输出 `events`。
    pub include_events: bool,
    /// 是否输出 `source` + `for_iter`。
    pub include_source: bool,
    /// 是否输出 widget 属性（content/label/value/…）。
    pub include_props: bool,
}

impl Default for VTreeAtomOptions {
    fn default() -> Self {
        Self {
            scope: None,
            depth: None,
            include_box: true,
            include_style: true,
            include_events: true,
            include_source: true,
            include_props: true,
        }
    }
}

// ============================================================================
// Builder
// ============================================================================

pub struct VTreeAtomBuilder;

impl VTreeAtomBuilder {
    /// 由快照 + 选项构建 Atom `Node` 树。
    pub fn build(snap: &StyledNodeSnapshot, opts: &VTreeAtomOptions) -> Node {
        let root_id = opts
            .scope
            .or_else(|| snap.vtree.root().map(|r| r.id));
        match root_id.and_then(|id| snap.vtree.get(id)) {
            Some(root) => Self::build_node(root, &snap.vtree, &snap.computed, opts, 0),
            None => Node::new("empty"),
        }
    }

    fn build_node(
        vnode: &VNode,
        vtree: &VTree,
        computed: &HashMap<VNodeId, ComputedNodeLite>,
        opts: &VTreeAtomOptions,
        depth: usize,
    ) -> Node {
        let mut node = Node::new(kind_keyword(vnode.kind));
        node.id = format!("vnode_{}", vnode.id.as_u64()).into();

        // widget 属性（作为 body props）
        if opts.include_props {
            Self::attach_widget_props(&mut node, &vnode.props);
        }

        // computed 元数据（全部为 props，不进 children）
        if let Some(c) = computed.get(&vnode.id) {
            Self::attach_computed(&mut node, c, opts);
        }

        // children —— 严格 1:1
        if let Some(max) = opts.depth {
            if depth >= max {
                if !vnode.children.is_empty() {
                    node.set_prop("_truncated_children", Value::Int(vnode.children.len() as i32));
                }
                return node;
            }
        }
        for cid in &vnode.children {
            if let Some(child) = vtree.get(*cid) {
                node = node.with_child(Self::build_node(child, vtree, computed, opts, depth + 1));
            }
        }
        node
    }

    fn attach_widget_props(node: &mut Node, props: &VNodeProps) {
        match props {
            VNodeProps::Empty => {}
            VNodeProps::Text { content } => node.set_prop("content", Value::Str(content.clone().into())),
            VNodeProps::Button { label } => node.set_prop("label", Value::Str(label.clone().into())),
            VNodeProps::Input { placeholder, value, password } => {
                node.set_prop("placeholder", Value::Str(placeholder.clone().into()));
                node.set_prop("value", Value::Str(value.clone().into()));
                node.set_prop("password", Value::Bool(*password));
            }
            VNodeProps::Textarea { placeholder, value } => {
                node.set_prop("placeholder", Value::Str(placeholder.clone().into()));
                node.set_prop("value", Value::Str(value.clone().into()));
            }
            VNodeProps::Checkbox { label, is_checked } => {
                node.set_prop("label", Value::Str(label.clone().into()));
                node.set_prop("checked", Value::Bool(*is_checked));
            }
            VNodeProps::Radio { label, is_selected } => {
                node.set_prop("label", Value::Str(label.clone().into()));
                node.set_prop("selected", Value::Bool(*is_selected));
            }
            VNodeProps::Select { options, selected_index } => {
                let arr: Vec<Value> = options
                    .iter()
                    .map(|o| Value::Str(o.clone().into()))
                    .collect();
                node.set_prop("options", Value::Array(arr.into()));
                node.set_prop(
                    "selected",
                    Value::Int(selected_index.unwrap_or(0) as i32),
                );
            }
            VNodeProps::Layout { spacing, padding } => {
                node.set_prop("spacing", Value::Int(*spacing as i32));
                node.set_prop("padding", Value::Int(*padding as i32));
            }
            VNodeProps::Container { padding, center_x, center_y } => {
                node.set_prop("padding", Value::Int(*padding as i32));
                node.set_prop("center_x", Value::Bool(*center_x));
                node.set_prop("center_y", Value::Bool(*center_y));
            }
            VNodeProps::Scrollable => {}
            VNodeProps::Slider { min, max, value, step } => {
                node.set_prop("min", Value::Float(*min as f64));
                node.set_prop("max", Value::Float(*max as f64));
                node.set_prop("value", Value::Float(*value as f64));
                if let Some(step) = step {
                    node.set_prop("step", Value::Float(*step as f64));
                }
            }
            VNodeProps::ProgressBar { progress } => {
                node.set_prop("progress", Value::Float(*progress as f64));
            }
            VNodeProps::List { spacing } => node.set_prop("spacing", Value::Int(*spacing as i32)),
            VNodeProps::Table { spacing, col_spacing } => {
                node.set_prop("spacing", Value::Int(*spacing as i32));
                node.set_prop("col_spacing", Value::Int(*col_spacing as i32));
            }
        }
    }

    fn attach_computed(node: &mut Node, c: &ComputedNodeLite, opts: &VTreeAtomOptions) {
        if opts.include_box {
            if let Some((x, y, w, h)) = c.bounds {
                node.set_prop("bbox", Self::rect_obj(x, y, w, h));
            }
            if let Some(bm) = &c.box_model {
                node.set_prop("box", Self::box_obj(bm));
            }
        }
        if opts.include_style {
            if !c.computed_style.is_empty() {
                let mut obj = Obj::new();
                for (k, v) in &c.computed_style {
                    obj.set(k.clone(), Value::Str(v.clone().into()));
                }
                node.set_prop("style", Value::Obj(obj));
            }
            if let Some(cls) = &c.raw_class {
                node.set_prop("class", Value::Str(cls.clone().into()));
            }
        }
        if opts.include_events && !c.events.is_empty() {
            let mut obj = Obj::new();
            for (ev, handler) in &c.events {
                obj.set(ev.clone(), Value::Str(handler.clone().into()));
            }
            node.set_prop("events", Value::Obj(obj));
        }
        if opts.include_source {
            if let Some(src) = &c.source {
                node.set_prop("source", Value::Str(src.clone().into()));
            }
            if let Some((var, idx, value_repr)) = &c.for_context {
                let mut obj = Obj::new();
                obj.set("var", Value::Str(var.clone().into()));
                if let Some(i) = idx {
                    obj.set("index", Value::Int(*i as i32));
                }
                obj.set("value", Value::Str(value_repr.clone().into()));
                node.set_prop("for_iter", Value::Obj(obj));
            }
        }
    }

    fn rect_obj(x: f32, y: f32, w: f32, h: f32) -> Value {
        let mut obj = Obj::new();
        obj.set("x", Value::Float(x as f64));
        obj.set("y", Value::Float(y as f64));
        obj.set("w", Value::Float(w as f64));
        obj.set("h", Value::Float(h as f64));
        Value::Obj(obj)
    }

    fn box_obj(bm: &BoxModel) -> Value {
        let mut obj = Obj::new();
        // bbox = border_box（iced 实测的包围盒，含 border）
        let bb = bm.border_box();
        obj.set(
            "bbox",
            Self::rect_obj(bb.x, bb.y, bb.width, bb.height),
        );
        obj.set(
            "content",
            Self::rect_obj(bm.content.x, bm.content.y, bm.content.width, bm.content.height),
        );
        obj.set("padding", Self::insets_obj(&bm.padding));
        obj.set("border", Self::insets_obj(&bm.border));
        obj.set("margin", Self::insets_obj(&bm.margin));
        Value::Obj(obj)
    }

    fn insets_obj(e: &crate::ui::debug::EdgeInsets) -> Value {
        let mut obj = Obj::new();
        obj.set("t", Value::Float(e.top as f64));
        obj.set("r", Value::Float(e.right as f64));
        obj.set("b", Value::Float(e.bottom as f64));
        obj.set("l", Value::Float(e.left as f64));
        Value::Obj(obj)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::debug::{BoxModel, EdgeInsets, EventHandlerInfo, InspectorCache, Rect};
    use crate::ui::mcp_server::StyledNodeSnapshot;
    use crate::ui::vnode::{VNode, VNodeKind, VNodeProps, VTree, VNodeId};

    /// 复制 mcp_server 测试里的样例树构建（独立，避免跨模块依赖）。
    fn sample_snapshot() -> StyledNodeSnapshot {
        let mut tree = VTree::new();
        tree.set_root(VNode::new(
            VNodeId::new(0),
            VNodeKind::Column,
            VNodeProps::Layout { spacing: 8, padding: 4 },
        ));
        tree.add_node(VNode::new(
            VNodeId::new(1),
            VNodeKind::Text,
            VNodeProps::Text { content: "Hello".into() },
        ));
        tree.get_mut(VNodeId::new(0)).unwrap().add_child(VNodeId::new(1));
        tree.add_node(VNode::new(
            VNodeId::new(2),
            VNodeKind::Button,
            VNodeProps::Button { label: "OK".into() },
        ));
        tree.get_mut(VNodeId::new(0)).unwrap().add_child(VNodeId::new(2));

        let mut cache = InspectorCache::new();
        let r = cache.get_mut_or_default(VNodeId::new(0));
        r.bounds = Some(Rect { x: 0.0, y: 0.0, width: 100.0, height: 50.0 });
        let b = cache.get_mut_or_default(VNodeId::new(2));
        b.bounds = Some(Rect { x: 40.0, y: 10.0, width: 60.0, height: 30.0 });
        b.box_model = Some(BoxModel {
            content: Rect { x: 44.0, y: 14.0, width: 52.0, height: 22.0 },
            padding: EdgeInsets { top: 4.0, right: 4.0, bottom: 4.0, left: 4.0 },
            border: EdgeInsets::default(),
            margin: EdgeInsets::default(),
        });
        b.computed_style.push(("color".into(), "#ffffff".into()));
        b.events.push(EventHandlerInfo { event: "press".into(), handler: ".Ok".into() });
        b.raw_class = Some("btn".into());

        StyledNodeSnapshot::from_live("Demo", &tree, &cache)
    }

    #[test]
    fn build_produces_atom_with_keyword_names_and_vnode_ids() {
        let snap = sample_snapshot();
        let atom = VTreeAtomBuilder::build(&snap, &VTreeAtomOptions::default()).to_string();

        // node names = source keywords
        assert!(atom.contains("col vnode_0"), "root named 'col vnode_0': {atom}");
        assert!(atom.contains("text vnode_1"), "text child: {atom}");
        assert!(atom.contains("button vnode_2"), "button child: {atom}");
    }

    #[test]
    fn build_attaches_widget_props_and_computed() {
        let snap = sample_snapshot();
        let atom = VTreeAtomBuilder::build(&snap, &VTreeAtomOptions::default()).to_string();

        // widget props
        assert!(atom.contains("content: "), "text content: {atom}");
        assert!(atom.contains("label: "), "button label: {atom}");

        // computed box/style/events as props (NOT children)
        assert!(atom.contains("bbox: "), "bbox prop: {atom}");
        assert!(atom.contains("style: "), "style prop: {atom}");
        assert!(atom.contains("events: "), "events prop: {atom}");
        assert!(atom.contains("class: "), "class prop: {atom}");

        // box model value present
        assert!(atom.contains("box: "), "box prop: {atom}");
        assert!(atom.contains("content:") && atom.contains("padding:"));
    }

    #[test]
    fn children_topology_is_one_to_one() {
        let snap = sample_snapshot();
        let atom = VTreeAtomBuilder::build(&snap, &VTreeAtomOptions::default()).to_string();

        // root 'col' has exactly 2 child nodes; bbox/style/etc are props, not nodes.
        // Count top-level child node openings under col: should be 2 (text, button).
        let col_idx = atom.find("col vnode_0").unwrap();
        let after = &atom[col_idx..];
        let child_openings = ["text vnode_1", "button vnode_2"];
        for c in child_openings {
            assert!(after.contains(c), "expected child {c}: {atom}");
        }
        // 'box'/'style'/'events' must appear as `key: {` props, never as bare `box vnode` nodes
        assert!(!atom.contains("box vnode_"), "box must be a prop not a node: {atom}");
        assert!(!atom.contains("style vnode_"), "style must be a prop: {atom}");
    }

    #[test]
    fn depth_truncates_children() {
        let snap = sample_snapshot();
        let atom = VTreeAtomBuilder::build(&snap, &VTreeAtomOptions { depth: Some(0), ..Default::default() }).to_string();
        // root only, children truncated
        assert!(!atom.contains("text vnode_1"), "no text child at depth 0: {atom}");
        assert!(atom.contains("_truncated_children: 2"), "truncation marker: {atom}");
    }

    #[test]
    fn scope_returns_subtree_only() {
        let snap = sample_snapshot();
        let atom = VTreeAtomBuilder::build(
            &snap,
            &VTreeAtomOptions { scope: Some(VNodeId::new(2)), ..Default::default() },
        )
        .to_string();
        assert!(atom.starts_with("button vnode_2"), "rooted at button: {atom}");
        assert!(!atom.contains("col vnode_0"), "sibling/root excluded: {atom}");
    }

    #[test]
    fn include_flags_omit_sections() {
        let snap = sample_snapshot();
        let off = VTreeAtomOptions {
            include_box: false,
            include_style: false,
            include_events: false,
            include_source: false,
            include_props: false,
            ..Default::default()
        };
        let atom = VTreeAtomBuilder::build(&snap, &off).to_string();
        assert!(!atom.contains("bbox:"), "no bbox: {atom}");
        assert!(!atom.contains("style:"), "no style: {atom}");
        assert!(!atom.contains("events:"), "no events: {atom}");
        assert!(!atom.contains("label:"), "no widget props: {atom}");
    }
}
