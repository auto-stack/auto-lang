# 134: Jet Generator View Body Implementation

> **Status:** ✅ COMPLETED (2025-03-19)

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement `generate_view_body()` in JetGenerator to fully support AURA view_tree syntax, enabling complete transpilation from Auto language to Jetpack Compose Kotlin code.

**Architecture:** The implementation will follow the same recursive node traversal pattern used in `vue.rs`, but output Kotlin/Compose code instead of HTML. We will add a `node_to_compose()` method that handles each `AuraNode` variant, mapping AURA tags to Compose components.

**Tech Stack:** Rust, Kotlin, Jetpack Compose, Material3

---

## Implementation Summary

**Completed Tasks:**
1. ✅ Added `node_to_compose()` method skeleton with recursive AuraNode traversal
2. ✅ Implemented `text_to_compose()` for Text nodes (literal and interpolated)
3. ✅ Implemented `element_to_compose()` with layout/form/list/generic dispatch
4. ✅ Implemented layout element conversion (col, row, box, card, scroll)
5. ✅ Implemented form element conversion (button, input, checkbox, switch, slider)
6. ✅ Implemented `for_loop_to_compose()` using items()/itemsIndexed()
7. ✅ Implemented `conditional_to_compose()` using Kotlin if/else
8. ✅ Implemented `component_to_compose()` for child component references
9. ✅ Implemented `link_to_compose()` for navigation
10. ✅ Implemented `generic_element_to_compose()` for HTML-like tags
11. ✅ Updated `generate_view_body()` to use `node_to_compose()`
12. ✅ Tested with unified-demo Counter widget - generates working Kotlin code

**Generated Counter.kt Example:**
```kotlin
@Composable
fun Counter(modifier: Modifier = Modifier) {
    var count by remember { mutableStateOf(0) }

    Column(modifier = Modifier) {
        Text("Counter Demo")
        Text("Count: $count")
        Row(modifier = Modifier, horizontalArrangement = Arrangement.Start) {
            Button { Text("-") }
            Button { Text("+") }
        }
    }
}
```

---

## Background

Currently, the `generate_view_body()` method in `jet/generator.rs` returns a placeholder:

```rust
fn generate_view_body(&mut self, _widget: &AuraWidget) -> GenResult<String> {
    // TODO: Implement full view body generation from widget.view_tree
    Ok("Column(modifier = modifier) {\n        // TODO: Generate view from AURA\n    }".to_string())
}
```

The `vue.rs` implementation has a complete `node_to_html()` method (lines 1319-1586) that handles:
- `AuraNode::Element` - HTML elements with props, events, children
- `AuraNode::Text` - Literal or interpolated text
- `AuraNode::ForLoop` - v-for loops
- `AuraNode::Conditional` - v-if/v-else conditionals
- `AuraNode::Component` - Child component references
- `AuraNode::Outlet` - Router outlet
- `AuraNode::Link` - Navigation links

The JetGenerator already has sub-generators for specific component types:
- `FormGenerator` - input, textarea, checkbox, switch, slider
- `LayoutGenerator` - col, row, box, card, scroll
- `ListGenerator` - list, list-row, grid, flow-row, flow-col
- `NavigationGenerator` - NavHost, routes

## Key Differences: Vue vs Compose

| Vue | Compose |
|-----|---------|
| `<div class="...">` | `Box(modifier = Modifier...)` |
| `<button @click="fn">` | `Button(onClick = { fn() })` |
| `v-for="item in list"` | `items(list) { item -> ... }` |
| `v-if="cond"` | `if (cond) { ... }` |
| `{{ variable }}` | `Text("$variable")` |
| `:class="{ active: cond }"` | `Modifier.then(if (cond) Modifier.background(...) else Modifier)` |

---

## Tasks

