# Plan 174: Conditional UI Backend Inclusion — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a `ui-headless` feature flag and headless backend to `crates/auto-lang` so default builds skip all UI deps, and developers can iterate on UI logic without compiling GPUI/ICED.

**Architecture:** Feature-gated modules within `crates/auto-lang/src/ui/`. The headless backend is a no-op renderer that implements the same `App` entry point but returns immediately with no window or event loop. Zero new external dependencies.

**Tech Stack:** Rust, Cargo features, existing `Component`/`View`/`VTree` system.

---

### Task 1: Add `ui-headless` Feature Flag to Cargo.toml

**Files:**
- Modify: `crates/auto-lang/Cargo.toml:26-29` (features section)

**Step 1: Add the new feature**

In `crates/auto-lang/Cargo.toml`, add `ui-headless` between the `ui` and `ui-interpreter` features:

```toml
# Plan 015: UI Features (AutoUI Core)
ui = []
ui-headless = ["ui"]              # NEW: No-op backend for development
ui-interpreter = ["ui"]
ui-gpui = ["ui", "dep:gpui-lib"]
ui-iced = ["ui", "dep:iced"]
```

**Step 2: Verify default build excludes UI**

Run: `cargo build -p auto-lang 2>&1 | head -5`
Expected: Compiles successfully, no ICED/GPUI deps resolved.

**Step 3: Verify headless feature compiles**

Run: `cargo build -p auto-lang --features ui-headless 2>&1 | head -5`
Expected: Compiles successfully, no new external deps pulled in.

**Step 4: Commit**

```bash
git add crates/auto-lang/Cargo.toml
git commit -m "feat(ui): add ui-headless feature flag"
```

---

### Task 2: Create Headless Backend Module

**Files:**
- Create: `crates/auto-lang/src/ui/headless/mod.rs`
- Create: `crates/auto-lang/src/ui/headless/renderer.rs`

**Step 1: Write headless module entry point**

Create `crates/auto-lang/src/ui/headless/mod.rs`:

```rust
//! Headless backend - no-op renderer for development and testing
//!
//! This backend provides the full Component/View/VTree pipeline
//! without any window creation, GPU access, or event loop.
//! Use it for fast iteration on UI logic and transpiler output.

mod renderer;

pub use renderer::HeadlessRenderer;
```

**Step 2: Write headless renderer**

Create `crates/auto-lang/src/ui/headless/renderer.rs`:

```rust
//! No-op renderer that builds VTree in memory but never displays

use crate::ui::component::Component;
use crate::ui::vnode_converter::view_to_vtree;
use crate::ui::app::AppResult;

/// Headless renderer - builds VTree in memory, no window
///
/// Use this for testing UI logic without opening windows:
///
/// ```ignore
/// use auto_lang::ui::headless::HeadlessRenderer;
///
/// let renderer = HeadlessRenderer::new();
/// let snapshot = renderer.render::<MyComponent>();
/// assert_eq!(snapshot.node_count(), 5);
/// ```
pub struct HeadlessRenderer {
    /// Track how many times render was called (useful for tests)
    render_count: usize,
}

impl HeadlessRenderer {
    pub fn new() -> Self {
        Self { render_count: 0 }
    }

    /// Render a component's view into a VTree without displaying it.
    /// Returns the VTree for assertions and inspection.
    pub fn render<C>(&mut self) -> crate::ui::vnode::VTree
    where
        C: Component + Default + 'static,
    {
        let component = C::default();
        let view = component.view();
        self.render_count += 1;
        view_to_vtree(&view)
    }

    /// Render with a pre-existing component instance.
    pub fn render_with<C>(&mut self, component: &C) -> crate::ui::vnode::VTree
    where
        C: Component + 'static,
    {
        let view = component.view();
        self.render_count += 1;
        view_to_vtree(&view)
    }

    /// Number of times render has been called.
    pub fn render_count(&self) -> usize {
        self.render_count
    }
}

impl Default for HeadlessRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Run a component headlessly (returns immediately, no event loop).
///
/// This is the headless equivalent of `App::run()`.
/// Useful for smoke-testing that a component's view() doesn't panic.
pub fn run_headless<C>()
where
    C: Component + Default + 'static,
{
    let mut renderer = HeadlessRenderer::new();
    let _tree = renderer.render::<C>();
}
```

**Step 3: Register the module in ui/mod.rs**

Add to `crates/auto-lang/src/ui/mod.rs` after the existing `#[cfg(feature = "ui-interpreter")]` blocks:

```rust
#[cfg(feature = "ui-headless")]
pub mod headless;
```

**Step 4: Verify compilation**

Run: `cargo build -p auto-lang --features ui-headless`
Expected: Compiles with zero errors and zero new deps.

