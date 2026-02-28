//! Iced Code Generator
//!
//! Generates Rust code using the Iced GUI framework (Elm-inspired).
//!
//! ## Output Format
//!
//! ```ignore
//! use iced::widget::{column, row, text, button, text_input, checkbox};
//! use iced::{Element, Application, Command, Settings};
//!
//! #[derive(Debug, Clone)]
//! pub enum Message {
//!     Increment,
//!     Decrement,
//! }
//!
//! pub struct Counter {
//!     count: i32,
//! }
//!
//! impl Application for Counter {
//!     type Executor = iced::executor::Default;
//!     type Message = Message;
//!     type Theme = iced::Theme;
//!     type Flags = ();
//!
//!     fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
//!         (Self { count: 0 }, Command::none())
//!     }
//!
//!     fn title(&self) -> String {
//!         "Counter".to_string()
//!     }
//!
//!     fn update(&mut self, message: Message) -> Command<Message> {
//!         match message {
//!             Message::Increment => self.count += 1,
//!             Message::Decrement => self.count -= 1,
//!         }
//!         Command::none()
//!     }
//!
//!     fn view(&self) -> Element<Message> {
//!         column![
//!             text(format!("Count: {}", self.count)),
//!             row![
//!                 button("-").on_press(Message::Decrement),
//!                 button("+").on_press(Message::Increment),
//!             ],
//!         ].into()
//!     }
//! }
//!
//! fn main() -> iced::Result {
//!     Counter::run(Settings::default())
//! }
//! ```
//!
//! Based on auto-ui/trans/rust_gen.rs, adapted for Iced.

use super::{BackendGenerator, GenError, GenResult};
use crate::aura::{AuraEvent, AuraExpr, AuraMessage, AuraMsgVariant, AuraNode, AuraStateDef, AuraStmt, AuraTextContent, AuraWidget, LogicPayload};
use std::collections::HashMap;

/// Iced code generator
pub struct IcedGenerator {
    /// Current widget name
    current_widget: Option<String>,

    /// Collected message variants
    message_variants: Vec<AuraMsgVariant>,
}

impl IcedGenerator {
    /// Create a new Iced generator
    pub fn new() -> Self {
        Self {
            current_widget: None,
            message_variants: Vec::new(),
        }
    }

    /// Reset state for new widget
    fn reset(&mut self) {
        self.message_variants.clear();
    }

    /// Generate complete Iced application from AuraWidget
    pub fn generate_iced(&mut self, widget: &AuraWidget) -> GenResult<String> {
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
        code.push_str("use iced::widget::{column, row, text, button, text_input, checkbox, Container};\n");
        code.push_str("use iced::{Element, Application, Command, Settings, Theme};\n\n");

        // Message enum
        if !self.message_variants.is_empty() {
            code.push_str(&self.generate_msg_enum()?);
            code.push('\n');
        }

        // Struct definition
        code.push_str(&self.generate_struct(widget));
        code.push('\n');

        // Application impl
        code.push_str(&self.generate_application_impl(widget));

        // Main function
        code.push_str(&self.generate_main(widget));

        Ok(code)
    }

    /// Generate Message enum definition
    fn generate_msg_enum(&self) -> GenResult<String> {
        let mut code = String::new();

        code.push_str("#[derive(Debug, Clone)]\n");
        code.push_str("pub enum Message {\n");

        for variant in &self.message_variants {
            if let Some(ref payload) = variant.payload {
                let rust_type = self.auto_type_to_iced(payload);
                code.push_str(&format!("    {}({}),\n", variant.name, rust_type));
            } else {
                code.push_str(&format!("    {},\n", variant.name));
            }
        }

        code.push_str("}\n");

        Ok(code)
    }

    /// Generate struct definition
    fn generate_struct(&self, widget: &AuraWidget) -> String {
        let mut code = String::new();

        code.push_str(&format!("pub struct {} {{\n", widget.name));

        for state in &widget.state_vars {
            let field_name = &state.name;
            let field_type = self.auto_type_to_iced(&state.type_info);
            code.push_str(&format!("    {}: {},\n", field_name, field_type));
        }

        code.push_str("}\n");

        code
    }

    /// Generate Application trait implementation
    fn generate_application_impl(&self, widget: &AuraWidget) -> String {
        let widget_name = &widget.name;
        let mut code = String::new();

        code.push_str(&format!("impl Application for {} {{\n", widget_name));

        // Associated types
        code.push_str("    type Executor = iced::executor::Default;\n");
        code.push_str("    type Message = Message;\n");
        code.push_str("    type Theme = Theme;\n");
        code.push_str("    type Flags = ();\n\n");

        // new() constructor
        code.push_str("    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {\n");
        code.push_str(&format!("        (Self {{\n"));
        for state in &widget.state_vars {
            let init = self.expr_to_iced(&state.initial);
            code.push_str(&format!("            {}: {},\n", state.name, init));
        }
        code.push_str("        }}, Command::none())\n");
        code.push_str("    }\n\n");

        // title()
        code.push_str(&format!("    fn title(&self) -> String {{\n"));
        code.push_str(&format!("        \"{}\".to_string()\n", widget_name));
        code.push_str("    }\n\n");

        // update()
        code.push_str(&self.generate_update_method(widget));
        code.push('\n');

        // view()
        code.push_str(&self.generate_view_method(widget));

        code.push_str("}\n\n");

        code
    }

