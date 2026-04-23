# A2UI Composer Analysis & AutoUI Implementation Plan

## 1. Technology Analysis: Google A2UI

### What is A2UI?
A2UI (Agent-to-User Interface) is Google's open declarative UI protocol that enables AI agents to safely render rich interactive interfaces without executing arbitrary code.

### Core Architecture
```
User Message → Agent Generates A2UI JSON → Streams to Client → Native Rendering → User Interaction → Events to Agent
```

### A2UI JSON Protocol (v0.8)
Key structure:
```json
{
  "surfaceUpdate": {
    "surfaceId": "booking",
    "components": [
      {"id": "title", "component": {"Text": {"text": {"literalString": "Book Your Table"}}}},
      {"id": "datetime", "component": {"DateTimeInput": {"value": {"path": "/booking/date"}}}},
      {"id": "submit-btn", "component": {"Button": {"child": "Confirm", "action": {"name": "confirm_booking"}}}}
    ]
  }
}
```

### Component Types (A2UI v0.8)
- **Layout**: Container, Row, Column, ScrollView
- **Form**: TextInput, NumberInput, DateTimeInput, Button, Checkbox, Radio, Select, Slider
- **Display**: Text, Image, Icon, Divider, Spacer
- **Data**: List, Table, Chart, Map
- **Navigation**: Tabs, Navigation, Link

### Data Binding Model
- Path-based: `{ "path": "/booking/date" }` binds to data model
- Literal values: `{ "literalString": "Hello" }`
- Actions send events back with context bindings

### A2UI Composer Features
1. **Visual Builder**: Drag & drop components, WYSIWYG
2. **Component Palette**: Categorized primitive components
3. **Property Inspector**: Edit component properties
4. **Canvas/Preview**: Live rendering of composed UI
5. **JSON Export**: One-click export to A2UI format
6. **AI Prompting**: Natural language to UI generation

---

## 2. Technology Analysis: AutoUI (auto-lang)

### Core UI Stack
```
Auto Source Code → Parser → AST → AURA Extractor → AuraWidget IR → Backend Generators
                                                    ↓
                                    ┌───────────────┼───────────────┐
                                    ↓               ↓               ↓
                                  Vue SFC      Jetpack Compose    ArkTS
                                  (a2vue)         (a2jet)         (a2ark)
```

### AURA IR (Auto UI Representation Abstract)
The heart of AutoUI — a backend-agnostic UI intermediate representation:

```rust
pub struct AuraWidget {
    pub name: String,
    pub state_vars: Vec<AuraStateDef>,    // model
    pub view_tree: AuraNode,               // view
    pub handlers: HashMap<String, LogicPayload>, // on
    pub messages: Vec<AuraMessage>,        // msg
    pub props: Vec<AuraProp>,              // props for reuse
}

pub enum AuraNode {
    Element { tag, props, events, children },
    Text(Literal | Interpolated),
    ForLoop { var, iterable, body },
    Conditional { condition, then_body, else_body },
    Component { name, props, events },
}
```

### Auto Language Widget Syntax
```auto
widget Counter {
    msg Msg { Inc, Dec }
    model { count int = 0 }
    view {
        col {
            button + { onclick: .Inc }
            h2 > Current Count: ${.count}
            button - { onclick: .Dec }
        }
    }
    on {
        .Inc => { .count += 1 }
        .Dec => { .count -= 1 }
    }
}
```

### View Enum (Runtime Abstraction)
```rust
pub enum View<M> {
    Text { content, style },
    Button { label, onclick, style },
    Input { placeholder, value, on_change, style },
    Column { children, spacing, style },
    Row { children, spacing, style },
    Checkbox { is_checked, label, on_toggle, style },
    Select { options, selected_index, on_select, style },
    Slider { min, max, value, on_change, style },
    Table { headers, rows, style },
    Accordion { items, on_toggle, style },
    Tabs { labels, contents, selected, on_select, style },
    // ... and more
}
```

### Existing Backends
| Backend | Status | Output |
|---------|--------|--------|
| a2vue | Complete | Vue 3 SFC (Tailwind + shadcn-vue) |
| a2jet | Complete | Kotlin/Jetpack Compose |
| a2ark | Complete | ArkTS/HarmonyOS |
| a2gpui | Partial | GPUI (Rust native desktop) |
| a2iced | Partial | Iced (Rust cross-platform) |

---

## 3. Architecture Mapping: A2UI ↔ AutoUI

### Conceptual Alignment
Both systems are **declarative UI protocols** — they describe WHAT to render, not HOW to render it.

