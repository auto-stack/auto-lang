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
use crate::ui::debug::{BuildProbe, ForIter};
use crate::ui::view::View;
use crate::ui::style::{Style, StyleClass, SizeValue};

// ============================================================================
// Tracked conversion side-channels — Plan 307 Task 9
// ============================================================================
//
// The *tracked* path (`build_with_debug`) threads three mutable accumulators
// down through every converter: `path` (the AuraNode-structural descent path,
// each segment = child index in its parent's `children` slice), `id_map`
// (path → AuraNodeId), and `probe` (per-path AutoUI data).
//
// `path` is the AuraNode-structural descent path. The probe stores it as
// `Vec<u16>` (cast from `usize`). For plain col/row containers this matches
// the View-structural path used by `view_to_vtree_with_paths`; for ForLoop
// output the two schemes may diverge — see the OPEN ISSUE in the Task 9
// report (reconciled in Task 12).
//
// The **untracked** `build()` path never reaches these methods, so its
// behaviour is byte-for-byte identical to before Task 9.

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

    /// Optional widget registry for child widget rendering
    widget_registry: Option<&'a crate::ui::widget_registry::WidgetRegistry>,
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
            widget_registry: None,
        }
    }

    /// Create a builder with widget registry for child widget support.
    pub fn with_registry(
        bridge: &'a VmBridge,
        widget_name: &str,
        registry: &'a crate::ui::widget_registry::WidgetRegistry,
    ) -> Self {
        Self {
            bridge,
            widget_name: widget_name.to_string(),
            widget_registry: Some(registry),
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

    /// Build a `View<DynamicMessage>` with debug sideband data (Plan 274 / 307 Task 9).
    ///
    /// Returns `(View, DebugIdMap, BuildProbe)` where:
    /// - the `DebugIdMap` records which AuraNodeId produced each View node, keyed
    ///   by the AuraNode-structural path (`Vec<usize>`);
    /// - the `BuildProbe` records AutoUI-specific per-path data (state bindings,
    ///   for-context, events) captured while walking the node tree. Task 9 fills
    ///   text-interpolation state bindings only.
    ///
    /// The probe is **enabled** (records normally). This preserves the
    /// historical behaviour relied on by Task 9-11 tests. For F12-off / MCP
    /// zero-overhead capture bypass (Plan 307 Task 18), use
    /// [`build_with_debug_gated`] with `capture_probe = false`.
    pub fn build_with_debug(&self, node: &AuraNode) -> (View<DynamicMessage>, DebugIdMap, BuildProbe) {
        self.build_with_debug_gated(node, true)
    }

    /// Gated variant of [`build_with_debug`] (Plan 307 Task 18 perf gate).
    ///
    /// When `capture_probe` is false, the returned `BuildProbe` is constructed
    /// disabled via [`BuildProbe::new_disabled`], so every `record_*` call
    /// during the node walk is a no-op — giving near-zero overhead when the
    /// debug layer is inactive (F12 off) or for the MCP sync path (which never
    /// needs probe data). The `DebugIdMap` is still populated (it is cheap and
    /// required by MCP), but no probe work happens.
    pub fn build_with_debug_gated(
        &self,
        node: &AuraNode,
        capture_probe: bool,
    ) -> (View<DynamicMessage>, DebugIdMap, BuildProbe) {
        let mut id_map = DebugIdMap::default();
        let mut probe = if capture_probe {
            BuildProbe::new()
        } else {
            BuildProbe::new_disabled()
        };
        let mut path = Vec::new();
        let view = self.convert_node_tracked_ctx(node, &mut path, &mut id_map, &mut probe, &Bindings::new());
        (view, id_map, probe)
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
                        // Try read_state_as_vec for Value::Int(array_id) refs
                        match self.bridge.read_state_as_vec(state_name) {
                            Ok(vec) => {
                                // Re-wrap as Array for consistent iteration
                                let owned: Vec<Value> = vec;
                                let arr = auto_val::Array::from(owned);
                                // Need to re-iterate — fall through to filter_map below
                                let children: Vec<View<DynamicMessage>> = arr.iter().enumerate()
                                    .filter_map(|(i, item)| {
                                        // Apply search filter if 'search' state exists and is non-empty
                                        if !self.matches_search(item) { return None; }
                                        let mut loop_bindings = bindings.clone();
                                        loop_bindings.insert(var.clone(), self.bridge.materialize_obj_ref(item));
                                        if let Some(idx_var) = index {
                                            loop_bindings.insert(idx_var.clone(), Value::Int(i as i32));
                                        }
                                        let views: Vec<View<DynamicMessage>> = body.iter()
                                            .map(|n| self.convert_node_with(n, &loop_bindings))
                                            .collect();
                                        if views.is_empty() { None }
                                        else if views.len() == 1 { Some(views.into_iter().next().unwrap()) }
                                        else { Some(View::Column { children: views, spacing: 0, padding: 0, style: None }) }
                                    })
                                    .collect();
                                return View::Column { children, spacing: 0, padding: 0, style: None };
                            }
                            Err(_) => return View::Empty,
                        }
                    }
                    Err(_) => {
                        return View::Empty;
                    }
                };

                let children: Vec<View<DynamicMessage>> = array.iter().enumerate()
                    .filter_map(|(i, item)| {
                        // Apply search filter if 'search' state exists and is non-empty
                        if !self.matches_search(item) { return None; }
                        let mut loop_bindings = bindings.clone();
                        // Bind loop variable (e.g., "note" → Value::Obj{title, body, time})
                        loop_bindings.insert(var.clone(), self.bridge.materialize_obj_ref(item));
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
            AuraNode::Component { name, props, events, .. } => {
                // Look up child widget in registry
                if let Some(registry) = self.widget_registry {
                    if let Some(child_widget) = registry.get(name) {
                        let prop_values: HashMap<String, AuraPropValue> = props.iter()
                            .map(|(k, v)| (k.clone(), AuraPropValue::Expr(v.clone())))
                            .collect();
                        return self.render_child_widget(child_widget, &prop_values, events, bindings);
                    }
                }
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

    /// Tracked node conversion: deep recursion that records per-path data into
    /// both `DebugIdMap` (AuraNodeId) and `BuildProbe` (state bindings). This is
    /// the Plan 307 Task 9 deep-threaded path; the untracked `build()` path
    /// never reaches here, so its behaviour is unchanged.
    fn convert_node_tracked_ctx(
        &self,
        node: &AuraNode,
        path: &mut Vec<usize>,
        id_map: &mut DebugIdMap,
        probe: &mut BuildProbe,
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
                self.convert_element_tracked_ctx(tag, props, events, children, path, id_map, probe, bindings)
            }
            AuraNode::Text(text_content) => {
                self.convert_text_tracked_ctx(text_content, path, probe, bindings)
            }
            AuraNode::ForLoop { var, index, iterable, body, .. } => {
                // Strip leading dot from iterable name (e.g., ".notes" → "notes")
                let state_name = iterable.strip_prefix('.').unwrap_or(iterable);
                // Read the iterable. `read_state_as_vec` handles BOTH an inline
                // `Value::Array` and a `Value::Int(array_id)` heap-array reference
                // (the latter is how `var x = []; x.push(...)` arrays are stored —
                // e.g. 016-calendar's `.days`). A bare `read_state` + `Value::Array`
                // match misses the heap-id form and silently renders an empty loop.
                let array: Vec<Value> = match self.bridge.read_state_as_vec(state_name) {
                    Ok(v) => v,
                    _ => return View::Empty,
                };
                let child_views: Vec<View<DynamicMessage>> = array.iter().enumerate()
                    .filter_map(|(i, item)| {
                        // Apply search filter if 'search' state exists and is non-empty
                        if !self.matches_search(item) { return None; }
                        let mut loop_bindings = bindings.clone();
                        loop_bindings.insert(var.clone(), self.bridge.materialize_obj_ref(item));
                        if let Some(idx_var) = index {
                            loop_bindings.insert(idx_var.clone(), Value::Int(i as i32));
                        }
                        // Include iteration index in path to ensure unique debug IDs
                        // across loop iterations (without this, all iterations produce
                        // identical paths, causing duplicate iced widget IDs).
                        //
                        // Plan 309 Phase 1 (Fix A): only push the body-node index
                        // when the body has >1 node. When the body is a single
                        // node, the iteration yields that node *directly* (no
                        // wrapping Column — see the `views.len() == 1` unwrap
                        // below), so the node's flattened VTree path is `[p, i]`.
                        // Unconditionally pushing `bi` (=0) recorded it at
                        // `[p, i, 0]`, diverging from the VTree path and leaving
                        // the inspector's AutoUI / source data empty for loop
                        // bodies. The multi-node case still wraps each iteration
                        // in a Column, so `bi` must be pushed there to match the
                        // extra VTree level. `record_for` is computed after the
                        // push, so it auto-reflects the corrected depth.
                        let body_len = body.len();
                        let views: Vec<View<DynamicMessage>> = body.iter()
                            .enumerate()
                            .filter_map(|(bi, n)| {
                                path.push(i);   // iteration index
                                if body_len > 1 { path.push(bi); }  // body node index (multi-node only)
                                // Record this iteration's context against the
                                // body node's path (Plan 307 Task 10). `index`
                                // is the 0-based iteration counter `i`, NOT the
                                // loop's optional index-variable name. Keep
                                // `iterable_repr` in its original ".notes" form.
                                let for_path: Vec<u16> =
                                    path.iter().map(|&x| x as u16).collect();
                                probe.record_for(&for_path, ForIter {
                                    var: var.clone(),
                                    index: Some(i),
                                    value_repr: value_to_display_string(item),
                                    iterable_repr: iterable.clone(),
                                });
                                let v = self.convert_node_tracked_ctx(n, path, id_map, probe, &loop_bindings);
                                if body_len > 1 { path.pop(); }
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
                // Plan 309 Phase 1 (Fix A, companion to the ForLoop fix):
                // mirror the loop's behaviour — only push the child index when
                // there is >1 child, because the single-child `child_views.len()
                // == 1` unwrap below yields that child directly (no wrapping
                // Column), so the node's flattened VTree path drops the index
                // level. Pushing it unconditionally diverged from the VTree path
                // for single-child conditionals, leaving inspector data empty.
                let body_len = body.len();
                let child_views: Vec<View<DynamicMessage>> = body
                    .iter()
                    .enumerate()
                    .map(|(i, n)| {
                        if body_len > 1 { path.push(i); }
                        let v = self.convert_node_tracked_ctx(n, path, id_map, probe, bindings);
                        if body_len > 1 { path.pop(); }
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
            AuraNode::Component { name, props, events, .. } => {
                // Look up child widget in registry
                if let Some(registry) = self.widget_registry {
                    if let Some(child_widget) = registry.get(name) {
                        let prop_values: HashMap<String, AuraPropValue> = props.iter()
                            .map(|(k, v)| (k.clone(), AuraPropValue::Expr(v.clone())))
                            .collect();
                        return self.render_child_widget(child_widget, &prop_values, events, bindings);
                    }
                }
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
                            let v = self.convert_node_tracked_ctx(n, path, id_map, probe, bindings);
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

    /// Tracked convert_element: dispatches by tag and recurses children with
    /// path/probe tracking (deep), instead of delegating to the untracked
    /// converters. Layout/prop extraction mirrors the untracked converters
    /// exactly; only the child recursion differs (it carries the side-channels).
    fn convert_element_tracked_ctx(
        &self,
        tag: &str,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        children: &[AuraNode],
        path: &mut Vec<usize>,
        id_map: &mut DebugIdMap,
        probe: &mut BuildProbe,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        // Record event handler bindings for this element, at this node's own
        // path (set by the caller's `path.push(child_index)`). Runs before the
        // `match tag` dispatch so it is unconditional — every element with
        // events (button/input/textarea/checkbox, etc.) is captured regardless
        // of whether its tag falls into the tracked or untracked converter arm.
        if !events.is_empty() {
            let ev_path: Vec<u16> = path.iter().map(|&x| x as u16).collect();
            for (event_name, ev) in events.iter() {
                probe.record_event(&ev_path, event_name, &ev.handler);
            }
        }

        // Plan 309 Phase 2b: record the declared class string (the `class`
        // prop, falling back to the inline `style=` prop) against this node's
        // path, so the inspector's Computed tab can show the original tokens.
        // `extract_string` resolves interpolations (static classes verbatim);
        // `Style` parsing (used by `extract_style`) would discard
        // whitespace/order and the `style=` fallback, so the probe keeps the
        // fuller string. `record_raw_class` is a no-op for `None`, so
        // class-less elements never gain a spurious probe entry.
        let raw_class = self
            .extract_string(props, "class")
            .or_else(|| self.extract_string(props, "style"));
        if raw_class.is_some() {
            let rc_path: Vec<u16> = path.iter().map(|&x| x as u16).collect();
            probe.record_raw_class(&rc_path, raw_class);
        }

        match tag {
            // Core layout widgets — recurse children with path tracking.
            "col" | "column" => self.convert_column_tracked_ctx(props, children, path, id_map, probe, bindings),
            "row" => self.convert_row_tracked_ctx(props, children, path, id_map, probe, bindings),
            "grid" => self.convert_grid_tracked_ctx(props, children, path, id_map, probe, bindings),
            "center" => self.convert_center_tracked_ctx(props, children, path, id_map, probe, bindings),
            "container" | "div" => self.convert_container_tracked_ctx(props, children, path, id_map, probe, bindings),

            // Text-bearing elements. The text/interpolation state bindings are
            // captured at this node's current path (the text element's path),
            // which is what the inspector wants.
            "text" | "label" | "h1" | "h2" | "h3" | "p" | "span" => {
                self.convert_text_element_tracked_ctx(tag, props, children, path, probe, bindings)
            }

            // Leaf/atom widgets with no AuraNode children — fall back to the
            // untracked converter. They have no nested text to probe (Task 9
            // scope is text interpolation only).
            "button" | "btn" => self.convert_button(props, events, bindings),
            "input" => self.convert_input(props, events, bindings),
            "textarea" => self.convert_textarea(props, events, bindings),
            "checkbox" | "check" => self.convert_checkbox(props, events, bindings),
            "img" | "image" => self.convert_image(props),
            "progress" => self.convert_progress(props),
            "spacer" => self.convert_spacer(props),
            "divider" | "hr" => self.convert_divider(props),
            "avatar" => self.convert_avatar(props),

            // Child widget lookup or fallback.
            _ => {
                if let Some(registry) = self.widget_registry {
                    if let Some(child_widget) = registry.get(tag) {
                        return self.render_child_widget(child_widget, props, events, bindings);
                    }
                }
                // Fallback: recurse children with path tracking, filtering Empty.
                let views: Vec<View<DynamicMessage>> = children
                    .iter()
                    .enumerate()
                    .filter_map(|(i, n)| {
                        path.push(i);
                        let v = self.convert_node_tracked_ctx(n, path, id_map, probe, bindings);
                        path.pop();
                        if matches!(v, View::Empty) { None } else { Some(v) }
                    })
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

    /// Tracked convert_column — mirrors `convert_column` but recurses via
    /// `convert_node_tracked_ctx` so each child gets its own path + probe data.
    fn convert_column_tracked_ctx(
        &self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        path: &mut Vec<usize>,
        id_map: &mut DebugIdMap,
        probe: &mut BuildProbe,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let spacing = self.extract_u16(props, "spacing").unwrap_or(0);
        let padding = self.extract_u16(props, "padding").unwrap_or(0);
        let style = self.extract_style(props);

        let child_views: Vec<View<DynamicMessage>> = children
            .iter()
            .enumerate()
            .map(|(i, n)| {
                path.push(i);
                let v = self.convert_node_tracked_ctx(n, path, id_map, probe, bindings);
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

    /// Tracked convert_grid — mirrors `convert_grid` but recurses via
    /// `convert_node_tracked_ctx` so each grid-item cell captures its own
    /// path + probe data (text bindings, raw class). Probe paths follow the
    /// SOURCE structure (flat grid-item indices under the grid); the row
    /// grouping is a rendering detail and does not perturb probe indexing.
    fn convert_grid_tracked_ctx(
        &self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        path: &mut Vec<usize>,
        id_map: &mut DebugIdMap,
        probe: &mut BuildProbe,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let cols = self
            .extract_u16(props, "cols")
            .or_else(|| self.extract_u16(props, "columns"))
            .map(|c| (c as usize).max(1))
            .unwrap_or(1);
        let gap = self.extract_u16(props, "gap").unwrap_or(0);
        let style = self.extract_style(props);

        // Flatten `for`-loop children into individual cells, assigning each cell
        // a sequential `cell_idx` path so build-time paths match the render-time
        // paths `render_dynamic_view`'s Grid arm visits (Plan 323). A bare `for`
        // inside a grid must yield one cell per iteration, not a wrapping Column.
        let mut cells: Vec<View<DynamicMessage>> = Vec::new();
        let mut cell_idx: usize = 0;
        for n in children.iter() {
            match n {
                AuraNode::ForLoop { var, index, iterable, body, .. } => {
                    let state_name = iterable.strip_prefix('.').unwrap_or(iterable);
                    // Use read_state_as_vec so heap-array refs (Value::Int(array_id),
                    // the form `var x = []; x.push(...)` produces — e.g. .days) are
                    // iterated, not just inline Value::Array. Otherwise the grid's
                    // `for cell in .days` renders empty even though state is populated.
                    let array: Vec<Value> = match self.bridge.read_state_as_vec(state_name) {
                        Ok(v) => v,
                        _ => continue,
                    };
                    let body_len = body.len();
                    for (i, item) in array.iter().enumerate() {
                        if !self.matches_search(item) {
                            continue;
                        }
                        let mut loop_bindings = bindings.clone();
                        loop_bindings.insert(var.clone(), self.bridge.materialize_obj_ref(item));
                        if let Some(idx_var) = index {
                            loop_bindings.insert(idx_var.clone(), Value::Int(i as i32));
                        }
                        let views: Vec<View<DynamicMessage>> = body
                            .iter()
                            .enumerate()
                            .filter_map(|(bi, bn)| {
                                path.push(cell_idx);
                                if body_len > 1 {
                                    path.push(bi);
                                }
                                let for_path: Vec<u16> =
                                    path.iter().map(|&x| x as u16).collect();
                                probe.record_for(&for_path, ForIter {
                                    var: var.clone(),
                                    index: Some(i),
                                    value_repr: value_to_display_string(item),
                                    iterable_repr: iterable.clone(),
                                });
                                let v = self.convert_node_tracked_ctx(
                                    bn, path, id_map, probe, &loop_bindings,
                                );
                                if body_len > 1 {
                                    path.pop();
                                }
                                path.pop();
                                Some(v)
                            })
                            .collect();
                        let cell = if views.is_empty() {
                            continue;
                        } else if views.len() == 1 {
                            views.into_iter().next().unwrap()
                        } else {
                            View::Column { children: views, spacing: 0, padding: 0, style: None }
                        };
                        if matches!(cell, View::Empty) {
                            continue;
                        }
                        cells.push(cell);
                        cell_idx += 1;
                    }
                }
                other => {
                    path.push(cell_idx);
                    let v = self.convert_node_tracked_ctx(other, path, id_map, probe, bindings);
                    path.pop();
                    if !matches!(v, View::Empty) {
                        cells.push(v);
                        cell_idx += 1;
                    }
                }
            }
        }

        if cells.is_empty() {
            return View::Empty;
        }

        // Decomposition (final-row padding + w-full rows + col-of-rows) moved
        // to the shared generic `build_grid` (Plan 319). Per-cell tracked
        // recursion is preserved, so cell i is still recorded at path [..i].
        // Bonus: build-time path [..i] now matches the render-time path that
        // `render_dynamic_view`'s Grid arm visits — previously the col-of-rows
        // split caused a build/render path mismatch for grid descendants.
        View::Grid { cols, gap, cells, style }
    }

    /// Tracked convert_row — mirrors `convert_row`'s Conditional-flattening but
    /// recurses via tracked converters. (For Task 9 scope, only text bindings
    /// matter; the flattening is preserved for behavioural parity.)
    fn convert_row_tracked_ctx(
        &self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        path: &mut Vec<usize>,
        id_map: &mut DebugIdMap,
        probe: &mut BuildProbe,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let spacing = self.extract_u16(props, "spacing").unwrap_or(0);
        let padding = self.extract_u16(props, "padding").unwrap_or(0);
        let style = self.extract_style(props);

        let mut child_views: Vec<View<DynamicMessage>> = Vec::new();
        for (i, n) in children.iter().enumerate() {
            if let AuraNode::Conditional { condition, then_body, else_body, .. } = n {
                let is_true = self.eval_condition_with(condition, bindings);
                let empty = Vec::new();
                let body = if is_true { then_body } else { else_body.as_ref().unwrap_or(&empty) };
                for child_node in body {
                    path.push(i);
                    let v = self.convert_node_tracked_ctx(child_node, path, id_map, probe, bindings);
                    path.pop();
                    child_views.push(v);
                }
            } else {
                path.push(i);
                let v = self.convert_node_tracked_ctx(n, path, id_map, probe, bindings);
                path.pop();
                child_views.push(v);
            }
        }

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

    /// Tracked convert_container — mirrors `convert_container`.
    fn convert_container_tracked_ctx(
        &self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        path: &mut Vec<usize>,
        id_map: &mut DebugIdMap,
        probe: &mut BuildProbe,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let padding = self.extract_u16(props, "padding").unwrap_or(0);
        let width = self.extract_u16(props, "width");
        let height = self.extract_u16(props, "height");
        let style = self.extract_style(props);

        let child_view = if children.is_empty() {
            View::Empty
        } else if children.len() == 1 {
            path.push(0);
            let v = self.convert_node_tracked_ctx(&children[0], path, id_map, probe, bindings);
            path.pop();
            v
        } else {
            let views: Vec<View<DynamicMessage>> = children
                .iter()
                .enumerate()
                .map(|(i, n)| {
                    path.push(i);
                    let v = self.convert_node_tracked_ctx(n, path, id_map, probe, bindings);
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

    /// Tracked convert_center — mirrors `convert_center`.
    fn convert_center_tracked_ctx(
        &self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        path: &mut Vec<usize>,
        id_map: &mut DebugIdMap,
        probe: &mut BuildProbe,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let style = self.extract_style(props);

        let child_view = if children.is_empty() {
            View::Empty
        } else if children.len() == 1 {
            path.push(0);
            let v = self.convert_node_tracked_ctx(&children[0], path, id_map, probe, bindings);
            path.pop();
            v
        } else {
            let views: Vec<View<DynamicMessage>> = children
                .iter()
                .enumerate()
                .map(|(i, n)| {
                    path.push(i);
                    let v = self.convert_node_tracked_ctx(n, path, id_map, probe, bindings);
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

    /// Tracked plain text node conversion. For an interpolated text node this
    /// records each `${.field}` binding at the current node's path. Literal text
    /// records nothing. The produced View is identical to `convert_text_with`.
    fn convert_text_tracked_ctx(
        &self,
        content: &AuraTextContent,
        path: &mut Vec<usize>,
        probe: &mut BuildProbe,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let resolved = match content {
            AuraTextContent::Literal(s) => s.clone(),
            AuraTextContent::Interpolated { template, bindings: tpl_bindings } => {
                self.resolve_interpolation_tracked(template, tpl_bindings, bindings, path, probe)
            }
        };
        View::Text {
            content: resolved,
            style: None,
        }
    }

    /// Resolve an interpolation template AND record each binding into the probe
    /// at the current path. Returns the same resolved string as
    /// `resolve_interpolation_with`.
    fn resolve_interpolation_tracked(
        &self,
        template: &str,
        tpl_bindings: &[String],
        loop_bindings: &Bindings,
        path: &mut Vec<usize>,
        probe: &mut BuildProbe,
    ) -> String {
        let mut result = template.to_string();
        // Only build the probe path when there is at least one binding to record;
        // an empty `tpl_bindings` needs no probe entries.
        if !tpl_bindings.is_empty() {
            let probe_path: Vec<u16> = path.iter().map(|&x| x as u16).collect();
            for field_name in tpl_bindings {
                let pattern = format!("${{{}}}", format!(".{}", field_name));
                let value_str = self.read_state_as_string_with(field_name, loop_bindings);
                // Record the state binding at the current node's path.
                probe.record_state(&probe_path, pattern.clone(), value_str.clone());
                result = result.replace(&pattern, &value_str);
            }
        }
        result
    }

    /// Tracked convert_text_element — mirrors `convert_text_element`'s content
    /// extraction but, when the content comes from an interpolated child text
    /// node, records the binding at the current (text element) path.
    fn convert_text_element_tracked_ctx(
        &self,
        tag: &str,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        path: &mut Vec<usize>,
        probe: &mut BuildProbe,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let probe_path: Vec<u16> = path.iter().map(|&x| x as u16).collect();
        let content = self.extract_string_with(props, "text", bindings)
            .or_else(|| self.extract_string_with(props, "content", bindings))
            .or_else(|| self.extract_string_with(props, "label", bindings))
            .unwrap_or_else(|| {
                // Try to get content from child text nodes. For interpolated
                // children, also record each binding at this element's path.
                children.iter()
                    .filter_map(|c| match c {
                        AuraNode::Text(AuraTextContent::Literal(s)) => Some(s.clone()),
                        AuraNode::Text(AuraTextContent::Interpolated { template, bindings: tpl_bindings }) => {
                            // Record bindings for this child, attributed to the
                            // text element (current path) — consistent with the
                            // plain-text-node case.
                            for field_name in tpl_bindings {
                                let pattern = format!("${{{}}}", format!(".{}", field_name));
                                let value_str = self.read_state_as_string_with(field_name, bindings);
                                probe.record_state(&probe_path, pattern, value_str);
                            }
                            Some(self.resolve_interpolation_with(template, tpl_bindings, bindings))
                        }
                        _ => None,
                    })
                    .collect::<Vec<String>>()
                    .join("")
            });

        let mut style = self.extract_style(props);

        // Apply default heading styles, merging with user-provided styles
        if matches!(tag, "h1" | "h2" | "h3") {
            let default = match tag {
                "h1" => Style::parse("text-4xl font-bold").ok(),
                "h2" => Style::parse("text-3xl font-bold").ok(),
                "h3" => Style::parse("text-xl font-semibold").ok(),
                _ => None,
            };
            if let Some(mut default) = default {
                if let Some(user) = style.take() {
                    default.classes.extend(user.classes);
                }
                style = Some(default);
            }
        }

        // Heading styling is applied via the `style` field, not by transforming
        // `content`; matches untracked behaviour.
        View::Text {
            content,
            style,
        }
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
            "grid" => self.convert_grid(props, children, bindings),

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
            "checkbox" | "check" => self.convert_checkbox(props, events, bindings),
            "container" | "div" => self.convert_container(props, children, bindings),

            // Image placeholder
            "img" | "image" => self.convert_image(props),

            // Utility widgets
            "progress" => self.convert_progress(props),
            "spacer" => self.convert_spacer(props),
            "divider" | "hr" => self.convert_divider(props),
            "avatar" => self.convert_avatar(props),

            // Child widget lookup or fallback
            _ => {
                // Check if this tag matches a registered child widget
                if let Some(registry) = self.widget_registry {
                    if let Some(child_widget) = registry.get(tag) {
                        return self.render_child_widget(child_widget, props, events, bindings);
                    }
                }

                // Fallback: wrap children in a column, filtering out Empty views
                let views: Vec<View<DynamicMessage>> = children
                    .iter()
                    .map(|n| self.convert_node_with(n, bindings))
                    .filter(|v| !matches!(v, View::Empty))
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

    /// Render a child widget by looking it up in the registry.
    ///
    /// This resolves props from parent state, injects them as state fields,
    /// creates a child VmBridge, and recursively renders the child's view tree.
    fn render_child_widget(
        &self,
        child_widget: &crate::aura::AuraWidget,
        props: &HashMap<String, AuraPropValue>,
        _events: &HashMap<String, AuraEvent>,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        // 1. Resolve prop values from parent state
        let mut resolved_props: HashMap<String, Value> = HashMap::new();
        for (prop_name, prop_value) in props {
            if let AuraPropValue::Expr(expr) = prop_value {
                if let Some(val) = self.resolve_expr_to_value(expr, bindings) {
                    resolved_props.insert(prop_name.clone(), val);
                }
            }
        }

        // 2. Clone the widget and inject props as state vars so VmBridge registers them
        let mut modified_widget = child_widget.clone();
        for (prop_name, _val) in &resolved_props {
            if modified_widget.state_vars.iter().any(|v| v.name == *prop_name) {
                continue;
            }
            modified_widget.state_vars.push(crate::aura::AuraStateDef {
                name: prop_name.clone(),
                type_info: crate::ast::Type::StrOwned,
                initial: AuraExpr::Literal("".to_string()),
                decorators: vec![],
            });
        }

        // 3. Create VmBridge (state fields now include props)
        let mut child_bridge = match VmBridge::new(&modified_widget) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Warning: Failed to create child widget '{}': {:?}", child_widget.name, e);
                return View::Empty;
            }
        };

        // 4. Overwrite prop state fields with actual resolved values
        for (prop_name, val) in &resolved_props {
            let _ = child_bridge.write_state(prop_name, val.clone());
        }

        // 4b. Sync parent state to child's matching model vars
        // This allows hardcoded handlers (NewNote, Edit, etc.) that update
        // parent state (editing, edit_title, edit_body) to flow into the child widget.
        for state_var in &child_widget.state_vars {
            if let Ok(parent_val) = self.bridge.read_state(&state_var.name) {
                let _ = child_bridge.write_state(&state_var.name, parent_val);
            }
        }

        // 5. Build a child view builder and render the child's view tree
        let child_builder = AuraViewBuilder {
            bridge: &child_bridge,
            widget_name: child_widget.name.clone(),
            widget_registry: self.widget_registry,
        };

        child_builder.build(&child_widget.view_tree)
    }

    // ========================================================================
    // Layout converters
    // ========================================================================

    /// Convert a column element.
    // Tracked twin: convert_column_tracked_ctx — keep widget logic in sync.
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
    // Tracked twin: convert_row_tracked_ctx — keep widget logic in sync.
    fn convert_row(
        &self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let spacing = self.extract_u16(props, "spacing").unwrap_or(0);
        let padding = self.extract_u16(props, "padding").unwrap_or(0);
        let style = self.extract_style(props);

        // Flatten Conditional children: in a row, multiple condition children
        // should be spread horizontally, not wrapped in a Column
        let mut child_views: Vec<View<DynamicMessage>> = Vec::new();
        for n in children {
            if let AuraNode::Conditional { condition, then_body, else_body, .. } = n {
                let is_true = self.eval_condition_with(condition, bindings);
                let empty = Vec::new();
                let body = if is_true { then_body } else { else_body.as_ref().unwrap_or(&empty) };
                for child_node in body {
                    child_views.push(self.convert_node_with(child_node, bindings));
                }
            } else {
                child_views.push(self.convert_node_with(n, bindings));
            }
        }

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

    /// Iterate a `for` loop's iterable, converting its body once per item, and
    /// return the resulting views **flat** (one per iteration; multi-node bodies
    /// are wrapped in a Column). Used by `convert_grid` to flatten `for`
    /// children into individual grid cells — a bare `for` inside a grid must
    /// yield one cell per iteration, not a single wrapping Column. (Plan 323.)
    fn for_loop_iterations(
        &self,
        var: &str,
        index: &Option<String>,
        iterable: &str,
        body: &[AuraNode],
        bindings: &Bindings,
    ) -> Vec<View<DynamicMessage>> {
        let state_name = iterable.strip_prefix('.').unwrap_or(iterable);
        let array = match self.bridge.read_state(state_name) {
            Ok(Value::Array(arr)) => arr,
            Ok(_) => match self.bridge.read_state_as_vec(state_name) {
                Ok(vec) => auto_val::Array::from(vec),
                Err(_) => return Vec::new(),
            },
            Err(_) => return Vec::new(),
        };
        array
            .iter()
            .enumerate()
            .filter_map(|(i, item)| {
                if !self.matches_search(item) {
                    return None;
                }
                let mut loop_bindings = bindings.clone();
                loop_bindings.insert(var.to_string(), item.clone());
                if let Some(idx_var) = index {
                    loop_bindings.insert(idx_var.clone(), Value::Int(i as i32));
                }
                let views: Vec<View<DynamicMessage>> = body
                    .iter()
                    .map(|n| self.convert_node_with(n, &loop_bindings))
                    .collect();
                if views.is_empty() {
                    None
                } else if views.len() == 1 {
                    Some(views.into_iter().next().unwrap())
                } else {
                    Some(View::Column { children: views, spacing: 0, padding: 0, style: None })
                }
            })
            .collect()
    }

    /// Convert a grid element. iced has no native grid layout, so decompose
    /// into a **Column of Rows**: chunk the (grid-item) children into rows of
    /// `cols`, each row a horizontal Row. Cells that carry `text-center`
    /// auto-expand to Fill width in the iced text renderer (see `into_iced`'s
    /// `Text` arm), so the columns come out equally sized — a faithful calendar
    /// grid without a real grid primitive. `grid-item` itself is transparent:
    /// it falls through to the generic fallback, which returns its single inner
    /// child, so converting each grid-item yields the cell content directly.
    /// Tracked twin: `convert_grid_tracked_ctx` — keep in sync.
    fn convert_grid(
        &self,
        props: &HashMap<String, AuraPropValue>,
        children: &[AuraNode],
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let cols = self
            .extract_u16(props, "cols")
            .or_else(|| self.extract_u16(props, "columns"))
            .map(|c| (c as usize).max(1))
            .unwrap_or(1);
        let gap = self.extract_u16(props, "gap").unwrap_or(0);
        let style = self.extract_style(props);

        // Flatten `for`-loop children into individual cells: a bare `for`
        // inside a grid must yield one cell per iteration, not a single
        // wrapping Column (Plan 323). Other children convert to one cell each.
        let cells: Vec<View<DynamicMessage>> = children
            .iter()
            .flat_map(|n| match n {
                AuraNode::ForLoop { var, index, iterable, body, .. } => {
                    self.for_loop_iterations(var, index, iterable, body, bindings)
                }
                other => {
                    let v = self.convert_node_with(other, bindings);
                    if matches!(v, View::Empty) { Vec::new() } else { vec![v] }
                }
            })
            .collect();

        if cells.is_empty() {
            return View::Empty;
        }

        // Grid decomposition (final-row padding + w-full rows + col-of-rows)
        // now lives in ONE place: the shared generic `build_grid` in the iced
        // renderer (plus the GPUI inline twin). Construct `View::Grid` here;
        // both render paths (render_dynamic_view VM, into_iced rust) consume
        // it identically, so they can never drift again. (Plan 319.)
        View::Grid { cols, gap, cells, style }
    }

    /// Convert a container element.
    // Tracked twin: convert_container_tracked_ctx — keep widget logic in sync.
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
    // Tracked twin: convert_center_tracked_ctx — keep widget logic in sync.
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

        // Apply default heading styles, merging with user-provided styles
        if matches!(tag, "h1" | "h2" | "h3") {
            let default = match tag {
                "h1" => Style::parse("text-4xl font-bold").ok(),
                "h2" => Style::parse("text-3xl font-bold").ok(),
                "h3" => Style::parse("text-xl font-semibold").ok(),
                _ => None,
            };
            if let Some(mut default) = default {
                if let Some(user) = style.take() {
                    default.classes.extend(user.classes);
                }
                style = Some(default);
            }
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

        // `variant` selects a base style preset (Tailwind classes); the user's
        // class/style augments it. "text"/absent = chromeless (renders as text
        // via the renderer's class-driven style); "primary" = filled blue.
        let variant = self.extract_string_with(props, "variant", bindings)
            .unwrap_or_default();
        let preset: &str = match variant.as_str() {
            "primary" => "bg-blue-500 hover:bg-blue-600 text-white font-medium rounded",
            // "text" and any other/absent value: no preset — chromeless by default.
            _ => "",
        };
        let style = {
            // Binding-aware so a class can come from the loop variable, e.g.
            // `class: cell.style` where each cell carries its own Tailwind class.
            let user = self.extract_string_with(props, "class", bindings)
                .or_else(|| self.extract_string_with(props, "style", bindings));
            let merged = match (preset, user.as_deref()) {
                ("", None) => String::new(),
                ("", Some(c)) => c.to_string(),
                (p, None) => p.to_string(),
                (p, Some(c)) => format!("{} {}", p, c),
            };
            if merged.is_empty() { None } else { Style::parse(&merged).ok() }
        };

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

        let on_submit = events.get("onenter")
            .or_else(|| events.get("enter"))
            .map(|event| self.event_to_message(&event.handler));

        let mut builder = View::<DynamicMessage>::input(placeholder).value(value);
        if password {
            builder = builder.password();
        }
        if let Some(msg) = on_change {
            builder = builder.on_change(msg);
        }
        if let Some(msg) = on_submit {
            builder = builder.on_submit(msg);
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
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        let label = self.extract_string(props, "text")
            .or_else(|| self.extract_string(props, "label"))
            .unwrap_or_default();

        // Resolve checked from state ref, literal, or binding path (e.g., todo.done)
        let is_checked = props.get("checked")
            .or_else(|| props.get("is_checked"))
            .map(|v| match v {
                AuraPropValue::Expr(expr) => {
                    self.resolve_expr_to_value(expr, bindings)
                        .map(|val| val.as_bool())
                }
                _ => None,
            })
            .flatten()
            .unwrap_or(false);

        let on_toggle = events.get("onclick")
            .or_else(|| events.get("change"))
            .or_else(|| events.get("onchange"))
            .map(|event| self.event_to_message_with(event, bindings));

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
            AuraExpr::Index { target, index } => {
                let target_val = self.resolve_expr_to_value(target, bindings)?;
                let index_val = self.resolve_expr_to_value(index, bindings)?;
                match (&target_val, &index_val) {
                    (Value::Array(arr), Value::Int(i)) => {
                        let idx = *i as usize;
                        if idx < arr.len() { Some(arr[idx].clone()) } else { None }
                    }
                    (Value::Obj(map), Value::Str(key)) => map.get(key.as_str()),
                    // Plan 336: index into a list/array stored as a VmRef or Int
                    // array_id (List<T> / Vec from `var x = []; x.push(...)`). The
                    // EditorPanel's `note: .notes[.active_id]` reads `.notes`
                    // (a VmRef to ListData) and indexes it. Deref to Vec<Value>
                    // first, then index. Use read_state_as_vec via a temp field
                    // name when the target is a StateRef; otherwise deref inline.
                    (Value::VmRef(r), Value::Int(i)) => {
                        let v = self.bridge.index_list(r.id, *i);
                        // Plan 336: list elements are struct ids (Int(4M)/VmRef);
                        // materialize so FieldAccess (.note.title/.note.body) resolves.
                        v.map(|e| self.bridge.materialize_obj_ref(&e))
                    }
                    (Value::Int(id), Value::Int(i)) if *id >= 2_000_000 => {
                        let v = self.bridge.index_list(*id as usize, *i);
                        v.map(|e| self.bridge.materialize_obj_ref(&e))
                    }
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

    /// Check if a loop item matches the current search filter.
    ///
    /// Reads the `search` state field. If it's empty or doesn't exist, all items match.
    /// If non-empty and the item is an Obj, checks if any string field contains the search text.
    fn matches_search(&self, item: &Value) -> bool {
        let search_text = match self.bridge.read_state("search") {
            Ok(Value::Str(s)) => s.to_string(),
            Ok(Value::String(s)) => s.to_string(),
            _ => return true, // no search field or non-string → show all
        };
        if search_text.is_empty() {
            return true;
        }
        let search_lower = search_text.to_lowercase();
        match item {
            Value::Obj(map) => {
                // Check title field for a match
                let title = map.get("title").map(|v| value_to_display_string(&v)).unwrap_or_default();
                title.to_lowercase().contains(&search_lower)
            }
            _ => true, // non-obj items always match
        }
    }

    /// Evaluate a condition string against current state (no bindings).
    fn eval_condition(&self, condition: &str) -> bool {
        self.eval_condition_with(condition, &Bindings::new())
    }

    /// Find an operator (e.g., " || " or " && ") at parenthesis depth 0 only.
    /// Returns the byte position of the operator start, or None if not found at top level.
    fn find_operator_at_depth0(cond: &str, op: &str) -> Option<usize> {
        let mut depth = 0i32;
        let op_bytes = op.as_bytes();
        let cond_bytes = cond.as_bytes();
        let mut i = 0;
        while i + op_bytes.len() <= cond_bytes.len() {
            match cond_bytes[i] {
                b'(' => depth += 1,
                b')' => depth -= 1,
                _ => {}
            }
            if depth == 0 && cond_bytes[i..].starts_with(op_bytes) {
                return Some(i);
            }
            i += 1;
        }
        None
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

        // Strip outer parentheses for grouped expressions like (...) —
        // do this BEFORE operator splitting so that inner || / && are not split prematurely.
        // Repeat in case of nested parens like ((expr)).
        let mut cond = cond;
        loop {
            if cond.starts_with('(') && cond.ends_with(')') {
                // Verify the closing ')' matches the opening '(' (balanced)
                let mut depth = 0i32;
                let mut matched = true;
                for (i, ch) in cond.char_indices() {
                    match ch {
                        '(' => depth += 1,
                        ')' => depth -= 1,
                        _ => {}
                    }
                    if depth == 0 && i < cond.len() - 1 {
                        // Closing paren found before end — outer parens don't match
                        matched = false;
                        break;
                    }
                }
                if matched {
                    cond = cond[1..cond.len()-1].trim();
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // Handle || (OR) — split at top level only (paren depth 0)
        if let Some(pos) = Self::find_operator_at_depth0(cond, " || ") {
            let left = &cond[..pos];
            let right = &cond[pos + 4..];
            return self.eval_condition_with(left, bindings)
                || self.eval_condition_with(right, bindings);
        }

        // Handle && (AND) — split at top level only (paren depth 0)
        if let Some(pos) = Self::find_operator_at_depth0(cond, " && ") {
            let left = &cond[..pos];
            let right = &cond[pos + 4..];
            return self.eval_condition_with(left, bindings)
                && self.eval_condition_with(right, bindings);
        }

        // Find operator to split into lhs op rhs
        let (lhs, op, rhs) = if let Some(pos) = cond.find(" == ") {
            (&cond[..pos], "==", cond[pos + 4..].trim())
        } else if let Some(pos) = cond.find(" != ") {
            (&cond[..pos], "!=", cond[pos + 4..].trim())
        } else if let Some(pos) = cond.find(" > ") {
            (&cond[..pos], ">", cond[pos + 3..].trim())
        } else if let Some(pos) = cond.find(" < ") {
            (&cond[..pos], "<", cond[pos + 3..].trim())
        } else if let Some(pos) = cond.find(" >= ") {
            (&cond[..pos], ">=", cond[pos + 4..].trim())
        } else if let Some(pos) = cond.find(" <= ") {
            (&cond[..pos], "<=", cond[pos + 4..].trim())
        } else if cond.starts_with('.') {
            // Bare state ref — truthy check
            return self.bridge.read_state(&cond[1..])
                .map(|v| v.as_bool())
                .unwrap_or(false);
        } else {
            // Try binding path truthy check
            return self.resolve_binding_path(cond, bindings)
                .map(|v| v.as_bool())
                .unwrap_or(false);
        };

        // Read state value for lhs
        // Normalize spaces inside .len() so "notes.len ( )" matches ".len()" suffix.
        // The parser may produce "len ( )" with spaces inside the parens.
        let lhs_normalized = lhs.replace(" ( ", "(").replace("( ", "(").replace(" )", ")");
        let lhs_val = if let Some(field_name) = lhs_normalized.strip_suffix(".len()") {
            // Strip leading dot from state ref (e.g., ".todos" → "todos")
            let field_name = field_name.trim_start_matches('.');
            match self.bridge.read_state(field_name) {
                Ok(Value::Array(arr)) => arr.len().to_string(),
                Ok(other) => {
                    // Also try read_state_as_vec for Value::Int(array_id) refs
                    match self.bridge.read_state_as_vec(field_name) {
                        Ok(vec) => vec.len().to_string(),
                        Err(_) => value_to_display_string(&other),
                    }
                }
                Err(_) => return false,
            }
        } else if let Some(val) = self.resolve_binding_path(lhs, bindings) {
            // Binding path (e.g., "todo.done")
            value_to_display_string(&val)
        } else if lhs.starts_with('.') {
            // State ref (e.g., ".filter")
            let name = &lhs[1..];
            match self.bridge.read_state(name) {
                Ok(v) => value_to_display_string(&v),
                Err(_) => return false,
            }
        } else {
            match self.bridge.read_state(lhs) {
                Ok(v) => value_to_display_string(&v),
                Err(_) => return false,
            }
        };

        // Resolve rhs: check loop bindings first, then try as literal
        let rhs_val = if let Some(val) = bindings.get(rhs) {
            value_to_display_string(val)
        } else if let Some(val) = self.resolve_binding_path(rhs, bindings) {
            value_to_display_string(&val)
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
        let event_name = extract_handler_name(&event.handler).to_string();
        // Resolve each declared parameter from the loop bindings (e.g.
        // `onclick: .SelectDay(cell.date)` → resolve `cell.date` against the
        // current iteration's bindings) and carry the values in `args`. The
        // dispatcher (DynamicComponent::on) forwards `args` to
        // `call_handler`, so a handler declared `.SelectDay(date) ->` receives
        // the value as its `date` parameter. (Previously the value was
        // string-encoded into `event_name` as `name:value`, which the dispatch
        // path never parsed — so payload onclicks silently no-op'd.)
        let mut args: Vec<Value> = Vec::with_capacity(event.params.len());
        for param_name in &event.params {
            if let Some(val) = self.resolve_binding_path(param_name, bindings) {
                args.push(val);
            }
        }
        DynamicMessage::Typed {
            widget_name: self.widget_name.clone(),
            event_name,
            args,
        }
    }

    /// Internal: convert handler string to DynamicMessage (used by event_to_message).
    /// Resolve a dotted binding path like "note.id" from loop variable bindings.
    /// Splits on '.', looks up the root in bindings, then navigates fields on Obj values.
    fn resolve_binding_path(&self, path: &str, bindings: &Bindings) -> Option<Value> {
        let parts: Vec<&str> = path.split('.').collect();
        if parts.is_empty() {
            return None;
        }
        // First segment: look up in bindings
        let mut val = bindings.get(parts[0])?.clone();
        // Remaining segments: field access on Obj
        for field in &parts[1..] {
            match val {
                Value::Obj(map) => {
                    val = map.get(*field)?;
                }
                _ => return None,
            }
        }
        Some(val)
    }

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
            api_imports: vec![],
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

    // ========================================================================
    // Plan 307 Task 9 — deep BuildProbe threading tests
    // ========================================================================

    #[test]
    fn build_with_debug_captures_nested_state_binding() {
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

        let node = AuraNode::Element {
            tag: "col".to_string(),
            props: HashMap::new(),
            events: HashMap::new(),
            children: vec![
                AuraNode::Text(AuraTextContent::Interpolated {
                    template: "Count: ${.count}".to_string(),
                    bindings: vec!["count".to_string()],
                }),
            ],
            span: None,
            debug_id: None,
        };
        let (_view, _id_map, probe) = builder.build_with_debug(&node);
        let snap = probe.snapshot();
        // exactly one path captured (the nested text node), with one state binding
        assert_eq!(snap.len(), 1, "nested text node should be probed");
        let entry = snap.values().next().unwrap();
        assert_eq!(entry.state_bindings.len(), 1);
        assert_eq!(entry.state_bindings[0].expr, "${.count}");
        assert_eq!(entry.state_bindings[0].current_value, "42");
    }

    #[test]
    fn build_with_debug_skips_literal_text_sibling() {
        // col with two text children: one interpolated, one literal.
        // Only the interpolated one should produce a probe entry.
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

        let node = AuraNode::Element {
            tag: "col".to_string(),
            props: HashMap::new(),
            events: HashMap::new(),
            children: vec![
                AuraNode::Text(AuraTextContent::Interpolated {
                    template: "Count: ${.count}".to_string(),
                    bindings: vec!["count".to_string()],
                }),
                AuraNode::Text(AuraTextContent::Literal("static".to_string())),
            ],
            span: None,
            debug_id: None,
        };
        let (_view, _id_map, probe) = builder.build_with_debug(&node);
        let snap = probe.snapshot();
        assert_eq!(snap.len(), 1, "only the interpolated text node is probed");
        let entry = snap.values().next().unwrap();
        assert_eq!(entry.state_bindings.len(), 1);
        assert_eq!(entry.state_bindings[0].expr, "${.count}");
    }

    #[test]
    fn build_with_debug_records_nothing_for_literal_text() {
        // A literal-only text node at top level records nothing.
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Test");

        let node = AuraNode::Text(AuraTextContent::Literal("just text".to_string()));
        let (_view, _id_map, probe) = builder.build_with_debug(&node);
        let snap = probe.snapshot();
        assert!(snap.is_empty(), "literal-only text must not be probed");
    }

    #[test]
    fn build_with_debug_captures_for_loop_context() {
        use crate::ui::debug::ForIter;
        // Widget declares an `items` state field (initial dummy, overwritten
        // below). `AuraExpr` has no array literal, so we seed via write_state.
        let widget = make_test_widget("List", vec![
            AuraStateDef {
                name: "items".to_string(),
                type_info: Type::List(Box::new(Type::StrSlice)),
                initial: AuraExpr::Literal(String::new()),
                decorators: vec![],
            },
        ]);
        let mut bridge = VmBridge::new(&widget).unwrap();
        bridge.write_state(
            "items",
            Value::Array(auto_val::Array::from(vec![
                Value::str("apple"),
                Value::str("banana"),
                Value::str("cherry"),
            ])),
        ).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "List");

        // for item in .items { text("${.item}") }
        let node = AuraNode::ForLoop {
            var: "item".to_string(),
            index: None,
            iterable: ".items".to_string(),
            body: vec![AuraNode::Text(AuraTextContent::Interpolated {
                template: "${.item}".to_string(),
                bindings: vec!["item".to_string()],
            })],
            span: None,
            debug_id: None,
        };
        let (_view, _id_map, probe) = builder.build_with_debug(&node);
        let snap = probe.snapshot();
        let for_entries: Vec<&ForIter> = snap.values()
            .filter_map(|e| e.for_context.as_ref())
            .collect();
        assert_eq!(for_entries.len(), 3, "three iterations captured");
        let mut by_index: Vec<(usize, &str)> = for_entries.iter()
            .map(|f| (f.index.unwrap(), f.value_repr.as_str()))
            .collect();
        by_index.sort_by_key(|(i, _)| *i);
        assert_eq!(by_index, vec![(0, "apple"), (1, "banana"), (2, "cherry")]);
        assert_eq!(for_entries[0].var, "item");
        assert_eq!(for_entries[0].iterable_repr, ".items");
    }

    #[test]
    fn build_with_debug_for_loop_single_body_path_matches_vtree() {
        // Plan 309 Phase 1 (Fix A): a ForLoop whose body is a single node
        // yields that node *directly* per iteration (no wrapping Column), so
        // its flattened VTree path is the one-segment `[i]`. The tracked
        // builder must record under the SAME path, or the renderer's
        // `probe.snapshot().get(&node.path)` lookup misses and the inspector's
        // AutoUI/source data stays empty for loop bodies.
        //
        // Before Fix A the builder recorded at `[i, 0]` (it pushed the
        // body-index unconditionally) — this test would fail with len==2.
        let widget = make_test_widget("List", vec![
            AuraStateDef {
                name: "items".to_string(),
                type_info: Type::List(Box::new(Type::StrSlice)),
                initial: AuraExpr::Literal(String::new()),
                decorators: vec![],
            },
        ]);
        let mut bridge = VmBridge::new(&widget).unwrap();
        bridge.write_state(
            "items",
            Value::Array(auto_val::Array::from(vec![
                Value::str("apple"),
                Value::str("banana"),
            ])),
        ).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "List");

        // for item in .items { text("${.item}") }  — body.len() == 1
        let node = AuraNode::ForLoop {
            var: "item".to_string(),
            index: None,
            iterable: ".items".to_string(),
            body: vec![AuraNode::Text(AuraTextContent::Interpolated {
                template: "${.item}".to_string(),
                bindings: vec!["item".to_string()],
            })],
            span: None,
            debug_id: None,
        };
        let (_view, _id_map, probe) = builder.build_with_debug(&node);
        let snap = probe.snapshot();
        let mut path_keys: Vec<Vec<u16>> = snap.keys().cloned().collect();
        path_keys.sort();
        // one entry per iteration (each combines for_context + state binding),
        // each at a single-segment path
        assert_eq!(path_keys, vec![vec![0u16], vec![1u16]],
            "Fix A: single-body loop body paths are [i], not [i, 0]");
        // each entry carries both the for-context and the state binding
        for k in &path_keys {
            let entry = snap.get(k).unwrap();
            assert!(entry.for_context.is_some(), "for_context present at {:?}", k);
            assert_eq!(entry.state_bindings.len(), 1, "state binding present at {:?}", k);
        }
    }

    #[test]
    fn build_with_debug_for_loop_multi_body_path_keeps_body_index() {
        // Plan 309 Phase 1 (Fix A) regression guard: a multi-node body is
        // wrapped in a Column per iteration, so the body-index level IS
        // present in the VTree — the builder must STILL push it (`[i, bi]`).
        let widget = make_test_widget("List", vec![
            AuraStateDef {
                name: "items".to_string(),
                type_info: Type::List(Box::new(Type::StrSlice)),
                initial: AuraExpr::Literal(String::new()),
                decorators: vec![],
            },
        ]);
        let mut bridge = VmBridge::new(&widget).unwrap();
        bridge.write_state(
            "items",
            Value::Array(auto_val::Array::from(vec![Value::str("x")])),
        ).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "List");

        // for item in .items { text("${.item}"); text("tail") }  — body.len() == 2
        let node = AuraNode::ForLoop {
            var: "item".to_string(),
            index: None,
            iterable: ".items".to_string(),
            body: vec![
                AuraNode::Text(AuraTextContent::Interpolated {
                    template: "${.item}".to_string(),
                    bindings: vec!["item".to_string()],
                }),
                AuraNode::Text(AuraTextContent::Literal("tail".to_string())),
            ],
            span: None,
            debug_id: None,
        };
        let (_view, _id_map, probe) = builder.build_with_debug(&node);
        let snap = probe.snapshot();
        // first body node at [0, 0]; literal "tail" produces no probe entry.
        // The interpolated node keeps the two-segment path (body-index present).
        assert!(snap.contains_key(&vec![0u16, 0u16]),
            "multi-body loop keeps body-index level: key [0,0] expected");
        assert!(!snap.contains_key(&vec![0u16]),
            "multi-body loop must NOT collapse to single-segment [0]");
    }

    // ========================================================================
    // Plan 323 — `for` inside `grid` must flatten to one cell per iteration
    // (both the non-tracked `build` path and the tracked DevTools path).
    // ========================================================================

    /// Shared grid-with-for-loop node: a `grid` whose only child is a `for`
    /// over a 7-element `items` array, body = single text per iteration.
    fn grid_with_for_loop_node() -> AuraNode {
        let for_loop = AuraNode::ForLoop {
            var: "item".to_string(),
            index: None,
            iterable: ".items".to_string(),
            body: vec![AuraNode::Text(AuraTextContent::Interpolated {
                template: "${.item}".to_string(),
                bindings: vec!["item".to_string()],
            })],
            span: None,
            debug_id: None,
        };
        AuraNode::element("grid")
            .with_prop("cols", AuraExpr::Int(7))
            .with_child(for_loop)
    }

    fn widget_with_items() -> (AuraWidget, VmBridge) {
        let widget = make_test_widget("Grid", vec![
            AuraStateDef {
                name: "items".to_string(),
                type_info: Type::List(Box::new(Type::StrSlice)),
                initial: AuraExpr::Literal(String::new()),
                decorators: vec![],
            },
        ]);
        let mut bridge = VmBridge::new(&widget).unwrap();
        bridge.write_state(
            "items",
            Value::Array(auto_val::Array::from(vec![
                Value::str("a"), Value::str("b"), Value::str("c"),
                Value::str("d"), Value::str("e"), Value::str("f"),
                Value::str("g"),
            ])),
        ).unwrap();
        (widget, bridge)
    }

    #[test]
    fn convert_grid_flattens_for_loop_into_cells() {
        // Non-tracked `build` path (used by into_iced / codegen). Before the
        // Plan 323 fix the `for` returned a single wrapping Column, so the grid
        // saw ONE cell instead of 7 → a "calendar" rendered as a single tall
        // column. This test pins the flattened behaviour.
        let (widget, bridge) = widget_with_items();
        let _ = widget;
        let builder = AuraViewBuilder::new(&bridge, "Grid");

        let view = builder.build(&grid_with_for_loop_node());
        match view {
            View::Grid { cols, cells, .. } => {
                assert_eq!(cols, 7);
                assert_eq!(cells.len(), 7,
                    "for inside grid must flatten to one cell per iteration");
            }
            other => panic!("Expected View::Grid with 7 cells, got {:?} (kind)",
                std::mem::discriminant(&other)),
        }
    }

    #[test]
    fn convert_grid_tracked_flattens_for_loop_into_cells() {
        // Tracked `build_with_debug` path (used by VM `render_dynamic_view` for
        // EVERY frame, even with F12 off). Same flattening requirement, plus the
        // per-cell probe paths must be the sequential cell indices [0..7] so
        // they match the render-time Grid-arm visit order.
        let (widget, bridge) = widget_with_items();
        let _ = widget;
        let builder = AuraViewBuilder::new(&bridge, "Grid");

        let (view, _id_map, probe) = builder.build_with_debug(&grid_with_for_loop_node());
        match view {
            View::Grid { cells, .. } => {
                assert_eq!(cells.len(), 7,
                    "tracked for-in-grid must also flatten to 7 cells");
            }
            other => panic!("Expected tracked View::Grid with 7 cells, got {:?} (kind)",
                std::mem::discriminant(&other)),
        }
        let snap = probe.snapshot();
        let mut keys: Vec<Vec<u16>> = snap.keys().cloned().collect();
        keys.sort();
        // Each iteration's for-context recorded at the flat cell path [0..7].
        assert_eq!(keys, vec![vec![0u16], vec![1], vec![2], vec![3], vec![4], vec![5], vec![6]],
            "tracked grid cell paths must be flat sequential [0..7]");
        for k in &keys {
            assert!(snap.get(k).unwrap().for_context.is_some(),
                "for_context present at flat cell path {:?}", k);
        }
    }

    #[test]
    fn convert_grid_for_loop_obj_cells_resolve_field_access() {
        // Plan 323 Phase 2: the calendar's real pattern — a `for` over an array
        // of Obj cells, each cell rendered as a `button cell.label`. This tests
        // that (a) the loop flattens to N cells AND (b) the FieldAccess label
        // `cell.label` actually resolves to each Obj's "label" string. If this
        // fails, an empty day grid is explained by the view path (hypothesis B),
        // not just the grid-flattening.
        let widget = make_test_widget("Cal", vec![
            AuraStateDef {
                name: "days".to_string(),
                type_info: Type::List(Box::new(Type::StrSlice)),
                initial: AuraExpr::Literal(String::new()),
                decorators: vec![],
            },
        ]);
        let mut bridge = VmBridge::new(&widget).unwrap();

        fn cell(label: &str, date: &str, other: bool) -> Value {
            let mut o = auto_val::Obj::new();
            o.set("label", Value::str(label));
            o.set("date", Value::str(date));
            o.set("is_other_month", Value::Bool(other));
            Value::Obj(o)
        }
        bridge.write_state(
            "days",
            Value::Array(auto_val::Array::from(vec![
                cell("31", "2026-05-31", true),
                cell("1", "2026-06-01", false),
                cell("2", "2026-06-02", false),
            ])),
        ).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Cal");

        // grid { for cell in .days { button cell.label { ... } } }
        let for_loop = AuraNode::ForLoop {
            var: "cell".to_string(),
            index: None,
            iterable: ".days".to_string(),
            body: vec![AuraNode::Element {
                tag: "button".to_string(),
                props: HashMap::from([(
                    "label".to_string(),
                    AuraPropValue::Expr(AuraExpr::FieldAccess {
                        object: Box::new(AuraExpr::StateRef("cell".to_string())),
                        field: "label".to_string(),
                    }),
                )]),
                events: HashMap::new(),
                children: vec![],
                span: None,
                debug_id: None,
            }],
            span: None,
            debug_id: None,
        };
        let grid = AuraNode::element("grid")
            .with_prop("cols", AuraExpr::Int(7))
            .with_child(for_loop);

        let view = builder.build(&grid);
        match view {
            View::Grid { cells, .. } => {
                assert_eq!(cells.len(), 3, "for over 3 Obj cells → 3 grid cells");
                let labels: Vec<String> = cells.iter().map(|c| match c {
                    View::Button { label, .. } => label.clone(),
                    _ => "(not button)".to_string(),
                }).collect();
                assert_eq!(labels, vec!["31", "1", "2"],
                    "button cell.label must resolve each Obj's label field");
            }
            other => panic!("Expected View::Grid, got discriminant {:?}",
                std::mem::discriminant(&other)),
        }
    }

    // ========================================================================
    // Plan 307 Task 11 — event handler binding capture
    // ========================================================================

    #[test]
    fn build_with_debug_captures_event_handler() {
        use crate::aura::AuraEvent;
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Test");

        let node = AuraNode::Element {
            tag: "button".to_string(),
            props: HashMap::from([
                ("label".to_string(), AuraPropValue::Expr(AuraExpr::Literal("Inc".to_string()))),
            ]),
            events: HashMap::from([
                ("onclick".to_string(), AuraEvent {
                    handler: ".Inc".to_string(),
                    params: vec![],
                }),
            ]),
            children: vec![],
            span: None,
            debug_id: None,
        };
        let (_view, _id_map, probe) = builder.build_with_debug(&node);
        let snap = probe.snapshot();
        let all_events: Vec<_> = snap.values().flat_map(|e| e.events.iter()).collect();
        assert_eq!(all_events.len(), 1);
        assert_eq!(all_events[0].event, "onclick");
        assert_eq!(all_events[0].handler, ".Inc");
    }

    #[test]
    fn build_with_debug_captures_event_handlers_distinct_paths() {
        use crate::aura::AuraEvent;
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Test");

        let button = |handler: &str| AuraNode::Element {
            tag: "button".to_string(),
            props: HashMap::new(),
            events: HashMap::from([
                ("onclick".to_string(), AuraEvent {
                    handler: handler.to_string(),
                    params: vec![],
                }),
            ]),
            children: vec![],
            span: None,
            debug_id: None,
        };

        let node = AuraNode::Element {
            tag: "col".to_string(),
            props: HashMap::new(),
            events: HashMap::new(),
            children: vec![button(".Inc"), button(".Dec")],
            span: None,
            debug_id: None,
        };
        let (_view, _id_map, probe) = builder.build_with_debug(&node);
        let snap = probe.snapshot();
        // Each button is captured at its own child path with one event each.
        let event_paths: Vec<(Vec<u16>, &str)> = snap.iter()
            .flat_map(|(path, e)| e.events.iter().map(move |ev| (path.clone(), ev.handler.as_str())))
            .collect();
        assert_eq!(event_paths.len(), 2, "two events captured");
        let handlers: Vec<&str> = {
            let mut h = event_paths.iter().map(|(_, h)| *h).collect::<Vec<_>>();
            h.sort();
            h
        };
        assert_eq!(handlers, vec![".Dec", ".Inc"]);
        // distinct paths
        assert_ne!(event_paths[0].0, event_paths[1].0, "distinct child paths");
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

    #[test]
    fn test_eval_condition_with_bindings() {
        // Simulate a todo item as a binding
        let mut bindings = Bindings::new();
        let mut todo_obj = auto_val::Obj::new();
        todo_obj.set("id", Value::Int(0));
        todo_obj.set("text", Value::str("Buy milk"));
        todo_obj.set("done", Value::Bool(false));
        bindings.insert("todo".to_string(), Value::Obj(todo_obj));

        // Set up state: filter = "active"
        let widget = make_test_widget("App", vec![
            AuraStateDef {
                name: "filter".to_string(),
                type_info: Type::StrOwned,
                initial: AuraExpr::Literal("active".to_string()),
                decorators: vec![],
            },
            AuraStateDef {
                name: "todos".to_string(),
                type_info: Type::StrOwned,
                initial: AuraExpr::Literal("[]".to_string()),
                decorators: vec![],
            },
        ]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "App");

        // Test: .filter == "active" && todo.done == false → should be true
        let cond1 = ".filter == \"active\" && todo.done == false";
        let r1 = builder.eval_condition_with(cond1, &bindings);
        eprintln!("cond1='{}' result={}", cond1, r1);
        assert!(r1, "Expected true for active filter with done=false, got false");

        // Test: .filter == "all" → should be true when filter is "active"? No, false
        let cond2 = ".filter == \"all\"";
        let r2 = builder.eval_condition_with(cond2, &bindings);
        eprintln!("cond2='{}' result={}", cond2, r2);
        assert!(!r2, "Expected false for 'all' filter when filter is 'active'");

        // Test the full compound condition: .filter == "all" || ( .filter == "active" && todo.done == false )
        let cond_full = ".filter == \"all\" || ( .filter == \"active\" && todo.done == false )";
        let r_full = builder.eval_condition_with(cond_full, &bindings);
        eprintln!("cond_full='{}' result={}", cond_full, r_full);

        // Also test the inner part directly
        let inner = ".filter == \"active\" && todo.done == false";
        eprintln!("inner='{}' result={}", inner, builder.eval_condition_with(inner, &bindings));

        // Test right side of || directly
        let right = "( .filter == \"active\" && todo.done == false )";
        eprintln!("right='{}' result={}", right, builder.eval_condition_with(right, &bindings));

        assert!(r_full, "Expected true for full condition with active filter + undone todo");

        // Test with done=true AND filter="completed"
        let mut todo_done = auto_val::Obj::new();
        todo_done.set("id", Value::Int(0));
        todo_done.set("text", Value::str("Done item"));
        todo_done.set("done", Value::Bool(true));
        let mut bindings_done = Bindings::new();
        bindings_done.insert("todo".to_string(), Value::Obj(todo_done));

        // Create a builder with filter="completed" state
        let widget_completed = make_test_widget("App", vec![
            AuraStateDef {
                name: "filter".to_string(),
                type_info: Type::StrOwned,
                initial: AuraExpr::Literal("completed".to_string()),
                decorators: vec![],
            },
        ]);
        let bridge_completed = VmBridge::new(&widget_completed).unwrap();
        let builder_completed = AuraViewBuilder::new(&bridge_completed, "App");

        let cond_completed = "( .filter == \"completed\" && todo.done == true )";
        assert!(builder_completed.eval_condition_with(cond_completed, &bindings_done)
            || {
                // Also try without parens (parser may produce either)
                let cond2 = ".filter == \"completed\" && todo.done == true";
                builder_completed.eval_condition_with(cond2, &bindings_done)
            },
            "Expected true for completed filter + done todo");

        // Active filter should NOT match done item
        let cond_active_done = ".filter == \"active\" && todo.done == false";
        assert!(!builder.eval_condition_with(cond_active_done, &bindings_done),
            "Expected false for active filter + done=true todo");

        // Test editing_id conditions (the "double input" bug)
        // When editing_id=-1 and todo.id=0, editing_id != todo.id
        // Need a builder with editing_id state
        let widget_edit = make_test_widget("App", vec![
            AuraStateDef {
                name: "editing_id".to_string(),
                type_info: Type::StrOwned,
                initial: AuraExpr::Literal("-1".to_string()),
                decorators: vec![],
            },
        ]);
        let bridge_edit = VmBridge::new(&widget_edit).unwrap();
        let builder_edit = AuraViewBuilder::new(&bridge_edit, "App");

        let cond_edit_eq = ".editing_id == todo.id";
        let r_eq = builder_edit.eval_condition_with(cond_edit_eq, &bindings);
        eprintln!("editing_id==-1, todo.id=0: '.editing_id == todo.id' => {}", r_eq);
        assert!(!r_eq, "editing_id=-1 should NOT equal todo.id=0");

        let cond_edit_neq = ".editing_id != todo.id";
        let r_neq = builder_edit.eval_condition_with(cond_edit_neq, &bindings);
        eprintln!("editing_id==-1, todo.id=0: '.editing_id != todo.id' => {}", r_neq);
        assert!(r_neq, "editing_id=-1 should NOT equal todo.id=0 (neq)");
    }
}
