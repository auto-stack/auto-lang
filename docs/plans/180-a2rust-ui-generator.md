# Plan 180: Add Rust/AutoUI Backend to `auto gen` (a2rust-ui)

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Wire the existing `RustGenerator` into `auto gen` so `.at` widget files generate unified Rust examples runnable with `cargo run --features ui-gpui`.

**Architecture:** Follow the same pattern as `jet.rs` and `ark.rs` in auto-man — a `rust_ui.rs` module that reads `.at` files from `front/`, generates Rust Component code via `RustGenerator`, wraps it in a `main()` function, and writes a complete `.rs` example file. The `BackendType::Rust` case in `automan.rs` gen/build/run dispatch routes to this module.

**Tech Stack:** Rust, AURA pipeline, `RustGenerator`, existing auto-man CLI framework.

**Source pattern:** `crates/auto-man/src/ark.rs` (simplest existing backend module).

---

## Phase 1: Rust UI Generator Module

### Task 1: Create `crates/auto-man/src/rust_ui.rs` — Core Generation

**Files:**
- Create: `crates/auto-man/src/rust_ui.rs`
- Modify: `crates/auto-man/src/lib.rs` (add `pub mod rust_ui;`)

**Step 1: Create `rust_ui.rs` with generation function**

The module follows the `ark.rs` pattern: parse `.at` files, extract AURA widgets, generate code via `RustGenerator`, then wrap with `main()` and imports.

