# AURA Widget Library Design

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans or superpowers:subagent-driven-development to implement this plan.

**Goal:** Create a reusable, extensible widget library for AURA that replaces hardcoded component definitions in generators with `.at` widget files.

**Architecture:** Widget definitions in `stdlib/aura/widgets/` with `#[spec]` and `#[backend(...)]` annotations. Generators use a WidgetRegistry to load widget specs at compile time. Re-exports via `mod.at` enable short import syntax.

**Tech Stack:** AutoLang widget syntax, annotation system, WidgetRegistry, generator integration

---

## Motivation

Currently, AURA components (Button, Text, Row, etc.) are hardcoded in each generator (`ark/generator.rs`, `jet/generator.rs`). This causes:
- Code duplication across backends
- Difficult to add new components
- Inconsistent behavior between backends
- No central documentation for widgets

A widget library solves these by:
1. Defining widgets once in `.at` files
2. Using annotations for backend-specific mapping
3. Providing auto-import for core widgets
4. Enabling extensible component system

---

## Widget Definition Syntax

### Location

```
stdlib/aura/widgets/
├── mod.at              # pub use layout:*, form:*, display:*
├── layout/
│   ├── mod.at          # pub use Row, Col, Stack, Scroll, Grid
│   ├── row.at
│   ├── col.at
│   ├── stack.at
│   ├── scroll.at
│   └── grid.at
├── form/
│   ├── mod.at          # pub use Button, Input, Switch, Checkbox, Slider
│   ├── button.at
│   ├── input.at
│   ├── switch.at
│   ├── checkbox.at
│   └── slider.at
├── display/
│   ├── mod.at          # pub use Text, Image, Icon, Progress, Divider
│   ├── text.at
│   ├── image.at
│   ├── icon.at
│   ├── progress.at
│   └── divider.at
├── navigation/
│   ├── mod.at          # pub use Nav, Link, Tab, Swiper
│   ├── nav.at
│   ├── link.at
│   ├── tab.at
│   └── swiper.at
└── semantic/
    ├── mod.at          # pub use Header, Footer, Main, Aside, Article
    ├── header.at
    ├── footer.at
    └── main.at
```

### Widget File Structure

```auto
// stdlib/aura/widgets/form/button.at

#[spec(category = Form, primary_prop = "text")]
#[backend(ark, component = "Button", import = "@kit.ArkUI")]
#[backend(jet, composable = "Button", import = "androidx.compose.material3.Button")]
#[backend(vue, component = "button", import = "@vue/runtime-core")]

/// Button component for user interactions.
///
/// # Props
/// - text: Button label text
/// - variant: "default" | "outline" | "ghost" | "destructive"
/// - size: "sm" | "md" | "lg"
/// - disabled: Whether button is disabled
///
/// # Events
/// - Click: Emitted when button is clicked
///
/// # Example
/// ```auto
/// Button (text: "Submit", variant: "primary", onclick: .Submit) {}
/// ```
widget Button {
    msg Msg { Click }

    model {
        text str = ""
        variant str = "default"
        size str = "md"
        disabled bool = false
    }

    computed {
        styleClasses => f"btn btn-${.variant} btn-${.size}"
    }

    view {
        button {
            style: .styleClasses
            text: .text
            onclick: .Click
            disabled: .disabled
        }
    }
}
```

---

## Annotations

### #[spec] - Component Metadata

```auto
#[spec(category = Form, primary_prop = "text")]
```

- `category`: Layout | Form | Display | Navigation | Semantic
- `primary_prop`: The most important prop (for shorthand syntax)
- `slot_support`: true | false (whether supports children)

### #[backend(...)] - Backend Mapping

```auto
#[backend(ark, component = "Button", import = "@kit.ArkUI")]
#[backend(jet, composable = "Button", import = "androidx.compose.material3.Button")]
```

- `ark`: `component` - ArkTS component name, `import` - import statement
- `jet`: `composable` - Compose function name, `import` - import statement
- `vue`: `component` - Vue component name, `import` - import statement

### Backend-Specific Logic

Use `if...else` for backend-specific rendering:

```auto
view {
    if target == "ark" {
        button {
            style: .styleClasses
            onClick: .Click
        }
    } else if target == "jet" {
        button {
            modifier: .styleClasses
            onClick: { .Click }
        }
    }
}
```

---

## Import System

### Auto-Import for Core Widgets

Core widgets are auto-imported:
- Layout: Row, Col, Stack, Scroll, Grid
- Display: Text, Image, Icon
- Form: Button, Input

No `use` statement needed for these.

### Explicit Import for Extended Widgets

```auto
// Import specific widgets
use aura.widgets: Swiper, Tab, Progress

// Or import by category
use aura.widgets.navigation: Swiper, Tab
use aura.widgets.form: Switch, Checkbox
```

### Re-export Structure

`stdlib/aura/widgets/mod.at`:
```auto
// Re-export all widgets for short imports
pub use layout: Row, Col, Stack, Scroll, Grid
pub use form: Button, Input, Switch, Checkbox, Slider
pub use display: Text, Image, Icon, Progress, Divider
pub use navigation: Nav, Link, Tab, Swiper
pub use semantic: Header, Footer, Main, Aside, Article
```

`stdlib/aura/widgets/layout/mod.at`:
```auto
pub use Row, Col, Stack, Scroll, Grid
```

---

## Generator Integration

### WidgetRegistry

```rust
pub struct WidgetRegistry {
    widgets: HashMap<String, WidgetSpec>,
    auto_imports: HashSet<String>,
}

pub struct WidgetSpec {
    name: String,
    category: WidgetCategory,
    props: Vec<PropSpec>,
    events: Vec<EventSpec>,
    backends: HashMap<String, BackendMapping>,
    source_path: PathBuf,
}

pub struct BackendMapping {
    component_name: String,
    import_statement: String,
}
```

### Generator Usage

```rust
// In ark/generator.rs
impl ArkGenerator {
    fn generate_widget(&self, widget: &AuraWidget) -> Result<String> {
        // Look up widget spec
        let spec = self.registry.get(&widget.name)?;

        // Get Ark-specific mapping
        let ark_mapping = spec.backends.get("ark")?;

        // Generate import
        self.add_import(&ark_mapping.import_statement);

        // Generate component call
        self.generate_component_call(&ark_mapping.component_name, widget)
    }
}
```

---

## Implementation Phases

### Phase 1: Core Infrastructure
- Define WidgetSpec, PropSpec, EventSpec types
- Implement WidgetRegistry with loading from `.at` files
- Add annotation parsing (#[spec], #[backend])

### Phase 2: Core Widgets
- Create widget files for Row, Col, Text, Button, Image
- Add backend annotations for Ark and Jet
- Implement auto-import list

### Phase 3: Generator Migration
- Update ArkGenerator to use WidgetRegistry
- Update JetGenerator to use WidgetRegistry
- Remove hardcoded component mappings

### Phase 4: Extended Widgets
- Create remaining widget files (Form, Navigation, Semantic)
- Add documentation comments
- Create widget gallery example

### Phase 5: Testing & Documentation
- Unit tests for WidgetRegistry
- Integration tests for generators
- Update AURA design doc with widget library section

---

## Success Criteria

1. All core widgets defined in `.at` files
2. Generators use WidgetRegistry (no hardcoded components)
3. Auto-import works for core widgets
4. Explicit import works for extended widgets
5. Backend-specific rendering via annotations or `if...else`
6. Documentation in widget files (doc comments)
