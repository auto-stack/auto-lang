//! # AuraViewBuilder - Converts AuraNode templates into View<DynamicMessage>
//!
//! This module traverses an AuraNode tree and builds a `View<DynamicMsg>` for
//! rendering, reading state values from `VmBridge` for state bindings like
//! `${.count}`.
//!
//! ## Architecture
//!
//! ```text
//! AuraNode tree (from AuraWidget.view_tree)
//!    |
//!    v
//! AuraViewBuilder
//!  - Resolves AuraExpr::StateRef via VmBridge::read_state()
//!  - Resolves ${.field} interpolations in text content
//!  - Maps AuraNode::Element tags to View variants
//!  - Creates DynamicMessage for event handlers
//!    |
//!    v
//! View<DynamicMessage> (ready for rendering)
//! ```
//!
//! ## Plan 205 Phase 2
//!
//! Phase 2 implements core widget conversion:
//! - text, button, column, row (core layout)
//! - State binding resolution from VmBridge
//! - String interpolation for `${.field}` patterns
//! - Event handler → DynamicMessage mapping

use std::collections::HashMap;

use auto_val::Value;

use crate::aura::{AuraExpr, AuraNode, AuraPropValue, AuraTextContent, AuraEvent};

/// Loop variable bindings: variable name → current Value.
/// Passed through the conversion call chain to resolve `FieldAccess`
/// expressions like `note.title` where `note` is a loop variable.
type Bindings = HashMap<String, Value>;
use crate::ui::interpreter::DynamicMessage;
use crate::ui::vm_bridge::VmBridge;
use crate::ui::debug_id_map::DebugIdMap;
use crate::ui::view::View;
use crate::ui::style::{Style, StyleClass, SizeValue};

// ============================================================================
// AuraViewBuilder
// ============================================================================

/// Builds a `View<DynamicMessage>` from an AuraNode template, reading state
/// from a `VmBridge`.
///
/// Each `AuraViewBuilder` is scoped to a single widget. The `widget_name` is
/// embedded in `DynamicMessage::Typed` variants so the event router can
/// dispatch messages back to the correct handler.
///
/// # Example
///
/// ```ignore
/// let bridge = VmBridge::new(&widget)?;
/// let builder = AuraViewBuilder::new(&bridge, "Counter");
/// let view = builder.build(&widget.view_tree);
/// ```
pub struct AuraViewBuilder<'a> {
    /// Reference to the VmBridge that holds widget state
    bridge: &'a VmBridge,

    /// Widget name, used in DynamicMessage routing
    widget_name: String,
}

impl<'a> AuraViewBuilder<'a> {
    /// Create a new builder bound to a VmBridge instance.
    ///
    /// # Arguments
    ///
    /// * `bridge` - VmBridge holding the widget's state
    /// * `widget_name` - Name of the widget (for message routing)
    pub fn new(bridge: &'a VmBridge, widget_name: &str) -> Self {
        Self {
            bridge,
            widget_name: widget_name.to_string(),
        }
    }

    /// Build a `View<DynamicMessage>` from an AuraNode template.
    ///
    /// Recursively traverses the AuraNode tree, converting each node into the
    /// corresponding View variant. State references are resolved from the
    /// VmBridge at build time.
    pub fn build(&self, node: &AuraNode) -> View<DynamicMessage> {
        self.convert_node_with(node, &Bindings::new())
    }

    /// Build a `View<DynamicMessage>` with DebugIdMap sideband mapping (Plan 274).
    ///
    /// Returns `(View, DebugIdMap)` where the DebugIdMap records which AuraNodeId
    /// produced each View node, keyed by the View tree path (`Vec<usize>`).
    pub fn build_with_debug(&self, node: &AuraNode) -> (View<DynamicMessage>, DebugIdMap) {
        let mut id_map = DebugIdMap::default();
        let mut path = Vec::new();
        let view = self.convert_node_tracked_with(node, &mut path, &mut id_map, &Bindings::new());
        (view, id_map)
    }

    // ========================================================================
    // Internal conversion
    // ========================================================================

    /// Dispatch an AuraNode to the appropriate converter (no bindings).
    fn convert_node(&self, node: &AuraNode) -> View<DynamicMessage> {
        self.convert_node_with(node, &Bindings::new())
    }

    /// Dispatch an AuraNode to the appropriate converter with loop variable bindings.
    fn convert_node_with(&self, node: &AuraNode, bindings: &Bindings) -> View<DynamicMessage> {
        match node {
            AuraNode::Element { tag, props, events, children, .. } => {
                self.convert_element(tag, props, events, children, bindings)
            }
            AuraNode::Text(text_content) => {
                self.convert_text_with(text_content, bindings)
            }
            AuraNode::ForLoop { var, index, iterable, body, .. } => {
                // Strip leading dot from iterable name (e.g., ".notes" → "notes")
                let state_name = iterable.strip_prefix('.').unwrap_or(iterable);
                // Read the iterable array from VmBridge state
                let array = match self.bridge.read_state(state_name) {
                    Ok(Value::Array(arr)) => arr,
                    Ok(other) => {
                        return View::Empty;
                    }
                    Err(_) => {
                        return View::Empty;
                    }
                };

                let children: Vec<View<DynamicMessage>> = array.iter().enumerate()
                    .filter_map(|(i, item)| {
                        let mut loop_bindings = bindings.clone();
                        // Bind loop variable (e.g., "note" → Value::Obj{title, body, time})
                        loop_bindings.insert(var.clone(), item.clone());
                        // Bind index variable if present (e.g., "i" → Value::Int(0))
                        if let Some(idx_var) = index {
                            loop_bindings.insert(idx_var.clone(), Value::Int(i as i32));
                        }
                        // Convert body nodes with the loop bindings active
                        let views: Vec<View<DynamicMessage>> = body.iter()
                            .map(|n| self.convert_node_with(n, &loop_bindings))
                            .collect();
                        if views.is_empty() {
                            None
                        } else if views.len() == 1 {
                            Some(views.into_iter().next().unwrap())
                        } else {
                            Some(View::Column {
                                children: views,
                                spacing: 0,
                                padding: 0,
                                style: None,
                            })
                        }
                    })
                    .collect();

                View::Column {
                    children,
                    spacing: 0,
                    padding: 0,
                    style: None,
                }
            }
            AuraNode::Conditional { condition, then_body, else_body, .. } => {
                let is_true = self.eval_condition_with(condition, bindings);
                let empty = Vec::new();
                let body = if is_true {
                    then_body
                } else {
                    else_body.as_ref().unwrap_or(&empty)
                };
                let children: Vec<View<DynamicMessage>> = body
                    .iter()
                    .map(|n| self.convert_node_with(n, bindings))
                    .collect();
                if children.is_empty() {
                    View::Empty
                } else if children.len() == 1 {
                    children.into_iter().next().unwrap()
                } else {
                    View::Column {
                        children,
                        spacing: 0,
                        padding: 0,
                        style: None,
                    }
                }
            }
            AuraNode::Component { name, .. } => {
                View::Text {
                    content: format!("<{} />", name),
                    style: None,
                }
            }
            AuraNode::Outlet => {
                View::Text {
                    content: "<outlet />".to_string(),
                    style: None,
                }
            }
            AuraNode::Link { text, children, .. } => {
                if !children.is_empty() {
                    let views: Vec<View<DynamicMessage>> = children
                        .iter()
                        .map(|n| self.convert_node_with(n, bindings))
                        .collect();
                    View::Column {
                        children: views,
                        spacing: 0,
                        padding: 0,
                        style: None,
                    }
                } else if !text.is_empty() {
                    View::Text {
                        content: text.clone(),
                        style: None,
                    }
                } else {
                    View::Empty
                }
            }
        }
    }

