# Plan 182: Debug Mode for Rust Desktop UI Frameworks

## Problem

Web projects have F12 DevTools. Tauri apps ship a WebInspector. Jet/Ark workflows use IDE previews. Rust desktop frameworks (GPUI, iced) have none of this. Developers must recompile and guess at layout issues.

## Solution

A `DebugLayer` that intercepts the VTree between the abstract UI tree and the backend renderer. When toggled on, it provides an in-process overlay with Chrome DevTools-inspired inspection: hover highlights, bounding-box visualization, live property editing, and source-file round-tripping.

The design is **backend-agnostic** (works on GPUI, iced, and headless) and **mode-agnostic** (works for both current transpiled mode and future AutoVM scripting mode).

---

## Architecture

### Pipeline Position

The DebugLayer sits between the VTree and the backend renderer:

```
Component → view() → View<M> → view_to_vtree() → VTree
                                                      ↓
                                                 DebugLayer
                                                 (intercepts VTree,
                                                  adds overlay,
                                                  tracks state)
                                                      ↓
                                        ┌─────────────┼─────────────┐
                                      GPUI          ICED         Headless
```

When disabled, the VTree passes through untouched — zero overhead.

### Core Types

```rust
// crates/auto-lang/src/ui/debug/mod.rs

/// Central debug controller, toggled at runtime.
pub struct DebugLayer<M> {
    enabled: bool,
    /// Maps VNodeId → layout bounds (filled by backend after layout).
    bounds: HashMap<VNodeId, Rect>,
    /// Currently hovered node.
    hovered: Option<VNodeId>,
    /// Currently selected node (clicked).
    selected: Option<VNodeId>,
    /// Right-side panel state.
    panel: DebugPanel,
    /// Source map: VNodeId → (.at file path, byte range).
    source_map: SourceMap,
    /// Pending edits (previewed but not committed).
    pending: PendingEdits,
    /// Edit sink — transpiled or VM.
    sink: Box<dyn DebugEditSink>,
    _marker: PhantomData<M>,
}

/// Layout rectangle reported by backend after render.
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub padding: EdgeInsets,
    pub margin: EdgeInsets,
}
```

### Activation

The DebugLayer is always instantiated but inert. Toggling it on triggers interception.

- Keyboard shortcut: `Ctrl+Shift+D`
- Programmatic API: `debug_layer.toggle()`
- CLI flag: `auto run --debug`

---

## Interactive Features

### State Machine

```rust
pub enum DebugState {
    /// Debug layer inactive.
    Disabled,
    /// Active, hover highlights visible, panel closed.
    InspectOnly,
    /// Panel open, full inspection.
    PanelOpen { tab: PanelTab },
    /// Drag-resizing a selected node.
    Resizing { node: VNodeId, handle: DragHandle },
    /// Editing a property inline.
    Editing { node: VNodeId, property: StyleProperty },
}

pub enum PanelTab {
    Elements,
    Console,
}
```

### Feature 1: Hover Highlight

Mouse move events are intercepted. A hit test against the `bounds` map finds the deepest VNodeId under the cursor. The overlay renders a semi-transparent blue border around the hovered node.

### Feature 2: Selection and Property Inspection

Clicking a hovered node selects it (persistent orange border). The Elements panel locks to the selected node and displays:

- Widget type (read-only)
- Computed position and size
- Box model: padding, margin
- All non-default style properties
- Source location (file and line range)

### Feature 3: Live Resize via Drag Handles

A selected node shows 8 drag handles at corners and edge midpoints. Dragging a handle updates the node's computed bounds in the DebugLayer and emits a `DebugEdit::StyleChange`.

### Feature 4: Inline Property Editing

The Elements panel renders editable fields for each style property. Changes are applied to the overlay rendering immediately (next frame) but are **not** written to source until the user commits.

Two-phase edit model:

1. **Preview**: Edit is applied to the overlay only. No files are touched.
2. **Commit**: User presses Save (or `Ctrl+S`). Edits are written back to the `.at` source file via the source map.

```rust
pub struct PendingEdits {
    edits: Vec<DebugEdit>,
    committed: bool,
}

pub enum DebugEdit {
    StyleChange {
        node: VNodeId,
        property: StyleProperty,
        old_value: StyleValue,
        new_value: StyleValue,
    },
    DeleteNode {
        node: VNodeId,
    },
    InsertNode {
        parent: VNodeId,
        index: usize,
        widget_code: String,
    },
    MoveNode {
        node: VNodeId,
        new_parent: VNodeId,
        new_index: usize,
    },
}
```