**Step 5: Commit**

```bash
git add crates/auto-lang/src/ui/headless/ crates/auto-lang/src/ui/mod.rs
git commit -m "feat(ui): add headless backend module with no-op renderer"
```

---

### Task 3: Update App.rs with Headless Dispatch

**Files:**
- Modify: `crates/auto-lang/src/ui/app.rs`

**Step 1: Add headless dispatch to App::run()**

Update `crates/auto-lang/src/ui/app.rs` to add a headless branch in `App::run()`. The method should check for headless first, then real backends:

```rust
use super::Component;

/// Error type for App operations
pub type AppResult<T> = Result<T, Box<dyn std::error::Error>>;

/// Unified App entry point
pub struct App;

impl App {
    /// Run the application with the selected backend.
    ///
    /// Backend priority:
    /// 1. `ui-headless` feature → headless (no window, returns immediately)
    /// 2. `ui-iced` feature → ICED backend
    /// 3. `ui-gpui` feature → GPUI backend
    /// 4. No backend → error message
    pub fn run<C>() -> AppResult<()>
    where
        C: Component + Default + 'static,
    {
        #[cfg(feature = "ui-headless")]
        {
            super::headless::run_headless::<C>();
            return Ok(());
        }

        #[cfg(all(feature = "ui-iced", not(feature = "ui-headless")))]
        {
            return Err(
                "Please use the ICED backend crate directly. \
                 Add the appropriate ICED runner to your Cargo.toml dependencies."
                    .into(),
            );
        }

        #[cfg(all(feature = "ui-gpui", not(any(feature = "ui-headless", feature = "ui-iced"))))]
        {
            return Err(
                "Please use the GPUI backend crate directly. \
                 Add the appropriate GPUI runner to your Cargo.toml dependencies."
                    .into(),
            );
        }

        #[cfg(not(any(feature = "ui-headless", feature = "ui-iced", feature = "ui-gpui")))]
        {
            return Err(
                "No backend enabled. Enable one of: 'ui-headless', 'ui-iced', or 'ui-gpui' \
                 in your Cargo.toml features."
                    .into(),
            );
        }
    }
}
```

**Step 2: Verify compilation**

Run: `cargo build -p auto-lang --features ui-headless`
Expected: Compiles with zero errors.

**Step 3: Commit**

```bash
git add crates/auto-lang/src/ui/app.rs
git commit -m "feat(ui): add headless backend dispatch to App::run()"
```

---

### Task 4: Add Headless Style Adapter

**Files:**
- Create: `crates/auto-lang/src/ui/style/headless_adapter.rs`
- Modify: `crates/auto-lang/src/ui/style/mod.rs`

**Step 1: Write headless style adapter**

Create `crates/auto-lang/src/ui/style/headless_adapter.rs`:

```rust
//! Headless style adapter - records style classes without applying them
//!
//! Useful for testing that the correct styles are generated
//! without needing a real rendering backend.

use super::{Style, StyleClass};

/// Headless style - stores parsed classes for inspection
pub struct HeadlessStyle {
    /// The parsed style classes (for assertions)
    pub classes: Vec<StyleClass>,
    /// The original input string (for debugging)
    pub source: Option<String>,
}

impl HeadlessStyle {
    /// Create a headless style from a parsed Style
    pub fn from_style(style: &Style) -> Self {
        Self {
            classes: style.classes.clone(),
            source: None,
        }
    }

    /// Create a headless style from a raw class string
    pub fn parse(input: &str) -> Result<Self, String> {
        let style = Style::parse(input)?;
        Ok(Self {
            classes: style.classes.clone(),
            source: Some(input.to_string()),
        })
    }

    /// Check if a specific style class type is present
    pub fn has_class<F>(&self, predicate: F) -> bool
    where
        F: Fn(&StyleClass) -> bool,
    {
        self.classes.iter().any(predicate)
    }

    /// Number of style classes
    pub fn len(&self) -> usize {
        self.classes.len()
    }

    /// Whether there are no style classes
    pub fn is_empty(&self) -> bool {
        self.classes.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_headless_parse() {
        let style = HeadlessStyle::parse("p-4 gap-2 bg-white").unwrap();
        assert_eq!(style.len(), 3);
        assert!(!style.is_empty());
    }

    #[test]
    fn test_headless_has_class() {
        let style = HeadlessStyle::parse("flex items-center").unwrap();
        assert!(style.has_class(|c| matches!(c, StyleClass::Flex)));
    }

    #[test]
    fn test_headless_source_preserved() {
        let style = HeadlessStyle::parse("p-4").unwrap();
        assert_eq!(style.source.as_deref(), Some("p-4"));
    }
}
```

