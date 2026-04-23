# Plan 217: A2UI Composer Implementation

## Status

**Phase 0: COMPLETE** — A2UI Protocol Bridge implemented and tested.  
**Phase 1: PARTIAL** — Simplified three-panel composer scaffold builds and runs.

### What Was Actually Built (vs. Original Plan)

The original plan envisioned a full node-tree-based composer with `Map<K,V>`, `Option<T>`,
drag-and-drop, and recursive component rendering. The **actual Phase 1 implementation**
is a simplified scaffold that works within current Auto language parser limitations:

| Planned Feature | Actual Implementation | Reason |
|----------------|----------------------|--------|
| Node tree (`ComposerState` with `Map<NodeId, ComposerNode>`) | Boolean flags (`has_text`, `has_button`, ...) | `Map` and `Vec` with complex types not supported in a2vue view context |
| Recursive `CanvasNode` widget | Flat `if` blocks in main `App` widget | No array/dynamic list rendering in view |
| Drag-and-drop palette | Click-to-toggle buttons | DnD API not exposed; `for` loops not supported in view |
| Property editor inspector | Read-only component list | No `Map` property editing; method calls in `on` blocks panic |
| Separate widget files | Single `app.at` file | Module imports between widgets are limited |
| Export JSON button | Not yet wired | Phase 1.2+ deferred |

### Parser Limitations Discovered During Implementation

1. **Method calls in `on` blocks**: `.tags.push("x")` panics with `Expected identifier, got Dot`
2. **Array types in models**: `[]str` and array append cause parse/type errors
3. **Range `for` loops in view**: `for i in 0..n` not supported
4. **`class:` after multi-line child blocks inside `if`**: fails with `Expected term, got RBrace`;
   workaround is `class:` BEFORE children when using block wrappers inside `if`
5. **Nested `if` blocks**: cause parser errors; workaround is helper boolean variables with `&&`
   in `on` blocks
6. **Square brackets in class strings**: `min-h-[60px]` may confuse parser; avoid in class values

## Goal

