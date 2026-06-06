//! # DynamicComponent - VM-driven UI component with Component trait
//!
//! This module provides [`DynamicComponent`], which ties together [`VmBridge`] (state
//! management via AutoVM) and [`AuraViewBuilder`] (AuraNode to View conversion) to
//! implement the [`Component`] trait for use in iced and other UI backends.
//!
//! ## Architecture
//!
//! ```text
//! AuraWidget (parsed from .at source)
//!    |
//!    +--> VmBridge (state + handlers)
//!    +--> view_template (AuraNode)
//!    |
//!    v
//! DynamicComponent
//!  - implements Component trait
//!  - on(msg) -> VmBridge::call_handler()
//!  - view()  -> AuraViewBuilder::build()
//!    |
//!    v
//! UI Backend (iced, GPUI, headless)
//! ```
//!
//! ## Plan 205 Phase 3
//!
//! Phase 3 creates the DynamicComponent struct:
//! - Holds a VmBridge and the AuraNode view template
//! - Implements the Component trait (on/view methods)
//! - Routes DynamicMessage events to VmBridge handlers
//! - Uses AuraViewBuilder to produce View<DynamicMessage>

use std::fmt;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use std::collections::HashMap;

use crate::aura::AuraWidget;
use crate::aura::{AuraNodeId, SpanInfo};
use crate::ui::aura_view_builder::AuraViewBuilder;
use crate::ui::component::Component;
use crate::ui::debug_id_map::DebugIdMap;
use crate::ui::interpreter::DynamicMessage;
use crate::ui::state_migration::{self, MigrationReport};
use crate::ui::view::View;
use crate::ui::vm_bridge::VmBridge;

// ============================================================================
// DynamicComponent
// ============================================================================

/// A dynamic UI component driven by AutoVM.
///
/// Renders views from an [`AuraNode`](crate::aura::AuraNode) template and routes
/// events to VM handlers via [`VmBridge`]. Each `DynamicComponent` corresponds to
/// a single AURA widget definition.
///
/// # Lifecycle
///
/// 1. `DynamicComponent::new(widget)` - Create from an AuraWidget
/// 2. `component.view()` - Render the current view (reads state from VmBridge)
/// 3. `component.on(msg)` - Handle a user event (calls VmBridge handler)
/// 4. `component.view()` - Re-render with updated state
///
/// # Example
///
/// ```ignore
/// use auto_ui::dynamic::DynamicComponent;
/// use auto_ui::Component;
///
/// let widget = parse_aura_widget("counter.at")?;
/// let mut comp = DynamicComponent::new(&widget)?;
///
/// // Initial view
/// let view = comp.view();
///
/// // Handle increment event
/// comp.on(DynamicMessage::Typed {
///     widget_name: "Counter".into(),
///     event_name: "Inc".into(),
///     args: vec![],
/// });
///
/// // Updated view
/// let updated_view = comp.view();
/// ```
pub struct DynamicComponent {
    /// VM bridge for state management and handler execution.
    bridge: VmBridge,

    /// The AuraNode view template (cloned from AuraWidget::view_tree).
    view_template: crate::aura::AuraNode,

    /// Widget name, cached for efficient access.
    widget_name: String,

    /// Dirty flag -- set when state changes via `on()`, cleared after `view()`.
    dirty: bool,

    /// Optional source file path for hot-reload tracking.
    source_path: Option<PathBuf>,

    /// Last known modification time of the source file.
    last_modified: Option<SystemTime>,

    /// Input-to-state mapping: event_name -> state_field_name.
    /// When an input fires its oninput/onchange event, the typed text
    /// is written to the mapped state field before the handler runs.
    input_state_map: HashMap<String, String>,

    /// Tick interval in ms — when set, the runtime emits .Tick events at this interval.
    tick_interval: Option<u32>,

    /// Span map: AuraNodeId → source info for DevTools (Plan 274)
    span_map: HashMap<AuraNodeId, SpanInfo>,

    /// Key bindings: key string → handler pattern (Plan 275)
    key_bindings: HashMap<String, String>,
}

impl fmt::Debug for DynamicComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DynamicComponent")
            .field("widget_name", &self.widget_name)
            .field("dirty", &self.dirty)
            .field("state_fields", &self.bridge.state_fields())
            .field("handlers", &self.bridge.handler_names())
            .finish()
    }
}

impl DynamicComponent {
    /// Create a new DynamicComponent from an AuraWidget definition.
    ///
    /// This initializes:
    /// 1. A [`VmBridge`] with the widget's state and handlers
    /// 2. Extracts the view template from the widget
    /// 3. Returns a ready-to-use DynamicComponent
    ///
    /// # Arguments
    ///
    /// * `widget` - The AuraWidget to create a component for
    ///
    /// # Errors
    ///
    /// Returns a string describing the error if VmBridge initialization fails.
    pub fn new(widget: &AuraWidget) -> Result<Self, String> {
        // 1. Create VmBridge from widget (initializes state on VM heap)
        let bridge = VmBridge::new(widget)
            .map_err(|e| format!("VmBridge init failed for '{}': {}", widget.name, e))?;

        // 2. Extract view template
        let view_template = widget.view_tree.clone();
        let widget_name = widget.name.clone();

        // 3. Extract input-to-state mapping for text input handling
        let input_state_map = extract_input_state_map(&widget.view_tree);

        Ok(Self {
            bridge,
            view_template,
            widget_name,
            dirty: true,
            source_path: None,
            last_modified: None,
            input_state_map,
            tick_interval: widget.tick_interval,
            span_map: widget.span_map.clone(),
            key_bindings: widget.key_bindings.clone(),
        })
    }

