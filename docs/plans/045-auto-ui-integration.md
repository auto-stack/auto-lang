# AutoLang ↔ AutoUI Integration Plan

## Objective

Enable AutoLang to parse/evaluate UI scripts (using `EvalMode::Config`) and send results to auto-ui for generating UI code. Support **both** runtime interpretation (for development/hot-reload) and transpilation to Rust (for production).

## Current State

### AutoLang (d:\autostack\auto-lang)
- ✅ `EvalMode::CONFIG` working - converts config syntax to hierarchical `Value::Node`
- ✅ Public APIs: `eval_config()`, `AutoConfig`, `AutoConfigReader`
- ✅ Full parser, evaluator, and transpiler infrastructure
- ✅ Node/Value system for hierarchical data

### AutoUI (d:\autostack\auto-ui)
- ✅ Phase 1-3 complete: workspace, Component trait, Iced backend
- ✅ Working examples: counter, todo, temp_converter, etc.
- ❌ **Missing**: AutoLang transpiler integration
- ❌ **Missing**: Runtime evaluation mode
- 📋 Existing analysis: [hello-at-transpilation-errors.md](../auto-ui/docs/analysis/hello-at-transpilation-errors.md)

### AutoUI Syntax Example (hello.at)
```auto
use auto.ui: View, widget, app, center, text

widget Hello {
    msg str

    fn view() View {
        text(msg) {}
    }
}

app CounterExample {
    center {
        Hello("Hello, World!")
    }
}
```

## Challenge Analysis

The current AutoLang parser doesn't recognize auto-ui specific syntax:
- `widget Hello` → Should expand to `type Hello is Widget` (macro-based)
- `app` → Normal type, instantiated with node syntax
- `fn view() View` → Component trait method
- `fn on(ev Msg)` → Message handler
- Layout syntax: `center { }`, `col { }`, `row { }`

**Key Design Decisions**:
1. **No new keywords**: Use existing AutoLang syntax with macro expansion
2. **`widget` as macro**: `widget Hello { ... }` expands to `type Hello is Widget { ... }`
3. **`app` as type**: Declared as normal type, e.g., `type CounterExample is App`
4. **Node instancing**: Use existing node instantiation syntax for creating UI trees

## Solution: Dual-Approach Integration

### Approach 1: Runtime Interpretation (Development)

**Goal**: Parse .at files at runtime, return `Node` structure, auto-ui converts to `View<M>`

**Flow**:
```
.at file → AutoLang parser → AST → EvalMode::CONFIG → Node → View<M>
```

**Implementation**:
1. Create `widget` macro (text-level or AST-level)
2. Define Widget/App traits in stdlib
3. Use `EvalMode::CONFIG` (no new mode needed)
4. Create `UINode` wrapper around `Value::Node`
5. Implement `Node → View<M>` conversion in auto-ui

**Pros**:
- ✅ Hot-reload support
- ✅ Fast iteration
- ✅ No compilation step

**Cons**:
- ❌ Runtime overhead
- ❌ No type checking until runtime

### Approach 2: Transpilation (Production)

**Goal**: Transpile .at to Rust code implementing Component trait

**Flow**:
```
.at file → AutoLang parser → AST → Transpiler → Rust code → Compile
```

**Implementation**:
1. Create `auto-ui-transpiler` crate
2. AST visitor for widget definitions
3. Code generation: struct, impl Component, main()
4. Integration with build.rs

**Pros**:
- ✅ Type safety
- ✅ Performance
- ✅ IDE support

**Cons**:
- ❌ No hot-reload
- ❌ Longer build times

## Implementation Plan

### Phase 1: AutoLang UI Macro System (Week 1)

**File**: `crates/auto-lang/src/macro.rs` (new) or extend existing macro system

#### 1.1 Define `widget` Macro

Create a macro that expands `widget` declarations to `type ... is Widget`:

**Input**:
```auto
widget Hello {
    msg str

    fn view() View {
        text(msg) {}
    }
}
```

**Expansion**:
```auto
type Hello is Widget {
    msg str

    #[vm]
    fn view() View {
        text(msg) {}
    }
}
```

**Implementation Options**:

**Option A: AST-Level Macro (Recommended)**
```rust
// In crates/auto-lang/src/macro/ui.rs (new)
pub fn expand_widget_macro(input: &ast::Stmt) -> ast::Stmt {
    // Parse: widget Name { ... }
    // Generate: type Name is Widget { ... }
    // - Add "is Widget" trait constraint
    // - Add #[vm] annotations to methods
    // - Return expanded type definition
}
```

**Option B: Text-Level Preprocessing**
```rust
// Simple string replacement before parsing
fn preprocess_widget(code: &str) -> String {
    // widget Hello { → type Hello is Widget {
    code.replace("widget ", "type $1 is Widget")
}
```

**Decision**: Start with Option B for simplicity, move to Option A if needed.

