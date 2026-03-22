# AURA Widget Library Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans or superpowers:subagent-driven-development to implement this plan task-by-task.

**Goal:** Replace hardcoded component registries with a widget library loaded from `.at` files in `stdlib/aura/widgets/`.

**Architecture:** WidgetSpec types in Rust load widget definitions from `.at` files. WidgetRegistry provides lookup by tag name. Generators use WidgetRegistry instead of hardcoded component maps.

**Tech Stack:** Rust, AutoLang widget syntax, annotation parsing, WidgetRegistry

---

## Phase 1: Core Infrastructure (WidgetSpec Types)

### Task 1: Define WidgetSpec Types

**Files:**
- Create: `crates/auto-lang/src/ui_gen/widget/spec.rs`
- Create: `crates/auto-lang/src/ui_gen/widget/mod.rs`

**Step 1: Write the failing test**

```rust
// In crates/auto-lang/src/ui_gen/widget/spec.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_widget_spec_creation() {
        let spec = WidgetSpec {
            name: "Button".to_string(),
            category: WidgetCategory::Form,
            primary_prop: Some("text".to_string()),
            has_children: false,
            backends: HashMap::new(),
        };
        assert_eq!(spec.name, "Button");
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p auto-lang widget::spec::tests::test_widget_spec_creation`
Expected: FAIL with "cannot find type WidgetSpec"

**Step 3: Write minimal implementation**

```rust
// In crates/auto-lang/src/ui_gen/widget/spec.rs

use std::collections::HashMap;

/// Widget category for organization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetCategory {
    Layout,
    Form,
    Display,
    Navigation,
    Semantic,
}

/// Backend-specific component mapping
#[derive(Debug, Clone)]
pub struct BackendMapping {
    /// Component/composable name in target backend
    pub component: String,
    /// Import statement (if required)
    pub import: Option<String>,
    /// Property mappings: AURA prop -> backend prop
    pub props: HashMap<String, String>,
    /// Event mappings: AURA event -> backend event
    pub events: HashMap<String, String>,
}

/// Widget specification loaded from .at files
#[derive(Debug, Clone)]
pub struct WidgetSpec {
    /// Widget name (e.g., "Button", "Text")
    pub name: String,
    /// Widget category
    pub category: WidgetCategory,
    /// Primary prop for shorthand syntax
    pub primary_prop: Option<String>,
    /// Whether widget supports children
    pub has_children: bool,
    /// Backend-specific mappings
    pub backends: HashMap<String, BackendMapping>,
}

impl WidgetSpec {
    /// Create a new widget spec
    pub fn new(name: &str, category: WidgetCategory) -> Self {
        Self {
            name: name.to_string(),
            category,
            primary_prop: None,
            has_children: false,
            backends: HashMap::new(),
        }
    }

    /// Get backend mapping
    pub fn backend(&self, backend: &str) -> Option<&BackendMapping> {
        self.backends.get(backend)
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p auto-lang widget::spec::tests::test_widget_spec_creation`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/auto-lang/src/ui_gen/widget/
git commit -m "feat(widget): add WidgetSpec types for widget library"
```

---

### Task 2: Create Widget Module Structure

**Files:**
- Create: `crates/auto-lang/src/ui_gen/widget/mod.rs`
- Create: `crates/auto-lang/src/ui_gen/widget/registry.rs`
- Modify: `crates/auto-lang/src/ui_gen/mod.rs`

**Step 1: Write the failing test**

```rust
// In crates/auto-lang/src/ui_gen/widget/registry.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = WidgetRegistry::new();
        assert!(registry.get("button").is_none()); // Empty registry
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p auto-lang widget::registry::tests::test_registry_creation`
Expected: FAIL with "cannot find type WidgetRegistry"

**Step 3: Write minimal implementation**

```rust
// In crates/auto-lang/src/ui_gen/widget/mod.rs
pub mod registry;
pub mod spec;

pub use registry::WidgetRegistry;
pub use spec::{BackendMapping, WidgetCategory, WidgetSpec};

// In crates/auto-lang/src/ui_gen/widget/registry.rs
use super::spec::WidgetSpec;
use std::collections::HashMap;

/// Widget registry for looking up widget specifications
pub struct WidgetRegistry {
    widgets: HashMap<String, WidgetSpec>,
}

impl WidgetRegistry {
    /// Create an empty registry
    pub fn new() -> Self {
        Self {
            widgets: HashMap::new(),
        }
    }

