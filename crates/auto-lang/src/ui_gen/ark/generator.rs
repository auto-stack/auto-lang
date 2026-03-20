//! ArkTS (HarmonyOS) Generator
//!
//! Main generator that produces ArkTS code from AURA widgets.
//!
//! ## Output Format
//!
//! ```arkts
//! @Entry
//! @Component
//! struct MyWidget {
//!     @State count: number = 0
//!
//!     build() {
//!         Column() {
//!             Button(this.count.toString())
//!                 .onClick(() => {
//!                     this.count++
//!                 })
//!         }
//!     }
//! }
//! ```

use super::components::ArkComponentRegistry;
use super::modifier::{class_to_modifier, prop_to_modifier};
use super::project::ArkProjectGenerator;
use super::state::{generate_dispatch_function, generate_handler_body, generate_msg_enum, generate_state_declarations};
use crate::aura::{AuraExpr, AuraNode, AuraPropValue, AuraTextContent, AuraWidget, LogicPayload};
use crate::ui_gen::{BackendGenerator, GenResult};
use std::collections::HashMap;

/// ArkTS code generator for HarmonyOS
///
/// This is the main entry point for generating ArkTS code from AURA widgets.
///
/// # Architecture
///
/// ```text
/// ArkGenerator
///     ├── ArkComponentRegistry (AURA → ArkTS component mappings)
///     ├── StateGenerator (@State declarations, dispatch function)
///     ├── ModifierDsl (AURA styles → ArkTS modifiers)
///     └── ProjectGenerator (full HarmonyOS project)
/// ```
///
/// # Example
///
/// ```rust
/// use auto_lang::ui_gen::ark::ArkGenerator;
/// use auto_lang::ui_gen::BackendGenerator;
///
/// let mut gen = ArkGenerator::new();
/// // gen.generate(&widget); // Generate widget code
/// // gen.generate_project("MyApp"); // Generate full project
/// ```
pub struct ArkGenerator {
    /// Sanitized struct name (avoiding conflicts with built-in components)
    sanitized_name: Option<String>,
    /// Current widget name
    current_widget: Option<String>,

    /// Component registry
    registry: ArkComponentRegistry,

    /// Collected modifiers for current component
    current_modifiers: Vec<String>,

    /// Current indentation level
    indent_level: usize,

    /// Current widget's handlers (for event resolution)
    current_handlers: HashMap<String, LogicPayload>,

    /// Whether current widget has messages
    has_messages: bool,
}

impl ArkGenerator {
    /// Create a new ArkGenerator
    pub fn new() -> Self {
        Self {
            current_widget: None,
            registry: ArkComponentRegistry::new(),
            current_modifiers: Vec::new(),
            indent_level: 0,
            current_handlers: HashMap::new(),
            has_messages: false,
            sanitized_name: None,
        }
    }

    /// Generate indentation string
    fn indent(&self) -> String {
        "  ".repeat(self.indent_level)
    }

    /// Check if a name conflicts with built-in ArkUI components
    fn is_builtin_component(name: &str) -> bool {
        // List of built-in ArkUI components that could conflict
        const BUILTIN_COMPONENTS: &[&str] = &[
            "Button", "Column", "Row", "Text", "Image", "List", "Grid", "Scroll",
            "Stack", "Flex", "GridRow", "GridCol", "Counter", "Toggle", "Checkbox",
            "Radio", "Select", "Slider", "Progress", "Rating", "TextInput", "TextArea",
            "Search", "Divider", "Span", "Canvas", "Video", "Web", "XComponent",
            "AlphabetIndexer", "Badge", "Blank", "Clock", "DataPanel", "DatePicker",
            "DatePickerDialog", "LoadingProgress", "Marquee", "Navigation", "NavRouter",
            "NavDestination", "Navigator", "Panel", "Refresh", "RelativeContainer",
            "SideBarContainer", "Stepper", "StepperItem", "Swiper", "Tabs", "TabContent",
            "TimePicker", "TimePickerDialog", "Timer", "TextPicker", "TextPickerDialog",
            "Toast", "Dialog", "AlertDialog", "ActionSheet", "Menu", "MenuItem",
            "MenuGroup", "ContextMenu", "Popup", "PromptAction", "Hyperlink",
        ];

        BUILTIN_COMPONENTS.contains(&name)
    }

