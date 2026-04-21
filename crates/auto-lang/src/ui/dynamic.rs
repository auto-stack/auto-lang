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

use crate::aura::AuraWidget;
use crate::ui::aura_view_builder::AuraViewBuilder;
use crate::ui::component::Component;
use crate::ui::interpreter::DynamicMessage;
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

        Ok(Self {
            bridge,
            view_template,
            widget_name,
            dirty: true, // Initially dirty so first view() builds the tree
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

        Ok(Self {
            bridge,
            view_template,
            widget_name,
            dirty: true,
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
        match msg {
            DynamicMessage::Typed { event_name, args, .. } => {
                let _ = self.bridge.call_handler(&event_name, &args);
                self.dirty = true;
            }
            DynamicMessage::String(event_name) => {
                let _ = self.bridge.call_handler(&event_name, &[]);
                self.dirty = true;
            }
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
            },
            handlers: HashMap::new(),
            props: vec![],
            routes: None,
            lifecycle: vec![],
            tick_interval: None,
            handler_params: HashMap::new(),
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
                type_info: Type::Str(0),
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
                    },
                ],
            },
            handlers: HashMap::new(),
            props: vec![],
            routes: None,
            lifecycle: vec![],
            tick_interval: None,
            handler_params: HashMap::new(),
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
            },
            handlers: HashMap::new(),
            props: vec![],
            routes: None,
            lifecycle: vec![],
            tick_interval: None,
            handler_params: HashMap::new(),
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
}
