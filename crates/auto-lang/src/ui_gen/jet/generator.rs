//! Jetpack Compose Generator
//!
//! Main generator that produces Kotlin/Compose code from AURA widgets.
//!
//! ## Output Format
//!
//! ```kotlin
//! package com.example.widgets
//!
//! import androidx.compose.foundation.layout.*
//! import androidx.compose.material3.*
//! import androidx.compose.runtime.*
//! import androidx.compose.ui.Modifier
//! import androidx.compose.ui.unit.dp
//!
//! @Composable
//! fun MyWidget(modifier: Modifier = Modifier) {
//!     var count by remember { mutableStateOf(0) }
//!
//!     Column(modifier = modifier) {
//!         Button(onClick = { count++ }) {
//!             Text("Click: $count")
//!         }
//!     }
//! }
//!
//! @Preview(showBackground = true)
//! @Composable
//! fun MyWidgetPreview() {
//!     MyWidget()
//! }
//! ```

use super::components::Material3Registry;
use super::modifier::ModifierDsl;
use super::state::StateConverter;
use crate::aura::AuraWidget;
use crate::ui_gen::{BackendGenerator, GenError, GenResult};
use std::collections::HashSet;

/// Jetpack Compose code generator
pub struct JetGenerator {
    /// Current widget name
    current_widget: Option<String>,

    /// Package name for generated code
    package: String,

    /// Collected imports
    imports: HashSet<String>,

    /// Material3 component registry
    #[allow(dead_code)]
    registry: Material3Registry,

    /// Modifier DSL converter
    #[allow(dead_code)]
    modifier_dsl: ModifierDsl,

    /// State converter
    state_converter: StateConverter,

    /// Components used in current widget
    #[allow(dead_code)]
    components_used: HashSet<String>,
}

impl JetGenerator {
    /// Create a new JetGenerator with default package
    pub fn new() -> Self {
        Self {
            current_widget: None,
            package: "com.example.widgets".to_string(),
            imports: HashSet::new(),
            registry: Material3Registry::new(),
            modifier_dsl: ModifierDsl::new(),
            state_converter: StateConverter::new(),
            components_used: HashSet::new(),
        }
    }

    /// Set package name (builder pattern)
    pub fn with_package(mut self, package: &str) -> Self {
        self.package = package.to_string();
        self
    }

    /// Get current package name
    pub fn package_name(&self) -> &str {
        &self.package
    }

    /// Generate @Composable function signature and body
    pub fn generate_composable(&mut self, name: &str, body: &str) -> String {
        format!(
            r#"@Composable
fun {}(
    modifier: Modifier = Modifier
) {{
    {}
}}"#,
            name, body
        )
    }

    /// Generate @Preview function
    pub fn generate_preview(&self, name: &str) -> String {
        format!(
            r#"@Preview(showBackground = true)
@Composable
fun {}Preview() {{
    {}()
}}"#,
            name, name
        )
    }

    /// Add import to collection
    pub fn add_import(&mut self, import: &str) {
        self.imports.insert(import.to_string());
    }

