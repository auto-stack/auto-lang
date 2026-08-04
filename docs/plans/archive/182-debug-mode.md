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

---

## Implementation Plan

### Module Structure

```
crates/auto-lang/src/ui/debug/
├── mod.rs              # DebugLayer, DebugState, DebugPanel, Rect, BoxModel, EdgeInsets, LayoutReporter
├── hit_test.rs         # hit_test() — point-in-bounds lookup
├── inspector.rs        # NodeInfo, inspect_node()
├── overlay.rs          # OverlayInfo, OverlayRect, OverlayColor, generate_overlay()
├── source_map.rs       # SourceLocation, SourceMap
└── edit_sink.rs        # DebugEditSink trait, DebugError, DebugEdit (stub)
```

Existing file that needs changes:

| File | Change |
|------|--------|
| `ui/mod.rs` | `pub mod debug;` + re-exports |

---

## Phase 1: Foundation (DONE)

Commit: `c0920464`

### Task 1: Create module structure and core types

**Files:**
- Create: `crates/auto-lang/src/ui/debug/mod.rs`
- Create: `crates/auto-lang/src/ui/debug/hit_test.rs`
- Create: `crates/auto-lang/src/ui/debug/edit_sink.rs`
- Modify: `crates/auto-lang/src/ui/mod.rs`

**Step 1:** Create `mod.rs` with core types:

- `Rect` — layout rectangle (x, y, width, height) with `contains(px, py)` method
- `DebugState` — enum: `Disabled`, `InspectOnly`, `PanelOpen`
- `DebugLayer` — central controller with:
  - `enabled: bool` — runtime toggle
  - `hovered: Option<VNodeId>` — currently hovered node
  - `selected: Option<VNodeId>` — clicked/selected node
  - `state: DebugState` — current debug state
  - `bounds: HashMap<VNodeId, Rect>` — layout bounds from backend
  - Methods: `toggle()`, `enable()`, `disable()`, `is_enabled()`
  - Methods: `set_bounds()`, `hover()`, `select_hovered()`, `deselect()`

- `LayoutReporter` trait — backends implement this to report layout bounds after render

**Step 2:** Create `hit_test.rs`:

- `hit_test(px, py, bounds) -> Option<VNodeId>` — linear scan O(n), returns smallest-area node containing the point. Uses existing `VNodeId(u64)` from `ui/vnode.rs`.

**Step 3:** Create `edit_sink.rs` (stub for Phase 4):

- `DebugError` struct with message field and Display impl
- `DebugEdit` enum with placeholder `_Phase4Stub` variant
- `DebugEditSink` trait with `apply(&self, edits: &[DebugEdit]) -> Result<(), DebugError>` signature

**Step 4:** Wire into `ui/mod.rs`:

- Add `pub mod debug;`
- Re-export: `DebugLayer`, `DebugState`, `Rect`, `LayoutReporter`

**Step 5:** Verify

```bash
cargo build -p auto-lang
cargo test -p auto-lang --lib --features ui -- debug::
```

Expected: Compiles clean, all tests pass.

---

## Phase 2: Selection and Panel (DONE)

Commit: `1e2035dd`

### Task 2: Add inspector and overlay modules

**Files:**
- Create: `crates/auto-lang/src/ui/debug/inspector.rs`
- Create: `crates/auto-lang/src/ui/debug/overlay.rs`
- Modify: `crates/auto-lang/src/ui/debug/mod.rs`

**Step 1:** Create `inspector.rs`:

- `NodeInfo` struct: `id: VNodeId`, `widget_type: VNodeKind`, `bounds: Rect`, `styles: HashMap<String, String>`
- `inspect_node(id, widget_type, bounds, styles) -> NodeInfo` — factory function
- `NodeInfo::render_info(&self) -> String` — formatted display with widget type, bounds, sorted style properties

Uses existing `VNodeKind` enum from `ui/vnode.rs` (Column, Row, Text, Button, etc.).

**Step 2:** Create `overlay.rs`:

- `OverlayColor` enum: `Hover` (blue), `Selection` (orange)
- `OverlayRect` struct: `id: VNodeId`, `bounds: Rect`, `color: OverlayColor`
- `OverlayInfo` struct: `hovered: Option<OverlayRect>`, `selected: Option<OverlayRect>`
- `generate_overlay(hovered_id, selected_id, bounds) -> OverlayInfo` — pure function, assembles overlay data from current hover/selection state and the bounds map

**Step 3:** Add `DebugPanel` to `mod.rs`:

- `DebugPanel` struct: `info: Option<NodeInfo>`, `box_model: Option<BoxModel>`, `source: Option<SourceLocation>`
- Methods: `set_selection(info, box_model, source)`, `clear()`, `has_selection()`, `render_info() -> String`
- Update `DebugLayer` to hold `panel: DebugPanel`
- Update `select_hovered()` to populate the panel with inspected node data
- Update `deselect()` and `toggle()` to clear panel

**Step 4:** Add convenience method `DebugLayer::overlay() -> OverlayInfo` that calls `generate_overlay()` with current hover/selection state.

**Step 5:** Verify

```bash
cargo test -p auto-lang --lib --features ui -- debug::
```

---

## Phase 3: Box Model and Source Map (DONE)

Commit: `1e2035dd`

### Task 3: Add box model display and source map

**Files:**
- Create: `crates/auto-lang/src/ui/debug/source_map.rs`
- Modify: `crates/auto-lang/src/ui/debug/mod.rs`

**Step 1:** Add `EdgeInsets` to `mod.rs`:

- `EdgeInsets` struct: `top: f32`, `right: f32`, `bottom: f32`, `left: f32`
- Constructors: `uniform(v)`, `symmetric(v_h, v_v)`, `only(top, right, bottom, left)`
- `is_zero() -> bool`
- `horizontal() -> f32`, `vertical() -> f32`

**Step 2:** Add `BoxModel` to `mod.rs`:

- `BoxModel` struct: `content: Rect`, `padding: EdgeInsets`, `margin: EdgeInsets`
- `from_bounds(content, padding, margin) -> BoxModel`
- `padding_box() -> Rect` — content + padding
- `margin_box() -> Rect` — content + padding + margin
- `render() -> String` — formatted box model display

**Step 3:** Update `DebugPanel::render_info()` to include box model and source location sections.

**Step 4:** Create `source_map.rs`:

- `SourceLocation` struct: `file: PathBuf`, `line_start: usize`, `line_end: usize` (1-based, inclusive)
- `SourceLocation::new(file, line_start, line_end) -> Self`
- `Display` impl: single-line format `"file.at:42"` or multi-line `"file.at:42-48"`
- `SourceMap` struct: `entries: HashMap<VNodeId, SourceLocation>`
- Methods: `new()`, `add_mapping()`, `get_location()`, `remove_mapping()`, `len()`, `is_empty()`, `clear()`
- Initially empty — populated in future work when AURA extraction preserves AST byte offsets

**Step 5:** Update `DebugLayer` to hold `source_map: SourceMap`.

**Step 6:** Verify

```bash
cargo test -p auto-lang --lib --features ui -- debug::
```

Expected: 55 total tests across all debug modules, all passing.

---

## Phase 4: Editing (NOT YET IMPLEMENTED)

Prerequisites: source_id field on View variants, AURA extraction preserving byte offsets.

### Task 4: Implement DebugEdit and PendingEdits

**Files:**
- Modify: `crates/auto-lang/src/ui/debug/edit_sink.rs`
- Modify: `crates/auto-lang/src/ui/debug/mod.rs`

**Step 1:** Expand `DebugEdit` enum in `edit_sink.rs`:

```rust
pub enum DebugEdit {
    StyleChange {
        node: VNodeId,
        property: String,
        old_value: String,
        new_value: String,
    },
    DeleteNode { node: VNodeId },
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

**Step 2:** Add `PendingEdits` struct to `mod.rs`:

- `edits: Vec<DebugEdit>` — list of uncommitted edits
- `add(edit)` — add an edit to the pending list
- `preview()` — apply edits to overlay rendering only (no files touched)
- `commit()` — write edits to .at source files via `DebugEditSink`
- `discard()` — clear all pending edits
- `is_empty() -> bool`

**Step 3:** Implement `TranspiledEditSink` in `edit_sink.rs`:

- Holds source map and file paths
- `apply(edits)` — sort edits by byte_start (descending), apply text transformations to .at source, re-parse to validate, re-generate Rust

**Step 4:** Add inline property editing to `DebugLayer`:

- `begin_edit(node, property, new_value)` — creates a `DebugEdit::StyleChange`
- Preview immediately in overlay
- `save_edits()` — commit via TranspiledEditSink
- `discard_edits()` — clear pending

**Step 5:** Add keyboard shortcut support:

- `Ctrl+S` — save pending edits
- `Escape` — discard pending edits

**Step 6:** Verify

```bash
cargo test -p auto-lang --lib --features ui -- debug::edit
```

### Task 5: Wire DebugEdit into panel

**Step 6:** Update `DebugPanel` to show editable fields for style properties.

- Each style property rendered as editable input
- Changes create `PendingEdits`
- Panel shows "Discard" and "Save source" buttons when edits are pending

---

## Phase 5: Widget Tree Manipulation (NOT YET IMPLEMENTED)

Prerequisites: Phase 4 complete.

### Task 6: Add context menu and tree operations

**Files:**
- Modify: `crates/auto-lang/src/ui/debug/mod.rs`
- Modify: `crates/auto-lang/src/ui/debug/inspector.rs`

**Step 1:** Add widget tree view to inspector:

- `WidgetTreeNode` struct: `id`, `kind`, `children: Vec<WidgetTreeNode>`, `expanded: bool`
- `build_widget_tree(vtree) -> WidgetTreeNode` — construct from VTree
- Collapsible tree rendering in Elements tab

**Step 2:** Add context menu actions:

- Right-click on a node shows: Delete, Insert Before, Insert After, Insert Child
- Delete → `DebugEdit::DeleteNode`
- Insert → opens mini-editor for widget expression → `DebugEdit::InsertNode`
- Move → `DebugEdit::MoveNode` (drag reorder)

**Step 3:** Add Console tab to `DebugPanel`:

- `ConsoleEntry` struct: `timestamp`, `message`, `entry_type` (Info, Edit, Error)
- Edit history displayed as console entries
- Errors from failed commits shown in red

**Step 4:** Verify

```bash
cargo test -p auto-lang --lib --features ui -- debug::
```

---

## Integration with Backend Renderers (Future)

After Phases 1-5, the debug layer needs integration with actual backend renderers:

| Backend | Integration Point |
|---------|------------------|
| GPUI | Post-layout: call `LayoutReporter::report_layout()` to populate bounds |
| Iced | Post-layout: extract bounds from iced's layout tree |
| Headless | No-op or mock bounds for testing |

### Task 7: GPUI LayoutReporter

**Files:**
- Modify: `crates/auto-lang/src/ui/gpui/` (or wherever GPUI renderer lives)

Implement `LayoutReporter` for GPUI backend. After GPUI computes layout, extract bounding rectangles for each VNode and populate the DebugLayer's bounds map.

### Task 8: Iced LayoutReporter

**Files:**
- Modify: `crates/auto-lang/src/ui/iced/` (or wherever Iced renderer lives)

Same pattern as GPUI — extract layout bounds after Iced's layout pass.

### Task 9: Wire DebugLayer into render pipeline

**Files:**
- Modify: `crates/auto-lang/src/ui/mod.rs`

Insert DebugLayer between VTree and backend renderer. When enabled, intercept VTree, add overlay nodes, forward to backend. When disabled, pass through untouched.

### Task 10: Add Ctrl+Shift+D keyboard shortcut

Wire the keyboard shortcut to toggle `DebugLayer::toggle()`.

---

## Summary

| Phase | Tasks | Status | Tests |
|-------|-------|--------|-------|
| Phase 1: Foundation | Task 1 | DONE (c0920464) | 15 |
| Phase 2: Selection + Panel | Task 2 | DONE (1e2035dd) | 26 |
| Phase 3: Box Model + Source Map | Task 3 | DONE (1e2035dd) | 55 total |
| Phase 4: Editing | Tasks 4-5 | NOT DONE | — |
| Phase 5: Widget Tree | Task 6 | NOT DONE | — |
| Backend Integration | Tasks 7-10 | NOT DONE | — |