### Task 1: Add `node_to_compose()` Method Skeleton

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/jet/generator.rs`

**Step 1: Add the method signature and match skeleton**

Add after line 279 (after `generate_view_body`):

```rust
/// Convert AuraNode to Compose Kotlin code
fn node_to_compose(&mut self, node: &AuraNode, indent: usize) -> GenResult<String> {
    let ind = "    ".repeat(indent);

    match node {
        AuraNode::Element { tag, props, events, children } => {
            self.element_to_compose(tag, props, events, children, indent)
        }
        AuraNode::Text(content) => {
            self.text_to_compose(content, indent)
        }
        AuraNode::ForLoop { var, index, iterable, body } => {
            self.for_loop_to_compose(var, index, iterable, body, indent)
        }
        AuraNode::Conditional { condition, then_body, else_body } => {
            self.conditional_to_compose(condition, then_body, else_body, indent)
        }
        AuraNode::Component { name, props, events } => {
            self.component_to_compose(name, props, events, indent)
        }
        AuraNode::Outlet => {
            // TODO: Navigation placeholder
            Ok(format!("{}// TODO: Router outlet\n", ind))
        }
        AuraNode::Link { to, text, href, children } => {
            self.link_to_compose(to, text, href, children, indent)
        }
    }
}
```

**Step 2: Commit**

```bash
git add crates/auto-lang/src/ui_gen/jet/generator.rs
git commit -m "feat(jet): add node_to_compose method skeleton"
```

---

### Task 2: Implement `text_to_compose()` Helper

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/jet/generator.rs`

**Step 1: Implement text content conversion**

```rust
/// Convert AuraTextContent to Compose Text composable
fn text_to_compose(&self, content: &AuraTextContent, indent: usize) -> GenResult<String> {
    let ind = "    ".repeat(indent);

    match content {
        AuraTextContent::Literal(s) => {
            Ok(format!("{}Text(\"{}\")\n", ind, s))
        }
        AuraTextContent::Interpolated { template, bindings } => {
            // Convert template to Kotlin string interpolation
            let mut kotlin_text = template.clone();
            for binding in bindings {
                // Replace ${.binding} with $binding (state reference)
                kotlin_text = kotlin_text.replace(
                    &format!("${{{}.{}}}", ".", binding),
                    &format!("${}", binding)
                );
                // Replace ${binding} with $binding (variable reference)
                kotlin_text = kotlin_text.replace(
                    &format!("${{{}}}", binding),
                    &format!("${}", binding)
                );
            }
            Ok(format!("{}Text(\"{}\")\n", ind, kotlin_text))
        }
    }
}
```

**Step 2: Add import for AuraTextContent**

Ensure the import is present at the top of the file:
```rust
use crate::aura::{AuraExpr, AuraNode, AuraPropValue, AuraTextContent, AuraWidget};
```

**Step 3: Commit**

```bash
git add crates/auto-lang/src/ui_gen/jet/generator.rs
git commit -m "feat(jet): implement text_to_compose for Text nodes"
```

---

### Task 3: Implement `element_to_compose()` - Layout Components

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/jet/generator.rs`

**Step 1: Implement element conversion with layout delegation**

```rust
/// Convert AuraNode::Element to Compose code
fn element_to_compose(
    &mut self,
    tag: &str,
    props: &HashMap<String, AuraPropValue>,
    events: &HashMap<String, AuraEvent>,
    children: &[AuraNode],
    indent: usize,
) -> GenResult<String> {
    let ind = "    ".repeat(indent);

    // Check if it's a layout element
    if Self::is_layout_tag(tag) {
        return self.layout_element_to_compose(tag, props, events, children, indent);
    }

    // Check if it's a form element
    if Self::is_form_tag(tag) {
        return self.form_element_to_compose(tag, props, events, children, indent);
    }

    // Check if it's a list element
    if Self::is_list_tag(tag) {
        return self.list_element_to_compose(tag, props, events, children, indent);
    }

    // Default: map to Compose component
    self.generic_element_to_compose(tag, props, events, children, indent)
}

/// Check if tag is a layout element
fn is_layout_tag(tag: &str) -> bool {
    matches!(tag, "col" | "column" | "row" | "box" | "container" | "card" | "scroll")
}

/// Check if tag is a form element
fn is_form_tag(tag: &str) -> bool {
    matches!(tag, "input" | "textarea" | "checkbox" | "switch" | "toggle" | "slider" | "button")
}

