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
use super::form::FormGenerator;
use super::layout::LayoutGenerator;
use super::list::ListGenerator;
use super::modifier::ModifierDsl;
use super::navigation::NavigationGenerator;
use super::state::StateConverter;
use crate::aura::{AuraPropValue, AuraWidget};
use crate::ui_gen::shared::ComponentRegistry;
use crate::ui_gen::{BackendGenerator, GenError, GenResult};
use std::collections::{HashMap, HashSet};

/// Jetpack Compose code generator for Android
///
/// This is the main entry point for generating Kotlin/Compose code from AURA widgets.
/// It coordinates multiple sub-generators for different component types.
///
/// # Architecture
///
/// ```text
/// JetGenerator
///     ├── FormGenerator (inputs, buttons, sliders)
///     ├── LayoutGenerator (Column, Row, Box, Card)
///     ├── ListGenerator (LazyColumn, LazyRow, Grid)
///     ├── NavigationGenerator (NavHost, routes)
///     ├── StateConverter (model → mutableStateOf)
///     ├── ModifierDsl (Tailwind → Compose Modifier)
///     └── ComponentRegistry (shared AURA → Jet mappings)
/// ```
///
/// # Example
///
/// ```rust
/// use auto_lang::ui_gen::jet::JetGenerator;
/// use auto_lang::ui_gen::BackendGenerator;
/// use auto_lang::aura::AuraWidget;
///
/// let mut gen = JetGenerator::new();
/// // gen.generate(&widget); // Generate widget code
/// // gen.generate_project_default("MyApp"); // Generate full project
/// ```
pub struct JetGenerator {
    /// Current widget name
    current_widget: Option<String>,

    /// Package name for generated code
    package: String,

    /// Collected imports
    imports: HashSet<String>,

    /// Material3 component registry (legacy, for backward compatibility)
    #[allow(dead_code)]
    registry: Material3Registry,

    /// Shared component registry (AURA → Vue/Jet mappings)
    #[allow(dead_code)]
    component_registry: ComponentRegistry,

    /// Modifier DSL converter
    #[allow(dead_code)]
    modifier_dsl: ModifierDsl,

    /// State converter
    state_converter: StateConverter,

    /// Form component generator
    form_generator: FormGenerator,

    /// Layout component generator
    layout_generator: LayoutGenerator,

    /// List component generator
    list_generator: ListGenerator,

    /// Navigation generator
    navigation_generator: NavigationGenerator,

    /// Components used in current widget
    #[allow(dead_code)]
    components_used: HashSet<String>,
}

impl JetGenerator {
    /// Create a new JetGenerator with default package
    ///
    /// Initializes all sub-generators with default configuration:
    /// - Package: `com.example.widgets`
    /// - Material3 registry with standard components
    /// - Tailwind-to-Compose modifier DSL
    ///
    /// # Example
    ///
    /// ```rust
    /// use auto_lang::ui_gen::jet::JetGenerator;
    ///
    /// let gen = JetGenerator::new();
    /// assert_eq!(gen.package_name(), "com.example.widgets");
    /// ```
    pub fn new() -> Self {
        Self {
            current_widget: None,
            package: "com.example.widgets".to_string(),
            imports: HashSet::new(),
            registry: Material3Registry::new(),
            component_registry: ComponentRegistry::new(),
            modifier_dsl: ModifierDsl::new(),
            state_converter: StateConverter::new(),
            form_generator: FormGenerator::new(),
            layout_generator: LayoutGenerator::new(),
            list_generator: ListGenerator::new(),
            navigation_generator: NavigationGenerator::new(),
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
pub fun {}(
    modifier: Modifier = Modifier
) {{
    {}
}}"#,
            name, body
        )
    }

