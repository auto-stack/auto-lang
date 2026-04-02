// Node → View Converter for Auto Language Integration
//
// This module converts AutoLang's Value::Node (runtime AST) into AutoUI's View<M>
// to enable runtime interpretation mode without transpilation.

use auto_val::{Value, Node};
use super::view::{View, SelectCallback};
use super::style::Style;

// 导出动态消息类型（当 interpreter feature 启用时）
#[cfg(feature = "interpreter")]
use super::interpreter::DynamicMessage;

/// Errors that can occur during node conversion
#[derive(Debug, Clone)]
pub enum ConversionError {
    /// Unknown node kind (e.g., "unknown_widget")
    UnknownKind { kind: String },
    /// Missing required property
    MissingProp { kind: String, prop: String },
    /// Invalid property value type
    InvalidPropType {
        kind: String,
        prop: String,
        expected: String,
        got: String,
    },
    /// Message type is required for this node (e.g., Button needs onclick)
    MessageRequired { kind: String },
}

impl std::fmt::Display for ConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConversionError::UnknownKind { kind } => {
                write!(f, "Unknown UI node kind: '{}'", kind)
            }
            ConversionError::MissingProp { kind, prop } => {
                write!(f, "Missing required property '{}' for node kind '{}'", prop, kind)
            }
            ConversionError::InvalidPropType { kind, prop, expected, got } => {
                write!(f, "Invalid type for property '{}' on node kind '{}': expected {}, got {}", prop, kind, expected, got)
            }
            ConversionError::MessageRequired { kind } => {
                write!(f, "Message type required for node kind '{}' (runtime interpretation not yet supported)", kind)
            }
        }
    }
}

impl std::error::Error for ConversionError {}

/// Result type for conversion operations
pub type ConversionResult<T> = Result<T, ConversionError>;

/// Convert a Value::Node to View<String> for runtime interpretation
///
/// # Message Type
///
/// Uses `View<String>` where messages are stored as string identifiers
/// (e.g., "button-clicked"). This is a temporary limitation - future versions
/// will support typed message conversion via metadata.
///
/// # Limitations
///
/// - Event handlers (onclick, on_change, etc.) stored as strings, not typed messages
/// - Style strings parsed at runtime (Plan 005 integration)
/// - Type safety limited compared to transpiled Rust code
///
/// # Example
///
/// ```ignore
/// // Auto code: `text("Hello") {}`
/// let node = Node::new("text").with_arg("Hello");
/// let view = convert_node(&node)?;
/// assert!(matches!(view, View::Text { content: "Hello", .. }));
/// ```
pub fn convert_node(node: &Node) -> ConversionResult<View<String>> {
    let kind = node.name.as_str();

    match kind {
        // Layout Components
        "center" => convert_center(node),
        "col" | "column" => convert_column(node),
        "row" => convert_row(node),
        "container" => convert_container(node),
        "scrollable" => convert_scrollable(node),

        // Element Components
        "text" | "label" => convert_text(node),
        "button" => convert_button(node),
        "input" => convert_input(node),
        "checkbox" => convert_checkbox(node),
        "radio" => convert_radio(node),
        "select" => convert_select(node),
        "list" => convert_list(node),
        "table" => convert_table(node),

        // Unknown kind
        _ => Err(ConversionError::UnknownKind {
            kind: kind.to_string(),
        }),
    }
}

// ============================================================================
// Layout Component Converters
// ============================================================================

/// Convert Center node: `center { child }`
fn convert_center(node: &Node) -> ConversionResult<View<String>> {
    // Center is a container with center_x and center_y
    let kids = extract_child_nodes(node)?;

    if kids.is_empty() {
        // Empty center - default to empty view
        return Ok(View::container(View::empty())
            .center()
            .center_x()
            .center_y()
            .build());
    }

    // Take first child as the centered content
    let child = convert_node(&kids[0])?;

    Ok(View::container(child)
        .center()
        .center_x()
        .center_y()
        .build())
}

/// Convert Column node: `col { spacing: 10, padding: 20, kids [...] }`
fn convert_column(node: &Node) -> ConversionResult<View<String>> {
    let spacing = extract_prop_u16(node, "spacing").unwrap_or(0);
    let padding = extract_prop_u16(node, "padding").unwrap_or(0);
    let style = extract_style(node)?;

    let kids = extract_child_nodes(node)?;
    let children: Result<Vec<_>, _> = kids.iter().map(convert_node).collect();
    let children = children?;

    let mut builder = View::col().spacing(spacing).padding(padding);

    if let Some(style) = style {
        builder = builder.with_style(style);
    }

    for child in children {
        builder = builder.child(child);
    }

    Ok(builder.build())
}