    /// Sanitize widget name to avoid conflicts with built-in components
    fn sanitize_widget_name(name: &str) -> String {
        if Self::is_builtin_component(name) {
            // Append "Widget" suffix to avoid conflict
            format!("{}Widget", name)
        } else {
            name.to_string()
        }
    }

    /// Generate full project
    pub fn generate_project(&self, name: &str) -> HashMap<String, String> {
        let gen = ArkProjectGenerator::new(name);
        gen.generate()
    }

    /// Generate full project with custom package
    pub fn generate_project_with_package(&self, name: &str, package: &str) -> HashMap<String, String> {
        let gen = ArkProjectGenerator::with_package(name, package);
        gen.generate()
    }

    /// Generate @Entry @Component struct from widget
    pub fn generate_entry_component(&mut self, widget: &AuraWidget) -> GenResult<String> {
        self.current_widget = Some(widget.name.clone());
        self.current_handlers = widget.handlers.clone();
        self.has_messages = !widget.messages.is_empty();

        // Sanitize widget name to avoid conflicts with built-in components
        let sanitized_name = Self::sanitize_widget_name(&widget.name);
        self.sanitized_name = Some(sanitized_name.clone());

        let mut lines = Vec::new();

        // Add import statement for ArkUI components (only Button - Column, Row, Text are built-in)
        lines.push("import { Button } from '@kit.ArkUI';".to_string());

        // Check if widget has routes - add NavPathStack import
        let has_routes = widget.routes.is_some();
        if has_routes {
            lines.push("import { NavPathStack } from '@kit.ArkUI';".to_string());
        }
        lines.push(String::new());

        // Generate Msg enum if widget has messages (before @Entry)
        let msg_enum = generate_msg_enum(widget);
        if !msg_enum.is_empty() {
            lines.push(msg_enum);
            lines.push("".to_string());
        }

        // @Entry @Component struct
        lines.push("@Entry".to_string());
        lines.push("@Component".to_string());
        lines.push(format!("struct {} {{", sanitized_name));

        self.indent_level = 1;

        // Add NavPathStack state if widget has routes
        if has_routes {
            lines.push(format!("{}@State navPathStack: NavPathStack = new NavPathStack()", self.indent()));
            lines.push(String::new());
        }

        // State declarations
        let state_decls = generate_state_declarations(widget);
        if !state_decls.is_empty() {
            for line in state_decls.lines() {
                lines.push(format!("{}{}", self.indent(), line));
            }
            lines.push("".to_string());
        }

        // Generate dispatch function if widget has messages and handlers
        let dispatch_fn = generate_dispatch_function(widget);
        if !dispatch_fn.is_empty() {
            for line in dispatch_fn.lines() {
                lines.push(format!("{}{}", self.indent(), line));
            }
            lines.push("".to_string());
        }

        // Generate @Builder functions for route pages
        if let Some(ref routes) = widget.routes {
            for route in &routes.routes {
                let builder_name = Self::page_to_builder_name(&route.module);
                lines.push(format!("{}@Builder", self.indent()));
                lines.push(format!("{}{}() {{", self.indent(), builder_name));
                lines.push(format!("{}  {}()", self.indent(), Self::module_to_component(&route.module)));
                lines.push(format!("{}}}", self.indent()));
                lines.push(String::new());
            }

            // Generate buildNavDestination builder for navDestination
            lines.push(format!("{}@Builder", self.indent()));
            lines.push(format!("{}buildNavDestination(name: string) {{", self.indent()));
            let mut first = true;
            for route in &routes.routes {
                let component_name = Self::module_to_component(&route.module);
                if first {
                    lines.push(format!("{}  if (name === '{}') {{", self.indent(), route.module));
                    first = false;
                } else {
                    lines.push(format!("{}  else if (name === '{}') {{", self.indent(), route.module));
                }
                lines.push(format!("{}    {}()", self.indent(), component_name));
                lines.push(format!("{}  }}", self.indent()));
            }
            lines.push(format!("{}}}", self.indent()));
            lines.push(String::new());
        }

        // build() method
        lines.push(format!("{}build() {{", self.indent()));
        self.indent_level = 2;

        // Generate UI tree from view_tree (not root)
        let ui_code = self.generate_node_with_routes(&widget.view_tree, has_routes)?;
        for line in ui_code.lines() {
            lines.push(format!("{}{}", self.indent(), line));
        }

        self.indent_level = 1;
        lines.push(format!("{}}}", self.indent()));

        self.indent_level = 0;
        lines.push("}".to_string());

        Ok(lines.join("\n"))
    }

