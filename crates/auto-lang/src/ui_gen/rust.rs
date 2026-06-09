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

    /// Child component names referenced in the current widget's view tree
    child_components: Vec<String>,

    /// Loop variables in scope (for generating correct references)
    loop_vars: Vec<String>,

    /// Maps input event variant name to field name for input text parsing
    input_fields: std::collections::HashMap<String, String>,

    /// State var types for lookup during handler generation
    state_types: std::collections::HashMap<String, String>,

    /// Prop names for lookup during handler generation (to add self. prefix)
    prop_names: std::collections::HashSet<String>,

    /// Prop types for checking if a prop needs Value index access
    prop_types: std::collections::HashMap<String, String>,

    /// Loop variables that iterate over Value-type collections (need ["field"] access)
    value_loop_vars: std::collections::HashSet<String>,

    /// Local variables in handler bodies that hold serde_json::Value results
    /// (from API function calls like `let note = create_note(...)`)
    value_locals: std::collections::HashSet<String>,
}

impl RustGenerator {
    /// Create a new Rust generator
    pub fn new() -> Self {
        Self {
            current_widget: None,
            message_variants: Vec::new(),
            needs_imports: true,
            indent: 0,
            child_components: Vec::new(),
            loop_vars: Vec::new(),
            input_fields: std::collections::HashMap::new(),
            state_types: std::collections::HashMap::new(),
            prop_names: std::collections::HashSet::new(),
            prop_types: std::collections::HashMap::new(),
            value_loop_vars: std::collections::HashSet::new(),
            value_locals: std::collections::HashSet::new(),
        }
    }