/// Convert Row node: `row { spacing: 10, padding: 20, kids [...] }`
fn convert_row(node: &Node) -> ConversionResult<View<String>> {
    let spacing = extract_prop_u16(node, "spacing").unwrap_or(0);
    let padding = extract_prop_u16(node, "padding").unwrap_or(0);
    let style = extract_style(node)?;

    let kids = extract_child_nodes(node)?;
    let children: Result<Vec<_>, _> = kids.iter().map(convert_node).collect();
    let children = children?;

    let mut builder = View::row().spacing(spacing).padding(padding);

    if let Some(style) = style {
        builder = builder.with_style(style);
    }

    for child in children {
        builder = builder.child(child);
    }

    Ok(builder.build())
}

/// Convert Container node: `container { width: 100, height: 200, child }`
fn convert_container(node: &Node) -> ConversionResult<View<String>> {
    let padding = extract_prop_u16(node, "padding").unwrap_or(0);
    let width = extract_prop_opt_u16(node, "width");
    let height = extract_prop_opt_u16(node, "height");
    let center_x = extract_prop_bool(node, "center_x").unwrap_or(false);
    let center_y = extract_prop_bool(node, "center_y").unwrap_or(false);
    let style = extract_style(node)?;

    let kids = extract_child_nodes(node)?;

    if kids.is_empty() {
        // Empty container
        let mut builder = View::container(View::empty()).padding(padding);
        if center_x { builder = builder.center_x(); }
        if center_y { builder = builder.center_y(); }
        if let Some(style) = style {
            builder = builder.with_style(style);
        }
        return Ok(builder.build());
    }

    let child = convert_node(&kids[0])?;

    let mut builder = View::container(child).padding(padding);
    if let Some(w) = width { builder = builder.width(w); }
    if let Some(h) = height { builder = builder.height(h); }
    if center_x { builder = builder.center_x(); }
    if center_y { builder = builder.center_y(); }
    if let Some(style) = style {
        builder = builder.with_style(style);
    }

    Ok(builder.build())
}

/// Convert Scrollable node: `scrollable { width: 100, height: 200, child }`
fn convert_scrollable(node: &Node) -> ConversionResult<View<String>> {
    let width = extract_prop_opt_u16(node, "width");
    let height = extract_prop_opt_u16(node, "height");
    let style = extract_style(node)?;

    let kids = extract_child_nodes(node)?;

    if kids.is_empty() {
        // Empty scrollable
        let mut builder = View::scrollable(View::empty());
        if let Some(w) = width { builder = builder.width(w); }
        if let Some(h) = height { builder = builder.height(h); }
        if let Some(style) = style {
            builder = builder.with_style(style);
        }
        return Ok(builder.build());
    }

    let child = convert_node(&kids[0])?;

    let mut builder = View::scrollable(child);
    if let Some(w) = width { builder = builder.width(w); }
    if let Some(h) = height { builder = builder.height(h); }
    if let Some(style) = style {
        builder = builder.with_style(style);
    }

    Ok(builder.build())
}

// ============================================================================
// Element Component Converters
// ============================================================================

/// Convert Text node: `text("Hello") { style: "text-lg" }`
fn convert_text(node: &Node) -> ConversionResult<View<String>> {
    // Get main argument as text content
    let content = extract_main_arg_str(node).unwrap_or_default();

    let style = extract_style(node)?;

    if let Some(style) = style {
        Ok(View::Text {
            content,
            style: Some(style),
        })
    } else {
        Ok(View::text(content))
    }
}

/// Convert Button node: `button("Click") { onclick: "clicked", style: "px-4" }`
fn convert_button(node: &Node) -> ConversionResult<View<String>> {
    let label = extract_main_arg_str(node).unwrap_or_default();

    // onclick is required for buttons
    let onclick = extract_prop_str(node, "onclick")
        .ok_or_else(|| ConversionError::MissingProp {
            kind: "button".to_string(),
            prop: "onclick".to_string(),
        })?;

    let style = extract_style(node)?;

    if let Some(style) = style {
        Ok(View::Button {
            label,
            onclick, // Store as string message ID
            style: Some(style),
        })
    } else {
        Ok(View::button(label, onclick))
    }
}

/// Convert Input node: `input("Email") { value: "", on_change: "changed" }`
fn convert_input(node: &Node) -> ConversionResult<View<String>> {
    let placeholder = extract_main_arg_str(node).unwrap_or_default();
    let value = extract_prop_str(node, "value").unwrap_or_default();
    let on_change = extract_prop_opt_str(node, "on_change");
    let width = extract_prop_opt_u16(node, "width");
    let password = extract_prop_bool(node, "password").unwrap_or(false);
    let style = extract_style(node)?;

    let mut builder = View::input(placeholder).value(value);

    if password {
        builder = builder.password();
    }
    if let Some(msg) = on_change {
        builder = builder.on_change(msg);
    }
    if let Some(w) = width {
        builder = builder.width(w);
    }
    if let Some(style) = style {
        builder = builder.with_style(style);
    }

    Ok(builder.build())
}