```rust
//! Rust/AutoUI project generation utilities
//!
//! Generates unified Rust examples from .at widget files using the AURA pipeline.
//! Output is a single .rs file with Component impl + main() that works with
//! both ICED and GPUI backends.

use std::fs;
use std::path::Path;

use colored::Colorize;
use auto_lang::ui_gen::rust::RustGenerator;
use auto_lang::ui_gen::BackendGenerator;
use auto_lang::Parser;
use auto_lang::session::CompilerSession;

use crate::AutoResult;

/// Generate Rust example from .at files in a project directory.
///
/// Reads all .at files from `front/` subdirectory, generates unified Rust code,
/// and writes to `rust/` output directory.
pub fn generate_rust_ui(project_dir: &Path, output_dir: Option<&Path>, _project: bool) -> AutoResult<()> {
    let front_dir = project_dir.join("front");
    if !front_dir.exists() {
        return Err("No front/ directory found".into());
    }

    // Determine output path
    let out = match output_dir {
        Some(o) => o.to_path_buf(),
        None => project_dir.join("rust"),
    };
    fs::create_dir_all(&out)?;

    println!("{}", "Generating Rust UI code (backend: rust-ui)".bright_cyan());

    // Collect all .at files
    let at_files: Vec<_> = collect_at_files(&front_dir)?;

    if at_files.is_empty() {
        println!("  No .at widget files found in front/");
        return Ok(());
    }

    let mut generated_components = String::new();

    for at_path in &at_files {
        let file_name = at_path.file_name().unwrap_or_default().to_string_lossy();
        println!("  {} {}", "Parsing".bright_cyan(), file_name);

        let component_code = compile_at_file(at_path)?;
        generated_components.push_str(&component_code);
        generated_components.push('\n');
    }

    // Generate complete example file with main()
    let project_name = parse_pac_name(&project_dir.join("pac.at"))
        .unwrap_or_else(|| "auto_ui_app".to_string());

    let full_code = wrap_example(&project_name, &generated_components);

    // Write output
    let output_path = out.join(format!("{}.rs", to_snake_case(&project_name)));
    fs::write(&output_path, &full_code)
        .map_err(|e| format!("Failed to write {}: {}", output_path.display(), e))?;

    println!("  {} {}", "Written".bright_green(), output_path.display());
    println!("  Run with: cargo run --features ui-gpui (or ui-iced)");

    Ok(())
}

/// Parse a single .at file to Rust Component code
fn compile_at_file(at_path: &Path) -> AutoResult<String> {
    let code = fs::read_to_string(at_path)
        .map_err(|e| format!("Failed to read {}: {}", at_path.display(), e))?;

    let session = CompilerSession::ui().with_backend("rust");
    let mut parser = Parser::from(code.as_str()).with_session(session);
    let ast = parser.parse().map_err(|e| format!("Parse error: {:?}", e))?;

    let mut generator = RustGenerator::new();
    let mut output = String::new();

    for stmt in &ast.stmts {
        if let auto_lang::ast::Stmt::WidgetDecl(widget_decl) = stmt {
            let aura_widget = auto_lang::aura::extract_widget_from_decl(widget_decl)
                .map_err(|e| e.to_string())?;
            let rust_code = generator.generate(&aura_widget)
                .map_err(|e| e.to_string())?;
            output.push_str(&rust_code);
            output.push('\n');
        }
    }

    Ok(output)
}

/// Wrap generated components in a complete example with main()
fn wrap_example(project_name: &str, components: &str) -> String {
    // Find the main widget name (first struct with "App" in the name, or first struct)
    let main_widget = extract_main_widget(components);

    format!(
r#"// Auto-generated from Auto language by a2rust-ui
// DO NOT EDIT - changes will be overwritten
//
// Run with:
//   cargo run --features ui-iced
//   cargo run --features ui-gpui

use auto_lang::ui::{{Component, View}};

{components}

fn main() -> auto_lang::ui::AppResult<()> {{
    #[cfg(feature = "ui-iced")]
    {{
        println!("Running with Iced backend");
        return auto_lang::ui::iced::run_app::<{main_widget}>();
    }}

    #[cfg(feature = "ui-gpui")]
    {{
        println!("Running with GPUI backend");
        return auto_lang::ui::gpui::run_app::<{main_widget}>("{project_name}");
    }}

    #[cfg(not(any(feature = "ui-iced", feature = "ui-gpui")))]
    {{
        Err("No backend enabled! Use --features ui-iced or ui-gpui".into())
    }}
}}
"#
    )
}

/// Extract the main widget name from generated components
fn extract_main_widget(components: &str) -> String {
    // Prefer "App" if it exists, otherwise use first struct
    if components.contains("struct App") {
        return "App".to_string();
    }
    // Find first struct name
    for line in components.lines() {
        if line.contains("pub struct ") {
            return line.split("pub struct ")
                .nth(1)
                .map(|s| s.split_whitespace().next().unwrap_or("App").to_string())
                .unwrap_or_else(|| "App".to_string());
        }
    }
    "App".to_string()
}

/// Collect .at files from a directory (non-recursive)
fn collect_at_files(dir: &Path) -> AutoResult<Vec<std::path::PathBuf>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir)
        .map_err(|e| format!("Failed to read dir {}: {}", dir.display(), e))?
    {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();
        if path.extension().map(|e| e == "at").unwrap_or(false) {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

/// Parse project name from pac.at
fn parse_pac_name(pac_path: &Path) -> Option<String> {
    let content = fs::read_to_string(pac_path).ok()?;
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("name:") {
            if let Some(colon_pos) = line.find(':') {
                let value = line[colon_pos + 1..].trim();
                let value = value.trim_matches('"').trim_matches('\'');
                return Some(value.to_string());
            }
        }
    }
    None
}

/// Convert CamelCase to snake_case
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(c.to_lowercase().next().unwrap_or(c));
    }
    result
}
```

**Step 2: Register module in `lib.rs`**

In `crates/auto-man/src/lib.rs`, add after existing module declarations:
```rust
pub mod rust_ui;
```

**Step 3: Verify compilation**

Run: `cargo build -p auto-man`
Expected: Compiles with zero errors in new file.

**Step 4: Commit**

```bash
git add crates/auto-man/src/rust_ui.rs crates/auto-man/src/lib.rs
git commit -m "feat(ui): add a2rust-ui generator module (Plan 180)"
```

---