/// Check if tag is a list element
fn is_list_tag(tag: &str) -> bool {
    matches!(tag, "list" | "lazy-column" | "list-row" | "lazy-row" | "grid" | "lazy-grid" | "flow-row" | "flow-col" | "flow-column")
}
```

**Step 2: Implement layout element conversion**

```rust
/// Convert layout elements to Compose
fn layout_element_to_compose(
    &mut self,
    tag: &str,
    props: &HashMap<String, AuraPropValue>,
    _events: &HashMap<String, AuraEvent>,
    children: &[AuraNode],
    indent: usize,
) -> GenResult<String> {
    let ind = "    ".repeat(indent);

    // Generate children content
    let mut children_content = String::new();
    for child in children {
        children_content.push_str(&self.node_to_compose(child, indent + 1)?);
    }

    // Use LayoutGenerator for the actual generation
    let result = match tag {
        "col" | "column" => self.layout_generator.generate_column(props, &children_content),
        "row" => self.layout_generator.generate_row(props, &children_content),
        "box" | "container" => self.layout_generator.generate_box(props, &children_content),
        "card" => self.layout_generator.generate_card(props, &children_content),
        "scroll" => self.layout_generator.generate_scroll(props, &children_content),
        _ => Err(GenError::UnsupportedExpr(format!("Unknown layout tag: {}", tag))),
    };

    // Prepend proper indentation
    result.map(|s| {
        s.lines()
            .map(|line| format!("{}{}", ind, line))
            .collect::<Vec<_>>()
            .join("\n")
            + "\n"
    })
}
```

**Step 3: Commit**

```bash
git add crates/auto-lang/src/ui_gen/jet/generator.rs
git commit -m "feat(jet): implement layout element conversion"
```

---

### Task 4: Implement `element_to_compose()` - Form Components

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/jet/generator.rs`

**Step 1: Implement form element conversion**

```rust
/// Convert form elements to Compose
fn form_element_to_compose(
    &mut self,
    tag: &str,
    props: &HashMap<String, AuraPropValue>,
    events: &HashMap<String, AuraEvent>,
    children: &[AuraNode],
    indent: usize,
) -> GenResult<String> {
    let ind = "    ".repeat(indent);

    match tag {
        "button" => self.button_to_compose(props, events, children, indent),
        "input" => {
            // Generate input with state binding
            let mut props_with_binding = props.clone();
            self.form_generator.generate_input(&props_with_binding)
                .map(|s| format!("{}{}\n", ind, s.trim()))
        }
        "checkbox" => self.form_generator.generate_checkbox(props)
                .map(|s| format!("{}{}\n", ind, s.trim())),
        "switch" | "toggle" => self.form_generator.generate_switch(props)
                .map(|s| format!("{}{}\n", ind, s.trim())),
        "slider" => self.form_generator.generate_slider(props)
                .map(|s| format!("{}{}\n", ind, s.trim())),
        _ => Err(GenError::UnsupportedExpr(format!("Unknown form tag: {}", tag))),
    }
}

/// Convert button to Compose Button
fn button_to_compose(
    &mut self,
    props: &HashMap<String, AuraPropValue>,
    events: &HashMap<String, AuraEvent>,
    children: &[AuraNode],
    indent: usize,
) -> GenResult<String> {
    let ind = "    ".repeat(indent);

    // Get onClick handler
    let on_click = events.get("click")
        .map(|e| self.event_to_lambda(e))
        .unwrap_or_default();

    // Get button text
    let text = props.get("text")
        .and_then(|p| self.extract_string_value(p))
        .unwrap_or_default();

    // Generate children content if any
    let mut content = if !text.is_empty() {
        format!("{}    Text(\"{}\")\n", ind, text)
    } else if !children.is_empty() {
        let mut s = String::new();
        for child in children {
            s.push_str(&self.node_to_compose(child, indent + 1)?);
        }
        s
    } else {
        format!("{}    Text(\"Button\")\n", ind)
    };

    Ok(format!(
        "{}Button(\n{}    onClick = {{ {} }}\n{}) {{\n{}}}\n",
        ind, ind, on_click, ind, content
    ))
}

/// Convert AuraEvent to Kotlin lambda
fn event_to_lambda(&self, event: &AuraEvent) -> String {
    let handler = &event.handler;
    let params = &event.params;

    if params.is_empty() {
        format!("{}()", handler.trim_start_matches('.'))
    } else {
        format!("{}({})", handler.trim_start_matches('.'), params.join(", "))
    }
}
```