#### 1.2 Define Widget Trait in AutoLang

**File**: `stdlib/auto/widget.at` (new)

```auto
// Widget trait - all UI widgets must implement this
type Widget {
    #[vm]
    fn view() View

    #[vm]
    fn on(ev Msg) {
        // Default: no-op
    }
}
```

#### 1.3 Define App Type

**File**: `stdlib/auto/app.at` (new)

```auto
// App type - application entry point
type App is Widget {
    title str

    #[vm]
    fn run() {
        // Entry point for running the app
    }
}
```

#### 1.4 No EvalMode::UI Needed

Since we're using macros and existing type system, **no new EvalMode is required**:
- Use `EvalMode::CONFIG` for UI scripts
- Widget types are normal types with `is Widget` constraint
- App instances are normal node instantiations

#### 1.5 Update Syntax Examples

**Before** (old syntax):
```auto
widget Hello { ... }
app CounterExample { ... }
```

**After** (new syntax):
```auto
// Use widget macro
widget Hello {
    msg str
    fn view() View { ... }
}

// App is just a type
type CounterExample is App {
    fn run() {
        center {
            Hello("Hello, World!")
        }
    }
}

// Instantiate with node syntax
CounterExample {
    title: "My App"
}
```

**Critical Files**:
- [crates/auto-lang/src/macro.rs](../auto-lang/crates/auto-lang/src/macro.rs) (or create macro/ui.rs)
- [stdlib/auto/widget.at](../auto-lang/stdlib/auto/widget.at) (new)
- [stdlib/auto/app.at](../auto-lang/stdlib/auto/app.at) (new)
- No parser changes needed!

### Phase 2: Node → View<M> Conversion (Week 1-2)

**File**: `crates/auto-ui/src/node_converter.rs` (new)

#### 2.1 Define UI Node Structure
```rust
pub struct UINode {
    pub kind: UINodeKind,
    pub props: IndexMap<String, Value>,
    pub children: Vec<UINode>,
}

pub enum UINodeKind {
    Widget,
    App,
    Layout,  // center, col, row
    Element, // text, button, etc.
}
```

#### 2.2 Implement Node → View<M>
```rust
impl<M: Clone + Debug + 'static> UINode {
    pub fn to_view(&self) -> View<M> {
        match &self.kind {
            UINodeKind::Element => self.element_to_view(),
            UINodeKind::Layout => self.layout_to_view(),
            // ...
        }
    }
}
```

#### 2.3 Add Public API to auto-ui
**File**: `crates/auto-ui/src/lib.rs`

```rust
pub fn from_auto_lang(code: &str) -> AutoResult<Box<dyn Component>> {
    // Use auto_lang::eval_ui()
    // Convert Node → View<M>
    // Return Component
}
```

### Phase 3: Transpiler Implementation (Week 2-3)

**File**: `crates/auto-ui-transpiler/src/lib.rs` (new crate)

#### 3.1 Create auto-ui-transpiler Crate
```toml
[dependencies]
auto-lang = { path = "../../auto-lang/crates/auto-lang" }
auto-ui = { path = "../auto-ui" }
```

#### 3.2 AST Visitor for Widgets
```rust
pub struct UITranspiler {
    widgets: Vec<WidgetDefinition>,
}

pub struct WidgetDefinition {
    pub name: String,
    pub fields: Vec<(String, String)>,
    pub view_method: Option<ViewMethod>,
    pub on_method: Option<OnMethod>,
}
```

#### 3.3 Code Generation
**Template**:
```rust
pub fn transpile_widget(w: &WidgetDefinition) -> String {
    format!(
        r#"
#[derive(Debug)]
pub struct {name} {{
    {fields}
}}

impl Component for {name} {{
    type Msg = {msg_type};

    fn on(&mut self, msg: Self::Msg) {{
        {on_method_body}
    }}

    fn view(&self) -> View<Self::Msg> {{
        {view_method_body}
    }}
}}
"#,
        // ... substitutions
    )
}
```

#### 3.4 Integration with build.rs
```rust
// In build.rs
fn main() {
    let at_files = glob("src/ui/**/*.at").unwrap();
    for at_file in at_files {
        auto_ui_transpiler::transpile_file(at_file)?;
    }
}
```

### Phase 4: Hot-Reload Support (Week 3)

**File**: `crates/auto-ui/src/hot_reload.rs` (new)

#### 4.1 File Watcher
```rust
pub struct UIWatcher {
    pub tx: mpsc::Sender<UIUpdate>,
}

pub enum UIUpdate {
    Reload(String),  // path
    Error(String),
}
```

#### 4.2 Runtime Reload Logic
```rust
impl<M: Component> UIWatcher {
    pub fn reload_component(&mut self, path: &str) -> AutoResult<Box<dyn Component>> {
        let code = fs::read_to_string(path)?;
        auto_ui::from_auto_lang(&code)  // Uses runtime interpretation
    }
}
```