    /// Tracked version of convert_node that records View path → AuraNodeId mappings.
    fn convert_node_tracked(
        &self,
        node: &AuraNode,
        path: &mut Vec<usize>,
        id_map: &mut DebugIdMap,
    ) -> View<DynamicMessage> {
        self.convert_node_tracked_with(node, path, id_map, &Bindings::new())
    }

    /// Tracked version with loop variable bindings.
    fn convert_node_tracked_with(
        &self,
        node: &AuraNode,
        path: &mut Vec<usize>,
        id_map: &mut DebugIdMap,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        // Record this node's debug_id at the current path
        let node_debug_id = match node {
            AuraNode::Element { debug_id, .. } => *debug_id,
            AuraNode::ForLoop { debug_id, .. } => *debug_id,
            AuraNode::Conditional { debug_id, .. } => *debug_id,
            AuraNode::Component { debug_id, .. } => *debug_id,
            AuraNode::Link { debug_id, .. } => *debug_id,
            _ => None,
        };
        if let Some(aura_id) = node_debug_id {
            id_map.record(path, aura_id);
        }

        match node {
            AuraNode::Element { tag, props, events, children, .. } => {
                self.convert_element_tracked_with(tag, props, events, children, path, id_map, bindings)
            }
            AuraNode::Text(text_content) => {
                self.convert_text_with(text_content, bindings)
            }
            AuraNode::ForLoop { var, index, iterable, body, .. } => {
                // Strip leading dot from iterable name (e.g., ".notes" → "notes")
                let state_name = iterable.strip_prefix('.').unwrap_or(iterable);
                let array = match self.bridge.read_state(state_name) {
                    Ok(Value::Array(arr)) => arr,
                    _ => return View::Empty,
                };
                let child_views: Vec<View<DynamicMessage>> = array.iter().enumerate()
                    .filter_map(|(i, item)| {
                        let mut loop_bindings = bindings.clone();
                        loop_bindings.insert(var.clone(), item.clone());
                        if let Some(idx_var) = index {
                            loop_bindings.insert(idx_var.clone(), Value::Int(i as i32));
                        }
                        // Include iteration index in path to ensure unique debug IDs
                        // across loop iterations (without this, all iterations produce
                        // identical paths, causing duplicate iced widget IDs)
                        let views: Vec<View<DynamicMessage>> = body.iter()
                            .enumerate()
                            .filter_map(|(bi, n)| {
                                path.push(i);   // iteration index
                                path.push(bi);  // body node index
                                let v = self.convert_node_tracked_with(n, path, id_map, &loop_bindings);
                                path.pop();
                                path.pop();
                                Some(v)
                            })
                            .collect();
                        if views.is_empty() { None }
                        else if views.len() == 1 { Some(views.into_iter().next().unwrap()) }
                        else { Some(View::Column { children: views, spacing: 0, padding: 0, style: None }) }
                    })
                    .collect();
                View::Column {
                    children: child_views,
                    spacing: 0,
                    padding: 0,
                    style: None,
                }
            }
            AuraNode::Conditional { condition, then_body, else_body, .. } => {
                let is_true = self.eval_condition_with(condition, bindings);
                let empty = Vec::new();
                let body = if is_true {
                    then_body
                } else {
                    else_body.as_ref().unwrap_or(&empty)
                };
                let child_views: Vec<View<DynamicMessage>> = body
                    .iter()
                    .enumerate()
                    .map(|(i, n)| {
                        path.push(i);
                        let v = self.convert_node_tracked_with(n, path, id_map, bindings);
                        path.pop();
                        v
                    })
                    .collect();
                if child_views.is_empty() {
                    View::Empty
                } else if child_views.len() == 1 {
                    child_views.into_iter().next().unwrap()
                } else {
                    View::Column {
                        children: child_views,
                        spacing: 0,
                        padding: 0,
                        style: None,
                    }
                }
            }
            AuraNode::Component { name, .. } => {
                View::Text {
                    content: format!("<{} />", name),
                    style: None,
                }
            }
            AuraNode::Outlet => {
                View::Text {
                    content: "<outlet />".to_string(),
                    style: None,
                }
            }
            AuraNode::Link { text, children, .. } => {
                if !children.is_empty() {
                    let views: Vec<View<DynamicMessage>> = children
                        .iter()
                        .enumerate()
                        .map(|(i, n)| {
                            path.push(i);
                            let v = self.convert_node_tracked_with(n, path, id_map, bindings);
                            path.pop();
                            v
                        })
                        .collect();
                    View::Column {
                        children: views,
                        spacing: 0,
                        padding: 0,
                        style: None,
                    }
                } else if !text.is_empty() {
                    View::Text {
                        content: text.clone(),
                        style: None,
                    }
                } else {
                    View::Empty
                }
            }
        }
    }

    /// Tracked version of convert_element that passes path/id_map to child conversions.
    fn convert_element_tracked(
        &self,
        tag: &str,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        children: &[AuraNode],
        path: &mut Vec<usize>,
        id_map: &mut DebugIdMap,
    ) -> View<DynamicMessage> {
        self.convert_element_tracked_with(tag, props, events, children, path, id_map, &Bindings::new())
    }

    /// Tracked convert_element with bindings support.
    /// For tracked mode, we delegate to the same _with methods used by untracked mode
    /// since the debug ID tracking is handled at the node level in convert_node_tracked_with.
    fn convert_element_tracked_with(
        &self,
        tag: &str,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        children: &[AuraNode],
        path: &mut Vec<usize>,
        id_map: &mut DebugIdMap,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        // Delegate to the untracked-with methods — they already handle bindings.
        // Debug ID tracking is done at the AuraNode level in convert_node_tracked_with.
        self.convert_element(tag, props, events, children, bindings)
    }

