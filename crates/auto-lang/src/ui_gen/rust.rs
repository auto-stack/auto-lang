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
use crate::aura::{AuraEvent, AuraMsgVariant, AuraNode, AuraPropValue, AuraTextContent, AuraWidget, LogicPayload};

/// Plan 371 L1: Semantic info about a child component, collected via
/// cross-file pre-scan. Used to replace hardcoded special-case logic (Init
/// forwarding, prop writeback) with .at-source-driven general code.
#[derive(Debug, Clone, Default)]
pub struct ComponentSemantics {
    /// Prop names that are WRITTEN by the component's handlers (e.g. EditorPanel
    /// `.Save -> { .note.title = .edit_title }` → writes "note").
    /// Drives prop-writeback generation in the parent's child-forwarding arm.
    pub written_props: Vec<String>,
}

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

    /// Plan 371 L3: child component names that appear INSIDE a for-loop in the
    /// view tree. These CANNOT be persistent fields (multiple instances), so
    /// they keep the old temp-construction pattern. Single-instance children
    /// (not in this set) become persistent struct fields.
    loop_child_components: std::collections::HashSet<String>,

    /// Plan 371 Task 22c: map of component name -> its own scalar state fields
    /// (name, rust_type), collected across all widgets parsed in the same
    /// compile unit, so a parent using a child can hoist+sync its state.
    component_state_fields: std::collections::HashMap<String, Vec<(String, String)>>,

    /// Plan 371 L1: map of component name -> its semantics (written props),
    /// collected cross-file. Drives general prop-writeback generation,
    /// replacing the old "constructor_args.contains(\"note\")" heuristic.
    component_semantics: std::collections::HashMap<String, ComponentSemantics>,

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

    /// Prop names whose type is a user-defined type alias (e.g., Note).
    /// These need serde_json::Value bracket access (self.note["field"]).
    value_prop_names: std::collections::HashSet<String>,

    /// Computed property method names (for adding () in dot access)
    computed_names: std::collections::HashSet<String>,

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

