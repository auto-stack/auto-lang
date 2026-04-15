# Plan 174: Conditional UI Backend Inclusion

**Date:** 2026-04-15
**Status:** Approved

## Objective

Enable conditional compilation of UI backends (GPUI, ICED) in the auto-lang workspace so that:
1. Default builds have zero UI overhead
2. A headless backend enables development velocity without heavy deps
3. Real backends are opt-in via Cargo features

## Problem

GPUI and ICED pull in large dependency trees. Building auto-lang should be fast by default, with UI backends only compiled when explicitly selected.

## Design

### Approach: Feature-gated modules (Option A)

Keep all UI code in `crates/auto-lang/src/ui/` with strengthened feature gating.

### Feature Flag Hierarchy

```
(no ui features)     → No UI code compiled at all
ui                   → Abstract types only (Component, View, VNode, Style)
ui-headless          → Abstract types + no-op backend (depends on ui)
ui-interpreter       → UI interpreter mode (depends on ui)
ui-gpui              → GPUI backend (depends on ui + gpui crate)
ui-iced              → ICED backend (depends on ui + iced crate)
```

Default features: `with-file-history` only. No UI by default.

### Headless Backend

A no-op backend that implements Component/View/App traits but renders nothing.

**What it does:**
- `App::run_headless()` returns immediately, no window
- View construction works normally (test view logic)
- Component::on() processes messages (test state updates)
- Style parsing works (verify style classes)
- VNode tree built in memory (assert on tree structure)

**What it does NOT do:**
- No window creation, no GPU, no rendering
- No event loop
- Zero external dependencies

### Module Organization

```
src/ui/
├── mod.rs                  # Gates ui-interpreter modules
├── component.rs            # [feature = "ui"]
├── view.rs                 # [feature = "ui"]
├── vnode.rs                # [feature = "ui"]
├── app.rs                  # Backend dispatch (include headless)
├── style/
│   ├── mod.rs              # Gates gpui_adapter/iced_adapter
│   ├── class.rs            # [feature = "ui"]
│   ├── parser.rs           # [feature = "ui"]
│   ├── gpui_adapter.rs     # [cfg(feature = "ui-gpui")]
│   ├── iced_adapter.rs     # [cfg(feature = "ui-iced")]
│   └── headless_adapter.rs # NEW: [cfg(feature = "ui-headless")]
├── headless/               # NEW: Headless backend module
│   ├── mod.rs              # [cfg(feature = "ui-headless")]
│   └── renderer.rs         # No-op renderer
└── ...
```

### Cargo.toml Changes

```toml
[features]
default = ["with-file-history"]  # No UI by default
ui = []
ui-headless = ["ui"]             # NEW
ui-interpreter = ["ui"]
ui-gpui = ["ui", "dep:gpui-lib"]
ui-iced = ["ui", "dep:iced"]
```

No new crate dependencies. `ui-headless` is pure Rust.

### Usage

```bash
# Development (fast, no UI deps)
cargo build                              # No UI at all
cargo build --features ui-headless       # Abstract UI + headless

# Production (real backend)
cargo build --features ui-iced           # ICED backend
cargo build --features ui-gpui           # GPUI backend
```

```rust
// Testing UI logic without rendering
#[cfg(test)]
mod tests {
    fn test_counter_view() {
        let counter = Counter::default();
        let view = counter.view();
        assert!(matches!(view, View::Column { .. }));
    }
}
```

## Success Criteria

1. `cargo build` compiles without resolving gpui/iced dependencies
2. `cargo build --features ui-headless` compiles with zero new external deps
3. `cargo build --features ui-iced` compiles with ICED backend
4. `cargo build --features ui-gpui` compiles with GPUI backend
5. Generated code is unchanged (targets abstract traits)
6. All existing tests pass with each feature combination