    /// Generate update() method implementation
    fn generate_update_method(&self, widget: &AuraWidget) -> String {
        let mut code = String::new();

        code.push_str("    fn update(&mut self, message: Message) -> Command<Message> {\n");

        if !self.message_variants.is_empty() {
            code.push_str("        match message {\n");

            // Generate match arms from handlers
            for (pattern, payload) in &widget.handlers {
                let variant_name = self.extract_variant_name(pattern);
                let body = self.generate_handler_body(payload);
                code.push_str(&format!("            Message::{} => {{\n", variant_name));
                code.push_str(&format!("                {};\n", body));
                code.push_str("            }\n");
            }

            code.push_str("            _ => {}\n");
            code.push_str("        }\n");
        }

        code.push_str("        Command::none()\n");
        code.push_str("    }\n");

        code
    }

    /// Generate view() method implementation
    fn generate_view_method(&self, widget: &AuraWidget) -> String {
        let mut code = String::new();

        code.push_str("    fn view(&self) -> Element<Message> {\n");

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
                let container_fn = self.tag_to_iced_widget(tag);

                if children.is_empty() {
                    // Single element without children
                    let mut builder = self.create_widget(tag, props);

                    // Add events
                    for (event, aura_event) in events {
                        builder = self.add_event_to_builder(&builder, event, aura_event);
                    }

                    format!("{}.into()", builder)
                } else {
                    // Element with children
                    let mut builder = self.create_widget(tag, props);

                    // Add children
                    for child in children {
                        let child_code = self.generate_view_tree(child);
                        builder = format!("{}.push({})", builder, child_code);
                    }

                    // Add events last
                    for (event, aura_event) in events {
                        builder = self.add_event_to_builder(&builder, event, aura_event);
                    }

                    format!("{}.into()", builder)
                }
            }

            AuraNode::Text(content) => {
                match content {
                    AuraTextContent::Literal(s) => {
                        format!("text(\"{}\").into()", s)
                    }
                    AuraTextContent::Interpolated { template, bindings } => {
                        // Generate format! string
                        let mut format_str = template.clone();
                        for binding in bindings.iter() {
                            format_str = format_str.replace(
                                &format!("${{{}.{}}}", ".", binding),
                                &format!("{{{}}}", binding)
                            );
                            format_str = format_str.replace(
                                &format!("${{{}}}", binding),
                                &format!("{{{}}}", binding)
                            );
                        }
                        // Generate self.binding for state references
                        let binding_refs: Vec<String> = bindings.iter()
                            .map(|b| format!("self.{}", b))
                            .collect();
                        format!("text(format!(\"{}\", {})).into()", format_str, binding_refs.join(", "))
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
                    format!("{}.enumerate().map(|({}, {})| {{ {} }}).collect::<Vec<_>>().into()", iter_expr, idx, var, body_code.join("\n"))
                } else {
                    format!("{}.iter().map(|{}| {{ {} }}).collect::<Vec<_>>().into()", iter_expr, var, body_code.join("\n"))
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

                for (event, aura_event) in events {
                    builder = self.add_event_to_builder(&builder, event, aura_event);
                }

                format!("{}.into()", builder)
            }
        }
    }

    /// Create widget based on tag
    fn create_widget(&self, tag: &str, props: &HashMap<String, AuraExpr>) -> String {
        match tag {
            "col" | "column" => "column![]".to_string(),
            "row" => "row![]".to_string(),
            "button" => {
                if let Some(text_expr) = props.get("text") {
                    let text = self.expr_to_iced(text_expr);
                    format!("button({})", text)
                } else {
                    "button(\" \")".to_string()
                }
            }
            "text" | "label" | "span" | "h1" | "h2" | "h3" => {
                if let Some(text_expr) = props.get("text") {
                    let text = self.expr_to_iced(text_expr);
                    format!("text({})", text)
                } else {
                    "text(\" \")".to_string()
                }
            }
            "input" => {
                let placeholder = props.get("placeholder")
                    .map(|p| self.expr_to_iced(p))
                    .unwrap_or_else(|| "\"\"".to_string());
                format!("text_input({}, \"\")", placeholder)
            }
            _ => "column![]".to_string(),
        }
    }

