//! Rust/GPUI Code Generator
//!
//! Generates Rust code implementing the `Component` trait from AURA widgets.
//!
//! ## Output Format
//!
//! ```ignore
//! // Auto-generated from Auto language
//! // DO NOT EDIT - changes will be overwritten
//!
//! use auto_ui::prelude::*;
//!
//! #[derive(Clone, Copy, Debug, PartialEq)]
//! pub enum Msg {
//!     Inc,
//!     Dec,
//! }
//!
//! #[derive(Debug)]
//! pub struct Counter {
//!     pub count: i32,
//! }
//!
//! impl Counter {
//!     pub fn new() -> Self {
//!         Self {
//!             count: 0,
//!         }
//!     }
//! }
//!
//! impl Component for Counter {
//!     type Msg = Msg;
//!
//!     fn on(&mut self, msg: Self::Msg) {
//!         match msg {
//!             Msg::Inc => {
//!                 self.count += 1;
//!             }
//!             Msg::Dec => {
//!                 self.count -= 1;
//!             }
//!         }
//!     }
//!
//!     fn view(&self) -> View<Self::Msg> {
//!         View::col()
//!             .child(View::button("+").on_click(|_| Msg::Inc))
//!             .child(View::text(&format!("Count: {}", self.count)))
//!             .build()
//!     }
//! }
//! ```
//!
//! Based on auto-ui/trans/rust_gen.rs, adapted for AuraWidget input.

use super::{BackendGenerator, GenResult};
use crate::aura::{AuraEvent, AuraExpr, AuraMsgVariant, AuraNode, AuraPropValue, AuraStmt, AuraTextContent, AuraWidget, LogicPayload};

/// Rust/GPUI code generator
pub struct RustGenerator {
    /// Current widget name
    current_widget: Option<String>,

    /// Collected message variants
    message_variants: Vec<AuraMsgVariant>,

    /// Whether we need imports
    needs_imports: bool,

    /// Indent level
    indent: usize,

    /// Loop variables in scope (for generating correct references)
    loop_vars: Vec<String>,
}

impl RustGenerator {
    /// Create a new Rust generator
    pub fn new() -> Self {
        Self {
            current_widget: None,
            message_variants: Vec::new(),
            needs_imports: true,
            indent: 0,
            loop_vars: Vec::new(),
        }
    }

    /// Reset state for new widget
    fn reset(&mut self) {
        self.message_variants.clear();
        self.needs_imports = true;
        self.indent = 0;
        self.loop_vars.clear();
    }

    /// Check if a name is a loop variable
    fn is_loop_var(&self, name: &str) -> bool {
        self.loop_vars.contains(&name.to_string())
    }

    /// Push loop variables into scope
    fn push_loop_vars(&mut self, var: &str, index: Option<&str>) {
        self.loop_vars.push(var.to_string());
        if let Some(idx) = index {
            self.loop_vars.push(idx.to_string());
        }
    }

    /// Pop loop variables from scope
    fn pop_loop_vars(&mut self, var: &str, index: Option<&str>) {
        self.loop_vars.retain(|v| v != var);
        if let Some(idx) = index {
            self.loop_vars.retain(|v| v != idx);
        }
    }

    /// Generate complete Rust code from AuraWidget
    pub fn generate_rust(&mut self, widget: &AuraWidget) -> GenResult<String> {
        self.current_widget = Some(widget.name.clone());
        self.reset();

        // Collect all message variants
        for msg in &widget.messages {
            for variant in &msg.variants {
                self.message_variants.push(variant.clone());
            }
        }

        let mut code = String::new();

        // File header
        code.push_str("// Auto-generated from Auto language\n");
        code.push_str("// DO NOT EDIT - changes will be overwritten\n\n");

        // Imports
        if self.needs_imports {
            code.push_str("use auto_ui::prelude::*;\n\n");
        }

        // Message enum
        if !self.message_variants.is_empty() {
            code.push_str(&self.generate_msg_enum()?);
            code.push('\n');
        }

        // Struct definition
        code.push_str(&self.generate_struct(widget));
        code.push('\n');

        // Constructor
        code.push_str(&self.generate_constructor(widget));
        code.push('\n');

        // Component impl
        code.push_str(&self.generate_component_impl(widget));

        // Computed properties impl (if any)
        if !widget.computed.is_empty() {
            code.push('\n');
            code.push_str(&self.generate_computed_impl(widget));
        }

        Ok(code)
    }

