//! Vue3/JavaScript Code Generator
//!
//! Generates Vue 3 Single File Components (SFC) from AURA widgets.
//! Supports two output modes:
//!
//! 1. **Plain Tailwind** - Native HTML elements with Tailwind CSS classes
//! 2. **shadcn-vue** - Pre-built accessible components from shadcn-vue
//!
//! ## Output Format (shadcn-vue mode)
//!
//! ```vue
//! <script setup>
//! import { ref } from 'vue'
//! import { Button } from '@/components/ui/button'
//! import { Input } from '@/components/ui/input'
//!
//! const count = ref(0)
//!
//! const handleInc = () => {
//!   count.value += 1
//! }
//! </script>
//!
//! <template>
//!   <div class="flex flex-col gap-2">
//!     <Button @click="handleInc">Increment</Button>
//!     <Input v-model="count" />
//!   </div>
//! </template>
//! ```
//!
//! ## Output Format (Plain Tailwind mode)
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
use std::collections::{HashMap, HashSet};

// ============================================================================
// shadcn-vue Component Registry
// ============================================================================

/// Maps AURA element tags to shadcn-vue component imports
pub struct ShadcnRegistry {
    /// Component imports needed: tag -> (module_path, component_names)
    components: HashMap<&'static str, (&'static str, Vec<&'static str>)>,
}

impl ShadcnRegistry {
    /// Create registry with all shadcn-vue component mappings
    pub fn new() -> Self {
        let mut components = HashMap::new();

        // === Content Elements ===
        components.insert("button",
            ("@/components/ui/button", vec!["Button"]));
        components.insert("input",
            ("@/components/ui/input", vec!["Input"]));
        components.insert("textarea",
            ("@/components/ui/textarea", vec!["Textarea"]));
        components.insert("checkbox",
            ("@/components/ui/checkbox", vec!["Checkbox"]));
        components.insert("toggle",
            ("@/components/ui/switch", vec!["Switch"]));
        components.insert("select",
            ("@/components/ui/select", vec!["Select", "SelectContent", "SelectItem", "SelectTrigger", "SelectValue"]));
        components.insert("option",
            ("@/components/ui/select", vec!["SelectItem"]));

        // === Navigation Elements ===
        components.insert("tabs",
            ("@/components/ui/tabs", vec!["Tabs", "TabsList", "TabsTrigger", "TabsContent"]));
        components.insert("tab",
            ("@/components/ui/tabs", vec!["TabsTrigger", "TabsContent"]));

        // === Overlay Elements ===
        components.insert("modal",
            ("@/components/ui/dialog", vec!["Dialog", "DialogContent", "DialogTrigger", "DialogTitle", "DialogDescription"]));
        components.insert("tooltip",
            ("@/components/ui/tooltip", vec!["Tooltip", "TooltipContent", "TooltipProvider", "TooltipTrigger"]));

        // === Form Elements ===
        components.insert("slider",
            ("@/components/ui/slider", vec!["Slider"]));
        components.insert("radio",
            ("@/components/ui/radio-group", vec!["RadioGroup", "RadioGroupItem"]));
        components.insert("radiogroup",
            ("@/components/ui/radio-group", vec!["RadioGroup"]));

        // === Feedback Elements ===
        components.insert("progress",
            ("@/components/ui/progress", vec!["Progress"]));
        components.insert("badge",
            ("@/components/ui/badge", vec!["Badge"]));
        components.insert("spinner",
            ("@/components/ui/skeleton", vec!["Skeleton"]));

        // === Display Elements ===
        components.insert("card",
            ("@/components/ui/card", vec!["Card", "CardHeader", "CardTitle", "CardDescription", "CardContent", "CardFooter"]));
        components.insert("avatar",
            ("@/components/ui/avatar", vec!["Avatar", "AvatarImage", "AvatarFallback"]));

        // === Data Elements ===
        components.insert("table",
            ("@/components/ui/table", vec!["Table", "TableHeader", "TableBody", "TableRow", "TableHead", "TableCell"]));
        components.insert("thead",
            ("@/components/ui/table", vec!["TableHeader"]));
        components.insert("tbody",
            ("@/components/ui/table", vec!["TableBody"]));
        components.insert("tr",
            ("@/components/ui/table", vec!["TableRow"]));
        components.insert("th",
            ("@/components/ui/table", vec!["TableHead"]));
        components.insert("td",
            ("@/components/ui/table", vec!["TableCell"]));

        // === Utility Elements ===
        components.insert("divider",
            ("@/components/ui/separator", vec!["Separator"]));
        components.insert("scroll",
            ("@/components/ui/scroll-area", vec!["ScrollArea"]));
        components.insert("label",
            ("@/components/ui/label", vec!["Label"]));

        Self { components }
    }