    /// Map tag to Iced widget function
    fn tag_to_iced_widget(&self, tag: &str) -> &'static str {
        match tag {
            "col" | "column" => "column",
            "row" => "row",
            "button" => "button",
            "text" | "label" | "span" => "text",
            "input" => "text_input",
            _ => "column",
        }
    }

    /// Add property to builder
    fn add_prop_to_builder(&self, builder: &str, key: &str, value: &AuraExpr) -> String {
        let value_str = self.expr_to_iced(value);
        match key {
            "class" | "className" => format!("{}.style(|_| {{}})", builder), // TODO: Add styling
            "padding" => format!("{}.padding({})", builder, value_str),
            "spacing" => format!("{}.spacing({})", builder, value_str),
            _ => builder.to_string(),
        }
    }

    /// Add event to builder
    fn add_event_to_builder(&self, builder: &str, event: &str, aura_event: &AuraEvent) -> String {
        let handler = self.handler_to_iced_message(&aura_event.handler, &aura_event.params);
        match event {
            "onclick" | "onClick" | "on_click" => {
                format!("{}.on_press({})", builder, handler)
            }
            "onchange" | "onChange" => {
                format!("{}.on_input({})", builder, handler)
            }
            _ => builder.to_string(),
        }
    }

    /// Convert handler pattern to Iced Message
    fn handler_to_iced_message(&self, handler: &str, params: &[String]) -> String {
        let variant = self.extract_variant_name(handler);
        if params.is_empty() {
            format!("Message::{}", variant)
        } else {
            format!("Message::{}({})", variant, params.join(", "))
        }
    }

    /// Extract variant name from pattern
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
                    .map(|s| self.stmt_to_iced(s))
                    .collect();
                bodies.join("; ")
            }
            LogicPayload::Bytecode(_) => {
                "// bytecode handler".to_string()
            }
        }
    }

    /// Convert AuraStmt to Iced
    fn stmt_to_iced(&self, stmt: &AuraStmt) -> String {
        match stmt {
            AuraStmt::Assign { target, value } => {
                let value_str = self.expr_to_iced(value);
                format!("self.{} = {}", target, value_str)
            }
            AuraStmt::Update { target, op, value } => {
                let value_str = self.expr_to_iced(value);
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

    /// Convert AuraExpr to Iced
    fn expr_to_iced(&self, expr: &AuraExpr) -> String {
        match expr {
            AuraExpr::Literal(s) => format!("\"{}\".to_string()", s),
            AuraExpr::Int(n) => n.to_string(),
            AuraExpr::Float(n) => n.to_string(),
            AuraExpr::Bool(b) => b.to_string(),
            AuraExpr::StateRef(name) => format!("self.{}", name),
            AuraExpr::Binary { left, op, right } => {
                let left_str = self.expr_to_iced(left);
                let right_str = self.expr_to_iced(right);
                let op_str = self.bin_op_to_iced(op);
                format!("{} {} {}", left_str, op_str, right_str)
            }
            AuraExpr::Unary { op, operand } => {
                let operand_str = self.expr_to_iced(operand);
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

    /// Convert binary operator to Iced
    fn bin_op_to_iced(&self, op: &crate::aura::AuraBinOp) -> &'static str {
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

    /// Convert condition to Iced expression
    fn convert_condition(&self, condition: &str) -> String {
        // Replace . with self. for state references
        let mut result = condition.to_string();
        // Simple replacement: .name -> self.name
        if result.starts_with('.') {
            result = format!("self{}", result);
        }
        result
    }

    /// Convert Auto type to Iced type
    fn auto_type_to_iced(&self, ty: &crate::ast::Type) -> String {
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

    /// Generate main function
    fn generate_main(&self, widget: &AuraWidget) -> String {
        let mut code = String::new();

        code.push_str("fn main() -> iced::Result {\n");
        code.push_str(&format!("    {}::run(Settings::default())\n", widget.name));
        code.push_str("}\n");

        code
    }
}

impl BackendGenerator for IcedGenerator {
    fn generate(&mut self, widget: &AuraWidget) -> GenResult<String> {
        self.generate_iced(widget)
    }

    fn extension(&self) -> &'static str {
        "rs"
    }
}

impl Default for IcedGenerator {
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
    fn test_iced_generator_creation() {
        let gen = IcedGenerator::new();
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

        let mut gen = IcedGenerator::new();
        let code = gen.generate(&widget).unwrap();

        assert!(code.contains("pub enum Message"));
        assert!(code.contains("Inc"));
        assert!(code.contains("Dec"));
        assert!(code.contains("pub struct Counter"));
        assert!(code.contains("count: i32"));
        assert!(code.contains("impl Application for Counter"));
    }

    #[test]
    fn test_auto_type_to_iced() {
        let gen = IcedGenerator::new();

        assert_eq!(gen.auto_type_to_iced(&Type::Int), "i32");
        assert_eq!(gen.auto_type_to_iced(&Type::Bool), "bool");
        assert_eq!(gen.auto_type_to_iced(&Type::Str(0)), "String");
        assert_eq!(gen.auto_type_to_iced(&Type::Float), "f32");
    }

    #[test]
    fn test_extract_variant_name() {
        let gen = IcedGenerator::new();

        assert_eq!(gen.extract_variant_name("Msg::Inc"), "Inc");
        assert_eq!(gen.extract_variant_name(".Inc"), "Inc");
        assert_eq!(gen.extract_variant_name("Dec"), "Dec");
    }
}