**Step 2: Add helper for extracting string values**

```rust
/// Extract string value from AuraPropValue
fn extract_string_value(&self, value: &AuraPropValue) -> Option<String> {
    match value {
        AuraPropValue::Expr(AuraExpr::Literal(s)) => Some(s.clone()),
        AuraPropValue::Expr(AuraExpr::StateRef(s)) => Some(s.clone()),
        _ => None,
    }
}
```

**Step 3: Commit**

```bash
git add crates/auto-lang/src/ui_gen/jet/generator.rs
git commit -m "feat(jet): implement form element conversion with button support"
```

---

### Task 5: Implement `for_loop_to_compose()` and `conditional_to_compose()`

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/jet/generator.rs`

**Step 1: Implement for loop conversion**

```rust
/// Convert for loop to Compose items() or forEach()
fn for_loop_to_compose(
    &mut self,
    var: &str,
    index: &Option<String>,
    iterable: &str,
    body: &[AuraNode],
    indent: usize,
) -> GenResult<String> {
    let ind = "    ".repeat(indent);

    // Generate body content
    let mut body_content = String::new();
    for child in body {
        body_content.push_str(&self.node_to_compose(child, indent + 2)?);
    }

    // Clean iterable name (remove leading ".")
    let iterable_clean = iterable.trim_start_matches('.');

    if let Some(idx) = index {
        // With index: items(items.withIndex().toList()) { (index, item) -> ... }
        // Or simpler: itemsIndexed(items) { index, item -> ... }
        Ok(format!(
            "{}itemsIndexed({}) {{ {}, {} ->\n{}}}\n",
            ind, iterable_clean, idx, var, body_content
        ))
    } else {
        // Without index: items(items) { item -> ... }
        Ok(format!(
            "{}items({}) {{ {} ->\n{}}}\n",
            ind, iterable_clean, var, body_content
        ))
    }
}
```

**Step 2: Implement conditional conversion**

```rust
/// Convert conditional to Kotlin if/else
fn conditional_to_compose(
    &mut self,
    condition: &AuraExpr,
    then_body: &[AuraNode],
    else_body: &Option<Vec<AuraNode>>,
    indent: usize,
) -> GenResult<String> {
    let ind = "    ".repeat(indent);

    // Convert condition to Kotlin expression
    let cond_kotlin = self.expr_to_kotlin(condition);

    // Generate then body
    let mut then_content = String::new();
    for child in then_body {
        then_content.push_str(&self.node_to_compose(child, indent + 1)?);
    }

    if let Some(else_nodes) = else_body {
        let mut else_content = String::new();
        for child in else_nodes {
            else_content.push_str(&self.node_to_compose(child, indent + 1)?);
        }
        Ok(format!(
            "{}if ({}) {{\n{}}} else {{\n{}}}\n",
            ind, cond_kotlin, then_content, else_content
        ))
    } else {
        Ok(format!(
            "{}if ({}) {{\n{}}}\n",
            ind, cond_kotlin, then_content
        ))
    }
}

/// Convert AuraExpr to Kotlin expression string
fn expr_to_kotlin(&self, expr: &AuraExpr) -> String {
    match expr {
        AuraExpr::Literal(s) => format!("\"{}\"", s),
        AuraExpr::Int(n) => n.to_string(),
        AuraExpr::Float(f) => f.to_string(),
        AuraExpr::Bool(b) => b.to_string(),
        AuraExpr::StateRef(s) => s.clone(),
        AuraExpr::Binary { left, op, right } => {
            let left_str = self.expr_to_kotlin(left);
            let right_str = self.expr_to_kotlin(right);
            let op_str = self.binop_to_kotlin(*op);
            format!("{} {} {}", left_str, op_str, right_str)
        }
        AuraExpr::Unary { op, operand } => {
            let operand_str = self.expr_to_kotlin(operand);
            match op {
                AuraUnaryOp::Neg => format!("-{}", operand_str),
                AuraUnaryOp::Not => format!("!{}", operand_str),
            }
        }
        AuraExpr::FieldAccess { object, field } => {
            let obj_str = self.expr_to_kotlin(object);
            format!("{}.{}", obj_str, field)
        }
        AuraExpr::MethodCall { object, method, args } => {
            let obj_str = self.expr_to_kotlin(object);
            let args_str = args.iter()
                .map(|a| self.expr_to_kotlin(a))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}.{}({})", obj_str, method, args_str)
        }
        _ => "/* unsupported expr */".to_string(),
    }
}

