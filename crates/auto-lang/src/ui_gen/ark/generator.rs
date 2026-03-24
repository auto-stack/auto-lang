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

use super::modifier::{prop_to_modifier, ArkModifierDsl};
use super::project::ArkProjectGenerator;
use super::state::{
    generate_dispatch_function, generate_handler_body, generate_interface,
    generate_interfaces_with_prefix, generate_msg_enum, generate_state_declarations_with_prefix,
};
use crate::ast::Type;
use crate::aura::{AuraExpr, AuraNode, AuraPropValue, AuraTextContent, AuraWidget, LogicPayload};
use crate::ui_gen::widget::WidgetRegistry;
use crate::ui_gen::{BackendGenerator, GenResult};
use std::collections::{HashMap, HashSet};

/// ArkTS code generator for HarmonyOS
///
/// This is the main entry point for generating ArkTS code from AURA widgets.
///
/// # Architecture
///
/// ```text
/// ArkGenerator
///     ├── WidgetRegistry (AURA → ArkTS component mappings)
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

    /// Widget registry for looking up widget specifications
    registry: WidgetRegistry,

    /// Custom widget imports (from use statements)
    custom_widgets: HashSet<String>,

    /// Collected modifiers for current component
    #[allow(dead_code)]
    current_modifiers: Vec<String>,

    /// Current indentation level
    indent_level: usize,

    /// Current widget's handlers (for event resolution)
    current_handlers: HashMap<String, LogicPayload>,

    /// Whether current widget has messages
    has_messages: bool,

    /// Loop variables in scope (to avoid prefixing with `this.`)
    loop_vars: HashSet<String>,

    /// State variable interface types (state var name -> interface type name)
    /// E.g., "items" -> "EnablementViewItem"
    state_interfaces: HashMap<String, String>,
}

impl ArkGenerator {
    /// Create a new ArkGenerator
    pub fn new() -> Self {
        Self {
            current_widget: None,
            registry: WidgetRegistry::with_defaults(),
            custom_widgets: HashSet::new(),
            current_modifiers: Vec::new(),
            indent_level: 0,
            current_handlers: HashMap::new(),
            has_messages: false,
            sanitized_name: None,
            loop_vars: HashSet::new(),
            state_interfaces: HashMap::new(),
        }
    }

    /// Register custom widget imports (from use statements)
    pub fn register_custom_widget(&mut self, name: &str) {
        self.custom_widgets.insert(name.to_string());
    }

    /// Register multiple custom widget imports
    pub fn register_custom_widgets(&mut self, names: &[&str]) {
        for name in names {
            self.custom_widgets.insert(name.to_string());
        }
    }

    /// Check if a tag looks like a custom component (starts with uppercase, not in registry)
    fn is_capitalized_component(&self, tag: &str) -> bool {
        // If it starts with uppercase and is not a built-in component, it's likely a custom widget
        tag.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
            && !Self::is_builtin_component(tag)
            && !self.registry.get(tag).is_some()
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

    /// Check if element is a Tabs container with TabsList + TabsContent children
    fn is_tabs_pattern(node: &AuraNode) -> bool {
        match node {
            AuraNode::Element { tag, children, .. } => {
                tag.to_lowercase() == "tabs" && children.iter().any(|c| {
                    matches!(c, AuraNode::Element { tag, .. } if tag.to_lowercase() == "tabslist" || tag.to_lowercase() == "tabscontent")
                })
            }
            _ => false,
        }
    }

    /// Check if widget view tree contains Tabs component
    fn widget_has_tabs(node: &AuraNode) -> bool {
        match node {
            AuraNode::Element { tag, children, .. } => {
                if tag.to_lowercase() == "tabs" {
                    return true;
                }
                children.iter().any(|c| Self::widget_has_tabs(c))
            }
            _ => false,
        }
    }

    /// Generate @Builder function for tab bar
    fn generate_tabs_builder(&self) -> String {
        let mut lines = Vec::new();

        lines.push("  @Builder".to_string());
        lines.push("  tabBarBuilder(title: string, targetIndex: number, selectedIcon?: Resource, unselectIcon?: Resource) {".to_string());
        lines.push("    Column() {".to_string());
        lines.push("      if (selectedIcon && unselectIcon) {".to_string());
        lines.push("        Image(this.currentIndex === targetIndex ? selectedIcon : unselectIcon)".to_string());
        lines.push("          .width(24)".to_string());
        lines.push("          .height(24)".to_string());
        lines.push("      }".to_string());
        lines.push("      Text(title)".to_string());
        lines.push("        .fontFamily('HarmonyHeiTi-Medium')".to_string());
        lines.push("        .fontSize(10)".to_string());
        lines.push("        .fontColor(this.currentIndex === targetIndex ? '#0A59F7' : 'rgba(0,0,0,0.60)')".to_string());
        lines.push("        .textAlign(TextAlign.Center)".to_string());
        lines.push("        .lineHeight(14)".to_string());
        lines.push("        .fontWeight(500)".to_string());
        lines.push("    }".to_string());
        lines.push("    .width('100%')".to_string());
        lines.push("    .height('100%')".to_string());
        lines.push("    .justifyContent(FlexAlign.Center)".to_string());
        lines.push("    .alignItems(HorizontalAlign.Center)".to_string());
        lines.push("    .onClick(() => {".to_string());
        lines.push("      this.currentIndex = targetIndex".to_string());
        lines.push("      this.tabsController.changeIndex(targetIndex)".to_string());
        lines.push("    })".to_string());
        lines.push("  }".to_string());

        lines.join("\n")
    }

    /// Generate Tabs component with TabContent children
    fn generate_tabs_component(&mut self, node: &AuraNode, tab_items: &[TabItem]) -> GenResult<String> {
        let mut lines = Vec::new();

        // Tabs header
        lines.push("    Tabs({ barPosition: BarPosition.End, controller: this.tabsController }) {".to_string());

        // Generate TabContent for each TabsContent child
        if let AuraNode::Element { children, .. } = node {
            let mut content_index = 0;
            for child in children {
                if let AuraNode::Element { tag, props, children: content_children, .. } = child {
                    if tag.to_lowercase() == "tabscontent" {
                        let tab_id = props.get("id").map(extract_string_from_prop).unwrap_or_default();

                        // Find matching tab item for label
                        let tab_item = tab_items.iter().find(|t| t.id == tab_id);

                        lines.push("      TabContent() {".to_string());

                        // Generate child content
                        for content_child in content_children {
                            let child_code = self.generate_node(content_child)?;
                            for line in child_code.lines() {
                                lines.push(format!("        {}", line));
                            }
                        }

                        lines.push("      }".to_string());

                        // Add tabBar with builder call
                        if let Some(item) = tab_item {
                            let icon_on = item.icon_on.as_deref().unwrap_or("");
                            let icon_off = item.icon_off.as_deref().unwrap_or("");
                            if !icon_on.is_empty() && !icon_off.is_empty() {
                                lines.push(format!("      .tabBar(this.tabBarBuilder('{}', {}, $r('{}'), $r('{}')))",
                                    item.label, content_index, icon_on, icon_off));
                            } else {
                                lines.push(format!("      .tabBar(this.tabBarBuilder('{}', {}))",
                                    item.label, content_index));
                            }
                        }

                        content_index += 1;
                    }
                }
            }
        }

        lines.push("    }".to_string());

        // Add Tabs modifiers
        lines.push("    .vertical(false)".to_string());
        lines.push("    .scrollable(false)".to_string());
        lines.push("    .backgroundColor('#F1F3F5')".to_string());

        Ok(lines.join("\n"))
    }
}

/// Extracted tab item data for @Builder generation
#[derive(Debug, Clone)]
pub struct TabItem {
    pub id: String,
    pub label: String,
    pub icon_on: Option<String>,
    pub icon_off: Option<String>,
}

/// Extract string from AuraPropValue (handles Expr(Literal) pattern)
fn extract_string_from_prop(value: &AuraPropValue) -> String {
    match value {
        AuraPropValue::Expr(AuraExpr::Literal(s)) => s.clone(),
        _ => String::new(),
    }
}

/// Extract optional string from AuraPropValue
fn extract_optional_string_from_prop(value: &AuraPropValue) -> Option<String> {
    match value {
        AuraPropValue::Expr(AuraExpr::Literal(s)) => Some(s.clone()),
        _ => None,
    }
}

/// Extract tab triggers from TabsList
fn extract_tab_triggers(tabs_list: &AuraNode) -> Vec<TabItem> {
    let mut items = Vec::new();

    if let AuraNode::Element { children, .. } = tabs_list {
        for child in children {
            if let AuraNode::Element { tag, props, .. } = child {
                if tag.to_lowercase() == "tabstrigger" {
                    let id = props.get("id").map(extract_string_from_prop).unwrap_or_default();
                    let label = props.get("label").map(extract_string_from_prop).unwrap_or_default();
                    items.push(TabItem {
                        id,
                        label,
                        icon_on: props.get("iconOn").and_then(extract_optional_string_from_prop),
                        icon_off: props.get("iconOff").and_then(extract_optional_string_from_prop),
                    });
                }
            }
        }
    }

    items
}

impl ArkGenerator {
    /// Sanitize widget name to avoid conflicts with built-in components
    fn sanitize_widget_name(name: &str) -> String {
        if Self::is_builtin_component(name) {
            // Append "Widget" suffix to avoid conflict
            format!("{}Widget", name)
        } else {
            name.to_string()
        }
    }

