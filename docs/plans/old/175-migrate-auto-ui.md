# Plan 175: Migrate auto-ui into auto-lang — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Migrate GPUI and ICED backend runners, examples, transpiler API, and CLI from the standalone `auto-ui` project into the `auto-lang` workspace.

**Architecture:** Feature-gated modules inside `crates/auto-lang/src/ui/`. GPUI at `ui/gpui/`, ICED at `ui/iced/`. Examples in `crates/auto-lang/examples/`. Transpiler API merged into `ui_gen/`. No new CLI commands — uses existing `auto gen` flow.

**Tech Stack:** Rust, Cargo features, ICED 0.14, GPUI 0.2.2, gpui-component 0.5.0, existing `IcedStyle`/`GpuiStyle` adapters.

**Source:** `d:\autostack\auto-ui`

---

## Phase 1: Backend Runners

### Task 1: Migrate ICED Backend Runner

**Files:**
- Create: `crates/auto-lang/src/ui/iced/mod.rs`
- Create: `crates/auto-lang/src/ui/iced/renderer.rs`
- Modify: `crates/auto-lang/Cargo.toml` (add naga dependency)
- Modify: `crates/auto-lang/src/ui/mod.rs` (register iced module)

**Step 1: Update Cargo.toml — add naga dependency**

The ICED backend in auto-ui uses `naga` with `termcolor` feature for Windows compatibility. Add it as an optional dependency:

In `crates/auto-lang/Cargo.toml`, after the `iced` dependency line:
```toml
naga = { version = "0.27", features = ["termcolor"], optional = true }
```

Update the `ui-iced` feature:
```toml
ui-iced = ["ui", "dep:iced", "dep:naga"]
```

**Step 2: Create `crates/auto-lang/src/ui/iced/mod.rs`**

```rust
//! ICED backend - renders View<M> using the Iced GUI framework
//!
//! This module provides the Iced backend adapter for AutoUI,
//! converting abstract View<M> trees into Iced Elements.

mod renderer;

pub use renderer::{IntoIcedElement, ComponentIced, run_app};
```

**Step 3: Create `crates/auto-lang/src/ui/iced/renderer.rs`**

Migrate from `d:\autostack\auto-ui\crates\auto-ui-iced\src\lib.rs` with these changes:

1. Replace `use auto_ui::` with `use crate::ui::`
2. Replace `use auto_ui::{View as AbstractView, Component}` with:
   ```rust
   use crate::ui::view::View as AbstractView;
   use crate::ui::component::Component;
   use crate::ui::style::iced_adapter::IcedStyle;
   use crate::ui::app::AppResult;
   ```
3. Add style support using `IcedStyle` adapter — where the original has `style: _`, apply the style:
   ```rust
   // Example for Text variant:
   AbstractView::Text { content, style } => {
       let mut text_widget = text(content);
       if let Some(s) = style {
           let iced_style = IcedStyle::from_style(&s);
           // Apply font size, weight, color from iced_style
           if let Some(size) = iced_style.font_size {
               text_widget = text_widget.size(size.value());
           }
           if let Some(color) = iced_style.text_color {
               text_widget = text_widget.style(move |_| iced::widget::text::Style {
                   color: Some(color),
                   ..Default::default()
               });
           }
       }
       text_widget.into()
   }
   ```
4. Apply similar pattern for all variants that currently have `style: _`
5. Replace `auto_ui::AppResult` with `AppResult` (from `crate::ui::app`)
6. Replace `auto_ui::{AccordionItem, SidebarPosition, ...}` with `crate::ui::view::{AccordionItem, SidebarPosition, ...}`
7. Update `run_app` to use `crate::ui::app::AppResult<()>` return type

**Step 4: Register the module in `ui/mod.rs`**

Add after the existing headless block:
```rust
#[cfg(feature = "ui-iced")]
pub mod iced;
```

**Step 5: Verify compilation**

Run: `cargo build -p auto-lang --features ui-iced`
Expected: Compiles with zero errors in new files.

**Step 6: Run ICED tests**

Run: `cargo test -p auto-lang --features ui-iced --lib -- iced`
Expected: 4 tests pass (text, button, column, checkbox conversion).

**Step 7: Commit**

```bash
git add crates/auto-lang/src/ui/iced/ crates/auto-lang/Cargo.toml crates/auto-lang/src/ui/mod.rs
git commit -m "feat(ui): migrate ICED backend runner with style support"
```

