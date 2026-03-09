//! Form Component Generators
//!
//! Generates Jetpack Compose form components from AURA elements.
//!
//! ## Supported Components
//! - `input` → `OutlinedTextField`
//! - `textarea` → `OutlinedTextField` (multi-line)
//! - `checkbox` → `Checkbox`
//! - `switch`/`toggle` → `Switch`
//! - `slider` → `Slider`

use crate::aura::{AuraPropValue, AuraExpr};
use crate::ui_gen::GenResult;
use std::collections::HashMap;

/// Form component generator
pub struct FormGenerator {
    /// Track imports needed for form components
    imports: Vec<String>,
}

impl FormGenerator {
    /// Create a new form generator
    pub fn new() -> Self {
        Self {
            imports: Vec::new(),
        }
    }

    /// Get required imports for generated form components
    pub fn get_imports(&self) -> &[String] {
        &self.imports
    }

    /// Clear imports for fresh generation
    pub fn clear_imports(&mut self) {
        self.imports.clear();
    }

    /// Add import if not already present
    pub fn add_import(&mut self, import: &str) {
        if !self.imports.iter().any(|i| i == import) {
            self.imports.push(import.to_string());
        }
    }

    /// Extract string value from prop
    fn extract_string(props: &HashMap<String, AuraPropValue>, key: &str) -> Option<String> {
        props.get(key).and_then(|p| match p {
            AuraPropValue::Expr(AuraExpr::Literal(s)) => Some(s.clone()),
            AuraPropValue::Expr(AuraExpr::StateRef(s)) => Some(s.clone()),
            _ => None,
        })
    }

    /// Extract bool value from prop
    fn extract_bool(props: &HashMap<String, AuraPropValue>, key: &str) -> bool {
        props.get(key).and_then(|p| match p {
            AuraPropValue::Expr(AuraExpr::Bool(b)) => Some(*b),
            _ => None,
        }).unwrap_or(false)
    }

    /// Extract int value from prop
    fn extract_int(props: &HashMap<String, AuraPropValue>, key: &str) -> Option<i64> {
        props.get(key).and_then(|p| match p {
            AuraPropValue::Expr(AuraExpr::Int(n)) => Some(*n),
            _ => None,
        })
    }

    /// Extract state reference from prop
    fn extract_state_ref(props: &HashMap<String, AuraPropValue>, key: &str) -> Option<String> {
        props.get(key).and_then(|p| match p {
            AuraPropValue::Expr(AuraExpr::StateRef(s)) => Some(s.clone()),
            _ => None,
        })
    }

    /// Generate OutlinedTextField for input element
    pub fn generate_input(&mut self, props: &HashMap<String, AuraPropValue>) -> GenResult<String> {
        self.add_import("androidx.compose.material3.OutlinedTextField");
        self.add_import("androidx.compose.material3.Text");
        self.add_import("androidx.compose.ui.Modifier");

        let mut parts = Vec::new();

        // Value binding (required)
        if let Some(state_ref) = Self::extract_state_ref(props, "value") {
            parts.push(format!("value = {}", state_ref));
            parts.push(format!("onValueChange = {{ {} = it }}", state_ref));
        } else {
            parts.push("value = \"\"".to_string());
            parts.push("onValueChange = { }".to_string());
        }

        // Placeholder
        if let Some(placeholder) = Self::extract_string(props, "placeholder") {
            parts.push(format!("placeholder = {{ Text(\"{}\") }}", placeholder));
        }

        // Label
        if let Some(label) = Self::extract_string(props, "label") {
            parts.push(format!("label = {{ Text(\"{}\") }}", label));
        }

        // Enabled/disabled
        if Self::extract_bool(props, "disabled") {
            parts.push("enabled = false".to_string());
        }

        // Type-specific options
        if let Some(type_val) = Self::extract_string(props, "type") {
            match type_val.as_str() {
                "password" => {
                    self.add_import("androidx.compose.ui.text.input.PasswordVisualTransformation");
                    parts.push("visualTransformation = PasswordVisualTransformation()".to_string());
                }
                "email" => {
                    self.add_import("androidx.compose.foundation.text.KeyboardOptions");
                    self.add_import("androidx.compose.ui.text.input.KeyboardType");
                    parts.push("keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Email)".to_string());
                }
                "number" => {
                    self.add_import("androidx.compose.foundation.text.KeyboardOptions");
                    self.add_import("androidx.compose.ui.text.input.KeyboardType");
                    parts.push("keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number)".to_string());
                }
                _ => {}
            }
        }

        parts.push("singleLine = true".to_string());
        parts.push("modifier = Modifier.fillMaxWidth()".to_string());

        Ok(format!("OutlinedTextField(\n        {}\n    )", parts.join(",\n        ")))
    }