    /// Tracked convert_column
    fn convert_column_tracked(
        &self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        path: &mut Vec<usize>,
        id_map: &mut DebugIdMap,
    ) -> View<DynamicMessage> {
        let spacing = self.extract_u16(props, "spacing").unwrap_or(0);
        let padding = self.extract_u16(props, "padding").unwrap_or(0);
        let style = self.extract_style(props);

        let child_views: Vec<View<DynamicMessage>> = children
            .iter()
            .enumerate()
            .map(|(i, n)| {
                path.push(i);
                let v = self.convert_node_tracked(n, path, id_map);
                path.pop();
                v
            })
            .collect();

        let mut builder = View::<DynamicMessage>::col()
            .spacing(spacing)
            .padding(padding);

        if let Some(s) = style {
            builder = builder.with_style(s);
        }

        for child in child_views {
            builder = builder.child(child);
        }

        builder.build()
    }

    /// Tracked convert_row
    fn convert_row_tracked(
        &self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        path: &mut Vec<usize>,
        id_map: &mut DebugIdMap,
    ) -> View<DynamicMessage> {
        let spacing = self.extract_u16(props, "spacing").unwrap_or(0);
        let padding = self.extract_u16(props, "padding").unwrap_or(0);
        let style = self.extract_style(props);

        let child_views: Vec<View<DynamicMessage>> = children
            .iter()
            .enumerate()
            .map(|(i, n)| {
                path.push(i);
                let v = self.convert_node_tracked(n, path, id_map);
                path.pop();
                v
            })
            .collect();

        let mut builder = View::<DynamicMessage>::row()
            .spacing(spacing)
            .padding(padding);

        if let Some(s) = style {
            builder = builder.with_style(s);
        }

        for child in child_views {
            builder = builder.child(child);
        }

        builder.build()
    }

    /// Tracked convert_container
    fn convert_container_tracked(
        &self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        path: &mut Vec<usize>,
        id_map: &mut DebugIdMap,
    ) -> View<DynamicMessage> {
        let padding = self.extract_u16(props, "padding").unwrap_or(0);
        let width = self.extract_u16(props, "width");
        let height = self.extract_u16(props, "height");
        let style = self.extract_style(props);

        let child_view = if children.is_empty() {
            View::Empty
        } else if children.len() == 1 {
            path.push(0);
            let v = self.convert_node_tracked(&children[0], path, id_map);
            path.pop();
            v
        } else {
            let views: Vec<View<DynamicMessage>> = children
                .iter()
                .enumerate()
                .map(|(i, n)| {
                    path.push(i);
                    let v = self.convert_node_tracked(n, path, id_map);
                    path.pop();
                    v
                })
                .collect();
            View::Column {
                children: views,
                spacing: 0,
                padding: 0,
                style: None,
            }
        };

        let mut builder = View::container(child_view).padding(padding);
        if let Some(w) = width {
            builder = builder.width(w);
        }
        if let Some(h) = height {
            builder = builder.height(h);
        }
        if let Some(s) = style {
            builder = builder.with_style(s);
        }

        builder.build()
    }

    /// Tracked convert_center
    fn convert_center_tracked(
        &self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        path: &mut Vec<usize>,
        id_map: &mut DebugIdMap,
    ) -> View<DynamicMessage> {
        let style = self.extract_style(props);

        let child_view = if children.is_empty() {
            View::Empty
        } else if children.len() == 1 {
            path.push(0);
            let v = self.convert_node_tracked(&children[0], path, id_map);
            path.pop();
            v
        } else {
            let views: Vec<View<DynamicMessage>> = children
                .iter()
                .enumerate()
                .map(|(i, n)| {
                    path.push(i);
                    let v = self.convert_node_tracked(n, path, id_map);
                    path.pop();
                    v
                })
                .collect();
            View::Column {
                children: views,
                spacing: 0,
                padding: 0,
                style: None,
            }
        };

        let full_style = match style {
            Some(s) => s.add(StyleClass::Width(SizeValue::Full)).add(StyleClass::Height(SizeValue::Full)),
            None => Style::default().add(StyleClass::Width(SizeValue::Full)).add(StyleClass::Height(SizeValue::Full)),
        };
        let mut builder = View::container(child_view).center_x().center_y();
        builder = builder.with_style(full_style);

        builder.build()
    }

    /// Convert an AuraNode::Element to a View variant based on the tag name.
    fn convert_element(
        &self,
        tag: &str,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        children: &[AuraNode],
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        match tag {
            // Core layout widgets
            "col" | "column" => self.convert_column(props, children, bindings),
            "row" => self.convert_row(props, children, bindings),

            // Core element widgets
            "text" | "label" | "h1" | "h2" | "h3" | "p" | "span" => {
                self.convert_text_element(tag, props, children, bindings)
            }
            "button" | "btn" => self.convert_button(props, events, bindings),

            // Layout wrappers
            "center" => self.convert_center(props, children, bindings),

            // Input widgets
            "input" => self.convert_input(props, events, bindings),
            "textarea" => self.convert_textarea(props, events, bindings),
            "checkbox" | "check" => self.convert_checkbox(props, events),
            "container" | "div" => self.convert_container(props, children, bindings),

            // Image placeholder
            "img" | "image" => self.convert_image(props),

            // Utility widgets
            "progress" => self.convert_progress(props),
            "spacer" => self.convert_spacer(props),
            "divider" | "hr" => self.convert_divider(props),
            "avatar" => self.convert_avatar(props),

            // Fallback: wrap children in a column
            _ => {
                let views: Vec<View<DynamicMessage>> = children
                    .iter()
                    .map(|n| self.convert_node_with(n, bindings))
                    .collect();
                if views.is_empty() {
                    View::Empty
                } else if views.len() == 1 {
                    views.into_iter().next().unwrap()
                } else {
                    View::Column {
                        children: views,
                        spacing: 0,
                        padding: 0,
                        style: None,
                    }
                }
            }
        }
    }

    // ========================================================================
    // Layout converters
    // ========================================================================

    /// Convert a column element.
    fn convert_column(
        &self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let spacing = self.extract_u16(props, "spacing").unwrap_or(0);
        let padding = self.extract_u16(props, "padding").unwrap_or(0);
        let style = self.extract_style(props);

        let child_views: Vec<View<DynamicMessage>> = children
            .iter()
            .map(|n| self.convert_node_with(n, bindings))
            .collect();

        let mut builder = View::<DynamicMessage>::col()
            .spacing(spacing)
            .padding(padding);

        if let Some(s) = style {
            builder = builder.with_style(s);
        }

        for child in child_views {
            builder = builder.child(child);
        }

        builder.build()
    }

