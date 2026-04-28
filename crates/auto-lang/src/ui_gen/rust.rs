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
            code.push_str("use auto_lang::ui::{Component, View};\n\n");
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

        // Default impl delegates to new()
        code.push_str(&format!(
            "impl Default for {} {{\n    fn default() -> Self {{ Self::new() }}\n}}\n",
            widget_name
        ));

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

    /// Check if a tag is a leaf element that has no children (text, button, etc.)
    fn is_leaf_tag(&self, tag: &str) -> bool {
        matches!(tag, "text" | "label" | "span" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "p" | "button")
    }

    /// Generate view tree code
    fn generate_view_tree(&mut self, node: &AuraNode) -> String {
        match node {
            AuraNode::Element { tag, props, events, children } => {
                let view_fn = self.tag_to_view_fn(tag);

                // Leaf tags (text, button) use ViewBuilder pattern: View::text(()).build()
                // Layout tags (col, row) use View::col() directly (returns ViewBuilder)
                let builder_start = if self.is_leaf_tag(tag.as_str()) {
                    format!("View::{}(())", view_fn)
                } else {
                    format!("View::{}()", view_fn)
                };

                if children.is_empty() {
                    // Single element without children
                    let mut builder = builder_start;

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
                    let mut builder = builder_start;

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
                    "class" | "className" => {
                        // Strip surrounding quotes from the expression if present
                        let class_str = value_str.trim_matches('"');
                        tailwind_to_methods(builder, class_str)
                    }
                    "style" => {
                        // Strip surrounding quotes from the expression if present
                        let style_str = value_str.trim_matches('"');
                        tailwind_to_methods(builder, style_str)
                    }
                    "padding" => format!("{}.padding({})", builder, value_str),
                    "spacing" => format!("{}.spacing({})", builder, value_str),
                    _ => builder.to_string(),
                }
            }
            AuraPropValue::StyleBinding(bindings) => {
                // For Rust, generate conditional class application
                let class_conditions: Vec<String> = bindings.iter()
                    .map(|b| {
                        let cond = self.expr_to_rust(&b.condition);
                        format!("if {} {{ \"{}\" }} else {{ \"\" }}", cond, b.style_name)
                    })
                    .collect();
                if class_conditions.is_empty() {
                    builder.to_string()
                } else {
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
            LogicPayload::AstStmts(_) => {
                "// TODO: a2ts delegation not yet supported for Rust backend".to_string()
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

/// Convert a Tailwind class string (e.g. "gap-4 p-4 bg-white items-center")
/// into chained method calls on a builder expression (e.g. ".gap(4).p(4).bg(\"white\").items_center()").
///
/// Classes that are not recognized are silently skipped so the generated code
/// always compiles.
fn tailwind_to_methods(builder: &str, class_str: &str) -> String {
    let mut result = builder.to_string();

    for class in class_str.split_whitespace() {
        result.push_str(&tailwind_single_to_method(class));
    }

    result
}

/// Convert a single Tailwind class token to a builder method call string.
fn tailwind_single_to_method(class: &str) -> String {
    // --- Spacing ---
    if let Some(rest) = class.strip_prefix("p-") {
        if rest == "0" { return ".p(0)".to_string(); }
        if let Ok(n) = rest.parse::<u16>() { return format!(".p({})", n); }
    }
    if let Some(rest) = class.strip_prefix("px-") {
        if let Ok(n) = rest.parse::<u16>() { return format!(".px({})", n); }
    }
    if let Some(rest) = class.strip_prefix("py-") {
        if let Ok(n) = rest.parse::<u16>() { return format!(".py({})", n); }
    }
    if let Some(rest) = class.strip_prefix("m-") {
        if let Ok(n) = rest.parse::<u16>() { return format!(".m({})", n); }
    }
    if let Some(rest) = class.strip_prefix("mx-") {
        if let Ok(n) = rest.parse::<u16>() { return format!(".mx({})", n); }
    }
    if let Some(rest) = class.strip_prefix("my-") {
        if let Ok(n) = rest.parse::<u16>() { return format!(".my({})", n); }
    }
    if let Some(rest) = class.strip_prefix("gap-") {
        if let Ok(n) = rest.parse::<u16>() { return format!(".gap({})", n); }
    }

    // --- Colors ---
    if let Some(color) = class.strip_prefix("bg-") {
        return format!(".bg(\"{}\")", color);
    }
    // text-{color} must come after text size/alignment checks below,
    // but we handle it here and let the ordering in match below
    // override for known text- keywords.

    // --- Sizing ---
    if class == "w-full" { return ".w_full()".to_string(); }
    if let Some(rest) = class.strip_prefix("w-") {
        if let Ok(n) = rest.parse::<u16>() { return format!(".w({})", n); }
    }
    if class == "h-full" { return ".h_full()".to_string(); }
    if let Some(rest) = class.strip_prefix("h-") {
        if let Ok(n) = rest.parse::<u16>() { return format!(".h({})", n); }
    }

    // --- Layout ---
    match class {
        "flex" => return ".flex()".to_string(),
        "flex-1" => return ".flex1()".to_string(),
        "flex-row" => return ".flex_row()".to_string(),
        "flex-col" => return ".flex_col()".to_string(),
        "items-center" => return ".items_center()".to_string(),
        "items-start" => return ".items_start()".to_string(),
        "items-end" => return ".items_end()".to_string(),
        "justify-center" => return ".justify_center()".to_string(),
        "justify-between" => return ".justify_between()".to_string(),
        "justify-start" => return String::new(), // no direct method, skip
        "justify-end" => return String::new(),    // no direct method, skip
        _ => {}
    }

    // --- Border radius ---
    match class {
        "rounded" => return ".rounded()".to_string(),
        "rounded-sm" => return ".rounded_sm()".to_string(),
        "rounded-md" => return ".rounded_md()".to_string(),
        "rounded-lg" => return ".rounded_lg()".to_string(),
        _ => {}
    }

    // --- Border ---
    if class == "border" { return ".border()".to_string(); }

    // --- Typography (text size) ---
    match class {
        "text-xs" | "text-sm" | "text-base" | "text-lg" | "text-xl" | "text-2xl" | "text-3xl" => {
            // These are font-size utilities; for now emit as a comment-style pass-through.
            // They have no direct builder method on layout builders.
            return String::new();
        }
        _ => {}
    }

    // --- Font weight ---
    match class {
        "font-bold" => return ".font_bold()".to_string(),
        "font-medium" => return ".font_medium()".to_string(),
        "font-normal" => return String::new(),
        _ => {}
    }

    // --- Text alignment ---
    match class {
        "text-center" | "text-left" | "text-right" => return String::new(),
        _ => {}
    }

    // --- Text color (must come after text-size/align) ---
    if let Some(color) = class.strip_prefix("text-") {
        return format!(".text_color(\"{}\")", color);
    }

    // --- Effects ---
    match class {
        "shadow" | "shadow-sm" | "shadow-md" | "shadow-lg" | "shadow-xl" | "shadow-2xl" | "shadow-none" => {
            return String::new(); // no direct builder method yet
        }
        _ => {}
    }

    // --- Opacity ---
    if class.starts_with("opacity-") { return String::new(); }

    // --- Position ---
    if class == "relative" || class == "absolute" { return String::new(); }

    // --- Z-index ---
    if class.starts_with("z-") { return String::new(); }

    // --- Overflow ---
    if class.starts_with("overflow") { return String::new(); }

    // --- Grid ---
    if class == "grid" || class.starts_with("grid-") { return String::new(); }
    if class.starts_with("col-") || class.starts_with("row-") { return String::new(); }

    // Unknown class -- skip silently
    String::new()
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
            tick_interval: None,
            handler_params: HashMap::new(),
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

    // ========== Plan 180 Phase 7: tailwind_to_methods tests ==========

    #[test]
    fn test_tailwind_single_padding() {
        assert_eq!(tailwind_single_to_method("p-4"), ".p(4)");
    }

    #[test]
    fn test_tailwind_single_padding_xy() {
        assert_eq!(tailwind_single_to_method("px-4"), ".px(4)");
        assert_eq!(tailwind_single_to_method("py-2"), ".py(2)");
    }

    #[test]
    fn test_tailwind_single_margin() {
        assert_eq!(tailwind_single_to_method("m-4"), ".m(4)");
        assert_eq!(tailwind_single_to_method("mx-2"), ".mx(2)");
        assert_eq!(tailwind_single_to_method("my-2"), ".my(2)");
    }

    #[test]
    fn test_tailwind_single_gap() {
        assert_eq!(tailwind_single_to_method("gap-4"), ".gap(4)");
    }

    #[test]
    fn test_tailwind_single_bg() {
        assert_eq!(tailwind_single_to_method("bg-white"), ".bg(\"white\")");
        assert_eq!(tailwind_single_to_method("bg-blue-500"), ".bg(\"blue-500\")");
    }

    #[test]
    fn test_tailwind_single_width() {
        assert_eq!(tailwind_single_to_method("w-full"), ".w_full()");
        assert_eq!(tailwind_single_to_method("w-10"), ".w(10)");
    }

    #[test]
    fn test_tailwind_single_height() {
        assert_eq!(tailwind_single_to_method("h-full"), ".h_full()");
        assert_eq!(tailwind_single_to_method("h-12"), ".h(12)");
    }

    #[test]
    fn test_tailwind_single_layout() {
        assert_eq!(tailwind_single_to_method("flex"), ".flex()");
        assert_eq!(tailwind_single_to_method("flex-1"), ".flex1()");
        assert_eq!(tailwind_single_to_method("flex-row"), ".flex_row()");
        assert_eq!(tailwind_single_to_method("flex-col"), ".flex_col()");
        assert_eq!(tailwind_single_to_method("items-center"), ".items_center()");
        assert_eq!(tailwind_single_to_method("justify-center"), ".justify_center()");
        assert_eq!(tailwind_single_to_method("justify-between"), ".justify_between()");
    }

    #[test]
    fn test_tailwind_single_border_radius() {
        assert_eq!(tailwind_single_to_method("rounded"), ".rounded()");
        assert_eq!(tailwind_single_to_method("rounded-sm"), ".rounded_sm()");
        assert_eq!(tailwind_single_to_method("rounded-md"), ".rounded_md()");
        assert_eq!(tailwind_single_to_method("rounded-lg"), ".rounded_lg()");
    }

    #[test]
    fn test_tailwind_single_border() {
        assert_eq!(tailwind_single_to_method("border"), ".border()");
    }

    #[test]
    fn test_tailwind_single_font_weight() {
        assert_eq!(tailwind_single_to_method("font-bold"), ".font_bold()");
        assert_eq!(tailwind_single_to_method("font-medium"), ".font_medium()");
    }

    #[test]
    fn test_tailwind_single_text_color() {
        assert_eq!(tailwind_single_to_method("text-slate-500"), ".text_color(\"slate-500\")");
    }

    #[test]
    fn test_tailwind_to_methods_chain() {
        let result = tailwind_to_methods("View::col()", "gap-4 p-4 bg-white items-center");
        assert_eq!(result, "View::col().gap(4).p(4).bg(\"white\").items_center()");
    }

    #[test]
    fn test_tailwind_to_methods_empty() {
        let result = tailwind_to_methods("View::col()", "");
        assert_eq!(result, "View::col()");
    }

    #[test]
    fn test_tailwind_to_methods_unknown_classes_skipped() {
        let result = tailwind_to_methods("View::col()", "p-4 unknown-class gap-2");
        assert_eq!(result, "View::col().p(4).gap(2)");
    }

    #[test]
    fn test_tailwind_to_methods_complex() {
        let result = tailwind_to_methods(
            "View::row()",
            "w-full h-full justify-center items-center bg-white"
        );
        assert_eq!(
            result,
            "View::row().w_full().h_full().justify_center().items_center().bg(\"white\")"
        );
    }
}