## Phase 2: Wire into auto-man CLI

### Task 2: Add `BackendType::Rust` to gen dispatch

**Files:**
- Modify: `crates/auto-man/src/automan.rs`

**Step 1: Add Rust case to gen (workspace mode)**

In the workspace gen loop (~line 1000), add a case before the `_ =>` fallback:

```rust
BackendType::Rust => {
    println!("  Generating Rust UI (backend: rust)");
    let output_path = if frontends.len() > 1 {
        output.as_ref().map(|o| {
            std::path::PathBuf::from(o).join("rust")
        }).or_else(|| Some(member_dir.join("rust")))
    } else {
        output.as_ref().map(|o| std::path::PathBuf::from(o))
    };
    if let Some(ref out) = output_path {
        crate::rust_ui::generate_rust_ui(&member_dir, Some(out.as_path()), project)?;
    } else {
        crate::rust_ui::generate_rust_ui(&member_dir, None, project)?;
    }
}
```

**Step 2: Add Rust case to gen (non-workspace mode)**

In the non-workspace gen loop (~line 1070), add a similar `BackendType::Rust =>` case.

**Step 3: Update build dispatch**

In `build()` (~line 849), the existing `"rust"` match arm already calls `self.transpile_auto()` and `self.pac.build()`. Add a check: if the project has a `front/` dir with widgets, also run `rust_ui::generate_rust_ui()` before building.

**Step 4: Verify compilation**

Run: `cargo build -p auto-man`
Expected: Compiles with zero errors.

**Step 5: Commit**

```bash
git add crates/auto-man/src/automan.rs
git commit -m "feat(ui): wire Rust backend into auto gen/build/run (Plan 180)"
```

---

## Phase 3: Update RustGenerator imports for auto-lang