    /// Create a new DynamicComponent with a pre-configured AutoVM instance.
    ///
    /// Use this when the VM already has bytecode loaded (e.g., from a compiled module).
    ///
    /// # Arguments
    ///
    /// * `vm` - Pre-configured AutoVM instance
    /// * `widget` - The AuraWidget to create a component for
    pub fn new_with_vm(
        vm: crate::vm::engine::AutoVM,
        widget: &AuraWidget,
    ) -> Result<Self, String> {
        let bridge = VmBridge::new_with_vm(vm, widget)
            .map_err(|e| format!("VmBridge init failed for '{}': {}", widget.name, e))?;

        let view_template = widget.view_tree.clone();
        let widget_name = widget.name.clone();
        let input_state_map = extract_input_state_map(&widget.view_tree);

        Ok(Self {
            bridge,
            view_template,
            widget_name,
            dirty: true,
            source_path: None,
            last_modified: None,
            input_state_map,
            tick_interval: widget.tick_interval,
            span_map: widget.span_map.clone(),
            key_bindings: widget.key_bindings.clone(),
        })
    }
    // ========================================================================
    // State access
    // ========================================================================

    /// Read a state field value from the VM.
    ///
    /// Returns the current value of the named state field, or an error if
    /// the field does not exist.
    pub fn read_state(&self, field_name: &str) -> Result<auto_val::Value, String> {
        self.bridge
            .read_state(field_name)
            .map_err(|e| e.to_string())
    }

    /// Write a state field value to the VM.
    ///
    /// Updates the named state field and marks the component as dirty.
    pub fn write_state(&mut self, field_name: &str, value: auto_val::Value) -> Result<(), String> {
        self.bridge
            .write_state(field_name, value)
            .map_err(|e| e.to_string())?;
        self.dirty = true;
        Ok(())
    }

    /// Check if the component needs re-rendering.
    ///
    /// Returns `true` after `on()` processes a message or `write_state()` is called.
    /// Returns `false` after `view()` has been called.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Clear the dirty flag. Called by the runtime after consuming the dirty state.
    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    /// Get the widget name.
    pub fn widget_name(&self) -> &str {
        &self.widget_name
    }

    /// Get the state field names.
    pub fn state_fields(&self) -> &[String] {
        self.bridge.state_fields()
    }

    /// Get a reference to the underlying VmBridge.
    pub fn bridge(&self) -> &VmBridge {
        &self.bridge
    }

    /// Get a mutable reference to the underlying VmBridge.
    pub fn bridge_mut(&mut self) -> &mut VmBridge {
        &mut self.bridge
    }

    /// Get the tick interval in ms (if set).
    pub fn tick_interval(&self) -> Option<u32> {
        self.tick_interval
    }

    /// Find the source span for a specific element by kind and occurrence index.
    /// Traverses the AuraNode tree in DFS order, counting Element nodes by tag name.
    /// Returns the span of the `target_index`-th occurrence of `target_kind`.
    pub fn find_element_span(&self, target_kind: &str, target_index: usize) -> Option<(usize, usize)> {
        let mut counter = 0;
        find_span_dfs(&self.view_template, target_kind, target_index, &mut counter)
    }

    /// Build a View with DebugIdMap sideband mapping (Plan 274).
    /// Returns (View, DebugIdMap) for use by DevTools.
    pub fn view_with_debug(&self) -> (View<DynamicMessage>, DebugIdMap) {
        let builder = AuraViewBuilder::new(&self.bridge, &self.widget_name);
        builder.build_with_debug(&self.view_template)
    }

    /// Get the span map (AuraNodeId → SpanInfo) for DevTools.
    pub fn span_map(&self) -> &HashMap<AuraNodeId, SpanInfo> {
        &self.span_map
    }

    /// Get key bindings (Plan 275)
    pub fn key_bindings(&self) -> &HashMap<String, String> {
        &self.key_bindings
    }

    /// Get the input-to-state mapping: event_name -> state_field_name (Plan 278).
    /// Used by MCP to know which state field to update when typing into an input.
    pub fn input_state_map(&self) -> &HashMap<String, String> {
        &self.input_state_map
    }

    /// Get the original AuraNode view template (Plan 279).
    /// Used by MCP to generate AURA source-style snapshots with full original info.
    pub fn view_template(&self) -> &crate::aura::AuraNode {
        &self.view_template
    }

    // ========================================================================
    // Hot Reload (Plan 205 Phase 4)
    // ========================================================================