    /// Convert a row element.
    fn convert_row(
        &self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let spacing = self.extract_u16(props, "spacing").unwrap_or(0);
        let padding = self.extract_u16(props, "padding").unwrap_or(0);
        let style = self.extract_style(props);

        let child_views: Vec<View<DynamicMessage>> = children
            .iter()
            .map(|n| self.convert_node_with(n, bindings))
            .collect();

        let mut builder = View::<DynamicMessage>::row()
            .spacing(spacing)
            .padding(padding);

        if let Some(s) = style {
            builder = builder.with_style(s);
        }

        for child in child_views {
            builder = builder.child(child);
        }

        builder.build()
    }

    /// Convert a container element.
    fn convert_container(
        &self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let padding = self.extract_u16(props, "padding").unwrap_or(0);
        let width = self.extract_u16(props, "width");
        let height = self.extract_u16(props, "height");
        let style = self.extract_style(props);

        let child_view = if children.is_empty() {
            View::Empty
        } else if children.len() == 1 {
            self.convert_node_with(&children[0], bindings)
        } else {
            let views: Vec<View<DynamicMessage>> = children
                .iter()
                .map(|n| self.convert_node_with(n, bindings))
                .collect();
            View::Column {
                children: views,
                spacing: 0,
                padding: 0,
                style: None,
            }
        };

        let mut builder = View::container(child_view).padding(padding);
        if let Some(w) = width {
            builder = builder.width(w);
        }
        if let Some(h) = height {
            builder = builder.height(h);
        }
        if let Some(s) = style {
            builder = builder.with_style(s);
        }

        builder.build()
    }

    /// Convert a center element: wraps child in a centered container.
    fn convert_center(
        &self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let style = self.extract_style(props);

        let child_view = if children.is_empty() {
            View::Empty
        } else if children.len() == 1 {
            self.convert_node_with(&children[0], bindings)
        } else {
            let views: Vec<View<DynamicMessage>> = children
                .iter()
                .map(|n| self.convert_node_with(n, bindings))
                .collect();
            View::Column {
                children: views,
                spacing: 0,
                padding: 0,
                style: None,
            }
        };

        // center defaults to w-full h-full so it fills its parent and centers content
        let full_style = match style {
            Some(s) => s.add(StyleClass::Width(SizeValue::Full)).add(StyleClass::Height(SizeValue::Full)),
            None => Style::default().add(StyleClass::Width(SizeValue::Full)).add(StyleClass::Height(SizeValue::Full)),
        };
        let mut builder = View::container(child_view).center_x().center_y();
        builder = builder.with_style(full_style);

        builder.build()
    }

    /// Convert an image element: create View::Image for actual rendering.
    fn convert_image(
        &self,
        props: &HashMap<String, AuraPropValue>,
    ) -> View<DynamicMessage> {
        let style = self.extract_style(props);
        let src = self.extract_string(props, "src").unwrap_or_default();
        View::Image { src, style }
    }

    /// Convert a progress element: shows a progress bar from 0.0 to 1.0.
    fn convert_progress(
        &self,
        props: &HashMap<String, AuraPropValue>,
    ) -> View<DynamicMessage> {
        let style = self.extract_style(props);

        // Extract value and max, compute progress ratio
        let value = self.extract_f64(props, "value").unwrap_or(0.0);
        let max = self.extract_f64(props, "max").unwrap_or(100.0);
        let progress = if max > 0.0 {
            (value / max).clamp(0.0, 1.0)
        } else {
            0.0
        };

        View::ProgressBar {
            progress: progress as f32,
            style,
        }
    }

    /// Convert a spacer element: fills remaining space in a flex layout.
    fn convert_spacer(
        &self,
        props: &HashMap<String, AuraPropValue>,
    ) -> View<DynamicMessage> {
        let style = self.extract_style(props);

        let child = View::Empty;
        let mut builder = View::container(child);
        if let Some(s) = style {
            builder = builder.with_style(s);
        } else {
            builder = builder.with_style(
                Style::parse("w-full").unwrap()
            );
        }
        builder.build()
    }

    /// Convert a divider element: renders a horizontal line separator.
    fn convert_divider(
        &self,
        _props: &HashMap<String, AuraPropValue>,
    ) -> View<DynamicMessage> {
        let child = View::Empty;
        let mut builder = View::container(child);
        builder = builder.with_style(
            Style::parse("w-full h-1 bg-gray-200").unwrap()
        );
        builder.build()
    }

    /// Convert an avatar element: colored circle placeholder.
    fn convert_avatar(
        &self,
        props: &HashMap<String, AuraPropValue>,
    ) -> View<DynamicMessage> {
        let style = self.extract_style(props);

        let child = View::Text {
            content: "".to_string(),
            style: None,
        };
        let mut builder = View::container(child);
        builder = builder.center_x().center_y();
        if let Some(s) = style {
            builder = builder.with_style(s);
        } else {
            builder = builder.with_style(
                Style::parse("bg-gray-300 rounded-full").unwrap()
            );
        }
        builder.build()
    }

    // ========================================================================
    // Element converters
    // ========================================================================

    /// Convert a text element.
    ///
    /// Text content can come from:
    /// - A `text` or `content` prop
    /// - A child text node
    /// - The tag's main argument
    fn convert_text_element(
        &self,
        tag: &str,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let content = self.extract_string_with(props, "text", bindings)
            .or_else(|| self.extract_string_with(props, "content", bindings))
            .or_else(|| self.extract_string_with(props, "label", bindings))
            .unwrap_or_else(|| {
                // Try to get content from child text nodes
                children.iter()
                    .filter_map(|c| match c {
                        AuraNode::Text(AuraTextContent::Literal(s)) => Some(s.clone()),
                        AuraNode::Text(AuraTextContent::Interpolated { template, bindings: tpl_bindings }) => {
                            Some(self.resolve_interpolation_with(template, tpl_bindings, bindings))
                        }
                        _ => None,
                    })
                    .collect::<Vec<String>>()
                    .join("")
            });

        let mut style = self.extract_style(props);

        // Apply default heading styles if no explicit style was provided
        if style.is_none() {
            style = match tag {
                "h1" => Style::parse("text-4xl font-bold").ok(),
                "h2" => Style::parse("text-3xl font-bold").ok(),
                "h3" => Style::parse("text-xl font-semibold").ok(),
                _ => None,
            };
        }

        // Map heading tags to styled text
        let styled_content = match tag {
            "h1" => content,
            "h2" => content,
            "h3" => content,
            _ => content,
        };

        View::Text {
            content: styled_content,
            style,
        }
    }