    /// Register a widget
    pub fn register(&mut self, spec: WidgetSpec) {
        self.widgets.insert(spec.name.to_lowercase(), spec);
    }

    /// Look up a widget by tag name (case-insensitive)
    pub fn get(&self, tag: &str) -> Option<&WidgetSpec> {
        self.widgets.get(&tag.to_lowercase())
    }

    /// Check if a widget exists
    pub fn contains(&self, tag: &str) -> bool {
        self.widgets.contains_key(&tag.to_lowercase())
    }
}

impl Default for WidgetRegistry {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 4: Update ui_gen/mod.rs**

```rust
// Add to crates/auto-lang/src/ui_gen/mod.rs
pub mod widget;

// Add re-exports
pub use widget::{WidgetCategory, WidgetRegistry, WidgetSpec};
```

**Step 5: Run test to verify it passes**

Run: `cargo test -p auto-lang widget::registry::tests::test_registry_creation`
Expected: PASS

**Step 6: Commit**

```bash
git add crates/auto-lang/src/ui_gen/widget/
git commit -m "feat(widget): add WidgetRegistry for widget lookup"
```

---

## Phase 2: Default Widget Registration

### Task 3: Register Core Layout Widgets

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/widget/registry.rs`

**Step 1: Write the failing test**

```rust
// In crates/auto-lang/src/ui_gen/widget/registry.rs

#[cfg(test)]
mod tests {
    // ... existing tests ...