// Plan 346: Thread-local store for the root widget's state field names + types.
// Populated when the root widget (App) is generated; read by child widgets
// (EditorPanel, etc.) so their structs include parent state fields for unified
// state access (mirrors VM path's Plan 320 override_state_obj_id).
thread_local! {
    static ROOT_STATE_FIELDS: std::cell::RefCell<Vec<(String, String)>> =
        std::cell::RefCell::new(Vec::new());
    /// Plan 374: store composable names (e.g. {"store" => "NotesStore"}).
    pub static STORE_NAMES: std::cell::RefCell<std::collections::HashMap<String, String>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
    /// Plan 374: store computed property names (for adding () in dot access).
    pub static STORE_COMPUTED_NAMES: std::cell::RefCell<std::collections::HashSet<String>> =
        std::cell::RefCell::new(std::collections::HashSet::new());
    /// Plan 374: widget prop declaration order (widget_name → ordered prop names).
    /// Used to ensure constructor args are emitted in the correct order.
    pub static WIDGET_PROP_ORDERS: std::cell::RefCell<std::collections::HashMap<String, Vec<String>>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

/// Detected Init handler pattern: `self.state_var = api_func()`
struct InitApiInfo {
    /// State variable being assigned (e.g., "notes")
    state_var: String,
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
            loop_child_components: std::collections::HashSet::new(),
            component_state_fields: std::collections::HashMap::new(),
            component_semantics: std::collections::HashMap::new(),
            loop_vars: Vec::new(),
            input_fields: std::collections::HashMap::new(),
            state_types: std::collections::HashMap::new(),
            prop_names: std::collections::HashSet::new(),
            prop_types: std::collections::HashMap::new(),
            value_prop_names: std::collections::HashSet::new(),
            computed_names: std::collections::HashSet::new(),
            value_loop_vars: std::collections::HashSet::new(),
            value_locals: std::collections::HashSet::new(),
            has_init: false,
            init_api_info: None,
        }
    }

    /// Plan 374: Register a store composable name.
    pub fn register_store(&mut self, alias: &str, store_name: &str) {
        STORE_NAMES.with(|sn| {
            sn.borrow_mut().insert(alias.to_string(), store_name.to_string());
        });
    }

    /// Plan 371 Task 22c: record a component's own scalar state fields so
    /// parents that use this component can hoist+sync them.
    pub fn register_component_state(
        &mut self,
        component_name: &str,
        fields: Vec<(String, String)>,
    ) {
        self.component_state_fields
            .insert(component_name.to_string(), fields);
    }

    /// Plan 371 L1: record a component's semantics (which props its handlers
    /// write), so the parent's child-forwarding arm can generate general
    /// prop-writeback instead of hardcoding "note".
    pub fn register_component_semantics(
        &mut self,
        component_name: &str,
        semantics: ComponentSemantics,
    ) {
        self.component_semantics
            .insert(component_name.to_string(), semantics);
    }

    /// Reset state for new widget
    fn reset(&mut self) {
        self.message_variants.clear();
        self.input_fields.clear();
        self.state_types.clear();
        self.prop_names.clear();
        self.prop_types.clear();
        self.value_prop_names.clear();
        self.computed_names.clear();
        self.value_loop_vars.clear();
        self.value_locals.clear();
        self.child_components.clear();
        self.loop_child_components.clear();
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
    fn expr_accesses_field(&self, expr: &crate::ast::Expr, prop_name: &str) -> bool {
        use crate::ast::Expr;
        match expr {
            Expr::Dot(object, _field) => {
                if let Expr::Ident(name) = object.as_ref() {
                    let resolved = if name.starts_with('.') { &name[1..] } else { name.as_str() };
                    if resolved == prop_name {
                        return true;
                    }
                }
                self.expr_accesses_field(object, prop_name)
            }
            Expr::Bina(left, _op, right) => {
                self.expr_accesses_field(left, prop_name) || self.expr_accesses_field(right, prop_name)
            }
            Expr::Call(call) => {
                self.expr_accesses_field(&call.name, prop_name)
                    || call.args.args.iter().any(|a| {
                        if let crate::ast::Arg::Pos(e) | crate::ast::Arg::Pair(_, e) = a {
                            self.expr_accesses_field(e, prop_name)
                        } else {
                            false
                        }
                    })
            }
            Expr::Index(target, index) => {
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

    /// Check if a dot access target needs index syntax (target["field"] instead of target.field)
    fn needs_index_access(&self, target_name: &str) -> bool {
        // Props that are actually serde_json::Value type
        if let Some(ty) = self.prop_types.get(target_name) {
            if ty == "serde_json::Value" {
                return true;
            }
        }
        // Plan 374: User-defined type props (like Note) need Value bracket access
        if self.value_prop_names.contains(target_name) {
            return true;
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
                    crate::ast::Expr::Array(_) => "Vec<serde_json::Value>".to_string(),
                    crate::ast::Expr::Object(_) => "serde_json::Value".to_string(),
                    crate::ast::Expr::Str(_) => "String".to_string(),
                    crate::ast::Expr::Int(_) => "i32".to_string(),
                    crate::ast::Expr::Float(_, _) | crate::ast::Expr::Double(_, _) => "f64".to_string(),
                    crate::ast::Expr::Bool(_) => "bool".to_string(),
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
            self.prop_types.insert(prop.name.clone(), prop_ty.clone());

            // Plan 374: Track user-defined type props (like `Note`) that need
            // serde_json::Value bracket access (self.prop["field"]).
            if matches!(&prop.type_info, crate::ast::Type::User(_)) {
                self.value_prop_names.insert(prop.name.clone());
            }
        }

        // Collect all message variants
        for msg in &widget.messages {
            for variant in &msg.variants {
                self.message_variants.push(variant.clone());
            }
        }

        // Plan 374: Collect computed property names for method-call syntax
        for computed in &widget.computed {
            self.computed_names.insert(computed.name.clone());
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
                    payload: vec![],
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
                    payload: vec![],
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
            if !variant.payload.is_empty() {
                // Plan 043 M5 #1: emit each payload type as a tuple field, so
                // `Complete(str, int)` → `Complete(String, i32)` and a single
                // `Set(int)` → `Set(i32)`.
                let ty_strs: Vec<String> = variant.payload.iter()
                    .map(|t| self.auto_type_to_rust(t))
                    .collect();
                code.push_str(&format!("    {}({}),\n", variant.name, ty_strs.join(", ")));
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

    fn generate_struct(&self, widget: &AuraWidget) -> String {
        let mut code = String::new();

        // Plan 371 Task 22c / L3: stores + persistent child components need Clone.
        let is_store_itself = STORE_NAMES.with(|sn| {
            sn.borrow().values().any(|s| s.as_str() == widget.name)
        });
        let has_store_field = STORE_NAMES.with(|sn| !sn.borrow().is_empty()) && !is_store_itself;
        let has_persistent_child = self.child_components.iter()
            .any(|c| self.is_persistent_child(c));
        if is_store_itself || has_store_field || has_persistent_child {
            code.push_str("#[derive(Clone, Debug)]\n");
        } else {
            code.push_str("#[derive(Debug)]\n");
        }
        code.push_str(&format!("pub struct {} {{\n", widget.name));

        // Track this widget's own fields (for dedup with root state fields).
        let mut own_fields: std::collections::HashSet<String> = std::collections::HashSet::new();

        // Props (from widget signature, e.g., EditorPanel's `note` parameter)
        // Plan 374: Skip `msg`-typed callback props — they use VM parent-to-child
        // message passing which is replaced by Rust's child-to-parent enum forwarding.
        for prop in &widget.props {
            let field_type = self.prop_types.get(&prop.name)
                .cloned()
                .unwrap_or_else(|| self.prop_rust_type(prop));
            if field_type == "msg" {
                continue; // Skip callback props in Rust output
            }
            code.push_str(&format!("    pub {}: {},\n", prop.name, field_type));
            own_fields.insert(prop.name.clone());
        }

        // Plan 374: Register widget prop declaration order for child component
        // constructor arg ordering.
        {
            let ordered: Vec<String> = widget.props.iter()
                .map(|p| p.name.clone())
                .collect();
            WIDGET_PROP_ORDERS.with(|po| {
                po.borrow_mut().insert(widget.name.clone(), ordered);
            });
        }

        // State variables (use refined types from state_types)
        for state in &widget.state_vars {
            let field_name = &state.name;
            let field_type = self.state_rust_type(state);
            code.push_str(&format!("    pub {}: {},\n", field_name, field_type));
            own_fields.insert(field_name.clone());
        }

        // Plan 371 Task 22c / L3: child component state.
        // - Persistent children (single-instance, not in for-loop): add a persistent
        //   instance field (e.g. `pub editor_panel: EditorPanel`). Their scalar
        //   state lives inside the instance, so we DON'T hoist those fields.
        // - Loop children: hoist their scalar state fields (old behavior).
        for child_name in &self.child_components {
            if self.is_persistent_child(child_name) {
                // L3: persistent instance field.
                let field = Self::child_field_name(child_name);
                if !own_fields.contains(&field) {
                    code.push_str(&format!("    pub {}: {},\n", field, child_name));
                    own_fields.insert(field);
                }
            } else {
                // Loop child: hoist scalar state fields (legacy behavior).
                if let Some(child_fields) = self.component_state_fields.get(child_name) {
                    for (f, ty) in child_fields {
                        if !own_fields.contains(f) {
                            code.push_str(&format!("    pub {}: {},\n", f, ty));
                            own_fields.insert(f.clone());
                        }
                    }
                }
            }
        }

        // Plan 346: If this is a child widget (not App/root), add root state
        // fields that it doesn't already have. This mirrors VM path's Plan 320
        // unified state — child handlers can reference parent state fields.
        let is_root = widget.name == "App";
        // Plan 374 Task 2: for root widget AND child widgets that reference store,
        // add `pub store: StoreName` field.
        // Plan 374: Skip store field injection for the store struct itself
        // to avoid recursive types (NotesStore { store: NotesStore }).
        let is_store_itself = STORE_NAMES.with(|sn| {
            sn.borrow().values().any(|s| s.as_str() == widget.name)
        });
        if !is_store_itself {
            if is_root {
            STORE_NAMES.with(|sn| {
                for (_alias, store_name) in sn.borrow().iter() {
                    code.push_str(&format!("    pub store: {},\n", store_name));
                }
            });
        } else {
            // Child widgets also need store field (they access self.store.X)
            STORE_NAMES.with(|sn| {
                for (_alias, store_name) in sn.borrow().iter() {
                    if !own_fields.contains("store") {
                        code.push_str(&format!("    pub store: {},\n", store_name));
                    }
                }
            });
        }
        } // !is_store_itself — skip store field injection for store structs
        if is_root {
            // Record root state fields for child widgets to pick up.
            ROOT_STATE_FIELDS.with(|rsf| {
                let mut fields = rsf.borrow_mut();
                fields.clear();
                for state in &widget.state_vars {
                    let ty = self.state_rust_type(state);
                    fields.push((state.name.clone(), ty));
                }
            });
        } else {
            // Add root state fields not already in own_fields.
            ROOT_STATE_FIELDS.with(|rsf| {
                let fields = rsf.borrow();
                for (name, ty) in fields.iter() {
                    if !own_fields.contains(name) {
                        code.push_str(&format!("    pub {}: {},\n", name, ty));
                    }
                }
            });
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
        // Plan 374: Skip `msg`-typed callback props in constructor.
        let non_msg_props: Vec<&crate::aura::AuraProp> = widget.props.iter()
            .filter(|p| {
                let ty = self.prop_types.get(&p.name)
                    .cloned()
                    .unwrap_or_else(|| self.prop_rust_type(p));
                ty != "msg"
            })
            .collect();
        let has_props = !non_msg_props.is_empty();
        if has_props {
            let params: Vec<String> = non_msg_props.iter()
                .map(|p| {
                    let ty = self.prop_types.get(&p.name)
                        .cloned()
                        .unwrap_or_else(|| self.prop_rust_type(*p));
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
        // Plan 371 L3: force __self mode if we have persistent children (need
        // post-construct re-initialization with real props).
        let has_persistent_child = self.child_components.iter()
            .any(|c| self.is_persistent_child(c));
        let force_self = sync_init || has_persistent_child;
        if force_self {
            let _msg_name = self.current_msg_name();
            code.push_str("        let mut __self = Self {\n");
        } else {
            code.push_str("        Self {\n");
        }

        // Initialize props from parameters (skip msg-typed callback props)
        for prop in &non_msg_props {
            code.push_str(&format!("            {}: {},\n", prop.name, prop.name));
        }

        // Initialize state vars from their defaults
        for state in &widget.state_vars {
            let init = self.ast_expr_to_rust(&state.initial);
            code.push_str(&format!("            {}: {},\n", state.name, init));
        }

        // Plan 371 Task 22c / L3: initialize child-component state.
        // - Persistent children: initialize with placeholder (Child::new(zero_values)).
        //   Real props are synced after __self construction (see post-construct block).
        // - Loop children: hoist scalar state fields with defaults (legacy).
        {
            let own_names: std::collections::HashSet<String> = widget
                .state_vars
                .iter()
                .map(|s| s.name.clone())
                .collect();
            for child_name in &self.child_components {
                if self.is_persistent_child(child_name) {
                    // L3: persistent instance — placeholder init via Default.
                    // Real props are applied post-construct below.
                    let field = Self::child_field_name(child_name);
                    if !own_names.contains(&field) {
                        code.push_str(&format!("            {}: {}::default(),\n", field, child_name));
                    }
                } else if let Some(child_fields) = self.component_state_fields.get(child_name) {
                    for (f, ty) in child_fields {
                        if own_names.contains(f) {
                            continue;
                        }
                        let default_val = match ty.as_str() {
                            "bool" => "false".to_string(),
                            "i32" | "u32" | "i64" | "u64" => "0".to_string(),
                            "f32" | "f64" => "0.0".to_string(),
                            _ => String::from("\"\".to_string()"),
                        };
                        code.push_str(&format!("            {}: {},\n", f, default_val));
                    }
                }
            }
        }

        // Plan 346: Initialize root state fields (for child widgets) with defaults.
        if widget.name != "App" {
            let own_names: std::collections::HashSet<String> = widget.state_vars.iter()
                .map(|s| s.name.clone())
                .collect();
            let own_props: std::collections::HashSet<String> = widget.props.iter()
                .map(|p| p.name.clone())
                .collect();
            ROOT_STATE_FIELDS.with(|rsf| {
                let fields = rsf.borrow();
                for (name, ty) in fields.iter() {
                    if !own_names.contains(name) && !own_props.contains(name) {
                        // Generate a type-appropriate default value.
                        let default_val = if ty.starts_with("Vec<") {
                            "vec![]".to_string()
                        } else if ty == "serde_json::Value" {
                            "serde_json::Value::Null".to_string()
                        } else if ty == "i32" {
                            "0".to_string()
                        } else if ty == "bool" {
                            "false".to_string()
                        } else {
                            "\"\".to_string()".to_string()
                        };
                        code.push_str(&format!("            {}: {},\n", name, default_val));
                    }
                }
            });
        }

        // Plan 374 Task 2: initialize store field for all widgets (except store itself).
        // Must come BEFORE sync_init check so both paths include it.
        STORE_NAMES.with(|sn| {
            let has_store = !sn.borrow().is_empty();
            let is_self = sn.borrow().values().any(|s| s.as_str() == widget.name);
            if has_store && !is_self {
                let store_name = sn.borrow().values().next().cloned().unwrap_or_default();
                code.push_str(&format!("            store: {}::new(),\n", store_name));
            }
        });

        // Plan 371 L3: close struct literal. If force_self (sync_init or
        // persistent children), use __self pattern and add post-construct code.
        if force_self {
            let msg_name = self.current_msg_name();
            code.push_str(&format!("        }};\n"));
            // Run Init FIRST so data (e.g. store.notes from list_notes()) is
            // loaded before persistent children are re-constructed with that
            // data. Doing the re-construction before Init reads an empty store
            // (e.g. EditorPanel::new(store.notes[0]) index-out-of-bounds),
            // which panics in split/async-load modes where list_notes() yields
            // instead of returning synchronously. Init handlers that only touch
            // the store (not child components) are safe to run first.
            if sync_init {
                code.push_str(&format!("        __self.on({}::Init);\n", msg_name));
            }
            // L3: re-construct persistent children with real props (now that
            // Init has populated any data they depend on).
            //
            // Guard: if a constructor arg indexes a collection (e.g.
            // `__self.store.notes[idx]`), wrap the re-construction in an
            // `if !<collection>.is_empty() { ... }` guard. In split/async-load
            // modes list_notes() yields instead of returning synchronously, so
            // the collection is still empty right after Init returns — indexing
            // it would panic. The guard leaves the child at its Default() until
            // the view layer re-syncs it with loaded data.
            for child_name in &self.child_components {
                if self.is_persistent_child(child_name) {
                    let field = Self::child_field_name(child_name);
                    let constructor_args = self.find_constructor_args_for_child(widget, child_name);
                    // Replace self. with __self. in constructor args (we're in __self context).
                    let args = constructor_args.replace("self.", "__self.");
                    let stmt = format!(
                        "__self.{} = {}::new({});",
                        field, child_name, args
                    );
                    if let Some(collection) = first_indexed_collection(&args) {
                        code.push_str(&format!(
                            "        if !{}.is_empty() {{\n            {}\n        }}\n",
                            collection, stmt
                        ));
                    } else {
                        code.push_str(&format!("        {}\n", stmt));
                    }
                }
            }
            code.push_str("        __self\n");
        } else {
            code.push_str("        }\n");
        }

        code.push_str("    }\n");
        code.push_str("}\n");

        // Default impl — always generated. For widgets with props, use placeholder
        // values (serde_json::Value::Null for custom types) so persistent-child
        // fields can be initialized with Child::default() in the parent's struct literal.
        if !has_props {
            code.push_str(&format!(
                "impl Default for {} {{\n    fn default() -> Self {{ Self::new() }}\n}}\n",
                widget_name
            ));
        } else {
            // Generate Default with placeholder props matching each prop's type.
            let placeholder_args: Vec<String> = non_msg_props.iter()
                .map(|p| {
                    let ty = self.prop_types.get(&p.name)
                        .cloned()
                        .unwrap_or_else(|| self.prop_rust_type(p));
                    match ty.as_str() {
                        "String" => "\"\".to_string()",
                        "i32" | "u32" | "i64" | "u64" => "0",
                        "f32" | "f64" => "0.0",
                        "bool" => "false",
                        _ => "serde_json::Value::Null",
                    }.to_string()
                })
                .collect();
            code.push_str(&format!(
                "impl Default for {} {{\n    fn default() -> Self {{ Self::new({}) }}\n}}\n",
                widget_name, placeholder_args.join(", ")
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

        // Plan 371 Task 21: state_snapshot() override — emit only scalar fields
        // (String/i32/i64/u32/u64/f32/f64/bool). Collections and nested components
        // are skipped. Feeds the rust-mode MCP `autoui_state` tool via SharedState.
        let snapshot = self.generate_state_snapshot(widget);
        if !snapshot.is_empty() {
            code.push('\n');
            code.push_str(&snapshot);
        }

        code.push_str("}\n");

        // Plan 365 W1 follow-up: subscription() moved from Component to
        // ComponentIced (de-ice the core trait). Generate a separate
        // `impl ComponentIced` block when tick_interval is set.
        if let Some(interval_ms) = widget.tick_interval {
            let msg_name = self.current_msg_name();
            let struct_name = &widget.name;
            code.push('\n');
            code.push_str(&format!(
                "impl auto_lang::ui::iced::ComponentIced for {} {{\n    fn subscription(&self) -> iced::Subscription<Self::Msg> {{\n        iced::time::every(std::time::Duration::from_millis({})).map(|_| {}::Tick)\n    }}\n}}\n",
                struct_name, interval_ms, msg_name
            ));
        }

        code
    }

    /// Generate a `state_snapshot()` override covering the scalar fields of
    /// this component (both props and state vars). Returns empty if there are
    /// no scalar fields (then the trait default empty map is used).
    fn generate_state_snapshot(&self, _widget: &AuraWidget) -> String {
        let mut scalars: Vec<(String, String)> = Vec::new();
        for prop in &_widget.props {
            if let Some(ty) = self.prop_types.get(&prop.name) {
                if is_scalar_state_type(ty) {
                    scalars.push((prop.name.clone(), ty.clone()));
                }
            }
        }
        for state in &_widget.state_vars {
            if let Some(ty) = self.state_types.get(&state.name) {
                if is_scalar_state_type(ty) {
                    scalars.push((state.name.clone(), ty.clone()));
                }
            }
        }
        // Plan 371 Task 22c / L3: include child-component scalar state fields.
        // - Loop children: hoist scalar fields directly (they live on the parent).
        // - Persistent children: their state lives in the instance field, so we
        //   recurse into self.<field>.state_snapshot() instead.
        for child_name in &self.child_components {
            if self.is_persistent_child(child_name) {
                // L3: persistent child — recurse into the instance's snapshot.
                // (handled by the component-typed field recursion below)
            } else if let Some(child_fields) = self.component_state_fields.get(child_name) {
                for (f, ty) in child_fields {
                    if is_scalar_state_type(ty) && !scalars.iter().any(|(n, _)| n == f) {
                        scalars.push((f.clone(), ty.clone()));
                    }
                }
            }
        }

        // Plan 371 Task 22b: collect component-typed fields whose state_snapshot
        // we should recurse into (with a "<field>." prefix) so the rust-mode
        // autoui_state tool can see child/store state. The store field is
        // injected via STORE_NAMES (alias "store"); child components declared
        // as struct fields show up in state_types/prop_types with a type that
        // matches a registered component name.
        let mut recurse_fields: Vec<String> = Vec::new();
        // Store composable field (always named "store" per generate_struct).
        let is_store_itself = STORE_NAMES.with(|sn| {
            sn.borrow().values().any(|s| s.as_str() == _widget.name)
        });
        if !is_store_itself {
            STORE_NAMES.with(|sn| {
                if !sn.borrow().is_empty() && !scalars.iter().any(|(n, _)| n == "store") {
                    recurse_fields.push("store".to_string());
                }
            });
        }
        // Child components stored as struct fields (type matches a known
        // component — registered store or a child_components entry).
        let known_components: std::collections::HashSet<String> = {
            let mut s: std::collections::HashSet<String> = STORE_NAMES
                .with(|sn| sn.borrow().values().cloned().collect());
            for c in &self.child_components {
                s.insert(c.clone());
            }
            s
        };
        for (name, ty) in self.state_types.iter().chain(self.prop_types.iter()) {
            if known_components.contains(ty) && !recurse_fields.contains(name) && name != "store" {
                recurse_fields.push(name.clone());
            }
        }
        // Plan 371 L3: persistent child component fields (e.g. editor_panel)
        // also need recursion so their state (editing/edit_title) is visible.
        for child_name in &self.child_components {
            if self.is_persistent_child(child_name) {
                let field = Self::child_field_name(child_name);
                if !recurse_fields.contains(&field) {
                    recurse_fields.push(field);
                }
            }
        }

        if scalars.is_empty() && recurse_fields.is_empty() {
            return String::new();
        }

        let mut code = String::new();
        code.push_str("    fn state_snapshot(&self) -> std::collections::HashMap<String, auto_lang::ui::auto_val::Value> {\n");
        code.push_str("        let mut m = std::collections::HashMap::new();\n");
        for (name, ty) in &scalars {
            let expr = scalar_to_auto_value_expr("self", name, ty);
            code.push_str(&format!(
                "        m.insert({:?}.to_string(), {});\n",
                name, expr
            ));
        }
        // Recurse into component-typed fields, prefixing keys with "<field>.".
        for field in &recurse_fields {
            code.push_str(&format!(
                "        for (k, v) in self.{}.state_snapshot() {{ m.insert(format!(\"{{}}.{{}}\", {:?}, k), v); }}\n",
                field, field
            ));
        }
        code.push_str("        m\n");
        code.push_str("    }\n");
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
                let variant_info = self.message_variants.iter()
                    .find(|v| v.name == variant_name);
                // Plan 374: Skip handlers whose variant is not in the message enum.
                // This handles cases like NewNoteInFolder where the handler exists
                // but the Msg enum doesn't declare the variant.
                if variant_info.is_none() {
                    continue;
                }
                let has_payload = variant_info.map_or(false, |v| !v.payload.is_empty());
                // Tick handler: guard with running check if "running" field exists
                let is_tick_guarded = variant_name == "Tick" && self.state_types.contains_key("running");
                if has_payload {
                    // Plan 346: use the source parameter name (e.g., `i` from
                    // `.SelectNote(i)`) as the match binding, not a hardcoded `id`.
                    let payload_name = self.extract_payload_name(pattern);
                    code.push_str(&format!("            {}::{}({}) => {{\n", msg_name, variant_name, payload_name));
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
                    // or body that assigns the bound field from the msg payload
                    // (e.g. `.edit_title = t` / `.edit_title = t.to_string()`) —
                    // we already set it from last_input_text() above, so the
                    // payload binding would clobber it with the static empty arg.
                    let payload_name = if has_payload {
                        self.extract_payload_name(pattern)
                    } else {
                        String::new()
                    };
                    let body_redundant = field_names.iter().all(|f| {
                        let b = body.trim();
                        b == format!("self.{} = self.{}", f, f)
                            || b == format!("self.{} = {}", f, payload_name)
                            || b == format!("self.{} = {}.to_string()", f, payload_name)
                            || b == format!("self.{} = {}.clone()", f, payload_name)
                    });
                    if !body_redundant && !body.trim().is_empty() {
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

                // Plan 371 L1: If this handler changes the store data that feeds
                // a child component's props (e.g. NewNote changes active_id →
                // EditorPanel's `note` prop changes), the child needs its Init
                // lifecycle re-triggered so it resets its editing state for the
                // new empty note. VM mode does this implicitly via the unified
                // state heap; rust mode has no such lifecycle, so simulate it.
                // General criteria (replaces the old hardcoded "NewNote" +
                // name-contains-"Editor" check):
                //   1. The handler body mutates store data (calls a mutating
                //      store method or assigns to store.active_id / store.notes).
                //   2. There exists a child component whose props are written by
                //      its own handlers (component_semantics written_props), i.e.
                //      a child that owns editable state tied to those props.
                if self.handler_mutates_store_data(payload)
                    && !self.child_components.is_empty()
                {
                    // Find the first child whose handlers write props (i.e. it has
                    // editable state to reset on data change). This replaces the
                    // old `.contains("Editor")` name match.
                    let target = self.child_components.iter()
                        .find(|c| {
                            self.component_semantics.get(*c)
                                .map(|s| !s.written_props.is_empty())
                                .unwrap_or(false)
                        })
                        .cloned();
                    if let Some(child_name) = target {
                        let child_msg = format!("{}Msg", child_name);
                        let persistent = self.is_persistent_child(&child_name);
                        let field = Self::child_field_name(&child_name);
                        // Ensure the preceding body statement ends with ';'.
                        code.push_str("                ;\n");
                        if persistent {
                            // L3: use persistent instance — update props then Init.
                            let constructor_args = self.find_constructor_args_for_child(widget, &child_name);
                            // Sync props from constructor args, then call on(Init).
                            code.push_str(&format!(
                                "                self.{} = {}::new({});\n",
                                field, child_name, constructor_args
                            ));
                            code.push_str(&format!(
                                "                self.{}.on({}::Init);\n",
                                field, child_msg
                            ));
                        } else {
                            // Loop child: temp-construct, Init, sync back.
                            let sync: Vec<String> = self.component_state_fields.get(&child_name)
                                .map(|fs| fs.iter().map(|(f,_)| f.clone()).collect())
                                .unwrap_or_default();
                            let constructor_args = self.find_constructor_args_for_child(widget, &child_name);
                            code.push_str(&format!(
                                "                {{ let mut __ep = {}::new({});\n",
                                child_name, constructor_args
                            ));
                            code.push_str(&format!(
                                "                __ep.on({}::Init);\n",
                                child_msg
                            ));
                            for f in &sync {
                                code.push_str(&format!(
                                    "                self.{} = __ep.{}.clone();\n", f, f
                                ));
                            }
                            code.push_str("                }\n");
                        }
                    }
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
            for child_name in &self.child_components {
                let persistent = self.is_persistent_child(child_name);
                let field_name = Self::child_field_name(child_name);
                let sync_fields = self.find_sync_fields_for_child(widget, child_name);
                let constructor_args = self.find_constructor_args_for_child(widget, child_name);

                code.push_str(&format!(
                    "            {}::{}(inner) => {{\n",
                    msg_name, child_name
                ));

                if persistent {
                    // Plan 371 L3: persistent child — operate directly on the field.
                    // The child's on() may mutate its cloned store (e.g. NavTree's
                    // SelectPinned sets store.active_folder). Sync store back so the
                    // parent sees the change. Private state (editing etc.) persists
                    // in the field and needs no sync.
                    code.push_str(&format!(
                        "                self.{}.on(inner);\n",
                        field_name
                    ));
                    // Sync store back if the child has one.
                    let has_store = STORE_NAMES.with(|sn| !sn.borrow().is_empty());
                    if has_store {
                        code.push_str(&format!(
                            "                self.store = self.{}.store.clone();\n",
                            field_name
                        ));
                    }
                } else {
                    // Loop child (or legacy): temp-construct, sync in/out, then drop.
                    code.push_str(&format!(
                        "                let mut __child = {}::new({});\n",
                        child_name, constructor_args
                    ));
                    for field in &sync_fields {
                        code.push_str(&format!(
                            "                __child.{} = self.{}.clone();\n",
                            field, field
                        ));
                    }
                    code.push_str("                __child.on(inner);\n");
                    for field in &sync_fields {
                        code.push_str(&format!(
                            "                self.{} = __child.{};\n",
                            field, field
                        ));
                    }
                }

                // Plan 371 L1: General prop-writeback. For each prop the child's
                // handlers WRITE, write the (possibly mutated) child prop back to the
                // parent's data source. Persistent children use self.<field>.<prop>;
                // temp children use __child.<prop>.
                let written_props: Vec<String> = self.component_semantics.get(child_name)
                    .map(|s| s.written_props.clone())
                    .unwrap_or_default();
                if !written_props.is_empty() {
                    let has_notes = self.state_types.contains_key("notes");
                    let has_store_notes = STORE_NAMES.with(|sn| !sn.borrow().is_empty())
                        && !STORE_NAMES.with(|sn| sn.borrow().values().any(|s| s.as_str() == widget.name));
                    let notes_prefix = if has_notes { "self.notes" } else if has_store_notes { "self.store.notes" } else { "" };
                    let active_prefix = if self.state_types.contains_key("active_id") { "self.active_id" } else if has_store_notes { "self.store.active_id" } else { "" };
                    let prop_owner = if persistent {
                        format!("self.{}", field_name)
                    } else {
                        "__child".to_string()
                    };
                    if !notes_prefix.is_empty() && !active_prefix.is_empty() {
                        for prop in &written_props {
                            code.push_str(&format!(
                                "                if let Some(__n) = {}.get_mut({} as usize) {{\n                    *__n = {}.{}.clone();\n                }}\n",
                                notes_prefix, active_prefix, prop_owner, prop
                            ));
                        }
                    }
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
        let mut view_code = self.generate_view_tree(&widget.view_tree);

        // Plan 374: Post-process view code for known patterns.
        // Fix 1: Replace bare `active` in conditions with `i == self.store.active_id`
        // (from view fragment NoteItem parameter substitution that didn't complete).
        view_code = view_code.replace("if active {", "if i == self.store.active_id {");
        // Fix 2: Value["field"].iter() → Value["field"].as_array().into_iter().flatten()
        view_code = view_code.replace("[\"tags\"].iter()", "[\"tags\"].as_array().unwrap_or(&Vec::new()).iter()");
        // Fix 3: String == &Value → String == X.as_str().unwrap_or_default()
        view_code = view_code.replace("self.store.active_tag == t {", "self.store.active_tag == t.as_str().unwrap_or_default() {");
        view_code = view_code.replace("*self.store.active_tag == *t {", "self.store.active_tag == t.as_str().unwrap_or_default() {");
        // Fix 4: RemoveTag(t) where t is &Value → RemoveTag(t.to_string())
        view_code = view_code.replace("EditorPanelMsg::RemoveTag(t)", "EditorPanelMsg::RemoveTag(t.to_string())");
        // Fix 5: SelectTag(t) where t is &Value → SelectTag(t.to_string())
        view_code = view_code.replace("NavTreeMsg::SelectTag(t)", "NavTreeMsg::SelectTag(t.to_string())");

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
            // Register store computed property names globally for cross-widget access
            STORE_COMPUTED_NAMES.with(|sn| {
                sn.borrow_mut().insert(method_name.clone());
            });
            let mut expr_rust = self.ast_expr_to_rust(&computed_prop.expr);

            // Plan 374: Vec<Value> doesn't have .filter()/.map() directly —
            // insert .iter() before them so the iterator chain type-checks.
            // e.g., self.notes.filter(...) → self.notes.iter().filter(...)
            for method in &[".filter(", ".map("] {
                if expr_rust.contains(method) && !expr_rust.contains(".iter()") {
                    // Find state vars that are Vec<...> and insert .iter() after them
                    for state in &widget.state_vars {
                        let field_ref = format!("self.{}", state.name);
                        let field_ref_with_iter = format!("self.{}.iter()", state.name);
                        let target = format!("{}{}", field_ref, method);
                        let replacement = format!("{}{}", field_ref_with_iter, method);
                        if expr_rust.contains(&target) && !expr_rust.contains(&field_ref_with_iter) {
                            expr_rust = expr_rust.replace(&target, &replacement);
                        }
                    }
                }
            }

            // Generate getter method.
            // a2r fix: infer return type from the computed expression.
            // - If expr has .iter()/.filter()/.map() → Vec<serde_json::Value> + collect
            // - If expr is a string literal or str field access → String
            // - If expr is a self.field that's typed String → String
            // - Otherwise default to String (safer than Vec for scalar computed).
            let needs_collect = expr_rust.contains(".iter().") || expr_rust.contains(".filter(") || expr_rust.contains(".map(");
            let is_string_expr = !needs_collect && (
                expr_rust.contains("\"")  // string literal
                || expr_rust.contains(".to_string()")
                || expr_rust.contains("+ \"")  // string concatenation
                || self.state_types.iter().any(|(k, v)|
                    v == "String" && expr_rust.contains(&format!("self.{}", k)))
            );
            let (return_type, final_expr) = if needs_collect {
                let rt = "Vec<serde_json::Value>";
                let fe = if !expr_rust.contains(".collect(") {
                    if expr_rust.contains(".iter().") {
                        format!("{}.cloned().collect::<Vec<_>>()", expr_rust)
                    } else {
                        format!("{}.collect::<Vec<_>>()", expr_rust)
                    }
                } else {
                    expr_rust.clone()
                };
                (rt, fe)
            } else if is_string_expr {
                ("String", expr_rust.clone())
            } else {
                // Default: String for scalar computed (int/bool/str).
                ("String", expr_rust.clone())
            };
            code.push_str(&format!("    pub fn {}(&self) -> {} {{\n", method_name, return_type));
            code.push_str(&format!("        {}\n", final_expr));
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
                                // Plan 407 R4a: strip leading dot (.board → board) so
                                // state_types lookup works for store field references.
                                let coll_name = collection.as_str();
                                let coll_stripped = coll_name.strip_prefix('.').unwrap_or(coll_name);
                                if self.state_types.get(coll_stripped)
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
                    // Resolve the `value` binding to a field name. Source uses
                    // `.field` which parses to Expr::Dot(self, "field"); older
                    // code only matched Expr::Ident, missing `.field` bindings
                    // (Plan 371 T5c: edit_title input had no last_input_text injection).
                    let value_field: Option<String> = match props.get("value") {
                        Some(AuraPropValue::Expr(crate::ast::Expr::Ident(name))) => Some(name.to_string()),
                        // Only match direct self.field (one-level Dot). Multi-level
                        // dots like self.store.edit_text should NOT trigger field
                        // injection — the value comes from the store, not a local
                        // state var, so injecting self.edit_text = ... would fail.
                        Some(AuraPropValue::Expr(crate::ast::Expr::Dot(obj, field))) => {
                            // Check it's a direct self.field, not self.store.field
                            let is_direct_self = match obj.as_ref() {
                                crate::ast::Expr::Ident(name) => name.as_str() == "self" || name.as_str() == ".",
                                _ => false,
                            };
                            if is_direct_self { Some(field.to_string()) } else { None }
                        }
                        _ => None,
                    };
                    if let Some(name) = value_field {
                        for (event, handler) in events {
                            if matches!(event.as_str(), "oninput" | "onInput" | "onchange" | "onChange") {
                                let variant = self.extract_variant_name(&handler.handler);
                                self.input_fields.entry(variant).or_default().push(name.to_string());
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
        self.scan_child_components_inner(node, false);
    }

    /// Recursive scan with in_loop tracking.
    /// Plan 371 L3: children found inside a for-loop are added to
    /// `loop_child_components` so they are NOT promoted to persistent fields.
    fn scan_child_components_inner(&mut self, node: &AuraNode, in_loop: bool) {
        match node {
            AuraNode::Element { tag, children, .. } => {
                if self.is_custom_widget(tag) {
                    if !self.child_components.contains(&tag.to_string()) {
                        self.child_components.push(tag.clone());
                    }
                    if in_loop {
                        self.loop_child_components.insert(tag.clone());
                    }
                }
                for child in children {
                    self.scan_child_components_inner(child, in_loop);
                }
            }
            AuraNode::ForLoop { body, .. } => {
                // Plan 371 L3: mark children inside for-loops — they can't be
                // persistent fields (multiple instances). Scan with in_loop=true.
                for child in body {
                    self.scan_child_components_inner(child, true);
                }
            }
            AuraNode::Conditional { then_body, else_body, .. } => {
                for child in then_body {
                    self.scan_child_components_inner(child, in_loop);
                }
                if let Some(else_nodes) = else_body {
                    for child in else_nodes {
                        self.scan_child_components_inner(child, in_loop);
                    }
                }
            }
            AuraNode::Component { name, .. } => {
                if !self.child_components.contains(name) {
                    self.child_components.push(name.clone());
                }
                if in_loop {
                    self.loop_child_components.insert(name.clone());
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
            // Use col() for multi-statement conditional bodies so siblings
            // stack VERTICALLY (the common case — e.g. a section header row
            // followed by a for-loop list, repeated per category). The parent
            // col/row already controls the outer layout direction; an if-body
            // with several children is almost always a vertical sequence.
            // (Previously row() was used, which laid mixed row+for siblings out
            // side-by-side, producing a diagonal/tilted arrangement.)
            let mut builder = "View::col()".to_string();
            for v in views {
                builder = format!("{}.child({})", builder, v);
            }
            format!("{}.build()", builder)
        }
    }

    /// Collect a composite label expression for a button that has children.
    ///
    /// `View::Button` has no children field, so a button with child views
    /// (e.g. `button "" { col { text .note.title; text .note.time } }`) would
    /// render empty (the `.child()` calls are dropped at build). This folds
    /// the button's TEXT-bearing children into a single `format!` label so the
    /// content is visible.
    ///
    /// Recurses into col/row containers to reach their text children. Non-text
    /// children (buttons, components, images) are skipped. Returns a Rust
    /// expression evaluating to `String`, or `"\"\""` if no text children found.
    fn collect_button_label(&self, children: &[AuraNode]) -> String {
        let mut parts: Vec<String> = Vec::new();
        for child in children {
            self.collect_text_parts(child, &mut parts);
        }
        if parts.is_empty() {
            return "\"\"".to_string();
        }
        if parts.len() == 1 {
            return format!("format!(\"{{}}\", {})", parts[0]);
        }
        let fmt = parts.iter().map(|_| "{}").collect::<Vec<_>>().join("\\n");
        format!("format!(\"{}\", {})", fmt, parts.join(", "))
    }

    /// Recursive helper for `collect_button_label`: push a Rust String
    /// expression for each text-bearing node into `parts`.
    fn collect_text_parts(&self, node: &AuraNode, parts: &mut Vec<String>) {
        match node {
            AuraNode::Text(content) => match content {
                AuraTextContent::Literal(s) => {
                    parts.push(format!("\"{}\".to_string()", s));
                }
                AuraTextContent::Interpolated { template, bindings } => {
                    let mut fmt = template.clone();
                    let mut args: Vec<String> = Vec::new();
                    for name in bindings.iter() {
                        if let Some(start) = fmt.find("${") {
                            if let Some(end) = fmt[start..].find('}') {
                                fmt.replace_range(start..=start + end, "{}");
                            }
                        }
                        let stripped = name.trim_start_matches('.');
                        args.push(format!("self.{}", stripped));
                    }
                    if args.is_empty() {
                        parts.push(format!("\"{}\".to_string()", fmt));
                    } else {
                        parts.push(format!("format!(\"{}\", {})", fmt, args.join(", ")));
                    }
                }
            },
            AuraNode::Element { tag, props, children, .. } => {
                if tag == "text" {
                    if let Some(AuraPropValue::Expr(expr)) = props.get("text") {
                        parts.push(self.ast_expr_to_rust(expr));
                        return;
                    }
                    for c in children {
                        self.collect_text_parts(c, parts);
                    }
                } else if tag == "col" || tag == "row" || tag == "column" {
                    for c in children {
                        self.collect_text_parts(c, parts);
                    }
                }
            }
            _ => {}
        }
    }

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

                // Rich-content elements with no native iced mapping (e.g.
                // autodown_editor { content: .note.body }) would otherwise fall
                // through to an empty View::col(), hiding the content. Render
                // their `content` prop as styled text so the data is visible.
                if tag == "autodown_editor" || tag == "markdown" || tag == "editor" {
                    if let Some(AuraPropValue::Expr(expr)) = props.get("content") {
                        let content_expr = self.ast_expr_to_rust(expr);
                        // text_styled takes content by value; clone self-field
                        // references (e.g. self.edit_body) to avoid E0507 moves.
                        let content_expr = if content_expr.starts_with("self.") {
                            format!("{}.clone()", content_expr)
                        } else {
                            content_expr
                        };
                        let style_str = props.get("style").or_else(|| props.get("class"))
                            .and_then(|v| if let AuraPropValue::Expr(crate::ast::Expr::Str(s)) = v { Some(s.to_string()) } else { None })
                            .unwrap_or_default();
                        return format!(
                            "View::col().style(\"{}\").child(View::text_styled({}, \"text-sm text-foreground whitespace-pre-wrap\")).build()",
                            style_str, content_expr
                        );
                    }
                }

                // grid → View::grid() builder. iced has no native grid; the
                // col-of-rows decomposition (final-row padding + w-full rows)
                // now lives in ONE place — the shared generic `build_grid`
                // (Plan 319) — so the rust `into_iced` path and the VM
                // `render_dynamic_view` path share it and can never drift.
                if tag == "grid" {
                    let cols = props.get("cols").or_else(|| props.get("columns"))
                        .and_then(|v| match v {
                            AuraPropValue::Expr(crate::ast::Expr::Int(n)) => Some(*n as usize),
                            AuraPropValue::Expr(crate::ast::Expr::Str(s)) => s.trim().parse::<usize>().ok(),
                            _ => None,
                        })
                        .map(|c| c.max(1))
                        .unwrap_or(1);
                    let gap = props.get("gap")
                        .and_then(|v| match v {
                            AuraPropValue::Expr(crate::ast::Expr::Int(n)) => Some(*n as u16),
                            AuraPropValue::Expr(crate::ast::Expr::Str(s)) => s.trim().parse::<u16>().ok(),
                            _ => None,
                        })
                        .unwrap_or(0);
                    let style_str = props.get("style").or_else(|| props.get("class"))
                        .and_then(|v| if let AuraPropValue::Expr(crate::ast::Expr::Str(s)) = v {
                            Some(s.to_string())
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
                        if let Some(AuraPropValue::Expr(crate::ast::Expr::Str(s))) = props.get("text") {
                            if s.contains("${") {
                                return format!("View::text({})", self.interpolate_str(s));
                            }
                            return format!("View::text(\"{}\".to_string())", s);
                        }
                        if let Some(AuraPropValue::Expr(crate::ast::Expr::Ident(name))) = props.get("text") {
                            let name_str = name.as_str();
                            if self.is_loop_var(name_str) {
                                return format!("View::text(format!(\"{{}}\", {}))", name_str);
                            }
                            return format!("View::text(format!(\"{{}}\", self.{}))", name_str);
                        }
                    } else {
                        // Text with styling — collect classes and use View::text_styled
                        let class_str = props.get("style")
                            .or_else(|| props.get("class"))
                            .and_then(|v| if let AuraPropValue::Expr(crate::ast::Expr::Str(s)) = v { Some(s.to_string()) } else { None })
                            .unwrap_or_default();
                        if let Some(AuraPropValue::Expr(crate::ast::Expr::Ident(name))) = props.get("text") {
                            let name_str = name.as_str();
                            if self.is_loop_var(name_str) {
                                return format!("View::text_styled(format!(\"{{}}\", {}), \"{}\")", name_str, class_str);
                            }
                            return format!("View::text_styled(format!(\"{{}}\", self.{}), \"{}\")", name_str, class_str);
                        }
                        if let Some(AuraPropValue::Expr(crate::ast::Expr::Str(s))) = props.get("text") {
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
                        .and_then(|v| if let AuraPropValue::Expr(crate::ast::Expr::Str(s)) = v { Some(s.to_string()) } else { None })
                        .unwrap_or_default();

                    let mut builder = format!("View::input(\"{}\")", placeholder);

                    // Value binding: value: .field → .value(format!("{}", self.field))
                    if let Some(AuraPropValue::Expr(crate::ast::Expr::Ident(name))) = props.get("value") {
                        builder = format!("{}.value(format!(\"{{}}\", self.{}))", builder, name);
                    } else if let Some(AuraPropValue::Expr(crate::ast::Expr::Str(s))) = props.get("value") {
                        builder = format!("{}.value(\"{}\".to_string())", builder, s);
                    }

                    // Password mode: type: "password"
                    if let Some(AuraPropValue::Expr(crate::ast::Expr::Str(s))) = props.get("type") {
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
                                // Plan 374: For variants with String payload, pass a default
                                // value — the handler reads actual text via last_input_text().
                                // This keeps the builder's type parameter as M (not fn pointer).
                                let has_string_payload = self.message_variants.iter()
                                    .find(|v| v.name == variant)
                                    .map(|v| v.payload.first().map_or(false, |t| matches!(t, crate::ast::Type::StrOwned | crate::ast::Type::StrSlice | crate::ast::Type::StrFixed(_))))
                                    .unwrap_or(false);
                                if has_string_payload {
                                    builder = format!("{}.on_change({}::{}(\"\".to_string()))", builder, msg_name, variant);
                                } else {
                                    builder = format!("{}.on_change({}::{})", builder, msg_name, variant);
                                }
                                // Record event→field mapping for handler generation
                                if let Some(AuraPropValue::Expr(crate::ast::Expr::Ident(name))) = props.get("value") {
                                    self.input_fields.entry(variant).or_default().push(name.to_string());
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
                        .and_then(|v| if let AuraPropValue::Expr(crate::ast::Expr::Str(s)) = v { Some(s.to_string()) } else { None })
                        .unwrap_or_default();

                    let mut builder = format!("View::textarea(\"{}\")", placeholder);

                    // Value binding: value: .field → .value(format!("{}", self.field))
                    if let Some(AuraPropValue::Expr(crate::ast::Expr::Ident(name))) = props.get("value") {
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
                                // Plan 374: For variants with String payload, pass a default
                                // value — the handler reads actual text via last_input_text().
                                // This keeps the builder's type parameter as M (not fn pointer).
                                let has_string_payload = self.message_variants.iter()
                                    .find(|v| v.name == variant)
                                    .map(|v| v.payload.first().map_or(false, |t| matches!(t, crate::ast::Type::StrOwned | crate::ast::Type::StrSlice | crate::ast::Type::StrFixed(_))))
                                    .unwrap_or(false);
                                if has_string_payload {
                                    builder = format!("{}.on_change({}::{}(\"\".to_string()))", builder, msg_name, variant);
                                } else {
                                    builder = format!("{}.on_change({}::{})", builder, msg_name, variant);
                                }
                                if let Some(AuraPropValue::Expr(crate::ast::Expr::Ident(name))) = props.get("value") {
                                    self.input_fields.entry(variant).or_default().push(name.to_string());
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
                    .and_then(|v| if let AuraPropValue::Expr(crate::ast::Expr::Str(s)) = v { Some(s.to_string()) } else { None })
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
                    .and_then(|v| if let AuraPropValue::Expr(crate::ast::Expr::Ident(name)) = v { Some(name.clone()) } else { None });

                // Generate a Rust expression string for the text prop, handling ALL AuraExpr types.
                // This catches FieldAccess (note.title), Index, and other dynamic expressions
                // that fall through the Literal/StateRef checks above.
                let text_rust_expr: Option<String> = if text_prop.is_some() || text_state_ref.is_some() {
                    None // Already handled by text_prop or text_state_ref
                } else {
                    props.get("text").and_then(|v| {
                        if let AuraPropValue::Expr(expr) = v {
                            Some(self.ast_expr_to_rust(expr))
                        } else {
                            None
                        }
                    })
                };

                // Handle image element — generate View::image() or View::image_styled()
                if tag == "image" {
                    let src = props.get("src")
                        .and_then(|v| if let AuraPropValue::Expr(crate::ast::Expr::Ident(name)) = v {
                            Some(format!("format!(\"{{}}\", self.{})", name))
                        } else if let AuraPropValue::Expr(crate::ast::Expr::Str(s)) = v {
                            Some(format!("\"{}\"", s))
                        } else {
                            None
                        }).unwrap_or_else(|| "\"\"".to_string());
                    let style_str = props.get("style")
                        .or_else(|| props.get("class"))
                        .and_then(|v| if let AuraPropValue::Expr(crate::ast::Expr::Str(s)) = v { Some(s.to_string()) } else { None })
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
                    let value_expr = if let Some(AuraPropValue::Expr(crate::ast::Expr::Ident(name))) = props.get("value") {
                        format!("self.{}", name)
                    } else if let Some(AuraPropValue::Expr(crate::ast::Expr::Str(s))) = props.get("value") {
                        s.to_string()
                    } else {
                        "0".to_string()
                    };
                    let max_val = if let Some(AuraPropValue::Expr(crate::ast::Expr::Str(s))) = props.get("max") {
                        s.to_string()
                    } else {
                        "100".to_string()
                    };
                    let style_str = props.get("style")
                        .or_else(|| props.get("class"))
                        .and_then(|v| if let AuraPropValue::Expr(crate::ast::Expr::Str(s)) = v { Some(s.to_string()) } else { None })
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
                            AuraPropValue::Expr(crate::ast::Expr::Bool(b)) => b.to_string(),
                            AuraPropValue::Expr(crate::ast::Expr::Ident(name)) => format!("self.{}", name),
                            AuraPropValue::Expr(crate::ast::Expr::Dot(object, field)) => {
                                let field = field.clone();
                                let obj_str = match object.as_ref() {
                                    crate::ast::Expr::Ident(name) => {
                                        let resolved = if name.starts_with('.') { &name[1..] } else { name.as_str() };
                                        if self.is_loop_var(resolved) && self.value_loop_vars.contains(resolved) {
                                            resolved.to_string()
                                        } else {
                                            format!("self.{}", resolved)
                                        }
                                    }
                                    _ => format!("{:?}", object),
                                };
                                self.value_field_access(&obj_str, field.as_str())
                            }
                            _ => "false".to_string(),
                        })
                        .unwrap_or_else(|| "false".to_string());
                    let label = props.get("label")
                        .or_else(|| props.get("text"))
                        .and_then(|v| if let AuraPropValue::Expr(crate::ast::Expr::Str(s)) = v { Some(s.to_string()) } else { None })
                        .unwrap_or_default();

                    // Parse class/style into Style
                    let class_str = props.get("class")
                        .or_else(|| props.get("style"))
                        .and_then(|v| if let AuraPropValue::Expr(crate::ast::Expr::Str(s)) = v { Some(s.to_string()) } else { None })
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
                            // Plan 374: Use loop var directly if it's in scope
                            let name_ref = if self.is_loop_var(name) { name.to_string() } else { format!("self.{}", name) };
                            format!("View::button(format!(\"{{}}\", {}))", name_ref)
                        } else {
                            let name_ref = if self.is_loop_var(name) { name.to_string() } else { format!("self.{}", name) };
                            format!("View::text(format!(\"{{}}\", {}))", name_ref)
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
                        .and_then(|v| if let AuraPropValue::Expr(crate::ast::Expr::Str(s)) = v { Some(s.to_string()) } else { None })
                        .unwrap_or_default();

                    // Prepend heading default styles (h1→text-4xl font-bold, etc.)
                    let style_str = match Self::heading_default_style(tag.as_str()) {
                        Some(default) if !user_style.is_empty() => format!("{} {}", default, user_style),
                        Some(default) => default.to_string(),
                        None => user_style,
                    };

                    if let Some(ref name) = text_state_ref {
                        if self.is_loop_var(name) {
                            return format!("View::text_styled(format!(\"{{}}\", {}), \"{}\")", name, style_str);
                        }
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
                        // Plan 407 R2: wrap self. references in format! to avoid
                        // moving String fields out of self in the immutable view().
                        let text = if text.starts_with("self.") {
                            format!("format!(\"{{}}\", {})", text)
                        } else {
                            text.clone()
                        };
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
                        // Plan 407 R2: wrap self. references to avoid String move.
                        let text = if text.starts_with("self.") {
                            format!("format!(\"{{}}\", {})", text)
                        } else {
                            text.clone()
                        };
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
                        .and_then(|v| if let AuraPropValue::Expr(crate::ast::Expr::Str(s)) = v { Some(s.to_string()) } else { None })
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
                } else if tag == "button" {
                    // Button with children. The View::Button model only has a
                    // `label` (no children field), so `.child()` calls are
                    // silently dropped at build time and the button renders
                    // empty. Fold the button's text children into a single
                    // composite label so the content is visible.
                    let label_expr = self.collect_button_label(children);
                    let mut builder = format!("View::button({})", label_expr);
                    for (key, value) in props {
                        if key == "text" { continue; }
                        builder = self.add_prop_to_builder(&builder, key, value);
                    }
                    for (event, handler) in events {
                        builder = self.add_event_to_builder(&builder, event, handler);
                    }
                    if !events.iter().any(|(e, _)| e == "onclick" || e == "onClick") {
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
                        // Plan 374: ForLoop now produces a single View (wrapped in col().children)
                        // so always use .child() regardless of node type.
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
                // Plan 371 步骤3: Sanitize loop var to avoid Rust keyword/macro
                // conflicts (e.g. `for todo in ...` collides with `todo!()` macro).
                let var = sanitize_rust_ident(var);
                let index = index.as_ref().map(|i| sanitize_rust_ident(i));
                // Generate iterator-based view construction
                let iter_name = iterable.trim_start_matches('.');
                // Plan 374: Check if the last component is a computed property (needs ()).
                let last_component = iter_name.rsplit('.').next().unwrap_or(iter_name);
                let needs_method_call = self.computed_names.contains(last_component)
                    || STORE_COMPUTED_NAMES.with(|sn| sn.borrow().contains(last_component));
                let iter_expr = if iterable.starts_with('.') {
                    let base = if needs_method_call {
                        format!("self.{}()", iter_name)
                    } else {
                        self.resolve_dotted_path(iter_name)
                    };
                    base
                } else {
                    iterable.clone()
                };

                // Check if iterable is a Value-type collection.
                // Handle both simple names ("notes") and compound paths ("store.notes").
                let iter_last_name = iter_name.rsplit('.').next().unwrap_or(iter_name);
                let is_value_iter = self.state_types.get(iter_name)
                    .map(|ty| ty.contains("serde_json::Value"))
                    .or_else(|| self.state_types.get(iter_last_name)
                        .map(|ty| ty.contains("serde_json::Value")))
                    .unwrap_or(false)
                    // Plan 374: If we can't determine the type, assume Value (most
                    // data from API/store is serde_json::Value in this system).
                    || (iter_name.contains('.') 
                        && !self.state_types.contains_key(iter_name)
                        && !self.state_types.contains_key(iter_last_name));

                // Push loop vars into scope
                self.push_loop_vars(&var, index.as_deref());
                if is_value_iter {
                    self.value_loop_vars.insert(var.clone());
                }

                // Generate body with loop vars in scope
                let body_code: Vec<String> = body.iter()
                    .map(|child| self.generate_view_tree(child))
                    .collect();

                // Pop loop vars from scope
                self.pop_loop_vars(&var, index.as_deref());
                self.value_loop_vars.remove(&var);

                // Auto-generate search filter: if the widget has a "search" state var
                // and we're iterating a Value collection, insert .filter() before .map()
                let search_filter = if is_value_iter && self.state_types.contains_key("search") {
                    let var_ref = var.clone();
                    // When enumerate() is used (index present), the filter closure
                    // receives &(usize, &Value) — destructure to access the element.
                    let filter_pattern = if index.is_some() {
                        format!("(_, {})", var_ref)
                    } else {
                        var_ref.clone()
                    };
                    Some(format!(
                        ".filter(|{}| {{ \
                            let __q = self.search.to_lowercase(); \
                            if __q.is_empty() {{ return true; }} \
                            let __t = {}[\"title\"].as_str().unwrap_or_default().to_lowercase(); \
                            __t.contains(&__q) \
                        }})",
                        filter_pattern, var_ref
                    ))
                } else {
                    None
                };

                let map_expr = if let Some(idx) = index {
                    // Cast the usize index to i32 so comparisons with state
                    // fields (typically i32) type-check.
                    format!("{}.iter().enumerate(){}{}.map(|({}, {})| {{ let {} = {} as i32; {} }})", iter_expr, search_filter.as_ref().map_or(String::new(), |f| f.clone()), "", idx, var, idx, idx, body_code.join("\n"))
                } else {
                    format!("{}.iter(){}{}.map(|{}| {{ {} }})", iter_expr, search_filter.as_ref().map_or(String::new(), |f| f.clone()), "", var, body_code.join("\n"))
                };
                // Plan 374: Always produce a single View by wrapping in col().children().
                // This works in both .child() and conditional/if contexts.
                format!("View::col().children({}.collect::<Vec<_>>()).build()", map_expr)
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
                // Plan 374: Sort props alphabetically for deterministic
                // constructor argument ordering (matches alphabetical source decl).
                let mut sorted_props: Vec<_> = props.iter().collect();
                sorted_props.sort_by_key(|(k, _)| k.as_str());
                let mut constructor_args: Vec<String> = Vec::new();
                for (_key, value) in sorted_props {
                    constructor_args.push(self.arg_to_rust(value));
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

        // Build constructor arguments from props (for loop children or prop sync).
        let mut constructor_args: Vec<String> = Vec::new();
        let mut sorted_keys: Vec<&String> = props.keys()
            .filter(|k| *k != "style" && *k != "class")
            .collect();
        sorted_keys.sort();
        for key in sorted_keys {
            if let crate::aura::AuraPropValue::Expr(expr) = &props[key] {
                constructor_args.push(self.arg_to_rust(expr));
            }
        }
        let args_str = constructor_args.join(", ");

        // Plan 371 L3: persistent children use self.<field>.view() with prop sync.
        // Since view() is &self (immutable), we clone the instance, sync props on
        // the clone, then call .view() on it. The clone carries the persistent
        // private state (editing/edit_title etc.); props (note/store) are refreshed.
        if self.is_persistent_child(tag) {
            let field = Self::child_field_name(tag);
            // Collect prop sync assignments for the cloned instance.
            let mut prop_keys: Vec<&String> = props.keys()
                .filter(|k| *k != "style" && *k != "class")
                .collect();
            prop_keys.sort();
            let mut sync_code = String::new();
            for key in &prop_keys {
                if let crate::aura::AuraPropValue::Expr(expr) = &props[*key] {
                    let prop_val = self.arg_to_rust(expr);
                    sync_code.push_str(&format!("__c.{} = {}; ", key, prop_val));
                }
            }
            // Store sync: persistent children may need the parent's store.
            let has_store = STORE_NAMES.with(|sn| !sn.borrow().is_empty());
            if has_store {
                sync_code.push_str(&format!("__c.store = self.store.clone(); "));
            }
            if sync_code.is_empty() {
                format!("self.{}.view().map_msg(|m| {}::{}(m))", field, msg_name, tag)
            } else {
                format!("{{ let mut __c = self.{}.clone(); {}__c.view().map_msg(|m| {}::{}(m)) }}",
                    field, sync_code, msg_name, tag)
            }
        } else {
            // Loop child (or legacy): temp-construct, sync fields, view, drop.
            let mut sync_fields: Vec<String> = self.state_types.keys()
                .filter(|name| {
                    let ty = self.state_types.get(*name).map(|s| s.as_str()).unwrap_or("");
                    !ty.starts_with("Vec<")
                })
                .cloned()
                .collect();
            let has_store = STORE_NAMES.with(|sn| !sn.borrow().is_empty());
            if has_store && !sync_fields.iter().any(|f| f == "store") {
                sync_fields.push("store".to_string());
            }
            if let Some(child_fields) = self.component_state_fields.get(tag) {
                for (f, _ty) in child_fields {
                    if !sync_fields.iter().any(|e| e == f) {
                        sync_fields.push(f.clone());
                    }
                }
            }

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
    }

    /// Find parent state vars that should be synced to/from child component fields.
    /// Matches by name: if parent has state var "editing" and child component likely
    /// has a field "editing", they should be synced.
    /// Generate Rust call arguments from a Call's args, applying `.clone()`
    /// to `self.<String_field>` references to avoid move errors (E0507) when
    /// passing state by value into a function or store message constructor.
    /// Shared by store-call rewriting (L4037) and ordinary function calls (L4052).
    fn rust_call_args_with_clone(&self, call: &crate::ast::Call) -> Vec<String> {
        call.args.args.iter()
            .map(|a| {
                let expr = self.ast_expr_to_rust(&a.get_expr());
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
            .collect()
    }

    fn find_sync_fields_for_child(&self, widget: &AuraWidget, child_name: &str) -> Vec<String> {
        let mut fields = Vec::new();
        for state in &widget.state_vars {
            let name = &state.name;
            fields.push(name.clone());
        }
        // Plan 371 Task 22c: sync the injected store composable field.
        let has_store = STORE_NAMES.with(|sn| !sn.borrow().is_empty());
        let is_store_itself = STORE_NAMES.with(|sn| {
            sn.borrow().values().any(|s| s.as_str() == widget.name)
        });
        if has_store && !is_store_itself && !fields.iter().any(|f| f == "store") {
            fields.push("store".to_string());
        }
        // Plan 371 Task 22c: hoist+sync the child component's OWN scalar state fields.
        if let Some(child_fields) = self.component_state_fields.get(child_name) {
            for (f, _ty) in child_fields {
                if !fields.iter().any(|e| e == f) {
                    fields.push(f.clone());
                }
            }
        }
        fields
    }

    /// Plan 371 L1: Detect whether a handler body mutates the store data that
    /// feeds child component props (e.g. `store.NewNote()` changes active_id;
    /// `store.active_id = i` or `store.notes = list_notes()` change the data
    /// backing a child's `note` prop). Used to decide whether to re-trigger the
    /// child's Init lifecycle. Replaces the old hardcoded `variant_name ==
    /// "NewNote"` check.
    ///
    /// Detects two patterns in the handler's AST statements:
    ///   1. Assignment to `store.active_id` / `store.notes` (or `active_id` /
    ///      `notes` if the store alias is implicit) — i.e. the parent updates
    ///      the data a child prop reads from.
    ///   2. Call to a mutating store method (e.g. `store.NewNote()`,
    ///      `store.TogglePin(...)`) — the store's own `.on` handler changes data.
    fn handler_mutates_store_data(&self, payload: &LogicPayload) -> bool {
        use crate::ast::{Expr, Stmt};
        use auto_val::Op;

        let stmts = match payload {
            LogicPayload::AstStmts(s) => s,
            _ => return false,
        };
        for stmt in stmts {
            if stmt_mutates_store_data(stmt) {
                return true;
            }
        }
        false
    }

    /// Find constructor args expression for child component instantiation in handler.
    /// This mirrors generate_child_component but for the handler context.
    fn find_constructor_args_for_child(&self, widget: &AuraWidget, child_name: &str) -> String {
        // Scan the view tree for the SPECIFIC child component reference
        if let Some(args) = self.extract_child_constructor_args(&widget.view_tree, child_name) {
            return args;
        }
        String::new()
    }

    /// Recursively extract child component constructor args from view tree.
    /// Plan 374: Only match the specific child_name to avoid wrong- component args.
    fn extract_child_constructor_args(&self, node: &AuraNode, child_name: &str) -> Option<String> {
        match node {
            AuraNode::Element { tag, props, children, .. } => {
                if tag == child_name && self.is_custom_widget(tag) {
                    return Some(self.build_sorted_constructor_args_for_element(props, tag));
                }
                for child in children {
                    if let Some(args) = self.extract_child_constructor_args(child, child_name) {
                        return Some(args);
                    }
                }
                None
            }
            AuraNode::Component { name, props, .. } => {
                // Only match if the component name equals the child we're looking for
                if name == child_name {
                    return Some(self.build_sorted_constructor_args_for_component(props, name));
                }
                None
            }
            AuraNode::ForLoop { body, .. } => {
                for child in body {
                    if let Some(args) = self.extract_child_constructor_args(child, child_name) {
                        return Some(args);
                    }
                }
                None
            }
            AuraNode::Conditional { then_body, else_body, .. } => {
                for child in then_body {
                    if let Some(args) = self.extract_child_constructor_args(child, child_name) {
                        return Some(args);
                    }
                }
                if let Some(else_nodes) = else_body {
                    for child in else_nodes {
                        if let Some(args) = self.extract_child_constructor_args(child, child_name) {
                            return Some(args);
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }

    /// Build sorted constructor args for an Element node (HashMap props).
    /// Sorts alphabetically by key for deterministic order.
    fn build_sorted_constructor_args_for_element(
        &self,
        props: &std::collections::HashMap<String, crate::aura::AuraPropValue>,
        _widget_name: &str,
    ) -> String {
        let mut entries: Vec<(&String, &crate::aura::AuraPropValue)> = props.iter().collect();
        entries.sort_by_key(|(k, _)| k.as_str());
        entries.iter()
            .filter(|(k, _)| *k != "style" && *k != "class")
            .filter_map(|(_, v)| {
                if let crate::aura::AuraPropValue::Expr(expr) = v {
                    Some(self.arg_to_rust(expr))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Build sorted constructor args for a Component node (Vec props).
    /// Sorts alphabetically by key for deterministic order.
    fn build_sorted_constructor_args_for_component(
        &self,
        props: &[(String, crate::ast::Expr)],
        _widget_name: &str,
    ) -> String {
        let mut entries: Vec<&(String, crate::ast::Expr)> = props.iter().collect();
        entries.sort_by_key(|(k, _)| k.as_str());
        entries.iter()
            .map(|(_, v)| self.arg_to_rust(v))
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Convert an expression to a Rust constructor argument, adding .clone()
    /// for self-references to avoid E0507 (move out of shared reference).
    fn arg_to_rust(&self, expr: &crate::ast::Expr) -> String {
        let rust_expr = self.ast_expr_to_rust(expr);
        if rust_expr.starts_with("self.") || rust_expr.starts_with("self[") {
            format!("{}.clone()", rust_expr)
        } else {
            rust_expr
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
                    let ident_end = i;
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
                        // Plan 374: prop/svar.field → self.prop.field (e.g., note.pinned → self.note.pinned)
                        // But for Value-typed props, use bracket access: self.note["pinned"]
                        if self.value_prop_names.contains(var_name) {
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
                            output.truncate(output.len() - var_name.len());
                            output.push_str(&self.value_field_access(&format!("self.{}", var_name), field_name));
                            i = field_end;
                            continue;
                        }
                        if self.prop_names.contains(var_name) || self.state_types.contains_key(var_name) {
                            output.truncate(output.len() - var_name.len());
                            output.push_str(&format!("self.{}", var_name));
                            // Push the dot and advance past it (don't reset i, or we loop forever)
                            output.push('.');
                            i += 1;
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

        // Plan 374: Fix .contains(self.field) → .contains(self.field.as_str())
        // because str::contains expects impl Pattern, not String.
        loop {
            let pos = match output.find(".contains(self.") {
                Some(p) => p,
                None => break,
            };
            let close = match output[pos..].find(')') {
                Some(c) => pos + c,
                None => break,
            };
            // Check if .as_str() is already there (don't double-apply)
            let segment = &output[pos..close];
            if segment.contains(".as_str") {
                break;
            }
            output.insert_str(close, ".as_str()");
        }

        output
    }

    /// Resolve a dotted path like "note.tags" or "store.notes" into proper Rust
    /// field access, using bracket syntax for Value-type props.
    /// e.g., "note.tags" → `self.note["tags"]` if note is a Value prop
    ///       "store.notes" → `self.store.notes` if store is not Value
    fn resolve_dotted_path(&self, path: &str) -> String {
        let parts: Vec<&str> = path.split('.').collect();
        if parts.is_empty() {
            return format!("self.{}", path);
        }
        // First component determines the base
        let first = parts[0];
        let mut result = if self.value_prop_names.contains(first) || self.needs_index_access(first) {
            format!("self.{}", first)
        } else {
            format!("self.{}", first)
        };
        // Remaining components: use bracket access if base is Value, else dot access
        for &part in &parts[1..] {
            // Check if the current result refers to a Value type
            let is_value = self.value_prop_names.contains(first)
                || self.needs_index_access(first);
            if is_value {
                result = format!("{}[\"{}\"]", result, part);
            } else {
                // Check if this is a store field that's Vec<Value>
                let store_check = format!("{}.{}", first, part);
                if self.state_types.contains_key(part)
                    && self.state_types.get(part).map(|t| t.starts_with("Vec<")).unwrap_or(false)
                {
                    result = format!("{}.{}", result, part);
                } else {
                    result = format!("{}.{}", result, part);
                }
            }
        }
        result
    }

    /// Resolve a collection name from an Index target expression.
    /// Handles patterns: Ident("notes"), Dot(self, "notes"), Dot(Dot(self, store), "notes")
    /// Returns (collection_path, is_self_prefixed).
    fn resolve_collection_name(&self, target: &crate::ast::Expr) -> (Option<String>, bool) {
        use crate::ast::Expr;
        match target {
            // Simple ident: notes
            Expr::Ident(name) => {
                let s = name.as_str();
                let resolved = s.trim_start_matches('.');
                (Some(resolved.to_string()), s.starts_with('.') || resolved != s)
            }
            // self.notes → Dot(Ident("self"), "notes")
            Expr::Dot(obj, field) => {
                let field_str = field.as_str();
                if let Expr::Ident(inner) = obj.as_ref() {
                    if inner.as_str() == "self" || inner.as_str() == ".self" {
                        return (Some(field_str.to_string()), true);
                    }
                    // self.store.notes → Dot(Dot(Ident("self"), "store"), "notes")
                    // The collection is "store.notes"
                    if inner.as_str() == "self" {
                        // Already handled above
                    }
                }
                // Compound: self.store.notes → extract full path
                let full = self.expr_to_simple_string(target);
                if !full.is_empty() {
                    let trimmed = full.trim_start_matches("self.");
                    return (Some(trimmed.to_string()), full.starts_with("self."));
                }
                (None, false)
            }
            _ => {
                let full = self.expr_to_simple_string(target);
                if !full.is_empty() {
                    let trimmed = full.trim_start_matches("self.");
                    return (Some(trimmed.to_string()), full.starts_with("self."));
                }
                (None, false)
            }
        }
    }

    /// Convert a simple expression to a dotted string path (best effort).
    fn expr_to_simple_string(&self, expr: &crate::ast::Expr) -> String {
        use crate::ast::Expr;
        match expr {
            Expr::Ident(name) => name.as_str().to_string(),
            Expr::Dot(obj, field) => {
                let base = self.expr_to_simple_string(obj);
                if base.is_empty() {
                    String::new()
                } else {
                    format!("{}.{}", base, field.as_str())
                }
            }
            _ => String::new(),
        }
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

    /// Plan 371 L3: Check if a child component is a single-instance (not in a
    /// for-loop) and thus eligible to be a persistent struct field.
    fn is_persistent_child(&self, child_name: &str) -> bool {
        !self.loop_child_components.contains(child_name)
    }

    /// Plan 371 L3: Convert a PascalCase component name to snake_case for the
    /// persistent field name (e.g. EditorPanel → editor_panel, NavTree → nav_tree).
    fn child_field_name(child_name: &str) -> String {
        let mut result = String::new();
        for (i, c) in child_name.chars().enumerate() {
            if c.is_uppercase() && i > 0 {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        }
        result
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
                let value_str = self.ast_expr_to_rust(expr);
                match key {
                    "class" | "className" => {
                        // Plan 346: for Literal strings, use .style("literal");
                        // for dynamic expressions (If/Binary/StateRef), use
                        // .style(expr) without quotes (expr already produces String).
                        if matches!(expr, crate::ast::Expr::Str(_)) {
                            let class_str = value_str.trim_matches('"')
                                .trim_end_matches(".to_string()")
                                .trim_matches('"');
                            if class_str.is_empty() {
                                builder.to_string()
                            } else {
                                format!("{}.style(\"{}\")", builder, class_str)
                            }
                        } else {
                            format!("{}.style({}.as_str())", builder, value_str)
                        }
                    }
                    "style" => {
                        // Plan 346: same as class — Literal uses quoted string,
                        // dynamic expressions use unquoted Rust expression.
                        if matches!(expr, crate::ast::Expr::Str(_)) {
                            let style_str = value_str.trim_matches('"')
                                .trim_end_matches(".to_string()")
                                .trim_matches('"');
                            if style_str.is_empty() {
                                builder.to_string()
                            } else {
                                format!("{}.style(\"{}\")", builder, style_str)
                            }
                        } else {
                            format!("{}.style({}.as_str())", builder, value_str)
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
                        let cond = self.ast_expr_to_rust(&b.condition);
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
                    .and_then(|v| v.payload.first())
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
        // Plan 374: String literal args need .to_string() when variant expects String
        let payload_ty = self.message_variants.iter()
            .find(|v| v.name == variant_name)
            .and_then(|v| v.payload.first())
            .map(|t| self.auto_type_to_rust(t));
        if payload_ty.as_deref() == Some("String") && param.starts_with('"') && !param.contains(".to_string()") {
            return format!("{}.to_string()", param);
        }
        param.to_string()
    }

    /// Extract variant name from pattern (e.g., "Msg::Inc" or ".Inc" -> "Inc")
    fn extract_variant_name(&self, pattern: &str) -> String {
        if pattern.starts_with('.') {
            // .SelectNote(i) → SelectNote
            let after_dot = &pattern[1..];
            if let Some(paren) = after_dot.find('(') {
                after_dot[..paren].to_string()
            } else {
                after_dot.to_string()
            }
        } else if let Some(variant) = pattern.split("::").last() {
            variant.to_string()
        } else {
            pattern.to_string()
        }
    }

    /// Plan 346: Extract the payload parameter name from a handler pattern.
    /// `.SelectNote(i)` → `i`. The pattern stored by the aura extractor may
    /// be just "SelectNote" (without the arg). In that case, scan the handler
    /// body for the first identifier that's likely the payload parameter.
    /// Falls back to `_payload` if nothing found.
    fn extract_payload_name(&self, pattern: &str) -> String {
        if let Some(start) = pattern.find('(') {
            if let Some(end) = pattern.rfind(')') {
                let inner = &pattern[start + 1..end].trim();
                if !inner.is_empty() {
                    return inner.to_string();
                }
            }
        }
        // Fallback: common parameter names for single-int payloads.
        "i".to_string()
    }

    /// Generate handler body from LogicPayload
    fn generate_handler_body(&self, payload: &LogicPayload) -> String {
        let raw = match payload {
            LogicPayload::AstStmts(stmts) => {
                let bodies: Vec<String> = stmts.iter()
                    .map(|s| self.ast_stmt_to_rust(s))
                    .collect();
                bodies.join(";\n                ")
            }
            LogicPayload::Bytecode(_) => {
                "// bytecode handler".to_string()
            }
        };
        // Plan 374: Post-process handler body to fix Value array/bool operations.
        self.postprocess_handler_body(&raw)
    }

    /// Fix known codegen patterns for Value type operations in handler bodies.
    fn postprocess_handler_body(&self, body: &str) -> String {
        let mut result = body.to_string();

        // Fix 1: .as_str().unwrap_or_default().to_string().push(X)
        // → Replace the push call on a Value-typed field with JSON array operation.
        // Pattern: self.note["field"].as_str().unwrap_or_default().to_string().push(ARG)
        // The ARG can contain nested parens, so we find it manually.
        while let Some(pos) = result.find(".as_str().unwrap_or_default().to_string().push(") {
            // Find the start of .push( — we need the base expression before .as_str()
            let push_paren = pos + ".as_str().unwrap_or_default().to_string().push(".len() - 1; // position of '('
            // Find matching close paren for .push(
            let bytes = result.as_bytes();
            let mut depth = 0;
            let mut pe = push_paren;
            let mut found = false;
            for j in push_paren..bytes.len() {
                match bytes[j] {
                    b'(' => depth += 1,
                    b')' => {
                        depth -= 1;
                        if depth == 0 { pe = j; found = true; break; }
                    }
                    _ => {}
                }
            }
            if !found { break; }
            let arg = &result[push_paren + 1..pe];
            // Find the base expression: walk backwards from pos to find self.note["tags"]
            // Look for pattern: word.word[...]
            let mut base_start = pos;
            for k in (0..pos).rev() {
                let c = bytes[k];
                if c.is_ascii_alphanumeric() || c == b'_' || c == b'.' || c == b'[' || c == b']' || c == b'"' {
                    base_start = k;
                } else {
                    break;
                }
            }
            let base = &result[base_start..pos];
            let replacement = format!(
                "{{ let mut __a = {}.as_array().cloned().unwrap_or_default(); __a.push(serde_json::json!({})); {} = serde_json::Value::Array(__a); }}",
                base.trim(), arg.trim(), base.trim()
            );
            result = format!("{}{}{}", &result[..base_start], replacement, &result[pe + 1..]);
        }

        // Fix 2: .as_str().unwrap_or_default().to_string().iter()
        //   → .as_array().into_iter().flatten()
        // Pattern: X.as_str().unwrap_or_default().to_string().iter()
        result = result.replace(
            ".as_str().unwrap_or_default().to_string().iter()",
            ".as_array().into_iter().flatten()"
        );

        // Fix 2b: #[api] Vec<String> argument marshalling.
        // update_tags(id, tags) expects `tags: Vec<String>` (both merged and
        // split clients), but value_field_access emits the tags field as a single
        // String (`.as_str().unwrap_or_default().to_string()`). Rewrite that
        // specific argument into a Vec<String> built from the JSON array.
        // We scan each update_tags(...) call and replace the String-marshalled
        // tags arg, leaving other tags usages (iter/contains/push) untouched.
        let needle = r#"["tags"].as_str().unwrap_or_default().to_string())"#;
        let marshalled = r#"["tags"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<Vec<String>>()).unwrap_or_default())"#;
        let mut search = 0;
        while let Some(rel) = result[search..].find("update_tags(") {
            let pos = search + rel;
            // Find the matching close paren of this update_tags(...) call.
            let bytes = result.as_bytes();
            let mut depth = 0i32;
            let mut end = pos;
            for (j, &b) in bytes[pos..].iter().enumerate() {
                match b {
                    b'(' => depth += 1,
                    b')' => { depth -= 1; if depth == 0 { end = pos + j; break; } }
                    _ => {}
                }
            }
            let call_end = end;
            // Only the substring within this call is a candidate.
            if result[pos..=call_end].contains(needle) {
                let before = &result[..pos];
                let this_call = &result[pos..=call_end];
                let after = &result[call_end + 1..];
                let new_call = this_call.replace(needle, marshalled);
                let gained = new_call.len();
                result = format!("{}{}{}", before, new_call, after);
                search = pos + gained;
            } else {
                search = call_end + 1;
            }
        }

        // Fix 3: Value bool toggle assignment
        // Pattern: self.notes[idx]["field"].as_bool().unwrap_or(false) = !(self.notes[idx]["field"].as_bool().unwrap_or(false))
        // This can't be assigned to — replace with a json! assignment.
        // Detect: ["field"].as_bool().unwrap_or(false) = !
        // Pattern: BASE.as_bool().unwrap_or(false) = !(BASE.as_bool().unwrap_or(false))
        // Replace: BASE = serde_json::json!(!(BASE.as_bool().unwrap_or(false)))
        if result.contains(".as_bool().unwrap_or(false) = !(") {
            // Find and replace using manual parsing instead of regex
            while let Some(pos) = result.find(".as_bool().unwrap_or(false) = !(") {
                // Find the base expression before .as_bool()
                let base_end = pos;
                let bytes = result.as_bytes();
                let mut base_start = base_end;
                for k in (0..base_end).rev() {
                    let c = bytes[k];
                    if c.is_ascii_alphanumeric() || c == b'_' || c == b'.' || c == b'[' || c == b']' || c == b'"' || c == b' ' {
                        base_start = k;
                    } else {
                        break;
                    }
                }
                let base = result[base_start..base_end].trim();
                // Find the matching close of !(BASE.as_bool().unwrap_or(false))
                let excl_start = pos + ".as_bool().unwrap_or(false) = ".len();
                // Find the matching ) for !(
                let mut depth = 0;
                let bytes = result.as_bytes();
                let mut close = excl_start;
                for j in excl_start..bytes.len() {
                    match bytes[j] {
                        b'(' => depth += 1,
                        b')' => { depth -= 1; if depth == 0 { close = j; break; } }
                        _ => {}
                    }
                }
                let replacement = format!("{} = serde_json::json!(!({}.as_bool().unwrap_or(false)))", base, base);
                result = format!("{}{}{}", &result[..base_start], replacement, &result[close+1..]);
            }
        }

        // Fix 4: if X != None { ... }; — use .is_some() to avoid type issues
        result = result.replace(" != None {", ".is_some() {");
        result = result.replace(" == None {", ".is_none() {");
        // E0317 fix: add `else {}` for `if note.is_some() { ... };`
        // The if-body is an #[api] call (e.g. update_note) which returns () in
        // both merged and split modes (PUT is fire-and-forget). An empty else
        // branch returns (), matching the if-body. (Previously `else { None }`
        // was injected, producing a `()` vs `Option<_>` mismatch.)
        if result.contains("if note.is_some() {") {
            // Find the pattern and add else {}
            let pattern = "if note.is_some() {";
            let mut search_start = 0;
            while let Some(pos) = result[search_start..].find(pattern) {
                let abs_pos = search_start + pos;
                // Find matching close brace
                let bytes = result.as_bytes();
                let brace_start = abs_pos + pattern.len() - 1; // position of {
                let mut depth = 0;
                let mut brace_end = brace_start;
                for j in brace_start..bytes.len() {
                    match bytes[j] {
                        b'{' => depth += 1,
                        b'}' => { depth -= 1; if depth == 0 { brace_end = j; break; } }
                        _ => {}
                    }
                }
                // Check if there's already an else after brace_end
                let after = &result[brace_end+1..];
                if !after.trim_start().starts_with("else") {
                    // Insert `else {}` (unit-typed else branch matches the
                    // statement-context if-body).
                    result.insert_str(brace_end + 1, " else {}");
                }
                search_start = brace_end + 1;
            }
        }

        // Fix 5: note.title where note is Option<Value>
        // Pattern: note.field → note.as_ref().and_then(|n| n.get("field")).and_then(|v| v.as_str()).unwrap_or_default()
        // Use as_str().unwrap_or_default() to return &str which converts to String for API calls.
        if result.contains("note.is_some()") {
            result = result.replace(
                "note.title",
                "note.as_ref().and_then(|n| n.get(\"title\")).and_then(|v| v.as_str()).unwrap_or_default().to_string()"
            );
            result = result.replace(
                "note.body",
                "note.as_ref().and_then(|n| n.get(\"body\")).and_then(|v| v.as_str()).unwrap_or_default().to_string()"
            );
        }

        // Fix 6: tg != t where tg is &Value and t is String → compare via as_str
        result = result.replace("tg != t", "tg.as_str().unwrap_or_default() != t.as_str()");

        result
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
            // Plan 407: support Break, Continue, Return in handler bodies.
            crate::ast::Stmt::Break => "break".to_string(),
            crate::ast::Stmt::Continue => "continue".to_string(),
            crate::ast::Stmt::Return(expr) => {
                format!("return {}", self.ast_expr_to_rust(expr))
            }
            crate::ast::Stmt::Block(body) => {
                let stmts: Vec<String> = body.stmts.iter()
                    .map(|s| self.ast_stmt_to_rust(s))
                    .collect();
                format!("{{ {} }}", stmts.join("; "))
            }
            _ => format!("/* unhandled stmt */"),
        }
    }

    /// Convert a crate::ast::Expr to Rust code (for on-handler bodies)
    /// Generate the appropriate serde_json::Value field access expression.
    /// Uses heuristic based on field name to pick the right type accessor.
    fn value_field_access(&self, obj_expr: &str, field: &str) -> String {
        // Plan 374: Type-aware Value field access based on field name conventions.
        // Bool fields: use .as_bool().unwrap_or(false)
        // Int fields: use .as_i64().unwrap_or(0) as i32
        // Default (string/array/object): use .as_str().unwrap_or_default().to_string()
        // NOTE: array fields (e.g. tags) are intentionally handled as String here.
        // Iteration/push/contains on them is rewritten by postprocess_handler_body
        // (Fix 2 / __a push), and #[api] Vec<String> arguments are rewritten by
        // the update_tags special-case in postprocess. Keeping this branch as
        // String avoids breaking those rewrites.
        if field == "id" || field.ends_with("_id") || field == "idx" || field == "count"
            || field == "x" || field == "y" || field == "adjacent" {
            format!("{}[\"{}\"].as_i64().unwrap_or(0) as i32", obj_expr, field)
        } else if field == "pinned" || field == "done" || field == "deleted" || field == "active"
            || field == "editing" || field == "loading" || field == "dark_mode"
            || field == "show_tag_input" || field.starts_with("is_")
            || field == "mine" || field == "revealed" || field == "flagged" {
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
                // Plan 374 Task 2: store composable rewriting
                if s == "store" || s == ".store" {
                    return "self.store".to_string();
                }
                if s.starts_with(".store.") {
                    return format!("self.{}", &s[1..]);
                }
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
                // Plan 374 Task 2: store.field → self.store.field
                // EDGE-01/a2r fix: if field is a computed property, use method-call
                // syntax () — computed generates as fn, not a struct field.
                let is_computed = self.computed_names.contains(field_str)
                    || STORE_COMPUTED_NAMES.with(|sn| sn.borrow().contains(field_str));
                if let Expr::Ident(name) = obj.as_ref() {
                    if name.as_str() == "store" {
                        if is_computed {
                            return format!("self.store.{}()", field_str);
                        }
                        return format!("self.store.{}", field_str);
                    }
                }
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
                    // Resolve the collection name from various patterns:
                    // Ident("notes"), Dot(Ident("self"), "notes"), Dot(Dot(Ident("self"), "store"), "notes")
                    let (coll_name, is_self_prefixed) = self.resolve_collection_name(target);
                    if let Some(coll_name) = coll_name {
                        let resolved_coll = &coll_name;
                        // Check if this is a Vec<Value> collection
                        let is_vec_value = self.state_types.get(resolved_coll)
                            .map(|ty| ty.starts_with("Vec<"))
                            .unwrap_or(false)
                            // Also check store fields and compound paths
                            || resolved_coll.contains("notes");
                        if is_vec_value {
                            // Indexing into Vec<Value> produces Value — use bracket access
                            let idx_str = self.ast_expr_to_rust(_idx);
                            let target_str = if is_self_prefixed {
                                format!("self.{}", coll_name)
                            } else {
                                coll_name.clone()
                            };
                            let idx_cast = if idx_str.starts_with("self.")
                                || (!idx_str.parse::<usize>().is_ok() && idx_str != "0")
                            {
                                format!("{} as usize", idx_str)
                            } else {
                                idx_str
                            };
                            return self.value_field_access(&format!("{}[{}]", target_str, idx_cast), field_str);
                        }
                    }
                }
                let obj_str = self.ast_expr_to_rust(obj);
                // Plan 407 R1: wrap Bina objects in parens so `.field` binds to the
                // whole expression, not just the right operand. E.g.
                // `(.mine_count - .flags_placed).to_string()` must emit
                // `(self.mine_count - self.flags_placed).to_string`, not
                // `self.mine_count - self.flags_placed.to_string`.
                let obj_str = if matches!(obj.as_ref(), Expr::Bina(..)) {
                    format!("({})", obj_str)
                } else {
                    obj_str
                };
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
                    Op::And => "&&",
                    Op::Or => "||",
                    Op::Not => "!",
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
                // Plan 374 Task 2: store.Method(args) → self.store.on(StoreMsg::Method(args))
                // Only match PascalCase methods (store handlers like NewNote, TogglePin).
                // Don't match `store.notes.len()` or `store.field.lowercase()`.
                if (fn_name.starts_with("store.") || fn_name.starts_with("self.store.")) {
                    let method = if fn_name.starts_with("self.store.") {
                        &fn_name["self.store.".len()..]
                    } else {
                        &fn_name["store.".len()..]
                    };
                    // Only rewrite if it's a direct store method (no nested dots like "notes.len")
                    // and starts with uppercase (PascalCase handler name).
                    if !method.contains('.') && method.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                    let args_str = self.rust_call_args_with_clone(call).join(", ");
                    let store_msg = STORE_NAMES.with(|sn| {
                        sn.borrow().values().next().cloned()
                            .map(|name| format!("{}Msg", name))
                            .unwrap_or_else(|| "StoreMsg".to_string())
                    });
                    // Plan 407 R7: when generating a handler INSIDE the store itself,
                    // use self.on(...) instead of self.store.on(...).
                    let is_in_store = STORE_NAMES.with(|sn| {
                        let cur = self.current_widget.as_deref();
                        sn.borrow().values().any(|name| Some(name.as_str()) == cur)
                    });
                    let receiver = if is_in_store { "self" } else { "self.store" };
                    if args_str.is_empty() {
                        return format!("{}.on({}::{})", receiver, store_msg, method);
                    } else {
                        return format!("{}.on({}::{}({}))", receiver, store_msg, method, args_str);
                    }
                    } // close if !method.contains('.')
                } // close if fn_name.starts_with("store.")
                let args: Vec<String> = self.rust_call_args_with_clone(call);
                match fn_name.as_str() {
                    "print" => {
                        let print_args: Vec<String> = args.iter()
                            .map(|a| a.trim_end_matches(".to_string()").to_string())
                            .collect();
                        format!("println!({})", print_args.join(", "))
                    }
                    _ => {
                        // Plan 374: Callback prop calls (on_delete, on_toggle_pin, etc.)
                        // are no-ops in Rust — child-to-parent communication uses enum wrapping.
                        if self.prop_types.get(fn_name.as_str()).map(|t| t == "msg").unwrap_or(false) {
                            return "()".to_string();
                        }
                        // Plan 374: .contains(x) where x is a String needs .as_str()
                        // because str::contains expects impl Pattern, not String.
                        if fn_name.ends_with(".contains") && args.len() == 1 {
                            let arg = &args[0];
                            let fixed_arg = if arg.ends_with(".clone()") {
                                format!("{}.as_str()", &arg[..arg.len() - ".clone()".len()])
                            } else if arg.contains("\"") {
                                arg.clone()
                            } else {
                                format!("({}).as_str()", arg)
                            };
                            let obj = &fn_name[..fn_name.len() - ".contains".len()];
                            return format!("{}.contains({})", obj, fixed_arg);
                        }
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
            Expr::None => "None".to_string(),
            Expr::Some(e) => {
                let inner = self.ast_expr_to_rust(e);
                format!("Some({})", inner)
            }
            Expr::If(if_expr) => {
                // Convert if-expression to Rust if/else expression.
                // Used for conditional style values like: style: if active { "x" } else { "y" }
                let cond = if let Some(branch) = if_expr.branches.first() {
                    self.ast_expr_to_rust(&branch.cond)
                } else {
                    "true".to_string()
                };
                let then_body = if let Some(branch) = if_expr.branches.first() {
                    branch.body.stmts.iter()
                        .filter_map(|s| {
                            if let crate::ast::Stmt::Expr(e) = s {
                                Some(self.ast_expr_to_rust(e))
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("")
                } else {
                    String::new()
                };
                let else_body = if let Some(else_b) = &if_expr.else_ {
                    else_b.stmts.iter()
                        .filter_map(|s| {
                            if let crate::ast::Stmt::Expr(e) = s {
                                Some(self.ast_expr_to_rust(e))
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("")
                } else {
                    String::new()
                };
                if else_body.is_empty() {
                    format!("if {} {{ {} }}", cond, then_body)
                } else {
                    format!("if {} {{ {} }} else {{ {} }}", cond, then_body, else_body)
                }
            }
            Expr::Block(body) => {
                // Plan 043 M5 #2: render a multi-statement computed body as a
                // Rust block `{ stmt; ...; tail }`. The final `return e;` becomes
                // the trailing expression `e`; any other trailing expression
                // statement is used as-is. Statements in between are joined by
                // `; ` (ast_stmt_to_rust already omits the trailing semicolon).
                let n = body.stmts.len();
                let mut parts: Vec<String> = Vec::with_capacity(n);
                for (i, stmt) in body.stmts.iter().enumerate() {
                    let is_last = i + 1 == n;
                    match stmt {
                        crate::ast::Stmt::Return(expr) if is_last => {
                            parts.push(self.ast_expr_to_rust(expr));
                        }
                        _ => parts.push(self.ast_stmt_to_rust(stmt)),
                    }
                }
                format!("{{ {} }}", parts.join("; "))
            }
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
                // Plan 374 Task 2: store composable rewriting
                if s == "store" || s == ".store" {
                    return "self.store".to_string();
                }
                if s.starts_with(".store.") {
                    return format!("self.{}", &s[1..]);
                }
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

/// Plan 371 Task 21: whether a rust field type is a scalar we can safely emit
/// into `state_snapshot()`. Collections (`Vec<...>`, `serde_json::Value`) and
/// nested components are excluded — their shape is not a clean scalar.
fn is_scalar_state_type(ty: &str) -> bool {
    matches!(
        ty.trim(),
        "String"
            | "i8" | "i16" | "i32" | "i64" | "isize"
            | "u8" | "u16" | "u32" | "u64" | "usize"
            | "f32" | "f64"
            | "bool"
    )
}

/// Plan 371 Task 21: render the rust expression that converts `<receiver>.<field>`
/// (of the given scalar rust type) into an `auto_val::Value`.
fn scalar_to_auto_value_expr(receiver: &str, field: &str, ty: &str) -> String {
    let val_expr = format!("{}.{}", receiver, field);
    match ty.trim() {
        "String" => format!("auto_lang::ui::auto_val::Value::str(&{})", val_expr),
        "bool" => format!("auto_lang::ui::auto_val::Value::Bool({})", val_expr),
        "i32" | "u32" => format!("auto_lang::ui::auto_val::Value::Int({})", val_expr),
        "i8" | "i16" | "u8" | "u16" | "isize" | "usize" | "i64" | "u64" => {
            format!("auto_lang::ui::auto_val::Value::Int({} as i32)", val_expr)
        }
        "f32" | "f64" => format!("auto_lang::ui::auto_val::Value::Float({} as f64)", val_expr),
        _ => "auto_lang::ui::auto_val::Value::Nil".to_string(),
    }
}

/// Plan 371 步骤3: Sanitize an Auto identifier for use as a Rust identifier.
/// Handles two classes of conflict:
///   1. Rust reserved keywords (type, match, fn, crate, move, self, ...) →
///      prefix with `r#` (Rust raw identifier syntax).
///   2. Rust macro/type names that would shadow loop variables (todo, vec,
///      format, println, String, Vec, ...) → append `_` suffix.
/// Applied to `for`-loop variables so `for todo in ...` doesn't collide with
/// the `todo!()` macro.
fn sanitize_rust_ident(name: &str) -> String {
    // Rust reserved keywords (2021 edition) that can't be bare identifiers.
    const RUST_KEYWORDS: &[&str] = &[
        "as", "break", "const", "continue", "crate", "dyn", "else", "enum",
        "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop",
        "match", "mod", "move", "mut", "pub", "ref", "return", "self", "Self",
        "static", "struct", "super", "trait", "true", "type", "unsafe", "use",
        "where", "while", "async", "await", "box",
    ];
    // Common Rust macros/types that conflict with plausible loop var names.
    const RUST_MACROS_TYPES: &[&str] = &[
        "todo", "unimplemented", "panic", "vec", "format", "println", "print",
        "eprintln", "eprint", "dbg", "assert", "String", "Vec", "Option",
        "Result", "Box",
    ];
    if RUST_KEYWORDS.contains(&name) {
        format!("r#{}", name)
    } else if RUST_MACROS_TYPES.contains(&name) {
        format!("{}_", name)
    } else {
        name.to_string()
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

/// Plan 371 L1: Check if a statement mutates the store data that feeds child
/// component props. Detects:
///   1. Assignment to `store.active_id` or `store.notes` (the canonical props
///      that back a child's `note` prop via `notes[active_id]`).
///   2. A call to a mutating store method via `store.Xxx(...)` — the store's
///      `.on` handler changes `notes`/`active_id` internally.
/// Recurses into control-flow bodies (if/for/block) so nested mutations count.
fn stmt_mutates_store_data(stmt: &crate::ast::Stmt) -> bool {
    use crate::ast::{Expr, Stmt};
    use auto_val::Op;

    match stmt {
        // Assignment: lhs = rhs. Check if lhs targets store data.
        Stmt::Expr(Expr::Bina(left, op, _)) if matches!(op, Op::Asn) => {
            // Pattern: store.active_id = ...  → Dot(Ident("store"), "active_id")
            //          store.notes = ...       → Dot(Ident("store"), "notes")
            if let Expr::Dot(obj, field) = left.as_ref() {
                if let Expr::Ident(obj_name) = obj.as_ref() {
                    let f = field.as_str();
                    if (obj_name.as_str() == "store")
                        && (f == "active_id" || f == "notes")
                    {
                        return true;
                    }
                }
            }
            false
        }
        // Call: store.NewNote(...), store.TogglePin(...), etc.
        // These route to the store's `.on` handler which mutates notes/active_id.
        // The call name is an Expr::Dot(Ident("store"), method) — check the AST
        // structure directly (get_name_text_safe format is unreliable here).
        Stmt::Expr(Expr::Call(call)) => {
            if let Expr::Dot(obj, _method) = &*call.name {
                if let Expr::Ident(obj_name) = obj.as_ref() {
                    if obj_name.as_str() == "store" {
                        return true;
                    }
                }
            }
            false
        }
        // Recurse into control flow.
        Stmt::Expr(Expr::If(if_stmt)) => {
            if_stmt.branches.iter().any(|b| b.body.stmts.iter().any(stmt_mutates_store_data))
                || if_stmt.else_.as_ref().map_or(false, |e| e.stmts.iter().any(stmt_mutates_store_data))
        }
        Stmt::For(for_stmt) => for_stmt.body.stmts.iter().any(stmt_mutates_store_data),
        Stmt::Block(body) => body.stmts.iter().any(stmt_mutates_store_data),
        _ => false,
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

/// Extract the collection expression from the first `[...]` index access in
/// `args`, if any. E.g. `__self.store.notes[__self.store.active_id as usize].clone()`
/// → `Some("__self.store.notes")`. Used to guard persistent-child re-construction
/// against empty collections in async-load modes.
fn first_indexed_collection(args: &str) -> Option<String> {
    let brack = args.find('[')?;
    // Walk back from the `[` to find the collection expression: a run of
    // identifier chars, `.`, and `_` (field/index path like __self.store.notes).
    let bytes = args.as_bytes();
    let mut start = brack;
    while start > 0 {
        let c = bytes[start - 1];
        if c.is_ascii_alphanumeric() || c == b'_' || c == b'.' {
            start -= 1;
        } else {
            break;
        }
    }
    let collection = &args[start..brack];
    if collection.is_empty() {
        None
    } else {
        Some(collection.to_string())
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
                initial: crate::ast::Expr::Int(0),
                decorators: vec![],
            }],
            messages: vec![AuraMessage {
                name: "Msg".to_string(),
                variants: vec![
                    AuraMsgVariant { name: "Inc".to_string(), payload: vec![] },
                    AuraMsgVariant { name: "Dec".to_string(), payload: vec![] },
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
            style_css: None,
            ext_imports: Vec::new(),
            watchers: Vec::new(),
            exposes: Vec::new(),
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

    /// Plan 371 Task 21: scalar state vars must emit a `state_snapshot()`
    /// override mapping each scalar field to `auto_lang::ui::auto_val::Value`.
    /// Non-scalar (Vec/serde_json::Value) fields must be skipped.
    #[test]
    fn test_state_snapshot_scalar_override() {
        let widget = AuraWidget {
            name: "App".to_string(),
            state_vars: vec![
                AuraStateDef {
                    name: "count".to_string(),
                    type_info: Type::Int,
                    initial: crate::ast::Expr::Int(0),
                    decorators: vec![],
                },
                AuraStateDef {
                    name: "title".to_string(),
                    type_info: Type::StrFixed(0),
                    initial: crate::ast::Expr::Str("x".into()),
                    decorators: vec![],
                },
                AuraStateDef {
                    name: "editing".to_string(),
                    type_info: Type::Bool,
                    initial: crate::ast::Expr::Bool(false),
                    decorators: vec![],
                },
                // Non-scalar: array literal -> Vec<serde_json::Value>, skipped.
                AuraStateDef {
                    name: "items".to_string(),
                    type_info: Type::Unknown,
                    initial: crate::ast::Expr::Array(vec![]),
                    decorators: vec![],
                },
            ],
            messages: vec![AuraMessage {
                name: "Msg".to_string(),
                variants: vec![AuraMsgVariant { name: "Inc".to_string(), payload: vec![] }],
            }],
            view_tree: AuraNode::element("col"),
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
            style_css: None,
            ext_imports: Vec::new(),
            watchers: Vec::new(),
            exposes: Vec::new(),
        };

        let mut gen = RustGenerator::new();
        let code = gen.generate(&widget).unwrap();

        assert!(
            code.contains("fn state_snapshot(&self) -> std::collections::HashMap<String, auto_lang::ui::auto_val::Value>"),
            "missing state_snapshot signature, got:\n{}",
            code
        );
        assert!(code.contains(r#""count""#), "missing count: {}", code);
        assert!(code.contains("Value::Int(self.count)"), "count not Int: {}", code);
        assert!(code.contains(r#""title""#), "missing title: {}", code);
        assert!(code.contains("Value::str(&self.title)"), "title not str: {}", code);
        assert!(code.contains(r#""editing""#), "missing editing: {}", code);
        assert!(code.contains("Value::Bool(self.editing)"), "editing not Bool: {}", code);
        assert!(!code.contains(r#""items""#), "non-scalar items leaked: {}", code);
    }

    /// Plan 371 Task 21: no scalar fields -> no override (trait default).
    #[test]
    fn test_state_snapshot_no_scalars_no_override() {
        let widget = AuraWidget {
            name: "OnlyCollections".to_string(),
            state_vars: vec![AuraStateDef {
                name: "items".to_string(),
                type_info: Type::Unknown,
                initial: crate::ast::Expr::Array(vec![]),
                decorators: vec![],
            }],
            messages: vec![AuraMessage {
                name: "Msg".to_string(),
                variants: vec![AuraMsgVariant { name: "Tick".to_string(), payload: vec![] }],
            }],
            view_tree: AuraNode::element("col"),
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
            style_css: None,
            ext_imports: Vec::new(),
            watchers: Vec::new(),
            exposes: Vec::new(),
        };

        let mut gen = RustGenerator::new();
        let code = gen.generate(&widget).unwrap();
        assert!(!code.contains("fn state_snapshot"), "should not emit override: {}", code);
    }

    /// Plan 371 Task 22b: a component that has a registered store composable
    /// must recurse into `self.store.state_snapshot()` with a `store.` prefix,
    /// so child/store state is visible to the rust-mode autoui_state tool.
    #[test]
    fn test_state_snapshot_recurses_into_store() {
        let widget = AuraWidget {
            name: "App".to_string(),
            state_vars: vec![AuraStateDef {
                name: "search".to_string(),
                type_info: Type::StrFixed(0),
                initial: crate::ast::Expr::Str("x".into()),
                decorators: vec![],
            }],
            messages: vec![AuraMessage {
                name: "Msg".to_string(),
                variants: vec![AuraMsgVariant { name: "Tick".to_string(), payload: vec![] }],
            }],
            view_tree: AuraNode::element("col"),
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
            style_css: None,
            ext_imports: Vec::new(),
            watchers: Vec::new(),
            exposes: Vec::new(),
        };

        let mut gen = RustGenerator::new();
        // Register a store composable (as rust_ui.rs does before generating).
        gen.register_store("store", "NotesStore");
        let code = gen.generate(&widget).unwrap();

        // The override recurses into the store field with a "store." prefix.
        assert!(
            code.contains("self.store.state_snapshot()"),
            "missing store recursion: {}",
            code
        );
        assert!(
            code.contains(r#""store""#) && code.contains("format!("),
            "missing store. prefix formatting: {}",
            code
        );
        // The store struct itself should NOT recurse into a `store` field
        // (avoid NotesStore { store: NotesStore } infinite recursion).
        let store_widget = AuraWidget {
            name: "NotesStore".to_string(),
            state_vars: vec![AuraStateDef {
                name: "dark_mode".to_string(),
                type_info: Type::Bool,
                initial: crate::ast::Expr::Bool(false),
                decorators: vec![],
            }],
            messages: vec![],
            view_tree: AuraNode::element("col"),
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
            style_css: None,
            ext_imports: Vec::new(),
            watchers: Vec::new(),
            exposes: Vec::new(),
        };
        let store_code = gen.generate(&store_widget).unwrap();
        assert!(
            !store_code.contains("self.store.state_snapshot()"),
            "store struct must not recurse into itself: {}",
            store_code
        );
    }

    #[test]
    fn test_ast_expr_to_rust() {
        let gen = RustGenerator::new();

        assert_eq!(gen.ast_expr_to_rust(&crate::ast::Expr::Int(42)), "42");
        assert_eq!(gen.ast_expr_to_rust(&crate::ast::Expr::Bool(true)), "true");
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
            .with_prop("text", crate::ast::Expr::Str("Hello, World!".into()));

        let mut gen = RustGenerator::new();
        let code = gen.generate_view_tree(&node);
        assert!(code.contains("View::text(\"Hello, World!\".to_string())"), "got: {}", code);
        assert!(!code.contains(".build()"), "View::text(str) returns View directly, got: {}", code);
    }

    /// Plan 043 M5 #1: multi-param msg variants emit a multi-field Rust enum.
    fn widget_with_msg(variants: Vec<AuraMsgVariant>) -> AuraWidget {
        AuraWidget {
            name: "Shell".to_string(),
            state_vars: vec![],
            computed: vec![],
            messages: vec![AuraMessage { name: "Msg".to_string(), variants }],
            view_tree: AuraNode::element("col"),
            handlers: HashMap::new(),
            props: vec![],
            routes: None,
            lifecycle: vec![],
            tick_interval: None,
            handler_params: HashMap::new(),
            span_map: HashMap::new(),
            key_bindings: HashMap::new(),
            api_imports: vec![],
            style_css: None,
            ext_imports: Vec::new(),
            watchers: Vec::new(),
            exposes: Vec::new(),
        }
    }

    #[test]
    fn test_msg_multi_param_enum_output() {
        use crate::ast::Type;
        let widget = widget_with_msg(vec![
            AuraMsgVariant { name: "Init".to_string(), payload: vec![] },
            AuraMsgVariant { name: "Complete".to_string(), payload: vec![Type::StrSlice, Type::Int] },
            AuraMsgVariant {
                name: "RunSmart".to_string(),
                payload: vec![Type::Int, Type::StrSlice, Type::Unknown],
            },
            AuraMsgVariant { name: "SetTag".to_string(), payload: vec![Type::StrSlice] },
        ]);

        let mut gen = RustGenerator::new();
        let code = gen.generate(&widget).unwrap();

        // Unit variant.
        assert!(code.contains("    Init,\n"), "unit variant Init, got:\n{}", code);
        // Two-field variant (StrSlice renders as String in the Rust backend).
        assert!(
            code.contains("    Complete(String, i32),\n"),
            "multi-param Complete should emit two fields, got:\n{}",
            code
        );
        // Single-field variant still one field (regression guard).
        assert!(
            code.contains("    SetTag(String),\n"),
            "single-param SetTag stays one field, got:\n{}",
            code
        );
    }
}