#### 4.3 Example with Hot-Reload
**File**: `crates/auto-ui-iced-examples/src/bin/hot_reload.rs`

```rust
fn main() -> iced::Result {
    let mut watcher = UIWatcher::new()?;

    iced::run(
        |ui| ui.view_iced(),
        HotReloadApp::new("src/ui/hello.at", &mut watcher)
    )
}
```

### Phase 5: Validation & Testing (Week 4)

#### 5.1 Test Cases

**T1: Runtime Interpretation**
```rust
#[test]
fn test_hello_runtime() {
    let code = r#"
        widget Hello {
            msg str
            fn view() View {
                text(msg) {}
            }
        }
    "#;

    let component = auto_ui::from_auto_lang(code).unwrap();
    assert_eq!(component.view().to_text(), "Hello, World!");
}
```

**T2: Transpilation**
```rust
#[test]
fn test_hello_transpile() {
    let input = fs::read_to_string("test_data/hello.at")?;
    auto_ui_transpiler::transpile_file("test_data/hello.at")?;

    let output = fs::read_to_string("test_data/hello.rs")?;
    assert!(output.contains("impl Component for Hello"));
}
```

**T3: End-to-End**
```bash
# Runtime
cargo run --bin hello_runtime

# Transpiled
cargo run --bin hello_transpiled
```

#### 5.2 Migration Examples

Convert existing .at files:
- ✅ hello.at → working
- ✅ counter.at → working with messages
- 📋 login.at → todo

## Critical Files Summary

### AutoLang (to create)
1. **crates/auto-lang/src/macro/ui.rs** - Widget macro expansion
2. **stdlib/auto/widget.at** - Widget trait definition
3. **stdlib/auto/app.at** - App type definition

### AutoLang (to modify)
1. **[crates/auto-lang/src/lib.rs](../auto-lang/crates/auto-lang/src/lib.rs)** - Add macro preprocessing hook

### AutoUI (to create)
1. **crates/auto-ui/src/node_converter.rs** - Node → View<M>
2. **crates/auto-ui/src/hot_reload.rs** - File watcher + reload
3. **crates/auto-ui-transpiler/** - New crate for .at → Rust

### AutoUI (to modify)
1. **[crates/auto-ui/src/lib.rs](../auto-ui/crates/auto-ui/src/lib.rs)** - Add from_auto_lang()
2. **Cargo.toml** - Add auto-lang dependency

## Verification Strategy

### V1: Runtime Mode
```bash
cd d:/autostack/auto-ui
cargo run --bin hello_runtime
# Should open window with "Hello, World!"
```

### V2: Transpile Mode
```bash
cd d:/autostack/auto-ui
cargo run --bin hello_transpiled
# Should open window with "Hello, World!"
```

### V3: Hot-Reload
```bash
cd d:/autostack/auto-ui
cargo run --bin hot_reload
# Edit hello.at → window updates automatically
```

### V4: Integration Tests
```bash
cd d:/autostack/auto-lang
cargo test -p auto-lang eval_ui
cargo test -p auto-lang transpile_ui
```

## Risks & Mitigations

### R1: Macro Expansion Complexity
**Risk**: Text-level preprocessing might be fragile (comments, strings, edge cases)
**Mitigation**: Start with simple regex, upgrade to AST-level macros if needed

### R2: Widget Trait Integration
**Risk**: AutoLang's trait system may not support `is Widget` yet
**Mitigation**: Implement basic trait constraints first, enhance later

### R2: Type Mismatch (Node vs View)
**Risk**: AutoLang's `Node` may not map cleanly to `View<M>`
**Mitigation**: Create `UINode` adapter layer

### R3: Message Type Inference
**Risk**: Hard to infer `Msg` type at runtime
**Mitigation**: Use dynamic dispatch (`Box<dyn Any>`) or require explicit enum

### R4: Build Complexity
**Risk**: build.rs integration might slow compilation
**Mitigation**: Make transpilation optional (feature flag)

## Success Criteria

1. ✅ Runtime interpretation works for hello.at
2. ✅ Transpilation works for hello.at
3. ✅ Hot-reload example runs
4. ✅ counter.at works with messages
5. ✅ All examples in auto-ui can use .at files
6. ✅ Documentation updated

## Timeline Estimate

- **Week 1**: Phase 1 (parser extensions) + Phase 2 (Node→View)
- **Week 2**: Phase 3 (transpiler)
- **Week 3**: Phase 4 (hot-reload)
- **Week 4**: Phase 5 (validation & testing)

**Total**: 3-4 weeks

## Next Steps

1. ✅ Review and approve this plan
2. Start Phase 1.1: Add widget/app to lexer
3. Create feature branch: `feature/auto-ui-integration`
4. Implement incrementally with tests