    #[test]
    fn test_default_widgets_col() {
        let registry = WidgetRegistry::with_defaults();
        let col = registry.get("col").unwrap();
        assert_eq!(col.name, "Column");
        assert_eq!(col.category, WidgetCategory::Layout);
        assert!(col.has_children);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p auto-lang widget::registry::tests::test_default_widgets_col`
Expected: FAIL with "widget not found" or "called `Option::unwrap()` on a `None` value"

**Step 3: Write implementation**

```rust
// In crates/auto-lang/src/ui_gen/widget/registry.rs

use super::spec::{BackendMapping, WidgetCategory, WidgetSpec};
use std::collections::HashMap;

impl WidgetRegistry {
    /// Create registry with default widgets
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register_defaults();
        registry
    }

    /// Register default widget specifications
    fn register_defaults(&mut self) {
        self.register_layout_widgets();
        self.register_form_widgets();
        self.register_display_widgets();
        self.register_navigation_widgets();
        self.register_semantic_widgets();
    }

    fn register_layout_widgets(&mut self) {
        // Column
        let mut col = WidgetSpec::new("Column", WidgetCategory::Layout);
        col.has_children = true;
        col.backends.insert("ark".to_string(), BackendMapping {
            component: "Column".to_string(),
            import: None, // Built-in
            props: HashMap::new(),
            events: HashMap::new(),
        });
        col.backends.insert("jet".to_string(), BackendMapping {
            component: "Column".to_string(),
            import: Some("androidx.compose.foundation.layout.Column".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
        });
        col.backends.insert("vue".to_string(), BackendMapping {
            component: "div".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
        });
        self.register(col);

        // Row
        let mut row = WidgetSpec::new("Row", WidgetCategory::Layout);
        row.has_children = true;
        row.backends.insert("ark".to_string(), BackendMapping {
            component: "Row".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
        });
        row.backends.insert("jet".to_string(), BackendMapping {
            component: "Row".to_string(),
            import: Some("androidx.compose.foundation.layout.Row".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
        });
        row.backends.insert("vue".to_string(), BackendMapping {
            component: "div".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
        });
        self.register(row);

        // Stack
        let mut stack = WidgetSpec::new("Stack", WidgetCategory::Layout);
        stack.has_children = true;
        stack.backends.insert("ark".to_string(), BackendMapping {
            component: "Stack".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
        });
        stack.backends.insert("jet".to_string(), BackendMapping {
            component: "Box".to_string(),
            import: Some("androidx.compose.foundation.layout.Box".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
        });
        stack.backends.insert("vue".to_string(), BackendMapping {
            component: "div".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
        });
        self.register(stack);

        // Scroll
        let mut scroll = WidgetSpec::new("Scroll", WidgetCategory::Layout);
        scroll.has_children = true;
        scroll.backends.insert("ark".to_string(), BackendMapping {
            component: "Scroll".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
        });
        self.register(scroll);
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p auto-lang widget::registry::tests::test_default_widgets_col`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/auto-lang/src/ui_gen/widget/registry.rs
git commit -m "feat(widget): register core layout widgets (Column, Row, Stack, Scroll)"
```

---

### Task 4: Register Form Widgets

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/widget/registry.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_default_widgets_button() {
    let registry = WidgetRegistry::with_defaults();
    let button = registry.get("button").unwrap();
    assert_eq!(button.name, "Button");
    assert_eq!(button.category, WidgetCategory::Form);

    let ark_mapping = button.backend("ark").unwrap();
    assert_eq!(ark_mapping.component, "Button");
    assert_eq!(ark_mapping.import, Some("@kit.ArkUI".to_string()));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p auto-lang widget::registry::tests::test_default_widgets_button`
Expected: FAIL

**Step 3: Write implementation**

```rust
// In register_form_widgets function
fn register_form_widgets(&mut self) {
    // Button
    let mut button = WidgetSpec::new("Button", WidgetCategory::Form);
    button.primary_prop = Some("text".to_string());
    button.backends.insert("ark".to_string(), BackendMapping {
        component: "Button".to_string(),
        import: Some("@kit.ArkUI".to_string()),
        props: HashMap::new(),
        events: HashMap::new(),
    });
    button.backends.insert("jet".to_string(), BackendMapping {
        component: "Button".to_string(),
        import: Some("androidx.compose.material3.Button".to_string()),
        props: HashMap::new(),
        events: HashMap::new(),
    });
    button.backends.insert("vue".to_string(), BackendMapping {
        component: "Button".to_string(),
        import: Some("@/components/ui/button/Button".to_string()),
        props: HashMap::new(),
        events: HashMap::new(),
    });
    self.register(button);

    // Input (TextInput in Ark)
    let mut input = WidgetSpec::new("Input", WidgetCategory::Form);
    input.backends.insert("ark".to_string(), BackendMapping {
        component: "TextInput".to_string(),
        import: None,
        props: HashMap::new(),
        events: HashMap::new(),
    });
    input.backends.insert("jet".to_string(), BackendMapping {
        component: "OutlinedTextField".to_string(),
        import: Some("androidx.compose.material3.OutlinedTextField".to_string()),
        props: HashMap::new(),
        events: HashMap::new(),
    });
    self.register(input);
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p auto-lang widget::registry::tests::test_default_widgets_button`
Expected: PASS

**Step 5: Commit**

```bash
git commit -am "feat(widget): register form widgets (Button, Input)"
```

---

### Task 5: Register Display and Navigation Widgets

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/widget/registry.rs`

**Step 1: Write failing tests**

```rust
#[test]
fn test_default_widgets_text() {
    let registry = WidgetRegistry::with_defaults();
    let text = registry.get("text").unwrap();
    assert_eq!(text.name, "Text");
    assert_eq!(text.category, WidgetCategory::Display);
}

#[test]
fn test_default_widgets_image() {
    let registry = WidgetRegistry::with_defaults();
    let image = registry.get("image").unwrap();
    assert_eq!(image.name, "Image");

    let ark_mapping = image.backend("ark").unwrap();
    assert_eq!(ark_mapping.component, "Image");
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p auto-lang widget::registry::tests::test_default_widgets_text`
Expected: FAIL

**Step 3: Write implementation**

```rust
fn register_display_widgets(&mut self) {
    // Text
    let mut text = WidgetSpec::new("Text", WidgetCategory::Display);
    text.primary_prop = Some("text".to_string());
    text.backends.insert("ark".to_string(), BackendMapping {
        component: "Text".to_string(),
        import: None,
        props: HashMap::new(),
        events: HashMap::new(),
    });
    text.backends.insert("jet".to_string(), BackendMapping {
        component: "Text".to_string(),
        import: Some("androidx.compose.material3.Text".to_string()),
        props: HashMap::new(),
        events: HashMap::new(),
    });
    text.backends.insert("vue".to_string(), BackendMapping {
        component: "span".to_string(),
        import: None,
        props: HashMap::new(),
        events: HashMap::new(),
    });
    self.register(text);

    // Image
    let mut image = WidgetSpec::new("Image", WidgetCategory::Display);
    image.backends.insert("ark".to_string(), BackendMapping {
        component: "Image".to_string(),
        import: None,
        props: HashMap::new(),
        events: HashMap::new(),
    });
    image.backends.insert("jet".to_string(), BackendMapping {
        component: "Image".to_string(),
        import: Some("androidx.compose.foundation.Image".to_string()),
        props: HashMap::new(),
        events: HashMap::new(),
    });
    image.backends.insert("vue".to_string(), BackendMapping {
        component: "img".to_string(),
        import: None,
        props: HashMap::new(),
        events: HashMap::new(),
    });
    self.register(image);

    // Icon, Progress, Divider, etc.
}

fn register_navigation_widgets(&mut self) {
    // Swiper
    let mut swiper = WidgetSpec::new("Swiper", WidgetCategory::Navigation);
    swiper.has_children = true;
    swiper.backends.insert("ark".to_string(), BackendMapping {
        component: "Swiper".to_string(),
        import: None,
        props: HashMap::new(),
        events: HashMap::new(),
    });
    self.register(swiper);
}

fn register_semantic_widgets(&mut self) {
    // Semantic HTML elements map to Column in Ark
    for tag in ["header", "footer", "nav", "main", "aside", "article", "section"] {
        let mut widget = WidgetSpec::new(tag, WidgetCategory::Semantic);
        widget.has_children = true;
        widget.backends.insert("ark".to_string(), BackendMapping {
            component: "Column".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
        });
        self.register(widget);
    }

    // Heading elements map to Text
    for tag in ["h1", "h2", "h3", "h4", "h5", "h6"] {
        let mut widget = WidgetSpec::new(tag, WidgetCategory::Display);
        widget.primary_prop = Some("text".to_string());
        widget.backends.insert("ark".to_string(), BackendMapping {
            component: "Text".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
        });
        self.register(widget);
    }
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p auto-lang widget::registry::tests::test_default_widgets`
Expected: All PASS

**Step 5: Commit**

```bash
git commit -am "feat(widget): register display, navigation, and semantic widgets"
```

---

## Phase 3: Generator Integration (Ark)

### Task 6: Integrate WidgetRegistry into ArkGenerator

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/ark/generator.rs`
- Modify: `crates/auto-lang/src/ui_gen/ark/components.rs` (or remove)

**Step 1: Write failing test**

```rust
// In crates/auto-lang/src/ui_gen/ark/generator.rs tests

#[test]
fn test_generator_uses_widget_registry() {
    let mut gen = ArkGenerator::new();
    // Should be able to get widget registry
    let widget = gen.registry.get("col");
    assert!(widget.is_some());
    assert_eq!(widget.unwrap().name, "Column");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p auto-lang ark::generator::tests::test_generator_uses_widget_registry`
Expected: FAIL (registry is private or doesn't have WidgetRegistry type)

**Step 3: Write implementation**

```rust
// In crates/auto-lang/src/ui_gen/ark/generator.rs

use crate::ui_gen::widget::WidgetRegistry;

pub struct ArkGenerator {
    // ... existing fields ...

    /// Widget registry (replaces ArkComponentRegistry)
    registry: WidgetRegistry,
}

impl ArkGenerator {
    pub fn new() -> Self {
        Self {
            // ... existing initialization ...
            registry: WidgetRegistry::with_defaults(),
        }
    }
}
```

**Step 4: Update generate_element to use WidgetRegistry**

```rust
fn generate_element(
    &mut self,
    tag: &str,
    props: &HashMap<String, AuraPropValue>,
    events: &HashMap<String, crate::aura::AuraEvent>,
    children: &[AuraNode],
) -> GenResult<String> {
    let mut lines = Vec::new();

    // Look up widget in registry
    if let Some(widget) = self.registry.get(tag) {
        let ark_mapping = widget.backend("ark").unwrap();
        let component_name = &ark_mapping.component;

        // ... rest of generation logic using widget spec ...
    } else {
        lines.push(format!("/* Unknown component: {} */", tag));
    }

    Ok(lines.join("\n"))
}
```

**Step 5: Run test to verify it passes**

Run: `cargo test -p auto-lang ark::generator::tests::test_generator_uses_widget_registry`
Expected: PASS

**Step 6: Run all Ark tests**

Run: `cargo test -p auto-lang ark`
Expected: All PASS

**Step 7: Commit**

```bash
git commit -am "feat(ark): integrate WidgetRegistry into ArkGenerator"
```

---

### Task 7: Remove Old ArkComponentRegistry

**Files:**
- Delete: `crates/auto-lang/src/ui_gen/ark/components.rs`
- Modify: `crates/auto-lang/src/ui_gen/ark/mod.rs`

**Step 1: Verify no references to old registry**

Run: `grep -r "ArkComponentRegistry" crates/auto-lang/src/ui_gen/ark/`
Expected: No matches (or only in comments)

**Step 2: Remove old file**

```bash
rm crates/auto-lang/src/ui_gen/ark/components.rs
```

**Step 3: Update mod.rs**

```rust
// Remove: pub mod components;
// Remove: use components::ArkComponentRegistry;
```

**Step 4: Run all tests**

Run: `cargo test -p auto-lang ark`
Expected: All PASS

**Step 5: Commit**

```bash
git add -A
git commit -m "refactor(ark): remove old ArkComponentRegistry"
```

---

## Phase 4: Widget Files in stdlib

### Task 8: Create stdlib/aura/widgets Directory Structure

**Files:**
- Create: `stdlib/aura/widgets/mod.at`
- Create: `stdlib/aura/widgets/layout/mod.at`
- Create: `stdlib/aura/widgets/layout/col.at`
- Create: `stdlib/aura/widgets/layout/row.at`

**Step 1: Create directory structure**

```bash
mkdir -p stdlib/aura/widgets/layout
mkdir -p stdlib/aura/widgets/form
mkdir -p stdlib/aura/widgets/display
mkdir -p stdlib/aura/widgets/navigation
mkdir -p stdlib/aura/widgets/semantic
```

**Step 2: Create col.at widget file**

```auto
// stdlib/aura/widgets/layout/col.at

#[spec(category = Layout, has_children = true)]
#[backend(ark, component = "Column")]
#[backend(jet, component = "Column", import = "androidx.compose.foundation.layout.Column")]
#[backend(vue, component = "div")]

/// Column layout component.
///
/// Arranges children vertically.
widget Column {
    model {
        gap int = 0
    }

    view {
        // Base element - no actual rendering, just spec
    }
}
```

**Step 3: Create row.at widget file**

```auto
// stdlib/aura/widgets/layout/row.at

#[spec(category = Layout, has_children = true)]
#[backend(ark, component = "Row")]
#[backend(jet, component = "Row", import = "androidx.compose.foundation.layout.Row")]
#[backend(vue, component = "div")]

/// Row layout component.
///
/// Arranges children horizontally.
widget Row {
    model {
        gap int = 0
    }

    view {}
}
```

**Step 4: Create layout/mod.at**

```auto
// stdlib/aura/widgets/layout/mod.at

pub use Column, Row, Stack, Scroll, Grid
```

**Step 5: Create main mod.at**

```auto
// stdlib/aura/widgets/mod.at

// Re-export all widgets for short imports
pub use layout: Column, Row, Stack, Scroll, Grid
pub use form: Button, Input, Switch, Checkbox, Slider
pub use display: Text, Image, Icon, Progress, Divider
pub use navigation: Swiper, Tab
pub use semantic: Header, Footer, Main
```

**Step 6: Commit**

```bash
git add stdlib/aura/widgets/
git commit -m "feat(widget): add widget .at files in stdlib/aura/widgets/"
```

---

### Task 9: Create Form Widget Files

**Files:**
- Create: `stdlib/aura/widgets/form/button.at`
- Create: `stdlib/aura/widgets/form/input.at`
- Create: `stdlib/aura/widgets/form/mod.at`

**Step 1: Create button.at**

```auto
// stdlib/aura/widgets/form/button.at

#[spec(category = Form, primary_prop = "text")]
#[backend(ark, component = "Button", import = "@kit.ArkUI")]
#[backend(jet, component = "Button", import = "androidx.compose.material3.Button")]
#[backend(vue, component = "Button", import = "@/components/ui/button/Button")]

/// Button component for user interactions.
///
/// # Props
/// - text: Button label
/// - variant: "default" | "outline" | "ghost"
/// - size: "sm" | "md" | "lg"
/// - disabled: Whether button is disabled
///
/// # Events
/// - onclick: Click event
widget Button {
    msg Msg { Click }

    model {
        text str = ""
        variant str = "default"
        size str = "md"
        disabled bool = false
    }

    view {}
}
```

**Step 2: Create input.at**

```auto
// stdlib/aura/widgets/form/input.at

#[spec(category = Form, primary_prop = "value")]
#[backend(ark, component = "TextInput")]
#[backend(jet, component = "OutlinedTextField", import = "androidx.compose.material3.OutlinedTextField")]
#[backend(vue, component = "Input", import = "@/components/ui/input/Input")]

/// Input component for text entry.
widget Input {
    msg Msg { Change(value: str) }

    model {
        value str = ""
        placeholder str = ""
        type str = "text"
        disabled bool = false
    }

    view {}
}
```

**Step 3: Create mod.at**

```auto
// stdlib/aura/widgets/form/mod.at

pub use Button, Input, Switch, Checkbox, Slider
```

**Step 4: Commit**

```bash
git commit -am "feat(widget): add form widget files (Button, Input)"
```

---

### Task 10: Create Display Widget Files

**Files:**
- Create: `stdlib/aura/widgets/display/text.at`
- Create: `stdlib/aura/widgets/display/image.at`
- Create: `stdlib/aura/widgets/display/mod.at`

**Step 1: Create text.at**

```auto
// stdlib/aura/widgets/display/text.at

#[spec(category = Display, primary_prop = "text")]
#[backend(ark, component = "Text")]
#[backend(jet, component = "Text", import = "androidx.compose.material3.Text")]
#[backend(vue, component = "span")]

/// Text display component.
widget Text {
    model {
        text str = ""
    }

    view {}
}
```

**Step 2: Create image.at**

```auto
// stdlib/aura/widgets/display/image.at

#[spec(category = Display, primary_prop = "src")]
#[backend(ark, component = "Image")]
#[backend(jet, component = "Image", import = "androidx.compose.foundation.Image")]
#[backend(vue, component = "img")]

/// Image display component.
widget Image {
    model {
        src str = ""
        alt str = ""
    }

    view {}
}
```

**Step 3: Commit**

```bash
git commit -am "feat(widget): add display widget files (Text, Image)"
```

---

## Phase 5: Testing & Documentation

### Task 11: Add WidgetRegistry Unit Tests

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/widget/registry.rs`

**Step 1: Add comprehensive tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_layout_widgets() {
        let registry = WidgetRegistry::with_defaults();
        for tag in ["col", "row", "stack", "scroll"] {
            assert!(registry.contains(tag), "Missing layout widget: {}", tag);
        }
    }

    #[test]
    fn test_all_form_widgets() {
        let registry = WidgetRegistry::with_defaults();
        for tag in ["button", "input"] {
            assert!(registry.contains(tag), "Missing form widget: {}", tag);
        }
    }

    #[test]
    fn test_semantic_widgets_map_to_column() {
        let registry = WidgetRegistry::with_defaults();
        for tag in ["header", "footer", "nav", "main"] {
            let widget = registry.get(tag).unwrap();
            let ark = widget.backend("ark").unwrap();
            assert_eq!(ark.component, "Column");
        }
    }

    #[test]
    fn test_case_insensitive_lookup() {
        let registry = WidgetRegistry::with_defaults();
        assert!(registry.get("BUTTON").is_some());
        assert!(registry.get("Button").is_some());
        assert!(registry.get("button").is_some());
    }
}
```

**Step 2: Run tests**

Run: `cargo test -p auto-lang widget::registry`
Expected: All PASS

**Step 3: Commit**

```bash
git commit -am "test(widget): add comprehensive WidgetRegistry tests"
```

---

### Task 12: Update Documentation

**Files:**
- Modify: `docs/design/aura.md`

**Step 1: Add widget library section**

```markdown
## Widget Library

AURA widgets are defined in `stdlib/aura/widgets/` with backend-specific
annotations.

### Widget Definition

\`\`\`auto
#[spec(category = Form, primary_prop = "text")]
#[backend(ark, component = "Button", import = "@kit.ArkUI")]
widget Button {
    model {
        text str = ""
    }
    view {}
}
\`\`\`

### Importing Widgets

Core widgets are auto-imported. Extended widgets require explicit import:

\`\`\`auto
use aura.widgets: Swiper, Tab
\`\`\`
```

**Step 2: Commit**

```bash
git commit -am "docs: add widget library section to AURA design doc"
```

---

## Success Criteria

- [ ] WidgetSpec and WidgetRegistry types defined
- [ ] All core widgets registered with defaults
- [ ] ArkGenerator uses WidgetRegistry
- [ ] Old ArkComponentRegistry removed
- [ ] Widget .at files created in stdlib/aura/widgets/
- [ ] All tests passing
- [ ] Documentation updated
