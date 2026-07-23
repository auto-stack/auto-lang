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
}

impl WidgetRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            widgets: HashMap::new(),
        }
    }

    /// Register a widget definition.
    pub fn register(&mut self, widget: AuraWidget) {
        self.widgets.insert(widget.name.clone(), widget);
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