    /// Generate OutlinedTextField for textarea element (multi-line)
    pub fn generate_textarea(&mut self, props: &HashMap<String, AuraPropValue>) -> GenResult<String> {
        self.add_import("androidx.compose.material3.OutlinedTextField");
        self.add_import("androidx.compose.material3.Text");
        self.add_import("androidx.compose.ui.Modifier");

        let mut parts = Vec::new();

        if let Some(state_ref) = Self::extract_state_ref(props, "value") {
            parts.push(format!("value = {}", state_ref));
            parts.push(format!("onValueChange = {{ {} = it }}", state_ref));
        } else {
            parts.push("value = \"\"".to_string());
            parts.push("onValueChange = { }".to_string());
        }

        if let Some(placeholder) = Self::extract_string(props, "placeholder") {
            parts.push(format!("placeholder = {{ Text(\"{}\") }}", placeholder));
        }

        if let Some(label) = Self::extract_string(props, "label") {
            parts.push(format!("label = {{ Text(\"{}\") }}", label));
        }

        if let Some(rows) = Self::extract_int(props, "rows") {
            parts.push(format!("maxLines = {}", rows));
        }

        if Self::extract_bool(props, "disabled") {
            parts.push("enabled = false".to_string());
        }

        parts.push("modifier = Modifier.fillMaxWidth()".to_string());

        Ok(format!("OutlinedTextField(\n        {}\n    )", parts.join(",\n        ")))
    }

    /// Generate Checkbox component
    pub fn generate_checkbox(&mut self, props: &HashMap<String, AuraPropValue>) -> GenResult<String> {
        self.add_import("androidx.compose.material3.Checkbox");
        self.add_import("androidx.compose.material3.Text");
        self.add_import("androidx.compose.foundation.layout.Row");
        self.add_import("androidx.compose.foundation.layout.Spacer");
        self.add_import("androidx.compose.foundation.layout.width");
        self.add_import("androidx.compose.ui.Alignment");
        self.add_import("androidx.compose.ui.Modifier");
        self.add_import("androidx.compose.ui.unit.dp");

        let state_ref = Self::extract_state_ref(props, "checked").unwrap_or_else(|| "checked".to_string());
        let label = Self::extract_string(props, "label");
        let disabled = Self::extract_bool(props, "disabled");

        let mut checkbox_parts = vec![
            format!("checked = {}", state_ref),
            format!("onCheckedChange = {{ {} = it }}", state_ref),
        ];
        if disabled {
            checkbox_parts.push("enabled = false".to_string());
        }

        // If there's a label, wrap in a Row
        if let Some(label_text) = label {
            Ok(format!(
r#"Row(
        verticalAlignment = Alignment.CenterVertically,
        modifier = Modifier.fillMaxWidth()
    ) {{
        Checkbox(
            {}
        )
        Spacer(modifier = Modifier.width(8.dp))
        Text("{}")
    }}"#,
                checkbox_parts.join(",\n            "),
                label_text
            ))
        } else {
            Ok(format!("Checkbox(\n        {}\n    )", checkbox_parts.join(",\n        ")))
        }
    }

    /// Generate Switch component
    pub fn generate_switch(&mut self, props: &HashMap<String, AuraPropValue>) -> GenResult<String> {
        self.add_import("androidx.compose.material3.Switch");
        self.add_import("androidx.compose.material3.Text");
        self.add_import("androidx.compose.foundation.layout.Row");
        self.add_import("androidx.compose.foundation.layout.Arrangement");
        self.add_import("androidx.compose.ui.Alignment");
        self.add_import("androidx.compose.ui.Modifier");

        let state_ref = Self::extract_state_ref(props, "checked").unwrap_or_else(|| "checked".to_string());
        let label = Self::extract_string(props, "label");
        let disabled = Self::extract_bool(props, "disabled");

        let mut switch_parts = vec![
            format!("checked = {}", state_ref),
            format!("onCheckedChange = {{ {} = it }}", state_ref),
        ];
        if disabled {
            switch_parts.push("enabled = false".to_string());
        }

        // If there's a label, wrap in a Row with space between
        if let Some(label_text) = label {
            Ok(format!(
r#"Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.SpaceBetween,
        modifier = Modifier.fillMaxWidth()
    ) {{
        Text("{}")
        Switch(
            {}
        )
    }}"#,
                label_text,
                switch_parts.join(",\n            ")
            ))
        } else {
            Ok(format!("Switch(\n        {}\n    )", switch_parts.join(",\n        ")))
        }
    }

    /// Generate Slider component
    pub fn generate_slider(&mut self, props: &HashMap<String, AuraPropValue>) -> GenResult<String> {
        self.add_import("androidx.compose.material3.Slider");
        self.add_import("androidx.compose.ui.Modifier");

        let state_ref = Self::extract_state_ref(props, "value").unwrap_or_else(|| "value".to_string());

        let min = Self::extract_int(props, "min").unwrap_or(0) as f32;
        let max = Self::extract_int(props, "max").unwrap_or(100) as f32;
        let step = Self::extract_int(props, "step");

        let mut parts = vec![
            format!("value = {}", state_ref),
            format!("onValueChange = {{ {} = it }}", state_ref),
            format!("valueRange = {}f..{}f", min, max),
        ];

        // If step is provided, add steps (steps = discrete points - 1)
        if let Some(step_val) = step {
            if step_val > 0 {
                let steps = ((max - min) / step_val as f32) as i64 - 1;
                if steps > 0 {
                    parts.push(format!("steps = {}", steps));
                }
            }
        }

        parts.push("modifier = Modifier.fillMaxWidth()".to_string());

        Ok(format!("Slider(\n        {}\n    )", parts.join(",\n        ")))
    }
}