/// Convert binary operator to Kotlin
fn binop_to_kotlin(&self, op: AuraBinOp) -> &'static str {
    match op {
        AuraBinOp::Add => "+",
        AuraBinOp::Sub => "-",
        AuraBinOp::Mul => "*",
        AuraBinOp::Div => "/",
        AuraBinOp::Mod => "%",
        AuraBinOp::Eq => "==",
        AuraBinOp::Ne => "!=",
        AuraBinOp::Lt => "<",
        AuraBinOp::Le => "<=",
        AuraBinOp::Gt => ">",
        AuraBinOp::Ge => ">=",
        AuraBinOp::And => "&&",
        AuraBinOp::Or => "||",
    }
}
```

**Step 3: Add imports for AuraExpr variants**

```rust
use crate::aura::{AuraBinOp, AuraExpr, AuraNode, AuraPropValue, AuraTextContent, AuraUnaryOp, AuraWidget};
```

**Step 4: Commit**

```bash
git add crates/auto-lang/src/ui_gen/jet/generator.rs
git commit -m "feat(jet): implement for loop and conditional conversion"
```

---

### Task 6: Implement `component_to_compose()` and `link_to_compose()`

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/jet/generator.rs`

**Step 1: Implement component reference conversion**

```rust
/// Convert child component reference to Compose call
fn component_to_compose(
    &mut self,
    name: &str,
    props: &HashMap<String, AuraExpr>,
    events: &HashMap<String, AuraEvent>,
    indent: usize,
) -> GenResult<String> {
    let ind = "    ".repeat(indent);

    // Track component reference for imports
    self.component_refs.push(name.to_string());

    // Build props string
    let mut props_parts = Vec::new();
    for (key, value) in props {
        let value_str = self.expr_to_kotlin(value);
        props_parts.push(format!("{} = {}", key, value_str));
    }

    // Build event handlers
    for (event, aura_event) in events {
        let handler = self.event_to_lambda(aura_event);
        // Map event names to Compose convention
        let compose_event = match event {
            "click" => "onClick".to_string(),
            _ => format!("on{}", event.chars().next().unwrap().to_uppercase().collect::<String>() + &event[1..]),
        };
        props_parts.push(format!("{} = {{ {} }}", compose_event, handler));
    }

    let props_str = if props_parts.is_empty() {
        String::new()
    } else {
        format!("\n{}    {}", ind, props_parts.join(&format!(",\n{}    ", ind)))
    };

    Ok(format!("{}{}({})\n", ind, name, props_str))
}
```

**Step 2: Implement link conversion**

```rust
/// Convert link to Compose navigation
fn link_to_compose(
    &mut self,
    to: &str,
    text: &str,
    href: &str,
    children: &[AuraNode],
    indent: usize,
) -> GenResult<String> {
    let ind = "    ".repeat(indent);

    if !href.is_empty() {
        // External link - use Text with clickable modifier
        let text_content = if text.is_empty() {
            // Get text from children
            let mut s = String::new();
            for child in children {
                if let AuraNode::Text(content) = child {
                    if let AuraTextContent::Literal(t) = content {
                        s.push_str(t);
                    }
                }
            }
            s
        } else {
            text.to_string()
        };

        Ok(format!(
            "{}Text(\n{}    \"{}\",\n{}    modifier = Modifier.clickable {{ /* open {} */ }}\n{})\n",
            ind, ind, text_content, ind, href, ind
        ))
    } else {
        // Internal navigation - use navigation generator
        self.navigation_generator.add_route(to, to);
        Ok(format!(
            "{}Button(onClick = {{ /* navigate to {} */ }}) {{\n{}    Text(\"{}\")\n{}}}\n",
            ind, to, ind, text, ind
        ))
    }
}
```

