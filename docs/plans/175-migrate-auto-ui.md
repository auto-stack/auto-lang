# Plan 175: Migrate auto-ui into auto-lang Workspace

**Date:** 2026-04-15
**Status:** Approved
**Depends on:** Plan 174 (Conditional UI Backend Inclusion)

## Objective

Migrate the remaining auto-ui code (backend runners, examples, transpiler API, CLI) into the auto-lang workspace, making auto-lang the single canonical home for all UI code.

## Source: `d:\autostack\auto-ui`

## Phase 1: Backend Runners

### Approach

GPUI and ICED backend runners go inside `crates/auto-lang/src/ui/` as feature-gated modules, extending the pattern from Plan 174.

### Module Layout

```
src/ui/
├── gpui/                    # [cfg(feature = "ui-gpui")]
│   ├── mod.rs               # Public API: run_app(), IntoGpuiElement, ComponentGpui
│   ├── renderer.rs          # View<M> → GPUI element conversion
│   ├── auto_render.rs       # GpuiComponentState, apply_style_to_div/button
│   └── vnode_entity.rs      # VNode → GPUI Entity rendering
├── iced/                    # [cfg(feature = "ui-iced")]
│   ├── mod.rs               # Public API: run_app(), IntoIcedElement, ComponentIced
│   └── renderer.rs          # View<M> → ICED element conversion (with style support)
├── headless/                # Already done (Plan 174)
└── ...
```

### Source Files → Destination

| auto-ui source | auto-lang destination | Lines |
|---|---|---|
| `auto-ui-gpui/src/lib.rs` | `ui/gpui/renderer.rs` | 893 |
| `auto-ui-gpui/src/auto_render.rs` | `ui/gpui/auto_render.rs` | 1541 |
| `auto-ui-gpui/src/vnode_entity.rs` | `ui/gpui/vnode_entity.rs` | 583 |
| `auto-ui-gpui/src/interpreter_component.rs` | `ui/gpui/interpreter_component.rs` | — |
| `auto-ui-iced/src/lib.rs` | `ui/iced/renderer.rs` | 574 |

### Key Adaptation

- Update import paths from `auto_ui::` to `crate::ui::`
- ICED runner: integrate `IcedStyle` adapter for unified style support (currently ignored)
- GPUI runner: already uses `GpuiStyle` adapter, mostly path updates

### Public API (preserved)

```rust
// GPUI backend
pub fn run_app<C>(title: &str) -> AppResult<()>
pub trait IntoGpuiElement<M> { fn into_gpui<F>(self, handle_msg: F) -> AnyElement }

// ICED backend
pub fn run_app<C>() -> AppResult<()>
pub trait IntoIcedElement<M> { fn into_iced(self) -> iced::Element<'static, M> }
```

### Cargo.toml Updates

Add `gpui-component` dependency (optional):
```toml
gpui-component = { version = "0.5.0", optional = true }

[features]
ui-gpui = ["ui", "dep:gpui-lib", "dep:gpui-component"]
```

## Phase 2: Examples

### Approach

Migrate 20 unified examples into `crates/auto-lang/examples/` with feature-gated backend selection.

### Example Layout

```
crates/auto-lang/examples/
├── ui_hello.rs
├── ui_counter.rs
├── ui_button.rs
├── ui_slider.rs
├── ui_layout.rs
├── ui_todo.rs
├── ui_table.rs
├── ui_gallery.rs
└── ... (12 more)
```

### Pattern

```rust
use auto_lang::ui::{Component, View};

struct Counter { count: i64 }
enum Msg { Inc, Dec }
impl Component for Counter { /* ... */ }

fn main() -> auto_lang::AppResult<()> {
    #[cfg(feature = "ui-iced")]
    return auto_lang::ui::iced::run_app::<Counter>();

    #[cfg(feature = "ui-gpui")]
    return auto_lang::ui::gpui::run_app::<Counter>("Counter");

    Err("No backend. Use --features ui-iced or ui-gpui".into())
}
```

### Run

```bash
cargo run -p auto-lang --example ui-counter --features ui-iced
cargo run -p auto-lang --example ui-counter --features ui-gpui
```

### Scope

Migrate only unified examples. Skip backend-specific examples (gpui-examples/, iced-examples/) and interpreter demos.

## Phase 3: Transpiler Merge

### Approach

Migrate only the active AURA-based transpiler code. Skip deprecated modules.

### What to Migrate

| auto-ui source | What | auto-lang destination |
|---|---|---|
| `trans/api.rs` | `transpile_aura()`, `transpile_vue_aura()`, `transpile_file()` | `ui_gen/api.rs` or merge into existing |
| `trans/mod.rs` | `CodeSink` utility | `ui_gen/mod.rs` |
| `cli/error.rs` | `TranspileError` with miette | Merge into existing error system |

### What to Skip (deprecated)

- `trans/rust_gen.rs` — replaced by `ui_gen/rust.rs` RustGenerator
- `trans/vue_gen.rs` — replaced by `ui_gen/vue.rs` VueGenerator
- `trans/dsl_preprocess.rs` — replaced by AURA pipeline
- `trans/auto_ui_trans.rs` — replaced by AURA pipeline

## Phase 4: CLI Integration

### Approach

No new subcommands. The transpile API functions from Phase 3 integrate into the existing `auto gen` pipeline.

### Flow

```bash
auto gen my_widget.at          # Generates Rust code (uses transpile_aura internally)
auto gen my_widget.at --vue    # Generates Vue code (uses transpile_vue_aura internally)
auto build                      # Builds with selected backend
auto run                        # Runs the built project
auto file.at                    # Runs .at script directly (interpreter)
```

### What Changes

- `auto gen` command calls `transpile_aura()` or `transpile_vue_aura()` when processing UI widgets
- Error types from `cli/error.rs` merge into auto-lang's `error.rs`

## Success Criteria

1. `cargo build` — no UI deps, fast
2. `cargo build --features ui-headless` — headless works
3. `cargo build --features ui-iced` — ICED backend compiles with style support
4. `cargo build --features ui-gpui` — GPUI backend compiles
5. `cargo run -p auto-lang --example ui-counter --features ui-iced` — example runs
6. `auto gen widget.at` — generates Rust code from AURA widget
7. All existing tests pass with each feature combination