    /// Generate Msg enum definition
    fn generate_msg_enum(&self) -> GenResult<String> {
        let mut code = String::new();

        code.push_str("#[derive(Clone, Copy, Debug, PartialEq)]\n");
        code.push_str("pub enum Msg {\n");

        for variant in &self.message_variants {
            code.push_str(&format!("    {},\n", variant.name));
        }

        code.push_str("}\n");

        Ok(code)
    }

    /// Generate struct definition
    fn generate_struct(&self, widget: &AuraWidget) -> String {
        let mut code = String::new();

        code.push_str("#[derive(Debug)]\n");
        code.push_str(&format!("pub struct {} {{\n", widget.name));

        for state in &widget.state_vars {
            let field_name = &state.name;
            let field_type = self.auto_type_to_rust(&state.type_info);
            code.push_str(&format!("    pub {}: {},\n", field_name, field_type));
        }

        code.push_str("}\n");

        code
    }

    /// Generate constructor
    fn generate_constructor(&self, widget: &AuraWidget) -> String {
        let widget_name = &widget.name;
        let mut code = String::new();

        code.push_str(&format!("impl {} {{\n", widget_name));

        // new() constructor
        code.push_str("    pub fn new() -> Self {\n");
        code.push_str("        Self {\n");

        for state in &widget.state_vars {
            let init = self.expr_to_rust(&state.initial);
            code.push_str(&format!("            {}: {},\n", state.name, init));
        }

        code.push_str("        }\n");
        code.push_str("    }\n");
        code.push_str("}\n");

        code
    }

    /// Generate Component trait implementation
    fn generate_component_impl(&mut self, widget: &AuraWidget) -> String {
        let widget_name = &widget.name;
        let mut code = String::new();

        code.push_str(&format!("impl Component for {} {{\n", widget_name));

        // Message type
        let msg_type = if !self.message_variants.is_empty() {
            "Msg"
        } else {
            "()"
        };
        code.push_str(&format!("    type Msg = {};\n\n", msg_type));

        // on() method
        code.push_str(&self.generate_on_method(widget));
        code.push('\n');

        // view() method
        code.push_str(&self.generate_view_method(widget));

        code.push_str("}\n");

        code
    }

    /// Generate on() method implementation
    fn generate_on_method(&self, widget: &AuraWidget) -> String {
        let mut code = String::new();

        code.push_str("    fn on(&mut self, msg: Self::Msg) {\n");

        if !self.message_variants.is_empty() {
            code.push_str("        match msg {\n");

            // Generate match arms from handlers
            for (pattern, payload) in &widget.handlers {
                let variant_name = self.extract_variant_name(pattern);
                let body = self.generate_handler_body(payload);
                code.push_str(&format!("            Msg::{} => {{\n", variant_name));
                code.push_str(&format!("                {}\n", body));
                code.push_str("            }\n");
            }

            code.push_str("            _ => {}\n");
            code.push_str("        }\n");
        }

        code.push_str("    }\n");

        code
    }

    /// Generate view() method implementation
    fn generate_view_method(&mut self, widget: &AuraWidget) -> String {
        let mut code = String::new();

        code.push_str("    fn view(&self) -> View<Self::Msg> {\n");

        // Generate view tree
        let view_code = self.generate_view_tree(&widget.view_tree);
        code.push_str(&format!("        {}\n", view_code));

        code.push_str("    }\n");

        code
    }