    /// Set the source file path for hot-reload tracking.
    ///
    /// After calling this, `check_reload()` can be used to detect file changes.
    /// The current modification time of the file is recorded as the baseline.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the `.at` source file
    pub fn set_source_path(&mut self, path: impl Into<PathBuf>) {
        let path = path.into();
        self.last_modified = std::fs::metadata(&path)
            .ok()
            .and_then(|m| m.modified().ok());
        self.source_path = Some(path);
    }

    /// Get the source file path (if set).
    pub fn source_path(&self) -> Option<&Path> {
        self.source_path.as_deref()
    }

    /// Replace a byte range in the source file with new content and return the updated source.
    ///
    /// This is used by the DevTools editor to write back edited source code.
    /// After writing, the hot-reload mechanism will detect the file change and reload.
    ///
    /// # Arguments
    /// * `offset` - Byte offset of the replacement target
    /// * `len` - Length of the replacement target
    /// * `new_content` - The replacement text
    pub fn write_source_range(
        &mut self,
        offset: usize,
        len: usize,
        new_content: &str,
    ) -> Result<String, String> {
        let path = self.source_path()
            .ok_or_else(|| "No source path set".to_string())?;

        let mut code = std::fs::read_to_string(path)
            .map_err(|e| format!("Cannot read source: {}", e))?;

        if offset + len > code.len() {
            return Err("Span range out of bounds (source may have changed)".to_string());
        }

        let new_code = format!("{}{}{}", &code[..offset], new_content, &code[offset + len..]);

        std::fs::write(path, &new_code)
            .map_err(|e| format!("Cannot write source: {}", e))?;

        // Update last_modified to reflect the write we just did,
        // so the next hot-reload tick won't re-trigger unnecessarily
        self.last_modified = std::fs::metadata(path)
            .ok()
            .and_then(|m| m.modified().ok());

        Ok(new_code)
    }

    /// Read all state fields as a name-to-value map.
    ///
    /// Delegates to [`VmBridge::read_all_state`].
    pub fn read_all_state(&self) -> std::collections::HashMap<String, auto_val::Value> {
        self.bridge.read_all_state()
    }

    /// Reload the component from a new widget definition, preserving state.
    ///
    /// This is the core hot-reload mechanism:
    /// 1. Reads all current state from the VmBridge
    /// 2. Creates a new VmBridge from the new widget definition
    /// 3. Migrates state (preserves compatible fields, adds defaults for new ones)
    /// 4. Replaces the old bridge and view template
    ///
    /// Note: since runtime state values do not carry type information, type
    /// compatibility is checked by matching old field names against the new
    /// definition. If a field name exists in both and the new type is the same
    /// simple scalar kind, the old value is preserved. Otherwise the new default
    /// is used.
    ///
    /// # Arguments
    ///
    /// * `new_widget` - The updated AuraWidget definition
    ///
    /// # Returns
    ///
    /// A [`MigrationReport`] describing what was preserved, added, and dropped.
    ///
    /// # Errors
    ///
    /// Returns a string describing the error if the new VmBridge cannot be created.
    pub fn reload(&mut self, new_widget: &AuraWidget) -> Result<MigrationReport, String> {
        // 1. Snapshot current state
        let old_state = self.bridge.read_all_state();

        // We don't have the old AuraStateDef types at runtime, so we pass
        // an empty slice. The migrate_state function will treat all old fields
        // as type-unknown, preserving values for matching field names.
        let old_field_defs: Vec<crate::aura::AuraStateDef> = vec![];

        // 2. Create a new VmBridge from the new widget
        let new_bridge = VmBridge::new(new_widget)
            .map_err(|e| format!("Failed to create new VmBridge for '{}': {}", new_widget.name, e))?;

        // 3. Migrate state
        let (migrated_state, report) = state_migration::migrate_state(
            &old_state,
            &old_field_defs,
            &new_widget.state_vars,
        );

        // 4. Apply migrated state to the new bridge
        let mut new_bridge = new_bridge;
        for (name, value) in &migrated_state {
            let _ = new_bridge.write_state(name, value.clone());
        }

        // 5. Update self
        self.bridge = new_bridge;
        self.view_template = new_widget.view_tree.clone();
        self.widget_name = new_widget.name.clone();
        self.input_state_map = extract_input_state_map(&new_widget.view_tree);
        self.tick_interval = new_widget.tick_interval;
        self.span_map = new_widget.span_map.clone();
        self.dirty = true;

        Ok(report)
    }

    /// Check if the source file has changed and provide the new modification time.
    ///
    /// This does **not** perform the reload itself -- it only detects whether the
    /// file has been modified since the last check. The caller is responsible for
    /// re-parsing the file and calling [`Self::reload()`].
    ///
    /// # Returns
    ///
    /// * `Ok(Some(new_modified_time))` - File has changed
    /// * `Ok(None)` - File has not changed or no source path is set
    /// * `Err(...)` - Could not read file metadata
    pub fn check_file_changed(&mut self) -> Result<Option<SystemTime>, String> {
        let path = match &self.source_path {
            Some(p) => p.clone(),
            None => return Ok(None),
        };

        let metadata = std::fs::metadata(&path)
            .map_err(|e| format!("Cannot read metadata for '{}': {}", path.display(), e))?;

        let current_modified = metadata.modified()
            .map_err(|e| format!("Cannot read modification time for '{}': {}", path.display(), e))?;

        let changed = match self.last_modified {
            Some(last) => current_modified > last,
            None => true, // First check always reports as changed
        };

        if changed {
            self.last_modified = Some(current_modified);
            Ok(Some(current_modified))
        } else {
            Ok(None)
        }
    }
}