**Step 3: Commit**

```bash
git add crates/auto-lang/src/ui_gen/jet/generator.rs
git commit -m "feat(jet): implement component and link conversion"
```

---

### Task 7: Implement `generic_element_to_compose()` for Common Elements

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/jet/generator.rs`

**Step 1: Implement generic element handler**

```rust
/// Convert generic element to Compose
fn generic_element_to_compose(
    &mut self,
    tag: &str,
    props: &HashMap<String, AuraPropValue>,
    events: &HashMap<String, AuraEvent>,
    children: &[AuraNode],
    indent: usize,
) -> GenResult<String> {
    let ind = "    ".repeat(indent);

    // Map common HTML-like tags to Compose
    let (compose_name, is_self_closing) = self.map_tag_to_compose(tag);

    // Build modifier from props
    let modifier = self.build_modifier(props);

    // Build event handlers
    let mut handlers = Vec::new();
    for (event, aura_event) in events {
        let handler = self.event_to_lambda(aura_event);
        let compose_event = self.map_event_to_compose(event);
        handlers.push(format!("{} = {{ {} }}", compose_event, handler));
    }

    // Generate children content
    let mut children_content = String::new();
    for child in children {
        children_content.push_str(&self.node_to_compose(child, indent + 1)?);
    }

    // Check for text prop
    let text_prop = props.get("text")
        .and_then(|p| self.extract_string_value(p));

    if is_self_closing {
        if !modifier.is_empty() || !handlers.is_empty() {
            let all_props = [&handlers[..], &[format!("modifier = {}", modifier)]].concat();
            Ok(format!("{}{}({})\n", ind, compose_name, all_props.join(", ")))
        } else {
            Ok(format!("{}{}()\n", ind, compose_name))
        }
    } else if let Some(text) = text_prop {
        if children.is_empty() {
            Ok(format!("{}{}(\"{}\")\n", ind, compose_name, text))
        } else {
            Ok(format!("{}{} {{\n{}    Text(\"{}\")\n{}}}\n", ind, compose_name, ind, text, ind))
        }
    } else {
        Ok(format!("{}{} {{\n{}}}\n", ind, compose_name, children_content))
    }
}

/// Map AURA tag to Compose component name
fn map_tag_to_compose(&self, tag: &str) -> (&'static str, bool) {
    match tag {
        "text" | "span" | "p" => ("Text", true),
        "div" | "section" | "article" => ("Box", false),
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => ("Text", true),
        "img" | "image" => ("Image", true),
        "icon" => ("Icon", true),
        "spacer" => ("Spacer", true),
        "divider" => ("HorizontalDivider", true),
        _ => ("Box", false),  // Default to Box container
    }
}

/// Map event name to Compose parameter name
fn map_event_to_compose(&self, event: &str) -> String {
    match event {
        "click" => "onClick".to_string(),
        "change" => "onValueChange".to_string(),
        "focus" => "onFocusChanged".to_string(),
        _ => format!("on{}", event.chars().next().unwrap().to_uppercase().collect::<String>() + &event[1..]),
    }
}

/// Build Modifier from props (Tailwind-like classes)
fn build_modifier(&self, props: &HashMap<String, AuraPropValue>) -> String {
    let mut modifiers = Vec::new();

    // Handle class prop (Tailwind-style)
    if let Some(value) = props.get("class") {
        if let Some(class_str) = self.extract_string_value(value) {
            // Use ModifierDsl to convert Tailwind to Compose
            let compose_mods = self.modifier_dsl.parse(&class_str);
            modifiers.extend(compose_mods);
        }
    }

    // Handle gap
    if let Some(value) = props.get("gap") {
        if let Some(AuraExpr::Int(n)) = self.extract_expr(value) {
            modifiers.push(format!(".padding({}.dp)", n));
        }
    }

    if modifiers.is_empty() {
        String::new()
    } else {
        format!("Modifier{}", modifiers.join(""))
    }
}

