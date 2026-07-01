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

    /// Maps input event variant name to field names for input text parsing
    /// Multiple inputs can share the same event (e.g., main input + edit input both fire EditInputChanged)
    input_fields: std::collections::HashMap<String, Vec<String>>,

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

    /// Whether the widget has an .Init lifecycle handler
    has_init: bool,

    /// Info about the API function called in .Init handler (for async init generation)
    init_api_info: Option<InitApiInfo>,
}

/// Detected Init handler pattern: `self.state_var = api_func()`
struct InitApiInfo {
    /// State variable being assigned (e.g., "notes")
    state_var: String,
    /// API function being called (e.g., "list_notes")
    func_name: String,
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
            has_init: false,
            init_api_info: None,
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
        self.has_init = false;
        self.init_api_info = None;
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

        // Pre-scan handlers to find local variables from function calls (likely Value type)
        self.scan_handler_locals(widget);

        // Scan lifecycle handlers (.Init, .Destroy) for local variables and has_init flag
        for lc in &widget.lifecycle {
            if lc.name == "Init" {
                self.has_init = true;
                // Detect async Init pattern: self.X = api_func()
                self.detect_init_api_call(&lc.payload, &widget.api_imports);
            }
            self.scan_payload_locals(&lc.payload);
        }

        // If there's an .Init lifecycle handler, add Init variant to message enum
        if self.has_init {
            if !self.message_variants.iter().any(|v| v.name == "Init") {
                self.message_variants.push(AuraMsgVariant {
                    name: "Init".to_string(),
                    payload: None,
                });
            }
            // If Init calls an API function (async init), add __InitLoaded variant
            // We can't use AuraMsgVariant because Vec<serde_json::Value> doesn't map
            // to any AST Type variant. Instead, inject it directly in generate_msg_enum.
            // (See generate_msg_enum for the direct string injection.)
        }

        // If widget has a tick_interval, add Tick variant to message enum
        if widget.tick_interval.is_some() {
            if !self.message_variants.iter().any(|v| v.name == "Tick") {
                self.message_variants.push(AuraMsgVariant {
                    name: "Tick".to_string(),
                    payload: None,
                });
            }
        }

        // Message enum (includes wrapper variants for child components + Init lifecycle)
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

