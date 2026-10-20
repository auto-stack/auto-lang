//! Select Anything 结果信封（PLAN-646 §2）——纯函数构建 + 三格式渲染。
//!
//! 信封：`{surface, app, rect, nodes[{kind, span, source, structure}]}`。
//! 三种呈现：Auto 源码文本（.at 原文切片、去公共缩进）、JSON、Atom。
//!
//! `source` 切片与源文件一致性是验收锚点：切片先逐字节取自源码子串，
//! 再做缩进归一（去公共缩进）供展示。

use std::collections::HashMap;

use auto_val::{Kid, Node, Value};
use serde_json::json;

use crate::ui::debug::Rect;
use crate::ui::mcp_server::{ComputedNodeLite, StyledNodeSnapshot};
use crate::ui::vnode::{kind_keyword, VNode, VNodeId, VTree};
use crate::ui::vtree_atom::VTreeAtomOptions;

use super::trim_to_topmost;

// ============================================================================
// 格式与信封
// ============================================================================

/// 面板/MCP 输出格式。`Auto` 为面板默认视图；MCP 默认 `Json`。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionFormat {
    Auto,
    Json,
    Atom,
}

impl SelectionFormat {
    pub fn label(self) -> &'static str {
        match self {
            SelectionFormat::Auto => "Auto",
            SelectionFormat::Json => "JSON",
            SelectionFormat::Atom => "Atom",
        }
    }

    /// 面板三视图循环切换。
    pub fn next(self) -> Self {
        match self {
            SelectionFormat::Auto => SelectionFormat::Json,
            SelectionFormat::Json => SelectionFormat::Atom,
            SelectionFormat::Atom => SelectionFormat::Auto,
        }
    }

    /// MCP `format` 参数解析（未知值回落 Json）。
    pub fn parse(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "auto" => SelectionFormat::Auto,
            "atom" => SelectionFormat::Atom,
            _ => SelectionFormat::Json,
        }
    }
}

/// 单个选中顶层节点。
#[derive(Debug)]
pub struct SelectedNode {
    /// VNodeId 数值（vnode_N 的 N）。
    pub id: u64,
    /// widget 关键字（kind_keyword）。
    pub kind: String,
    /// .at 源码字节区间 (offset, len)。合成节点为 None。
    pub span: Option<(usize, usize)>,
    /// 原文切片（已去公共缩进）。无 span 或无源码全文时为 None。
    pub source: Option<String>,
    /// 子树结构（Atom Node；由 VTreeAtomBuilder 以 scope=本节点构建）。
    pub structure: Node,
}

/// 框选结果信封。
#[derive(Debug)]
pub struct SelectionResult {
    /// "vm" | "vue"
    pub surface: &'static str,
    /// App 名/id。
    pub app: String,
    /// 框选矩形 (x, y, w, h)。
    pub rect: (f32, f32, f32, f32),
    /// 顶层节点集（文档序）。
    pub nodes: Vec<SelectedNode>,
}

/// 由 topmost 节点集构建信封。
///
/// - `source_text`：.at 源码全文（VM 端 `AppState.source_code`；None = 不可切片）。
/// - structure 经 [`crate::ui::vtree_atom::VTreeAtomBuilder`]（scope=节点，
///   无 computed——盒模型/样式不在信封契约内）。
pub fn build_selection_result(
    surface: &'static str,
    app: &str,
    rect: Rect,
    topmost: &[VNodeId],
    vtree: &VTree,
    source_text: Option<&str>,
) -> SelectionResult {
    let snap = StyledNodeSnapshot {
        widget_name: app.to_string(),
        vtree: vtree.clone(),
        computed: HashMap::<VNodeId, ComputedNodeLite>::new(),
    };
    let opts = VTreeAtomOptions {
        include_box: false,
        include_style: false,
        ..Default::default()
    };
    let nodes = topmost
        .iter()
        .filter_map(|&id| {
            let vnode = vtree.get(id)?;
            let span = vnode.source_span.map(|s| (s.offset, s.len));
            let source = span
                .and_then(|(off, len)| slice_source(source_text, off, len))
                .map(dedent);
            let structure = crate::ui::vtree_atom::VTreeAtomBuilder::build(
                &snap,
                &VTreeAtomOptions {
                    scope: Some(id),
                    ..opts.clone()
                },
            );
            Some(SelectedNode {
                id: id.as_u64(),
                kind: kind_keyword(vnode.kind).to_string(),
                span,
                source,
                structure,
            })
        })
        .collect();
    SelectionResult {
        surface,
        app: app.to_string(),
        rect: (rect.x, rect.y, rect.width, rect.height),
        nodes,
    }
}