| A2UI Concept | AutoUI Equivalent | Notes |
|-------------|-------------------|-------|
| `surfaceUpdate` | `AuraWidget` view tree | Surface = Widget |
| `components[]` | `AuraNode::Element` children | Component list = View children |
| `component.Text` | `AuraNode::Element { tag: "text" }` | Direct mapping |
| `component.Button` | `AuraNode::Element { tag: "button" }` | Direct mapping |
| `value.path` | `AuraExpr::StateRef` | Data binding |
| `action.name` | `AuraEvent { handler }` | Event routing |
| `id` | Implicit via tree structure | Could add explicit IDs |
| Data model (external) | `AuraWidget.state_vars` | AutoUI embeds model in widget |

### Key Differences
1. **A2UI** is pure JSON, transport-oriented, agent-generated
2. **AURA** is code-oriented, compiled, with embedded logic handlers
3. **A2UI** uses path-based data binding to external model
4. **AURA** uses state references with reactive updates
5. **A2UI** is read-only UI description (no logic in JSON)
6. **AURA** includes event handlers (LogicPayload: AstBlock | Bytecode)

### Bridge Strategy
Since AURA is a superset of A2UI's capabilities (AURA has logic handlers), we can:
- **Export**: Convert `AuraWidget` → A2UI JSON (lossy: handlers become action names)
- **Import**: Convert A2UI JSON → `AuraWidget` (generate stub handlers)

---

## 4. Incremental Implementation Plan

### Phase 0: Foundation — A2UI Protocol Bridge
**Goal**: Enable bidirectional A2UI ↔ AURA conversion

**Tasks**:
1. Define A2UI JSON schema types in Rust (`crates/auto-lang/src/a2ui/`)
   - `A2UISurfaceUpdate`, `A2UIComponent`, `A2UIValue`, `A2UIAction`
   - Serde serialize/deserialize
2. Implement `aura_to_a2ui()` converter
   - Map AuraNode tags to A2UI component types
   - Convert state refs to path bindings
   - Extract action names from event handlers
3. Implement `a2ui_to_aura()` importer
   - Map A2UI components to AuraNode elements
   - Generate stub state vars from path references
   - Create placeholder event handlers
4. Add CLI command: `auto a2ui export widget.at` and `auto a2ui import widget.json`

**Deliverable**: Rust library + CLI for A2UI JSON conversion

---

### Phase 1: Composer Core — AutoUI Web Application
**Goal**: Build the visual composer as an AutoUI app compiling to Vue

**Tasks**:
1. Create project: `examples/a2ui-composer/`
2. Define data model (`composer.at`):
   ```auto
   widget Composer {
       msg Msg { 
           AddComponent(tag, parent_id)
           RemoveComponent(id)
           MoveComponent(id, new_parent_id, index)
           UpdateProp(id, key, value)
           SelectComponent(id)
           ExportJSON
       }
       model {
           components Vec<ComposerNode> = []
           selected_id Option<int> = None
           canvas_size CanvasSize = { width: 800, height: 600 }
       }
   }
   ```
3. Build three-panel layout:
   - **Left**: Component Palette (draggable component types)
   - **Center**: Canvas (nested tree visualization)
   - **Right**: Property Inspector (edit selected component)
4. Implement AutoUI widgets for the composer:
   - `PaletteItem` — draggable component card
   - `CanvasNode` — recursive tree renderer with selection
   - `PropEditor` — type-aware property input (string, number, bool, enum)
   - `Toolbar` — export, clear, undo/redo

**Deliverable**: Running AutoUI web app with visual layout

---

### Phase 2: Interactivity — Drag & Drop + Live Preview
**Goal**: Make the composer fully interactive

**Tasks**:
1. Add drag-and-drop support:
   - Drag from palette to canvas
   - Reorder within canvas (drag to reposition)
   - Visual drop indicators
2. Implement live preview:
   - Render selected component subtree using AutoUI's Vue generator
   - Or use a lightweight renderer component
   - Show both "edit mode" (with borders/handles) and "preview mode"
3. Add property binding editor:
   - Toggle between literal value and state path binding
   - Visual indicator for bound vs literal props
4. Implement JSON panel:
   - Split view: visual editor | JSON editor
   - Real-time sync between visual and JSON
   - Syntax highlighting for JSON

**Deliverable**: Fully interactive composer with DnD and live preview

---

### Phase 3: A2UI Export/Import + Component Library
**Goal**: Full A2UI compatibility

**Tasks**:
1. Integrate Phase 0 bridge into the web app:
   - "Export A2UI" button → downloads JSON
   - "Import A2UI" button → parses JSON into composer state
2. Build comprehensive component library matching A2UI spec:
   - Layout: Container, Row, Column, ScrollView
   - Form: TextInput, NumberInput, DateTimeInput, Button, Checkbox, Radio, Select, Slider
   - Display: Text, Image, Icon, Divider
   - Data: List, Table