    /// Get the interface type for a state variable (for ForEach item type)
    fn get_interface_type(&self, state_var_name: &str) -> String {
        // Look up the interface type from state_interfaces map
        if let Some(interface_name) = self.state_interfaces.get(state_var_name) {
            return interface_name.clone();
        }
        // No interface defined - use `any` as fallback
        "any".to_string()
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

    /// Generate @Entry @Component struct from widget with custom imports
    pub fn generate_entry_component_with_imports(
        &mut self,
        widget: &AuraWidget,
        custom_imports: &[String],
    ) -> GenResult<String> {
        // Register custom widgets from imports
        for import in custom_imports {
            self.register_custom_widget(import);
        }
        self.generate_entry_component(widget)
    }

    /// Generate @Entry @Component struct from widget
    pub fn generate_entry_component(&mut self, widget: &AuraWidget) -> GenResult<String> {
        self.current_widget = Some(widget.name.clone());
        self.current_handlers = widget.handlers.clone();
        self.has_messages = !widget.messages.is_empty();

        // Sanitize widget name to avoid conflicts with built-in components
        let sanitized_name = Self::sanitize_widget_name(&widget.name);
        self.sanitized_name = Some(sanitized_name.clone());

        // Scan view tree for custom components (capitalized tags not in registry)
        let mut detected_components = HashSet::new();
        self.collect_custom_components(&widget.view_tree, &mut detected_components);
        self.custom_widgets.extend(detected_components);

        let mut lines = Vec::new();

        // Check if widget has routes
        let has_routes = widget.routes.is_some();
        // Check if this is the App widget (entry point)
        let is_app_widget = widget.name == "App";
        // Check if widget uses navigation (has links)
        let uses_navigation = Self::widget_uses_navigation(&widget.view_tree);

        // Find the index/default route component (first route or route with path "/")
        let index_component = if let Some(ref routes) = widget.routes {
            routes.routes.first().map(|r| Self::module_to_component(&r.module))
        } else {
            None
        };

        // Add import statement for ArkUI components (only Button - Column, Row, Text, NavPathStack are built-in)
        lines.push("import { Button } from '@kit.ArkUI';".to_string());
        lines.push(String::new());

        // For App widget with routes, import child pages
        if let Some(ref routes) = widget.routes {
            for route in &routes.routes {
                // Import uses actual widget name (e.g., Index from ./Index)
                let component_name = Self::capitalize_module(&route.module);
                lines.push(format!("import {{ {} }} from './{}';", component_name, component_name));
            }
            if !routes.routes.is_empty() {
                lines.push(String::new());
            }
        }

        // Import custom widgets used in view (detected from custom_widgets set)
        let custom_imports: Vec<_> = self.custom_widgets.iter()
            .filter(|w| {
                // Only import if it's used in the view tree
                Self::widget_uses_custom_component(&widget.view_tree, w)
            })
            .collect();

        for custom_widget in &custom_imports {
            lines.push(format!("import {{ {} }} from './{}';", custom_widget, custom_widget));
        }
        if !custom_imports.is_empty() {
            lines.push(String::new());
        }

        // Generate Msg enum if widget has messages (before @Entry/@Component)
        let msg_enum = generate_msg_enum(widget);
        if !msg_enum.is_empty() {
            lines.push(msg_enum);
            lines.push("".to_string());
        }

        // Generate interfaces for array-of-objects state variables (before @Entry/@Component)
        let interfaces = generate_interfaces_with_prefix(widget, &sanitized_name);

        // Store interface type mappings for use in ForEach generation
        for state_var in &widget.state_vars {
            let base_interface_name = super::state::to_pascal_case(&state_var.name);
            let prefixed_interface_name = format!("{}{}", sanitized_name, base_interface_name);
            if interfaces.iter().any(|i| i.name == prefixed_interface_name) {
                self.state_interfaces.insert(state_var.name.clone(), prefixed_interface_name);
            }
        }

        for interface in &interfaces {
            lines.push(generate_interface(interface));
            lines.push("".to_string());
        }

        // @Entry for App widget (with or without routes)
        // @Preview for child pages (helpful for DevEco Studio preview)
        if is_app_widget {
            lines.push("@Entry".to_string());
        } else {
            lines.push("@Preview".to_string());
        }
        lines.push("@Component".to_string());

        // Add export for child pages (non-App widgets)
        let struct_keyword = if is_app_widget { "struct" } else { "export struct" };
        lines.push(format!("{} {} {{", struct_keyword, sanitized_name));

        self.indent_level = 1;

        // Add NavPathStack with @Provide decorator if widget has routes (App widget)
        if has_routes {
            lines.push(format!("{}@Provide('pathStack') pathStack: NavPathStack = new NavPathStack()", self.indent()));
            lines.push(String::new());
        }

        // Add @Consume decorator if widget uses navigation but doesn't have routes (child page with links)
        if uses_navigation && !has_routes {
            lines.push(format!("{}@Consume('pathStack') pathStack: NavPathStack", self.indent()));
            lines.push(String::new());
        }

        // State declarations
        let state_decls = generate_state_declarations_with_prefix(widget, &sanitized_name);
        if !state_decls.is_empty() {
            for line in state_decls.lines() {
                lines.push(format!("{}{}", self.indent(), line));
            }
            lines.push("".to_string());
        }

        // Generate aboutToAppear() for widgets with NavParam decorator
        let about_to_appear = self.generate_about_to_appear(widget);
        if !about_to_appear.is_empty() {
            for line in about_to_appear.lines() {
                lines.push(format!("{}{}", self.indent(), line));
            }
            lines.push("".to_string());
        }

        // Check if widget contains Tabs - add controller and index state
        let has_tabs = Self::widget_has_tabs(&widget.view_tree);
        if has_tabs {
            lines.push(format!("{}@State currentIndex: number = 0", self.indent()));
            lines.push(format!("{}private tabsController: TabsController = new TabsController()", self.indent()));
            lines.push(String::new());
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

        // Generate @Builder for tabs if widget has tabs
        if has_tabs {
            let tabs_builder = self.generate_tabs_builder();
            for line in tabs_builder.lines() {
                lines.push(format!("{}{}", self.indent(), line));
            }
            lines.push(String::new());
        }

        // build() method
        lines.push(format!("{}build() {{", self.indent()));
        self.indent_level = 2;

        // Check if this widget uses navigation (has Link nodes)
        let uses_navigation = Self::widget_uses_navigation(&widget.view_tree);

        // For child pages that use navigation, wrap content in NavDestination
        // This is needed for @Consume('pathStack') to work properly
        // Regular widgets without navigation links don't need NavDestination
        let needs_nav_destination = !is_app_widget && uses_navigation;
        if needs_nav_destination {
            lines.push(format!("{}NavDestination() {{", self.indent()));
            self.indent_level += 1;
        }

        // Check if root is a custom component (for App widget, needs container wrapper)
        let root_is_custom_component = is_app_widget && !has_routes && self.is_custom_component_node(&widget.view_tree);
        if root_is_custom_component {
            // Wrap custom component in Column for @Entry requirement
            lines.push(format!("{}Column() {{", self.indent()));
            self.indent_level += 1;
        }

        // Generate UI tree from view_tree (not root)
        let ui_code = self.generate_node_with_routes(&widget.view_tree, has_routes, index_component.as_deref())?;
        for line in ui_code.lines() {
            lines.push(format!("{}{}", self.indent(), line));
        }

        // Close Column wrapper for custom component
        if root_is_custom_component {
            self.indent_level -= 1;
            lines.push(format!("{}}}", self.indent()));
        }

        // Close NavDestination for child pages that use navigation
        if needs_nav_destination {
            self.indent_level -= 1;
            lines.push(format!("{}}}", self.indent()));
        }

        self.indent_level = 1;
        lines.push(format!("{}}}", self.indent()));

        self.indent_level = 0;
        lines.push("}".to_string());

        Ok(lines.join("\n"))
    }

    /// Check if widget uses navigation (has Link nodes that navigate)
    fn widget_uses_navigation(node: &AuraNode) -> bool {
        match node {
            AuraNode::Link { .. } => true,
            AuraNode::Element { children, .. } => {
                children.iter().any(|c| Self::widget_uses_navigation(c))
            }
            AuraNode::Conditional { then_body, else_body, .. } => {
                then_body.iter().any(|c| Self::widget_uses_navigation(c))
                    || else_body.as_ref().map_or(false, |e| e.iter().any(|c| Self::widget_uses_navigation(c)))
            }
            AuraNode::ForLoop { body, .. } => {
                body.iter().any(|c| Self::widget_uses_navigation(c))
            }
            _ => false,
        }
    }

    /// Check if a view tree uses a specific custom component
    fn widget_uses_custom_component(node: &AuraNode, component_name: &str) -> bool {
        match node {
            AuraNode::Element { tag, children, .. } => {
                // Check if this element is the custom component
                if tag == component_name {
                    return true;
                }
                // Recursively check children
                children.iter().any(|c| Self::widget_uses_custom_component(c, component_name))
            }
            AuraNode::Conditional { then_body, else_body, .. } => {
                then_body.iter().any(|c| Self::widget_uses_custom_component(c, component_name))
                    || else_body.as_ref().map_or(false, |e| e.iter().any(|c| Self::widget_uses_custom_component(c, component_name)))
            }
            AuraNode::ForLoop { body, .. } => {
                body.iter().any(|c| Self::widget_uses_custom_component(c, component_name))
            }
            _ => false,
        }
    }

    /// Check if a node is a custom component (for @Entry root wrapping)
    fn is_custom_component_node(&self, node: &AuraNode) -> bool {
        match node {
            AuraNode::Element { tag, .. } => {
                // Check if this is a custom component (capitalized, not in registry)
                self.is_capitalized_component(tag)
            }
            AuraNode::Component { .. } => true,
            _ => false,
        }
    }

    /// Scan view tree for custom components (capitalized tags not in registry)
    fn collect_custom_components(&self, node: &AuraNode, components: &mut HashSet<String>) {
        match node {
            AuraNode::Element { tag, children, .. } => {
                // Check if this is a custom component (capitalized, not built-in, not in registry)
                if self.is_capitalized_component(tag) {
                    components.insert(tag.clone());
                }
                // Recursively scan children
                for child in children {
                    self.collect_custom_components(child, components);
                }
            }
            AuraNode::Conditional { then_body, else_body, .. } => {
                for child in then_body {
                    self.collect_custom_components(child, components);
                }
                if let Some(else_nodes) = else_body {
                    for child in else_nodes {
                        self.collect_custom_components(child, components);
                    }
                }
            }
            AuraNode::ForLoop { body, .. } => {
                for child in body {
                    self.collect_custom_components(child, components);
                }
            }
            _ => {}
        }
    }

    /// Convert page module name to @Builder function name
    fn page_to_builder_name(module: &str) -> String {
        // e.g., "counter" -> "CounterBuilder"
        let mut chars = module.chars();
        let first = chars.next().map(|c| c.to_uppercase().collect::<String>()).unwrap_or_default();
        let rest: String = chars.collect();
        format!("{}{}Builder", first, rest)
    }

    /// Capitalize module name (e.g., "counter" -> "Counter", "index" -> "Index")
    fn capitalize_module(module: &str) -> String {
        let mut chars = module.chars();
        let first = chars.next().map(|c| c.to_uppercase().collect::<String>()).unwrap_or_default();
        let rest: String = chars.collect();
        format!("{}{}", first, rest)
    }

    /// Generate props string for custom component constructor
    fn generate_custom_component_props(&self, props: &HashMap<String, AuraPropValue>) -> String {
        if props.is_empty() {
            return String::new();
        }

        let mut prop_pairs: Vec<String> = Vec::new();
        for (key, value) in props {
            // Skip style and class as they're not model props
            if key == "style" || key == "class" {
                continue;
            }
            let value_str = match value {
                AuraPropValue::Expr(expr) => self.expr_to_ark_string(expr),
                AuraPropValue::StyleBinding(_) => continue,
            };
            prop_pairs.push(format!("{}: {}", key, value_str));
        }

        if prop_pairs.is_empty() {
            String::new()
        } else {
            format!("{{ {} }}", prop_pairs.join(", "))
        }
    }

    /// Convert module name to component name
    /// This should match the actual widget name in the source file
    fn module_to_component(module: &str) -> String {
        // e.g., "counter" -> "Counter", "index" -> "Index"
        Self::capitalize_module(module)
    }

    /// Generate ArkTS code for a node, with route awareness
    fn generate_node_with_routes(&mut self, node: &AuraNode, has_routes: bool, index_component: Option<&str>) -> GenResult<String> {
        match node {
            AuraNode::Element {
                tag,
                props,
                events,
                children,
            } => {
                // Special handling for root col when routes exist - wrap in Navigation
                if tag.to_lowercase() == "col" && has_routes {
                    return self.generate_navigation_root(props, events, children, index_component);
                }
                self.generate_element(tag, props, events, children)
            }
            AuraNode::Outlet => {
                // Outlet in navigation context - render index page directly
                if let Some(index) = index_component {
                    Ok(format!("{}()", index))
                } else {
                    Ok("// Outlet - no default route".to_string())
                }
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
        index_component: Option<&str>,
    ) -> GenResult<String> {
        let mut lines = Vec::new();

        // Navigation component with pathStack
        lines.push("Navigation(this.pathStack) {".to_string());
        self.indent_level += 1;

        // Generate children (header, outlet, etc.)
        for child in children {
            // Pass index_component to handle Outlet replacement
            let child_code = self.generate_node_with_routes(child, true, index_component)?;
            for line in child_code.lines() {
                lines.push(format!("{}{}", self.indent(), line));
            }
        }

        self.indent_level -= 1;
        lines.push(format!("{}}}", self.indent()));

        // Add navDestination modifier for route handling
        lines.push(format!("{}.navDestination(this.buildNavDestination)", self.indent()));

        // Add common Navigation modifiers
        lines.last_mut().unwrap().push_str("\n    .hideTitleBar(true)");
        lines.last_mut().unwrap().push_str("\n    .mode(NavigationMode.Stack)");

        // Add modifiers
        let modifiers = self.generate_modifiers(props, events, None, Some("Navigation"));
        if !modifiers.is_empty() {
            lines.last_mut().unwrap().push_str(&modifiers);
        }

        Ok(lines.join("\n"))
    }

    /// Generate buildNavDestination builder for navDestination
    #[allow(dead_code)]
    fn generate_nav_destination_builder(&self, routes: &crate::aura::AuraRoutes) -> String {
        let mut lines = Vec::new();

        lines.push("@Builder".to_string());
        lines.push("buildNavDestination(name: string) {".to_string());

        for (i, route) in routes.routes.iter().enumerate() {
            let component_name = Self::module_to_component(&route.module);
            if route.module == "index" || i == 0 {
                lines.push(format!("  if (name === '{}') {{", route.module));
                lines.push(format!("    {}()", component_name));
                lines.push("  }".to_string());
            } else {
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

        // Check for Tabs pattern before regular element handling
        // Tags may be lowercase or PascalCase (e.g., "tabslist" or "TabsList")
        if tag.to_lowercase() == "tabs" && children.iter().any(|c| {
            matches!(c, AuraNode::Element { tag, .. } if tag.to_lowercase() == "tabslist" || tag.to_lowercase() == "tabscontent")
        }) {
            // Extract TabsList children
            let tabs_list = children.iter().find(|c| {
                matches!(c, AuraNode::Element { tag, .. } if tag.to_lowercase() == "tabslist")
            });

            let tab_items = if let Some(list) = tabs_list {
                extract_tab_triggers(list)
            } else {
                Vec::new()
            };

            // Create a temporary node for the tabs pattern
            let tabs_node = AuraNode::Element {
                tag: tag.to_string(),
                props: props.clone(),
                events: events.clone(),
                children: children.to_vec(),
            };

            return self.generate_tabs_component(&tabs_node, &tab_items);
        }

        // Look up widget in the new widget registry
        if let Some(widget) = self.registry.get(tag) {
            // Merge default props from widget spec with user-provided props
            // User props take precedence over default props
            let mut merged_props = props.clone();
            for (key, value) in &widget.default_props {
                if !merged_props.contains_key(key) {
                    merged_props.insert(key.clone(), AuraPropValue::Expr(AuraExpr::Literal(value.clone())));
                }
            }

            // Get the ArkTS backend mapping
            if let Some(ark_mapping) = widget.backend("ark") {
                let component_name = &ark_mapping.component;

                // Get content argument from primary_prop
                // The primary_prop defines the shorthand property (e.g., Text "Hello" uses "text" prop)
                let content_arg = if let Some(primary_prop) = &widget.primary_prop {
                    if let Some(prop_value) = merged_props.get(primary_prop) {
                        match prop_value {
                            AuraPropValue::Expr(expr) => {
                                self.expr_to_ark_string(expr)
                            }
                            _ => String::new()
                        }
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };

                // Component call with content argument
                let component_call = if content_arg.is_empty() {
                    format!("{}()", component_name)
                } else {
                    format!("{}({})", component_name, content_arg)
                };

                // Generate modifiers (to be placed AFTER the component body)
                let modifiers = self.generate_modifiers(&merged_props, events, widget.primary_prop.as_deref(), Some(tag));

                // Start component call
                lines.push(component_call);

                // Children - body comes BEFORE modifiers
                // Use has_children from widget spec
                if widget.has_children && !children.is_empty() {
                    lines.last_mut().unwrap().push_str(" {");
                    self.indent_level += 1;

                    for child in children {
                        let child_code = self.generate_node(child)?;
                        for line in child_code.lines() {
                            lines.push(format!("{}{}", self.indent(), line));
                        }
                    }

                    self.indent_level -= 1;
                    // Close body, then add modifiers on same line
                    let closing = format!("{}}}", self.indent());
                    if modifiers.is_empty() {
                        lines.push(closing);
                    } else {
                        lines.push(format!("{}{}", closing, modifiers));
                    }
                } else {
                    // No children - add modifiers directly
                    if !modifiers.is_empty() {
                        lines.last_mut().unwrap().push_str(&modifiers);
                    }
                }
            } else {
                // No ArkTS mapping - emit as comment
                lines.push(format!("/* No ArkTS mapping for: {} */", tag));
            }
        } else if self.custom_widgets.contains(tag) || self.is_capitalized_component(tag) {
            // Custom widget (from use statement) - call it directly as a component
            let component_name = Self::capitalize_module(tag);

            // Generate props as constructor arguments
            let props_str = self.generate_custom_component_props(props);
            lines.push(format!("{}({})", component_name, props_str));

            // Handle children if any
            if !children.is_empty() {
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
}

/// Get sort order for a modifier string (lower = earlier in output)
fn modifier_order(modifier: &str) -> u8 {
    // Order: width/height -> layout (align/justify) -> spacing -> typography -> visual -> events
    if modifier.contains(".width") || modifier.contains(".height") {
        1
    } else if modifier.contains(".alignItems") || modifier.contains(".justifyContent") {
        2
    } else if modifier.contains(".padding") || modifier.contains(".margin") {
        3
    } else if modifier.contains(".fontSize") || modifier.contains(".fontWeight") || modifier.contains(".fontColor") || modifier.contains(".fontFamily") {
        4
    } else if modifier.contains(".backgroundColor") || modifier.contains(".borderRadius") || modifier.contains(".border") {
        5
    } else if modifier.contains(".onClick") {
        10
    } else {
        8 // Default: middle of the pack
    }
}

impl ArkGenerator {
    /// Generate modifiers from props and events
    fn generate_modifiers(
        &self,
        props: &HashMap<String, AuraPropValue>,
        events: &HashMap<String, crate::aura::AuraEvent>,
        primary_prop: Option<&str>,
        tag: Option<&str>,
    ) -> String {
        let mut modifiers = Vec::new();
        let dsl = ArkModifierDsl::new();

        // Process props - extract string/number values from AuraExpr
        for (key, value) in props {
            // Skip props that are handled as constructor arguments (primary_prop)
            if let Some(primary) = primary_prop {
                if key == primary {
                    continue;
                }
            }
            // Handle style prop using ArkModifierDsl
            if key == "style" || key == "class" {
                if let Some(style_str) = self.extract_style_string(value) {
                    let style_modifiers = dsl.convert_style_with_tag(&style_str, tag);
                    modifiers.extend(style_modifiers);
                }
                continue;
            }
            if let Some(modifier) = self.prop_to_modifier(key, value) {
                modifiers.push(modifier);
            }
        }

        // Process events - generate onClick handlers
        for (event_name, event) in events {
            let event_lower = event_name.to_lowercase();
            if event_lower == "click" || event_lower == "onclick" {
                // Check for nav() function call
                let handler_code = if event.handler == "nav" {
                    // Generate pathStack.pushPathByName() call
                    // params[0] = route name, params[1] = optional data
                    self.generate_nav_call(&event.params)
                } else if event.handler == "console" {
                    // Generate console.log() call
                    self.generate_console_call(&event.params)
                } else if self.has_messages && event.handler.starts_with('.') {
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

        // Process style bindings from StyleBinding variant
        for value in props.values() {
            if let AuraPropValue::StyleBinding(bindings) = value {
                for binding in bindings {
                    // Use ArkModifierDsl for style conversion
                    let style_modifiers = dsl.convert_style_with_tag(&binding.style_name, tag);
                    // For now, apply the style unconditionally
                    // TODO: Support conditional style application
                    modifiers.extend(style_modifiers);
                }
            }
        }

        if modifiers.is_empty() {
            String::new()
        } else {
            // Sort modifiers for stable output order
            modifiers.sort_by(|a, b| modifier_order(a).cmp(&modifier_order(b)));
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

    /// Extract style string from AuraPropValue
    fn extract_style_string(&self, value: &AuraPropValue) -> Option<String> {
        match value {
            AuraPropValue::Expr(AuraExpr::Literal(s)) => Some(s.clone()),
            AuraPropValue::StyleBinding(bindings) => {
                // Combine all style names
                Some(bindings.iter().map(|b| b.style_name.as_str()).collect::<Vec<_>>().join(" "))
            }
            _ => None,
        }
    }

    /// Convert prop to modifier
    fn prop_to_modifier(&self, key: &str, value: &AuraPropValue) -> Option<String> {
        match value {
            AuraPropValue::Expr(expr) => self.expr_to_modifier(key, expr),
            AuraPropValue::StyleBinding(_) => None, // Handled separately
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

    /// Generate nav() call to pathStack.pushPathByName()
    ///
    /// Takes params like ["articleDetail", "this.item"] and generates:
    /// `this.pathStack.pushPathByName('articleDetail', this.item)`
    fn generate_nav_call(&self, params: &[String]) -> String {
        if params.is_empty() {
            return "// nav() requires route name".to_string();
        }

        let route_name = &params[0];
        let nav_param = if params.len() > 1 {
            // Join remaining params as the second argument
            let param_str = params[1..].join(", ");
            // If it's an object literal, add type annotation for ArkTS
            // Use ES Object type wrapper to satisfy ArkTS type checking
            if param_str.starts_with("{ ") && param_str.ends_with(" }") {
                // Create an ES Object with the properties
                let obj_content = &param_str[2..param_str.len()-2]; // Remove "{ " and " }"
                let props: Vec<&str> = obj_content.split(", ").collect();
                let mut obj_builder = String::from("Object({");
                for (i, prop) in props.iter().enumerate() {
                    if i > 0 {
                        obj_builder.push_str(", ");
                    }
                    obj_builder.push_str(prop);
                }
                obj_builder.push_str("})");
                obj_builder
            } else {
                param_str
            }
        } else {
            // Empty string for no param
            "''".to_string()
        };

        format!("this.pathStack.pushPathByName({}, {})", route_name, nav_param)
    }

    /// Generate console.log() call from params
    ///
    /// Takes params like ["'clicked'", "this.title"] and generates:
    /// `console.log('clicked', this.title)`
    fn generate_console_call(&self, params: &[String]) -> String {
        if params.is_empty() {
            return "console.log('click')".to_string();
        }

        format!("console.log({})", params.join(", "))
    }

    /// Generate aboutToAppear() lifecycle method for NavDestination pages
    ///
    /// Generates code to retrieve navigation params using getParamByName()
    /// Example:
    /// ```
    /// aboutToAppear(): void {
    ///   this.articleDetail = this.pathStack.getParamByName('articleDetail')[0] as ArticleClass;
    /// }
    /// ```
    fn generate_about_to_appear(&self, widget: &AuraWidget) -> String {
        let mut nav_params = Vec::new();

        // Find state vars with NavParam decorator
        for state_var in &widget.state_vars {
            for decorator in &state_var.decorators {
                if decorator.name == "NavParam" {
                    // Get route name from decorator arg
                    let route_name = decorator.args.first()
                        .map(|s| s.as_str())
                        .unwrap_or(&state_var.name);

                    // Get type name for casting
                    let type_name = Self::type_to_ark_string(&state_var.type_info);

                    nav_params.push((state_var.name.clone(), route_name.to_string(), type_name));
                }
            }
        }

        if nav_params.is_empty() {
            return String::new();
        }

        let mut lines = Vec::new();
        lines.push("aboutToAppear(): void {".to_string());

        for (var_name, route_name, type_name) in nav_params {
            lines.push(format!("  this.{} = this.pathStack.getParamByName('{}')[0] as {}",
                var_name, route_name, type_name));
        }

        lines.push("}".to_string());
        lines.join("\n")
    }

    /// Convert Type to ArkTS type string
    fn type_to_ark_string(ty: &Type) -> String {
        match ty {
            Type::Int | Type::Uint | Type::I64 | Type::U64 | Type::Float | Type::Double => "number".to_string(),
            Type::Bool => "boolean".to_string(),
            Type::Str(_) | Type::CStr | Type::StrSlice => "string".to_string(),
            Type::User(type_decl) => type_decl.name.to_string(),
            Type::Option(inner) => format!("{} | null", Self::type_to_ark_string(inner)),
            _ => ty.unique_name().to_string(), // Fallback
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

    /// Convert AuraExpr to ArkTS code string
    fn expr_to_ark_string(&self, expr: &AuraExpr) -> String {
        match expr {
            AuraExpr::Literal(value) => {
                // Check if it's a resource reference like $r('app.media.xxx')
                if value.starts_with("$r(") {
                    value.clone()
                } else {
                    format!("'{}'", value)
                }
            }
            AuraExpr::StateRef(field) => {
                // Check if this is a loop variable (should not be prefixed with `this.`)
                if self.loop_vars.contains(field.as_str()) {
                    field.clone()
                } else {
                    format!("this.{}", field)
                }
            }
            AuraExpr::FieldAccess { object, field } => {
                let obj_str = self.expr_to_ark_string(object);
                format!("{}.{}", obj_str, field)
            }
            AuraExpr::Int(n) => n.to_string(),
            AuraExpr::Float(f) => f.to_string(),
            AuraExpr::Bool(b) => b.to_string(),
            AuraExpr::Array(elems) => {
                let items: Vec<String> = elems.iter()
                    .map(|e| self.expr_to_ark_string(e))
                    .collect();
                format!("[{}]", items.join(", "))
            }
            AuraExpr::Object(fields) => {
                let pairs: Vec<String> = fields.iter()
                    .map(|(k, v)| {
                        let val = self.expr_to_ark_string(v);
                        format!("{}: {}", k, val)
                    })
                    .collect();
                format!("{{{}}}", pairs.join(", "))
            }
            _ => String::new(),
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

        let index_name = index.unwrap_or("index");
        let index_param = index.map(|i| format!(", {}: number", i)).unwrap_or_default();

        // Strip leading dot from iterable if present (e.g., ".items" -> "items")
        let iterable_name = iterable.strip_prefix('.').unwrap_or(iterable);

        // Add loop variable to loop_vars so expr_to_ark_string knows not to prefix with `this.`
        self.loop_vars.insert(var.to_string());
        if let Some(idx) = index {
            self.loop_vars.insert(idx.to_string());
        }

        // Get what interface type to use for loop variable
        let item_type = self.get_interface_type(iterable_name);

        // Generate ForEach with item type and key function
        lines.push(format!(
            "{}ForEach(this.{}, ({}: {}) => {{",
            self.indent(),
            iterable_name,
            var,
            item_type
        ));
        self.indent_level += 1;

        for child in body {
            let child_code = self.generate_node(child)?;
            for line in child_code.lines() {
                lines.push(format!("{}{}", self.indent(), line));
            }
        }

        self.indent_level -= 1;

        // Add key function with proper types and return type
        // Key function: (item: Type): string => item.id)
        lines.push(format!(
            "{}}}, ({}: {}): string => {}.id)",
            self.indent(),
            var,
            item_type,
            var
        ));

        // Remove loop variable from loop_vars
        self.loop_vars.remove(var);
        if let Some(idx) = index {
            self.loop_vars.remove(idx);
        }

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
            // ArkTS syntax: Column() { children }.onClick(() => { ... })
            lines.push(format!("Column() {{"));
            self.indent_level += 1;
            lines.push(format!("{}Text(\"{}\")", self.indent(), text));
            self.indent_level -= 1;
            lines.push(format!("{}}}", self.indent()));
            // onClick modifier comes AFTER the closing brace
            lines.push(format!("{}.onClick(() => {{", self.indent()));
            lines.push(format!("{}  this.pathStack.pushPathByName('{}', '')", self.indent(), route_name));
            lines.push(format!("{}}})", self.indent()));
        } else if !children.is_empty() {
            // Link with children
            lines.push(format!("Column() {{"));
            self.indent_level += 1;

            for child in children {
                let child_code = self.generate_node(child)?;
                for line in child_code.lines() {
                    lines.push(format!("{}{}", self.indent(), line));
                }
            }

            self.indent_level -= 1;
            lines.push(format!("{}}}", self.indent()));
            // onClick modifier comes AFTER the closing brace
            lines.push(format!("{}.onClick(() => {{", self.indent()));
            lines.push(format!("{}  this.pathStack.pushPathByName('{}', '')", self.indent(), route_name));
            lines.push(format!("{}}})", self.indent()));
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
    fn test_generator_uses_widget_registry() {
        // The generator should have access to the widget registry
        // and it should have default widgets registered
        let gen = ArkGenerator::new();

        // Test that the registry has default widgets
        let widget = gen.registry.get("col");
        assert!(widget.is_some(), "col widget should be registered");
        assert_eq!(widget.unwrap().name, "Column");

        // Test other default widgets
        assert!(gen.registry.get("row").is_some());
        assert!(gen.registry.get("text").is_some());
        assert!(gen.registry.get("button").is_some());
        assert!(gen.registry.get("image").is_some());
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
                decorators: vec![],
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
        // Create a simple widget without routes (child page - should have @Component only)
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

        // Import should appear before @Component decorator (child pages don't have @Entry)
        let import_pos = code.find("import").expect("Import should exist");
        let component_pos = code.find("@Component").expect("@Component should exist");
        assert!(
            import_pos < component_pos,
            "Import should appear before @Component decorator"
        );

        // Child pages (without routes) should NOT have @Entry
        assert!(
            !code.contains("@Entry"),
            "Child pages without routes should not have @Entry decorator"
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

    #[test]
    fn test_app_widget_with_routes_has_entry() {
        // Test that App widget with routes has @Entry and @Provide
        use crate::aura::{AuraRoute, AuraRoutes};

        let widget = AuraWidget {
            name: "App".to_string(),
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
            routes: Some(AuraRoutes {
                routes: vec![AuraRoute {
                    path: "/".to_string(),
                    module: "index".to_string(),
                    params: vec![],
                }],
            }),
        };

        let mut gen = ArkGenerator::new();
        let code = gen.generate_entry_component(&widget).unwrap();

        // App widget with routes should have @Entry
        assert!(
            code.contains("@Entry"),
            "App widget with routes should have @Entry decorator"
        );

        // App widget should have @Provide for pathStack
        assert!(
            code.contains("@Provide('pathStack')"),
            "App widget should have @Provide('pathStack') for navigation"
        );

        // NavPathStack is built-in, should NOT be imported
        assert!(
            !code.contains("import { NavPathStack }"),
            "NavPathStack is built-in and should not be imported"
        );

        // App widget should import child pages
        assert!(
            code.contains("import { Index } from './Index';"),
            "App widget should import child pages"
        );

        // App widget should have Navigation component
        assert!(
            code.contains("Navigation(this.pathStack)"),
            "App widget should have Navigation component with pathStack"
        );
    }

    #[test]
    fn test_app_widget_without_routes_is_entry_page() {
        // Test that App widget without routes is a simple entry page (no Navigation)
        let widget = AuraWidget {
            name: "App".to_string(),
            state_vars: vec![],
            computed: vec![],
            messages: vec![],
            view_tree: AuraNode::Element {
                tag: "col".to_string(),
                props: HashMap::new(),
                events: HashMap::new(),
                children: vec![AuraNode::Text(AuraTextContent::Literal("Hello, World!".to_string()))],
            },
            handlers: HashMap::new(),
            props: vec![],
            routes: None, // No routes - simple entry page
        };

        let mut gen = ArkGenerator::new();
        let code = gen.generate_entry_component(&widget).unwrap();

        // App widget without routes should STILL have @Entry (it's the entry point)
        assert!(
            code.contains("@Entry"),
            "App widget without routes should have @Entry decorator (it's the entry page)"
        );

        // App widget without routes should NOT have @Provide (no navigation)
        assert!(
            !code.contains("@Provide('pathStack')"),
            "App widget without routes should not have @Provide('pathStack')"
        );

        // App widget without routes should NOT have Navigation component
        assert!(
            !code.contains("Navigation("),
            "App widget without routes should not have Navigation component"
        );

        // App widget without routes should NOT be wrapped in NavDestination
        assert!(
            !code.contains("NavDestination()"),
            "App widget without routes should not be wrapped in NavDestination"
        );

        // App widget without routes should NOT be exported (it's the entry)
        assert!(
            !code.contains("export struct"),
            "App widget should not be exported (it's the entry point)"
        );
    }

    #[test]
    fn test_child_page_with_navigation_has_consume() {
        // Test that child pages with navigation links have @Consume and are wrapped in NavDestination
        let widget = AuraWidget {
            name: "IndexPage".to_string(),
            state_vars: vec![],
            computed: vec![],
            messages: vec![],
            view_tree: AuraNode::Link {
                to: "counter".to_string(),
                text: "Go to Counter".to_string(),
                href: String::new(),
                children: vec![],
            },
            handlers: HashMap::new(),
            props: vec![],
            routes: None, // No routes - this is a child page
        };

        let mut gen = ArkGenerator::new();
        let code = gen.generate_entry_component(&widget).unwrap();

        // Child page should NOT have @Entry
        assert!(
            !code.contains("@Entry"),
            "Child page without routes should not have @Entry decorator"
        );

        // Child page WITH navigation links should have @Consume for pathStack
        assert!(
            code.contains("@Consume('pathStack')"),
            "Child page with navigation links should have @Consume('pathStack')"
        );

        // Child page should be wrapped in NavDestination
        assert!(
            code.contains("NavDestination()"),
            "Child page content should be wrapped in NavDestination()"
        );

        // NavPathStack is built-in, should NOT be imported
        assert!(
            !code.contains("import { NavPathStack }"),
            "NavPathStack is built-in and should not be imported"
        );

        // Child page should have export keyword
        assert!(
            code.contains("export struct"),
            "Child page should be exported with 'export struct'"
        );
    }

    #[test]
    fn test_image_with_url_source() {
        // Test that Image component with URL source generates correct code
        let mut props = HashMap::new();
        props.insert("src".to_string(), AuraPropValue::Expr(AuraExpr::Literal("https://example.com/logo.png".to_string())));

        let widget = AuraWidget {
            name: "TestApp".to_string(),
            state_vars: vec![],
            computed: vec![],
            messages: vec![],
            view_tree: AuraNode::Element {
                tag: "image".to_string(),
                props,
                events: HashMap::new(),
                children: vec![],
            },
            handlers: HashMap::new(),
            props: vec![],
            routes: None,
        };

        let mut gen = ArkGenerator::new();
        let code = gen.generate_entry_component(&widget).unwrap();

        // Image should have URL as constructor argument (quoted)
        assert!(
            code.contains("Image('https://example.com/logo.png')"),
            "Image with URL should have quoted URL as constructor argument, got: {}",
            code
        );

        // Should NOT have .src() modifier
        assert!(
            !code.contains(".src("),
            "Image src should NOT be a modifier, got: {}",
            code
        );
    }

    #[test]
    fn test_image_with_resource_reference() {
        // Test that Image component with $r() resource reference generates correct code
        let mut props = HashMap::new();
        props.insert("src".to_string(), AuraPropValue::Expr(AuraExpr::Literal("$r('app.media.icon')".to_string())));

        let widget = AuraWidget {
            name: "TestApp".to_string(),
            state_vars: vec![],
            computed: vec![],
            messages: vec![],
            view_tree: AuraNode::Element {
                tag: "image".to_string(),
                props,
                events: HashMap::new(),
                children: vec![],
            },
            handlers: HashMap::new(),
            props: vec![],
            routes: None,
        };

        let mut gen = ArkGenerator::new();
        let code = gen.generate_entry_component(&widget).unwrap();

        // Image should have $r() as constructor argument (NOT quoted)
        assert!(
            code.contains("Image($r('app.media.icon'))"),
            "Image with $r() should have resource reference as constructor argument without quotes, got: {}",
            code
        );

        // Should NOT have extra quotes around $r()
        assert!(
            !code.contains("Image('$r("),
            "Image $r() should NOT be wrapped in extra quotes, got: {}",
            code
        );
    }

    // ============================================================================
    // a2ark Test Framework - AURA -> ArkTS transpilation tests
    // ============================================================================

    use std::path::PathBuf;
    use std::fs::{read_to_string, File};
    use std::io::Write;

    /// Helper function for a2ark tests
    ///
    /// This function reads an AURA widget from test/a2ark/{case}/input.at,
    /// generates ArkTS code, and compares it with input.expected.ets.
    /// If the output differs, it writes to input.wrong.ets for debugging.
    fn test_a2ark(case: &str) -> Result<(), Box<dyn std::error::Error>> {
        let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        println!("Directory of cargo: {}", d.display());

        let src_path = d.join(format!("test/a2ark/{}/input.at", case));
        println!("src_path: {}", src_path.display());

        let src = read_to_string(&src_path)?;
        println!("Source:\n{}", src);

        // Parse the AURA source file with UI scenario (required for widget syntax)
        let session = crate::session::CompilerSession::ui();
        let mut parser = crate::parser::Parser::from(src.as_str());
        parser = parser.with_session(session);
        let ast = parser.parse()?;

        // Extract AURA widgets from AST
        let mut widgets = Vec::new();
        for stmt in &ast.stmts {
            if let crate::ast::Stmt::WidgetDecl(widget_decl) = stmt {
                let aura_widget = crate::aura::extract_widget_from_decl(widget_decl)?;
                widgets.push(aura_widget);
            }
        }

        if widgets.is_empty() {
            return Err("No widget declarations found in input file".into());
        }

        // Generate ArkTS code for each widget
        let mut gen = super::ArkGenerator::new();
        let mut output = String::new();
        for widget in &widgets {
            let code = gen.generate_entry_component(widget)?;
            output.push_str(&code);
            output.push('\n');
        }

        // Read expected output
        let exp_path = d.join(format!("test/a2ark/{}/input.expected.ets", case));
        let expected = if exp_path.is_file() {
            read_to_string(&exp_path)?
        } else {
            // Create empty expected file if it doesn't exist
            String::new()
        };

        // Normalize whitespace for comparison (trailing whitespace, newlines)
        let output_normalized = normalize_output(&output);
        let expected_normalized = normalize_output(&expected);

        if output_normalized != expected_normalized {
            // Write wrong output for debugging
            let wrong_path = d.join(format!("test/a2ark/{}/input.wrong.ets", case));
            let mut file = File::create(&wrong_path)?;
            file.write_all(output.as_bytes())?;
            println!("Written wrong output to: {}", wrong_path.display());

            return Err(format!(
                "Output mismatch for {}. See input.wrong.ets for actual output.\nExpected:\n{}\n\nActual:\n{}",
                case, expected_normalized, output_normalized
            ).into());
        }

        Ok(())
    }

    /// Normalize output for comparison (trim trailing whitespace, normalize newlines)
    fn normalize_output(s: &str) -> String {
        s.lines()
            .map(|line| line.trim_end())
            .collect::<Vec<_>>()
            .join("\n")
            .trim_end()
            .to_string()
    }

    #[test]
    fn test_001_column() {
        test_a2ark("001_column").unwrap();
    }

    #[test]
    fn test_002_row() {
        test_a2ark("002_row").unwrap();
    }

    #[test]
    fn test_003_box() {
        test_a2ark("003_box").unwrap();
    }

    #[test]
    fn test_004_text() {
        test_a2ark("004_text").unwrap();
    }

    #[test]
    fn test_005_button() {
        test_a2ark("005_button").unwrap();
    }

    #[test]
    fn test_006_input() {
        test_a2ark("006_input").unwrap();
    }

    #[test]
    fn test_007_image() {
        test_a2ark("007_image").unwrap();
    }

    #[test]
    fn test_008_form_widgets() {
        test_a2ark("008_form_widgets").unwrap();
    }

    #[test]
    fn test_010_table() {
        test_a2ark("010_table").unwrap();
    }

    #[test]
    fn test_011_tabs() {
        test_a2ark("011_tabs").unwrap();
    }

    #[test]
    fn test_012_dialog() {
        test_a2ark("012_dialog").unwrap();
    }

    #[test]
    fn test_013_for_loop() {
        test_a2ark("013_for_loop").unwrap();
    }

    #[test]
    fn test_014_array_objects() {
        test_a2ark("014_array_objects").unwrap();
    }

    #[test]
    fn test_015_tabs() {
        test_a2ark("015_tabs").unwrap();
    }

    #[test]
    fn test_016_nav() {
        test_a2ark("016_nav").unwrap();
    }

    #[test]
    fn test_017_decorators() {
        test_a2ark("017_decorators").unwrap();
    }
}
