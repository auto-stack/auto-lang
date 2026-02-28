# Plan 097: TodoMVC Example Implementation

## Objective

Implement a complete TodoMVC example in AutoLang that can be compiled to multiple UI backends:
- **Vue.js** - Web framework
- **Iced** - Rust native GUI (Elm-inspired)
- **GPUI** - Zed's GPU-accelerated UI framework

This plan defines the language features needed and the implementation strategy for each backend.

---

## TodoMVC Specification

Based on [todomvc.com](https://todomvc.com/) specification:

### Core Features

| Feature | Description |
|---------|-------------|
| Add Todo | Input field, Enter to add new todo |
| Edit Todo | Double-click to edit, Enter to save, Escape to cancel |
| Delete Todo | Click X button to remove |
| Toggle Todo | Checkbox to mark complete/incomplete |
| Toggle All | Master checkbox to toggle all todos |
| Filter | All / Active / Completed tabs |
| Clear Completed | Button to remove all completed todos |
| Persistence | Local storage (optional for v1) |
| Routing | Hash-based routing (#/, #/active, #/completed) |

### Data Model

```auto
type Todo {
    id int
    text str
    done bool
}

type Filter {
    All
    Active
    Completed
}
```

---

## Required AURA Language Features

### 1. View List Rendering (for loops)

**Syntax:**
```auto
view {
    col {
        for todo in .todos {
            TodoItem (todo: todo, onToggle: .Toggle, onDelete: .Delete)
        }
    }
}
```

**With index:**
```auto
for i, todo in .todos {
    row { text `${i}: ${todo.text}` }
}
```

**Vue output:**
```vue
<div v-for="todo in todos" :key="todo.id">
  <TodoItem :todo="todo" @toggle="onToggle" @delete="onDelete" />
</div>
```

**Iced output:**
```rust
column(todos.iter().map(|todo| {
    TodoItem::new(todo).on_toggle(on_toggle).on_delete(on_delete)
}))
```

**GPUI output:**
```rust
v_flex()
    .children(self.todos.iter().map(|todo| {
        todo_item(todo.clone())
            .on_toggle(...)
            .on_delete(...)
    }))
```

### 2. Input Element

**Syntax:**
```auto
input (
    placeholder: "What needs to be done?",
    value: .newTodo,
    oninput: .UpdateNewTodo,
    onenter: .Add
)
```

**With autofocus:**
```auto
input (
    placeholder: "Add todo",
    value: .newTodo,
    autofocus: true,
    onenter: .Add
)
```

**Vue output:**
```vue
<input
  v-model="newTodo"
  placeholder="What needs to be done?"
  @keyup.enter="onAdd"
/>
```

**Iced output:**
```rust
text_input("What needs to be done?", &self.new_todo)
    .on_input(Message::UpdateNewTodo)
    .on_submit(Message::Add)
```

**GPUI output:**
```rust
TextInput::new(cx, "What needs to be done?", &self.new_todo)
    .on_input(|s| Message::UpdateNewTodo(s.to_string()))
    .on_action(|cx| cx.emit(Message::Add))
```

### 3. Checkbox Element

**Syntax:**
```auto
checkbox (
    checked: todo.done,
    onchange: .Toggle(todo.id)
)
```

**Vue output:**
```vue
<input type="checkbox" :checked="todo.done" @change="onToggle(todo.id)" />
```

**Iced output:**
```rust
checkbox(todo.done, "").on_toggle(move |_| Message::Toggle(todo.id))
```

**GPUI output:**
```rust
Checkbox::new(cx, todo.done)
    .on_click(|cx| cx.emit(Message::Toggle(todo.id)))
```

### 4. Conditional Rendering

**Syntax:**
```auto
if .todos.len > 0 {
    footer {
        text `${.activeCount} items left`
    }
}
```

**With else:**
```auto
if .todos.len == 0 {
    text "No todos yet!"
} else {
    text `${.todos.len} todos`
}
```

**Vue output:**
```vue
<div v-if="todos.length > 0">
  <footer>{{ activeCount }} items left</footer>
</div>
```

**Iced output:**
```rust
if !self.todos.is_empty() {
    container(text(format!("{} items left", self.active_count)))
} else {
    container(text("No todos yet!"))
}
```

### 5. Computed Properties

**Syntax:**
```auto
computed {
    activeCount => .todos.filter(|t| !t.done).len
    completedCount => .todos.filter(|t| t.done).len
    filteredTodos => match .filter {
        Filter::All => .todos
        Filter::Active => .todos.filter(|t| !t.done)
        Filter::Completed => .todos.filter(|t| t.done)
    }
}
```

**Vue output:**
```javascript
const activeCount = computed(() =>
  todos.value.filter(t => !t.done).length
)
```

**Iced output:**
```rust
fn active_count(&self) -> usize {
    self.todos.iter().filter(|t| !t.done).count()
}
```

### 6. Array Operations

**Syntax:**
```auto
on {
    Add => {
        let todo = Todo {
            id: .nextId,
            text: .newTodo,
            done: false
        }
        todos.push(todo)
        newTodo = ""
        nextId = nextId + 1
    }

    Delete(id) => {
        todos = todos.filter(|t| t.id != id)
    }

    Toggle(id) => {
        for t in todos {
            if t.id == id {
                t.done = !t.done
            }
        }
    }

    ClearCompleted => {
        todos = todos.filter(|t| !t.done)
    }
}
```

### 7. Dynamic Class Binding

**Syntax:**
```auto
row (class: { "completed": todo.done, "editing": todo.editing }) {
    // ...
}
```

**Vue output:**
```vue
<div :class="{ completed: todo.done, editing: todo.editing }">
```

### 8. Event with Parameters

**Syntax:**
```auto
button (text: "×", onclick: .Delete(todo.id))
```

**Vue output:**
```vue
<button @click="onDelete(todo.id)">×</button>
```

---

## File Structure

```
examples/
├── todomvc.at              # Main TodoMVC widget
└── todomvc/
    ├── types.at            # Todo, Filter types
    ├── todo_item.at        # TodoItem sub-component
    └── todo_footer.at      # Footer sub-component

tmp/todomvc/
├── vue/                    # Vue.js output
│   ├── package.json
│   ├── src/
│   │   ├── App.vue
│   │   ├── TodoMVC.vue     # Generated
│   │   └── main.js
│   └── src-tauri/          # Tauri desktop wrapper
│
├── iced/                   # Iced output
│   ├── Cargo.toml
│   └── src/
│       └── main.rs
│
└── gpui/                   # GPUI output
    ├── Cargo.toml
    └── src/
        └── main.rs
```

---

## Implementation Phases

### Phase 1: Parser Extensions (Week 1)

**Goal:** Extend AURA parser to support new syntax

**Tasks:**
1. [ ] Add `for` loop parsing in view blocks
   - `for item in .list { ... }`
   - `for i, item in .list { ... }`

2. [ ] Add `if/else` parsing in view blocks
   - `if condition { ... }`
   - `if condition { ... } else { ... }`

3. [ ] Add new element types to ViewNode enum
   - `ViewNode::ForLoop { var, iterable, body }`
   - `ViewNode::Conditional { condition, then_body, else_body }`

4. [ ] Add `input` and `checkbox` element parsing
   - Special handling for `value`, `checked`, `autofocus` props

5. [ ] Add event with parameters parsing
   - `onclick: .Delete(todo.id)`
   - Store as `ViewEvent { name, handler, params }`

6. [ ] Add `computed` block parsing
   - New block type in widget definition

**Files to modify:**
- `crates/auto-lang/src/parser.rs`
- `crates/auto-lang/src/ast/ui.rs`

### Phase 2: Vue.js Code Generator (Week 1-2)

**Goal:** Generate working Vue 3 SFC from AURA

**Tasks:**
1. [ ] Implement `for` loop → `v-for` transformation
2. [ ] Implement `if/else` → `v-if/v-else` transformation
3. [ ] Implement `input` element with `v-model`
4. [ ] Implement `checkbox` element
5. [ ] Implement computed properties with `computed()`
6. [ ] Implement event handlers with parameters
7. [ ] Implement dynamic class binding

**Files to modify:**
- `crates/auto-lang/src/ui_gen/vue.rs`

**Test:**
```bash
auto ui examples/todomvc.at -b vue -o tmp/todomvc/vue/src
cd tmp/todomvc/vue && npm run dev
```

### Phase 3: Iced Code Generator (Week 2-3)

**Goal:** Generate working Iced application from AURA

**Iced Architecture:**
```rust
// Generated structure
struct TodoApp {
    todos: Vec<Todo>,
    new_todo: String,
    filter: Filter,
    next_id: usize,
}

#[derive(Debug, Clone)]
enum Message {
    Add,
    Delete(usize),
    Toggle(usize),
    UpdateNewTodo(String),
    SetFilter(Filter),
    ClearCompleted,
}

impl Application for TodoApp {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) { ... }
    fn title(&self) -> String { "TodoMVC".into() }
    fn update(&mut self, message: Message) -> Command<Message> { ... }
    fn view(&self) -> Element<Message> { ... }
}
```

**Tasks:**
1. [ ] Create Iced generator module `ui_gen/iced.rs`
2. [ ] Implement widget → Iced Application trait
3. [ ] Implement model → struct fields
4. [ ] Implement on block → Message enum + update()
5. [ ] Implement view block → view() method
6. [ ] Implement for loop → iterator map
7. [ ] Implement if/else → conditional Element
8. [ ] Implement input → TextInput widget
9. [ ] Implement checkbox → Checkbox widget
10. [ ] Generate Cargo.toml with dependencies

**Files to create/modify:**
- `crates/auto-lang/src/ui_gen/iced.rs` (new)
- `crates/auto-lang/src/ui_gen/mod.rs`

**Test:**
```bash
auto ui examples/todomvc.at -b iced -o tmp/todomvc/iced
cd tmp/todomvc/iced && cargo run
```

### Phase 4: GPUI Code Generator (Week 3-4)

**Goal:** Generate working GPUI application from AURA

**GPUI Architecture:**
```rust
// Generated structure
struct TodoApp {
    todos: Vec<Model<Todo>>,
    new_todo: Model<String>,
    filter: Model<Filter>,
    next_id: Model<usize>,
}

impl TodoApp {
    fn new(cx: &mut AppContext) -> View<Self> {
        // Initialize state
    }

    fn view(cx: &mut AppContext) -> impl IntoElement {
        // Build UI
    }
}

impl Render for TodoApp {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        // Render view tree
    }
}
```

**Tasks:**
1. [ ] Create GPUI generator module `ui_gen/gpui.rs`
2. [ ] Implement widget → GPUI View struct
3. [ ] Implement model → Model<T> reactive state
4. [ ] Implement on block → event handlers
5. [ ] Implement view block → render() method
6. [ ] Implement for loop → v_flex().children()
7. [ ] Implement if/else → conditional rendering
8. [ ] Implement input → TextInput component
9. [ ] Implement checkbox → Checkbox component
10. [ ] Generate Cargo.toml with gpui dependency

**Files to create/modify:**
- `crates/auto-lang/src/ui_gen/gpui.rs` (new)
- `crates/auto-lang/src/ui_gen/mod.rs`

**Test:**
```bash
auto ui examples/todomvc.at -b gpui -o tmp/todomvc/gpui
cd tmp/todomvc/gpui && cargo run
```

### Phase 5: Sub-component Support (Week 4)

**Goal:** Support modular widget composition

**Syntax:**
```auto
widget TodoItem {
    props {
        todo Todo
    }

    msg Msg { Toggle, Delete, Edit }

    view {
        row (class: { "completed": .todo.done }) {
            checkbox (checked: .todo.done, onchange: .Toggle)
            text `${.todo.text}`
            button (text: "×", onclick: .Delete)
        }
    }

    on {
        Toggle => {
            emit ToggleEvent { id: .todo.id }
        }
    }
}

widget TodoMVC {
    // ...
    view {
        for todo in .todos {
            TodoItem (todo: todo, onToggle: .Toggle, onDelete: .Delete)
        }
    }
}
```

**Tasks:**
1. [ ] Add `props` block parsing
2. [ ] Add component instantiation in view
3. [ ] Add event forwarding/emit
4. [ ] Generate sub-components in output

### Phase 6: Testing & Polish (Week 5)

**Tasks:**
1. [ ] Create test suite for AURA parsing
2. [ ] Create test suite for each backend generator
3. [ ] Add visual regression tests
4. [ ] Optimize generated code
5. [ ] Add CSS styling support
6. [ ] Documentation

---

## Technical Details

### ViewNode AST Extensions

```rust
// In ast/ui.rs

pub enum ViewNode {
    // Existing
    Text(ViewText),
    Element {
        tag: String,
        props: Vec<ViewProp>,
        events: Vec<ViewEvent>,
        children: Vec<ViewNode>,
    },

    // New
    ForLoop {
        var: String,           // "todo"
        index: Option<String>, // Some("i") or None
        iterable: String,      // ".todos"
        body: Vec<ViewNode>,
    },
    Conditional {
        condition: String,     // ".todos.len > 0"
        then_body: Vec<ViewNode>,
        else_body: Option<Vec<ViewNode>>,
    },
    Component {
        name: String,          // "TodoItem"
        props: Vec<ViewProp>,
        events: Vec<ViewEvent>,
    },
}

pub struct ViewProp {
    pub name: String,
    pub value: ViewPropValue,
}

pub enum ViewPropValue {
    Expr(Expr),
    Binding(String),        // Direct state binding ".count"
    ClassBinding(Vec<(String, String)>), // { "completed": todo.done }
}

pub struct ViewEvent {
    pub name: String,       // "onclick"
    pub handler: String,    // ".Delete"
    pub params: Vec<Expr>,  // [todo.id]
}
```

### Computed Block AST

```rust
pub struct ComputedBlock {
    pub properties: Vec<ComputedProperty>,
}

pub struct ComputedProperty {
    pub name: String,
    pub body: Expr,  // Lambda or expression
}
```

### Generator Interface

```rust
// In ui_gen/mod.rs

pub trait BackendGenerator {
    fn generate(&self, widget: &AuraWidget) -> GenResult<String>;
    fn file_extension(&self) -> &str;
    fn requires_additional_files(&self) -> Vec<GeneratedFile>;
}

pub struct GeneratedFile {
    pub path: String,
    pub content: String,
}
```

---

## Dependencies

### Vue Backend
```json
{
  "dependencies": {
    "vue": "^3.4.0"
  },
  "devDependencies": {
    "@vitejs/plugin-vue": "^5.0.0",
    "vite": "^5.0.0",
    "@tauri-apps/cli": "^2.0.0"  // Optional for desktop
  }
}
```

### Iced Backend
```toml
[dependencies]
iced = { version = "0.13", features = ["tokio"] }
```

### GPUI Backend
```toml
[dependencies]
gpui = { git = "https://github.com/zed-industries/zed" }
```

---

## Success Criteria

1. **Parseability:** TodoMVC example parses without errors
2. **Vue Output:** Running Vue app with full TodoMVC functionality
3. **Iced Output:** Native desktop app with full TodoMVC functionality
4. **GPUI Output:** GPU-accelerated desktop app with full TodoMVC functionality
5. **Code Quality:** Generated code is readable and idiomatic
6. **Performance:** Smooth 60fps on all backends

---

## References

- [TodoMVC Specification](https://todomvc.com/)
- [Vue 3 Documentation](https://vuejs.org/)
- [Iced Documentation](https://iced.rs/)
- [GPUI Repository](https://github.com/zed-industries/zed)
- [Plan 096: AURA Architecture](./096-scenario-ui.md)

---

## Progress Tracking

| Phase | Status | Completion |
|-------|--------|------------|
| Phase 1: Parser Extensions | ✅ Complete | 100% |
| Phase 2: Vue.js Generator | ✅ Complete | 100% |
| Phase 3: Rust/AutoUI Generator | ✅ Complete | 100% |
| Phase 4: Testing & Polish | Not Started | 0% |

### Architecture Decision (2026-02-28)

**Key Insight**: The Iced and GPUI generators should NOT be separate. Instead:

```
AURA → Rust Generator → AutoUI Components (abstract)
                              ↓
                    ../auto-ui crate handles:
                    - Iced backend implementation
                    - GPUI backend implementation
                    - Other future backends
```

The `rust.rs` generator produces code using the abstract `Component` trait
and `View` builder pattern from `auto_ui::prelude::*`. The `auto-ui` crate
provides backend-specific implementations.

This means:
- Only **2 generators** needed: `vue` (JavaScript) and `rust` (AutoUI)
- Backend-specific code is in `auto-ui` crate, not `auto-lang`
- CLI supports: `-b vue` and `-b rust`

### Phase 1 Completed (2026-02-28)

- ✅ `for` loop parsing in view blocks
- ✅ `if/else` conditional parsing in view blocks
- ✅ Event handler with parameters parsing
- ✅ ViewNode AST extensions (ForLoop, Conditional, Component)
- ✅ AuraNode IR extensions
- ✅ Vue generator v-for/v-if support
- ✅ Rust generator iterator support
- ✅ AURA atom serialization

### Phase 2 Completed (2026-02-28)

- ✅ Loop variable scoping for interpolations
- ✅ Text node parsing with regular strings
- ✅ Event parameter extraction (AuraEvent with params)
- ✅ Input/checkbox element support (v-model)
- ✅ For loop variable order (idx, item → v-for="(item, idx)")

### Phase 3 Completed (2026-02-28)

- ✅ Single Rust generator for AutoUI abstraction
- ✅ Loop variable tracking and scope handling
- ✅ Format strings with {} placeholders
- ✅ Condition conversion (.count → self.count)
- ✅ Removed redundant Iced generator
- ✅ CLI supports: `-b vue`, `-b rust`

### Known Issues

None - all Phase 1 and Phase 2 issues resolved.