    /// Generate computed properties impl block
    fn generate_computed_impl(&self, widget: &AuraWidget) -> String {
        let widget_name = &widget.name;
        let mut code = String::new();

        code.push_str(&format!("impl {} {{\n", widget_name));

        for computed_prop in &widget.computed {
            let method_name = &computed_prop.name;
            let expr_rust = self.expr_to_rust(&computed_prop.expr);

            // Generate getter method
            code.push_str(&format!("    pub fn {}(&self) -> impl std::fmt::Display {{\n", method_name));
            code.push_str(&format!("        {}\n", expr_rust));
            code.push_str("    }\n\n");
        }

        code.push_str("}\n");

        code
    }

    /// Generate view tree code
    fn generate_view_tree(&mut self, node: &AuraNode) -> String {
        match node {
            AuraNode::Element { tag, props, events, children } => {
                let view_fn = self.tag_to_view_fn(tag);

                if children.is_empty() {
                    // Single element without children
                    let mut builder = format!("View::{}()", view_fn);

                    // Add props
                    for (key, value) in props {
                        builder = self.add_prop_to_builder(&builder, key, value);
                    }

                    // Add events
                    for (event, handler) in events {
                        builder = self.add_event_to_builder(&builder, event, handler);
                    }

                    format!("{}.build()", builder)
                } else {
                    // Element with children
                    let mut builder = format!("View::{}()", view_fn);

                    // Add props and events
                    for (key, value) in props {
                        builder = self.add_prop_to_builder(&builder, key, value);
                    }

                    // Add children
                    for child in children {
                        let child_code = self.generate_view_tree(child);
                        builder = format!("{}.child({})", builder, child_code);
                    }

                    // Add events last
                    for (event, handler) in events {
                        builder = self.add_event_to_builder(&builder, event, handler);
                    }

                    format!("{}.build()", builder)
                }
            }

            AuraNode::Text(content) => {
                match content {
                    AuraTextContent::Literal(s) => {
                        format!("View::text(\"{}\")", s)
                    }
                    AuraTextContent::Interpolated { template, bindings } => {
                        // Convert template to format! string with {} placeholders
                        let mut format_str = template.clone();
                        let mut format_args = Vec::new();

                        for binding in bindings.iter() {
                            // Replace ${.binding} and ${binding} with {}
                            format_str = format_str.replace(
                                &format!("${{{}.{}}}", ".", binding),
                                "{}"
                            );
                            format_str = format_str.replace(
                                &format!("${{{}}}", binding),
                                "{}"
                            );

                            // Use binding directly if loop var, otherwise self.binding
                            let arg = if self.is_loop_var(binding) {
                                binding.clone()
                            } else {
                                format!("self.{}", binding)
                            };
                            format_args.push(arg);
                        }

                        format!("View::text(format!(\"{}\", {}))", format_str, format_args.join(", "))
                    }
                }
            }

            AuraNode::ForLoop { var, index, iterable, body } => {
                // Generate iterator-based view construction
                let iter_expr = if iterable.starts_with('.') {
                    format!("self.{}", iterable.trim_start_matches('.'))
                } else {
                    iterable.clone()
                };

                // Push loop vars into scope
                self.push_loop_vars(var, index.as_deref());

                // Generate body with loop vars in scope
                let body_code: Vec<String> = body.iter()
                    .map(|child| self.generate_view_tree(child))
                    .collect();

                // Pop loop vars from scope
                self.pop_loop_vars(var, index.as_deref());

                if let Some(idx) = index {
                    format!("{}.enumerate().map(|({}, {})| {{ {} }}).collect::<Vec<_>>()", iter_expr, idx, var, body_code.join("\n"))
                } else {
                    format!("{}.iter().map(|{}| {{ {} }}).collect::<Vec<_>>()", iter_expr, var, body_code.join("\n"))
                }
            }

            AuraNode::Conditional { condition, then_body, else_body } => {
                let rust_condition = self.convert_condition(condition);
                let then_code: Vec<String> = then_body.iter()
                    .map(|child| self.generate_view_tree(child))
                    .collect();

                if let Some(else_nodes) = else_body {
                    let else_code: Vec<String> = else_nodes.iter()
                        .map(|child| self.generate_view_tree(child))
                        .collect();
                    format!("if {} {{ {} }} else {{ {} }}", rust_condition, then_code.join("\n"), else_code.join("\n"))
                } else {
                    format!("if {} {{ {} }}", rust_condition, then_code.join("\n"))
                }
            }

            AuraNode::Component { name, props, events } => {
                // Generate component instantiation
                let mut builder = format!("{}::new()", name);

                for (key, value) in props {
                    builder = self.add_prop_to_builder(&builder, key, &AuraPropValue::Expr(value.clone()));
                }

                for (event, handler) in events {
                    builder = self.add_event_to_builder(&builder, event, handler);
                }

                format!("{}.build()", builder)
            }

            // Plan 105: Router outlet and link
            AuraNode::Outlet => {
                // Rust router outlet placeholder
                "View::outlet()".to_string()
            }

            AuraNode::Link { to, text, href, children } => {
                // Rust router link or external link
                let children_code: Vec<String> = children.iter()
                    .map(|child| self.generate_view_tree(child))
                    .collect();

                if !href.is_empty() {
                    // External link
                    let text_content = if text.is_empty() {
                        children_code.join(", ")
                    } else {
                        format!("\"{}\"", text)
                    };
                    format!("View::external_link(\"{}\").text({})", href, text_content)
                } else {
                    let text_arg = if text.is_empty() {
                        String::new()
                    } else {
                        format!(".text(\"{}\")", text)
                    };
                    format!("View::link(\"{}\").children(vec![{}]){}.build()", to, children_code.join(", "), text_arg)
                }
            }
        }
    }

