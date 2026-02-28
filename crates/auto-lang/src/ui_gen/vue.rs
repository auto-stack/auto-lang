//! Vue3/JavaScript Code Generator
//!
//! Generates Vue 3 Single File Components (SFC) from AURA widgets.
//!
//! ## Output Format
//!
//! ```vue
//! <script setup>
//! import { ref, computed } from 'vue'
//!
//! // State variables → ref()
//! const count = ref(0)
//!
//! // Event handlers
//! const handleInc = () => {
//!   count.value += 1
//! }
//! </script>
//!
//! <template>
//!   <div class="flex flex-col">
//!     <button @click="handleInc">+</button>
//!     <h2>Count: {{ count }}</h2>
//!   </div>
//! </template>
//!
//! <style scoped>
//! /* Component styles */
//! </style>
//! ```

use super::{BackendGenerator, GenError, GenResult};
use crate::aura::{AuraEvent, AuraExpr, AuraNode, AuraPropValue, AuraStateDef, AuraStmt, AuraTextContent, AuraWidget, LogicPayload};
use std::collections::HashMap;

/// Vue3 SFC generator
pub struct VueGenerator {
    /// Current widget name
    current_widget: Option<String>,

    /// Collected imports
    imports: Vec<String>,

    /// State variable names (for ref() detection)
    state_names: Vec<String>,

    /// Event handler definitions
    handlers: Vec<(String, String)>,

    /// Event names for emit
    emit_events: Vec<String>,

    /// Whether emit is needed
    has_emit: bool,

    /// Component references (other widgets)
    component_refs: Vec<String>,

    /// Tailwind classes for wrapper
    wrapper_classes: String,
}

impl VueGenerator {
    /// Create a new Vue generator
    pub fn new() -> Self {
        Self {
            current_widget: None,
            imports: Vec::new(),
            state_names: Vec::new(),
            handlers: Vec::new(),
            emit_events: Vec::new(),
            has_emit: false,
            component_refs: Vec::new(),
            wrapper_classes: String::new(),
        }
    }

    /// Reset state for new widget
    fn reset(&mut self) {
        self.imports.clear();
        self.state_names.clear();
        self.handlers.clear();
        self.emit_events.clear();
        self.has_emit = false;
        self.component_refs.clear();
        self.wrapper_classes.clear();
    }

    /// Generate complete Vue3 SFC
    pub fn generate_sfc(&mut self, widget: &AuraWidget) -> GenResult<String> {
        self.current_widget = Some(widget.name.clone());
        self.reset();

        let script = self.generate_script(widget)?;
        let template = self.generate_template(&widget.view_tree)?;
        let style = self.generate_style();

        Ok(format!(
            r#"<!-- {} component - Auto-generated from Auto language -->
<script setup>
{}
</script>

<template>
{}
</template>

<style scoped>
{}
</style>
"#,
            widget.name, script, template, style
        ))
    }

    /// Generate <script setup> content
    fn generate_script(&mut self, widget: &AuraWidget) -> GenResult<String> {
        let mut script = String::new();

        // Determine needed imports
        let needs_ref = !widget.state_vars.is_empty();
        let needs_computed = !widget.computed.is_empty();

        // Generate import statement
        let mut imports = Vec::new();
        if needs_ref {
            imports.push("ref");
        }
        if needs_computed {
            imports.push("computed");
        }
        if !imports.is_empty() {
            script.push_str(&format!("import {{ {} }} from 'vue'\n\n", imports.join(", ")));
        }

        // Generate state variables as ref()
        for state in &widget.state_vars {
            self.state_names.push(state.name.clone());
            let init = self.expr_to_js(&state.initial)?;
            script.push_str(&format!("const {} = ref({})\n", state.name, init));
        }

        if !widget.state_vars.is_empty() {
            script.push('\n');
        }

        // Generate computed properties
        for computed_prop in &widget.computed {
            let expr_js = self.expr_to_js(&computed_prop.expr)?;
            script.push_str(&format!(
                "const {} = computed(() => {})\n",
                computed_prop.name, expr_js
            ));
        }

        if !widget.computed.is_empty() {
            script.push('\n');
        }

        // Generate emit if needed
        if self.has_emit {
            script.push_str("const emit = defineEmits<{\n");
            for event in &self.emit_events {
                script.push_str(&format!("  {}: []\n", event));
            }
            script.push_str("}>()\n\n");
        }

        // Generate event handlers
        for (pattern, payload) in &widget.handlers {
            let handler_name = self.pattern_to_handler_name(pattern);
            let body = self.generate_handler_body(payload)?;
            self.handlers.push((handler_name.clone(), body));
        }

        // Output handler functions
        for (handler_name, handler_body) in &self.handlers {
            if handler_body.is_empty() {
                script.push_str(&format!("function {}() {{\n  // TODO\n}}\n\n", handler_name));
            } else {
                script.push_str(&format!("function {}() {{\n  {}\n}}\n\n", handler_name, handler_body));
            }
        }

        Ok(script)
    }