impl SelectionResult {
    /// 按格式渲染为可复制文本。
    pub fn render(&self, fmt: SelectionFormat) -> String {
        match fmt {
            SelectionFormat::Auto => self.render_auto(),
            SelectionFormat::Json => self.render_json(),
            SelectionFormat::Atom => self.render_atom(),
        }
    }

    /// Auto 源码文本（面板默认视图）。
    pub fn render_auto(&self) -> String {
        let n = self.nodes.len();
        let mut out = format!(
            "// ── AutoUI Select Anything ── surface={} app={} rect=({:.0},{:.0},{:.0},{:.0}) nodes={}\n",
            self.surface, self.app, self.rect.0, self.rect.1, self.rect.2, self.rect.3, n
        );
        if n == 0 {
            out.push_str("// (no nodes selected)\n");
            return out;
        }
        for (i, node) in self.nodes.iter().enumerate() {
            match (node.span, &node.source) {
                (Some((off, len)), Some(src)) => {
                    out.push_str(&format!(
                        "\n// [{}/{}] {}  span={}..{}\n",
                        i + 1,
                        n,
                        node.kind,
                        off,
                        off + len
                    ));
                    out.push_str(src);
                    if !src.ends_with('\n') {
                        out.push('\n');
                    }
                }
                _ => {
                    out.push_str(&format!(
                        "\n// [{}/{}] {}  (synthetic, no source span)\n",
                        i + 1,
                        n,
                        node.kind
                    ));
                    out.push_str(&node.structure.to_string());
                }
            }
        }
        out
    }

    /// JSON 信封（pretty）。
    pub fn render_json(&self) -> String {
        serde_json::to_string_pretty(&self.to_json()).unwrap_or_else(|_| "{}".to_string())
    }