impl Default for FormGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aura::{AuraPropValue, AuraExpr};
    use std::collections::HashMap;

    #[test]
    fn test_generate_input_basic() {
        let mut gen = FormGenerator::new();
        let mut props = HashMap::new();

        props.insert("value".to_string(), AuraPropValue::Expr(AuraExpr::StateRef("text".to_string())));

        let result = gen.generate_input(&props);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("OutlinedTextField"));
        assert!(code.contains("value = text"));
        assert!(code.contains("onValueChange = { text = it }"));
    }

    #[test]
    fn test_generate_input_with_placeholder() {
        let mut gen = FormGenerator::new();
        let mut props = HashMap::new();

        props.insert("value".to_string(), AuraPropValue::Expr(AuraExpr::StateRef("email".to_string())));
        props.insert("placeholder".to_string(), AuraPropValue::Expr(AuraExpr::Literal("Enter email".to_string())));

        let result = gen.generate_input(&props);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("placeholder = { Text(\"Enter email\") }"));
    }

    #[test]
    fn test_generate_input_with_label() {
        let mut gen = FormGenerator::new();
        let mut props = HashMap::new();

        props.insert("value".to_string(), AuraPropValue::Expr(AuraExpr::StateRef("name".to_string())));
        props.insert("label".to_string(), AuraPropValue::Expr(AuraExpr::Literal("Name".to_string())));

        let result = gen.generate_input(&props);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("label = { Text(\"Name\") }"));
    }

    #[test]
    fn test_generate_input_password_type() {
        let mut gen = FormGenerator::new();
        let mut props = HashMap::new();

        props.insert("value".to_string(), AuraPropValue::Expr(AuraExpr::StateRef("password".to_string())));
        props.insert("type".to_string(), AuraPropValue::Expr(AuraExpr::Literal("password".to_string())));

        let result = gen.generate_input(&props);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("visualTransformation = PasswordVisualTransformation()"));
    }

    #[test]
    fn test_generate_input_disabled() {
        let mut gen = FormGenerator::new();
        let mut props = HashMap::new();

        props.insert("value".to_string(), AuraPropValue::Expr(AuraExpr::StateRef("text".to_string())));
        props.insert("disabled".to_string(), AuraPropValue::Expr(AuraExpr::Bool(true)));

        let result = gen.generate_input(&props);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("enabled = false"));
    }

    #[test]
    fn test_generate_textarea() {
        let mut gen = FormGenerator::new();
        let mut props = HashMap::new();

        props.insert("value".to_string(), AuraPropValue::Expr(AuraExpr::StateRef("content".to_string())));
        props.insert("rows".to_string(), AuraPropValue::Expr(AuraExpr::Int(5)));

        let result = gen.generate_textarea(&props);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("maxLines = 5"));
    }

    #[test]
    fn test_generate_checkbox_basic() {
        let mut gen = FormGenerator::new();
        let mut props = HashMap::new();

        props.insert("checked".to_string(), AuraPropValue::Expr(AuraExpr::StateRef("done".to_string())));

        let result = gen.generate_checkbox(&props);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("Checkbox"));
        assert!(code.contains("checked = done"));
        assert!(code.contains("onCheckedChange = { done = it }"));
    }

    #[test]
    fn test_generate_checkbox_with_label() {
        let mut gen = FormGenerator::new();
        let mut props = HashMap::new();

        props.insert("checked".to_string(), AuraPropValue::Expr(AuraExpr::StateRef("agree".to_string())));
        props.insert("label".to_string(), AuraPropValue::Expr(AuraExpr::Literal("I agree".to_string())));

        let result = gen.generate_checkbox(&props);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("Row"));
        assert!(code.contains("Checkbox"));
        assert!(code.contains("Text(\"I agree\")"));
    }

    #[test]
    fn test_generate_switch_basic() {
        let mut gen = FormGenerator::new();
        let mut props = HashMap::new();

        props.insert("checked".to_string(), AuraPropValue::Expr(AuraExpr::StateRef("enabled".to_string())));

        let result = gen.generate_switch(&props);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("Switch"));
        assert!(code.contains("checked = enabled"));
        assert!(code.contains("onCheckedChange = { enabled = it }"));
    }

    #[test]
    fn test_generate_switch_with_label() {
        let mut gen = FormGenerator::new();
        let mut props = HashMap::new();

        props.insert("checked".to_string(), AuraPropValue::Expr(AuraExpr::StateRef("notifications".to_string())));
        props.insert("label".to_string(), AuraPropValue::Expr(AuraExpr::Literal("Enable notifications".to_string())));

        let result = gen.generate_switch(&props);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("Row"));
        assert!(code.contains("Text(\"Enable notifications\")"));
        assert!(code.contains("Switch"));
    }

    #[test]
    fn test_generate_slider_basic() {
        let mut gen = FormGenerator::new();
        let mut props = HashMap::new();

        props.insert("value".to_string(), AuraPropValue::Expr(AuraExpr::StateRef("volume".to_string())));

        let result = gen.generate_slider(&props);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("Slider"));
        assert!(code.contains("value = volume"));
        assert!(code.contains("onValueChange = { volume = it }"));
    }

    #[test]
    fn test_generate_slider_with_range() {
        let mut gen = FormGenerator::new();
        let mut props = HashMap::new();

        props.insert("value".to_string(), AuraPropValue::Expr(AuraExpr::StateRef("progress".to_string())));
        props.insert("min".to_string(), AuraPropValue::Expr(AuraExpr::Int(0)));
        props.insert("max".to_string(), AuraPropValue::Expr(AuraExpr::Int(100)));

        let result = gen.generate_slider(&props);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("valueRange = 0f..100f"));
    }

    #[test]
    fn test_generate_slider_with_step() {
        let mut gen = FormGenerator::new();
        let mut props = HashMap::new();

        props.insert("value".to_string(), AuraPropValue::Expr(AuraExpr::StateRef("rating".to_string())));
        props.insert("min".to_string(), AuraPropValue::Expr(AuraExpr::Int(0)));
        props.insert("max".to_string(), AuraPropValue::Expr(AuraExpr::Int(10)));
        props.insert("step".to_string(), AuraPropValue::Expr(AuraExpr::Int(1)));

        let result = gen.generate_slider(&props);
        assert!(result.is_ok());
        let code = result.unwrap();
        // For range 0-10 with step 1, steps = (10-0)/1 - 1 = 9
        assert!(code.contains("steps = 9"));
    }
}
