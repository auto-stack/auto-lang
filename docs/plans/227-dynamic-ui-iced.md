# Plan 227: DynamicComponent with Iced Backend

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make `auto counter.at` automatically detect `widget`/`app` keywords and open an iced window for dynamic UI rendering.

**Architecture:** `run_file()` detects UI keywords via string scan, branches to UI parsing path (CompilerSession::ui), extracts AuraWidget, creates DynamicComponent, runs it in iced via `iced::application()` (lower-level API that doesn't require Default). A `Send`-safe wrapper message type bridges the non-Send `DynamicMessage` to iced's `Message: Send` requirement.

**Tech Stack:** iced 0.14, existing DynamicComponent/VmBridge/AuraViewBuilder, CompilerSession UI scenario

---

### Task 1: Wire feature flags in Cargo.toml

**Files:**
- Modify: `crates/auto-lang/Cargo.toml:35`

**Step 1: Update ui-iced feature to imply ui-interpreter**

Change line 35 from:
```toml
ui-iced = ["ui", "dep:iced"]
```
to:
```toml
ui-iced = ["ui", "dep:iced", "ui-interpreter"]
```

**Step 2: Build to verify compilation**

Run: `cargo build -p auto-lang --features "ui-iced"`
Expected: Compiles successfully (may have warnings, no errors)

**Step 3: Commit**

```bash
git add crates/auto-lang/Cargo.toml
git commit -m "feat(ui): ui-iced feature now implies ui-interpreter"
```

---

### Task 2: Add Send-safe IcedMessage wrapper

`DynamicMessage` contains `Vec<Value>` which uses `Rc<RefCell<T>>` — not `Send`. Iced requires `Message: Send`. We create a thin `Send`-safe wrapper that only carries the event name (args are always empty for click events in AuraViewBuilder).

**Files:**
- Modify: `crates/auto-lang/src/ui/iced/renderer.rs`

**Step 1: Add IcedMessage type after the imports**

After the existing imports (line 13), add:

```rust
/// Send-safe message type for iced boundary.
///
/// DynamicMessage contains Vec<Value> with Rc-based types that are not Send.
/// This wrapper carries only the event name — sufficient for all current
/// AuraViewBuilder events (onclick handlers always have empty args).
#[derive(Clone, Debug)]
pub struct IcedMessage {
    pub widget: String,
    pub event: String,
}

impl IcedMessage {
    fn from_dynamic(msg: &DynamicMessage) -> Self {
        match msg {
            DynamicMessage::Typed { widget_name, event_name, .. } => IcedMessage {
                widget: widget_name.clone(),
                event: event_name.clone(),
            },
            DynamicMessage::String(name) => IcedMessage {
                widget: String::new(),
                event: name.clone(),
            },
        }
    }

    fn to_dynamic(&self) -> DynamicMessage {
        DynamicMessage::Typed {
            widget_name: self.widget.clone(),
            event_name: self.event.clone(),
            args: vec![],
        }
    }
}

// Safety: IcedMessage contains only String fields — fully Send + Sync.
unsafe impl Send for IcedMessage {}
```

**Step 2: Add the import for DynamicMessage**

At the top of the file, add to the imports:

```rust
use crate::ui::interpreter::DynamicMessage;
```

**Step 3: Build to verify**

Run: `cargo build -p auto-lang --features "ui-iced"`
Expected: Compiles successfully

**Step 4: Commit**

```bash
git add crates/auto-lang/src/ui/iced/renderer.rs
git commit -m "feat(ui): add IcedMessage Send-safe wrapper for iced boundary"
```

---

### Task 3: Add run_dynamic_iced() function

This is the core iced entry point for DynamicComponent. Uses `iced::application()` which doesn't require `Default` — it takes a boot function that creates initial state.

**Files:**
- Modify: `crates/auto-lang/src/ui/iced/renderer.rs`

**Step 1: Add DynamicState wrapper and run function**

After the `IcedMessage` impl block (from Task 2), add:

```rust
/// Wrapper that holds DynamicComponent for iced's application API.
struct DynamicState {
    component: DynamicComponent,
}

/// Run a DynamicComponent in an iced window.
///
/// This is the entry point for `auto counter.at` UI mode.
/// It converts View<DynamicMessage> to View<IcedMessage> at the boundary
/// to satisfy iced's `Message: Send` requirement.
pub fn run_dynamic_iced(component: DynamicComponent) -> AppResult<String> {
    let title = format!("Auto - {}", component.widget_name());

    let boot = |_state: &mut DynamicState, _msg: IcedMessage| -> iced::Task<IcedMessage> {
        iced::Task::none()
    };

    let update = |state: &mut DynamicState, msg: IcedMessage| -> iced::Task<IcedMessage> {
        let dyn_msg = msg.to_dynamic();
        state.component.on(dyn_msg);
        iced::Task::none()
    };

    let view_fn = |state: &mut DynamicState| -> iced::Element<'static, IcedMessage> {
        let view = state.component.view();
        // Convert View<DynamicMessage> to View<IcedMessage> then into_iced
        let converted = convert_view_messages(view);
        converted.into_iced()
    };

    let app = iced::application(title, update, view_fn)
        .window_size(800.0, 600.0);

    app.run()?;

    Ok("UI closed".to_string())
}

/// Recursively convert View<DynamicMessage> to View<IcedMessage>.
fn convert_view_messages(view: View<DynamicMessage>) -> View<IcedMessage> {
    match view {
        View::Empty => View::Empty,
        View::Text { content, style } => View::Text { content, style },
        View::Button { label, onclick, style } => View::Button {
            label,
            onclick: IcedMessage::from_dynamic(&onclick),
            style,
        },
        View::Row { children, spacing, padding, style } => View::Row {
            children: children.into_iter().map(convert_view_messages).collect(),
            spacing, padding, style,
        },
        View::Column { children, spacing, padding, style } => View::Column {
            children: children.into_iter().map(convert_view_messages).collect(),
            spacing, padding, style,
        },
        View::Input { placeholder, value, on_change, width, password, style } => View::Input {
            placeholder, value,
            on_change: on_change.map(|m| IcedMessage::from_dynamic(&m)),
            width, password, style,
        },
        View::Checkbox { is_checked, label, on_toggle, style } => View::Checkbox {
            is_checked, label,
            on_toggle: on_toggle.map(|m| IcedMessage::from_dynamic(&m)),
            style,
        },
        View::Container { child, padding, width, height, center_x, center_y, style } => View::Container {
            child: Box::new(convert_view_messages(*child)),
            padding, width, height, center_x, center_y, style,
        },
        View::Scrollable { child, width, height, style } => View::Scrollable {
            child: Box::new(convert_view_messages(*child)),
            width, height, style,
        },
        View::Radio { label, is_selected, on_select, style } => View::Radio {
            label, is_selected,
            on_select: on_select.map(|m| IcedMessage::from_dynamic(&m)),
            style,
        },
        View::Select { options, selected_index, on_select, style } => View::Select {
            options, selected_index, style,
            on_select: on_select.map(|cb| {
                // SelectCallback needs a new one — this is complex, skip for now
                // by passing through (it will need a conversion)
                cb
            }),
        },
        // For remaining variants, use a fallback or implement as needed
        other => {
            // For variants we haven't explicitly converted, return Empty as fallback
            View::Empty
        }
    }
}
```

**Important note:** The `convert_view_messages` function has a problem with `Select::on_select` which uses a `SelectCallback` type. Let me check what that type is.

**Step 2: Check SelectCallback type**

Run: Search for `SelectCallback` in `crates/auto-lang/src/ui/view.rs`

If SelectCallback is complex or non-Clone, we'll handle it in the fallback branch for now (Radio, Select, etc. can be added later).

**Step 3: Import DynamicComponent**

Add to imports at top of file:
```rust
use crate::ui::dynamic::DynamicComponent;
```

**Step 4: Build and fix compilation errors**

Run: `cargo build -p auto-lang --features "ui-iced"`

There will likely be issues with:
- The `iced::application()` API — need to verify exact signature
- The `View` variant conversions for types we haven't matched

Fix each compilation error iteratively. The key constraint is that `IntoIcedElement` is already implemented for `View<M>` generically, so once we have `View<IcedMessage>`, the `into_iced()` call will work.

**Step 5: Commit**

```bash
git add crates/auto-lang/src/ui/iced/renderer.rs
git commit -m "feat(ui): add run_dynamic_iced() entry point for DynamicComponent"
```

---

### Task 4: Add has_ui_keywords() and run_file_dynamic_ui() to lib.rs

**Files:**
- Modify: `crates/auto-lang/src/lib.rs`

**Step 1: Add has_ui_keywords() function**

Add before `run_file()` (around line 882):

```rust
/// Quick string scan for UI keywords ("widget" or "app") at the start of a line.
/// Returns true if found, signaling that the file should be parsed with UI scenario.
/// False positives are harmless — the UI parse path will fail gracefully.
#[cfg(feature = "ui-iced")]
fn has_ui_keywords(code: &str) -> bool {
    for line in code.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("widget ") || trimmed.starts_with("app ") {
            return true;
        }
        // Also match `widget {` and `app {` (no space after)
        if trimmed.starts_with("widget{") || trimmed.starts_with("app{") {
            return true;
        }
    }
    false
}
```

**Step 2: Add run_file_dynamic_ui() function**

After `has_ui_keywords`:

```rust
/// Parse a .at file as UI scenario, extract AuraWidget, and run with iced.
#[cfg(feature = "ui-iced")]
fn run_file_dynamic_ui(code: &str) -> AutoResult<String> {
    use crate::session::CompilerSession;
    use crate::ui::dynamic::DynamicComponent;
    use crate::ui::iced::run_dynamic_iced;

    // 1. Parse with UI scenario
    let session = CompilerSession::ui();
    let mut parser = Parser::from(code).with_session(session);
    let ast = parser.parse()?;

    // 2. Extract first AuraWidget
    let mut widget = None;
    for stmt in &ast.stmts {
        if let crate::ast::Stmt::WidgetDecl(decl) = stmt {
            widget = Some(
                crate::aura::extract_widget_from_decl(decl)
                    .map_err(|e| e.to_string())?
            );
            break;
        }
    }

    let widget = widget.ok_or("No widget declaration found")?;

    // 3. Create DynamicComponent
    let comp = DynamicComponent::new(&widget)
        .map_err(|e| format!("DynamicComponent init failed: {}", e))?;

    // 4. Run iced (blocks until window closes)
    run_dynamic_iced(comp)
}
```

**Step 3: Modify run_file() to branch on UI keywords**

In `run_file()` (line 883), add the UI detection check after reading the file:

```rust
pub fn run_file(path: &str) -> AutoResult<String> {
    let code = std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

    // Plan 227: Detect UI keywords and run with iced backend
    #[cfg(feature = "ui-iced")]
    if has_ui_keywords(&code) {
        return run_file_dynamic_ui(&code);
    }

    run(&code)
}
```

**Step 4: Build to verify**

Run: `cargo build -p auto-lang --features "ui-iced"`
Expected: Compiles successfully

**Step 5: Commit**

```bash
git add crates/auto-lang/src/lib.rs
git commit -m "feat(ui): run_file auto-detects widget/app and runs iced UI"
```

---

### Task 5: Update iced/mod.rs re-exports

**Files:**
- Modify: `crates/auto-lang/src/ui/iced/mod.rs`

**Step 1: Add run_dynamic_iced to re-exports**

Change line 8 from:
```rust
pub use renderer::{IntoIcedElement, ComponentIced, run_app};
```
to:
```rust
pub use renderer::{IntoIcedElement, ComponentIced, IcedMessage, run_app, run_dynamic_iced};
```

**Step 2: Build to verify**

Run: `cargo build -p auto-lang --features "ui-iced"`
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add crates/auto-lang/src/ui/iced/mod.rs
git commit -m "feat(ui): export run_dynamic_iced from iced module"
```

---

### Task 6: End-to-end test with a widget .at file

**Files:**
- Create: `tmp/test_counter.at`

**Step 1: Create a test widget file**

```bash
mkdir -p tmp
cat > tmp/test_counter.at << 'EOF'
widget Counter {
    model {
        count int = 0
    }
    view {
        col {
            text(text: "Count: ${.count}")
            button(text: "+") {
                onclick: .Increment
            }
            button(text: "-") {
                onclick: .Decrement
            }
        }
    }
    on Increment {
        count = count + 1
    }
    on Decrement {
        count = count - 1
    }
}
EOF
```

**Step 2: Build the auto binary with ui-iced**

Run: `cargo build --features "ui-iced"`
Expected: Compiles the auto CLI with UI support

**Step 3: Run the test**

Run: `./target/debug/auto tmp/test_counter.at`
Expected: An iced window opens showing a counter with + and - buttons.
Clicking buttons should update the count.

**Step 4: If issues found, debug and fix**

Common issues:
- Parse error: The widget syntax may not match the parser's expectations. Check with `auto` CLI error output.
- VmBridge init failure: Check that state variables and handlers are extracted correctly.
- View build failure: Check AuraViewBuilder conversion for the specific AuraNode types.

**Step 5: Commit the working state**

```bash
git add -A
git commit -m "test: verify dynamic UI with iced end-to-end (counter widget)"
```

---

### Task 7: Handle compilation edge cases and add fallback

**Files:**
- Modify: `crates/auto-lang/src/lib.rs`

**Step 1: Add graceful fallback when UI parsing fails**

In `run_file_dynamic_ui`, if parsing fails with UI scenario, fall back to normal execution:

```rust
#[cfg(feature = "ui-iced")]
fn run_file_dynamic_ui(code: &str) -> AutoResult<String> {
    use crate::session::CompilerSession;
    use crate::ui::dynamic::DynamicComponent;
    use crate::ui::iced::run_dynamic_iced;

    let session = CompilerSession::ui();
    let mut parser = Parser::from(code).with_session(session);

    // If UI parsing fails, fall back to normal script execution
    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(_) => return run(code),
    };

    let mut widget = None;
    for stmt in &ast.stmts {
        if let crate::ast::Stmt::WidgetDecl(decl) = stmt {
            widget = Some(
                crate::aura::extract_widget_from_decl(decl)
                    .map_err(|e| e.to_string())?
            );
            break;
        }
    }

    let widget = match widget {
        Some(w) => w,
        None => return run(code),  // No widget found, fall back
    };

    let comp = DynamicComponent::new(&widget)
        .map_err(|e| format!("DynamicComponent init failed: {}", e))?;

    run_dynamic_iced(comp)
}
```

**Step 2: Build and test**

Run: `cargo build --features "ui-iced"`
Expected: Compiles successfully

**Step 3: Test that normal .at files still work**

Run: `./target/debug/auto tmp/hello.at` with a simple `print("hello")` script.
Expected: Prints "hello" without opening a window (no widget keyword).

**Step 4: Final commit**

```bash
git add crates/auto-lang/src/lib.rs
git commit -m "feat(ui): add fallback to script mode when UI parsing fails"
```