### Feature 5: Widget Tree Manipulation

The Elements tab shows the full VNode hierarchy as a collapsible tree. Right-clicking a node offers:

- **Delete**: Marks the node as deleted in the overlay. On commit, removes it from the `.at` source.
- **Insert Before/After/Child**: Opens a mini-editor for a widget expression. The new node appears in the overlay immediately and is spliced into the `.at` source on commit.

---

## Debug Panel Layout (Right-Side Docked)

The panel is docked to the right side of the window, suited for wide desktop screens. It is itself rendered as a View tree through the existing pipeline, so it works on all backends automatically.

```
┌───────────────────────────────────┬─────────────────────────┐
│                                   │ Elements │ Console       │
│                                   │─────────────────────────│
│                                   │ ▼ col.root              │
│                                   │   ▼ row.header          │
│          App content               │     text "Title"        │
│       (shrunk to make room)        │     button "Save"  ←    │
│                                   │   ▼ col.body            │
│                                   │     text "Content"      │
│                                   │─────────────────────────│
│                                   │ Styles                  │
│                                   │ ───────                 │
│                                   │ Button                  │
│  ┌─────────────────┐              │   bg: "blue"     [✎]   │
│  │  hover box      │              │   color: "white" [✎]   │
│  └─────────────────┘              │─────────────────────────│
│                                   │ Computed                │
│                                   │ ─────────               │
│                                   │  width: 120             │
│                                   │  height: 36             │
│                                   │  x: 100, y: 80         │
│                                   │─────────────────────────│
│                                   │ Layout                  │
│                                   │ ┌──── margin ────┐     │
│                                   │ │ ┌── padding ─┐ │     │
│                                   │ │ │  120 × 36  │ │     │
│                                   │ │ └────────────┘ │     │
│                                   │ └───────────────┘     │
│                                   │                         │
│                                   │ Source: app.at:42-48    │
│                                   │ [Discard] [Save source] │
└───────────────────────────────────┴─────────────────────────┘
```

**Tabs**:

- **Elements**: Widget tree on top, styles + computed + layout stacked below. All visible simultaneously — no sub-tabs.
- **Console**: Debug logs, render counts, edit history.

---

## Source Map and `.at` File Round-Tripping

### Source Map Generation

The source map is generated during the AURA extraction pipeline. A `SourceMapCollector` walks the AST in parallel with AURA extraction, preserving byte offsets that the current pipeline discards.

```
.at source file
    ↓ Parser
AST (with byte offsets on every node)
    ↓ AURA extraction (existing)
AuraWidget (with AuraNode tree)
    ↓ SourceMapCollector (NEW — walks AST in parallel)
SourceMap: AuraNodeId → (file, byte_start, byte_end)
    ↓ Code generation (existing)
Generated Rust code
```

Every `AuraNode` already corresponds to an AST node with known byte offsets. We preserve those offsets through the pipeline.

```rust
pub struct SourceMap {
    entries: HashMap<AuraNodeId, SourceLocation>,
    aura_to_vnode: HashMap<AuraNodeId, VNodeId>,
}

pub struct SourceLocation {
    pub file: PathBuf,
    pub byte_start: usize,
    pub byte_end: usize,
    pub node_kind: String,
}
```

### Edit Pipeline (on Commit)

```
DebugEdit[]
    ↓ Sort by byte_start (descending — bottom of file first)
    ↓ Apply text transformations to .at source
    ↓ Re-parse .at file (validate it still compiles)
    ↓ Re-extract AURA
    ↓ Re-generate Rust
    ↓ Prompt restart (MVP) or hot-reload (future)
```

Editing bottom-up keeps byte offsets valid for earlier edits.

### Edge Cases

| Scenario | Strategy |
|----------|----------|
| Edit breaks syntax | Roll back the `.at` file, show error in panel |
| Multiple widgets per file | Each widget's AuraNode tree has its own byte ranges |
| Shared styles (class refs) | Edit the class definition, not the inline usage |
| Generated `.rs` with no `.at` source | Fall back to editing `.rs` directly, show warning |
| File changed externally | Detect via mtime, warn user, offer reload or abort |

