//! Widget Registry for the Auto UI Interpreter
//!
//! Stores child widget definitions loaded from `use` imports,
//! enabling the interpreter to render custom component tags.

use std::collections::HashMap;
use crate::aura::AuraWidget;

/// Registry mapping widget names to their AuraWidget definitions.
#[derive(Debug, Default)]
pub struct WidgetRegistry {
    widgets: HashMap<String, AuraWidget>,
    /// Plan 408: route module → widget name alias map.
    /// Maps e.g. "button" → "ButtonPage" so render_outlet can find the page
    /// widget by route.module. Stored separately from `widgets` to avoid
    /// name collisions with built-in UI elements (e.g. a route module named
    /// "button" must not shadow the `<button>` element).
    route_aliases: HashMap<String, String>,
}

impl WidgetRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            widgets: HashMap::new(),
            route_aliases: HashMap::new(),
        }
    }

    /// Register a widget definition.
    pub fn register(&mut self, widget: AuraWidget) {
        self.widgets.insert(widget.name.clone(), widget);
    }

    /// Plan 408: Register a route module alias (module name → widget name).
    /// Used by render_outlet to look up page widgets by route.module.
    pub fn register_route_alias(&mut self, module: &str, widget_name: &str) {
        self.route_aliases.insert(module.to_string(), widget_name.to_string());
    }

    /// Plan 408: Look up a widget by route module name via the alias map.
    /// Returns the widget definition if the module alias exists and the
    /// target widget is registered.
    pub fn get_by_route_module(&self, module: &str) -> Option<&AuraWidget> {
        let widget_name = self.route_aliases.get(module)?;
        self.widgets.get(widget_name)
    }

    /// Look up a widget by name.
    pub fn get(&self, name: &str) -> Option<&AuraWidget> {
        self.widgets.get(name)
    }

    /// Check if a widget with the given name is registered.
    pub fn contains(&self, name: &str) -> bool {
        self.widgets.contains_key(name)
    }

    /// Number of registered widgets.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.widgets.len()
    }

    /// Whether the registry is empty.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.widgets.is_empty()
    }

    /// Plan 320: iterate all registered widgets (for single-VM compilation).
    pub fn all(&self) -> impl Iterator<Item = AuraWidget> + '_ {
        self.widgets.values().cloned()
    }
}
