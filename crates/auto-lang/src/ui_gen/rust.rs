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

use super::{BackendGenerator, GenError, GenResult};
use crate::aura::{AuraExpr, AuraMessage, AuraMsgVariant, AuraNode, AuraStateDef, AuraStmt, AuraTextContent, AuraWidget, LogicPayload};
use std::collections::HashMap;

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
}

impl RustGenerator {
    /// Create a new Rust generator
    pub fn new() -> Self {
        Self {
            current_widget: None,
            message_variants: Vec::new(),
            needs_imports: true,
            indent: 0,
        }
    }

    /// Reset state for new widget
    fn reset(&mut self) {
        self.message_variants.clear();
        self.needs_imports = true;
        self.indent = 0;
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
    fn generate_view_method(&self, widget: &AuraWidget) -> String {
        let mut code = String::new();

        code.push_str("    fn view(&self) -> View<Self::Msg> {\n");

        // Generate view tree
        let view_code = self.generate_view_tree(&widget.view_tree);
        code.push_str(&format!("        {}\n", view_code));

        code.push_str("    }\n");

        code
    }

    /// Generate view tree code
    fn generate_view_tree(&self, node: &AuraNode) -> String {
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
                        // Generate format! string
                        // Template has ${.binding} format, convert to Rust format!({binding})
                        let mut format_str = template.clone();
                        for binding in bindings.iter() {
                            // Replace ${.binding} with {binding}
                            format_str = format_str.replace(
                                &format!("${{{}.{}}}", ".", binding),
                                &format!("{{{}}}", binding)
                            );
                            // Also replace ${binding} with {binding}
                            format_str = format_str.replace(
                                &format!("${{{}}}", binding),
                                &format!("{{{}}}", binding)
                            );
                        }
                        // Generate self.binding for state references
                        let binding_refs: Vec<String> = bindings.iter()
                            .map(|b| format!("self.{}", b))
                            .collect();
                        format!("View::text(format!(\"{}\", {}))", format_str, binding_refs.join(", "))
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

                let body_code: Vec<String> = body.iter()
                    .map(|child| self.generate_view_tree(child))
                    .collect();

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
                    builder = self.add_prop_to_builder(&builder, key, value);
                }

                for (event, handler) in events {
                    builder = self.add_event_to_builder(&builder, event, handler);
                }

                format!("{}.build()", builder)
            }
        }
    }

    /// Convert AURA condition to Rust expression
    fn convert_condition(&self, condition: &str) -> String {
        condition.trim().to_string()
    }

    /// Map tag to View builder function
    fn tag_to_view_fn(&self, tag: &str) -> &'static str {
        match tag {
            "col" | "column" => "col",
            "row" => "row",
            "button" => "button",
            "text" | "label" | "span" => "text",
            "h1" | "h2" | "h3" => "text",
            "input" => "input",
            "center" => "center",
            _ => "col",
        }
    }

    /// Add property to builder
    fn add_prop_to_builder(&self, builder: &str, key: &str, value: &AuraExpr) -> String {
        let value_str = self.expr_to_rust(value);
        match key {
            "class" | "className" => format!("{}.class(\"{}\")", builder, value_str),
            "style" => format!("{}.style(\"{}\")", builder, value_str),
            "padding" => format!("{}.padding({})", builder, value_str),
            "spacing" => format!("{}.spacing({})", builder, value_str),
            _ => builder.to_string(),
        }
    }

    /// Add event to builder
    fn add_event_to_builder(&self, builder: &str, event: &str, handler: &str) -> String {
        let handler_fn = self.handler_to_rust_closure(handler);
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
    fn handler_to_rust_closure(&self, handler: &str) -> String {
        let variant = self.extract_variant_name(handler);
        format!("|_| Msg::{}", variant)
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
            crate::ast::Type::Str(_) => "String".to_string(),
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
