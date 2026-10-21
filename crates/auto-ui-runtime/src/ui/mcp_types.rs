//! # MCP Types for AutoUI Desktop Integration (Plan 278)
//!
//! Defines data structures for the MCP tools that allow AI agents to
//! inspect and manipulate AutoUI desktop interfaces.
//!
//! ## Tools
//!
//! - `autoui_snapshot` — Capture full page structure as AURA text
//! - `autoui_inspect` — Inspect a single element by AuraNodeId
//! - `autoui_action`  — Perform an action on an element

use std::fmt;

use auto_ui_compiler::aura::AuraNodeId;
use auto_val::Value;

// ============================================================================
// Snapshot Types
// ============================================================================

/// A complete snapshot of the current UI page.
///
/// Contains the widget name, all state variable values,
/// and the full component tree with properties and actions.
#[derive(Debug, Clone)]
pub struct UiSnapshot {
    /// Widget name (e.g., "TodoApp")
    pub widget_name: String,
    /// State variable values: (name, value, type_hint)
    pub state: Vec<(String, String, String)>,
    /// Root of the component tree
    pub tree: UiNode,
}

/// A node in the UI component tree.
///
/// Each node represents a single visual component (Button, Input, Text, etc.)
/// with its current properties, available actions, and children.
#[derive(Debug, Clone)]
pub struct UiNode {
    /// Stable component ID (AuraNodeId)
    pub id: AuraNodeId,
    /// Component kind (e.g., "Button", "Input", "Text", "Column")
    pub kind: String,
    /// Key-value properties (e.g., "label" -> "Add", "placeholder" -> "Type here")
    pub props: Vec<(String, String)>,
    /// Available actions on this component
    pub actions: Vec<UiAction>,
    /// Child nodes
    pub children: Vec<UiNode>,
}

/// An action available on a UI component.
#[derive(Debug, Clone)]
pub struct UiAction {
    /// Action name (e.g., "press", "type", "toggle", "select", "set_value")
    pub name: String,
    /// Handler that will be triggered (e.g., ".AddTodo")
    pub handler: String,
}

// ============================================================================
// Inspect Types
// ============================================================================

/// Detailed information about a single UI element.
#[derive(Debug, Clone)]
pub struct ElementInfo {
    /// Component ID
    pub id: AuraNodeId,
    /// Component kind
    pub kind: String,
    /// Key-value properties
    pub props: Vec<(String, String)>,
    /// Available actions
    pub actions: Vec<UiAction>,
    /// Source location (if available)
    pub source_location: Option<SourceLocation>,
}

/// Source code location for a UI element.
#[derive(Debug, Clone)]
pub struct SourceLocation {
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub col: usize,
}

// ============================================================================
// Action Types
// ============================================================================

/// Actions that can be performed on UI elements.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiActionType {
    /// Press a button (triggers onclick handler)
    Press,
    /// Type text into an input/textarea
    TypeText,
    /// Toggle a checkbox
    Toggle,
    /// Select an option from a dropdown/radio
    SelectOption,
    /// Set a slider value
    SetValue,
    /// Clear an input/textarea (Plan 299)
    Clear,
    /// Submit an input (triggers onsubmit/onenter — e.g. pressing Enter in a
    /// command prompt). Plan 371 续篇 / ash-gui M1:needed because the keyboard
    /// tool dispatches a global key handler, not the iced input's on_submit.
    Submit,
    /// Scroll a Scrollable to a y offset (PLAN-043 T6)。经合成事件
    /// `__mcp_scroll` 直落 iced scroll_to——scrollable 无 VM handler 可提
    /// 取，与 press/type 的 handler 通道不同。
    Scroll,
    /// Drag a MouseArea widget through its press/move/release handlers
    /// (PLAN-043 T10)。经合成事件 `__mcp_drag` 在 update_inner 连发
    /// on_with_input_for（down → move(x,y)×n → up），与 PointerArea
    /// 闭包产出的消息同构——.at 拖拽数学、emit、SetScrollTop fast-path、
    /// 写臂 scroll_to 全走真实机制。mouse-area 无 vnode 可寻址
    /// （vnode_converter 映射为 Text），寻址由 value 携带 widget 名。
    Drag,
    /// Resize a table column (PLAN-045 T7)。对 Table 节点直发（坐标命中
    /// 层由 rust 单测覆盖——待澄清②）经合成事件 `__mcp_resize_col`
    ///（input_value = "t{表键}␟col␟width"）走 OnColResize 拦截同一
    /// rust 直写快道。
    ResizeCol,
    /// Synthesize a single key press into the autodown editor shell
    /// (PLAN-057，055 D2)。经合成事件 `__mcp_key`（input_value =
    /// "storage_key␟widget␟event␟keyspec"）直调 core.handle_input
    /// (KeyPressed)——真实按键的同构 core 处理路径；text_changed 时按
    /// 编辑壳 on_change 同形态发布（input_value: Some(全文)）。
    KeyPress,
    /// Synthesize a mouse drag sequence inside the autodown editor shell
    /// (PLAN-057，055 D2)。经合成事件 `__mcp_drag_ade`（input_value =
    /// "storage_key␟widget␟event␟x0,y0;x1,y1;..."）逐段直调
    /// core.handle_input（MousePressed → MouseDragged×n →
    /// MouseReleased）——列宽拖拽（055）与拖选（048 T3）面同覆盖。
    EditorDrag,
}