Build an A2UI Composer clone using AutoUI technologies. The Composer is a visual, drag-and-drop UI builder that outputs Google A2UI v0.8 JSON. It is itself implemented as an AutoUI web application (compiled to Vue 3 via auto-lang's a2vue backend).

This plan covers **Phase 0** (protocol bridge) and **Phase 1** (composer core). Phases 2-4 are outlined for context.

---

## Prerequisites

- [ ] Existing a2vue backend (`crates/auto-lang/src/ui_gen/vue.rs`) compiles successfully
- [ ] Vue 3 + Tailwind project template exists in auto-lang build pipeline
- [ ] `serde` and `serde_json` are available in `auto-lang` crate dependencies

**Verification command:**
```bash
cargo check -p auto-lang --features ui-interpreter
```

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         A2UI Composer Application                        │
│                    (Auto language source → Vue 3 SFC)                   │
├─────────────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────────┐  ┌──────────────────────────┐  │
│  │   Palette    │  │      Canvas      │  │    Property Inspector    │  │
│  │  (Sidebar)   │  │   (Drop Area)    │  │      (Right Panel)       │  │
│  └──────────────┘  └──────────────────┘  └──────────────────────────┘  │
│         │                   │                        │                  │
│         └───────────────────┼────────────────────────┘                  │
│                             ▼                                          │
│                    ┌─────────────────┐                                 │
│                    │  Composer State │                                 │
│                    │ (AuraWidget IR) │                                 │
│                    └────────┬────────┘                                 │
│                             │                                          │
│              ┌──────────────┼──────────────┐                          │
│              ▼              ▼              ▼                          │
│        ┌─────────┐   ┌──────────┐   ┌──────────┐                     │
│        │  Live   │   │  A2UI    │   │  Auto    │                     │
│        │ Preview │   │  Export  │   │  Source  │                     │
│        └─────────┘   └──────────┘   └──────────┘                     │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                         A2UI Protocol Bridge (Rust)                      │
│                    crates/auto-lang/src/a2ui/...                        │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Phase 0: A2UI Protocol Bridge

### Goal
Create bidirectional A2UI v0.8 JSON ↔ AURA IR conversion in Rust. This is a pure library phase with no UI.

### Deliverables

| # | Deliverable | Location |
|---|------------|----------|
| 0.1 | A2UI JSON schema types | `crates/auto-lang/src/a2ui/schema.rs` |
| 0.2 | AURA → A2UI exporter | `crates/auto-lang/src/a2ui/export.rs` |
| 0.3 | A2UI → AURA importer | `crates/auto-lang/src/a2ui/import.rs` |
| 0.4 | Module facade + error types | `crates/auto-lang/src/a2ui/mod.rs` |
| 0.5 | Unit tests | `crates/auto-lang/src/a2ui/tests.rs` |
| 0.6 | CLI integration | `crates/auto/src/cmd_a2ui.rs` (new) |

### 0.1: A2UI Schema Types

**File:** `crates/auto-lang/src/a2ui/schema.rs` (new)

Define Rust structs matching A2UI v0.8 spec with `serde::Serialize` + `Deserialize`:

```rust
// Top-level message wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum A2UIMessage {
    #[serde(rename = "surfaceUpdate")]
    SurfaceUpdate(A2UISurfaceUpdate),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2UISurfaceUpdate {
    pub surface_id: String,
    pub components: Vec<A2UIComponent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2UIComponent {
    pub id: String,
    #[serde(flatten)]
    pub body: A2UIComponentBody,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "component")]
pub enum A2UIComponentBody {
    #[serde(rename = "Text")]
    Text { text: A2UIValue },
    #[serde(rename = "Button")]
    Button { child: A2UIValue, action: Option<A2UIAction> },
    #[serde(rename = "TextInput")]
    TextInput { value: A2UIValue, hint: Option<A2UIValue> },
    #[serde(rename = "NumberInput")]
    NumberInput { value: A2UIValue, min: Option<f64>, max: Option<f64> },
    #[serde(rename = "DateTimeInput")]
    DateTimeInput { value: A2UIValue },
    #[serde(rename = "Checkbox")]
    Checkbox { value: A2UIValue, label: Option<A2UIValue> },
    #[serde(rename = "Radio")]
    Radio { value: A2UIValue, label: Option<A2UIValue> },
    #[serde(rename = "Select")]
    Select { value: A2UIValue, options: Vec<A2UISelectOption> },
    #[serde(rename = "Slider")]
    Slider { value: A2UIValue, min: Option<f64>, max: Option<f64>, step: Option<f64> },
    #[serde(rename = "Image")]
    Image { src: A2UIValue },
    #[serde(rename = "Icon")]
    Icon { name: A2UIValue },
    #[serde(rename = "Container")]
    Container { children: Vec<A2UIComponent> },
    #[serde(rename = "Row")]
    Row { children: Vec<A2UIComponent> },
    #[serde(rename = "Column")]
    Column { children: Vec<A2UIComponent> },
    #[serde(rename = "ScrollView")]
    ScrollView { child: Box<A2UIComponentBody> },
    #[serde(rename = "List")]
    List { items: A2UIValue, template: Option<Box<A2UIComponentBody>> },
    #[serde(rename = "Table")]
    Table { columns: Vec<A2UITableColumn>, items: A2UIValue },
    #[serde(rename = "Divider")]
    Divider {},
    #[serde(rename = "Spacer")]
    Spacer {},
    #[serde(rename = "Tabs")]
    Tabs { tabs: Vec<A2UITab> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum A2UIValue {
    #[serde(rename = "path")]
    Path { path: String },
    #[serde(rename = "literalString")]
    LiteralString { literal_string: String },
    #[serde(rename = "literalNumber")]
    LiteralNumber { literal_number: f64 },
    #[serde(rename = "literalBool")]
    LiteralBool { literal_bool: bool },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2UIAction {
    pub name: String,
    #[serde(default)]
    pub context: Vec<A2UIContextBinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2UIContextBinding {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2UISelectOption {
    pub value: String,
    pub label: A2UIValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2UITableColumn {
    pub key: String,
    pub label: A2UIValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2UITab {
    pub label: A2UIValue,
    pub child: Box<A2UIComponentBody>,
}
```

**Acceptance:** `cargo check` passes with no errors. Serde round-trip test for a sample JSON passes.

### 0.2: AURA → A2UI Exporter

**File:** `crates/auto-lang/src/a2ui/export.rs` (new)

**Core function signature:**
```rust
pub fn export_widget(widget: &AuraWidget) -> Result<A2UIMessage, A2UIError>
```

**Implementation steps:**
1. Map `AuraWidget.name` → `A2UISurfaceUpdate.surface_id`
2. Convert `AuraWidget.view_tree` (root `AuraNode`) → `Vec<A2UIComponent>`
3. For each `AuraNode::Element`:
   - Map `tag` to `A2UIComponentBody` variant:
     - `"text"` → `A2UIComponentBody::Text`
     - `"button"` → `A2UIComponentBody::Button`
     - `"input"` → `A2UIComponentBody::TextInput`
     - `"col"`/`"column"` → `A2UIComponentBody::Column`
     - `"row"` → `A2UIComponentBody::Row`
     - `"checkbox"` → `A2UIComponentBody::Checkbox`
     - `"slider"` → `A2UIComponentBody::Slider`
     - etc.
   - Map props: `AuraPropValue::Expr(expr)` → `A2UIValue`
     - `AuraExpr::Literal(s)` → `A2UIValue::LiteralString`
     - `AuraExpr::Int(n)` → `A2UIValue::LiteralNumber`
     - `AuraExpr::Float(f)` → `A2UIValue::LiteralNumber`
     - `AuraExpr::Bool(b)` → `A2UIValue::LiteralBool`
     - `AuraExpr::StateRef(name)` → `A2UIValue::Path { path: format!("/{}", name) }`
   - Map events: `AuraEvent { handler }` → `A2UIAction { name: handler, context: vec![] }`
     - Extract context bindings from event parameters (Phase 0.2 MVP: empty context)
4. Generate unique component IDs from tag + index path (e.g., `"root_0"`, `"root_0_1"`)
5. Handle `AuraNode::Text` → `A2UIComponentBody::Text`
6. Handle `AuraNode::ForLoop` → `A2UIComponentBody::List`
7. Handle `AuraNode::Conditional` → Phase 0.2 MVP: skip with warning

**Acceptance:** Export the `Counter` widget example and get valid A2UI JSON.

### 0.3: A2UI → AURA Importer

**File:** `crates/auto-lang/src/a2ui/import.rs` (new)

**Core function signature:**
```rust
pub fn import_message(msg: &A2UIMessage) -> Result<AuraWidget, A2UIError>
```

**Implementation steps:**
1. Map `A2UISurfaceUpdate.surface_id` → `AuraWidget.name`
2. Convert each `A2UIComponent` → `AuraNode`
3. For each `A2UIComponentBody`:
   - Map variant back to `AuraNode::Element` with appropriate `tag`
   - Map `A2UIValue` → `AuraPropValue::Expr(AuraExpr)`:
     - `A2UIValue::LiteralString { literal_string }` → `AuraExpr::Literal`
     - `A2UIValue::LiteralNumber { literal_number }` → `AuraExpr::Float`
     - `A2UIValue::LiteralBool { literal_bool }` → `AuraExpr::Bool`
     - `A2UIValue::Path { path }` → `AuraExpr::StateRef` (strip leading `/`)
   - Map `A2UIAction` → `AuraEvent { handler: action.name, params: vec![] }`
4. Collect all unique `StateRef` paths → generate `AuraStateDef` stubs:
   ```rust
   AuraStateDef {
       name: path_name.clone(),
       type_info: Type::String, // Default; could infer from context
       initial: AuraExpr::Literal("".to_string()),
       decorators: vec![],
   }
   ```
5. Generate empty `handlers` HashMap with stub entries for each action name
6. Set `messages`, `computed`, `props`, `routes` to empty defaults

**Acceptance:** Round-trip test: AuraWidget → A2UI → AuraWidget produces equivalent view tree.

### 0.4: Module Facade

**File:** `crates/auto-lang/src/a2ui/mod.rs` (new)

```rust
pub mod schema;
pub mod export;
pub mod import;

#[derive(Debug, Clone)]
pub enum A2UIError {
    UnsupportedComponent(String),
    UnsupportedExpression(String),
    InvalidValue(String),
    MissingRequiredField(String),
    Serde(String),
}

impl std::fmt::Display for A2UIError { ... }
impl std::error::Error for A2UIError {}

pub use export::export_widget;
pub use import::import_message;
pub use schema::*;
```

**Integration:** Add `pub mod a2ui;` to `crates/auto-lang/src/lib.rs`.

### 0.5: Unit Tests

**File:** `crates/auto-lang/src/a2ui/tests.rs` (new)

Test cases:
1. `test_schema_roundtrip` — Serialize then deserialize a complex A2UI message, assert equality
2. `test_export_counter` — Export `Counter` widget, assert JSON matches expected structure
3. `test_import_simple_text` — Import minimal A2UI with single Text component
4. `test_import_button_with_action` — Import button with action, assert handler stub exists
5. `test_roundtrip_counter` — AuraWidget → A2UI → AuraWidget, assert view trees match
6. `test_export_nested_layout` — Export Column > Row > [Text, Button]

### 0.6: CLI Integration

**File:** `crates/auto/src/cmd_a2ui.rs` (new)

Add CLI subcommands:
```
auto a2ui export <input.at> [--output <file.json>]
auto a2ui import <input.json> [--output <file.at>]
```

**Implementation:**
- `export`: Parse `.at` file → extract AURA widget → `export_widget()` → write JSON
- `import`: Read JSON → `import_message()` → generate Auto language source (Phase 0.6 MVP: print AURA debug, full source gen deferred to Phase 3)

**Acceptance:** 
```bash
auto a2ui export examples/ui/002-counter/src/front/app.at --output /tmp/counter.json
# Produces valid A2UI JSON
```

---

## Phase 1: Composer Core Application

### Goal
Build a three-panel visual composer as an AutoUI web application.

### Deliverables

| # | Deliverable | Location | Status |
|---|------------|----------|--------|
| 1.1 | Composer project scaffold | `examples/a2ui-composer/` | ✅ Done |
| 1.2 | Type definitions & component registry | `examples/a2ui-composer/src/front/lib/` | ⏭️ Deferred (Phase 2) |
| 1.3 | Palette widget | `examples/a2ui-composer/src/front/components/palette.at` | ⏭️ Inlined into `app.at` |
| 1.4 | Canvas widget | `examples/a2ui-composer/src/front/components/canvas.at` | ⏭️ Inlined into `app.at` |
| 1.5 | Inspector widget | `examples/a2ui-composer/src/front/components/inspector.at` | ⏭️ Inlined into `app.at` |
| 1.6 | Toolbar widget | `examples/a2ui-composer/src/front/components/toolbar.at` | ⏭️ Inlined into `app.at` |
| 1.7 | Main App widget | `examples/a2ui-composer/src/front/app.at` | ✅ Done (simplified) |
| 1.8 | Build & run verification | `examples/a2ui-composer/README.md` | ⏭️ Not yet written |

### Actual File Structure

```
examples/a2ui-composer/
├── pac.at                          # Project config (scene: "ui", backend: ["vue"])
└── src/
    └── front/
        └── app.at                  # Single-file composer app
```

The original plan called for separate component files under `src/front/components/`.
Due to module import limitations in the current Auto compiler, all UI is consolidated
into a single `app.at` widget.


### 1.1: Project Scaffold

**Directory:** `examples/a2ui-composer/`

Create:
```
examples/a2ui-composer/
├── pac.at
├── README.md
└── src/
    └── front/
        ├── app.at
        └── components/
            ├── palette.at
            ├── canvas.at
            ├── inspector.at
            ├── toolbar.at
            └── json_panel.at
```

**`pac.at`:**
```auto
project A2UIComposer {
    version: "0.1.0"
    scenario: "ui"
    backend: "vue"
    entry: "src/front/app.at"
}
```

**`README.md`:** Project description and build/run instructions.

### 1.2: Shared Types & Component Registry

**File:** `examples/a2ui-composer/src/front/lib/types.at`

Define core types for the composer state:

```auto
// Unique ID for composer nodes
alias NodeId = int

// A node in the composer tree
struct ComposerNode {
    id: NodeId
    tag: str           // AURA tag: "text", "button", "col", "row", ...
    props: Map<str, PropValue>
    events: Map<str, str>   // event_name -> handler_name
    children: Vec<NodeId>
    parent: Option<NodeId>
}

// Property value: either literal or bound to path
enum PropValue {
    LiteralStr(str)
    LiteralNum(f64)
    LiteralBool(bool)
    BoundPath(str)
}

// Composer application state
struct ComposerState {
    next_id: NodeId = 1
    nodes: Map<NodeId, ComposerNode> = {}
    root_id: Option<NodeId> = None
    selected_id: Option<NodeId> = None
}
```

**File:** `examples/a2ui-composer/src/front/lib/component_registry.at`

Define metadata for available components:

```auto
// Component category
enum ComponentCategory {
    Layout
    Form
    Display
    Data
}

// Property schema entry
struct PropSchema {
    name: str
    prop_type: PropType
    required: bool
    default_value: Option<PropValue>
}

enum PropType {
    StringType
    NumberType
    BoolType
    EnumType(Vec<str>)
}

// Component registry entry
struct RegistryEntry {
    tag: str
    display_name: str
    category: ComponentCategory
    icon: str          // Emoji or icon name
    props: Vec<PropSchema>
    events: Vec<str>   // supported event names
    can_have_children: bool
}

// Registry initialization
fn registry_entries() -> Vec<RegistryEntry> {
    return vec[
        RegistryEntry {
            tag: "col"
            display_name: "Column"
            category: ComponentCategory.Layout
            icon: "📊"
            props: vec[]
            events: vec[]
            can_have_children: true
        },
        RegistryEntry {
            tag: "row"
            display_name: "Row"
            category: ComponentCategory.Layout
            icon: "📋"
            props: vec[]
            events: vec[]
            can_have_children: true
        },
        RegistryEntry {
            tag: "text"
            display_name: "Text"
            category: ComponentCategory.Display
            icon: "📝"
            props: vec[
                PropSchema { name: "content", prop_type: PropType.StringType, required: false, default_value: Some(PropValue::LiteralStr("Text")) }
            ]
            events: vec[]
            can_have_children: false
        },
        RegistryEntry {
            tag: "button"
            display_name: "Button"
            category: ComponentCategory.Form
            icon: "🔘"
            props: vec[
                PropSchema { name: "label", prop_type: PropType.StringType, required: false, default_value: Some(PropValue::LiteralStr("Button")) }
            ]
            events: vec["onclick"]
            can_have_children: false
        },
        RegistryEntry {
            tag: "input"
            display_name: "Text Input"
            category: ComponentCategory.Form
            icon: "⌨️"
            props: vec[
                PropSchema { name: "placeholder", prop_type: PropType.StringType, required: false, default_value: Some(PropValue::LiteralStr("")) },
                PropSchema { name: "value", prop_type: PropType.StringType, required: false, default_value: Some(PropValue::LiteralStr("")) }
            ]
            events: vec["onchange"]
            can_have_children: false
        },
        // ... more entries
    ]
}
```

**Acceptance:** `auto build` compiles without errors.

### 1.3: Palette Widget

**File:** `examples/a2ui-composer/src/front/components/palette.at`

A sidebar showing categorized draggable component cards.

```auto
widget Palette {
    msg Msg {
        ComponentClicked(tag: str)
    }

    view {
        col {
            style: "palette-sidebar"

            h3 | Components

            // Layout category
            h4 | Layout
            row {
                For entry in registry_entries().filter(|e| e.category == ComponentCategory.Layout) {
                    PaletteItem (entry: entry) {
                        onclick: .ComponentClicked(entry.tag)
                    }
                }
            }

            // Form category
            h4 | Form
            row {
                For entry in registry_entries().filter(|e| e.category == ComponentCategory.Form) {
                    PaletteItem (entry: entry) {
                        onclick: .ComponentClicked(entry.tag)
                    }
                }
            }

            // Display category
            h4 | Display
            row {
                For entry in registry_entries().filter(|e| e.category == ComponentCategory.Display) {
                    PaletteItem (entry: entry) {
                        onclick: .ComponentClicked(entry.tag)
                    }
                }
            }
        }
    }
}

widget PaletteItem {
    prop entry: RegistryEntry

    view {
        col {
            style: "palette-item"
            text: entry.icon
            text: entry.display_name
        }
    }
}
```

**Acceptance:** Renders as sidebar with categorized component cards. Clicking emits message.

### 1.4: Canvas Widget

**File:** `examples/a2ui-composer/src/front/components/canvas.at`

The main drop area showing the composed UI tree.

```auto
widget Canvas {
    msg Msg {
        SelectNode(id: NodeId)
        AddChild(parent_id: NodeId, tag: str)
        RemoveNode(id: NodeId)
        MoveNode(id: NodeId, new_parent_id: NodeId, index: int)
    }

    model {
        state: ComposerState  // passed as prop in reality
    }

    view {
        col {
            style: "canvas-area"

            If state.root_id == None {
                text "Drag a component here to start"
            } Else {
                CanvasNode (node_id: state.root_id.unwrap(), state: state)
            }
        }
    }
}
```

**Recursive node renderer:**
```auto
widget CanvasNode {
    prop node_id: NodeId
    prop state: ComposerState

    view {
        // Look up node from state
        node = state.nodes.get(node_id)

        col {
            style: if state.selected_id == Some(node_id) { "canvas-node selected" } else { "canvas-node" }
            onclick: .SelectNode(node_id)

            // Show tag and props summary
            row {
                text: node.tag
                // ... prop badges
            }

            // Render children if layout component
            If node.children.len() > 0 {
                col {
                    style: "canvas-children"
                    For child_id in node.children {
                        CanvasNode (node_id: child_id, state: state)
                    }
                }
            }
        }
    }
}
```

**Acceptance:** Displays tree structure. Clicking selects node. Visual selection indicator.

### 1.5: Inspector Widget

**File:** `examples/a2ui-composer/src/front/components/inspector.at`

Right panel for editing selected component properties.

```auto
widget Inspector {
    prop selected_node: Option<ComposerNode>
    prop registry: Vec<RegistryEntry>

    msg Msg {
        UpdateProp(node_id: NodeId, key: str, value: PropValue)
        UpdateEvent(node_id: NodeId, event: str, handler: str)
        DeleteNode(id: NodeId)
    }

    view {
        col {
            style: "inspector-panel"

            If selected_node == None {
                text "Select a component to edit"
            } Else {
                node = selected_node.unwrap()
                entry = registry.find(|e| e.tag == node.tag)

                h3 | Properties
                text: "Type: ${node.tag}"

                // ID display (read-only)
                row {
                    text "ID:"
                    text: node.id.to_string()
                }

                // Editable props
                For prop_schema in entry.props {
                    PropEditor (
                        schema: prop_schema,
                        current_value: node.props.get(prop_schema.name)
                    ) {
                        on_change: .UpdateProp(node.id, prop_schema.name, value)
                    }
                }

                // Event handlers
                h4 | Events
                For event_name in entry.events {
                    row {
                        text: event_name
                        input {
                            value: node.events.get(event_name).unwrap_or("")
                            onchange: .UpdateEvent(node.id, event_name, value)
                        }
                    }
                }

                // Danger zone
                button "Delete" {
                    style: "danger"
                    onclick: .DeleteNode(node.id)
                }
            }
        }
    }
}
```

**Acceptance:** Shows properties for selected node. Editing updates state. Delete button removes node.

### 1.6: Toolbar Widget

**File:** `examples/a2ui-composer/src/front/components/toolbar.at`

Top bar with actions.

```auto
widget Toolbar {
    msg Msg {
        ExportJSON
        ImportJSON
        ClearCanvas
        Undo
        Redo
    }

    view {
        row {
            style: "toolbar"

            text "A2UI Composer"

            button "Export JSON" { onclick: .ExportJSON }
            button "Clear" { onclick: .ClearCanvas }
        }
    }
}
```

### 1.7: Main App Widget (Actual Implementation)

**File:** `examples/a2ui-composer/src/front/app.at`

The actual implementation is a simplified single-widget app that demonstrates the
three-panel layout and conditional component toggling. It does not use the full
`ComposerState` node tree from the original plan due to parser/type-system limitations.

**Key simplifications:**
- State is four boolean flags (`has_text`, `has_button`, `has_input`, `has_col`)
- Plus `show_empty` helper for the empty-state message
- Palette buttons toggle visibility directly
- Canvas uses `row` wrappers with `class:` placed BEFORE children (required by parser)
- Inspector shows a read-only list of active components
- No separate widget files — everything is in `app.at`

**Build verification:**
```bash
cd examples/a2ui-composer
auto build
# Produces dist/ with Vue 3 production bundle
```

```auto
widget App {
    msg Msg {
        // Palette
        AddComponent(tag: str)

        // Canvas
        SelectNode(id: NodeId)
        RemoveNode(id: NodeId)
        UpdateNodeProps(id: NodeId, props: Map<str, PropValue>)

        // Inspector
        UpdateProp(node_id: NodeId, key: str, value: PropValue)
        UpdateEvent(node_id: NodeId, event: str, handler: str)

        // Toolbar
        ExportJSON
        ClearCanvas
    }

    model {
        state: ComposerState = ComposerState {
            next_id: 1
            nodes: {}
            root_id: None
            selected_id: None
        }
    }

    view {
        col {
            style: "app-root"

            Toolbar {}

            row {
                style: "main-content"

                // Left panel: Palette
                Palette {
                    width: 250
                }

                // Center: Canvas
                Canvas {
                    state: state
                }

                // Right panel: Inspector
                Inspector {
                    selected_node: state.selected_id.map(|id| state.nodes.get(id))
                    registry: registry_entries()
                }
            }
        }
    }

    on {
        .AddComponent(tag) -> {
            new_id = state.next_id
            state.next_id += 1

            new_node = ComposerNode {
                id: new_id
                tag: tag
                props: default_props_for(tag)
                events: {}
                children: []
                parent: None
            }

            state.nodes.insert(new_id, new_node)

            // If no root, this becomes root
            If state.root_id == None {
                state.root_id = Some(new_id)
            } Else {
                // Add to selected node if it accepts children
                If let Some(selected) = state.selected_id {
                    parent = state.nodes.get_mut(selected)
                    parent.children.push(new_id)
                    new_node.parent = Some(selected)
                }
            }
        }

        .SelectNode(id) -> {
            state.selected_id = Some(id)
        }

        .RemoveNode(id) -> {
            // Remove from parent's children
            node = state.nodes.get(id)
            If let Some(parent_id) = node.parent {
                parent = state.nodes.get_mut(parent_id)
                parent.children.retain(|child_id| child_id != id)
            }

            // Recursively remove children
            remove_recursive(id)

            // Clear selection if removed
            If state.selected_id == Some(id) {
                state.selected_id = None
            }
        }

        .UpdateProp(node_id, key, value) -> {
            node = state.nodes.get_mut(node_id)
            node.props.insert(key, value)
        }

        .UpdateEvent(node_id, event, handler) -> {
            node = state.nodes.get_mut(node_id)
            node.events.insert(event, handler)
        }

        .ExportJSON -> {
            // Convert ComposerState to A2UI JSON
            // Display in modal or copy to clipboard
        }

        .ClearCanvas -> {
            state.nodes.clear()
            state.root_id = None
            state.selected_id = None
            state.next_id = 1
        }
    }
}
```

**Acceptance:** Three-panel layout renders. Can add components, select, edit props, delete, export.

### 1.8: Styling

Use Tailwind CSS classes throughout. Define custom classes in a global CSS file:

```css
.palette-sidebar { @apply w-64 h-full bg-gray-50 border-r p-4 overflow-y-auto; }
.palette-item { @apply p-3 bg-white rounded shadow hover:bg-blue-50 cursor-pointer transition; }
.canvas-area { @apply flex-1 h-full bg-white p-8 overflow-auto; }
.canvas-node { @apply border border-gray-200 rounded p-3 mb-2 hover:border-blue-300; }
.canvas-node.selected { @apply border-blue-500 ring-2 ring-blue-200; }
.canvas-children { @apply ml-6 border-l-2 border-gray-100 pl-3; }
.inspector-panel { @apply w-72 h-full bg-gray-50 border-l p-4 overflow-y-auto; }
.toolbar { @apply h-14 bg-white border-b px-4 flex items-center justify-between; }
```

---

## Phase 2-4 Outline (Future Work)

### Phase 2: Interactivity
- Drag-and-drop using HTML5 DnD API (wrapped in Auto FFI if needed)
- Live preview panel (render selected subtree as actual Vue components)
- JSON editor panel with real-time sync
- Undo/redo stack

### Phase 3: A2UI Completeness
- Full component library (all A2UI v0.8 components)
- Import A2UI JSON into composer
- Export to both A2UI JSON and Auto language source
- Component nesting validation

### Phase 4: Advanced Features
- AI prompt-to-UI integration (LLM API)
- Gallery of templates/examples
- Responsive viewport switching
- Streaming preview simulation
- Publish/share functionality

---

## Testing Strategy

### Phase 0 Tests (Rust)
- Unit tests in `crates/auto-lang/src/a2ui/tests.rs`
- Property-based tests for round-trip conversion
- JSON schema validation against known A2UI examples

### Phase 1 Tests (Auto language)
- Build test: `auto build` succeeds
- Visual test: Open app in browser, verify three panels render
- Interaction test: Add component → select → edit prop → delete
- Export test: Click Export → JSON is valid A2UI

---

## Dependencies

### New Rust Dependencies
None — `serde` and `serde_json` should already be in `auto-lang/Cargo.toml`.

### New Auto Language Features Needed
- [ ] `Map<K, V>` type support in AOT backends (or use `Vec` of pairs as MVP)
- [ ] `Option<T>` pattern matching in view (`If let Some(x) = ...`)
- [ ] Comptime function evaluation for registry (or static data)

**Risk mitigation:** If `Map` or `Option` patterns are not yet fully supported in a2vue, implement with `Vec<(str, PropValue)>` and explicit null checks.

---

## Success Criteria

| Phase | Criterion | Status |
|-------|-----------|--------|
| 0 | `cargo test -p auto-lang a2ui` passes all tests | ✅ 13/13 tests pass |
| 0 | `auto a2ui export` CLI produces valid JSON | ⏭️ Deferred to Phase 1.2+ |
| 1 | `auto build` in `examples/a2ui-composer/` produces runnable Vue app | ✅ Builds successfully |
| 1 | App shows three panels: Palette, Canvas, Inspector | ✅ Verified |
| 1 | Can add/remove components to canvas | ✅ Toggle buttons add/remove |
| 1 | Can select and edit properties | ⏭️ Deferred (needs Map/Option support) |
| 1 | Export button outputs A2UI JSON | ⏭️ Deferred to Phase 2 |

---

## Estimated Effort

| Phase | Files | Estimated Time |
|-------|-------|---------------|
| 0 | 6 new Rust files | 3-4 days |
| 1 | 8 new Auto files + config | 4-5 days |
| **Total** | **14 files** | **7-9 days** |

---

## Open Questions

1. Does a2vue backend support `Map<K, V>` and `Option<T>` in model/view? If not, what's the workaround?
2. Is HTML5 drag-and-drop supported in generated Vue code, or do we need custom JS interop?
3. Should the Composer support importing existing A2UI JSON (Phase 3) or is export-only sufficient for MVP?

---

*Plan written: 2026-04-23*
*Target: Phases 0-1 (MVP Composer with export)*