    /// Get shadcn-vue component info for a tag
    pub fn get(&self, tag: &str) -> Option<(&'static str, &Vec<&'static str>)> {
        self.components.get(tag).map(|(path, names)| (*path, names))
    }

    /// Check if tag has a shadcn-vue component
    pub fn has_component(&self, tag: &str) -> bool {
        self.components.contains_key(tag)
    }

    /// Get the primary component name for a tag (first in the list)
    pub fn primary_component(&self, tag: &str) -> Option<&'static str> {
        self.components.get(tag).and_then(|(_, names)| names.first().copied())
    }
}

impl Default for ShadcnRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Vue Generator
// ============================================================================

/// Generation mode for Vue output
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VueMode {
    /// Plain HTML with Tailwind CSS classes
    #[default]
    Plain,
    /// shadcn-vue components with accessibility built-in
    Shadcn,
}

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

    /// Generation mode (Plain or Shadcn)
    mode: VueMode,

    /// shadcn-vue component registry
    shadcn_registry: ShadcnRegistry,

    /// Track which shadcn-vue components are used
    shadcn_components_used: HashSet<String>,
}

impl VueGenerator {
    /// Create a new Vue generator (Plain Tailwind mode)
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
            mode: VueMode::Plain,
            shadcn_registry: ShadcnRegistry::new(),
            shadcn_components_used: HashSet::new(),
        }
    }

    /// Create a new Vue generator in shadcn-vue mode
    pub fn new_shadcn() -> Self {
        Self {
            mode: VueMode::Shadcn,
            ..Self::new()
        }
    }

    /// Set the generation mode
    pub fn with_mode(mut self, mode: VueMode) -> Self {
        self.mode = mode;
        self
    }

    /// Check if using shadcn-vue mode
    pub fn is_shadcn(&self) -> bool {
        self.mode == VueMode::Shadcn
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
        self.shadcn_components_used.clear();
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

                // Check if this is a shadcn-vue component
                let is_shadcn_component = self.is_shadcn() && self.shadcn_registry.has_component(tag);

                // Build attributes
                let (attrs, text_content) = if is_shadcn_component {
                    // Use shadcn-specific attribute generation
                    let (shadcn_attrs, slot_content) = self.generate_shadcn_attrs(tag, props, events);
                    (shadcn_attrs, slot_content)
                } else {
                    // Use plain Tailwind attribute generation
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

                    // Props as attributes
                    for (key, value) in props {
                        if key == "class" {
                            continue; // Already handled
                        }
                        if key == "text" {
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

                    (attrs, text_content)
                };

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

    /// Map AutoUI tag to HTML tag or shadcn-vue component
    fn map_tag(&mut self, tag: &str, self_closing: bool) -> String {
        // If in shadcn mode and tag has a shadcn component, use it
        if self.is_shadcn() {
            if let Some(component_name) = self.shadcn_component_name(tag) {
                self.register_shadcn_component(tag);
                return component_name.to_string();
            }
        }

        // Fallback to plain HTML tags
        match tag {
            // Layout (no shadcn components, use Tailwind)
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

            // Typography (no shadcn components)
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

        // In shadcn mode, skip default classes for components that have shadcn versions
        // (shadcn components have their own styling)
        let skip_defaults = self.is_shadcn() && self.shadcn_registry.has_component(tag);

        // Default classes based on tag (only in Plain mode or for non-shadcn elements)
        if !skip_defaults {
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

    // ========================================================================
    // shadcn-vue Component-specific Prop Handling
    // ========================================================================

    /// Generate shadcn-vue component attributes based on element type
    fn generate_shadcn_attrs(
        &self,
        tag: &str,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, AuraEvent>,
    ) -> (Vec<String>, Option<String>) {
        let mut attrs = Vec::new();
        let mut slot_content: Option<String> = None;

        match tag {
            // === Button ===
            "button" => {
                // Handle variant prop
                if let Some(value) = props.get("variant") {
                    let variant = self.extract_string_value(value).unwrap_or("default");
                    attrs.push(format!("variant=\"{}\"", variant));
                }
                // Handle size prop
                if let Some(value) = props.get("size") {
                    let size = self.extract_string_value(value).unwrap_or("default");
                    attrs.push(format!("size=\"{}\"", size));
                }
                // Handle disabled
                if let Some(value) = props.get("disabled") {
                    if self.extract_bool_value(value) {
                        attrs.push("disabled".to_string());
                    }
                }
                // Text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            // === Input ===
            "input" => {
                // v-model for value
                if let Some(value) = props.get("value") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model=\"{}\"", model));
                    }
                }
                // type prop
                if let Some(value) = props.get("type") {
                    let type_val = self.extract_string_value(value).unwrap_or("text");
                    attrs.push(format!("type=\"{}\"", type_val));
                }
                // placeholder
                if let Some(value) = props.get("placeholder") {
                    let placeholder = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("placeholder=\"{}\"", placeholder));
                }
                // disabled
                if let Some(value) = props.get("disabled") {
                    if self.extract_bool_value(value) {
                        attrs.push("disabled".to_string());
                    }
                }
            }

            // === Textarea ===
            "textarea" => {
                // v-model for value
                if let Some(value) = props.get("value") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model=\"{}\"", model));
                    }
                }
                // placeholder
                if let Some(value) = props.get("placeholder") {
                    let placeholder = self.extract_string_value(value).unwrap_or("");
                    attrs.push(format!("placeholder=\"{}\"", placeholder));
                }
                // rows
                if let Some(value) = props.get("rows") {
                    let rows = self.extract_int_value(value).unwrap_or(3);
                    attrs.push(format!(":rows=\"{}\"", rows));
                }
                // disabled
                if let Some(value) = props.get("disabled") {
                    if self.extract_bool_value(value) {
                        attrs.push("disabled".to_string());
                    }
                }
            }

            // === Checkbox ===
            "checkbox" => {
                // v-model:checked for checked state
                if let Some(value) = props.get("checked") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model:checked=\"{}\"", model));
                    }
                }
                // disabled
                if let Some(value) = props.get("disabled") {
                    if self.extract_bool_value(value) {
                        attrs.push("disabled".to_string());
                    }
                }
            }

            // === Switch/Toggle ===
            "toggle" | "switch" => {
                // v-model:checked for checked state
                if let Some(value) = props.get("checked") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model:checked=\"{}\"", model));
                    }
                }
                // disabled
                if let Some(value) = props.get("disabled") {
                    if self.extract_bool_value(value) {
                        attrs.push("disabled".to_string());
                    }
                }
            }

            // === Select ===
            "select" => {
                // v-model for value
                if let Some(value) = props.get("value") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model=\"{}\"", model));
                    }
                }
                // disabled
                if let Some(value) = props.get("disabled") {
                    if self.extract_bool_value(value) {
                        attrs.push("disabled".to_string());
                    }
                }
            }

            // === Slider ===
            "slider" => {
                // v-model for value
                if let Some(value) = props.get("value") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model=\"{}\"", model));
                    }
                }
                // min/max/step
                if let Some(value) = props.get("min") {
                    let min = self.extract_int_value(value).unwrap_or(0);
                    attrs.push(format!(":min=\"{}\"", min));
                }
                if let Some(value) = props.get("max") {
                    let max = self.extract_int_value(value).unwrap_or(100);
                    attrs.push(format!(":max=\"{}\"", max));
                }
                if let Some(value) = props.get("step") {
                    let step = self.extract_int_value(value).unwrap_or(1);
                    attrs.push(format!(":step=\"{}\"", step));
                }
                // disabled
                if let Some(value) = props.get("disabled") {
                    if self.extract_bool_value(value) {
                        attrs.push("disabled".to_string());
                    }
                }
            }

            // === Progress ===
            "progress" => {
                // v-model for value
                if let Some(value) = props.get("value") {
                    if let Some(model) = self.extract_state_ref(value) {
                        attrs.push(format!("v-model=\"{}\"", model));
                    } else if let Some(int_val) = self.extract_int_value(value) {
                        attrs.push(format!(":model-value=\"{}\"", int_val));
                    }
                }
                // max
                if let Some(value) = props.get("max") {
                    let max = self.extract_int_value(value).unwrap_or(100);
                    attrs.push(format!(":max=\"{}\"", max));
                }
            }

            // === Badge ===
            "badge" => {
                // variant
                if let Some(value) = props.get("type") {
                    let variant = self.extract_string_value(value).unwrap_or("default");
                    attrs.push(format!("variant=\"{}\"", variant));
                }
                // Text becomes slot content
                if let Some(value) = props.get("text") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            // === Card ===
            "card" => {
                // Cards use slots for header/content
            }

            // === Avatar ===
            "avatar" => {
                // src for image
                if let Some(value) = props.get("src") {
                    let src = self.extract_string_value(value).unwrap_or("");
                    // AvatarImage component
                    attrs.push(format!("src=\"{}\"", src));
                }
                // alt/fallback
                if let Some(value) = props.get("name") {
                    slot_content = self.prop_to_text_content(value).ok();
                }
            }

            _ => {
                // Default handling for other components
            }
        }

        // Add event handlers
        for (event, aura_event) in events {
            let vue_event = self.shadcn_event_to_vue(tag, event);
            let handler_fn = self.handler_to_function_call_with_params(&aura_event.handler, &aura_event.params);
            attrs.push(format!("{}=\"{}\"", vue_event, handler_fn));
        }

        (attrs, slot_content)
    }

    /// Convert AutoUI event to Vue event for shadcn-vue components
    fn shadcn_event_to_vue(&self, _tag: &str, event: &str) -> String {
        match event {
            "onclick" | "onClick" | "on_click" => "@click".to_string(),
            "oninput" | "onInput" => "@update:modelValue".to_string(),
            "onchange" | "onChange" => "@update:modelValue".to_string(),
            "onenter" | "onEnter" => "@keyup.enter".to_string(),
            _ => format!("@{}", event.trim_start_matches("on")),
        }
    }

    /// Extract string value from AuraPropValue
    fn extract_string_value<'a>(&self, value: &'a AuraPropValue) -> Option<&'a str> {
        match value {
            AuraPropValue::Expr(AuraExpr::Literal(s)) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Extract boolean value from AuraPropValue
    fn extract_bool_value(&self, value: &AuraPropValue) -> bool {
        match value {
            AuraPropValue::Expr(AuraExpr::Bool(b)) => *b,
            AuraPropValue::Expr(AuraExpr::Literal(s)) => s == "true",
            _ => false,
        }
    }

    /// Extract integer value from AuraPropValue
    fn extract_int_value(&self, value: &AuraPropValue) -> Option<i64> {
        match value {
            AuraPropValue::Expr(AuraExpr::Int(n)) => Some(*n),
            AuraPropValue::Expr(AuraExpr::Literal(s)) => s.parse().ok(),
            _ => None,
        }
    }

    /// Extract state reference from AuraPropValue
    fn extract_state_ref(&self, value: &AuraPropValue) -> Option<String> {
        match value {
            AuraPropValue::Expr(AuraExpr::StateRef(name)) => Some(name.clone()),
            _ => None,
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

    // ========================================================================
    // shadcn-vue Support Methods
    // ========================================================================

    /// Register a shadcn-vue component as used
    fn register_shadcn_component(&mut self, tag: &str) {
        if self.is_shadcn() {
            if let Some(component_name) = self.shadcn_registry.primary_component(tag) {
                self.shadcn_components_used.insert(component_name.to_string());
            }
        }
    }

    /// Generate shadcn-vue import statements
    fn generate_shadcn_imports(&self) -> String {
        if !self.is_shadcn() || self.shadcn_components_used.is_empty() {
            return String::new();
        }

        // Group components by their module path
        let mut module_imports: HashMap<&str, Vec<&str>> = HashMap::new();

        for component_name in &self.shadcn_components_used {
            // Find which module this component belongs to
            for (tag, (module, components)) in &self.shadcn_registry.components {
                if components.contains(&component_name.as_str()) {
                    module_imports.entry(module).or_default().push(component_name.as_str());
                    break;
                }
            }
        }

        let mut imports = Vec::new();
        for (module, components) in module_imports {
            let unique_components: HashSet<&str> = components.into_iter().collect();
            let mut sorted: Vec<&str> = unique_components.into_iter().collect();
            sorted.sort();
            imports.push(format!("import {{ {} }} from '{}'", sorted.join(", "), module));
        }

        imports.sort();
        imports.join("\n")
    }

    /// Get shadcn-vue component name for a tag
    fn shadcn_component_name(&self, tag: &str) -> Option<&'static str> {
        if self.is_shadcn() {
            self.shadcn_registry.primary_component(tag)
        } else {
            None
        }
    }

    /// Generate components.json for shadcn-vue project setup
    pub fn generate_components_json() -> String {
        r#"{
  "$schema": "https://shadcn-vue.com/schema.json",
  "style": "default",
  "typescript": true,
  "tsConfigPath": "./tsconfig.json",
  "tailwind": {
    "config": "tailwind.config.js",
    "css": "src/assets/index.css",
    "baseColor": "slate",
    "cssVariables": true
  },
  "framework": "vite",
  "aliases": {
    "components": "@/components",
    "utils": "@/lib/utils"
  }
}"#.to_string()
    }

    /// Generate package.json for shadcn-vue project
    pub fn generate_package_json(project_name: &str) -> String {
        format!(r#"{{
  "name": "{}",
  "version": "0.0.0",
  "private": true,
  "type": "module",
  "scripts": {{
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview"
  }},
  "dependencies": {{
    "vue": "^3.4.0",
    "vue-router": "^4.2.0",
    "@vueuse/core": "^10.7.0",
    "radix-vue": "^1.4.0",
    "class-variance-authority": "^0.7.0",
    "clsx": "^2.1.0",
    "tailwind-merge": "^2.2.0",
    "lucide-vue-next": "^0.312.0"
  }},
  "devDependencies": {{
    "@vitejs/plugin-vue": "^5.0.0",
    "vite": "^5.0.0",
    "typescript": "^5.3.0",
    "vue-tsc": "^1.8.0",
    "tailwindcss": "^3.4.0",
    "autoprefixer": "^10.4.0",
    "postcss": "^8.4.0"
  }}
}}"#, project_name)
    }

    /// Generate vite.config.ts for shadcn-vue project
    pub fn generate_vite_config() -> String {
        r#"import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': resolve(__dirname, './src'),
    },
  },
})
"#.to_string()
    }

    /// Generate tailwind.config.js for shadcn-vue project
    pub fn generate_tailwind_config() -> String {
        r#"/** @type {import('tailwindcss').Config} */
export default {
  darkMode: ["class"],
  content: [
    "./index.html",
    "./src/**/*.{vue,js,ts,jsx,tsx}",
  ],
  theme: {
    container: {
      center: true,
      padding: "2rem",
      screens: {
        "2xl": "1400px",
      },
    },
    extend: {
      colors: {
        border: "hsl(var(--border))",
        input: "hsl(var(--input))",
        ring: "hsl(var(--ring))",
        background: "hsl(var(--background))",
        foreground: "hsl(var(--foreground))",
        primary: {
          DEFAULT: "hsl(var(--primary))",
          foreground: "hsl(var(--primary-foreground))",
        },
        secondary: {
          DEFAULT: "hsl(var(--secondary))",
          foreground: "hsl(var(--secondary-foreground))",
        },
        destructive: {
          DEFAULT: "hsl(var(--destructive))",
          foreground: "hsl(var(--destructive-foreground))",
        },
        muted: {
          DEFAULT: "hsl(var(--muted))",
          foreground: "hsl(var(--muted-foreground))",
        },
        accent: {
          DEFAULT: "hsl(var(--accent))",
          foreground: "hsl(var(--accent-foreground))",
        },
        popover: {
          DEFAULT: "hsl(var(--popover))",
          foreground: "hsl(var(--popover-foreground))",
        },
        card: {
          DEFAULT: "hsl(var(--card))",
          foreground: "hsl(var(--card-foreground))",
        },
      },
      borderRadius: {
        lg: "var(--radius)",
        md: "calc(var(--radius) - 2px)",
        sm: "calc(var(--radius) - 4px)",
      },
      keyframes: {
        "accordion-down": {
          from: { height: 0 },
          to: { height: "var(--radix-accordion-content-height)" },
        },
        "accordion-up": {
          from: { height: "var(--radix-accordion-content-height)" },
          to: { height: 0 },
        },
      },
      animation: {
        "accordion-down": "accordion-down 0.2s ease-out",
        "accordion-up": "accordion-up 0.2s ease-out",
      },
    },
  },
  plugins: [require("tailwindcss-animate")],
}
"#.to_string()
    }

    /// Generate lib/utils.ts for shadcn-vue project
    pub fn generate_utils_ts() -> String {
        r#"import { type ClassValue, clsx } from 'clsx'
import { twMerge } from 'tailwind-merge'

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}
"#.to_string()
    }

    /// Generate base CSS file with CSS variables
    pub fn generate_base_css() -> String {
        r#"@tailwind base;
@tailwind components;
@tailwind utilities;

@layer base {
  :root {
    --background: 0 0% 100%;
    --foreground: 222.2 84% 4.9%;
    --card: 0 0% 100%;
    --card-foreground: 222.2 84% 4.9%;
    --popover: 0 0% 100%;
    --popover-foreground: 222.2 84% 4.9%;
    --primary: 222.2 47.4% 11.2%;
    --primary-foreground: 210 40% 98%;
    --secondary: 210 40% 96.1%;
    --secondary-foreground: 222.2 47.4% 11.2%;
    --muted: 210 40% 96.1%;
    --muted-foreground: 215.4 16.3% 46.9%;
    --accent: 210 40% 96.1%;
    --accent-foreground: 222.2 47.4% 11.2%;
    --destructive: 0 84.2% 60.2%;
    --destructive-foreground: 210 40% 98%;
    --border: 214.3 31.8% 91.4%;
    --input: 214.3 31.8% 91.4%;
    --ring: 222.2 84% 4.9%;
    --radius: 0.5rem;
  }

  .dark {
    --background: 222.2 84% 4.9%;
    --foreground: 210 40% 98%;
    --card: 222.2 84% 4.9%;
    --card-foreground: 210 40% 98%;
    --popover: 222.2 84% 4.9%;
    --popover-foreground: 210 40% 98%;
    --primary: 210 40% 98%;
    --primary-foreground: 222.2 47.4% 11.2%;
    --secondary: 217.2 32.6% 17.5%;
    --secondary-foreground: 210 40% 98%;
    --muted: 217.2 32.6% 17.5%;
    --muted-foreground: 215 20.2% 65.1%;
    --accent: 217.2 32.6% 17.5%;
    --accent-foreground: 210 40% 98%;
    --destructive: 0 62.8% 30.6%;
    --destructive-foreground: 210 40% 98%;
    --border: 217.2 32.6% 17.5%;
    --input: 217.2 32.6% 17.5%;
    --ring: 212.7 26.8% 83.9%;
  }
}

@layer base {
  * {
    @apply border-border;
  }
  body {
    @apply bg-background text-foreground;
  }
}
"#.to_string()
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

    #[test]
    fn test_shadcn_mode() {
        let gen = VueGenerator::new_shadcn();
        assert!(gen.is_shadcn());

        let gen = VueGenerator::new().with_mode(VueMode::Shadcn);
        assert!(gen.is_shadcn());

        let gen = VueGenerator::new();
        assert!(!gen.is_shadcn());
    }

    #[test]
    fn test_shadcn_map_tag() {
        let mut gen = VueGenerator::new_shadcn();

        // Should return shadcn component names
        assert_eq!(gen.map_tag("button", false), "Button");
        assert_eq!(gen.map_tag("input", true), "Input");
        assert_eq!(gen.map_tag("textarea", true), "Textarea");
        assert_eq!(gen.map_tag("checkbox", true), "Checkbox");
        assert_eq!(gen.map_tag("toggle", true), "Switch");
        assert_eq!(gen.map_tag("select", false), "Select");
        assert_eq!(gen.map_tag("progress", true), "Progress");
        assert_eq!(gen.map_tag("badge", true), "Badge");
        assert_eq!(gen.map_tag("card", false), "Card");
        assert_eq!(gen.map_tag("avatar", true), "Avatar");
        assert_eq!(gen.map_tag("slider", true), "Slider");

        // Layout elements should still return div
        assert_eq!(gen.map_tag("col", false), "div");
        assert_eq!(gen.map_tag("row", false), "div");
    }

    #[test]
    fn test_shadcn_registry() {
        let registry = ShadcnRegistry::new();

        // Check component mappings exist
        assert!(registry.has_component("button"));
        assert!(registry.has_component("input"));
        assert!(registry.has_component("checkbox"));
        assert!(registry.has_component("modal"));
        assert!(registry.has_component("tabs"));
        assert!(registry.has_component("table"));

        // Check primary component names
        assert_eq!(registry.primary_component("button"), Some("Button"));
        assert_eq!(registry.primary_component("input"), Some("Input"));
        assert_eq!(registry.primary_component("toggle"), Some("Switch"));
    }

    #[test]
    fn test_generate_shadcn_attrs_button() {
        let gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        let events = HashMap::new();

        // Test button with text
        props.insert("text".to_string(), AuraPropValue::Expr(AuraExpr::Literal("Click me".to_string())));
        let (attrs, slot_content) = gen.generate_shadcn_attrs("button", &props, &events);

        assert!(slot_content.is_some());
        assert_eq!(slot_content.unwrap(), "Click me");
    }

    #[test]
    fn test_generate_shadcn_attrs_input() {
        let gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        let events = HashMap::new();

        // Test input with v-model
        props.insert("value".to_string(), AuraPropValue::Expr(AuraExpr::StateRef("name".to_string())));
        props.insert("placeholder".to_string(), AuraPropValue::Expr(AuraExpr::Literal("Enter name".to_string())));
        let (attrs, _) = gen.generate_shadcn_attrs("input", &props, &events);

        assert!(attrs.iter().any(|a| a.contains("v-model=\"name\"")));
        assert!(attrs.iter().any(|a| a.contains("placeholder=\"Enter name\"")));
    }

    #[test]
    fn test_generate_shadcn_attrs_checkbox() {
        let gen = VueGenerator::new_shadcn();
        let mut props = HashMap::new();
        let events = HashMap::new();

        // Test checkbox with v-model:checked
        props.insert("checked".to_string(), AuraPropValue::Expr(AuraExpr::StateRef("done".to_string())));
        let (attrs, _) = gen.generate_shadcn_attrs("checkbox", &props, &events);

        assert!(attrs.iter().any(|a| a.contains("v-model:checked=\"done\"")));
    }

    #[test]
    fn test_generate_project_files() {
        // Test scaffold file generation
        let components_json = VueGenerator::generate_components_json();
        assert!(components_json.contains("shadcn-vue"));
        assert!(components_json.contains("tailwind"));

        let package_json = VueGenerator::generate_package_json("test-project");
        assert!(package_json.contains("test-project"));
        assert!(package_json.contains("radix-vue"));
        assert!(package_json.contains("tailwind-merge"));

        let vite_config = VueGenerator::generate_vite_config();
        assert!(vite_config.contains("@vitejs/plugin-vue"));
        assert!(vite_config.contains("alias"));

        let utils_ts = VueGenerator::generate_utils_ts();
        assert!(utils_ts.contains("cn"));
        assert!(utils_ts.contains("clsx"));
        assert!(utils_ts.contains("tailwind-merge"));

        let base_css = VueGenerator::generate_base_css();
        assert!(base_css.contains("--background"));
        assert!(base_css.contains("--primary"));
    }
}