    /// Convert a button element.
    fn convert_button(
        &self,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let label = self.extract_string_with(props, "text", bindings)
            .or_else(|| self.extract_string_with(props, "label", bindings))
            .unwrap_or_else(|| "Button".to_string());

        let style = self.extract_style(props);

        // Resolve the onclick event handler to a DynamicMessage
        let onclick = events.get("onclick")
            .or_else(|| events.get("click"))
            .map(|event| self.event_to_message_with(event, bindings))
            .unwrap_or_else(|| DynamicMessage::String("click".to_string()));

        View::Button {
            label,
            onclick,
            style,
        }
    }

    /// Convert an input element.
    fn convert_input(
        &self,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let placeholder = self.extract_string_with(props, "placeholder", bindings)
            .or_else(|| self.extract_string_with(props, "text", bindings))
            .unwrap_or_default();

        // Resolve value from state if it's a StateRef
        let value = self.extract_string_with(props, "value", bindings).unwrap_or_default();

        let style = self.extract_style(props);
        let width = self.extract_u16(props, "width");
        let password = self.extract_bool(props, "password").unwrap_or(false);

        let on_change = events.get("onchange")
            .or_else(|| events.get("change"))
            .or_else(|| events.get("oninput"))
            .or_else(|| events.get("input"))
            .map(|event| self.event_to_message(&event.handler));

        let mut builder = View::<DynamicMessage>::input(placeholder).value(value);
        if password {
            builder = builder.password();
        }
        if let Some(msg) = on_change {
            builder = builder.on_change(msg);
        }
        if let Some(w) = width {
            builder = builder.width(w);
        }
        if let Some(s) = style {
            builder = builder.with_style(s);
        }

        builder.build()
    }

    /// Convert a textarea element.
    fn convert_textarea(
        &self,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let placeholder = self.extract_string_with(props, "placeholder", bindings)
            .unwrap_or_default();

        let value = self.extract_string_with(props, "value", bindings).unwrap_or_default();

        let style = self.extract_style(props);
        let height = self.extract_u16(props, "height");

        let on_change = events.get("onchange")
            .or_else(|| events.get("change"))
            .or_else(|| events.get("oninput"))
            .or_else(|| events.get("input"))
            .map(|event| self.event_to_message(&event.handler));

        let mut builder = View::<DynamicMessage>::textarea(placeholder).value(value);
        if let Some(msg) = on_change {
            builder = builder.on_change(msg);
        }
        if let Some(h) = height {
            builder = builder.height(h);
        }
        if let Some(s) = style {
            builder = builder.with_style(s);
        }

        builder.build()
    }

    /// Convert a checkbox element.
    fn convert_checkbox(
        &self,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
    ) -> View<DynamicMessage> {
        let label = self.extract_string(props, "text")
            .or_else(|| self.extract_string(props, "label"))
            .unwrap_or_default();

        // Resolve checked from state ref or literal
        let is_checked = props.get("checked")
            .or_else(|| props.get("is_checked"))
            .map(|v| match v {
                AuraPropValue::Expr(AuraExpr::Bool(b)) => Some(*b),
                AuraPropValue::Expr(AuraExpr::StateRef(name)) => {
                    self.bridge.read_state(name)
                        .map(|val| val.as_bool())
                        .ok()
                }
                _ => None,
            })
            .flatten()
            .unwrap_or(false);

        let on_toggle = events.get("onclick")
            .or_else(|| events.get("change"))
            .or_else(|| events.get("onchange"))
            .map(|event| self.event_to_message(&event.handler));

        let style = self.extract_style(props);

        let mut view = View::checkbox(is_checked, label);
        if let Some(msg) = on_toggle {
            view = view.on_toggle(msg);
        }
        if let Some(s) = style {
            if let View::Checkbox { style: ref mut st, .. } = view {
                *st = Some(s);
            }
        }
        view
    }

    // ========================================================================
    // Text content conversion
    // ========================================================================

    /// Convert an AuraTextContent to a string, resolving interpolations.
    fn convert_text_with(&self, content: &AuraTextContent, bindings: &Bindings) -> View<DynamicMessage> {
        let resolved = match content {
            AuraTextContent::Literal(s) => s.clone(),
            AuraTextContent::Interpolated { template, bindings: tpl_bindings } => {
                self.resolve_interpolation_with(template, tpl_bindings, bindings)
            }
        };

        View::Text {
            content: resolved,
            style: None,
        }
    }

    // ========================================================================
    // State resolution
    // ========================================================================

    /// Resolve a string interpolation template containing `${.field}` references.
    fn resolve_interpolation(&self, template: &str, bindings: &[String]) -> String {
        self.resolve_interpolation_with(template, bindings, &Bindings::new())
    }

    /// Resolve interpolation with loop variable bindings support.
    fn resolve_interpolation_with(&self, template: &str, tpl_bindings: &[String], loop_bindings: &Bindings) -> String {
        let mut result = template.to_string();

        for field_name in tpl_bindings {
            let pattern = format!("${{{}}}", format!(".{}", field_name));
            let value_str = self.read_state_as_string_with(field_name, loop_bindings);
            result = result.replace(&pattern, &value_str);
        }

        result
    }

    /// Read a state field value as a display string.
    fn read_state_as_string(&self, field_name: &str) -> String {
        self.read_state_as_string_with(field_name, &Bindings::new())
    }

    /// Read a state field value as a display string, checking loop bindings first.
    fn read_state_as_string_with(&self, field_name: &str, bindings: &Bindings) -> String {
        // Check loop bindings first (e.g., "note" in `for note in .notes`)
        if let Some(val) = bindings.get(field_name) {
            return value_to_display_string(val);
        }
        match self.bridge.read_state(field_name) {
            Ok(value) => value_to_display_string(&value),
            Err(_) => format!("${{{}}}", field_name),
        }
    }

    /// Resolve an AuraExpr to a display string (no bindings).
    fn resolve_expr_to_string(&self, expr: &AuraExpr) -> String {
        self.resolve_expr_to_string_with(expr, &Bindings::new())
    }

    /// Resolve an AuraExpr to a display string with loop variable bindings.
    fn resolve_expr_to_string_with(&self, expr: &AuraExpr, bindings: &Bindings) -> String {
        match expr {
            AuraExpr::Literal(s) => self.resolve_literal_interpolation_with(s, bindings),
            AuraExpr::Int(i) => i.to_string(),
            AuraExpr::Float(f) => f.to_string(),
            AuraExpr::Bool(b) => b.to_string(),
            AuraExpr::StateRef(name) => self.read_state_as_string_with(name, bindings),
            AuraExpr::FieldAccess { object, field } => {
                let obj_val = self.resolve_expr_to_value(object, bindings);
                match obj_val {
                    Some(Value::Obj(map)) => {
                        map.get(field.as_str())
                            .map(|v| value_to_display_string(&v))
                            .unwrap_or_default()
                    }
                    _ => String::new(),
                }
            }
            _ => String::new(),
        }
    }