/// Convert Checkbox node: `checkbox("Remember") { is_checked: true, on_toggle: "toggle" }`
fn convert_checkbox(node: &Node) -> ConversionResult<View<String>> {
    let label = extract_main_arg_str(node).unwrap_or_default();
    let is_checked = extract_prop_bool(node, "is_checked").unwrap_or(false);
    let on_toggle = extract_prop_opt_str(node, "on_toggle");
    let style = extract_style(node)?;

    let mut view = View::checkbox(is_checked, label);

    if let Some(msg) = on_toggle {
        view = view.on_toggle(msg);
    }
    if let Some(style) = style {
        if let View::Checkbox { style: s, .. } = &mut view {
            *s = Some(style);
        }
    }

    Ok(view)
}

/// Convert Radio node: `radio("Option") { is_selected: false, on_select: "select" }`
fn convert_radio(node: &Node) -> ConversionResult<View<String>> {
    let label = extract_main_arg_str(node).unwrap_or_default();
    let is_selected = extract_prop_bool(node, "is_selected").unwrap_or(false);
    let on_select = extract_prop_opt_str(node, "on_select");
    let style = extract_style(node)?;

    let mut view = View::radio(is_selected, label);

    if let Some(msg) = on_select {
        view = view.on_select(msg);
    }
    if let Some(style) = style {
        if let View::Radio { style: s, .. } = &mut view {
            *s = Some(style);
        }
    }

    Ok(view)
}

/// Convert Select node: `select { options: ["A", "B"], selected_index: 0, on_select: "change" }`
fn convert_select(node: &Node) -> ConversionResult<View<String>> {
    let options = extract_prop_str_array(node, "options").unwrap_or_default();
    let selected_index = extract_prop_opt_usize(node, "selected_index");
    let on_select = extract_prop_opt_str(node, "on_select");
    let style = extract_style(node)?;

    let mut view = View::select(options);

    if let Some(idx) = selected_index {
        view = view.selected(idx);
    }
    if let Some(msg) = on_select {
        // Create a callback that returns the message
        // Note: This ignores the selected value and just returns the fixed message
        // This is a limitation of the Auto language string-based message system
        view = view.on_choose(move |_index, _value| msg.clone());
    }
    if let Some(style) = style {
        if let View::Select { style: s, .. } = &mut view {
            *s = Some(style);
        }
    }

    Ok(view)
}

/// Convert List node: `list { spacing: 10, items: [...] }`
fn convert_list(node: &Node) -> ConversionResult<View<String>> {
    let spacing = extract_prop_u16(node, "spacing").unwrap_or(0);
    let style = extract_style(node)?;

    let kids = extract_child_nodes(node)?;
    let items: Result<Vec<_>, _> = kids.iter().map(convert_node).collect();
    let items = items?;

    let mut builder = View::list(items).spacing(spacing);

    if let Some(style) = style {
        builder = builder.with_style(style);
    }

    Ok(builder.build())
}

/// Convert Table node: `table { headers: [...], rows: [[...]], spacing: 5 }`
fn convert_table(node: &Node) -> ConversionResult<View<String>> {
    let spacing = extract_prop_u16(node, "spacing").unwrap_or(0);
    let col_spacing = extract_prop_u16(node, "col_spacing").unwrap_or(0);
    let style = extract_style(node)?;

    // Extract headers from child nodes
    let header_kids = extract_child_nodes_by_name(node, "header");
    let headers: Result<Vec<_>, _> = header_kids.iter().map(convert_node).collect();
    let headers = headers?;

    // Extract rows from child nodes
    let row_kids = extract_child_nodes_by_name(node, "row");
    let mut rows = Vec::new();

    for row_node in &row_kids {
        let cell_kids = extract_child_nodes(row_node)?;
        let row: Result<Vec<_>, _> = cell_kids.iter().map(convert_node).collect();
        rows.push(row?);
    }

    let mut builder = View::table(headers, rows).spacing(spacing).col_spacing(col_spacing);

    if let Some(style) = style {
        builder = builder.with_style(style);
    }

    Ok(builder.build())
}

// ============================================================================
// Property Extraction Helpers
// ============================================================================

/// Extract main argument as string
fn extract_main_arg_str(node: &Node) -> Option<String> {
    let arg = node.main_arg();
    match arg {
        Value::Str(s) => Some(s.to_string()),
        Value::String(s) => Some(s.as_str().to_string()),
        _ => None,
    }
}

/// Extract property as string
fn extract_prop_str(node: &Node, key: &str) -> Option<String> {
    let value = node.get_prop(key);
    match value {
        Value::Str(s) => Some(s.to_string()),
        Value::String(s) => Some(s.as_str().to_string()),
        _ => None,
    }
}