    /// Reset state for new widget
    fn reset(&mut self) {
        self.message_variants.clear();
        self.input_fields.clear();
        self.state_types.clear();
        self.prop_names.clear();
        self.prop_types.clear();
        self.value_loop_vars.clear();
        self.value_locals.clear();
        self.child_components.clear();
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

    /// Get the widget-specific Msg enum name (e.g., "AppMsg", "EditorPanelMsg")
    fn current_msg_name(&self) -> String {
        match &self.current_widget {
            Some(name) => format!("{}Msg", name),
            None => "Msg".to_string(),
        }
    }

    /// Get the Rust type for a state var, using refined type from initial expression
    fn state_rust_type(&self, state: &crate::aura::AuraStateDef) -> String {
        self.state_types.get(&state.name)
            .cloned()
            .unwrap_or_else(|| self.auto_type_to_rust(&state.type_info))
    }

    /// Get the Rust type for a prop
    fn prop_rust_type(&self, prop: &crate::aura::AuraProp) -> String {
        self.auto_type_to_rust(&prop.type_info)
    }

    /// Check if any handler body accesses prop_name.field (dot access on a prop)
    fn prop_needs_value_type(&self, widget: &AuraWidget, prop_name: &str) -> bool {
        for (_pattern, payload) in &widget.handlers {
            let body_str = self.generate_handler_body(payload);
            // Look for self.{prop_name}.field patterns
            if body_str.contains(&format!("self.{}.", prop_name)) {
                return true;
            }
        }
        false
    }

    /// Check if the view tree contains field access on a prop (e.g., note.title)
    /// indicating the prop needs to be serde_json::Value, not String
    fn view_accesses_prop_field(&self, node: &AuraNode, prop_name: &str) -> bool {
        match node {
            AuraNode::Element { props, children, .. } => {
                // Check if any prop value is a FieldAccess on our prop
                for (_key, value) in props {
                    if let crate::aura::AuraPropValue::Expr(expr) = value {
                        if self.expr_accesses_field(expr, prop_name) {
                            return true;
                        }
                    }
                }
                for child in children {
                    if self.view_accesses_prop_field(child, prop_name) {
                        return true;
                    }
                }
            }
            AuraNode::ForLoop { body, .. } => {
                for child in body {
                    if self.view_accesses_prop_field(child, prop_name) {
                        return true;
                    }
                }
            }
            AuraNode::Conditional { then_body, else_body, .. } => {
                for child in then_body {
                    if self.view_accesses_prop_field(child, prop_name) {
                        return true;
                    }
                }
                if let Some(else_nodes) = else_body {
                    for child in else_nodes {
                        if self.view_accesses_prop_field(child, prop_name) {
                            return true;
                        }
                    }
                }
            }
            _ => {}
        }
        false
    }

    /// Check if an expression accesses a field on the given prop name
    fn expr_accesses_field(&self, expr: &AuraExpr, prop_name: &str) -> bool {
        match expr {
            AuraExpr::FieldAccess { object, field: _ } => {
                if let AuraExpr::StateRef(name) = object.as_ref() {
                    if name == prop_name {
                        return true;
                    }
                }
                self.expr_accesses_field(object, prop_name)
            }
            AuraExpr::Binary { left, right, .. } => {
                self.expr_accesses_field(left, prop_name) || self.expr_accesses_field(right, prop_name)
            }
            AuraExpr::MethodCall { object, args, .. } => {
                self.expr_accesses_field(object, prop_name)
                    || args.iter().any(|a| self.expr_accesses_field(a, prop_name))
            }
            AuraExpr::Index { target, index } => {
                self.expr_accesses_field(target, prop_name)
                    || self.expr_accesses_field(index, prop_name)
            }
            _ => false,
        }
    }

    /// Check if a name is a loop variable
    fn is_loop_var(&self, name: &str) -> bool {
        self.loop_vars.contains(&name.to_string())
    }

    /// Check if a variable has serde_json::Value type (exact match, not Vec<Value>)
    fn is_value_type_var(&self, name: &str) -> bool {
        // Check state vars
        if let Some(ty) = self.state_types.get(name) {
            return ty == "serde_json::Value";
        }
        // Check props
        if let Some(ty) = self.prop_types.get(name) {
            return ty == "serde_json::Value";
        }
        false
    }

    /// Check if a dot access target needs index syntax (target["field"] instead of target.field)
    fn needs_index_access(&self, target_name: &str) -> bool {
        // Props that are actually serde_json::Value type
        if let Some(ty) = self.prop_types.get(target_name) {
            return ty == "serde_json::Value";
        }
        // State vars that are serde_json::Value (not Vec<Value>)
        if let Some(ty) = self.state_types.get(target_name) {
            return ty == "serde_json::Value";
        }
        // Loop variables iterating over Value-type collections
        if self.value_loop_vars.contains(target_name) {
            return true;
        }
        // Local variables from function call results (likely serde_json::Value)
        if self.value_locals.contains(target_name) {
            return true;
        }
        false
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
            let ty = if matches!(state.type_info, crate::ast::Type::Unknown) {
                // Infer type from initial expression for untyped state vars
                match &state.initial {
                    AuraExpr::Array(_) => "Vec<serde_json::Value>".to_string(),
                    AuraExpr::Object(_) => "serde_json::Value".to_string(),
                    AuraExpr::Literal(_) => "String".to_string(),
                    AuraExpr::Int(_) => "i32".to_string(),
                    AuraExpr::Float(_) => "f64".to_string(),
                    AuraExpr::Bool(_) => "bool".to_string(),
                    _ => self.auto_type_to_rust(&state.type_info),
                }
            } else {
                self.auto_type_to_rust(&state.type_info)
            };
            self.state_types.insert(state.name.clone(), ty);
        }

        // Populate prop_names and prop_types for self. prefix resolution and type checking
        for prop in &widget.props {
            self.prop_names.insert(prop.name.clone());
            let mut prop_ty = self.prop_rust_type(prop);
            // Apply the same Value upgrade logic as generate_struct
            if self.prop_needs_value_type(widget, &prop.name) && prop_ty == "String" {
                prop_ty = "serde_json::Value".to_string();
            }
            // Also check if the view tree accesses fields on this prop (e.g., note.title)
            // which means it needs to be serde_json::Value, not String
            if prop_ty == "String" && self.view_accesses_prop_field(&widget.view_tree, &prop.name) {
                prop_ty = "serde_json::Value".to_string();
            }
            self.prop_types.insert(prop.name.clone(), prop_ty);
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

        // Pre-scan view tree for child component references (needed for wrapper msg variants)
        self.scan_child_components(&widget.view_tree);

        // Message enum (includes wrapper variants for child components)
        if !self.message_variants.is_empty() || !self.child_components.is_empty() {
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

        // Pre-scan handlers to find local variables from function calls (likely Value type)
        self.scan_handler_locals(widget);

        // Component impl
        code.push_str(&self.generate_component_impl(widget));

        // Computed properties impl (if any)
        if !widget.computed.is_empty() {
            code.push('\n');
            code.push_str(&self.generate_computed_impl(widget));
        }

        // NOTE: API function stubs are generated at the file level in rust_ui.rs,
        // not per-widget, to avoid duplicate definitions.

        Ok(code)
    }

    /// Generate Msg enum definition
    fn generate_msg_enum(&self) -> GenResult<String> {
        let mut code = String::new();
        let msg_name = self.current_msg_name();

        code.push_str("#[derive(Clone, Debug, PartialEq)]\n");
        code.push_str(&format!("pub enum {} {{\n", msg_name));

        for variant in &self.message_variants {
            if let Some(ref ty) = variant.payload {
                let ty_str = self.auto_type_to_rust(ty);
                code.push_str(&format!("    {}({}),\n", variant.name, ty_str));
            } else {
                code.push_str(&format!("    {},\n", variant.name));
            }
        }

        // Add wrapper variants for child components (e.g., EditorPanel(EditorPanelMsg))
        for child_name in &self.child_components {
            let child_msg = format!("{}Msg", child_name);
            code.push_str(&format!("    {}({}),\n", child_name, child_msg));
        }

        code.push_str("}\n");

        Ok(code)
    }

    /// Generate stub functions for API imports.
    /// These are placeholder implementations that return dummy values so the code compiles.
    /// In production, these would be replaced with actual HTTP client calls.
    fn generate_api_stubs(&self, api_imports: &[String]) -> String {
        let mut code = String::new();
        code.push_str("// API function stubs (TODO: replace with real HTTP client calls)\n");
        for fn_name in api_imports {
            let lower = fn_name.to_lowercase();
            if lower.starts_with("list_") || lower.starts_with("list") {
                // Returns Vec<serde_json::Value>
                code.push_str(&format!(
                    "fn {}() -> Vec<serde_json::Value> {{\n    // TODO: HTTP GET request\n    vec![]\n}}\n\n",
                    fn_name
                ));
            } else if lower.starts_with("create_") {
                // Returns serde_json::Value
                code.push_str(&format!(
                    "fn {}(_title: String, _body: String) -> serde_json::Value {{\n    // TODO: HTTP POST request\n    serde_json::json!({{\"id\": 0, \"title\": _title, \"body\": _body, \"time\": \"now\"}})\n}}\n\n",
                    fn_name
                ));
            } else if lower.starts_with("update_") {
                // Returns nothing meaningful
                code.push_str(&format!(
                    "fn {}(_id: i32, _title: String, _body: String) {{\n    // TODO: HTTP PUT request\n}}\n\n",
                    fn_name
                ));
            } else if lower.starts_with("delete_") {
                // Returns nothing meaningful
                code.push_str(&format!(
                    "fn {}(_id: i32) {{\n    // TODO: HTTP DELETE request\n}}\n\n",
                    fn_name
                ));
            } else if lower.starts_with("get_") {
                // Returns Option<serde_json::Value>
                code.push_str(&format!(
                    "fn {}(_id: i32) -> Option<serde_json::Value> {{\n    // TODO: HTTP GET request\n    None\n}}\n\n",
                    fn_name
                ));
            } else {
                // Generic stub
                code.push_str(&format!(
                    "fn {}() {{\n    // TODO: implement API call\n    todo!(\"{}\");\n}}\n\n",
                    fn_name, fn_name
                ));
            }
        }
        code
    }
    fn generate_struct(&self, widget: &AuraWidget) -> String {
        let mut code = String::new();

        code.push_str("#[derive(Debug)]\n");
        code.push_str(&format!("pub struct {} {{\n", widget.name));

        // Props (from widget signature, e.g., EditorPanel's `note` parameter)
        for prop in &widget.props {
            // Use pre-computed prop type (which includes Value upgrade from initial pass)
            let field_type = self.prop_types.get(&prop.name)
                .cloned()
                .unwrap_or_else(|| self.prop_rust_type(prop));
            code.push_str(&format!("    pub {}: {},\n", prop.name, field_type));
        }

        // State variables (use refined types from state_types)
        for state in &widget.state_vars {
            let field_name = &state.name;
            let field_type = self.state_rust_type(state);
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

        // new() constructor — accepts props as parameters
        let has_props = !widget.props.is_empty();
        if has_props {
            let params: Vec<String> = widget.props.iter()
                .map(|p| {
                    let ty = self.prop_types.get(&p.name)
                        .cloned()
                        .unwrap_or_else(|| self.prop_rust_type(p));
                    format!("{}: {}", p.name, ty)
                })
                .collect();
            code.push_str(&format!("    pub fn new({}) -> Self {{\n", params.join(", ")));
        } else {
            code.push_str("    pub fn new() -> Self {\n");
        }
        code.push_str("        Self {\n");

        // Initialize props from parameters
        for prop in &widget.props {
            code.push_str(&format!("            {}: {},\n", prop.name, prop.name));
        }

        // Initialize state vars from their defaults
        for state in &widget.state_vars {
            let init = self.expr_to_rust(&state.initial);
            code.push_str(&format!("            {}: {},\n", state.name, init));
        }

        code.push_str("        }\n");
        code.push_str("    }\n");
        code.push_str("}\n");

        // Default impl — only for widgets without props (props require arguments)
        if !has_props {
            code.push_str(&format!(
                "impl Default for {} {{\n    fn default() -> Self {{ Self::new() }}\n}}\n",
                widget_name
            ));
        }

        code
    }

    /// Generate Component trait implementation
    fn generate_component_impl(&mut self, widget: &AuraWidget) -> String {
        let widget_name = &widget.name;
        let mut code = String::new();

        code.push_str(&format!("impl Component for {} {{\n", widget_name));

        // Message type
        let msg_type = if !self.message_variants.is_empty() {
            self.current_msg_name()
        } else {
            "()".to_string()
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
        let msg_name = self.current_msg_name();

        code.push_str("    fn on(&mut self, msg: Self::Msg) {\n");

        if !self.message_variants.is_empty() {
            code.push_str("        match msg {\n");

            // Generate match arms from handlers
            for (pattern, payload) in &widget.handlers {
                let variant_name = self.extract_variant_name(pattern);
                let body = self.generate_handler_body(payload);
                // Check if variant has payload — if so, bind it to a variable
                let has_payload = self.message_variants.iter()
                    .find(|v| v.name == variant_name)
                    .map_or(false, |v| v.payload.is_some());
                if has_payload {
                    // Use a named binding instead of _ so handler body can reference it
                    code.push_str(&format!("            {}::{}(id) => {{\n", msg_name, variant_name));
                } else {
                    code.push_str(&format!("            {}::{} => {{\n", msg_name, variant_name));
                }

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

            // Add handler forwarding for child component message wrappers.
            // Strategy: create a temp child instance, sync parent state fields that match
            // child field names, call child.on(inner), then sync back.
            for child_name in &self.child_components {
                let child_msg = format!("{}Msg", child_name);
                // Find parent state vars that likely correspond to child fields
                // (same name in parent state as in child component)
                let sync_fields = self.find_sync_fields_for_child(widget);
                let constructor_args = self.find_constructor_args_for_child(widget);

                code.push_str(&format!(
                    "            {}::{}(inner) => {{\n",
                    msg_name, child_name
                ));

                // Create temporary child instance with constructor args
                code.push_str(&format!(
                    "                let mut __child = {}::new({});\n",
                    child_name, constructor_args
                ));

                // Sync parent state vars to child fields (by name matching)
                for field in &sync_fields {
                    code.push_str(&format!(
                        "                __child.{} = self.{}.clone();\n",
                        field, field
                    ));
                }

                // Apply the message
                code.push_str("                __child.on(inner);\n");

                // Sync child fields back to parent state
                for field in &sync_fields {
                    code.push_str(&format!(
                        "                self.{} = __child.{};\n",
                        field, field
                    ));
                }

                // Sync the note data back if the child has a "note" prop
                // and the parent has notes[active_id]
                if self.state_types.contains_key("notes") && self.state_types.contains_key("active_id") {
                    code.push_str(&format!(
                        "                if let Some(__n) = self.notes.get_mut(self.active_id as usize) {{\n                    *__n = __child.note.clone();\n                }}\n"
                    ));
                }

                // Check if the child's note was marked as deleted via .note.deleted = true
                // If so, remove the note at active_id from the parent's notes array
                if self.state_types.contains_key("notes") && self.state_types.contains_key("active_id") {
                    code.push_str(&format!(
                        "                if __child.note[\"deleted\"].as_bool().unwrap_or(false) {{\n                    self.notes.remove(self.active_id as usize);\n                    if self.active_id >= self.notes.len() as i32 && !self.notes.is_empty() {{\n                        self.active_id = self.notes.len() as i32 - 1;\n                    }}\n                    self.editing = false;\n                }}\n"
                    ));
                }

                code.push_str("            }\n");
            }

            // Wildcard arm must come AFTER all named arms (including child forwarding)
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

    /// Pre-scan handler bodies to find local `let` bindings from function calls.
    /// These locals likely hold `serde_json::Value` results and need index access
    /// for field reads (e.g., `note.id` → `note["id"]`).
    fn scan_handler_locals(&mut self, widget: &AuraWidget) {
        for (_pattern, payload) in &widget.handlers {
            match payload {
                LogicPayload::AstStmts(stmts) => {
                    for stmt in stmts {
                        if let crate::ast::Stmt::Store(store) = stmt {
                            if matches!(store.kind, crate::ast::StoreKind::Let | crate::ast::StoreKind::Const) {
                                // Check if the value is a function call (likely returns Value)
                                if matches!(&store.expr, crate::ast::Expr::Call(_)) {
                                    self.value_locals.insert(store.name.as_str().to_string());
                                }
                            }
                        }
                    }
                }
                LogicPayload::AstBlock(stmts) => {
                    for stmt in stmts {
                        if let AuraStmt::Assign { target, value } = stmt {
                            // Aura Assign with a method call value and no dot in target = local var
                            if !target.contains('.') {
                                if matches!(value, AuraExpr::MethodCall { .. }) {
                                    self.value_locals.insert(target.clone());
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    /// Pre-scan view tree to find input/textarea elements and record event→field mappings
    fn scan_input_fields(&mut self, node: &AuraNode) {
        match node {
            AuraNode::Element { tag, props, events, children, .. } => {
                if tag == "input" || tag == "textarea" {
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
            AuraNode::ForLoop { body, .. } => {
                for child in body {
                    self.scan_input_fields(child);
                }
            }
            AuraNode::Conditional { then_body, else_body, .. } => {
                for child in then_body {
                    self.scan_input_fields(child);
                }
                if let Some(else_nodes) = else_body {
                    for child in else_nodes {
                        self.scan_input_fields(child);
                    }
                }
            }
            _ => {}
        }
    }

    /// Pre-scan the view tree to find custom widget references (e.g., EditorPanel, Sidebar).
    /// These need wrapper message variants in the parent's enum.
    fn scan_child_components(&mut self, node: &AuraNode) {
        match node {
            AuraNode::Element { tag, children, .. } => {
                if self.is_custom_widget(tag) && !self.child_components.contains(&tag.to_string()) {
                    self.child_components.push(tag.clone());
                }
                for child in children {
                    self.scan_child_components(child);
                }
            }
            AuraNode::ForLoop { body, .. } => {
                for child in body {
                    self.scan_child_components(child);
                }
            }
            AuraNode::Conditional { then_body, else_body, .. } => {
                for child in then_body {
                    self.scan_child_components(child);
                }
                if let Some(else_nodes) = else_body {
                    for child in else_nodes {
                        self.scan_child_components(child);
                    }
                }
            }
            AuraNode::Component { name, .. } => {
                if !self.child_components.contains(name) {
                    self.child_components.push(name.clone());
                }
            }
            _ => {}
        }
    }

    /// Generate view tree code
    fn generate_view_tree(&mut self, node: &AuraNode) -> String {
        match node {
            AuraNode::Element { tag, props, events, children, .. } => {
                // Handle custom widget references (e.g., EditorPanel, Sidebar)
                if self.is_custom_widget(tag) {
                    return self.generate_child_component(tag, props);
                }

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
                                let msg_name = self.current_msg_name();
                                builder = format!("{}.on_change({}::{})", builder, msg_name, variant);
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

                // Special handling for textarea elements — View::textarea(placeholder).value(...).on_change(...)
                if tag == "textarea" {
                    let placeholder = props.get("placeholder")
                        .and_then(|v| if let AuraPropValue::Expr(AuraExpr::Literal(s)) = v { Some(s.clone()) } else { None })
                        .unwrap_or_default();

                    let mut builder = format!("View::textarea(\"{}\")", placeholder);

                    // Value binding: value: .field → .value(format!("{}", self.field))
                    if let Some(AuraPropValue::Expr(AuraExpr::StateRef(name))) = props.get("value") {
                        builder = format!("{}.value(format!(\"{{}}\", self.{}))", builder, name);
                    }

                    // Other props (skip placeholder, value)
                    for (key, value) in props {
                        if key == "placeholder" || key == "value" { continue; }
                        builder = self.add_prop_to_builder(&builder, key, value);
                    }

                    // Events: oninput/onchange → on_change
                    for (event, handler) in events {
                        match event.as_str() {
                            "oninput" | "onInput" | "onchange" | "onChange" => {
                                let variant = self.extract_variant_name(&handler.handler);
                                let msg_name = self.current_msg_name();
                                builder = format!("{}.on_change({}::{})", builder, msg_name, variant);
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

                // Generate a Rust expression string for the text prop, handling ALL AuraExpr types.
                // This catches FieldAccess (note.title), Index, and other dynamic expressions
                // that fall through the Literal/StateRef checks above.
                let text_rust_expr: Option<String> = if text_prop.is_some() || text_state_ref.is_some() {
                    None // Already handled by text_prop or text_state_ref
                } else {
                    props.get("text").and_then(|v| {
                        if let AuraPropValue::Expr(expr) = v {
                            Some(self.expr_to_rust(expr))
                        } else {
                            None
                        }
                    })
                };

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
                    } else if let Some(ref text) = text_rust_expr {
                        // Dynamic expression (FieldAccess, Index, etc.) as text content
                        if tag == "button" {
                            format!("View::button({})", text)
                        } else {
                            format!("View::text({})", text)
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
                    if let Some(ref text) = text_rust_expr {
                        // text_rust_expr already produces a String (expr_to_rust adds .to_string())
                        return format!("View::text_styled({}, \"{}\")", text, style_str);
                    }
                }

                // Whether the "text" prop was consumed as a constructor arg
                let text_prop_consumed = self.is_leaf_tag(tag.as_str())
                    && (text_prop.is_some() || text_state_ref.is_some() || text_rust_expr.is_some());

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
                    if let Some(ref text) = text_rust_expr {
                        return format!("View::text({})", text);
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

                    // Add children — use .children() for for-loops (which produce Vec<View>),
                    // .child() for single views
                    for child in children {
                        let is_for_loop = matches!(child, AuraNode::ForLoop { .. });
                        let child_code = self.generate_view_tree(child);
                        if is_for_loop {
                            builder = format!("{}.children({})", builder, child_code);
                        } else {
                            builder = format!("{}.child({})", builder, child_code);
                        }
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
                let iter_name = iterable.trim_start_matches('.');
                let iter_expr = if iterable.starts_with('.') {
                    format!("self.{}", iter_name)
                } else {
                    iterable.clone()
                };

                // Check if iterable is a Value-type collection
                let is_value_iter = self.state_types.get(iter_name)
                    .map(|ty| ty.contains("serde_json::Value"))
                    .unwrap_or(false);

                // Push loop vars into scope
                self.push_loop_vars(var, index.as_deref());
                if is_value_iter {
                    self.value_loop_vars.insert(var.clone());
                }

                // Generate body with loop vars in scope
                let body_code: Vec<String> = body.iter()
                    .map(|child| self.generate_view_tree(child))
                    .collect();

                // Pop loop vars from scope
                self.pop_loop_vars(var, index.as_deref());
                self.value_loop_vars.remove(var);

                // Auto-generate search filter: if the widget has a "search" state var
                // and we're iterating a Value collection, insert .filter() before .map()
                let search_filter = if is_value_iter && self.state_types.contains_key("search") {
                    let var_ref = var.clone();
                    Some(format!(
                        ".filter(|{}| {{ \
                            let __q = self.search.to_lowercase(); \
                            if __q.is_empty() {{ return true; }} \
                            let __t = {}[\"title\"].as_str().unwrap_or_default().to_lowercase(); \
                            let __b = {}[\"body\"].as_str().unwrap_or_default().to_lowercase(); \
                            __t.contains(&__q) || __b.contains(&__q) \
                        }})",
                        var_ref, var_ref, var_ref
                    ))
                } else {
                    None
                };

                if let Some(idx) = index {
                    format!("{}.enumerate(){}{}.map(|({}, {})| {{ {} }})", iter_expr, search_filter.as_ref().map_or(String::new(), |f| f.clone()), "", idx, var, body_code.join("\n"))
                } else {
                    format!("{}.iter(){}{}.map(|{}| {{ {} }})", iter_expr, search_filter.as_ref().map_or(String::new(), |f| f.clone()), "", var, body_code.join("\n"))
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

            AuraNode::Component { name, props, .. } => {
                // Generate component instantiation with message wrapping
                let msg_name = self.current_msg_name();
                let mut constructor_args: Vec<String> = Vec::new();
                for (_key, value) in props {
                    let rust_expr = self.expr_to_rust(value);
                    constructor_args.push(rust_expr);
                }
                let args_str = constructor_args.join(", ");
                format!(
                    "{}::new({}).view().map_msg(|m| {}::{}(m))",
                    name, args_str, msg_name, name
                )
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

    /// Generate child component instantiation with message wrapping.
    /// E.g., EditorPanel(note: .notes[.active_id]) →
    ///   EditorPanel::new(self.notes[self.active_id as usize]).view().map_msg(|m| AppMsg::EditorPanel(m))
    fn generate_child_component(&self, tag: &str, props: &std::collections::HashMap<String, crate::aura::AuraPropValue>) -> String {
        let msg_name = self.current_msg_name();

        // Build constructor arguments from props
        // Props like "note: .notes[.active_id]" need to be converted to Rust expressions
        let mut constructor_args: Vec<String> = Vec::new();
        for (key, value) in props {
            if key == "style" || key == "class" { continue; }
            if let crate::aura::AuraPropValue::Expr(expr) = value {
                let rust_expr = self.expr_to_rust(expr);
                constructor_args.push(rust_expr);
            }
        }

        let args_str = constructor_args.join(", ");

        // Find parent state vars that should be synced to child before rendering.
        // This ensures the child's view reflects the current parent state (e.g., editing=true).
        let sync_fields: Vec<String> = self.state_types.keys()
            .filter(|name| {
                let ty = self.state_types.get(*name).map(|s| s.as_str()).unwrap_or("");
                !ty.starts_with("Vec<") && !name.ends_with("_id") && **name != "notes" && **name != "search"
            })
            .cloned()
            .collect();

        if sync_fields.is_empty() {
            format!(
                "{}::new({}).view().map_msg(|m| {}::{}(m))",
                tag, args_str, msg_name, tag
            )
        } else {
            let mut code = format!("{{ let mut __{} = {}::new({}); ", tag.to_lowercase(), tag, args_str);
            for field in &sync_fields {
                code.push_str(&format!("__{}.{} = self.{}.clone(); ", tag.to_lowercase(), field, field));
            }
            code.push_str(&format!("__{}.view().map_msg(|m| {}::{}(m)) }}", tag.to_lowercase(), msg_name, tag));
            code
        }
    }

    /// Find parent state vars that should be synced to/from child component fields.
    /// Matches by name: if parent has state var "editing" and child component likely
    /// has a field "editing", they should be synced.
    fn find_sync_fields_for_child(&self, widget: &AuraWidget) -> Vec<String> {
        let mut fields = Vec::new();
        for state in &widget.state_vars {
            let name = &state.name;
            // Skip collection types and id types — these don't map to child fields
            let ty = self.state_types.get(name).map(|s| s.as_str()).unwrap_or("");
            if ty.starts_with("Vec<") || name.ends_with("_id") || name == "notes" || name == "search" {
                continue;
            }
            // State vars like "editing", "edit_title", "edit_body" are candidates
            // to sync with child components that have the same fields
            fields.push(name.clone());
        }
        fields
    }

    /// Find constructor args expression for child component instantiation in handler.
    /// This mirrors generate_child_component but for the handler context.
    fn find_constructor_args_for_child(&self, widget: &AuraWidget) -> String {
        // Scan the view tree for the child component reference to get its props
        if let Some(args) = self.extract_child_constructor_args(&widget.view_tree) {
            return args;
        }
        String::new()
    }

    /// Recursively extract child component constructor args from view tree
    fn extract_child_constructor_args(&self, node: &AuraNode) -> Option<String> {
        match node {
            AuraNode::Element { tag, props, children, .. } => {
                if self.is_custom_widget(tag) {
                    let mut constructor_args: Vec<String> = Vec::new();
                    for (key, value) in props {
                        if key == "style" || key == "class" { continue; }
                        if let crate::aura::AuraPropValue::Expr(expr) = value {
                            let rust_expr = self.expr_to_rust(expr);
                            constructor_args.push(rust_expr);
                        }
                    }
                    return Some(constructor_args.join(", "));
                }
                for child in children {
                    if let Some(args) = self.extract_child_constructor_args(child) {
                        return Some(args);
                    }
                }
                None
            }
            AuraNode::ForLoop { body, .. } => {
                for child in body {
                    if let Some(args) = self.extract_child_constructor_args(child) {
                        return Some(args);
                    }
                }
                None
            }
            AuraNode::Conditional { then_body, else_body, .. } => {
                for child in then_body {
                    if let Some(args) = self.extract_child_constructor_args(child) {
                        return Some(args);
                    }
                }
                if let Some(else_nodes) = else_body {
                    for child in else_nodes {
                        if let Some(args) = self.extract_child_constructor_args(child) {
                            return Some(args);
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }

    /// Convert AURA condition to Rust expression
    fn convert_condition(&self, condition: &str) -> String {
        let result = condition.trim().to_string();

        // Replace state-ref dots like ".notes" → "self.notes", but NOT method call dots
        // like ".len()" or ".to_string()". A state-ref dot is one where the previous
        // character is NOT alphanumeric/underscore (i.e. it's at a word boundary).
        let bytes = result.as_bytes();
        let mut output = String::new();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'.'
                && i + 1 < bytes.len()
                && bytes[i + 1].is_ascii_alphabetic()
            {
                // Check if this dot is a method call (preceded by ident char)
                let is_method_call = i > 0
                    && (bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'_' || bytes[i - 1] == b')');
                if is_method_call {
                    output.push('.');
                } else {
                    output.push_str("self.");
                }
            } else {
                output.push(bytes[i] as char);
            }
            i += 1;
        }

        // Fix double self references
        output = output.replace("self.self.", "self.");

        output
    }

    /// Check if a tag is a custom widget reference (uppercase first letter, not a known tag)
    fn is_custom_widget(&self, tag: &str) -> bool {
        // Known tags that should not be treated as custom widgets
        const KNOWN_TAGS: &[&str] = &[
            "col", "column", "row", "grid", "scroll", "container", "center",
            "button", "input", "textarea", "checkbox", "toggle", "select", "option", "link",
            "text", "label", "span", "h1", "h2", "h3", "h4", "h5", "h6", "p",
            "table", "thead", "tbody", "tr", "th", "td", "tree", "tree_item",
            "tabs", "tab",
            "modal", "tooltip",
            "slider", "radio", "radiogroup",
            "progress", "badge", "spinner",
            "card", "avatar",
            "image", "icon",
            "divider", "spacer",
            "for", "if",
        ];
        // Custom widgets start with uppercase letter
        tag.chars().next().map_or(false, |c| c.is_uppercase()) && !KNOWN_TAGS.contains(&tag)
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
        let msg_name = self.current_msg_name();
        format!("|_| {}::{}", msg_name, variant)
    }

    /// Convert handler pattern to Rust closure with parameters
    fn handler_to_rust_closure_with_params(&self, handler: &str, params: &[String]) -> String {
        let variant = self.extract_variant_name(handler);
        let msg_name = self.current_msg_name();
        if params.is_empty() {
            format!("|_| {}::{}", msg_name, variant)
        } else {
            // Convert dot access on Value-type vars to index access
            let converted_params: Vec<String> = params.iter()
                .map(|p| self.convert_param_value_access(p, &variant))
                .collect();
            format!("|_| {}::{}({})", msg_name, variant, converted_params.join(", "))
        }
    }

    /// Convert dot access in param expressions for Value-type variables
    /// e.g., "note.id" → "note[\"id\"].as_i64().unwrap_or(0) as i32" for i32 payloads
    fn convert_param_value_access(&self, param: &str, variant_name: &str) -> String {
        // Check for patterns like "varname.field" or "varname.field.subfield"
        let parts: Vec<&str> = param.split('.').collect();
        if parts.len() >= 2 {
            let var_name = parts[0];
            if self.value_loop_vars.contains(var_name) || self.needs_index_access(var_name) {
                let field = parts[1..].join(".");
                // Check payload type to determine conversion
                let payload_ty = self.message_variants.iter()
                    .find(|v| v.name == variant_name)
                    .and_then(|v| v.payload.as_ref())
                    .map(|t| self.auto_type_to_rust(t));
                return match payload_ty.as_deref() {
                    Some("i32") => format!("{}[\"{}\"].as_i64().unwrap_or(0) as i32", var_name, field),
                    Some("i64") => format!("{}[\"{}\"].as_i64().unwrap_or(0)", var_name, field),
                    Some("String") => format!("{}[\"{}\"].as_str().unwrap_or_default().to_string()", var_name, field),
                    Some("bool") => format!("{}[\"{}\"].as_bool().unwrap_or(false)", var_name, field),
                    _ => format!("{}[\"{}\"]", var_name, field),
                };
            }
        }
        param.to_string()
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
                // Let/Const are local variables — use let binding
                // Var/Field are state variables — but only if they exist in state_types
                match store.kind {
                    crate::ast::StoreKind::Let | crate::ast::StoreKind::Const => {
                        format!("let {} = {}", name, value)
                    }
                    _ => {
                        // If name is a known state var, use self. prefix
                        if self.state_types.contains_key(name) {
                            format!("self.{} = {}", name, value)
                        } else {
                            // Otherwise it's a local var in handler context
                            format!("let {} = {}", name, value)
                        }
                    }
                }
            }
            crate::ast::Stmt::Expr(expr) => {
                self.ast_expr_to_rust(expr)
            }
            _ => format!("/* unhandled stmt */"),
        }
    }

    /// Convert a crate::ast::Expr to Rust code (for on-handler bodies)
    /// Generate the appropriate serde_json::Value field access expression.
    /// Uses heuristic based on field name to pick the right type accessor.
    fn value_field_access(&self, obj_expr: &str, field: &str) -> String {
        if field == "id" || field.ends_with("_id") {
            format!("{}[\"{}\"].as_i64().unwrap_or(0) as i32", obj_expr, field)
        } else if field == "deleted" || field.starts_with("is_") {
            format!("{}[\"{}\"].as_bool().unwrap_or(false)", obj_expr, field)
        } else {
            format!("{}[\"{}\"].as_str().unwrap_or_default().to_string()", obj_expr, field)
        }
    }

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
                    let path = &s[1..];
                    // Check for dotted path on Value-type var (e.g., ".note.title")
                    if let Some(dot_pos) = path.find('.') {
                        let first = &path[..dot_pos];
                        if self.needs_index_access(first) {
                            let field = &path[dot_pos + 1..];
                            // Reading from serde_json::Value: use index + string conversion
                            return format!("self.{}[\"{}\"].as_str().unwrap_or_default().to_string()", first, field);
                        }
                    }
                    format!("self.{}", path)
                } else if self.state_types.contains_key(s) || self.prop_names.contains(s) {
                    format!("self.{}", s)
                } else {
                    s.to_string()
                }
            }
            Expr::Dot(obj, field) => {
                let field_str = field.as_str();
                // Detect pattern: Dot(Dot(Ident("self"), prop_name), field_name)
                // This is self.prop_name.field_name — check if prop_name is Value-type
                if let Expr::Dot(inner_obj, inner_field) = obj.as_ref() {
                    if let Expr::Ident(inner_name) = inner_obj.as_ref() {
                        let inner_s = inner_name.as_str();
                        let prop_name = inner_field.as_str();
                        // Pattern: self.prop_name.field_str
                        if (inner_s == "self" || inner_s.starts_with('.')) && self.needs_index_access(prop_name) {
                            // Reading from Value: self.note["field"] with type-aware accessor
                            let obj_expr = format!("self.{}", prop_name);
                            return self.value_field_access(&obj_expr, field_str);
                        }
                    }
                }
                // If accessing a field on a Value-type prop directly: obj.field where obj is a prop
                if let Expr::Ident(name) = obj.as_ref() {
                    let s = name.as_str();
                    let resolved = if s.starts_with('.') { &s[1..] } else { s };
                    if self.needs_index_access(resolved) {
                        let obj_str = if s == "self" || s.starts_with('.') {
                            format!("self.{}", resolved)
                        } else if self.state_types.contains_key(resolved) || self.prop_names.contains(resolved) {
                            format!("self.{}", resolved)
                        } else {
                            resolved.to_string()
                        };
                        return self.value_field_access(&obj_str, field_str);
                    }
                }
                let obj_str = self.ast_expr_to_rust(obj);
                format!("{}.{}", obj_str, field_str)
            }
            Expr::Bina(left, op, right) => {
                // Assignment: .count = expr → self.count = expr
                if matches!(op, Op::Asn) {
                    // Check if target is a Value field write like self.note.title = value
                    // Pattern: Dot(Dot(Ident("self"), "note"), "title")
                    if let Expr::Dot(outer_obj, outer_field) = left.as_ref() {
                        if let Expr::Dot(inner_obj, inner_field) = outer_obj.as_ref() {
                            if let Expr::Ident(inner_name) = inner_obj.as_ref() {
                                let inner_s = inner_name.as_str();
                                let prop_name = inner_field.as_str();
                                if (inner_s == "self" || inner_s.starts_with('.')) && self.needs_index_access(prop_name) {
                                    let field = outer_field.as_str();
                                    let value = self.ast_expr_to_rust(right);
                                    // Write to Value field: self.note["title"] = json!(value)
                                    return format!("self.{}[\"{}\"] = serde_json::json!({})", prop_name, field, value);
                                }
                            }
                        }
                    }
                    // Also check for single-dot Ident pattern like ".note.title"
                    if let Expr::Ident(name) = left.as_ref() {
                        let s = name.as_str();
                        if s.starts_with('.') {
                            let path = &s[1..];
                            if let Some(dot_pos) = path.find('.') {
                                let first = &path[..dot_pos];
                                if self.needs_index_access(first) {
                                    let field = &path[dot_pos + 1..];
                                    let value = self.ast_expr_to_rust(right);
                                    return format!("self.{}[\"{}\"] = serde_json::json!({})", first, field, value);
                                }
                            }
                        }
                    }
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
                    .map(|a| {
                        let expr = self.ast_expr_to_rust(&a.get_expr());
                        // In &mut self methods, passing self.field by value moves it.
                        // Add .clone() for String-typed fields to avoid E0507.
                        if expr.starts_with("self.") {
                            let field_name = &expr[5..];
                            // Don't clone for index access patterns like self.note["id"]
                            if !field_name.contains('[') {
                                if let Some(ty) = self.state_types.get(field_name) {
                                    if ty == "String" {
                                        return format!("{}.clone()", expr);
                                    }
                                }
                            }
                        }
                        expr
                    })
                    .collect();
                match fn_name.as_str() {
                    "print" => {
                        let print_args: Vec<String> = args.iter()
                            .map(|a| a.trim_end_matches(".to_string()").to_string())
                            .collect();
                        format!("println!({})", print_args.join(", "))
                    }
                    _ => {
                        let result = if fn_name.ends_with(".remove") {
                            // .remove() takes usize, cast args
                            let casted_args: Vec<String> = args.iter()
                                .map(|a| format!("{} as usize", a))
                                .collect();
                            format!("{}({})", fn_name, casted_args.join(", "))
                        } else if fn_name.ends_with(".push") {
                            // .push() for Value vectors — clone args that are value_locals
                            // to avoid borrow-after-move when the local is used later
                            let cloned_args: Vec<String> = args.iter()
                                .map(|a| {
                                    let bare = a.trim_start_matches("self.");
                                    if self.value_locals.contains(bare) {
                                        format!("{}.clone()", a)
                                    } else {
                                        a.clone()
                                    }
                                })
                                .collect();
                            format!("{}({})", fn_name, cloned_args.join(", "))
                        } else {
                            format!("{}({})", fn_name, args.join(", "))
                        };
                        // .len() returns usize — cast to i32 for AURA compatibility
                        if fn_name.ends_with(".len") {
                            format!("{} as i32", result)
                        } else {
                            result
                        }
                    }
                }
            }
            Expr::Object(pairs) => {
                let fields: Vec<String> = pairs.iter()
                    .map(|p| {
                        let key_str = match &p.key {
                            crate::ast::Key::NamedKey(name) => format!("\"{}\"", name.as_str()),
                            crate::ast::Key::IntKey(i) => i.to_string(),
                            crate::ast::Key::BoolKey(b) => b.to_string(),
                            crate::ast::Key::StrKey(s) => format!("\"{}\"", s),
                        };
                        let value = self.ast_expr_to_json_value(&p.value);
                        format!("{}: {}", key_str, value)
                    })
                    .collect();
                format!("serde_json::json!({{{}}})", fields.join(", "))
            }
            Expr::Array(elems) => {
                let elems_str: Vec<String> = elems.iter()
                    .map(|e| self.ast_expr_to_rust(e))
                    .collect();
                format!("vec![{}]", elems_str.join(", "))
            }
            Expr::Index(target, index) => {
                let target_str = self.ast_expr_to_rust(target);
                let index_str = self.ast_expr_to_rust(index);
                format!("{}[{}]", target_str, index_str)
            }
            Expr::Nil | Expr::Null => "serde_json::Value::Null".to_string(),
            _ => format!("/* expr */"),
        }
    }

    /// Generate a json!()-compatible value expression (strings without .to_string())
    fn ast_expr_to_json_value(&self, expr: &crate::ast::Expr) -> String {
        use crate::ast::Expr;
        match expr {
            Expr::Str(s) => format!("\"{}\"", s),
            Expr::I64(n) => n.to_string(),
            Expr::Int(n) => n.to_string(),
            Expr::U64(n) => n.to_string(),
            Expr::Uint(n) => n.to_string(),
            Expr::Bool(b) => b.to_string(),
            Expr::Ident(name) => {
                let s = name.as_str();
                if s.starts_with('.') {
                    format!("self.{}", &s[1..])
                } else if self.state_types.contains_key(s) || self.prop_names.contains(s) {
                    format!("self.{}", s)
                } else {
                    s.to_string()
                }
            }
            Expr::Object(pairs) => {
                let fields: Vec<String> = pairs.iter()
                    .map(|p| {
                        let key_str = match &p.key {
                            crate::ast::Key::NamedKey(name) => format!("\"{}\"", name.as_str()),
                            crate::ast::Key::IntKey(i) => i.to_string(),
                            _ => String::new(),
                        };
                        let value = self.ast_expr_to_json_value(&p.value);
                        format!("{}: {}", key_str, value)
                    })
                    .collect();
                format!("serde_json::json!({{{}}})", fields.join(", "))
            }
            _ => self.ast_expr_to_rust(expr),
        }
    }

    /// Generate a json!()-compatible value from AuraExpr (strings without .to_string())
    fn expr_to_json_value(&self, expr: &AuraExpr) -> String {
        match expr {
            AuraExpr::Literal(s) => format!("\"{}\"", s),
            AuraExpr::Int(n) => n.to_string(),
            AuraExpr::Float(n) => n.to_string(),
            AuraExpr::Bool(b) => b.to_string(),
            AuraExpr::StateRef(name) => format!("self.{}", name),
            AuraExpr::Object(fields) => {
                let pairs: Vec<String> = fields.iter()
                    .map(|(k, v)| format!("\"{}\": {}", k, self.expr_to_json_value(v)))
                    .collect();
                format!("serde_json::json!({{{}}})", pairs.join(", "))
            }
            _ => self.expr_to_rust(expr),
        }
    }

    /// Convert a dotted target path to Rust, using index access for Value-type vars
    /// e.g., "note.title" → "self.note[\"title\"]" when note is a Value prop
    fn convert_target_to_rust(&self, target: &str) -> String {
        let parts: Vec<&str> = target.split('.').collect();
        if parts.len() >= 2 {
            let first = parts[0];
            if self.needs_index_access(first) {
                let field = parts[1..].join(".");
                return format!("self.{}[\"{}\"]", first, field);
            }
        }
        format!("self.{}", target)
    }

    /// Convert AuraStmt to Rust
    fn stmt_to_rust(&self, stmt: &AuraStmt) -> String {
        match stmt {
            AuraStmt::Assign { target, value } => {
                let value_str = self.expr_to_rust(value);
                // Check if target is a dotted path on a Value-type var
                let parts: Vec<&str> = target.split('.').collect();
                if parts.len() >= 2 {
                    let first = parts[0];
                    if self.needs_index_access(first) {
                        // Write to Value field: self.note["title"] = json!(value)
                        let field = parts[1..].join(".");
                        return format!("self.{}[\"{}\"] = serde_json::json!({})", first, field, value_str);
                    }
                }
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
                // .remove() takes usize — cast i32 args
                if method == "remove" {
                    let casted_args: Vec<String> = args_str.iter()
                        .map(|a| format!("{} as usize", a))
                        .collect();
                    format!("self.{}.{}({})", object, method, casted_args.join(", "))
                } else {
                    format!("self.{}.{}({})", object, method, args_str.join(", "))
                }
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
                // Convert .len to .len() as i32 for Rust (AURA uses i32 for lengths)
                if method == "len" && args.is_empty() {
                    format!("{}.len() as i32", object_str)
                } else if method == "remove" {
                    // .remove() takes usize — cast args
                    let casted_args: Vec<String> = args_str.iter()
                        .map(|a| format!("{} as usize", a))
                        .collect();
                    format!("{}.{}({})", object_str, method, casted_args.join(", "))
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
                    .map(|(k, v)| format!("\"{}\": {}", k, self.expr_to_json_value(v)))
                    .collect();
                format!("serde_json::json!({{{}}})", pairs.join(", "))
            }
            AuraExpr::Lambda { params, body } => {
                let body_str = self.expr_to_rust(body);
                format!("|{}| {}", params.join(", "), body_str)
            }
            AuraExpr::FieldAccess { object, field } => {
                // Check if object is a loop variable FIRST (before computing object_str)
                // to avoid generating self.note when it should be just note
                if let AuraExpr::StateRef(name) = object.as_ref() {
                    if self.value_loop_vars.contains(name) {
                        // Loop variable is &serde_json::Value — use index access
                        // Use just the name (not self.name) since it's a closure param
                        return self.value_field_access(name, field);
                    }
                }
                let object_str = self.expr_to_rust(object);
                // Check if object is a Value-type state variable needing index access
                if let AuraExpr::StateRef(name) = object.as_ref() {
                    if self.needs_index_access(name) {
                        // Reading from serde_json::Value: use index + type conversion
                        return self.value_field_access(&object_str, field);
                    }
                }
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
            AuraExpr::Index { target, index } => {
                let target_str = self.expr_to_rust(target);
                let index_str = self.expr_to_rust(index);
                // Vec indexing requires usize — add cast if index is an i32 expression
                let index_cast = if index_str.starts_with("self.") {
                    // Likely an i32 state var, cast to usize
                    format!("{} as usize", index_str)
                } else {
                    index_str
                };
                format!("{}[{}].clone()", target_str, index_cast)
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
        use crate::ast::Type;
        match ty {
            Type::Int => "i32".to_string(),
            Type::Uint => "u32".to_string(),
            Type::I64 => "i64".to_string(),
            Type::U64 => "u64".to_string(),
            Type::Float => "f32".to_string(),
            Type::Double => "f64".to_string(),
            Type::Bool => "bool".to_string(),
            Type::StrFixed(_) | Type::StrOwned | Type::StrSlice => "String".to_string(),
            Type::Void => "()".to_string(),
            Type::Array(arr) => format!("Vec<{}>", self.auto_type_to_rust(&arr.elem)),
            Type::RuntimeArray(arr) => format!("Vec<{}>", self.auto_type_to_rust(&arr.elem)),
            Type::List(inner) => format!("Vec<{}>", self.auto_type_to_rust(inner)),
            Type::Slice(sl) => format!("Vec<{}>", self.auto_type_to_rust(&sl.elem)),
            Type::Map(k, v) => format!("std::collections::HashMap<{}, {}>", self.auto_type_to_rust(k), self.auto_type_to_rust(v)),
            Type::User(td) => td.name.to_string(),
            Type::Unknown => "serde_json::Value".to_string(),
            _ => "serde_json::Value".to_string(), // Fallback for unrecognized types
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
            key_bindings: HashMap::new(),
            api_imports: vec![],
        }
;

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