    /// Generate import statements from collected imports
    pub fn generate_imports(&self) -> String {
        let mut imports: Vec<&str> = self.imports.iter().map(|s| s.as_str()).collect();
        imports.sort();
        imports.dedup();

        imports
            .iter()
            .map(|i| format!("import {}", i))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Generate package declaration
    pub fn generate_package(&self) -> String {
        format!("package {}\n", self.package)
    }

    /// Convert Type to string representation
    fn type_to_string(ty: &crate::ast::Type) -> String {
        use crate::ast::Type;
        match ty {
            Type::Int => "int".to_string(),
            Type::Uint => "uint".to_string(),
            Type::Float => "float".to_string(),
            Type::Double => "double".to_string(),
            Type::Str(_) => "str".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Char => "char".to_string(),
            Type::Byte => "byte".to_string(),
            Type::I64 => "i64".to_string(),
            Type::U64 => "u64".to_string(),
            _ => "unknown".to_string(),
        }
    }

    /// Convert AuraExpr to default value string
    fn expr_to_default(expr: &crate::aura::AuraExpr) -> String {
        use crate::aura::AuraExpr;
        match expr {
            AuraExpr::Int(n) => n.to_string(),
            AuraExpr::Float(n) => {
                let s = n.to_string();
                if s.contains('.') {
                    s
                } else {
                    format!("{}.0", s)
                }
            }
            AuraExpr::Literal(s) => format!("\"{}\"", s),
            AuraExpr::Bool(b) => b.to_string(),
            _ => "null".to_string(),
        }
    }

    /// Generate state declarations for a widget
    fn generate_state_declarations(&self, widget: &AuraWidget) -> String {
        widget
            .state_vars
            .iter()
            .map(|state_def| {
                let name = &state_def.name;
                let type_str = Self::type_to_string(&state_def.type_info);
                let default = Self::expr_to_default(&state_def.initial);
                self.state_converter.convert_model(name, &type_str, &default)
            })
            .collect::<Vec<_>>()
            .join("\n    ")
    }

    /// Generate view body (placeholder for now)
    fn generate_view_body(&mut self, _widget: &AuraWidget) -> GenResult<String> {
        // TODO: Implement full view body generation from widget.view_tree
        // For now, return a placeholder
        Ok("Column(modifier = modifier) {\n        // TODO: Generate view from AURA\n    }".to_string())
    }

    /// Generate event handlers for a widget (placeholder)
    fn generate_handlers(&self, _widget: &AuraWidget) -> String {
        // TODO: Implement handler generation from widget.handlers
        String::new()
    }
}

impl Default for JetGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl BackendGenerator for JetGenerator {
    fn generate(&mut self, widget: &AuraWidget) -> GenResult<String> {
        self.current_widget = Some(widget.name.clone());

        // Reset state for new widget
        self.imports.clear();
        self.components_used.clear();

        // Add standard Compose imports
        self.add_import("androidx.compose.foundation.layout.*");
        self.add_import("androidx.compose.material3.*");
        self.add_import("androidx.compose.runtime.*");
        self.add_import("androidx.compose.ui.Modifier");
        self.add_import("androidx.compose.ui.unit.dp");
        self.add_import("androidx.compose.ui.tooling.preview.Preview");

        // Generate components
        let state_decls = self.generate_state_declarations(widget);
        let view_body = self.generate_view_body(widget)?;
        let _handlers = self.generate_handlers(widget);

        // Assemble final code
        let package_decl = self.generate_package();
        let imports = self.generate_imports();
        let composable_name = &widget.name;
        let preview = self.generate_preview(composable_name);

        let code = if state_decls.is_empty() {
            format!(
                r#"{}// Auto-generated by a2jet

{}

@Composable
fun {}(
    modifier: Modifier = Modifier
) {{
    {}
}}

{}
"#,
                package_decl, imports, composable_name, view_body, preview
            )
        } else {
            format!(
                r#"{}// Auto-generated by a2jet

{}

@Composable
fun {}(
    modifier: Modifier = Modifier
) {{
    {}

    {}
}}

{}
"#,
                package_decl, imports, composable_name, state_decls, view_body, preview
            )
        };

        Ok(code)
    }

    fn extension(&self) -> &'static str {
        "kt"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_simple_composable() {
        let mut gen = JetGenerator::new();
        let result = gen.generate_composable("MyWidget", "Column { Text(\"Hello\") }");
        assert!(result.contains("@Composable"));
        assert!(result.contains("fun MyWidget"));
        assert!(result.contains("Column"));
    }

    #[test]
    fn test_generate_preview() {
        let gen = JetGenerator::new();
        let result = gen.generate_preview("MyWidget");
        assert!(result.contains("@Preview"));
        assert!(result.contains("fun MyWidgetPreview"));
    }

    #[test]
    fn test_extension() {
        let gen = JetGenerator::new();
        assert_eq!(gen.extension(), "kt");
    }

    #[test]
    fn test_package_declaration() {
        let gen = JetGenerator::new();
        let result = gen.generate_package();
        assert!(result.contains("package"));
        assert!(result.contains("com.example.widgets"));
    }

    #[test]
    fn test_with_package() {
        let gen = JetGenerator::new().with_package("com.myapp.ui");
        assert_eq!(gen.package_name(), "com.myapp.ui");
    }

    #[test]
    fn test_import_collection() {
        let mut gen = JetGenerator::new();
        gen.add_import("androidx.compose.material3.*");
        gen.add_import("androidx.compose.runtime.*");
        let imports = gen.generate_imports();
        assert!(imports.contains("androidx.compose.material3.*"));
        assert!(imports.contains("androidx.compose.runtime.*"));
    }

    #[test]
    fn test_import_deduplication() {
        let mut gen = JetGenerator::new();
        gen.add_import("androidx.compose.material3.*");
        gen.add_import("androidx.compose.material3.*");
        gen.add_import("androidx.compose.runtime.*");
        let imports = gen.generate_imports();
        // Count occurrences of the import
        let count = imports.matches("androidx.compose.material3.*").count();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_import_sorting() {
        let mut gen = JetGenerator::new();
        gen.add_import("androidx.compose.ui.Modifier");
        gen.add_import("androidx.compose.foundation.layout.*");
        gen.add_import("androidx.compose.material3.*");
        let imports = gen.generate_imports();
        // Check that imports are sorted alphabetically
        let lines: Vec<&str> = imports.lines().collect();
        assert!(lines.len() >= 3);
        // androidx.compose.foundation should come before androidx.compose.material
        assert!(lines[0].contains("foundation"));
    }
}