/// Extract optional property as string
fn extract_prop_opt_str(node: &Node, key: &str) -> Option<String> {
    extract_prop_str(node, key)
}

/// Extract property as u16
fn extract_prop_u16(node: &Node, key: &str) -> Option<u16> {
    let value = node.get_prop(key);
    match value {
        Value::Int(i) if i >= 0 && i <= u16::MAX as i32 => Some(i as u16),
        Value::Uint(u) if u <= u16::MAX as u32 => Some(u as u16),
        Value::USize(u) if u <= u16::MAX as usize => Some(u as u16),
        _ => None,
    }
}

/// Extract optional property as u16
fn extract_prop_opt_u16(node: &Node, key: &str) -> Option<u16> {
    extract_prop_u16(node, key)
}

/// Extract optional property as usize
fn extract_prop_opt_usize(node: &Node, key: &str) -> Option<usize> {
    let value = node.get_prop(key);
    match value {
        Value::Int(i) if i >= 0 => Some(i as usize),
        Value::Uint(u) => Some(u as usize),
        Value::USize(u) => Some(u),
        _ => None,
    }
}

/// Extract property as bool
fn extract_prop_bool(node: &Node, key: &str) -> Option<bool> {
    let value = node.get_prop(key);
    match value {
        Value::Bool(b) => Some(b),
        _ => None,
    }
}