    /// Convert page module name to @Builder function name
    fn page_to_builder_name(module: &str) -> String {
        // e.g., "counter" -> "CounterBuilder"
        let mut chars = module.chars();
        let first = chars.next().map(|c| c.to_uppercase().collect::<String>()).unwrap_or_default();
        let rest: String = chars.collect();
        format!("{}{}Builder", first, rest)
    }

    /// Convert module name to component name
    fn module_to_component(module: &str) -> String {
        // e.g., "counter" -> "CounterPage" or "index" -> "IndexPage"
        let mut chars = module.chars();
        let first = chars.next().map(|c| c.to_uppercase().collect::<String>()).unwrap_or_default();
        let rest: String = chars.collect();
        format!("{}{}Page", first, rest)
    }

    /// Generate ArkTS code for a node, with route awareness
    fn generate_node_with_routes(&mut self, node: &AuraNode, has_routes: bool) -> GenResult<String> {
        match node {
            AuraNode::Element {
                tag,
                props,
                events,
                children,
            } => {
                // Special handling for root col when routes exist - wrap in Navigation
                if tag == "col" && has_routes {
                    return self.generate_navigation_root(props, events, children);
                }
                self.generate_element(tag, props, events, children)
            }
            AuraNode::Outlet => {
                // Outlet in navigation context - handled by navDestination
                Ok("// Outlet - router placeholder".to_string())
            }
            _ => self.generate_node(node),
        }
    }

    /// Generate Navigation component with navDestination for routing
    fn generate_navigation_root(
        &mut self,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, crate::aura::AuraEvent>,
        children: &[AuraNode],
    ) -> GenResult<String> {
        let mut lines = Vec::new();

        // Navigation component with navPathStack
        lines.push("Navigation(this.navPathStack) {".to_string());
        self.indent_level += 1;

        // Generate children (header, outlet, etc.)
        for child in children {
            let child_code = self.generate_node(child)?;
            for line in child_code.lines() {
                lines.push(format!("{}{}", self.indent(), line));
            }
        }

        self.indent_level -= 1;
        lines.push(format!("{}}}", self.indent()));

        // Add navDestination modifier for route handling
        lines.push(format!("{}.navDestination(this.buildNavDestination)", self.indent()));

        // Add modifiers
        let modifiers = self.generate_modifiers(props, events);
        if !modifiers.is_empty() {
            lines.last_mut().unwrap().push_str(&modifiers);
        }

        Ok(lines.join("\n"))
    }

    /// Generate buildNavDestination builder for navDestination
    fn generate_nav_destination_builder(&self, routes: &crate::aura::AuraRoutes) -> String {
        let mut lines = Vec::new();

        lines.push("@Builder".to_string());
        lines.push("buildNavDestination(name: string) {".to_string());
        lines.push("  if (name === 'index') {".to_string());
        lines.push("    IndexPage()".to_string());
        lines.push("  }".to_string());

        for route in &routes.routes {
            if route.module != "index" {
                let component_name = Self::module_to_component(&route.module);
                lines.push(format!("  else if (name === '{}') {{", route.module));
                lines.push(format!("    {}()", component_name));
                lines.push("  }".to_string());
            }
        }

        lines.push("}".to_string());
        lines.join("\n")
    }

