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

    /// Maps input event variant name to field name for input text parsing
    input_fields: std::collections::HashMap<String, String>,

    /// State var types for lookup during handler generation
    state_types: std::collections::HashMap<String, String>,
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
            input_fields: std::collections::HashMap::new(),
            state_types: std::collections::HashMap::new(),
        }
    }

    /// Reset state for new widget
    fn reset(&mut self) {
        self.message_variants.clear();
        self.input_fields.clear();
        self.state_types.clear();
        self.needs_imports = true;
        self.indent = 0;
        self.loop_vars.clear();
    }

    /// Convert a string containing `${.field}` markers to a Rust `format!()` call
    fn interpolate_str(&self, s: &str) -> String {
        let mut format_str = s.to_string();
        let mut format_args = Vec::new();

        // Extract ${.field} and ${field} patterns
        let re = regex::Regex::new(r"\$\{\.?(\w+)\}").unwrap();
        for cap in re.captures_iter(s) {
            let binding = &cap[1];
            let arg = if self.is_loop_var(binding) {
                binding.to_string()
            } else {
                format!("self.{}", binding)
            };
            if !format_args.contains(&arg) {
                format_args.push(arg);
            }
        }

        // Replace ${.field} and ${field} with {}
        format_str = re.replace_all(&format_str, "{}").to_string();

        if format_args.is_empty() {
            format!("\"{}\"", s)
        } else {
            format!("format!(\"{}\", {})", format_str, format_args.join(", "))
        }
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

        // Populate state_types for handler generation
        for state in &widget.state_vars {
            self.state_types.insert(state.name.clone(), self.auto_type_to_rust(&state.type_info));
        }

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

        // Pre-scan view tree for input event→field mappings
        self.scan_input_fields(&widget.view_tree);

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

                // If this event is from an input, prepend input text parsing
                if let Some(field_name) = self.input_fields.get(&variant_name) {
                    let rust_type = self.state_types.get(field_name).map(|s| s.as_str()).unwrap_or("f64");
                    code.push_str(&format!(
                        "                let _text = auto_lang::ui::iced::last_input_text();\n"
                    ));
                    if rust_type == "String" {
                        code.push_str(&format!(
                            "                self.{} = _text;\n",
                            field_name
                        ));
                    } else {
                        let parse_method = match rust_type {
                            "i32" => "parse::<i32>()",
                            "i64" => "parse::<i64>()",
                            "u32" => "parse::<u32>()",
                            "u64" => "parse::<u64>()",
                            "f32" => "parse::<f32>()",
                            "f64" => "parse::<f64>()",
                            "bool" => "parse::<bool>()",
                            _ => "parse::<f64>()",
                        };
                        code.push_str(&format!(
                            "                self.{} = _text.{}.unwrap_or(self.{});\n",
                            field_name, parse_method, field_name
                        ));
                    }

                    // Skip redundant self-assignment body (e.g. `.email = .email`)
                    let self_assign = format!("self.{} = self.{}", field_name, field_name);
                    if !body.trim().eq(&self_assign) && !body.trim().is_empty() {
                        code.push_str(&format!("                {}\n", body));
                    }
                } else {
                    code.push_str(&format!("                {}\n", body));
                }

                code.push_str("            }\n");
            }

            if self.message_variants.len() > widget.handlers.len() {
                code.push_str("            _ => {}\n");
            }
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

    /// Pre-scan view tree to find input elements and record event→field mappings
    fn scan_input_fields(&mut self, node: &AuraNode) {
        match node {
            AuraNode::Element { tag, props, events, children, .. } => {
                if tag == "input" {
                    if let Some(AuraPropValue::Expr(AuraExpr::StateRef(name))) = props.get("value") {
                        for (event, handler) in events {
                            if matches!(event.as_str(), "oninput" | "onInput" | "onchange" | "onChange") {
                                let variant = self.extract_variant_name(&handler.handler);
                                self.input_fields.insert(variant, name.clone());
                            }
                        }
                    }
                }
                for child in children {
                    self.scan_input_fields(child);
                }
            }
            _ => {}
        }
    }

    /// Generate view tree code
    fn generate_view_tree(&mut self, node: &AuraNode) -> String {
        match node {
            AuraNode::Element { tag, props, events, children, .. } => {
                let view_fn = self.tag_to_view_fn(tag);

                // For text elements with a "text" prop and no extra styling/events,
                // emit View::text("content") or View::text(format!(...)) directly.
                if tag == "text" && children.is_empty() && events.is_empty() {
                    let style_count = props.keys()
                        .filter(|k| *k != "text")
                        .count();
                    if style_count == 0 {
                        if let Some(AuraPropValue::Expr(AuraExpr::Literal(s))) = props.get("text") {
                            if s.contains("${") {
                                return format!("View::text({})", self.interpolate_str(s));
                            }
                            return format!("View::text(\"{}\".to_string())", s);
                        }
                        if let Some(AuraPropValue::Expr(AuraExpr::StateRef(name))) = props.get("text") {
                            return format!("View::text(format!(\"{{}}\", self.{}))", name);
                        }
                    } else {
                        // Text with styling — collect classes and use View::text_styled
                        let class_str = props.get("style")
                            .or_else(|| props.get("class"))
                            .and_then(|v| if let AuraPropValue::Expr(AuraExpr::Literal(s)) = v { Some(s.clone()) } else { None })
                            .unwrap_or_default();
                        if let Some(AuraPropValue::Expr(AuraExpr::StateRef(name))) = props.get("text") {
                            return format!("View::text_styled(format!(\"{{}}\", self.{}), \"{}\")", name, class_str);
                        }
                        if let Some(AuraPropValue::Expr(AuraExpr::Literal(s))) = props.get("text") {
                            return format!("View::text_styled(\"{}\".to_string(), \"{}\")", s, class_str);
                        }
                    }
                }

                // Special handling for input elements — View::input(placeholder).value(...).on_change(...)
                if tag == "input" {
                    let placeholder = props.get("placeholder")
                        .and_then(|v| if let AuraPropValue::Expr(AuraExpr::Literal(s)) = v { Some(s.clone()) } else { None })
                        .unwrap_or_default();

                    let mut builder = format!("View::input(\"{}\")", placeholder);

                    // Value binding: value: .field → .value(format!("{}", self.field))
                    if let Some(AuraPropValue::Expr(AuraExpr::StateRef(name))) = props.get("value") {
                        builder = format!("{}.value(format!(\"{{}}\", self.{}))", builder, name);
                    } else if let Some(AuraPropValue::Expr(AuraExpr::Literal(s))) = props.get("value") {
                        builder = format!("{}.value(\"{}\".to_string())", builder, s);
                    }

                    // Password mode: type: "password"
                    if let Some(AuraPropValue::Expr(AuraExpr::Literal(s))) = props.get("type") {
                        if s == "password" {
                            builder = format!("{}.password()", builder);
                        }
                    }

                    // Other props (class, style, width — skip placeholder, value, type)
                    for (key, value) in props {
                        if key == "placeholder" || key == "value" || key == "type" { continue; }
                        builder = self.add_prop_to_builder(&builder, key, value);
                    }

                    // Events: oninput/onchange → on_change (takes M, not a closure)
                    for (event, handler) in events {
                        match event.as_str() {
                            "oninput" | "onInput" | "onchange" | "onChange" => {
                                let variant = self.extract_variant_name(&handler.handler);
                                builder = format!("{}.on_change(Msg::{})", builder, variant);
                                // Record event→field mapping for handler generation
                                if let Some(AuraPropValue::Expr(AuraExpr::StateRef(name))) = props.get("value") {
                                    self.input_fields.insert(variant, name.clone());
                                }
                            }
                            _ => {}
                        }
                    }

                    return format!("{}.build()", builder);
                }

                // For leaf tags (text, button) with a "text" prop, use it as the initial value.
                // For buttons: View::button("-") instead of View::button(())
                // For text with state ref: View::text(format!("{}", self.name))
                let text_prop = props.get("text")
                    .and_then(|v| if let AuraPropValue::Expr(AuraExpr::Literal(s)) = v { Some(s.clone()) } else { None });

                // Check if text prop is a state reference (text .name)
                let text_state_ref = props.get("text")
                    .and_then(|v| if let AuraPropValue::Expr(AuraExpr::StateRef(name)) = v { Some(name.clone()) } else { None });

                // Handle image element — generate View::image() or View::image_styled()
                if tag == "image" {
                    let src = props.get("src")
                        .and_then(|v| if let AuraPropValue::Expr(AuraExpr::StateRef(name)) = v {
                            Some(format!("format!(\"{{}}\", self.{})", name))
                        } else if let AuraPropValue::Expr(AuraExpr::Literal(s)) = v {
                            Some(format!("\"{}\"", s))
                        } else {
                            None
                        }).unwrap_or_else(|| "\"\"".to_string());
                    let style_str = props.get("style")
                        .or_else(|| props.get("class"))
                        .and_then(|v| if let AuraPropValue::Expr(AuraExpr::Literal(s)) = v { Some(s.clone()) } else { None })
                        .unwrap_or_default();
                    if style_str.is_empty() {
                        return format!("View::image({})", src);
                    } else {
                        return format!("View::image_styled({}, \"{}\")", src, style_str);
                    }
                }

                // Handle spacer — returns View directly, no builder
                if tag == "spacer" {
                    return "View::spacer()".to_string();
                }

                // Handle divider — returns View directly, no builder
                if tag == "divider" {
                    return "View::divider()".to_string();
                }

                // Handle progress — View::progress_bar(value / max)
                if tag == "progress" {
                    let value_expr = if let Some(AuraPropValue::Expr(AuraExpr::StateRef(name))) = props.get("value") {
                        format!("self.{}", name)
                    } else if let Some(AuraPropValue::Expr(AuraExpr::Literal(s))) = props.get("value") {
                        s.clone()
                    } else {
                        "0".to_string()
                    };
                    let max_val = if let Some(AuraPropValue::Expr(AuraExpr::Literal(s))) = props.get("max") {
                        s.clone()
                    } else {
                        "100".to_string()
                    };
                    let style_str = props.get("style")
                        .or_else(|| props.get("class"))
                        .and_then(|v| if let AuraPropValue::Expr(AuraExpr::Literal(s)) = v { Some(s.clone()) } else { None })
                        .unwrap_or_default();
                    if style_str.is_empty() {
                        return format!("View::progress_bar({} as f32 / {} as f32)", value_expr, max_val);
                    } else {
                        return format!("View::progress_bar_styled({} as f32 / {} as f32, \"{}\")", value_expr, max_val, style_str);
                    }
                }

                let builder_start = if self.is_leaf_tag(tag.as_str()) {
                    if let Some(ref name) = text_state_ref {
                        if tag == "button" {
                            format!("View::button(format!(\"{{}}\", self.{}))", name)
                        } else {
                            format!("View::text(format!(\"{{}}\", self.{}))", name)
                        }
                    } else if let Some(label) = &text_prop {
                        if tag == "button" {
                            format!("View::{}(\"{}\")", view_fn, label)
                        } else if label.contains("${") {
                            format!("View::{}({})", view_fn, self.interpolate_str(label))
                        } else {
                            format!("View::{}(\"{}\")", view_fn, label)
                        }
                    } else {
                        format!("View::{}(())", view_fn)
                    }
                } else {
                    format!("View::{}()", view_fn)
                };

                // Check if any styling props exist (class/style)
                let has_styling = props.keys().any(|k| k == "style" || k == "class");

                // For non-button leaf tags with text content and styling but no children,
                // use View::text_styled() to avoid builder pattern issues
                // (View::text("str") returns View, not ViewBuilder, so chaining won't work)
                if self.is_leaf_tag(tag.as_str()) && tag != "button" && children.is_empty() && has_styling {
                    let style_str = props.get("style")
                        .or_else(|| props.get("class"))
                        .and_then(|v| if let AuraPropValue::Expr(AuraExpr::Literal(s)) = v { Some(s.clone()) } else { None })
                        .unwrap_or_default();

                    if let Some(ref name) = text_state_ref {
                        return format!("View::text_styled(format!(\"{{}}\", self.{}), \"{}\")", name, style_str);
                    }
                    if let Some(label) = &text_prop {
                        return format!("View::text_styled(\"{}\".to_string(), \"{}\")", label, style_str);
                    }
                }

                // Whether the "text" prop was consumed as a constructor arg
                let text_prop_consumed = self.is_leaf_tag(tag.as_str()) && (text_prop.is_some() || text_state_ref.is_some());

                // Non-button leaf tags with text and no styling/children:
                // View::text("str") returns View<M> directly, NOT a builder.
                // Skip .build() to avoid compile error.
                if self.is_leaf_tag(tag.as_str()) && tag != "button" && children.is_empty() && !has_styling {
                    if let Some(ref name) = text_state_ref {
                        return format!("View::text(format!(\"{{}}\", self.{}))", name);
                    }
                    if let Some(label) = &text_prop {
                        if label.contains("${") {
                            return format!("View::text({})", self.interpolate_str(label));
                        }
                        return format!("View::text(\"{}\".to_string())", label);
                    }
                    // Leaf tag without text content but no styling — e.g. avatar
                    // These go through the builder path
                }

                if children.is_empty() {
                    // Single element without children
                    let mut builder = builder_start;

                    // Add props (skip "text" if already used as constructor arg)
                    for (key, value) in props {
                        if text_prop_consumed && key == "text" { continue; }
                        builder = self.add_prop_to_builder(&builder, key, value);
                    }

                    // Add events
                    for (event, handler) in events {
                        builder = self.add_event_to_builder(&builder, event, handler);
                    }

                    // Button without onclick — add no-op handler to prevent panic
                    if tag == "button" && !events.iter().any(|(e, _)| e == "onclick" || e == "onClick") {
                        builder = format!("{}.on_click(|_| ())", builder);
                    }

                    format!("{}.build()", builder)
                } else {
                    // Element with children
                    let mut builder = builder_start;

                    // Add props (skip "text" if already used as constructor arg)
                    for (key, value) in props {
                        if text_prop_consumed && key == "text" { continue; }
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

                    // Button without onclick — add no-op handler to prevent panic
                    if tag == "button" && !events.iter().any(|(e, _)| e == "onclick" || e == "onClick") {
                        builder = format!("{}.on_click(|_| ())", builder);
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

            AuraNode::ForLoop { var, index, iterable, body, .. } => {
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

            AuraNode::Conditional { condition, then_body, else_body, .. } => {
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
                    format!("if {} {{ {} }} else {{ View::empty() }}", rust_condition, then_code.join("\n"))
                }
            }

            AuraNode::Component { name, props, events, .. } => {
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

            AuraNode::Link { to, text, href, children, .. } => {
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
            "center" => "center",

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
                        // Strip quotes and .to_string() suffix from expression
                        let class_str = value_str.trim_matches('"')
                            .trim_end_matches(".to_string()")
                            .trim_matches('"');
                        tailwind_to_methods(builder, class_str)
                    }
                    "style" => {
                        let style_str = value_str.trim_matches('"')
                            .trim_end_matches(".to_string()")
                            .trim_matches('"');
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
            "onchange" | "onChange" | "oninput" | "onInput" => {
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
            LogicPayload::AstStmts(stmts) => {
                let bodies: Vec<String> = stmts.iter()
                    .map(|s| self.ast_stmt_to_rust(s))
                    .collect();
                bodies.join(";\n                ")
            }
            LogicPayload::Bytecode(_) => {
                "// bytecode handler".to_string()
            }
        }
    }

    /// Convert a crate::ast::Stmt to Rust code (for on-handler bodies)
    fn ast_stmt_to_rust(&self, stmt: &crate::ast::Stmt) -> String {
        match stmt {
            crate::ast::Stmt::Store(store) => {
                let name = store.name.as_str();
                let value = self.ast_expr_to_rust(&store.expr);
                format!("self.{} = {}", name, value)
            }
            crate::ast::Stmt::Expr(expr) => {
                self.ast_expr_to_rust(expr)
            }
            _ => format!("/* unhandled stmt */"),
        }
    }

    /// Convert a crate::ast::Expr to Rust code (for on-handler bodies)
    fn ast_expr_to_rust(&self, expr: &crate::ast::Expr) -> String {
        use crate::ast::Expr;
        use auto_val::Op;
        match expr {
            Expr::Str(s) => format!("\"{}\".to_string()", s),
            Expr::I64(n) => n.to_string(),
            Expr::Int(n) => n.to_string(),
            Expr::U64(n) => n.to_string(),
            Expr::Uint(n) => n.to_string(),
            Expr::Float(n, _) => {
                let s = format!("{}", n);
                if s.contains('.') { s } else { format!("{}.0", n) }
            }
            Expr::Double(n, _) => {
                let s = format!("{}", n);
                if s.contains('.') { s } else { format!("{}.0", n) }
            }
            Expr::Bool(b) => b.to_string(),
            Expr::Ident(name) => {
                let s = name.as_str();
                if s.starts_with('.') {
                    format!("self.{}", &s[1..])
                } else {
                    s.to_string()
                }
            }
            Expr::Dot(obj, field) => {
                let obj_str = self.ast_expr_to_rust(obj);
                format!("{}.{}", obj_str, field)
            }
            Expr::Bina(left, op, right) => {
                // Assignment: .count = expr → self.count = expr
                if matches!(op, Op::Asn) {
                    let target = self.ast_expr_to_rust(left);
                    let value = self.ast_expr_to_rust(right);
                    return format!("{} = {}", target, value);
                }
                // Compound assignment: .count += expr → self.count += expr
                if matches!(op, Op::AddEq | Op::SubEq | Op::MulEq | Op::DivEq) {
                    let target = self.ast_expr_to_rust(left);
                    let value = self.ast_expr_to_rust(right);
                    let op_str = match op {
                        Op::AddEq => "+=",
                        Op::SubEq => "-=",
                        Op::MulEq => "*=",
                        Op::DivEq => "/=",
                        _ => unreachable!(),
                    };
                    return format!("{} {} {}", target, op_str, value);
                }
                let left_str = self.ast_expr_to_rust(left);
                let right_str = self.ast_expr_to_rust(right);
                let op_str = match op {
                    Op::Add => "+",
                    Op::Sub => "-",
                    Op::Mul => "*",
                    Op::Div => "/",
                    Op::Mod => "%",
                    Op::Eq => "==",
                    Op::Neq => "!=",
                    Op::Lt => "<",
                    Op::Le => "<=",
                    Op::Gt => ">",
                    Op::Ge => ">=",
                    _ => "?",
                };
                let my_prec = bin_op_precedence(op);
                // Wrap child in parens if its precedence is lower (needs grouping)
                let left_wrapped = if bin_child_needs_parens(left, my_prec) {
                    format!("({})", left_str)
                } else {
                    left_str
                };
                let right_wrapped = if bin_child_needs_parens(right, my_prec) {
                    format!("({})", right_str)
                } else {
                    right_str
                };
                format!("{} {} {}", left_wrapped, op_str, right_wrapped)
            }
            Expr::Call(call) => {
                let fn_name: String = call.get_name_text_safe()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| self.ast_expr_to_rust(&call.name));
                let args: Vec<String> = call.args.args.iter()
                    .map(|a| self.ast_expr_to_rust(&a.get_expr()))
                    .collect();
                match fn_name.as_str() {
                    "print" => {
                        let print_args: Vec<String> = args.iter()
                            .map(|a| a.trim_end_matches(".to_string()").to_string())
                            .collect();
                        format!("println!({})", print_args.join(", "))
                    }
                    _ => format!("{}({})", fn_name, args.join(", ")),
                }
            }
            _ => format!("/* expr */"),
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
            AuraExpr::Literal(s) => format!("\"{}\".to_string()", s),
            AuraExpr::Int(n) => n.to_string(),
            AuraExpr::Float(n) => {
                let s = n.to_string();
                if s.contains('.') { s } else { format!("{}.0", n) }
            }
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
            crate::ast::Type::StrFixed(_) | crate::ast::Type::StrOwned | crate::ast::Type::StrSlice => "String".to_string(),
            crate::ast::Type::Void => "()".to_string(),
            _ => "i32".to_string(), // Default fallback
        }
    }
}

/// Return precedence level for binary operators (higher = tighter binding)
fn bin_op_precedence(op: &auto_val::Op) -> u8 {
    use auto_val::Op;
    match op {
        Op::Mul | Op::Div | Op::Mod => 5,
        Op::Add | Op::Sub => 4,
        Op::Eq | Op::Neq | Op::Lt | Op::Le | Op::Gt | Op::Ge => 3,
        Op::And => 2,
        Op::Or => 1,
        _ => 0,
    }
}

/// Check if a child expression needs parentheses when used inside a parent binary op
fn bin_child_needs_parens(expr: &crate::ast::Expr, parent_prec: u8) -> bool {
    use crate::ast::Expr;
    use auto_val::Op;
    if let Expr::Bina(_, child_op, _) = expr {
        let child_prec = bin_op_precedence(child_op);
        // Only needs parens for assignment-like ops or lower precedence
        !matches!(child_op, Op::Asn | Op::AddEq | Op::SubEq | Op::MulEq | Op::DivEq)
            && child_prec < parent_prec
    } else {
        false
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
    let mut residual_classes: Vec<&str> = Vec::new();

    for class in class_str.split_whitespace() {
        let method = tailwind_single_to_method(class);
        if method.is_empty() {
            residual_classes.push(class);
        } else {
            result.push_str(&method);
        }
    }

    // Pass through unrecognized classes as a .style() call
    if !residual_classes.is_empty() {
        result.push_str(&format!(".style(\"{}\")", residual_classes.join(" ")));
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
            span_map: HashMap::new(),
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
        assert_eq!(gen.auto_type_to_rust(&Type::StrFixed(0)), "String");
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
    fn test_tailwind_to_methods_unknown_classes_passthrough() {
        let result = tailwind_to_methods("View::col()", "p-4 unknown-class gap-2");
        assert_eq!(result, "View::col().p(4).gap(2).style(\"unknown-class\")");
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

    #[test]
    fn test_text_element_with_text_prop() {
        // text "Hello, World!" parsed as Element { tag: "text", props: { text: "Hello, World!" } }
        let node = AuraNode::element("text")
            .with_prop("text", AuraExpr::Literal("Hello, World!".to_string()));

        let mut gen = RustGenerator::new();
        let code = gen.generate_view_tree(&node);
        assert!(code.contains("View::text(\"Hello, World!\")"), "got: {}", code);
        assert!(!code.contains(".build()"), "View::text(str) returns View directly, got: {}", code);
    }
}