**Step 2: Register in style/mod.rs**

Add to `crates/auto-lang/src/ui/style/mod.rs`, after the existing `#[cfg(feature = "ui-iced")]` block:

```rust
#[cfg(feature = "ui-headless")]
pub mod headless_adapter;

#[cfg(feature = "ui-headless")]
pub use headless_adapter::HeadlessStyle;
```

**Step 3: Verify compilation**

Run: `cargo build -p auto-lang --features ui-headless`
Expected: Compiles with zero errors.

**Step 4: Run new tests**

Run: `cargo test -p auto-lang --features ui-headless -- headless`
Expected: 3 tests pass (test_headless_parse, test_headless_has_class, test_headless_source_preserved).

**Step 5: Commit**

```bash
git add crates/auto-lang/src/ui/style/headless_adapter.rs crates/auto-lang/src/ui/style/mod.rs
git commit -m "feat(ui): add headless style adapter for testing"
```

---

### Task 5: Add Integration Tests for Feature Combinations

**Files:**
- Modify: `crates/auto-lang/src/ui/headless/renderer.rs` (add tests)

**Step 1: Write headless renderer integration tests**

Add to the bottom of `crates/auto-lang/src/ui/headless/renderer.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::view::View;
    use crate::ui::component::Component;
    use std::fmt;

    #[derive(Clone, Debug)]
    enum CounterMsg {
        Increment,
        Decrement,
    }

    #[derive(Debug, Default)]
    struct Counter {
        count: i64,
    }

    impl Component for Counter {
        type Msg = CounterMsg;

        fn on(&mut self, msg: Self::Msg) {
            match msg {
                CounterMsg::Increment => self.count += 1,
                CounterMsg::Decrement => self.count -= 1,
            }
        }

        fn view(&self) -> View<Self::Msg> {
            View::col()
                .child(View::text(format!("Count: {}", self.count)))
                .child(View::button("+", CounterMsg::Increment))
                .child(View::button("-", CounterMsg::Decrement))
                .build()
        }
    }

    #[test]
    fn test_headless_render_counter() {
        let mut renderer = HeadlessRenderer::new();
        let tree = renderer.render::<Counter>();
        assert!(!tree.is_empty());
        // Root (Column) + 3 children (Text, Button, Button)
        assert_eq!(tree.node_count(), 4);
    }

    #[test]
    fn test_headless_render_count_increments() {
        let mut renderer = HeadlessRenderer::new();
        let _ = renderer.render::<Counter>();
        let _ = renderer.render::<Counter>();
        assert_eq!(renderer.render_count(), 2);
    }

    #[test]
    fn test_headless_render_with_modified_state() {
        let mut counter = Counter::default();
        counter.on(CounterMsg::Increment);
        counter.on(CounterMsg::Increment);

        let mut renderer = HeadlessRenderer::new();
        let tree = renderer.render_with(&counter);
        assert!(!tree.is_empty());
        // Verify the tree was built (component with count=2 renders correctly)
        assert_eq!(tree.node_count(), 4);
    }

    #[test]
    fn test_run_headless_no_panic() {
        // Smoke test: run_headless should not panic
        run_headless::<Counter>();
    }
}
```

**Step 2: Run headless tests**

Run: `cargo test -p auto-lang --features ui-headless -- ui::headless`
Expected: 4 tests pass.

**Step 3: Commit**

```bash
git add crates/auto-lang/src/ui/headless/renderer.rs
git commit -m "test(ui): add headless renderer integration tests"
```

---

### Task 6: Verify All Feature Combinations

**Files:** None (verification only)

**Step 1: Default build (no UI)**

Run: `cargo build -p auto-lang`
Expected: Success, no UI deps resolved.

**Step 2: UI abstract only**

Run: `cargo build -p auto-lang --features ui`
Expected: Success, no GPUI/ICED deps.

**Step 3: Headless**

Run: `cargo build -p auto-lang --features ui-headless`
Expected: Success, no GPUI/ICED deps, headless module compiled.

**Step 4: ICED backend**

Run: `cargo build -p auto-lang --features ui-iced`
Expected: Success (will take longer due to ICED deps).

**Step 5: GPUI backend**

Run: `cargo build -p auto-lang --features ui-gpui`
Expected: Success (will take longer due to GPUI deps).

**Step 6: All existing tests still pass**

Run: `cargo test -p auto-lang`
Expected: All existing tests pass (no regressions).

**Step 7: Headless tests pass**

Run: `cargo test -p auto-lang --features ui-headless`
Expected: All existing tests + new headless tests pass.

**Step 8: Commit (if any fixes needed)**

```bash
git add -A
git commit -m "fix(ui): address feature combination issues"
```