### Mode-Agnostic Edit Sink

```rust
pub trait DebugEditSink {
    fn apply(&self, edits: &[DebugEdit]) -> Result<(), DebugError>;
}

/// Transpiled mode: write .at file, regenerate, prompt restart.
struct TranspiledEditSink { /* source_map, file paths */ }

/// Future: Scripting mode. Patch VM state directly, instant effect.
struct VmEditSink { /* vm handle */ }
```

When the AutoVM-based renderer arrives, plugging in `VmEditSink` gives instant hot reload with no changes to the DebugLayer.

---

## Module Structure

### New Files

```
crates/auto-lang/src/ui/debug/
├── mod.rs              // DebugLayer, DebugState, DebugPanel
├── overlay.rs          // Overlay VTree generation (hover boxes, selection, drag handles)
├── inspector.rs        // Widget tree + properties panel rendering
├── hit_test.rs         // Point-in-bounds lookup
├── edit.rs             // DebugEdit enum, PendingEdits, edit pipeline
├── source_map.rs       // SourceMap, SourceLocation, AuraNodeId → .at byte mapping
└── edit_sink.rs        // DebugEditSink trait, TranspiledEditSink, (future: VmEditSink)
```

### Existing Files That Need Changes

| File | Change |
|------|--------|
| `ui/view.rs` | View variants get `source_id: Option<AuraNodeId>` |
| `ui/vnode.rs` | VNode stores `bounds: Option<Rect>` after layout |
| `ui/gpui/renderer.rs` | Report layout bounds back to DebugLayer |
| `ui/iced/renderer.rs` | Report layout bounds back to DebugLayer |
| `ui/headless/renderer.rs` | No-op or mock bounds |
| `ui/mod.rs` | Wire DebugLayer into the render pipeline |
| `ui/event_router.rs` | Route debug-mode mouse events to DebugLayer |
| `aura/extract.rs` | Preserve AST byte offsets through extraction |
| `ui_gen/rust.rs` | Emit source map alongside generated Rust code |

### Backend Integration Contract

```rust
pub trait LayoutReporter {
    /// Populate bounds for each VNodeId after the backend computes layout.
    fn report_layout(&self, bounds: &mut HashMap<VNodeId, Rect>);
}
```

Each backend hooks into its post-layout phase to extract bounds. The overlay uses the same backend's normal rendering path — no special drawing code.

---

## Future-Proofing

These features are designed out of scope for the MVP but the architecture accommodates them:

- **AutoVM scripting mode**: Plug in `VmEditSink` for instant hot reload without file writes
- **Hot reload in transpiled mode**: File watcher + re-extraction pipeline, no full restart
- **Performance profiling**: Render-cycle instrumentation in the DebugLayer intercept
- **Breakpoints on state changes**: Hook into the Component `on()` message handler
- **Event flow visualization**: Trace messages through the EventRouter
- **Undo/redo for edits**: `PendingEdits` already stores old/new values — add a stack
- **Drag-and-drop widget reordering**: `DebugEdit::MoveNode` is defined, just needs UI

---

## MVP Scope (Plan 182 Implementation)

The MVP delivers the core inspection loop. Features are phased:

### Phase 1: Foundation
- `DebugLayer` struct with toggle and state machine
- `LayoutReporter` trait + GPUI implementation
- `HitTest` module
- Hover highlight overlay

### Phase 2: Selection and Panel
- Selection with persistent highlight
- Right-side docked panel with Elements tab
- Widget tree view (collapsible)
- Styles + computed values display (read-only)

### Phase 3: Box Model and Source Map
- Visual box model diagram in panel
- `SourceMap` generation during AURA extraction
- Source location display in panel
- `source_id` field on View variants

### Phase 4: Editing
- Inline property editing (styles)
- `PendingEdits` with preview phase
- `TranspiledEditSink` — write edits to `.at` files
- Bottom-up edit application with rollback on syntax error
- Save button to commit

### Phase 5: Widget Tree Manipulation
- Right-click context menu (delete, insert)
- `DebugEdit::DeleteNode` and `InsertNode`
- Console tab with edit history

### Not in MVP
- Drag handles for resize (complex hit-testing on handles)
- AutoVM `VmEditSink`
- Hot reload in transpiled mode
- Performance profiling, breakpoints, event tracing
