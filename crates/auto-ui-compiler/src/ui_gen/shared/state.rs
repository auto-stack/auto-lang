//! State Analyzer
//!
//! Analyzes AURA widget state (model, computed, msg) and generates
//! platform-specific state management code.
//!
//! ## AURA State Model
//!
//! ```auto
//! widget Counter {
//!     msg Msg { Increment, Decrement, SetValue(v: int) }
//!
//!     model {
//!         count int = 0
//!         name str = "Counter"
//!     }
//!
//!     computed {
//!         double => .count * 2
//!         displayText => f"Count: ${.count}"
//!     }
//!
//!     view { ... }
//!
//!     on {
//!         Increment => { count = count + 1 }
//!         Decrement => { count = count - 1 }
//!     }
//! }
//! ```

use std::collections::HashMap;

/// Analyzed widget state
#[derive(Debug, Clone, Default)]
pub struct WidgetState {
    /// Model properties
    pub model: HashMap<String, ModelProperty>,
    /// Computed properties
    pub computed: HashMap<String, ComputedProperty>,
    /// Message definitions
    pub messages: HashMap<String, MessageDef>,
    /// Event handlers
    pub handlers: HashMap<String, EventHandler>,
}

/// Model property definition
#[derive(Debug, Clone)]
pub struct ModelProperty {
    /// Property name
    pub name: String,
    /// Type annotation
    pub type_annotation: String,
    /// Default value (as string)
    pub default_value: Option<String>,
    /// Is the property mutable
    pub mutable: bool,
}

/// Computed property definition
#[derive(Debug, Clone)]
pub struct ComputedProperty {
    /// Property name
    pub name: String,
    /// Expression (as string)
    pub expression: String,
    /// Dependencies (model properties used)
    pub dependencies: Vec<String>,
}

/// Message definition
#[derive(Debug, Clone)]
pub struct MessageDef {
    /// Message name
    pub name: String,
    /// Payload type (if any)
    pub payload: Option<String>,
    /// Payload parameter name
    pub payload_name: Option<String>,
}

/// Event handler definition
#[derive(Debug, Clone)]
pub struct EventHandler {
    /// Message name being handled
    pub message: String,
    /// Handler body (as string)
    pub body: String,
}

/// State analyzer
pub struct StateAnalyzer {
    /// Current widget state
    state: WidgetState,
}

impl StateAnalyzer {
    /// Create a new state analyzer
    pub fn new() -> Self {
        Self {
            state: WidgetState::default(),
        }
    }

    /// Analyze a widget and return its state
    pub fn analyze(&mut self) -> &WidgetState {
        &self.state
    }

    /// Add a model property
    pub fn add_model(&mut self, prop: ModelProperty) {
        self.state.model.insert(prop.name.clone(), prop);
    }

    /// Add a computed property
    pub fn add_computed(&mut self, prop: ComputedProperty) {
        self.state.computed.insert(prop.name.clone(), prop);
    }

    /// Add a message definition
    pub fn add_message(&mut self, msg: MessageDef) {
        self.state.messages.insert(msg.name.clone(), msg);
    }

    /// Add an event handler
    pub fn add_handler(&mut self, handler: EventHandler) {
        self.state.handlers.insert(handler.message.clone(), handler);
    }

    /// Reset state for a new widget
    pub fn reset(&mut self) {
        self.state = WidgetState::default();
    }

    /// Generate Vue state code
    pub fn generate_vue_state(&self) -> String {
        let mut lines = Vec::new();

        // Model properties -> ref()
        for (_, prop) in &self.state.model {
            let default = prop.default_value.as_deref().unwrap_or("0");
            lines.push(format!("const {} = ref({})", prop.name, default));
        }

        // Computed properties -> computed()
        if !self.state.computed.is_empty() {
            lines.push("".to_string());
            for (_, prop) in &self.state.computed {
                lines.push(format!(
                    "const {} = computed(() => {})",
                    prop.name, prop.expression
                ));
            }
        }

        lines.join("\n")
    }

    /// Generate Jetpack Compose state code
    pub fn generate_jet_state(&self) -> String {
        let mut lines = Vec::new();

        // Model properties -> var by remember { mutableStateOf() }
        for (_, prop) in &self.state.model {
            let default = prop.default_value.as_deref().unwrap_or("0");
            let jet_default = Self::convert_default_to_jet(&prop.type_annotation, default);
            lines.push(format!(
                "var {} by remember {{ mutableStateOf({}) }}",
                prop.name, jet_default
            ));
        }

        // Computed properties -> derivedStateOf
        if !self.state.computed.is_empty() {
            lines.push("".to_string());
            for (_, prop) in &self.state.computed {
                let jet_expr = Self::convert_expr_to_jet(&prop.expression);
                lines.push(format!(
                    "val {} by derivedStateOf {{ {} }}",
                    prop.name, jet_expr
                ));
            }
        }

        lines.join("\n")
    }