/// Extract property as u32
fn extract_prop_u32(node: &Node, key: &str) -> Option<u32> {
    let value = node.get_prop(key);
    match value {
        Value::Int(i) if i >= 0 => Some(i as u32),
        Value::Uint(u) => Some(u as u32),
        Value::USize(u) => {
            if u <= u32::MAX as usize {
                Some(u as u32)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Extract property as usize
fn extract_prop_usize(node: &Node, key: &str) -> Option<usize> {
    let value = node.get_prop(key);
    match value {
        Value::Int(i) if i >= 0 => Some(i as usize),
        Value::Uint(u) => Some(u as usize),
        Value::USize(u) => Some(u),
        _ => None,
    }
}

/// Extract children as strings (for Select options)
fn extract_children_strings(node: &Node) -> ConversionResult<Vec<String>> {
    let mut strings = Vec::new();

    // 使用 kids_iter() 获取子节点
    for (_, kid) in node.kids_iter() {
        if let auto_val::Kid::Node(child_node) = kid {
            // 尝试将节点的第一个参数转换为字符串
            // 使用 main_arg() 获取主参数
            let main_arg = child_node.main_arg();
            match main_arg {
                Value::Str(s) => strings.push(s.to_string()),
                Value::String(s) => strings.push(s.as_str().to_string()),
                Value::Int(i) => strings.push(i.to_string()),
                Value::Uint(u) => strings.push(u.to_string()),
                Value::Bool(b) => strings.push(b.to_string()),
                _ => {
                    // 如果第一个参数不是简单值，使用节点名称
                    strings.push(child_node.name.to_string());
                }
            }
        }
    }

    Ok(strings)
}

/// Extract property as string array
fn extract_prop_str_array(node: &Node, key: &str) -> Option<Vec<String>> {
    let value = node.get_prop(key);
    match value {
        Value::Array(arr) => {
            let strs: Vec<_> = arr.iter().filter_map(|v| match v {
                Value::Str(s) => Some(s.to_string()),
                Value::String(s) => Some(s.as_str().to_string()),
                _ => None,
            }).collect();
            if strs.is_empty() { None } else { Some(strs) }
        }
        _ => None,
    }
}

/// Extract child nodes (all nodes in kids)
fn extract_child_nodes(node: &Node) -> ConversionResult<Vec<Node>> {
    let kids = node.kids_iter()
        .filter_map(|(_, kid)| {
            if let auto_val::Kid::Node(n) = kid {
                Some(n.clone())
            } else {
                None
            }
        })
        .collect();

    Ok(kids)
}

/// Extract child nodes by name
fn extract_child_nodes_by_name(node: &Node, name: &str) -> Vec<Node> {
    node.kids_iter()
        .filter_map(|(_, kid)| {
            if let auto_val::Kid::Node(n) = kid {
                if n.name.as_str() == name {
                    Some(n.clone())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect()
}

/// Extract style string and parse it (Plan 005 integration)
fn extract_style(node: &Node) -> ConversionResult<Option<Style>> {
    let style_str = extract_prop_str(node, "style");

    if let Some(s) = style_str {
        // Parse style string using Plan 005's Style::parse
        Style::parse(&s)
            .map(Some)
            .map_err(|e| ConversionError::InvalidPropType {
                kind: node.name.to_string(),
                prop: "style".to_string(),
                expected: "valid Tailwind CSS class string".to_string(),
                got: format!("invalid: {}", e),
            })
    } else {
        Ok(None)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use auto_val::Node;

    #[test]
    fn test_convert_text() {
        let node = Node::new("text").with_arg("Hello World");
        let view = convert_node(&node).unwrap();

        match view {
            View::Text { content, .. } => {
                assert_eq!(content, "Hello World");
            }
            _ => panic!("Expected View::Text"),
        }
    }

    #[test]
    fn test_convert_button() {
        let node = Node::new("button")
            .with_arg("Click Me")
            .with_prop("onclick", "button-clicked");
        let view = convert_node(&node).unwrap();

        match view {
            View::Button { label, onclick, .. } => {
                assert_eq!(label, "Click Me");
                assert_eq!(onclick, "button-clicked");
            }
            _ => panic!("Expected View::Button"),
        }
    }

    #[test]
    fn test_convert_column() {
        let node = Node::new("col")
            .with_prop("spacing", 10u32)
            .with_child(Node::new("text").with_arg("Item 1"))
            .with_child(Node::new("text").with_arg("Item 2"));

        let view = convert_node(&node).unwrap();

        match view {
            View::Column { spacing, children, .. } => {
                assert_eq!(spacing, 10);
                assert_eq!(children.len(), 2);
            }
            _ => panic!("Expected View::Column"),
        }
    }

    #[test]
    fn test_convert_row() {
        let node = Node::new("row")
            .with_prop("spacing", 5u32)
            .with_child(Node::new("text").with_arg("A"))
            .with_child(Node::new("text").with_arg("B"));

        let view = convert_node(&node).unwrap();

        match view {
            View::Row { spacing, children, .. } => {
                assert_eq!(spacing, 5);
                assert_eq!(children.len(), 2);
            }
            _ => panic!("Expected View::Row"),
        }
    }

    #[test]
    fn test_convert_container() {
        let node = Node::new("container")
            .with_prop("width", 100u32)
            .with_prop("height", 200u32)
            .with_child(Node::new("text").with_arg("Content"));

        let view = convert_node(&node).unwrap();

        match view {
            View::Container { width, height, .. } => {
                assert_eq!(width, Some(100));
                assert_eq!(height, Some(200));
            }
            _ => panic!("Expected View::Container"),
        }
    }

    #[test]
    fn test_convert_checkbox() {
        let node = Node::new("checkbox")
            .with_arg("Remember me")
            .with_prop("is_checked", true)
            .with_prop("on_toggle", "toggle-checkbox");

        let view = convert_node(&node).unwrap();

        match view {
            View::Checkbox { is_checked, label, on_toggle, .. } => {
                assert_eq!(is_checked, true);
                assert_eq!(label, "Remember me");
                assert_eq!(on_toggle, Some("toggle-checkbox".to_string()));
            }
            _ => panic!("Expected View::Checkbox"),
        }
    }

    #[test]
    fn test_unknown_kind() {
        let node = Node::new("unknown_widget");
        let result = convert_node(&node);

        match result {
            Err(ConversionError::UnknownKind { kind }) => {
                assert_eq!(kind, "unknown_widget");
            }
            _ => panic!("Expected UnknownKind error"),
        }
    }

    #[test]
    fn test_button_missing_onclick() {
        let node = Node::new("button").with_arg("Click");
        let result = convert_node(&node);

        match result {
            Err(ConversionError::MissingProp { kind, prop }) => {
                assert_eq!(kind, "button");
                assert_eq!(prop, "onclick");
            }
            _ => panic!("Expected MissingProp error"),
        }
    }

    #[test]
    fn test_nested_layout() {
        let node = Node::new("col")
            .with_prop("spacing", 10u32)
            .with_child(
                Node::new("row")
                    .with_prop("spacing", 5u32)
                    .with_child(Node::new("text").with_arg("A"))
                    .with_child(Node::new("text").with_arg("B"))
            )
            .with_child(Node::new("text").with_arg("C"));

        let view = convert_node(&node).unwrap();

        match view {
            View::Column { children, .. } => {
                assert_eq!(children.len(), 2);

                // First child should be a Row
                match &children[0] {
                    View::Row { children: row_children, .. } => {
                        assert_eq!(row_children.len(), 2);
                    }
                    _ => panic!("Expected first child to be View::Row"),
                }
            }
            _ => panic!("Expected View::Column"),
        }
    }
}

// ============================================================================
// 动态消息支持（Plan 011: Auto 动态解释器）
// ============================================================================

/// 将 Node 转换为使用动态消息的 View
///
/// 此函数与 `convert_node()` 类似，但使用 `DynamicMessage` 而非 `String` 作为消息类型，
/// 保留类型信息用于运行时事件路由。
///
/// # 消息类型
///
/// - **简单事件**: 使用 `DynamicMessage::String(event_name)`
/// - **类型化事件**: 使用 `DynamicMessage::Typed { widget_name, event_name, args }`
///
/// # 示例
///
/// ```ignore
/// use auto_ui::interpreter::DynamicMessage;
///
/// // Button with simple string message
/// let button = Node::new("button")
///     .with_arg("Click")
///     .with_prop("onclick", "button-clicked");
///
/// let view = convert_node_dynamic(&button, None)?;
/// assert!(matches!(view, View::Button { onclick: DynamicMessage::String(..), .. }));
///
/// // Button with typed message (metadata provided)
/// let view = convert_node_dynamic(&button, Some(("MyWidget", &symbol_table)))?;
/// assert!(matches!(view, View::Button { onclick: DynamicMessage::Typed { .. }, .. }));
/// ```
#[cfg(feature = "interpreter")]
pub fn convert_node_dynamic(
    node: &Node,
    metadata: Option<(&str, &auto_lang::Universe)>, // (widget_name, scope)
) -> ConversionResult<View<DynamicMessage>> {
    let kind = node.name.as_str();

    match kind {
        // 布局组件（递归处理子节点）
        "center" => convert_center_dynamic(node, metadata),
        "col" | "column" => convert_column_dynamic(node, metadata),
        "row" => convert_row_dynamic(node, metadata),
        "container" => convert_container_dynamic(node, metadata),
        "scrollable" => convert_scrollable_dynamic(node, metadata),

        // 元素组件
        "text" | "label" => convert_text_dynamic(node),
        "button" => convert_button_dynamic(node, metadata),
        "input" => convert_input_dynamic(node, metadata),
        "checkbox" => convert_checkbox_dynamic(node, metadata),
        "radio" => convert_radio_dynamic(node, metadata),
        "select" => convert_select_dynamic(node, metadata),
        "list" => convert_list_dynamic(node, metadata),
        "table" => convert_table_dynamic(node, metadata),

        // 未知类型
        _ => Err(ConversionError::UnknownKind {
            kind: kind.to_string(),
        }),
    }
}

// ============================================================================
// 动态消息转换辅助函数
// ============================================================================

/// 提取事件处理程序并转换为 DynamicMessage
#[cfg(feature = "interpreter")]
fn extract_event_handler(
    node: &Node,
    prop_name: &str,
    _metadata: Option<&str>,
) -> ConversionResult<DynamicMessage> {
    let event_str = extract_prop_str(node, prop_name)
        .ok_or_else(|| ConversionError::MissingProp {
            kind: node.name.to_string(),  // AutoStr → String
            prop: prop_name.to_string(),
        })?;

    // TODO: 如果提供了 metadata，尝试将其转换为类型化消息
    // 目前先使用简单的字符串消息
    Ok(DynamicMessage::String(event_str))
}

// 这里我们使用现有的转换函数，但包装返回类型
// 由于 Rust 的类型系统，我们需要为每个组件类型实现动态版本

#[cfg(feature = "interpreter")]
fn convert_center_dynamic(
    node: &Node,
    metadata: Option<(&str, &auto_lang::Universe)>,
) -> ConversionResult<View<DynamicMessage>> {
    let children = extract_children_dynamic(node, metadata)?;
    let style = extract_style(node)?;

    // Center 使用 Container 实现，设置 center_x 和 center_y
    let child = if children.is_empty() {
        View::Empty
    } else {
        // 合并所有子节点到一个容器中
        children.into_iter().fold(View::Empty, |acc, child| {
            if matches!(acc, View::Empty) {
                child
            } else {
                // 多个子节点，用 Column 包装
                View::Column {
                    children: vec![acc, child],
                    spacing: 0,
                    padding: 0,
                    style: None,
                }
            }
        })
    };

    Ok(View::Container {
        child: Box::new(child),
        padding: 0,
        width: None,
        height: None,
        center_x: true,
        center_y: true,
        style,
    })
}

#[cfg(feature = "interpreter")]
fn convert_column_dynamic(
    node: &Node,
    metadata: Option<(&str, &auto_lang::Universe)>,
) -> ConversionResult<View<DynamicMessage>> {
    let spacing = extract_prop_u32(node, "spacing").unwrap_or(0) as u16;
    let padding = extract_prop_u32(node, "padding").unwrap_or(0) as u16;
    let children = extract_children_dynamic(node, metadata)?;
    let style = extract_style(node)?;
    Ok(View::Column { spacing, padding, children, style })
}

#[cfg(feature = "interpreter")]
fn convert_row_dynamic(
    node: &Node,
    metadata: Option<(&str, &auto_lang::Universe)>,
) -> ConversionResult<View<DynamicMessage>> {
    let spacing = extract_prop_u32(node, "spacing").unwrap_or(0) as u16;
    let padding = extract_prop_u32(node, "padding").unwrap_or(0) as u16;
    let children = extract_children_dynamic(node, metadata)?;
    let style = extract_style(node)?;
    Ok(View::Row { spacing, padding, children, style })
}

#[cfg(feature = "interpreter")]
fn convert_container_dynamic(
    node: &Node,
    metadata: Option<(&str, &auto_lang::Universe)>,
) -> ConversionResult<View<DynamicMessage>> {
    let children = extract_children_dynamic(node, metadata)?;
    let padding = extract_prop_u32(node, "padding").unwrap_or(0) as u16;
    let width = extract_prop_u32(node, "width").map(|w| w as u16);
    let height = extract_prop_u32(node, "height").map(|h| h as u16);
    let center_x = extract_prop_bool(node, "center_x").unwrap_or(false);
    let center_y = extract_prop_bool(node, "center_y").unwrap_or(false);
    let style = extract_style(node)?;

    // 合并所有子节点
    let child = if children.is_empty() {
        View::Empty
    } else {
        children.into_iter().fold(View::Empty, |acc, child| {
            if matches!(acc, View::Empty) {
                child
            } else {
                View::Column {
                    children: vec![acc, child],
                    spacing: 0,
                    padding: 0,
                    style: None,
                }
            }
        })
    };

    Ok(View::Container {
        child: Box::new(child),
        padding,
        width,
        height,
        center_x,
        center_y,
        style,
    })
}

#[cfg(feature = "interpreter")]
fn convert_scrollable_dynamic(
    node: &Node,
    metadata: Option<(&str, &auto_lang::Universe)>,
) -> ConversionResult<View<DynamicMessage>> {
    let children = extract_children_dynamic(node, metadata)?;
    let width = extract_prop_u32(node, "width").map(|w| w as u16);
    let height = extract_prop_u32(node, "height").map(|h| h as u16);
    let style = extract_style(node)?;

    // 合并所有子节点
    let child = if children.is_empty() {
        View::Empty
    } else {
        children.into_iter().fold(View::Empty, |acc, child| {
            if matches!(acc, View::Empty) {
                child
            } else {
                View::Column {
                    children: vec![acc, child],
                    spacing: 0,
                    padding: 0,
                    style: None,
                }
            }
        })
    };

    Ok(View::Scrollable {
        child: Box::new(child),
        width,
        height,
        style,
    })
}

#[cfg(feature = "interpreter")]
fn convert_text_dynamic(node: &Node) -> ConversionResult<View<DynamicMessage>> {
    let content = extract_main_arg_str(node)
        .unwrap_or_else(|| String::from(""));  // 默认空字符串
    let style = extract_style(node)?;
    Ok(View::Text { content, style })
}

#[cfg(feature = "interpreter")]
fn convert_button_dynamic(
    node: &Node,
    metadata: Option<(&str, &auto_lang::Universe)>,
) -> ConversionResult<View<DynamicMessage>> {
    let label = extract_main_arg_str(node)
        .unwrap_or_else(|| String::from("Button"));
    let onclick = extract_event_handler(node, "onclick", metadata.map(|(name, _)|name))?;
    let style = extract_style(node)?;
    Ok(View::Button { label, onclick, style })
}

#[cfg(feature = "interpreter")]
fn convert_input_dynamic(
    node: &Node,
    metadata: Option<(&str, &auto_lang::Universe)>,
) -> ConversionResult<View<DynamicMessage>> {
    let placeholder = extract_main_arg_str(node).unwrap_or_default();
    let on_change = extract_event_handler(node, "onchange", metadata.map(|(name, _)|name))?;
    let value = extract_prop_str(node, "value").unwrap_or_default();
    let width = extract_prop_u32(node, "width");  // Option<u32>
    let width = width.map(|w| w as u16);  // Option<u16>
    let password = extract_prop_bool(node, "password").unwrap_or(false);
    let style = extract_style(node)?;
    Ok(View::Input { placeholder, value, on_change: Some(on_change), width, password, style })
}

#[cfg(feature = "interpreter")]
fn convert_checkbox_dynamic(
    node: &Node,
    _metadata: Option<(&str, &auto_lang::Universe)>,
) -> ConversionResult<View<DynamicMessage>> {
    let label = extract_main_arg_str(node).unwrap_or_default();
    let is_checked = extract_prop_bool(node, "checked").unwrap_or(false);
    let on_toggle = extract_prop_str(node, "ontoggle")
        .map(|s| Some(DynamicMessage::String(s)))
        .unwrap_or(None);
    let style = extract_style(node)?;
    Ok(View::Checkbox { is_checked, label, on_toggle, style })
}

#[cfg(feature = "interpreter")]
fn convert_radio_dynamic(
    node: &Node,
    _metadata: Option<(&str, &auto_lang::Universe)>,
) -> ConversionResult<View<DynamicMessage>> {
    let label = extract_main_arg_str(node).unwrap_or_default();
    let is_selected = extract_prop_bool(node, "checked").unwrap_or(false);
    let on_select = extract_prop_str(node, "onselect")
        .map(|s| Some(DynamicMessage::String(s)))
        .unwrap_or(None);
    let style = extract_style(node)?;
    Ok(View::Radio { label, is_selected, on_select, style })
}

#[cfg(feature = "interpreter")]
fn convert_select_dynamic(
    node: &Node,
    _metadata: Option<(&str, &auto_lang::Universe)>,
) -> ConversionResult<View<DynamicMessage>> {
    let options = extract_children_strings(node)?;
    let selected_index = extract_prop_usize(node, "selected");

    // 创建 SelectCallback - 将选项索引转换为事件字符串
    let on_select = extract_prop_str(node, "onselect")
        .map(|event_str| {
            SelectCallback::new(move |_index: usize, _selected: &str| {
                DynamicMessage::String(event_str.clone())
            })
        });

    let style = extract_style(node)?;
    Ok(View::Select { options, selected_index, on_select, style })
}

#[cfg(feature = "interpreter")]
fn convert_list_dynamic(
    node: &Node,
    metadata: Option<(&str, &auto_lang::Universe)>,
) -> ConversionResult<View<DynamicMessage>> {
    let items = extract_children_dynamic(node, metadata)?;
    let spacing = extract_prop_u32(node, "spacing").unwrap_or(0) as u16;
    let style = extract_style(node)?;
    Ok(View::List { items, spacing, style })
}

#[cfg(feature = "interpreter")]
fn convert_table_dynamic(
    node: &Node,
    metadata: Option<(&str, &auto_lang::Universe)>,
) -> ConversionResult<View<DynamicMessage>> {
    let children = extract_children_dynamic(node, metadata)?;

    // Table 需要分离 headers 和 rows
    // 假设第一个子节点是 header 行，其余是数据行
    let headers = if !children.is_empty() {
        // 尝试从第一个子节点提取 headers
        match &children[0] {
            View::Row { children: header_children, .. } => header_children.clone(),
            _ => vec![],
        }
    } else {
        vec![]
    };

    let rows = if children.len() > 1 {
        children[1..].iter().filter_map(|child| {
            if let View::Row { children: row_children, .. } = child {
                Some(row_children.clone())
            } else {
                None
            }
        }).collect()
    } else {
        vec![]
    };

    let spacing = extract_prop_u32(node, "spacing").unwrap_or(0) as u16;
    let col_spacing = extract_prop_u32(node, "col_spacing").unwrap_or(spacing as u32) as u16;
    let style = extract_style(node)?;
    Ok(View::Table { headers, rows, spacing, col_spacing, style })
}

/// 递归提取子节点并转换为动态消息 View
#[cfg(feature = "interpreter")]
fn extract_children_dynamic(
    node: &Node,
    metadata: Option<(&str, &auto_lang::Universe)>,
) -> ConversionResult<Vec<View<DynamicMessage>>> {
    let mut children = Vec::new();

    // 使用 kids_iter() 获取子节点
    for (_, kid) in node.kids_iter() {
        if let auto_val::Kid::Node(child_node) = kid {
            let view = convert_node_dynamic(&child_node, metadata)?;
            children.push(view);
        }
    }

    Ok(children)
}

// ============================================================================
// 动态消息转换的测试
// ============================================================================

#[cfg(test)]
#[cfg(feature = "interpreter")]
mod tests_dynamic {
    use super::*;
    use super::interpreter::DynamicMessage;

    #[test]
    fn test_convert_text_dynamic() {
        let node = Node::new("text").with_arg("Hello");
        let view = convert_node_dynamic(&node, None).unwrap();

        match view {
            View::Text { content, .. } => {
                assert_eq!(content, "Hello");
            }
            _ => panic!("Expected View::Text"),
        }
    }

    #[test]
    fn test_convert_button_dynamic() {
        let node = Node::new("button")
            .with_arg("Click")
            .with_prop("onclick", "button-clicked");

        let view = convert_node_dynamic(&node, None).unwrap();

        match view {
            View::Button { label, onclick, .. } => {
                assert_eq!(label, "Click");
                assert!(matches!(onclick, DynamicMessage::String(_)));
            }
            _ => panic!("Expected View::Button"),
        }
    }

    #[test]
    fn test_convert_column_dynamic() {
        let node = Node::new("col")
            .with_prop("spacing", 10u32)
            .with_child(Node::new("text").with_arg("A"))
            .with_child(Node::new("text").with_arg("B"));

        let view = convert_node_dynamic(&node, None).unwrap();

        match view {
            View::Column { spacing, children, .. } => {
                assert_eq!(spacing, 10);
                assert_eq!(children.len(), 2);
            }
            _ => panic!("Expected View::Column"),
        }
    }
}