    /// Convert AURA condition to Rust expression
    fn convert_condition(&self, condition: &str) -> String {
        // Replace . with self. for state references
        let mut result = condition.trim().to_string();

        // Simple approach: replace ".name" at word boundaries with "self.name"
        // This handles cases like ".count > 0" -> "self.count > 0"
        result = result.replace(".", "self.");

        // Fix double self references
        result = result.replace("self.self.", "self.");

        result
    }

    /// Map tag to View builder function
    fn tag_to_view_fn(&self, tag: &str) -> &'static str {
        match tag {
            // Layout
            "col" | "column" => "col",
            "row" => "row",
            "grid" => "grid",
            "scroll" => "scroll",
            "container" => "container",

            // Content
            "button" => "button",
            "input" => "input",
            "textarea" => "textarea",
            "checkbox" => "checkbox",
            "toggle" => "toggle",
            "select" => "select",
            "option" => "option",
            "link" => "link",

            // Typography
            "text" | "label" | "span" => "text",
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => "text",
            "p" => "text",

            // Data
            "table" => "table",
            "thead" => "thead",
            "tbody" => "tbody",
            "tr" => "tr",
            "th" => "th",
            "td" => "td",
            "tree" => "tree",
            "tree_item" => "tree_item",

            // Navigation
            "tabs" => "tabs",
            "tab" => "tab",

            // Overlay
            "modal" => "modal",
            "tooltip" => "tooltip",

            // Form
            "slider" => "slider",
            "radio" => "radio",
            "radiogroup" => "radiogroup",

            // Feedback
            "progress" => "progress",
            "badge" => "badge",
            "spinner" => "spinner",

            // Display
            "card" => "card",
            "avatar" => "avatar",

            // Media
            "image" => "image",
            "icon" => "icon",

            // Utility
            "divider" => "divider",
            "spacer" => "spacer",

            _ => "col",
        }
    }

    /// Add property to builder
    fn add_prop_to_builder(&self, builder: &str, key: &str, value: &AuraPropValue) -> String {
        match value {
            AuraPropValue::Expr(expr) => {
                let value_str = self.expr_to_rust(expr);
                match key {
                    "class" | "className" => format!("{}.class(\"{}\")", builder, value_str),
                    "style" => format!("{}.style(\"{}\")", builder, value_str),
                    "padding" => format!("{}.padding({})", builder, value_str),
                    "spacing" => format!("{}.spacing({})", builder, value_str),
                    _ => builder.to_string(),
                }
            }
            AuraPropValue::StyleBinding(bindings) => {
                // For Rust, we'll generate conditional class application
                // This is a simplified approach - a real implementation would need to
                // integrate with the view builder pattern
                let class_conditions: Vec<String> = bindings.iter()
                    .map(|b| {
                        let cond = self.expr_to_rust(&b.condition);
                        format!("if {} {{ \"{}\" }} else {{ \"\" }}", cond, b.style_name)
                    })
                    .collect();
                if class_conditions.is_empty() {
                    builder.to_string()
                } else {
                    // Generate a combined conditional class string
                    format!("{}.class({})", builder, class_conditions.join(" + \" \" + "))
                }
            }
        }
    }

    /// Add event to builder
    fn add_event_to_builder(&self, builder: &str, event: &str, aura_event: &AuraEvent) -> String {
        let handler_fn = self.handler_to_rust_closure_with_params(&aura_event.handler, &aura_event.params);
        match event {
            "onclick" | "onClick" | "on_click" => {
                format!("{}.on_click({})", builder, handler_fn)
            }
            "onchange" | "onChange" => {
                format!("{}.on_change({})", builder, handler_fn)
            }
            _ => builder.to_string(),
        }
    }

    /// Convert handler pattern to Rust closure
    #[allow(dead_code)]
    fn handler_to_rust_closure(&self, handler: &str) -> String {
        let variant = self.extract_variant_name(handler);
        format!("|_| Msg::{}", variant)
    }

    /// Convert handler pattern to Rust closure with parameters
    fn handler_to_rust_closure_with_params(&self, handler: &str, params: &[String]) -> String {
        let variant = self.extract_variant_name(handler);
        if params.is_empty() {
            format!("|_| Msg::{}", variant)
        } else {
            format!("|_| Msg::{}({})", variant, params.join(", "))
        }
    }

    /// Extract variant name from pattern (e.g., "Msg::Inc" or ".Inc" -> "Inc")
    fn extract_variant_name(&self, pattern: &str) -> String {
        if pattern.starts_with('.') {
            pattern[1..].to_string()
        } else if let Some(variant) = pattern.split("::").last() {
            variant.to_string()
        } else {
            pattern.to_string()
        }
    }

    /// Generate handler body from LogicPayload
    fn generate_handler_body(&self, payload: &LogicPayload) -> String {
        match payload {
            LogicPayload::AstBlock(stmts) => {
                let bodies: Vec<String> = stmts.iter()
                    .map(|s| self.stmt_to_rust(s))
                    .collect();
                bodies.join(";\n                ")
            }
            LogicPayload::Bytecode(_) => {
                "// bytecode handler".to_string()
            }
        }
    }

    /// Convert AuraStmt to Rust
    fn stmt_to_rust(&self, stmt: &AuraStmt) -> String {
        match stmt {
            AuraStmt::Assign { target, value } => {
                let value_str = self.expr_to_rust(value);
                format!("self.{} = {}", target, value_str)
            }
            AuraStmt::Update { target, op, value } => {
                let value_str = self.expr_to_rust(value);
                let op_str = match op {
                    crate::aura::AuraUpdateOp::AddAssign => "+=",
                    crate::aura::AuraUpdateOp::SubAssign => "-=",
                    crate::aura::AuraUpdateOp::MulAssign => "*=",
                    crate::aura::AuraUpdateOp::DivAssign => "/=",
                };
                format!("self.{} {} {}", target, op_str, value_str)
            }
            AuraStmt::MethodCall { object, method, args } => {
                let args_str: Vec<String> = args.iter()
                    .map(|a| self.expr_to_rust(a))
                    .collect();
                format!("self.{}.{}({})", object, method, args_str.join(", "))
            }
        }
    }

    /// Convert AuraExpr to Rust
    fn expr_to_rust(&self, expr: &AuraExpr) -> String {
        match expr {
            AuraExpr::Literal(s) => format!("\"{}\"", s),
            AuraExpr::Int(n) => n.to_string(),
            AuraExpr::Float(n) => n.to_string(),
            AuraExpr::Bool(b) => b.to_string(),
            AuraExpr::StateRef(name) => format!("self.{}", name),
            AuraExpr::Binary { left, op, right } => {
                let left_str = self.expr_to_rust(left);
                let right_str = self.expr_to_rust(right);
                let op_str = self.bin_op_to_rust(op);
                format!("{} {} {}", left_str, op_str, right_str)
            }
            AuraExpr::Unary { op, operand } => {
                let operand_str = self.expr_to_rust(operand);
                let op_str = match op {
                    crate::aura::AuraUnaryOp::Neg => "-",
                    crate::aura::AuraUnaryOp::Not => "!",
                };
                format!("{}{}", op_str, operand_str)
            }
            AuraExpr::MsgVariant { msg_type, variant } => {
                format!("{}::{}", msg_type, variant)
            }
            AuraExpr::MethodCall { object, method, args } => {
                let object_str = self.expr_to_rust(object);
                let args_str: Vec<String> = args.iter()
                    .map(|a| self.expr_to_rust(a))
                    .collect();
                // Convert .len to .len() for Rust
                if method == "len" && args.is_empty() {
                    format!("{}.len()", object_str)
                } else if method == "filter" {
                    // filter takes a closure
                    format!("{}.{}({})", object_str, method, args_str.join(", "))
                } else {
                    format!("{}.{}({})", object_str, method, args_str.join(", "))
                }
            }
            AuraExpr::Array(elems) => {
                let elems_str: Vec<String> = elems.iter()
                    .map(|e| self.expr_to_rust(e))
                    .collect();
                format!("vec![{}]", elems_str.join(", "))
            }
            AuraExpr::Object(fields) => {
                let pairs: Vec<String> = fields.iter()
                    .map(|(k, v)| format!("{}: {}", k, self.expr_to_rust(v)))
                    .collect();
                format!("{{{}}}", pairs.join(", "))
            }
            AuraExpr::Lambda { params, body } => {
                let body_str = self.expr_to_rust(body);
                format!("|{}| {}", params.join(", "), body_str)
            }
            AuraExpr::FieldAccess { object, field } => {
                let object_str = self.expr_to_rust(object);
                format!("{}.{}", object_str, field)
            }
            AuraExpr::NavCall { path, params } => {
                let params_str: Vec<String> = params.iter()
                    .map(|(k, v)| format!("{}: {}", k, self.expr_to_rust(v)))
                    .collect();
                format!("nav_to(\"{}\", {{ {} }})", path, params_str.join(", "))
            }
            AuraExpr::Constructor { type_name, args } => {
                let args_str: Vec<String> = args.iter()
                    .map(|a| self.expr_to_rust(a))
                    .collect();
                format!("{}::new({})", type_name, args_str.join(", "))
            }
        }
    }

    /// Convert binary operator to Rust
    fn bin_op_to_rust(&self, op: &crate::aura::AuraBinOp) -> &'static str {
        match op {
            crate::aura::AuraBinOp::Add => "+",
            crate::aura::AuraBinOp::Sub => "-",
            crate::aura::AuraBinOp::Mul => "*",
            crate::aura::AuraBinOp::Div => "/",
            crate::aura::AuraBinOp::Mod => "%",
            crate::aura::AuraBinOp::Eq => "==",
            crate::aura::AuraBinOp::Ne => "!=",
            crate::aura::AuraBinOp::Lt => "<",
            crate::aura::AuraBinOp::Le => "<=",
            crate::aura::AuraBinOp::Gt => ">",
            crate::aura::AuraBinOp::Ge => ">=",
            crate::aura::AuraBinOp::And => "&&",
            crate::aura::AuraBinOp::Or => "||",
        }
    }

    /// Convert Auto type to Rust type
    fn auto_type_to_rust(&self, ty: &crate::ast::Type) -> String {
        match ty {
            crate::ast::Type::Int => "i32".to_string(),
            crate::ast::Type::Uint => "u32".to_string(),
            crate::ast::Type::I64 => "i64".to_string(),
            crate::ast::Type::U64 => "u64".to_string(),
            crate::ast::Type::Float => "f32".to_string(),
            crate::ast::Type::Double => "f64".to_string(),
            crate::ast::Type::Bool => "bool".to_string(),
            crate::ast::Type::Str(_) | crate::ast::Type::String => "String".to_string(),
            crate::ast::Type::Void => "()".to_string(),
            _ => "i32".to_string(), // Default fallback
        }
    }
}