    /// Generate ArkTS code for a single node
    fn generate_node(&mut self, node: &AuraNode) -> GenResult<String> {
        match node {
            AuraNode::Element {
                tag,
                props,
                events,
                children,
            } => self.generate_element(tag, props, events, children),
            AuraNode::Text(text_content) => self.generate_text(text_content),
            AuraNode::ForLoop {
                var,
                index,
                iterable,
                body,
            } => self.generate_for_loop(var, index.as_deref(), iterable, body),
            AuraNode::Conditional {
                condition,
                then_body,
                else_body,
            } => self.generate_conditional(condition, then_body, else_body.as_deref()),
            AuraNode::Component { name, props, events } => {
                self.generate_component(name, props, events)
            }
            AuraNode::Outlet => Ok("// Outlet - router placeholder".to_string()),
            AuraNode::Link {
                to,
                text,
                href,
                children,
            } => self.generate_link(to, text, href, children),
        }
    }

    /// Generate element component
    fn generate_element(
        &mut self,
        tag: &str,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, crate::aura::AuraEvent>,
        children: &[AuraNode],
    ) -> GenResult<String> {
        let mut lines = Vec::new();

        // Look up component
        if let Some(component) = self.registry.get(tag) {
            // Get text content for components with content (like Button, Text)
            let content_arg = if component.has_content {
                if let Some(AuraPropValue::Expr(AuraExpr::Literal(text))) = props.get("text") {
                    format!("'{}'", text)
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            // Component call with content argument
            let component_call = if content_arg.is_empty() {
                format!("{}()", component.name)
            } else {
                format!("{}({})", component.name, content_arg)
            };

            let modifiers = self.generate_modifiers(props, events);
            lines.push(format!("{}{}", component_call, modifiers));

            // Children
            if component.has_children && !children.is_empty() {
                lines.last_mut().unwrap().push_str(" {");
                self.indent_level += 1;

                for child in children {
                    let child_code = self.generate_node(child)?;
                    for line in child_code.lines() {
                        lines.push(format!("{}{}", self.indent(), line));
                    }
                }

                self.indent_level -= 1;
                lines.push(format!("{}}}", self.indent()));
            }
        } else {
            // Unknown component - emit as comment
            lines.push(format!("/* Unknown component: {} */", tag));
        }

        Ok(lines.join("\n"))
    }

    /// Generate modifiers from props and events
    fn generate_modifiers(
        &self,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, crate::aura::AuraEvent>,
    ) -> String {
        let mut modifiers = Vec::new();

        // Process props - extract string/number values from AuraExpr
        for (key, value) in props {
            if key == "text" {
                continue;
            }
            if let Some(modifier) = self.prop_to_modifier(key, value) {
                modifiers.push(modifier);
            }
        }

        // Process events - generate onClick handlers
        for (event_name, event) in events {
            if event_name == "click" || event_name == "onclick" {
                // Use dispatch pattern if widget has messages, otherwise direct state update
                let handler_code = if self.has_messages && event.handler.starts_with('.') {
                    // Extract message name and generate dispatch call
                    let msg_name = &event.handler[1..];
                    format!("this.dispatch(Msg.{})", msg_name)
                } else {
                    // Fall back to direct state update
                    self.generate_handler_code(&event.handler)
                };
                modifiers.push(format!(
                    ".onClick(() => {{\n    {}\n  }})",
                    handler_code
                ));
            }
        }

        // Process class bindings from ClassBinding variant
        for value in props.values() {
            if let AuraPropValue::ClassBinding(bindings) = value {
                for binding in bindings {
                    // Evaluate condition to determine if class should apply
                    if let Some(modifier) = class_to_modifier(&binding.class_name) {
                        // For now, apply the class unconditionally
                        // TODO: Support conditional class application
                        modifiers.push(modifier);
                    }
                }
            }
        }

        if modifiers.is_empty() {
            String::new()
        } else {
            format!(
                "\n{}",
                modifiers
                    .iter()
                    .map(|m| format!("{}{}", self.indent(), m))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        }
    }

    /// Convert prop to modifier
    fn prop_to_modifier(&self, key: &str, value: &AuraPropValue) -> Option<String> {
        match value {
            AuraPropValue::Expr(expr) => self.expr_to_modifier(key, expr),
            AuraPropValue::ClassBinding(_) => None, // Handled separately
        }
    }

    /// Convert AuraExpr to modifier
    fn expr_to_modifier(&self, key: &str, expr: &AuraExpr) -> Option<String> {
        match expr {
            AuraExpr::Literal(s) => prop_to_modifier(key, s, None),
            AuraExpr::Int(n) => prop_to_modifier(key, &n.to_string(), None),
            AuraExpr::Float(n) => prop_to_modifier(key, &n.to_string(), None),
            AuraExpr::Bool(b) => prop_to_modifier(key, &b.to_string(), None),
            AuraExpr::StateRef(name) => {
                // Generate binding to state
                Some(format!(".{}(this.{})", key, name))
            }
            _ => None,
        }
    }

    /// Generate handler code from handler string (e.g., ".Inc" -> "this.count += 1")
    fn generate_handler_code(&self, handler: &str) -> String {
        // Handler is in format ".MsgName" - look up in current_handlers
        if handler.starts_with('.') {
            let msg_name = &handler[1..];
            if let Some(payload) = self.current_handlers.get(handler) {
                // Generate the handler body directly
                return generate_handler_body(payload);
            }

            // Fallback: generate simple increment/decrement based on name
            match msg_name {
                "Inc" => "this.count += 1".to_string(),
                "Dec" => "this.count -= 1".to_string(),
                _ => format!("// TODO: handler for {}", msg_name),
            }
        } else {
            format!("// Unknown handler: {}", handler)
        }
    }

    /// Generate text node
    fn generate_text(&self, text: &AuraTextContent) -> GenResult<String> {
        match text {
            AuraTextContent::Literal(s) => Ok(format!("Text(\"{}\")", s)),
            AuraTextContent::Interpolated { template, bindings } => {
                // Convert ${.field} or ${..field} to ${this.field} for ArkTS template literals
                let mut result = template.clone();
                for binding in bindings {
                    // Replace ${.field} with ${this.field}
                    result = result.replace(&format!("${{.{}}}", binding), &format!("${{this.{}}}", binding));
                    // Also handle ${..field} (double dot) pattern
                    result = result.replace(&format!("${{..{}}}", binding), &format!("${{this.{}}}", binding));
                }
                Ok(format!("Text(`{}`)", result))
            }
        }
    }

    /// Generate for loop
    fn generate_for_loop(
        &mut self,
        var: &str,
        index: Option<&str>,
        iterable: &str,
        body: &[AuraNode],
    ) -> GenResult<String> {
        let mut lines = Vec::new();

        // Generate ForEach for ArkTS
        let index_param = index.map(|i| format!(", {}: number", i)).unwrap_or_default();
        lines.push(format!(
            "{}ForEach(this.{}, ({}: any{}) => {{",
            self.indent(),
            iterable,
            var,
            index_param
        ));
        self.indent_level += 1;

        for child in body {
            let child_code = self.generate_node(child)?;
            for line in child_code.lines() {
                lines.push(format!("{}{}", self.indent(), line));
            }
        }

        self.indent_level -= 1;
        lines.push(format!("{}}})", self.indent()));

        Ok(lines.join("\n"))
    }

    /// Generate component instantiation
    fn generate_component(
        &mut self,
        name: &str,
        props: &HashMap<String, AuraExpr>,
        events: &HashMap<String, crate::aura::AuraEvent>,
    ) -> GenResult<String> {
        let mut lines = Vec::new();

        // Component call
        lines.push(format!("{}()", name));

        // Generate modifiers from props
        let modifiers: Vec<String> = props
            .iter()
            .filter_map(|(key, expr)| self.expr_to_modifier(key, expr))
            .collect();

        // Add event handlers
        let event_modifiers: Vec<String> = events
            .iter()
            .filter_map(|(event_name, event)| {
                if event_name == "click" || event_name == "onclick" {
                    Some(format!(
                        ".onClick(() => {{ this.dispatch(Msg.{}) }})",
                        event.handler
                    ))
                } else {
                    None
                }
            })
            .collect();

        if !modifiers.is_empty() || !event_modifiers.is_empty() {
            let all_modifiers: Vec<String> = modifiers.into_iter().chain(event_modifiers).collect();
            lines.last_mut().unwrap().push_str(&format!(
                "\n{}",
                all_modifiers
                    .iter()
                    .map(|m| format!("{}{}", self.indent(), m))
                    .collect::<Vec<_>>()
                    .join("\n")
            ));
        }

        Ok(lines.join("\n"))
    }

    /// Generate navigation link
    fn generate_link(
        &mut self,
        to: &str,
        text: &str,
        href: &str,
        children: &[AuraNode],
    ) -> GenResult<String> {
        let mut lines = Vec::new();

        // Use external href if provided, otherwise use internal to
        let target = if !href.is_empty() { href } else { to };

        // Extract route name from path (e.g., "/counter" -> "counter")
        let route_name = target.trim_start_matches('/');

        if !text.is_empty() {
            // Simple text link - use Column with onClick for navigation
            lines.push(format!("Column()"));
            lines.last_mut().unwrap().push_str(&format!(
                "\n{}.onClick(() => {{\n    this.navPathStack.pushPathByName('{}')\n  }})",
                self.indent(),
                route_name
            ));
            lines.last_mut().unwrap().push_str(" {");
            self.indent_level += 1;
            lines.push(format!("{}Text(\"{}\")", self.indent(), text));
            self.indent_level -= 1;
            lines.push(format!("{}}}", self.indent()));
        } else if !children.is_empty() {
            // Link with children
            lines.push(format!("Column()"));
            lines.last_mut().unwrap().push_str(&format!(
                "\n{}.onClick(() => {{\n    this.navPathStack.pushPathByName('{}')\n  }})",
                self.indent(),
                route_name
            ));
            lines.last_mut().unwrap().push_str(" {");
            self.indent_level += 1;

            for child in children {
                let child_code = self.generate_node(child)?;
                for line in child_code.lines() {
                    lines.push(format!("{}{}", self.indent(), line));
                }
            }

            self.indent_level -= 1;
            lines.push(format!("{}}}", self.indent()));
        }

        Ok(lines.join("\n"))
    }

    /// Generate conditional
    fn generate_conditional(
        &mut self,
        condition: &str,
        then_body: &[AuraNode],
        else_body: Option<&[AuraNode]>,
    ) -> GenResult<String> {
        let mut lines = Vec::new();

        lines.push(format!("{}if ({}) {{", self.indent(), condition));
        self.indent_level += 1;

        for child in then_body {
            let child_code = self.generate_node(child)?;
            for line in child_code.lines() {
                lines.push(format!("{}{}", self.indent(), line));
            }
        }

        self.indent_level -= 1;

        if let Some(else_nodes) = else_body {
            lines.push(format!("{}}} else {{", self.indent()));
            self.indent_level += 1;

            for child in else_nodes {
                let child_code = self.generate_node(child)?;
                for line in child_code.lines() {
                    lines.push(format!("{}{}", self.indent(), line));
                }
            }

            self.indent_level -= 1;
        }

        lines.push(format!("{}}}", self.indent()));

        Ok(lines.join("\n"))
    }
}

impl Default for ArkGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl BackendGenerator for ArkGenerator {
    fn generate(&mut self, widget: &AuraWidget) -> GenResult<String> {
        self.generate_entry_component(widget)
    }

    fn extension(&self) -> &'static str {
        "ets"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aura::{AuraExpr, AuraMessage, AuraMsgVariant, AuraNode, AuraStateDef, Type};
    use std::collections::HashMap;

    #[test]
    fn test_generator_extension() {
        let gen = ArkGenerator::new();
        assert_eq!(gen.extension(), "ets");
    }

    #[test]
    fn test_project_generation() {
        let gen = ArkGenerator::new();
        let files = gen.generate_project("TestApp");

        assert!(files.contains_key("build-profile.json5"));
        assert!(files.contains_key("oh-package.json5"));
        // Note: Page files (App.ets, etc.) are generated from AURA, not scaffolding
        assert!(files.contains_key("entry/src/main/ets/entryability/EntryAbility.ets"));
    }

    #[test]
    fn test_custom_package() {
        let gen = ArkGenerator::new();
        let files = gen.generate_project_with_package("TestApp", "com.company.test");

        let oh_package = files.get("oh-package.json5").unwrap();
        assert!(oh_package.contains("TestApp"));

        // Check that custom package is in app.json5
        let app_json = files.get("AppScope/app.json5").unwrap();
        assert!(app_json.contains("com.company.test"));
    }

    #[test]
    fn test_dispatch_pattern_generation() {
        // Create a widget with messages and handlers
        let widget = AuraWidget {
            name: "Counter".to_string(),
            state_vars: vec![AuraStateDef {
                name: "count".to_string(),
                type_info: Type::Int,
                initial: AuraExpr::Int(0),
            }],
            computed: vec![],
            messages: vec![AuraMessage {
                name: "Msg".to_string(),
                variants: vec![
                    AuraMsgVariant {
                        name: "Inc".to_string(),
                        payload: None,
                    },
                    AuraMsgVariant {
                        name: "Dec".to_string(),
                        payload: None,
                    },
                ],
            }],
            view_tree: AuraNode::Element {
                tag: "col".to_string(),
                props: HashMap::new(),
                events: HashMap::new(),
                children: vec![],
            },
            handlers: HashMap::new(),
            props: vec![],
            routes: None,
        };

        let mut gen = ArkGenerator::new();
        let code = gen.generate_entry_component(&widget).unwrap();

        // Should contain Msg enum
        assert!(code.contains("enum Msg {"), "Should generate Msg enum");
        assert!(code.contains("Inc,"), "Should contain Inc variant");
        assert!(code.contains("Dec,"), "Should contain Dec variant");

        // Should contain dispatch function
        assert!(
            code.contains("private dispatch(msg: Msg): void"),
            "Should generate dispatch function"
        );

        // Should NOT contain direct state update in event handlers
        // (dispatch pattern is used instead)
    }

    #[test]
    fn test_generated_file_has_imports() {
        // Create a simple widget
        let widget = AuraWidget {
            name: "TestWidget".to_string(),
            state_vars: vec![],
            computed: vec![],
            messages: vec![],
            view_tree: AuraNode::Element {
                tag: "col".to_string(),
                props: HashMap::new(),
                events: HashMap::new(),
                children: vec![],
            },
            handlers: HashMap::new(),
            props: vec![],
            routes: None,
        };

        let mut gen = ArkGenerator::new();
        let code = gen.generate_entry_component(&widget).unwrap();

        // Should contain import statement at the top (only Button - built-ins don't need import)
        assert!(
            code.contains("import { Button } from '@kit.ArkUI';"),
            "Should contain ArkUI import statement for Button only"
        );

        // Import should appear before @Entry decorator
        let import_pos = code.find("import").expect("Import should exist");
        let entry_pos = code.find("@Entry").expect("@Entry should exist");
        assert!(
            import_pos < entry_pos,
            "Import should appear before @Entry decorator"
        );
    }

    #[test]
    fn test_semantic_header_element() {
        // Test that semantic HTML elements like header are transpiled to Column
        let widget = AuraWidget {
            name: "TestApp".to_string(),
            state_vars: vec![],
            computed: vec![],
            messages: vec![],
            view_tree: AuraNode::Element {
                tag: "header".to_string(),
                props: HashMap::new(),
                events: HashMap::new(),
                children: vec![AuraNode::Text(AuraTextContent::Literal("Hello".to_string()))],
            },
            handlers: HashMap::new(),
            props: vec![],
            routes: None,
        };

        let mut gen = ArkGenerator::new();
        let code = gen.generate_entry_component(&widget).unwrap();

        // header should be transpiled to Column, not "Unknown component"
        assert!(
            code.contains("Column()"),
            "header should be transpiled to Column, got: {}",
            code
        );
        assert!(
            !code.contains("Unknown component"),
            "Should not contain 'Unknown component' comment, got: {}",
            code
        );
    }
}
