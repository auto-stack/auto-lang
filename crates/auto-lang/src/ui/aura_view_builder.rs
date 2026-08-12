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
//!  - Resolves Expr::Ident state references via VmBridge::read_state()
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

use auto_val::{Op, Value};

use crate::ast::Expr;
use crate::aura::{AuraNode, AuraPropValue, AuraTextContent, AuraEvent, aura_events_get_base};

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

    /// Plan 318: imported declarations shared with child widgets.
    import_stmts: Option<&'a [crate::ast::Stmt]>,

    /// Plan 320: when rendering a child widget's view tree, this overrides the
    /// bridge's root state_obj_id so read_state reads from the child's state
    /// object instead of the root widget's. None = use root state.
    override_state_obj_id: Option<u64>,

    /// Plan 401/VM-routing: the root widget's route table (`routes {}` block).
    /// `outlet` reads `__current_route` state and renders the matching page
    /// widget from this table. None = no routes (outlet renders empty).
    routes: Option<&'a [crate::aura::AuraRoute]>,

    /// EDGE-16 第五层(VM computed 求值):当前 widget 的 computed 属性表。
    /// resolve_expr_to_value / read_state_as_string_with 解析 `.foo` 时,先查
    /// 此表:命中则用 computed.expr 在当前 bindings 下求值(递归),未命中再
    /// 回退 state。None = 根 widget 在 build_with_debug_gated 时由调用方传入。
    computed: Option<&'a [crate::aura::AuraComputed]>,
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
            import_stmts: None,
            override_state_obj_id: None,
            routes: None,
            computed: None,
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
            import_stmts: None,
            override_state_obj_id: None,
            routes: None,
            computed: None,
        }
    }

    /// Create a builder with widget registry AND shared import declarations.
    /// Plan 318: child widgets (EditorPanel) need back.api functions to be
    /// available when their handlers are compiled; passing them here lets
    /// render_child_widget reuse the parent's loaded imports.
    pub fn with_registry_and_imports(
        bridge: &'a VmBridge,
        widget_name: &str,
        registry: &'a crate::ui::widget_registry::WidgetRegistry,
        import_stmts: &'a [crate::ast::Stmt],
    ) -> Self {
        Self {
            bridge,
            widget_name: widget_name.to_string(),
            widget_registry: Some(registry),
            import_stmts: Some(import_stmts),
            override_state_obj_id: None,
            routes: None,
            computed: None,
        }
    }

    /// Plan 401/VM-routing: attach the root widget's route table so `outlet`
    /// can resolve the current route to a page widget.
    pub fn with_routes(mut self, routes: &'a [crate::aura::AuraRoute]) -> Self {
        self.routes = Some(routes);
        self
    }

    /// EDGE-16 第五层:传入当前 widget 的 computed 属性表,供 resolve 时
    /// 求值 computed 引用(如 `.status_glyph`)。
    pub fn with_computed(mut self, computed: &'a [crate::aura::AuraComputed]) -> Self {
        self.computed = Some(computed);
        self
    }

    /// Build a `View<DynamicMessage>` from an AuraNode template.
    ///
    /// Recursively traverses the AuraNode tree, converting each node into the
    /// corresponding View variant. State references are resolved from the
    /// VmBridge at build time.
    pub fn build(&self, node: &AuraNode) -> View<DynamicMessage> {
        self.convert_node_with(node, &Bindings::new())
    }

    /// Plan 320: read a state field. When override_state_obj_id is set (rendering
    /// a child widget's view), reads from the child's state object. Otherwise
    /// reads from the root widget's state (legacy behavior).
    /// Plan 370 D-GAP-4: also handles "store.field" paths by stripping the
    /// "store." prefix (store fields are merged into root state as bare names).
    fn read_state(&self, field_name: &str) -> Result<auto_val::Value, String> {
        // Strip "store." prefix for merged store fields
        let name = field_name.strip_prefix("store.").unwrap_or(field_name);
        if let Some(child_id) = self.override_state_obj_id {
            // Try child state first, fall back to root state for store fields
            match self.bridge.read_child_state(child_id, name) {
                Ok(v) => Ok(v),
                Err(_) => self.bridge.read_state(name).map_err(|e| e.to_string()),
            }
        } else {
            self.bridge.read_state(name)
                .map_err(|e| e.to_string())
        }
    }

    /// Plan 320: read a state field as Vec<Value> (override-aware).
    /// Plan 370 D-GAP-4: also handles "store.field" paths by stripping the
    /// "store." prefix (store fields are merged into root state as bare names).
    fn read_state_as_vec(&self, field_name: &str) -> Result<Vec<auto_val::Value>, String> {
        let name = field_name.strip_prefix("store.").unwrap_or(field_name);
        if let Some(child_id) = self.override_state_obj_id {
            // Try child state first, fall back to root state for store fields
            match self.bridge.read_child_state_as_vec(child_id, name) {
                Ok(v) => Ok(v),
                Err(_) => self.bridge.read_state_as_vec(name).map_err(|e| e.to_string()),
            }
        } else {
            self.bridge.read_state_as_vec(name)
                .map_err(|e| e.to_string())
        }
    }

    /// Plan 370 (Issue 2): resolve a `for`-loop iterable that may be a
    /// dotted path like `.note.tags` or `.store.notes`. Simple field names
    /// (`.notes`) go through `read_state`/`read_state_as_vec`; dotted paths
    /// (`.note.tags`) are resolved field-by-field via `resolve_expr_to_value`
    /// so that a prop object's sub-fields (e.g. a Note's tags array stored
    /// as a heap-id Int) can be iterated.
    fn resolve_iterable(&self, iterable: &str, bindings: &Bindings) -> Option<Vec<auto_val::Value>> {
        // Simple state field (no interior dot after stripping the leading '.')
        let stripped = iterable.strip_prefix('.').unwrap_or(iterable);
        let has_inner_dot = stripped.contains('.') && !stripped.starts_with("store.");
        if !has_inner_dot {
            // Delegate to existing state-read helpers.
            if let Ok(arr) = self.read_state_as_vec(stripped) {
                return Some(arr);
            }
            return None;
        }
        // Dotted path — build an Expr and resolve. `.note.tags` →
        // Dot(Dot(Ident("."), "note"), "tags").
        let expr = Self::parse_dot_path_to_expr(iterable)?;
        let val = self.resolve_expr_to_value(&expr, bindings)?;
        match val {
            auto_val::Value::Array(arr) => Some(arr.iter().cloned().collect()),
            // Plan 390 §15 H3b: arrays are ListData<Value> in heap_objects (4M+).
            auto_val::Value::Int(id) if id >= 4_000_000 => {
                // Heap array id — deref via bridge
                Some(self.bridge.index_list_all(id as usize))
            }
            auto_val::Value::VmRef(r) => {
                Some(self.bridge.index_list_all(r.id))
            }
            _ => None,
        }
    }

    /// Build a chained Dot expr from a dotted path string like `.note.tags`.
    fn parse_dot_path_to_expr(path: &str) -> Option<Expr> {
        let parts: Vec<&str> = path.split('.').filter(|p| !p.is_empty()).collect();
        if parts.is_empty() {
            return None;
        }
        // First segment is the root (e.g. "" from leading '.', or "note").
        // For `.note.tags`, parts = ["note", "tags"].
        let mut expr = Expr::Ident(parts[0].into());
        for part in &parts[1..] {
            expr = Expr::Dot(Box::new(expr), part.to_string().into());
        }
        Some(expr)
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
                // Plan 370 (Issue 2): for dotted prop paths like `.note.tags`,
                // resolve via resolve_iterable (handles field-by-field deref).
                // For simple state fields, use read_state/read_state_as_vec.
                let stripped = iterable.strip_prefix('.').unwrap_or(iterable);
                let has_inner_dot = stripped.contains('.') && !stripped.starts_with("store.");
                let array: auto_val::Array = if has_inner_dot {
                    match self.resolve_iterable(iterable, bindings) {
                        Some(elems) => auto_val::Array::from(elems),
                        None => return View::Empty,
                    }
                } else {
                // Plan 046:裸标识符 iterable 可能是外层循环绑定的变量 ——
                // 先查 bindings(同 tracked 路径修复)。命中则解包成 Array。
                if let Some(val) = bindings.get(state_name).cloned() {
                    match val {
                        Value::Array(arr) => arr,
                        Value::Int(id) if id >= 4_000_000 => {
                            auto_val::Array::from(self.bridge.index_list_all(id as usize))
                        }
                        Value::VmRef(r) => {
                            auto_val::Array::from(self.bridge.index_list_all(r.id))
                        }
                        _ => return View::Empty,
                    }
                } else {
                // Read the iterable array from VmBridge state
                match self.read_state(state_name) {
                    Ok(Value::Array(arr)) => arr,
                    Ok(_other) => {
                        // Try read_state_as_vec for Value::Int(array_id) refs
                        match self.read_state_as_vec(state_name) {
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
                }
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
                            // Plan 370 (Issue 1): if the single body view is
                            // Empty (e.g. a false `if` inside the loop), skip
                            // it so the loop doesn't emit a text("") spacer.
                            let v = views.into_iter().next().unwrap();
                            if matches!(v, View::Empty) { None } else { Some(v) }
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
            AuraNode::Component { name, props, events, children, .. } => {
                // Plan 408: nav-link renders as a navigable button (like link).
                if name == "nav-link" || name == "nav_link" {
                    let prop_map: HashMap<String, AuraPropValue> = props.iter()
                        .map(|(k, v)| (k.clone(), AuraPropValue::Expr(v.clone())))
                        .collect();
                    let to = self.extract_string(&prop_map, "to")
                        .or_else(|| self.extract_string(&prop_map, "href"))
                        .unwrap_or_default();
                    let label = self.extract_string(&prop_map, "label")
                        .or_else(|| self.extract_string(&prop_map, "text"))
                        .unwrap_or_default();
                    let icon = self.extract_string(&prop_map, "icon").unwrap_or_default();
                    return self.render_link_button_with_icon(&label, &[], &to, &icon, bindings, false);
                }
                // Plan 410: category-section → column (recurse component-card
                // children). Vue codegen builds a fancy card grid; VM renders a
                // simple column so the home page's component list isn't blank.
                if name == "category-section" || name == "category_section" {
                    let child_views: Vec<View<DynamicMessage>> = children
                        .iter()
                        .filter_map(|n| {
                            let v = self.convert_node_with(n, bindings);
                            if matches!(v, View::Empty) { None } else { Some(v) }
                        })
                        .collect();
                    return View::Column { children: child_views, spacing: 0, padding: 0, style: None };
                }
                // Plan 410: component-card → navigable link button (to + name + desc).
                if name == "component-card" || name == "component_card" || name == "componentcard" {
                    let prop_map: HashMap<String, AuraPropValue> = props.iter()
                        .map(|(k, v)| (k.clone(), AuraPropValue::Expr(v.clone())))
                        .collect();
                    let to = self.extract_string(&prop_map, "to").unwrap_or_default();
                    let card_name = self.extract_string(&prop_map, "name").unwrap_or_default();
                    let desc = self.extract_string(&prop_map, "desc").unwrap_or_default();
                    let label = if desc.is_empty() { card_name } else { format!("{} — {}", card_name, desc) };
                    let icon = self.extract_string(&prop_map, "icon").unwrap_or_default();
                    return self.render_link_button_with_icon(&label, &[], &to, &icon, bindings, false);
                }
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
                // Plan 401/VM-routing: render the page widget matching the
                // current route (the iced equivalent of vue's <router-view>).
                self.render_outlet(bindings)
            }
            AuraNode::Link { text, children, to, .. } => {
                // Plan 401/VM-routing: render a link as a clickable button whose
                // onclick carries the target path as a __navigate message. The
                // update loop intercepts __navigate and sets __current_route.
                self.render_link_button(text, children, to, bindings)
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
                // Plan 370 (Issue 2): for dotted prop paths like `.note.tags`,
                // resolve via resolve_iterable (handles field-by-field deref).
                let stripped = iterable.strip_prefix('.').unwrap_or(iterable);
                let has_inner_dot = stripped.contains('.') && !stripped.starts_with("store.");
                let array: Vec<Value> = if has_inner_dot {
                    match self.resolve_iterable(iterable, bindings) {
                        Some(v) => v,
                        None => return View::Empty,
                    }
                } else {
                // Plan 046:裸标识符 iterable(如内层 for 的 `row`)可能是外层循环
                // 绑定的变量 —— 先查 bindings,命中则解包成 Vec<Value>(同 resolve_iterable
                // :253-263 的 match 逻辑)。未命中再 fallback 到 read_state_as_vec。
                if let Some(val) = bindings.get(state_name).cloned() {
                    match val {
                        auto_val::Value::Array(arr) => arr.iter().cloned().collect(),
                        auto_val::Value::Int(id) if id >= 4_000_000 => {
                            self.bridge.index_list_all(id as usize)
                        }
                        auto_val::Value::VmRef(r) => {
                            self.bridge.index_list_all(r.id)
                        }
                        _ => {
                            log::warn!("view_builder: bindings['{}'] is not iterable: {:?}", state_name, val);
                            return View::Empty;
                        }
                    }
                } else {
                // Read the iterable. `read_state_as_vec` handles BOTH an inline
                // `Value::Array` and a `Value::Int(array_id)` heap-array reference
                // (the latter is how `var x = []; x.push(...)` arrays are stored —
                // e.g. 016-calendar's `.days`). A bare `read_state` + `Value::Array`
                // match misses the heap-id form and silently renders an empty loop.
                match self.read_state_as_vec(state_name) {
                    Ok(v) => v,
                    Err(e) => {
                        log::warn!("view_builder: read_state_as_vec('{}') failed: {}", state_name, e);
                        return View::Empty;
                    }
                }
                }
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
                        else if views.len() == 1 {
                            // Plan 370 (Issue 1): skip Empty body views (see convert_node_with ForLoop).
                            let v = views.into_iter().next().unwrap();
                            if matches!(v, View::Empty) { None } else { Some(v) }
                        }
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
            AuraNode::Component { name, props, events, children, .. } => {
                // Plan 408: nav-link renders as a navigable button (like link).
                if name == "nav-link" || name == "nav_link" {
                    let prop_map: HashMap<String, AuraPropValue> = props.iter()
                        .map(|(k, v)| (k.clone(), AuraPropValue::Expr(v.clone())))
                        .collect();
                    let to = self.extract_string(&prop_map, "to")
                        .or_else(|| self.extract_string(&prop_map, "href"))
                        .unwrap_or_default();
                    let label = self.extract_string(&prop_map, "label")
                        .or_else(|| self.extract_string(&prop_map, "text"))
                        .unwrap_or_default();
                    let icon = self.extract_string(&prop_map, "icon").unwrap_or_default();
                    return self.render_link_button_with_icon(&label, &[], &to, &icon, bindings, false);
                }
                // Plan 410: category-section → column (recurse component-card
                // children). Vue codegen builds a fancy card grid; VM renders a
                // simple column so the home page's component list isn't blank.
                if name == "category-section" || name == "category_section" {
                    let child_views: Vec<View<DynamicMessage>> = children
                        .iter()
                        .filter_map(|n| {
                            let v = self.convert_node_with(n, bindings);
                            if matches!(v, View::Empty) { None } else { Some(v) }
                        })
                        .collect();
                    return View::Column { children: child_views, spacing: 0, padding: 0, style: None };
                }
                // Plan 410: component-card → navigable link button (to + name + desc).
                if name == "component-card" || name == "component_card" || name == "componentcard" {
                    let prop_map: HashMap<String, AuraPropValue> = props.iter()
                        .map(|(k, v)| (k.clone(), AuraPropValue::Expr(v.clone())))
                        .collect();
                    let to = self.extract_string(&prop_map, "to").unwrap_or_default();
                    let card_name = self.extract_string(&prop_map, "name").unwrap_or_default();
                    let desc = self.extract_string(&prop_map, "desc").unwrap_or_default();
                    let label = if desc.is_empty() { card_name } else { format!("{} — {}", card_name, desc) };
                    let icon = self.extract_string(&prop_map, "icon").unwrap_or_default();
                    return self.render_link_button_with_icon(&label, &[], &to, &icon, bindings, false);
                }
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
                // Plan 401/VM-routing: render the page widget matching the
                // current route (the iced equivalent of vue's <router-view>).
                self.render_outlet(bindings)
            }
            AuraNode::Link { text, children, to, .. } => {
                // Plan 401/VM-routing: render link as a clickable button (same
                // as the untracked path); tracked children are flattened into
                // the button label.
                let _ = (path, id_map, probe);
                self.render_link_button(text, children, to, bindings)
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
                self.convert_text_element_tracked_ctx(tag, props, events, children, path, probe, bindings)
            }

            // Leaf/atom widgets with no AuraNode children — fall back to the
            // untracked converter. They have no nested text to probe (Task 9
            // scope is text interpolation only).
            "button" | "btn" => self.convert_button(props, events, children, bindings),
            "input" => self.convert_input(props, events, bindings),
            "textarea" => self.convert_textarea(props, events, bindings),
            // Plan 370 D-GAP-3: AutoDownEditor → textarea (plain-text degradation)
            "autodown_editor" | "autodowneditor" | "autodown" | "markdown_editor" => {
                self.convert_textarea(props, events, bindings)
            }
            "checkbox" | "check" => self.convert_checkbox(props, events, bindings),
            "img" | "image" | "icon" => self.convert_image_or_icon(props),
            "progress" => self.convert_progress(props),
            "spacer" => self.convert_spacer(props),
            "divider" | "hr" => self.convert_divider(props),
            "avatar" => self.convert_avatar(props),

            // Child widget lookup or fallback.
            _ => {
                // Plan 408: nav-link renders as a navigable button (like link).
                if tag == "nav-link" || tag == "nav_link" {
                    let to = self.extract_string(props, "to")
                        .or_else(|| self.extract_string(props, "href"))
                        .unwrap_or_default();
                    let label = self.extract_string(props, "label")
                        .or_else(|| self.extract_string(props, "text"))
                        .unwrap_or_default();
                    let icon = self.extract_string(props, "icon").unwrap_or_default();
                    return self.render_link_button_with_icon(&label, &[], &to, &icon, bindings, false);
                }
                if let Some(registry) = self.widget_registry {
                    if let Some(child_widget) = registry.get(tag) {
                        return self.render_child_widget(child_widget, props, events, bindings);
                    }
                }
                // Plan 410: category-section → column container (recurse its
                // component-card children). Vue codegen builds a fancy card grid;
                // VM renders a simple column so the home page's component list
                // isn't blank (View::Empty).
                if tag == "category-section" || tag == "category_section" {
                    return self.convert_column_tracked_ctx(props, children, path, id_map, probe, bindings);
                }
                // Plan 410: component-card → navigable link button (to + name).
                if tag == "component-card" || tag == "component_card" || tag == "componentcard" {
                    let to = self.extract_string(props, "to").unwrap_or_default();
                    let name = self.extract_string(props, "name").unwrap_or_default();
                    let desc = self.extract_string(props, "desc").unwrap_or_default();
                    let label = if desc.is_empty() { name } else { format!("{} — {}", name, desc) };
                    let icon = self.extract_string(props, "icon").unwrap_or_default();
                    return self.render_link_button_with_icon(&label, &[], &to, &icon, bindings, false);
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
            // Plan 370 (Issue 1): drop visually-empty spacers (see convert_column).
            .filter(|v| !is_visually_empty(v))
            .collect();

        let mut builder = View::<DynamicMessage>::col()
            .spacing(spacing)
            .padding(padding);
        // Plan 048:提取 overflow 标志 + style clone(builder.with_style 会 move style)。
        // style_clone 传给 Scrollable,让 build_scrollable 读到 flex-1 → height(Fill)。
        let needs_scroll = style.as_ref().map_or(false, |s| {
            s.classes.iter().any(|c| matches!(c, crate::ui::style::StyleClass::OverflowYAuto | crate::ui::style::StyleClass::OverflowAuto))
        });
        let scroll_style = if needs_scroll { style.clone() } else { None };
        if let Some(s) = style {
            builder = builder.with_style(s);
        }
        for child in child_views {
            builder = builder.child(child);
        }
        let col_view = builder.build();
        // Plan 048:overflow-y-auto / overflow-auto → Scrollable。
        if needs_scroll {
            return View::Scrollable {
                child: Box::new(col_view),
                width: None, height: None, style: scroll_style,
            };
        }
        col_view
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
                    // Plan 046:裸标识符 iterable 可能是外层循环变量(嵌套 for),
                    // 先查 bindings(同主 ForLoop 修复)。未命中再 fallback 到 state。
                    let array: Vec<Value> = if let Some(val) = bindings.get(state_name).cloned() {
                        match val {
                            Value::Array(arr) => arr.iter().cloned().collect(),
                            Value::Int(id) if id >= 4_000_000 => self.bridge.index_list_all(id as usize),
                            Value::VmRef(r) => self.bridge.index_list_all(r.id),
                            _ => continue,
                        }
                    } else {
                    // Use read_state_as_vec so heap-array refs (Value::Int(array_id),
                    // the form `var x = []; x.push(...)` produces — e.g. .days) are
                    // iterated, not just inline Value::Array. Otherwise the grid's
                    // `for cell in .days` renders empty even though state is populated.
                    match self.read_state_as_vec(state_name) {
                        Ok(v) => v,
                        _ => continue,
                    }
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
            match n {
                AuraNode::Conditional { condition, then_body, else_body, .. } => {
                    let is_true = self.eval_condition_with(condition, bindings);
                    let empty = Vec::new();
                    let body = if is_true { then_body } else { else_body.as_ref().unwrap_or(&empty) };
                    for child_node in body {
                        path.push(i);
                        let v = self.convert_node_tracked_ctx(child_node, path, id_map, probe, bindings);
                        path.pop();
                        child_views.push(v);
                    }
                }
                AuraNode::ForLoop { var, index, iterable, body, .. } => {
                    // Plan 047:flatten ForLoop into row(同 untracked convert_row)。
                    // 用 for_loop_iterations(放弃 row 内 ForLoop 的 probe 追踪,
                    // inspector 调试信息不影响渲染正确性)。
                    child_views.extend(self.for_loop_iterations(var, index, iterable, body, bindings));
                }
                _ => {
                    path.push(i);
                    let v = self.convert_node_tracked_ctx(n, path, id_map, probe, bindings);
                    path.pop();
                    child_views.push(v);
                }
            }
        }

        let mut builder = View::<DynamicMessage>::row()
            .spacing(spacing)
            .padding(padding);
        if let Some(s) = style {
            builder = builder.with_style(s);
        }
        for child in child_views {
            // Plan 370 (Issue 1): drop View::Empty spacers (see convert_column).
            if is_visually_empty(&child) {
                continue;
            }
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
        events: &HashMap<String, AuraEvent>,
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

        // Apply default heading styles, merging with user-provided styles.
        // Plan 409 §8: headings carry the theme color (text-primary) so page
        // titles / section headers use the accent — "主要操作和显眼的内容".
        if matches!(tag, "h1" | "h2" | "h3") {
            let default = match tag {
                "h1" => Style::parse("text-4xl font-bold text-primary").ok(),
                "h2" => Style::parse("text-3xl font-bold text-primary").ok(),
                "h3" => Style::parse("text-xl font-semibold text-primary").ok(),
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
        //
        // If this text element has an onclick/click event, render it as a
        // Button so the click handler fires (View::Text has no onclick field).
        if let Some(event) = aura_events_get_base(events, "onclick")
            .or_else(|| aura_events_get_base(events, "click")) {
            let onclick = self.event_to_message_with(event, bindings);
            return View::Button {
                label: content,
                onclick,
                style,
                on_right_click: None,
                content: None,
            };
        }

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
                self.convert_text_element(tag, props, events, children, bindings)
            }
            "button" | "btn" => self.convert_button(props, events, children, bindings),

            // Layout wrappers
            "center" => self.convert_center(props, children, bindings),

            // Input widgets
            "input" => self.convert_input(props, events, bindings),
            "textarea" => self.convert_textarea(props, events, bindings),
            // Plan 370 D-GAP-3: AutoDownEditor → textarea (plain-text degradation)
            "autodown_editor" | "autodowneditor" | "autodown" | "markdown_editor" => {
                self.convert_textarea(props, events, bindings)
            }
            "checkbox" | "check" => self.convert_checkbox(props, events, bindings),
            "container" | "div" => self.convert_container(props, children, bindings),

            // Image / Icon
            "img" | "image" | "icon" => self.convert_image_or_icon(props),

            // Utility widgets
            "progress" => self.convert_progress(props),
            "spacer" => self.convert_spacer(props),
            "divider" | "hr" => self.convert_divider(props),
            "avatar" => self.convert_avatar(props),

            // Child widget lookup or fallback
            _ => {
                // Plan 408: nav-link renders as a navigable button (like link).
                // nav-link is parsed as an Element (not Component), so handle it
                // here in the fallback before child-widget lookup. Extract the
                // `to`/`href` prop for navigation and `label`/`text` for display.
                if tag == "nav-link" || tag == "nav_link" {
                    let to = self.extract_string(props, "to")
                        .or_else(|| self.extract_string(props, "href"))
                        .unwrap_or_default();
                    let label = self.extract_string(props, "label")
                        .or_else(|| self.extract_string(props, "text"))
                        .unwrap_or_default();
                    let icon = self.extract_string(props, "icon").unwrap_or_default();
                    return self.render_link_button_with_icon(&label, &[], &to, &icon, bindings, false);
                }
                // Plan 410: category-section → column (recurse component-card
                // children). Vue codegen builds a fancy card grid; VM renders a
                // simple column so the home page's component list isn't blank.
                if tag == "category-section" || tag == "category_section" {
                    let child_views: Vec<View<DynamicMessage>> = children
                        .iter()
                        .map(|n| self.convert_node_with(n, bindings))
                        .filter(|v| !matches!(v, View::Empty))
                        .collect();
                    return View::Column { children: child_views, spacing: 0, padding: 0, style: None };
                }
                // Plan 409 §10 组 E: preview-card / codeblock VM 识别。vue codegen
                // 对它们做特殊处理(generate_previewcard_html / generate_codeblock_html
                // 生成预览区 + code toggle + Auto/Vue tabs + copy);VM 无这些命令式
                // 能力,简化:preview-card → Column(递归 children,即真正的预览 UI),
                // codeblock → Text(代码内容;否则空 children 落到 Empty,安装命令整
                // 段消失)。
                if tag == "preview-card" || tag == "preview_card" || tag == "previewcard" {
                    let child_views: Vec<View<DynamicMessage>> = children
                        .iter()
                        .map(|n| self.convert_node_with(n, bindings))
                        .filter(|v| !matches!(v, View::Empty))
                        .collect();
                    return View::Column { children: child_views, spacing: 0, padding: 0, style: None };
                }
                if tag == "codeblock" || tag == "code_block" || tag == "code-block" {
                    // 内容优先级对齐 vue codegen(vue.rs ~4462):code → text → children text.
                    let code = self.extract_string(props, "code")
                        .or_else(|| self.extract_string(props, "text"))
                        .or_else(|| self.extract_children_text(children, bindings))
                        .unwrap_or_default();
                    let lang = self.extract_string(props, "lang").unwrap_or_default();
                    let content = if lang.is_empty() { code } else { format!("{}: {}", lang, code) };
                    return View::Text { content, style: None };
                }
                // Plan 410: component-card → navigable link button (to + name + desc).
                if tag == "component-card" || tag == "component_card" || tag == "componentcard" {
                    let to = self.extract_string(props, "to").unwrap_or_default();
                    let name = self.extract_string(props, "name").unwrap_or_default();
                    let desc = self.extract_string(props, "desc").unwrap_or_default();
                    let label = if desc.is_empty() { name } else { format!("{} — {}", name, desc) };
                    let icon = self.extract_string(props, "icon").unwrap_or_default();
                    return self.render_link_button_with_icon(&label, &[], &to, &icon, bindings, false);
                }
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

    /// Plan 408: Map common lucide icon names to emoji for vm/iced rendering.
    /// iced has no icon font; this gives nav-link items visual distinction.
    fn icon_to_emoji(icon: &str) -> Option<&'static str> {
        match icon {
            "bell" => Some("\u{1F514}"),           // 🔔
            "command" => Some("\u{2318}"),         // ⌘
            "image" => Some("\u{1F5BC}"),          // 🖼️
            "layout-grid" => Some("\u{25A6}"),     // ▦
            "menu" => Some("\u{2630}"),            // ☰
            "mouse-pointer-click" => Some("\u{1F5B1}"), // 🖱️
            "navigation" => Some("\u{1F9ED}"),     // 🧭
            "search" => Some("\u{1F50D}"),         // 🔍
            "square-stack" => Some("\u{1F4E6}"),   // 📦
            "type" => Some("\u{1F4DD}"),           // 📝
            "home" => Some("\u{1F3E0}"),           // 🏠
            "settings" => Some("\u{2699}"),        // ⚙️
            "layers" => Some("\u{1F5C2}"),         // 🗂️
            "arrow-right" => Some("\u{27A1}"),     // ➡️
            "folder" => Some("\u{1F4C1}"),         // 📁
            "mail" => Some("\u{1F4E7}"),           // 📧
            "book" => Some("\u{1F4D6}"),           // 📖
            "github" => Some("\u{1F47D}"),         // 👽 (no github emoji)
            "check-square" => Some("\u{2705}"),    // ✅
            _ => None,
        }
    }

    /// Plan 401/VM-routing: render a `link (to: "/path")` as a clickable
    /// button. The onclick carries a `__navigate` DynamicMessage with the target
    /// path as its sole arg; the update loop intercepts `__navigate` and sets
    /// `__current_route`, causing `outlet` to re-render the new page. The label
    /// comes from the link's text or its first text child.
    /// Plan 408: optional icon name is mapped to emoji and prepended to label.
    fn render_link_button(&self, text: &str, children: &[crate::aura::AuraNode], to: &str, _bindings: &Bindings) -> View<DynamicMessage> {
        // Plan 409 §10 组 C: header link（AuraNode::Link，如 Docs/Widgets）走这里，
        // 用主题色 text-primary。nav-link/component-card 不走这里（它们直接调
        // render_link_button_with_icon 并传 themed=false）。
        self.render_link_button_with_icon(text, children, to, "", _bindings, true)
    }

    /// Plan 408: render_link_button with an optional icon name.
    ///
    /// Plan 409 §6: when the link has child nodes, convert them into a content
    /// container rendered *inside* the button — vue parity for
    /// `link (to:) { text / row / icon ... }`. The `label` is still derived
    /// from the children (for the snapshot builder / accessibility), but the
    /// rendered content is the container, not a flattened `to` string.
    fn render_link_button_with_icon(&self, text: &str, children: &[crate::aura::AuraNode], to: &str, icon: &str, bindings: &Bindings, themed: bool) -> View<DynamicMessage> {
        // Plan 409 §6: convert child nodes into a content subtree. When
        // non-empty, the button renders this container instead of the label
        // string (link is no longer a leaf).
        let content: Option<Box<View<DynamicMessage>>> = if children.is_empty() {
            None
        } else {
            let views: Vec<View<DynamicMessage>> = children
                .iter()
                .map(|n| self.convert_node_with(n, bindings))
                .filter(|v| !matches!(v, View::Empty))
                .collect();
            if views.is_empty() {
                None
            } else if views.len() == 1 {
                Some(Box::new(views.into_iter().next().unwrap()))
            } else {
                Some(Box::new(View::Row {
                    children: views,
                    spacing: 0,
                    padding: 0,
                    style: None,
                }))
            }
        };

        // Resolve a display label: prefer explicit text, else the children's
        // text (recursively — so a `text (text: "Docs")` child yields "Docs",
        // not the raw path), else fall back to the path itself.
        let mut label = if !text.is_empty() {
            text.to_string()
        } else if let Some(derived) = self.extract_children_text(children, bindings) {
            derived
        } else {
            to.to_string()
        };
        // Plan 408: embed icon name in label using PUA markers. Only applies to
        // the label fallback (nav-link passes empty children → content is
        // None); when a content container is present the icon belongs to the
        // children subtree instead.
        if !icon.is_empty() && content.is_none() {
            label = format!("\u{EE01}{}\u{EE02}{}", icon, label);
        }
        View::Button {
            label,
            content,
            onclick: crate::ui::interpreter::DynamicMessage::Typed {
                widget_name: self.widget_name.clone(),
                event_name: "__navigate".to_string(),
                args: vec![auto_val::Value::str(to)],
            },
            // Plan 409 §10 组 C: themed=true → text-primary（header Docs/Widgets
            // link，走 AuraNode::Link，与 vue router-link 默认 text-primary 一致）；
            // themed=false → None（nav-link/component-card 普通色，§8.3 与 sidebar
            // 一致）。Button 无显式 text_color 时 renderer 用 OnBackground 默认色。
            style: if themed { Style::parse("text-primary").ok() } else { None },
            on_right_click: None,
        }
    }

    /// Plan 401/VM-routing: render the page widget for the current route.
    ///
    /// Reads `__current_route` (e.g. "/book/1") from VM state, matches it
    /// against the routes table (honouring `:param` segments), and renders the
    /// matching page widget via `render_child_widget` — the iced equivalent of
    /// vue's `<router-view>`. Dynamic-segment values are written into the
    /// `__route_params` state object so page handlers can read them via
    /// `router.param("id")`. No routes / no match / empty route → `View::Empty`.
    fn render_outlet(&self, bindings: &Bindings) -> View<DynamicMessage> {
        let (Some(registry), Some(routes)) = (self.widget_registry, self.routes) else {
            return View::Empty;
        };

        // Current route path. Default to the first route (the index "/") when
        // unset, so the app boots into its home page without an explicit init.
        let current = match self.read_state("__current_route") {
            Ok(auto_val::Value::Str(s)) if !s.is_empty() => s.to_string(),
            _ => routes.first().map(|r| r.path.clone()).unwrap_or_default(),
        };

        // Match the current path against each route pattern. A pattern segment
        // starting with ':' matches any single path segment and is captured as
        // a route param (e.g. "/book/:id" matches "/book/3" → id=3).
        let cur_segs: Vec<&str> = current.trim_matches('/').split('/').filter(|s| !s.is_empty()).collect();
        for route in routes {
            let pat_segs: Vec<&str> = route.path.trim_matches('/').split('/').filter(|s| !s.is_empty()).collect();
            if pat_segs.len() != cur_segs.len() {
                continue;
            }
            let mut params: std::collections::HashMap<String, String> = std::collections::HashMap::new();
            let mut matched = true;
            for (pat, cur) in pat_segs.iter().zip(cur_segs.iter()) {
                if let Some(param_name) = pat.strip_prefix(':') {
                    params.insert(param_name.to_string(), cur.to_string());
                } else if *pat != *cur {
                    matched = false;
                    break;
                }
            }
            if !matched {
                continue;
            }
            // Found the route. (Param persistence into __route_params is handled
            // by the navigation codegen — render time is read-only here.)
            let _ = params;
            // Plan 408: look up the page widget via the route module alias map
            // (route.module → widget.name, e.g. "button" → "ButtonPage").
            // Using the alias map instead of registry.get avoids shadowing
            // built-in UI elements like <button>, <input>.
            if let Some(page_widget) = registry.get_by_route_module(&route.module) {
                let empty_props: HashMap<String, AuraPropValue> = HashMap::new();
                let empty_events: HashMap<String, AuraEvent> = HashMap::new();
                return self.render_child_widget(page_widget, &empty_props, &empty_events, bindings);
            }
            // Page widget not registered → show a textual placeholder so the
            // gap is visible (e.g. "page book_detail not loaded").
            return View::Text {
                content: format!("<outlet: page {} not loaded>", route.module),
                style: None,
            };
        }
        // No route matched the current path.
        View::Text {
            content: format!("<outlet: no route for {}>", current),
            style: None,
        }
    }

    /// Render a child widget by looking it up in the registry.
    ///
    /// This resolves props from parent state, injects them as state fields,
    /// creates a child VmBridge, and recursively renders the child's view tree.
    /// Plan 320: render a child widget WITHOUT creating a new VM.
    /// Uses the same VmBridge (same VM), creates/updates the child's state
    /// object on the heap, and renders the child's view tree with an
    /// override_state_obj_id so read_state reads from the child's state.
    fn render_child_widget(
        &self,
        child_widget: &crate::aura::AuraWidget,
        props: &HashMap<String, AuraPropValue>,
        _events: &HashMap<String, AuraEvent>,
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        // 1. Resolve prop values from parent state.
        let mut resolved_props: HashMap<String, Value> = HashMap::new();
        for (prop_name, prop_value) in props {
            if let AuraPropValue::Expr(expr) = prop_value {
                if let Some(val) = self.resolve_expr_to_value(expr, bindings) {
                    resolved_props.insert(prop_name.clone(), val);
                }
            }
        }

        // 2. Collect child state field names (model vars + injected props).
        let mut child_field_names: Vec<String> = child_widget.state_vars
            .iter()
            .map(|v| v.name.clone())
            .collect();
        for prop_name in resolved_props.keys() {
            if !child_field_names.contains(prop_name) {
                child_field_names.push(prop_name.clone());
            }
        }

        // 3. Also sync matching parent state → child (editing, edit_title, etc.)
        //    so child handlers can read/write shared parent state.
        for field_name in &child_field_names {
            if !resolved_props.contains_key(field_name) {
                if let Ok(parent_val) = self.bridge.read_state(field_name) {
                    resolved_props.insert(field_name.clone(), parent_val);
                }
            }
        }

        // 4. Seed child model-var defaults (e.g. `var collapsed bool = false`)
        //    that the parent didn't pass. Plan 049: without this, a child view
        //    that reads its own model var (`if !.collapsed`, a computed like
        //    `collapse_glyph`) hits read_state Err on first render → the
        //    condition short-circuits to false (body hidden) and the string
        //    falls back to "${collapsed}" / "${collapse_glyph}".
        //    Only runs when the parent didn't provide the prop (parent wins).
        for state_var in &child_widget.state_vars {
            if !resolved_props.contains_key(&state_var.name) {
                resolved_props.insert(
                    state_var.name.clone(),
                    eval_initial_without_vm(&state_var.initial),
                );
            }
        }

        // 5. Ensure child state object exists on the VM heap + write props.
        let child_state_id = self.bridge.ensure_child_state(
            &child_widget.name,
            &child_field_names,
            &resolved_props,
        );

        // 5. Build a child view builder using the SAME bridge but with
        //    override_state_obj_id pointing to the child's state object.
        let child_builder = AuraViewBuilder {
            bridge: self.bridge,
            widget_name: child_widget.name.clone(),
            widget_registry: self.widget_registry,
            import_stmts: self.import_stmts,
            override_state_obj_id: Some(child_state_id),
            routes: None,
            computed: Some(&child_widget.computed),
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
            // Plan 370 (Issue 1): drop visually-empty children (View::Empty
            // or View::Text with blank content). False `if` branches and
            // non-matching `for` iterations yield these, and the renderer
            // turns them into text("") — a one-line-tall spacer. Stacking
            // several produced large blank gaps in the NavTree.
            .filter(|v| !is_visually_empty(v))
            .collect();

        let mut builder = View::<DynamicMessage>::col()
            .spacing(spacing)
            .padding(padding);

        // Plan 048:提取 overflow 标志 + style clone(builder.with_style 会 move style)。
        let needs_scroll = style.as_ref().map_or(false, |s| {
            s.classes.iter().any(|c| matches!(c, crate::ui::style::StyleClass::OverflowYAuto | crate::ui::style::StyleClass::OverflowAuto))
        });
        let scroll_style = if needs_scroll { style.clone() } else { None };

        if let Some(s) = style {
            builder = builder.with_style(s);
        }

        for child in child_views {
            builder = builder.child(child);
        }

        let col_view = builder.build();
        // Plan 048:overflow-y-auto / overflow-auto → Scrollable。
        if needs_scroll {
            return View::Scrollable {
                child: Box::new(col_view),
                width: None, height: None, style: scroll_style,
            };
        }
        col_view
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
        // should be spread horizontally, not wrapped in a Column.
        // Plan 047:also flatten ForLoop children(for cell in row → cells 横向排列,
        // 而非被 ForLoop 包成单个 Column 导致纵向堆叠)。
        let mut child_views: Vec<View<DynamicMessage>> = Vec::new();
        for n in children {
            match n {
                AuraNode::Conditional { condition, then_body, else_body, .. } => {
                    let is_true = self.eval_condition_with(condition, bindings);
                    let empty = Vec::new();
                    let body = if is_true { then_body } else { else_body.as_ref().unwrap_or(&empty) };
                    for child_node in body {
                        child_views.push(self.convert_node_with(child_node, bindings));
                    }
                }
                AuraNode::ForLoop { var, index, iterable, body, .. } => {
                    // 同 convert_grid(:1675):用 for_loop_iterations spread 进 row
                    child_views.extend(self.for_loop_iterations(var, index, iterable, body, bindings));
                }
                _ => {
                    child_views.push(self.convert_node_with(n, bindings));
                }
            }
        }

        let mut builder = View::<DynamicMessage>::row()
            .spacing(spacing)
            .padding(padding);

        if let Some(s) = style {
            builder = builder.with_style(s);
        }

        for child in child_views {
            // Plan 370 (Issue 1): drop View::Empty spacers (see convert_column).
            if is_visually_empty(&child) {
                continue;
            }
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
        // Plan 046:裸标识符 iterable 可能是外层循环变量(嵌套 for),
        // 先查 bindings。未命中再 fallback 到 read_state。
        let array = if let Some(val) = bindings.get(state_name).cloned() {
            match val {
                Value::Array(arr) => arr,
                Value::Int(id) if id >= 4_000_000 => {
                    auto_val::Array::from(self.bridge.index_list_all(id as usize))
                }
                Value::VmRef(r) => {
                    auto_val::Array::from(self.bridge.index_list_all(r.id))
                }
                _ => return Vec::new(),
            }
        } else {
        match self.read_state(state_name) {
            Ok(Value::Array(arr)) => arr,
            Ok(_) => match self.read_state_as_vec(state_name) {
                Ok(vec) => auto_val::Array::from(vec),
                Err(_) => return Vec::new(),
            },
            Err(_) => return Vec::new(),
        }
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
                    // Plan 370 (Issue 1): skip Empty body views (see convert_node_with ForLoop).
                    let v = views.into_iter().next().unwrap();
                    if matches!(v, View::Empty) { None } else { Some(v) }
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
    /// Convert an image or icon element.
    /// Plan 408: `icon (name: "bell")` uses a `lucide:{name}` synthetic src
    /// that the iced renderer resolves to a bundled SVG glyph.
    fn convert_image_or_icon(
        &self,
        props: &HashMap<String, AuraPropValue>,
    ) -> View<DynamicMessage> {
        let style = self.extract_style(props);
        // icon: name → "lucide:{name}" synthetic src
        if let Some(name) = self.extract_string(props, "name") {
            if !name.is_empty() {
                return View::Image { src: format!("lucide:{}", name), style };
            }
        }
        // image: src as-is
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
        events: &HashMap<String, AuraEvent>,
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

        // Apply default heading styles, merging with user-provided styles.
        // Plan 409 §8: headings carry the theme color (text-primary) so page
        // titles / section headers use the accent — "主要操作和显眼的内容".
        if matches!(tag, "h1" | "h2" | "h3") {
            let default = match tag {
                "h1" => Style::parse("text-4xl font-bold text-primary").ok(),
                "h2" => Style::parse("text-3xl font-bold text-primary").ok(),
                "h3" => Style::parse("text-xl font-semibold text-primary").ok(),
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

        // If this text element has an onclick/click event, render it as a
        // Button so the click handler fires (View::Text has no onclick field).
        // The Button renderer applies chromeless styling when no bg/border is
        // present, so the visual appearance matches plain text.
        if let Some(event) = aura_events_get_base(events, "onclick")
            .or_else(|| aura_events_get_base(events, "click")) {
            let onclick = self.event_to_message_with(event, bindings);
            return View::Button {
                label: styled_content,
                onclick,
                style,
                on_right_click: None,
                content: None,
            };
        }

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
        children: &[AuraNode],
        bindings: &Bindings,
    ) -> View<DynamicMessage> {
        // Plan 409 §10 组 A: icon-only buttons (header hamburger / search /
        // theme) previously fell back to the default "Button" text because the
        // `icon` prop was ignored. When an `icon` is present, embed it in the
        // label via PUA markers (same scheme nav-link uses, §2.3) and only use
        // a real text/label/child text if provided — so `button (icon:"menu")`
        // renders just the icon, not "Button".
        let icon = self.extract_string_with(props, "icon", bindings).unwrap_or_default();
        let label = if !icon.is_empty() {
            let text_part = self.extract_string_with(props, "text", bindings)
                .or_else(|| self.extract_string_with(props, "label", bindings))
                .or_else(|| self.extract_children_text(children, bindings))
                .unwrap_or_default();
            format!("\u{EE01}{}\u{EE02}{}", icon, text_part)
        } else {
            self.extract_string_with(props, "text", bindings)
                .or_else(|| self.extract_string_with(props, "label", bindings))
                .or_else(|| self.extract_children_text(children, bindings))
                .unwrap_or_else(|| "Button".to_string())
        };

        // `variant` selects a base style preset (Tailwind classes); the user's
        // class/style augments it. "text"/absent = chromeless (renders as text
        // via the renderer's class-driven style); "primary" = theme-colored
        // filled button (Plan 409 §8: theme-aware instead of hardcoded blue).
        let variant = self.extract_string_with(props, "variant", bindings)
            .unwrap_or_default();
        let preset: &str = match variant.as_str() {
            "primary" => "bg-primary text-primary-foreground font-medium rounded",
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

        // Resolve the onclick event handler to a DynamicMessage.
        // Base-aware lookup: event keys may carry modifiers (onclick.self).
        let onclick = aura_events_get_base(events, "onclick")
            .or_else(|| aura_events_get_base(events, "click"))
            .map(|event| self.event_to_message_with(event, bindings))
            .unwrap_or_else(|| DynamicMessage::String("click".to_string()));

        // Plan 402: resolve oncontextmenu (right-click) handler for flagging
        let on_right_click = aura_events_get_base(events, "oncontextmenu")
            .or_else(|| aura_events_get_base(events, "contextmenu"))
            .map(|event| self.event_to_message_with(event, bindings));

        View::Button {
            label,
            onclick,
            style,
            on_right_click,
            content: None,
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

        let on_change = aura_events_get_base(events, "onchange")
            .or_else(|| aura_events_get_base(events, "change"))
            .or_else(|| aura_events_get_base(events, "oninput"))
            .or_else(|| aura_events_get_base(events, "input"))
            .map(|event| self.event_to_message(&event.handler));

        let on_submit = aura_events_get_base(events, "onenter")
            .or_else(|| aura_events_get_base(events, "enter"))
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

        // Plan 370: autodown_editor uses `content:` for its text; standard
        // inputs use `value:`. Accept either so editor bodies render.
        let value = self.extract_string_with(props, "value", bindings)
            .or_else(|| self.extract_string_with(props, "content", bindings))
            .unwrap_or_default();

        let style = self.extract_style(props);
        let height = self.extract_u16(props, "height");

        let on_change = aura_events_get_base(events, "onchange")
            .or_else(|| aura_events_get_base(events, "change"))
            .or_else(|| aura_events_get_base(events, "oninput"))
            .or_else(|| aura_events_get_base(events, "input"))
            .or_else(|| aura_events_get_base(events, "onupdate"))
            .or_else(|| aura_events_get_base(events, "update"))
            .map(|event| self.event_to_message(&event.handler));

        // Plan 053 M4: textarea Enter → on_submit (mirrors convert_input).
        let on_submit = aura_events_get_base(events, "onenter")
            .or_else(|| aura_events_get_base(events, "enter"))
            .map(|event| self.event_to_message(&event.handler));

        let mut builder = View::<DynamicMessage>::textarea(placeholder).value(value);
        if let Some(msg) = on_change {
            builder = builder.on_change(msg);
        }
        if let Some(msg) = on_submit {
            builder = builder.on_submit(msg);
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

        let on_toggle = aura_events_get_base(events, "onclick")
            .or_else(|| aura_events_get_base(events, "change"))
            .or_else(|| aura_events_get_base(events, "onchange"))
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

    /// Read a state field value as a display string, checking loop bindings first.
    fn read_state_as_string_with(&self, field_name: &str, bindings: &Bindings) -> String {
        // Check loop bindings first (e.g., "note" in `for note in .notes`)
        if let Some(val) = bindings.get(field_name) {
            return value_to_display_string(val);
        }
        // EDGE-16 第五层:查 computed(如 .status_glyph),命中则求值。
        if let Some(val) = self.eval_computed(field_name, bindings) {
            return value_to_display_string(&val);
        }
        match self.read_state(field_name) {
            Ok(value) => value_to_display_string(&value),
            Err(_) => format!("${{{}}}", field_name),
        }
    }

    /// Resolve a base AST `Expr` to a display string with loop variable bindings.
    fn resolve_expr_to_string_with(&self, expr: &Expr, bindings: &Bindings) -> String {
        match expr {
            Expr::Str(s) => self.resolve_literal_interpolation_with(s, bindings),
            Expr::Int(i) => i.to_string(),
            Expr::Float(f, _) => f.to_string(),
            Expr::Double(f, _) => f.to_string(),
            Expr::Bool(b) => b.to_string(),
            // State reference: identifier whose name starts with "." (e.g. ".count").
            Expr::Ident(name) => {
                let field_name = name.as_str().trim_start_matches('.');
                self.read_state_as_string_with(field_name, bindings)
            }
            // Field access: object.field → Dot(object, field)
            Expr::Dot(object, field) => {
                // Plan 402: handle .store.X path — flatten to read root state X.
                // Mirrors resolve_expr_to_value's store flattening (D-GAP-4).
                // Without this, `.store.mines_label` resolves to empty (read_state
                // has no "store" field since store fields are bare-named in root
                // state) → empty text node → filtered out → label disappears.
                if let Expr::Dot(inner_obj, store_field) = object.as_ref() {
                    if store_field.as_str() == "store"
                        && matches!(inner_obj.as_ref(), Expr::Ident(n) if n.as_str() == "." || n.as_str() == "self")
                    {
                        return self.read_state_as_string_with(field.as_str(), bindings);
                    }
                }
                // Plan 370 (Issue 4): single-level self-ref `.field` (parsed as
                // Dot(Ident("."), field)) — read directly from state, mirroring
                // the fix in resolve_expr_to_value. Without this, `.edit_title`
                // falls through to resolve_expr_to_value(Ident(".")) which reads
                // state field "" and returns None → empty string.
                if let Expr::Ident(name) = object.as_ref() {
                    if name.as_str() == "." || name.as_str() == "self" {
                        return self.read_state_as_string_with(field.as_str(), bindings);
                    }
                }
                let obj_val = self.resolve_expr_to_value(object, bindings);
                let field_str = field.as_str();
                match obj_val {
                    Some(Value::Obj(map)) => {
                        map.get(field_str)
                            .map(|v| value_to_display_string(&v))
                            .unwrap_or_default()
                    }
                    Some(Value::Int(id)) if id >= 4_000_000 => {
                        let raw = Value::Int(id);
                        let materialized = self.bridge.materialize_obj_ref(&raw);
                        if let Value::Obj(map) = materialized {
                            map.get(field_str)
                                .map(|v| value_to_display_string(&v))
                                .unwrap_or_default()
                        } else {
                            String::new()
                        }
                    }
                    Some(Value::VmRef(r)) => {
                        let materialized = self.bridge.materialize_obj_ref(&Value::VmRef(r));
                        if let Value::Obj(map) = materialized {
                            map.get(field_str)
                                .map(|v| value_to_display_string(&v))
                                .unwrap_or_default()
                        } else {
                            String::new()
                        }
                    }
                    _ => String::new(),
                }
            }
            // Plan 339: if-expression for conditional string values (e.g. style)
            Expr::If(if_expr) => {
                if let Some(cond) = if_expr.branches.first() {
                    let cond_val = self.resolve_expr_to_value(&cond.cond, bindings);
                    let is_true = match cond_val {
                        Some(Value::Bool(false)) | Some(Value::Nil) | None => false,
                        Some(Value::Int(i)) if i == 0 => false,
                        _ => true,
                    };
                    if is_true {
                        // then body must be a single expression (Plan 339 contract)
                        if cond.body.stmts.len() == 1 {
                            if let crate::ast::Stmt::Expr(e) = &cond.body.stmts[0] {
                                return self.resolve_expr_to_string_with(e, bindings);
                            }
                        }
                        String::new()
                    } else if let Some(else_body) = &if_expr.else_ {
                        if else_body.stmts.len() == 1 {
                            if let crate::ast::Stmt::Expr(e) = &else_body.stmts[0] {
                                return self.resolve_expr_to_string_with(e, bindings);
                            }
                        }
                        String::new()
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                }
            }
            _ => String::new(),
        }
    }

    /// EDGE-16 第五层:求值 computed 属性。查 self.computed 表,命中则用其
    /// expr 在当前 bindings 下递归求值(resolve_expr_to_value)。用 visited
    /// 防递归循环(computed 引用自身)。未命中返回 None。
    fn eval_computed(&self, name: &str, bindings: &Bindings) -> Option<Value> {
        if let Some(computed_list) = self.computed {
            if let Some(c) = computed_list.iter().find(|c| c.name == name) {
                // 防 computed 递归引用自身(bindings 里已有同名则跳过)
                if !bindings.contains_key(name) {
                    return self.resolve_expr_to_value(&c.expr, bindings);
                }
            }
        }
        None
    }

    /// Resolve a base AST `Expr` to a Value, checking loop bindings and VmBridge state.
    fn resolve_expr_to_value(&self, expr: &Expr, bindings: &Bindings) -> Option<Value> {
        match expr {
            // State reference: identifier whose name starts with "." (e.g. ".count")
            // or a plain identifier (loop var / state).
            Expr::Ident(name) => {
                let field_name = name.as_str().trim_start_matches('.');
                bindings.get(field_name).cloned()
                    .or_else(|| self.eval_computed(field_name, bindings))
                    .or_else(|| self.read_state(field_name).ok())
            }
            // Field access: object.field → Dot(object, field)
            Expr::Dot(object, field) => {
                // Plan 370 D-GAP-4: handle .store.X path — flatten to read root state X.
                // Store fields are merged into root state as bare names.
                if let Expr::Dot(inner_obj, store_field) = object.as_ref() {
                    if store_field.as_str() == "store"
                        && matches!(inner_obj.as_ref(), Expr::Ident(n) if n.as_str() == "." || n.as_str() == "self")
                    {
                        return self.read_state(field.as_str()).ok();
                    }
                }
                // Also handle bare "store.X" (Ident("store") Dot field)
                if let Expr::Ident(name) = object.as_ref() {
                    if name.as_str() == "store" {
                        return self.read_state(field.as_str()).ok();
                    }
                    // Plan 370 (Issue 2): handle single-level self-reference
                    // `.field` (parsed as Dot(Ident("."), field)). Without this,
                    // `.note` in a child widget (e.g. EditorPanel's `text note.title`)
                    // falls through to resolve_expr_to_value(Ident(".")) which
                    // reads state field "" and fails → note resolves to None →
                    // the entire panel renders empty.
                    if name.as_str() == "." || name.as_str() == "self" {
                        // EDGE-16 第五层:`.field` 可能是循环变量(for b in ...
                        // 里的 .b),先查 bindings,再回退 state。
                        if let Some(v) = bindings.get(field.as_str()) {
                            return Some(v.clone());
                        }
                        return self.eval_computed(field.as_str(), bindings)
                            .or_else(|| self.read_state(field.as_str()).ok());
                    }
                }
                let obj = self.resolve_expr_to_value(object, bindings)?;
                let field_str = field.as_str();
                match obj {
                    Value::Obj(map) => map.get(field_str),
                    // Plan 320: raw struct heap id from Index — materialize to Obj
                    // so FieldAccess can read fields.
                    Value::Int(id) if id >= 4_000_000 => {
                        let raw = Value::Int(id);
                        let materialized = self.bridge.materialize_obj_ref(&raw);
                        if let Value::Obj(map) = materialized {
                            map.get(field_str)
                        } else {
                            None
                        }
                    }
                    // Plan 322: VmRef struct instances (from list rebuild after delete)
                    Value::VmRef(r) => {
                        let materialized = self.bridge.materialize_obj_ref(&Value::VmRef(r));
                        if let Value::Obj(map) = materialized {
                            map.get(field_str)
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }
            // Index access: target[index]
            Expr::Index(target, index) => {
                let target_val = self.resolve_expr_to_value(target, bindings)?;
                let index_val = self.resolve_expr_to_value(index, bindings)?;
                match (&target_val, &index_val) {
                    (Value::Array(arr), Value::Int(i)) => {
                        let idx = *i as usize;
                        if idx < arr.len() { Some(arr[idx].clone()) } else { None }
                    }
                    (Value::Obj(map), Value::Str(key)) => map.get(key.as_str()),
                    // Plan 318: index into a list/array stored as a VmRef or Int
                    // array_id (List<T> / Vec from `var x = []; x.push(...)`). The
                    // EditorPanel's `note: .notes[.active_id]` reads `.notes`
                    // (a VmRef to ListData) and indexes it. Deref to Vec<Value>
                    // first, then index. Use read_state_as_vec via a temp field
                    // name when the target is a StateRef; otherwise deref inline.
                    (Value::VmRef(r), Value::Int(i)) => {
                        // Plan 318/337: return the raw element (struct heap id)
                        // without materializing. View FieldAccess handles
                        // materialization via materialize_obj_ref; handler
                        // GET_FIELD needs the raw id to do heap_objects lookup.
                        self.bridge.index_list(r.id, *i)
                    }
                    // Plan 390 §15 H3b: arrays are ListData<Value> in heap_objects (4M+).
                    (Value::Int(id), Value::Int(i)) if *id >= 4_000_000 => {
                        self.bridge.index_list(*id as usize, *i)
                    }
                    _ => None,
                }
            }
            Expr::Int(i) => Some(Value::Int(*i)),
            Expr::Float(f, _) => Some(Value::Double(*f)),
            Expr::Double(f, _) => Some(Value::Double(*f)),
            Expr::Bool(b) => Some(Value::Bool(*b)),
            Expr::Str(s) => Some(Value::Str(s.clone())),
            // Plan 339: binary comparison for if-expressions (e.g., i == .active_index)
            Expr::Bina(left, Op::Eq, right) => {
                let l = self.resolve_expr_to_value(left, bindings)?;
                let r = self.resolve_expr_to_value(right, bindings)?;
                Some(Value::Bool(l == r))
            }
            Expr::Bina(left, Op::Neq, right) => {
                let l = self.resolve_expr_to_value(left, bindings)?;
                let r = self.resolve_expr_to_value(right, bindings)?;
                Some(Value::Bool(l != r))
            }
            // Plan 339: conditional if-expression for style/attribute values
            Expr::If(if_expr) => {
                if let Some(branch) = if_expr.branches.first() {
                    let cond_val = self.resolve_expr_to_value(&branch.cond, bindings)?;
                    let is_true = match &cond_val {
                        Value::Bool(false) | Value::Nil => false,
                        Value::Int(i) if *i == 0 => false,
                        _ => true,
                    };
                    if is_true {
                        // then body must be a single expression (Plan 339 contract)
                        if branch.body.stmts.len() == 1 {
                            if let crate::ast::Stmt::Expr(e) = &branch.body.stmts[0] {
                                return self.resolve_expr_to_value(e, bindings);
                            }
                        }
                        None
                    } else if let Some(else_body) = &if_expr.else_ {
                        if else_body.stmts.len() == 1 {
                            if let crate::ast::Stmt::Expr(e) = &else_body.stmts[0] {
                                return self.resolve_expr_to_value(e, bindings);
                            }
                        }
                        None
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Check if a loop item matches the current search filter.
    ///
    /// Reads the `search` state field. If it's empty or doesn't exist, all items match.
    /// If non-empty and the item is an Obj, checks if any string field contains the search text.
    fn matches_search(&self, item: &Value) -> bool {
        let search_text = match self.read_state("search") {
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

        // Plan 049: leading negation — `! .collapsed` / `!.collapsed` / `!x`.
        // Handled AFTER || / && (so `!a || b` splits on || first, then the left
        // arm recurses here) but BEFORE operator splitting, so `a != b` (the
        // "!=" appears mid-string, not as a prefix) is unaffected. Guard "!="
        // as a bare prefix (e.g. `!= something`) — that is a comparison, not
        // a negation.
        if cond.starts_with('!') {
            let rest = cond[1..].trim_start();
            if !rest.starts_with('=') {
                return !self.eval_condition_with(rest, bindings);
            }
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
            // Bare state ref — truthy check.
            // Plan 370 (Issue 2): dotted prop paths like `.note.pinned`
            // need field-by-field resolution (read_state only looks up a
            // single field name, so `read_state("note.pinned")` fails).
            let path = &cond[1..];
            let has_inner_dot = path.contains('.') && !path.starts_with("store.");
            if has_inner_dot {
                if let Some(expr) = Self::parse_dot_path_to_expr(cond) {
                    return self.resolve_expr_to_value(&expr, bindings)
                        .map(|v| v.as_bool())
                        .unwrap_or(false);
                }
                return false;
            }
            return self.read_state(path)
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
            match self.read_state(field_name) {
                Ok(Value::Array(arr)) => arr.len().to_string(),
                Ok(other) => {
                    // Also try read_state_as_vec for Value::Int(array_id) refs
                    match self.read_state_as_vec(field_name) {
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
            // State ref (e.g., ".filter") or nested prop path (e.g., ".block.output)
            let name = &lhs[1..];
            if name.contains('.') {
                // EDGE-16: 嵌套路径(.block.output)用 Dot expr resolve,
                // read_state 只查单字段名取不到。
                if let Some(expr) = Self::parse_dot_path_to_expr(lhs) {
                    match self.resolve_expr_to_value(&expr, bindings) {
                        Some(v) => value_to_display_string(&v),
                        None => return false,
                    }
                } else {
                    return false;
                }
            } else {
                match self.read_state(name) {
                    Ok(v) => value_to_display_string(&v),
                    Err(_) => return false,
                }
            }
        } else {
            match self.read_state(lhs) {
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
    /// F-strings like `f"Count: ${.count}"` are extracted as `Expr::Str`
    /// with the template preserved. This method scans for `${.name}` patterns
    /// and substitutes current state values.
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
        // Resolve each declared parameter. A param can be:
        //   - a loop-variable reference (e.g. `i`, `note.id`) → resolve from
        //     bindings;
        //   - a literal (e.g. `"indigo"`, `42`) → use directly.
        // Previously only binding references were resolved; literals like
        // `onclick: .SetAccent("indigo")` were dropped (param not in bindings
        // → args empty → handler received no arg → no-op / stack mismatch).
        let mut args: Vec<Value> = Vec::with_capacity(event.params.len());
        for param in &event.params {
            if let Some(val) = self.resolve_binding_path(param, bindings) {
                args.push(val);
            } else {
                // Not a binding — treat as a literal value.
                args.push(parse_event_param_literal(param));
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
    /// Extract text content from child nodes (for elements like button whose
    /// label comes from inner `text` children rather than a primary prop).
    /// Walks children recursively, resolving each text element's "text"/literal
    /// content with the given bindings, and joins them with newlines. Returns
    /// None if no text found. Recurses into container elements (row/col/etc.)
    /// so that a button like:
    ///   button { row { text note.title } text note.time }
    /// yields "Welcome\nJust now" (newline-separated), so the iced renderer
    /// can render title and time on separate lines with different styling.
    fn extract_children_text(&self, children: &[AuraNode], bindings: &Bindings) -> Option<String> {
        let parts: Vec<String> = children.iter().filter_map(|c| match c {
            AuraNode::Element { tag, props, .. }
                if matches!(tag.as_str(), "text" | "label" | "h1" | "h2" | "h3" | "p" | "span") =>
            {
                self.extract_string_with(props, "text", bindings)
                    .or_else(|| self.extract_string_with(props, "label", bindings))
            }
            AuraNode::Element { tag, children, .. }
                if matches!(tag.as_str(), "row" | "col" | "column" | "container" | "scrollable" | "grid") =>
            {
                // Recurse into layout containers to find nested text.
                self.extract_children_text(children, bindings)
            }
            AuraNode::Text(AuraTextContent::Literal(s)) => Some(s.clone()),
            AuraNode::Text(AuraTextContent::Interpolated { template, bindings: tpl_bindings }) => {
                Some(self.resolve_interpolation_with(template, tpl_bindings, bindings))
            }
            _ => None,
        }).collect();
        if parts.is_empty() { None } else { Some(parts.join("\n")) }
    }

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
                Expr::Int(i) => {
                    if *i >= 0 && *i <= u16::MAX as i32 {
                        Some(*i as u16)
                    } else {
                        None
                    }
                }
                Expr::Float(f, _) | Expr::Double(f, _) => {
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
            AuraPropValue::Expr(Expr::Bool(b)) => Some(*b),
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
                Expr::Int(i) => Some(*i as f64),
                Expr::Float(f, _) | Expr::Double(f, _) => Some(*f),
                Expr::Ident(name) => {
                    let field_name = name.as_str().trim_start_matches('.');
                    match self.read_state(field_name) {
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

/// Plan 049: evaluate a child model-var `initial` expression to a Value WITHOUT
/// a VM. Mirrors `eval_expr_to_value` (vm_bridge.rs) for the literal cases so
/// `var collapsed bool = false` → `Value::Bool(false)`. Complex initializers
/// (type literals / `.new(...)` calls, which need the VM heap) fall back to
/// `Value::Nil` — the same default the VM path uses for unresolvable exprs.
fn eval_initial_without_vm(expr: &Expr) -> Value {
    match expr {
        Expr::Int(i) => Value::Int(*i),
        Expr::I64(i) => Value::Int(*i as i32),
        Expr::Uint(u) => Value::Uint(*u),
        Expr::U64(u) => Value::Uint(*u as u32),
        Expr::Byte(b) => Value::Int(*b as i32),
        Expr::I8(i) => Value::Int(*i as i32),
        Expr::U8(u) => Value::Int(*u as i32),
        Expr::Float(f, _) => Value::Double(*f),
        Expr::Double(f, _) => Value::Double(*f),
        Expr::Bool(b) => Value::Bool(*b),
        Expr::Char(c) => Value::Int(*c as i32),
        Expr::Str(s) => Value::Str(s.clone()),
        Expr::CStr(s) => Value::Str(s.clone()),
        Expr::Unary(op, operand) => {
            let val = eval_initial_without_vm(operand);
            match op {
                Op::Sub => match val {
                    Value::Int(i) => Value::Int(-i),
                    Value::Double(f) => Value::Double(-f),
                    Value::Float(f) => Value::Float(-f),
                    _ => Value::Int(0),
                },
                Op::Not => match val {
                    Value::Bool(b) => Value::Bool(!b),
                    _ => Value::Bool(true),
                },
                _ => Value::Int(0),
            }
        }
        Expr::Array(elements) => {
            let values: Vec<Value> = elements.iter().map(eval_initial_without_vm).collect();
            Value::Array(auto_val::Array::from(values))
        }
        Expr::Object(pairs) => {
            let mut obj = auto_val::Obj::new();
            for pair in pairs {
                obj.set(pair.key.to_astr(), eval_initial_without_vm(&pair.value));
            }
            Value::Obj(obj)
        }
        _ => Value::Nil,
    }
}

/// Plan 370 (Issue 1): returns true for views that carry no visible content
/// and should be dropped from layouts to avoid spurious blank space. Covers
/// `View::Empty` and `View::Text { content: "", .. }` (the latter renders as
/// a one-line-tall `text("")` spacer in iced).
fn is_visually_empty(v: &View<DynamicMessage>) -> bool {
    match v {
        View::Empty => true,
        View::Text { content, .. } => content.is_empty(),
        _ => false,
    }
}

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

/// Parse an event-handler parameter that is NOT a loop-variable binding into
/// a literal Value. The parser stores onclick args as raw strings; a quoted
/// string like `"indigo"` (with the quotes preserved) becomes a Str, a number
/// becomes Int, anything else is treated as a bare string identifier.
fn parse_event_param_literal(param: &str) -> Value {
    let trimmed = param.trim();
    // Quoted string literal: "indigo" → Str("indigo")
    if (trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2)
        || (trimmed.starts_with('\'') && trimmed.ends_with('\'') && trimmed.len() >= 2)
    {
        return Value::Str(trimmed[1..trimmed.len() - 1].into());
    }
    // Integer literal: 42 → Int(42)
    if let Ok(i) = trimmed.parse::<i32>() {
        return Value::Int(i);
    }
    // Boolean literal
    match trimmed {
        "true" => return Value::Bool(true),
        "false" => return Value::Bool(false),
        _ => {}
    }
    // Fallback: treat as a bare string (identifier-like)
    Value::Str(trimmed.into())
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
            style_css: None,
            ext_imports: Vec::new(),
            watchers: Vec::new(),
            exposes: Vec::new(),
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

    /// REGRESSION: a button whose label comes from inner `text` children
    /// (not a primary prop) must render those children's text as its label.
    ///
    /// This is the NoteItem pattern from 015-notes (sidebar.at):
    ///   button { text note.title { ... } text note.time { ... } }
    /// Before the fix, convert_button ignored children entirely, so the note
    /// title never appeared in the rendered sidebar.
    #[test]
    fn test_button_label_from_children() {
        let widget = make_test_widget("Test", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Test");

        // button with a text child (literal string as the "text" prop)
        let text_child = AuraNode::Element {
            tag: "text".to_string(),
            props: {
                let mut m = HashMap::new();
                m.insert("text".to_string(), AuraPropValue::Expr(Expr::Str("Welcome".into())));
                m
            },
            events: HashMap::new(),
            children: Vec::new(),
            span: None,
            debug_id: None,
        };
        let button = AuraNode::Element {
            tag: "button".to_string(),
            props: HashMap::new(), // no "text"/"label" prop — label must come from children
            events: HashMap::new(),
            children: vec![text_child],
            span: None,
            debug_id: None,
        };

        let view = builder.build(&button);
        match view {
            View::Button { label, .. } => {
                assert_eq!(label, "Welcome", "button label should come from children");
            }
            other => panic!("Expected View::Button, got {:?}", other),
        }
    }

    #[test]
    fn test_build_text_with_state_ref() {
        let widget = make_test_widget("Counter", vec![
            AuraStateDef {
                name: "count".to_string(),
                type_info: Type::Int,
                initial: Expr::Int(42),
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
    // Plan 049 — block 折叠 VM 修复回归测试
    // ========================================================================

    #[test]
    fn test_conditional_negation_renders_then_body() {
        // Plan 049:BlockItem 折叠 body 的条件 `if !.collapsed` 在 collapsed=false
        // 时必须为 true。修复前 eval_condition_with 不认识 `!` 前缀,`! .collapsed`
        // 落入 resolve_binding_path → None → false,导致 ls 结果默认全隐藏。
        let widget = make_test_widget("Fold", vec![
            AuraStateDef {
                name: "collapsed".to_string(),
                type_info: Type::Bool,
                initial: Expr::Bool(false),
                decorators: vec![],
            },
        ]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Fold");

        let node = AuraNode::Conditional {
            condition: "! .collapsed".to_string(),
            then_body: vec![AuraNode::text("body")],
            else_body: None,
            span: None,
            debug_id: None,
        };
        match builder.build(&node) {
            View::Text { content, .. } => {
                assert_eq!(content, "body", "!false must render the then_body");
            }
            other => panic!("expected View::Text body, got {:?}", other),
        }
    }

    #[test]
    fn test_conditional_negation_hides_when_true() {
        // collapsed=true → `if !.collapsed` → false → body 隐藏(折叠生效)。
        let widget = make_test_widget("Fold", vec![
            AuraStateDef {
                name: "collapsed".to_string(),
                type_info: Type::Bool,
                initial: Expr::Bool(true),
                decorators: vec![],
            },
        ]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Fold");

        let node = AuraNode::Conditional {
            condition: "! .collapsed".to_string(),
            then_body: vec![AuraNode::text("body")],
            else_body: None,
            span: None,
            debug_id: None,
        };
        assert!(
            matches!(builder.build(&node), View::Empty),
            "!true must hide the body"
        );
    }

    #[test]
    fn test_text_onclick_becomes_toggle_button() {
        // Plan 049:VM 的 convert_row 丢弃 row 的 onclick,所以 block_item.at
        // 把折叠切换挂到 text 元素上 —— text 带 onclick 必须转成 chromeless
        // Button,且消息是 Typed{ event_name: "ToggleCollapse" }。
        let widget = make_test_widget("Fold", vec![]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Fold");

        let node = AuraNode::element("text")
            .with_prop("text", Expr::Ident(".collapse_glyph".into()))
            .with_event("onclick", ".ToggleCollapse");
        match builder.build(&node) {
            View::Button { onclick, .. } => match onclick {
                DynamicMessage::Typed { event_name, .. } => {
                    assert_eq!(event_name, "ToggleCollapse");
                }
                other => panic!("expected Typed ToggleCollapse, got {:?}", other),
            },
            other => panic!("text-with-onclick must convert to Button, got {:?}", other),
        }
    }

    #[test]
    fn test_child_model_var_default_seeded_into_state() {
        // Plan 049:渲染子组件(BlockList → BlockItem)时,子组件 model 变量
        // 的默认值必须种入统一 root state。否则 `.collapsed` 首次读取 Err →
        // 条件 false(内容隐藏)+ 字符串回退 "${collapse_glyph}"。
        let child = make_test_widget("BlockItem", vec![
            AuraStateDef {
                name: "collapsed".to_string(),
                type_info: Type::Bool,
                initial: Expr::Bool(false),
                decorators: vec![],
            },
        ]);
        let parent = make_test_widget("BlockList", vec![]);
        let bridge = VmBridge::new(&parent).unwrap();
        let mut registry = crate::ui::widget_registry::WidgetRegistry::new();
        registry.register(child);
        let builder = AuraViewBuilder::with_registry(&bridge, "BlockList", &registry);

        let node = AuraNode::Component {
            name: "BlockItem".to_string(),
            props: vec![],
            events: HashMap::new(),
            children: vec![],
            span: None,
            debug_id: None,
        };
        builder.build(&node);
        assert_eq!(
            bridge.read_state("collapsed").unwrap(),
            Value::Bool(false),
            "child model var default must be seeded into root state"
        );
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
                initial: Expr::Int(42),
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
                initial: Expr::Int(42),
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
        // below). State expr has no array literal, so we seed via write_state.
        let widget = make_test_widget("List", vec![
            AuraStateDef {
                name: "items".to_string(),
                type_info: Type::List(Box::new(Type::StrSlice)),
                initial: Expr::Str(String::new().into()),
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
                initial: Expr::Str(String::new().into()),
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
                initial: Expr::Str(String::new().into()),
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
            .with_prop("cols", Expr::Int(7))
            .with_child(for_loop)
    }

    fn widget_with_items() -> (AuraWidget, VmBridge) {
        let widget = make_test_widget("Grid", vec![
            AuraStateDef {
                name: "items".to_string(),
                type_info: Type::List(Box::new(Type::StrSlice)),
                initial: Expr::Str(String::new().into()),
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
                initial: Expr::Str(String::new().into()),
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
                    AuraPropValue::Expr(Expr::Dot(
                        Box::new(Expr::Ident(".cell".into())),
                        "label".into(),
                    )),
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
            .with_prop("cols", Expr::Int(7))
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
                ("label".to_string(), AuraPropValue::Expr(Expr::Str("Inc".into()))),
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
                ("spacing".to_string(), AuraPropValue::Expr(Expr::Int(10))),
                ("padding".to_string(), AuraPropValue::Expr(Expr::Int(5))),
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
                ("spacing".to_string(), AuraPropValue::Expr(Expr::Int(8))),
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
                ("text".to_string(), AuraPropValue::Expr(Expr::Str("Increment".into()))),
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
                initial: Expr::Int(7),
                decorators: vec![],
            },
        ]);
        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "Counter");

        let node = AuraNode::Element {
            tag: "text".to_string(),
            props: HashMap::from([
                ("text".to_string(), AuraPropValue::Expr(Expr::Ident(".count".into()))),
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
                ("label".to_string(), AuraPropValue::Expr(Expr::Str("Reset".into()))),
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
                initial: Expr::Str("active".into()),
                decorators: vec![],
            },
            AuraStateDef {
                name: "todos".to_string(),
                type_info: Type::StrOwned,
                initial: Expr::Str("[]".into()),
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
                initial: Expr::Str("completed".into()),
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
                initial: Expr::Str("-1".into()),
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