    /// Generate Vue event handlers
    pub fn generate_vue_handlers(&self) -> String {
        let mut lines = Vec::new();

        for (_, handler) in &self.state.handlers {
            let vue_body = Self::convert_handler_to_vue(&handler.body);
            lines.push(format!(
                "const handle{} = () => {{\n  {}\n}}",
                handler.message, vue_body
            ));
        }

        lines.join("\n\n")
    }

    /// Generate Jetpack Compose event handlers
    pub fn generate_jet_handlers(&self) -> String {
        let mut lines = Vec::new();

        for (_, handler) in &self.state.handlers {
            let jet_body = Self::convert_handler_to_jet(&handler.body);
            lines.push(format!(
                "val on{} = {{\n  {}\n}}",
                handler.message, jet_body
            ));
        }

        lines.join("\n\n")
    }

    /// Convert default value to Jet syntax
    fn convert_default_to_jet(type_annotation: &str, default: &str) -> String {
        match type_annotation {
            "int" => default.to_string(),
            "float" | "double" => format!("{}f", default),
            "bool" => default.to_string(),
            "str" => format!("\"{}\"", default.trim_matches('"')),
            _ => default.to_string(),
        }
    }

    /// Convert expression to Jet syntax
    fn convert_expr_to_jet(expr: &str) -> String {
        let mut result = expr.to_string();

        // .property -> property
        result = result.replace(".count", "count");
        result = result.replace(".name", "name");

        // f"..." -> "..."
        if result.starts_with("f\"") {
            result = result[2..].to_string();
            // ${.prop} -> $prop
            result = result.replace("${.", "$");
        }

        result
    }

    /// Convert handler body to Vue syntax
    fn convert_handler_to_vue(body: &str) -> String {
        let mut result = body.to_string();

        // property = value -> property.value = value
        for prop_name in ["count", "name", "text", "value", "selected"] {
            result = result.replace(
                &format!("{} = ", prop_name),
                &format!("{}.value = ", prop_name),
            );
        }

        result
    }

    /// Convert handler body to Jet syntax
    fn convert_handler_to_jet(body: &str) -> String {
        let mut result = body.to_string();

        // property = value -> property = value (already correct for Jet)
        // Just need to handle increment/decrement
        result = result.replace("count + 1", "count + 1");
        result = result.replace("count - 1", "count - 1");

        result
    }
}

impl Default for StateAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vue_state_generation() {
        let mut analyzer = StateAnalyzer::new();
        analyzer.add_model(ModelProperty {
            name: "count".to_string(),
            type_annotation: "int".to_string(),
            default_value: Some("0".to_string()),
            mutable: true,
        });
        analyzer.add_computed(ComputedProperty {
            name: "double".to_string(),
            expression: ".count * 2".to_string(),
            dependencies: vec!["count".to_string()],
        });

        let vue_code = analyzer.generate_vue_state();
        assert!(vue_code.contains("const count = ref(0)"));
        assert!(vue_code.contains("const double = computed"));
    }

    #[test]
    fn test_jet_state_generation() {
        let mut analyzer = StateAnalyzer::new();
        analyzer.add_model(ModelProperty {
            name: "count".to_string(),
            type_annotation: "int".to_string(),
            default_value: Some("0".to_string()),
            mutable: true,
        });
        analyzer.add_computed(ComputedProperty {
            name: "double".to_string(),
            expression: ".count * 2".to_string(),
            dependencies: vec!["count".to_string()],
        });

        let jet_code = analyzer.generate_jet_state();
        assert!(jet_code.contains("var count by remember"));
        assert!(jet_code.contains("mutableStateOf(0)"));
        assert!(jet_code.contains("val double by derivedStateOf"));
    }

    #[test]
    fn test_message_with_payload() {
        let mut analyzer = StateAnalyzer::new();
        analyzer.add_message(MessageDef {
            name: "SetValue".to_string(),
            payload: Some("int".to_string()),
            payload_name: Some("v".to_string()),
        });

        assert!(analyzer.state.messages.contains_key("SetValue"));
    }
}