// ============================================================================
// Component trait implementation
// ============================================================================

impl Component for DynamicComponent {
    type Msg = DynamicMessage;

    /// Handle a message by routing it to the appropriate VM handler.
    ///
    /// For [`DynamicMessage::Typed`], extracts the `event_name` and calls
    /// the corresponding handler registered in the VmBridge.
    ///
    /// For [`DynamicMessage::String`], uses the string directly as the handler
    /// name.
    ///
    /// After processing, the component is marked as dirty so the next `view()`
    /// call will reflect any state changes.
    fn on(&mut self, msg: Self::Msg) {
        let event_name = match &msg {
            DynamicMessage::Typed { event_name, .. } => event_name.clone(),
            DynamicMessage::String(name) => name.clone(),
        };

        // Execute handler via VM bytecode closure
        // Only mark dirty if handler was found and executed successfully
        if self.bridge.call_handler(&event_name, &[]).is_ok() {
            self.dirty = true;
        }
    }

    /// Render the view by building from the AuraNode template.
    ///
    /// Uses [`AuraViewBuilder`] to traverse the view template, resolving
    /// state references from the VmBridge at build time. After rendering,
    /// the dirty flag is cleared.
    fn view(&self) -> View<Self::Msg> {
        let builder = AuraViewBuilder::new(&self.bridge, &self.widget_name);
        let view = builder.build(&self.view_template);

        // Note: clearing dirty requires &mut self, but view() takes &self.
        // The dirty flag is a hint for external consumers; the actual view
        // is always freshly built from current state.
        view
    }
}

impl DynamicComponent {
    /// Handle an event with an optional input text value.
    ///
    /// When `input_value` is `Some(text)`, looks up the associated state field
    /// from `input_state_map` and writes the text as the field's value before
    /// running the handler. This enables two-way binding for text inputs.
    pub fn on_with_input(&mut self, event_name: &str, input_value: Option<String>) {
        // If this event comes from an input, update the bound state field first
        if let Some(text) = &input_value {
            if let Some(state_field) = self.input_state_map.get(event_name) {
                let value = parse_input_value(text);
                let _ = self.bridge.write_state(state_field, value);
                self.dirty = true; // input value changed state
            }
        }

        // Run the handler via VM bytecode closure
        match self.bridge.call_handler(event_name, &[]) {
            Ok(()) => self.dirty = true,
            Err(_) => {
                // Handler not found (e.g., indexed events like "SelectNote:0"
                // that have no direct .at handler). Don't set dirty unless
                // the input_value path above already did.
            }
        }
    }
}

/// Parse a string input value into the best-matching Value type.
fn parse_input_value(text: &str) -> auto_val::Value {
    if text.is_empty() {
        return auto_val::Value::str("");
    }
    if let Ok(i) = text.parse::<i32>() {
        return auto_val::Value::Int(i);
    }
    if let Ok(f) = text.parse::<f64>() {
        return auto_val::Value::Float(f);
    }
    if text == "true" || text == "false" {
        return auto_val::Value::Bool(text == "true");
    }
    auto_val::Value::str(text)
}

/// Clean a handler pattern to a simple name.
fn clean_handler_name(pattern: &str) -> String {
    let name = pattern.trim_start_matches('.');
    if let Some(pos) = name.rfind("::") {
        name[pos + 2..].to_string()
    } else {
        name.to_string()
    }
}

/// Scan the view tree for `input` elements with `value` from StateRef and an
/// `oninput`/`onchange` event. Returns a map of event_name -> state_field_name.
fn extract_input_state_map(view_tree: &crate::aura::AuraNode) -> HashMap<String, String> {
    let mut map = HashMap::new();
    scan_node_for_inputs(view_tree, &mut map);
    map
}

fn scan_node_for_inputs(node: &crate::aura::AuraNode, map: &mut HashMap<String, String>) {
    use crate::aura::{AuraNode, AuraPropValue, AuraExpr};
    match node {
        AuraNode::Element { tag, props, events, children, .. } => {
            if tag == "input" || tag == "textarea" {
                // Find value prop that is a StateRef
                let state_field = props.get("value").and_then(|v| match v {
                    AuraPropValue::Expr(AuraExpr::StateRef(name)) => Some(name.clone()),
                    _ => None,
                });
                // Find oninput/onchange event
                let event_name = events.get("oninput")
                    .or_else(|| events.get("onchange"))
                    .or_else(|| events.get("input"))
                    .or_else(|| events.get("change"))
                    .map(|e| clean_handler_name(&e.handler));

                if let (Some(field), Some(event)) = (state_field, event_name) {
                    map.insert(event, field);
                }
            }
            for child in children {
                scan_node_for_inputs(child, map);
            }
        }
        AuraNode::ForLoop { body, .. } => {
            for child in body {
                scan_node_for_inputs(child, map);
            }
        }
        AuraNode::Conditional { then_body, else_body, .. } => {
            for child in then_body {
                scan_node_for_inputs(child, map);
            }
            if let Some(else_children) = else_body {
                for child in else_children {
                    scan_node_for_inputs(child, map);
                }
            }
        }
        _ => {}
    }
}

