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
//! - `chip` → `AssistChip`, `FilterChip`, `InputChip`, `SuggestionChip`
//! - `progress` → `CircularProgressIndicator`, `LinearProgressIndicator`
//! - `image` → `AsyncImage` (Coil)
//! - `badge` → `Badge`

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

    /// Generate Progress indicator component
    ///
    /// # Variants
    /// - `"circular"` (default) → `CircularProgressIndicator`
    /// - `"linear"` → `LinearProgressIndicator`
    ///
    /// # Modes
    /// - **Indeterminate** (default): No `value` prop - shows continuous animation
    /// - **Determinate**: With `value` prop (0.0-1.0) - shows specific progress
    ///
    /// # Props
    /// - `type`: "circular" (default) or "linear"
    /// - `value`: Progress value (0.0-1.0), optional. If absent, indeterminate mode
    /// - `color`: Custom progress color (optional)
    ///
    /// # Examples
    /// ```auto
    /// Progress {}                           // Circular indeterminate
    /// Progress (type: "linear") {}          // Linear indeterminate
    /// Progress (value: 0.7) {}              // Circular determinate (70%)
    /// Progress (type: "linear", value: 0.5) {} // Linear determinate (50%)
    /// ```
    pub fn generate_progress(&mut self, props: &HashMap<String, AuraPropValue>) -> GenResult<String> {
        // Extract type (default: circular)
        let progress_type = Self::extract_string(props, "type")
            .unwrap_or_else(|| "circular".to_string());

        // Extract value (optional - if present, determinate mode)
        let value = Self::extract_float(props, "value");

        // Add imports based on type
        match progress_type.as_str() {
            "linear" => {
                self.add_import("androidx.compose.material3.LinearProgressIndicator");
            }
            _ => {
                self.add_import("androidx.compose.material3.CircularProgressIndicator");
            }
        }

        // Build progress indicator
        let mut parts = Vec::new();

        // Determinate mode: add progress parameter
        if let Some(v) = value {
            parts.push(format!("progress = {}f", v));
        }

        // Custom color (optional)
        if let Some(color) = Self::extract_string(props, "color") {
            self.add_import("androidx.compose.ui.graphics.Color");
            parts.push(format!("color = Color({})", Self::parse_color(&color)));
        }

        // Generate based on type
        match progress_type.as_str() {
            "linear" => {
                if parts.is_empty() {
                    Ok("LinearProgressIndicator()".to_string())
                } else {
                    Ok(format!("LinearProgressIndicator(\n        {}\n    )", parts.join(",\n        ")))
                }
            }
            _ => {
                if parts.is_empty() {
                    Ok("CircularProgressIndicator()".to_string())
                } else {
                    Ok(format!("CircularProgressIndicator(\n        {}\n    )", parts.join(",\n        ")))
                }
            }
        }
    }

    /// Extract float value from prop
    fn extract_float(props: &HashMap<String, AuraPropValue>, key: &str) -> Option<f64> {
        props.get(key).and_then(|p| match p {
            AuraPropValue::Expr(AuraExpr::Float(n)) => Some(*n),
            AuraPropValue::Expr(AuraExpr::Int(n)) => Some(*n as f64),
            _ => None,
        })
    }

    /// Parse color string to Compose Color format
    /// Supports: hex (#RRGGBB, #AARRGGBB), named colors (blue, red, etc.)
    fn parse_color(color: &str) -> String {
        let color = color.trim();

        // Handle hex colors
        if color.starts_with('#') {
            let hex = color.trim_start_matches('#');
            match hex.len() {
                6 => format!("0xFF{}", hex.to_uppercase()),
                8 => format!("0x{}", hex.to_uppercase()),
                _ => color.to_string(),
            }
        } else {
            // Named colors
            match color.to_lowercase().as_str() {
                "blue" => "Color.Blue".to_string(),
                "red" => "Color.Red".to_string(),
                "green" => "Color.Green".to_string(),
                "yellow" => "Color.Yellow".to_string(),
                "black" => "Color.Black".to_string(),
                "white" => "Color.White".to_string(),
                "gray" | "grey" => "Color.Gray".to_string(),
                _ => color.to_string(),
            }
        }
    }

    /// Generate Image component
    ///
    /// Uses Coil's AsyncImage for loading network images.
    ///
    /// # Props
    /// - `src`: Image URL or resource path (required)
    /// - `contentDescription`: Accessibility description (optional, defaults to "Image")
    /// - `modifier`: Additional modifier (optional)
    ///
    /// # Examples
    /// ```auto
    /// Image (src: "https://example.com/image.png")
    /// Image (src: .avatarUrl, contentDescription: "User avatar")
    /// ```
    pub fn generate_image(&mut self, props: &HashMap<String, AuraPropValue>) -> GenResult<String> {
        // Add Coil AsyncImage import
        self.add_import("coil.compose.AsyncImage");
        self.add_import("androidx.compose.ui.Modifier");

        // Extract src (required)
        let src = Self::extract_string(props, "src")
            .or_else(|| Self::extract_state_ref(props, "src"))
            .unwrap_or_else(|| "".to_string());

        // Extract contentDescription (optional)
        let content_description = Self::extract_string(props, "contentDescription")
            .unwrap_or_else(|| "Image".to_string());

        // Build AsyncImage call
        let mut parts = Vec::new();

        // Model (src)
        parts.push(format!("model = \"{}\"", src));

        // Content description
        parts.push(format!("contentDescription = \"{}\"", content_description));

        // Modifier placeholder
        parts.push("modifier = Modifier".to_string());

        Ok(format!("AsyncImage(\n        {}\n    )", parts.join(",\n        ")))
    }

    /// Generate Badge component
    ///
    /// # Variants
    /// - Default (with count): Shows a number badge
    /// - `"dot"`: Shows a small dot indicator
    ///
    /// # Props
    /// - `count`: Badge count number (optional, if absent shows dot)
    /// - `variant`: "dot" for small dot indicator
    ///
    /// # Examples
    /// ```auto
    /// Badge (count: 5) {}
    /// Badge (variant: "dot") {}
    /// ```
    ///
    /// # Kotlin Output
    /// ```kotlin
    /// Badge { Text("5") }
    /// Badge {}
    /// ```
    pub fn generate_badge(&mut self, props: &HashMap<String, AuraPropValue>) -> GenResult<String> {
        self.add_import("androidx.compose.material3.Badge");
        self.add_import("androidx.compose.material3.Text");

        // Extract variant
        let variant = Self::extract_string(props, "variant")
            .unwrap_or_default();

        // Extract count (optional)
        let count = Self::extract_int(props, "count");

        match variant.as_str() {
            "dot" => {
                // Dot badge - no content
                Ok("Badge { }".to_string())
            }
            _ => {
                // Default: show count if provided
                if let Some(c) = count {
                    Ok(format!("Badge {{ Text(\"{}\") }}", c))
                } else {
                    Ok("Badge { }".to_string())
                }
            }
        }
    }

    /// Generate Chip component
    ///
    /// # Variants
    /// - `"assist"` (default) → `AssistChip` - clickable action chip
    /// - `"filter"` → `FilterChip` - toggleable filter chip
    /// - `"input"` → `InputChip` - dismissible input chip
    /// - `"suggestion"` → `SuggestionChip` - suggestion chip
    ///
    /// # Props
    /// - `text` / primary prop: Chip label text
    /// - `variant`: Chip type (assist, filter, input, suggestion)
    /// - `selected`: For FilterChip, whether it's selected
    /// - `icon`: Leading icon name
    /// - `onClick`: Click handler reference
    /// - `onDismiss`: For InputChip, dismiss handler
    pub fn generate_chip(&mut self, props: &HashMap<String, AuraPropValue>) -> GenResult<String> {
        // Extract variant (default: "assist")
        let variant = Self::extract_string(props, "variant")
            .unwrap_or_else(|| "assist".to_string());

        // Extract text (primary prop or "text" prop)
        let text = Self::extract_string(props, "text")
            .or_else(|| Self::extract_string(props, "label"))
            .unwrap_or_default();

        // Add imports based on variant
        match variant.as_str() {
            "filter" => {
                self.add_import("androidx.compose.material3.FilterChip");
            }
            "input" => {
                self.add_import("androidx.compose.material3.InputChip");
            }
            "suggestion" => {
                self.add_import("androidx.compose.material3.SuggestionChip");
            }
            _ => {
                // "assist" or any other value defaults to AssistChip
                self.add_import("androidx.compose.material3.AssistChip");
            }
        }
        self.add_import("androidx.compose.material3.Text");
        self.add_import("androidx.compose.ui.Modifier");

        // Build chip based on variant
        match variant.as_str() {
            "filter" => self.generate_filter_chip(props, &text),
            "input" => self.generate_input_chip(props, &text),
            "suggestion" => self.generate_suggestion_chip(props, &text),
            _ => self.generate_assist_chip(props, &text),
        }
    }

    /// Generate AssistChip
    fn generate_assist_chip(&mut self, props: &HashMap<String, AuraPropValue>, text: &str) -> GenResult<String> {
        let mut parts = Vec::new();

        // onClick
        parts.push("onClick = {}".to_string());

        // Label
        parts.push(format!("label = {{ Text(\"{}\") }}", text));

        // Leading icon
        if let Some(icon) = Self::extract_string(props, "icon") {
            self.add_import("androidx.compose.material.icons.Icons");
            self.add_import(&format!("androidx.compose.material.icons.filled.{}", Self::capitalize_icon(&icon)));
            self.add_import("androidx.compose.material3.Icon");
            parts.push(format!(
                "leadingIcon = {{ Icon(Icons.Default.{}, contentDescription = null) }}",
                Self::capitalize_icon(&icon)
            ));
        }

        // Enabled
        if Self::extract_bool(props, "disabled") {
            parts.push("enabled = false".to_string());
        }

        Ok(format!("AssistChip(\n        {}\n    )", parts.join(",\n        ")))
    }

    /// Generate FilterChip
    fn generate_filter_chip(&mut self, props: &HashMap<String, AuraPropValue>, text: &str) -> GenResult<String> {
        let mut parts = Vec::new();

        // Selected state
        let selected = Self::extract_bool(props, "selected");
        parts.push(format!("selected = {}", selected));

        // onClick
        parts.push("onClick = {}".to_string());

        // Label
        parts.push(format!("label = {{ Text(\"{}\") }}", text));

        // Leading icon (optional)
        if let Some(icon) = Self::extract_string(props, "icon") {
            self.add_import("androidx.compose.material.icons.Icons");
            self.add_import(&format!("androidx.compose.material.icons.filled.{}", Self::capitalize_icon(&icon)));
            self.add_import("androidx.compose.material3.Icon");
            parts.push(format!(
                "leadingIcon = if (selected) {{ {{ Icon(Icons.Default.{}, contentDescription = null) }} }} else null",
                Self::capitalize_icon(&icon)
            ));
        }

        // Enabled
        if Self::extract_bool(props, "disabled") {
            parts.push("enabled = false".to_string());
        }

        Ok(format!("FilterChip(\n        {}\n    )", parts.join(",\n        ")))
    }

    /// Generate InputChip
    fn generate_input_chip(&mut self, props: &HashMap<String, AuraPropValue>, text: &str) -> GenResult<String> {
        let mut parts = Vec::new();

        // Selected state
        let selected = Self::extract_bool(props, "selected");
        parts.push(format!("selected = {}", selected));

        // onClick
        parts.push("onClick = {}".to_string());

        // Label
        parts.push(format!("label = {{ Text(\"{}\") }}", text));

        // onDismiss (for dismissible chips)
        if Self::extract_string(props, "onDismiss").is_some() {
            parts.push("onDismissRequest = {}".to_string());
        }

        // Leading icon (optional)
        if let Some(icon) = Self::extract_string(props, "icon") {
            self.add_import("androidx.compose.material.icons.Icons");
            self.add_import(&format!("androidx.compose.material.icons.filled.{}", Self::capitalize_icon(&icon)));
            self.add_import("androidx.compose.material3.Icon");
            parts.push(format!(
                "avatar = {{ Icon(Icons.Default.{}, contentDescription = null) }}",
                Self::capitalize_icon(&icon)
            ));
        }

        // Enabled
        if Self::extract_bool(props, "disabled") {
            parts.push("enabled = false".to_string());
        }

        Ok(format!("InputChip(\n        {}\n    )", parts.join(",\n        ")))
    }

    /// Generate SuggestionChip
    fn generate_suggestion_chip(&mut self, props: &HashMap<String, AuraPropValue>, text: &str) -> GenResult<String> {
        let mut parts = Vec::new();

        // onClick
        parts.push("onClick = {}".to_string());

        // Label
        parts.push(format!("label = {{ Text(\"{}\") }}", text));

        // Icon (optional)
        if let Some(icon) = Self::extract_string(props, "icon") {
            self.add_import("androidx.compose.material.icons.Icons");
            self.add_import(&format!("androidx.compose.material.icons.filled.{}", Self::capitalize_icon(&icon)));
            self.add_import("androidx.compose.material3.Icon");
            parts.push(format!(
                "icon = {{ Icon(Icons.Default.{}, contentDescription = null) }}",
                Self::capitalize_icon(&icon)
            ));
        }

        // Enabled
        if Self::extract_bool(props, "disabled") {
            parts.push("enabled = false".to_string());
        }

        Ok(format!("SuggestionChip(\n        {}\n    )", parts.join(",\n        ")))
    }

    /// Capitalize icon name (e.g., "add" -> "Add")
    fn capitalize_icon(icon: &str) -> String {
        let mut chars = icon.chars();
        match chars.next() {
            Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
            None => icon.to_string(),
        }
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

    // =========================================================================
    // Edge Case Tests (Task 6)
    // =========================================================================

    #[test]
    fn test_input_without_value() {
        // Input without value binding should still generate
        let mut gen = FormGenerator::new();
        let props = HashMap::new();

        let result = gen.generate_input(&props);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("OutlinedTextField"));
        assert!(code.contains("value = \"\""));
    }

    #[test]
    fn test_checkbox_disabled() {
        let mut gen = FormGenerator::new();
        let mut props = HashMap::new();

        props.insert("checked".to_string(), AuraPropValue::Expr(AuraExpr::StateRef("done".to_string())));
        props.insert("disabled".to_string(), AuraPropValue::Expr(AuraExpr::Bool(true)));

        let result = gen.generate_checkbox(&props);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("enabled = false"));
    }

    #[test]
    fn test_switch_disabled() {
        let mut gen = FormGenerator::new();
        let mut props = HashMap::new();

        props.insert("checked".to_string(), AuraPropValue::Expr(AuraExpr::StateRef("enabled".to_string())));
        props.insert("disabled".to_string(), AuraPropValue::Expr(AuraExpr::Bool(true)));

        let result = gen.generate_switch(&props);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("enabled = false"));
    }

    #[test]
    fn test_slider_default_range() {
        // Slider without min/max should use 0..100
        let mut gen = FormGenerator::new();
        let mut props = HashMap::new();

        props.insert("value".to_string(), AuraPropValue::Expr(AuraExpr::StateRef("value".to_string())));

        let result = gen.generate_slider(&props);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("valueRange = 0f..100f"));
    }

    #[test]
    fn test_import_collection() {
        let mut gen = FormGenerator::new();
        let mut props = HashMap::new();

        props.insert("value".to_string(), AuraPropValue::Expr(AuraExpr::StateRef("text".to_string())));
        props.insert("type".to_string(), AuraPropValue::Expr(AuraExpr::Literal("password".to_string())));

        let _ = gen.generate_input(&props);

        let imports = gen.get_imports();
        assert!(imports.iter().any(|i| i.contains("PasswordVisualTransformation")));
    }

    #[test]
    fn test_textarea_without_value() {
        // Textarea without value should still generate
        let mut gen = FormGenerator::new();
        let props = HashMap::new();

        let result = gen.generate_textarea(&props);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("OutlinedTextField"));
        assert!(code.contains("value = \"\""));
    }

    #[test]
    fn test_checkbox_without_state_ref() {
        // Checkbox without state ref should use default
        let mut gen = FormGenerator::new();
        let props = HashMap::new();

        let result = gen.generate_checkbox(&props);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("checked = checked"));
    }

    #[test]
    fn test_switch_without_state_ref() {
        // Switch without state ref should use default
        let mut gen = FormGenerator::new();
        let props = HashMap::new();

        let result = gen.generate_switch(&props);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("checked = checked"));
    }

    #[test]
    fn test_slider_without_state_ref() {
        // Slider without state ref should use default
        let mut gen = FormGenerator::new();
        let props = HashMap::new();

        let result = gen.generate_slider(&props);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("value = value"));
    }

    #[test]
    fn test_input_email_type() {
        let mut gen = FormGenerator::new();
        let mut props = HashMap::new();

        props.insert("value".to_string(), AuraPropValue::Expr(AuraExpr::StateRef("email".to_string())));
        props.insert("type".to_string(), AuraPropValue::Expr(AuraExpr::Literal("email".to_string())));

        let result = gen.generate_input(&props);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("KeyboardType.Email"));

        // Verify import is collected
        let imports = gen.get_imports();
        assert!(imports.iter().any(|i| i.contains("KeyboardOptions")));
    }

    #[test]
    fn test_input_number_type() {
        let mut gen = FormGenerator::new();
        let mut props = HashMap::new();

        props.insert("value".to_string(), AuraPropValue::Expr(AuraExpr::StateRef("age".to_string())));
        props.insert("type".to_string(), AuraPropValue::Expr(AuraExpr::Literal("number".to_string())));

        let result = gen.generate_input(&props);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("KeyboardType.Number"));

        // Verify import is collected
        let imports = gen.get_imports();
        assert!(imports.iter().any(|i| i.contains("KeyboardType")));
    }

    #[test]
    fn test_textarea_disabled() {
        let mut gen = FormGenerator::new();
        let mut props = HashMap::new();

        props.insert("value".to_string(), AuraPropValue::Expr(AuraExpr::StateRef("content".to_string())));
        props.insert("disabled".to_string(), AuraPropValue::Expr(AuraExpr::Bool(true)));

        let result = gen.generate_textarea(&props);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("enabled = false"));
    }

    #[test]
    fn test_import_deduplication() {
        let mut gen = FormGenerator::new();
        let mut props = HashMap::new();

        props.insert("value".to_string(), AuraPropValue::Expr(AuraExpr::StateRef("text".to_string())));

        // Generate multiple inputs
        let _ = gen.generate_input(&props);
        let _ = gen.generate_input(&props);

        // Verify no duplicate imports
        let imports = gen.get_imports();
        let outlined_count = imports.iter().filter(|i| i.contains("OutlinedTextField")).count();
        assert_eq!(outlined_count, 1);
    }

    #[test]
    fn test_clear_imports() {
        let mut gen = FormGenerator::new();
        let mut props = HashMap::new();

        props.insert("value".to_string(), AuraPropValue::Expr(AuraExpr::StateRef("text".to_string())));

        let _ = gen.generate_input(&props);
        assert!(!gen.get_imports().is_empty());

        gen.clear_imports();
        assert!(gen.get_imports().is_empty());
    }

    // ========================================
    // Chip Tests
    // ========================================

    #[test]
    fn test_chip_assist() {
        let mut gen = FormGenerator::new();
        let mut props = HashMap::new();

        props.insert("variant".to_string(), AuraPropValue::Expr(AuraExpr::Literal("assist".to_string())));
        props.insert("text".to_string(), AuraPropValue::Expr(AuraExpr::Literal("Add Item".to_string())));

        let result = gen.generate_chip(&props);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("AssistChip"));
        assert!(code.contains("label = { Text(\"Add Item\") }"));
        assert!(code.contains("onClick = {}"));
    }

    #[test]
    fn test_chip_assist_with_icon() {
        let mut gen = FormGenerator::new();
        let mut props = HashMap::new();

        props.insert("variant".to_string(), AuraPropValue::Expr(AuraExpr::Literal("assist".to_string())));
        props.insert("text".to_string(), AuraPropValue::Expr(AuraExpr::Literal("Add".to_string())));
        props.insert("icon".to_string(), AuraPropValue::Expr(AuraExpr::Literal("add".to_string())));

        let result = gen.generate_chip(&props);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("AssistChip"));
        assert!(code.contains("leadingIcon = { Icon(Icons.Default.Add, contentDescription = null) }"));

        // Verify icon imports
        let imports = gen.get_imports();
        assert!(imports.iter().any(|i| i.contains("androidx.compose.material.icons.filled.Add")));
    }

    #[test]
    fn test_chip_filter() {
        let mut gen = FormGenerator::new();
        let mut props = HashMap::new();

        props.insert("variant".to_string(), AuraPropValue::Expr(AuraExpr::Literal("filter".to_string())));
        props.insert("text".to_string(), AuraPropValue::Expr(AuraExpr::Literal("Filter".to_string())));
        props.insert("selected".to_string(), AuraPropValue::Expr(AuraExpr::Bool(true)));

        let result = gen.generate_chip(&props);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("FilterChip"));
        assert!(code.contains("selected = true"));
        assert!(code.contains("label = { Text(\"Filter\") }"));
    }

    #[test]
    fn test_chip_filter_not_selected() {
        let mut gen = FormGenerator::new();
        let mut props = HashMap::new();

        props.insert("variant".to_string(), AuraPropValue::Expr(AuraExpr::Literal("filter".to_string())));
        props.insert("text".to_string(), AuraPropValue::Expr(AuraExpr::Literal("Option".to_string())));
        // No selected prop - defaults to false

        let result = gen.generate_chip(&props);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("selected = false"));
    }

    #[test]
    fn test_chip_input() {
        let mut gen = FormGenerator::new();
        let mut props = HashMap::new();

        props.insert("variant".to_string(), AuraPropValue::Expr(AuraExpr::Literal("input".to_string())));
        props.insert("text".to_string(), AuraPropValue::Expr(AuraExpr::Literal("Tag".to_string())));
        props.insert("onDismiss".to_string(), AuraPropValue::Expr(AuraExpr::Literal("RemoveChip".to_string())));

        let result = gen.generate_chip(&props);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("InputChip"));
        assert!(code.contains("label = { Text(\"Tag\") }"));
        assert!(code.contains("onDismissRequest = {}"));
    }

    #[test]
    fn test_chip_suggestion() {
        let mut gen = FormGenerator::new();
        let mut props = HashMap::new();

        props.insert("variant".to_string(), AuraPropValue::Expr(AuraExpr::Literal("suggestion".to_string())));
        props.insert("text".to_string(), AuraPropValue::Expr(AuraExpr::Literal("Suggestion".to_string())));

        let result = gen.generate_chip(&props);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("SuggestionChip"));
        assert!(code.contains("label = { Text(\"Suggestion\") }"));
        assert!(code.contains("onClick = {}"));
    }

    #[test]
    fn test_chip_default_is_assist() {
        // Without variant prop, should default to AssistChip
        let mut gen = FormGenerator::new();
        let mut props = HashMap::new();

        props.insert("text".to_string(), AuraPropValue::Expr(AuraExpr::Literal("Default".to_string())));

        let result = gen.generate_chip(&props);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("AssistChip"));
    }

    #[test]
    fn test_chip_disabled() {
        let mut gen = FormGenerator::new();
        let mut props = HashMap::new();

        props.insert("variant".to_string(), AuraPropValue::Expr(AuraExpr::Literal("assist".to_string())));
        props.insert("text".to_string(), AuraPropValue::Expr(AuraExpr::Literal("Disabled".to_string())));
        props.insert("disabled".to_string(), AuraPropValue::Expr(AuraExpr::Bool(true)));

        let result = gen.generate_chip(&props);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("enabled = false"));
    }

    #[test]
    fn test_chip_icon_capitalization() {
        // Test that icon names are properly capitalized
        let mut gen = FormGenerator::new();
        let mut props = HashMap::new();

        props.insert("variant".to_string(), AuraPropValue::Expr(AuraExpr::Literal("assist".to_string())));
        props.insert("text".to_string(), AuraPropValue::Expr(AuraExpr::Literal("Settings".to_string())));
        props.insert("icon".to_string(), AuraPropValue::Expr(AuraExpr::Literal("settings".to_string())));

        let result = gen.generate_chip(&props);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("Icons.Default.Settings"));

        let imports = gen.get_imports();
        assert!(imports.iter().any(|i| i.contains("filled.Settings")));
    }
}