---

### Task 2: Migrate GPUI Backend Runner — Core Module

**Files:**
- Create: `crates/auto-lang/src/ui/gpui/mod.rs`
- Create: `crates/auto-lang/src/ui/gpui/renderer.rs`
- Modify: `crates/auto-lang/Cargo.toml` (add gpui-component dependency)

**Step 1: Update Cargo.toml — add gpui-component**

```toml
gpui-component = { version = "0.5.0", optional = true }
```

Update the `ui-gpui` feature:
```toml
ui-gpui = ["ui", "dep:gpui-lib", "dep:gpui-component"]
```

**Step 2: Create `crates/auto-lang/src/ui/gpui/mod.rs`**

```rust
//! GPUI backend - renders View<M> using the GPUI framework
//!
//! This module provides the GPUI backend adapter for AutoUI,
//! converting abstract View<M> trees into GPUI Elements.

mod renderer;

pub use renderer::{IntoGpuiElement, ComponentGpui, GpuiContext, GpuiMessageBridge, run_app};
```

**Step 3: Create `crates/auto-lang/src/ui/gpui/renderer.rs`**

Migrate from `d:\autostack\auto-ui\crates\auto-ui-gpui\src\lib.rs` (893 lines) with these changes:

1. Replace `use auto_ui::` with `use crate::ui::`
2. Replace `use auto_ui::{View as AbstractView, Component, Style}` with:
   ```rust
   use crate::ui::view::View as AbstractView;
   use crate::ui::component::Component;
   use crate::ui::style::Style;
   use crate::ui::app::AppResult;
   ```
3. Replace `use auto_ui::style::gpui_adapter::GpuiStyle` with `use crate::ui::style::gpui_adapter::GpuiStyle`
4. Replace `use anyhow::Error` with `Box<dyn std::error::Error>` (match AppResult pattern)
5. Update `run_app` return type to `AppResult<()>`

**Step 4: Verify compilation**

Run: `cargo build -p auto-lang --features ui-gpui`
Expected: Compiles with zero errors.

**Step 5: Commit**

```bash
git add crates/auto-lang/src/ui/gpui/renderer.rs crates/auto-lang/src/ui/gpui/mod.rs crates/auto-lang/Cargo.toml crates/auto-lang/src/ui/mod.rs
git commit -m "feat(ui): migrate GPUI backend runner (core renderer)"
```

---

### Task 3: Migrate GPUI Backend — Auto Render

**Files:**
- Create: `crates/auto-lang/src/ui/gpui/auto_render.rs`
- Modify: `crates/auto-lang/src/ui/gpui/mod.rs` (add pub mod)

**Step 1: Migrate `auto_render.rs`**

Migrate from `d:\autostack\auto-ui\crates\auto-ui-gpui\src\auto_render.rs` (1541 lines) with:
- Replace `use auto_ui::` with `use crate::ui::`
- Replace `use auto_ui::style::gpui_adapter::{GpuiStyle, GpuiFontWeight}` with `use crate::ui::style::gpui_adapter::{GpuiStyle, GpuiFontWeight}`
- Replace `use auto_ui::SelectCallback` with `use crate::ui::view::SelectCallback`

**Step 2: Register in mod.rs**

Add to `crates/auto-lang/src/ui/gpui/mod.rs`:
```rust
pub mod auto_render;
pub use auto_render::{GpuiComponentState, ViewExt};
```

**Step 3: Verify compilation**

Run: `cargo build -p auto-lang --features ui-gpui`
Expected: Compiles with zero errors.

**Step 4: Commit**

```bash
git add crates/auto-lang/src/ui/gpui/auto_render.rs crates/auto-lang/src/ui/gpui/mod.rs
git commit -m "feat(ui): migrate GPUI auto_render with component state"
```

---

### Task 4: Migrate GPUI Backend — VNode Entity

**Files:**
- Create: `crates/auto-lang/src/ui/gpui/vnode_entity.rs`
- Modify: `crates/auto-lang/src/ui/gpui/mod.rs` (add pub mod)

**Step 1: Migrate `vnode_entity.rs`**

Migrate from `d:\autostack\auto-ui\crates\auto-ui-gpui\src\vnode_entity.rs` (583 lines) with:
- Replace `use auto_ui::vnode::{VNodeId, VNodeKind, VNodeProps, VTree}` with `use crate::ui::vnode::{VNodeId, VNodeKind, VNodeProps, VTree}`
- Gate interpreter imports with `#[cfg(feature = "ui-interpreter")]`