    /// Generate handler function body from LogicPayload
    fn generate_handler_body(&self, payload: &LogicPayload) -> GenResult<String> {
        match payload {
            LogicPayload::AstBlock(stmts) => {
                let body: Vec<String> = stmts.iter()
                    .map(|s| self.stmt_to_js(s))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(body.join("\n  "))
            }
            LogicPayload::Bytecode(_) => {
                Err(GenError::UnsupportedStmt("Bytecode not supported in Vue generator".to_string()))
            }
        }
    }

    /// Generate <template> content from view tree
    fn generate_template(&mut self, root: &AuraNode) -> GenResult<String> {
        let mut template = String::new();

        // Wrapper div with Tailwind classes
        let classes = if self.wrapper_classes.is_empty() {
            "flex flex-col".to_string()
        } else {
            format!("flex flex-col {}", self.wrapper_classes)
        };

        template.push_str(&format!("  <div class=\"{}\">\n", classes));
        template.push_str(&self.node_to_html(root, 2)?);
        template.push_str("  </div>\n");

        Ok(template)
    }

    /// Generate <style scoped> content
    fn generate_style(&self) -> String {
        "/* Component styles */\n".to_string()
    }

    /// Convert AuraNode to HTML string
    fn node_to_html(&mut self, node: &AuraNode, indent: usize) -> GenResult<String> {
        let ind = "  ".repeat(indent);

        match node {
            AuraNode::Element { tag, props, events, children } => {
                let html_tag = self.map_tag(tag, children.is_empty());

                // Build attributes
                let mut attrs = Vec::new();
                let mut text_content: Option<String> = None;

                // Class attribute (both static and dynamic)
                let (static_classes, dynamic_classes) = self.extract_classes(tag, props);
                if !static_classes.is_empty() {
                    attrs.push(format!("class=\"{}\"", static_classes));
                }
                if let Some(dynamic) = dynamic_classes {
                    attrs.push(format!(":class=\"{}\"", dynamic));
                }

                // Check for input type (for special handling)
                let _input_type = props.get("type").and_then(|t| {
                    if let AuraPropValue::Expr(AuraExpr::Literal(s)) = t {
                        Some(s.clone())
                    } else {
                        None
                    }
                });

                // Props as attributes (except 'text' which becomes element content)
                for (key, value) in props {
                    if key == "class" {
                        continue; // Already handled
                    }
                    if key == "text" {
                        // Extract text content to render as element content
                        text_content = Some(self.prop_to_text_content(value)?);
                        continue;
                    }

                    // Handle two-way binding (bind:value -> v-model)
                    if key.starts_with("bind:") {
                        let bind_target = key.strip_prefix("bind:").unwrap();
                        let model_value = match value {
                            AuraPropValue::Expr(AuraExpr::StateRef(name)) => name.clone(),
                            _ => "value".to_string(),
                        };
                        // For checkbox, use v-model for the checked state
                        if tag == "input" && bind_target == "checked" {
                            attrs.push(format!("v-model=\"{}\"", model_value));
                        } else if tag == "input" && bind_target == "value" {
                            attrs.push(format!("v-model=\"{}\"", model_value));
                        } else {
                            attrs.push(format!("v-model=\"{}\"", model_value));
                        }
                        continue;
                    }

                    let value_str = self.prop_to_attr_value(value)?;
                    attrs.push(format!("{}={}", key, value_str));
                }

                // Event handlers
                for (event, aura_event) in events {
                    let vue_event = self.auto_event_to_vue(event);
                    let handler_fn = self.handler_to_function_call_with_params(&aura_event.handler, &aura_event.params);
                    attrs.push(format!("{}=\"{}\"", vue_event, handler_fn));
                }

                let attr_str = if attrs.is_empty() {
                    String::new()
                } else {
                    format!(" {}", attrs.join(" "))
                };

                // Check if we have text content (render as inline content)
                if let Some(text) = &text_content {
                    if children.is_empty() {
                        // <button @click="handler">text</button>
                        Ok(format!("{}<{}{}>{}</{}>\n", ind, html_tag, attr_str, text, html_tag))
                    } else {
                        // Has both text and children - unusual but handle it
                        let mut html = format!("{}<{}{}>{}\n", ind, html_tag, attr_str, text);
                        for child in children {
                            html.push_str(&self.node_to_html(child, indent + 1)?);
                        }
                        html.push_str(&format!("{}</{}>\n", ind, html_tag));
                        Ok(html)
                    }
                } else if children.is_empty() {
                    Ok(format!("{}<{}{} />\n", ind, html_tag, attr_str))
                } else {
                    let mut html = format!("{}<{}{}>\n", ind, html_tag, attr_str);
                    for child in children {
                        html.push_str(&self.node_to_html(child, indent + 1)?);
                    }
                    html.push_str(&format!("{}</{}>\n", ind, html_tag));
                    Ok(html)
                }
            }

            AuraNode::Text(content) => {
                match content {
                    AuraTextContent::Literal(s) => {
                        Ok(format!("{}{}\n", ind, s))
                    }
                    AuraTextContent::Interpolated { template, bindings } => {
                        // Convert template to Vue interpolation
                        let mut vue_text = template.clone();
                        for binding in bindings {
                            // Replace ${.binding} with {{ binding }} (state reference)
                            vue_text = vue_text.replace(
                                &format!("${{{}.{}}}", ".", binding),
                                &format!("{{{{ {} }}}}", binding)
                            );
                            // Replace ${binding} with {{ binding }} (variable reference)
                            vue_text = vue_text.replace(
                                &format!("${{{}}}", binding),
                                &format!("{{{{ {} }}}}", binding)
                            );
                            // Also handle $binding format (without braces)
                            vue_text = vue_text.replace(
                                &format!("${}", binding),
                                &format!("{{{{ {} }}}}", binding)
                            );
                        }
                        Ok(format!("{}{}\n", ind, vue_text))
                    }
                }
            }

            AuraNode::ForLoop { var, index, iterable, body } => {
                // Generate v-for directive
                // Auto syntax: for idx, item in list (index first, value second)
                // Vue syntax: v-for="(item, index) in list" (value first, index second)
                // So we need to swap the order for Vue
                let v_for = if let Some(idx) = index {
                    format!("v-for=\"({}, {}) in {}\"", var, idx, iterable.trim_start_matches('.'))
                } else {
                    format!("v-for=\"{} in {}\"", var, iterable.trim_start_matches('.'))
                };

                // Wrap body in a container with v-for
                let mut body_html = String::new();
                for child in body {
                    body_html.push_str(&self.node_to_html(child, indent + 1)?);
                }

                // Use template tag for the loop wrapper
                Ok(format!("{}<template {}>\n{}{}</template>\n", ind, v_for, body_html, ind))
            }

            AuraNode::Conditional { condition, then_body, else_body } => {
                // Convert condition to Vue expression
                let vue_condition = self.convert_condition(condition);

                let mut then_html = String::new();
                for child in then_body {
                    then_html.push_str(&self.node_to_html(child, indent + 1)?);
                }

                if let Some(else_nodes) = else_body {
                    let mut else_html = String::new();
                    for child in else_nodes {
                        else_html.push_str(&self.node_to_html(child, indent + 1)?);
                    }
                    Ok(format!(
                        "{}<template v-if=\"{}\">\n{}{}</template>\n{}<template v-else>\n{}{}</template>\n",
                        ind, vue_condition, then_html, ind, ind, else_html, ind
                    ))
                } else {
                    Ok(format!("{}<template v-if=\"{}\">\n{}{}</template>\n", ind, vue_condition, then_html, ind))
                }
            }

            AuraNode::Component { name, props, events } => {
                // Build props as bindings
                let mut attrs = Vec::new();
                for (key, value) in props {
                    let value_str = self.prop_to_attr_value(&AuraPropValue::Expr(value.clone()))?;
                    attrs.push(format!(":{}={}", key, value_str));
                }

                // Event handlers
                for (event, aura_event) in events {
                    let vue_event = self.auto_event_to_vue(event);
                    let handler_fn = self.handler_to_function_call_with_params(&aura_event.handler, &aura_event.params);
                    attrs.push(format!("{}=\"{}\"", vue_event, handler_fn));
                }

                let attr_str = if attrs.is_empty() {
                    String::new()
                } else {
                    format!(" {}", attrs.join(" "))
                };

                self.component_refs.push(name.clone());
                Ok(format!("{}<{}{} />\n", ind, name, attr_str))
            }
        }
    }