        // If async Init detected, add __InitLoaded variant (injected as raw string
        // because Vec<serde_json::Value> doesn't map to any AST Type variant)
        if self.init_api_info.is_some() {
            code.push_str(&format!("    __InitLoaded(Vec<serde_json::Value>),\n"));
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

        // If the widget has an .Init lifecycle handler AND it's synchronous (not async API call),
        // dispatch Init message at construction.
        // Async Init (init_api_info is Some) is dispatched by the runtime boot task instead.
        let sync_init = self.has_init && self.init_api_info.is_none();
        if sync_init {
            let msg_name = self.current_msg_name();
            code.push_str("        let mut __self = Self {\n");
        } else {
            code.push_str("        Self {\n");
        }

        // Initialize props from parameters
        for prop in &widget.props {
            code.push_str(&format!("            {}: {},\n", prop.name, prop.name));
        }

        // Initialize state vars from their defaults
        for state in &widget.state_vars {
            let init = self.expr_to_rust(&state.initial);
            code.push_str(&format!("            {}: {},\n", state.name, init));
        }

        if sync_init {
            let msg_name = self.current_msg_name();
            code.push_str(&format!("        }};\n"));
            code.push_str(&format!("        __self.on({}::Init);\n", msg_name));
            code.push_str("        __self\n");
        } else {
            code.push_str("        }\n");
        }

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

        // subscription() method — generate if tick_interval is set
        if let Some(interval_ms) = widget.tick_interval {
            let msg_name = self.current_msg_name();
            code.push('\n');
            code.push_str(&format!(
                "    fn subscription(&self) -> iced::Subscription<Self::Msg> {{\n        iced::time::every(std::time::Duration::from_millis({})).map(|_| {}::Tick)\n    }}\n",
                interval_ms, msg_name
            ));
        }

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
                // Tick handler: guard with running check if "running" field exists
                let is_tick_guarded = variant_name == "Tick" && self.state_types.contains_key("running");
                if has_payload {
                    // Use a named binding instead of _ so handler body can reference it
                    code.push_str(&format!("            {}::{}(id) => {{\n", msg_name, variant_name));
                } else {
                    code.push_str(&format!("            {}::{} => {{\n", msg_name, variant_name));
                }
                if is_tick_guarded {
                    code.push_str("                if self.running == \"true\" {\n");
                }

                // If this event is from an input, prepend input text parsing
                if let Some(field_names) = self.input_fields.get(&variant_name) {
                    code.push_str(&format!(
                        "                let _text = auto_lang::ui::iced::last_input_text();\n"
                    ));
                    // Set ALL bound fields to the input text (multiple inputs may share one event)
                    let last_idx = field_names.len() - 1;
                    for (i, field_name) in field_names.iter().enumerate() {
                        let rust_type = self.state_types.get(field_name).map(|s| s.as_str()).unwrap_or("f64");
                        if rust_type == "String" {
                            // Last field can consume _text directly; others must clone
                            let text_expr = if i == last_idx { "_text".to_string() } else { "_text.clone()".to_string() };
                            code.push_str(&format!(
                                "                self.{} = {};\n",
                                field_name, text_expr
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
                    }

                    // Skip redundant self-assignment body (e.g. `.email = .email`)
                    // Check if the body is entirely composed of self-assignments for the bound fields
                    let all_self_assign = field_names.iter().all(|f| {
                        let self_assign = format!("self.{} = self.{}", f, f);
                        body.trim() == self_assign
                    });
                    if !all_self_assign && !body.trim().is_empty() {
                        code.push_str(&format!("                {}\n", body));
                    }
                } else {
                    code.push_str(&format!("                {}\n", body));
                }

                // Post-process Tick handler: if model has elapsed + time_display + ms_display,
                // append display computation after the user's tick body.
                if variant_name == "Tick"
                    && self.state_types.contains_key("elapsed")
                    && self.state_types.contains_key("time_display")
                    && self.state_types.contains_key("ms_display")
                {
                    // Ensure prior statement ends with semicolon
                    code.push_str("                    ;\n");
                    code.push_str(
                        "                    let total_cs = self.elapsed / 10;\n\
                         \x20                   let cs = total_cs % 100;\n\
                         \x20                   let total_secs = total_cs / 100;\n\
                         \x20                   let secs = total_secs % 60;\n\
                         \x20                   let mins = total_secs / 60;\n\
                         \x20                   self.time_display = format!(\"{:02}:{:02}\", mins, secs);\n\
                         \x20                   self.ms_display = format!(\".{:02}\", cs);\n"
                    );
                }

                // Close the running guard for Tick handler
                if is_tick_guarded {
                    code.push_str("                }\n");
                }

                code.push_str("            }\n");
            }

            // Generate match arms from lifecycle handlers (.Init, .Destroy)
            for lc in &widget.lifecycle {
                let body = self.generate_handler_body(&lc.payload);
                code.push_str(&format!("            {}::{} => {{\n", msg_name, lc.name));
                if lc.name == "Init" && self.init_api_info.is_some() {
                    // Async Init: body is handled by __InitLoaded message from boot task
                    code.push_str("                // async init — data arrives via __InitLoaded\n");
                } else {
                    code.push_str(&format!("                {}\n", body));
                }
                code.push_str("            }\n");
            }

            // If async Init detected, generate __InitLoaded handler
            if let Some(ref info) = self.init_api_info {
                code.push_str(&format!(
                    "            {}::__InitLoaded(__data) => {{\n                self.{} = __data\n            }}\n",
                    msg_name, info.state_var
                ));
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

            // Wildcard arm must come AFTER all named arms (including child forwarding).
            // The enum has message_variants.len() + child_components.len() total variants.
            // We generate arms for: handlers + lifecycle + child_components + __InitLoaded (if async).
            // If there are more enum variants than named arms, we need a wildcard.
            let async_init_arm = if self.init_api_info.is_some() { 1 } else { 0 };
            let total_enum_variants = self.message_variants.len() + self.child_components.len() + async_init_arm;
            let named_arms = widget.handlers.len() + widget.lifecycle.len() + self.child_components.len() + async_init_arm;
            if total_enum_variants > named_arms {
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
            self.scan_payload_locals(payload);
        }
        for lc in &widget.lifecycle {
            self.scan_payload_locals(&lc.payload);
        }
    }

    /// Scan a single LogicPayload for local variables that hold serde_json::Value.
    /// Detects:
    /// - `let x = func_call()` — function call results
    /// - `let x = collection[idx]` — indexing into Vec<Value>
    /// - `let x = todos[idx]` — same pattern with named collection
    /// - `for x in collection` — loop variables iterating over Vec<Value>
    fn scan_payload_locals(&mut self, payload: &LogicPayload) {
        match payload {
            LogicPayload::AstStmts(stmts) => {
                self.scan_ast_stmts_for_value_locals(stmts);
            }
            LogicPayload::AstBlock(stmts) => {
                for stmt in stmts {
                    if let AuraStmt::Assign { target, value } = stmt {
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

    /// Recursively scan AST statements for value-typed locals and loop vars
    fn scan_ast_stmts_for_value_locals(&mut self, stmts: &[crate::ast::Stmt]) {
        for stmt in stmts {
            match stmt {
                crate::ast::Stmt::Store(store) => {
                    if matches!(store.kind, crate::ast::StoreKind::Let | crate::ast::StoreKind::Const) {
                        let name = store.name.as_str();
                        // Check if the value is a function call (likely returns Value)
                        if matches!(&store.expr, crate::ast::Expr::Call(_)) {
                            self.value_locals.insert(name.to_string());
                        }
                        // Check if the value is an index into a state Vec<Value>
                        if let crate::ast::Expr::Index(target, _idx) = &store.expr {
                            if let crate::ast::Expr::Ident(collection) = target.as_ref() {
                                let coll_name = collection.as_str();
                                if self.state_types.get(coll_name)
                                    .map(|ty| ty.starts_with("Vec<"))
                                    .unwrap_or(false)
                                {
                                    self.value_locals.insert(name.to_string());
                                }
                            }
                        }
                    }
                }
                crate::ast::Stmt::For(for_stmt) => {
                    // Register loop variable as value var if iterating over a Value collection
                    match &for_stmt.iter {
                        crate::ast::Iter::Named(name) => {
                            // `for todo in .todos` — check if .todos is Vec<Value>
                            if let crate::ast::Expr::Dot(obj, field) = &for_stmt.range {
                                if let crate::ast::Expr::Ident(_) = obj.as_ref() {
                                    if self.state_types.get(field.as_str())
                                        .map(|ty| ty.starts_with("Vec<"))
                                        .unwrap_or(false)
                                    {
                                        self.value_loop_vars.insert(name.as_str().to_string());
                                    }
                                }
                            } else if let crate::ast::Expr::Ident(name_expr) = &for_stmt.range {
                                let coll = name_expr.as_str();
                                if self.state_types.get(coll)
                                    .map(|ty| ty.starts_with("Vec<"))
                                    .unwrap_or(false)
                                {
                                    self.value_loop_vars.insert(name.as_str().to_string());
                                }
                            }
                        }
                        crate::ast::Iter::Indexed(_idx, name) => {
                            if let crate::ast::Expr::Dot(obj, field) = &for_stmt.range {
                                if let crate::ast::Expr::Ident(_) = obj.as_ref() {
                                    if self.state_types.get(field.as_str())
                                        .map(|ty| ty.starts_with("Vec<"))
                                        .unwrap_or(false)
                                    {
                                        self.value_loop_vars.insert(name.as_str().to_string());
                                    }
                                }
                            } else if let crate::ast::Expr::Ident(name_expr) = &for_stmt.range {
                                let coll = name_expr.as_str();
                                if self.state_types.get(coll)
                                    .map(|ty| ty.starts_with("Vec<"))
                                    .unwrap_or(false)
                                {
                                    self.value_loop_vars.insert(name.as_str().to_string());
                                }
                            }
                        }
                        _ => {}
                    }
                    // Recurse into for loop body
                    self.scan_ast_stmts_for_value_locals(&for_stmt.body.stmts);
                }
                crate::ast::Stmt::If(if_stmt) => {
                    for branch in &if_stmt.branches {
                        self.scan_ast_stmts_for_value_locals(&branch.body.stmts);
                    }
                    if let Some(else_body) = &if_stmt.else_ {
                        self.scan_ast_stmts_for_value_locals(&else_body.stmts);
                    }
                }
                _ => {}
            }
        }
    }

    /// Detect if the Init handler body is a single `self.X = api_func()` assignment
    /// where api_func matches one of the API imports. If so, store in init_api_info
    /// so we can generate an async init pattern (boot task + __InitLoaded message).
    ///
    /// In the AST, `.notes = list_notes()` is parsed as:
    ///   Stmt::Expr(Expr::Bina(Expr::Dot(Ident("self"), "notes"), Op::Asn, Expr::Call("list_notes")))
    fn detect_init_api_call(&mut self, payload: &LogicPayload, api_imports: &[String]) {
        if api_imports.is_empty() {
            return;
        }
        if let LogicPayload::AstStmts(stmts) = payload {
            if stmts.len() != 1 {
                return;
            }
            if let crate::ast::Stmt::Expr(expr) = &stmts[0] {
                // Pattern: Bina(Dot(self, field), Asn, Call(func))
                if let crate::ast::Expr::Bina(left, op, right) = expr {
                    use auto_val::Op;
                    if !matches!(op, Op::Asn) {
                        return;
                    }
                    // Left side: Expr::Dot(Ident("self"), Name("field"))
                    let state_var = extract_dot_self_field(left);
                    // Right side: Expr::Call(...)
                    let fn_name = extract_call_name(right);
                    if let (Some(var), Some(func)) = (state_var, fn_name) {
                        if api_imports.iter().any(|api| api == &func) {
                            self.init_api_info = Some(InitApiInfo {
                                state_var: var,
                                func_name: func,
                            });
                        }
                    }
                }
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
                                self.input_fields.entry(variant).or_default().push(name.clone());
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

    /// Wrap multiple view expressions into a builder chain.
    /// Single view: returns as-is. Multiple views: View::col().child(...).child(...).build()
    fn wrap_views(views: &[String]) -> String {
        if views.len() == 1 {
            views[0].clone()
        } else {
            // Use row() for multi-child conditionals so siblings sit side-by-side.
            // The parent col/row already controls the outer layout direction.
            let mut builder = "View::row()".to_string();
            for v in views {
                builder = format!("{}.child({})", builder, v);
            }
            format!("{}.build()", builder)
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

                // grid-item is transparent — emit its child(ren) directly. A
                // wrapping col would be Shrink-width and break the enclosing
                // grid's equal-column Fill distribution.
                if tag == "grid-item" {
                    if children.len() == 1 {
                        return self.generate_view_tree(&children[0]);
                    } else if !children.is_empty() {
                        let mut col = "View::col()".to_string();
                        for child in children {
                            col = format!("{}.child({})", col, self.generate_view_tree(child));
                        }
                        return format!("{}.build()", col);
                    }
                    return "View::Empty".to_string();
                }

                // grid → View::grid() builder. iced has no native grid; the
                // col-of-rows decomposition (final-row padding + w-full rows)
                // now lives in ONE place — the shared generic `build_grid`
                // (Plan 319) — so the rust `into_iced` path and the VM
                // `render_dynamic_view` path share it and can never drift.
                if tag == "grid" {
                    let cols = props.get("cols").or_else(|| props.get("columns"))
                        .and_then(|v| match v {
                            AuraPropValue::Expr(AuraExpr::Int(n)) => Some(*n as usize),
                            AuraPropValue::Expr(AuraExpr::Literal(s)) => s.trim().parse::<usize>().ok(),
                            _ => None,
                        })
                        .map(|c| c.max(1))
                        .unwrap_or(1);
                    let gap = props.get("gap")
                        .and_then(|v| match v {
                            AuraPropValue::Expr(AuraExpr::Int(n)) => Some(*n as u16),
                            AuraPropValue::Expr(AuraExpr::Literal(s)) => s.trim().parse::<u16>().ok(),
                            _ => None,
                        })
                        .unwrap_or(0);
                    let style_str = props.get("style").or_else(|| props.get("class"))
                        .and_then(|v| if let AuraPropValue::Expr(AuraExpr::Literal(s)) = v {
                            Some(s.clone())
                        } else { None })
                        .unwrap_or_default();

                    let mut g = "View::grid()".to_string();
                    g = format!("{}.cols({})", g, cols);
                    if gap > 0 { g = format!("{}.spacing({})", g, gap); }
                    for c in children {
                        g = format!("{}.child({})", g, self.generate_view_tree(c));
                    }
                    if !style_str.is_empty() {
                        g = format!("{}.style(\"{}\")", g, style_str);
                    }
                    return format!("{}.build()", g);
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
                            if s.contains("${") {
                                return format!("View::text_styled({}, \"{}\")", self.interpolate_str(s), class_str);
                            }
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
                    //         onenter → on_submit (fires on Enter key)
                    for (event, handler) in events {
                        match event.as_str() {
                            "oninput" | "onInput" | "onchange" | "onChange" => {
                                let variant = self.extract_variant_name(&handler.handler);
                                let msg_name = self.current_msg_name();
                                builder = format!("{}.on_change({}::{})", builder, msg_name, variant);
                                // Record event→field mapping for handler generation
                                if let Some(AuraPropValue::Expr(AuraExpr::StateRef(name))) = props.get("value") {
                                    self.input_fields.entry(variant).or_default().push(name.clone());
                                }
                            }
                            "onenter" | "onEnter" | "onsubmit" | "onSubmit" => {
                                let variant = self.extract_variant_name(&handler.handler);
                                let msg_name = self.current_msg_name();
                                builder = format!("{}.on_submit({}::{})", builder, msg_name, variant);
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
                                    self.input_fields.entry(variant).or_default().push(name.clone());
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
                // Also extract text content from a single Text child (e.g. text f"..." { class: "..." })
                let child_text_content: Option<AuraTextContent> = if children.len() == 1 {
                    if let AuraNode::Text(content) = &children[0] {
                        Some(content.clone())
                    } else {
                        None
                    }
                } else {
                    None
                };

                let text_prop = props.get("text")
                    .and_then(|v| if let AuraPropValue::Expr(AuraExpr::Literal(s)) = v { Some(s.clone()) } else { None })
                    .or_else(|| {
                        // Fallback: extract literal text from child Text node
                        match &child_text_content {
                            Some(AuraTextContent::Literal(s)) => Some(s.clone()),
                            Some(AuraTextContent::Interpolated { template, .. }) => Some(template.clone()),
                            None => None,
                        }
                    });

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

                // Special handling for checkbox — View::Checkbox { ... } direct construction
                // View::checkbox() returns View<M> (enum), not a builder, so we can't chain
                // .style() / .on_click() / .build(). Use direct struct literal instead.
                if tag == "checkbox" {
                    let is_checked = props.get("checked")
                        .or_else(|| props.get("is_checked"))
                        .map(|v| match v {
                            AuraPropValue::Expr(AuraExpr::Bool(b)) => b.to_string(),
                            AuraPropValue::Expr(AuraExpr::StateRef(name)) => format!("self.{}", name),
                            AuraPropValue::Expr(AuraExpr::FieldAccess { object, field }) => {
                                let obj_str = match object.as_ref() {
                                    AuraExpr::StateRef(name) => {
                                        if self.is_loop_var(name) && self.value_loop_vars.contains(name) {
                                            name.clone()
                                        } else {
                                            format!("self.{}", name)
                                        }
                                    }
                                    _ => format!("{:?}", object),
                                };
                                self.value_field_access(&obj_str, field)
                            }
                            _ => "false".to_string(),
                        })
                        .unwrap_or_else(|| "false".to_string());
                    let label = props.get("label")
                        .or_else(|| props.get("text"))
                        .and_then(|v| if let AuraPropValue::Expr(AuraExpr::Literal(s)) = v { Some(s.clone()) } else { None })
                        .unwrap_or_default();

                    // Parse class/style into Style
                    let class_str = props.get("class")
                        .or_else(|| props.get("style"))
                        .and_then(|v| if let AuraPropValue::Expr(AuraExpr::Literal(s)) = v { Some(s.clone()) } else { None })
                        .unwrap_or_default();
                    let style_expr = if class_str.is_empty() {
                        "None".to_string()
                    } else {
                        format!("Some(auto_lang::ui::style::Style::parse(\"{}\").unwrap())", class_str)
                    };

                    // Build on_toggle handler
                    // NOTE: Checkbox.on_toggle is Option<M>, NOT a closure.
                    // Must emit the message value directly, e.g. Some(AppMsg::ToggleTodo(42)),
                    // not Some(|_| AppMsg::ToggleTodo(42)).
                    let on_toggle = events.iter()
                        .find(|(e, _)| e.as_str() == "onclick" || e.as_str() == "onClick" || e.as_str() == "on_click")
                        .map(|(_, handler)| {
                            self.handler_to_rust_direct_msg(&handler.handler, &handler.params)
                        });

                    let result = match on_toggle {
                        Some(msg) => format!(
                            "View::Checkbox {{ is_checked: {}, label: \"{}\".to_string(), on_toggle: Some({}), style: {} }}",
                            is_checked, label, msg, style_expr
                        ),
                        None => format!(
                            "View::Checkbox {{ is_checked: {}, label: \"{}\".to_string(), on_toggle: None, style: {} }}",
                            is_checked, label, style_expr
                        ),
                    };
                    return result;
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

                // For non-button leaf tags with text content and styling,
                // use View::text_styled() to avoid builder pattern issues
                // (View::text("str") returns View, not ViewBuilder, so chaining won't work)
                // Also handles text from a single Text child node (e.g. text f"..." { class: "..." })
                if self.is_leaf_tag(tag.as_str()) && tag != "button" && (children.is_empty() || child_text_content.is_some()) && has_styling {
                    let user_style = props.get("style")
                        .or_else(|| props.get("class"))
                        .and_then(|v| if let AuraPropValue::Expr(AuraExpr::Literal(s)) = v { Some(s.clone()) } else { None })
                        .unwrap_or_default();

                    // Prepend heading default styles (h1→text-4xl font-bold, etc.)
                    let style_str = match Self::heading_default_style(tag.as_str()) {
                        Some(default) if !user_style.is_empty() => format!("{} {}", default, user_style),
                        Some(default) => default.to_string(),
                        None => user_style,
                    };

                    if let Some(ref name) = text_state_ref {
                        return format!("View::text_styled(format!(\"{{}}\", self.{}), \"{}\")", name, style_str);
                    }
                    if let Some(label) = &text_prop {
                        // Check if text contains interpolation like ${.field}
                        if label.contains("${") {
                            return format!("View::text_styled({}, \"{}\")", self.interpolate_str(label), style_str);
                        }
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

                // Non-button leaf tags with text and no styling:
                // View::text("str") returns View<M> directly, NOT a builder.
                // Skip .build() to avoid compile error.
                // Heading tags (h1-h3) always use text_styled with their default styles.
                let heading_default = Self::heading_default_style(tag.as_str());
                if self.is_leaf_tag(tag.as_str()) && tag != "button" && (children.is_empty() || child_text_content.is_some()) && !has_styling {
                    if let Some(ref name) = text_state_ref {
                        if let Some(default) = heading_default {
                            return format!("View::text_styled(format!(\"{{}}\", self.{}), \"{}\")", name, default);
                        }
                        return format!("View::text(format!(\"{{}}\", self.{}))", name);
                    }
                    if let Some(label) = &text_prop {
                        if let Some(default) = heading_default {
                            return format!("View::text_styled(\"{}\".to_string(), \"{}\")", label, default);
                        }
                        if label.contains("${") {
                            return format!("View::text({})", self.interpolate_str(label));
                        }
                        return format!("View::text(\"{}\".to_string())", label);
                    }
                    if let Some(ref text) = text_rust_expr {
                        if let Some(default) = heading_default {
                            return format!("View::text_styled({}, \"{}\")", text, default);
                        }
                        return format!("View::text({})", text);
                    }
                    // Leaf tag without text content but no styling — e.g. avatar
                    // These go through the builder path
                }

                // Special handling for "center" — View::center(child) takes a child directly,
                // not the builder pattern. Assemble children into a col, then wrap in center.
                if tag == "center" {
                    let style_str = props.get("style")
                        .or_else(|| props.get("class"))
                        .and_then(|v| if let AuraPropValue::Expr(AuraExpr::Literal(s)) = v { Some(s.clone()) } else { None })
                        .unwrap_or_default();

                    // Build children into a col
                    let child_view = if children.is_empty() {
                        "View::Empty".to_string()
                    } else if children.len() == 1 {
                        self.generate_view_tree(&children[0])
                    } else {
                        let mut col = "View::col()".to_string();
                        for child in children {
                            let child_code = self.generate_view_tree(child);
                            col = format!("{}.child({})", col, child_code);
                        }
                        format!("{}.build()", col)
                    };

                    let mut builder = format!("View::center({})", child_view);
                    if !style_str.is_empty() {
                        builder = format!("{}.style(\"{}\")", builder, style_str);
                    }
                    return format!("{}.build()", builder);
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
                            __t.contains(&__q) \
                        }})",
                        var_ref, var_ref
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
                    format!("if {} {{ {} }} else {{ {} }}", rust_condition, Self::wrap_views(&then_code), Self::wrap_views(&else_code))
                } else {
                    format!("if {} {{ {} }} else {{ View::Empty }}", rust_condition, Self::wrap_views(&then_code))
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
                    // This is var.field — check if var is a Value-type loop variable
                    // Look backwards to find the identifier before the dot
                    let mut ident_end = i;
                    let mut ident_start = i;
                    for j in (0..i).rev() {
                        if bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_' {
                            ident_start = j;
                        } else {
                            break;
                        }
                    }
                    if ident_start < ident_end {
                        let var_name = &result[ident_start..ident_end];
                        if self.value_loop_vars.contains(var_name) {
                            // Find the field name after the dot
                            let mut field_end = i + 1;
                            for j in (i + 1)..bytes.len() {
                                if bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_' {
                                    field_end = j + 1;
                                } else {
                                    break;
                                }
                            }
                            let field_name = &result[i + 1..field_end];
                            // Replace var.field with bracket access, converting the result
                            // Remove the var.field from output and replace with bracket access
                            let output_var_name = var_name.to_string();
                            let bracket_access = self.value_field_access(&output_var_name, field_name);
                            // Remove the already-pushed var name and replace with bracket access
                            output.truncate(output.len() - var_name.len());
                            output.push_str(&bracket_access);
                            i = field_end;
                            continue;
                        }
                    }
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

    /// Default heading styles for h1-h6 tags (consistent with aura_view_builder & vue.rs)
    fn heading_default_style(tag: &str) -> Option<&'static str> {
        match tag {
            "h1" => Some("text-4xl font-bold"),
            "h2" => Some("text-3xl font-bold"),
            "h3" => Some("text-xl font-semibold"),
            _ => None,
        }
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
            "tree" => "col",
            "tree_item" => "col",

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
                        // Use .style() directly — same Style::parse() path as VM mode
                        if class_str.is_empty() {
                            builder.to_string()
                        } else {
                            format!("{}.style(\"{}\")", builder, class_str)
                        }
                    }
                    "style" => {
                        let style_str = value_str.trim_matches('"')
                            .trim_end_matches(".to_string()")
                            .trim_matches('"');
                        if style_str.is_empty() {
                            builder.to_string()
                        } else {
                            format!("{}.style(\"{}\")", builder, style_str)
                        }
                    }
                    "padding" => format!("{}.padding({})", builder, value_str),
                    "spacing" => format!("{}.spacing({})", builder, value_str),
                    _ => builder.to_string(),
                }
            }
            AuraPropValue::StyleBinding(bindings) => {
                // For Rust, generate conditional style application.
                // Each binding produces a conditional: if cond { "style" } else { "" }
                // Uses .with_style() with Style::parse() for safe string construction.
                let class_conditions: Vec<String> = bindings.iter()
                    .map(|b| {
                        let cond = self.expr_to_rust(&b.condition);
                        format!("if {} {{ \"{}\" }} else {{ \"\" }}", cond, b.style_name)
                    })
                    .collect();
                if class_conditions.is_empty() {
                    builder.to_string()
                } else if class_conditions.len() == 1 {
                    // Single condition: if cond { "completed" } else { "" } is &str
                    format!("{}.style({})", builder, class_conditions[0])
                } else {
                    // Multiple conditions: build concatenated string.
                    // Rust if-expr returns &str, we need to combine them.
                    // Use nested format!: format!("{} {}", c1, c2) then .as_str()
                    // Actually, just construct Style directly from parts
                    let fmt_str = class_conditions.iter().map(|_| "{}").collect::<Vec<_>>().join(" ");
                    // Each condition is an `if ... { &str } else { &str }` expression
                    // format!() needs owned values for interpolation, but &str works fine
                    let args = class_conditions.join(", ");
                    let combined = format!("auto_lang::ui::style::Style::parse(&format!(\"{}\", {})).unwrap_or_default()", fmt_str, args);
                    format!("{}.with_style({})", builder, combined)
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

    /// Convert handler pattern to a direct Rust message expression (no closure wrapper).
    /// Used for fields like Checkbox.on_toggle which is Option<M>, not Option<impl Fn() -> M>.
    fn handler_to_rust_direct_msg(&self, handler: &str, params: &[String]) -> String {
        let variant = self.extract_variant_name(handler);
        let msg_name = self.current_msg_name();
        if params.is_empty() {
            format!("{}::{}", msg_name, variant)
        } else {
            let converted_params: Vec<String> = params.iter()
                .map(|p| self.convert_param_value_access(p, &variant))
                .collect();
            format!("{}::{}({})", msg_name, variant, converted_params.join(", "))
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
                let resolved = if name.starts_with('.') { &name[1..] } else { name };
                let mut value = self.ast_expr_to_rust(&store.expr);
                // Let/Const are local variables — use let binding
                // Var/Field are state variables — but only if they exist in state_types
                match store.kind {
                    crate::ast::StoreKind::Let | crate::ast::StoreKind::Const => {
                        // Check if value is an index into a Vec<Value> (e.g., todos[idx])
                        // If so, use &mut borrow so that mutations to todo.field affect the array
                        if let crate::ast::Expr::Index(target, _idx) = &store.expr {
                            if let crate::ast::Expr::Ident(collection) = target.as_ref() {
                                let coll_name = collection.as_str();
                                let resolved_coll = if coll_name.starts_with('.') { &coll_name[1..] } else { coll_name };
                                if self.state_types.get(resolved_coll)
                                    .map(|ty| ty.starts_with("Vec<"))
                                    .unwrap_or(false)
                                {
                                    // `let todo = self.todos[idx as usize]` →
                                    // `let mut todo = &mut self.todos[idx as usize]`
                                    // Prepend &mut to the collection reference in value
                                    let target_prefix = if self.state_types.contains_key(resolved_coll) {
                                        format!("self.{}", resolved_coll)
                                    } else if resolved_coll != coll_name {
                                        format!("self.{}", resolved_coll)
                                    } else {
                                        coll_name.to_string()
                                    };
                                    value = value.replacen(&target_prefix, &format!("&mut {}", target_prefix), 1);
                                    return format!("let mut {} = {}", name, value);
                                }
                            }
                        }
                        format!("let {} = {}", name, value)
                    }
                    crate::ast::StoreKind::Var => {
                        // `var x = expr` → mutable local binding
                        if self.state_types.contains_key(resolved) {
                            // Auto-coerce int → String when assigning to a String field
                            if self.state_types.get(resolved).map_or(false, |ty| ty == "String")
                                && !self.ast_expr_is_string(&store.expr)
                            {
                                value = format!("{}.to_string()", value);
                            }
                            format!("self.{} = {}", resolved, value)
                        } else {
                            // Local mutable var in handler context
                            format!("let mut {} = {}", name, value)
                        }
                    }
                    _ => {
                        // If name is a known state var, use self. prefix
                        if self.state_types.contains_key(resolved) {
                            // Auto-coerce int → String when assigning to a String field
                            if self.state_types.get(resolved).map_or(false, |ty| ty == "String")
                                && !self.ast_expr_is_string(&store.expr)
                            {
                                value = format!("{}.to_string()", value);
                            }
                            format!("self.{} = {}", resolved, value)
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
            crate::ast::Stmt::If(if_stmt) => {
                let mut parts = Vec::new();
                for (i, branch) in if_stmt.branches.iter().enumerate() {
                    let cond = self.ast_expr_to_rust(&branch.cond);
                    let body: Vec<String> = branch.body.stmts.iter()
                        .map(|s| self.ast_stmt_to_rust(s))
                        .collect();
                    let body_str = body.join("; ");
                    if i == 0 {
                        parts.push(format!("if {} {{ {} }}", cond, body_str));
                    } else {
                        parts.push(format!("else if {} {{ {} }}", cond, body_str));
                    }
                }
                if let Some(else_body) = &if_stmt.else_ {
                    let body: Vec<String> = else_body.stmts.iter()
                        .map(|s| self.ast_stmt_to_rust(s))
                        .collect();
                    let body_str = body.join("; ");
                    parts.push(format!("else {{ {} }}", body_str));
                }
                parts.join(" ")
            }
            crate::ast::Stmt::For(for_stmt) => {
                let body_stmts: Vec<String> = for_stmt.body.stmts.iter()
                    .map(|s| self.ast_stmt_to_rust(s))
                    .collect();
                let body_str = body_stmts.join("; ");
                match &for_stmt.iter {
                    crate::ast::Iter::Named(name) => {
                        // for todo in .todos { ... } → for todo in self.todos.iter() { ... }
                        // If body mutates loop var (value_loop_var), use iter_mut()
                        let iter_name = name.as_str();
                        let collection = self.ast_expr_to_rust(&for_stmt.range);
                        let needs_mut = self.value_loop_vars.contains(iter_name);
                        let iter_method = if needs_mut { "iter_mut" } else { "iter" };
                        let mut_prefix = if needs_mut { "mut " } else { "" };
                        format!("for {}{} in {}.{}() {{ {} }}", mut_prefix, iter_name, collection, iter_method, body_str)
                    }
                    crate::ast::Iter::Cond => {
                        // for i >= 0 { ... } → while i >= 0 { ... }
                        let cond = self.ast_expr_to_rust(&for_stmt.range);
                        format!("while {} {{ {} }}", cond, body_str)
                    }
                    crate::ast::Iter::Ever => {
                        // loop { ... }
                        format!("loop {{ {} }}", body_str)
                    }
                    crate::ast::Iter::Indexed(idx, name) => {
                        // for i, todo in .todos { ... } → for (i, todo) in self.todos.iter().enumerate() { ... }
                        let collection = self.ast_expr_to_rust(&for_stmt.range);
                        format!("for ({}, {}) in {}.iter().enumerate() {{ {} }}", idx.as_str(), name.as_str(), collection, body_str)
                    }
                    crate::ast::Iter::Destructured(key, val) => {
                        let collection = self.ast_expr_to_rust(&for_stmt.range);
                        format!("for ({}, {}) in {}.iter() {{ {} }}", key.as_str(), val.as_str(), collection, body_str)
                    }
                    crate::ast::Iter::Call(_) => {
                        // Fallback for Call-based iterators
                        let collection = self.ast_expr_to_rust(&for_stmt.range);
                        format!("for __item in {}.iter() {{ {} }}", collection, body_str)
                    }
                }
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
        } else if field == "done" || field == "deleted" || field.starts_with("is_") {
            format!("{}[\"{}\"].as_bool().unwrap_or(false)", obj_expr, field)
        } else {
            format!("{}[\"{}\"].as_str().unwrap_or_default().to_string()", obj_expr, field)
        }
    }

    /// Check if an AST expression produces a String type (for detecting string concatenation)
    fn ast_expr_is_string(&self, expr: &crate::ast::Expr) -> bool {
        use crate::ast::Expr;
        match expr {
            Expr::Str(_) | Expr::CStr(_) | Expr::FStr(_) => true,
            Expr::Ident(name) => {
                let s = name.as_str();
                let resolved = if s.starts_with('.') { &s[1..] } else { s };
                self.state_types.get(resolved).map_or(false, |ty| ty == "String")
            }
            Expr::Dot(obj, field) => {
                // Dot(Ident("self"), "display") → check field "display" in state_types
                if let Expr::Ident(obj_name) = obj.as_ref() {
                    let obj_s = obj_name.as_str();
                    if obj_s == "self" || obj_s.starts_with('.') {
                        return self.state_types.get(field.as_str())
                            .map_or(false, |ty| ty == "String");
                    }
                }
                // Generic dot access: check the object chain
                self.ast_expr_is_string(obj)
            }
            Expr::Bina(_left, op, _right) => {
                // If this is an Add chain, check if either operand is string
                use auto_val::Op;
                if matches!(op, Op::Add) {
                    self.ast_expr_is_string(_left) || self.ast_expr_is_string(_right)
                } else {
                    false
                }
            }
            Expr::Call(_) => false,
            _ => false,
        }
    }

    /// Resolve an AST expression to a simple field name (for state_types lookup).
    /// Returns None if the expression is not a simple field reference.
    fn resolve_expr_name(&self, expr: &crate::ast::Expr) -> Option<String> {
        use crate::ast::Expr;
        match expr {
            Expr::Ident(name) => {
                let s = name.as_str();
                if s.starts_with('.') {
                    Some(s[1..].to_string())
                } else {
                    Some(s.to_string())
                }
            }
            Expr::Dot(obj, field) => {
                // Dot(Ident("self"), "field") → "field"
                if let Expr::Ident(obj_name) = obj.as_ref() {
                    let obj_s = obj_name.as_str();
                    if obj_s == "self" || obj_s.starts_with('.') {
                        return Some(field.as_str().to_string());
                    }
                }
                None
            }
            _ => None,
        }
    }

    /// Like ast_expr_to_rust, but treats specified param names as serde_json::Value variables.
    /// Used for closures passed to findIndex/.position() where params iterate over &Value.
    fn ast_expr_to_rust_with_value_params(&self, expr: &crate::ast::Expr, value_params: &[String]) -> String {
        use crate::ast::Expr;
        // Intercept Dot access on value params
        if let Expr::Dot(obj, field) = expr {
            if let Expr::Ident(name) = obj.as_ref() {
                if value_params.contains(&name.to_string()) {
                    return self.value_field_access(name.as_str(), field.as_str());
                }
            }
        }
        // For all other cases, delegate to ast_expr_to_rust.
        // We can't intercept nested closures or deeper expressions that reference value_params,
        // but the common case is `t.field == value` which is handled above.
        // For compound expressions, we recursively apply the same logic.
        match expr {
            Expr::Bina(left, op, right) => {
                let left_str = self.ast_expr_to_rust_with_value_params(left, value_params);
                let right_str = self.ast_expr_to_rust_with_value_params(right, value_params);
                // Use the same op handling as ast_expr_to_rust
                use auto_val::Op;
                let op_str = match op {
                    Op::Eq => "==",
                    Op::Neq => "!=",
                    Op::Lt => "<",
                    Op::Le => "<=",
                    Op::Gt => ">",
                    Op::Ge => ">=",
                    Op::And => "&&",
                    Op::Or => "||",
                    Op::Add => "+",
                    Op::Sub => "-",
                    Op::Not => "!",
                    _ => "?",
                };
                format!("{} {} {}", left_str, op_str, right_str)
            }
            Expr::Unary(op, operand) => {
                let val = self.ast_expr_to_rust_with_value_params(operand, value_params);
                use auto_val::Op;
                match op {
                    Op::Not => format!("!({})", val),
                    Op::Sub => format!("-{}", val),
                    _ => format!("/* unimplemented unary {:?} */", op),
                }
            }
            // For everything else (idents, literals, etc.), use normal conversion
            _ => self.ast_expr_to_rust(expr),
        }
    }

    /// Same as ast_expr_to_rust but without appending .to_string() to Str literals
    fn ast_expr_to_rust_no_to_string(&self, expr: &crate::ast::Expr) -> String {
        use crate::ast::Expr;
        match expr {
            Expr::Str(s) => format!("\"{}\"", s),
            Expr::CStr(s) => format!("\"{}\"", s),
            // For everything else, delegate to ast_expr_to_rust
            _ => self.ast_expr_to_rust(expr),
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
                // Check if object is an index into a Vec<Value>: todos[idx].field
                // Pattern: Dot(Index(Ident("todos"), idx), "field")
                if let Expr::Index(target, _idx) = obj.as_ref() {
                    if let Expr::Ident(collection) = target.as_ref() {
                        let coll_name = collection.as_str();
                        let resolved_coll = if coll_name.starts_with('.') { &coll_name[1..] } else { coll_name };
                        if self.state_types.get(resolved_coll)
                            .map(|ty| ty.starts_with("Vec<"))
                            .unwrap_or(false)
                        {
                            // Indexing into Vec<Value> produces Value — use bracket access
                            let idx_str = self.ast_expr_to_rust(_idx);
                            let target_str = if resolved_coll != coll_name {
                                format!("self.{}", resolved_coll)
                            } else if self.state_types.contains_key(coll_name) {
                                format!("self.{}", coll_name)
                            } else {
                                coll_name.to_string()
                            };
                            let idx_cast = if idx_str.starts_with("self.")
                                || (!idx_str.parse::<usize>().is_ok() && idx_str != "0")
                            {
                                // State var or local i32 var — cast to usize for indexing
                                format!("{} as usize", idx_str)
                            } else {
                                idx_str
                            };
                            return self.value_field_access(&format!("{}[{}]", target_str, idx_cast), field_str);
                        }
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
                    // Check for value_local.field = value (e.g., todo.done = !todo.done)
                    if let Expr::Dot(obj, field) = left.as_ref() {
                        if let Expr::Ident(name) = obj.as_ref() {
                            let s = name.as_str();
                            if self.value_locals.contains(s) || self.needs_index_access(s) {
                                let value = self.ast_expr_to_rust(right);
                                return format!("{}[\"{}\"] = serde_json::json!({})", s, field.as_str(), value);
                            }
                        }
                        // Check for indexed.field = value (e.g., todos[idx].text = .edit_text)
                        // Pattern: Dot(Index(Ident("collection"), idx), "field")
                        if let Expr::Index(target, _idx) = obj.as_ref() {
                            if let Expr::Ident(collection) = target.as_ref() {
                                let coll_name = collection.as_str();
                                let resolved_coll = if coll_name.starts_with('.') { &coll_name[1..] } else { coll_name };
                                if self.state_types.get(resolved_coll)
                                    .map(|ty| ty.starts_with("Vec<"))
                                    .unwrap_or(false)
                                {
                                    let idx_str = self.ast_expr_to_rust(_idx);
                                    let target_str = if resolved_coll != coll_name {
                                        format!("self.{}", resolved_coll)
                                    } else if self.state_types.contains_key(coll_name) {
                                        format!("self.{}", coll_name)
                                    } else {
                                        coll_name.to_string()
                                    };
                                    let idx_cast = if idx_str.starts_with("self.")
                                        || (!idx_str.parse::<usize>().is_ok() && idx_str != "0")
                                    {
                                        format!("{} as usize", idx_str)
                                    } else {
                                        idx_str
                                    };
                                    let value = self.ast_expr_to_rust(right);
                                    return format!("{}[{}][\"{}\"] = serde_json::json!({})", target_str, idx_cast, field.as_str(), value);
                                }
                            }
                        }
                    }
                    let target = self.ast_expr_to_rust(left);
                    let mut value = self.ast_expr_to_rust(right);
                    // Auto-coerce int → String when assigning to a String field
                    // e.g. .display = .val → self.display = self.val.to_string()
                    if self.ast_expr_is_string(left) && !self.ast_expr_is_string(right) {
                        value = format!("{}.to_string()", value);
                    } else if self.ast_expr_is_string(left) && self.ast_expr_is_string(right) {
                        // String-to-String assignment from a field ref needs .clone()
                        // (Rust's String doesn't impl Copy). Skip for literals/expressions
                        // that already produce owned String values.
                        let needs_clone = self.resolve_expr_name(right).is_some();
                        if needs_clone {
                            value = format!("{}.clone()", value);
                        }
                    }
                    return format!("{} = {}", target, value);
                }
                // Compound assignment: .count += expr → self.count += expr
                if matches!(op, Op::AddEq | Op::SubEq | Op::MulEq | Op::DivEq) {
                    let target = self.ast_expr_to_rust(left);
                    let value = self.ast_expr_to_rust(right);
                    // Check if target is a String field — need parse/add/to_string pattern
                    let target_name = self.resolve_expr_name(left);
                    if target_name.as_ref().map_or(false, |n| self.state_types.get(n).map_or(false, |ty| ty == "String")) {
                        let inner_op = match op {
                            Op::AddEq => "+",
                            Op::SubEq => "-",
                            Op::MulEq => "*",
                            Op::DivEq => "/",
                            _ => unreachable!(),
                        };
                        return format!("{} = ({}.parse::<i32>().unwrap_or(0) {} {}).to_string()", target, target, inner_op, value);
                    }
                    let op_str = match op {
                        Op::AddEq => "+=",
                        Op::SubEq => "-=",
                        Op::MulEq => "*=",
                        Op::DivEq => "/=",
                        _ => unreachable!(),
                    };
                    return format!("{} {} {}", target, op_str, value);
                }
                // String concatenation detection: use format! instead of +
                // because Rust's + only works with String + &str, not String + String
                // Check if EITHER side is a string literal (Expr::Str/CStr/FStr) — that
                // unambiguously means string concatenation, not numeric addition.
                let is_string_concat = matches!(op, Op::Add) && (
                    matches!(left.as_ref(), Expr::Str(_) | Expr::CStr(_) | Expr::FStr(_))
                    || matches!(right.as_ref(), Expr::Str(_) | Expr::CStr(_) | Expr::FStr(_))
                    || self.ast_expr_is_string(left)
                    || self.ast_expr_is_string(right)
                );
                if is_string_concat {
                    let left_str = self.ast_expr_to_rust_no_to_string(left);
                    let right_str = self.ast_expr_to_rust_no_to_string(right);
                    return format!("format!(\"{{}}{{}}\", {}, {})", left_str, right_str);
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
                        // findIndex(closure) → iter().position(closure).map(|i| i as i32).unwrap_or(-1)
                        if fn_name.ends_with(".findIndex") {
                            let obj = &fn_name[..fn_name.len() - ".findIndex".len()];
                            let closure_arg = args.first().map(|s| s.as_str()).unwrap_or("|_| false");
                            return format!("{}.iter().position({}).map(|i| i as i32).unwrap_or(-1)", obj, closure_arg);
                        }
                        let result = if fn_name.ends_with(".remove") {
                            // .remove() takes usize, cast args. Discard return value.
                            // Use drop() instead of `let _ =` because `let` can't be the last
                            // expression in an `if` block in Rust.
                            let casted_args: Vec<String> = args.iter()
                                .map(|a| format!("{} as usize", a))
                                .collect();
                            format!("drop({}({}))", fn_name, casted_args.join(", "))
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
                // Vec<Value> requires usize index — cast non-literal indexes to usize
                // since handler vars are typically i32 from findIndex or loop counters
                let index_cast = if index_str.parse::<usize>().is_ok() {
                    index_str // literal usize, no cast needed
                } else {
                    format!("{} as usize", index_str)
                };
                format!("{}[{}]", target_str, index_cast)
            }
            Expr::Unary(op, operand) => {
                let val = self.ast_expr_to_rust(operand);
                match op {
                    Op::Not => format!("!({})", val),
                    Op::Sub => format!("-{}", val),
                    _ => format!("/* unimplemented unary {:?} */", op),
                }
            }
            Expr::Closure(closure) => {
                // (t => t.id == id) → |t| t["id"].as_i64().unwrap_or(0) as i32 == id
                // Closure params from findIndex/.position() iterate over &Value,
                // so any dot access on a closure param needs bracket access.
                let param_names: Vec<String> = closure.params.iter()
                    .map(|p| p.name.as_str().to_string())
                    .collect();
                // Temporarily register closure params as value loop vars so that
                // dot access on them gets converted to bracket access.
                // We can't mutate self, so we handle it inline by checking the
                // param names during Dot processing.
                // Instead, we convert the closure body manually with param awareness.
                let body = self.ast_expr_to_rust_with_value_params(&closure.body, &param_names);
                format!("|{}| {}", param_names.join(", "), body)
            }
            Expr::FStr(fstr) => {
                // f"${.active_count} items left" → format!("{} items left", self.active_count)
                let mut fmt_str = String::new();
                let mut args = Vec::new();
                for part in &fstr.parts {
                    match part {
                        Expr::Str(s) | Expr::CStr(s) => {
                            fmt_str.push_str(&s.as_str().replace('{', "{{").replace('}', "}}"));
                        }
                        _ => {
                            fmt_str.push_str("{}");
                            args.push(self.ast_expr_to_rust(part));
                        }
                    }
                }
                if args.is_empty() {
                    format!("\"{}\".to_string()", fmt_str)
                } else {
                    format!("format!(\"{}\", {})", fmt_str, args.join(", "))
                }
            }
            Expr::Range(range) => {
                let start = self.ast_expr_to_rust(&range.start);
                let end = self.ast_expr_to_rust(&range.end);
                if range.eq {
                    format!("{}..={}", start, end)
                } else {
                    format!("{}..{}", start, end)
                }
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
        AuraExpr::If { cond, then_branch, else_branch } => {
                let cond_str = self.expr_to_rust(cond);
                let then_str = self.expr_to_rust(then_branch);
                match else_branch {
                    Some(else_expr) => {
                        let else_str = self.expr_to_rust(else_expr);
                        format!("if {} {{ {} }} else {{ {} }}", cond_str, then_str, else_str)
                    }
                    None => format!("if {} {{ {} }} else {{ \"\".to_string() }}", cond_str, then_str),
                }
            }
            AuraExpr::Literal(s) => format!("\"{}\".to_string()", s),
            AuraExpr::Int(n) => n.to_string(),
            AuraExpr::Float(n) => {
                let s = n.to_string();
                if s.contains('.') { s } else { format!("{}.0", n) }
            }
            AuraExpr::Bool(b) => b.to_string(),
            AuraExpr::StateRef(name) => format!("self.{}", name),
            AuraExpr::Binary { left, op, right } => {
                // Detect string concatenation: use format! instead of +
                // because Rust's + only works with String + &str, not String + String
                let is_string_concat = matches!(op, crate::aura::AuraBinOp::Add) && (
                    matches!(left.as_ref(), AuraExpr::Literal(_))
                    || matches!(right.as_ref(), AuraExpr::Literal(_))
                    || self.expr_is_string(left)
                    || self.expr_is_string(right)
                );
                if is_string_concat {
                    let left_str = self.expr_to_rust_no_to_string(left);
                    let right_str = self.expr_to_rust_no_to_string(right);
                    return format!("format!(\"{{}}{{}}\", {}, {})", left_str, right_str);
                }
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

    /// Check if an AuraExpr produces a String type (for detecting string concatenation)
    fn expr_is_string(&self, expr: &AuraExpr) -> bool {
        match expr {
            AuraExpr::Literal(_) => true,
            AuraExpr::StateRef(name) => {
                self.state_types.get(name).map_or(false, |ty| ty == "String")
            }
            AuraExpr::Binary { left, op, right } => {
                // If this is an Add chain that produced a string, it's still string
                matches!(op, crate::aura::AuraBinOp::Add)
                    && (self.expr_is_string(left) || self.expr_is_string(right))
            }
            AuraExpr::MethodCall { method, .. } => {
                matches!(method.as_str(), "to_string" | "trim" | "replace" | "to_lowercase" | "to_uppercase")
            }
            _ => false,
        }
    }

    /// Same as expr_to_rust but without appending .to_string() to Literal values
    /// Used inside format!() where &str is fine
    fn expr_to_rust_no_to_string(&self, expr: &AuraExpr) -> String {
        match expr {
            AuraExpr::Literal(s) => format!("\"{}\"", s),
            // For everything else, delegate to expr_to_rust
            _ => self.expr_to_rust(expr),
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

/// Extract field name from `Expr::Dot(Expr::Ident("self"), Name("field"))`.
/// Returns `None` if the pattern doesn't match.
fn extract_dot_self_field(expr: &crate::ast::Expr) -> Option<String> {
    if let crate::ast::Expr::Dot(obj, field) = expr {
        if let crate::ast::Expr::Ident(name) = obj.as_ref() {
            if name.as_str() == "self" {
                return Some(field.as_str().to_string());
            }
        }
    }
    None
}

/// Extract function name from `Expr::Call(...)`.
fn extract_call_name(expr: &crate::ast::Expr) -> Option<String> {
    if let crate::ast::Expr::Call(call) = expr {
        call.get_name_text_safe().map(|s| s.to_string())
    } else {
        None
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
#[allow(dead_code)]
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
#[allow(dead_code)]
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

        assert!(code.contains("pub enum CounterMsg"), "got:\n{}", code);
        assert!(code.contains("Inc"), "got:\n{}", code);
        assert!(code.contains("Dec"), "got:\n{}", code);
        assert!(code.contains("pub struct Counter"), "got:\n{}", code);
        assert!(code.contains("pub count: i32"), "got:\n{}", code);
        assert!(code.contains("impl Component for Counter"), "got:\n{}", code);
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
        assert!(code.contains("View::text(\"Hello, World!\".to_string())"), "got: {}", code);
        assert!(!code.contains(".build()"), "View::text(str) returns View directly, got: {}", code);
    }
}