3. Add component-specific property schemas:
   - Each component type has defined editable properties
   - Type validation (number inputs for numeric props, etc.)
4. Implement component grouping/categorization in palette

**Deliverable**: A2UI-compatible composer with full component library

---

### Phase 4: Advanced Features
**Goal**: Match full A2UI Composer experience

**Tasks**:
1. **AI Integration**: Connect to LLM for prompt-to-UI generation
   - Input: "Create a restaurant booking form"
   - Process: LLM generates A2UI JSON → Import to composer
   - Requires: LLM API integration, prompt engineering
2. **Streaming Preview**: Simulate A2UI streaming updates
   - Show components appearing incrementally
   - Demonstrates A2UI protocol behavior
3. **Gallery/Templates**: Pre-built example UIs
   - Restaurant finder (A2UI demo)
   - Budget tracker
   - Booking form
   - Contact form
4. **Responsive Preview**: Toggle between desktop/mobile viewports
5. **Multi-backend Export**: Export to both A2UI JSON and Auto language source

**Deliverable**: Feature-complete A2UI Composer clone

---

## 5. Technical Implementation Details

### File Structure
```
examples/a2ui-composer/
├── pac.at                          # Project config (scenario: "ui", backend: "vue")
├── src/
│   ├── front/
│   │   ├── app.at                  # Main Composer widget
│   │   ├── components/
│   │   │   ├── palette.at          # Component palette panel
│   │   │   ├── canvas.at           # Canvas/drop area
│   │   │   ├── inspector.at        # Property inspector
│   │   │   ├── toolbar.at          # Top toolbar
│   │   │   ├── node_renderer.at    # Recursive node tree
│   │   │   ├── prop_editor.at      # Type-aware property editor
│   │   │   ├── preview.at          # Live preview panel
│   │   │   └── json_panel.at       # JSON editor panel
│   │   └── lib/
│   │       ├── a2ui_bridge.at      # AURA ↔ A2UI conversion (Auto wrappers)
│   │       ├── component_registry.at # Component metadata definitions
│   │       └── types.at            # Shared type definitions
│   └── back/
│       └── api.at                  # Optional: AI generation API
```

### New Rust Modules Needed
```
crates/auto-lang/src/
├── a2ui/
│   ├── mod.rs          # Module exports
│   ├── schema.rs       # A2UI JSON types (serde)
│   ├── export.rs       # AuraWidget → A2UI JSON
│   └── import.rs       # A2UI JSON → AuraWidget
```

### Vue Generator Extensions
The existing `ui_gen/vue.rs` can already generate Vue SFCs from AURA. For the composer itself:
- The composer app IS an AutoUI app → compiles to Vue via `auto build`
- The preview panel reuses the same Vue generator for live preview
- No changes needed to the generator for Phase 1-2

### A2UI Type Definitions (Rust)
```rust
// crates/auto-lang/src/a2ui/schema.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2UISurfaceUpdate {
    pub surface_id: String,
    pub components: Vec<A2UIComponent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2UIComponent {
    pub id: String,
    #[serde(flatten)]
    pub component: A2UIComponentType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "component")]
pub enum A2UIComponentType {
    Text { text: A2UIValue },
    Button { child: A2UIValue, action: Option<A2UIAction> },
    TextInput { value: A2UIValue, hint: Option<A2UIValue> },
    // ... etc
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum A2UIValue {
    Path { path: String },
    LiteralString { literal_string: String },
    LiteralNumber { literal_number: f64 },
    LiteralBool { literal_bool: bool },
}
```

---

## 6. Risk Analysis & Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| A2UI spec changes | Medium | Version-gated schema, follow Google repo |
| Drag-and-drop in generated Vue | Medium | Use native HTML5 DnD API, wrap in Auto FFI |
| Recursive widget rendering | Low | Test with deeply nested trees |
| AI generation quality | High | Use structured output, validate JSON |
| Vue build complexity | Low | Reuse existing a2vue backend |

---

## 7. Success Criteria

1. **Phase 0**: `auto a2ui export widget.at` produces valid A2UI v0.8 JSON
2. **Phase 1**: Composer app renders three-panel layout, compiles to Vue
3. **Phase 2**: Can drag components, edit properties, see live preview
4. **Phase 3**: Export/Import A2UI JSON round-trips correctly
5. **Phase 4**: AI prompt generates valid UI, gallery has 5+ examples

---

## 8. Next Steps

1. **Approve plan** and select starting phase
2. **Phase 0**: Implement A2UI schema types and bridge
3. **Phase 1**: Scaffold composer app with basic layout
4. Iterate through phases with regular demos