**Step 2: Register in mod.rs**

Add to `crates/auto-lang/src/ui/gpui/mod.rs`:
```rust
pub mod vnode_entity;
pub use vnode_entity::VNodeEntity;
```

**Step 3: Verify compilation**

Run: `cargo build -p auto-lang --features ui-gpui`
Expected: Compiles with zero errors.

**Step 4: Commit**

```bash
git add crates/auto-lang/src/ui/gpui/vnode_entity.rs crates/auto-lang/src/ui/gpui/mod.rs
git commit -m "feat(ui): migrate GPUI VNode entity renderer"
```

---

### Task 5: Update lib.rs Re-exports for Backend Modules

**Files:**
- Modify: `crates/auto-lang/src/lib.rs` (add backend re-exports)

**Step 1: Add ICED and GPUI re-exports**

After the existing UI re-exports in `lib.rs`, add:
```rust
#[cfg(feature = "ui-iced")]
pub use ui::iced::{IntoIcedElement, ComponentIced};

#[cfg(feature = "ui-gpui")]
pub use ui::gpui::{IntoGpuiElement, ComponentGpui, GpuiComponentState, VNodeEntity};
```

**Step 2: Verify all feature combinations**

Run: `cargo build -p auto-lang --features ui-iced && cargo build -p auto-lang --features ui-gpui && cargo build -p auto-lang --features ui-headless && cargo build -p auto-lang`
Expected: All four compile successfully.

**Step 3: Commit**

```bash
git add crates/auto-lang/src/lib.rs
git commit -m "feat(ui): add public re-exports for ICED and GPUI backends"
```

---

### Task 6: Verify Phase 1 — All Feature Combinations

**Files:** None (verification only)

**Step 1: Default build (no UI)**
Run: `cargo build -p auto-lang`

**Step 2: Headless**
Run: `cargo build -p auto-lang --features ui-headless && cargo test -p auto-lang --features ui-headless --lib -- headless`

**Step 3: ICED backend**
Run: `cargo build -p auto-lang --features ui-iced && cargo test -p auto-lang --features ui-iced --lib -- iced`

**Step 4: GPUI backend**
Run: `cargo build -p auto-lang --features ui-gpui`

**Step 5: All existing tests**
Run: `cargo test -p auto-lang --lib`
Expected: All 2695+ existing tests pass (no regressions).

---

## Phase 2: Examples (Outline)

### Task 7: Create Example Template + First Example (ui-counter)

- Create `crates/auto-lang/examples/ui_counter.rs` with Component + backend selection pattern
- Add `[[example]]` to Cargo.toml
- Verify: `cargo run -p auto-lang --example ui-counter --features ui-iced`

### Task 8: Migrate Remaining 19 Unified Examples

Batch migrate: ui-hello, ui-button, ui-checkbox, ui-input, ui-layout, ui-list, ui-table, ui-slider, ui-progress, ui-select, ui-radio, ui-scroll, ui-container, ui-accordion, ui-sidebar, ui-tabs, ui-navigation-rail, ui-gallery, ui-todo

Source: `d:\autostack\auto-ui\examples\unified-*\src\main.rs`

## Phase 3: Transpiler Merge (Outline)

### Task 9: Migrate CodeSink Utility

- From `auto-ui/crates/auto-ui/src/trans/mod.rs` → `crates/auto-lang/src/ui_gen/code_sink.rs`
- `CodeSink` struct with import management and indentation

### Task 10: Migrate Transpiler API Functions

- From `auto-ui/crates/auto-ui/src/trans/api.rs` → `crates/auto-lang/src/ui_gen/api.rs`
- `transpile_aura()`, `transpile_vue_aura()`, `transpile_file()`
- Update import paths from `auto_ui::` to `crate::`

### Task 11: Merge TranspileError into Error System

- From `auto-ui/crates/auto-ui/src/cli/error.rs`
- Merge `TranspileError` into `crates/auto-lang/src/error.rs` or a new `ui_gen/error.rs`

## Phase 4: CLI Integration (Outline)

### Task 12: Wire transpile functions into auto gen

- Ensure `auto gen` command can call `transpile_aura()` for UI widget files
- No new CLI commands needed — uses existing `auto gen` flow