### Task 3: Fix generated import paths

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/rust.rs`

**Step 1: Update import line**

In `RustGenerator::generate_rust()`, the generated code uses:
```rust
use auto_ui::prelude::*;
```
This needs to be updated to match the new location in auto-lang:
```rust
use auto_lang::ui::{Component, View};
```

**Step 2: Verify existing tests still pass**

Run: `cargo test -p auto-lang --lib -- rust`
Expected: All existing RustGenerator tests pass (5 tests).

**Step 3: Commit**

```bash
git add crates/auto-lang/src/ui_gen/rust.rs
git commit -m "fix(ui): update RustGenerator import paths for auto-lang (Plan 180)"
```

---

## Phase 4: Test with unified-demo

### Task 4: Add "rust" backend to unified-demo pac.at

**Files:**
- Modify: `examples/unified-demo/pac.at`

**Step 1: Update backend list**

Change:
```
backend: ["vue", "jet", "arkts"]
```
To:
```
backend: ["vue", "jet", "arkts", "rust"]
```

**Step 2: Run auto gen**

```bash
cd examples/unified-demo
cargo run -p auto-man -- gen
```

Expected: Generates `rust/unified_demo.rs` with Component code from `front/app.at`.

**Step 3: Inspect generated output**

Verify the generated .rs file has:
- `use auto_lang::ui::{Component, View};`
- The `App` struct and `Component` impl
- A `main()` function with ICED/GPUI backend selection

**Step 4: Commit**

```bash
git add examples/unified-demo/pac.at
git commit -m "feat(demo): add rust backend to unified-demo (Plan 180)"
```

---

## Phase 5: Create a Simple .at Counter Example

### Task 5: Create counter widget .at source + verify end-to-end

**Files:**
- Create: `examples/rust-counter/front/counter.at`
- Create: `examples/rust-counter/pac.at`

**Step 1: Create `pac.at`**

```auto
name: "rust-counter"
version: "1.0.0"
scene: "ui"
backend: "rust"
```

**Step 2: Create `front/counter.at`**

Use the `/auto-lang-creator` skill to generate correct Auto syntax:

```auto
widget Counter {
    state {
        count int = 0
    }

    msg Msg {
        Inc
        Dec
    }

    view {
        Col {
            Button "Increment (+)" onclick: .Inc
            Text "Count: {count}"
            Button "Decrement (-)" onclick: .Dec
        }
    }
}
```

**Step 3: Run auto gen**

```bash
cd examples/rust-counter
cargo run -p auto-man -- gen
```

**Step 4: Verify generated code compiles**

Copy generated .rs to examples/ and build:
```bash
cargo build -p auto-lang --features ui-gpui
```

**Step 5: Commit**

```bash
git add examples/rust-counter/
git commit -m "feat(examples): add rust-counter .at source with a2rust-ui gen (Plan 180)"
```

---

## Phase 6: Verification

### Task 6: Full integration test

**Step 1: Test gen for all backends**

```bash
cd examples/unified-demo
cargo run -p auto-man -- gen
```
Expected: Generates vue/, jet/, arkts/, and rust/ output directories.

**Step 2: Test gen for single rust backend**

```bash
cd examples/rust-counter
cargo run -p auto-man -- gen
```
Expected: Generates rust/counter.rs.

**Step 3: Run existing auto-man tests**

```bash
cargo test -p auto-man
```
Expected: All existing tests pass, no regressions.

**Step 4: Run auto-lang lib tests**

```bash
cargo test -p auto-lang --lib
```
Expected: All tests pass including RustGenerator tests.

---

## Phase 7: Typed Tailwind-like View Styling API

The generated view code currently emits `.style(""p-4 gap-4"")` with double-quoted strings
that don't compile. The root cause: ViewBuilder's style API uses raw strings, and the
RustGenerator doesn't know the actual method names.

Solution: Add typed methods to ViewBuilder (`.p(4)`, `.gap(2)`, `.bg("white")`, `.color("gray-700")`)
that push `StyleClass` variants into the existing `style` field. Update RustGenerator to emit these.
No adapter changes needed — `GpuiStyle`/`IcedStyle` already consume `StyleClass`.

### Task 7: Add typed style methods to ViewBuilder

**Files:**
- Modify: `crates/auto-lang/src/ui/view.rs`
- Modify: `crates/auto-lang/src/ui/style/mod.rs`

**Step 1: Add `add_class` helper to `Style`**

In `crates/auto-lang/src/ui/style/mod.rs`, add after the existing `add()` method:

```rust
/// Add a style class (mutable)
pub fn add_class(&mut self, class: StyleClass) {
    self.classes.push(class);
}
```

**Step 2: Add typed methods to `ViewBuilder<M>`**

After the existing `.with_style()` method (~line 480), add:

```rust
// === Typed Tailwind-like style methods ===
// Each method pushes a StyleClass into the style field.
// Unit parameter: 1 unit = 4px (matching Tailwind convention).