impl fmt::Display for UiActionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UiActionType::Press => write!(f, "press"),
            UiActionType::TypeText => write!(f, "type_text"),
            UiActionType::Toggle => write!(f, "toggle"),
            UiActionType::SelectOption => write!(f, "select_option"),
            UiActionType::SetValue => write!(f, "set_value"),
            UiActionType::Clear => write!(f, "clear"),
            UiActionType::Submit => write!(f, "submit"),
            UiActionType::Scroll => write!(f, "scroll"),
            UiActionType::Drag => write!(f, "drag"),
            UiActionType::ResizeCol => write!(f, "resize_col"),
            UiActionType::KeyPress => write!(f, "key_press"),
            UiActionType::EditorDrag => write!(f, "editor_drag"),
        }
    }
}

/// Result of performing an action on a UI element.
#[derive(Debug, Clone)]
pub struct ActionResult {
    /// Whether the action succeeded
    pub status: String,
    /// Target element ID
    pub element_id: AuraNodeId,
    /// Action that was performed
    pub action: String,
    /// Handler that was triggered (if any)
    pub handler: Option<String>,
    /// State changes: (field_name, before, after)
    pub state_changes: Vec<(String, String, String)>,
}

// ============================================================================
// AURA Text Formatting
// ============================================================================

impl UiSnapshot {
    /// Format the snapshot as AURA text.
    ///
    /// Returns a human-readable, machine-parseable text representation
    /// of the complete UI state.
    pub fn to_aura_string(&self) -> String {
        let mut lines = Vec::new();

        lines.push("AuraUI Snapshot v1".to_string());
        lines.push(format!("widget: {:?}", self.widget_name));

        // State section
        if !self.state.is_empty() {
            lines.push("state:".to_string());
            for (name, value, type_hint) in &self.state {
                lines.push(format!("  {}: {} ({})", name, value, type_hint));
            }
        }

        // Tree section
        lines.push("".to_string());
        lines.push("tree:".to_string());
        self.tree.format_aura(&mut lines, 1);

        lines.join("\n")
    }
}

impl UiNode {
    /// Format this node and its children as AURA text.
    fn format_aura(&self, lines: &mut Vec<String>, indent: usize) {
        let pad = "  ".repeat(indent);

        // Component header: Kind #aura_N {
        lines.push(format!("{}{} #{} {{", pad, self.kind, self.id));

        let inner_pad = "  ".repeat(indent + 1);

        // Properties
        for (key, value) in &self.props {
            lines.push(format!("{}{}: {:?}", inner_pad, key, value));
        }

        // Actions
        if !self.actions.is_empty() {
            let action_strs: Vec<String> = self
                .actions
                .iter()
                .map(|a| format!("{} -> {}", a.name, a.handler))
                .collect();
            lines.push(format!(
                "{}actions: [{}]",
                inner_pad,
                action_strs.join(", ")
            ));
        }

        // Children
        for child in &self.children {
            child.format_aura(lines, indent + 1);
        }

        // Closing brace
        lines.push(format!("{}}}", pad));
    }
}