/// Extract expression from AuraPropValue
fn extract_expr(&self, value: &AuraPropValue) -> Option<&AuraExpr> {
    match value {
        AuraPropValue::Expr(expr) => Some(expr),
        _ => None,
    }
}
```

**Step 2: Add ModifierDsl field to JetGenerator struct**

Find the JetGenerator struct definition and add:
```rust
/// Modifier DSL converter
modifier_dsl: ModifierDsl,
```

And initialize it in `new()`:
```rust
modifier_dsl: ModifierDsl::new(),
```

**Step 3: Commit**

```bash
git add crates/auto-lang/src/ui_gen/jet/generator.rs
git commit -m "feat(jet): implement generic element and modifier conversion"
```

---

### Task 8: Implement `generate_view_body()` to Use `node_to_compose()`

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/jet/generator.rs`

**Step 1: Update generate_view_body to traverse view_tree**

Replace the placeholder implementation:

```rust
/// Generate view body from widget's view_tree
fn generate_view_body(&mut self, widget: &AuraWidget) -> GenResult<String> {
    let mut body = String::new();

    // Process each node in the view tree
    for node in &widget.view_tree {
        body.push_str(&self.node_to_compose(node, 1)?);
    }

    // If empty, provide a default Column
    if body.is_empty() {
        body = "    Column(modifier = modifier) {\n        // Empty view\n    }\n".to_string();
    }

    Ok(body)
}
```

**Step 2: Run tests**

```bash
cargo test -p auto-lang jet
```

**Step 3: Commit**

```bash
git add crates/auto-lang/src/ui_gen/jet/generator.rs
git commit -m "feat(jet): connect generate_view_body to node_to_compose"
```

---

### Task 9: Add Imports and Finalize

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/jet/generator.rs`

**Step 1: Ensure all imports are present**

```rust
use crate::aura::{AuraBinOp, AuraEvent, AuraExpr, AuraNode, AuraPropValue, AuraTextContent, AuraUnaryOp, AuraWidget};
```

**Step 2: Add component_refs field if not present**

Check if `component_refs` exists in JetGenerator struct. If not, add:
```rust
/// Referenced child components
component_refs: Vec<String>,
```

And initialize in `new()`:
```rust
component_refs: Vec::new(),
```

**Step 3: Run full test suite**

```bash
cargo test -p auto-lang jet
cargo test -p auto-lang -- trans
```

**Step 4: Commit**

```bash
git add crates/auto-lang/src/ui_gen/jet/generator.rs
git commit -m "feat(jet): add imports and finalize view body generation"
```

---

### Task 10: Test with unified-demo

**Files:**
- Test: `examples/unified-demo/jet/`

**Step 1: Run auto gen on unified-demo**

```bash
cd examples/unified-demo
cargo run -- gen --backend jet
```

**Step 2: Verify Counter.kt has generated content**

Check that `app/src/main/java/com/example/unified_demo/ui/widgets/Counter.kt` contains actual UI code instead of `// TODO: Generate view from AURA`.

**Step 3: Run build**

```bash
cargo run -- build --backend jet
```

**Step 4: Commit**

```bash
git add examples/unified-demo/jet/
git commit -m "test(jet): verify view body generation with unified-demo"
```

---

## Success Criteria

1. `generate_view_body()` produces valid Kotlin/Compose code from AURA view_tree
2. All AuraNode variants are handled (Element, Text, ForLoop, Conditional, Component, Outlet, Link)
3. Layout elements (col, row, box, card) delegate to LayoutGenerator
4. Form elements (input, button, checkbox, switch, slider) delegate to FormGenerator
5. List elements (list, grid, flow-row) delegate to ListGenerator
6. State references (`.count`) convert to Kotlin variables (`count`)
7. Event handlers convert to Kotlin lambdas (`{ increment() }`)
8. Tests pass: `cargo test -p auto-lang jet`
9. unified-demo Counter.kt shows actual generated content

## Related Plans

- Plan 113: a2jet Design (initial architecture)
- Plan 114: Form Components
- Plan 115: Layout & Navigation
- Plan 116: Lists & Data
- Plan 117: Project Generation
- Plan 118: Documentation & Tests