    /// Resolve an AuraExpr to a Value, checking loop bindings and VmBridge state.
    fn resolve_expr_to_value(&self, expr: &AuraExpr, bindings: &Bindings) -> Option<Value> {
        match expr {
            AuraExpr::StateRef(name) => {
                bindings.get(name).cloned()
                    .or_else(|| self.bridge.read_state(name).ok())
            }
            AuraExpr::FieldAccess { object, field } => {
                let obj = self.resolve_expr_to_value(object, bindings)?;
                match obj {
                    Value::Obj(map) => map.get(field.as_str()),
                    _ => None,
                }
            }
            AuraExpr::Int(i) => Some(Value::Int(*i as i32)),
            AuraExpr::Float(f) => Some(Value::Double(*f)),
            AuraExpr::Bool(b) => Some(Value::Bool(*b)),
            AuraExpr::Literal(s) => Some(Value::Str(s.as_str().into())),
            _ => None,
        }
    }

    /// Evaluate a condition string against current state (no bindings).
    fn eval_condition(&self, condition: &str) -> bool {
        self.eval_condition_with(condition, &Bindings::new())
    }

    /// Evaluate a condition string against current state with loop variable bindings.
    ///
    /// Supports patterns like:
    /// - `.running == "true"` — state ref compared to string literal
    /// - `.count > 0` — state ref compared to number
    /// - `.active_id == i` — state ref compared to loop index variable
    /// - `.flag` — bare state ref (truthy check)
    fn eval_condition_with(&self, condition: &str, bindings: &Bindings) -> bool {
        let cond = condition.trim();

        // Strip leading dot for state ref
        let (lhs, op, rhs) = if let Some(rest) = cond.strip_prefix('.') {
            // Find operator
            if let Some(pos) = rest.find(" == ") {
                (&rest[..pos], "==", rest[pos + 4..].trim())
            } else if let Some(pos) = rest.find(" != ") {
                (&rest[..pos], "!=", rest[pos + 4..].trim())
            } else if let Some(pos) = rest.find(" > ") {
                (&rest[..pos], ">", rest[pos + 3..].trim())
            } else if let Some(pos) = rest.find(" < ") {
                (&rest[..pos], "<", rest[pos + 3..].trim())
            } else if let Some(pos) = rest.find(" >= ") {
                (&rest[..pos], ">=", rest[pos + 4..].trim())
            } else if let Some(pos) = rest.find(" <= ") {
                (&rest[..pos], "<=", rest[pos + 4..].trim())
            } else {
                // Bare state ref — truthy check
                return self.bridge.read_state(rest)
                    .map(|v| v.as_bool())
                    .unwrap_or(false);
            }
        } else {
            // No leading dot — treat as bare bool state ref
            return self.bridge.read_state(cond)
                .map(|v| v.as_bool())
                .unwrap_or(false);
        };

        // Read state value for lhs
        let lhs_val = match self.bridge.read_state(lhs) {
            Ok(v) => value_to_display_string(&v),
            Err(_) => return false,
        };

        // Resolve rhs: check loop bindings first, then try as literal
        let rhs_val = if let Some(val) = bindings.get(rhs) {
            value_to_display_string(val)
        } else {
            rhs.trim_matches('"').to_string()
        };

        // Compare
        match op {
            "==" => lhs_val == rhs_val,
            "!=" => lhs_val != rhs_val,
            ">" | "<" | ">=" | "<=" => {
                let lhs_num: f64 = match lhs_val.parse() {
                    Ok(n) => n,
                    Err(_) => return false,
                };
                let rhs_num: f64 = match rhs_val.parse() {
                    Ok(n) => n,
                    Err(_) => return false,
                };
                match op {
                    ">" => lhs_num > rhs_num,
                    "<" => lhs_num < rhs_num,
                    ">=" => lhs_num >= rhs_num,
                    "<=" => lhs_num <= rhs_num,
                    _ => false,
                }
            }
            _ => false,
        }
    }

    /// Resolve `${.field}` interpolation patterns in a literal string.
    ///
    /// F-strings like `f"Count: ${.count}"` are extracted as `AuraExpr::Literal`
    /// with the template preserved. This method scans for `${.name}` patterns
    /// and substitutes current state values.
    /// Resolve `${.field}` interpolation patterns in a literal string (no bindings).
    fn resolve_literal_interpolation(&self, s: &str) -> String {
        self.resolve_literal_interpolation_with(s, &Bindings::new())
    }

    /// Resolve `${.field}` interpolation patterns with loop bindings support.
    fn resolve_literal_interpolation_with(&self, s: &str, bindings: &Bindings) -> String {
        if !s.contains("${.") {
            return s.to_string();
        }

        let mut result = s.to_string();
        // Scan for ${.fieldname} patterns and resolve from state
        let bytes = s.as_bytes();
        let len = bytes.len();
        let mut i = 0;
        let mut replacements: Vec<(String, String)> = Vec::new();

        while i + 4 < len {
            if &bytes[i..i+3] == b"${." {
                // Found start of interpolation: ${.
                let start = i;
                let mut end = i + 3;
                while end < len && bytes[end] != b'}' {
                    end += 1;
                }
                if end < len && bytes[end] == b'}' {
                    let field_name = &s[start + 3..end];
                    // Validate field name is alphanumeric/underscore
                    if !field_name.is_empty() && field_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                        let full_pattern = s[start..end + 1].to_string();
                        let value = self.read_state_as_string_with(field_name, bindings);
                        replacements.push((full_pattern, value));
                    }
                }
                i = end + 1;
            } else {
                i += 1;
            }
        }

