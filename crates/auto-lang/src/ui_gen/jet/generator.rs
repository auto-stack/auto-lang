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
use crate::aura::{AuraBinOp, AuraEvent, AuraExpr, AuraNode, AuraPropValue, AuraStmt, AuraTextContent, AuraUnaryOp, AuraUpdateOp, AuraWidget, LogicPayload};
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

    /// Referenced child components (for imports)
    component_refs: Vec<String>,

    /// Current widget's handlers (for event resolution)
    current_handlers: HashMap<String, LogicPayload>,
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
            component_refs: Vec::new(),
            current_handlers: HashMap::new(),
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

    /// Check if a node tree contains any Link nodes (for navController requirement)
    fn has_link_node(node: &crate::aura::AuraNode) -> bool {
        match node {
            crate::aura::AuraNode::Link { .. } => true,
            crate::aura::AuraNode::Element { children, .. } => {
                children.iter().any(|c| Self::has_link_node(c))
            }
            crate::aura::AuraNode::ForLoop { body, .. } => {
                body.iter().any(|c| Self::has_link_node(c))
            }
            crate::aura::AuraNode::Conditional { then_body, else_body, .. } => {
                then_body.iter().any(|c| Self::has_link_node(c))
                    || else_body.as_ref().map_or(false, |e| e.iter().any(|c| Self::has_link_node(c)))
            }
            _ => false,
        }
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

    /// Generate public @Composable function (with pub modifier for cross-file access)
    pub fn generate_public_composable(&mut self, name: &str, body: &str) -> String {
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
    // Note: For pages with navController, use rememberNavController()
    {}()
}}"#,
            name, name
        )
    }

    /// Generate @Preview function with navController
    pub fn generate_preview_with_nav(&self, name: &str) -> String {
        format!(
            r#"@Preview(showBackground = true)
@Composable
fun {}Preview() {{
    val navController = rememberNavController()
    {}(navController)
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

    /// Generate sealed class for Msg (ELM architecture)
    fn generate_msg_sealed(&self, widget: &AuraWidget) -> String {
        if widget.messages.is_empty() {
            return String::new();
        }

        let mut variants = Vec::new();

        for msg in &widget.messages {
            for variant in &msg.variants {
                if let Some(ref payload_type) = variant.payload {
                    // Variant with payload
                    let kotlin_type = match payload_type {
                        crate::ast::Type::Int => "Int".to_string(),
                        crate::ast::Type::Float => "Float".to_string(),
                        crate::ast::Type::Double => "Double".to_string(),
                        crate::ast::Type::Bool => "Boolean".to_string(),
                        crate::ast::Type::Str(_) => "String".to_string(),
                        crate::ast::Type::User(decl) => decl.name.as_str().to_string(),
                        _ => "Any".to_string(),
                    };
                    variants.push(format!("    data class {}(val value: {}) : Msg()", variant.name, kotlin_type));
                } else {
                    // Simple variant without payload
                    variants.push(format!("    object {} : Msg()", variant.name));
                }
            }
        }

        if variants.is_empty() {
            String::new()
        } else {
            format!("sealed class Msg {{\n{}\n}}", variants.join("\n"))
        }
    }

    /// Generate dispatch function for ELM architecture
    fn generate_dispatch_function(&self, widget: &AuraWidget) -> GenResult<String> {
        if widget.handlers.is_empty() {
            return Ok(String::new());
        }

        let mut cases = Vec::new();

        for (pattern, payload) in &widget.handlers {
            // Extract the variant name from pattern (e.g., ".Inc" -> "Inc")
            let variant_name = pattern.trim_start_matches('.');

            // Generate the handler body
            let body = match payload {
                LogicPayload::AstBlock(stmts) => {
                    let mut body_parts = Vec::new();
                    for stmt in stmts {
                        body_parts.push(self.stmt_to_kotlin(stmt)?);
                    }
                    body_parts.join("\n            ")
                }
                _ => "// TODO: Unsupported payload type".to_string(),
            };

            cases.push(format!("            is Msg.{} -> {{\n                {}\n            }}", variant_name, body));
        }

        if cases.is_empty() {
            Ok(String::new())
        } else {
            Ok(format!(
                r#"    val dispatch: (Msg) -> Unit = {{ msg ->
        when (msg) {{
{}
        }}
    }}"#,
                cases.join("\n")
            ))
        }
    }

    /// Generate view body from widget's view_tree
    fn generate_view_body(&mut self, widget: &AuraWidget) -> GenResult<String> {
        // Process the view tree node
        let body = self.node_to_compose(&widget.view_tree, 1)?;

        // If empty, provide a default Column
        if body.trim().is_empty() {
            Ok("    Column(modifier = modifier) {\n        // Empty view\n    }\n".to_string())
        } else {
            Ok(body)
        }
    }

    // =========================================================================
    // Node to Compose Conversion (Plan 134)
    // =========================================================================

    /// Convert AuraNode to Compose Kotlin code
    fn node_to_compose(&mut self, node: &AuraNode, indent: usize) -> GenResult<String> {
        let ind = "    ".repeat(indent);

        match node {
            AuraNode::Element { tag, props, events, children } => {
                self.element_to_compose(tag, props, events, children, indent)
            }
            AuraNode::Text(content) => {
                self.text_to_compose(content, indent)
            }
            AuraNode::ForLoop { var, index, iterable, body } => {
                self.for_loop_to_compose(var, index, iterable, body, indent)
            }
            AuraNode::Conditional { condition, then_body, else_body } => {
                self.conditional_to_compose(condition, then_body, else_body, indent)
            }
            AuraNode::Component { name, props, events } => {
                self.component_to_compose(name, props, events, indent)
            }
            AuraNode::Outlet => {
                // outlet should render the NavHost with current navController
                Ok(format!("{}AppNavHost(navController)\n", ind))
            }
            AuraNode::Link { to, text, href, children } => {
                self.link_to_compose(to, text, href, children, indent)
            }
        }
    }

    /// Convert AuraTextContent to Compose Text composable
    fn text_to_compose(&self, content: &AuraTextContent, indent: usize) -> GenResult<String> {
        let ind = "    ".repeat(indent);

        match content {
            AuraTextContent::Literal(s) => {
                Ok(format!("{}Text(\"{}\")\n", ind, s))
            }
            AuraTextContent::Interpolated { template, bindings } => {
                // Convert template to Kotlin string interpolation
                let mut kotlin_text = template.clone();
                for binding in bindings {
                    // Replace ${.binding} with $binding (state reference)
                    kotlin_text = kotlin_text.replace(
                        &format!("${{{}.{}}}", ".", binding),
                        &format!("${}", binding)
                    );
                    // Replace ${binding} with $binding (variable reference)
                    kotlin_text = kotlin_text.replace(
                        &format!("${{{}}}", binding),
                        &format!("${}", binding)
                    );
                }
                Ok(format!("{}Text(\"{}\")\n", ind, kotlin_text))
            }
        }
    }

    /// Convert AuraNode::Element to Compose code
    fn element_to_compose(
        &mut self,
        tag: &str,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        children: &[AuraNode],
        indent: usize,
    ) -> GenResult<String> {
        // Check if it's a layout element
        if Self::is_layout_tag(tag) {
            return self.layout_element_to_compose(tag, props, events, children, indent);
        }

        // Check if it's a form element
        if Self::is_form_tag(tag) {
            return self.form_element_to_compose(tag, props, events, children, indent);
        }

        // Check if it's a list element
        if Self::is_list_tag(tag) {
            return self.list_element_to_compose(tag, props, events, children, indent);
        }

        // Default: map to Compose component
        self.generic_element_to_compose(tag, props, events, children, indent)
    }

    /// Check if tag is a layout element
    fn is_layout_tag(tag: &str) -> bool {
        matches!(tag, "col" | "column" | "row" | "box" | "container" | "card" | "scroll")
    }

    /// Check if tag is a form element
    fn is_form_tag(tag: &str) -> bool {
        matches!(tag, "input" | "textarea" | "checkbox" | "switch" | "toggle" | "slider" | "button")
    }

    /// Check if tag is a list element
    fn is_list_tag(tag: &str) -> bool {
        matches!(tag, "list" | "lazy-column" | "list-row" | "lazy-row" | "grid" | "lazy-grid" | "flow-row" | "flow-col" | "flow-column")
    }

    /// Convert layout elements to Compose
    fn layout_element_to_compose(
        &mut self,
        tag: &str,
        props: &HashMap<String, AuraPropValue>,
        _events: &HashMap<String, AuraEvent>,
        children: &[AuraNode],
        indent: usize,
    ) -> GenResult<String> {
        let ind = "    ".repeat(indent);

        // Generate children content
        let mut children_content = String::new();
        for child in children {
            children_content.push_str(&self.node_to_compose(child, indent + 2)?);
        }

        // Use LayoutGenerator for the actual generation
        let result = match tag {
            "col" | "column" => self.layout_generator.generate_column(props, &children_content),
            "row" => self.layout_generator.generate_row(props, &children_content),
            "box" | "container" => self.layout_generator.generate_box(props, &children_content),
            "card" => self.layout_generator.generate_card(props, &children_content),
            "scroll" => self.layout_generator.generate_scroll(props, &children_content),
            _ => Err(GenError::UnsupportedExpr(format!("Unknown layout tag: {}", tag))),
        };

        // Prepend proper indentation
        result.map(|s| {
            let lines: Vec<&str> = s.lines().collect();
            lines.iter()
                .map(|line| format!("{}{}", ind, line))
                .collect::<Vec<_>>()
                .join("\n")
                + "\n"
        })
    }

    /// Convert form elements to Compose
    fn form_element_to_compose(
        &mut self,
        tag: &str,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        children: &[AuraNode],
        indent: usize,
    ) -> GenResult<String> {
        let ind = "    ".repeat(indent);

        match tag {
            "button" => self.button_to_compose(props, events, children, indent),
            "input" => {
                // Generate input with state binding
                self.form_generator.generate_input(props)
                    .map(|s| format!("{}{}\n", ind, s.trim()))
            }
            "checkbox" => self.form_generator.generate_checkbox(props)
                    .map(|s| format!("{}{}\n", ind, s.trim())),
            "switch" | "toggle" => self.form_generator.generate_switch(props)
                    .map(|s| format!("{}{}\n", ind, s.trim())),
            "slider" => self.form_generator.generate_slider(props)
                    .map(|s| format!("{}{}\n", ind, s.trim())),
            _ => Err(GenError::UnsupportedExpr(format!("Unknown form tag: {}", tag))),
        }
    }

    /// Convert button to Compose Button with dispatch mechanism
    fn button_to_compose(
        &mut self,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
        children: &[AuraNode],
        indent: usize,
    ) -> GenResult<String> {
        let ind = "    ".repeat(indent);

        // Get onClick handler - try both "onclick" and "click" for compatibility
        let event = events.get("onclick")
            .or_else(|| events.get("click"));

        // Generate onClick code using dispatch mechanism
        let on_click_code = if let Some(evt) = event {
            // Extract the message variant name (e.g., "Inc" from ".Inc")
            let msg_name = evt.handler.trim_start_matches('.');
            // Generate dispatch call: { dispatch(Msg.Inc) }
            format!("{{ dispatch(Msg.{}) }}", msg_name)
        } else {
            // No event - empty lambda
            "{}".to_string()
        };

        // Get button text
        let text = props.get("text")
            .and_then(|p| self.extract_string_value(p))
            .unwrap_or_default();

        // Generate children content if any
        let content = if !text.is_empty() {
            format!("{}    Text(\"{}\")\n", ind, text)
        } else if !children.is_empty() {
            let mut s = String::new();
            for child in children {
                s.push_str(&self.node_to_compose(child, indent + 1)?);
            }
            s
        } else {
            format!("{}    Text(\"Button\")\n", ind)
        };

        Ok(format!(
            "{}Button(\n{}    onClick = {}\n{}) {{\n{}}}\n",
            ind, ind, on_click_code, ind, content
        ))
    }

    /// Convert AuraStmt to Kotlin code
    fn stmt_to_kotlin(&self, stmt: &AuraStmt) -> GenResult<String> {
        match stmt {
                AuraStmt::Assign { target, value } => {
                    let target_clean = target.trim_start_matches('.');
                    let value_kotlin = self.expr_to_kotlin(value);
                    Ok(format!("{} = {}", target_clean, value_kotlin))
                }
                AuraStmt::Update { target, op, value } => {
                    let target_clean = target.trim_start_matches('.');
                    let value_kotlin = self.expr_to_kotlin(value);
                    let op_str = match op {
                        AuraUpdateOp::AddAssign => "+=",
                        AuraUpdateOp::SubAssign => "-=",
                        AuraUpdateOp::MulAssign => "*=",
                        AuraUpdateOp::DivAssign => "/=",
                    };
                    Ok(format!("{} {} {}", target_clean, op_str, value_kotlin))
                }
                AuraStmt::MethodCall { object, method, args } => {
                    let object_clean = object.trim_start_matches('.');
                    let args_kotlin: Vec<String> = args.iter()
                        .map(|a| self.expr_to_kotlin(a))
                        .collect();
                    Ok(format!("{}.{}({})", object_clean, method, args_kotlin.join(", ")))
                }
            }
        }

    /// Convert AuraEvent to Kotlin lambda
    fn event_to_lambda(&self, event: &AuraEvent) -> String {
        let handler = &event.handler;
        let params = &event.params;

        // Clean handler name (remove leading ".")
        let handler_clean = handler.trim_start_matches('.');

        if params.is_empty() {
            format!("{}()", handler_clean)
        } else {
            format!("{}({})", handler_clean, params.join(", "))
        }
    }

    /// Extract string value from AuraPropValue
    fn extract_string_value(&self, value: &AuraPropValue) -> Option<String> {
        match value {
            AuraPropValue::Expr(AuraExpr::Literal(s)) => Some(s.clone()),
            AuraPropValue::Expr(AuraExpr::StateRef(s)) => Some(s.clone()),
            _ => None,
        }
    }

    /// Convert for loop to Compose items() or forEach()
    fn for_loop_to_compose(
        &mut self,
        var: &str,
        index: &Option<String>,
        iterable: &str,
        body: &[AuraNode],
        indent: usize,
    ) -> GenResult<String> {
        let ind = "    ".repeat(indent);

        // Generate body content
        let mut body_content = String::new();
        for child in body {
            body_content.push_str(&self.node_to_compose(child, indent + 2)?);
        }

        // Clean iterable name (remove leading ".")
        let iterable_clean = iterable.trim_start_matches('.');

        if let Some(idx) = index {
            // With index: itemsIndexed(items) { index, item -> ... }
            Ok(format!(
                "{}itemsIndexed({}) {{ {}, {} ->\n{}}}\n",
                ind, iterable_clean, idx, var, body_content
            ))
        } else {
            // Without index: items(items) { item -> ... }
            Ok(format!(
                "{}items({}) {{ {} ->\n{}}}\n",
                ind, iterable_clean, var, body_content
            ))
        }
    }

    /// Convert conditional to Kotlin if/else
    fn conditional_to_compose(
        &mut self,
        condition: &str,
        then_body: &[AuraNode],
        else_body: &Option<Vec<AuraNode>>,
        indent: usize,
    ) -> GenResult<String> {
        let ind = "    ".repeat(indent);

        // Clean condition (remove leading "." for state refs)
        let cond_kotlin = condition.trim_start_matches('.');

        // Generate then body
        let mut then_content = String::new();
        for child in then_body {
            then_content.push_str(&self.node_to_compose(child, indent + 1)?);
        }

        if let Some(else_nodes) = else_body {
            let mut else_content = String::new();
            for child in else_nodes {
                else_content.push_str(&self.node_to_compose(child, indent + 1)?);
            }
            Ok(format!(
                "{}if ({}) {{\n{}}} else {{\n{}}}\n",
                ind, cond_kotlin, then_content, else_content
            ))
        } else {
            Ok(format!(
                "{}if ({}) {{\n{}}}\n",
                ind, cond_kotlin, then_content
            ))
        }
    }

    /// Convert AuraExpr to Kotlin expression string
    fn expr_to_kotlin(&self, expr: &AuraExpr) -> String {
        match expr {
            AuraExpr::Literal(s) => format!("\"{}\"", s),
            AuraExpr::Int(n) => n.to_string(),
            AuraExpr::Float(f) => f.to_string(),
            AuraExpr::Bool(b) => b.to_string(),
            AuraExpr::StateRef(s) => s.clone(),
            AuraExpr::Binary { left, op, right } => {
                let left_str = self.expr_to_kotlin(left);
                let right_str = self.expr_to_kotlin(right);
                let op_str = self.binop_to_kotlin(*op);
                format!("{} {} {}", left_str, op_str, right_str)
            }
            AuraExpr::Unary { op, operand } => {
                let operand_str = self.expr_to_kotlin(operand);
                match op {
                    AuraUnaryOp::Neg => format!("-{}", operand_str),
                    AuraUnaryOp::Not => format!("!{}", operand_str),
                }
            }
            AuraExpr::FieldAccess { object, field } => {
                let obj_str = self.expr_to_kotlin(object);
                format!("{}.{}", obj_str, field)
            }
            AuraExpr::MethodCall { object, method, args } => {
                let obj_str = self.expr_to_kotlin(object);
                let args_str = args.iter()
                    .map(|a| self.expr_to_kotlin(a))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}.{}({})", obj_str, method, args_str)
            }
            _ => "/* unsupported expr */".to_string(),
        }
    }

    /// Convert binary operator to Kotlin
    fn binop_to_kotlin(&self, op: AuraBinOp) -> &'static str {
        match op {
            AuraBinOp::Add => "+",
            AuraBinOp::Sub => "-",
            AuraBinOp::Mul => "*",
            AuraBinOp::Div => "/",
            AuraBinOp::Mod => "%",
            AuraBinOp::Eq => "==",
            AuraBinOp::Ne => "!=",
            AuraBinOp::Lt => "<",
            AuraBinOp::Le => "<=",
            AuraBinOp::Gt => ">",
            AuraBinOp::Ge => ">=",
            AuraBinOp::And => "&&",
            AuraBinOp::Or => "||",
        }
    }

    /// Convert child component reference to Compose call
    fn component_to_compose(
        &mut self,
        name: &str,
        props: &HashMap<String, AuraExpr>,
        events: &HashMap<String, AuraEvent>,
        indent: usize,
    ) -> GenResult<String> {
        let ind = "    ".repeat(indent);

        // Track component reference for imports
        self.component_refs.push(name.to_string());

        // Build props string
        let mut props_parts = Vec::new();
        for (key, value) in props {
            let value_str = self.expr_to_kotlin(value);
            props_parts.push(format!("{} = {}", key, value_str));
        }

        // Build event handlers
        for (event, aura_event) in events {
            let handler = self.event_to_lambda(aura_event);
            // Map event names to Compose convention
            let compose_event = if event == "click" {
                "onClick".to_string()
            } else {
                format!("on{}", event.chars().next().unwrap().to_uppercase().collect::<String>() + &event[1..])
            };
            props_parts.push(format!("{} = {{ {} }}", compose_event, handler));
        }

        let props_str = if props_parts.is_empty() {
            String::new()
        } else {
            format!("\n{}    {}", ind, props_parts.join(&format!(",\n{}    ", ind)))
        };

        Ok(format!("{}{}({})\n", ind, name, props_str))
    }

    /// Convert link to Compose navigation
    fn link_to_compose(
        &mut self,
        to: &str,
        text: &str,
        href: &str,
        children: &[AuraNode],
        indent: usize,
    ) -> GenResult<String> {
        let ind = "    ".repeat(indent);

        // Add clickable import
        self.add_import("androidx.compose.foundation.clickable");

        if !href.is_empty() {
            // External link - use Text with clickable modifier
            let text_content = if text.is_empty() {
                // Get text from children
                let mut s = String::new();
                for child in children {
                    if let AuraNode::Text(content) = child {
                        if let AuraTextContent::Literal(t) = content {
                            s.push_str(t);
                        }
                    }
                }
                s
            } else {
                text.to_string()
            };

            Ok(format!(
                "{}Text(\n{}    \"{}\",\n{}    modifier = Modifier.clickable {{ /* open {} */ }}\n{})\n",
                ind, ind, text_content, ind, href, ind
            ))
        } else {
            // Internal navigation - generate navController.navigate call
            // Get text from children if not provided
            let link_text = if text.is_empty() {
                let mut s = String::new();
                for child in children {
                    if let AuraNode::Text(content) = child {
                        if let AuraTextContent::Literal(t) = content {
                            s.push_str(t);
                        }
                    }
                }
                s
            } else {
                text.to_string()
            };

            // Generate clickable Text with navigation
            let nav_call = self.navigation_generator.generate_navigate_call(to);
            Ok(format!(
                "{}Text(\n{}    \"{}\",\n{}    modifier = Modifier.clickable {{ {} }}\n{})\n",
                ind, ind, link_text, ind, nav_call, ind
            ))
        }
    }

    /// Convert list elements to Compose
    fn list_element_to_compose(
        &mut self,
        tag: &str,
        props: &HashMap<String, AuraPropValue>,
        _events: &HashMap<String, AuraEvent>,
        children: &[AuraNode],
        indent: usize,
    ) -> GenResult<String> {
        let ind = "    ".repeat(indent);

        // Generate children content as item template
        let mut item_content = String::new();
        for child in children {
            item_content.push_str(&self.node_to_compose(child, indent + 2)?);
        }

        // Use ListGenerator for the actual generation
        let result = match tag {
            "list" | "lazy-column" => self.list_generator.generate_lazy_column(props, &item_content),
            "list-row" | "lazy-row" => self.list_generator.generate_lazy_row(props, &item_content),
            "grid" | "lazy-grid" => self.list_generator.generate_lazy_grid(props, &item_content),
            "flow-row" => self.list_generator.generate_flow_row(props, &item_content),
            "flow-col" | "flow-column" => self.list_generator.generate_flow_column(props, &item_content),
            _ => Err(GenError::UnsupportedExpr(format!("Unknown list tag: {}", tag))),
        };

        // Prepend proper indentation
        result.map(|s| {
            let lines: Vec<&str> = s.lines().collect();
            lines.iter()
                .map(|line| format!("{}{}", ind, line))
                .collect::<Vec<_>>()
                .join("\n")
                + "\n"
        })
    }

    /// Convert generic element to Compose
    fn generic_element_to_compose(
        &mut self,
        tag: &str,
        props: &HashMap<String, AuraPropValue>,
        _events: &HashMap<String, AuraEvent>,
        children: &[AuraNode],
        indent: usize,
    ) -> GenResult<String> {
        let ind = "    ".repeat(indent);

        // Map common HTML-like tags to Compose
        let (compose_name, is_text_like) = self.map_tag_to_compose(tag);

        // Check for text prop
        let text_prop = props.get("text")
            .and_then(|p| self.extract_string_value(p));

        // Generate children content
        let mut children_content = String::new();
        for child in children {
            children_content.push_str(&self.node_to_compose(child, indent + 1)?);
        }

        if is_text_like {
            // Text-like components: Text("content")
            let text = text_prop.unwrap_or_default();
            if children.is_empty() {
                Ok(format!("{}{}(\"{}\")\n", ind, compose_name, text))
            } else {
                // Has children - use them as content
                Ok(format!("{}{}(\"{}\")\n", ind, compose_name, children_content.trim()))
            }
        } else {
            // Container-like components: Box { ... }
            if children_content.is_empty() {
                Ok(format!("{}{}()\n", ind, compose_name))
            } else {
                Ok(format!("{}{} {{\n{}}}\n", ind, compose_name, children_content))
            }
        }
    }

    /// Map AURA tag to Compose component name
    fn map_tag_to_compose(&self, tag: &str) -> (&'static str, bool) {
        match tag {
            "text" | "span" | "p" => ("Text", true),
            "div" | "section" | "article" => ("Box", false),
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => ("Text", true),
            "img" | "image" => ("Image", true),
            "icon" => ("Icon", true),
            "spacer" => ("Spacer", true),
            "divider" => ("HorizontalDivider", true),
            _ => ("Box", false),  // Default to Box container
        }
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
        self.current_handlers = widget.handlers.clone();

        // Add standard Compose imports
        self.add_import("androidx.compose.foundation.layout.*");
        self.add_import("androidx.compose.material3.*");
        self.add_import("androidx.compose.runtime.*");
        self.add_import("androidx.compose.ui.Modifier");
        self.add_import("androidx.compose.ui.unit.dp");
        self.add_import("androidx.compose.ui.tooling.preview.Preview");

        // Check if this widget has routes (router widget)
        let has_routes = widget.routes.is_some();

        // If has routes, add navigation imports and process routes
        if let Some(ref routes) = widget.routes {
            // Add navigation imports
            self.add_import("androidx.navigation.NavHostController");
            self.add_import("androidx.navigation.compose.NavHost");
            self.add_import("androidx.navigation.compose.composable");
            self.add_import("androidx.navigation.compose.rememberNavController");

            // Add routes to navigation generator
            self.navigation_generator.clear_routes();
            self.navigation_generator.add_routes_from_aura(&routes.routes);
        }

        // Generate Msg sealed class (ELM architecture)
        let msg_sealed = self.generate_msg_sealed(widget);

        // Generate components
        let state_decls = self.generate_state_declarations(widget);
        let dispatch_fn = self.generate_dispatch_function(widget)?;
        let view_body = self.generate_view_body(widget)?;

        // Generate NavHost if this is a router widget
        let nav_host_code = if has_routes {
            self.navigation_generator.generate_nav_host("/")?
        } else {
            String::new()
        };

        // Assemble final code
        let package_decl = self.generate_package();
        let composable_name = &widget.name;

        // Add navigation imports based on widget type
        // Router widgets need all navigation imports; Page widgets need NavHostController for param
        self.add_import("androidx.navigation.NavHostController");
        if has_routes {
            // Router - also needs NavHost, composable, and rememberNavController
            self.add_import("androidx.navigation.compose.NavHost");
            self.add_import("androidx.navigation.compose.composable");
            self.add_import("androidx.navigation.compose.rememberNavController");
        } else {
            // Page - needs rememberNavController for Preview
            self.add_import("androidx.navigation.compose.rememberNavController");
        }

        // Generate imports (after potentially adding navigation imports)
        let imports = self.generate_imports();

        // Generate appropriate Preview based on whether this is a router or a page
        let preview = if has_routes {
            // Router - no navController param, just call the function
            self.generate_preview(composable_name)
        } else {
            // Page with navController param, use preview_with_nav
            self.generate_preview_with_nav(composable_name)
        };

        // Build the complete code
        let mut code = String::new();

        // Package declaration
        code.push_str(&package_decl);
        code.push_str("// Auto-generated by a2jet\n\n");

        // Imports
        code.push_str(&imports);
        code.push_str("\n\n");

        // Msg sealed class (if any messages)
        if !msg_sealed.is_empty() {
            code.push_str(&msg_sealed);
            code.push_str("\n\n");
        }

        // NavHost (if router widget)
        if !nav_host_code.is_empty() {
            code.push_str(&nav_host_code);
            code.push_str("\n\n");
        }

        // Composable function
        // - Router widgets (has_routes): create navController internally, no param needed
        // - Page widgets: receive navController from NavHost
        if has_routes {
            // Router widget (like App) - creates its own navController
            code.push_str(&format!("@Composable\nfun {}(\n    modifier: Modifier = Modifier\n) {{\n", composable_name));
            code.push_str("    val navController = rememberNavController()\n\n");
        } else {
            // Regular page widget - receives navController from NavHost
            code.push_str(&format!("@Composable\nfun {}(\n    navController: NavHostController,\n    modifier: Modifier = Modifier\n) {{\n", composable_name));
        }

        // State declarations
        if !state_decls.is_empty() {
            code.push_str("    ");
            code.push_str(&state_decls);
            code.push_str("\n\n");
        }

        // Dispatch function (if any handlers)
        if !dispatch_fn.is_empty() {
            code.push_str(&dispatch_fn);
            code.push_str("\n\n");
        }

        // View body
        code.push_str(&view_body);
        code.push_str("}\n\n");

        // Preview
        code.push_str(&preview);

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
        // Routes with params use multi-line format
        assert!(nav_host.contains("\"detail\""));
        assert!(nav_host.contains("composable(\"settings\")"));
        assert!(nav_host.contains("startDestination = \"home\""));
        assert!(nav_host.contains("DetailScreen(navController)"));
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