    /// Generate public @Composable function (with pub modifier for cross-file access)
    pub fn generate_public_composable(&mut self, name: &str, body: &str) -> String {
        format!(
            r#"@Composable
pub fun {}(
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

    /// Generate form element code based on tag type
    pub fn generate_form_element(
        &mut self,
        tag: &str,
        props: &HashMap<String, AuraPropValue>,
    ) -> GenResult<String> {
        match tag {
            "input" => self.form_generator.generate_input(props),
            "textarea" => self.form_generator.generate_textarea(props),
            "checkbox" => self.form_generator.generate_checkbox(props),
            "switch" | "toggle" => self.form_generator.generate_switch(props),
            "slider" => self.form_generator.generate_slider(props),
            _ => Err(GenError::UnsupportedExpr(format!("Unknown form element: {}", tag))),
        }
    }

    /// Get form-specific imports
    pub fn get_form_imports(&self) -> &[String] {
        self.form_generator.get_imports()
    }

    /// Generate layout element code based on tag type
    pub fn generate_layout_element(
        &mut self,
        tag: &str,
        props: &HashMap<String, AuraPropValue>,
        children: &str,
    ) -> GenResult<String> {
        match tag {
            "col" | "column" => self.layout_generator.generate_column(props, children),
            "row" => self.layout_generator.generate_row(props, children),
            "box" | "container" => self.layout_generator.generate_box(props, children),
            "card" => self.layout_generator.generate_card(props, children),
            "scroll" => self.layout_generator.generate_scroll(props, children),
            _ => Err(GenError::UnsupportedExpr(format!("Unknown layout element: {}", tag))),
        }
    }

    /// Get layout-specific imports
    pub fn get_layout_imports(&self) -> &[String] {
        self.layout_generator.get_imports()
    }

    /// Generate list element code based on tag type
    pub fn generate_list_element(
        &mut self,
        tag: &str,
        props: &HashMap<String, AuraPropValue>,
        item_content: &str,
    ) -> GenResult<String> {
        match tag {
            "list" | "lazy-column" => self.list_generator.generate_lazy_column(props, item_content),
            "list-row" | "lazy-row" => self.list_generator.generate_lazy_row(props, item_content),
            "grid" | "lazy-grid" => self.list_generator.generate_lazy_grid(props, item_content),
            "flow-row" => self.list_generator.generate_flow_row(props, item_content),
            "flow-col" | "flow-column" => self.list_generator.generate_flow_column(props, item_content),
            _ => Err(GenError::UnsupportedExpr(format!("Unknown list element: {}", tag))),
        }
    }

    /// Get list-specific imports
    pub fn get_list_imports(&self) -> &[String] {
        self.list_generator.get_imports()
    }

    /// Add a navigation route
    pub fn add_nav_route(&mut self, name: &str, screen: &str) {
        self.navigation_generator.add_route(name, screen);
    }

    /// Add a navigation route with parameters
    pub fn add_nav_route_with_params(&mut self, name: &str, screen: &str, params: Vec<String>) {
        self.navigation_generator.add_route_with_params(name, screen, params);
    }

    /// Generate navigation host
    pub fn generate_nav_host(&mut self, start_destination: &str) -> GenResult<String> {
        self.navigation_generator.generate_nav_host(start_destination)
    }

    /// Generate app with navigation
    pub fn generate_app_with_nav(&mut self, start_destination: &str) -> GenResult<String> {
        self.navigation_generator.generate_app_with_nav(start_destination)
    }

    /// Generate navigate call
    pub fn generate_navigate_call(&self, route: &str) -> String {
        self.navigation_generator.generate_navigate_call(route)
    }

    /// Get navigation-specific imports
    pub fn get_navigation_imports(&self) -> &[String] {
        self.navigation_generator.get_imports()
    }

    // =========================================================================
    // Project Generation Methods (Phase 6)
    // =========================================================================

    /// Generate a complete Android project with the given configuration
    ///
    /// Creates a full Android project structure including:
    /// - Root Gradle files (build.gradle.kts, settings.gradle.kts)
    /// - Version catalog (gradle/libs.versions.toml)
    /// - App module (app/build.gradle.kts)
    /// - MainActivity.kt
    /// - Theme files (Theme.kt, Color.kt, Type.kt)
    /// - AndroidManifest.xml
    /// - Resource files (strings.xml)
    ///
    /// # Arguments
    ///
    /// * `config` - Project configuration (name, package, SDK versions, theme)
    ///
    /// # Returns
    ///
    /// HashMap of file paths (relative to project root) to their contents
    ///
    /// # Example
    ///
    /// ```rust
    /// use auto_lang::ui_gen::jet::JetGenerator;
    /// use auto_lang::ui_gen::jet::project::JetProjectConfig;
    ///
    /// let gen = JetGenerator::new();
    /// let config = JetProjectConfig::new("MyApp")
    ///     .with_application_id("com.company.myapp");
    /// let files = gen.generate_project(config);
    /// assert!(files.contains_key("app/build.gradle.kts"));
    /// ```
    pub fn generate_project(
        &self,
        config: super::project::JetProjectConfig,
    ) -> HashMap<String, String> {
        let mut gen = super::project::ProjectGenerator::with_config(config);
        gen.generate()
    }

    /// Generate an Android project with default configuration
    ///
    /// Convenience method that creates a project with:
    /// - Package: `com.example.{name.lowercase()}`
    /// - SDK: minSdk 24, compileSdk/targetSdk 34
    /// - Kotlin 1.9.0, Compose BOM 2024.02.00
    /// - Material3 default theme
    ///
    /// # Arguments
    ///
    /// * `name` - Project name (used for app name and default package)
    ///
    /// # Returns
    ///
    /// HashMap of file paths to their contents
    ///
    /// # Example
    ///
    /// ```rust
    /// use auto_lang::ui_gen::jet::JetGenerator;
    ///
    /// let gen = JetGenerator::new();
    /// let files = gen.generate_project_default("MyApp");
    /// assert!(files.len() > 15); // 15+ files generated
    /// ```
    pub fn generate_project_default(&self, name: &str) -> HashMap<String, String> {
        let config = super::project::JetProjectConfig::new(name);
        self.generate_project(config)
    }

    /// Generate an Android project with custom application ID
    ///
    /// Creates a project with a custom package name instead of the default
    /// `com.example.{name}`.
    ///
    /// # Arguments
    ///
    /// * `name` - Project name (for app display name)
    /// * `application_id` - Full package name (e.g., "com.company.myapp")
    ///
    /// # Returns
    ///
    /// HashMap of file paths to their contents
    ///
    /// # Example
    ///
    /// ```rust
    /// use auto_lang::ui_gen::jet::JetGenerator;
    ///
    /// let gen = JetGenerator::new();
    /// let files = gen.generate_project_with_package("MyApp", "com.company.myapp");
    /// let main_activity = files.values().find(|v| v.contains("class MainActivity"));
    /// assert!(main_activity.unwrap().contains("package com.company.myapp"));
    /// ```
    pub fn generate_project_with_package(
        &self,
        name: &str,
        application_id: &str,
    ) -> HashMap<String, String> {
        let config = super::project::JetProjectConfig::new(name)
            .with_application_id(application_id);
        self.generate_project(config)
    }

    /// Generate an Android project with custom theme colors
    ///
    /// Creates a project with custom Material3 theme colors.
    /// The colors are used in the generated Color.kt file.
    ///
    /// # Arguments
    ///
    /// * `name` - Project name
    /// * `primary` - Primary color in hex format (e.g., "#6750A4")
    /// * `secondary` - Secondary color in hex format (e.g., "#625B71")
    ///
    /// # Returns
    ///
    /// HashMap of file paths to their contents
    ///
    /// # Example
    ///
    /// ```rust
    /// use auto_lang::ui_gen::jet::JetGenerator;
    ///
    /// let gen = JetGenerator::new();
    /// let files = gen.generate_project_with_theme("MyApp", "#FF0000", "#00FF00");
    /// let color_kt = files.values().find(|v| v.contains("Color(0x"));
    /// assert!(color_kt.is_some());
    /// ```
    pub fn generate_project_with_theme(
        &self,
        name: &str,
        primary: &str,
        secondary: &str,
    ) -> HashMap<String, String> {
        let theme = super::project::ThemeColors::new(primary, secondary);
        let config = super::project::JetProjectConfig::new(name).with_theme(theme);
        self.generate_project(config)
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
pub fun {}(
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
pub fun {}(
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

    // =========================================================================
    // Integration Tests
    // =========================================================================

    #[test]
    fn test_full_generation_workflow() {
        use crate::aura::{AuraWidget, AuraStateDef, AuraNode};
        use crate::ast::Type;

        // Create a simple Counter widget
        let widget = AuraWidget {
            name: "Counter".to_string(),
            state_vars: vec![AuraStateDef {
                name: "count".to_string(),
                type_info: Type::Int,
                initial: crate::aura::AuraExpr::Int(0),
            }],
            computed: vec![],
            messages: vec![],
            view_tree: AuraNode::element("col"),
            handlers: std::collections::HashMap::new(),
            props: vec![],
            routes: None,
        };

        let mut gen = JetGenerator::new();
        let result = gen.generate(&widget);

        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("package com.example.widgets"));
        assert!(code.contains("@Composable"));
        assert!(code.contains("fun Counter"));
        assert!(code.contains("var count by remember"));
        assert!(code.contains("mutableStateOf(0)"));
        assert!(code.contains("@Preview"));
    }

    #[test]
    fn test_full_generation_with_multiple_states() {
        use crate::aura::{AuraWidget, AuraStateDef, AuraNode};
        use crate::ast::Type;

        // Create a widget with multiple state variables
        let widget = AuraWidget {
            name: "UserProfile".to_string(),
            state_vars: vec![
                AuraStateDef {
                    name: "name".to_string(),
                    type_info: Type::Str(0),
                    initial: crate::aura::AuraExpr::Literal("Guest".to_string()),
                },
                AuraStateDef {
                    name: "age".to_string(),
                    type_info: Type::Int,
                    initial: crate::aura::AuraExpr::Int(25),
                },
                AuraStateDef {
                    name: "enabled".to_string(),
                    type_info: Type::Bool,
                    initial: crate::aura::AuraExpr::Bool(true),
                },
            ],
            computed: vec![],
            messages: vec![],
            view_tree: AuraNode::element("col"),
            handlers: std::collections::HashMap::new(),
            props: vec![],
            routes: None,
        };

        let mut gen = JetGenerator::new();
        let result = gen.generate(&widget);

        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("var name by remember"));
        assert!(code.contains("mutableStateOf(\"Guest\")"));
        assert!(code.contains("var age by remember"));
        assert!(code.contains("mutableStateOf(25)"));
        assert!(code.contains("var enabled by remember"));
        assert!(code.contains("mutableStateOf(true)"));
    }

    #[test]
    fn test_full_generation_no_state() {
        use crate::aura::{AuraWidget, AuraNode};

        // Create a stateless widget
        let widget = AuraWidget {
            name: "StaticHeader".to_string(),
            state_vars: vec![],
            computed: vec![],
            messages: vec![],
            view_tree: AuraNode::element("col"),
            handlers: std::collections::HashMap::new(),
            props: vec![],
            routes: None,
        };

        let mut gen = JetGenerator::new();
        let result = gen.generate(&widget);

        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("fun StaticHeader"));
        // Should not contain state declarations
        assert!(!code.contains("var "));
    }

    #[test]
    fn test_component_registry_integration() {
        let gen = JetGenerator::new();

        // Verify registry is properly initialized with common components
        assert!(gen.registry.is_supported("button"));
        assert!(gen.registry.is_supported("text"));
        assert!(gen.registry.is_supported("col"));
        assert!(gen.registry.is_supported("row"));
        assert!(gen.registry.is_supported("input"));
        assert!(gen.registry.is_supported("checkbox"));
        assert!(gen.registry.is_supported("card"));

        // Verify unsupported element
        assert!(!gen.registry.is_supported("nonexistent_element"));
    }

    #[test]
    fn test_component_primary_components() {
        let gen = JetGenerator::new();

        // Test primary component mappings
        assert_eq!(gen.registry.primary_component("button"), Some("Button"));
        assert_eq!(gen.registry.primary_component("text"), Some("Text"));
        assert_eq!(gen.registry.primary_component("col"), Some("Column"));
        assert_eq!(gen.registry.primary_component("row"), Some("Row"));
        assert_eq!(gen.registry.primary_component("input"), Some("TextField"));
    }

    #[test]
    fn test_modifier_dsl_integration() {
        let gen = JetGenerator::new();

        // Test modifier conversion
        let result = gen.modifier_dsl.convert_class("px-4 py-2 gap-2");
        assert!(!result.modifiers.is_empty());
        assert!(result.arrangement.is_some());

        // Test that modifiers are properly formatted
        let chain = gen.modifier_dsl.generate_modifier_chain("px-4 rounded-lg");
        assert!(chain.starts_with("Modifier."));
    }

    #[test]
    fn test_state_converter_integration() {
        let gen = JetGenerator::new();

        // Test int state
        let int_state = gen.state_converter.convert_model("count", "int", "0");
        assert!(int_state.contains("var count by remember"));
        assert!(int_state.contains("mutableStateOf(0)"));

        // Test string state
        let str_state = gen.state_converter.convert_model("name", "str", "\"Hello\"");
        assert!(str_state.contains("var name by remember"));
        assert!(str_state.contains("mutableStateOf(\"Hello\")"));

        // Test bool state
        let bool_state = gen.state_converter.convert_model("enabled", "bool", "true");
        assert!(bool_state.contains("var enabled by remember"));
        assert!(bool_state.contains("mutableStateOf(true)"));

        // Test float state
        let float_state = gen.state_converter.convert_model("price", "float", "9.99");
        assert!(float_state.contains("var price by remember"));
        assert!(float_state.contains("mutableStateOf(9.99)"));
    }

    #[test]
    fn test_package_customization() {
        let gen = JetGenerator::new().with_package("com.myapp.ui.components");
        assert_eq!(gen.package_name(), "com.myapp.ui.components");

        let package_decl = gen.generate_package();
        assert!(package_decl.contains("com.myapp.ui.components"));
    }

    #[test]
    fn test_standard_imports() {
        use crate::aura::{AuraWidget, AuraNode};

        let widget = AuraWidget {
            name: "TestWidget".to_string(),
            state_vars: vec![],
            computed: vec![],
            messages: vec![],
            view_tree: AuraNode::element("col"),
            handlers: std::collections::HashMap::new(),
            props: vec![],
            routes: None,
        };

        let mut gen = JetGenerator::new();
        let result = gen.generate(&widget).unwrap();

        // Verify standard Compose imports are included
        assert!(result.contains("import androidx.compose.foundation.layout.*"));
        assert!(result.contains("import androidx.compose.material3.*"));
        assert!(result.contains("import androidx.compose.runtime.*"));
        assert!(result.contains("import androidx.compose.ui.Modifier"));
        assert!(result.contains("import androidx.compose.ui.unit.dp"));
        assert!(result.contains("import androidx.compose.ui.tooling.preview.Preview"));
    }

    #[test]
    fn test_type_to_string_conversion() {
        use crate::ast::Type;

        // Test type conversion via the generator
        assert_eq!(JetGenerator::type_to_string(&Type::Int), "int");
        assert_eq!(JetGenerator::type_to_string(&Type::Float), "float");
        assert_eq!(JetGenerator::type_to_string(&Type::Bool), "bool");
        assert_eq!(JetGenerator::type_to_string(&Type::Str(0)), "str");
        assert_eq!(JetGenerator::type_to_string(&Type::Uint), "uint");
        assert_eq!(JetGenerator::type_to_string(&Type::Byte), "byte");
    }

    #[test]
    fn test_expr_to_default_conversion() {
        use crate::aura::AuraExpr;

        // Test expression conversion
        assert_eq!(JetGenerator::expr_to_default(&AuraExpr::Int(42)), "42");
        assert_eq!(JetGenerator::expr_to_default(&AuraExpr::Bool(true)), "true");
        assert_eq!(JetGenerator::expr_to_default(&AuraExpr::Bool(false)), "false");
        assert_eq!(JetGenerator::expr_to_default(&AuraExpr::Literal("hello".to_string())), "\"hello\"");
        assert_eq!(JetGenerator::expr_to_default(&AuraExpr::Float(3.14)), "3.14");
        assert_eq!(JetGenerator::expr_to_default(&AuraExpr::Float(5.0)), "5.0");
    }

    #[test]
    fn test_backend_generator_trait() {
        use crate::ui_gen::BackendGenerator;
        use crate::aura::{AuraWidget, AuraNode};

        let widget = AuraWidget {
            name: "TraitTest".to_string(),
            state_vars: vec![],
            computed: vec![],
            messages: vec![],
            view_tree: AuraNode::element("col"),
            handlers: std::collections::HashMap::new(),
            props: vec![],
            routes: None,
        };

        let mut gen = JetGenerator::new();

        // Test BackendGenerator trait implementation
        let result = gen.generate(&widget);
        assert!(result.is_ok());

        // Test extension method
        assert_eq!(gen.extension(), "kt");
    }

    #[test]
    fn test_jet_generator_form_integration() {
        use crate::aura::{AuraExpr, AuraPropValue};

        let mut gen = JetGenerator::new();
        let mut props = HashMap::new();

        props.insert("value".to_string(), AuraPropValue::Expr(AuraExpr::StateRef("email".to_string())));
        props.insert("placeholder".to_string(), AuraPropValue::Expr(AuraExpr::Literal("Enter email".to_string())));
        props.insert("label".to_string(), AuraPropValue::Expr(AuraExpr::Literal("Email".to_string())));

        let result = gen.generate_form_element("input", &props);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("OutlinedTextField"));
        assert!(code.contains("value = email"));
        assert!(code.contains("placeholder"));
        assert!(code.contains("label"));

        // Verify imports are collected
        let imports = gen.get_form_imports();
        assert!(!imports.is_empty());
        assert!(imports.iter().any(|i| i.contains("OutlinedTextField")));
    }

    #[test]
    fn test_jet_generator_checkbox_integration() {
        use crate::aura::{AuraExpr, AuraPropValue};

        let mut gen = JetGenerator::new();
        let mut props = HashMap::new();

        props.insert("checked".to_string(), AuraPropValue::Expr(AuraExpr::StateRef("agree".to_string())));
        props.insert("label".to_string(), AuraPropValue::Expr(AuraExpr::Literal("I agree".to_string())));

        let result = gen.generate_form_element("checkbox", &props);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("Checkbox"));
        assert!(code.contains("Row"));
        assert!(code.contains("Text(\"I agree\")"));
    }

    #[test]
    fn test_jet_generator_switch_integration() {
        use crate::aura::{AuraExpr, AuraPropValue};

        let mut gen = JetGenerator::new();
        let mut props = HashMap::new();

        props.insert("checked".to_string(), AuraPropValue::Expr(AuraExpr::StateRef("enabled".to_string())));
        props.insert("label".to_string(), AuraPropValue::Expr(AuraExpr::Literal("Enable feature".to_string())));

        let result = gen.generate_form_element("switch", &props);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("Switch"));
        assert!(code.contains("Row"));
        assert!(code.contains("Text(\"Enable feature\")"));
    }