    /// Atom 结构文本（逐节点拼接）。
    pub fn render_atom(&self) -> String {
        self.nodes
            .iter()
            .map(|nd| nd.structure.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// 信封 JSON 值（MCP 返回与 JSON 视图共用）。
    pub fn to_json(&self) -> serde_json::Value {
        json!({
            "surface": self.surface,
            "app": self.app,
            "rect": [self.rect.0, self.rect.1, self.rect.2, self.rect.3],
            "nodes": self.nodes.iter().map(|nd| json!({
                "id": format!("vnode_{}", nd.id),
                "kind": nd.kind,
                "span": nd.span.map(|(o, l)| json!([o, l])),
                "source": nd.source,
                "structure": node_to_json(&nd.structure),
            })).collect::<Vec<_>>(),
        })
    }
}

// ============================================================================
// 切片与缩进归一
// ============================================================================

/// 按 span 字节区间切片；非 char 边界或越界 → None（降级安全）。
pub fn slice_source(text: Option<&str>, offset: usize, len: usize) -> Option<String> {
    let text = text?;
    let end = offset.checked_add(len)?;
    if end > text.len() {
        return None;
    }
    // is_char_boundary 防御非对齐 span（正常 span 源自解析器，必然对齐）。
    if !text.is_char_boundary(offset) || !text.is_char_boundary(end) {
        return None;
    }
    Some(text[offset..end].to_string())
}

/// 去公共缩进 + 去首尾空行（切片展示归一）。
pub fn dedent(text: String) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let non_empty: Vec<&&str> = lines.iter().filter(|l| !l.trim().is_empty()).collect();
    if non_empty.is_empty() {
        return String::new();
    }
    let prefix = non_empty
        .iter()
        .map(|l| l.len() - l.trim_start().len())
        .min()
        .unwrap_or(0);
    let trimmed: &[&str] = {
        let first = lines.iter().position(|l| !l.trim().is_empty()).unwrap_or(0);
        let last = lines
            .iter()
            .rposition(|l| !l.trim().is_empty())
            .unwrap_or(0);
        &lines[first..=last]
    };
    trimmed
        .iter()
        .map(|l| l.get(prefix..).unwrap_or(l))
        .collect::<Vec<_>>()
        .join("\n")
}

// ============================================================================
// auto_val::Node → JSON walker（本地转换，不动 auto-val crate）
// ============================================================================

/// [`Value`] → serde_json。VTreeAtomBuilder 产物只会出现 Str/Int/Float/Bool，
/// 其余变体防御性映射（未知走 Display 字符串），永不 panic。
pub fn value_to_json(v: &Value) -> serde_json::Value {
    match v {
        Value::Byte(b) => json!(b),
        Value::Int(n) => json!(n),
        Value::Uint(n) => json!(n),
        Value::USize(n) => json!(n),
        Value::I8(n) => json!(n),
        Value::U8(n) => json!(n),
        Value::I64(n) => json!(n),
        Value::Float(f) | Value::Double(f) => json!(f),
        Value::Bool(b) => json!(b),
        Value::Char(c) => json!(c.to_string()),
        Value::Str(s) => json!(s.to_string()),
        Value::String(s) => json!(s.to_string()),
        Value::StrSlice(s) => json!(s.to_string()),
        Value::CStr(s) => json!(s.to_string()),
        Value::Array(a) | Value::Block(a) => {
            serde_json::Value::Array(a.iter().map(value_to_json).collect())
        }
        Value::Obj(o) => {
            let mut m = serde_json::Map::new();
            for (k, val) in o.iter() {
                m.insert(k.to_astr().to_string(), value_to_json(val));
            }
            serde_json::Value::Object(m)
        }
        Value::Node(n) => node_to_json(n),
        Value::Pair(k, val) => {
            let mut m = serde_json::Map::new();
            m.insert(k.to_astr().to_string(), value_to_json(val));
            serde_json::Value::Object(m)
        }
        Value::Some(inner) | Value::Ok(inner) => value_to_json(inner),
        Value::Nil | Value::Null | Value::None | Value::Void => serde_json::Value::Null,
        other => json!(other.to_string()),
    }
}

/// [`Node`] → serde_json：`{tag, id?, props?, children?}`。
/// VTreeAtomBuilder 产物为规则节点（body props + node kids），lazy kids 跳过。
pub fn node_to_json(n: &Node) -> serde_json::Value {
    let mut m = serde_json::Map::new();
    m.insert("tag".to_string(), json!(n.name.to_string()));
    let id = n.id.to_string();
    if !id.is_empty() {
        m.insert("id".to_string(), json!(id));
    }
    let props: serde_json::Map<String, serde_json::Value> = n
        .body_props_iter()
        .map(|(k, v)| (k.to_astr().to_string(), value_to_json(v)))
        .collect();
    if !props.is_empty() {
        m.insert("props".to_string(), serde_json::Value::Object(props));
    }
    let children: Vec<serde_json::Value> = n
        .kids_iter()
        .filter_map(|(_, kid)| match kid {
            Kid::Node(child) => Some(node_to_json(child)),
            Kid::Lazy(_) => None,
        })
        .collect();
    if !children.is_empty() {
        m.insert("children".to_string(), serde_json::Value::Array(children));
    }
    serde_json::Value::Object(m)
}

// 供 trim 引用方（mod.rs re-export）与 VM 交互臂复用的便利包装：
/// 一步完成 select→trim→envelope（VM `__bounds_collected` 臂与 MCP 工具共用）。
pub fn select_envelope(
    surface: &'static str,
    app: &str,
    rect: Rect,
    bounds: &HashMap<VNodeId, Rect>,
    vtree: &VTree,
    source_text: Option<&str>,
) -> SelectionResult {
    let hit = super::select_nodes(rect, bounds);
    let topmost = trim_to_topmost(&hit, vtree);
    build_selection_result(surface, app, rect, &topmost, vtree, source_text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::debug::SourceSpan;
    use crate::ui::vnode::{VNode, VNodeKind, VNodeProps};

    /// root(1, span 0..41) → text(2, span 8..24)
    fn sample_tree() -> VTree {
        let mut t = VTree::new();
        let root = VNode::new(VNodeId::new(1), VNodeKind::Column, VNodeProps::Empty);
        t.set_root(root);
        let mut text = VNode::new(VNodeId::new(2), VNodeKind::Text, VNodeProps::Empty)
            .with_parent(VNodeId::new(1));
        text.source_span = Some(SourceSpan {
            offset: TEXT_SPAN.0,
            len: TEXT_SPAN.1,
        });
        t.add_node(text);
        t.get_mut(VNodeId::new(1))
            .unwrap()
            .add_child(VNodeId::new(2));
        t
    }

    const SAMPLE_SRC: &str = "app Notes {\n    col {\n        text \"hello world!\"\n    }\n}\n";
    /// `text "hello world!"` 在 SAMPLE_SRC 中的字节区间。
    const TEXT_SPAN: (usize, usize) = (30, 19);

    #[test]
    fn slice_is_byte_exact_substring() {
        // 验收锚点：切片逐字节等于源码子串（缩进归一前）
        let raw = slice_source(Some(SAMPLE_SRC), TEXT_SPAN.0, TEXT_SPAN.1).unwrap();
        assert_eq!(raw, &SAMPLE_SRC[30..49]);
        assert_eq!(raw, "text \"hello world!\"");
    }

    #[test]
    fn slice_out_of_bounds_is_none() {
        assert!(slice_source(None, 0, 4).is_none());
        assert!(slice_source(Some("ab"), 1, 4).is_none());
        // 非 char 边界防御
        assert!(slice_source(Some("中文"), 1, 2).is_none());
    }

    #[test]
    fn dedent_strips_common_indent_and_blank_edges() {
        let src = "\n    col {\n        text \"x\"\n    }\n";
        assert_eq!(dedent(src.to_string()), "col {\n    text \"x\"\n}");
        assert_eq!(dedent("   \n  a\n   ".to_string()), "a");
    }

    #[test]
    fn envelope_build_and_auto_render() {
        let t = sample_tree();
        let res = build_selection_result(
            "vm",
            "Notes",
            Rect::new(0.0, 0.0, 100.0, 50.0),
            &[VNodeId::new(1)],
            &t,
            Some(SAMPLE_SRC),
        );
        assert_eq!(res.surface, "vm");
        assert_eq!(res.app, "Notes");
        assert_eq!(res.rect, (0.0, 0.0, 100.0, 50.0));
        assert_eq!(res.nodes.len(), 1);
        let n0 = &res.nodes[0];
        assert_eq!(n0.kind, "col");
        // col 节点无 span（测试树只给 text 设了 span）→ 合成降级
        assert!(n0.span.is_none() && n0.source.is_none());

        let auto = res.render(SelectionFormat::Auto);
        assert!(auto.contains("surface=vm app=Notes"));
        assert!(auto.contains("nodes=1"));
        assert!(auto.contains("(synthetic, no source span)"));
        assert!(auto.contains("col vnode_1"), "structure atom: {auto}");
        // 子孙（text, 有 span）出现在 structure 中 —— VTree 拓扑完整
        assert!(auto.contains("text vnode_2"));
    }

    #[test]
    fn envelope_source_slice_for_span_node() {
        let t = sample_tree();
        let res = build_selection_result(
            "vm",
            "Notes",
            Rect::new(0.0, 0.0, 10.0, 10.0),
            &[VNodeId::new(2)],
            &t,
            Some(SAMPLE_SRC),
        );
        let auto = res.render_auto();
        assert!(auto.contains("[1/1] text  span=30..49"), "{auto}");
        assert!(auto.contains("text \"hello world!\""));
        // JSON 信封字段完整
        let j = res.to_json();
        assert_eq!(j["surface"], "vm");
        assert_eq!(j["nodes"][0]["kind"], "text");
        assert_eq!(j["nodes"][0]["span"], json!([30, 19]));
        assert!(j["nodes"][0]["source"]
            .as_str()
            .unwrap()
            .starts_with("text"));
        assert_eq!(j["nodes"][0]["structure"]["tag"], "text");
        // Atom 视图可输出
        assert!(res.render_atom().contains("text vnode_2"));
    }

    #[test]
    fn empty_selection_renders_marker() {
        let t = sample_tree();
        let res = select_envelope(
            "vm",
            "Notes",
            Rect::new(900.0, 900.0, 10.0, 10.0),
            &HashMap::new(),
            &t,
            Some(SAMPLE_SRC),
        );
        assert!(res.nodes.is_empty());
        assert!(res.render_auto().contains("(no nodes selected)"));
        assert_eq!(res.to_json()["nodes"], json!([]));
    }

    #[test]
    fn node_to_json_round_trip_basic() {
        use auto_val::Node as AN;
        let mut n = AN::new("col");
        n.id = "vnode_0".into();
        n.set_prop("content", Value::Str("hi".into()));
        let mut child = AN::new("text");
        child.set_prop("content", Value::Int(3));
        let n = n.with_child(child);

        let j = node_to_json(&n);
        assert_eq!(j["tag"], "col");
        assert_eq!(j["id"], "vnode_0");
        assert_eq!(j["props"]["content"], "hi");
        assert_eq!(j["children"][0]["tag"], "text");
        assert_eq!(j["children"][0]["props"]["content"], 3);
    }

    #[test]
    fn select_envelope_trims_to_topmost() {
        let t = sample_tree();
        // 覆盖两节点中心的矩形 → 双命中 → 修剪为 root
        let mut bounds = HashMap::new();
        bounds.insert(VNodeId::new(1), Rect::new(0.0, 0.0, 100.0, 100.0));
        bounds.insert(VNodeId::new(2), Rect::new(10.0, 10.0, 50.0, 20.0));
        let res = select_envelope(
            "vm",
            "N",
            Rect::new(0.0, 0.0, 100.0, 100.0),
            &bounds,
            &t,
            None,
        );
        assert_eq!(res.nodes.len(), 1);
        assert_eq!(res.nodes[0].kind, "col");
    }

    #[test]
    fn format_parse_and_cycle() {
        assert_eq!(SelectionFormat::parse("auto"), SelectionFormat::Auto);
        assert_eq!(SelectionFormat::parse("ATOM"), SelectionFormat::Atom);
        assert_eq!(SelectionFormat::parse("bogus"), SelectionFormat::Json);
        assert_eq!(
            SelectionFormat::Auto.next().next().next(),
            SelectionFormat::Auto
        );
    }
}