impl BackendGenerator for RustGenerator {
    fn generate(&mut self, widget: &AuraWidget) -> GenResult<String> {
        self.generate_rust(widget)
    }

    fn extension(&self) -> &'static str {
        "rs"
    }
}

impl Default for RustGenerator {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Type;
    use crate::aura::{AuraMessage, AuraStateDef};
    use std::collections::HashMap;

    #[test]
    fn test_rust_generator_creation() {
        let gen = RustGenerator::new();
        assert!(gen.current_widget.is_none());
    }

    #[test]
    fn test_simple_counter() {
        let widget = AuraWidget {
            name: "Counter".to_string(),
            state_vars: vec![AuraStateDef {
                name: "count".to_string(),
                type_info: Type::Int,
                initial: AuraExpr::Int(0),
                decorators: vec![],
            }],
            messages: vec![AuraMessage {
                name: "Msg".to_string(),
                variants: vec![
                    AuraMsgVariant { name: "Inc".to_string(), payload: None },
                    AuraMsgVariant { name: "Dec".to_string(), payload: None },
                ],
            }],
            view_tree: AuraNode::element("col")
                .with_child(AuraNode::text("Count: 0")),
            handlers: HashMap::new(),
            props: vec![],
            computed: vec![],
            routes: None,
            lifecycle: vec![],
        };

        let mut gen = RustGenerator::new();
        let code = gen.generate(&widget).unwrap();

        assert!(code.contains("pub enum Msg"));
        assert!(code.contains("Inc"));
        assert!(code.contains("Dec"));
        assert!(code.contains("pub struct Counter"));
        assert!(code.contains("pub count: i32"));
        assert!(code.contains("impl Component for Counter"));
    }