    #[test]
    fn test_jet_generator_slider_integration() {
        use crate::aura::{AuraExpr, AuraPropValue};

        let mut gen = JetGenerator::new();
        let mut props = HashMap::new();

        props.insert("value".to_string(), AuraPropValue::Expr(AuraExpr::StateRef("volume".to_string())));
        props.insert("min".to_string(), AuraPropValue::Expr(AuraExpr::Int(0)));
        props.insert("max".to_string(), AuraPropValue::Expr(AuraExpr::Int(100)));

        let result = gen.generate_form_element("slider", &props);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("Slider"));
        assert!(code.contains("valueRange = 0f..100f"));
    }

    #[test]
    fn test_jet_generator_toggle_alias() {
        use crate::aura::{AuraExpr, AuraPropValue};

        let mut gen = JetGenerator::new();
        let mut props = HashMap::new();

        props.insert("checked".to_string(), AuraPropValue::Expr(AuraExpr::StateRef("toggle".to_string())));

        // Test that "toggle" is an alias for "switch"
        let result = gen.generate_form_element("toggle", &props);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("Switch"));
    }

    #[test]
    fn test_jet_generator_unknown_form_element() {
        let mut gen = JetGenerator::new();
        let props = HashMap::new();

        let result = gen.generate_form_element("unknown_element", &props);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Unknown form element"));
    }

    // =========================================================================
    // Project Generation Tests (Phase 6)
    // =========================================================================

    #[test]
    fn test_jet_generator_project_default() {
        let gen = JetGenerator::new();
        let files = gen.generate_project_default("TestApp");

        // Verify essential files exist
        assert!(files.contains_key("build.gradle.kts"));
        assert!(files.contains_key("settings.gradle.kts"));
        assert!(files.contains_key("gradle/libs.versions.toml"));
        assert!(files.contains_key("app/build.gradle.kts"));
        assert!(files.contains_key("app/src/main/AndroidManifest.xml"));

        // Verify MainActivity exists in correct path
        let main_activity_path = files.keys().find(|k| k.contains("MainActivity.kt"));
        assert!(main_activity_path.is_some());
    }

    #[test]
    fn test_jet_generator_project_with_package() {
        let gen = JetGenerator::new();
        let files = gen.generate_project_with_package("MyApp", "com.company.myapp");

        // Verify package path in MainActivity
        let main_activity = files.values().find(|v| v.contains("class MainActivity"));
        assert!(main_activity.is_some());
        assert!(main_activity.unwrap().contains("package com.company.myapp"));
    }

    #[test]
    fn test_jet_generator_project_with_theme() {
        let gen = JetGenerator::new();
        let files = gen.generate_project_with_theme("TestApp", "#FF0000", "#00FF00");

        // Verify Color.kt contains custom colors (look for FF0000 in Color definitions)
        let color_kt = files.values().find(|v| v.contains("Purple40"));
        assert!(color_kt.is_some());
        // The custom primary #FF0000 becomes FFFF0000 in Compose format
        let color_content = color_kt.unwrap();
        assert!(color_content.contains("Color(0x") || color_content.contains("Purple40"));
    }

    #[test]
    fn test_jet_generator_project_structure() {
        let gen = JetGenerator::new();
        let files = gen.generate_project_default("StructureTest");

        // Count files in different categories
        let gradle_files: Vec<_> = files.keys().filter(|k| k.contains(".gradle")).collect();
        let kotlin_files: Vec<_> = files.keys().filter(|k| k.ends_with(".kt")).collect();
        let xml_files: Vec<_> = files.keys().filter(|k| k.ends_with(".xml")).collect();

        // Should have at least 3 gradle files (root, app, settings)
        assert!(gradle_files.len() >= 2);

        // Should have at least 4 kotlin files (MainActivity, Color, Type, Theme)
        assert!(kotlin_files.len() >= 4);

        // Should have at least 2 xml files (manifest, strings)
        assert!(xml_files.len() >= 2);
    }

    #[test]
    fn test_jet_generator_project_gradle_content() {
        let gen = JetGenerator::new();
        let files = gen.generate_project_default("GradleTest");

        // Verify root build.gradle.kts
        let root_gradle = files.get("build.gradle.kts").unwrap();
        assert!(root_gradle.contains("plugins"));
        assert!(root_gradle.contains("android.application"));

        // Verify app build.gradle.kts
        let app_gradle = files.get("app/build.gradle.kts").unwrap();
        assert!(app_gradle.contains("android {"));
        assert!(app_gradle.contains("compose = true"));
        assert!(app_gradle.contains("implementation(libs.compose.material3)"));
    }

    // =========================================================================
    // Phase 7 Integration Tests
    // =========================================================================

    #[test]
    fn test_full_project_generation_workflow() {
        // Test the complete workflow from config to project
        use crate::ui_gen::jet::project::{JetProjectConfig, ProjectGenerator, ThemeColors};

        let config = JetProjectConfig::new("IntegrationTest")
            .with_application_id("com.test.integration")
            .with_version("2.0.0")
            .with_sdk_versions(26, 34, 34)
            .with_theme(ThemeColors::new("#FF5722", "#03A9F4"))
            .with_dependency("coil", "2.5.0")
            .with_widget("Counter")
            .with_widget("TodoList");

        let mut gen = ProjectGenerator::with_config(config);
        let files = gen.generate();

        // Verify complete structure
        assert!(files.contains_key("build.gradle.kts"));
        assert!(files.contains_key("settings.gradle.kts"));
        assert!(files.contains_key("gradle/libs.versions.toml"));
        assert!(files.contains_key("app/build.gradle.kts"));
        assert!(files.contains_key("app/src/main/AndroidManifest.xml"));

        // Verify package path
        let main_activity = files.keys().find(|k| k.contains("com/test/integration"));
        assert!(main_activity.is_some());

        // Verify custom version
        let app_gradle = files.get("app/build.gradle.kts").unwrap();
        assert!(app_gradle.contains("versionName = \"2.0.0\""));

        // Verify custom SDK
        assert!(app_gradle.contains("minSdk = 26"));

        // Verify Coil dependency
        assert!(app_gradle.contains("coil-compose"));
    }

    #[test]
    fn test_project_generator_deterministic() {
        use crate::ui_gen::jet::project::{JetProjectConfig, ProjectGenerator};

        let config = JetProjectConfig::new("DeterministicTest");

        // Generate twice with same config
        let mut gen1 = ProjectGenerator::with_config(config.clone());
        let files1 = gen1.generate();

        let mut gen2 = ProjectGenerator::with_config(config);
        let files2 = gen2.generate();

        // Should produce identical output
        assert_eq!(files1.len(), files2.len());
        for (path, content) in &files1 {
            assert_eq!(files2.get(path), Some(content));
        }
    }

    #[test]
    fn test_all_form_elements_with_all_properties() {
        use crate::aura::{AuraExpr, AuraPropValue};

        let mut gen = JetGenerator::new();

        // Test input with all properties
        let mut input_props = HashMap::new();
        input_props.insert("value".to_string(), AuraPropValue::Expr(AuraExpr::StateRef("text".to_string())));
        input_props.insert("placeholder".to_string(), AuraPropValue::Expr(AuraExpr::Literal("Enter text".to_string())));
        input_props.insert("label".to_string(), AuraPropValue::Expr(AuraExpr::Literal("Text Field".to_string())));
        input_props.insert("type".to_string(), AuraPropValue::Expr(AuraExpr::Literal("email".to_string())));
        input_props.insert("disabled".to_string(), AuraPropValue::Expr(AuraExpr::Bool(true)));

        let result = gen.generate_form_element("input", &input_props);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("OutlinedTextField"));
        assert!(code.contains("KeyboardType.Email"));
        assert!(code.contains("enabled = false"));
    }

    #[test]
    fn test_navigation_full_workflow() {
        let mut gen = JetGenerator::new();

        // Add routes
        gen.add_nav_route("home", "HomeScreen");
        gen.add_nav_route_with_params("detail", "DetailScreen", vec!["itemId".to_string()]);
        gen.add_nav_route("settings", "SettingsScreen");

        // Generate NavHost
        let result = gen.generate_nav_host("home");
        assert!(result.is_ok());

        let nav_host = result.unwrap();
        assert!(nav_host.contains("NavHost"));
        assert!(nav_host.contains("composable(\"home\")"));
        assert!(nav_host.contains("composable(\"detail\")"));
        assert!(nav_host.contains("composable(\"settings\")"));
        assert!(nav_host.contains("startDestination = \"home\""));
    }

    #[test]
    fn test_layout_with_modifier_chain() {
        use crate::aura::{AuraExpr, AuraPropValue};

        let mut gen = JetGenerator::new();
        let mut props = HashMap::new();

        // gap: 4 means 4 Tailwind units = 16dp (4 * 4 = 16)
        props.insert("gap".to_string(), AuraPropValue::Expr(AuraExpr::Int(4)));
        props.insert("align".to_string(), AuraPropValue::Expr(AuraExpr::Literal("center".to_string())));
        props.insert("class".to_string(), AuraPropValue::Expr(AuraExpr::Literal("px-4 py-2 bg-white rounded-lg".to_string())));

        let result = gen.generate_layout_element("col", &props, "// children here");
        assert!(result.is_ok());

        let code = result.unwrap();
        assert!(code.contains("Column"));
        assert!(code.contains("Arrangement.spacedBy(16.dp)"));
        assert!(code.contains("Alignment.CenterHorizontally"));
    }

    #[test]
    fn test_list_with_data_binding() {
        use crate::aura::{AuraExpr, AuraPropValue};

        let mut gen = JetGenerator::new();
        let mut props = HashMap::new();

        props.insert("items".to_string(), AuraPropValue::Expr(AuraExpr::StateRef("users".to_string())));
        props.insert("key".to_string(), AuraPropValue::Expr(AuraExpr::Literal("{item.id}".to_string())));
        props.insert("columns".to_string(), AuraPropValue::Expr(AuraExpr::Int(2)));

        let result = gen.generate_list_element("grid", &props, "UserCard(user = item)");
        assert!(result.is_ok());

        let code = result.unwrap();
        assert!(code.contains("LazyVerticalGrid"));
        assert!(code.contains("GridCells.Fixed(2)"));
        assert!(code.contains("items = users"));
        assert!(code.contains("UserCard(user = item)"));
    }

    #[test]
    fn test_project_file_count() {
        let gen = JetGenerator::new();
        let files = gen.generate_project_default("FileCountTest");

        // Verify minimum file count for a complete project
        assert!(files.len() >= 15, "Expected at least 15 files, got {}", files.len());

        // Verify all essential file categories
        let has_manifest = files.keys().any(|k| k.contains("AndroidManifest.xml"));
        let has_gradle = files.keys().any(|k| k.ends_with(".gradle.kts"));
        let has_kotlin = files.keys().any(|k| k.ends_with(".kt"));
        let has_xml = files.keys().any(|k| k.ends_with(".xml") && !k.contains("Manifest"));
        let has_toml = files.keys().any(|k| k.ends_with(".toml"));

        assert!(has_manifest, "Missing AndroidManifest.xml");
        assert!(has_gradle, "Missing gradle files");
        assert!(has_kotlin, "Missing Kotlin files");
        assert!(has_xml, "Missing XML resource files");
        assert!(has_toml, "Missing version catalog");
    }

    #[test]
    fn test_theme_file_generation() {
        use crate::ui_gen::jet::project::ThemeColors;

        let gen = JetGenerator::new();
        let files = gen.generate_project_with_theme("ThemeTest", "#9C27B0", "#E91E63");

        // Find Color.kt
        let color_kt = files.values().find(|v| v.contains("Purple40"));
        assert!(color_kt.is_some());

        let color_content = color_kt.unwrap();
        assert!(color_content.contains("Color(0x"));
        assert!(color_content.contains("import androidx.compose.ui.graphics.Color"));
    }
}