    /// Convert AURA condition to Vue expression
    fn convert_condition(&mut self, condition: &str) -> String {
        // Convert .var to var, .len to .length, etc.
        let mut result = condition.trim().to_string();

        // Replace .len with .length
        result = result.replace(".len", ".length");

        // Remove leading dot from state references (.count -> count)
        // Pattern: .identifier (at word boundary)
        let mut converted = String::new();
        let chars: Vec<char> = result.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            if chars[i] == '.' && (i == 0 || !chars[i-1].is_alphanumeric()) {
                // Check if this is a number (like 0.5)
                if i + 1 < chars.len() && chars[i+1].is_ascii_digit() {
                    converted.push('.');
                    i += 1;
                    continue;
                }
                // Skip the dot (remove state prefix)
                i += 1;
                continue;
            }
            converted.push(chars[i]);
            i += 1;
        }

        converted
    }

    /// Map AutoUI tag to HTML tag
    fn map_tag(&mut self, tag: &str, self_closing: bool) -> String {
        match tag {
            // Layout
            "col" | "column" => "div".to_string(),
            "row" => "div".to_string(),
            "grid" => "div".to_string(),
            "scroll" => "div".to_string(),
            "container" => "div".to_string(),
            "center" => "div".to_string(),

            // Content
            "button" => "button".to_string(),
            "input" => "input".to_string(),
            "textarea" => "textarea".to_string(),
            "checkbox" => "input".to_string(),
            "toggle" => "button".to_string(),
            "select" => "select".to_string(),
            "option" => "option".to_string(),
            "link" => "a".to_string(),

            // Typography
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => tag.to_string(),
            "text" | "label" | "span" => "span".to_string(),
            "p" => "p".to_string(),

            // Data
            "table" => "table".to_string(),
            "thead" => "thead".to_string(),
            "tbody" => "tbody".to_string(),
            "tr" => "tr".to_string(),
            "th" => "th".to_string(),
            "td" => "td".to_string(),
            "tree" => "ul".to_string(),
            "tree_item" => "li".to_string(),

            // Navigation
            "tabs" => "div".to_string(),
            "tab" => "button".to_string(),

            // Overlay
            "modal" => "div".to_string(),
            "tooltip" => "span".to_string(),

            // Form
            "slider" => "input".to_string(),
            "radio" => "input".to_string(),
            "radiogroup" => "div".to_string(),

            // Feedback
            "progress" => "progress".to_string(),
            "badge" => "span".to_string(),
            "spinner" => "div".to_string(),

            // Display
            "card" => "div".to_string(),
            "avatar" => "img".to_string(),

            // Media
            "image" => "img".to_string(),
            "icon" => "span".to_string(),

            // Utility
            "divider" => "hr".to_string(),
            "spacer" => "div".to_string(),

            // Special
            "div" => "div".to_string(),
            "+" => if self_closing { "span".to_string() } else { "span".to_string() },
            "-" => if self_closing { "span".to_string() } else { "span".to_string() },

            _ => {
                // Check if it's a PascalCase component name
                if tag.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                    self.component_refs.push(tag.to_string());
                    tag.to_string()
                } else {
                    "div".to_string()
                }
            }
        }
    }

    /// Extract Tailwind classes from tag and props
    /// Returns (static_classes, dynamic_class_binding)
    fn extract_classes(&self, tag: &str, props: &HashMap<String, AuraPropValue>) -> (String, Option<String>) {
        let mut classes = Vec::new();
        let mut dynamic_binding: Option<String> = None;

        // Default classes based on tag
        match tag {
            // Layout
            "col" | "column" => classes.push("flex flex-col".to_string()),
            "row" => classes.push("flex flex-row".to_string()),
            "grid" => classes.push("grid".to_string()),
            "scroll" => classes.push("overflow-auto".to_string()),
            "container" => classes.push("max-w-7xl mx-auto".to_string()),
            "center" => classes.push("flex items-center justify-center".to_string()),

            // Content
            "button" => classes.push("px-4 py-2 rounded".to_string()),
            "input" => classes.push("border rounded px-2 py-1".to_string()),
            "textarea" => classes.push("border rounded px-2 py-1".to_string()),
            "checkbox" => classes.push("w-4 h-4".to_string()),
            "toggle" => classes.push("relative w-10 h-6 rounded-full".to_string()),
            "select" => classes.push("border rounded px-2 py-1".to_string()),
            "link" => classes.push("text-blue-600 underline".to_string()),

            // Data
            "table" => classes.push("min-w-full border".to_string()),
            "thead" => classes.push("bg-gray-100".to_string()),
            "th" => classes.push("px-4 py-2 text-left font-semibold".to_string()),
            "td" => classes.push("px-4 py-2".to_string()),
            "tree" => classes.push("list-none pl-4".to_string()),
            "tree_item" => classes.push("py-1".to_string()),

            // Navigation
            "tabs" => classes.push("flex border-b".to_string()),
            "tab" => classes.push("px-4 py-2 border-b-2 border-transparent".to_string()),

            // Overlay
            "modal" => classes.push("fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center".to_string()),

            // Form
            "slider" => classes.push("w-full".to_string()),
            "radiogroup" => classes.push("flex flex-col gap-2".to_string()),

            // Feedback
            "progress" => classes.push("w-full h-2 rounded".to_string()),
            "badge" => classes.push("px-2 py-1 text-xs rounded-full".to_string()),
            "spinner" => classes.push("animate-spin w-6 h-6 border-2 border-gray-300 border-t-blue-600 rounded-full".to_string()),

            // Display
            "card" => classes.push("bg-white rounded-lg shadow p-4".to_string()),
            "avatar" => classes.push("w-10 h-10 rounded-full".to_string()),

            // Media
            "image" => classes.push("max-w-full".to_string()),
            "icon" => classes.push("w-5 h-5".to_string()),

            // Utility
            "divider" => classes.push("border-t border-gray-300".to_string()),
            "spacer" => classes.push("flex-1".to_string()),

            _ => {}
        }

        // Class prop
        if let Some(value) = props.get("class") {
            match value {
                AuraPropValue::Expr(AuraExpr::Literal(s)) => {
                    classes.push(s.clone());
                }
                AuraPropValue::ClassBinding(bindings) => {
                    // Generate dynamic class binding: { completed: todo.done, editing: todo.editing }
                    let binding_strs: Vec<String> = bindings.iter()
                        .map(|b| {
                            let cond = self.expr_to_js(&b.condition).unwrap_or_else(|_| "false".to_string());
                            format!("{}: {}", b.class_name, cond)
                        })
                        .collect();
                    dynamic_binding = Some(format!("{{ {} }}", binding_strs.join(", ")));
                }
                _ => {}
            }
        }

        (classes.join(" "), dynamic_binding)
    }

    /// Convert AuraExpr to JS value string
    fn expr_to_js(&self, expr: &AuraExpr) -> GenResult<String> {
        match expr {
            AuraExpr::Literal(s) => Ok(format!("'{}'", s)),
            AuraExpr::Int(n) => Ok(n.to_string()),
            AuraExpr::Float(n) => Ok(n.to_string()),
            AuraExpr::Bool(b) => Ok(b.to_string()),
            AuraExpr::StateRef(name) => {
                if self.state_names.contains(&name.to_string()) {
                    Ok(format!("{}.value", name))
                } else {
                    Ok(name.clone())
                }
            }
            AuraExpr::Binary { left, op, right } => {
                let left_js = self.expr_to_js(left)?;
                let right_js = self.expr_to_js(right)?;
                let op_js = self.bin_op_to_js(op);
                Ok(format!("{} {} {}", left_js, op_js, right_js))
            }
            AuraExpr::Unary { op, operand } => {
                let operand_js = self.expr_to_js(operand)?;
                let op_js = match op {
                    crate::aura::AuraUnaryOp::Neg => "-",
                    crate::aura::AuraUnaryOp::Not => "!",
                };
                Ok(format!("{}{}", op_js, operand_js))
            }
            AuraExpr::MsgVariant { msg_type, variant } => {
                Ok(format!("{}.{}", msg_type, variant))
            }
            AuraExpr::MethodCall { object, method, args } => {
                let object_js = self.expr_to_js(object)?;
                let args_js: Vec<String> = args.iter()
                    .map(|a| self.expr_to_js(a))
                    .collect::<Result<Vec<_>, _>>()?;
                // Convert .len to .length for JavaScript
                let method_js = if method == "len" { "length" } else { method.as_str() };
                Ok(format!("{}.{}({})", object_js, method_js, args_js.join(", ")))
            }
            AuraExpr::Array(elems) => {
                let elems_js: Vec<String> = elems.iter()
                    .map(|e| self.expr_to_js(e))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(format!("[{}]", elems_js.join(", ")))
            }
            AuraExpr::Lambda { params, body } => {
                let body_js = self.expr_to_js(body)?;
                Ok(format!("({}) => {}", params.join(", "), body_js))
            }
            AuraExpr::FieldAccess { object, field } => {
                let object_js = self.expr_to_js(object)?;
                Ok(format!("{}.{}", object_js, field))
            }
        }
    }

    /// Convert AuraStmt to JS statement
    fn stmt_to_js(&self, stmt: &AuraStmt) -> GenResult<String> {
        match stmt {
            AuraStmt::Assign { target, value } => {
                let value_js = self.expr_to_js(value)?;
                // Check if target is a ref
                if self.state_names.contains(target) {
                    Ok(format!("{}.value = {}", target, value_js))
                } else {
                    Ok(format!("{} = {}", target, value_js))
                }
            }
            AuraStmt::Update { target, op, value } => {
                let value_js = self.expr_to_js(value)?;
                let op_js = match op {
                    crate::aura::AuraUpdateOp::AddAssign => "+=",
                    crate::aura::AuraUpdateOp::SubAssign => "-=",
                    crate::aura::AuraUpdateOp::MulAssign => "*=",
                    crate::aura::AuraUpdateOp::DivAssign => "/=",
                };
                if self.state_names.contains(target) {
                    Ok(format!("{}.value {} {}", target, op_js, value_js))
                } else {
                    Ok(format!("{} {} {}", target, op_js, value_js))
                }
            }
            AuraStmt::MethodCall { object, method, args } => {
                let args_js: Vec<String> = args.iter()
                    .map(|a| self.expr_to_js(a))
                    .collect::<Result<Vec<_>, _>>()?;
                if self.state_names.contains(object) {
                    Ok(format!("{}.value.{}({})", object, method, args_js.join(", ")))
                } else {
                    Ok(format!("{}.{}({})", object, method, args_js.join(", ")))
                }
            }
        }
    }

    /// Convert binary operator to JS
    fn bin_op_to_js(&self, op: &crate::aura::AuraBinOp) -> &'static str {
        match op {
            crate::aura::AuraBinOp::Add => "+",
            crate::aura::AuraBinOp::Sub => "-",
            crate::aura::AuraBinOp::Mul => "*",
            crate::aura::AuraBinOp::Div => "/",
            crate::aura::AuraBinOp::Mod => "%",
            crate::aura::AuraBinOp::Eq => "===",
            crate::aura::AuraBinOp::Ne => "!==",
            crate::aura::AuraBinOp::Lt => "<",
            crate::aura::AuraBinOp::Le => "<=",
            crate::aura::AuraBinOp::Gt => ">",
            crate::aura::AuraBinOp::Ge => ">=",
            crate::aura::AuraBinOp::And => "&&",
            crate::aura::AuraBinOp::Or => "||",
        }
    }

    /// Convert prop value to HTML attribute value
    fn prop_to_attr_value(&self, value: &AuraPropValue) -> GenResult<String> {
        match value {
            AuraPropValue::Expr(expr) => {
                match expr {
                    AuraExpr::Literal(s) => Ok(format!("\"{}\"", s)),
                    AuraExpr::Int(n) => Ok(format!("\"{}\"", n)),
                    AuraExpr::Bool(b) => Ok(format!("\"{}\"", b)),
                    AuraExpr::StateRef(name) => Ok(format!("\"{{{{ {} }}}}\"", name)),
                    _ => Ok(format!("\"{{{{ {} }}}}\"", "value")),
                }
            }
            AuraPropValue::ClassBinding(_) => {
                // Class bindings are handled separately in extract_classes
                Ok("\"\"".to_string())
            }
        }
    }

    /// Convert prop value to text content (for rendering inside element)
    fn prop_to_text_content(&self, value: &AuraPropValue) -> GenResult<String> {
        match value {
            AuraPropValue::Expr(expr) => {
                match expr {
                    AuraExpr::Literal(s) => Ok(s.clone()),
                    AuraExpr::Int(n) => Ok(n.to_string()),
                    AuraExpr::Bool(b) => Ok(b.to_string()),
                    AuraExpr::StateRef(name) => Ok(format!("{{{{ {} }}}}", name)),
                    _ => Ok("value".to_string()),
                }
            }
            AuraPropValue::ClassBinding(_) => {
                Ok("".to_string())
            }
        }
    }

    /// Convert AutoUI event name to Vue event
    fn auto_event_to_vue(&self, event: &str) -> String {
        match event {
            "onclick" | "onClick" | "on_click" => "@click".to_string(),
            "oninput" | "onInput" => "@input".to_string(),
            "onchange" | "onChange" => "@change".to_string(),
            _ => format!("@{}", event.trim_start_matches("on")),
        }
    }

    /// Convert handler pattern to function name
    fn pattern_to_handler_name(&self, pattern: &str) -> String {
        // Check for dot prefix first (e.g., ".Inc")
        if pattern.starts_with('.') {
            format!("on{}", &pattern[1..])
        } else if let Some(variant) = pattern.split("::").last() {
            // Pattern like "Msg::Inc" -> "onInc"
            format!("on{}", variant)
        } else {
            format!("on{}", pattern)
        }
    }

    /// Convert handler reference to function call
    fn handler_to_function_call(&self, handler: &str) -> String {
        // Check for dot prefix first
        if handler.starts_with('.') {
            format!("on{}", &handler[1..])
        } else if let Some(variant) = handler.split("::").last() {
            // Handler like "Msg::Inc" -> "onInc"
            format!("on{}", variant)
        } else {
            format!("on{}", handler)
        }
    }

    /// Convert handler to Vue function call with parameters
    fn handler_to_function_call_with_params(&self, handler: &str, params: &[String]) -> String {
        let func_name = self.handler_to_function_call(handler);
        if params.is_empty() {
            func_name
        } else {
            format!("{}({})", func_name, params.join(", "))
        }
    }
}