impl ElementInfo {
    /// Format as AURA text.
    pub fn to_aura_string(&self) -> String {
        let mut lines = Vec::new();

        lines.push(format!("Inspect #{}", self.id));
        lines.push(format!("  type: {}", self.kind));
        lines.push("  properties:".to_string());
        for (key, value) in &self.props {
            lines.push(format!("    {}: {:?}", key, value));
        }
        if !self.actions.is_empty() {
            lines.push("  actions:".to_string());
            for action in &self.actions {
                lines.push(format!("    {} -> {}", action.name, action.handler));
            }
        }
        if let Some(loc) = &self.source_location {
            lines.push(format!("  source: line {}, col {}", loc.line, loc.col));
        }

        lines.join("\n")
    }
}

impl ActionResult {
    /// Format as AURA text.
    pub fn to_aura_string(&self) -> String {
        let mut lines = Vec::new();

        lines.push("ActionResult".to_string());
        lines.push(format!("  status: {}", self.status));
        lines.push(format!("  element: #{}", self.element_id));
        lines.push(format!("  action: {}", self.action));
        if let Some(handler) = &self.handler {
            lines.push(format!("  handler: {}", handler));
        }
        if !self.state_changes.is_empty() {
            lines.push("  state_changes:".to_string());
            for (field, before, after) in &self.state_changes {
                lines.push(format!("    {}: {} -> {}", field, before, after));
            }
        }

        lines.join("\n")
    }
}

/// Format a Value for display in AURA output.
pub fn format_value(v: &Value) -> String {
    match v {
        Value::Int(i) => i.to_string(),
        Value::Float(f) => format!("{:.2}", f),
        Value::Bool(b) => b.to_string(),
        Value::Str(s) => format!("{:?}", s), // quoted string
        Value::Null => "null".to_string(),
        other => other.to_string(),
    }
}

/// Infer a type hint from a Value.
pub fn type_hint(v: &Value) -> &'static str {
    match v {
        Value::Int(_) => "int",
        Value::Float(_) => "float",
        Value::Bool(_) => "bool",
        Value::Str(_) => "str",
        Value::Null => "null",
        Value::Array(_) => "list",
        Value::Obj(_) => "object",
        _ => "unknown",
    }
}

// ============================================================================
// PLAN-814 R9-03：以下三项自 iced 模块上移的单源定义（纯数据/纯函数，
// 无 iced 类型）——headless 构建的 mcp_server 消费面；iced 侧
// （renderer.rs / scroll_state_reader.rs）经再导出保持旧路径与类型身份。
// ============================================================================

/// typed onclick payload 在 `event` String 中的分隔符（原 iced/renderer
/// 单源上移——iced Send 边界与 decode_payload（dynamic.rs）配对）。
pub(crate) const PAYLOAD_SEP: char = '\u{1F}';

/// 事件参数编码（原 iced/renderer 单源上移；纯 string 构建，无 iced）。
pub(crate) fn encode_payload(event_name: &str, args: &[auto_val::Value]) -> String {
    if args.is_empty() {
        return event_name.to_string();
    }
    let mut out = String::from(event_name);
    for v in args {
        let (tc, val) = match v {
            auto_val::Value::Int(i) => ("i", i.to_string()),
            auto_val::Value::Uint(u) => ("u", u.to_string()),
            auto_val::Value::Bool(b) => ("b", (if *b { "1" } else { "0" }).to_string()),
            auto_val::Value::Float(f) => ("f", f.to_string()),
            auto_val::Value::Double(d) => ("d", d.to_string()),
            auto_val::Value::Str(s) => ("s", s.as_str().to_string()),
            _ => continue,
        };
        out.push(PAYLOAD_SEP);
        out.push_str(tc);
        out.push(PAYLOAD_SEP);
        out.push_str(&val);
    }
    out
}

/// pane 几何读回条目（原 iced/scroll_state_reader 单源上移；纯数据）。
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct ScrollStateEntry {
    /// viewport 矩形 [x, y, w, h]（窗口坐标，pane 的可见裁剪区）。
    pub viewport: [f32; 4],
    /// 滚动 offset [ox, oy]（正向滚动为正；overscroll 瞬态 clamp 0）。
    pub offset: [f32; 2],
    /// content extent [w, h]。
    pub content: [f32; 2],
}

/// 读回结果：widget id 串 → pane 几何（键形如 `scroll_ctl_<controller>`
/// / `vnode_<hash>` / `aura_N`——renderer 句柄派生稳定 id）。
pub type ScrollStateMap = std::collections::HashMap<String, ScrollStateEntry>;