    #[test]
    fn test_auto_type_to_rust() {
        let gen = RustGenerator::new();

        assert_eq!(gen.auto_type_to_rust(&Type::Int), "i32");
        assert_eq!(gen.auto_type_to_rust(&Type::Bool), "bool");
        assert_eq!(gen.auto_type_to_rust(&Type::Str(0)), "String");
        assert_eq!(gen.auto_type_to_rust(&Type::Float), "f32");
    }

    #[test]
    fn test_expr_to_rust() {
        let gen = RustGenerator::new();

        assert_eq!(gen.expr_to_rust(&AuraExpr::Int(42)), "42");
        assert_eq!(gen.expr_to_rust(&AuraExpr::Bool(true)), "true");
        assert_eq!(gen.expr_to_rust(&AuraExpr::StateRef("count".to_string())), "self.count");
    }

    #[test]
    fn test_extract_variant_name() {
        let gen = RustGenerator::new();

        assert_eq!(gen.extract_variant_name("Msg::Inc"), "Inc");
        assert_eq!(gen.extract_variant_name(".Inc"), "Inc");
        assert_eq!(gen.extract_variant_name("Dec"), "Dec");
    }

    #[test]
    fn test_tag_to_view_fn() {
        let gen = RustGenerator::new();

        assert_eq!(gen.tag_to_view_fn("col"), "col");
        assert_eq!(gen.tag_to_view_fn("button"), "button");
        assert_eq!(gen.tag_to_view_fn("text"), "text");
    }
}