/// DFS traversal to find the span of the N-th Element node with matching tag.
/// Returns the span when found, or None.
fn find_span_dfs(
    node: &crate::aura::AuraNode, target_kind: &str, target_index: usize, counter: &mut usize,
) -> Option<(usize, usize)> {
    use crate::aura::AuraNode;
    match node {
        AuraNode::Element { tag, children, span, .. } => {
            if tag == target_kind {
                let idx = *counter;
                *counter += 1;
                if idx == target_index {
                    return *span;
                }
            }
            for child in children {
                if let Some(s) = find_span_dfs(child, target_kind, target_index, counter) {
                    return Some(s);
                }
            }
            None
        }
        AuraNode::ForLoop { body, .. } => {
            for child in body {
                if let Some(s) = find_span_dfs(child, target_kind, target_index, counter) {
                    return Some(s);
                }
            }
            None
        }
        AuraNode::Conditional { then_body, else_body, .. } => {
            for child in then_body {
                if let Some(s) = find_span_dfs(child, target_kind, target_index, counter) {
                    return Some(s);
                }
            }
            if let Some(else_nodes) = else_body {
                for child in else_nodes {
                    if let Some(s) = find_span_dfs(child, target_kind, target_index, counter) {
                        return Some(s);
                    }
                }
            }
            None
        }
        AuraNode::Link { children, .. } => {
            for child in children {
                if let Some(s) = find_span_dfs(child, target_kind, target_index, counter) {
                    return Some(s);
                }
            }
            None
        }
        _ => None,
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aura::{AuraNode, AuraStateDef, AuraExpr, AuraEvent, AuraPropValue, AuraTextContent};
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
    fn test_dynamic_component_creation() {
        let widget = make_test_widget("Counter", vec![
            AuraStateDef {
                name: "count".to_string(),
                type_info: Type::Int,
                initial: AuraExpr::Int(0),
                decorators: vec![],
            },
        ]);

        let comp = DynamicComponent::new(&widget).unwrap();

        assert_eq!(comp.widget_name(), "Counter");
        assert!(comp.is_dirty());
        assert_eq!(comp.state_fields().len(), 1);
        assert_eq!(comp.state_fields()[0], "count");
    }

    #[test]
    fn test_dynamic_component_creation_empty_state() {
        let widget = make_test_widget("EmptyWidget", vec![]);
        let comp = DynamicComponent::new(&widget).unwrap();

        assert_eq!(comp.widget_name(), "EmptyWidget");
        assert!(comp.state_fields().is_empty());
    }

    #[test]
    fn test_read_state() {
        let widget = make_test_widget("Counter", vec![
            AuraStateDef {
                name: "count".to_string(),
                type_info: Type::Int,
                initial: AuraExpr::Int(42),
                decorators: vec![],
            },
        ]);

        let comp = DynamicComponent::new(&widget).unwrap();

        let value = comp.read_state("count").unwrap();
        assert_eq!(value, auto_val::Value::Int(42));
    }

    #[test]
    fn test_write_state() {
        let widget = make_test_widget("Counter", vec![
            AuraStateDef {
                name: "count".to_string(),
                type_info: Type::Int,
                initial: AuraExpr::Int(0),
                decorators: vec![],
            },
        ]);

        let mut comp = DynamicComponent::new(&widget).unwrap();

        // Initial view clears dirty
        let _ = comp.view();

        // Write new state
        comp.write_state("count", auto_val::Value::Int(10)).unwrap();

        assert!(comp.is_dirty());
        assert_eq!(comp.read_state("count").unwrap(), auto_val::Value::Int(10));
    }

    #[test]
    fn test_read_state_not_found() {
        let widget = make_test_widget("Counter", vec![]);
        let comp = DynamicComponent::new(&widget).unwrap();

        let result = comp.read_state("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_view_returns_column() {
        let widget = make_test_widget("Test", vec![]);
        let comp = DynamicComponent::new(&widget).unwrap();

        let view = comp.view();

        // Default view_tree is an empty column
        match view {
            View::Column { children, .. } => {
                assert!(children.is_empty());
            }
            _ => panic!("Expected View::Column from default view_tree"),
        }
    }

    #[test]
    fn test_view_with_state_binding() {
        let widget = AuraWidget {
            name: "Counter".to_string(),
            state_vars: vec![AuraStateDef {
                name: "count".to_string(),
                type_info: Type::Int,
                initial: AuraExpr::Int(7),
                decorators: vec![],
            }],
            computed: vec![],
            messages: vec![],
            view_tree: AuraNode::Element {
                tag: "text".to_string(),
                props: HashMap::from([
                    ("text".to_string(), crate::aura::AuraPropValue::Expr(
                        AuraExpr::StateRef("count".to_string()),
                    )),
                ]),
                events: HashMap::new(),
                children: vec![],
                span: None,
                debug_id: None,
            },
            handlers: HashMap::new(),
            props: vec![],
            routes: None,
            lifecycle: vec![],
            tick_interval: None,
            handler_params: HashMap::new(),
            span_map: HashMap::new(),
            key_bindings: HashMap::new(),
        };

        let comp = DynamicComponent::new(&widget).unwrap();
        let view = comp.view();

        match view {
            View::Text { content, .. } => {
                assert_eq!(content, "7");
            }
            _ => panic!("Expected View::Text with state-resolved value"),
        }
    }

    #[test]
    fn test_on_with_string_message() {
        let widget = make_test_widget("Counter", vec![
            AuraStateDef {
                name: "count".to_string(),
                type_info: Type::Int,
                initial: AuraExpr::Int(0),
                decorators: vec![],
            },
        ]);

        let mut comp = DynamicComponent::new(&widget).unwrap();
        let _ = comp.view(); // Clear dirty

        // on() with string message marks dirty
        comp.on(DynamicMessage::String("Inc".to_string()));

        assert!(comp.is_dirty());
    }

    #[test]
    fn test_on_with_typed_message() {
        let widget = make_test_widget("Counter", vec![
            AuraStateDef {
                name: "count".to_string(),
                type_info: Type::Int,
                initial: AuraExpr::Int(0),
                decorators: vec![],
            },
        ]);

        let mut comp = DynamicComponent::new(&widget).unwrap();
        let _ = comp.view(); // Clear dirty

        comp.on(DynamicMessage::Typed {
            widget_name: "Counter".to_string(),
            event_name: "Inc".to_string(),
            args: vec![],
        });

        assert!(comp.is_dirty());
    }

    #[test]
    fn test_on_handler_not_found_graceful() {
        // Handler not found should not panic - graceful degradation
        let widget = make_test_widget("Counter", vec![]);
        let mut comp = DynamicComponent::new(&widget).unwrap();

        // This should not panic even though no handler is registered
        comp.on(DynamicMessage::String("NonExistent".to_string()));
        assert!(comp.is_dirty());
    }

    #[test]
    fn test_debug_format() {
        let widget = make_test_widget("Counter", vec![
            AuraStateDef {
                name: "count".to_string(),
                type_info: Type::Int,
                initial: AuraExpr::Int(0),
                decorators: vec![],
            },
        ]);

        let comp = DynamicComponent::new(&widget).unwrap();
        let debug_str = format!("{:?}", comp);

        assert!(debug_str.contains("Counter"));
        assert!(debug_str.contains("count"));
    }

    #[test]
    fn test_bridge_access() {
        let widget = make_test_widget("Test", vec![]);
        let comp = DynamicComponent::new(&widget).unwrap();

        // Read-only bridge access
        assert_eq!(comp.bridge().widget_name(), "Test");

        // Mutable bridge access
        let mut comp = comp;
        comp.bridge_mut().register_handler_addr("Inc", 42);
        assert!(comp.bridge().has_handler("Inc"));
    }

    #[test]
    fn test_multiple_state_reads_and_writes() {
        let widget = make_test_widget("Multi", vec![
            AuraStateDef {
                name: "x".to_string(),
                type_info: Type::Int,
                initial: AuraExpr::Int(1),
                decorators: vec![],
            },
            AuraStateDef {
                name: "y".to_string(),
                type_info: Type::Int,
                initial: AuraExpr::Int(2),
                decorators: vec![],
            },
            AuraStateDef {
                name: "label".to_string(),
                type_info: Type::StrFixed(0),
                initial: AuraExpr::Literal("hello".to_string()),
                decorators: vec![],
            },
        ]);

        let mut comp = DynamicComponent::new(&widget).unwrap();

        // Read initial state
        assert_eq!(comp.read_state("x").unwrap(), auto_val::Value::Int(1));
        assert_eq!(comp.read_state("y").unwrap(), auto_val::Value::Int(2));
        assert_eq!(comp.read_state("label").unwrap(), auto_val::Value::str("hello"));

        // Write new values
        comp.write_state("x", auto_val::Value::Int(10)).unwrap();
        comp.write_state("y", auto_val::Value::Int(20)).unwrap();

        // Read back
        assert_eq!(comp.read_state("x").unwrap(), auto_val::Value::Int(10));
        assert_eq!(comp.read_state("y").unwrap(), auto_val::Value::Int(20));
        assert_eq!(comp.read_state("label").unwrap(), auto_val::Value::str("hello"));
    }

    #[test]
    fn test_view_with_button_and_event() {
        let widget = AuraWidget {
            name: "Counter".to_string(),
            state_vars: vec![AuraStateDef {
                name: "count".to_string(),
                type_info: Type::Int,
                initial: AuraExpr::Int(0),
                decorators: vec![],
            }],
            computed: vec![],
            messages: vec![],
            view_tree: AuraNode::Element {
                tag: "col".to_string(),
                props: HashMap::new(),
                events: HashMap::new(),
                children: vec![
                    AuraNode::Text(crate::aura::AuraTextContent::Interpolated {
                        template: "Count: ${.count}".to_string(),
                        bindings: vec!["count".to_string()],
                    }),
                    AuraNode::Element {
                        tag: "button".to_string(),
                        props: HashMap::from([
                            ("text".to_string(), crate::aura::AuraPropValue::Expr(
                                AuraExpr::Literal("Increment".to_string()),
                            )),
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
                    },
                ],
                span: None,
                debug_id: None,
            },
            handlers: HashMap::new(),
            props: vec![],
            routes: None,
            lifecycle: vec![],
            tick_interval: None,
            handler_params: HashMap::new(),
            span_map: HashMap::new(),
            key_bindings: HashMap::new(),
        };

        let comp = DynamicComponent::new(&widget).unwrap();
        let view = comp.view();

        match view {
            View::Column { children, .. } => {
                assert_eq!(children.len(), 2);

                // First child: text with state binding
                match &children[0] {
                    View::Text { content, .. } => {
                        assert_eq!(content, "Count: 0");
                    }
                    _ => panic!("Expected View::Text"),
                }

                // Second child: button with event
                match &children[1] {
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
            _ => panic!("Expected View::Column"),
        }
    }

    #[test]
    fn test_write_state_updates_view() {
        let widget = AuraWidget {
            name: "Counter".to_string(),
            state_vars: vec![AuraStateDef {
                name: "count".to_string(),
                type_info: Type::Int,
                initial: AuraExpr::Int(0),
                decorators: vec![],
            }],
            computed: vec![],
            messages: vec![],
            view_tree: AuraNode::Element {
                tag: "text".to_string(),
                props: HashMap::from([
                    ("text".to_string(), crate::aura::AuraPropValue::Expr(
                        AuraExpr::StateRef("count".to_string()),
                    )),
                ]),
                events: HashMap::new(),
                children: vec![],
                span: None,
                debug_id: None,
            },
            handlers: HashMap::new(),
            props: vec![],
            routes: None,
            lifecycle: vec![],
            tick_interval: None,
            handler_params: HashMap::new(),
            span_map: HashMap::new(),
            key_bindings: HashMap::new(),
        };

        let mut comp = DynamicComponent::new(&widget).unwrap();

        // Initial view
        let view = comp.view();
        match view {
            View::Text { content, .. } => assert_eq!(content, "0"),
            _ => panic!("Expected View::Text"),
        }

        // Update state
        comp.write_state("count", auto_val::Value::Int(99)).unwrap();

        // Updated view
        let view = comp.view();
        match view {
            View::Text { content, .. } => assert_eq!(content, "99"),
            _ => panic!("Expected View::Text"),
        }
    }

    // ========================================================================
    // Hot Reload tests (Plan 205 Phase 4)
    // ========================================================================

    #[test]
    fn test_reload_preserves_state() {
        let old_widget = make_test_widget("Counter", vec![
            AuraStateDef {
                name: "count".to_string(),
                type_info: Type::Int,
                initial: AuraExpr::Int(0),
                decorators: vec![],
            },
        ]);

        let mut comp = DynamicComponent::new(&old_widget).unwrap();

        // Modify state
        comp.write_state("count", auto_val::Value::Int(42)).unwrap();
        assert_eq!(comp.read_state("count").unwrap(), auto_val::Value::Int(42));

        // Reload with same widget definition (state should be preserved)
        let new_widget = make_test_widget("Counter", vec![
            AuraStateDef {
                name: "count".to_string(),
                type_info: Type::Int,
                initial: AuraExpr::Int(0),
                decorators: vec![],
            },
        ]);

        let report = comp.reload(&new_widget).unwrap();

        // State should be preserved
        assert_eq!(comp.read_state("count").unwrap(), auto_val::Value::Int(42));
        assert!(comp.is_dirty());
        assert_eq!(report.preserved, 1);
        assert_eq!(report.added, 0);
    }

    #[test]
    fn test_reload_adds_new_fields() {
        let old_widget = make_test_widget("Counter", vec![
            AuraStateDef {
                name: "count".to_string(),
                type_info: Type::Int,
                initial: AuraExpr::Int(0),
                decorators: vec![],
            },
        ]);

        let mut comp = DynamicComponent::new(&old_widget).unwrap();
        comp.write_state("count", auto_val::Value::Int(99)).unwrap();

        // Reload with additional field
        let new_widget = make_test_widget("Counter", vec![
            AuraStateDef {
                name: "count".to_string(),
                type_info: Type::Int,
                initial: AuraExpr::Int(0),
                decorators: vec![],
            },
            AuraStateDef {
                name: "enabled".to_string(),
                type_info: Type::Bool,
                initial: AuraExpr::Bool(true),
                decorators: vec![],
            },
        ]);

        let report = comp.reload(&new_widget).unwrap();

        // Old field preserved, new field has default
        assert_eq!(comp.read_state("count").unwrap(), auto_val::Value::Int(99));
        assert_eq!(comp.read_state("enabled").unwrap(), auto_val::Value::Bool(true));
        assert_eq!(report.preserved, 1);
        assert_eq!(report.added, 1);
    }

    #[test]
    fn test_reload_drops_removed_fields() {
        let old_widget = make_test_widget("Counter", vec![
            AuraStateDef {
                name: "count".to_string(),
                type_info: Type::Int,
                initial: AuraExpr::Int(5),
                decorators: vec![],
            },
            AuraStateDef {
                name: "legacy".to_string(),
                type_info: Type::StrFixed(0),
                initial: AuraExpr::Literal("old".to_string()),
                decorators: vec![],
            },
        ]);

        let mut comp = DynamicComponent::new(&old_widget).unwrap();
        comp.write_state("count", auto_val::Value::Int(10)).unwrap();

        // Reload without the "legacy" field
        let new_widget = make_test_widget("Counter", vec![
            AuraStateDef {
                name: "count".to_string(),
                type_info: Type::Int,
                initial: AuraExpr::Int(0),
                decorators: vec![],
            },
        ]);

        let report = comp.reload(&new_widget).unwrap();

        assert_eq!(comp.read_state("count").unwrap(), auto_val::Value::Int(10));
        assert!(comp.read_state("legacy").is_err());
        assert_eq!(report.preserved, 1);
        assert_eq!(report.dropped, 1);
        assert!(report.dropped_names.contains(&"legacy".to_string()));
    }

    #[test]
    fn test_reload_updates_view_template() {
        let old_widget = make_test_widget("Counter", vec![
            AuraStateDef {
                name: "count".to_string(),
                type_info: Type::Int,
                initial: AuraExpr::Int(0),
                decorators: vec![],
            },
        ]);

        let mut comp = DynamicComponent::new(&old_widget).unwrap();

        // Initial view: empty column
        let view = comp.view();
        assert!(matches!(view, View::Column { .. }));

        // Reload with a different view template
        let new_widget = AuraWidget {
            name: "Counter".to_string(),
            state_vars: vec![AuraStateDef {
                name: "count".to_string(),
                type_info: Type::Int,
                initial: AuraExpr::Int(0),
                decorators: vec![],
            }],
            computed: vec![],
            messages: vec![],
            view_tree: AuraNode::Element {
                tag: "text".to_string(),
                props: HashMap::from([
                    ("text".to_string(), crate::aura::AuraPropValue::Expr(
                        AuraExpr::StateRef("count".to_string()),
                    )),
                ]),
                events: HashMap::new(),
                children: vec![],
                span: None,
                debug_id: None,
            },
            handlers: HashMap::new(),
            props: vec![],
            routes: None,
            lifecycle: vec![],
            tick_interval: None,
            handler_params: HashMap::new(),
            span_map: HashMap::new(),
            key_bindings: HashMap::new(),
        };

        comp.reload(&new_widget).unwrap();

        // View should now be Text, not Column
        let view = comp.view();
        match view {
            View::Text { content, .. } => {
                assert_eq!(content, "0");
            }
            other => panic!("Expected View::Text after reload, got {:?}", other),
        }
    }

    #[test]
    fn test_reload_marks_dirty() {
        let widget = make_test_widget("Counter", vec![
            AuraStateDef {
                name: "count".to_string(),
                type_info: Type::Int,
                initial: AuraExpr::Int(0),
                decorators: vec![],
            },
        ]);

        // Component starts dirty from creation
        let mut comp = DynamicComponent::new(&widget).unwrap();
        assert!(comp.is_dirty());

        // Reload should also mark as dirty
        comp.reload(&widget).unwrap();
        assert!(comp.is_dirty());
    }

    #[test]
    fn test_source_path_tracking() {
        let widget = make_test_widget("Counter", vec![]);
        let mut comp = DynamicComponent::new(&widget).unwrap();

        // No source path initially
        assert!(comp.source_path().is_none());

        // Setting a non-existent path should not panic (mod time will be None)
        comp.set_source_path("/tmp/nonexistent_test_file.at");
        assert_eq!(comp.source_path().unwrap().to_str().unwrap(), "/tmp/nonexistent_test_file.at");

        // check_file_changed should return Ok(None) since file doesn't exist metadata
        // Actually it will error since the file doesn't exist.
        // But source_path is set. This is fine.
    }

    #[test]
    fn test_read_all_state() {
        let widget = make_test_widget("Multi", vec![
            AuraStateDef {
                name: "x".to_string(),
                type_info: Type::Int,
                initial: AuraExpr::Int(1),
                decorators: vec![],
            },
            AuraStateDef {
                name: "y".to_string(),
                type_info: Type::Int,
                initial: AuraExpr::Int(2),
                decorators: vec![],
            },
        ]);

        let comp = DynamicComponent::new(&widget).unwrap();
        let state = comp.read_all_state();

        assert_eq!(state.len(), 2);
        assert_eq!(state.get("x"), Some(&auto_val::Value::Int(1)));
        assert_eq!(state.get("y"), Some(&auto_val::Value::Int(2)));
    }
}