        for (pattern, value) in replacements {
            result = result.replace(&pattern, &value);
        }
        result
    }

    // ========================================================================
    // Event helpers
    // ========================================================================

    /// Convert an event handler pattern to a DynamicMessage (no bindings).
    ///
    /// Patterns like ".Inc", "Msg::Inc", or "Inc" are normalized to a
    /// `DynamicMessage::Typed` with the widget name and extracted handler name.
    fn event_to_message(&self, handler: &str) -> DynamicMessage {
        self.event_to_message_impl(handler, &Bindings::new())
    }

    /// Convert an AuraEvent to a DynamicMessage with loop variable bindings.
    ///
    /// Resolves event parameters from bindings (e.g., loop variable `i`)
    /// and encodes integer parameters into the event_name using `name:idx`
    /// format (e.g., `"SelectNote:2"`), leveraging the existing indexed event
    /// dispatch in the iced renderer.
    fn event_to_message_with(&self, event: &AuraEvent, bindings: &Bindings) -> DynamicMessage {
        let handler_name = extract_handler_name(&event.handler);
        // Resolve first parameter from bindings (typically a loop index variable)
        let final_name = if let Some(param_name) = event.params.first() {
            if let Some(Value::Int(idx)) = bindings.get(param_name) {
                format!("{}:{}", handler_name, idx)
            } else {
                handler_name.to_string()
            }
        } else {
            handler_name.to_string()
        };
        DynamicMessage::Typed {
            widget_name: self.widget_name.clone(),
            event_name: final_name,
            args: vec![],
        }
    }

    /// Internal: convert handler string to DynamicMessage (used by event_to_message).
    fn event_to_message_impl(&self, handler: &str, _bindings: &Bindings) -> DynamicMessage {
        let handler_name = extract_handler_name(handler);
        DynamicMessage::Typed {
            widget_name: self.widget_name.clone(),
            event_name: handler_name.to_string(),
            args: vec![],
        }
    }

    // ========================================================================
    // Property extraction helpers
    // ========================================================================

    /// Extract a string property from AuraNode props (no bindings).
    fn extract_string(
        &self,
        props: &HashMap<String, AuraPropValue>,
        key: &str,
    ) -> Option<String> {
        self.extract_string_with(props, key, &Bindings::new())
    }

    /// Extract a string property with loop variable bindings support.
    fn extract_string_with(
        &self,
        props: &HashMap<String, AuraPropValue>,
        key: &str,
        bindings: &Bindings,
    ) -> Option<String> {
        let prop = props.get(key)?;
        match prop {
            AuraPropValue::Expr(expr) => {
                let result = self.resolve_expr_to_string_with(expr, bindings);
                Some(result)
            }
            AuraPropValue::StyleBinding(_) => None,
        }
    }

    /// Extract a u16 property from AuraNode props.
    fn extract_u16(
        &self,
        props: &HashMap<String, AuraPropValue>,
        key: &str,
    ) -> Option<u16> {
        match props.get(key)? {
            AuraPropValue::Expr(expr) => match expr {
                AuraExpr::Int(i) => {
                    if *i >= 0 && *i <= u16::MAX as i64 {
                        Some(*i as u16)
                    } else {
                        None
                    }
                }
                AuraExpr::Float(f) => {
                    if *f >= 0.0 && *f <= u16::MAX as f64 {
                        Some(*f as u16)
                    } else {
                        None
                    }
                }
                _ => None,
            },
            AuraPropValue::StyleBinding(_) => None,
        }
    }

    /// Extract a bool property from AuraNode props.
    fn extract_bool(
        &self,
        props: &HashMap<String, AuraPropValue>,
        key: &str,
    ) -> Option<bool> {
        match props.get(key)? {
            AuraPropValue::Expr(AuraExpr::Bool(b)) => Some(*b),
            _ => None,
        }
    }

    /// Extract a float property from AuraNode props (supports StateRef resolution).
    fn extract_f64(
        &self,
        props: &HashMap<String, AuraPropValue>,
        key: &str,
    ) -> Option<f64> {
        match props.get(key)? {
            AuraPropValue::Expr(expr) => match expr {
                AuraExpr::Int(i) => Some(*i as f64),
                AuraExpr::Float(f) => Some(*f),
                AuraExpr::StateRef(name) => {
                    match self.bridge.read_state(name) {
                        Ok(value) => match value {
                            Value::Int(i) => Some(i as f64),
                            Value::Float(f) => Some(f as f64),
                            Value::Double(f) => Some(f),
                            _ => None,
                        },
                        Err(_) => None,
                    }
                }
                _ => None,
            },
            AuraPropValue::StyleBinding(_) => None,
        }
    }

    /// Extract a style property from AuraNode props.
    ///
    /// Looks for a "class" or "style" prop and parses it into a Style object.
    fn extract_style(
        &self,
        props: &HashMap<String, AuraPropValue>,
    ) -> Option<Style> {
        let style_str = self.extract_string(props, "class")
            .or_else(|| self.extract_string(props, "style"))?;

        Style::parse(&style_str).ok()
    }
}

// ============================================================================
// Free helper functions
// ============================================================================

/// Extract a clean handler name from an event pattern.
///
/// Patterns:
/// - ".Inc"       -> "Inc"
/// - "Msg::Inc"   -> "Inc"
/// - "Inc"        -> "Inc"
fn extract_handler_name(pattern: &str) -> &str {
    let name = pattern.trim_start_matches('.');
    if let Some(pos) = name.rfind("::") {
        &name[pos + 2..]
    } else {
        name
    }
}