impl BackendGenerator for VueGenerator {
    fn generate(&mut self, widget: &AuraWidget) -> GenResult<String> {
        self.generate_sfc(widget)
    }

    fn extension(&self) -> &'static str {
        "vue"
    }
}

impl Default for VueGenerator {
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
    use crate::aura::{AuraMessage, AuraMsgVariant};
    use std::collections::HashMap;

    #[test]
    fn test_vue_generator_creation() {
        let gen = VueGenerator::new();
        assert!(gen.current_widget.is_none());
    }

    #[test]
    fn test_simple_counter() {
        let widget = AuraWidget {
            name: "Counter".to_string(),
            state_vars: vec![AuraStateDef {
                name: "count".to_string(),
                type_info: crate::ast::Type::Int,
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
            computed: vec![],
        };

        let mut gen = VueGenerator::new();
        let sfc = gen.generate(&widget).unwrap();

        assert!(sfc.contains("<script setup>"));
        assert!(sfc.contains("import { ref } from 'vue'"));
        assert!(sfc.contains("const count = ref(0)"));
        assert!(sfc.contains("<template>"));
        assert!(sfc.contains("<style scoped>"));
    }

    #[test]
    fn test_expr_to_js() {
        let gen = VueGenerator::new();

        assert_eq!(gen.expr_to_js(&AuraExpr::Int(42)).unwrap(), "42");
        assert_eq!(gen.expr_to_js(&AuraExpr::Bool(true)).unwrap(), "true");
        assert_eq!(gen.expr_to_js(&AuraExpr::Literal("hello".to_string())).unwrap(), "'hello'");
    }

    #[test]
    fn test_map_tag() {
        let mut gen = VueGenerator::new();

        assert_eq!(gen.map_tag("col", true), "div");
        assert_eq!(gen.map_tag("button", false), "button");
        assert_eq!(gen.map_tag("h2", false), "h2");
    }

    #[test]
    fn test_pattern_to_handler_name() {
        let gen = VueGenerator::new();

        assert_eq!(gen.pattern_to_handler_name("Msg::Inc"), "onInc");
        assert_eq!(gen.pattern_to_handler_name(".Inc"), "onInc");
        assert_eq!(gen.pattern_to_handler_name("Dec"), "onDec");
    }
}