pub fn p(mut self, n: u16) -> Self {
    self.style.get_or_insert_with(Style::empty).add_class(StyleClass::Padding(SizeValue::Fixed(n)));
    self
}
pub fn px(mut self, n: u16) -> Self {
    self.style.get_or_insert_with(Style::empty).add_class(StyleClass::PaddingX(SizeValue::Fixed(n)));
    self
}
pub fn py(mut self, n: u16) -> Self {
    self.style.get_or_insert_with(Style::empty).add_class(StyleClass::PaddingY(SizeValue::Fixed(n)));
    self
}
pub fn m(mut self, n: u16) -> Self {
    self.style.get_or_insert_with(Style::empty).add_class(StyleClass::Margin(SizeValue::Fixed(n)));
    self
}
pub fn gap(mut self, n: u16) -> Self {
    self.style.get_or_insert_with(Style::empty).add_class(StyleClass::Gap(SizeValue::Fixed(n)));
    self
}
pub fn w(mut self, n: u16) -> Self {
    self.style.get_or_insert_with(Style::empty).add_class(StyleClass::Width(SizeValue::Fixed(n)));
    self
}
pub fn w_full(mut self) -> Self {
    self.style.get_or_insert_with(Style::empty).add_class(StyleClass::Width(SizeValue::Full));
    self
}
pub fn h(mut self, n: u16) -> Self {
    self.style.get_or_insert_with(Style::empty).add_class(StyleClass::Height(SizeValue::Fixed(n)));
    self
}
pub fn h_full(mut self) -> Self {
    self.style.get_or_insert_with(Style::empty).add_class(StyleClass::Height(SizeValue::Full));
    self
}
pub fn bg(mut self, color: &str) -> Self {
    if let Some(c) = Color::from_name(color) {
        self.style.get_or_insert_with(Style::empty).add_class(StyleClass::BackgroundColor(c));
    }
    self
}
pub fn color(mut self, color: &str) -> Self {
    if let Some(c) = Color::from_name(color) {
        self.style.get_or_insert_with(Style::empty).add_class(StyleClass::TextColor(c));
    }
    self
}
pub fn rounded(mut self, n: u16) -> Self {
    self.style.get_or_insert_with(Style::empty).add_class(StyleClass::Rounded(SizeValue::Fixed(n)));
    self
}
pub fn flex(mut self) -> Self {
    self.style.get_or_insert_with(Style::empty).add_class(StyleClass::Flex);
    self
}
pub fn flex1(mut self) -> Self {
    self.style.get_or_insert_with(Style::empty).add_class(StyleClass::Flex1);
    self
}
pub fn items_center(mut self) -> Self {
    self.style.get_or_insert_with(Style::empty).add_class(StyleClass::ItemsCenter);
    self
}
pub fn items_start(mut self) -> Self {
    self.style.get_or_insert_with(Style::empty).add_class(StyleClass::ItemsStart);
    self
}
pub fn justify_between(mut self) -> Self {
    self.style.get_or_insert_with(Style::empty).add_class(StyleClass::JustifyBetween);
    self
}
pub fn justify_center(mut self) -> Self {
    self.style.get_or_insert_with(Style::empty).add_class(StyleClass::JustifyCenter);
    self
}
```

**Step 3: Verify compilation**

Run: `cargo build -p auto-lang --features ui-iced`

**Step 4: Commit**

```bash
git add crates/auto-lang/src/ui/view.rs crates/auto-lang/src/ui/style/mod.rs
git commit -m "feat(ui): add typed Tailwind-like style methods to ViewBuilder (Plan 180)"
```

---

### Task 8: Update RustGenerator to emit typed methods

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/rust.rs`

**Step 1: Add `tailwind_to_typed_methods` and `tailwind_class_to_method` helpers**

Add these new methods to `RustGenerator`. They convert Tailwind class strings
like `"p-4 gap-2 bg-white"` into chained method calls like `.p(4).gap(2).bg("white")`:

```rust
fn tailwind_to_typed_methods(&self, builder: &str, class_str: &str) -> String {
    let classes = class_str.trim_matches('"').trim_matches('\'');
    let mut result = builder.to_string();
    for class in classes.split_whitespace() {
        let method = self.tailwind_class_to_method(class);
        if !method.is_empty() {
            result = format!("{}{}", result, method);
        }
    }
    result
}

fn tailwind_class_to_method(&self, class: &str) -> String {
    // Spacing
    if let Some(n) = class.strip_prefix("p-") {
        if let Ok(v) = n.parse::<u16>() { return format!(".p({})", v); }
    }
    if let Some(n) = class.strip_prefix("px-") {
        if let Ok(v) = n.parse::<u16>() { return format!(".px({})", v); }
    }
    if let Some(n) = class.strip_prefix("py-") {
        if let Ok(v) = n.parse::<u16>() { return format!(".py({})", v); }
    }
    if let Some(n) = class.strip_prefix("gap-") {
        if let Ok(v) = n.parse::<u16>() { return format!(".gap({})", v); }
    }
    if let Some(n) = class.strip_prefix("m-") {
        if let Ok(v) = n.parse::<u16>() { return format!(".m({})", v); }
    }
    // Sizing
    if class == "w-full" { return ".w_full()".into(); }
    if let Some(n) = class.strip_prefix("w-") {
        if let Ok(v) = n.parse::<u16>() { return format!(".w({})", v); }
    }
    if class == "h-full" { return ".h_full()".into(); }
    if let Some(n) = class.strip_prefix("h-") {
        if let Ok(v) = n.parse::<u16>() { return format!(".h({})", v); }
    }
    // Colors
    if let Some(c) = class.strip_prefix("bg-") { return format!(".bg(\"{}\")", c); }
    if let Some(c) = class.strip_prefix("text-") {
        if !c.starts_with(|c: char| c.is_ascii_digit()) {
            return format!(".color(\"{}\")", c);
        }
    }
    // Border radius
    if class == "rounded" { return ".rounded(2)".into(); }
    if let Some(n) = class.strip_prefix("rounded-") {
        if let Ok(v) = n.parse::<u16>() { return format!(".rounded({})", v); }
    }
    // Layout
    match class {
        "flex" => ".flex()".into(),
        "flex-1" => ".flex1()".into(),
        "items-center" => ".items_center()".into(),
        "items-start" => ".items_start()".into(),
        "justify-between" => ".justify_between()".into(),
        "justify-center" => ".justify_center()".into(),
        _ => String::new(),
    }
}
```

**Step 2: Update `add_prop_to_builder` to use typed methods**

Replace the existing method. Key change: "class"/"style" keys now route through
`tailwind_to_typed_methods()` instead of emitting `.style("...")`:

```rust
fn add_prop_to_builder(&self, builder: &str, key: &str, value: &AuraPropValue) -> String {
    match value {
        AuraPropValue::Expr(expr) => {
            match key {
                "class" | "className" | "style" => {
                    self.tailwind_to_typed_methods(builder, &self.expr_to_rust(expr))
                }
                "padding" => format!("{}.p({})", builder, self.expr_to_rust(expr)),
                "spacing" | "gap" => format!("{}.gap({})", builder, self.expr_to_rust(expr)),
                "width" => format!("{}.w({})", builder, self.expr_to_rust(expr)),
                "height" => format!("{}.h({})", builder, self.expr_to_rust(expr)),
                "background" | "bg" => format!("{}.bg({})", builder, self.expr_to_rust(expr)),
                "color" | "textColor" => format!("{}.color({})", builder, self.expr_to_rust(expr)),
                "rounded" | "borderRadius" => format!("{}.rounded({})", builder, self.expr_to_rust(expr)),
                _ => builder.to_string(),
            }
        }
        AuraPropValue::StyleBinding(bindings) => {
            let _ = bindings;
            builder.to_string()
        }
    }
}
```

**Step 3: Verify tests pass**

Run: `cargo test -p auto-lang --lib -- rust`
Expected: All RustGenerator tests pass.

**Step 4: Commit**

```bash
git add crates/auto-lang/src/ui_gen/rust.rs
git commit -m "feat(ui): update RustGenerator to emit typed style methods (Plan 180)"
```

---

### Task 9: End-to-end test — re-generate and build rust-counter

**Step 1: Re-generate**

```bash
cd examples/rust-counter && rm -rf rust && auto gen
```

**Step 2: Verify generated code uses typed methods**

Check `rust/src/main.rs` — `view()` should emit `.p(4).gap(4).child(...)` instead of `.style(""p-4 gap-4"")`.

**Step 3: Build the generated project**

```bash
cd examples/rust-counter/rust && cargo build
```

Expected: Compiles with zero errors.

**Step 4: Run all tests**

```bash
cargo test -p auto-lang --lib -- ui
cargo test -p auto-man --lib
```

**Step 5: Commit**

```bash
git add examples/rust-counter/
git commit -m "test(ui): verify typed style methods in generated rust-counter (Plan 180)"
```