/// Convert a Value to a display string suitable for UI rendering.
fn value_to_display_string(value: &Value) -> String {
    match value {
        Value::Int(i) => i.to_string(),
        Value::Float(f) => format!("{}", f),
        Value::Double(f) => format!("{}", f),
        Value::Bool(b) => b.to_string(),
        Value::Str(s) => s.to_string(),
        Value::String(s) => s.as_str().to_string(),
        Value::Nil => String::new(),
        _ => value.to_string(),
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aura::{AuraEvent, AuraStateDef, AuraWidget};
    use crate::ast::Type;
    use std::collections::HashMap;

    /// Helper: create a minimal AuraWidget for testing.
    fn make_test_widget(name: &str, state_vars: Vec<AuraStateDef>) -> AuraWidget {
        AuraWidget {
            name: name.to_string(),
            state_vars,
            computed: vec![],
            messages: vec![],
            view_tree: AuraNode::element("col"),
            handlers: HashMap::new(),
            props: vec![],
            routes: None,
            lifecycle: vec![],
            tick_interval: None,
            handler_params: HashMap::new(),
            span_map: HashMap::new(),
            key_bindings: HashMap::new(),
        }
    }

    #[test]
    fn test_build_text_literal() {
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Test");

        let node = AuraNode::text("Hello World");
        let view = builder.build(&node);

        match view {
            View::Text { content, .. } => {
                assert_eq!(content, "Hello World");
            }
            _ => panic!("Expected View::Text"),
        }
    }

    #[test]
    fn test_build_text_with_state_ref() {
        let widget = make_test_widget("Counter", vec![
            AuraStateDef {
                name: "count".to_string(),
                type_info: Type::Int,
                initial: AuraExpr::Int(42),
                decorators: vec![],
            },
        ]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Counter");

        let node = AuraNode::Text(AuraTextContent::Interpolated {
            template: "Count: ${.count}".to_string(),
            bindings: vec!["count".to_string()],
        });
        let view = builder.build(&node);

        match view {
            View::Text { content, .. } => {
                assert_eq!(content, "Count: 42");
            }
            _ => panic!("Expected View::Text"),
        }
    }

    #[test]
    fn test_build_column_with_children() {
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Test");

        let node = AuraNode::Element {
            tag: "col".to_string(),
            props: HashMap::from([
                ("spacing".to_string(), AuraPropValue::Expr(AuraExpr::Int(10))),
                ("padding".to_string(), AuraPropValue::Expr(AuraExpr::Int(5))),
            ]),
            events: HashMap::new(),
            children: vec![
                AuraNode::text("Child 1"),
                AuraNode::text("Child 2"),
            ],
            span: None,
            debug_id: None,
        };
        let view = builder.build(&node);

        match view {
            View::Column { spacing, padding, children, .. } => {
                assert_eq!(spacing, 10);
                assert_eq!(padding, 5);
                assert_eq!(children.len(), 2);
            }
            _ => panic!("Expected View::Column"),
        }
    }

    #[test]
    fn test_build_row() {
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Test");

        let node = AuraNode::Element {
            tag: "row".to_string(),
            props: HashMap::from([
                ("spacing".to_string(), AuraPropValue::Expr(AuraExpr::Int(8))),
            ]),
            events: HashMap::new(),
            children: vec![
                AuraNode::text("A"),
                AuraNode::text("B"),
            ],
            span: None,
            debug_id: None,
        };
        let view = builder.build(&node);

        match view {
            View::Row { spacing, children, .. } => {
                assert_eq!(spacing, 8);
                assert_eq!(children.len(), 2);
            }
            _ => panic!("Expected View::Row"),
        }
    }

    #[test]
    fn test_build_button_with_event() {
        let widget = make_test_widget("Counter", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Counter");

        let node = AuraNode::Element {
            span: None,
            debug_id: None,
            tag: "button".to_string(),
            props: HashMap::from([
                ("text".to_string(), AuraPropValue::Expr(AuraExpr::Literal("Increment".to_string()))),
            ]),
            events: HashMap::from([
                ("onclick".to_string(), AuraEvent {
                    handler: ".Inc".to_string(),
                    params: vec![],
                }),
            ]),
            children: vec![],
        };
        let view = builder.build(&node);

        match view {
            View::Button { label, onclick, .. } => {
                assert_eq!(label, "Increment");
                match onclick {
                    DynamicMessage::Typed { widget_name, event_name, args } => {
                        assert_eq!(widget_name, "Counter");
                        assert_eq!(event_name, "Inc");
                        assert!(args.is_empty());
                    }
                    _ => panic!("Expected DynamicMessage::Typed"),
                }
            }
            _ => panic!("Expected View::Button"),
        }
    }

    #[test]
    fn test_build_nested_layout() {
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Test");

        let node = AuraNode::Element {
            tag: "col".to_string(),
            props: HashMap::new(),
            events: HashMap::new(),
            span: None,
            debug_id: None,
            children: vec![
                AuraNode::Element {
                    tag: "row".to_string(),
                    props: HashMap::new(),
                    events: HashMap::new(),
                    span: None,
                    debug_id: None,
                    children: vec![
                        AuraNode::text("Left"),
                        AuraNode::text("Right"),
                    ],
                },
                AuraNode::text("Bottom"),
            ],
        };
        let view = builder.build(&node);

        match view {
            View::Column { children, .. } => {
                assert_eq!(children.len(), 2);
                match &children[0] {
                    View::Row { children: row_children, .. } => {
                        assert_eq!(row_children.len(), 2);
                    }
                    _ => panic!("Expected View::Row as first child"),
                }
            }
            _ => panic!("Expected View::Column"),
        }
    }

    #[test]
    fn test_build_unknown_tag_fallback() {
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Test");

        let node = AuraNode::Element {
            tag: "custom_widget".to_string(),
            props: HashMap::new(),
            events: HashMap::new(),
            children: vec![
                AuraNode::text("Content"),
            ],
            span: None,
            debug_id: None,
        };
        let view = builder.build(&node);

        // Should render the child directly as fallback
        match view {
            View::Text { content, .. } => {
                assert_eq!(content, "Content");
            }
            _ => panic!("Expected View::Text (single child fallback)"),
        }
    }

    #[test]
    fn test_extract_handler_name() {
        assert_eq!(extract_handler_name(".Inc"), "Inc");
        assert_eq!(extract_handler_name("Msg::Inc"), "Inc");
        assert_eq!(extract_handler_name("Inc"), "Inc");
        assert_eq!(extract_handler_name(".AddItem"), "AddItem");
    }

    #[test]
    fn test_state_binding_in_text_element() {
        let widget = make_test_widget("Counter", vec![
            AuraStateDef {
                name: "count".to_string(),
                type_info: Type::Int,
                initial: AuraExpr::Int(7),
                decorators: vec![],
            },
        ]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Counter");

        let node = AuraNode::Element {
            tag: "text".to_string(),
            props: HashMap::from([
                ("text".to_string(), AuraPropValue::Expr(AuraExpr::StateRef("count".to_string()))),
            ]),
            events: HashMap::new(),
            span: None,
            debug_id: None,
            children: vec![],
        };
        let view = builder.build(&node);

        match view {
            View::Text { content, .. } => {
                assert_eq!(content, "7");
            }
            _ => panic!("Expected View::Text with state-resolved value"),
        }
    }

    #[test]
    fn test_button_msg_variant_handler() {
        let widget = make_test_widget("Counter", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Counter");

        let node = AuraNode::Element {
            tag: "button".to_string(),
            props: HashMap::from([
                ("label".to_string(), AuraPropValue::Expr(AuraExpr::Literal("Reset".to_string()))),
            ]),
            events: HashMap::from([
                ("onclick".to_string(), AuraEvent {
                    handler: "Msg::Reset".to_string(),
                    params: vec![],
                }),
            ]),
            span: None,
            debug_id: None,
            children: vec![],
        };
        let view = builder.build(&node);

        match view {
            View::Button { label, onclick, .. } => {
                assert_eq!(label, "Reset");
                match onclick {
                    DynamicMessage::Typed { event_name, .. } => {
                        assert_eq!(event_name, "Reset");
                    }
                    _ => panic!("Expected DynamicMessage::Typed"),
                }
            }
            _ => panic!("Expected View::Button"),
        }
    }

    #[test]
    fn test_value_to_display_string() {
        assert_eq!(value_to_display_string(&Value::Int(42)), "42");
        assert_eq!(value_to_display_string(&Value::Bool(true)), "true");
        assert_eq!(value_to_display_string(&Value::str("hello")), "hello");
        assert_eq!(value_to_display_string(&Value::Nil), "");
    }
}
